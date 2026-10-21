# 0005 · Rootfs EROFS in sola lettura, nessuno spegnimento ordinato

**Stato:** accettata · ottobre 2026

## Contesto

Il quadro perde l'alimentazione ogni volta che si gira la chiave. Uno spegnimento ordinato richiederebbe supercondensatori e circuiti dedicati.

## Decisione

- **Rootfs EROFS compresso, montato in sola lettura.**
- `/boot` (FAT) montato in sola lettura: contiene il profilo modificabile da PC.
- `/tmp` e `/run` in tmpfs.
- Nessuno spegnimento ordinato del Pi; solo il Pico ha una piccola riserva di energia (ADR 0004).

## Conseguenze

- Si può togliere corrente in qualsiasi momento senza danni.
- Impostazioni utente e km vivono nella FRAM, non sulla SD.
- L'immagine di sviluppo (rete, ssh, scrittura) è un defconfig separato.
