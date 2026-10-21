# Hardware

| Cartella | Contenuto | Come si rigenera |
|---|---|---|
| [`kicad/`](kicad/) | Schema elettrico KiCad 9 del modulo sensori, 8 fogli. **Generato** da [`generate.py`](kicad/generate.py): non modificare i file `.kicad_sch` a mano | `python3 hardware/kicad/generate.py` |
| [`harness/`](harness/) | Cablaggi in WireViz (YAML) | `wireviz -f st -o hardware/exports/harness hardware/harness/<file>.yml` |
| [`exports/kicad/`](exports/kicad/) | Schema in PDF e un SVG per foglio | `hardware/kicad/export.sh` (ERC e controllo dei pin compresi) |
| [`exports/harness/`](exports/harness/) | Disegni dei cablaggi (SVG) e distinte (TSV) | vedi `harness/` |
| [`perfboard/`](perfboard/) | Disposizione per zone del millefori 90 × 70 mm (`layout.svg`) | `python3 hardware/perfboard/layout.py` |
| [`enclosure/`](enclosure/) | Contenitore parametrico in OpenSCAD: cornice, guscio, palpebra | `hardware/cad-export.sh` |
| [`bracket/`](bracket/) | Staffa del sensore ruota in OpenSCAD | `hardware/cad-export.sh` |
| [`exports/cad/`](exports/cad/) | STL da stampare, DXF/SVG della staffa, anteprime PNG | `hardware/cad-export.sh` (anche con Docker, vedi sotto) |

Dove sono spiegati nel manuale:
- ogni foglio dello schema e il calcolo dei componenti: [capitolo 04](../docs/manuale/04-schemi-elettrici.md)
- cablaggi e piedinature: [capitolo 05](../docs/manuale/05-schemi-collegamenti.md)
- costruzione del millefori: [capitolo 06](../docs/manuale/06-costruzione.md)
- contenitore e staffa: [capitolo 07](../docs/manuale/07-contenitore-e-staffa.md)

Le parti meccaniche si esportano con OpenSCAD 2021.01, anche via Docker:

```sh
docker run --rm -u 0 -v "$PWD":/w -w /w openscad/openscad:2021.01 hardware/cad-export.sh
```

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
