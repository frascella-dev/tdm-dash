# 02 · System architecture

## In brief

tdm-dash is split into two computers, each with the job it does best:

| Part | Job | Why |
|---|---|---|
| **Raspberry Pi Pico (RP2040)**, microcontroller | Counts wheel and coil pulses, reads warning lights, buttons and voltages, **keeps the km** in a FRAM memory, drives the backlight | It is ready a few milliseconds after power-on, measures time with microsecond precision, does not hang and has no file system to corrupt |
| **Raspberry Pi Zero 2 W**, Linux computer | Draws the graphical interface on the display | It has the power for smooth graphics at 800×480; with a minimal Linux it boots in a few seconds |

The two communicate over **USB** with small packets protected by a CRC. The Pi never writes to the SD card during use: if the key is turned off suddenly, nothing gets corrupted.

## Block diagram

```mermaid
flowchart LR
  subgraph MOTO["TDM 850 (3VD) wiring"]
    KEY["+12V switched (Br)"]
    SIG["Warning lights: neutral Sb · high beam Y · turn signals Ch/Dg · lights L"]
    COIL["Coil negative (tachometer)"]
    NTC["NTC coolant sender"]
  end
  subgraph PWR["Protected power supply"]
    F["2A fuse"] --> RP["Reverse-polarity P-MOSFET"] --> TVS["TVS SMCJ24A"] --> LC["LC filter"] --> BUCK["Buck 50V→5V D36V28F5"]
  end
  KEY --> F
  subgraph IO["Signal conditioning"]
    OPTO["Opto EL817 ×5"]
    TACH["Zener + H11L1"]
    ADC["Dividers / pull-up + clamp"]
  end
  SIG --> OPTO
  COIL --> TACH
  NTC --> ADC
  KEY --> ADC
  WHEEL["LJ12A3 inductive sensor (disc bolts)"] --> OPTO
  LUX["TEMT6000 ambient light"] --> ADC
  BTN["MODE / SET buttons"] --> MCU
  OPTO --> MCU
  TACH --> MCU
  ADC --> MCU
  MCU["RP2040 Pico · Rust embassy · FRAM km"] -- "USB CDC (postcard+COBS+CRC)" --> PI["Pi Zero 2 W · Buildroot · Rust/Slint"]
  BUCK --> MCU
  BUCK --> PI
  PI -- DPI --> LCD["7 inch LCD 800×480 1000 nit"]
  MCU -- "Backlight PWM" --> BL["Boost TPS61169"] --> LCD
  SIG --> LED["Passive backup LEDs"]
```

## The path of a reading: from the wheel to the needle

```mermaid
flowchart LR
  A["Disc bolt<br/>passes in front of the sensor"] --> B["LJ12A3: NPN output<br/>to ground"]
  B --> C["Optocoupler<br/>12 V → 3.3 V"]
  C --> D["RP2040: edge with<br/>µs timestamp"]
  D --> E["PulseMeter: average over<br/>whole wheel revolutions"]
  E --> F["FRAM: +distance<br/>every 10 m"]
  E --> G["Telemetry 50 Hz<br/>over USB"]
  G --> H["Pi: speed = distance/time,<br/>median, filter"]
  H --> I["Legal margin<br/>×1.03 +1 km/h"]
  I --> J["Needle (spring)<br/>and digits (hysteresis)"]
```

Why the average "over whole revolutions": the disc bolts may not be spaced perfectly evenly. By always measuring a whole number of wheel revolutions, the spacing error cancels out.

## Software

```mermaid
flowchart TB
  subgraph PI["Raspberry Pi Zero 2 W"]
    direction TB
    FW["GPU firmware (start_cd.elf)"] --> K["Trimmed Linux 6.12 kernel<br/>DRM vc4 · DPI · USB ACM · watchdog"]
    K --> INIT["BusyBox init"]
    INIT --> APP["tdm-dash (Rust)<br/>logic + Slint UI"]
    APP --> DRM["DRM/KMS<br/>double buffer + page-flip"]
  end
  subgraph PICO["Raspberry Pi Pico"]
    direction TB
    E["Rust + embassy-rp"] --> T1["Pulse task<br/>(high priority)"]
    E --> T2["ADC and inputs task"]
    E --> T3["FRAM task"]
    E --> T4["USB task"]
    E --> T5["Backlight task"]
  end
  T4 <-- "USB CDC" --> APP
```

