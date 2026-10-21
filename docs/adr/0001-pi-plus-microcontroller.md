# 0001 · Pi Zero 2 W for graphics, RP2040 for I/O

**Status:** accepted · October 2026

## Context

The dashboard must count wheel pulses (up to ≈ 170 Hz) and coil pulses (≈ 140 Hz), read warning lights and voltages, keep the km and draw a smooth 800×480 interface. It must be cheap and withstand power fails.

The limits of a Raspberry Pi alone:
- it has no ADC
- it has no hardware capture timers
- in user space it has jitter from tens of µs to milliseconds
- it counts nothing during the 3–5 s of boot
- with the DPI display the UART is not available

## Decision

- **Raspberry Pi Zero 2 W** (−20…+70 °C, official Buildroot, GPU) for graphics only.
- **Raspberry Pi Pico (RP2040)**, or Pico 2, for all the real-time I/O, the km and the backlight.

Rejected alternative: Allwinner T113-S3. It is cheaper and boots faster, but it has no GPU and a smaller community.

## Consequences

- The Pico is up in milliseconds: no pulse is lost while Linux boots.
- The Pi uses no GPIO beyond the DPI.
- Two firmwares to maintain, but with shared code (`tdm-core`, `tdm-proto`).
- The Pi Zero 2 W is often out of stock: order it first.
