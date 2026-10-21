"""Sheet 3: tachometer input from the coil negative."""

from schgen.draw import Sheet

TITLE = "Tachometer (coil negative)"


def build(lib) -> Sheet:
    s = Sheet(lib, 3, "3 Tachometer", TITLE, "tach.kicad_sch")
    s.heading(
        "Tachometer input",
        [
            "The coil negative swings by a few hundred volts at every spark.",
            "D301 passes the positive spikes, R301 and R302 limit the current, D302 and C301",
            "clamp and filter them; the H11L1 Schmitt trigger gives the Pico a clean edge.",
        ],
    )
    y = 2500  # signal line
    d301 = s.part("D301", "Diode:1N4007", "1N4007", (1900, y), rot=180, anchor="A", A="BIKE_TACH", K="TACH_1")
    s.port("BIKE_TACH", d301["A"], 1400, shape="input")
    r301 = s.part("R301", "Device:R", "22k 1W", (2500, y), rot=90, anchor="1", p1="TACH_1", p2="TACH_2")
    r302 = s.part("R302", "Device:R", "22k 1W", (3100, y), rot=90, anchor="1", p1="TACH_2", p2="TACH_3")
    s.chain(d301["K"], r301["1"])
    s.chain(r301["2"], r302["1"])
    d302 = s.part("D302", "Device:D_Zener", "1N4744A 15V", (3900, 2750), rot=270, anchor="K", K="TACH_3", A="GND")
    c301 = s.part("C301", "Device:C", "2.2n", (4900, 2750), anchor="1", p1="TACH_3", p2="GND")
    r303 = s.part("R303", "Device:R", "2.2k", (5400, y), rot=90, anchor="1", p1="TACH_3", p2="TACH_LED")
    s.wire(r302["2"], (3900, y), (4900, y), r303["1"])
    s.wire((3900, y), d302["K"])
    s.wire((4900, y), c301["1"])
    s.gnd(d302["A"])
    s.gnd(c301["2"])

    u301 = s.part("U301", "Isolator:H11L1", "H11L1", (6100, y), anchor="1",
                  p1="TACH_LED", p2="GND", p4="IGN", p5="GND", p6="+3V3")
    u301.fields = {"Reference": (250, -330, "left"), "Value": (250, -230, "left")}
    s.chain(r303["2"], u301["1"])
    s.wire(u301["2"], (6000, 2700), (6000, 3050))
    s.gnd((6000, 3050), length=0)
    s.rail("+3V3", u301["6"])
    s.gnd(u301["5"], length=150)
    r304 = s.part("R304", "Device:R", "4.7k", (7300, 2450), anchor="2", p1="+3V3", p2="IGN")
    s.rail("+3V3", r304["1"])
    s.wire(u301["4"], (7300, 2600))
    s.wire(r304["2"], (7300, 2600))
    s.port("IGN", (7300, 2600), 8200, shape="output")

    c302 = s.part("C302", "Device:C", "100n", (9300, 2150), anchor="1", p1="+3V3", p2="GND")
    s.rail("+3V3", c302["1"])
    s.gnd(c302["2"])
    s.note(9050, 2800, "decoupling\nof the H11L1")

    s.note(1600, 3450, "passes only the\npositive spikes")
    s.note(2450, 3450, "2 × 22 kΩ 1 W:\nlimit the current")
    s.note(3700, 3450, "clamp at 15 V,\nfilter")
    s.note(5800, 3450, "Schmitt trigger:\nclean edges")
    s.note(7250, 3450, "IGN → GP3,\nactive low")
    s.verify(1300, 4100, "[TO VERIFY] tachometer wire and its waveform, with the oscilloscope:\n"
                         "docs/research/original-wiring.md")
    return s
