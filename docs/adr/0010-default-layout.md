# 0010 · Default layout: Classic

**Status:** accepted · October 2026

## Context

The design proposal (milestone M1) has three riding layouts: Classic, Sport and Touring. The owner asked to go ahead with the milestones without stopping for the choices; the final choice remains theirs.

## Decision

- **Classic** is the default in the `yamaha-tdm850-3vd.toml` profile: two dials, the reading closest to the original TDM cluster.
- Sport and Touring stay available, from Settings or from the profile (`[display] layout`).
- The choice made from Settings is saved in the Pico FRAM and takes precedence over the profile.

## Consequences

- Changing the default only needs one line in the profile, no recompiling.
- All three layouts are covered by the reference screens (`design/screenshots/`) and by the rendering test.
