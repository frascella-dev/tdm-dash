# 0008 · Profili moto in TOML a strati

**Stato:** accettata · ottobre 2026

## Contesto

Il progetto deve poter servire altre moto senza ricompilare: circonferenza e impulsi per giro della ruota, impulsi per giro motore, zona rossa, curva del sensore della temperatura, soglie della batteria, ingressi collegati.

## Decisione

**Profilo in TOML**, con `schema = 1` e `deny_unknown_fields`. Il primo file trovato vince, in quest'ordine:
1. `/boot/tdm-dash.toml`, modificabile da PC
2. `/etc/tdm-dash/default.toml`, nell'immagine
3. profilo compilato nell'app

**In caso di errore** si usa il profilo integrato e si mostra l'avviso `ConfigError`.

**Regolazioni utente** (tema, layout, luminosità, correzione fine della velocità): salvate nella FRAM tramite il Pico, applicate sopra il profilo.

**Niente ereditarietà tra profili:** ogni profilo è completo e autosufficiente.

## Conseguenze

- Per una moto nuova basta copiare un profilo esistente e modificarlo.
- Tutti i profili nel repo sono verificati dai test (parsing, curva NTC monotona).
