# 0007 · Licenses

**Status:** accepted · October 2026

## Context

The project is public and extensible. Slint is available under GPLv3, under a royalty-free license (which **excludes** embedded systems, naming a car dashboard as an example) or under a commercial licence.

## Decision

| Part | License |
|---|---|
| Project code (crates, firmware, tools) | **MIT OR Apache-2.0** |
| Distributed system image (contains Slint) | **GPL-3.0** as a whole |
| Hardware (KiCad, harnesses, CAD) | **CERN-OHL-P-2.0** |
| Documentation and manual | **CC-BY-SA-4.0** |
| Bundled fonts | **OFL-1.1** |

## Consequences

- Every image release includes the output of Buildroot's `make legal-info` (sources and licenses).
- Anyone who wants a closed commercial derivative must buy Slint's commercial license or replace the interface layer.
- No Yamaha logos or trademarks in the repo, in the splash or in the design.
