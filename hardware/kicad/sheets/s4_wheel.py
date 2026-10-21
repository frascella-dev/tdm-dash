"""Sheet 4: wheel speed sensor."""

from schgen.draw import Sheet

TITLE = "Wheel sensor (LJ12A3)"


def build(lib) -> Sheet:
    s = Sheet(lib, 4, "4 Wheel sensor", TITLE, "wheel.kicad_sch")
    s.heading(
        "Wheel speed sensor",
        [
            "LJ12A3-4-Z/BX inductive sensor, NPN normally open: in front of a bolt its output goes to ground.",
            "The sensor sinks the current of the EL817 LED, which is fed from +12V_P through R401;",
            "R402 pulls the output up for fast edges.",
        ],
    )
    j401 = s.part("J401", "tdm:Conn_Sensor3", "Deutsch DTM, 3 pins", (2000, 2300), anchor="1",
                  p1="+12V_P", p2="WHEEL_SIG", p3="GND")
    y1, y2 = 2000, 2900  # LED anode line, sensor output line
    s.wire(j401["1"], (2200, 2300), (2200, y1), (2500, y1), (2900, y1))
    s.rail("+12V_P", (2500, y1))
    c401 = s.part("C401", "Device:C", "100n", (2900, 2150), anchor="1", p1="+12V_P", p2="GND")
    s.wire((2900, y1), c401["1"])
    s.gnd(c401["2"])
    r401 = s.part("R401", "Device:R", "2.2k", (3300, y1), rot=90, anchor="1", p1="+12V_P", p2="WHEEL_LED")
    s.wire((2900, y1), r401["1"])
    d401 = s.part("D401", "Diode:1N4148", "1N4148", (4000, 2300), rot=270, anchor="K", K="WHEEL_LED", A="WHEEL_SIG")
    u401 = s.part("U401", "Isolator:EL817", "EL817", (4800, 2700), anchor="1",
                  p1="WHEEL_LED", p2="WHEEL_SIG", p4="WHEEL", p3="GND")
    u401.fields = {"Reference": (-300, -330, "left"), "Value": (-300, -250, "left")}
    s.wire(r401["2"], (4000, y1), (4600, y1), (4600, 2700), u401["1"])
    s.wire((4000, y1), d401["K"])
    s.wire(j401["2"], (2400, 2400), (2400, y2), (4000, y2), u401["2"])
    s.wire(d401["A"], (4000, y2))
    s.wire(j401["3"], (2300, 2500))
    s.gnd((2300, 2500))
    s.wire(u401["3"], (5500, y2), (5500, 3100))
    s.gnd((5500, 3100))
    r402 = s.part("R402", "Device:R", "4.7k", (5900, 2550), anchor="2", p1="+3V3", p2="WHEEL")
    s.rail("+3V3", r402["1"])
    s.wire(u401["4"], (5900, 2700))
    s.wire(r402["2"], (5900, 2700))
    s.port("WHEEL", (5900, 2700), 6800, shape="output")

    s.note(1250, 3300, "Sensor wires: brown +12 V,\nblack signal, blue ground")
    s.note(3250, 3300, "D401 protects the LED\nfrom reverse voltage")
    s.note(5800, 3300, "WHEEL → GP2,\nactive low")
    return s
