#!/usr/bin/env python3
"""Builds docs/assets/banner.svg, the banner at the top of the README.

Text is converted to outlines with the dashboard font (Saira SemiCondensed,
OFL, in crates/tdm-dash/fonts), so the SVG looks the same everywhere and
needs no web font. The bike is docs/assets/tdm-850.svg (see
tools/stl_outline.py). Colors are the day theme of design/README.md.

    python3 tools/make_banner.py        # needs fonttools
"""

from __future__ import annotations

import math
import re
from pathlib import Path

from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.ttLib import TTFont

ROOT = Path(__file__).resolve().parent.parent
FONTS = ROOT / "crates" / "tdm-dash" / "fonts"
BIKE = ROOT / "docs" / "assets" / "tdm-850.svg"
OUT = ROOT / "docs" / "assets" / "banner.svg"

W, H = 1280, 360
BG, PANEL, INK, DIM, FAINT, NEEDLE = "#eceae4", "#dcd9d1", "#0f1113", "#575b5d", "#bdb9b0", "#d9410b"


def text_path(font_file: str, text: str, size: float, x: float, y: float) -> tuple[str, float]:
    """SVG path data for `text` with its baseline at (x, y); returns (d, width)."""
    font = TTFont(FONTS / font_file)
    glyphs = font.getGlyphSet()
    cmap = font.getBestCmap()
    scale = size / font["head"].unitsPerEm
    pen = SVGPathPen(glyphs)
    cursor = 0.0
    for ch in text:
        name = cmap[ord(ch)]
        glyphs[name].draw(TransformPen(pen, (scale, 0, 0, -scale, x + cursor, y)))
        cursor += glyphs[name].width * scale
    return pen.getCommands(), cursor


def bike(x: float, y: float, height: float) -> tuple[str, float]:
    svg = BIKE.read_text(encoding="utf-8")
    vw, vh = map(float, re.search(r'viewBox="0 0 ([\d.]+) ([\d.]+)"', svg).groups())
    d = re.search(r' d="([^"]+)"', svg).group(1)
    s = height / vh
    g = f'<path transform="translate({x:.1f} {y:.1f}) scale({s:.4f})" fill="{INK}" fill-rule="evenodd" d="{d}"/>'
    return g, vw * s


def gauge(cx: float, cy: float, r: float) -> str:
    """Faint speedometer ticks behind the bike, as on the Classic layout."""
    out = []
    for i in range(0, 41):
        a = math.radians(210 - i * 6)
        major = i % 5 == 0
        r0 = r - (22 if major else 12)
        x0, y0 = cx + r0 * math.cos(a), cy - r0 * math.sin(a)
        x1, y1 = cx + r * math.cos(a), cy - r * math.sin(a)
        out.append(f'<line x1="{x0:.1f}" y1="{y0:.1f}" x2="{x1:.1f}" y2="{y1:.1f}" stroke-width="{4 if major else 2}"/>')
    arc = (
        f'<path d="M{cx + r * math.cos(math.radians(210)):.1f} {cy - r * math.sin(math.radians(210)):.1f} '
        f'A{r} {r} 0 1 1 {cx + r * math.cos(math.radians(-30)):.1f} {cy - r * math.sin(math.radians(-30)):.1f}" '
        f'fill="none" stroke-width="2"/>'
    )
    return f'<g stroke="{FAINT}" stroke-linecap="round">{arc}{"".join(out)}</g>'


def main() -> int:
    left = 72
    title, tw = text_path("SairaSemiCondensed-Bold.ttf", "tdm-dash", 132, left, 168)
    sub, sw = text_path("SairaSemiCondensed-SemiBold.ttf", "Open-source digital instrument cluster", 40, left, 252)
    tags, gw = text_path(
        "SairaSemiCondensed-Medium.ttf", "Raspberry Pi Zero 2 W · RP2040 · Rust · Slint · Buildroot", 25, left, 300
    )
    bike_h = 290
    _, bw = bike(0, 0, bike_h)
    bx = W - 56 - bw
    bike_svg, _ = bike(bx, 36, bike_h)
    widest = left + max(tw, sw, gw)
    if widest > bx - 24:
        raise SystemExit(f"text ({widest:.0f}) runs into the bike ({bx:.0f})")
    svg = f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}" role="img" aria-labelledby="t">
<title id="t">tdm-dash · open-source digital instrument cluster</title>
<rect width="{W}" height="{H}" rx="18" fill="{BG}"/>
{gauge(bx + bw * 0.52, 214, 168)}
<path fill="{INK}" d="{title}"/>
<rect x="{left}" y="190" width="132" height="8" rx="4" fill="{NEEDLE}"/>
<path fill="{INK}" d="{sub}"/>
<path fill="{DIM}" d="{tags}"/>
{bike_svg}
</svg>
"""
    OUT.write_text(svg, encoding="utf-8")
    print(f"{OUT.relative_to(ROOT)}: {len(svg) // 1024} KiB, text up to x={widest:.0f}, bike from x={bx:.0f}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
