"""Bike mount at 1:1: the aluminum plate, and a fit template of the dashboard.

The plate is screwed to the three lugs of the pod and carries two M6 screws in
place of the studs of the original cluster, which go through the rubber
grommets of the fairing stay. Its outline comes from the OpenSCAD export
(hardware/exports/cad/dash-mount.svg), so the print is the shape that gets
cut. The fit template is the outline of the display unit at full size, to cut
in cardboard and hold on the bike before printing anything. Every value comes
from hardware/enclosure/enclosure.scad.
"""

from __future__ import annotations

import math
from pathlib import Path

from sheet import (CENTER, FILL, GREY, HIDDEN, INK, MEDIUM, NOTE, THICK, THIN, VERIFY, Drawing, View, center_mark,
                   diameter, fmt, frame, hdim, leader, notes, scad_params, scale_bar, svg_outline, vdim)

SCAD = "hardware/enclosure/enclosure.scad"
OUTLINE = "hardware/exports/cad/dash-mount.svg"


def color(*values) -> str:
    return VERIFY if any(getattr(v, "verify", False) for v in values) else INK


def studs(p) -> list[tuple[float, float]]:
    return [(p["stud_x"] + s * p["stud_pitch"] / 2, p["stud_y"]) for s in (-1, 1)]


def pod_screws(p) -> list[tuple[float, float]]:
    """The four M3 screws that close the pod, as `pod_screws` in the .scad file."""
    dx, dy = p["pod_in_w"] / 2 - 3, p["pod_in_h"] / 2 - 3
    return [(sx * dx, p["pod_cy"] + sy * dy) for sx in (-1, 1) for sy in (-1, 1)]


def stud_pad(p) -> float:
    return max(p["grommet_d"] / 2, 1.5 * p["m6_hole"])


def m6_len(p) -> int:
    need = p["plate_t"] + p["grommet_stack"] + 1.6 + 6 + 2
    return next(n for n in (16, 20, 25, 30, 35, 40, 45, 50) if n >= need)


def check_outline(polys, p) -> None:
    """Refuses an outline exported from other parameters than the .scad ones."""
    xs = [x for poly in polys for x, _ in poly]
    ys = [y for poly in polys for _, y in poly]
    pads = [(s, stud_pad(p)) for s in studs(p)] + [(tuple(m), p["lug_pad"]) for m in p["mount_points"]]
    want = (min(c[0] - r for c, r in pads), max(c[0] + r for c, r in pads),
            min(c[1] - r for c, r in pads), max(c[1] + r for c, r in pads))
    if any(abs(a - b) > 0.1 for a, b in zip((min(xs), max(xs), min(ys), max(ys)), want)):
        raise SystemExit(f"{OUTLINE} does not match {SCAD}: run hardware/cad-export.sh first")


def outline_path(d: Drawing, v: View, polys, **kw) -> None:
    cmds = []
    for poly in polys:
        cmds += [("M", v(*poly[0]))] + [("L", v(*q)) for q in poly[1:]] + [("Z",)]
    d.path(cmds, **kw)


