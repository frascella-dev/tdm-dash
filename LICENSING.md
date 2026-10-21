# Licenze

tdm-dash comprende software, un'immagine di sistema, hardware e documentazione, con licenze diverse (vedi [ADR 0007](docs/adr/0007-licenze.md)).

| Parte | Percorso | Licenza |
|---|---|---|
| Codice sorgente (crate Rust, firmware, strumenti) | `crates/`, `firmware/`, `tools/`, `os/` | [MIT](LICENSE-MIT) **oppure** [Apache-2.0](LICENSE-APACHE), a scelta |
| Immagine del sistema distribuita (`sdcard.img`) | release su GitHub | **GPL-3.0-only** nel suo insieme, perché include [Slint](https://github.com/slint-ui/slint) in licenza GPLv3. Ogni release include l'output di `make legal-info` con sorgenti e licenze di tutti i componenti |
| Hardware (schemi KiCad, cablaggi, CAD) | `hardware/` | [CERN-OHL-P-2.0](https://ohwr.org/cern_ohl_p_v2.txt) |
| Documentazione, manuale, design | `docs/`, `design/` | [CC-BY-SA-4.0](https://creativecommons.org/licenses/by-sa/4.0/) |
| Font inclusi | `crates/tdm-dash/fonts/`, `design/` | [SIL OFL-1.1](https://openfontlicense.org) |

## Contributi

Salvo indicazione contraria, ogni contributo inviato per l'inclusione in questo progetto è concesso con le stesse licenze della parte che modifica, senza termini aggiuntivi.

## Slint e uso commerciale

La licenza royalty-free di Slint **esclude** i sistemi embedded. Chi vuole distribuire un derivato chiuso deve:
- acquistare una licenza commerciale di Slint, **oppure**
- sostituire lo strato di interfaccia (`crates/tdm-dash`).

Il resto del codice (protocollo, misure, logica, firmware) resta MIT/Apache.

## Esclusione di garanzia

Il progetto è fornito "così com'è", senza garanzie di alcun tipo. Non è un dispositivo certificato. Vedi le licenze e il capitolo [Introduzione e sicurezza](docs/manuale/00-introduzione-e-sicurezza.md).
