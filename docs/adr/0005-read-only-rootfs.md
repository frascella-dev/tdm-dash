# 0005 · Read-only EROFS rootfs, no orderly shutdown

**Status:** accepted · October 2026

## Context

The dashboard loses power every time the key is turned. An orderly shutdown would need supercapacitors and dedicated circuits.

## Decision

- **Compressed EROFS rootfs, mounted read-only.**
- `/boot` (FAT) mounted read-only: it holds the profile, editable from a PC.
- `/tmp` and `/run` in tmpfs.
- No orderly shutdown of the Pi; only the Pico has a small energy reserve (ADR 0004).

## Consequences

- Power can be cut at any time without damage.
- User settings and km live in the FRAM, not on the SD card.
- The development image (network, ssh, writable) is a separate defconfig.
