"""Zone plan of the 90 x 70 mm perfboard, at 2:1.

The board is split into one zone per schematic sheet, each listing the parts
that go in it, on the real 2.54 mm hole grid (30 x 24 holes, columns 1-30 and
rows A-X). It is a placement plan, not a wiring diagram: the connections are
those of the schematic (hardware/exports/kicad/tdm-dash.pdf). The board size
and its mounting holes come from hardware/enclosure/enclosure.scad.
"""

from __future__ import annotations

from pathlib import Path

from sheet import (GREY, INK, NOTE, THICK, VERIFY, Drawing, View, fmt, frame, scad_params, scale_bar, text_width, wrap)

SCAD = "hardware/enclosure/enclosure.scad"
COLS, ROWS = 30, 24
PITCH = 2.54
BOARD = (90.0, 70.0)
S = 2.0  # 2:1

# (title, col0, row0, col1, row1, color, parts): columns and rows from 0,
# inclusive, seen from the component side with the harness connectors on the
# left edge
ZONES = [
    ("Bike connectors", 0, 0, 1, 15, "#3a3f44",
     ["J101 (6 pins), J102 (4 pins): adapter harness", "J401 (3 pins): wheel sensor cable"]),
    ("Power", 2, 0, 13, 7, "#c98a14",
     ["Q101 IRF4905 lying flat · R101 10k · D101 12 V", "D102 SMCJ24A · L101 22 µH · C101 100 µF · C102 1 µF",
      "U101 Pololu D36V28F5 in a socket · D103 1N5819 · C103 470 µF"]),
    ("Tachometer", 14, 0, 23, 7, "#d9410b",
     ["D301 1N4007 · R301, R302 22k 1 W, raised off the board", "D302 1N4744A · C301 2.2 nF · R303 2.2k",
      "U301 H11L1 in a socket · R304 4.7k · C302 100 nF"]),
    ("FRAM and 5 V out", 24, 0, 29, 7, "#575b5d",
     ["U602 FRAM breakout on an 8-pin header", "R601, R602 2.2k · C601 100 nF", "J103 (2 pins): 5 V to the Pi"]),
    ("Warning lights", 2, 8, 15, 15, "#2b9a4d",
     ["5 channels, each: EL817 in a socket, 10k, 1N4148,", "100 nF and 4.7k (3.3k for neutral); outputs to GP6–GP10"]),
    ("Wheel sensor", 16, 8, 21, 15, "#3b6ff0",
     ["U401 EL817 · R401 2.2k · R402 4.7k · D401 1N4148 · C401 100 nF"]),
    ("Analog inputs", 22, 8, 29, 15, "#7a4fd6",
     ["R501 100k · R502 22k · R503 100 Ω · R504, R505 1k", "C501–C503 100 nF · D501–D503 BAT54S on SOT-23 adapters"]),
    ("Buttons, light, backlight", 0, 16, 7, 23, "#8b6f4e",
     ["J601 buttons · J501 TEMT6000 · J701 to the TPS61169 breakout",
      "R603, R604 1k · C602, C603 100 nF · R701 1k · R702 100k"]),
    ("Raspberry Pi Pico", 8, 16, 28, 23, "#1d6f8a",
     ["U601 on 2 × 20 headers: columns 10–29, rows Q and X", "USB pair on the TP2/TP3 pads under the Pico (J702)"]),
]
SHEETS = ["1, 4", "1", "3", "1, 6", "2", "4", "5", "5–7", "6"]
OFF_BOARD = ("F101 inline fuse (adapter harness) · U701 TPS61169 next to the Kippah · "
             "SW601, SW602 and the TEMT6000 on the enclosure · sheet 8, backup LEDs: on the panel (chapter 06)")


def tint(color: str, k: float = 0.16) -> str:
    """The color mixed with white, for zone fills (the PDF has no transparency)."""
    h = color.lstrip("#")
    rgb = [int(h[i:i + 2], 16) for i in (0, 2, 4)]
    return "#" + "".join(f"{round(255 - (255 - c) * k):02x}" for c in rgb)


