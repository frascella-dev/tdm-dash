"""Sheet 6: Raspberry Pi Pico, FRAM and buttons."""

from schgen.draw import LEFT, RIGHT, Sheet

TITLE = "Pico, FRAM and buttons"

# Pico GPIOs and their nets (firmware/src/main.rs; checked by tools/check_pinmap.py)
GPIO = {2: "WHEEL", 3: "IGN", 4: "I2C_SDA", 5: "I2C_SCL", 6: "NEUTRAL", 7: "HIGH_BEAM", 8: "TURN_L",
        9: "TURN_R", 10: "ILLUM", 11: "BTN_MODE", 12: "BTN_SET", 15: "BL_PWM"}
PORTS_IN = ("WHEEL", "IGN", "NEUTRAL", "HIGH_BEAM", "TURN_L", "TURN_R", "ILLUM")
LOCAL = ("I2C_SDA", "I2C_SCL", "BTN_MODE", "BTN_SET")


def build(lib) -> Sheet:
    s = Sheet(lib, 6, "6 MCU", TITLE, "mcu.kicad_sch")
    s.heading(
        "Raspberry Pi Pico, FRAM and buttons",
        [
            "Raspberry Pi Pico or Pico 2: same pinout, same firmware. Unused pins are left open.",
            "The FRAM keeps the odometer (docs/adr/0004); MODE and SET are filtered against bounce and discharges.",
        ],
    )
    nets = {f"GPIO{n}": GPIO.get(n) for n in range(23)}
    nets.update({"GPIO26_ADC0": "VBAT_SENSE", "GPIO27_ADC1": "COOLANT_SENSE", "GPIO28_ADC2": "LIGHT_SENSE"})
    nets.update({"RUN": None, "3V3_EN": None, "ADC_VREF": None, "VBUS": None})
    nets.update({"VSYS": "VSYS_PICO", "3V3": "+3V3", "GND": "GND", "AGND": "GND"})
    u = s.part("U601", "MCU_Module:RaspberryPi_Pico", "Raspberry Pi Pico", (4000, 3800), nets=nets)
    u.fields = {"Reference": (400, -1600, "left"), "Value": (400, -1520, "left")}
    for n, net in GPIO.items():
        pin = u[f"GPIO{n}"]
        if net in LOCAL:
            s.wire(pin, (2700, pin.pos[1]))
            s.label(net, (2700, pin.pos[1]), LEFT)
        else:
            s.port(net, pin, 2300, shape="output" if net == "BL_PWM" else "input")
    for name in ("GPIO26_ADC0", "GPIO27_ADC1", "GPIO28_ADC2"):
        s.port(nets[name], u[name], 5700, shape="input")
    s.wire(u["AGND"], (5000, u["AGND"].pos[1]))
    s.gnd((5000, u["AGND"].pos[1]))
    s.rail("VSYS_PICO", u["VSYS"])
    s.rail("+3V3", u["3V3"])
    s.gnd(u["GND"])
    s.note(2200, 5300, "GP2 wheel and GP3 ignition are counted\nby the PIO, with edge timestamps")

    # FRAM -------------------------------------------------------------------
    f = s.part("U602", "tdm:FRAM_Breakout", "MB85RC256V (Adafruit 1895)", (8600, 2600),
               VCC="+3V3", GND="GND", SDA="I2C_SDA", SCL="I2C_SCL", WP="GND", A0="GND", A1="GND", A2="GND")
    f.fields = {"Reference": (100, -450, "left"), "Value": (100, -370, "left")}
    s.rail("+3V3", f["VCC"])
    s.gnd(f["GND"])
    s.wire(f["WP"], (9300, 2450), (9300, 2850))
    for pin in ("A0", "A1", "A2"):
        s.wire(f[pin], (9300, f[pin].pos[1]))
    s.gnd((9300, 2850), length=0)
    r601 = s.part("R601", "Device:R", "2.2k", (7400, 1850), anchor="2", p1="+3V3", p2="I2C_SDA")
    r602 = s.part("R602", "Device:R", "2.2k", (7600, 2950), anchor="2", p1="+3V3", p2="I2C_SCL")
    s.rail("+3V3", r601["1"])
    s.rail("+3V3", r602["1"])
    s.wire((6800, 2000), (7400, 2000), (7900, 2000), (7900, 2450), f["SDA"])
    s.wire(r601["2"], (7400, 2000))
    s.label("I2C_SDA", (6800, 2000), LEFT)
    s.wire((6800, 3100), (7600, 3100), (8000, 3100), (8000, 2550), f["SCL"])
    s.wire(r602["2"], (7600, 3100))
    s.label("I2C_SCL", (6800, 3100), LEFT)
    c601 = s.part("C601", "Device:C", "100n", (10300, 2600), anchor="1", p1="+3V3", p2="GND")
    s.rail("+3V3", c601["1"])
    s.gnd(c601["2"])
    s.note(7900, 3450, "I²C at 1 MHz, address 0x50 (A0-A2 to ground)")
    s.verify(7900, 3650, "[TO VERIFY] header pin order of the breakout")

    # buttons ----------------------------------------------------------------
    j601 = s.part("J601", "tdm:Conn_Buttons3", "to the buttons", (9400, 4300), anchor="1",
                  p1="BTN_MODE_SW", p2="BTN_SET_SW", p3="GND")
    filters = {}
    for net, y, rr, cc in (("BTN_MODE", 4300, "R603", "C602"), ("BTN_SET", 5100, "R604", "C603")):
        r = s.part(rr, "Device:R", "1k", (8200, y), rot=90, anchor="1", p1=net, p2=f"{net}_SW")
        c = s.part(cc, "Device:C", "100n", (7800, y + 150), anchor="1", p1=net, p2="GND")
        s.wire((7300, y), (7800, y), r["1"])
        s.wire((7800, y), c["1"])
        s.gnd(c["2"])
        s.label(net, (7300, y), LEFT)
        s.label(f"{net}_SW", (8600, y), RIGHT)  # on the wire to J601
        filters[net] = r
    s.wire(filters["BTN_MODE"]["2"], j601["1"])
    s.wire(filters["BTN_SET"]["2"], (9200, 5100), (9200, 4400), j601["2"])
    s.wire(j601["3"], (9300, 4500))
    s.gnd((9300, 4500))

    s.rect(9000, 5300, 10950, 6350)
    s.note(9050, 5350, "On the enclosure, through the harness")
    for ref, net, y in (("SW601", "BTN_MODE_SW", 5700), ("SW602", "BTN_SET_SW", 6100)):
        sw = s.part(ref, "Switch:SW_Push", "MODE" if ref == "SW601" else "SET", (10100, y), anchor="1",
                    p1=net, p2="GND")
        s.wire(sw["1"], (10000, y))
        s.label(net, (10000, y), LEFT)
        s.wire(sw["2"], (10600, y))
        s.gnd((10600, y))
    s.note(6900, 5850, "1 kΩ and 100 nF: debounce and\nprotection against discharges")
    return s
