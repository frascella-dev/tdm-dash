#!/usr/bin/env python3
"""Checks that the netlist KiCad reads from the schematic is the declared one.

hardware/kicad/generate.py declares the net of every pin and draws the wires
between them; this compares the connections KiCad exports from the drawn
schematic with that declaration, as groups of pins (net names may differ:
KiCad names unnamed nets after a pin). Power symbols (#PWR, #FLG) and
unconnected pins are ignored.

    python3 hardware/kicad/generate.py --out /tmp/gen --nets-json nets.json
    kicad-cli sch export netlist --format kicadsexpr -o tdm-dash.net hardware/kicad/tdm-dash.kicad_sch
    python3 tools/check_netlist.py tdm-dash.net nets.json
"""

from __future__ import annotations

import json
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.dont_write_bytecode = True
sys.path.insert(0, os.path.join(ROOT, "hardware", "kicad"))
from sexpr import find, find_all, parse  # noqa: E402


def kicad_groups(path: str) -> dict[frozenset, str]:
    with open(path, encoding="utf-8") as f:
        tree = parse(f.read())[0]
    groups = {}
    for net in find_all(find(tree, "nets"), "net"):
        pins = frozenset(
            f"{find(n, 'ref')[1]}.{find(n, 'pin')[1]}"
            for n in find_all(net, "node")
            if not str(find(n, "ref")[1]).startswith("#")
        )
        if len(pins) > 1:
            groups[pins] = str(find(net, "name")[1])
    return groups


def main() -> int:
    if len(sys.argv) != 3:
        print(__doc__)
        return 2
    drawn = kicad_groups(sys.argv[1])
    with open(sys.argv[2], encoding="utf-8") as f:
        declared = {frozenset(pins): name for name, pins in json.load(f).items()}
    errors = []
    for pins, name in sorted(declared.items(), key=lambda kv: kv[1]):
        if pins not in drawn:
            near = [sorted(g) for g in drawn if g & pins]
            errors.append(f"declared net {name} {sorted(pins)} is drawn as {near}")
    for pins, name in sorted(drawn.items(), key=lambda kv: kv[1]):
        if pins not in declared:
            errors.append(f"KiCad net {name} {sorted(pins)} is not declared")
    for e in errors:
        print(f"ERROR: {e}", file=sys.stderr)
    if errors:
        return 1
    print(f"netlist: the {len(declared)} declared nets are exactly the ones drawn")
    return 0


if __name__ == "__main__":
    sys.exit(main())
