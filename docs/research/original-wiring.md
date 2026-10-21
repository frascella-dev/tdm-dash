# Research: original wiring of the TDM 850 (3VD)

Collected data on the wiring and the instrument cluster of the first series Yamaha TDM 850 (1991–1995). It is the basis on which the digital dashboard is designed. [Chapter 01 of the manual](../manual/01-original-wiring.md) explains the same data in a more readable form.

Every fact is tagged:

| Tag | Meaning |
|---|---|
| **[VERIFIED]** | Confirmed by at least two independent sources, or by an official source (Yamaha manual, Haynes), or measured on the bike |
| **[TO VERIFY]** | A single source, or a source about a similar model (4TX, TRX 850, XTZ 750) |
| **[HYPOTHESIS]** | Our own deduction or a general Yamaha convention; no specific source |

When a fact is measured on the bike, update the tag and record the measurement in [`bike-measurements.md`](bike-measurements.md).

## 1. Model identification

| Fact | Value | Status | Source |
|---|---|---|---|
| First series code | **3VD**, 1991–1995, 360° crankshaft | [VERIFIED] | [Wikipedia](https://en.wikipedia.org/wiki/Yamaha_TDM850), [CMSNL](https://www.cmsnl.com/yamaha-tdm850-1992-3vd4-europe-223vd300e2_model37920/partslist/) |
| 1992 Europe variant | **3VD4** (3VD1/3VD2 1991, 3VD5 1993, 3VD7 1994, 3VD9/3VDA 1995) | [VERIFIED] | CMSNL, [oem-parts.hu](https://oem-parts.hu/en/parts/yamaha/motor-robogo/700cc-1099cc/tdm-850/tdm850-3vd5-1993/meter) |
| Other first series codes | 4EP (Japan 1992–94), 4CM (Sweden), 4DT (USA) | [TO VERIFY] | [monotaro](https://www.monotaro.com/bp/YAMAHA/c-60/b-7477/bk-438/), CMSNL, [tdm-forum t=33812](https://tdm-forum.net/viewtopic.php?t=33812) |
| Second series | **4TX** from 1996, 270° crankshaft (same engine as the TRX 850). Different wiring, do **not** use its diagrams without comparison | [VERIFIED] | [1996 service manual](https://www.manualslib.com/manual/927445/Yamaha-Tdm850-1996.html) |

The frame number of the bike confirms the exact variant.

## 2. Original cluster

| Item | Present? | Notes | Status | Source |
|---|---|---|---|---|
| Speedometer | Yes, **mechanical, cable driven** | Worm gear drive on the front hub | [VERIFIED] | [Haynes p.190](https://www.manualslib.com/manual/927505/Yamaha-Tdm850.html?page=190), [tdm-forum t=16997](https://tdm-forum.net/viewtopic.php?t=16997) |
| Odometer and trip | Yes, mechanical, inside the speedometer | They are lost with the replacement: photograph the reading first | [VERIFIED] | same |
| Tachometer | Yes, electronic | Part number 3VD-83540-00 | [VERIFIED] | [bike-parts-yam](https://www.bike-parts-yam.com/yamaha-motorcycle/850-MOTO/1995/TDM/TDM850/SPEEDOMETER/28_2762-2762/B39/0/7260) |
| Coolant temperature gauge | Yes, needle | Possibly built into the tachometer body | [VERIFIED] presence, [TO VERIFY] position | [Haynes p.102](https://www.manualslib.com/manual/927505/Yamaha-Tdm850.html?page=102), [FR manual](https://www.manualslib.fr/manual/1046161/Yamaha-Tdm850.html) |
| Fuel gauge / reserve indicator | **No** | Only an OFF/ON/RES tap. They arrive with the 1999 model | [VERIFIED] | [TDMitalia FAQ](https://www.tdmitalia.net/?u=faq/tecniche002), [Haynes p.192](https://www.manualslib.com/manual/927505/Yamaha-Tdm850.html?page=192) |
| Oil indicator | **No** | Level with a dipstick on the cap | [VERIFIED] | [Haynes p.38](https://www.manualslib.com/manual/927505/Yamaha-Tdm850.html?page=38), [tdm-forum t=2164](https://tdm-forum.net/viewtopic.php?t=2164) |
| Clock | **No** | | [VERIFIED] | same |
| Neutral indicator | Yes, green | | [VERIFIED] | [tdm-forum t=27514](https://tdm-forum.net/viewtopic.php?t=27514) |
| High beam indicator | Yes, blue | | [VERIFIED] | same |
| Turn signal indicator | Yes, **a single one** for both sides | Connected between the two turn signal lines | [VERIFIED] | [TDMFWiki LED-Blinker](https://wiki.tdm-forum.net/index.php/LED-Blinker_an_die_TDM), [tdm-forum t=29044](https://tdm-forum.net/viewtopic.php?t=29044) |
| Side stand indicator | Not found | | [HYPOTHESIS] absent | — |
| Bulbs | T10 wedge, 12 V 3.4 W (warning lights and illumination) | The forum also reports 2–3 W | [TO VERIFY] | [4TX 1996 manual](https://www.journey.cz/moto/navody/Yamaha%20TDM850%20rv96.pdf), [tdm-forum t=4338](https://tdm-forum.net/viewtopic.php?t=4338) |
| Mounting | Two rubber anti-vibration mounts, spacer, clips and nuts. The fairing must be removed to reach the lower nuts | | [VERIFIED] | [oem-parts.hu](https://oem-parts.hu/en/parts/yamaha/motor-robogo/700cc-1099cc/tdm-850/tdm850-3vd5-1993/meter), [Haynes p.189](https://www.manualslib.com/manual/927505/Yamaha-Tdm850.html?page=189) |
| Cluster harness | Cord set 3VD-83509-00 | A used one provides the right female connectors for the adapter | [VERIFIED] part number | bike-parts-yam |
| Cluster dimensions | Not found | To be measured | — | — |

## 3. Speedometer and front wheel

| Fact | Value | Status | Source |
|---|---|---|---|
| Drive | Cable from the drive on the front hub. The drive has tabs that engage the hub and is part of the spacer stack on the axle | [VERIFIED] | [Haynes p.190](https://www.manualslib.com/manual/927505/Yamaha-Tdm850.html?page=190) |
| Cable ratio | Not found for the TDM. The Japanese standard is 1400 cable turns/km | [HYPOTHESIS] | [xjbikes](https://xjbikes.com/forums/threads/gear-ratios-for-speedometer-and-tachometer.14114/) |
| Front tyre | 110/80-18 | [VERIFIED] | [Rider](https://ridermagazine.com/2007/05/22/retrospective-yamaha-tdm-850-1992-1993/), [TDMitalia](https://www.tdmitalia.net/?u=lemoto%2Fdatitdm850-91) |
| Rear tyre | 150/70-17 | [VERIFIED] | same |
| Nominal front circumference | 457.2 + 2 × 88 = 633.2 mm diameter → **1989 mm** | [HYPOTHESIS] calculation | — |
| Rolling circumference | **1950–1970 mm** (squash under load) | [HYPOTHESIS] estimate; calibrated with GPS | — |
| Front disc bolts | Number and material to be measured (needed for the inductive sensor) | — | — |

With the inductive sensor on the bolts the cable ratio is not needed: the pulses per wheel revolution are measured directly.

## 4. Tachometer

| Fact | Value | Status | Source |
|---|---|---|---|
| Signal source | Coil primary negative, in parallel with the TCI unit. The 3VD does **not** have a dedicated tachometer output (the 4TX does) | [VERIFIED] | [tdm-forum t=6889](https://tdm-forum.net/viewtopic.php?t=6889), [t=7095](https://tdm-forum.net/viewtopic.php?t=7095), [t=33812](https://tdm-forum.net/viewtopic.php?t=33812) |
| Coil | A single one, 3VD-82310-00 (later 3LD-82310-09), two plug leads (wasted spark) | [VERIFIED] | [Yambits](https://yambits.co.uk/tdm850-ignition-coil-1991-1995-p-86771.html) |
| Coil resistance | Primary 2.38–3.22 Ω, secondary 12–18 kΩ | [TO VERIFY] | [tdm-forum t=32766](https://tdm-forum.net/viewtopic.php?t=32766) |
| Pulses per revolution | **1** (360° twin: "fires every revolution like a two-stroke") | [TO VERIFY] | tdm-forum t=6889 |
| Frequencies | 16.7 Hz at 1000 rpm, 133 Hz at 8000 rpm | [HYPOTHESIS] calculation | — |
| Signal shape | ≈ +12 V at rest, ≈ 0 V while the coil charges, then an inductive spike of **a few hundred volts** at every spark, with ringing | [HYPOTHESIS] typical TCI behaviour | [Acewell](https://www.acewell-meter.com/wp-content/uploads/2016/02/ACE-27-28XX-EN-V5-manual-20150512.pdf), [Koso RX2N](https://www.manualslib.com/manual/1885543/Koso-Rx2n.html?page=16) |
| Wire colour | Orange (O) or grey (Gy) | [HYPOTHESIS] | [xjbikes](https://xjbikes.com/forums/threads/wiring-question-xj650-tach.125606/), [tenere.co.uk](https://tenere.co.uk/forum/viewtopic.php?p=27246) |
| Red line | ≈ 8000 rpm. Peak power at 7500 rpm | [TO VERIFY] | [motorcyclespecs](https://www.motorcyclespecs.co.za/model/yamaha/yamaha_tdm850%2091.htm) |

## 5. Wires and connectors at the cluster

Yamaha colour code: B black, Br brown, Ch chocolate, Dg dark green, G green, Gy grey, L blue, O orange, P pink, R red, Sb sky blue, W white, Y yellow. Two letters separated by `/` give the wire and the stripe (e.g. `L/W` = blue with a white stripe). Source: [TDM '96/'97 manual](https://manualmachine.com/yamaha/tdm850/8415354-user-manual/).

| Function | Probable colour | Logic | Status | Source |
|---|---|---|---|---|
| Switched +12 V | Br | +12 V with the ignition key ON | [TO VERIFY] | [TDM '99 supplement](https://tdm-trx-tracer.nl/forums/applications/core/interface/file/attachment.php?id=2543), [trx850.com](http://www.trx850.com/phpBB_forum/viewtopic.php?t=10256) |
| Ground | B | — | [HYPOTHESIS] convention | — |
| Cluster illumination | L | +12 V with the position lights | [HYPOTHESIS] convention | — |
| Neutral indicator | Sb | The lamp gets +12 V; the neutral switch **closes to ground** → active low | [HYPOTHESIS] convention; switch on the left near the front sprocket ([t=36168](https://tdm-forum.net/viewtopic.php?t=36168)) | — |
| High beam indicator | Y | +12 V with the high beam on → active high | [HYPOTHESIS] convention | — |
| Left / right turn signal | Ch / Dg | Flashing +12 V; the single indicator is connected **between** Ch and Dg | [VERIFIED] the single indicator; [TO VERIFY] the colours on the 3VD | [t=29044](https://tdm-forum.net/viewtopic.php?t=29044) |
| Tachometer | O or Gy | Coil negative | [HYPOTHESIS] | section 4 |
| Coolant sender | ? | Signal to the NTC to ground | [TO VERIFY] | — |
| Connector | A **6-pin** behind the instruments is mentioned, make unknown | — | [TO VERIFY] | [t=2164](https://tdm-forum.net/viewtopic.php?t=2164) |
| Permanent +12 V at the cluster | Probably absent (there is no clock) | — | [HYPOTHESIS] | — |

**Consequences for the project:**
- Neutral is active low: with the bulb removed, the line floats when not in neutral, so the input must power its own optocoupler from +12 V.
- The turn signals must be read with **two separate inputs**, both active at +12 V, each with its own pull-down load to ground.
- Koso uses the same logic too ("neutral (−)", "high beam (+12V)"), [RX2 manual](https://media.motoblouz.com/uploads/products_document/79537/notice-compteur-koso-rx2.pdf).

## 6. Flasher

| Fact | Value | Status | Source |
|---|---|---|---|
| Type | 3-terminal relay, load sensitive | [TO VERIFY] | [TDMFWiki](https://wiki.tdm-forum.net/index.php/LED-Blinker_an_die_TDM) |
| Effect of removing the indicator | The indicator load (3.4 W) is small compared with the turn signals (2 × 21 W per side): the rate should not change | [HYPOTHESIS] | — |

## 7. Coolant temperature

| Fact | Value | Status | Source |
|---|---|---|---|
| Instrument | Needle, no warning light (the warning light arrives in 1999) | [VERIFIED] | [Haynes p.102](https://www.manualslib.com/manual/927505/Yamaha-Tdm850.html?page=102) |
| Sender | In the thermostat housing, one wire, NTC to ground | [TO VERIFY] | same |
| Resistance | **47–53 Ω at 80 °C**, **26–29 Ω at 100 °C** (one copy says 26–30 Ω) | [TO VERIFY] | same |
| Other points of the curve | Not found: measured in a water bath (20/40/60/80/90/100 °C) | — | — |
| Alternative | Koso M14×1.5 adapter for a separate sensor, listed for the TDM 850 | [TO VERIFY] | [kedo](https://www.kedo.com/koso-adaptor-m14x1-5-15mm-for-temp-sensor-oil-water-1-8-x28tpi.html) |

## 8. Charging and battery

| Fact | Value | Status | Source |
|---|---|---|---|
| Alternator | Permanent magnet rotor, three-phase, three white wires, ≈ 280 W (the forum also says 300–340 W) | [TO VERIFY] | [tdm-forum t=34705](https://tdm-forum.net/viewtopic.php?t=34705) |
| Regulator | Shindengen SH629A-12 (3VD-81960-00/-01), shunt type | [VERIFIED] | [officine08](https://www.officine08.com/p67451_regulator-yamaha-tdm-850.html), [eBay](https://www.ebay.de/itm/143232889593) |
| Known weak point | Regulator and stator connector burning out | [VERIFIED] | various forums |
| Charging voltage | 14.0–14.5 V above 1200–2000 rpm | [TO VERIFY] | forum |
| Battery | **YTX12-BS**, 12 V 10 Ah | [VERIFIED] | [Yuasa](https://www.motoracingshop.com/it/batteria-yuasa-per-yamaha-tdm-850-91-95-ytx12-bs-da-12v-10ah-152x88x131.html) |
| Fuses | Headlight 15 A, signal 15 A, ignition 7.5 A (4TX data) | [TO VERIFY] on the 3VD | [4TX manual](https://www.journey.cz/moto/navody/Yamaha%20TDM850%20rv96.pdf) |
| Interference | A faulty regulator can damage the tachometer electronics; a loose positive terminal makes the needles jump | [VERIFIED] | [tenere.co.uk](https://www.tenere.co.uk/forum/viewtopic.php?t=1479), [tdm-forum t=18602](https://tdm-forum.net/viewtopic.php?t=18602) |

## 9. Fuel

| Fact | Value | Status | Source |
|---|---|---|---|
| Tank | 18 L, of which 3.5 L reserve (the 4TX: 20 L / 3.1 L) | [VERIFIED] | [3VD owner's manual](https://www.scribd.com/document/696775450/User-manual-TDM850) |
| Sensor | None | [VERIFIED] | section 2 |

The range can only be **estimated** from the km ridden since the last fill-up (feature planned after v1.0).

## 10. Reference documentation

**Manuals:**
- **3VD owner's manual (3VD-28199)**, with a fold-out wiring diagram in the paper copies: [manualslib](https://www.manualslib.com/manual/1937486/Yamaha-Tdm850.html), [Scribd](https://www.scribd.com/document/696775450/User-manual-TDM850), [archive.org (1995)](https://archive.org/details/YamahaTDM850OwnersManual1995), [French edition](https://www.manualslib.fr/manual/1046161/Yamaha-Tdm850.html).
- **Haynes 3540** (TDM850, TRX850, XTZ750), with colour wiring diagrams: [manualslib](https://www.manualslib.com/manual/927505/Yamaha-Tdm850.html).
- **4TX service manuals** (similar, not identical): [rv96](https://www.journey.cz/moto/navody/Yamaha%20TDM850%20rv96.pdf), ['97](https://www.journey.cz/moto/navody/Yamaha%20TDM850%20'97%20Service%20Manual.pdf), [1997 supplement](https://archive.org/details/printermanual-yamaha-tdm-850-factory-1997-service-manual-supplement).
- **3VD workshop manual** (aftermarket, PDF): [retromotorcyclemanuals](https://retromotorcyclemanuals.com/en-us/products/1991-1995-yamaha-tdm850-tdm-850-service-manual).
- **Ignitech wiring diagram for the TDM 850**, useful for the colours at the TCI pins: [ignitech](https://www.ignitech.cz/en/vyrobky/tci/mototci.htm).

**Italian sources:** [tdmitalia.net (1991 data)](https://www.tdmitalia.net/?u=lemoto%2Fdatitdm850-91), [Motosprint "Fai da te" guide](https://www.tdmitalia.net/documenti/manuali/FaidateTDM850.pdf). Caution: the "libretto uso e manutenzione" on [ManualMachine](https://manualmachine.com/yamaha/tdm850/10910352-user-manual/) is for the **4TX**, not the 3VD.

**Forums:** [tdm-forum.net, electrical section](https://tdm-forum.net/viewforum.php?f=21), [Carpe TDM](http://www.carpe-tdm.net/ipb/index.php?showtopic=36935), [tdm-trx-tracer.nl](https://tdm-trx-tracer.nl/forums/forum/4-tdm/).

## 11. Similar projects

- **Koso, Acewell, motogadget** fitted to TDMs and TRXs: [trx850.com t=10550](http://www.trx850.com/phpBB_forum/viewtopic.php?t=10550), [tdm-forum t=35804](https://tdm-forum.net/viewtopic.php?t=35804), [t=20197](https://tdm-forum.net/viewtopic.php?t=20197).
- **Cable to electronic signal converters:** Daytona lists the TDM850 and TRX850 as compatible ([caferacergarage](https://caferacergarage.com/product/daytona-speed-pulse-converter/)); Koso "JIS Type A" with an M12×1.0 thread ([RevZilla](https://www.revzilla.com/motorcycle/koso-speed-signal-converter-for-koso-speedometers)).
- **Tachometer from the coil on other bikes:** the coil negative gives steadier readings than a wire wrapped around the plug lead ([kzrider](https://www.kzrider.com/forum/4-electrical/98375-update-on-acewell-digital-speedo-tach)); a 1 kΩ resistor with a capacitor helps ([wildguzzi](https://wildguzzi.com/forum/index.php?topic=94616.0)).
- No DIY project (Arduino or similar) for the TDM was found.

## 12. Discrepancies between sources

| Topic | Discrepancy | How we resolve it |
|---|---|---|
| Fuel gauge | Some reviews mention it, but they refer to the 1999 model | The 3VD does not have one |
| Bulbs | 3.4 W (1996 manual), 1.4/2 W (1999), 2–3 W (forum) | Irrelevant: we remove them |
| Tank | 18 L / 3.5 L (3VD) versus 20 L / 3.1 L (4TX) | We use the 3VD figure |
| Alternator | 280 W versus 300–340 W | Irrelevant: the dashboard draws ≈ 5 W |
| XTZ 750 tachometer wire | Green/yellow in one post, orange → grey in another | Measured on the bike |
| 4EP years | 1992–94 Japan versus a 1999 listing | Irrelevant |
