# A · Appendices

## A.1 Yamaha wire color code

| Code | Color | Code | Color |
|---|---|---|---|
| B | Black | L | Blue |
| Br | Brown | O | Orange |
| Ch | Chocolate | P | Pink |
| Dg | Dark green | R | Red |
| G | Green | Sb | Sky blue |
| Gy | Grey | W | White |
| Lg | Light green | Y | Yellow |

Two codes separated by `/` give the wire color and the stripe color: `L/W` is blue with a white stripe.

## A.2 Glossary

| Term | Meaning |
|---|---|
| Active high / low | An "active high" input is on when it receives +12 V; "active low" when it is connected to ground |
| ADC | Analog-to-digital converter: turns a voltage into a number |
| Blanking | Time after a pulse during which the firmware ignores further pulses (coil ringing) |
| Buildroot | Tool that builds a minimal Linux system from scratch |
| COBS | Encoding that lets packets be separated by a 0x00 byte |
| CRC | Check code that detects corrupted packets |
| DPI | Parallel video interface of the Raspberry Pi to RGB displays |
| DRM/KMS | Linux kernel interface for drawing directly on the display |
| EROFS | Compressed read-only file system |
| FRAM | Non-volatile memory that can be written practically forever, as fast as RAM |
| MCU | Microcontroller (here the Raspberry Pi Pico) |
| NTC | Resistor whose value decreases when the temperature rises |
| Optocoupler | Component that passes a signal with light, electrically isolating the two sides |
| PWM | Very fast on/off signal, used to adjust brightness |
| TCI | Transistor-controlled inductive-discharge ignition unit of the TDM |
| TVS | Diode that absorbs short overvoltages |

## A.3 Measurement sheet

The sheet to fill in on the bike is in [`docs/research/bike-measurements.md`](../research/bike-measurements.md).

## A.4 Sources

All sources on the original wiring, with their reliability markers, are collected in [`docs/research/original-wiring.md`](../research/original-wiring.md).
