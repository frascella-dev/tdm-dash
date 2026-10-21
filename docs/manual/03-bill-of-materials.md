# 03 · Bill of materials

Indicative prices as of **October 2026**, VAT included where the shop shows it. Prices from non-EU shops (Adafruit, Pololu, Newhaven direct) do not include shipping, import VAT and customs: for an order from the USA count on roughly 30–40% more. Items marked "estimate" have not been checked on a shop.

## Summary

| Group | Estimated cost |
|---|---|
| Main electronics (Pi, Pico, display, memory) | 130–150 € |
| Inputs, protections, power supply | 20–40 € |
| Sensor, harnesses, connectors | 15–25 € |
| Enclosure and small parts | 20–30 € |
| **Total** | **≈ 190–240 €** |
| Recommended tools (not included) | oscilloscope ≈ 40 €, Debug Probe ≈ 12 € |

The biggest cost is the 7" 1000 nit display (≈ 78 €). A 7" that is not sunlight-readable (Waveshare, ≈ 300 nit, ≈ 35 €) costs less but washes out in the sun: it is not recommended. The 7–9" bar-type displays do not exceed 600 nit (see [ADR 0009](../adr/0009-seven-inch-display.md)).

## 1. Main electronics

| # | Component | Part number / model | Qty | Price | Where | Alternative | Notes |
|---|---|---|---|---|---|---|---|
| 1 | Computer | **Raspberry Pi Zero 2 W** | 1 | 17–20 € | [Kubii](https://www.kubii.com/en/raspberry-pi-zero/3455-raspberry-pi-zero-2-w-5056561800004.html), [The Pi Hut](https://thepihut.com/products/raspberry-pi-zero-2) | — | Temperature −20…+70 °C. **Often out of stock**: set up a stock alert. Without soldered header |
| 2 | microSD card | Industrial pSLC 8–16 GB (e.g. SanDisk Industrial) | 1 | 10–15 € (estimate) | Mouser, Farnell, Amazon | A good ordinary microSD for development | Better resistance to vibration and power fail |
| 3 | Display | **Newhaven NHD-7.0-800480AF-ASXP**, 7" 800×480, IPS, 1000 cd/m² (min. 800), anti-glare, 24-bit RGB on 40-pin FFC | 1 | ≈ 78 € (64.09 € + VAT) | [RS Components](https://ie.rs-online.com/web/p/lcd-color-displays/0218681), [Newhaven](https://newhavendisplay.com/content/specs/NHD-7.0-800480AF-ASXP.pdf) (74.53 $) | Newhaven NHD-7.0-HDMI-N-RSXN: 7" 800×480 1000 nit with HDMI board and drivers included, ≈ 95 € | −20…+70 °C. Active area 154.2 × 85.6 mm, outline 165 × 104 mm, 133 ppi. Backlight **180 mA at 9.3 V** (≈ 1.7 W), no on-board driver. Pixel clock 23–27 MHz. Datasheet checked (see [ADR 0009](../adr/0009-seven-inch-display.md)). Other panels and bar displays: [chapter 09](09-software.md#other-displays) |
| 4 | DPI adapter | **Adafruit DPI TFT Kippah** | 1 | ≈ 17 € (19.95 $) | [Adafruit](https://www.adafruit.com/product/2453) | Own FPC adapter (Molex 54104-4031 connector) | The 40-pin pinout of the Newhaven is the standard one supported by the Kippah (18-bit RGB from the Pi). The Kippah backlight driver cannot be PWM-dimmed and whether its maximum current is enough for 180 mA is **[TO VERIFY]**: the TPS61169 is used instead (row 5) |
| 5 | Backlight driver | **TPS61169** on Adafruit breakout | 1 | ≈ 4 € (4.50 $) | [Adafruit](https://www.adafruit.com/product/6354) | — | Boost up to 40 V, PWM input from the Pico. For 180 mA the sense resistor is ≈ 1.1–1.2 Ω (204 mV reference); whether it can be replaced on the breakout is **[TO VERIFY]** |
| 6 | Microcontroller | **Raspberry Pi Pico** (RP2040) | 1 | ≈ 4 € | The Pi Hut, Kubii (Pico 2: [The Pi Hut](https://thepihut.com/products/raspberry-pi-pico-2)) | Pico 2 (RP2350), same firmware | Pico 2: −20…+85 °C |
| 7 | Odometer memory | **FRAM MB85RC256V** (32 KB, I²C) on breakout | 1 | 5–9 € | [Adafruit](https://www.adafruit.com/product/1895), [eBay](https://www.ebay.de/itm/294830793064) | — | 10¹² writes, 10 years at 85 °C |
| 8 | Light sensor | **TEMT6000** on breakout | 1 | ≈ 2 € (estimate) | AliExpress, Amazon | — | For automatic brightness and day/night theme |

## 2. Power supply and protections

| # | Component | Part number / model | Qty | Price | Notes |
|---|---|---|---|---|---|
| 9 | Fuse | Waterproof fuse holder + 2 A mini blade | 1 | ≈ 2 € | As close as possible to the +12 V tap point |
| 10 | Reverse polarity protection | P-MOSFET ≥ 40 V, ≥ 5 A, e.g. **IRF4905** or SQJ401EP + 12 V gate zener + 10 kΩ resistor | 1 | ≈ 1 € | Negligible drop, unlike a diode |
| 11 | Transient suppressor | **TVS SMCJ24A** (1500 W) | 1 | ≈ 0.50 € | Clamp ≈ 39 V: the buck must withstand at least 50 V |
| 12 | Filter | 10–22 µH ≥ 2 A inductor + 100 µF 50 V electrolytic + 1 µF 50 V ceramic | 1 | ≈ 1.50 € | |
| 13 | 12→5 V converter | **Pololu D36V28F5** (input up to 50 V, 3.2 A) | 1 | ≈ 19 € (21.95 $) | [Pololu](https://www.pololu.com/product/3782), [The Pi Hut](https://thepihut.com/products/pololu-5v-3-2a-step-down-voltage-regulator-d36v28f5). Cheap alternative: 60 V LM2596**HV** module (≈ 4 €, beware of fakes). **Do not use the MP1584** (28 V max) |
| 14 | Pico reserve | 1N5819 Schottky + 470 µF 10 V electrolytic | 1 | ≈ 0.50 € | Keeps the Pico powered > 50 ms to save the km |

## 3. Inputs

| # | Component | Part number | Qty | Price | Use |
|---|---|---|---|---|---|
| 15 | Optocoupler | **EL817** (or PC817) | 6 | ≈ 1.50 € | Neutral, high beam, left turn signal, right turn signal, illumination, wheel sensor |
| 16 | Threshold optocoupler | **H11L1** | 1 | ≈ 1.50 € | Tachometer (Schmitt trigger output) |
| 17 | Diodes | 1N4148 ×8, 1N4007 ×2, BAT54S ×3 | — | ≈ 1 € | Protections and anti-parallel on the optos |
| 18 | Zener | 15 V 1 W (1N4744A) ×1 | 1 | ≈ 0.20 € | Tachometer |
| 19 | Resistors | 1/4 W 1% assortment; **22 kΩ 1 W ×2** (≥ 350 V) for the tachometer | — | ≈ 3 € | |
| 20 | Capacitors | Ceramic 100 nF ×15, 2.2 nF ×1 (tachometer, value to be refined with the oscilloscope) | — | ≈ 1 € | |
| 21 | Buttons | IP67 waterproof momentary panel push buttons, 12 mm, body ≤ 22 mm behind the panel | 2 | ≈ 4 € | MODE and SET, in the wings of the bezel |
| 22 | Backup LEDs | 3 mm LEDs: green ×2 (turn signals), blue ×1 (high beam), green ×1 (neutral) + 1.5 kΩ resistors and 1N4148 | — | ≈ 1 € | They work even with the electronics off |

## 4. Speed sensor and harnesses

| # | Component | Part number | Qty | Price | Notes |
|---|---|---|---|---|---|
| 23 | Speed sensor | **LJ12A3-4-Z/BX** inductive NPN NO, 6–36 V, 4 mm range | 1 | ≈ 6 € | [AZ-Delivery](https://az-delivery.de/en/products/lj12a3-4-z-bx-induktiver-naherungssensor). It only sees steel: check the bolts with a magnet |
| 24 | Sensor bracket | 3 mm aluminum sheet, or 3D print in ASA/PA-CF | 1 | ≈ 3 € | 1–2 mm gap from the bolt heads |
| 25 | Dashboard connector | **Superseal 1.5** 6–8 pins, male + female + terminals + seals | 1 | ≈ 5 € (estimate) | Waterproof |
| 26 | Sensor connector | **Deutsch DTM** 3 pins | 1 | ≈ 4 € (estimate) | |
| 27 | Used original cluster harness | **3VD-83509-00** (cord set) or matching connector | 1 | 5–15 € (used, estimate) | Provides the right connector for the adapter without cutting the bike |
| 28 | Cable | 0.5 mm² FLRY automotive wire in several colors, 3-core shielded cable for the sensor, twisted pair for the USB | — | ≈ 8 € | |
| 29 | Ferrite | Snap-on ferrite ring for the USB cable | 1 | ≈ 1 € | |

## 5. Building and enclosure

| # | Component | Qty | Price | Notes |
|---|---|---|---|---|
| 30 | Double-sided 7×9 cm perfboard, headers and turned-pin sockets | 1 | ≈ 3 € | Dedicated PCB only after 500 km of testing |
| 31 | Matte black **ASA** filament (≈ 200 g) | — | ≈ 6 € | ASA resists UV and heat (≈ 93 °C); PETG ages worse in the sun |
| 32 | 2 mm anti-glare polycarbonate or PMMA window | 1 | ≈ 4 € (estimate) | |
| 33 | 1 mm EPDM foam sheet (two gaskets) and 2 mm adhesive foam pads, M12 breather vent (Gore type), PG7 cable gland, Ø 4 mm acrylic rod for the light sensor | 1 | ≈ 6 € (estimate) | Prevents condensation; see chapter 07 |
| 34 | M3 ×13 and M5 ×3 heat-set inserts, stainless M3 × 10 ×9 and M3 × 30 ×4 screws, M2.5 × 6 self-tapping ×8 | — | ≈ 5 € | Display unit, pod and boards |
| 35 | Conformal coating varnish | 1 | ≈ 6 € | Protects against moisture |

## 6. Development tools (optional)

| Component | Price | Use |
|---|---|---|
| Raspberry Pi Debug Probe | ≈ 12 € | Firmware debugging (defmt via RTT) |
| Cheap oscilloscope (handheld DSO) | ≈ 40 € | Tachometer waveform, strongly recommended |
| 0–30 V bench power supply | 30–50 € | Bench acceptance test of the protections |
| Spare coolant sender | 10–20 € | Measuring the curve in a water bath without draining the system |

## Recommended order

1. Right away: Pi Zero 2 W (because of availability), display and Kippah (long shipping), Pico, FRAM.
2. After the [bike measurements](../research/bike-measurements.md): speed sensor, connectors, used harness, bracket (they depend on the bolts and the pinout).
3. After the schematic is confirmed (chapter 04): passive components, protections, perfboard.
