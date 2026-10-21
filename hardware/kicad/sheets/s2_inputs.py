"""Sheet 2: warning light inputs through optocouplers."""

from schgen.draw import NOTE_BLUE, Sheet

TITLE = "Warning light inputs (optocouplers)"
PITCH = 1200  # mils between channels


def opto_out(s: Sheet, u, net: str, yc: int):
    """Optocoupler LED cathode and emitter to ground, collector to the Pico."""
    s.wire(u["2"], (4100, yc + 200), (4100, yc + 500))
    s.gnd((4100, yc + 500))
    s.wire(u["3"], (4900, yc + 200), (4900, yc + 500))
    s.gnd((4900, yc + 500))
    s.port(net, u["4"], 5600, shape="output")


def build(lib) -> Sheet:
    s = Sheet(lib, 2, "2 Warning lights", TITLE, "inputs.kicad_sch")
    s.heading(
        "Warning light inputs",
        [
            "High beam, turn signals and illumination are active at +12 V: 4.7 kΩ and the EL817 LED to ground,",
            "10 kΩ to ground so an open wire reads off. Neutral is active to ground: the gearbox switch grounds",
            "the wire, so the LED is fed from +12V_P. Outputs are active low, with the Pico's internal pull-ups.",
        ],
    )
    channels = [
        ("NEUTRAL", "Neutral (active to ground)", "GP6", 5),
        ("HIGH_BEAM", "High beam", "GP7", 1),
        ("TURN_L", "Left turn signal", "GP8", 2),
        ("TURN_R", "Right turn signal", "GP9", 3),
        ("ILLUM", "Illumination (lights on)", "GP10", 4),
    ]
    for i, (net, title, gp, k) in enumerate(channels):
        yc = 1650 + i * PITCH
        bike = f"BIKE_{net}"
        led = f"{net}_LED"
        s.text(1000, yc - 330, f"{title}  →  {gp}", size=60, bold=True, color=NOTE_BLUE)
        if net == "NEUTRAL":
            # +12V_P → R → LED → bike wire, which the gearbox switch grounds
            r = s.part(f"R2{k}1", "Device:R", "3.3k", (2400, yc), rot=90, anchor="1", p1="+12V_P", p2=led)
            s.wire((2000, yc), r["1"])
            s.rail("+12V_P", (2000, yc), length=0)
            d = s.part(f"D2{k}1", "Diode:1N4148", "1N4148", (3100, yc + 200), rot=270, anchor="K", K=led, A=bike)
            c = s.part(f"C2{k}1", "Device:C", "100n", (3600, yc + 200), anchor="1", p1=led, p2=bike)
            u = s.part(f"U2{k}1", "Isolator:EL817", "EL817", (4200, yc), anchor="1", p1=led, p2=bike, p4=net, p3="GND")
            s.wire(r["2"], (3100, yc), (3600, yc), u["1"])
            s.wire((3100, yc), d["K"])
            s.wire((3600, yc), c["1"])
            low = yc + 600
            s.wire((1400, low), (3100, low), (3600, low), (4100, low), (4100, yc + 200), u["2"])
            s.wire(d["A"], (3100, low))
            s.wire(c["2"], (3600, low))
            s.labels.append(_port(s, bike, (1400, low)))
            s.wire(u["3"], (4900, yc + 200), (4900, yc + 500))
            s.gnd((4900, yc + 500))
            s.port(net, u["4"], 5600, shape="output")
        else:
            rpd = s.part(f"R2{k}1", "Device:R", "10k", (2000, yc + 200), anchor="1", p1=bike, p2="GND")
            r = s.part(f"R2{k}2", "Device:R", "4.7k", (2400, yc), rot=90, anchor="1", p1=bike, p2=led)
            d = s.part(f"D2{k}1", "Diode:1N4148", "1N4148", (3100, yc + 200), rot=270, anchor="K", K=led, A="GND")
            c = s.part(f"C2{k}1", "Device:C", "100n", (3600, yc + 200), anchor="1", p1=led, p2="GND")
            u = s.part(f"U2{k}1", "Isolator:EL817", "EL817", (4200, yc), anchor="1", p1=led, p2="GND", p4=net, p3="GND")
            s.wire((1400, yc), (2000, yc), r["1"])
            s.labels.append(_port(s, bike, (1400, yc)))
            s.wire((2000, yc), rpd["1"])
            s.gnd(rpd["2"])
            s.wire(r["2"], (3100, yc), (3600, yc), u["1"])
            s.wire((3100, yc), d["K"])
            s.wire((3600, yc), c["1"])
            s.gnd(d["A"])
            s.gnd(c["2"])
            opto_out(s, u, net, yc)
        u.fields = {"Reference": (-300, -330, "left"), "Value": (-300, -250, "left")}

    s.note(6500, 1300, "Channel by channel:\n"
                       "R2x2 4.7 kΩ: about 2 mA in the LED at 12 V\n"
                       "D2x1: protects the LED from reverse voltage\n"
                       "C2x1: filters spikes on the bike wire\n"
                       "R2x1 10 kΩ: an open wire reads off\n\n"
                       "The EL817 collector pulls the GPIO low\n"
                       "when the warning light is on.")
    s.verify(6500, 3000, "[TO VERIFY] neutral logic and wire colors,\n"
                         "with the bike measurements:\ndocs/research/original-wiring.md")
    return s


def _port(s: Sheet, net: str, at):
    from schgen.draw import LEFT, Label

    return Label("hier", net, at, LEFT, "input")
