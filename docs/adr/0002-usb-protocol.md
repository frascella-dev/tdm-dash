# 0002 · USB CDC with postcard, COBS and CRC-16 with direction tag

**Status:** accepted · October 2026

## Context

The DPI display takes GPIO 14/15 of the UART. The Pico and the Pi must exchange telemetry at 50 Hz and rare commands, robustly against interference, reconnections and the echo of a tty not yet in raw mode.

## Decision

**Transport:** USB CDC-ACM. The Pico is the device; the Pi is the host, with `dwc2,dr_mode=host` forced because the ID pin is not connected.

**Frame format:** `postcard(message) ‖ CRC-16`, COBS encoded and terminated by `0x00`.

**CRC with direction tag:** the CRC also covers a byte that is not transmitted: `0xA5` for frames from the Pico to the Pi, `0x5A` for those from the Pi to the Pico. A Pico frame sent back by the tty echo fails the check and cannot be mistaken for a command, for example `ResetTrip`.

**Dialogue rules:**
- **the host speaks first:** the Pico stays silent until it receives a valid frame
- `PROTO_VERSION` in the `Hello` message
- variants and fields are only appended at the end

## Consequences

- The same `tdm-proto` crate (`no_std`) is used by the firmware, the app and the simulator.
- The Pi looks for the device by VID:PID in `/sys/class/tty/ttyACM*`, without assuming `ttyACM0`.
- The USB cable must be soldered and protected (short twisted pair, ferrite).
