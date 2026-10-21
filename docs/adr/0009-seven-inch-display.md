# 0009 · 7" 800×480 1000 nit DPI display

**Status:** accepted · October 2026. Replaces the choice of the 4.3" Crystalfontz CFAF800480H0-043SN made during planning. [ADR 0013](0013-other-displays-and-bars.md) extends the software to other panels, bars included; this one stays the reference display.

## Context

The 4.3" was judged too small for the bike. The options evaluated:

| Option | Brightness | Connection | Notes |
|---|---|---|---|
| Stretched bars 7.8–8.8" (Waveshare 7.9" 400×1280, Topway 8.8" 480×1920) | 550–600 nit | HDMI | No bar that private buyers can purchase exceeds 600 nit; they are natively portrait (rotation in software); up to 2.4 times the pixels of 800×480; the 8.8" is 252 mm wide |
| 7.84" 850 nit bars | 850 nit | MIPI DSI | Quote only, they need a board with DSI and a panel driver written by us |
| 7" 1024×600 (Newhaven NHD-7.0-HDMI-HR-RSXP) | 1100 nit | HDMI | ≈ 100–160 €, 1.6 times the pixels, mini-HDMI to be secured |
| **7" 800×480 (Newhaven NHD-7.0-800480AF-ASXP)** | **1000 nit** | **DPI** | ≈ 78 €, same pixels as the initial design |

## Decision

**Newhaven NHD-7.0-800480AF-ASXP**, connected to the Pi Zero 2 W over DPI through the Adafruit DPI TFT Kippah. Data from the [datasheet](https://newhavendisplay.com/content/specs/NHD-7.0-800480AF-ASXP.pdf):

| Fact | Value |
|---|---|
| Panel | 7" IPS, 800×480, normally black, transmissive, anti-glare |
| Brightness | 1000 cd/m² typical, 800 minimum, at 180 mA |
| Active area / outline | 154.2 × 85.6 mm / 165 × 104 mm, 133 ppi |
| Operating temperature | −20 … +70 °C |
| Interface | 24-bit parallel RGB, 40-pin 0.5 mm pitch FFC (Molex 54104-4031), Sitronix ST7277 driver, DE/SYNC/SYNC-DE mode |
| Pinout | 1 LED-K, 2 LED-A, 3 GND, 4 VDD 3.3 V, 5–12 R0–R7, 13–20 G0–G7, 21–28 B0–B7, 29 GND, 30 DCLK, 31 DISP, 32 HSYNC, 33 VSYNC, 34 DE, 35 BIST_EN, 36 GND, 37–40 NC |
| Logic supply | 3.3 V, 55 mA typical |
| Backlight | 180 mA (135–225) at 9.3 V (8.3–10.2), **no on-board driver** |
| Timing | DCLK 23–27 MHz (25 typical); horizontal 800 + 4/8/8 (typical pulse/back/front porch); vertical 480 + 4/16/16 |

The pinout is the standard 40-pin one for RGB panels, the same used by the Kippah. The Pi drives the panel at 18 bits (RGB666): the two low bits of each color are tied by the Kippah.

**Backlight:** TPS61169 PWM-driven by the Pico (ADR 0006), with a sense resistor of ≈ 1.1–1.2 Ω for about 170–180 mA. `DISP` must be held high and `BIST_EN` low.

## Consequences

- **Software and design unchanged:** the resolution stays 800×480, so the software renderer load does not change and the design artifact stays valid. Only the physical size changes: everything is 1.6 times larger than on the 4.3".
- **Higher power draw:** ≈ 2 W for the display instead of ≈ 0.8 W. The 3.2 A converter is enough.
- **Enclosure:** larger, at least 175 × 115 mm at the front with the bezel. The space behind the fairing must be checked during the bike measurements.
- **Cost** of the display: from ≈ 40 € (shipping included) to ≈ 78 €. Estimated total ≈ 190–240 €.
- **To verify:**
  - whether the Kippah backlight driver handles 180 mA
  - whether the resistor of the TPS61169 breakout can be replaced
- **Linux configuration:** `vc4-kms-dpi-generic` with a 25 MHz clock and the timings above (milestone M4).
