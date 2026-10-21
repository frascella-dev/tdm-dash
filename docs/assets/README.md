# Immagini del progetto

| File | Cos'è | Come si rigenera |
|---|---|---|
| `banner.svg` | Banner in cima al README: titolo, tecnologie e la moto su un quadrante | `python3 tools/make_banner.py` (serve `fonttools`) |
| `tdm-850.svg` | Profilo della moto, disegno al tratto in un solo percorso SVG | `python3 tools/stl_outline.py <modello>.stl docs/assets/tdm-850.svg --drop …` (serve `numpy`) |

`tdm-850.svg` è ricavato da un modello STL in rilievo (`yamaha-tdm-850-v1.stl`) fornito dal proprietario del progetto. Lo script prende le facce di base del rilievo, ne segue il contorno e semplifica le curve. Le scritte e i loghi del modello originale (marchio sul serbatoio e sul coperchio frizione, logo del modello) sono tolti con le opzioni `--drop`: il repository non contiene marchi registrati.

```sh
python3 tools/stl_outline.py yamaha-tdm-850-v1.stl docs/assets/tdm-850.svg \
    --drop 120,56,156,74 --drop 148,59,194,89 --drop 99,139,115,144 --title "TDM 850, profilo"
python3 tools/make_banner.py
```

I testi del banner sono convertiti in tracciati con il font del quadro (Saira SemiCondensed, licenza OFL, in `crates/tdm-dash/fonts/`), quindi il file non dipende da font installati. I colori sono quelli del tema "giorno" (vedi `design/README.md`).
