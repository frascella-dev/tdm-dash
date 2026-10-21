# Contributing

Thanks for your interest! Contributions are welcome, in particular:
- **measurements on the bike:** pinouts, waveforms, sensor curves, with photos
- **profiles for other bikes** in `profiles/`
- corrections to the manual

## Rules

- **Language:** documentation, code, comments and commit messages in English.
- **Diagrams:** Mermaid inside Markdown. Schematics only in KiCad (`hardware/kicad/`), wiring harnesses in WireViz (`hardware/harness/`).
- **Facts about the original wiring:** always give the source and the tag ([VERIFIED], [TO VERIFY], [HYPOTHESIS]) as in [`docs/research/original-wiring.md`](docs/research/original-wiring.md).
- **Architecture decisions:** a new ADR in `docs/adr/`.
- **Before a PR:**

  ```sh
  cargo fmt --all
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace
  ```

- **Protocol:** variants and fields are only appended at the end; an incompatible change bumps `PROTO_VERSION`.

## Profiles for other bikes

1. Copy `profiles/yamaha-tdm850-3vd.toml` to `profiles/<make>-<model>-<code>.toml`.
2. Edit the wheel circumference, pulses per revolution, red line, coolant sender curve and inputs.
3. Add the sources or measurements you used to the PR.

The tests automatically check that every profile is valid.
