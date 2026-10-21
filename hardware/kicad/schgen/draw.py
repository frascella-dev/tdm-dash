"""The drawing model: parts placed on a sheet, the wires between their pins,
labels, ports and notes. Coordinates are mils on the sheet (y down).

Each part carries the declared net of every pin, which is the electrical
specification; the drawing must connect exactly those pins (schgen.check).
"""

from __future__ import annotations

from dataclasses import dataclass, field

from .geom import DOWN, GRID, LEFT, RIGHT, UP, Pt, add, lib_dir, lib_to_sheet
from .library import Library

# nets drawn with power symbols, and the library symbol for each
POWER = {
    "GND": "power:GND",
    "+3V3": "power:+3V3",
    "+5V": "power:+5V",
    "+12V_P": "power:+12V",
    "VSYS_PICO": "power:+5V",
}

ORANGE = (200, 90, 0)  # [TO VERIFY] marks
NOTE_BLUE = (0, 0, 132)


@dataclass
class PinRef:
    part: Part
    number: str
    name: str
    type: str
    pos: Pt
    out: Pt  # direction away from the body
    net: str | None

    def __repr__(self) -> str:
        return f"{self.part.ref}.{self.number}"


@dataclass
class Part:
    ref: str
    lib_id: str
    value: str
    x: int
    y: int
    rot: int
    mirror: str | None
    nets: dict[str, str | None]
    pins: dict[str, PinRef] = field(default_factory=dict)
    # field placement: {"Reference": (dx, dy, justify) relative to the part}
    fields: dict[str, tuple[int, int, str]] = field(default_factory=dict)
    hide_value: bool = False
    extra: dict[str, str] = field(default_factory=dict)

    def __getitem__(self, key) -> PinRef:
        key = str(key)
        if key in self.pins:
            return self.pins[key]
        found = [p for p in self.pins.values() if p.name == key]
        # stacked pins (the Pico's GND) share one point: any of them will do
        if not found or len({p.pos for p in found}) != 1:
            raise KeyError(f"{self.ref}: no single pin {key!r}")
        return sorted(found, key=lambda p: int(p.number) if p.number.isdigit() else 0)[0]


@dataclass
class Label:
    kind: str  # "local", "hier"
    net: str
    pos: Pt
    out: Pt  # direction the text extends to
    shape: str = "passive"


@dataclass
class Power:
    net: str
    pos: Pt
    out: Pt  # DOWN for ground, UP for supplies


@dataclass
class Text:
    x: int
    y: int
    text: str
    size: int = 50
    bold: bool = False
    color: tuple[int, int, int] | None = None
    justify: str = "left top"
    angle: int = 0


@dataclass
class Rect:
    x1: int
    y1: int
    x2: int
    y2: int
    dashed: bool = True