def build(root: Path, rev: str, date: str) -> Drawing:
    """The plate, seen from the front of the bike as it goes on, and a section through a stud."""
    p = scad_params(root / SCAD)
    polys = svg_outline(root / OUTLINE)
    check_outline(polys, p)
    d = Drawing("tdm-dash · dashboard mount plate, 1:1")
    frame(d, "Dashboard mount plate", "mechanical parts · Yamaha TDM 850 (3VD)", "1:1", rev, date, SCAD)

    st, lugs = studs(p), [tuple(m) for m in p["mount_points"]]
    sc = color(p["stud_pitch"], p["stud_x"], p["stud_y"])
    xs = [x for poly in polys for x, _ in poly]
    ys = [y for poly in polys for _, y in poly]
    v = View(112, 72 + (max(ys) - 36.5), 1.0, sx=-1)   # from the bike side: the part's x runs leftwards

    outline_path(d, v, polys, color=INK, width=THICK, fill=FILL, evenodd=True)
    d.line(v(0, max(ys) + 4), v(0, min(ys) - 4), GREY, THIN, CENTER)
    for c in st:
        center_mark(d, v(*c), p["m6_hole"] / 2, 4)
    for c in lugs:
        center_mark(d, v(*c), p["m5_hole"] / 2, 4)

    # dimensions: stud pitch under the plate, lug pattern over it, a chain on the free side
    side = [c for c in lugs if abs(c[1] - min(m[1] for m in lugs)) < 0.01]
    top = max(lugs, key=lambda c: c[1])
    hdim(d, v(*st[0]), v(*st[1]), v(0, min(ys))[1] + 8, fmt(p["stud_pitch"]), sc)
    if len(side) == 2:
        hdim(d, v(*side[0]), v(*side[1]), v(0, max(ys))[1] - 8, fmt(abs(side[1][0] - side[0][0])))
    lug_side = max(side, key=lambda c: c[0])
    stud_side = max(st, key=lambda c: c[0])
    x_chain = v(max(xs), 0)[0] - 6                     # left of the outline, which is the part's +x
    vdim(d, v(*lug_side), v(*top), x_chain, fmt(top[1] - lug_side[1]))
    vdim(d, v(*stud_side), v(*lug_side), x_chain, fmt(abs(lug_side[1] - stud_side[1])), sc)
    if abs(p["stud_x"]) > 0.05:
        hdim(d, v(0, p["stud_y"]), v(p["stud_x"], p["stud_y"]), v(0, p["stud_y"])[1] - 6, fmt(abs(p["stud_x"])), sc)

    # callouts
    s_left = min(st, key=lambda c: v(*c)[0])
    diameter(d, v(*s_left), p["m6_hole"] / 2, 125, f"2× Ø{fmt(p['m6_hole'])}", reach=8)
    l_right = max(side, key=lambda c: v(*c)[0])
    diameter(d, v(*l_right), p["m5_hole"] / 2, -35, f"3× Ø{fmt(p['m5_hole'])}", reach=7)
    q = max(pod_screws(p), key=lambda c: (c[1], v(*c)[0]))   # the upper one on the right of the page
    u = (-q[0] / math.hypot(*q), -q[1] / math.hypot(*q))
    tip = v(q[0] + 4.5 * u[0], q[1] + 4.5 * u[1])
    if v(0, max(ys))[1] < tip[1]:
        leader(d, tip, (tip[0] + 9, v(0, max(ys))[1] - 1), "4× R4.5")

    # section through a stud, 1:1: the order of the parts on the bike
    sx0, sy0 = 40.0, 146.0
    g, stack, t = float(p["grommet_d"]), float(p["grommet_stack"]), float(p["plate_t"])
    gc = color(p["grommet_d"], p["grommet_stack"])
    d.text(sx0 - g / 2 - 6, sy0 - 4, "SECTION THROUGH A STUD, 1:1", 2.6, INK, bold=True)
    y = sy0
    d.rect(sx0 - 5, y, sx0 + 5, y + 4, INK, MEDIUM, fill="#ffffff")                    # M6 head
    y += 4
    d.rect(sx0 - g / 2, y, sx0 + g / 2, y + t, INK, THICK, fill=FILL)                   # plate
    y_plate = y
    y += t
    fl = (stack - 2) / 2                                                                # grommet flanges
    d.rect(sx0 - g / 2 - 14, y + fl, sx0 - 7, y + fl + 2, INK, MEDIUM, fill="#d9d9d9")  # stay tab
    d.rect(sx0 + 7, y + fl, sx0 + g / 2 + 14, y + fl + 2, INK, MEDIUM, fill="#d9d9d9")
    grommet = [(-g / 2, 0), (g / 2, 0), (g / 2, fl), (7, fl), (7, fl + 2), (g / 2, fl + 2), (g / 2, stack),
               (-g / 2, stack), (-g / 2, fl + 2), (-7, fl + 2), (-7, fl), (-g / 2, fl)]
    d.polyline([(sx0 + a, y + b) for a, b in grommet], gc, MEDIUM, closed=True, fill="#bdbdbd")
    d.rect(sx0 - 4.5, y, sx0 + 4.5, y + stack, gc, MEDIUM, fill="#ffffff")             # collar
    y_grommet = y
    y += stack
    d.rect(sx0 - 6, y, sx0 + 6, y + 1.6, INK, MEDIUM, fill="#ffffff")                   # washer
    y_washer = y
    y += 1.6
    d.rect(sx0 - 5, y, sx0 + 5, y + 6, INK, MEDIUM, fill="#ffffff")                     # nut
    for x in (sx0 - 2, sx0 + 2):
        d.line((x, y), (x, y + 6), INK, THIN)
    y_nut = y
    end = sy0 + 4 + m6_len(p)
    d.rect(sx0 - 3, sy0 + 4, sx0 + 3, end, INK, MEDIUM, fill="#ffffff")                 # shank, not cut
    d.line((sx0, sy0 - 2), (sx0, end + 2), GREY, THIN, CENTER)
    rows = [
        (sy0 + 2, f"M6 × {m6_len(p)} hex screw, head on the pod side", gc),
        (y_plate + t / 2, f"mount plate, {fmt(t)} mm aluminum", INK),
        (y_grommet + fl / 2, "original grommet 90480-14555", gc),
        (y_grommet + fl + 1, "fairing stay, tab of the original cluster", INK),
        (y_grommet + stack - fl / 2, "original collar 90387-07391, inside the grommet", gc),
        (y_washer + 0.8, "original washer 90201-06059", INK),
        (y_nut + 3, "original nut 95316-06600, or an M6 nyloc", INK),
    ]
    lx = sx0 + g / 2 + 18
    last = -1e9
    for yy, text, c in rows:
        ty = max(yy, last + 3.4)
        d.line((sx0 + (g / 2 + 14 if "stay" in text else 6), yy), (lx - 1, ty), c, THIN)
        d.text(lx, ty, text, 2.2, c, middle=True)
        last = ty

    scale_bar(d, 24, 188, 1.0)
    w, h = max(xs) - min(xs), max(ys) - min(ys)
    notes(d, 210, 20, [
        (f"Print the PDF at 100 % on A4 landscape and check the 50 mm bar. Glue the print on a {fmt(t)} mm "
         f"aluminum plate (5754 or 6082, blank {math.ceil(w) + 10} × {math.ceil(h) + 10} mm), center-punch "
         "the hole centers, drill, then cut and file the outline.", INK),
        ("Seen from the front of the bike, as it goes on. Three M5 × 10 button head screws hold it in "
         "the inserts of the pod lugs; the two M6 screws take the place of the studs of the original "
         "cluster.", INK),
        ("Put the M6 screws in before screwing the plate to the pod: their heads stay between the "
         "plate and the display unit. The notches keep the four pod screws reachable.", INK),
        ("On the bike every M6 screw goes through an original grommet of the fairing stay, with the "
         "collar inside, then the washer and the nut: tighten until the collar is clamped, about "
         "7 Nm [HYPOTHESIS].", INK),
        ("Orange values are [TO VERIFY] on the bike (docs/research/bike-measurements.md): the stud "
         "pitch and position, the grommet. Set them in enclosure.scad, run cad-export.sh and "
         "generate.py, and try the fit template before cutting.", VERIFY),
    ], width=73, size=2.4)
    return d


