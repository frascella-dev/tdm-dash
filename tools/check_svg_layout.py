#!/usr/bin/env python3
"""Checks the layout of the schematic sheets exported by kicad-cli as SVG.

KiCad writes every string of the stroke font as
`<g class="stroked-text"><desc>TEXT</desc><path .../>...</g>`, so the exact
box of each text can be measured. The check fails when:

- two texts overlap;
- a wire (KiCad default color #009600) crosses a text;
- a symbol outline (#840000) crosses a reference, value or label;
- a text leaves the frame or enters the title block.

    python3 tools/check_svg_layout.py hardware/exports/kicad/*.svg

The frame and title block positions are those of hardware/kicad/tdm-dash.kicad_wks
on A4 landscape.
"""

from __future__ import annotations

import re
import sys
import xml.etree.ElementTree as ET

NS = "{http://www.w3.org/2000/svg}"
WIRE = "#009600"
OUTLINE = "#840000"
PAGE = (297.0, 210.0)
MARGIN, STRIP = 10.0, 4.0
TITLE_BLOCK = (124.0, 34.0)  # from the bottom-right corner of the margin box
TOL = 0.08  # mm: boxes closer than this are not an overlap


def numbers(d: str) -> list[tuple[float, float]]:
    v = [float(x) for x in re.findall(r"-?\d+(?:\.\d+)?", d)]
    return list(zip(v[0::2], v[1::2]))


def stroke_of(style: str | None) -> str | None:
    if not style:
        return None
    m = re.search(r"stroke:(#[0-9A-Fa-f]{6})", style)
    return m.group(1).upper() if m else None


class Text:
    def __init__(self, text, pts, color):
        self.text = text
        xs = [p[0] for p in pts]
        ys = [p[1] for p in pts]
        self.box = (min(xs), min(ys), max(xs), max(ys))
        self.color = color

    def __repr__(self):
        x1, y1, x2, y2 = self.box
        return f"'{self.text}' at ({x1:.1f}, {y1:.1f}) mm"


def collect(path: str):
    texts: list[Text] = []
    segs: dict[str, list] = {WIRE: [], OUTLINE: []}

    def walk(el, color):
        style_color = stroke_of(el.get("style"))
        color = style_color or color
        if el.tag == NS + "g" and el.get("class") == "stroked-text":
            desc = el.find(NS + "desc")
            pts = []
            for p in el.iter(NS + "path"):
                pts += numbers(p.get("d", ""))
            if pts and desc is not None and (desc.text or "").strip():
                texts.append(Text(desc.text.strip(), pts, color))
            return
        if el.tag == NS + "path" and color in segs:
            pts = numbers(el.get("d", ""))
            segs[color] += list(zip(pts, pts[1:]))
        for child in el:
            walk(child, color)

    walk(ET.parse(path).getroot(), None)
    # KiCad plots some fields twice at the same place: keep one of each
    seen, unique = set(), []
    for t in texts:
        key = (t.text, *(round(v, 2) for v in t.box))
        if key not in seen:
            seen.add(key)
            unique.append(t)
    return unique, segs


def overlap(a, b, tol=TOL) -> bool:
    return a[0] + tol < b[2] and b[0] + tol < a[2] and a[1] + tol < b[3] and b[1] + tol < a[3]


def seg_hits_box(p, q, box, tol=TOL) -> bool:
    x1, y1, x2, y2 = box[0] + tol, box[1] + tol, box[2] - tol, box[3] - tol
    if x1 >= x2 or y1 >= y2:
        return False
    # Liang-Barsky clipping of the segment against the box
    t0, t1 = 0.0, 1.0
    dx, dy = q[0] - p[0], q[1] - p[1]
    for pp, qq in ((-dx, p[0] - x1), (dx, x2 - p[0]), (-dy, p[1] - y1), (dy, y2 - p[1])):
        if pp == 0:
            if qq < 0:
                return False
        else:
            t = qq / pp
            if pp < 0:
                t0 = max(t0, t)
            else:
                t1 = min(t1, t)
            if t0 > t1:
                return False
    return True


def check(path: str) -> list[str]:
    texts, segs = collect(path)
    w, h = PAGE
    inner = (MARGIN + STRIP, MARGIN + STRIP, w - MARGIN - STRIP, h - MARGIN - STRIP)
    tb = (w - MARGIN - TITLE_BLOCK[0], h - MARGIN - TITLE_BLOCK[1], w - MARGIN, h - MARGIN)

    def sheet_item(box):  # frame, zone marks and title block belong to the drawing sheet
        x1, y1, x2, y2 = box
        in_strip = x1 < inner[0] or y1 < inner[1] or x2 > inner[2] or y2 > inner[3]
        return in_strip or overlap(box, tb, 0)

    drawing = [t for t in texts if not (t.color == OUTLINE and sheet_item(t.box))]
    errors = []
    for t in drawing:
        x1, y1, x2, y2 = t.box
        if x1 < inner[0] or y1 < inner[1] or x2 > inner[2] or y2 > inner[3]:
            errors.append(f"{t} leaves the frame")
        elif overlap(t.box, tb, 0):
            errors.append(f"{t} enters the title block")
    for i, a in enumerate(drawing):
        for b in drawing[i + 1 :]:
            if overlap(a.box, b.box):
                errors.append(f"{a} overlaps {b}")
    for t in drawing:
        for p, q in segs[WIRE]:
            if seg_hits_box(p, q, t.box):
                errors.append(f"{t} is crossed by a wire")
                break
    outlines = [(p, q) for p, q in segs[OUTLINE] if not sheet_item((min(p[0], q[0]), min(p[1], q[1]),
                                                                     max(p[0], q[0]), max(p[1], q[1])))]
    for t in drawing:
        if t.color == OUTLINE:
            continue  # pin numbers sit on their pins by design
        for p, q in outlines:
            if seg_hits_box(p, q, t.box, tol=0.15):
                errors.append(f"{t} is crossed by a symbol outline")
                break
    return errors


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    bad = 0
    for path in sys.argv[1:]:
        errors = check(path)
        name = path.rsplit("/", 1)[-1]
        if errors:
            bad += 1
            print(f"{name}: {len(errors)} layout problems")
            for e in errors:
                print(f"  {e}")
        else:
            print(f"{name}: layout ok")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
