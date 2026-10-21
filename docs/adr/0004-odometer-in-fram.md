# 0004 · Odometer on the microcontroller, in FRAM

**Status:** accepted · October 2026

## Context

The km must never go backwards or be lost, even when the key is turned off suddenly, the battery is disconnected or the Pi hangs. SD cards wear out with frequent writes and power interruptions.

## Decision

- The **Pico is the source of truth**. It accumulates the odometer and two trips in micrometres at every valid wheel pulse, even when the Pi has not booted yet.
- **MB85RC256V FRAM memory** on I²C (10¹² writes). Each record is `magic | seq | data | crc32`, written alternating between two slots; at startup the valid slot with the highest `seq` is loaded.
- Write **every 10 m**, on power fail (below 7 V for 2 ms, with 470 µF of reserve) and when a trip is reset.
- **Blank FRAM:** start from 0 and accept `InitOdometer` **once only**, with the value photographed from the original cluster.
- **Corrupted FRAM:** never overwritten; a fault is reported.
- The Pi **only displays**.

## Consequences

- At most 10 m lost in case of a power failure.
- No writes to the SD card during use.
- A wheel recalibration only changes future distance, not the distance already ridden.
