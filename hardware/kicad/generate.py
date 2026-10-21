#!/usr/bin/env python3
"""Generates the tdm-dash KiCad 9 schematic from the circuit description below.

Writing `.kicad_sch` files by hand is fragile, so the schematic is described
here as a list of parts per sheet with a net name on every pin. The script:

- copies each symbol from the KiCad libraries into the schematic, so the
  project opens without any library setup;
- draws a short wire on every pin, ending in a net label (local when the net
  stays on one sheet, global otherwise) or a power symbol (GND, +3V3, +5V);
- marks unused pins with a no-connect flag;
- writes a root sheet holding the eight sub-sheets.

UUIDs are derived from names, so running it twice gives identical files.

    python3 hardware/kicad/generate.py [--symbols /usr/share/kicad/symbols]

The output is checked by `kicad-cli sch erc` and by tools/check_pinmap.py.
"""

from __future__ import annotations

import argparse
import copy
import json
import math
import os
import sys
import uuid
from collections import defaultdict
from dataclasses import dataclass, field

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sexpr import Q, dump, find, find_all, parse  # noqa: E402

PROJECT = "tdm-dash"
VERSION = 20250114  # KiCad 9.0 schematic format
DATE = "2026-10-21"
REV = "0.1"
NS = uuid.UUID("6f1d3c52-0b7e-4b8e-9b1e-7d0d9a3c2e10")
POWER_NETS = {"GND": "power:GND", "+3V3": "power:+3V3", "+5V": "power:+5V"}
STUB = 2.54
# The KiCad Pico symbol marks both GND and AGND as power outputs, which ERC
# reports as two drivers on one net. AGND is just a ground pin here.
PIN_TYPE_PATCHES = {("MCU_Module:RaspberryPi_Pico", "33"): "passive"}


def uid(*parts: str) -> Q:
    return Q(str(uuid.uuid5(NS, "/".join(parts))))


# --------------------------------------------------------------------------
# symbol libraries


class Library:
    def __init__(self, root: str):
        self.root = root
        self.files: dict[str, dict[str, list]] = {}

    def _lib(self, nick: str) -> dict[str, list]:
        if nick not in self.files:
            path = os.path.join(self.root, f"{nick}.kicad_sym")
            with open(path, encoding="utf-8") as f:
                tree = parse(f.read())[0]
            self.files[nick] = {s[1]: s for s in find_all(tree, "symbol")}
        return self.files[nick]

    def symbol(self, lib_id: str) -> list:
        """Returns the symbol flattened (no `extends`) and named `Lib:Name`."""
        nick, name = lib_id.split(":")
        lib = self._lib(nick)
        sym = copy.deepcopy(lib[name])
        parent = find(sym, "extends")
        if parent is not None:
            base = self.symbol(f"{nick}:{parent[1]}")
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
                if (lib_id, str(find(pin, "number")[1])) in PIN_TYPE_PATCHES:
                    pin[1] = PIN_TYPE_PATCHES[(lib_id, str(find(pin, "number")[1]))]
        return sym


def sym_pins(sym: list) -> list[dict]:
    pins = []
    for unit in find_all(sym, "symbol"):
        for p in find_all(unit, "pin"):
            at = find(p, "at")
            hidden = any(isinstance(x, list) and x[:2] == ["hide", "yes"] for x in p) or "hide" in p
            pins.append(
                {
                    "type": p[1],
                    "x": float(at[1]),
                    "y": float(at[2]),
                    "angle": float(at[3]),
                    "name": str(find(p, "name")[1]),
                    "number": str(find(p, "number")[1]),
                    "hidden": hidden,
                }
            )
    return pins


def rot(x: float, y: float, deg: float) -> tuple[float, float]:
    a = math.radians(deg)
    return (x * math.cos(a) - y * math.sin(a), x * math.sin(a) + y * math.cos(a))


def r2(v: float) -> float:
    return round(v, 4) + 0.0


# --------------------------------------------------------------------------
# circuit description


@dataclass
class Part:
    ref: str
    lib_id: str
    value: str
    x: float
    y: float
    rot: int = 0
    nets: dict[str, str | None] = field(default_factory=dict)
    footprint: str = ""
    fields: dict[str, str] = field(default_factory=dict)


@dataclass
class Sheet:
    file: str
    name: str
    title: str
    paper: str = "A4"
    parts: list[Part] = field(default_factory=list)
    notes: list[tuple[float, float, str]] = field(default_factory=list)
    flags: list[tuple[float, float, str]] = field(default_factory=list)  # PWR_FLAG

    def add(self, ref, lib_id, value, x, y, rot=0, footprint="", **nets):
        """Places a part; pins are given by number or name (`p1=...`, `GND=...`)."""
        self.parts.append(Part(ref, lib_id, value, x, y, rot, dict(nets), footprint))

    def note(self, x, y, text):
        self.notes.append((x, y, text))

    def flag(self, x, y, net):
        self.flags.append((x, y, net))


def u(n: float) -> float:
    """Grid units of 1.27 mm, so every pin lands on KiCad's grid."""
    return round(n * 1.27, 4)


