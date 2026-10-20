# 0003 · Slint con piattaforma DRM propria e renderer software

**Stato:** accettata, da confermare con il benchmark su hardware (M4.0) · ottobre 2026

## Contesto

L'interfaccia va scritta in Rust, deve essere leggera e bella: lancette anti-aliasing e font curati. Il backend `linuxkms` di Slint si porta dietro libinput, quindi udev, e xkbcommon, in contrasto con un sistema senza udev. La GPU del Pi Zero 2 W (VideoCore IV) offre solo GLES 2.0 tramite Mesa vc4.

## Decisione

- **Slint 1.18**, con il **renderer software**.
- Una **piattaforma Slint propria**: `MinimalSoftwareWindow` più due buffer DRM "dumb" con page-flip sincronizzato al vblank, usando il crate `drm`.
- Prima di scriverla si verifica se `backend-linuxkms` si compila senza libinput; in quel caso si usa quello.

Il renderer software non supporta le trasformazioni, quindi:
- le lancette sono `Path` con i punti calcolati in Rust (sin/cos)
- i quadranti sono immagini generate all'avvio con tiny-skia a partire dal profilo
- nessuna rotazione né sfocatura nel design

Piano B, solo se il benchmark fallisce (p95 > 33 ms): renderer FemtoVG su Mesa vc4 GLES2.

## Conseguenze

- Il binario dipende solo da glibc e la cross-compilazione è banale.
- L'app sa con esattezza quando il primo frame è sullo schermo, e solo allora accende la retroilluminazione.
- Lo stesso codice gira sul PC (winit + renderer software) con lo stesso aspetto.
- L'immagine distribuita è GPLv3 per via di Slint (vedi ADR 0007).
