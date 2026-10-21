"""A4 drawing sheets for the mechanical parts: SVG for the manual, PDF to print.

Units are millimetres on the page, origin at the top-left corner, y down. The
frame and the title block repeat the ones of the schematic
(hardware/kicad/schgen/writer.py), so every drawing of the project looks the
same.

Text is Helvetica. The PDF uses the standard font, with nothing embedded; the
SVG asks for Helvetica or its metric twins Arial and Liberation Sans, so the
widths computed here hold in both. Both files are written without dates, so
an unchanged drawing gives unchanged files.
"""

from __future__ import annotations

import ast
import math
import operator
import re
from pathlib import Path

PAGE_W, PAGE_H = 297.0, 210.0      # A4 landscape
MARGIN, STRIP = 10.0, 4.0          # frame margin, zone strip
TB_W, TB_H = 124.0, 34.0           # title block, from the frame corner
# drawing area inside the zone strip, and the title block in its corner
AREA = (MARGIN + STRIP, MARGIN + STRIP, PAGE_W - MARGIN - STRIP, PAGE_H - MARGIN - STRIP)
TITLE_BLOCK = (PAGE_W - MARGIN - TB_W, PAGE_H - MARGIN - TB_H, AREA[2], AREA[3])

FRAME = "#840000"    # KiCad drawing-sheet red
INK = "#000000"
NOTE = "#000084"     # notes, as on the schematic
VERIFY = "#c85a00"   # [TO VERIFY], as on the schematic
GREY = "#6e6e6e"
FILL = "#ececec"     # faces of the parts

THICK, MEDIUM, THIN = 0.5, 0.35, 0.18  # outlines; hidden edges; dimensions and center lines
HIDDEN = (2.0, 1.0)
CENTER = (6.0, 1.0, 0.5, 1.0)
TEXT = 3.0           # body text, em size
DIM_TEXT = 3.0       # dimension values
ARROW_L, ARROW_W = 2.6, 0.9
GAP, OVER = 1.0, 2.0  # extension lines: gap from the part, overshoot past the dimension line
CAP = 0.72           # Helvetica cap height, in em

FONT = "Helvetica, Arial, 'Liberation Sans', sans-serif"

# ---------------------------------------------------------------- text metrics

# Helvetica and Helvetica-Bold advance widths (1/1000 em), characters 32-126
_HELV = [
    278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278,
    556, 556, 556, 556, 556, 556, 556, 556, 556, 556, 278, 278, 584, 584, 584, 556,
    1015, 667, 667, 722, 722, 667, 611, 778, 722, 278, 500, 667, 556, 833, 722, 778,
    667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 278, 278, 278, 469, 556,
    333, 556, 556, 500, 556, 556, 278, 556, 556, 222, 222, 500, 222, 833, 556, 556,
    556, 556, 333, 500, 278, 556, 500, 722, 500, 500, 500, 334, 260, 334, 584,
]
_HELV_BOLD = [
    278, 333, 474, 556, 556, 889, 722, 238, 333, 333, 389, 584, 278, 333, 278, 278,
    556, 556, 556, 556, 556, 556, 556, 556, 556, 556, 333, 333, 584, 584, 584, 611,
    975, 722, 722, 722, 722, 667, 611, 778, 722, 278, 556, 722, 611, 833, 722, 778,
    667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 333, 278, 333, 584, 556,
    333, 556, 611, 556, 611, 556, 333, 611, 611, 278, 278, 556, 278, 889, 611, 611,
    611, 611, 389, 556, 333, 611, 556, 778, 556, 556, 500, 389, 280, 389, 584,
]
assert len(_HELV) == len(_HELV_BOLD) == 95
# other characters of the WinAnsi encoding: code, width, bold width
_WINANSI = {
    "Ø": (0xD8, 778, 778), "ø": (0xF8, 611, 611), "°": (0xB0, 400, 400), "±": (0xB1, 584, 584),
    "×": (0xD7, 584, 584), "·": (0xB7, 278, 278), "–": (0x96, 556, 556), "—": (0x97, 1000, 1000),
    "µ": (0xB5, 556, 611), "²": (0xB2, 333, 333), "’": (0x92, 222, 278), "‘": (0x91, 222, 278),
    "“": (0x93, 333, 500), "”": (0x94, 333, 500), "…": (0x85, 1000, 1000), "•": (0x95, 350, 350),
}
# not in WinAnsi: the PDF takes them from the Symbol font (code, width)
_SYMBOL = {
    "Ω": (0x57, 768), "→": (0xAE, 987), "←": (0xAC, 987), "↑": (0xAD, 603), "↓": (0xAF, 603),
    "≥": (0xB3, 549), "≤": (0xA3, 549), "≈": (0xBB, 549),
}


