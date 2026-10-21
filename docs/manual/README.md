# tdm-dash manual

Manual for the design, construction, installation and use of the **tdm-dash** digital instrument cluster for the first-series Yamaha TDM 850 (3VD, 1991–1995).

The manual grows together with the project: each milestone adds or updates the chapters that concern it. All chapters have a first draft; the parts that depend on the bike measurements and on the bench tests are marked [TO VERIFY] and must be revised with the real measurements.

| # | Chapter | Status |
|---|---|---|
| 00 | [Introduction and safety](00-introduction-and-safety.md) | First draft |
| 01 | [The original wiring of the TDM 850](01-original-wiring.md) | First draft, to be confirmed with the measurements |
| 02 | [System architecture](02-architecture.md) | First draft |
| 03 | [Bill of materials](03-bill-of-materials.md) | First draft, prices October 2026 |
| 04 | [Schematics](04-schematics.md) | First draft, to be revised after the measurements |
| 05 | [Wiring harnesses](05-wiring-harnesses.md) | First draft, bike wire colors still to be measured |
| 06 | [Building the electronics](06-building-the-electronics.md) | First draft, to be tested on the bench |
| 07 | [Enclosure and bracket](07-enclosure-and-bracket.md) | First draft, measurements to be checked on the parts |
| 08 | [Installation on the bike](08-installation.md) | First draft, to be completed with the measurements |
| 09 | [Software: firmware, image and profile](09-software.md) | First draft; image to be tested on the hardware |
| 10 | [Calibration](10-calibration.md) | First draft, tools in `tdmctl calibrate` |
| 11 | [Usage](11-usage.md) | First draft, screens from the real software |
| 12 | [Diagnostics and troubleshooting](12-troubleshooting.md) | First draft |
| 13 | [Maintenance](13-maintenance.md) | First draft |
| A | [Appendices](A-appendices.md) | Color codes, glossary, measurement sheet |

## Conventions

- **Diagrams:** they are written in [Mermaid](https://mermaid.js.org/) inside the Markdown files, so GitHub shows them directly and they stay editable as text. The real schematics come from KiCad (`hardware/kicad/`), the wiring harnesses from WireViz (`hardware/harness/`); the generated SVG images are in `hardware/exports/`.
- **Data status:** for the original wiring, the markers of the [research](../research/original-wiring.md) are used: **[VERIFIED]**, **[TO VERIFY]**, **[HYPOTHESIS]**.
- **Notices:**

> **Caution:** risk of damage to the bike or to the electronics.

> **Danger:** risk to people.

- **Units:** metric system; voltages are referred to the battery negative, unless stated otherwise.
