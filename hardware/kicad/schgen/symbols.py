"""Symbols defined by this project: off-the-shelf modules and connectors drawn
as boxes with named pins, so the sheet says what each pin is.

All pins are passive: the modules are not drivers for ERC, the power flags
on the sheets are.
"""

from __future__ import annotations

from sexpr import Q

from .geom import mm

PIN_LEN = 100  # mils
PITCH = 100


def _font(size: float = 1.27):
    return ["font", ["size", size, size]]


def _prop(name: str, value: str, x: int, y: int, hide: bool = False, justify: str | None = None):
    eff = ["effects", _font()]
    if justify:
        eff.append(["justify", *justify.split()])
    if hide:
        eff.append(["hide", "yes"])
    return ["property", Q(name), Q(value), ["at", mm(x), mm(y), 0], eff]


def box(
    lib_id: str,
    ref: str,
    value: str,
    description: str,
    left=(),
    right=(),
    top=(),
    bottom=(),
    width: int = 600,
    show_numbers: bool = True,
) -> list:
    """A rectangle with pins on its sides; each side lists (number, name)."""
    rows = max(len(left), len(right), 1)
    half_h = (rows * PITCH + PITCH) // 2
    half_w = width // 2
    name = lib_id.split(":")[1]
    pins = []

    def pin(number: str, pname: str, x: int, y: int, angle: int):
        pins.append(
            [
                "pin",
                "passive",
                "line",
                ["at", mm(x), mm(y), angle],
                ["length", mm(PIN_LEN)],
                ["name", Q(pname), ["effects", _font()]],
                ["number", Q(number), ["effects", _font()]],
            ]
        )

    for i, (num, pname) in enumerate(left):
        pin(num, pname, -half_w - PIN_LEN, half_h - PITCH * (i + 1), 0)
    for i, (num, pname) in enumerate(right):
        pin(num, pname, half_w + PIN_LEN, half_h - PITCH * (i + 1), 180)
    # top and bottom pins are centered on the box
    for i, (num, pname) in enumerate(top):
        pin(num, pname, int((i - (len(top) - 1) / 2) * PITCH), half_h + PIN_LEN, 270)
    for i, (num, pname) in enumerate(bottom):
        pin(num, pname, int((i - (len(bottom) - 1) / 2) * PITCH), -half_h - PIN_LEN, 90)

    pin_names = ["pin_names", ["offset", 0.508]]
    out = [
        "symbol",
        Q(lib_id),
        pin_names,
        *([] if show_numbers else [["pin_numbers", ["hide", "yes"]]]),
        ["exclude_from_sim", "no"],
        ["in_bom", "yes"],
        ["on_board", "yes"],
        _prop("Reference", ref, -half_w, half_h + 80, justify="left bottom"),
        _prop("Value", value, -half_w, -half_h - 80, justify="left top"),
        _prop("Footprint", "", 0, 0, hide=True),
        _prop("Datasheet", "", 0, 0, hide=True),
        _prop("Description", description, 0, 0, hide=True),
        [
            "symbol",
            Q(f"{name}_0_1"),
            [
                "rectangle",
                ["start", mm(-half_w), mm(half_h)],
                ["end", mm(half_w), mm(-half_h)],
                ["stroke", ["width", 0.254], ["type", "default"]],
                ["fill", ["type", "background"]],
            ],
        ],
        ["symbol", Q(f"{name}_1_1"), *pins],
        ["embedded_fonts", "no"],
    ]
    return out


def connector(lib_id: str, value: str, description: str, pins, side: str = "right", width: int = 500) -> list:
    """A connector as a narrow box with the pin names inside.

    Pins are drawn in the order given, which need not be the numbering:
    ground goes last, so its symbol can point down without crossing wires.

    `side` is where the wires leave: "right" for connectors on the left edge
    of a sheet, "left" for those on the right edge.
    """
    if side == "right":
        return box(lib_id, "J", value, description, right=pins, width=width)
    return box(lib_id, "J", value, description, left=pins, width=width)


def project_symbols() -> dict[str, list]:
    """Every symbol defined here, by lib_id."""
    s = {}

    def add(sym):
        s[str(sym[1])] = sym

    add(connector("tdm:Conn_Bike6", "Superseal 1.5, 6 pins", "Harness connector, first group of bike wires",
                  [("1", "+12V_KEY"), ("3", "NEUTRAL"), ("4", "HIGH_BEAM"), ("5", "TURN_L"), ("6", "TURN_R"), ("2", "GND")],
                  width=900))
    add(connector("tdm:Conn_Bike4", "Superseal 1.5, 4 pins", "Harness connector, second group of bike wires",
                  [("1", "TACH"), ("2", "COOLANT"), ("3", "ILLUM"), ("4", "GND")], width=900))
    add(connector("tdm:Conn_Sensor3", "to the wheel sensor", "Wheel sensor cable, from the DTM connector at the fork",
                  [("1", "+12V"), ("2", "SIG"), ("3", "GND")], width=600))
    add(connector("tdm:Conn_Light3", "to the TEMT6000", "Light sensor module, behind the window",
                  [("1", "VCC"), ("3", "SIG"), ("2", "GND")], width=600))
    add(connector("tdm:Conn_Buttons3", "to the buttons", "MODE and SET buttons on the enclosure",
                  [("1", "MODE"), ("2", "SET"), ("3", "GND")], side="left", width=600))
    add(connector("tdm:Conn_Pi5V", "to the Pi PWR IN", "5 V supply of the Raspberry Pi Zero 2 W",
                  [("1", "+5V"), ("2", "GND")], side="left", width=600))
    add(connector("tdm:Conn_BLDriver3", "to the TPS61169 breakout", "Supply and dimming of the backlight driver",
                  [("1", "+5V"), ("2", "CTRL"), ("3", "GND")], side="left", width=600))
    add(connector("tdm:Conn_Backlight2", "display backlight", "Backlight LEDs of the panel, reached on the Kippah",
                  [("1", "LED_A"), ("2", "LED_K")], side="left", width=700))
    add(connector("tdm:Conn_USB_Pico", "Pico TP2/TP3 pads", "USB data from the pads on the back of the Pico",
                  [("1", "VBUS"), ("2", "D-"), ("3", "D+"), ("4", "GND")], width=600))
    add(connector("tdm:Conn_USB_Pi", "to the Pi USB port", "Micro-USB plug in the Pi Zero 2 W USB (OTG) port",
                  [("1", "VBUS"), ("2", "D-"), ("3", "D+"), ("4", "ID"), ("5", "GND")], side="left", width=600))
    add(box("tdm:D36V28F5", "U", "Pololu D36V28F5", "5 V 3.2 A step-down module, input up to 50 V",
            left=[("1", "VIN"), ("4", "EN"), ("5", "PG")], right=[("3", "VOUT")], bottom=[("2", "GND")],
            width=1000))
    add(box("tdm:TPS61169_Module", "U", "TPS61169 (Adafruit 6354)", "Constant-current LED boost module",
            left=[("1", "VIN"), ("3", "CTRL")], right=[("4", "LED+"), ("5", "LED-")], bottom=[("2", "GND")],
            width=1000))
    add(box("tdm:FRAM_Breakout", "U", "MB85RC256V (Adafruit 1895)",
            "32 KB I2C FRAM breakout; header pin order [TO VERIFY]",
            left=[("5", "SDA"), ("4", "SCL")], right=[("3", "WP"), ("8", "A0"), ("7", "A1"), ("6", "A2")],
            top=[("1", "VCC")], bottom=[("2", "GND")], width=800))
    return s
