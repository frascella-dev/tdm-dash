# 05 · Wiring harnesses

The harnesses are described in [WireViz](https://github.com/wireviz/WireViz): one YAML file per harness in [`hardware/harness/`](../../hardware/harness/), from which the drawing (SVG) and the bill of wires and connectors (TSV) are generated. The generated files are in [`hardware/exports/harness/`](../../hardware/exports/harness/).

```sh
pip install wireviz==0.4.1        # Graphviz is needed too (apt install graphviz)
for f in hardware/harness/*.yml; do wireviz -f st -o hardware/exports/harness "$f"; done
```

```mermaid
flowchart LR
  MOTO["Original bike<br/>harness"] -- "bike-adapter" --> Q["Sensor module<br/>(perfboard)"]
  S["LJ12A3 sensor"] -- "wheel-sensor" --> Q
  Q -- "pi: 5 V + USB" --> PI["Pi Zero 2 W"]
  Q -- "backlight" --> D["Newhaven display"]
  PI -- "Kippah DPI" --> D
  P["Buttons and<br/>light sensor"] -- "panel" --> Q
```

Rules for all harnesses:
- **Cable:** FLRY-B automotive, 0.5 mm² for the power supplies and the bike signals, thinner only inside the enclosure.
- **Connectors:** all waterproof (Superseal 1.5, Deutsch DTM). Solder joints must be covered with adhesive-lined heat-shrink tubing.
- **Original wiring:** **never cut it**. The adapter starts from a used cluster harness (3VD-83509-00) or from the matching connector.
- **Wire colors:** those of the bike remain [TO VERIFY] until the [bike measurements](../research/bike-measurements.md) are done. The colors of the new wires are our choice and are given in the tables.

## Adapter from the original cluster

![Adapter harness](../../hardware/exports/harness/bike-adapter.svg)

File: [`bike-adapter.yml`](../../hardware/harness/bike-adapter.yml). Bill of materials: [`bike-adapter.bom.tsv`](../../hardware/exports/harness/bike-adapter.bom.tsv).

| From (bike side) | Yamaha color | New wire | To | Net | Function |
|---|---|---|---|---|---|
| +12 V switched | Br [TO VERIFY] | red, with 2 A fuse | Superseal 6p pin 1 → J101.1 | `+12V_KEY` | Power supply |
| Ground | B [HYPOTHESIS] | black | Superseal 6p pin 2 → J101.2 | `GND` | Ground |
| Neutral indicator | Sb [HYPOTHESIS] | light blue | Superseal 6p pin 3 → J101.3 | `BIKE_NEUTRAL` | Neutral, active to ground |
| High beam indicator | Y [HYPOTHESIS] | yellow | Superseal 6p pin 4 → J101.4 | `BIKE_HIGH_BEAM` | +12 V with high beam |
| Left turn signal | Ch [TO VERIFY] | brown | Superseal 6p pin 5 → J101.5 | `BIKE_TURN_L` | +12 V flashing |
| Right turn signal | Dg [TO VERIFY] | green | Superseal 6p pin 6 → J101.6 | `BIKE_TURN_R` | +12 V flashing |
| Tachometer | O or Gy [HYPOTHESIS] | orange | Superseal 4p pin 1 → J102.1 | `BIKE_TACH` | Coil negative, spikes of hundreds of volts |
| Coolant sender | ? [TO VERIFY] | violet | Superseal 4p pin 2 → J102.2 | `BIKE_COOLANT` | NTC to ground |
| Illumination | L [HYPOTHESIS] | blue | Superseal 4p pin 3 → J102.3 | `BIKE_ILLUM` | +12 V with the lights |
| Ground | B [HYPOTHESIS] | black | Superseal 4p pin 4 → J102.4 | `GND` | Ground |

> **Caution:** the fuse goes **on the bike side** of the red wire, as close as possible to the point where the +12 V is taken. This way it also protects the stretch of cable to the dashboard.

## Speed sensor

![Wheel sensor harness](../../hardware/exports/harness/wheel-sensor.svg)

File: [`wheel-sensor.yml`](../../hardware/harness/wheel-sensor.yml).

| From | Wire | To | Net |
|---|---|---|---|
| LJ12A3 brown (+V) | sensor brown | DTM pin 1 | `+12V_P` |
| LJ12A3 black (output) | sensor black | DTM pin 2 | `WHEEL_SIG` |
| LJ12A3 blue (0 V) | sensor blue | DTM pin 3 | `GND` |
| DTM pin 1 | red, shielded cable | J401.1 | `+12V_P` |
| DTM pin 2 | white, shielded cable | J401.2 | `WHEEL_SIG` |
| DTM pin 3 | black, shielded cable | J401.3 | `GND` |
| Shield | — | J401.3, dashboard side only | `GND` |

The sensor colors come from its datasheet. The DTM connector sits near the fork yoke, so the sensor can be unplugged to remove the wheel. Leave the cable enough slack for the full fork travel and for the steering at full lock on both sides.

## Pi Zero 2 W: power and USB

![Pi harness](../../hardware/exports/harness/pi.svg)

File: [`pi.yml`](../../hardware/harness/pi.yml). Nothing is soldered on the Pi: the two cables end in micro-USB plugs.

| From | Wire | To | Function |
|---|---|---|---|
| J103.1 (+5V) | red 0.5 mm² | micro-USB pin 1 (VBUS), **PWR IN** port | Pi power supply |
| J103.2 (GND) | black 0.5 mm² | micro-USB pin 5 (GND), **PWR IN** port | Ground |
| Pico pad TP2 (D−) | white, twisted pair | micro-USB pin 2 (D−), **USB** port | Data |
| Pico pad TP3 (D+) | green, twisted pair | micro-USB pin 3 (D+), **USB** port | Data |
| Pico pin 38 (GND) | black | micro-USB pin 5 (GND), **USB** port | Ground |
| — | jumper in the plug | micro-USB pin 4 (ID) → pin 5 | The Pi acts as host (like an OTG cable) |

Pin 1 (VBUS) of the data plug **stays disconnected**: the Pico is already powered from VSYS. If the Pi also powered it over USB, the two supplies would be tied together. A snap-on ferrite on the data cable reduces noise; keep the cable short (15 cm) and twisted.

## Backlight

![Backlight harness](../../hardware/exports/harness/backlight.svg)

File: [`backlight.yml`](../../hardware/harness/backlight.yml).

| From | Wire | To | Net |
|---|---|---|---|
| TPS61169 LED+ | red | J701.1 | `BL_LED_A` |
| TPS61169 LED− | black | J701.2 | `BL_LED_K` |
| J701.1 | red | display LED anode | `BL_LED_A` |
| J701.2 | black | display LED cathode | `BL_LED_K` |

> **Caution:** the display's 40-pin flat cable goes into the Kippah, which has its own non-adjustable backlight driver. It still has to be checked **[TO VERIFY]** which connector pins carry the LEDs and how to detach them from the Kippah driver to connect them to the TPS61169. Before connecting, check the Newhaven datasheet and the Kippah board: a wrong voltage on the LEDs damages them.

## Panel: buttons and light sensor

![Panel harness](../../hardware/exports/harness/panel.svg)

File: [`panel.yml`](../../hardware/harness/panel.yml).

| From | Wire | To | Net |
|---|---|---|---|
| J601.1 | yellow | MODE button | `BTN_MODE_SW` |
| J601.2 | white | SET button | `BTN_SET_SW` |
| J601.3 | black (two wires) | other contact of both buttons | `GND` |
| J501.1 | red | TEMT6000 VCC | `+3V3` |
| J501.2 | black | TEMT6000 GND | `GND` |
| J501.3 | yellow | TEMT6000 SIG | `LIGHT_RAW` |

## Reference pinouts

**Raspberry Pi Pico** (top view, USB at the top). Only the pins used; the full list is in [sheet 6](04-schematics.md#6--pico-fram-and-buttons).

| Pin | Function | Pin | Function |
|---|---|---|---|
| 3 | GND | 31 | GP26 battery |
| 4 | GP2 wheel | 32 | GP27 coolant |
| 5 | GP3 tachometer | 33 | AGND |
| 6 | GP4 SDA | 34 | GP28 light |
| 7 | GP5 SCL | 36 | 3V3 (output) |
| 9–12 | GP6–GP9 neutral, high beam, turn signals | 38 | GND |
| 14 | GP10 illumination | 39 | VSYS (5 V input) |
| 15, 16 | GP11 MODE, GP12 SET | 40 | VBUS (not connected) |
| 20 | GP15 backlight | TP2, TP3 | USB D−, D+ (on the back) |

**Raspberry Pi Zero 2 W.**
- Two micro-USB ports: **PWR IN** (power only) and **USB** (data, the one closer to the center of the board).
- The 40-pin header is taken by the Kippah, which uses the GPIOs for the display's DPI signal.

**Micro-USB (plug).**

| Pin | 1 | 2 | 3 | 4 | 5 |
|---|---|---|---|---|---|
| Signal | VBUS | D− | D+ | ID | GND |

**Connectors on the perfboard.**

| Connector | Pins |
|---|---|
| J101 (Superseal 6p) | 1 +12V_KEY · 2 GND · 3 neutral · 4 high beam · 5 left turn signal · 6 right turn signal |
| J102 (Superseal 4p) | 1 tachometer · 2 coolant sender · 3 illumination · 4 GND |
| J103 | 1 +5V · 2 GND (to the Pi) |
| J401 (to the sensor DTM) | 1 +12V_P · 2 signal · 3 GND |
| J501 (TEMT6000) | 1 +3V3 · 2 GND · 3 signal |
| J601 (buttons) | 1 MODE · 2 SET · 3 GND |
| J701 (backlight) | 1 LED+ · 2 LED− |