class Sheet:
    """One schematic sheet.

    Helpers take pins (`part["K"]`) or points; points are (x, y) in mils.
    """

    def __init__(self, lib: Library, num: int, name: str, title: str, file: str):
        self.lib = lib
        self.num = num
        self.name = name
        self.title = title
        self.file = file
        self.parts: list[Part] = []
        self.wires: list[tuple[Pt, Pt]] = []
        self.labels: list[Label] = []
        self.powers: list[Power] = []
        self.flags: list[tuple[Pt, Pt]] = []  # PWR_FLAG: (pos, direction)
        self.ncs: list[Pt] = []
        self.texts: list[Text] = []
        self.rects: list[Rect] = []

    # parts --------------------------------------------------------------

    def part(self, ref, lib_id, value, at: Pt, rot=0, mirror=None, anchor=None, nets=None, **kw) -> Part:
        """Places a part. With `anchor` (a pin), `at` is where that pin lands.

        Nets are given by pin number or name: `nets={"1": "GND"}` or as
        keywords (`p1="GND"`, `K="TACH_1"`).
        """
        declared: dict[str, str | None] = dict(nets or {})
        for k, v in kw.items():
            declared[k[1:] if k[0] == "p" and k[1:].isdigit() else k] = v
        p = Part(ref, lib_id, value, at[0], at[1], rot, mirror, {})
        lib_pins = self.lib.pins(lib_id)
        if anchor is not None:
            lp = next((q for q in lib_pins if q.number == str(anchor) or q.name == str(anchor)), None)
            if lp is None:
                raise KeyError(f"{ref}: no pin {anchor}")
            dx, dy = lib_to_sheet(lp.x, lp.y, rot, mirror)
            p.x, p.y = at[0] - dx, at[1] - dy
        if p.x % GRID or p.y % GRID:
            raise ValueError(f"{ref} at {p.x},{p.y} is off the {GRID} mil grid")
        used = set()
        for lp in lib_pins:
            if lp.type == "no_connect":
                continue
            key = next((k for k in (lp.number, lp.name) if k in declared), None)
            if key is None:
                raise ValueError(f"{ref}: pin {lp.number} ({lp.name}) has no net")
            used.add(key)
            dx, dy = lib_to_sheet(lp.x, lp.y, rot, mirror)
            out = lib_dir(lp.angle + 180, rot, mirror)
            if lp.number in p.pins:  # stacked pins (the Pico's GND) share one point
                continue
            p.pins[lp.number] = PinRef(p, lp.number, lp.name, lp.type, (p.x + dx, p.y + dy), out, declared[key])
        extra = set(declared) - used
        if extra:
            raise ValueError(f"{ref}: unknown pins {sorted(extra)}")
        p.nets = {num: pin.net for num, pin in p.pins.items()}
        self.parts.append(p)
        return p

    # wires --------------------------------------------------------------

    @staticmethod
    def pt(x) -> Pt:
        return x.pos if isinstance(x, PinRef) else (int(x[0]), int(x[1]))

    def wire(self, *pts):
        """A polyline through pins and points; every step must be straight."""
        ps = [self.pt(x) for x in pts]
        for a, b in zip(ps, ps[1:]):
            if a == b:
                continue
            if a[0] != b[0] and a[1] != b[1]:
                raise ValueError(f"{self.name}: diagonal wire {a} -> {b}")
            self.wires.append((a, b))

    def route(self, a, b, first: str = "h"):
        """Wire from a to b with at most one corner (horizontal first by default)."""
        pa, pb = self.pt(a), self.pt(b)
        if pa[0] == pb[0] or pa[1] == pb[1]:
            self.wire(pa, pb)
        elif first == "h":
            self.wire(pa, (pb[0], pa[1]), pb)
        else:
            self.wire(pa, (pa[0], pb[1]), pb)

    def chain(self, *pins):
        """Straight wires through pins that sit on one line."""
        ps = [self.pt(x) for x in pins]
        if not (len({p[0] for p in ps}) == 1 or len({p[1] for p in ps}) == 1):
            raise ValueError(f"{self.name}: chain {pins} is not straight")
        self.wire(*ps)

    def _stub(self, pin_or_pt, d: Pt, length: int) -> Pt:
        start = self.pt(pin_or_pt)
        if length:
            end = add(start, d, length)
            self.wire(start, end)
            return end
        return start

    def gnd(self, pin, length: int = 100, out: Pt = DOWN) -> Pt:
        """Ground symbol below a pin (or a point), after a wire of `length`.

        `out` turns the symbol, for the ground pin of a connector whose other
        pins are below it.
        """
        end = self._stub(pin, out, length)
        self.powers.append(Power("GND", end, out))
        return end

    def rail(self, net: str, pin, length: int = 100) -> Pt:
        """Supply symbol above a pin (or a point)."""
        if net not in POWER:
            raise ValueError(f"{net} is not a power net")
        end = self._stub(pin, UP, length)
        self.powers.append(Power(net, end, UP))
        return end

    def flag(self, at: Pt, direction: Pt = UP):
        """PWR_FLAG on a wire end or pin: the net is driven (for ERC)."""
        self.flags.append((self.pt(at), direction))

    def port(self, net: str, pin_or_pt, x: int, shape: str = "passive") -> Pt:
        """Hierarchical label at column x, with a horizontal wire to it."""
        start = self.pt(pin_or_pt)
        end = (x, start[1])
        self.wire(start, end)
        self.labels.append(Label("hier", net, end, RIGHT if x > start[0] else LEFT, shape))
        return end

    def label(self, net: str, at, out: Pt = UP):
        """Local label at a wire end; the text extends towards `out`."""
        self.labels.append(Label("local", net, self.pt(at), out))

    def nc(self, pin):
        self.ncs.append(self.pt(pin))

    # notes ----------------------------------------------------------------

    def text(self, x, y, text, size=50, bold=False, color=None, justify="left top", angle=0):
        self.texts.append(Text(x, y, text, size, bold, color, justify, angle))

    def heading(self, title: str, lines: list[str], x: int = 650, y: int = 650, width: int = 0):
        """Title and short description at the top left of the sheet."""
        self.text(x, y, title, size=100, bold=True)
        if lines:
            self.text(x, y + 170, "\n".join(lines), size=60, color=NOTE_BLUE)

    def note(self, x, y, text, size=55):
        self.text(x, y, text, size=size, color=NOTE_BLUE)

    def verify(self, x, y, text, size=50):
        """A [TO VERIFY] note, in orange."""
        self.text(x, y, text, size=size, color=ORANGE)

    def rect(self, x1, y1, x2, y2, dashed=True):
        self.rects.append(Rect(x1, y1, x2, y2, dashed))


@dataclass
class SheetBlock:
    """A sheet symbol on the root sheet."""

    sheet: Sheet
    x: int
    y: int
    w: int
    h: int
    desc: str = ""
    # pin name -> (position, side) with side "left", "right", "top", "bottom"
    pins: dict[str, tuple[Pt, str]] = field(default_factory=dict)

    def pin(self, name: str, side: str, offset: int) -> Pt:
        """A sheet pin `offset` mils from the top (left/right) or left (top/bottom)."""
        pos = {
            "left": (self.x, self.y + offset),
            "right": (self.x + self.w, self.y + offset),
            "top": (self.x + offset, self.y),
            "bottom": (self.x + offset, self.y + self.h),
        }[side]
        self.pins[name] = (pos, side)
        return pos


class Root:
    """The root sheet: one block per sub-sheet and the wires between them."""

    def __init__(self, title: str):
        self.title = title
        self.blocks: list[SheetBlock] = []
        self.wires: list[tuple[Pt, Pt]] = []
        self.labels: list[Label] = []
        self.texts: list[Text] = []
        self.rects: list[Rect] = []

    def block(self, sheet: Sheet, x, y, w, h, desc="") -> SheetBlock:
        b = SheetBlock(sheet, x, y, w, h, desc)
        self.blocks.append(b)
        return b

    wire = Sheet.wire
    route = Sheet.route
    text = Sheet.text
    note = Sheet.note
    verify = Sheet.verify
    rect = Sheet.rect
    label = Sheet.label
    name = "root"

    @staticmethod
    def pt(x) -> Pt:
        return (int(x[0]), int(x[1]))
