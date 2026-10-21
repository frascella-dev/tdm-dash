#!/usr/bin/env python3
"""Generates the tdm-dash KiCad 9 schematic.

The circuit is described in hardware/kicad/sheets/, one module per sheet:
each part is placed with the declared net of every pin (the electrical
specification), and the wires, labels and power symbols that draw those
connections. The generator:

- normalises the drawing as KiCad reads it (wires split at every T, with a
  junction dot) and refuses what KiCad would connect differently from what
  is meant: wires ending in the middle of a wire, overlaps, four-way
  junctions, wires through symbols, labels on several wires;
- checks that the drawn connections are exactly the declared nets;
- writes the sub-sheets, the root sheet with the block diagram, the project
  and the drawing sheet (title block).

Symbols come from hardware/kicad/symbols (copied from KiCad 9.0.9 by
vendor_symbols.py) and schgen/symbols.py; UUIDs are derived from content,
so the output is the same on every run.

    python3 hardware/kicad/generate.py              # writes hardware/kicad/
    python3 hardware/kicad/generate.py --nets-json nets.json
    python3 hardware/kicad/generate.py --only 3 --out /tmp/x   # one sheet, for drawing it

The output is checked by `kicad-cli sch erc`, tools/check_netlist.py and
tools/check_pinmap.py (hardware/kicad/export.sh).
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from collections import defaultdict

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

from sexpr import dump  # noqa: E402
from schgen.check import DrawingError, check  # noqa: E402
from schgen.draw import Root  # noqa: E402
from schgen.library import Library  # noqa: E402
from schgen.symbols import project_symbols  # noqa: E402
from schgen.writer import PROJECT, WKS, Writer, drawing_sheet, project_file  # noqa: E402
from sheets import SHEETS, root  # noqa: E402

DATE = "2026-10-21"
REV = "0.2"


def declared_nets(sheets) -> dict[str, set[str]]:
    nets: dict[str, set[str]] = defaultdict(set)
    for sh in sheets:
        for p in sh.parts:
            for pin in p.pins.values():
                if pin.net is not None:
                    nets[pin.net].add(f"{p.ref}.{pin.number}")
    return nets


def single_sheet_root(sheet) -> Root:
    """A root with just one block, its ports ending in labels (for --only)."""
    r = Root(f"{sheet.title} (drawing check)")
    hier = sorted({lb.net for lb in sheet.labels if lb.kind == "hier"})
    b = r.block(sheet, 1000, 1000, 2000, 200 * (len(hier) + 1))
    for i, net in enumerate(hier):
        pos = b.pin(net, "right", 200 * (i + 1))
        end = (pos[0] + 300, pos[1])
        r.wire(pos, end)
        r.label(net, end, (1, 0))
    return r


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--symbols", default=os.path.join(HERE, "symbols"))
    ap.add_argument("--out", default=HERE)
    ap.add_argument("--nets-json", help="also write the declared netlist to this file")
    ap.add_argument("--only", type=int, help="build one sheet and a stub root (to work on its drawing)")
    a = ap.parse_args()

    lib = Library(a.symbols, project_symbols())
    sheets = [build(lib) for build in SHEETS]
    if a.only is not None:
        sheets = [s for s in sheets if s.num == a.only]
        r = single_sheet_root(sheets[0])
    else:
        r = root.build(sheets)
    nets = declared_nets(sheets)
    if a.only is None:
        lonely = {n: sorted(p) for n, p in nets.items() if len(p) < 2}
        if lonely:
            raise SystemExit(f"nets with a single pin: {lonely}")
    try:
        drawn = check(sheets, r, nets)
    except DrawingError as e:
        print(f"drawing errors:\n{e}", file=sys.stderr)
        return 1

    w = Writer(sheets, r, drawn, DATE, REV)
    os.makedirs(a.out, exist_ok=True)

    def write(name, text):
        with open(os.path.join(a.out, name), "w", encoding="utf-8") as f:
            f.write(text)

    write(f"{PROJECT}.kicad_sch", dump(w.root_file()) + "\n")
    for sh in sheets:
        write(sh.file, dump(w.sheet_file(sh)) + "\n")
    write(f"{PROJECT}.kicad_pro", json.dumps(project_file(len(SHEETS)), indent=2) + "\n")
    write(WKS, dump(drawing_sheet()) + "\n")
    if a.nets_json:
        with open(a.nets_json, "w", encoding="utf-8") as f:
            json.dump({n: sorted(p) for n, p in sorted(nets.items())}, f, indent=1)
            f.write("\n")
    parts = sum(len(s.parts) for s in sheets)
    print(f"{len(sheets)} sheets, {parts} parts, {len(nets)} nets, drawing checked")
    return 0


if __name__ == "__main__":
    sys.exit(main())