| Component | Choice | Where in the repo |
|---|---|---|
| Operating system | Buildroot 2026.08, read-only EROFS rootfs, no udev, no networking in the release image | `os/` |
| Interface | Rust + Slint, software renderer, own DRM platform | `crates/tdm-dash/` |
| Logic (filters, warnings, states) | Rust, no hardware dependencies, tested on the PC | `crates/tdm-logic/` |
| Protocol | postcard + COBS + CRC-16, shared crate | `crates/tdm-proto/` |
| Measurements and odometer | Rust `no_std`, **the same code** in the firmware and in the simulator | `crates/tdm-core/` |
| Firmware | Rust + embassy-rp | `firmware/` |
| Bike profiles | TOML | `profiles/` |

## Boot

```mermaid
sequenceDiagram
  autonumber
  participant K as Key
  participant M as Pico (MCU)
  participant P as Pi Zero 2 W
  participant D as Display
  K->>M: +12 V
  Note over M: ≈ 10 ms: reads FRAM,<br/>already counts pulses and km
  K->>P: +5 V
  Note over P: GPU firmware, kernel,<br/>init (target < 4 s)
  P->>D: first frame (backlight still off)
  P->>M: GetInfo
  M-->>P: Hello (version, configuration, km)
  P->>M: SetConfig (if different from the profile)
  P->>M: SetBacklight (300 ms fade)
  M->>D: Backlight PWM
  Note over P,D: self-test: needles 0 → full scale → 0
  loop every 20 ms
    M-->>P: Telemetry
  end
  loop every 250 ms
    P->>M: Heartbeat
  end
```

While Linux boots the microcontroller is already working: if you ride off straight away, the km covered in the first seconds are not lost.

## Interface states

```mermaid
stateDiagram-v2
  [*] --> Splash: first page-flip
  Splash --> SelfTest: link ok, speed < 5
  Splash --> Riding: link ok, speed ≥ 5<br/>(restart while moving)
  Splash --> Riding: no link after 3 s<br/>(Sensor module link lost warning)
  SelfTest --> Riding: 1.2 s, or speed ≥ 5
  state Riding {
    [*] --> Main
    Main --> Trip: short MODE
    Trip --> Diagnostics: short MODE (if enabled)
    Diagnostics --> Main: short MODE
    Trip --> Main: short MODE
  }
  Riding --> Settings: long SET, speed < 3
  Settings --> Riding: exit, 20 s of inactivity or speed ≥ 5
  Riding --> Off: PowerFail
  Settings --> Off: PowerFail
  Off --> [*]
```

**Warnings** are not states, but a layer on top of the active screen:
- **Critical** (red band): link lost, critical temperature, overvoltage.
- **Caution** (amber banner), can be snoozed for 5 minutes with SET: high temperature, battery not charging, battery low.
- **Information** (icon): turn signal left on, fuel estimate.

## Shutdown and power fail

```mermaid
sequenceDiagram
  participant K as Key / battery
  participant M as Pico (MCU)
  participant P as Pi
  K->>M: voltage < 7 V for 2 ms
  M->>M: saves km to FRAM (≈ 1 ms, 470 µF reserve)
  M-->>P: PowerFail
  M->>M: switches off the backlight
  Note over P: read-only rootfs:<br/>can lose power at any moment
```

The Pi does not need an orderly shutdown. The FRAM withstands about 10¹² writes: even writing every 10 m, 100 000 km means 10⁷ writes, a tiny fraction of its life.

## Fail-safe behavior ("fail-dark")

| Fault | What happens | Why |
|---|---|---|
| The Pi app hangs | After 2 s without Heartbeat the Pico switches off the backlight; the watchdog restarts the Pi | Better a black screen than a frozen, wrong speed |
| The USB link drops | After 0.3 s dimmed digits; after 1 s "---" and a critical warning; automatic reconnection | Old data must not look valid |
| The Pico restarts | The Pi notices (uptime starting over) and resets the filters | No needle jumps |
| The FRAM is unreadable | The Pico works, reports the fault, does not overwrite the data | The km history is not lost |
| Electronics off or faulty | The **backup LEDs** on the 12 V lines show turn signals, high beam and neutral | Mandatory warning lights always visible |

## Extensibility: bike profiles

Everything that depends on the bike lives in a TOML file (`profiles/`):
- pulses per wheel revolution and circumference
- pulses per engine revolution, red line
- coolant sender curve, battery thresholds
- which inputs are connected and with which logic
- layout and theme

The file in use is on the `/boot` partition of the SD card, readable and editable from any PC. For another bike a new profile is enough; if the profile has an error, the dashboard uses the built-in one and reports it.
