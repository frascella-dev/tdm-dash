# 0013 · Interface for other panel sizes and elongated bar displays

**Status:** accepted · October 2026. Extends [ADR 0009](0009-seven-inch-display.md): the Newhaven 7" 800×480 stays the reference display, but it is no longer the only one the software can drive.

## Context

Until now the whole interface was drawn for 800×480:
- `ui/dash.slint` had a fixed 800×480 window with absolute coordinates.
- The dial faces were 340 pixels.
- The glyphs were embedded as bitmaps at 20 fixed sizes. The software renderer takes the largest embedded size not above the requested one, so on a bigger panel the text would not have grown.

ADR 0009 rejected the elongated bars (1280×400, 1920×480, natively portrait 400×1280 panels) for the bike because none is bright enough. They remain attractive for other bikes and other fairings, and so do the common 1024×600 and 1280×720 panels. The interface must work on all of them without a copy of every screen per resolution.

## Decision

**Two layout families**, chosen from the aspect ratio of the panel after the rotation (`crates/tdm-dash/src/screen.rs`):

| Family | Aspect ratio | Smallest logical canvas | Examples (panel → logical canvas) |
|---|---|---|---|
| **wide** | below 2.2:1 | 800×480 | 800×480 → 800×480 · 1024×600 → 819×480 · 1280×720 → 853×480 · 1024×768 → 800×600 |
| **bar** | 2.2:1 and above | 1200×400 | 1280×400 → 1280×400 · 1920×480 → 1600×400 · 1920×720 → 1200×450 · 400×1280 at 90° → 1280×400 |

- **The scale is the largest that keeps the canvas at least that size:**
  - wide: `min(width/800, height/480)`
  - bar: `min(width/1200, height/400)`
  - The scale is given to Slint as the window scale factor, so `dash.slint` keeps working in logical pixels.
- **The frame stretches with the window:** the telltale row, the information strip, the warning banners and the trip, diagnostics and settings screens.
- **The wide layouts** stay on the 800×480 canvas of the design mock-up, centred. At 800×480 the picture is the same as before, pixel for pixel.
- **The bar layouts** are new versions of Classic, Sport and Touring for the band between the telltales and the strip:
  - **Classic:** two 300-pixel dials with the coolant column, and trip data at the sides.
  - **Sport:** a straight rev bar across the whole panel, in place of the arc, with the speed in the middle.
  - **Touring:** speed and rev bar on the left, trip data in a 2×2 grid, and coolant on the right.
  - Beyond 1920 logical pixels the bar layouts stop growing and stay centred.
- **Glyphs as signed distance fields** (`with_sdf_fonts`): one embedded set, drawn at any size.
  - Compared with the bitmaps on the same screens at 800×480:
    - the 12–13 pixel texts look the same
    - the binary shrinks from 18.5 to 10.4 MB
    - drawing the 200-pixel digits of Sport costs about 0.2 ms more per frame on the PC
  - **Fallback**, if the bench on the Pi shows a problem: bitmaps at a few fixed scales. The logical canvas would absorb the difference.
- **Dial faces** are drawn with tiny-skia at the panel's pixel size, so they are never stretched.
- **Profile, `[display]` section**, all keys optional, so existing profiles stay valid:
  - `rotation` is 0, 90, 180 or 270; `rotate_180` remains, meaning 180.
  - `form` forces `wide` or `bar` instead of `auto`.
  - `scale` forces the scale.
- **KMS:** the program uses the panel's preferred mode and any of the four rotations; the renderer rotates in software.

## Consequences

- **Covered:**
  - DPI panels with other timings
  - HDMI panels, including most bars, which need only a change to `config.txt`
  - the desktop simulator, with `--size`
- **The reference screens** are generated for each family: `design/screenshots/<size>/`. The design mock-up `design/index.html` stays at 800×480.
- **Drawing cost** grows with the pixels. On the PC, at p50 per frame:

  | Panel | Classic | Sport | Touring |
  |---|---|---|---|
  | 800×480 | 0.44 ms | 0.64 ms | 0.13 ms |
  | 1920×480 | 0.57 ms | 0.20 ms | 0.49 ms |

  The real figure on the Pi Zero 2 W must be measured on the bench, with `tdm-dash --bench 600 --size WxH`.
- **Fail-dark (ADR 0006) is not guaranteed on HDMI panels.** Their backlight is driven by the panel's own board, not by the Pico.
  - On these panels a hang leaves the last frame on screen until the watchdog reboots the Pi (10 s).
  - Restoring fail-dark needs the Pico to cut the panel supply with a MOSFET. That is a separate hardware change, not made here.
- **Not covered:**
  - square or round panels (1:1)
  - portrait mounting, with the dashboard taller than wide
  - DSI panels, which need a kernel driver for each panel
  - 4:3 and 5:4 panels use the wide layouts, with the extra height around the centre
