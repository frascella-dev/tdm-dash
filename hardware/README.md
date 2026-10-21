# Hardware

| Folder | Contents | How to regenerate |
|---|---|---|
| [`kicad/`](kicad/) | KiCad 9 schematic of the sensor module, 8 sheets. **Generated** by [`generate.py`](kicad/generate.py): do not edit the `.kicad_sch` files by hand | `python3 hardware/kicad/generate.py` |
| [`harness/`](harness/) | Wiring harnesses in WireViz (YAML) | `wireviz -f st -o hardware/exports/harness hardware/harness/<file>.yml` |
| [`exports/kicad/`](exports/kicad/) | Schematic as PDF and one SVG per sheet | `hardware/kicad/export.sh` (ERC and pin check included) |
| [`exports/harness/`](exports/harness/) | Harness drawings (SVG) and bills of materials (TSV) | see `harness/` |
| [`perfboard/`](perfboard/) | Zone layout of the 90 × 70 mm perfboard (`layout.svg`) | `python3 hardware/perfboard/layout.py` |
| [`enclosure/`](enclosure/) | Parametric enclosure in OpenSCAD: bezel, rear shell, sun visor | `hardware/cad-export.sh` |
| [`bracket/`](bracket/) | Wheel sensor bracket in OpenSCAD | `hardware/cad-export.sh` |
| [`exports/cad/`](exports/cad/) | STL files to print, DXF/SVG of the bracket, PNG previews | `hardware/cad-export.sh` (also with Docker, see below) |

Where they are explained in the manual:
- every schematic sheet and the component calculations: [chapter 04](../docs/manual/04-schematics.md)
- wiring harnesses and pinouts: [chapter 05](../docs/manual/05-wiring-harnesses.md)
- building the perfboard: [chapter 06](../docs/manual/06-building-the-electronics.md)
- enclosure and bracket: [chapter 07](../docs/manual/07-enclosure-and-bracket.md)

The mechanical parts are exported with OpenSCAD 2021.01, also through Docker:

```sh
docker run --rm -u 0 -v "$PWD":/w -w /w openscad/openscad:2021.01 hardware/cad-export.sh
```

## Editing the schematic

1. Edit the circuit description in `kicad/generate.py` (function `build()`): each component has its KiCad symbol and the net on each pin.
2. Regenerate and check with KiCad 9, also through Docker:

   ```sh
   docker run --rm -u 0 -v "$PWD":/w -w /w kicad/kicad:9.0.9 \
       sh -c 'python3 hardware/kicad/generate.py && hardware/kicad/export.sh'
   ```

   `export.sh` runs the ERC and [`tools/check_pinmap.py`](../tools/check_pinmap.py), which compares the Pico GPIOs with `firmware/src/main.rs`, then exports PDF and SVG.
3. Commit both `generate.py` and the generated files: CI (the `Hardware` workflow) checks that they match.

Opening the project in KiCad to look at it is perfectly fine. Changes made in the editor must however be carried over to `generate.py`, otherwise the next generation erases them.

## Licence

Hardware under CERN-OHL-P-2.0 (see [LICENSING.md](../LICENSING.md)).
