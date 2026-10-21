# 0006 · Retroilluminazione comandata dal microcontrollore, "fail-dark"

**Stato:** accettata · ottobre 2026

## Contesto

Un quadro bloccato che mostra 80 km/h quando si è fermi, o viceversa, è peggio di un quadro spento. Il display va acceso solo quando mostra dati validi.

## Decisione

- Il **Pico pilota il PWM della retroilluminazione** tramite il boost TPS61169.
- Il Pi invia `SetBacklight` solo dopo il primo frame sullo schermo, poi un `Heartbeat` ogni 250 ms.
- Se l'heartbeat manca per più di **2 s**, il Pico spegne la retroilluminazione; il watchdog del Pi lo riavvia.
- **LED passivi di backup** sulle linee a 12 V (frecce, abbaglianti, folle) garantiscono le spie obbligatorie anche con l'elettronica spenta.

## Conseguenze

- Nessun frame "vecchio" visibile all'avvio o in caso di blocco.
- Il Pi non usa altri GPIO oltre al DPI.
- Il Pico misura il tempo dall'accensione al primo `SetBacklight`: è il tempo di avvio percepito.
