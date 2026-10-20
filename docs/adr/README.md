# Architecture Decision Records

Ogni file registra una decisione di progetto: il contesto, la scelta e le conseguenze. Una decisione superata non si cancella: si scrive un nuovo ADR che la sostituisce e si aggiorna lo stato del vecchio.

| # | Decisione | Stato |
|---|---|---|
| [0001](0001-pi-piu-microcontrollore.md) | Pi Zero 2 W per la grafica, RP2040 per l'I/O | Accettata |
| [0002](0002-protocollo-usb.md) | USB CDC con postcard, COBS e CRC-16 con tag di direzione | Accettata |
| [0003](0003-piattaforma-slint-drm.md) | Slint con piattaforma DRM propria e renderer software | Accettata, da confermare col benchmark |
| [0004](0004-contachilometri-su-fram.md) | Contachilometri sul microcontrollore, in FRAM | Accettata |
| [0005](0005-rootfs-sola-lettura.md) | Rootfs EROFS in sola lettura, nessuno spegnimento ordinato | Accettata |
| [0006](0006-retroilluminazione-fail-dark.md) | Retroilluminazione comandata dal microcontrollore, "fail-dark" | Accettata |
| [0007](0007-licenze.md) | Licenze del codice, dell'immagine, dell'hardware e dei documenti | Accettata |
| [0008](0008-profili-toml.md) | Profili moto in TOML a strati | Accettata |
| [0009](0009-display-7-pollici.md) | Display Newhaven 7" 800×480 da 1000 nit in DPI | Accettata |
