"""Writes the KiCad 9 files: sub-sheets, root sheet, project and drawing sheet.

UUIDs are derived from content (sheet, kind, reference or coordinates), so
the same description always gives the same files.
"""

from __future__ import annotations

import copy
import uuid

from sexpr import Q, find, find_all

from .draw import POWER, Part, Root, Sheet
from .geom import DOWN, LEFT, RIGHT, UP, lib_to_sheet, mil, mm

PROJECT = "tdm-dash"
VERSION = 20250114  # KiCad 9.0 schematic format
NS = uuid.UUID("6f1d3c52-0b7e-4b8e-9b1e-7d0d9a3c2e10")
WKS = f"{PROJECT}.kicad_wks"
FONT = 1.27


def uid(*parts) -> Q:
    return Q(str(uuid.uuid5(NS, "/".join(str(p) for p in parts))))


def xy(p) -> list:
    return ["xy", mm(p[0]), mm(p[1])]


def at(p, angle=0) -> list:
    return ["at", mm(p[0]), mm(p[1]), angle]


def font(size_mil: int | None = None, bold=False, color=None) -> list:
    s = mm(size_mil) if size_mil else FONT
    f = ["font", ["size", s, s]]
    if bold:
        f.append(["bold", "yes"])
    if color:
        f.append(["color", *color, 1])
    return f


def label_angle(out) -> tuple[int, str]:
    """KiCad angle and justification of a label whose text extends towards `out`."""
    return {RIGHT: (0, "left"), LEFT: (180, "right"), UP: (90, "left"), DOWN: (270, "right")}[out]