def two(sheet: Sheet, ref, lib_id, value, x, y, a, b, horizontal=True, footprint=""):
    """Two-pin part; `a` on pin 1 (left or top), `b` on pin 2."""
    sheet.add(ref, lib_id, value, x, y, 90 if horizontal else 0, footprint, p1=a, p2=b)


def diode(sheet: Sheet, ref, lib_id, value, x, y, anode, cathode, vertical=False):
    """Diode drawn anode→cathode left to right (or top to bottom)."""
    sheet.add(ref, lib_id, value, x, y, 90 if vertical else 180, p1=cathode, p2=anode)


def build() -> list[Sheet]:
    sheets = []

    # 1 ----------------------------------------------------------------------
    s = Sheet("power.kicad_sch", "1 Alimentazione", "Alimentazione e protezioni")
    s.note(u(8), u(10), "Alimentazione e protezioni")
    s.note(
        u(8),
        u(14),
        "+12V_KEY: +12 V sotto chiave dal cablaggio della moto (filo Br [DA VERIFICARE]).\n"
        "Catena: fusibile 2 A → P-MOSFET contro l'inversione → TVS → filtro LC → buck 5 V.\n"
        "Il Pico è alimentato attraverso D103 con 470 µF di riserva per salvare i km a chiave tolta.",
    )
    s.add("J101", "Connector_Generic:Conn_01x06", "Superseal 1.5 6p (cablaggio)", u(20), u(50),
          p1="+12V_KEY", p2="GND", p3="BIKE_NEUTRAL", p4="BIKE_HIGH_BEAM", p5="BIKE_TURN_L", p6="BIKE_TURN_R")
    s.add("J102", "Connector_Generic:Conn_01x04", "Superseal 1.5 4p (cablaggio)", u(20), u(80),
          p1="BIKE_TACH", p2="BIKE_COOLANT", p3="BIKE_ILLUM", p4="GND")
    two(s, "F101", "Device:Fuse", "2A", u(52), u(40), "+12V_KEY", "+12V_F")
    s.add("Q101", "Transistor_FET:IRF4905", "IRF4905", u(80), u(42), 90,
          D="+12V_F", S="+12V_P", G="Q_GATE")
    two(s, "R101", "Device:R", "10k", u(78), u(66), "Q_GATE", "GND", horizontal=False)
    s.add("D101", "Device:D_Zener", "12V", u(92), u(58), 270, p1="+12V_P", p2="Q_GATE")
    s.add("D102", "Device:D_Zener", "SMCJ24A", u(110), u(58), 270, p1="+12V_P", p2="GND")
    two(s, "L101", "Device:L", "22uH 2A", u(128), u(40), "+12V_P", "+12V_FILT")
    two(s, "C101", "Device:C_Polarized", "100uF 50V", u(144), u(58), "+12V_FILT", "GND", horizontal=False)
    two(s, "C102", "Device:C", "1uF 50V", u(156), u(58), "+12V_FILT", "GND", horizontal=False)
    s.add("U101", "Connector_Generic:Conn_01x05", "Pololu D36V28F5", u(190), u(46),
          p1="+12V_FILT", p2="GND", p3="+5V", p4=None, p5=None)
    s.note(u(178), u(56), "Modulo buck 5 V 3,2 A, ingresso fino a 50 V.\n1 VIN · 2 GND · 3 VOUT · 4 EN · 5 PG")
    diode(s, "D103", "Diode:1N5819", "1N5819", u(150), u(90), "+5V", "VSYS_PICO")
    two(s, "C103", "Device:C_Polarized", "470uF 10V", u(170), u(98), "VSYS_PICO", "GND", horizontal=False)
    s.add("J103", "Connector_Generic:Conn_01x02", "5V al Pi Zero 2 W", u(196), u(88), p1="+5V", p2="GND")
    s.flag(u(130), u(84), "+5V")
    s.flag(u(180), u(84), "VSYS_PICO")
    sheets.append(s)

    # 2 ----------------------------------------------------------------------
    s = Sheet("inputs.kicad_sch", "2 Spie", "Ingressi delle spie (optoaccoppiatori)", "A3")
    s.note(u(8), u(10), "Ingressi delle spie")
    s.note(
        u(8),
        u(14),
        "Abbaglianti, frecce e illuminazione sono attivi a +12 V: 4,7 kΩ → LED dell'EL817 → massa.\n"
        "La folle è attiva verso massa (il contatto del cambio chiude il filo a massa):\n"
        "+12V_P → 3,3 kΩ → LED → filo della folle. Uscite attive basse con pull-up interno del Pico.\n"
        "Colori dei fili e logica della folle [DA VERIFICARE] con i rilievi sulla moto.",
    )
    chans = [
        ("HIGH_BEAM", "Abbaglianti"),
        ("TURN_L", "Freccia sinistra"),
        ("TURN_R", "Freccia destra"),
        ("ILLUM", "Illuminazione"),
    ]
    for i, (net, label) in enumerate(chans):
        k = i + 1
        y = u(40 + i * 34)
        bike = f"BIKE_{net}"
        a = f"{net}_LED"
        s.note(u(10), y - u(8), label)
        two(s, f"R2{k}1", "Device:R", "10k", u(24), y + u(8), bike, "GND", horizontal=False)
        two(s, f"R2{k}2", "Device:R", "4.7k", u(44), y, bike, a)
        diode(s, f"D2{k}1", "Diode:1N4148", "1N4148", u(64), y + u(10), "GND", a, vertical=True)
        s.parts[-1].rot = 270  # cathode up, towards the LED anode
        two(s, f"C2{k}1", "Device:C", "100n", u(78), y + u(10), a, "GND", horizontal=False)
        s.add(f"U2{k}1", "Isolator:EL817", "EL817", u(104), y + u(2), p1=a, p2="GND", p4=net, p3="GND")
    y = u(40 + 4 * 34)
    s.note(u(10), y - u(8), "Folle (attiva verso massa)")
    two(s, "R251", "Device:R", "3.3k", u(44), y, "+12V_P", "NEUTRAL_LED")
    diode(s, "D251", "Diode:1N4148", "1N4148", u(64), y + u(10), "BIKE_NEUTRAL", "NEUTRAL_LED", vertical=True)
    s.parts[-1].rot = 270
    two(s, "C251", "Device:C", "100n", u(78), y + u(10), "NEUTRAL_LED", "BIKE_NEUTRAL", horizontal=False)
    s.add("U251", "Isolator:EL817", "EL817", u(104), y + u(2), p1="NEUTRAL_LED", p2="BIKE_NEUTRAL", p4="NEUTRAL", p3="GND")
    s.note(u(150), u(40), "Uscite verso il Pico\nNEUTRAL → GP6\nHIGH_BEAM → GP7\nTURN_L → GP8\nTURN_R → GP9\nILLUM → GP10")
    sheets.append(s)

    # 3 ----------------------------------------------------------------------
    s = Sheet("tach.kicad_sch", "3 Contagiri", "Contagiri (negativo bobina)")
    s.note(u(8), u(10), "Contagiri")
    s.note(
        u(8),
        u(14),
        "Il negativo della bobina ha picchi di alcune centinaia di volt a ogni scintilla.\n"
        "1N4007 lascia passare solo i positivi, 2 × 22 kΩ 1 W limitano la corrente, lo zener\n"
        "da 15 V e il 2,2 nF tagliano il picco; l'H11L1 (trigger di Schmitt) dà un fronte pulito.\n"
        "Filo del contagiri e forma d'onda [DA VERIFICARE] con l'oscilloscopio.",
    )
    diode(s, "D301", "Diode:1N4007", "1N4007", u(30), u(50), "BIKE_TACH", "TACH_1")
    two(s, "R301", "Device:R", "22k 1W", u(50), u(50), "TACH_1", "TACH_2")
    two(s, "R302", "Device:R", "22k 1W", u(70), u(50), "TACH_2", "TACH_3")
    s.add("D302", "Device:D_Zener", "1N4744A 15V", u(90), u(66), 270, p1="TACH_3", p2="GND")
    two(s, "C301", "Device:C", "2.2n", u(104), u(66), "TACH_3", "GND", horizontal=False)
    two(s, "R303", "Device:R", "2.2k", u(120), u(50), "TACH_3", "TACH_LED")
    s.add("U301", "Isolator:H11L1", "H11L1", u(150), u(56), p1="TACH_LED", p2="GND", p4="IGN", p5="GND", p6="+3V3")
    two(s, "R304", "Device:R", "4.7k", u(176), u(44), "+3V3", "IGN", horizontal=False)
    two(s, "C302", "Device:C", "100n", u(190), u(44), "+3V3", "GND", horizontal=False)
    s.note(u(170), u(70), "IGN → GP3, attivo basso")
    sheets.append(s)

    # 4 ----------------------------------------------------------------------
    s = Sheet("wheel.kicad_sch", "4 Ruota", "Sensore ruota (LJ12A3)")
    s.note(u(8), u(10), "Sensore di velocità")
    s.note(
        u(8),
        u(14),
        "LJ12A3-4-Z/BX induttivo NPN normalmente aperto: davanti a un bullone l'uscita va a massa.\n"
        "+12V_P → 2,2 kΩ → LED dell'EL817 → uscita del sensore. Pull-up esterno da 4,7 kΩ per fronti veloci.\n"
        "Colori del sensore: marrone +12 V, blu massa, nero segnale.",
    )
    s.add("J401", "Connector_Generic:Conn_01x03", "Deutsch DTM 3p (sensore)", u(24), u(52),
          p1="+12V_P", p2="WHEEL_SIG", p3="GND")
    two(s, "C401", "Device:C", "100n", u(44), u(62), "+12V_P", "GND", horizontal=False)
    two(s, "R401", "Device:R", "2.2k", u(64), u(44), "+12V_P", "WHEEL_LED")
    diode(s, "D401", "Diode:1N4148", "1N4148", u(84), u(56), "WHEEL_SIG", "WHEEL_LED", vertical=True)
    s.parts[-1].rot = 270
    s.add("U401", "Isolator:EL817", "EL817", u(112), u(48), p1="WHEEL_LED", p2="WHEEL_SIG", p4="WHEEL", p3="GND")
    two(s, "R402", "Device:R", "4.7k", u(140), u(40), "+3V3", "WHEEL", horizontal=False)
    s.note(u(132), u(64), "WHEEL → GP2, attivo basso")
    sheets.append(s)

    # 5 ----------------------------------------------------------------------
    s = Sheet("analog.kicad_sch", "5 Analogici", "Ingressi analogici")
    s.note(u(8), u(10), "Ingressi analogici")
    s.note(
        u(8),
        u(14),
        "Batteria: partitore 100k/22k (18,3 V → 3,3 V), filtro 100 nF, clamp BAT54S verso 0 e 3,3 V.\n"
        "Acqua: il sensore NTC va a massa nel motore; pull-up da 100 Ω verso 3,3 V, poi 1 kΩ e clamp.\n"
        "Luce: modulo TEMT6000 alimentato a 3,3 V.",
    )
    two(s, "R501", "Device:R", "100k", u(24), u(44), "+12V_P", "VBAT_SENSE", horizontal=False)
    two(s, "R502", "Device:R", "22k", u(24), u(64), "VBAT_SENSE", "GND", horizontal=False)
    two(s, "C501", "Device:C", "100n", u(40), u(64), "VBAT_SENSE", "GND", horizontal=False)
    s.add("D501", "Diode:BAT54S", "BAT54S", u(62), u(52), A="GND", K="+3V3", COM="VBAT_SENSE")
    s.note(u(48), u(74), "VBAT_SENSE → GP26 (ADC0)")

    two(s, "R503", "Device:R", "100 0.25W", u(96), u(44), "+3V3", "BIKE_COOLANT", horizontal=False)
    two(s, "R504", "Device:R", "1k", u(114), u(54), "BIKE_COOLANT", "COOLANT_SENSE")
    two(s, "C502", "Device:C", "100n", u(130), u(64), "COOLANT_SENSE", "GND", horizontal=False)
    s.add("D502", "Diode:BAT54S", "BAT54S", u(150), u(52), A="GND", K="+3V3", COM="COOLANT_SENSE")
    s.note(u(118), u(74), "COOLANT_SENSE → GP27 (ADC1)")

    s.add("J501", "Connector_Generic:Conn_01x03", "TEMT6000 (modulo)", u(176), u(52),
          p1="+3V3", p2="GND", p3="LIGHT_RAW")
    two(s, "R505", "Device:R", "1k", u(196), u(54), "LIGHT_RAW", "LIGHT_SENSE")
    two(s, "C503", "Device:C", "100n", u(212), u(64), "LIGHT_SENSE", "GND", horizontal=False)
    s.add("D503", "Diode:BAT54S", "BAT54S", u(196), u(80), A="GND", K="+3V3", COM="LIGHT_SENSE")
    s.note(u(176), u(92), "LIGHT_SENSE → GP28 (ADC2)")
    sheets.append(s)

    # 6 ----------------------------------------------------------------------
    s = Sheet("mcu.kicad_sch", "6 MCU", "Pico, FRAM e pulsanti", "A3")
    s.note(u(8), u(10), "Microcontrollore, memoria e pulsanti")
    s.note(
        u(8),
        u(14),
        "Raspberry Pi Pico (o Pico 2): stessa piedinatura, stesso firmware.\n"
        "FRAM MB85RC256V all'indirizzo 0x50 (A0–A2 a massa), I²C a 1 MHz.\n"
        "Pulsanti MODE e SET verso massa, 1 kΩ + 100 nF contro le scariche e i rimbalzi.",
    )
    gp = {
        2: "WHEEL", 3: "IGN", 4: "I2C_SDA", 5: "I2C_SCL", 6: "NEUTRAL", 7: "HIGH_BEAM", 8: "TURN_L",
        9: "TURN_R", 10: "ILLUM", 11: "BTN_MODE", 12: "BTN_SET", 15: "BL_PWM",
    }
    pico = {f"GPIO{n}": gp.get(n) for n in [*range(0, 23)]}
    pico.update({"GPIO26_ADC0": "VBAT_SENSE", "GPIO27_ADC1": "COOLANT_SENSE", "GPIO28_ADC2": "LIGHT_SENSE"})
    pico.update({"RUN": None, "3V3_EN": None, "ADC_VREF": None, "VBUS": None})
    pico.update({"VSYS": "VSYS_PICO", "3V3": "+3V3", "GND": "GND", "AGND": "GND"})
    s.add("U601", "MCU_Module:RaspberryPi_Pico", "Raspberry Pi Pico", u(130), u(120), **pico)
    s.add("U602", "Memory_EEPROM:24LC256", "MB85RC256V (FRAM)", u(250), u(80),
          A0="GND", A1="GND", A2="GND", GND="GND", SDA="I2C_SDA", SCL="I2C_SCL", WP="GND", VCC="+3V3")
    two(s, "R601", "Device:R", "2.2k", u(280), u(64), "+3V3", "I2C_SDA", horizontal=False)
    two(s, "R602", "Device:R", "2.2k", u(292), u(64), "+3V3", "I2C_SCL", horizontal=False)
    two(s, "C601", "Device:C", "100n", u(226), u(64), "+3V3", "GND", horizontal=False)
    s.note(u(222), u(104), "FRAM su breakout (Adafruit 1895):\nil connettore porta VCC, GND, SDA, SCL.")

    s.add("J601", "Connector_Generic:Conn_01x03", "Pulsanti (contenitore)", u(230), u(140),
          p1="BTN_MODE_SW", p2="BTN_SET_SW", p3="GND")
    s.add("SW601", "Switch:SW_Push", "MODE", u(210), u(160), p1="BTN_MODE_SW", p2="GND")
    s.add("SW602", "Switch:SW_Push", "SET", u(210), u(176), p1="BTN_SET_SW", p2="GND")
    two(s, "R603", "Device:R", "1k", u(260), u(156), "BTN_MODE_SW", "BTN_MODE")
    two(s, "C602", "Device:C", "100n", u(280), u(164), "BTN_MODE", "GND", horizontal=False)
    two(s, "R604", "Device:R", "1k", u(260), u(180), "BTN_SET_SW", "BTN_SET")
    two(s, "C603", "Device:C", "100n", u(280), u(188), "BTN_SET", "GND", horizontal=False)
    sheets.append(s)

    # 7 ----------------------------------------------------------------------
    s = Sheet("backlight.kicad_sch", "7 Retroilluminazione e Pi", "Retroilluminazione e collegamento al Pi")
    s.note(u(8), u(10), "Retroilluminazione e USB verso il Pi")
    s.note(
        u(8),
        u(14),
        "Il Pico comanda il TPS61169 in PWM a 25 kHz (GP15). 100 kΩ verso massa: finché il\n"
        "firmware non parte la retroilluminazione resta spenta (fail-dark).\n"
        "Display Newhaven: LED 180 mA a 9,3 V. Pin del modulo e del display [DA VERIFICARE].\n"
        "USB: solo D+, D− e massa. VBUS non collegato: il Pico si alimenta da VSYS.",
    )
    two(s, "R701", "Device:R", "1k", u(30), u(50), "BL_PWM", "BL_CTRL")
    two(s, "R702", "Device:R", "100k", u(48), u(60), "BL_CTRL", "GND", horizontal=False)
    s.add("U701", "Connector_Generic:Conn_01x05", "TPS61169 (Adafruit 6354)", u(80), u(54),
          p1="+5V", p2="GND", p3="BL_CTRL", p4="BL_LED_A", p5="BL_LED_K")
    s.note(u(70), u(68), "1 VIN · 2 GND · 3 CTRL · 4 LED+ · 5 LED−")
    s.add("J701", "Connector_Generic:Conn_01x02", "Retroilluminazione display", u(124), u(52),
          p1="BL_LED_A", p2="BL_LED_K")
    s.add("J702", "Connector_Generic:Conn_01x04", "USB del Pico (TP2/TP3)", u(40), u(100),
          p1=None, p2="USB_DM", p3="USB_DP", p4="GND")
    s.add("J703", "Connector_Generic:Conn_01x04", "USB del Pi Zero 2 W", u(110), u(100),
          p1=None, p2="USB_DM", p3="USB_DP", p4="GND")
    s.note(u(24), u(116), "1 VBUS (non collegato) · 2 D− · 3 D+ · 4 GND\nDoppino intrecciato corto, ferrite sul cavo.")
    s.note(u(150), u(96), "Il display si collega al Pi con l'Adafruit DPI Kippah\n(connettore FFC 40 poli, fuori da questa scheda).")
    sheets.append(s)

    # 8 ----------------------------------------------------------------------
    s = Sheet("leds.kicad_sch", "8 LED di backup", "LED di backup")
    s.note(u(8), u(10), "LED di backup")
    s.note(
        u(8),
        u(14),
        "Collegati direttamente alle linee a 12 V della moto: si accendono anche a elettronica\n"
        "spenta. Le frecce usano una sola coppia di fili come la spia originale (Ch–Dg):\n"
        "la freccia accesa alimenta il suo LED, l'altra linea va a massa attraverso le lampadine.\n"
        "Il 1N4148 in serie protegge ogni LED dai 12 V inversi.",
    )
    two(s, "R801", "Device:R", "1.5k", u(30), u(44), "BIKE_TURN_L", "LED_L_D")
    diode(s, "D805", "Diode:1N4148", "1N4148", u(50), u(44), "LED_L_D", "LED_L_A")
    s.add("D801", "Device:LED", "verde (freccia SX)", u(70), u(44), 180, p1="BIKE_TURN_R", p2="LED_L_A")
    two(s, "R802", "Device:R", "1.5k", u(30), u(64), "BIKE_TURN_R", "LED_R_D")
    diode(s, "D806", "Diode:1N4148", "1N4148", u(50), u(64), "LED_R_D", "LED_R_A")
    s.add("D802", "Device:LED", "verde (freccia DX)", u(70), u(64), 180, p1="BIKE_TURN_L", p2="LED_R_A")
    two(s, "R803", "Device:R", "1.5k", u(120), u(44), "BIKE_HIGH_BEAM", "LED_HB_A")
    s.add("D803", "Device:LED", "blu (abbaglianti)", u(140), u(44), 180, p1="GND", p2="LED_HB_A")
    two(s, "R804", "Device:R", "1.5k", u(120), u(64), "+12V_P", "LED_N_A")
    s.add("D804", "Device:LED", "verde (folle)", u(140), u(64), 180, p1="BIKE_NEUTRAL", p2="LED_N_A")
    sheets.append(s)
    return sheets


