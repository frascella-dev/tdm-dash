# Project images

| File | What it is | How to regenerate it |
|---|---|---|
| `banner.svg` | Banner at the top of the README: title, technologies and the bike on a dial | `python3 tools/make_banner.py` (needs `fonttools`) |
| `tdm-850.svg` | Side view of the bike, line art in a single SVG path | `python3 tools/stl_outline.py <model>.stl docs/assets/tdm-850.svg --drop …` (needs `numpy`) |

`tdm-850.svg` comes from a relief STL model (`yamaha-tdm-850-v1.stl`) supplied by the project owner. The script takes the base faces of the relief, follows their outline and simplifies the curves. The lettering and logos of the original model (brand on the tank and on the clutch cover, model logo) are removed with the `--drop` options: the repository contains no registered trademarks.

```sh
python3 tools/stl_outline.py yamaha-tdm-850-v1.stl docs/assets/tdm-850.svg \
    --drop 120,56,156,74 --drop 148,59,194,89 --drop 99,139,115,144 --title "TDM 850, side view"
python3 tools/make_banner.py
```

The banner texts are converted to outlines with the dashboard font (Saira SemiCondensed, OFL license, in `crates/tdm-dash/fonts/`), so the file does not depend on installed fonts. The colors are those of the day theme (see `design/README.md`).
