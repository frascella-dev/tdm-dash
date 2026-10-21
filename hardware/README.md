# Hardware

| Cartella | Contenuto | Come si rigenera |
|---|---|---|
| [`kicad/`](kicad/) | Schema elettrico KiCad 9 del modulo sensori, 8 fogli. **Generato** da [`generate.py`](kicad/generate.py): non modificare i file `.kicad_sch` a mano | `python3 hardware/kicad/generate.py` |
| [`harness/`](harness/) | Cablaggi in WireViz (YAML) | `wireviz -f st -o hardware/exports/harness hardware/harness/<file>.yml` |
| [`exports/kicad/`](exports/kicad/) | Schema in PDF e un SVG per foglio | `hardware/kicad/export.sh` (ERC e controllo dei pin compresi) |
| [`exports/harness/`](exports/harness/) | Disegni dei cablaggi (SVG) e distinte (TSV) | vedi `harness/` |

La spiegazione di ogni foglio e il calcolo dei componenti sono nel [capitolo 04 del manuale](../docs/manuale/04-schemi-elettrici.md); i cablaggi e le piedinature nel [capitolo 05](../docs/manuale/05-schemi-collegamenti.md).

## Modificare lo schema

1. Modifica la descrizione del circuito in `kicad/generate.py` (funzione `build()`): ogni componente ha il suo simbolo KiCad e la rete su ciascun pin.
2. Rigenera e controlla con KiCad 9, anche via Docker:

   ```sh
   docker run --rm -u 0 -v "$PWD":/w -w /w kicad/kicad:9.0.9 \
       sh -c 'python3 hardware/kicad/generate.py && hardware/kicad/export.sh'
   ```

   `export.sh` esegue l'ERC e [`tools/check_pinmap.py`](../tools/check_pinmap.py), che confronta i GPIO del Pico con `firmware/src/main.rs`, poi esporta PDF e SVG.
3. Fai il commit sia di `generate.py` sia dei file generati: la CI (workflow `Hardware`) controlla che coincidano.

Aprire il progetto in KiCad per guardarlo va benissimo. Le modifiche fatte nell'editor vanno però riportate in `generate.py`, altrimenti la prossima generazione le cancella.

## Licenza

Hardware sotto CERN-OHL-P-2.0 (vedi [LICENSING.md](../LICENSING.md)).
