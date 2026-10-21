#!/usr/bin/env python3
"""Cross-checks the KiCad schematic against the firmware and the profile code.

Reads the netlist exported by KiCad 9

    kicad-cli sch export netlist --format kicadsexpr -o tdm-dash.net hardware/kicad/tdm-dash.kicad_sch

and checks that:

- every Pico GPIO used by `firmware/src/main.rs` carries the net the
  schematic expects for that function, and no other GPIO is connected;
- the battery divider in the schematic gives the ratio hard-coded as the
  `vbat_divider` default in `crates/tdm-logic/src/profile.rs`;
- the coolant pull-up matches `coolant_pullup_ohm` in the TDM profile.

    python3 tools/check_pinmap.py tdm-dash.net
"""

from __future__ import annotations

import argparse
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.dont_write_bytecode = True
sys.path.insert(0, os.path.join(ROOT, "hardware", "kicad"))
from sexpr import find, find_all, parse  # noqa: E402

PICO = "RaspberryPi_Pico"


def firmware_pins(src: str) -> dict[str, int]:
    """Net name -> GPIO number, read from the firmware's pin setup."""
    pins: dict[str, int] = {}

    def one(pattern: str) -> re.Match:
        m = re.search(pattern, src, re.S)
        if not m:
            raise SystemExit(f"firmware: pattern not found: {pattern}")
        return m

    edges = {"Wheel": "WHEEL", "Ignition": "IGN"}
    for gpio, src_name in re.findall(r"edge_task\(Input::new\(p\.PIN_(\d+),[^)]*\),\s*Src::(\w+)\)", src):
        pins[edges[src_name]] = int(gpio)
    m = one(r"I2c::new_blocking\(p\.I2C\d,\s*p\.PIN_(\d+),\s*p\.PIN_(\d+)")
    pins["I2C_SCL"], pins["I2C_SDA"] = int(m.group(1)), int(m.group(2))
    pins["BL_PWM"] = int(one(r"Pwm::new_output_\w\(p\.PWM_SLICE\d+,\s*p\.PIN_(\d+)").group(1))
    # same order as the tdm_proto::input bits
    inputs = re.findall(r"PIN_(\d+)", one(r"inputs:\s*\[\s*(Input::new.*?)\]").group(1))
    for net, gpio in zip(["NEUTRAL", "HIGH_BEAM", "TURN_L", "TURN_R", "ILLUM"], inputs, strict=True):
        pins[net] = int(gpio)
    pins["BTN_MODE"] = int(one(r"btn_mode:\s*Input::new\(p\.PIN_(\d+)").group(1))
    pins["BTN_SET"] = int(one(r"btn_set:\s*Input::new\(p\.PIN_(\d+)").group(1))
    adc = re.findall(r"PIN_(\d+)", one(r"ch:\s*\[\s*(AdcChannel::new_pin.*?)\]").group(1))
    for net, gpio in zip(["VBAT_SENSE", "COOLANT_SENSE", "LIGHT_SENSE"], adc, strict=True):
        pins[net] = int(gpio)
    return pins


def parse_value(v: str) -> float:
    m = re.match(r"([\d.]+)\s*([kKM]?)", v)
    if not m:
        raise SystemExit(f"cannot read resistor value {v!r}")
    return float(m.group(1)) * {"": 1, "k": 1e3, "K": 1e3, "M": 1e6}[m.group(2)]


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("netlist")
    a = ap.parse_args()

    with open(a.netlist, encoding="utf-8") as f:
        tree = parse(f.read())[0]
    values: dict[str, str] = {}
    pico_ref = None
    for comp in find_all(find(tree, "components"), "comp"):
        ref = str(find(comp, "ref")[1])
        values[ref] = str(find(comp, "value")[1])
        if str(find(find(comp, "libsource"), "part")[1]) == PICO:
            pico_ref = ref
    if pico_ref is None:
        raise SystemExit("no Raspberry Pi Pico in the netlist")

    gpio_net: dict[int, str] = {}
    net_nodes: dict[str, list[str]] = {}
    for net in find_all(find(tree, "nets"), "net"):
        name = str(find(net, "name")[1]).rsplit("/", 1)[-1]
        nodes = find_all(net, "node")
        net_nodes[name] = [f"{find(n, 'ref')[1]}.{find(n, 'pin')[1]}" for n in nodes]
        for n in nodes:
            if str(find(n, "ref")[1]) != pico_ref:
                continue
            func = find(n, "pinfunction")
            m = re.match(r"GPIO(\d+)", str(func[1])) if func else None
            if m and not name.startswith("unconnected-"):
                gpio_net[int(m.group(1))] = name

    with open(os.path.join(ROOT, "firmware", "src", "main.rs"), encoding="utf-8") as f:
        fw = firmware_pins(f.read())

    errors = []
    for net, gpio in sorted(fw.items(), key=lambda kv: kv[1]):
        got = gpio_net.get(gpio)
        status = "ok" if got == net else "ERRORE"
        print(f"  GP{gpio:<2}  firmware {net:<14} schema {got or '-':<14} {status}")
        if got != net:
            errors.append(f"GP{gpio}: firmware expects {net}, schematic has {got}")
    for gpio, net in sorted(gpio_net.items()):
        if gpio not in fw.values():
            errors.append(f"GP{gpio} is connected to {net} but the firmware does not use it")

    # battery divider: one resistor from +12V_P, one to GND
    def resistors_on(net: str) -> set[str]:
        return {n.split(".")[0] for n in net_nodes.get(net, []) if n.startswith("R")}

    sense = resistors_on("VBAT_SENSE")
    top = sense & resistors_on("+12V_P")
    bottom = sense & resistors_on("GND")
    if len(top) != 1 or len(bottom) != 1:
        errors.append(f"battery divider not found (top {top}, bottom {bottom})")
    else:
        rt, rb = parse_value(values[top.pop()]), parse_value(values[bottom.pop()])
        ratio = (rt + rb) / rb
        with open(os.path.join(ROOT, "crates", "tdm-logic", "src", "profile.rs"), encoding="utf-8") as f:
            m = re.search(r"vbat_divider:\s*([\d.]+)\s*/\s*([\d.]+)", f.read())
        code = float(m.group(1)) / float(m.group(2))
        print(f"  partitore batteria: schema {ratio:.4f}, codice {code:.4f}")
        if abs(ratio - code) > 1e-6:
            errors.append(f"battery divider {ratio:.4f} differs from vbat_divider {code:.4f}")

    pull = resistors_on("BIKE_COOLANT") & resistors_on("+3V3")
    with open(os.path.join(ROOT, "profiles", "yamaha-tdm850-3vd.toml"), encoding="utf-8") as f:
        prof = float(re.search(r"coolant_pullup_ohm\s*=\s*([\d.]+)", f.read()).group(1))
    if len(pull) != 1:
        errors.append(f"coolant pull-up not found ({pull})")
    else:
        ohm = parse_value(values[pull.pop()])
        print(f"  pull-up sensore acqua: schema {ohm:g} Ω, profilo {prof:g} Ω")
        if ohm != prof:
            errors.append(f"coolant pull-up {ohm:g} Ω differs from the profile ({prof:g} Ω)")

    for e in errors:
        print(f"ERRORE: {e}", file=sys.stderr)
    if not errors:
        print("schema, firmware e profilo concordano")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
