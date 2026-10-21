"""Sheet 1: bike connectors, protection, 5 V supply and Pico hold-up."""

from schgen.draw import DOWN, Sheet

TITLE = "Power supply and protection"


def build(lib) -> Sheet:
    s = Sheet(lib, 1, "1 Power", TITLE, "power.kicad_sch")
    s.heading(
        "Bike connectors, protection and 5 V supply",
        [
            "+12V_KEY is the switched 12 V of the bike. Chain: 2 A fuse in the harness, P-MOSFET against reversed",
            "polarity, 24 V TVS for the load dump, LC filter, 5 V buck converter for the Pi, the backlight and the Pico.",
            "The Pico runs from VSYS_PICO through D103: C103 keeps it alive long enough to save the trip.",
        ],
    )

    # bike connectors -----------------------------------------------------
    j101 = s.part("J101", "tdm:Conn_Bike6", "Superseal 1.5, 6 pins", (2000, 2000), anchor="1",
                  p1="+12V_KEY", p2="GND", p3="BIKE_NEUTRAL", p4="BIKE_HIGH_BEAM", p5="BIKE_TURN_L", p6="BIKE_TURN_R")
    for pin, net in (("3", "BIKE_NEUTRAL"), ("4", "BIKE_HIGH_BEAM"), ("5", "BIKE_TURN_L"), ("6", "BIKE_TURN_R")):
        s.port(net, j101[pin], 2700, shape="output")
    s.wire(j101["2"], (2100, 2500))
    s.gnd((2100, 2500))
    j102 = s.part("J102", "tdm:Conn_Bike4", "Superseal 1.5, 4 pins", (2000, 3300), anchor="1",
                  p1="BIKE_TACH", p2="BIKE_COOLANT", p3="BIKE_ILLUM", p4="GND")
    for pin, net in (("1", "BIKE_TACH"), ("2", "BIKE_COOLANT"), ("3", "BIKE_ILLUM")):
        s.port(net, j102[pin], 2700, shape="output")
    s.wire(j102["4"], (2100, 3600))
    s.gnd((2100, 3600))
    s.verify(950, 4050, "[TO VERIFY] pin numbers and wire colors of the\n"
                        "original cluster connector: docs/research/original-wiring.md")

    # protection and filter --------------------------------------------------
    y = 1500  # +12 V line
    f101 = s.part("F101", "Device:Fuse", "2A, in the harness", (2600, y), rot=90, anchor="1", p1="+12V_KEY", p2="+12V_F")
    s.wire(j101["1"], (2200, 2000), (2200, y), f101["1"])
    q101 = s.part("Q101", "Transistor_FET:IRF4905", "IRF4905", (3300, y), rot=90, anchor="D",
                  D="+12V_F", S="+12V_P", G="Q_GATE")
    q101.fields = {"Reference": (-150, -400, "left"), "Value": (-150, -320, "left")}
    s.chain(f101["2"], q101["D"])
    # gate: R101 to ground, D101 limits Vgs
    gx, gy = q101["G"].pos
    r101 = s.part("R101", "Device:R", "10k", (gx, 2200), anchor="1", p1="Q_GATE", p2="GND")
    d101 = s.part("D101", "Device:D_Zener", "12V", (3900, 1800), rot=270, anchor="K", K="+12V_P", A="Q_GATE")
    s.wire(q101["G"], (gx, 2100), r101["1"])
    s.wire((gx, 2100), d101["A"])
    s.gnd(r101["2"])
    d102 = s.part("D102", "Device:D_Zener", "SMCJ24A", (4700, 1800), rot=270, anchor="K", K="+12V_P", A="GND")
    s.gnd(d102["A"])
    l101 = s.part("L101", "Device:L", "22uH 2A", (5100, y), rot=90, anchor="1", p1="+12V_P", p2="+12V_FILT")
    s.wire(q101["S"], (3900, y), (4300, y), (4700, y), l101["1"])
    s.wire((3900, y), d101["K"])
    s.wire((4700, y), d102["K"])
    s.rail("+12V_P", (4300, y))
    c101 = s.part("C101", "Device:C_Polarized", "100uF 50V", (5800, 1800), anchor="1", p1="+12V_FILT", p2="GND")
    c102 = s.part("C102", "Device:C", "1uF 50V", (6500, 1800), anchor="1", p1="+12V_FILT", p2="GND")
    s.gnd(c101["2"])
    s.gnd(c102["2"])

    # 5 V converter and outputs ------------------------------------------------
    u101 = s.part("U101", "tdm:D36V28F5", "Pololu D36V28F5", (7100, y), anchor="VIN",
                  VIN="+12V_FILT", GND="GND", VOUT="+5V", EN=None, PG=None)
    u101.fields = {"Reference": (-500, -330, "left"), "Value": (-500, -250, "left")}
    s.wire(l101["2"], (5800, y), (6500, y), u101["VIN"])
    s.wire((5800, y), c101["1"])
    s.wire((6500, y), c102["1"])
    s.gnd(u101["GND"])
    j103 = s.part("J103", "tdm:Conn_Pi5V", "to the Pi PWR IN", (9600, y), anchor="1", p1="+5V", p2="GND")
    s.wire(u101["VOUT"], (8700, y), (9000, y), j103["1"])
    s.rail("+5V", (8700, y))
    s.wire(j103["2"], (9500, 1600))
    s.gnd((9500, 1600))
    d103 = s.part("D103", "Diode:1N5819", "1N5819", (9300, 2300), rot=180, anchor="A", A="+5V", K="VSYS_PICO")
    s.wire((9000, y), (9000, 2300), d103["A"])
    c103 = s.part("C103", "Device:C_Polarized", "470uF 10V", (10000, 2550), anchor="1", p1="VSYS_PICO", p2="GND")
    s.wire(d103["K"], (10000, 2300), c103["1"])
    s.rail("VSYS_PICO", (10000, 2300))
    s.gnd(c103["2"])

    # notes -------------------------------------------------------------------
    s.note(2950, 2850, "Q101: reverse polarity\nprotection; D101 keeps\nVgs within 12 V")
    s.note(4450, 2400, "TVS:\nload dump")
    s.note(5400, 2400, "LC filter")
    s.note(6900, 2400, "5 V 3.2 A buck,\nEN and PG unused")
    s.note(8700, 3150, "hold-up for the Pico:\nthe trip is saved\nwhen the key goes off")

    # power flags: these nets are driven by the parts above (for the ERC)
    for i, net in enumerate(("+12V_P", "+5V", "VSYS_PICO")):
        x = 3500 + 900 * i
        s.wire((x, 4700), (x, 5000))
        s.rail(net, (x, 4700), length=0)
        s.flag((x, 5000), DOWN)
    s.note(3150, 5450, "Power flags: the ERC learns that these rails are supplied on this sheet.")
    return s