# --------------------------------------------------------------------------
# writer


def effects(size=1.27, justify=None, hide=False):
    e = ["effects", ["font", ["size", size, size]]]
    if justify:
        e.append(["justify", *justify.split()])
    if hide:
        e.append(["hide", "yes"])
    return e


class Writer:
    def __init__(self, lib: Library, sheets: list[Sheet]):
        self.lib = lib
        self.sheets = sheets
        self.root_uuid = uid("root")
        self.pwr = 0
        # which sheets use each net decides between local and global labels
        self.net_sheets: dict[str, set[str]] = defaultdict(set)
        self.net_pins: dict[str, list[str]] = defaultdict(list)
        for sh in sheets:
            for p in sh.parts:
                for pin in self.resolve(p):
                    if pin["net"]:
                        self.net_sheets[pin["net"]].add(sh.file)
                        self.net_pins[pin["net"]].append(f"{p.ref}.{pin['number']}")

    def resolve(self, p: Part) -> list[dict]:
        """Pins of a part with their net; every pin must be accounted for."""
        sym = self.lib.symbol(p.lib_id)
        pins = sym_pins(sym)
        used = set()
        out = []
        for pin in pins:
            if pin["type"] == "no_connect":
                continue
            keys = [f"p{pin['number']}", pin["number"], pin["name"]]
            key = next((k for k in keys if k in p.nets), None)
            if key is None:
                raise SystemExit(f"{p.ref}: pin {pin['number']} ({pin['name']}) has no net")
            used.add(key)
            out.append({**pin, "net": p.nets[key]})
        extra = set(p.nets) - used
        if extra:
            raise SystemExit(f"{p.ref}: unknown pins {sorted(extra)}")
        return out

    # geometry --------------------------------------------------------------

    @staticmethod
    def pin_end(p: Part, pin: dict) -> tuple[float, float, float]:
        """Schematic position of the pin's connection point and outward angle."""
        dx, dy = rot(pin["x"], pin["y"], p.rot)
        out = (pin["angle"] + 180 + p.rot) % 360
        return r2(p.x + dx), r2(p.y - dy), out

    @staticmethod
    def step(x, y, angle, d):
        a = math.radians(angle)
        return r2(x + d * math.cos(a)), r2(y - d * math.sin(a))

    # elements --------------------------------------------------------------

    def wire(self, key, x1, y1, x2, y2):
        return [
            "wire",
            ["pts", ["xy", x1, y1], ["xy", x2, y2]],
            ["stroke", ["width", 0], ["type", "default"]],
            ["uuid", uid(key, "wire")],
        ]

    def label(self, key, net, x, y, angle, glob):
        just = "left" if angle in (0, 90) else "right"
        if glob:
            return [
                "global_label",
                Q(net),
                ["shape", "passive"],
                ["at", x, y, angle],
                ["fields_autoplaced", "yes"],
                ["effects", ["font", ["size", 1.27, 1.27]], ["justify", just]],
                ["uuid", uid(key, "label")],
                [
                    "property",
                    Q("Intersheetrefs"),
                    Q("${INTERSHEET_REFS}"),
                    ["at", x, y, 0],
                    ["effects", ["font", ["size", 1.27, 1.27]], ["justify", just], ["hide", "yes"]],
                ],
            ]
        return [
            "label",
            Q(net),
            ["at", x, y, angle],
            ["effects", ["font", ["size", 1.27, 1.27]], ["justify", just, "bottom"]],
            ["uuid", uid(key, "label")],
        ]

    def instance(self, sheet: Sheet, p: Part, sym: list, ref: str, value: str):
        path = f"/{self.root_uuid}/{uid('sheet', sheet.file)}"
        node = [
            "symbol",
            ["lib_id", Q(sym[1])],
            ["at", p.x, p.y, p.rot],
            ["unit", 1],
            ["exclude_from_sim", "no"],
            ["in_bom", "no" if ref.startswith("#") else "yes"],
            ["on_board", "yes"],
            ["dnp", "no"],
            ["uuid", uid(sheet.file, ref)],
        ]
        placed = self.field_spots(p, sym) if not ref.startswith("#") else {}
        if not placed and not ref.startswith("#"):
            # some symbols (the optocouplers) put reference and value on one
            # line, which a three-digit reference overruns: stack them instead
            lib_at = {str(pr[1]): find(pr, "at") for pr in find_all(sym, "property")}
            ra, va = lib_at["Reference"], lib_at["Value"]
            if float(ra[2]) == float(va[2]) and float(ra[1]) != float(va[1]):
                x0, y0 = r2(p.x + float(ra[1])), r2(p.y - float(ra[2]))
                placed = {"Reference": (x0, r2(y0 - 2.54), "left"), "Value": (x0, y0, "left")}
        for prop in find_all(sym, "property"):
            name = str(prop[1])
            val = {"Reference": ref, "Value": value}.get(name, str(prop[2]))
            if name == "Footprint" and p.footprint:
                val = p.footprint
            eff = copy.deepcopy(find(prop, "effects"))
            if name in placed:
                x, y, just = placed[name]
                eff = [x for x in eff if not (isinstance(x, list) and x[0] == "justify")]
                if just:
                    eff.append(["justify", just])
                # field angles are stored relative to the symbol rotation
                node.append(["property", Q(name), Q(val), ["at", x, y, p.rot % 180], eff])
                continue
            at = find(prop, "at")
            dx, dy = rot(float(at[1]), float(at[2]), p.rot)
            ang = (float(at[3]) + p.rot) % 180
            node.append(["property", Q(name), Q(val), ["at", r2(p.x + dx), r2(p.y - dy), int(ang)], eff])
        for f_name, f_val in p.fields.items():
            node.append(["property", Q(f_name), Q(f_val), ["at", p.x, p.y, 0], effects(hide=True)])
        seen = set()
        for pin in sym_pins(sym):
            if pin["number"] in seen:
                continue
            seen.add(pin["number"])
            node.append(["pin", Q(pin["number"]), ["uuid", uid(sheet.file, ref, "pin", pin["number"])]])
        node.append(
            ["instances", ["project", Q(PROJECT), ["path", Q(path), ["reference", Q(ref)], ["unit", 1]]]]
        )
        return node

    def field_spots(self, p: Part, sym: list) -> dict:
        """Reference and value positions for rotated or two-pin parts.

        Library positions only suit the symbol drawn upright; once rotated,
        reference and value land on top of each other.
        """
        pins = [pn for pn in sym_pins(sym) if pn["type"] != "no_connect"]
        if p.rot == 0 and len(pins) > 2:
            return {}
        ends = [self.pin_end(p, pn)[:2] for pn in pins]
        xs, ys = [e[0] for e in ends], [e[1] for e in ends]
        cx, cy = r2((min(xs) + max(xs)) / 2), r2((min(ys) + max(ys)) / 2)
        if len(pins) == 2 and ys[0] == ys[1]:  # horizontal
            return {"Reference": (cx, r2(cy - 2.794), None), "Value": (cx, r2(cy + 2.794), None)}
        if len(pins) == 2:  # vertical
            return {"Reference": (r2(cx + 2.794), r2(cy - 1.27), "left"), "Value": (r2(cx + 2.794), r2(cy + 1.27), "left")}
        top = min(ys)
        return {"Reference": (cx, r2(top - 8.89), None), "Value": (cx, r2(top - 6.35), None)}

    def power(self, sheet: Sheet, net: str, x, y, angle, key: str, lib_syms: dict):
        lib_id = POWER_NETS.get(net, "power:PWR_FLAG")
        sym = lib_syms.setdefault(lib_id, self.lib.symbol(lib_id))
        if net == "GND":
            r = {270: 0, 0: 90, 90: 180, 180: 270}[angle]
        else:
            r = {90: 0, 180: 90, 270: 180, 0: 270}[angle]
        self.pwr += 1
        ref = f"#PWR{self.pwr:03d}" if lib_id != "power:PWR_FLAG" else f"#FLG{self.pwr:03d}"
        part = Part(ref, lib_id, net if lib_id != "power:PWR_FLAG" else "PWR_FLAG", x, y, r)
        return self.instance(sheet, part, sym, ref, part.value)

    def text(self, key, x, y, txt, size):
        return [
            "text",
            Q(txt),
            ["exclude_from_sim", "no"],
            ["at", x, y, 0],
            ["effects", ["font", ["size", size, size]], ["justify", "left", "top"]],
            ["uuid", uid(key, "text")],
        ]

    def title_block(self, title: str, page_comment: str):
        return [
            "title_block",
            ["title", Q(title)],
            ["date", Q(DATE)],
            ["rev", Q(REV)],
            ["company", Q("tdm-dash · CERN-OHL-P-2.0")],
            ["comment", 1, Q(page_comment)],
            ["comment", 2, Q("Generato da hardware/kicad/generate.py: non modificare a mano")],
        ]

    def sheet_file(self, sheet: Sheet) -> list:
        lib_syms: dict[str, list] = {}
        body: list = []
        for p in sheet.parts:
            sym = lib_syms.setdefault(p.lib_id, self.lib.symbol(p.lib_id))
            body.append(self.instance(sheet, p, sym, p.ref, p.value))
            done = set()
            for pin in self.resolve(p):
                x, y, out = self.pin_end(p, pin)
                if (x, y) in done:
                    continue  # stacked pins (the Pico's GND)
                done.add((x, y))
                key = f"{sheet.file}/{p.ref}/{pin['number']}"
                net = pin["net"]
                if net is None:
                    body.append(["no_connect", ["at", x, y], ["uuid", uid(key, "nc")]])
                    continue
                x2, y2 = self.step(x, y, out, STUB)
                body.append(self.wire(key, x, y, x2, y2))
                if net in POWER_NETS:
                    body.append(self.power(sheet, net, x2, y2, out, key, lib_syms))
                else:
                    glob = len(self.net_sheets[net]) > 1
                    body.append(self.label(key, net, x2, y2, out, glob))
        for i, (x, y, net) in enumerate(sheet.flags):
            key = f"{sheet.file}/flag/{i}"
            body.append(self.power(sheet, "PWR_FLAG", x, y, 90, key, lib_syms))
            x2, y2 = r2(x), r2(y + STUB)
            body.append(self.wire(key, x, y, x2, y2))
            if net in POWER_NETS:
                body.append(self.power(sheet, net, x2, y2, 270, key + "/p", lib_syms))
            else:
                body.append(self.label(key, net, x2, y2, 270, len(self.net_sheets[net]) > 1))
        for i, (x, y, txt) in enumerate(sheet.notes):
            body.append(self.text(f"{sheet.file}/note/{i}", x, y, txt, 2.54 if i == 0 else 1.5))
        page = self.sheets.index(sheet) + 2
        return [
            "kicad_sch",
            ["version", VERSION],
            ["generator", Q("eeschema")],
            ["generator_version", Q("9.0")],
            ["uuid", uid("sheet", sheet.file)],
            ["paper", Q(sheet.paper)],
            self.title_block(sheet.title, f"Foglio {page - 1} di {len(self.sheets)}"),
            ["lib_symbols", *[lib_syms[k] for k in sorted(lib_syms)]],
            *body,
            ["embedded_fonts", "no"],
        ]

    def root_file(self) -> list:
        body = []
        for i, sh in enumerate(self.sheets):
            col, row = i % 4, i // 4
            x, y = u(20 + col * 72), u(50 + row * 40)
            w, h = u(60), u(20)
            body.append(
                [
                    "sheet",
                    ["at", x, y],
                    ["size", w, h],
                    ["exclude_from_sim", "no"],
                    ["in_bom", "yes"],
                    ["on_board", "yes"],
                    ["dnp", "no"],
                    ["fields_autoplaced", "yes"],
                    ["stroke", ["width", 0.1524], ["type", "solid"]],
                    ["fill", ["color", 0, 0, 0, 0.0]],
                    ["uuid", uid("sheet", sh.file)],
                    [
                        "property",
                        Q("Sheetname"),
                        Q(sh.name),
                        ["at", x, r2(y - 0.7116), 0],
                        ["effects", ["font", ["size", 1.27, 1.27]], ["justify", "left", "bottom"]],
                    ],
                    [
                        "property",
                        Q("Sheetfile"),
                        Q(sh.file),
                        ["at", x, r2(y + h + 0.5846), 0],
                        ["effects", ["font", ["size", 1.27, 1.27]], ["justify", "left", "top"]],
                    ],
                    ["instances", ["project", Q(PROJECT), ["path", Q(f"/{self.root_uuid}"), ["page", Q(str(i + 2))]]]],
                ]
            )
        body.append(
            self.text(
                "root/title",
                u(20),
                u(20),
                "tdm-dash · modulo sensori (Pico) per Yamaha TDM 850 (3VD)",
                3.0,
            )
        )
        body.append(
            self.text(
                "root/desc",
                u(20),
                u(28),
                "Schema generato da hardware/kicad/generate.py. I colori dei fili e la piedinatura\n"
                "del connettore originale sono [DA VERIFICARE]: vedi docs/ricerca/impianto-originale.md.\n"
                "Capitolo 04 del manuale: spiegazione di ogni foglio e calcolo dei componenti.",
                1.5,
            )
        )
        return [
            "kicad_sch",
            ["version", VERSION],
            ["generator", Q("eeschema")],
            ["generator_version", Q("9.0")],
            ["uuid", self.root_uuid],
            ["paper", Q("A4")],
            self.title_block("tdm-dash · modulo sensori", f"{len(self.sheets)} fogli"),
            ["lib_symbols"],
            *body,
            ["sheet_instances", ["path", Q("/"), ["page", Q("1")]]],
            ["embedded_fonts", "no"],
        ]


