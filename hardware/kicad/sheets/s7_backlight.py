"""Sheet 7: display backlight driver and USB link to the Pi."""

from schgen.draw import LEFT, RIGHT, Sheet

TITLE = "Backlight and link to the Pi"


def build(lib) -> Sheet:
    s = Sheet(lib, 7, "7 Backlight and Pi", TITLE, "backlight.kicad_sch")
    s.heading(
        "Backlight and USB link to the Pi",
        [
            "The Pico dims the backlight with 25 kHz PWM on GP15. R702 holds CTRL low until the firmware runs,",
            "so the display stays dark if the Pi or the Pico hang (fail-dark, docs/adr/0006).",
        ],
    )
    # backlight ----------------------------------------------------------------
    y = 2300
    s.text(1000, y - 800, "Display backlight", size=60, bold=True)
    r701 = s.part("R701", "Device:R", "1k", (2000, y), rot=90, anchor="1", p1="BL_PWM", p2="BL_CTRL")
    s.port("BL_PWM", r701["1"], 1400, shape="input")
    r702 = s.part("R702", "Device:R", "100k", (2700, y + 200), anchor="1", p1="BL_CTRL", p2="GND")
    j701 = s.part("J701", "tdm:Conn_BLDriver3", "to the TPS61169 breakout", (4100, y), anchor="2",
                  p1="+5V", p2="BL_CTRL", p3="GND")
    s.wire(r701["2"], (2700, y), j701["2"])
    s.wire((2700, y), r702["1"])
    s.gnd(r702["2"])
    s.label("BL_CTRL", (3100, y), RIGHT)  # on the wire to J701
    s.wire(j701["1"], (3700, y - 100))
    s.rail("+5V", (3700, y - 100))
    s.wire(j701["3"], (3900, y + 100))
    s.gnd((3900, y + 100))
    s.note(1000, y + 800, "R701 and R702: CTRL stays low (backlight off)\nuntil the Pico drives GP15")

    # the breakout is off the board, next to the Kippah: J701 feeds it
    s.rect(5300, y - 700, 9800, y + 1300)
    s.note(5400, y - 650, "Off the board: TPS61169 breakout next to the Kippah,\n"
                          "wired to J701 (backlight harness, chapter 05)")
    u701 = s.part("U701", "tdm:TPS61169_Module", "TPS61169 (Adafruit 6354)", (6400, y), anchor="CTRL",
                  VIN="+5V", GND="GND", CTRL="BL_CTRL", **{"LED+": "BL_LED_A", "LED-": "BL_LED_K"})
    u701.fields = {"Reference": (-500, -280, "left"), "Value": (-500, -200, "left")}
    s.wire(u701["VIN"], (6300, y - 100))
    s.rail("+5V", (6300, y - 100))
    s.wire(u701["CTRL"], (5900, y))
    s.label("BL_CTRL", (5900, y), LEFT)
    s.gnd(u701["GND"])
    j704 = s.part("J704", "tdm:Conn_Backlight2", "display backlight", (8400, y - 100), anchor="1",
                  p1="BL_LED_A", p2="BL_LED_K")
    s.chain(u701["LED+"], j704["1"])
    s.chain(u701["LED-"], j704["2"])
    s.note(5400, y + 750, "Newhaven panel: LEDs at 180 mA, 9.3 V")
    s.verify(5400, y + 1000, "[TO VERIFY] breakout pins, and the panel LED pins\n"
                             "on the Kippah: panel datasheet, chapter 05")

    # USB ----------------------------------------------------------------------
    y = 4700
    s.text(1000, y - 800, "USB between the Pico and the Pi Zero 2 W", size=60, bold=True)
    j702 = s.part("J702", "tdm:Conn_USB_Pico", "Pico TP2/TP3 pads", (2000, y - 100), anchor="1",
                  p1=None, p2="USB_DM", p3="USB_DP", p4="GND")
    j703 = s.part("J703", "tdm:Conn_USB_Pi", "to the Pi USB port", (5000, y - 100), anchor="1",
                  p1=None, p2="USB_DM", p3="USB_DP", p4="GND", p5="GND")
    s.chain(j702["2"], j703["2"])
    s.chain(j702["3"], j703["3"])
    s.wire(j702["4"], (3500, y + 200), (4800, y + 200), j703["4"])
    s.gnd((3500, y + 200))
    s.wire(j703["5"], (4800, y + 300), (4800, y + 200))  # ID tied to GND in the plug
    s.note(2300, y + 650, "Short twisted pair with a clip-on ferrite.\n"
                          "VBUS is not connected: the Pico is supplied from VSYS (sheet 1).\n"
                          "ID is tied to GND in the plug, so the Pi works as USB host.")
    s.note(6300, y - 300, "The display itself connects to the Pi\n"
                          "through the Adafruit DPI Kippah (40-pin FFC),\nnot through this board.")
    return s
