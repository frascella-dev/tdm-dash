# Manuale di tdm-dash

Manuale di progettazione, costruzione, installazione e uso del quadro digitale **tdm-dash** per la Yamaha TDM 850 prima serie (3VD, 1991–1995).

Il manuale cresce insieme al progetto: ogni milestone aggiunge o aggiorna i capitoli che la riguardano. I capitoli senza link non sono ancora scritti.

| # | Capitolo | Stato |
|---|---|---|
| 00 | [Introduzione e sicurezza](00-introduzione-e-sicurezza.md) | Prima stesura |
| 01 | [L'impianto originale della TDM 850](01-impianto-originale.md) | Prima stesura, da confermare con i rilievi |
| 02 | [Architettura del sistema](02-architettura.md) | Prima stesura |
| 03 | [Distinta componenti](03-distinta-componenti.md) | Prima stesura, prezzi ottobre 2026 |
| 04 | [Schemi elettrici](04-schemi-elettrici.md) | Prima stesura, da rivedere dopo i rilievi |
| 05 | [Schemi dei collegamenti](05-schemi-collegamenti.md) | Prima stesura, colori della moto da rilevare |
| 06 | [Costruzione dell'elettronica](06-costruzione.md) | Prima stesura, da provare al banco |
| 07 | [Contenitore e staffa](07-contenitore-e-staffa.md) | Prima stesura, misure da verificare sui pezzi |
| 08 | [Installazione sulla moto](08-installazione.md) | Prima stesura, da completare con i rilievi |
| 09 | [Software: firmware, immagine e profilo](09-software.md) | Prima stesura; immagine da provare sull'hardware |
| 10 | Calibrazione | Milestone M6 |
| 11 | [Uso](11-uso.md) | Prima stesura, schermate dal software vero |
| 12 | Diagnostica e risoluzione dei problemi | Milestone M6 |
| 13 | Manutenzione | Milestone M6 |
| A | [Appendici](A-appendici.md) | Codice colori, glossario, scheda rilievi |

## Convenzioni

- **Diagrammi:** sono scritti in [Mermaid](https://mermaid.js.org/) dentro i file Markdown, così GitHub li mostra direttamente e restano modificabili come testo. Gli schemi elettrici veri vengono da KiCad (`hardware/kicad/`), i cablaggi da WireViz (`hardware/harness/`); le immagini SVG generate stanno in `hardware/exports/`.
- **Stato dei dati:** sull'impianto originale si usano le marcature della [ricerca](../ricerca/impianto-originale.md): **[VERIFICATO]**, **[DA VERIFICARE]**, **[IPOTESI]**.
- **Avvisi:**

> **Attenzione:** rischio di danni alla moto o all'elettronica.

> **Pericolo:** rischio per la persona.

- **Unità:** sistema metrico; le tensioni sono riferite al negativo della batteria, salvo indicazione diversa.
