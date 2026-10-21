"""Normalises the drawing the way KiCad 9 will read it, refuses what KiCad
would connect differently from what is meant, and proves that the drawn
connections are exactly the declared netlist.

KiCad's rules (eeschema 9.0): a wire connects only through its two ends; a
wire end in the middle of another wire connects only if that wire is split
there (with a junction dot); labels and power symbols connect at their
anchor point.
"""

from __future__ import annotations

from collections import defaultdict

from .draw import POWER, Root, Sheet
from .geom import GRID, Box, Pt, lib_to_sheet, on_segment


class DrawingError(Exception):
    pass


Seg = tuple[Pt, Pt]


def _norm(s: Seg) -> Seg:
    return (s[0], s[1]) if s[0] <= s[1] else (s[1], s[0])


def normalize(name: str, wires: list[Seg], anchors: set[Pt]) -> tuple[list[Seg], list[Pt]]:
    """Splits wires at T points, merges collinear runs, finds junction dots.

    `anchors` are the points where something else connects (pins, labels,
    power symbols, sheet pins). Returns the wires and the junctions.
    """
    errors = []
    segs = sorted({_norm(s) for s in wires if s[0] != s[1]})
    for a, b in segs:
        if a[0] != b[0] and a[1] != b[1]:
            errors.append(f"diagonal wire {a}-{b}")
        for p in (a, b):
            if p[0] % GRID or p[1] % GRID:
                errors.append(f"wire end {p} off the {GRID} mil grid")
    # collinear overlaps would be merged by KiCad behind our back
    for i, (a, b) in enumerate(segs):
        for c, d in segs[i + 1 :]:
            if a[1] == b[1] == c[1] == d[1] and min(b[0], d[0]) > max(a[0], c[0]):
                errors.append(f"overlapping wires {a}-{b} and {c}-{d}")
            if a[0] == b[0] == c[0] == d[0] and min(b[1], d[1]) > max(a[1], c[1]):
                errors.append(f"overlapping wires {a}-{b} and {c}-{d}")
    if errors:
        raise DrawingError(f"{name}: " + "; ".join(sorted(set(errors))))
    # split at wire ends that land in the middle of another wire (T points)
    ends = {p for s in segs for p in s}
    out: list[Seg] = []
    for a, b in segs:
        cuts = sorted(p for p in ends if on_segment(p, a, b))
        pts = [a, *cuts, b]
        out.extend(_norm((p, q)) for p, q in zip(pts, pts[1:]))
    for p in sorted(anchors):
        for a, b in out:
            if on_segment(p, a, b):
                errors.append(f"pin or label at {p} lies on the middle of wire {a}-{b}")
    if errors:
        raise DrawingError(f"{name}: " + "; ".join(sorted(set(errors))))
    # merge collinear pieces meeting at a point where nothing else connects
    changed = True
    while changed:
        changed = False
        deg: dict[Pt, list[Seg]] = defaultdict(list)
        for s in out:
            deg[s[0]].append(s)
            deg[s[1]].append(s)
        for p, ss in sorted(deg.items()):
            if len(ss) == 2 and p not in anchors:
                s1, s2 = ss
                horiz = s1[0][1] == s1[1][1] == s2[0][1] == s2[1][1]
                vert = s1[0][0] == s1[1][0] == s2[0][0] == s2[1][0]
                if horiz or vert:
                    q1 = s1[0] if s1[1] == p else s1[1]
                    q2 = s2[0] if s2[1] == p else s2[1]
                    out.remove(s1)
                    out.remove(s2)
                    out.append(_norm((q1, q2)))
                    changed = True
                    break
    out = sorted(set(out))
    # junction dots where three or more things meet
    count: dict[Pt, int] = defaultdict(int)
    for a, b in out:
        count[a] += 1
        count[b] += 1
    for p in anchors:
        if p in count:
            count[p] += 1
    junctions = sorted(p for p, n in count.items() if n >= 3)
    four = [p for p, n in count.items() if n >= 4]
    if four:
        raise DrawingError(f"{name}: four-way connections at {sorted(four)}")
    return out, junctions


