#!/usr/bin/env python3
"""Copies the KiCad library symbols the schematic uses into hardware/kicad/symbols/.

The generator reads its symbols from there, so it runs without KiCad and
always gives the same output. Run it in the KiCad image when a sheet starts
using a new library symbol, or when moving to a new KiCad version:

    docker run --rm -u 0 -v "$PWD":/w -w /w kicad/kicad:9.0.9 \\
        python3 hardware/kicad/vendor_symbols.py

Symbols that extend another one bring their parent along.
"""

from __future__ import annotations

import argparse
import os
import sys
from collections import defaultdict

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from sexpr import Q, dump, find, find_all, parse  # noqa: E402

USED = [
    "Connector_Generic:Conn_01x02",
    "Connector_Generic:Conn_01x03",
    "Connector_Generic:Conn_01x04",
    "Connector_Generic:Conn_01x06",
    "Device:C",
    "Device:C_Polarized",
    "Device:D_Zener",
    "Device:Fuse",
    "Device:L",
    "Device:LED",
    "Device:R",
    "Diode:1N4007",
    "Diode:1N4148",
    "Diode:1N5819",
    "Diode:BAT54S",
    "Isolator:EL817",
    "Isolator:H11L1",
    "MCU_Module:RaspberryPi_Pico",
    "Switch:SW_Push",
    "Transistor_FET:IRF4905",
    "power:+12V",
    "power:+3V3",
    "power:+5V",
    "power:GND",
    "power:PWR_FLAG",
]


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--from", dest="src", default="/usr/share/kicad/symbols")
    ap.add_argument("--out", default=os.path.join(HERE, "symbols"))
    a = ap.parse_args()

    wanted: dict[str, set[str]] = defaultdict(set)
    for lib_id in USED:
        nick, name = lib_id.split(":")
        wanted[nick].add(name)
    os.makedirs(a.out, exist_ok=True)
    for nick in sorted(wanted):
        with open(os.path.join(a.src, f"{nick}.kicad_sym"), encoding="utf-8") as f:
            tree = parse(f.read())[0]
        syms = {str(s[1]): s for s in find_all(tree, "symbol")}
        names = set(wanted[nick])
        todo = list(names)
        while todo:  # parents of derived symbols
            ext = find(syms[todo.pop()], "extends")
            if ext is not None and str(ext[1]) not in names:
                names.add(str(ext[1]))
                todo.append(str(ext[1]))
        out = ["kicad_symbol_lib", *[x for x in tree[1:] if not (isinstance(x, list) and x[0] == "symbol")]]
        out += [syms[n] for n in sorted(names)]
        for x in out:
            if isinstance(x, list) and x[0] == "generator":
                x[1] = Q("tdm-dash vendor_symbols.py")
        with open(os.path.join(a.out, f"{nick}.kicad_sym"), "w", encoding="utf-8") as f:
            f.write(dump(out) + "\n")
        print(f"{nick}: {', '.join(sorted(names))}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
