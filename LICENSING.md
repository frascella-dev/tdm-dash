# Licensing

tdm-dash includes software, a system image, hardware and documentation, under different licenses (see [ADR 0007](docs/adr/0007-licenses.md)).

| Part | Path | License |
|---|---|---|
| Source code (Rust crates, firmware, tools) | `crates/`, `firmware/`, `tools/`, `os/` | [MIT](LICENSE-MIT) **or** [Apache-2.0](LICENSE-APACHE), at your option |
| Distributed system image (`sdcard.img`) | GitHub releases | **GPL-3.0-only** as a whole, because it includes [Slint](https://github.com/slint-ui/slint) under the GPLv3 licence. Every release includes the output of `make legal-info` with the sources and licenses of all components |
| Hardware (KiCad schematics, harnesses, CAD) | `hardware/` | [CERN-OHL-P-2.0](https://ohwr.org/cern_ohl_p_v2.txt) |
| Documentation, manual, design | `docs/`, `design/` | [CC-BY-SA-4.0](https://creativecommons.org/licenses/by-sa/4.0/) |
| Bundled fonts | `crates/tdm-dash/fonts/`, `design/` | [SIL OFL-1.1](https://openfontlicense.org) |

## Contributions

Unless stated otherwise, every contribution submitted for inclusion in this project is granted under the same licenses as the part it modifies, without any additional terms.

## Slint and commercial use

Slint's royalty-free license **excludes** embedded systems. Anyone who wants to distribute a closed derivative must:
- buy a commercial Slint license, **or**
- replace the interface layer (`crates/tdm-dash`).

The rest of the code (protocol, measurement, logic, firmware) stays MIT/Apache.

## Disclaimer of warranty

The project is provided "as is", without warranty of any kind. It is not a certified device. See the licenses and the [Introduction and safety](docs/manual/00-introduction-and-safety.md) chapter.
