# 0012 · English repository, bilingual dashboard

**Status:** accepted · October 2026. Replaces the language rule of the first version of `CLAUDE.md` (Italian documentation, English code).

## Context

The project started with the documentation in Italian, the owner's language, and the code in English. A public repository meant to be extended to other motorcycles reaches more people in English. The owner rides with the dashboard, though, and prefers its texts in Italian.

## Decision

- **Everything in the repository is in English:** documentation, code, comments, commit messages, CLI output, schematic notes, harness labels. File and folder names are English too (`docs/manual`, `docs/research`).
- **The status markers** of the facts about the original wiring are `[VERIFIED]`, `[TO VERIFY]` and `[HYPOTHESIS]`.
- **The dashboard texts are bilingual.** `crates/tdm-logic/src/i18n.rs` holds every text the rider can see:
  - each text is a `Msg` variant, translated by two functions, `en()` and `it()`, each matching on every variant, so a new text does not compile until it exists in both languages
  - texts with values (units, diagnostics rows) are functions in the same file
  - the fixed labels of the Slint screens are the `Labels` global, filled from the same table at start-up
  - numbers follow the language: `48,213` and `212.4` in English, `48 213` and `212,4` in Italian
- **The language is chosen in the profile**, `[display] language = "en"` or `"it"`, English when absent. The TDM 850 profile sets `"it"`. `tdm-dash --language` overrides it.
- **`tdmctl` is English only:** it is a tool for whoever builds and calibrates the dashboard.

## Consequences

- Adding a language means one more function in `i18n.rs` and one more `Language` variant; the compiler lists every text to translate.
- The reference screens are generated in English; a few Italian copies (`design/screenshots/it-*.png`) show that the Italian texts fit their boxes.
- The hidden `Text` with the accented characters stays in `dash.slint`: the software renderer only draws the glyphs it embeds.
