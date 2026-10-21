<p align="center"><img src="docs/assets/banner.svg" alt="tdm-dash: quadro strumenti digitale open-source" width="100%"></p>

# tdm-dash

**Quadro strumenti digitale open-source per Yamaha TDM 850 prima serie (3VD, 1991–1995).**
*Open-source digital instrument cluster for the first-generation Yamaha TDM 850. Documentation is in Italian; code and comments are in English.*

> **Stato:** progettazione completa, hardware non ancora costruito.
> - Il software del quadro e il simulatore funzionano sul PC.
> - Pronti ma non ancora provati sull'hardware: firmware del Pico, immagine Linux, schemi elettrici, cablaggi, contenitore, strumenti di calibrazione e manuale.
> - Mancano i rilievi sulla moto e il collaudo al banco e su strada.
>
> **Non** utilizzabile su strada.

tdm-dash sostituisce il quadro originale con un display da 7" leggibile al sole (1000 nit) e mostra:
- velocità e giri
- contachilometri totale e due parziali
- temperatura dell'acqua e tensione della batteria, con avvisi
- spie: folle, abbaglianti, frecce sinistra e destra separate

È economico (≈ 190–240 €), **reversibile** (l'impianto della moto non si taglia) ed **estensibile** ad altre moto tramite profili TOML.

## Come è fatto

```mermaid
flowchart LR
  MOTO["Impianto 12 V<br/>spie · bobina · sensore acqua"] --> IO["Ingressi protetti<br/>optoaccoppiatori"]
  RUOTA["Sensore induttivo<br/>sui bulloni del disco"] --> IO
  IO --> MCU["Raspberry Pi Pico<br/>Rust · km in FRAM"]
  MCU -- USB --> PI["Raspberry Pi Zero 2 W<br/>Linux Buildroot · UI Rust/Slint"]
  PI -- DPI --> LCD["Display 7 pollici<br/>800×480 · 1000 nit"]
```

| Parte | Tecnologia |
|---|---|
| Interfaccia | Rust + [Slint](https://slint.dev), renderer software su DRM/KMS |
| Sistema operativo | Linux minimale compilato con [Buildroot](https://buildroot.org), rootfs in sola lettura, avvio < 4 s |
| Microcontrollore | Rust + [embassy](https://embassy.dev) su RP2040 / RP2350 |
| Hardware | Schema elettrico KiCad, cablaggi WireViz, contenitore stampato in 3D |

## Documentazione

- **[Manuale](docs/manual/README.md)**: progettazione, costruzione, installazione, calibrazione, uso, diagnostica
- **[Ricerca sull'impianto originale](docs/research/original-wiring.md)**, con le fonti
- **[Scheda dei rilievi sulla moto](docs/research/bike-measurements.md)**
- **[Decisioni di architettura](docs/adr/README.md)**

## Struttura del repository

| Cartella | Contenuto |
|---|---|
| [`crates/`](crates/) | tdm-proto (protocollo), tdm-core (misure, km), tdm-logic, tdm-link (`tdmctl`), tdm-dash (interfaccia) |
| [`firmware/`](firmware/) | Firmware del Pico |
| [`profiles/`](profiles/) | Profili moto (TOML) |
| [`design/`](design/) | Proposta di design dell'interfaccia (HTML interattivo) |
| [`os/`](os/) | Immagine Linux: Buildroot e albero esterno |
| [`hardware/`](hardware/) | Schema KiCad, cablaggi WireViz, millefori, contenitore e staffa (OpenSCAD) |
| [`docs/`](docs/) | Manuale, ricerca, ADR |
| [`tools/`](tools/) | Controlli della documentazione e degli schemi, conversione UF2 |

## Sviluppo

| Schermata Classic | Schermata Sport |
|---|---|
| ![Classic](design/screenshots/03-classic-giorno.png) | ![Sport](design/screenshots/06-sport-notte.png) |

```sh
cargo run -p tdm-dash                                   # quadro sul PC con la moto simulata (tasti: capitolo 11)
cargo run -p tdm-link --bin tdmctl -- sim mountain      # simulazione testuale
cargo test --workspace                                  # test sul PC
cargo run --release -p tdm-dash --no-default-features -- --screenshot out   # schermate di riferimento
cargo run --release -p tdm-dash --no-default-features -- --bench 600         # tempo di disegno
cargo build --release --target aarch64-unknown-linux-gnu -p tdm-dash --no-default-features --features kms   # binario per il Pi
make -C os release                                      # immagine per la scheda SD (Buildroot, circa un'ora)
```

Nella CI (GitHub Actions) girano:
- test Rust, firmware per RP2040 e RP2350, binario per il Pi
- ERC dello schema e controllo incrociato con i pin del firmware
- cablaggi, parti stampabili e controlli della documentazione
- immagine completa, sui tag e una volta alla settimana

La toolchain Rust (1.97.1, la stessa di Buildroot 2026.08) e i target vengono installati automaticamente da `rust-toolchain.toml`.

## Roadmap

| Milestone | Contenuto | Stato |
|---|---|---|
| M0 | Scheletro del repo, ricerca, manuale (cap. 00–03) | Fatto |
| M1 | Proposta di design interattiva a 800×480 | Fatto |
| M2 | Workspace Rust, simulatore desktop | Fatto |
| M3 / M3b | Firmware del Pico · schemi elettrici e dei collegamenti | Fatto, da provare sull'hardware |
| M4 | Sistema operativo Buildroot | Fatto, da provare sull'hardware |
| M5 | Contenitore, staffa, millefori, manuale di costruzione (cap. 06–08) | Progetto fatto; costruzione a cura del proprietario |
| M6 | Calibrazione (`tdmctl calibrate`), manuale (cap. 10, 12, 13) | Strumenti fatti; rilievi, montaggio e 500 km di prova da fare |

Prossimi passi sulla moto:
1. [rilievi](docs/research/bike-measurements.md)
2. prova del display al banco con Raspberry Pi OS: tempi DPI e benchmark
3. costruzione (capitoli 06–07)
4. prova in parallelo (capitolo 08)
5. calibrazione (capitolo 10)

## Licenze

Codice MIT o Apache-2.0, immagine del sistema GPLv3 (per via di Slint), hardware CERN-OHL-P-2.0, documentazione CC-BY-SA-4.0. Dettagli in [LICENSING.md](LICENSING.md).

Yamaha e TDM sono marchi dei rispettivi proprietari. Questo progetto non è affiliato né approvato da Yamaha Motor Co., Ltd.

> **Avvertenza:** progetto sperimentale e non certificato. Leggi il capitolo [Introduzione e sicurezza](docs/manual/00-introduction-and-safety.md) prima di montarlo su una moto.
