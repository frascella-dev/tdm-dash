# 13 · Maintenance

The dashboard has no moving parts and its software does not "get dirty": the SD card is read-only. Maintenance is mostly about the sensor, the connectors and the seals.

## Periodic checks

| When | What | How |
|---|---|---|
| Every ride | Warning lights and dashboard at power-on | The warning light test at start-up must light everything; the turn signals must also flash on the backup LEDs |
| Every 1000 km | Wheel sensor | Distance from the bolts 1–2 mm, sensor nuts tight, cable intact along the fork |
| Every 5000 km or every year | Connectors | Superseal and DTM: no oxide, seals in place. A thin film of dielectric grease on the seals, not on the contacts |
| Every year | Enclosure | Window without cracks, EPDM gasket still elastic, screws tight. No condensation inside; if there is, check the breather vent |
| Every year | Backup | Copy `tdm-dash.toml` from the SD card to the PC; note down the km (they are in the FRAM, not on the SD card) |
| After washing with a pressure washer | Enclosure | Do not aim the jet at the enclosure or at the sensor connector |

## Changing tires

The tire circumference varies between brands, sizes and wear (up to 2–3%). After changing the **front** tire:

1. If the size is the same (110/80-18), leave the profile as it is for the first 300 km.
2. Then redo the circumference calibration with GPS (chapter 10).
3. With a different size, calculate a provisional circumference right away. You need the tire diameter, which depends on rim (inches), width (mm) and aspect ratio (%):
   - **Diameter:** rim × 25.4 + 2 × width × aspect ratio / 100. For a 110/80-18: 18 × 25.4 + 2 × 110 × 0.80 = 633 mm.
   - **Circumference:** diameter × 3.1416 × 0.97 (squash under load): about 1930 mm.
   - Then calibrate with GPS.

When you remove the front wheel, the sensor stays on the fork. On refitting, check the distance from the bolts: it depends on the position of the disc.

## Changing the discs or the bolts

The sensor reads the bolt heads of the front disc, which must be **magnetic steel**. With new discs:
- **Bolts:** count them, and if the number changes update `pulses_per_rev` in the `[speed]` section of the profile.
- **Stainless bolts (not magnetic):** the sensor sees them poorly. Use steel bolts, or glue steel washers under the heads.
- **Distance:** check the sensor distance again.

## Updates

- **Pi image:** rewrite the SD card with the new version (chapter 09). First copy `tdm-dash.toml` and put it back on the boot partition after writing. The km are not lost: they are in the FRAM.
- **Pico firmware:** with `tdmctl bootsel` from the development image, or with the BOOTSEL button if the enclosure is open (chapter 09). Image and firmware of the **same release** must be updated together: if the protocol changes, the dashboard shows "Module firmware not compatible".

## Replacing components

| Component | What stays and what to redo |
|---|---|
| SD card | Rewrite the image and copy the profile: nothing else |
| Raspberry Pi Zero 2 W | Move the SD card to the new Pi |
| Pico | Load the firmware onto the new Pico. Km and settings stay in the FRAM |
| FRAM | The km are lost: set the last known value with `tdmctl init-odo` (chapter 08). This is why it is worth noting down the km now and then |
| Display | Check the timings in `config.txt` if the model changes |
| Wheel sensor | Same model: only the distance adjustment |
| Coolant sender | Redo the water-bath curve if the brand changes (chapter 10) |

## Winter storage

- **Battery:** with the bike stopped for months, disconnect it, or keep it on a battery maintainer. The dashboard draws nothing with the key removed: it is powered only when the ignition key is on.
- **Enclosure:** check that it is dry inside before covering the bike.
- **When you start riding again:** switch on with the engine off and check warning lights, voltage and temperature (sender at ambient temperature).
