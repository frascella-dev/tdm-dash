"""The schematic, one module per sheet (see hardware/kicad/generate.py)."""

from . import root, s1_power, s2_inputs, s3_tach, s4_wheel, s5_analog, s6_mcu, s7_backlight, s8_leds

SHEETS = [
    s1_power.build,
    s2_inputs.build,
    s3_tach.build,
    s4_wheel.build,
    s5_analog.build,
    s6_mcu.build,
    s7_backlight.build,
    s8_leds.build,
]
