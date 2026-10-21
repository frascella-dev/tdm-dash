# 0004 · Contachilometri sul microcontrollore, in FRAM

**Stato:** accettata · ottobre 2026

## Contesto

I km non devono mai tornare indietro né andare persi, anche con chiave girata di colpo, batteria scollegata o Pi bloccato. Le schede SD si rovinano con le scritture frequenti e le interruzioni di corrente.

## Decisione

- Il **Pico è la sorgente di verità**. Accumula odometro e due parziali in micrometri a ogni impulso di ruota valido, anche quando il Pi non è ancora avviato.
- **Memoria FRAM MB85RC256V** su I²C (10¹² scritture). Ogni record è `magic | seq | dati | crc32`, scritto alternando due slot; all'avvio si carica lo slot valido con `seq` più alto.
- Scrittura **ogni 10 m**, alla caduta di tensione (sotto 7 V per 2 ms, con 470 µF di riserva) e all'azzeramento di un parziale.
- **FRAM vuota:** si parte da 0 e si accetta **una sola volta** `InitOdometer`, con il valore fotografato dal quadro originale.
- **FRAM corrotta:** mai sovrascritta; segnalazione di guasto.
- Il Pi **visualizza soltanto**.

## Conseguenze

- Al massimo 10 m persi in caso di guasto dell'alimentazione.
- Nessuna scrittura sulla SD durante l'uso.
- Una ricalibrazione della ruota cambia solo la distanza futura, non quella già percorsa.
