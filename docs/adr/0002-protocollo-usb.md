# 0002 · USB CDC con postcard, COBS e CRC-16 con tag di direzione

**Stato:** accettata · ottobre 2026

## Contesto

Il display DPI occupa i GPIO 14/15 della UART. Il Pico e il Pi devono scambiarsi telemetria a 50 Hz e comandi rari, in modo robusto contro disturbi, riconnessioni ed eco di una tty non ancora in modalità raw.

## Decisione

**Trasporto:** USB CDC-ACM. Il Pico è il dispositivo; il Pi è l'host, con `dwc2,dr_mode=host` forzato perché il pin ID non è collegato.

**Formato del frame:** `postcard(messaggio) ‖ CRC-16`, codificato COBS e terminato da `0x00`.

**CRC con tag di direzione:** il CRC copre anche un byte che non viene trasmesso: `0xA5` per i frame dal Pico al Pi, `0x5A` per quelli dal Pi al Pico. Un frame del Pico rimandato indietro dall'eco della tty non passa il controllo e non può essere scambiato per un comando, per esempio `ResetTrip`.

**Regole di dialogo:**
- **parla prima l'host:** il Pico tace finché non riceve un frame valido
- `PROTO_VERSION` nel messaggio `Hello`
- varianti e campi si aggiungono solo in coda

## Conseguenze

- Lo stesso crate `tdm-proto` (`no_std`) è usato da firmware, app e simulatore.
- Il Pi cerca il dispositivo per VID:PID in `/sys/class/tty/ttyACM*`, senza dare per scontato `ttyACM0`.
- Il cavo USB va saldato e protetto (doppino corto, ferrite).
