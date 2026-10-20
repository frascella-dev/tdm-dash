# tdm-dash: notes for Claude

Digital instrument cluster for a 1992 Yamaha TDM 850 (3VD): Raspberry Pi Zero 2 W (Buildroot Linux + Rust/Slint UI) and an RP2040 co-processor (Rust/embassy) over USB CDC. The approved plan, milestones M0–M6, is summarised in `README.md` and `docs/adr/`.

## Language

- Documentation (`docs/`, `README.md`): **Italian**.
- Code, comments, commit messages: **English**.

## Diagrams

- Everything that is not a real schematic is **Mermaid inside Markdown**. No ASCII art diagrams.
- Real schematics: KiCad 9 in `hardware/kicad/`.
- Wiring harnesses: WireViz YAML in `hardware/harness/`.
- Exports go to `hardware/exports/`.

## Facts about the bike

- Every fact about the original wiring is tagged [VERIFICATO] / [DA VERIFICARE] / [IPOTESI] in `docs/ricerca/impianto-originale.md`.
- Never present a [DA VERIFICARE] wire colour as certain in schematics or harnesses.

## Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build -p tdm-proto -p tdm-core --target thumbv6m-none-eabi   # no_std check
python3 tools/check_mermaid.py --mmdc <path-to-mmdc> [--chrome <chromium>]
```

In the cloud container, Chromium is at `/opt/pw-browsers/chromium-1194/chrome-linux/chrome`. Install mermaid-cli with:

```sh
PUPPETEER_SKIP_DOWNLOAD=1 npm i --no-save @mermaid-js/mermaid-cli@11.16.0
```

## Invariants

- `tdm-proto` and `tdm-core` are `no_std` and shared by the firmware and the simulator.
- Protocol variants and fields are append-only; breaking changes bump `PROTO_VERSION`.
- The MCU is the source of truth for the odometer (FRAM). The Pi only displays it.
- The displayed speed must never be below the true speed (UNECE R39); keep the legal-bounds test green.
- No Yamaha logos or trademarks in assets.
