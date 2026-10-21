"""Root sheet: block diagram of the sensor module.

Every signal that crosses sheets is a hierarchical label in its sheet and a
pin of that sheet's block here; the wires between the blocks are the real
connections. Supplies are global power symbols and are not drawn here.
"""

from schgen.draw import NOTE_BLUE, ORANGE, Root

TITLE = "Overview: block diagram"

# y of each signal, shared by the blocks it connects, so wires run straight
Y = {
    "NEUTRAL": 2000, "HIGH_BEAM": 2200, "TURN_L": 2400, "TURN_R": 2600, "ILLUM": 2800,
    "TACH": 3600, "IGN": 3600, "WHEEL": 4300,
    "COOLANT": 5000, "VBAT_SENSE": 5000, "COOLANT_SENSE": 5200, "LIGHT_SENSE": 5400,
    "BL_PWM": 2000,
}

SHEETS = [
    ("1 Power", "bike connectors J101 and J102, harness fuse, reverse polarity, TVS, LC filter, 5 V buck, Pico hold-up"),
    ("2 Warning lights", "neutral, high beam, turn signals and illumination through EL817 optocouplers"),
    ("3 Tachometer", "coil negative to the H11L1 Schmitt trigger"),
    ("4 Wheel sensor", "LJ12A3 inductive sensor on J401, through an EL817"),
    ("5 Analog inputs", "battery voltage, coolant sender and ambient light to the ADC"),
    ("6 MCU", "Raspberry Pi Pico, FRAM for the odometer, MODE and SET buttons"),
    ("7 Backlight and Pi", "TPS61169 backlight driver dimmed by the Pico, USB link to the Pi"),
    ("8 Backup LEDs", "indicators wired straight to the bike, alive with the electronics off"),
]


def build(sheets) -> Root:
    by_num = {s.num: s for s in sheets}
    r = Root(TITLE)
    r.text(650, 650, "tdm-dash sensor module: block diagram", size=100, bold=True)

    x1, x2, x6, x7 = 700, 4600, 7400, 9600
    power = r.block(by_num[1], x1, 1700, 1800, 3900)
    inputs = r.block(by_num[2], x2, 1800, 1600, 1200)
    tach = r.block(by_num[3], x2, 3400, 1600, 400)
    wheel = r.block(by_num[4], x2, 4100, 1600, 400)
    analog = r.block(by_num[5], x2, 4800, 1600, 800)
    mcu = r.block(by_num[6], x6, 1800, 1600, 3800)
    light = r.block(by_num[7], x7, 1800, 1300, 800)
    leds = r.block(by_num[8], 2800, 1000, 1400, 750)

    def link(a, b, net_a, net_b, y):
        pa = a.pin(net_a, "right", y - a.y)
        pb = b.pin(net_b, "left", y - b.y)
        r.wire(pa, pb)
        return pa

    # bike wires: power block to the inputs, the tachometer and the analog block
    drops = {"NEUTRAL": 3000, "HIGH_BEAM": 3300, "TURN_L": 3600, "TURN_R": 3900}
    for sig in ("NEUTRAL", "HIGH_BEAM", "TURN_L", "TURN_R", "ILLUM"):
        y = Y[sig]
        link(power, inputs, f"BIKE_{sig}", f"BIKE_{sig}", y)
        if sig in drops:  # the backup LEDs take the same wires
            x = drops[sig]
            top = leds.pin(f"BIKE_{sig}", "bottom", x - leds.x)
            r.wire((x, y), top)
    link(power, tach, "BIKE_TACH", "BIKE_TACH", Y["TACH"])
    link(power, analog, "BIKE_COOLANT", "BIKE_COOLANT", Y["COOLANT"])
    # signals to the Pico
    for sig in ("NEUTRAL", "HIGH_BEAM", "TURN_L", "TURN_R", "ILLUM"):
        link(inputs, mcu, sig, sig, Y[sig])
    link(tach, mcu, "IGN", "IGN", Y["IGN"])
    link(wheel, mcu, "WHEEL", "WHEEL", Y["WHEEL"])
    for sig in ("VBAT_SENSE", "COOLANT_SENSE", "LIGHT_SENSE"):
        link(analog, mcu, sig, sig, Y[sig])
    link(mcu, light, "BL_PWM", "BL_PWM", Y["BL_PWM"])

    # what is inside the larger blocks
    r.note(800, 3900, "J101, J102: bike harness\nharness fuse, reverse\npolarity, TVS, LC filter,\n5 V buck, Pico hold-up")
    r.note(8100, 3700, "Raspberry Pi Pico\nFRAM (odometer)\nMODE and SET")
    r.note(9700, 2250, "TPS61169 driver\nUSB to the Pi")

    # legend
    r.rect(650, 5950, 6250, 7650)
    r.text(750, 6050, "Each block is one sheet; the wires between the blocks are the signals that cross sheets.",
           size=50, color=NOTE_BLUE)
    r.text(750, 6200, "Sheets", size=60, bold=True)
    r.text(750, 6330, "\n".join(f"{name}: {desc}" for name, desc in SHEETS), size=50)
    r.text(750, 7000, "Net names", size=60, bold=True)
    r.text(750, 7130, "BIKE_…  wire from the bike harness: up to 14.5 V, with spikes\n"
                      "+12V_P protected 12 V · +5V buck output · VSYS_PICO Pico supply · +3V3 Pico regulator\n"
                      "Supplies are global power symbols and are not drawn on this page.", size=50)
    r.text(750, 7500, "Orange notes marked [TO VERIFY] wait for the bike measurements "
                      "(docs/research/original-wiring.md).", size=50, color=ORANGE)
    return r
