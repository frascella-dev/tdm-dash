#!/bin/sh
# Exports the printable parts and the previews used by the manual.
#
# Needs OpenSCAD 2021.01 (and xvfb-run for the PNG previews). Without it:
#
#   docker run --rm -u 0 -v "$PWD":/w -w /w openscad/openscad:2021.01 hardware/cad-export.sh
#
# Writes hardware/exports/cad/: STL of every part, DXF and SVG of the bracket
# outline (for cutting the aluminium plate) and PNG previews.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
out="$here/exports/cad"
mkdir -p "$out"
enc="$here/enclosure/enclosure.scad"
brk="$here/bracket/sensor-bracket.scad"

for part in front rear visor; do
    openscad -q -D "part=\"$part\"" -o "$out/enclosure-$part.stl" "$enc"
done
openscad -q -D 'out="3d"' -o "$out/sensor-bracket.stl" "$brk"
openscad -q -D 'out="2d"' -o "$out/sensor-bracket.dxf" "$brk"
openscad -q -D 'out="2d"' -o "$out/sensor-bracket.svg" "$brk"

if command -v xvfb-run >/dev/null; then
    png() { xvfb-run -a openscad -q --imgsize=1000,700 --viewall --autocenter --colorscheme=Tomorrow "$@"; }
    png -D 'part="all"' --camera=0,0,0,20,0,-30,0 -o "$out/enclosure-assembly.png" "$enc"
    png -D 'part="all"' --camera=0,0,0,160,0,150,0 -o "$out/enclosure-back.png" "$enc"
    png -D 'part="rear"' --camera=0,0,0,40,0,20,0 -o "$out/enclosure-rear-inside.png" "$enc"
    png -D 'out="3d"' -o "$out/sensor-bracket.png" "$brk"
fi
ls -la "$out"