def template(root: Path, rev: str, date: str) -> Drawing:
    """The display unit at full size, seen from the rider, with the pod, plate and studs behind it."""
    p = scad_params(root / SCAD)
    polys = svg_outline(root / OUTLINE)
    check_outline(polys, p)
    d = Drawing("tdm-dash · fit template, 1:1")
    frame(d, "Fit template", "mechanical parts · Yamaha TDM 850 (3VD)", "1:1", rev, date, SCAD)

    hw, top, bot = p["half_w"], p["top_y"], p["bottom_y"]
    f = View(125, 86, 1.0)
    st = studs(p)
    sc = color(p["stud_pitch"], p["stud_x"], p["stud_y"])

    d.rounded_polygon([f(*q) for q in p["outline_pts"]], float(p["corner_r"]), INK, THICK, fill="#ffffff")
    ax, ay = p["aa_dx"], p["aa_dy"]
    wx, wy = (p["aa_w"] + 1) / 2 + p["front_t"], (p["aa_h"] + 1) / 2 + p["front_t"]
    d.rect(*f(ax - wx, ay + wy), *f(ax + wx, ay - wy), GREY, THIN)
    for sx in (-1, 1):
        c = f(sx * p["button_x"], p["button_y"])
        d.circle(c, p["button_d"] / 2, GREY, THIN)
        center_mark(d, c, p["button_d"] / 2, 2)
    d.circle(f(p["light_x"], p["rim_screw_y"]), p["light_d"] / 2, GREY, THIN)
    d.line(f(-hw - 4, 0), f(hw + 4, 0), GREY, THIN, CENTER)
    d.line(f(0, bot - 4), f(0, top + 4), GREY, THIN, CENTER)

    # behind the display unit: the pod and the plate, hidden lines
    pw, ph, pcy = p["pod_w"], p["pod_h"], p["pod_cy"]
    d.rounded_rect(*f(-pw / 2, pcy + ph / 2), *f(pw / 2, pcy - ph / 2), p["pod_r"], GREY, MEDIUM, HIDDEN)
    outline_path(d, f, polys[:1], color=GREY, width=MEDIUM, dash=HIDDEN)
    for c in st:
        d.circle(f(*c), p["m6_hole"] / 2, sc, MEDIUM)
        center_mark(d, f(*c), p["m6_hole"] / 2, 5)

    hdim(d, f(-hw, top), f(hw, top), f(0, top)[1] - 6, fmt(2 * hw))
    low = min(y for poly in polys for _, y in poly)
    hdim(d, f(*st[0]), f(*st[1]), f(0, low)[1] + 6, fmt(p["stud_pitch"]), sc, text_at=0.35)
    vdim(d, f(p["stud_x"], 0), f(p["stud_x"], p["stud_y"]), f(p["stud_x"], 0)[0] + 9, fmt(-p["stud_y"]), sc,
         text_at=0.45)
    x0, y0 = f(-hw + 10, top)
    d.text(x0, y0 + 7, "FIT TEMPLATE · seen from the rider", 3.0, INK, bold=True)
    d.text(x0, y0 + 11.5, "cut along the thick line, pierce the two crosses", 2.2, NOTE)

    scale_bar(d, 24, 186, 1.0)
    unit, depth = p["unit_d"], -p["z_back"]
    notes(d, 237, 20, [
        (f"Print at 100 %: the bar measures 50 mm and the width {fmt(2 * hw)} mm. Glue on cardboard "
         "and cut along the thick line.", INK),
        ("Hold it where the original cluster was, the face towards the rider. It must clear the "
         "windscreen, the inner fairing panels and the ignition key with the handlebar at full lock on "
         "both sides.", INK),
        ("The crosses are the M6 studs of the mount plate: they must fall on the grommets of the "
         "fairing stay. If not, measure the grommets from the center lines and set stud_pitch, "
         "stud_x and stud_y in enclosure.scad.", VERIFY),
        (f"Dashed: the pod and the mount plate, behind the display unit. Depth behind the face: "
         f"{fmt(unit)} mm display unit, {fmt(depth)} mm with the pod, {fmt(depth + p['plate_t'])} mm "
         "with the plate.", INK),
    ], width=46, size=2.2)
    return d
