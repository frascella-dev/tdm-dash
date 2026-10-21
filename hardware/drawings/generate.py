#!/usr/bin/env python3
"""Writes the dimensioned drawings of the mechanical parts, as SVG and PDF.

    python3 hardware/drawings/generate.py

Reads the OpenSCAD parameters, and the 2D exports of the sensor bracket and
the mount plate (run hardware/cad-export.sh first after changing them), and
writes hardware/exports/drawings/. Needs only Python 3.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import bracket  # noqa: E402
import enclosure  # noqa: E402
import mount  # noqa: E402
import perfboard  # noqa: E402

ROOT = HERE.parent.parent
OUT = ROOT / "hardware" / "exports" / "drawings"
REV, DATE = "0.2", "2026-10-21"

DRAWINGS = {
    "sensor-bracket": bracket.build,
    "enclosure": enclosure.build,
    "dash-mount": mount.build,
    "fit-template": mount.template,
    "perfboard": perfboard.build,
}


def main() -> int:
    for name, build in DRAWINGS.items():
        build(ROOT, REV, DATE).save(OUT / name)
        print(f"{name}: svg and pdf")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