class Writer:
    def __init__(self, sheets: list[Sheet], root: Root, drawn: dict, date: str, rev: str):
        self.sheets = sheets
        self.root = root
        self.drawn = drawn
        self.date = date
        self.rev = rev
        self.root_uuid = uid("root")

    # parts ------------------------------------------------------------------

    def field_spots(self, sh: Sheet, p: Part) -> dict:
        """Reference and value positions: next to two-pin parts, as in the
        library for the others, unless the sheet placed them."""
        spots = {}
        pins = list(p.pins.values())
        small = p.lib_id.split(":")[0] in ("Device", "Diode")
        if small and len(pins) == 2 and not p.fields:
            (x1, y1), (x2, y2) = pins[0].pos, pins[1].pos
            cx, cy = (x1 + x2) // 2, (y1 + y2) // 2
            if y1 == y2:  # horizontal: reference above, value below
                spots["Reference"] = (cx - p.x, cy - 110 - p.y, "")
                spots["Value"] = (cx - p.x, cy + 110 - p.y, "")
            else:  # vertical: both on the right
                spots["Reference"] = (cx + 110 - p.x, cy - 55 - p.y, "left")
                spots["Value"] = (cx + 110 - p.x, cy + 55 - p.y, "left")
        spots.update(p.fields)
        return spots

    def instance(self, sh: Sheet, p: Part, sym: list, ref: str, value: str, hide_ref=False) -> list:
        path = f"/{self.root_uuid}/{uid('sheet', sh.file)}"
        node = [
            "symbol",
            ["lib_id", Q(sym[1])],
            ["at", mm(p.x), mm(p.y), p.rot],
        ]
        if p.mirror:
            node.append(["mirror", p.mirror])
        node += [
            ["unit", 1],
            ["exclude_from_sim", "no"],
            ["in_bom", "no" if ref.startswith("#") else "yes"],
            ["on_board", "yes"],
            ["dnp", "no"],
            ["uuid", uid(sh.file, "symbol", ref)],
        ]
        spots = self.field_spots(sh, p) if not ref.startswith("#") else {}
        for prop in find_all(sym, "property"):
            name = str(prop[1])
            val = {"Reference": ref, "Value": value}.get(name, str(prop[2]))
            if name in p.extra:
                val = p.extra[name]
            eff = copy.deepcopy(find(prop, "effects"))
            hidden = name not in ("Reference", "Value") or (name == "Value" and p.hide_value)
            hidden = hidden or (name == "Reference" and hide_ref)
            eff = [e for e in eff if not (isinstance(e, list) and e[0] == "hide")]
            if hidden:
                eff.append(["hide", "yes"])
            if name in spots:
                dx, dy, just = spots[name]
                eff = [e for e in eff if not (isinstance(e, list) and e[0] == "justify")]
                if just:
                    eff.append(["justify", *just.split()])
                # field angles are stored relative to the symbol rotation
                node.append(["property", Q(name), Q(val), ["at", mm(p.x + dx), mm(p.y + dy), p.rot % 180], eff])
                continue
            pa = find(prop, "at")
            dx, dy = lib_to_sheet(mil(pa[1]), mil(pa[2]), p.rot, p.mirror)
            ang = int(round(float(pa[3]) + p.rot)) % 180
            node.append(["property", Q(name), Q(val), ["at", mm(p.x + dx), mm(p.y + dy), ang], eff])
        seen = set()
        for unit in find_all(sym, "symbol"):
            for pin in find_all(unit, "pin"):
                num = str(find(pin, "number")[1])
                if num in seen:
                    continue
                seen.add(num)
                node.append(["pin", Q(num), ["uuid", uid(sh.file, ref, "pin", num)]])
        node.append(["instances", ["project", Q(PROJECT), ["path", Q(path), ["reference", Q(ref)], ["unit", 1]]]])
        return node

    def power_symbol(self, sh: Sheet, net: str, pos, out, ref: str, flag=False) -> tuple[list, str]:
        lib_id = "power:PWR_FLAG" if flag else POWER[net]
        if lib_id == "power:GND":
            rot = {DOWN: 0, LEFT: 270, UP: 180, RIGHT: 90}[out]
        else:
            rot = {UP: 0, LEFT: 90, DOWN: 180, RIGHT: 270}[out]
        p = Part(ref, lib_id, "PWR_FLAG" if flag else net, pos[0], pos[1], rot, None, {})
        sym = sh.lib.symbol(lib_id)
        return self.instance(sh, p, sym, ref, p.value, hide_ref=True), lib_id

    # sheets -----------------------------------------------------------------

    def title_block(self, title: str) -> list:
        return ["title_block", ["title", Q(title)], ["date", Q(self.date)], ["rev", Q(self.rev)],
                ["company", Q("tdm-dash")]]

    def common(self, key: str, wires, junctions, texts, rects) -> list:
        body = []
        for p in junctions:
            body.append(["junction", at(p)[:3], ["diameter", 0], ["color", 0, 0, 0, 0], ["uuid", uid(key, "junction", *p)]])
        for a, b in wires:
            body.append(["wire", ["pts", xy(a), xy(b)], ["stroke", ["width", 0], ["type", "default"]],
                         ["uuid", uid(key, "wire", *a, *b)]])
        for r in rects:
            body.append(["rectangle", ["start", mm(r.x1), mm(r.y1)], ["end", mm(r.x2), mm(r.y2)],
                         ["stroke", ["width", 0.1524 if r.dashed else 0.2032], ["type", "dash" if r.dashed else "solid"]],
                         ["fill", ["type", "none"]], ["uuid", uid(key, "rect", r.x1, r.y1, r.x2, r.y2)]])
        for t in texts:
            body.append(["text", Q(t.text), ["exclude_from_sim", "no"], at((t.x, t.y), t.angle),
                         ["effects", font(t.size, t.bold, t.color), ["justify", *t.justify.split()]],
                         ["uuid", uid(key, "text", t.x, t.y, t.text)]])
        return body

    def label(self, key: str, lb) -> list:
        angle, just = label_angle(lb.out)
        if lb.kind == "local":
            return ["label", Q(lb.net), at(lb.pos, angle), ["fields_autoplaced", "yes"],
                    ["effects", font(), ["justify", just, "bottom"]], ["uuid", uid(key, "label", lb.net, *lb.pos)]]
        return ["hierarchical_label", Q(lb.net), ["shape", lb.shape], at(lb.pos, angle), ["fields_autoplaced", "yes"],
                ["effects", font(), ["justify", just]], ["uuid", uid(key, "hlabel", lb.net, *lb.pos)]]

    def sheet_file(self, sh: Sheet) -> list:
        lib_syms: dict[str, list] = {}
        body: list = []
        wires, junctions = self.drawn[sh.file]
        body += self.common(sh.file, wires, junctions, sh.texts, sh.rects)
        for p in sorted(sh.parts, key=lambda q: q.ref):
            sym = lib_syms.setdefault(p.lib_id, sh.lib.symbol(p.lib_id))
            body.append(self.instance(sh, p, sym, p.ref, p.value))
            for pin in sorted(p.pins.values(), key=lambda q: q.number):
                if pin.net is None:
                    body.append(["no_connect", at(pin.pos)[:3], ["uuid", uid(sh.file, "nc", p.ref, pin.number)]])
        n = 0
        for pw in sorted(sh.powers, key=lambda q: (q.pos, q.net)):
            n += 1
            node, lib_id = self.power_symbol(sh, pw.net, pw.pos, pw.out, f"#PWR{sh.num}{n:02d}")
            lib_syms.setdefault(lib_id, sh.lib.symbol(lib_id))
            body.append(node)
        for i, (pos, d) in enumerate(sorted(sh.flags)):
            node, lib_id = self.power_symbol(sh, "", pos, d, f"#FLG{sh.num}{i + 1:02d}", flag=True)
            lib_syms.setdefault(lib_id, sh.lib.symbol(lib_id))
            body.append(node)
        for lb in sorted(sh.labels, key=lambda q: (q.kind, q.net, q.pos)):
            body.append(self.label(sh.file, lb))
        return [
            "kicad_sch",
            ["version", VERSION],
            ["generator", Q("eeschema")],
            ["generator_version", Q("9.0")],
            ["uuid", uid("sheet", sh.file)],
            ["paper", Q("A4")],
            self.title_block(sh.title),
            ["lib_symbols", *[lib_syms[k] for k in sorted(lib_syms)]],
            *body,
            ["embedded_fonts", "no"],
        ]

    def root_file(self) -> list:
        r = self.root
        wires, junctions = self.drawn["root"]
        body = self.common("root", wires, junctions, r.texts, r.rects)
        for lb in sorted(r.labels, key=lambda q: (q.net, q.pos)):
            body.append(self.label("root", lb))
        for b in r.blocks:
            sh = b.sheet
            node = [
                "sheet",
                at((b.x, b.y))[:3],
                ["size", mm(b.w), mm(b.h)],
                ["exclude_from_sim", "no"],
                ["in_bom", "yes"],
                ["on_board", "yes"],
                ["dnp", "no"],
                ["fields_autoplaced", "no"],
                ["stroke", ["width", 0.2032], ["type", "solid"]],
                ["fill", ["color", 255, 255, 255, 0.0]],
                ["uuid", uid("sheet", sh.file)],
                ["property", Q("Sheetname"), Q(sh.name), at((b.x, b.y - 30)),
                 ["effects", font(60, bold=True), ["justify", "left", "bottom"]]],
                ["property", Q("Sheetfile"), Q(sh.file), at((b.x, b.y + b.h + 30)),
                 ["effects", font(), ["justify", "left", "top"], ["hide", "yes"]]],
            ]
            shapes = {lb.net: lb.shape for lb in sh.labels if lb.kind == "hier"}
            for pname, (pos, side) in sorted(b.pins.items()):
                angle, just = {"left": (180, "left"), "right": (0, "right"), "top": (90, "right"),
                               "bottom": (270, "left")}[side]
                node.append(["pin", Q(pname), shapes.get(pname, "passive"), at(pos, angle),
                             ["uuid", uid("sheetpin", sh.file, pname)],
                             ["effects", font(), ["justify", just]]])
            node.append(["instances", ["project", Q(PROJECT), ["path", Q(f"/{self.root_uuid}"), ["page", Q(str(sh.num))]]]])
            body.append(node)
        return [
            "kicad_sch",
            ["version", VERSION],
            ["generator", Q("eeschema")],
            ["generator_version", Q("9.0")],
            ["uuid", self.root_uuid],
            ["paper", Q("A4")],
            self.title_block(r.title),
            ["lib_symbols"],
            *body,
            ["sheet_instances", ["path", Q("/"), ["page", Q("0")]]],
            ["embedded_fonts", "no"],
        ]


