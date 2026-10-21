# 0007 · Licenze

**Stato:** accettata · ottobre 2026

## Contesto

Il progetto è pubblico ed estensibile. Slint è disponibile in GPLv3, con licenza royalty-free (che **esclude** i sistemi embedded, citando come esempio proprio un quadro auto) o commerciale.

## Decisione

| Parte | Licenza |
|---|---|
| Codice del progetto (crate, firmware, strumenti) | **MIT OR Apache-2.0** |
| Immagine del sistema distribuita (contiene Slint) | **GPL-3.0** nel suo insieme |
| Hardware (KiCad, cablaggi, CAD) | **CERN-OHL-P-2.0** |
| Documentazione e manuale | **CC-BY-SA-4.0** |
| Font inclusi | **OFL-1.1** |

## Conseguenze

- Ogni release dell'immagine include l'output di `make legal-info` di Buildroot (sorgenti e licenze).
- Chi vuole un derivato commerciale chiuso deve comprare la licenza commerciale di Slint o sostituire lo strato di interfaccia.
- Nessun logo o marchio Yamaha nel repo, nello splash o nel design.
