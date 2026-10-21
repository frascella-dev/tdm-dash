#!/usr/bin/env python3
"""Draws the zone plan of the 90 x 70 mm perfboard (layout.svg).

The board is split into one zone per schematic sheet, each listing the parts
that go in it, on the real 2.54 mm hole grid (30 x 24 holes). It is a
placement plan, not a wiring diagram: the connections are those of the KiCad
schematic (hardware/exports/kicad/tdm-dash.pdf).

    python3 hardware/perfboard/layout.py
"""

from __future__ import annotations

from pathlib import Path
from xml.sax.saxutils import escape

COLS, ROWS = 30, 24
PITCH = 2.54
SCALE = 9.0                       # px per mm
M = 40                            # margin, px
OUT = Path(__file__).with_name("layout.svg")

# (title, col0, row0, col1, row1, colour, parts) — columns and rows inclusive,
# seen from the component side, connectors on the left edge.
ZONES = [
    ("Bike connectors", 0, 0, 1, 15, "#0f1113",
     ["J101 6p: +12V_KEY, GND, neutral, high beam, turn signals", "J102 4p: tachometer, coolant, lights, GND", "J401 3p: wheel sensor"]),
    ("Power (sheet 1)", 2, 0, 13, 7, "#c98a14",
     ["Q101 IRF4905 (TO-220 lying flat) · R101 10k · D101 12 V", "D102 SMCJ24A · L101 22 µH · C101 100 µF · C102 1 µF", "U101 Pololu D36V28F5 in a socket", "D103 1N5819 · C103 470 µF"]),
    ("Tachometer (sheet 3)", 14, 0, 23, 7, "#d9410b",
     ["D301 1N4007 · R301, R302 22k 1 W (raised off the board)", "D302 1N4744A · C301 2.2 nF · R303 2.2k", "U301 H11L1 in a socket · R304 4.7k · C302 100 nF"]),
    ("FRAM and 5 V output (sheet 6)", 24, 0, 29, 7, "#575b5d",
     ["U602 FRAM breakout on an 8-pin header", "J103 2p: 5 V to the Pi"]),
    ("Warning lights, 5 channels (sheet 2)", 2, 8, 15, 15, "#2bb655",
     ["per channel: EL817 in a socket, 4.7k (neutral 3.3k),", "10k, 1N4148, 100 nF · outputs to GP6–GP10"]),
    ("Wheel sensor (sheet 4)", 16, 8, 21, 15, "#3b6ff0",
     ["U401 EL817 · R401 2.2k · R402 4.7k", "D401 1N4148 · C401 100 nF"]),
    ("Analog inputs (sheet 5)", 22, 8, 29, 15, "#7a4fd6",
     ["R501 100k · R502 22k · R503 100 Ω · R504, R505 1k", "C501–C503 100 nF", "D501–D503 BAT54S on SOT-23 adapters"]),
    ("Buttons, light, backlight (sheets 6–7)", 0, 16, 7, 23, "#8b877e",
     ["J601 buttons · J501 TEMT6000 · J701 to the TPS61169", "R603, R604 1k · C602, C603 100 nF · R701 1k · R702 100k"]),
    ("Raspberry Pi Pico on 2×20 headers (sheet 6)", 8, 16, 28, 23, "#0f1113",
     ["USB towards the edge; twisted pair soldered to the TP2/TP3", "pads under the Pico (chapter 05)"]),
]


def hole(c: int, r: int) -> tuple[float, float]:
    return M + (c + 0.5) * PITCH * SCALE, M + (r + 0.5) * PITCH * SCALE


def main() -> int:
    w_mm, h_mm = COLS * PITCH, ROWS * PITCH
    board_w = w_mm * SCALE + 2 * M
    legend_x = board_w + 10
    W = int(legend_x + 450)
    H = int(max(h_mm * SCALE + 2 * M + 30, 60 + sum(28 + 17 * len(z[6]) for z in ZONES)))
    s = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}" '
        'font-family="DejaVu Sans, Helvetica, Arial, sans-serif">',
        "<title>tdm-dash · 90 × 70 mm perfboard layout</title>",
        f'<rect width="{W}" height="{H}" fill="#ffffff"/>',
        f'<rect x="{M - 6}" y="{M - 6}" width="{w_mm * SCALE + 12:.0f}" height="{h_mm * SCALE + 12:.0f}" rx="8" '
        'fill="#e7d9b0" stroke="#8b7a4a" stroke-width="2"/>',
    ]
    for c in range(COLS):
        for r in range(ROWS):
            x, y = hole(c, r)
            s.append(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="3.2" fill="#b98d3e"/>'
                     f'<circle cx="{x:.1f}" cy="{y:.1f}" r="1.4" fill="#5b4a2a"/>')
    ly = M
    for n, (title, c0, r0, c1, r1, colour, parts) in enumerate(ZONES, start=1):
        x0, y0 = hole(c0, r0)
        x1, y1 = hole(c1, r1)
        pad = PITCH * SCALE * 0.45
        x0, y0, x1, y1 = x0 - pad, y0 - pad, x1 + pad, y1 + pad
        s.append(f'<rect x="{x0:.1f}" y="{y0:.1f}" width="{x1 - x0:.1f}" height="{y1 - y0:.1f}" rx="6" '
                 f'fill="{colour}" fill-opacity="0.16" stroke="{colour}" stroke-width="2.5"/>')
        cx, cy = (x0 + x1) / 2, (y0 + y1) / 2
        s.append(f'<circle cx="{cx:.1f}" cy="{cy:.1f}" r="15" fill="{colour}"/>'
                 f'<text x="{cx:.1f}" y="{cy + 6:.1f}" font-size="17" font-weight="bold" fill="#fff" '
                 f'text-anchor="middle">{n}</text>')
        # legend
        s.append(f'<circle cx="{legend_x + 14}" cy="{ly + 2}" r="12" fill="{colour}"/>'
                 f'<text x="{legend_x + 14}" y="{ly + 7}" font-size="14" font-weight="bold" fill="#fff" '
                 f'text-anchor="middle">{n}</text>'
                 f'<text x="{legend_x + 34}" y="{ly + 7}" font-size="15" font-weight="bold" fill="{colour}">'
                 f'{escape(title)}</text>')
        for i, line in enumerate(parts):
            s.append(f'<text x="{legend_x + 34}" y="{ly + 26 + i * 17}" font-size="13" fill="#1d1f21">{escape(line)}</text>')
        ly += 28 + 17 * len(parts)
    s.append(f'<text x="{M}" y="{h_mm * SCALE + 2 * M + 16:.0f}" font-size="13" fill="#575b5d">Component side · '
             f'2.54 mm pitch · {COLS} × {ROWS} holes · the zones follow the KiCad schematic sheets</text>')
    s.append("</svg>")
    OUT.write_text("\n".join(s) + "\n", encoding="utf-8")
    print(f"{OUT.name}: {W} × {H} px")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
