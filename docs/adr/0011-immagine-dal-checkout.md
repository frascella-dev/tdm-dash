# 0011 · L'immagine compila il quadro dal checkout

**Stato:** accettata · ottobre 2026

## Contesto

Il pacchetto Buildroot `tdm-dash` deve compilare il programma del quadro. Ci sono due strade:

- **Scaricarlo da un tag GitHub**, come i pacchetti normali. Serve un tag e il suo hash prima di ogni build, e l'immagine può contenere un programma di un commit diverso da quello dei file `os/` che la costruiscono.
- **Usare la cartella del repository** che contiene l'albero esterno (metodo `local` di Buildroot).

Il sistema operativo, il programma e il profilo stanno nello stesso repository e cambiano insieme: un cambio del protocollo, per esempio, tocca firmware, programma e immagine nello stesso commit.

## Decisione

- `TDM_DASH_SITE_METHOD = local`, con `TDM_DASH_SITE` uguale alla radice del repository. La versione è `git describe --always --dirty`.
- La sincronizzazione esclude `os/` (Buildroot e le sue uscite), `firmware/`, `hardware/`, `docs/` e `design/`.
- Il metodo `local` salta il passo di download e quindi il `cargo vendor` che Buildroot esegue per i pacchetti cargo. Lo stesso comando gira in un hook prima della configurazione, poi la compilazione avviene `--offline --locked` come per gli altri pacchetti cargo.
- Il profilo di compilazione del workspace (LTO fat, una sola unità di codice, `panic = abort`) viene reimpostato con variabili d'ambiente, perché l'infrastruttura cargo di Buildroot lo sovrascrive.

## Conseguenze

- Ogni immagine corrisponde esattamente a un commit, anche durante lo sviluppo e nella CI delle pull request.
- `legal-info` non archivia il sorgente di `tdm-dash` (Buildroot non lo fa per i pacchetti `local`). Il sorgente da pubblicare per la GPL-3.0 è il repository al tag della release: le note di rilascio lo indicano.
- Il `cargo vendor` richiede la rete al momento della compilazione. Le dipendenze restano in cache in `os/dl/br-cargo-home`, che la CI salva tra una build e l'altra.
