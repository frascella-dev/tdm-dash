# 0010 · Layout predefinito: Classic

**Stato:** accettata · ottobre 2026

## Contesto

La proposta di design (milestone M1) ha tre layout di guida: Classic, Sport e Touring. Il proprietario ha chiesto di procedere con le milestone senza fermarsi per le scelte; quella definitiva resta sua.

## Decisione

- **Classic** è il predefinito nel profilo `yamaha-tdm850-3vd.toml`: due quadranti, la lettura più vicina al quadro originale della TDM.
- Sport e Touring restano disponibili, da Impostazioni o dal profilo (`[display] layout`).
- La scelta fatta da Impostazioni viene salvata nella FRAM del Pico e ha la precedenza sul profilo.

## Conseguenze

- Cambiare il predefinito richiede solo una riga nel profilo, nessuna ricompilazione.
- Tutti e tre i layout sono coperti dalle schermate di riferimento (`design/screenshots/`) e dal test di rendering.
