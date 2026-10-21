"""Sheet 8: backup indicator LEDs, wired straight to the bike lines."""

from schgen.draw import LEFT, RIGHT, Label, Sheet

TITLE = "Backup LEDs"


def port(s: Sheet, net: str, at, out):
    s.labels.append(Label("hier", net, at, out, "input"))


def build(lib) -> Sheet:
    s = Sheet(lib, 8, "8 Backup LEDs", TITLE, "leds.kicad_sch")
    s.heading(
        "Backup LEDs",
        [
            "Wired straight to the bike's 12 V lines, so they light up even with the electronics off.",
            "The turn signals use the pair of wires of the original indicator: the flashing side feeds its LED,",
            "the other side goes to ground through its bulbs. The 1N4148 protects each LED from the reverse 12 V.",
        ],
    )
    rows = [
        ("BIKE_TURN_L", "LED_L_D", "LED_L_A", "BIKE_TURN_R", "R801", "D805", "D801", "green (left turn)",
         "Left turn signal"),
        ("BIKE_TURN_R", "LED_R_D", "LED_R_A", "BIKE_TURN_L", "R802", "D806", "D802", "green (right turn)",
         "Right turn signal"),
    ]
    y = 2100
    for src, mid, anode, ret, rr, dd, led, colour, title in rows:
        s.text(1000, y - 500, title, size=60, bold=True)
        r = s.part(rr, "Device:R", "1.5k", (2000, y), rot=90, anchor="1", p1=src, p2=mid)
        d = s.part(dd, "Diode:1N4148", "1N4148", (2700, y), rot=180, anchor="A", A=mid, K=anode)
        lamp = s.part(led, "Device:LED", colour, (3400, y), rot=180, anchor="A", A=anode, K=ret)
        s.wire((1400, y), r["1"])
        port(s, src, (1400, y), LEFT)
        s.chain(r["2"], d["A"])
        s.chain(d["K"], lamp["A"])
        s.wire(lamp["K"], (4400, y))
        port(s, ret, (4400, y), RIGHT)
        y += 1100
    s.note(5400, 2150, "Current flows from the flashing side,\n"
                       "through its LED, to the other side,\n"
                       "which the bulbs hold near ground.")

    s.text(1000, y - 500, "High beam", size=60, bold=True)
    r803 = s.part("R803", "Device:R", "1.5k", (2000, y), rot=90, anchor="1", p1="BIKE_HIGH_BEAM", p2="LED_HB_A")
    d803 = s.part("D803", "Device:LED", "blue (high beam)", (3400, y), rot=180, anchor="A", A="LED_HB_A", K="GND")
    s.wire((1400, y), r803["1"])
    port(s, "BIKE_HIGH_BEAM", (1400, y), LEFT)
    s.chain(r803["2"], d803["A"])
    s.wire(d803["K"], (3900, y), (3900, y + 200))
    s.gnd((3900, y + 200), length=0)

    y += 1100
    s.text(1000, y - 500, "Neutral", size=60, bold=True)
    r804 = s.part("R804", "Device:R", "1.5k", (2000, y), rot=90, anchor="1", p1="+12V_P", p2="LED_N_A")
    d804 = s.part("D804", "Device:LED", "green (neutral)", (3400, y), rot=180, anchor="A", A="LED_N_A", K="BIKE_NEUTRAL")
    s.wire((1600, y), r804["1"])
    s.rail("+12V_P", (1600, y), length=0)
    s.chain(r804["2"], d804["A"])
    s.wire(d804["K"], (4400, y))
    port(s, "BIKE_NEUTRAL", (4400, y), RIGHT)
    s.note(5400, y - 100, "The gearbox switch grounds the neutral wire.")
    return s