def build(root: Path, rev: str, date: str) -> Drawing:
    p = scad_params(root / SCAD)
    d = Drawing("tdm-dash · perfboard layout, 2:1")
    frame(d, "Perfboard layout", "sensor module · Yamaha TDM 850 (3VD)", "2:1", rev, date,
          "hardware/drawings/perfboard.py")
    bw, bh = BOARD
    v = View(26 + bw / 2 * S, 22 + bh / 2 * S, S)      # board center; part y up

    def hole(c: int, r: int):
        """Center of a hole, part coordinates from the board center."""
        return (-(COLS - 1) / 2 * PITCH + c * PITCH, (ROWS - 1) / 2 * PITCH - r * PITCH)

    def box(c0, r0, c1, r1, pad=0.45):
        x0, y0 = v(*hole(c0, r0))
        x1, y1 = v(*hole(c1, r1))
        e = PITCH * S * pad
        return x0 - e, y0 - e, x1 + e, y1 + e

    d.rect(*v(-bw / 2, bh / 2), *v(bw / 2, -bh / 2), INK, THICK, fill="#f4ecd6")
    for _, c0, r0, c1, r1, color, _ in ZONES:
        d.rect(*box(c0, r0, c1, r1), color, 0.01, fill=tint(color))
    for c in range(COLS):
        for r in range(ROWS):
            d.circle(v(*hole(c, r)), 0.85 * S, fill="#d9b66e", stroke=False)
            d.circle(v(*hole(c, r)), 0.4 * S, fill="#ffffff", stroke=False)
    # coordinates: columns 1-30 above, rows A-X on the left
    for c in range(COLS):
        x = v(*hole(c, 0))[0]
        d.text(x, v(0, bh / 2)[1] - 1.2, str(c + 1), 1.8, GREY, "middle")
    for r in range(ROWS):
        y = v(*hole(0, r))[1]
        d.text(v(-bw / 2, 0)[0] - 1.2, y, "ABCDEFGHIJKLMNOPQRSTUVWX"[r], 1.8, GREY, "end", middle=True)
    # mounting holes, as in the enclosure
    mc = VERIFY if getattr(p["perf_holes"][0][0], "verify", False) else INK
    for mx, my in p["perf_holes"]:
        d.circle(v(mx, my), 1.6 * S, mc, THICK, fill="#ffffff")

    # the Pico: its 2 x 20 header pins on rows Q and X, micro-USB at the right end
    _, pc0, pr0, pc1, pr1, pico, _ = ZONES[-1]
    for c in range(pc1 - 19, pc1 + 1):
        for r in (pr0, pr1):
            x, y = v(*hole(c, r))
            d.rect(x - 1.6, y - 1.6, x + 1.6, y + 1.6, pico, 0.35)
    cx = (hole(pc1 - 19, 0)[0] + hole(pc1, 0)[0]) / 2
    cy = (hole(0, pr0)[1] + hole(0, pr1)[1]) / 2
    d.rect(*v(cx + 25.5 - 5.5, cy + 4), *v(cx + 25.5 + 1.3, cy - 4), pico, 0.35, fill="#ffffff")
    d.text(v(cx + 25.5 - 2.1, cy)[0], v(cx, cy)[1], "USB", 2.2, pico, "middle", bold=True, angle=-90,
           middle=True)

    # zone numbers and names
    for n, (title, c0, r0, c1, r1, color, _) in enumerate(ZONES, start=1):
        x0, y0, x1, y1 = box(c0, r0, c1, r1)
        d.rect(x0, y0, x1, y1, color, 0.5)
        bx, by = x0 + 3.6, y0 + 3.6
        d.circle((bx, by), 2.6, fill=color, stroke=False)
        d.text(bx, by, str(n), 2.8, "#ffffff", "middle", bold=True, middle=True)
        if text_width(title, 2.6, True) + 9 < x1 - x0:
            d.rect(bx + 3.2, by - 2.0, bx + 4.6 + text_width(title, 2.6, True), by + 2.0, color, 0.01,
                   fill="#ffffff")
            d.text(bx + 3.9, by, title, 2.6, color, bold=True, middle=True)
        else:
            d.rect(bx - 2.0, by + 3.2, bx + 2.0, by + 4.6 + text_width(title, 2.6, True), color, 0.01,
                   fill="#ffffff")
            d.text(bx, by + 3.9, title, 2.6, color, bold=True, angle=90, middle=True)

    # legend
    lx, ly, lw = 213, 20, 68
    d.text(lx, ly + 2.4, f"Perfboard {fmt(bw)} × {fmt(bh)} mm, {COLS} × {ROWS} holes", 2.6, INK, bold=True)
    ly = d.lines(lx, ly + 4.0, wrap("Seen from the component side: harness connectors on the left edge, "
                                   "Pico USB at the right edge. One zone per schematic sheet.", lw, 2.1),
                 2.1, NOTE, 1.3) + 3.0
    letters = "ABCDEFGHIJKLMNOPQRSTUVWX"
    for n, ((title, c0, r0, c1, r1, color, parts), sheets) in enumerate(zip(ZONES, SHEETS), start=1):
        d.circle((lx + 2.2, ly + 1.0), 2.1, fill=color, stroke=False)
        d.text(lx + 2.2, ly + 1.0, str(n), 2.3, "#ffffff", "middle", bold=True, middle=True)
        d.text(lx + 5.6, ly + 1.0, title, 2.4, color, bold=True, middle=True)
        where = f"sheet {sheets} · {c0 + 1}–{c1 + 1} / {letters[r0]}–{letters[r1]}"
        d.text(lx + lw, ly + 1.0, where, 1.9, GREY, "end", middle=True)
        rows = [row for part in parts for row in wrap(part, lw - 5.6, 2.0)]
        ly = d.lines(lx + 5.6, ly + 3.4, rows, 2.0, INK, 1.3) + 3.0

    # what belongs to the module but is not soldered on the board
    nx, ny = 92, 170
    d.text(nx, ny + 2.0, "Not on this board", 2.4, INK, bold=True)
    ny = d.lines(nx, ny + 4.0, wrap(OFF_BOARD, 66, 2.0), 2.0, INK, 1.3)
    xs = sorted({float(h[0]) for h in p["perf_holes"]})
    ys = sorted({float(h[1]) for h in p["perf_holes"]})
    d.lines(nx, ny + 1.6, wrap(f"Mounting holes {fmt(xs[-1] - xs[0])} × {fmt(ys[-1] - ys[0])} mm, "
                               "from enclosure.scad" + (": [TO VERIFY] on the board in hand." if mc == VERIFY
                                                       else "."), 66, 2.0), 2.0, mc, 1.3)

    scale_bar(d, 22, 177, S)
    return d
