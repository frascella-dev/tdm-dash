"""Symbols: the KiCad library symbols copied into hardware/kicad/symbols, and
the ones this project defines (schgen.symbols)."""

from __future__ import annotations

import copy
import os
from dataclasses import dataclass

from sexpr import Q, find, find_all, parse

from .geom import Box, mil

# The KiCad Pico symbol marks both GND and AGND as power outputs, which ERC
# reports as two drivers on one net. AGND is just a ground pin here.
PIN_TYPE_PATCHES = {("MCU_Module:RaspberryPi_Pico", "33"): "passive"}


@dataclass(frozen=True)
class LibPin:
    number: str
    name: str
    type: str
    x: int  # mils, library coordinates (y up)
    y: int
    angle: int  # direction from the connection point towards the body
    hidden: bool


class Library:
    def __init__(self, root: str, custom: dict[str, list] | None = None):
        self.root = root
        self.custom = custom or {}
        self.files: dict[str, dict[str, list]] = {}
        self.cache: dict[str, list] = {}

    def _lib(self, nick: str) -> dict[str, list]:
        if nick not in self.files:
            path = os.path.join(self.root, f"{nick}.kicad_sym")
            with open(path, encoding="utf-8") as f:
                tree = parse(f.read())[0]
            self.files[nick] = {str(s[1]): s for s in find_all(tree, "symbol")}
        return self.files[nick]

    def symbol(self, lib_id: str) -> list:
        """The symbol flattened (no `extends`) and named `Lib:Name`."""
        if lib_id not in self.cache:
            self.cache[lib_id] = self._load(lib_id)
        return copy.deepcopy(self.cache[lib_id])

    def _load(self, lib_id: str) -> list:
        if lib_id in self.custom:
            return copy.deepcopy(self.custom[lib_id])
        nick, name = lib_id.split(":")
        sym = copy.deepcopy(self._lib(nick)[name])
        parent = find(sym, "extends")
        if parent is not None:
            base = self._load(f"{nick}:{parent[1]}")
            pname = base[1].split(":")[1]
            props = find_all(sym, "property")
            out = [x for x in base if not (isinstance(x, list) and x[0] == "property")]
            # child properties replace the parent's, kept in the parent's slot
            idx = next(i for i, x in enumerate(out) if isinstance(x, list) and x[0] == "symbol")
            out[idx:idx] = props
            for x in out:
                if isinstance(x, list) and x[0] == "symbol":
                    x[1] = Q(name + x[1][len(pname) :])
            sym = out
        sym[1] = Q(lib_id)
        for unit in find_all(sym, "symbol"):
            for pin in find_all(unit, "pin"):
                key = (lib_id, str(find(pin, "number")[1]))
                if key in PIN_TYPE_PATCHES:
                    pin[1] = PIN_TYPE_PATCHES[key]
        return sym

    def pins(self, lib_id: str) -> list[LibPin]:
        out = []
        for unit in find_all(self.symbol(lib_id), "symbol"):
            for p in find_all(unit, "pin"):
                at = find(p, "at")
                hidden = any(isinstance(x, list) and x[:2] == ["hide", "yes"] for x in p) or "hide" in p
                out.append(
                    LibPin(
                        number=str(find(p, "number")[1]),
                        name=str(find(p, "name")[1]),
                        type=str(p[1]),
                        x=mil(at[1]),
                        y=mil(at[2]),
                        angle=int(round(float(at[3]))) % 360,
                        hidden=hidden,
                    )
                )
        return out

    def body(self, lib_id: str) -> Box | None:
        """Bounding box of the drawing (pins excluded), library coordinates."""
        pts: list[tuple[int, int]] = []

        def walk(n):
            if not isinstance(n, list) or not n:
                return
            if n[0] == "pin":
                return
            if n[0] == "rectangle":
                for k in ("start", "end"):
                    p = find(n, k)
                    pts.append((mil(p[1]), mil(p[2])))
            elif n[0] in ("polyline", "bezier"):
                for p in find_all(find(n, "pts"), "xy"):
                    pts.append((mil(p[1]), mil(p[2])))
            elif n[0] == "circle":
                c = find(n, "center")
                r = mil(find(n, "radius")[1])
                cx, cy = mil(c[1]), mil(c[2])
                pts.extend([(cx - r, cy - r), (cx + r, cy + r)])
            elif n[0] == "arc":
                for k in ("start", "mid", "end"):
                    p = find(n, k)
                    pts.append((mil(p[1]), mil(p[2])))
            else:
                for x in n:
                    walk(x)

        for unit in find_all(self.symbol(lib_id), "symbol"):
            walk(unit)
        return Box.around(pts) if pts else None
