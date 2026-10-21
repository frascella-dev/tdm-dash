"""Integer geometry for the generator.

Everything is in mils (1 mil = 0.0254 mm), so coordinates are exact. KiCad's
schematic grid is 50 mil (1.27 mm): every connection point sits on it.
Library symbols have y pointing up, sheets have y pointing down.
"""

from __future__ import annotations

from dataclasses import dataclass

MM = 0.0254  # mm per mil
GRID = 50

Pt = tuple[int, int]
RIGHT: Pt = (1, 0)
LEFT: Pt = (-1, 0)
UP: Pt = (0, -1)
DOWN: Pt = (0, 1)


def mm(v: int) -> float:
    """Mils to millimetres, as KiCad writes them."""
    return round(v * MM, 4) + 0.0


def mil(v) -> int:
    """Millimetres (number or string) to mils."""
    return int(round(float(v) / MM))


def lib_to_sheet(x: int, y: int, rot: int, mirror: str | None = None) -> Pt:
    """Sheet offset of a library point for a symbol rotated by `rot` degrees.

    The rotation is counter-clockwise as seen on the sheet. Mirroring is only
    accepted together with 0 or 180 degrees, where its order with the
    rotation does not matter.
    """
    r = rot % 360
    if r == 0:
        rx, ry = x, y
    elif r == 90:
        rx, ry = -y, x
    elif r == 180:
        rx, ry = -x, -y
    elif r == 270:
        rx, ry = y, -x
    else:
        raise ValueError(f"rotation {rot} is not a multiple of 90")
    sx, sy = rx, -ry
    if mirror is not None:
        if r not in (0, 180):
            raise ValueError("mirror only with rotation 0 or 180")
        if mirror == "x":
            sy = -sy
        elif mirror == "y":
            sx = -sx
        else:
            raise ValueError(mirror)
    return sx, sy


def lib_dir(angle: float, rot: int, mirror: str | None = None) -> Pt:
    """Sheet direction of a library angle (0 = +x, 90 = +y up)."""
    v = {0: (1, 0), 90: (0, 1), 180: (-1, 0), 270: (0, -1)}[int(round(angle)) % 360]
    return lib_to_sheet(v[0], v[1], rot, mirror)


def add(p: Pt, d: Pt, k: int = 1) -> Pt:
    return (p[0] + d[0] * k, p[1] + d[1] * k)


@dataclass(frozen=True)
class Box:
    x1: int
    y1: int
    x2: int
    y2: int

    @staticmethod
    def around(points) -> Box:
        xs = [p[0] for p in points]
        ys = [p[1] for p in points]
        return Box(min(xs), min(ys), max(xs), max(ys))

    def grow(self, d: int) -> Box:
        return Box(self.x1 - d, self.y1 - d, self.x2 + d, self.y2 + d)

    def overlaps(self, o: Box) -> bool:
        return self.x1 < o.x2 and o.x1 < self.x2 and self.y1 < o.y2 and o.y1 < self.y2

    def inside(self, p: Pt) -> bool:
        """Strictly inside (the border does not count)."""
        return self.x1 < p[0] < self.x2 and self.y1 < p[1] < self.y2

    def cut_by(self, a: Pt, b: Pt) -> bool:
        """Whether the orthogonal segment a-b passes through the inside."""
        if a[1] == b[1]:
            lo, hi = sorted((a[0], b[0]))
            return self.y1 < a[1] < self.y2 and lo < self.x2 and self.x1 < hi
        lo, hi = sorted((a[1], b[1]))
        return self.x1 < a[0] < self.x2 and lo < self.y2 and self.y1 < hi


def on_segment(p: Pt, a: Pt, b: Pt) -> bool:
    """Whether p lies strictly between the ends of the orthogonal segment a-b."""
    if a[1] == b[1] == p[1]:
        return min(a[0], b[0]) < p[0] < max(a[0], b[0])
    if a[0] == b[0] == p[0]:
        return min(a[1], b[1]) < p[1] < max(a[1], b[1])
    return False


def text_width(text: str, size: int) -> int:
    """Width of KiCad's stroke font text, rounded up (about 0.9 em per character)."""
    return int(0.92 * size * max(len(line) for line in text.split("\n"))) + 1