def char_width(c: str, bold: bool = False) -> float:
    """Advance width of one character, in em."""
    o = ord(c)
    if 32 <= o <= 126:
        return (_HELV_BOLD if bold else _HELV)[o - 32] / 1000
    if c in _WINANSI:
        return _WINANSI[c][2 if bold else 1] / 1000
    if c in _SYMBOL:
        return _SYMBOL[c][1] / 1000
    raise ValueError(f"character {c!r} has no Helvetica or Symbol glyph")


def text_width(s: str, size: float, bold: bool = False) -> float:
    return sum(char_width(c, bold) for c in s) * size


# ---------------------------------------------------------------- geometry


def fmt(v: float) -> str:
    """A length as written on a drawing: 83, 10.5, 12.25."""
    s = f"{v:.2f}".rstrip("0").rstrip(".")
    return "0" if s in ("-0", "") else s


def _num(v: float) -> str:
    s = f"{v:.3f}".rstrip("0").rstrip(".")
    return "0" if s in ("-0", "") else s


def unit(dx: float, dy: float) -> tuple[float, float]:
    n = math.hypot(dx, dy)
    return dx / n, dy / n


def arc_points(c, r: float, a0: float, a1: float) -> list:
    """Cubic Bezier pieces of an arc, page angles in degrees (clockwise, y down).

    Returns the start point followed by (c1, c2, end) triples.
    """
    n = max(1, math.ceil(abs(a1 - a0) / 90))
    step = math.radians(a1 - a0) / n
    k = 4 / 3 * math.tan(step / 4)
    t = math.radians(a0)
    out = [(c[0] + r * math.cos(t), c[1] + r * math.sin(t))]
    for _ in range(n):
        t2 = t + step
        p0 = (c[0] + r * math.cos(t), c[1] + r * math.sin(t))
        p3 = (c[0] + r * math.cos(t2), c[1] + r * math.sin(t2))
        c1 = (p0[0] - k * r * math.sin(t), p0[1] + k * r * math.cos(t))
        c2 = (p3[0] + k * r * math.sin(t2), p3[1] - k * r * math.cos(t2))
        out.append((c1, c2, p3))
        t = t2
    return out


class View:
    """Places a part drawn in its own millimetres (y up) on the page.

    `sx` is -1 for a view from behind, where the part's x runs leftwards.
    """

    def __init__(self, ox: float, oy: float, scale: float = 1.0, sx: int = 1):
        self.ox, self.oy, self.scale, self.sx = ox, oy, scale, sx

    def __call__(self, x: float, y: float) -> tuple[float, float]:
        return self.ox + self.sx * self.scale * x, self.oy - self.scale * y

    def len(self, v: float) -> float:
        return v * self.scale


# ---------------------------------------------------------------- drawing


