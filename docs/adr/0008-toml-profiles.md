# 0008 · Layered TOML bike profiles

**Status:** accepted · October 2026

## Context

The project must be able to serve other bikes without recompiling: wheel circumference and pulses per revolution, pulses per engine revolution, red line, coolant sender curve, battery thresholds, connected inputs.

## Decision

**Profile in TOML**, with `schema = 1` and `deny_unknown_fields`. The first file found wins, in this order:
1. `/boot/tdm-dash.toml`, editable from a PC
2. `/etc/tdm-dash/default.toml`, in the image
3. profile compiled into the app

**On error** the built-in profile is used and the `ConfigError` warning is shown.

**User settings** (theme, layout, brightness, fine speed correction): saved in the FRAM through the Pico, applied on top of the profile.

**No inheritance between profiles:** each profile is complete and self-contained.

## Consequences

- For a new bike it is enough to copy an existing profile and edit it.
- All the profiles in the repo are checked by the tests (parsing, monotonic NTC curve).
