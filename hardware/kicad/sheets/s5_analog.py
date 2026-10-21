"""Sheet 5: battery, coolant and light inputs to the Pico ADC."""

from schgen.draw import LEFT, Label, Sheet

TITLE = "Analog inputs"


def clamp(s: Sheet, ref: str, net: str, x: int, y: int):
    """BAT54S above the line at x: clamps the node to ground and 3.3 V."""
    d = s.part(ref, "Diode:BAT54S", "BAT54S", (x, y - 100), anchor="COM", A="GND", K="+3V3", COM=net)
    d.fields = {"Reference": (0, -230, ""), "Value": (0, -150, "")}
    s.wire(d["COM"], (x, y))
    s.wire(d["A"], (x - 400, y - 300))
    s.gnd((x - 400, y - 300), length=0)
    s.rail("+3V3", d["K"])
    return d


def build(lib) -> Sheet:
    s = Sheet(lib, 5, "5 Analog inputs", TITLE, "analog.kicad_sch")
    s.heading(
        "Analog inputs",
        [
            "Each input reaches its Pico ADC pin through a filter capacitor and a BAT54S that clamps it",
            "between ground and 3.3 V, so a spike on the bike wiring cannot reach the GPIO.",
        ],
    )
    # battery voltage --------------------------------------------------------
    y = 2200
    s.text(1000, y - 900, "Battery voltage  →  GP26 (ADC0)", size=60, bold=True)
    r501 = s.part("R501", "Device:R", "100k", (2200, y), rot=90, anchor="1", p1="+12V_P", p2="VBAT_SENSE")
    s.wire((1800, y), r501["1"])
    s.rail("+12V_P", (1800, y), length=0)
    r502 = s.part("R502", "Device:R", "22k", (2900, y + 200), anchor="1", p1="VBAT_SENSE", p2="GND")
    c501 = s.part("C501", "Device:C", "100n", (3500, y + 200), anchor="1", p1="VBAT_SENSE", p2="GND")
    clamp(s, "D501", "VBAT_SENSE", 4300, y)
    s.wire(r501["2"], (2900, y), (3500, y), (4300, y))
    s.wire((2900, y), r502["1"])
    s.wire((3500, y), c501["1"])
    s.gnd(r502["2"])
    s.gnd(c501["2"])
    s.port("VBAT_SENSE", (4300, y), 5300, shape="output")
    s.note(6600, y - 300, "100k / 22k: 18.3 V at the battery\ngives 3.3 V, the top of the ADC scale")

    # coolant temperature ----------------------------------------------------
    y = 4000
    s.text(1000, y - 900, "Coolant temperature  →  GP27 (ADC1)", size=60, bold=True)
    r503 = s.part("R503", "Device:R", "100 0.25W", (2000, y - 150), anchor="2", p1="+3V3", p2="BIKE_COOLANT")
    s.rail("+3V3", r503["1"])
    r504 = s.part("R504", "Device:R", "1k", (2500, y), rot=90, anchor="1", p1="BIKE_COOLANT", p2="COOLANT_SENSE")
    s.wire((1400, y), (2000, y), r504["1"])
    s.labels.append(Label("hier", "BIKE_COOLANT", (1400, y), LEFT, "input"))
    s.wire(r503["2"], (2000, y))
    c502 = s.part("C502", "Device:C", "100n", (3500, y + 200), anchor="1", p1="COOLANT_SENSE", p2="GND")
    clamp(s, "D502", "COOLANT_SENSE", 4300, y)
    s.wire(r504["2"], (3500, y), (4300, y))
    s.wire((3500, y), c502["1"])
    s.gnd(c502["2"])
    s.port("COOLANT_SENSE", (4300, y), 5300, shape="output")
    s.note(6600, y - 300, "The sender is an NTC to ground in the engine:\n"
                          "R503 pulls it up to 3.3 V, R504 limits the\ncurrent into the clamp")

    # ambient light ------------------------------------------------------------
    y = 5600
    s.text(1000, y - 900, "Ambient light  →  GP28 (ADC2)", size=60, bold=True)
    j501 = s.part("J501", "tdm:Conn_Light3", "to the TEMT6000", (2000, y - 100), anchor="1",
                  p1="+3V3", p2="GND", p3="LIGHT_RAW")
    s.wire(j501["1"], (2200, y - 100))
    s.rail("+3V3", (2200, y - 100))
    s.wire(j501["2"], (2100, y + 100))
    s.gnd((2100, y + 100))
    r505 = s.part("R505", "Device:R", "1k", (2500, y), rot=90, anchor="1", p1="LIGHT_RAW", p2="LIGHT_SENSE")
    s.chain(j501["3"], r505["1"])
    c503 = s.part("C503", "Device:C", "100n", (3500, y + 200), anchor="1", p1="LIGHT_SENSE", p2="GND")
    clamp(s, "D503", "LIGHT_SENSE", 4300, y)
    s.wire(r505["2"], (3500, y), (4300, y))
    s.wire((3500, y), c503["1"])
    s.gnd(c503["2"])
    s.port("LIGHT_SENSE", (4300, y), 5300, shape="output")
    s.note(6600, y - 1000, "TEMT6000 module behind the window,\nsupplied at 3.3 V")
    return s
