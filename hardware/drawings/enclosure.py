"""Enclosure at 1:2: front, back and bottom, with the main dimensions.

The display unit (bezel and cover) is thin; the pod behind it holds the
perfboard and carries the bike mount. The parts are printed from the STL
files; this sheet is for checking them and for fitting the enclosure to the
bike. Every value comes from hardware/enclosure/enclosure.scad.
"""

from __future__ import annotations

from pathlib import Path

from sheet import (CENTER, FILL, GREY, INK, MEDIUM, NOTE, THICK, THIN, VERIFY, Drawing, View, diameter, fmt, frame,
                   hdim, notes, scad_params, scale_bar, vdim)

SCAD = "hardware/enclosure/enclosure.scad"
S = 0.5  # 1:2


def color(*values) -> str:
    return VERIFY if any(getattr(v, "verify", False) for v in values) else INK


def label(d: Drawing, x: float, y: float, title: str, sub: str) -> None:
    d.text(x, y, title, 3.0, INK, "middle", bold=True)
    d.text(x, y + 3.6, sub, 2.2, NOTE, "middle")


def build(root: Path, rev: str, date: str) -> Drawing:
    p = scad_params(root / SCAD)
    d = Drawing("tdm-dash · enclosure, 1:2")
    frame(d, "Enclosure", "mechanical parts · Yamaha TDM 850 (3VD)", "1:2", rev, date, SCAD)

    hw, top, bot = p["half_w"], p["top_y"], p["bottom_y"]
    outline = p["outline_pts"]
    r = float(p["corner_r"])
    bx, by, bd = p["button_x"], p["button_y"], p["button_d"]
    unit, z_back = p["unit_d"], p["z_back"]
    pw, ph, pcy = p["pod_w"], p["pod_h"], p["pod_cy"]
    pod_screws = [(sx * (p["pod_in_w"] / 2 - 3), pcy + sy * (p["pod_in_h"] / 2 - 3)) for sx in (-1, 1) for sy in (-1, 1)]
    mounts = p["mount_points"]
    mc = color(*[v for m in mounts for v in m])

    # ------------------------------------------------------------ front
    f = View(82, 63, S)
    d.rounded_polygon([f(*q) for q in outline], r * S, INK, THICK, fill=FILL)
    ax, ay = p["aa_dx"], p["aa_dy"]
    inner = ((p["aa_w"] + 1) / 2, (p["aa_h"] + 1) / 2)                  # back of the face
    outer = (inner[0] + p["front_t"], inner[1] + p["front_t"])          # 45° chamfer on the face
    d.rect(*f(ax - outer[0], ay + outer[1]), *f(ax + outer[0], ay - outer[1]), INK, THICK, fill="#ffffff")
    d.rect(*f(ax - inner[0], ay + inner[1]), *f(ax + inner[0], ay - inner[1]), INK, MEDIUM)
    for sx in (-1, 1):
        for sy in (-1, 1):
            d.line(f(ax + sx * inner[0], ay + sy * inner[1]), f(ax + sx * outer[0], ay + sy * outer[1]), INK, MEDIUM)
        d.circle(f(sx * bx, by), bd / 2 * S, INK, MEDIUM, fill="#ffffff")
        d.line(f(sx * bx - 9, by), f(sx * bx + 9, by), GREY, THIN, CENTER)
        d.line(f(sx * bx, by - 9), f(sx * bx, by + 9), GREY, THIN, CENTER)
    light = (p["light_x"], p["rim_screw_y"])
    d.circle(f(*light), p["light_d"] / 2 * S, INK, MEDIUM, fill="#ffffff")
    d.line(f(-hw - 4, 0), f(hw + 4, 0), GREY, THIN, CENTER)
    d.line(f(0, bot - 4), f(0, top + 4), GREY, THIN, CENTER)
    hdim(d, f(-hw, top), f(hw, top), f(0, top)[1] - 7, fmt(2 * hw))
    vdim(d, f(-hw, bot), f(-hw, top), f(-hw, 0)[0] - 7, fmt(top - bot))
    hdim(d, f(ax - outer[0], ay + outer[1]), f(ax + outer[0], ay + outer[1]), f(0, ay + outer[1])[1] + 7,
         f"window {fmt(2 * outer[0])}", text_at=0.3)
    vdim(d, f(ax + outer[0], ay - outer[1]), f(ax + outer[0], ay + outer[1]), f(ax + outer[0], 0)[0] - 8,
         fmt(2 * outer[1]), text_at=0.3)
    hdim(d, f(-bx, by), f(bx, by), f(0, by)[1] + 12, fmt(2 * bx), text_at=0.7)
    vdim(d, f(hw, bot), f(bx, by), f(hw, 0)[0] + 6, fmt(by - bot))
    diameter(d, f(*light), p["light_d"] / 2 * S, 60, f"Ø{fmt(p['light_d'])} light pipe", reach=5)
    label(d, f(0, 0)[0], f(0, bot)[1] + 8, "FRONT", f"rider side · buttons 2× Ø{fmt(bd)}: SET left, MODE right")

    # ------------------------------------------------------------ back
    k = View(212, 63, S, sx=-1)               # seen from behind: the part's x runs leftwards
    d.rounded_polygon([k(*q) for q in outline], r * S, INK, THICK, fill=FILL)
    for q in p["screws"]:
        d.circle(k(*q), 2.8 * S, INK, THIN, fill="#ffffff")
    for sx in (-1, 1):
        d.circle(k(sx * bx, by), p["cup_d"] / 2 * S, INK, MEDIUM, fill="#d9d9d9")
    lug_r = 7
    for m in mounts:                           # lugs, drawn under the pod outline
        d.circle(k(*m), lug_r * S, INK, MEDIUM, fill="#d9d9d9")
    x0, y0 = k(pw / 2, pcy + ph / 2)
    x1, y1 = k(-pw / 2, pcy - ph / 2)
    d.rounded_polygon([(x0, y0), (x1, y0), (x1, y1), (x0, y1)], p["pod_r"] * S, INK, THICK, fill="#d9d9d9")
    for q in pod_screws:
        d.circle(k(*q), 3.0 * S, INK, THIN, fill="#ffffff")
    for m in mounts:
        d.circle(k(*m), p["mount_insert_d"] / 2 * S, mc, MEDIUM, fill="#ffffff")
        d.line(k(m[0] - 9, m[1]), k(m[0] + 9, m[1]), GREY, THIN, CENTER)
        d.line(k(m[0], m[1] - 9), k(m[0], m[1] + 9), GREY, THIN, CENTER)
    side = sorted((m for m in mounts if abs(m[0]) > 1), key=lambda m: m[0])
    upper = [m for m in mounts if abs(m[0]) <= 1]
    if len(side) == 2:
        hdim(d, k(*side[1]), k(*side[0]), k(0, pcy - ph / 2)[1] + 5, fmt(side[1][0] - side[0][0]), mc)
    if side and upper:
        vdim(d, k(*side[0]), k(*upper[0]), k(side[0][0], 0)[0] + 9, fmt(upper[0][1] - side[0][1]), mc)
    hdim(d, k(pw / 2, pcy + ph / 2), k(-pw / 2, pcy + ph / 2), k(0, top)[1] - 7, fmt(pw), color(pw))
    vdim(d, k(-pw / 2, pcy - ph / 2), k(-pw / 2, pcy + ph / 2), k(-pw / 2, 0)[0] - 6, fmt(ph), text_at=0.7)
    label(d, k(0, 0)[0], k(0, bot)[1] + 8, "BACK",
          f"bike side · 3× M5 inserts Ø{fmt(p['mount_insert_d'])} in the lugs · 4× M3 pod screws")

    # ------------------------------------------------------------ bottom
    b = View(82, 117, S)                      # part (x, z): front face at the top
    zc = -(p["bezel_d"] + p["gasket_t"])      # front face of the cover
    d.rect(*b(-hw, 0), *b(hw, -p["bezel_d"]), INK, THICK, fill=FILL)
    d.rect(*b(-hw, zc), *b(hw, -unit), INK, THICK, fill=FILL)
    for sx in (-1, 1):
        d.rect(*b(sx * bx - p["cup_d"] / 2, -unit), *b(sx * bx + p["cup_d"] / 2, p["cup_z"]), INK, MEDIUM,
               fill="#d9d9d9")
    for m in mounts:
        if abs(m[0]) > pw / 2:
            d.rect(*b(m[0] - lug_r, z_back + p["lug_t"]), *b(m[0] + lug_r, z_back), INK, MEDIUM, fill="#d9d9d9")
    d.rect(*b(-pw / 2, -unit), *b(pw / 2, z_back), INK, THICK, fill=FILL)
    zp = -unit - p["port_z"]
    for x, dia, name in ((p["gland_x"], p["gland_d"], "gland"), (p["vent_x"], p["vent_d"], "vent")):
        c = b(x, zp)
        d.circle(c, dia / 2 * S, INK, MEDIUM, fill="#ffffff")
        d.text(c[0] + 1, c[1] + dia / 2 * S + 3.0, f"{name} Ø{fmt(dia)}", 2.2, INK)
    base = b(0, z_back)[1]
    vdim(d, b(-hw, z_back), b(-hw, 0), b(-hw, 0)[0] - 7, fmt(-z_back), color(z_back))
    vdim(d, b(hw, -unit), b(hw, 0), b(hw, 0)[0] + 6, fmt(unit), color(unit))
    hdim(d, b(-pw / 2, z_back), b(pw / 2, z_back), base + 6, fmt(pw), color(pw))
    hdim(d, b(p["gland_x"], zp), b(p["vent_x"], zp), base + 12, fmt(p["vent_x"] - p["gland_x"]))
    vdim(d, b(pw / 2, -unit), b(pw / 2, zp), b(pw / 2, 0)[0] + 6, fmt(p["port_z"]))
    label(d, b(0, 0)[0], base + 21, "BOTTOM", "seen from below, rider side at the top")

    scale_bar(d, 22, 182, S)
    notes(d, 150, 112, [
        (f"Display unit {fmt(2 * hw)} × {fmt(top - bot)} × {fmt(unit)} mm: bezel and cover, M3 screws "
         "from the back. The pod behind it holds the perfboard; the Pi Zero 2 W and the Kippah sit on "
         "the cover, inside the pod.", INK),
        ("Bike mount: 3 M5 heat-set inserts in the lugs of the pod. Two 12 mm IP67 buttons in the "
         "wings, their bodies in the cups of the cover.", INK),
        ("Orange values are [TO VERIFY] before printing (chapter 07): display thickness, board heights "
         "that set the pod depth, button bodies, mount points.", VERIFY),
    ], width=130, size=2.4)
    return d
