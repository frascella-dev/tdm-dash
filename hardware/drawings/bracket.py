"""Sensor bracket at 1:1: the outline to glue on the plate, with its dimensions.

The parameters come from hardware/bracket/sensor-bracket.scad and the outline
from its 2D export (hardware/exports/cad/sensor-bracket.svg), so the print is
the shape that gets cut.
"""

from __future__ import annotations

import math
from pathlib import Path

from sheet import (CENTER, FILL, GREY, HIDDEN, INK, MEDIUM, THICK, THIN, VERIFY, Drawing, View, angle_dim, diameter,
                   dim, fmt, frame, hdim, leader, notes, scad_params, scale_bar, svg_outline, vdim)

SCAD = "hardware/bracket/sensor-bracket.scad"
OUTLINE = "hardware/exports/cad/sensor-bracket.svg"


def color(*values) -> str:
    return VERIFY if any(getattr(v, "verify", False) for v in values) else INK


def check_outline(polys, p) -> None:
    """Refuses an outline exported from other parameters than the .scad ones."""
    xs = [x for poly in polys for x, _ in poly]
    lobe = p["bolt_d"] / 2 + p["margin"]
    right = p["bolt_pitch"] + lobe if p["second_bolt"] else None
    if abs(min(xs) + lobe) > 0.05 or (right is not None and abs(max(xs) - right) > 0.05):
        raise SystemExit(f"{OUTLINE} does not match {SCAD}: run hardware/cad-export.sh first")


def build(root: Path, rev: str, date: str) -> Drawing:
    p = scad_params(root / SCAD)
    polys = svg_outline(root / OUTLINE)
    check_outline(polys, p)
    d = Drawing("tdm-dash · sensor bracket, 1:1")
    frame(d, "Sensor bracket", "mechanical parts · Yamaha TDM 850 (3VD)", "1:1", rev, date, SCAD)

    bolt_r, sensor_r = p["bolt_d"] / 2, p["sensor_d"] / 2
    lobe = bolt_r + p["margin"]
    a = math.radians(p["angle"])
    along = (math.cos(a), math.sin(a))
    sensor = (p["reach"] * along[0], p["reach"] * along[1])
    ends = [(sensor[0] + s * p["slot"] / 2 * along[0], sensor[1] + s * p["slot"] / 2 * along[1]) for s in (-1, 1)]
    v = View(66, 116)

    # the part, holes included
    cmds = []
    for poly in polys:
        cmds += [("M", v(*poly[0]))] + [("L", v(*q)) for q in poly[1:]] + [("Z",)]
    d.path(cmds, INK, THICK, fill=FILL, evenodd=True)

    # center lines: bolt line, sensor arm, crosses on every hole center
    b1, b2 = (0.0, 0.0), (float(p["bolt_pitch"]), 0.0)
    far = b2[0] + bolt_r + 3 if p["second_bolt"] else lobe + 6
    d.line(v(-bolt_r - 3, 0), v(far, 0), GREY, THIN, CENTER)
    tip = p["reach"] + p["slot"] / 2 + sensor_r + 3
    d.line(v(0, 0), v(tip * along[0], tip * along[1]), GREY, THIN, CENTER)
    holes = [b1, b2] if p["second_bolt"] else [b1]
    for c in holes:
        d.line(v(c[0], -bolt_r - 3), v(c[0], bolt_r + 3), GREY, THIN, CENTER)
    across = (-along[1], along[0])
    for c in ends:
        d.line(v(c[0] - (sensor_r + 3) * across[0], c[1] - (sensor_r + 3) * across[1]),
               v(c[0] + (sensor_r + 3) * across[0], c[1] + (sensor_r + 3) * across[1]), GREY, THIN, CENTER)

    # dimensions
    if p["second_bolt"]:
        hdim(d, v(*b1), v(*b2), v(0, -lobe)[1] + 10, fmt(p["bolt_pitch"]), color(p["bolt_pitch"]))
    page_arm = (v(*sensor)[0] - v(0, 0)[0], v(*sensor)[1] - v(0, 0)[1])
    left = (page_arm[1] / math.hypot(*page_arm), -page_arm[0] / math.hypot(*page_arm))
    off = sensor_r + p["margin"] + 9
    o = v(0, 0)
    dim(d, o, v(*sensor), page_arm, (o[0] + left[0] * off, o[1] + left[1] * off),
        fmt(p["reach"]), color(p["reach"]))
    s0, s1 = v(*ends[0]), v(*ends[1])
    dim(d, s0, s1, page_arm, (s1[0] - left[0] * off, s1[1] - left[1] * off),
        fmt(p["slot"]), color(p["slot"]))
    angle_dim(d, o, 26, -float(p["angle"]), 0, f"{fmt(p['angle'])}°", color(p["angle"]))
    n = "2× " if p["second_bolt"] else ""
    diameter(d, o, bolt_r, 135, f"{n}Ø{fmt(p['bolt_d'])}", color(p["bolt_d"]), reach=p["margin"] + 7)
    out = v(ends[1][0] + sensor_r * across[0], ends[1][1] + sensor_r * across[1])
    leader(d, out, (out[0] + 12, out[1] - 10), f"slot {fmt(p['sensor_d'])} wide", color(p["sensor_d"]))

    # edge view under the part: the plate thickness, holes as hidden lines
    xs = [x for poly in polys for x, _ in poly]
    e = View(v.ox, v(0, -lobe)[1] + 24)
    x0, x1, t = min(xs), max(xs), float(p["t"])
    d.rect(*e(x0, t), *e(x1, 0), INK, THICK, fill=FILL)
    spans = [(c[0] - bolt_r, c[0] + bolt_r) for c in holes]
    spans.append((min(c[0] for c in ends) - sensor_r, max(c[0] for c in ends) + sensor_r))
    for lo, hi in spans:
        for x in (lo, hi):
            d.line(e(x, t), e(x, 0), INK, MEDIUM, HIDDEN)
    vdim(d, e(x0, 0), e(x0, t), e(x0, 0)[0] - 8, fmt(p["t"]), color(p["t"]))
    d.text(e(x0, 0)[0], e(0, 0)[1] + 12, f"Plate {fmt(p['t'])} mm, aluminum 5754 or 6082", 3.0, INK, bold=True)

    scale_bar(d, 24, 182, 1.0)
    notes(d, 172, 20, [
        ("Print the PDF at 100 % (actual size) on A4 landscape and check the 50 mm bar.", INK),
        ("Glue the print on the plate, center-punch the crossings of the center lines, "
         "drill the holes, then cut and file the outline.", INK),
        ("Only aluminum may go under a brake caliper bolt. A printed bracket is a fit check, "
         "never a part (chapter 07).", INK),
        (f"The sensor slides in the slot: {fmt(p['reach'])} ± {fmt(p['slot'] / 2)} mm from the bolt. "
         "Its two M12 nuts set the 1–2 mm gap to the disc bolts.", INK),
        ("Dimensions in mm, from sensor-bracket.scad. Orange values are [TO VERIFY] on the bike "
         "(docs/research/bike-measurements.md): set them in the .scad file and regenerate.", VERIFY),
    ], width=108)
    return d
