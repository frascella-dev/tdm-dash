# Contribuire

Grazie per l'interesse! Contributi benvenuti, in particolare:
- **misure sulla moto:** piedinature, forme d'onda, curve dei sensori, con foto
- **profili per altre moto** in `profiles/`
- correzioni al manuale

## Regole

- **Lingua:** documentazione in italiano; codice, commenti e messaggi di commit in inglese.
- **Diagrammi:** Mermaid dentro Markdown. Gli schemi elettrici solo in KiCad (`hardware/kicad/`), i cablaggi in WireViz (`hardware/harness/`).
- **Dati sull'impianto originale:** indica sempre la fonte e la marcatura ([VERIFICATO], [DA VERIFICARE], [IPOTESI]) come in [`docs/ricerca/impianto-originale.md`](docs/ricerca/impianto-originale.md).
- **Decisioni di architettura:** un nuovo ADR in `docs/adr/`.
- **Prima di una PR:**

  ```sh
  cargo fmt --all
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace
  ```

- **Protocollo:** varianti e campi si aggiungono solo in coda; una modifica incompatibile incrementa `PROTO_VERSION`.

## Profili per altre moto

1. Copia `profiles/yamaha-tdm850-3vd.toml` in `profiles/<marca>-<modello>-<codice>.toml`.
2. Modifica circonferenza della ruota, impulsi per giro, zona rossa, curva del sensore della temperatura e ingressi.
3. Aggiungi nella PR le fonti o le misure usate.

I test verificano automaticamente che ogni profilo sia valido.
