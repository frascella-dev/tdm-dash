# 0006 · Backlight driven by the microcontroller, "fail-dark"

**Status:** accepted · October 2026

## Context

A frozen dashboard showing 80 km/h when stopped, or the other way round, is worse than a dark dashboard. The display must only be lit when it shows valid data.

## Decision

- The **Pico drives the backlight PWM** through the TPS61169 boost.
- The Pi sends `SetBacklight` only after the first frame is on the screen, then a `Heartbeat` every 250 ms.
- If the heartbeat is missing for more than **2 s**, the Pico turns the backlight off; the Pi watchdog restarts it.
- **Passive backup LEDs** on the 12 V lines (turn signals, high beam, neutral) guarantee the mandatory warning lights even with the electronics off.

## Consequences

- No "stale" frame is visible at startup or when frozen.
- The Pi uses no GPIO beyond the DPI.
- The Pico measures the time from power-on to the first `SetBacklight`: it is the perceived boot time.