class UnionFind:
    def __init__(self):
        self.parent: dict = {}

    def find(self, x):
        self.parent.setdefault(x, x)
        while self.parent[x] != x:
            self.parent[x] = self.parent[self.parent[x]]
            x = self.parent[x]
        return x

    def union(self, a, b):
        ra, rb = self.find(a), self.find(b)
        if ra != rb:
            self.parent[max(ra, rb)] = min(ra, rb)


def part_body(sheet: Sheet, part) -> Box | None:
    body = sheet.lib.body(part.lib_id)
    if body is None:
        return None
    corners = [lib_to_sheet(x, y, part.rot, part.mirror) for x in (body.x1, body.x2) for y in (body.y1, body.y2)]
    return Box.around([(part.x + dx, part.y + dy) for dx, dy in corners])


def check(sheets: list[Sheet], root: Root, declared: dict[str, set[str]]) -> dict:
    """Validates every sheet and the root; returns the normalised drawing.

    `declared` maps each net to its pins ("R101.1"). The result holds, per
    sheet file, the wires and junctions to write.
    """
    uf = UnionFind()
    result = {}
    names: dict = defaultdict(set)  # union-find root -> net names seen
    errors: list[str] = []

    for sh in sheets:
        key = sh.file
        anchors: set[Pt] = set()
        pin_at: dict[Pt, list] = defaultdict(list)
        for p in sh.parts:
            for pin in p.pins.values():
                pin_at[pin.pos].append(pin)
                anchors.add(pin.pos)
        for pw in sh.powers:
            anchors.add(pw.pos)
        for pos, _ in sh.flags:
            anchors.add(pos)
        # labels are soft anchors: a label may sit on the middle of a wire
        ends_of_wires = {p for w in sh.wires for p in w}
        for lb in sh.labels:
            if lb.pos in ends_of_wires:
                anchors.add(lb.pos)
        wires, junctions = normalize(sh.name, sh.wires, anchors)
        result[key] = (wires, junctions)
        wire_ends: dict[Pt, int] = defaultdict(int)
        for a, b in wires:
            wire_ends[a] += 1
            wire_ends[b] += 1
            uf.union((key, a), (key, b))
        # pins
        for pos, pins in pin_at.items():
            for pin in pins:
                uf.union((key, pos), ("pin", pin.part.ref, pin.number))
        for pw in sh.powers:
            uf.union((key, pw.pos), ("power", pw.net))
            if pw.pos not in wire_ends and pw.pos not in pin_at:
                errors.append(f"{sh.name}: power symbol {pw.net} at {pw.pos} touches nothing")
        for pos, _ in sh.flags:
            if pos not in wire_ends and pos not in pin_at:
                errors.append(f"{sh.name}: power flag at {pos} touches nothing")
        for lb in sh.labels:
            if lb.kind == "local":
                uf.union((key, lb.pos), ("local", key, lb.net))
            else:
                uf.union((key, lb.pos), ("hier", key, lb.net))
            # a label either ends one wire, or sits on the middle of one wire
            ends = wire_ends.get(lb.pos, 0) + len(pin_at.get(lb.pos, []))
            inner = [(a, b) for a, b in wires if on_segment(lb.pos, a, b)]
            if ends == 1 and not inner:
                continue
            if ends == 0 and len(inner) == 1:
                uf.union((key, lb.pos), (key, inner[0][0]))
                continue
            errors.append(f"{sh.name}: label {lb.net} at {lb.pos} must end or sit on exactly one wire")
        # dangling wire ends
        for p, n in wire_ends.items():
            if n == 1 and p not in anchors:
                errors.append(f"{sh.name}: wire end at {p} is not connected")
        # no-connect pins must stay alone; declared nets must be drawn
        for p in sh.parts:
            for pin in p.pins.values():
                if pin.net is None and pin.pos in wire_ends:
                    errors.append(f"{sh.name}: {pin} is declared unconnected but has a wire")
        # wires through symbol bodies
        for p in sh.parts:
            box = part_body(sh, p)
            if box is None:
                continue
            for a, b in wires:
                # a wire leaving one of the part's pins outwards does not cross
                # it, even where the drawing (LED arrows) reaches over the pin
                leaving = False
                for pin in p.pins.values():
                    for end, other in ((a, b), (b, a)):
                        if end == pin.pos:
                            d = (other[0] - end[0], other[1] - end[1])
                            unit = ((d[0] > 0) - (d[0] < 0), (d[1] > 0) - (d[1] < 0))
                            leaving = leaving or unit == pin.out
                if not leaving and box.cut_by(a, b):
                    errors.append(f"{sh.name}: wire {a}-{b} crosses the body of {p.ref}")

    # root sheet
    root_anchors: set[Pt] = set()
    blocks = {b.sheet.file: b for b in root.blocks}
    for b in root.blocks:
        for pname, (pos, _side) in b.pins.items():
            root_anchors.add(pos)
            uf.union(("root", pos), ("hier", b.sheet.file, pname))
    for lb in root.labels:
        root_anchors.add(lb.pos)
        uf.union(("root", lb.pos), ("local", "root", lb.net))
    rwires, rjunctions = normalize("root", root.wires, root_anchors)
    result["root"] = (rwires, rjunctions)
    rends: dict[Pt, int] = defaultdict(int)
    for a, b in rwires:
        rends[a] += 1
        rends[b] += 1
        uf.union(("root", a), ("root", b))
    for p, n in rends.items():
        if n == 1 and p not in root_anchors:
            errors.append(f"root: wire end at {p} is not connected")
    for b in root.blocks:
        for pname, (pos, _side) in b.pins.items():
            if pos not in rends:
                errors.append(f"root: sheet pin {b.sheet.name}/{pname} has no wire")
    # hierarchical labels and sheet pins must match one to one
    for sh in sheets:
        hl = {lb.net for lb in sh.labels if lb.kind == "hier"}
        pins = set(blocks[sh.file].pins) if sh.file in blocks else set()
        for n in sorted(hl - pins):
            errors.append(f"{sh.name}: hierarchical label {n} has no sheet pin")
        for n in sorted(pins - hl):
            errors.append(f"{sh.name}: sheet pin {n} has no hierarchical label")

    # names carried by each net
    for sh in sheets:
        for lb in sh.labels:
            names[uf.find((sh.file, lb.pos))].add(("label", lb.net))
        for pw in sh.powers:
            names[uf.find((sh.file, pw.pos))].add(("power", pw.net))
    for lb in root.labels:
        names[uf.find(("root", lb.pos))].add(("label", lb.net))
    for r, ns in names.items():
        if len({n for _, n in ns}) > 1:
            errors.append(f"one net carries several names: {sorted(n for _, n in ns)}")
        kinds = {k for k, _ in ns}
        if "power" in kinds and "label" in kinds:
            errors.append(f"power symbol and label on one net: {sorted(n for _, n in ns)}")
    for sh in sheets:
        for pw in sh.powers:
            if pw.net not in POWER:
                errors.append(f"{sh.name}: {pw.net} is not a power net")

    # the drawn nets are exactly the declared ones
    groups: dict = defaultdict(set)
    for sh in sheets:
        for p in sh.parts:
            for pin in p.pins.values():
                if pin.net is not None:
                    groups[uf.find(("pin", p.ref, pin.number))].add(f"{p.ref}.{pin.number}")
    drawn = {frozenset(g) for g in groups.values()}
    wanted = {frozenset(v) for v in declared.values()}
    for net, pins in sorted(declared.items()):
        if frozenset(pins) not in drawn:
            found = [sorted(g) for g in drawn if g & pins]
            errors.append(f"net {net}: declared {sorted(pins)}, drawn as {found}")
    for g in sorted(drawn - wanted, key=sorted):
        if not any(g == w for w in wanted):
            errors.append(f"drawn net {sorted(g)} matches no declared net")
    if errors:
        raise DrawingError("\n".join(sorted(set(errors))))
    return result
