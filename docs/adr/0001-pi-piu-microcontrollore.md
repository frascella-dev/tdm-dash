# 0001 · Pi Zero 2 W per la grafica, RP2040 per l'I/O

**Stato:** accettata · ottobre 2026

## Contesto

Il quadro deve contare impulsi di ruota (fino a ≈ 170 Hz) e bobina (≈ 140 Hz), leggere spie e tensioni, conservare i km e disegnare un'interfaccia fluida a 800×480. Deve costare poco e sopportare cadute di tensione.

I limiti del solo Raspberry Pi:
- non ha ADC
- non ha timer di cattura hardware
- in user space ha jitter da decine di µs a millisecondi
- non conta nulla durante i 3–5 s di avvio
- con il display DPI la UART non è disponibile

## Decisione

- **Raspberry Pi Zero 2 W** (−20…+70 °C, Buildroot ufficiale, GPU) solo per la grafica.
- **Raspberry Pi Pico (RP2040)**, o Pico 2, per tutto l'I/O in tempo reale, i km e la retroilluminazione.

Alternativa scartata: Allwinner T113-S3. È più economico e si avvia più in fretta, ma non ha GPU e ha una community più piccola.

## Conseguenze

- Il Pico è attivo in millisecondi: nessun impulso perso durante l'avvio di Linux.
- Il Pi resta senza GPIO usati oltre al DPI.
- Due firmware da mantenere, ma con codice condiviso (`tdm-core`, `tdm-proto`).
- Il Pi Zero 2 W è spesso esaurito: va ordinato per primo.
