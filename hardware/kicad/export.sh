#!/bin/sh
# ERC, PDF/SVG export and firmware cross-check of the schematic.
#
# Needs KiCad 9 (`kicad-cli`). Without it, run it in the official image:
#
#   docker run --rm -u 0 -v "$PWD":/w -w /w kicad/kicad:9.0 hardware/kicad/export.sh
#
# Writes hardware/exports/kicad/{tdm-dash.pdf,*.svg}; the ERC report and the
# netlist (which carry a timestamp) go to a temporary directory.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
out="$here/../exports/kicad"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
mkdir -p "$out"
cd "$here"

kicad-cli sch erc --severity-all --exit-code-violations -o "$tmp/erc.rpt" tdm-dash.kicad_sch ||
    { cat "$tmp/erc.rpt"; exit 1; }
kicad-cli sch export netlist --format kicadsexpr -o "$tmp/tdm-dash.net" tdm-dash.kicad_sch
python3 "$here/../../tools/check_pinmap.py" "$tmp/tdm-dash.net"

kicad-cli sch export pdf -o "$out/tdm-dash.pdf" tdm-dash.kicad_sch
kicad-cli sch export svg -o "$tmp/svg" tdm-dash.kicad_sch
rm -f "$out"/*.svg
for f in "$tmp"/svg/*.svg; do
    # "tdm-dash-3 Contagiri.svg" -> "3-contagiri.svg", the root sheet -> "0-indice.svg"
    name=$(basename "$f" .svg | sed 's/^tdm-dash-\{0,1\}//; s/ /-/g' | tr 'A-Z' 'a-z')
    # drop the export time from the <title>, so unchanged sheets give unchanged files
    sed 's/ date [0-9/]* [0-9:]* </ </' "$f" >"$out/${name:-0-indice}.svg"
done
ls "$out"