class Drawing:
    """A list of paths and texts, written out as SVG and as PDF."""

    def __init__(self, title: str):
        self.title = title
        self.items: list = []

    # paths: a list of ("M", p) ("L", p) ("C", c1, c2, p) ("Z",) commands
    def path(self, cmds, color=INK, width=THIN, dash=None, fill=None, evenodd=False, stroke=True, circle=None):
        self.items.append(("path", cmds, color if stroke else None, width, dash, fill, evenodd, circle))

    def line(self, a, b, color=INK, width=THIN, dash=None):
        self.path([("M", a), ("L", b)], color, width, dash)

    def polyline(self, pts, color=INK, width=THIN, dash=None, closed=False, fill=None):
        cmds = [("M", pts[0])] + [("L", p) for p in pts[1:]]
        if closed:
            cmds.append(("Z",))
        self.path(cmds, color, width, dash, fill)

    def rect(self, x0, y0, x1, y1, color=INK, width=THIN, dash=None, fill=None, stroke=True):
        cmds = [("M", (x0, y0)), ("L", (x1, y0)), ("L", (x1, y1)), ("L", (x0, y1)), ("Z",)]
        self.path(cmds, color, width, dash, fill, stroke=stroke)

    def arc(self, c, r, a0, a1, color=INK, width=THIN, dash=None):
        pts = arc_points(c, r, a0, a1)
        self.path([("M", pts[0])] + [("C", *seg) for seg in pts[1:]], color, width, dash)

    def rounded_rect(self, x0, y0, x1, y1, r, color=INK, width=THIN, dash=None, fill=None):
        x0, x1 = min(x0, x1), max(x0, x1)
        y0, y1 = min(y0, y1), max(y0, y1)
        cmds = [("M", (x0 + r, y0)), ("L", (x1 - r, y0))]
        for c, a0 in (((x1 - r, y0 + r), -90), ((x1 - r, y1 - r), 0), ((x0 + r, y1 - r), 90), ((x0 + r, y0 + r), 180)):
            cmds += [("C", *seg) for seg in arc_points(c, r, a0, a0 + 90)[1:]]
            nxt = {-90: (x1, y1 - r), 0: (x0 + r, y1), 90: (x0, y0 + r), 180: None}[a0]
            if nxt:
                cmds.append(("L", nxt))
        cmds.append(("Z",))
        self.path(cmds, color, width, dash, fill)

    def circle(self, c, r, color=INK, width=THIN, dash=None, fill=None, stroke=True):
        pts = arc_points(c, r, 0, 360)
        cmds = [("M", pts[0])] + [("C", *seg) for seg in pts[1:]] + [("Z",)]
        self.path(cmds, color, width, dash, fill, stroke=stroke, circle=(c, r))

    def text(self, x, y, s, size=TEXT, color=INK, anchor="start", bold=False, angle=0.0, middle=False):
        """Text with its baseline at (x, y); `middle` centers the capitals on y instead."""
        if middle:  # move the baseline half a cap height against the glyphs' up direction
            x -= math.sin(math.radians(angle)) * size * CAP / 2
            y += math.cos(math.radians(angle)) * size * CAP / 2
        for c in s:
            char_width(c, bold)  # refuse glyphs the PDF cannot draw
        self.items.append(("text", x, y, s, size, color, anchor, bold, angle))

    def lines(self, x, y, rows, size=TEXT, color=INK, leading=1.45, bold=False) -> float:
        """Several lines of text from the top; returns the y just below the last one."""
        for i, row in enumerate(rows):
            self.text(x, y + size * CAP + i * size * leading, row, size, color, bold=bold)
        return y + size * CAP + (len(rows) - 1) * size * leading + size * 0.25

    # -------------------------------------------------------------- output

    def svg(self) -> str:
        out = [
            f'<svg xmlns="http://www.w3.org/2000/svg" width="{_num(PAGE_W)}mm" height="{_num(PAGE_H)}mm" '
            f'viewBox="0 0 {_num(PAGE_W)} {_num(PAGE_H)}" font-family="{FONT}">',
            f"<title>{_esc(self.title)}</title>",
            f'<rect width="{_num(PAGE_W)}" height="{_num(PAGE_H)}" fill="#ffffff"/>',
        ]
        for it in self.items:
            if it[0] == "path":
                _, cmds, color, width, dash, fill, evenodd, circle = it
                if circle:  # the PDF draws circles as four Bezier arcs, the SVG has a shorter form
                    (cx, cy), r = circle
                    attrs = [f'cx="{_num(cx)}" cy="{_num(cy)}" r="{_num(r)}"', f'fill="{fill or "none"}"']
                else:
                    attrs = [f'd="{" ".join(_svg_cmd(c) for c in cmds)}"', f'fill="{fill or "none"}"']
                if fill and evenodd:
                    attrs.append('fill-rule="evenodd"')
                if color:
                    attrs.append(f'stroke="{color}" stroke-width="{_num(width)}" '
                                 'stroke-linecap="round" stroke-linejoin="round"')
                    if dash:
                        attrs.append(f'stroke-dasharray="{" ".join(_num(v) for v in dash)}"')
                out.append(f'<{"circle" if circle else "path"} {" ".join(attrs)}/>')
            else:
                _, x, y, s, size, color, anchor, bold, angle = it
                attrs = [f'x="{_num(x)}" y="{_num(y)}" font-size="{_num(size)}" fill="{color}"']
                if anchor != "start":
                    attrs.append(f'text-anchor="{anchor}"')
                if bold:
                    attrs.append('font-weight="bold"')
                if angle:
                    attrs.append(f'transform="rotate({_num(angle)} {_num(x)} {_num(y)})"')
                out.append(f'<text {" ".join(attrs)}>{_esc(s)}</text>')
        out.append("</svg>")
        return "\n".join(out) + "\n"

    def pdf(self) -> bytes:
        k = 72 / 25.4
        ops = [f"{_num(k)} 0 0 {_num(-k)} 0 {_num(PAGE_H * k)} cm", "1 J 1 j"]
        for it in self.items:
            if it[0] == "path":
                _, cmds, color, width, dash, fill, evenodd, _ = it
                for c in cmds:
                    if c[0] == "M":
                        ops.append(f"{_num(c[1][0])} {_num(c[1][1])} m")
                    elif c[0] == "L":
                        ops.append(f"{_num(c[1][0])} {_num(c[1][1])} l")
                    elif c[0] == "C":
                        ops.append(" ".join(f"{_num(p[0])} {_num(p[1])}" for p in c[1:]) + " c")
                    else:
                        ops.append("h")
                paint = []
                if fill:
                    paint.append(f"{_rgb(fill)} rg")
                if color:
                    paint.append(f"{_rgb(color)} RG {_num(width)} w [{' '.join(_num(v) for v in dash or ())}] 0 d")
                op = ("B" if color else "f") if fill else "S"
                if fill and evenodd:
                    op += "*"
                ops.append(" ".join(paint + [op]))
            else:
                _, x, y, s, size, color, anchor, bold, angle = it
                a = math.radians(angle)
                ca, sa = math.cos(a), math.sin(a)
                w = text_width(s, size, bold) * {"start": 0, "middle": 0.5, "end": 1}[anchor]
                x0, y0 = x - w * ca, y - w * sa
                runs = []
                font = "/F2" if bold else "/F1"
                for kind, chunk in _runs(s):
                    f = "/F3" if kind == "symbol" else font
                    runs.append(f"{f} {_num(size)} Tf ({chunk}) Tj")
                ops.append(f"BT {_rgb(color)} rg {_num(ca)} {_num(sa)} {_num(sa)} {_num(-ca)} "
                           f"{_num(x0)} {_num(y0)} Tm " + " ".join(runs) + " ET")
        content = "\n".join(ops).encode("latin-1")
        objs = [
            b"<< /Type /Catalog /Pages 2 0 R >>",
            b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
            (f"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {_num(PAGE_W * k)} {_num(PAGE_H * k)}] "
             "/Resources << /Font << /F1 5 0 R /F2 6 0 R /F3 7 0 R >> >> /Contents 4 0 R >>").encode(),
            f"<< /Length {len(content)} >>\nstream\n".encode() + content + b"\nendstream",
            b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>",
            b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>",
            b"<< /Type /Font /Subtype /Type1 /BaseFont /Symbol >>",
            f"<< /Title ({_pdf_str(self.title)}) /Producer (tdm-dash hardware/drawings) >>".encode("latin-1"),
        ]
        out = bytearray(b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n")
        offsets = []
        for i, o in enumerate(objs, start=1):
            offsets.append(len(out))
            out += f"{i} 0 obj\n".encode() + o + b"\nendobj\n"
        xref = len(out)
        out += f"xref\n0 {len(objs) + 1}\n0000000000 65535 f \n".encode()
        for off in offsets:
            out += f"{off:010d} 00000 n \n".encode()
        out += (f"trailer\n<< /Size {len(objs) + 1} /Root 1 0 R /Info {len(objs)} 0 R >>\n"
                f"startxref\n{xref}\n%%EOF\n").encode()
        return bytes(out)

    def text_boxes(self):
        """(index, corners) of every text, from the descender to the ascender."""
        for i, it in enumerate(self.items):
            if it[0] != "text":
                continue
            _, x, y, s, size, _, anchor, bold, angle = it
            w = text_width(s, size, bold)
            x0 = -w * {"start": 0, "middle": 0.5, "end": 1}[anchor]
            a = math.radians(angle)
            ca, sa = math.cos(a), math.sin(a)
            corners = []
            for u, t in ((x0, 0.21 * size), (x0 + w, 0.21 * size), (x0 + w, -0.75 * size), (x0, -0.75 * size)):
                corners.append((x + u * ca - t * sa, y + u * sa + t * ca))
            yield i, corners

    def check(self) -> list[str]:
        """Texts that overlap, leave the drawing area or enter the title block."""
        frame_items = getattr(self, "frame_items", set())
        boxes = [(i, c) for i, c in self.text_boxes() if i not in frame_items]
        errors = []
        for n, (i, a) in enumerate(boxes):
            xs, ys = [p[0] for p in a], [p[1] for p in a]
            if min(xs) < AREA[0] or max(xs) > AREA[2] or min(ys) < AREA[1] or max(ys) > AREA[3]:
                errors.append(f"text {self.items[i][3]!r} leaves the drawing area")
            tb = TITLE_BLOCK
            if max(xs) > tb[0] and max(ys) > tb[1]:
                errors.append(f"text {self.items[i][3]!r} enters the title block")
            for j, b in boxes[n + 1:]:
                if _overlap(a, b, 0.1):
                    errors.append(f"texts {self.items[i][3]!r} and {self.items[j][3]!r} overlap")
        return errors

    def save(self, stem: Path) -> None:
        errors = self.check()
        if errors:
            raise SystemExit(f"{stem.name}: " + "; ".join(errors))
        stem.parent.mkdir(parents=True, exist_ok=True)
        stem.with_suffix(".svg").write_text(self.svg(), encoding="utf-8")
        stem.with_suffix(".pdf").write_bytes(self.pdf())


def _overlap(a, b, tol: float) -> bool:
    """Separating-axis test for two convex quadrilaterals, shrunk by `tol`."""
    for poly in (a, b):
        for k in range(4):
            p, q = poly[k], poly[(k + 1) % 4]
            n = (q[1] - p[1], p[0] - q[0])
            ln = math.hypot(*n) or 1.0
            pa = [(v[0] * n[0] + v[1] * n[1]) / ln for v in a]
            pb = [(v[0] * n[0] + v[1] * n[1]) / ln for v in b]
            if max(pa) - tol <= min(pb) + tol or max(pb) - tol <= min(pa) + tol:
                return False
    return True


def _svg_cmd(c) -> str:
    if c[0] == "Z":
        return "Z"
    return c[0] + " " + " ".join(f"{_num(p[0])} {_num(p[1])}" for p in c[1:])


def _esc(s: str) -> str:
    return s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


def _rgb(color: str) -> str:
    h = color.lstrip("#")
    return " ".join(_num(int(h[i:i + 2], 16) / 255) for i in (0, 2, 4))


def _pdf_str(s: str) -> str:
    """A WinAnsi PDF string body, with the escapes PDF wants."""
    out = []
    for c in s:
        code = ord(c) if 32 <= ord(c) <= 126 else _WINANSI[c][0]
        if c in "()\\":
            out.append("\\" + c)
        elif code > 126:
            out.append(f"\\{code:03o}")
        else:
            out.append(c)
    return "".join(out)


def _runs(s: str):
    """Splits a text into WinAnsi and Symbol runs, as PDF string bodies."""
    runs: list = []
    for c in s:
        kind = "symbol" if c in _SYMBOL else "text"
        body = f"\\{_SYMBOL[c][0]:03o}" if kind == "symbol" else _pdf_str(c)
        if runs and runs[-1][0] == kind:
            runs[-1][1] += body
        else:
            runs.append([kind, body])
    return [(k, b) for k, b in runs]


# ---------------------------------------------------------------- frame


def frame(d: Drawing, title: str, subtitle: str, scale: str, rev: str, date: str, source: str) -> None:
    """Frame, zone marks and title block, laid out as on the schematic."""
    start = len(d.items)
    x0, y0, x1, y1 = MARGIN, MARGIN, PAGE_W - MARGIN, PAGE_H - MARGIN
    d.rect(x0, y0, x1, y1, FRAME, 0.3)
    d.rect(x0 + STRIP, y0 + STRIP, x1 - STRIP, y1 - STRIP, FRAME, 0.15)
    # zones every 50 mm: numbers along the top and bottom, letters at the sides
    for i in range(1, 6):
        x = x0 + 50 * i
        d.line((x, y0), (x, y0 + STRIP), FRAME, 0.15)
        d.line((x, y1 - STRIP), (x, y1), FRAME, 0.15)
    for i in range(6):
        x = x0 + 25 + 50 * i
        d.text(x, y0 + STRIP / 2, str(i + 1), 1.8, FRAME, "middle", middle=True)
        d.text(x, y1 - STRIP / 2, str(i + 1), 1.8, FRAME, "middle", middle=True)
    for i in range(1, 4):
        y = y0 + 50 * i
        d.line((x0, y), (x0 + STRIP, y), FRAME, 0.15)
        d.line((x1 - STRIP, y), (x1, y), FRAME, 0.15)
    for i in range(4):
        y = y0 + 25 + 50 * i
        d.text(x0 + STRIP / 2, y, "ABCD"[i], 1.8, FRAME, "middle", middle=True)
        d.text(x1 - STRIP / 2, y, "ABCD"[i], 1.8, FRAME, "middle", middle=True)

    # title block: rows from the bottom, as in the KiCad drawing sheet
    bx0, by0, bx1, by1 = TITLE_BLOCK
    d.rect(bx0, by0, bx1, by1, FRAME, 0.3)
    r1, r2, r3 = y1 - 12, y1 - 23, y1 - 29
    for y in (r1, r2, r3):
        d.line((bx0, y), (bx1, y), FRAME, 0.15)
    cells = [bx0, x1 - 90, x1 - 68, x1 - 28]
    for x in cells[1:]:
        d.line((x, r1), (x, by1), FRAME, 0.15)
    for x, cap, val in zip(cells, ("SCALE", "REV", "DATE", "SIZE"), (scale, rev, date, "A4")):
        d.text(x + 1.5, r1 + 1.6, cap, 1.4, FRAME, middle=True)
        d.text(x + 1.5, by1 - 2.8, val, 2.5, FRAME, bold=True, middle=True)
    d.text(bx0 + 1.5, (r1 + r2) / 2, title, 3.6, FRAME, bold=True, middle=True)
    d.text(bx0 + 1.5, (r2 + r3) / 2, "tdm-dash", 2.8, FRAME, bold=True, middle=True)
    d.text(bx0 + 22, (r2 + r3) / 2, subtitle, 1.95, FRAME, middle=True)
    d.text(bx1 - 1.5, (r2 + r3) / 2, "CERN-OHL-P-2.0", 1.95, FRAME, "end", middle=True)
    d.text(bx0 + 1.5, (r3 + by0) / 2, "github.com/frascella-dev/tdm-dash · generated by "
           f"hardware/drawings/generate.py from {source}, do not edit by hand", 1.4, FRAME, middle=True)
    d.frame_items = set(range(start, len(d.items)))


# ---------------------------------------------------------------- dimensions


def arrow(d: Drawing, tip, direction, color=INK) -> None:
    """Filled arrowhead with its point at `tip`, pointing along `direction`."""
    ux, uy = unit(*direction)
    bx, by = tip[0] - ux * ARROW_L, tip[1] - uy * ARROW_L
    nx, ny = -uy * ARROW_W / 2, ux * ARROW_W / 2
    d.polyline([tip, (bx + nx, by + ny), (bx - nx, by - ny)], color, 0.05, closed=True, fill=color)


def _upright(angle: float) -> float:
    """Text angle that reads from the bottom or from the right, as drawings do."""
    while angle <= -90:
        angle += 180
    while angle > 90:
        angle -= 180
    return angle


def dim(d: Drawing, a, b, direction, through, text: str, color=INK, text_at: float = 0.5) -> None:
    """Linear dimension between the points `a` and `b` of a part.

    The distance is measured along `direction`; the dimension line runs
    through the point `through`. Extension lines start 1 mm away from the
    part and end 2 mm past the dimension line. When the value does not fit
    between the arrows, the arrows go outside. `text_at` slides the value
    along the line (0.5 is the middle).
    """
    u = unit(*direction)
    n = (-u[1], u[0])
    feet = []
    for p in (a, b):
        off = (through[0] - p[0]) * n[0] + (through[1] - p[1]) * n[1]
        foot = (p[0] + n[0] * off, p[1] + n[1] * off)
        if abs(off) > GAP:
            s = 1 if off > 0 else -1
            d.line((p[0] + n[0] * s * GAP, p[1] + n[1] * s * GAP),
                   (foot[0] + n[0] * s * OVER, foot[1] + n[1] * s * OVER), color)
        feet.append(foot)
    fa, fb = feet
    length = math.dist(fa, fb)
    angle = _upright(math.degrees(math.atan2(u[1], u[0])))
    tw = text_width(text, DIM_TEXT)
    inside = length >= 2 * ARROW_L + 2.0
    if inside:
        d.line(fa, fb, color)
        arrow(d, fa, (fa[0] - fb[0], fa[1] - fb[1]), color)
        arrow(d, fb, (fb[0] - fa[0], fb[1] - fa[1]), color)
    else:
        e = unit(fb[0] - fa[0], fb[1] - fa[1])
        d.line((fa[0] - e[0] * 5, fa[1] - e[1] * 5), (fb[0] + e[0] * 5, fb[1] + e[1] * 5), color)
        arrow(d, fa, e, color)
        arrow(d, fb, (-e[0], -e[1]), color)
    mx = fa[0] + (fb[0] - fa[0]) * text_at
    my = fa[1] + (fb[1] - fa[1]) * text_at
    if tw + 0.8 > length:
        # small value: beside the outer arrow, on the far side
        e = unit(fb[0] - fa[0], fb[1] - fa[1])
        mx, my = fb[0] + e[0] * (6 + tw / 2), fb[1] + e[1] * (6 + tw / 2)
    up = (math.sin(math.radians(angle)), -math.cos(math.radians(angle)))
    d.text(mx + up[0] * 0.9, my + up[1] * 0.9, text, DIM_TEXT, color, "middle", angle=angle)


def hdim(d, a, b, y, text, color=INK, **kw):
    """Horizontal dimension, its line at page height `y`."""
    dim(d, a, b, (1, 0), (a[0], y), text, color, **kw)


def vdim(d, a, b, x, text, color=INK, **kw):
    """Vertical dimension, its line at page abscissa `x`; the value reads from the right."""
    dim(d, a, b, (0, -1), (x, a[1]), text, color, **kw)


def leader(d: Drawing, tip, knee, text: str, color=INK, size=DIM_TEXT, tip_arrow=True) -> None:
    """Arrow from `tip` to `knee`, then a short shoulder under the text."""
    side = 1 if knee[0] >= tip[0] else -1
    tw = text_width(text, size)
    end = (knee[0] + side * (tw + 1.0), knee[1])
    d.polyline([tip, knee, end], color)
    if tip_arrow:
        arrow(d, tip, (tip[0] - knee[0], tip[1] - knee[1]), color)
    d.text(knee[0] + side * 0.5, knee[1] - 0.9, text, size, color, "start" if side > 0 else "end")


def diameter(d: Drawing, c, r: float, angle: float, text: str, color=INK, reach: float = 8.0) -> None:
    """Diameter or radius callout: arrow on the edge at `angle` (page degrees), text outside."""
    a = math.radians(angle)
    tip = (c[0] + r * math.cos(a), c[1] + r * math.sin(a))
    knee = (c[0] + (r + reach) * math.cos(a), c[1] + (r + reach) * math.sin(a))
    leader(d, tip, knee, text, color)


def angle_dim(d: Drawing, c, r: float, a0: float, a1: float, text: str, color=INK) -> None:
    """Angle between two directions from `c`, page degrees, a0 < a1."""
    d.arc(c, r, a0, a1, color)
    for a, s in ((a0, -1), (a1, 1)):
        t = math.radians(a)
        tip = (c[0] + r * math.cos(t), c[1] + r * math.sin(t))
        arrow(d, tip, (-s * math.sin(t), s * math.cos(t)), color)
    m = math.radians((a0 + a1) / 2)
    tw = text_width(text, DIM_TEXT)
    rr = r + 1.5 + max(tw / 2 * abs(math.cos(m)), DIM_TEXT * CAP / 2 * abs(math.sin(m))) + 1
    d.text(c[0] + rr * math.cos(m), c[1] + rr * math.sin(m), text, DIM_TEXT, color, "middle", middle=True)


def center_mark(d: Drawing, c, r: float, ext: float = 2.5, angle: float = 0.0) -> None:
    """Center lines of a hole, reaching `ext` past its edge."""
    for a in (angle, angle + 90):
        t = math.radians(a)
        dx, dy = (r + ext) * math.cos(t), (r + ext) * math.sin(t)
        d.line((c[0] - dx, c[1] - dy), (c[0] + dx, c[1] + dy), GREY, THIN, CENTER)


def scale_bar(d: Drawing, x: float, y: float, scale: float) -> None:
    """A 50 mm bar to check the print: it must measure 50 mm on paper.

    Labelled in the units of the part at the given drawing scale.
    """
    h = 2.0
    for i in range(5):
        d.rect(x + 10 * i, y, x + 10 * (i + 1), y + h, INK, THIN, fill=INK if i % 2 == 0 else "#ffffff")
    for i in range(6):
        d.text(x + 10 * i, y + h + 3.4, fmt(10 * i / scale), 2.2, INK, "middle")
    d.text(x + 51 + text_width(fmt(50 / scale), 2.2) / 2, y + h + 3.4, "mm", 2.2)
    d.text(x, y - 1.6, "Print at 100 % (actual size): this bar measures 50 mm", 2.2, NOTE)


def notes(d: Drawing, x: float, y: float, items: list, width: float, size=2.6, title="NOTES") -> float:
    """Numbered notes, wrapped to `width`; each item is (text, color). Returns the bottom y."""
    d.text(x, y + size * CAP, title, size, INK, bold=True)
    y += size * 2.0
    lead = size * 1.4
    for i, (text, color) in enumerate(items, start=1):
        num = f"{i}."
        d.text(x, y + size * CAP, num, size, color)
        for row in wrap(text, width - 5, size):
            d.text(x + 5, y + size * CAP, row, size, color)
            y += lead
        y += size * 0.5
    return y


def wrap(text: str, width: float, size: float) -> list[str]:
    rows: list[str] = []
    for para in text.split("\n"):
        row = ""
        for word in para.split(" "):
            trial = f"{row} {word}".strip()
            if row and text_width(trial, size) > width:
                rows.append(row)
                row = word
            else:
                row = trial
        rows.append(row)
    return rows


# ---------------------------------------------------------------- OpenSCAD parameters

_OPS = {ast.Add: operator.add, ast.Sub: operator.sub, ast.Mult: operator.mul, ast.Div: operator.truediv}


class Param(float):
    """A number read from a .scad file; `verify` is set when it, or a value it
    is computed from, is marked [TO VERIFY]."""

    verify: bool

    def __new__(cls, value: float, verify: bool):
        obj = super().__new__(cls, value)
        obj.verify = verify
        return obj


def scad_params(path: Path) -> dict:
    """The top-level assignments of an OpenSCAD file.

    Numbers, booleans, strings, lists and arithmetic on earlier values are
    evaluated; anything else (function calls, list comprehensions) is
    skipped. A value marked [TO VERIFY] in its comment, or computed from such
    a value, comes back with `verify` set, so drawings can print it in
    orange.
    """
    env: dict = {}
    pat = re.compile(r"^([A-Za-z_]\w*)\s*=\s*([^;]*);\s*(?://(.*))?$")
    # the one comprehension the models use: a grid of points
    grid = re.compile(r"^\[\s*for\s*\(\s*(\w+)\s*=\s*(\[[^\]]*\])\s*,\s*(\w+)\s*=\s*(\[[^\]]*\])\s*\)\s*"
                      r"\[\s*(\w+)\s*,\s*(\w+)\s*\]\s*\]$")
    for line in path.read_text(encoding="utf-8").splitlines():
        m = pat.match(line)
        if not m:
            continue
        name, expr, comment = m.group(1), m.group(2), m.group(3) or ""
        g = grid.match(expr.strip())
        if g and (g.group(5), g.group(6)) == (g.group(1), g.group(3)):
            expr = "[" + ", ".join(f"[{a}, {b}]" for a in ast.literal_eval(g.group(2))
                                   for b in ast.literal_eval(g.group(4))) + "]"
        try:
            value, verify = _eval(ast.parse(expr.replace("true", "True").replace("false", "False"),
                                            mode="eval").body, env)
        except (ValueError, SyntaxError, KeyError):
            continue
        verify = verify or "[TO VERIFY]" in comment
        env[name] = _mark(value, verify)
    return env


def _mark(value, verify: bool):
    if isinstance(value, bool) or isinstance(value, str):
        return value
    if isinstance(value, (int, float)):
        return Param(value, verify)
    return [_mark(v, verify) for v in value]


def _eval(node, env) -> tuple:
    if isinstance(node, ast.Constant) and isinstance(node.value, (int, float, str, bool)):
        return node.value, False
    if isinstance(node, ast.Name):
        v = env[node.id]
        return v, bool(getattr(v, "verify", False)) or _any_verify(v)
    if isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.USub):
        v, f = _eval(node.operand, env)
        return -v, f
    if isinstance(node, ast.BinOp) and type(node.op) in _OPS:
        a, fa = _eval(node.left, env)
        b, fb = _eval(node.right, env)
        return _OPS[type(node.op)](float(a), float(b)), fa or fb
    if isinstance(node, ast.List):
        vals = [_eval(e, env) for e in node.elts]
        return [v for v, _ in vals], any(f for _, f in vals)
    raise ValueError(f"cannot evaluate {ast.dump(node)}")


def _any_verify(v) -> bool:
    if isinstance(v, list):
        return any(_any_verify(x) for x in v)
    return bool(getattr(v, "verify", False))


def svg_outline(path: Path) -> list[list[tuple[float, float]]]:
    """The closed polygons of an OpenSCAD 2D SVG export, in part coordinates (y up)."""
    data = path.read_text(encoding="utf-8")
    polys = []
    for d in re.findall(r'<path d="([^"]*)"', data):
        for sub in re.split(r"(?=M)", d):
            pts = [(float(x), -float(y)) for x, y in re.findall(r"(-?[\d.e-]+),(-?[\d.e-]+)", sub)]
            if pts:
                polys.append(pts)
    return polys
