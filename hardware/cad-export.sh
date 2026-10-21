#!/bin/sh
# Exports the printable parts and the previews used by the manual.
#
# Needs OpenSCAD 2021.01 (and xvfb-run for the PNG previews). Without it:
#
#   docker run --rm -u 0 -v "$PWD":/w -w /w openscad/openscad:2021.01 hardware/cad-export.sh
#
# Writes hardware/exports/cad/: STL of every part, DXF and SVG of the bracket
# outline (for cutting the aluminum plate) and PNG previews. The dimensioned
# drawing of the bracket reads that outline: run
# `python3 hardware/drawings/generate.py` afterwards.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
out="$here/exports/cad"
mkdir -p "$out"
enc="$here/enclosure/enclosure.scad"
brk="$here/bracket/sensor-bracket.scad"

for part in front cover rear; do
    openscad -q -D "part=\"$part\"" -o "$out/enclosure-$part.stl" "$enc"
done
openscad -q -D 'out="3d"' -o "$out/sensor-bracket.stl" "$brk"
openscad -q -D 'out="2d"' -o "$out/sensor-bracket.dxf" "$brk"
openscad -q -D 'out="2d"' -o "$out/sensor-bracket.svg" "$brk"

# Previews. The enclosure has Y up and Z towards the rider: the camera turns
# about Y to look from a side (ry > 0: from the right) and about X to look
# from above (rx < 0): --camera=0,0,0,rx,ry,0,0.
if command -v xvfb-run >/dev/null; then
    png() { xvfb-run -a openscad -q --imgsize=1000,700 --viewall --autocenter --colorscheme=Tomorrow "$@"; }
    png -D 'part="all"' --camera=0,0,0,-20,30,0,0 -o "$out/enclosure-assembly.png" "$enc"
    png -D 'part="exploded"' --camera=0,0,0,-18,55,0,0 -o "$out/enclosure-exploded.png" "$enc"
    png -D 'part="all"' --camera=0,0,0,-20,-150,0,0 -o "$out/enclosure-back.png" "$enc"
    png -D 'part="rear"' --camera=0,0,0,-25,20,0,0 -o "$out/enclosure-rear-inside.png" "$enc"
fi
ls -la "$out"