def project_file() -> dict:
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
        "schematic": {"legacy_lib_dir": "", "legacy_lib_list": []},
        "sheets": [],
        "boards": [],
        "text_variables": {},
    }


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--symbols", default=os.environ.get("KICAD9_SYMBOL_DIR", "/usr/share/kicad/symbols"))
    ap.add_argument("--out", default=os.path.dirname(os.path.abspath(__file__)))
    a = ap.parse_args()

    sheets = build()
    w = Writer(Library(a.symbols), sheets)
    lonely = {n: p for n, p in w.net_pins.items() if len(p) < 2}
    if lonely:
        raise SystemExit(f"nets with a single pin: {lonely}")

    def write(name, tree):
        with open(os.path.join(a.out, name), "w", encoding="utf-8") as f:
            f.write(dump(tree) + "\n")

    write(f"{PROJECT}.kicad_sch", w.root_file())
    for sh in sheets:
        write(sh.file, w.sheet_file(sh))
    with open(os.path.join(a.out, f"{PROJECT}.kicad_pro"), "w", encoding="utf-8") as f:
        json.dump(project_file(), f, indent=2)
        f.write("\n")
    nets = sorted(w.net_pins)
    print(f"{len(sheets)} fogli, {sum(len(s.parts) for s in sheets)} componenti, {len(nets)} reti")
    return 0


if __name__ == "__main__":
    sys.exit(main())
