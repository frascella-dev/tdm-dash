"""Enclosure at 1:2: front, side, bottom and back, with the main dimensions.

The parts are printed from the STL files; this sheet is for checking them and
for fitting the enclosure to the bike. Every value comes from
hardware/enclosure/enclosure.scad.
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

    W, H = p["outer_w"], p["outer_h"]
    w2, h2, r = W / 2, H / 2, p["corner_r"]
    bez, depth = p["bezel_d"], p["depth"]
    back = -(bez + depth)
    screws = [(sx * (p["inner_w"] / 2 + p["rim"] / 2), sy * (p["inner_h"] / 2 + p["rim"] / 2))
              for sx in (-1, 1) for sy in (-1, 1)]
    screws += [(0, p["inner_h"] / 2 + p["rim"] / 2), (0, -(p["inner_h"] / 2 + p["rim"] / 2))]

    # ------------------------------------------------------------ front
    f = View(76, 60, S)
    d.rounded_rect(*f(-w2, h2), *f(w2, -h2), r * S, INK, THICK, fill=FILL)
    ax, ay = p["aa_dx"], p["aa_dy"]
    inner = ((p["aa_w"] + 1) / 2, (p["aa_h"] + 1) / 2)                  # back of the front plate
    outer = (inner[0] + p["front_t"], inner[1] + p["front_t"])          # 45° chamfer on the face
    d.rect(*f(ax - outer[0], ay + outer[1]), *f(ax + outer[0], ay - outer[1]), INK, THICK, fill="#ffffff")
    d.rect(*f(ax - inner[0], ay + inner[1]), *f(ax + inner[0], ay - inner[1]), INK, MEDIUM)
    for sx in (-1, 1):
        for sy in (-1, 1):
            d.line(f(ax + sx * inner[0], ay + sy * inner[1]), f(ax + sx * outer[0], ay + sy * outer[1]), INK, MEDIUM)
    light = (W / 4, -(p["inner_h"] / 2 + p["rim"] / 2))
    d.circle(f(*light), p["light_d"] / 2 * S, INK, MEDIUM, fill="#ffffff")
    d.line(f(-w2 - 4, 0), f(w2 + 4, 0), GREY, THIN, CENTER)
    d.line(f(0, -h2 - 4), f(0, h2 + 4), GREY, THIN, CENTER)
    hdim(d, f(-w2, h2), f(w2, h2), f(0, h2)[1] - 7, fmt(W))
    vdim(d, f(-w2, -h2), f(-w2, h2), f(-w2, 0)[0] - 7, fmt(H))
    hdim(d, f(ax - outer[0], ay + outer[1]), f(ax + outer[0], ay + outer[1]), f(0, ay + outer[1])[1] + 7,
         f"window {fmt(2 * outer[0])}", text_at=0.3)
    vdim(d, f(ax + outer[0], ay - outer[1]), f(ax + outer[0], ay + outer[1]), f(ax + outer[0], 0)[0] - 8,
         fmt(2 * outer[1]), text_at=0.3)
    diameter(d, f(*light), p["light_d"] / 2 * S, 60, f"Ø{fmt(p['light_d'])} light pipe", reach=5)
    label(d, f(0, 0)[0], f(0, -h2)[1] + 8, "FRONT", "rider side, visor not shown")

    # ------------------------------------------------------------ right side
    sv = View(150, 60, S, sx=-1)              # part (z, y): front face on the left, z grows leftwards
    vt, vl = p["visor_t"], p["visor_len"]
    d.rect(*sv(-bez, h2), *sv(back, -h2), INK, THICK, fill=FILL)
    d.rect(*sv(0, h2), *sv(-bez, -h2), INK, THICK, fill="#d6d6d6")
    d.rect(*sv(back + 6, h2 + vt), *sv(vl, h2), INK, THICK, fill="#d6d6d6")
    cheek_low = h2 * (1 - 2 * p["visor_cheek"])
    d.polyline([sv(0, h2), sv(vl, h2), sv(0, cheek_low)], INK, THICK, closed=True, fill="#d6d6d6")
    hdim(d, sv(0, h2 + vt), sv(vl, h2 + vt), sv(0, h2 + vt)[1] - 6, fmt(vl))
    hdim(d, sv(0, -h2), sv(-bez, -h2), sv(0, -h2)[1] + 4, fmt(bez), color(bez))
    hdim(d, sv(0, -h2), sv(back, -h2), sv(0, -h2)[1] + 10, fmt(bez + depth), color(bez))
    label(d, sv(back / 2, 0)[0], sv(0, -h2)[1] + 18, "RIGHT SIDE", "bezel, shell and visor")

    # ------------------------------------------------------------ back
    k = View(236, 60, S, sx=-1)               # seen from behind: the part's x runs leftwards
    d.rounded_rect(*k(-w2, h2), *k(w2, -h2), r * S, INK, THICK, fill=FILL)
    d.line(k(-w2 - 4, 0), k(w2 + 4, 0), GREY, THIN, CENTER)
    d.line(k(0, -h2 - 4), k(0, h2 + 4), GREY, THIN, CENTER)
    for c in screws:
        d.circle(k(*c), 3.0 * S, INK, MEDIUM, fill="#ffffff")
        d.circle(k(*c), 1.7 * S, INK, THIN)
    mounts = p["mount_points"]
    mc = color(*[v for m in mounts for v in m])
    for m in mounts:
        d.circle(k(*m), p["mount_insert_d"] / 2 * S, mc, MEDIUM, fill="#ffffff")
        if m[0]:
            d.line(k(m[0], m[1] - 5), k(m[0], m[1] + 5), GREY, THIN, CENTER)
        if m[1]:
            d.line(k(m[0] - 5, m[1]), k(m[0] + 5, m[1]), GREY, THIN, CENTER)
    top = sorted((m for m in mounts if m[1] > 0), key=lambda m: m[0])
    low = [m for m in mounts if m[1] < 0]
    if len(top) == 2:
        hdim(d, k(*top[1]), k(*top[0]), k(0, top[0][1])[1] + 7, fmt(top[1][0] - top[0][0]), mc, text_at=0.3)
    if len(top) == 2 and low:
        vdim(d, k(*low[0]), k(*top[1]), k(*top[1])[0] - 6, fmt(top[1][1] - low[0][1]), mc, text_at=0.3)
    label(d, k(0, 0)[0], k(0, -h2)[1] + 8, "BACK",
          f"bike side · 3× M5 inserts Ø{fmt(p['mount_insert_d'])} · 6× M3, Ø6 counterbore")

    # ------------------------------------------------------------ bottom
    b = View(76, 113, S)                      # part (x, z): front face at the top
    d.rect(*b(-w2, 0), *b(w2, back), INK, THICK, fill=FILL)
    d.line(b(-w2, -bez), b(w2, -bez), INK, MEDIUM)
    z_mid = back + depth / 2
    holes = [(-w2 + 30, z_mid, p["gland_d"], "gland"), (-w2 + 55, z_mid, p["vent_d"], "vent")]
    holes += [(w2 - 30 - i * p["button_pitch"], z_mid + 4, p["button_d"], "button") for i in (1, 0)]
    for x, z, dia, name in holes:
        c = b(x, z)
        d.circle(c, dia / 2 * S, INK, MEDIUM, fill="#ffffff")
        d.line(b(x, z - dia / 2 - 2), b(x, z + dia / 2 + 2), GREY, THIN, CENTER)
        d.text(c[0], c[1] - dia / 2 * S - 1.4, name, 2.2, NOTE, "middle")
        d.text(c[0] + 0.8, b(0, back)[1] - 1.5, f"Ø{fmt(dia)}", 2.4, INK)
    left, right = b(-w2, back), b(w2, back)
    base = b(0, back)[1]
    hdim(d, left, b(holes[0][0], holes[0][1]), base + 6, fmt(holes[0][0] + w2))
    hdim(d, left, b(holes[1][0], holes[1][1]), base + 12, fmt(holes[1][0] + w2))
    hdim(d, b(holes[3][0], holes[3][1]), right, base + 6, fmt(w2 - holes[3][0]))
    hdim(d, b(holes[2][0], holes[2][1]), right, base + 12, fmt(w2 - holes[2][0]))
    vdim(d, b(-w2, back), b(-w2, z_mid), b(-w2, 0)[0] - 6, fmt(z_mid - back))
    vdim(d, b(w2, back), b(w2, z_mid + 4), b(w2, 0)[0] + 6, fmt(z_mid + 4 - back))
    label(d, b(0, 0)[0], base + 20, "BOTTOM", "seen from below, rider side at the top")

    scale_bar(d, 22, 182, S)
    notes(d, 142, 116, [
        ("Dimensions in mm. The parts are printed from the STL files in hardware/exports/cad/: "
         "this sheet is for checking them and for fitting the bike mount.", INK),
        (f"Bike mount: 3 M5 heat-set inserts in {fmt(p['mount_insert_d'])} mm holes, 11 mm deep. "
         "Closing: 6 M3 × 16 screws from the back into inserts in the bezel, 1 mm EPDM gasket.", INK),
        ("Orange values are [TO VERIFY] before printing (chapter 07): the display thickness, "
         "which sets the bezel depth, and the mount points on the bike.", VERIFY),
    ], width=138, size=2.4)
    return d