def project_file(last_sheet: int) -> dict:
    return {
        "meta": {"filename": f"{PROJECT}.kicad_pro", "version": 3},
        "erc": {
            "erc_exclusions": [],
            "meta": {"version": 0},
            "pin_map": [],
            "rule_severities": {
                # symbols are embedded on purpose: no library table needed
                "lib_symbol_issues": "ignore",
                "lib_symbol_mismatch": "ignore",
            },
        },
        "schematic": {"legacy_lib_dir": "", "legacy_lib_list": [], "page_layout_descr_file": WKS},
        "sheets": [],
        "boards": [],
        "text_variables": {"LAST_SHEET": str(last_sheet)},
    }


def drawing_sheet() -> list:
    """Frame and title block, shared by every sheet.

    Coordinates are millimetres from the bottom-right corner of the margin
    box unless marked `ltcorner`/`lbcorner`/`rtcorner`; x grows to the left
    and y upwards from the bottom-right corner.
    """
    s = 4.0  # zone strip
    w, h = 124.0, 34.0  # title block outer corner, from the bottom-right
    items: list = [
        ["setup", ["textsize", 1.27, 1.27], ["linewidth", 0.15], ["textlinewidth", 0.15],
         ["left_margin", 10], ["right_margin", 10], ["top_margin", 10], ["bottom_margin", 10]],
        ["rect", ["name", Q("")], ["start", 0, 0, "ltcorner"], ["end", 0, 0], ["linewidth", 0.3]],
        ["rect", ["name", Q("")], ["start", s, s, "ltcorner"], ["end", s, s], ["linewidth", 0.15]],
    ]
    # zone marks every 50 mm, numbers along the top and bottom, letters at the sides
    for corner, y1, y2, ty in (("ltcorner", 0, s, s / 2), ("lbcorner", 0, s, s / 2)):
        items.append(["line", ["name", Q("")], ["start", 50, y1, corner], ["end", 50, y2, corner],
                      ["repeat", 6], ["incrx", 50]])
        items.append(["tbtext", Q("1"), ["name", Q("")], ["pos", 25, ty, corner], ["font", ["size", 1.3, 1.3]],
                      ["justify", "center"], ["repeat", 6], ["incrx", 50]])
    for corner in ("ltcorner", "rtcorner"):
        items.append(["line", ["name", Q("")], ["start", 0, 50, corner], ["end", s, 50, corner],
                      ["repeat", 4], ["incry", 50]])
        items.append(["tbtext", Q("A"), ["name", Q("")], ["pos", s / 2, 25, corner], ["font", ["size", 1.3, 1.3]],
                      ["justify", "center"], ["repeat", 4], ["incry", 50]])

    def line(x1, y1, x2, y2, width=0.15):
        items.append(["line", ["name", Q("")], ["start", x1, y1], ["end", x2, y2], ["linewidth", width]])

    def text(t, x, y, size, bold=False, justify=None):
        f = ["font", ["size", size, size]]
        if bold:
            f.append("bold")
        node = ["tbtext", Q(t), ["name", Q("")], ["pos", x, y], f]
        if justify:
            node.append(["justify", *justify.split()])
        items.append(node)

    x0, y0 = s, s  # bottom-right corner of the block (inner frame)
    items.append(["rect", ["name", Q("")], ["start", w, h], ["end", x0, y0], ["linewidth", 0.3]])
    # rows, from the bottom: sheet data, title, project, origin
    r1, r2, r3 = y0 + 8, y0 + 19, y0 + 25
    line(w, r1, x0, r1)
    line(w, r2, x0, r2)
    line(w, r3, x0, r3)
    # sheet data cells: sheet | rev | date | size
    cells = [(w, w - 34), (w - 34, w - 56), (w - 56, w - 96), (w - 96, x0)]
    for left, _ in cells[1:]:
        line(left, y0, left, r1)
    for (left, _), (cap, val) in zip(cells, [("SHEET", "${#} / ${LAST_SHEET}"), ("REV", "${REVISION}"),
                                            ("DATE", "${ISSUE_DATE}"), ("SIZE", "${PAPER}")]):
        text(cap, left - 1.5, r1 - 1.6, 1.0)
        text(val, left - 1.5, y0 + 2.8, 1.8, bold=True)
    text("${TITLE}", w - 1.5, (r1 + r2) / 2, 2.6, bold=True)
    text("tdm-dash", w - 1.5, (r2 + r3) / 2, 2.0, bold=True)
    text("sensor module · Yamaha TDM 850 (3VD)", w - 22, (r2 + r3) / 2, 1.4)
    text("CERN-OHL-P-2.0", x0 + 1.5, (r2 + r3) / 2, 1.4, justify="right")
    text("github.com/frascella-dev/tdm-dash · generated by hardware/kicad/generate.py, do not edit by hand",
         w - 1.5, (r3 + h) / 2, 1.0)
    return ["kicad_wks", ["version", 20231118], ["generator", Q("tdm-dash")], ["generator_version", Q("9.0")],
            *items]
