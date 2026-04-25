# Changelog

All notable changes to this project are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Versioning follows [Semantic Versioning](https://semver.org/).

---

## [0.4.0] — 2026-04-19

### Added
- `src/html.rs`: `render_word_html(word)` and `render_html(text)` — HTML output
  with `<span class="syl syl-a/b">` alternating syllable spans, word wrapping,
  and `<span class="liaison">` inter-word liaison markers.
- WASM: `renderHtml` / `renderWordHtml` exported via `wasm-bindgen`.
- HTML character escaping for `<`, `>`, `&`, `"`, `'`.

### Changed
- Core bumped `0.3.0 → 0.4.0`; WASM `0.1.0 → 0.2.0`.

---

## [0.3.0] — 2026-04-19

### Added
- `src/liaisons.rs`: `liaison_amont`, `liaison_aval`, `liaison_possible` — port of
  `liaisonAmont` / `liaisonAval` from LireCouleur 6 (`module.js` ll. 1041-1057).
- `LIAISONS_AVAL` word list (40 entries) embedded in `src/data.rs` via `generate_data.py`.
- `h` aspiré vs muet correctly handled: `hôtel` allows liaison, `héros`/`haricot` block.

### Changed
- Core bumped `0.2.0 → 0.3.0`.

---

## [0.2.0] — 2026-04-18

### Added
- `wasm/` crate (`syllabify-fr-wasm 0.1.0`): WebAssembly bindings via `wasm-bindgen`.
  - Exports `syllables(word)`, `syllabifyText(text)`, `phonemes(word)`.
  - Target: ESM (`wasm-pack build --target web`); gzip budget < 300 KB.
- Feature-gate `regex-full` (default) / `regex-lite` for WASM size.
- Workspace with two members: root + `wasm/`.
- `ffi/` crate (`syllabify-fr-ffi 0.1.0`): C FFI bindings (`cdylib` + `staticlib`).
  - Exports `syllabify_syllables`, `syllabify_text_json`, `syllabify_phonemes`,
    `syllabify_render_word_html`, `syllabify_render_html`, `syllabify_free`.
  - Pre-generated C header `ffi/include/syllabify_fr.h`.
- `py/` crate (`syllabify-fr-py 0.1.0`): Python bindings via PyO3 0.23.
  - Exports `syllables`, `syllabify_text`, `phonemes`, `render_html`, `render_word_html`.
  - Build with maturin; test suite: 11 pytest tests.

### Changed
- Core bumped `0.1.0 → 0.2.0`.

### Fixed
- Regex cache: replaced `OnceLock<Mutex<HashMap<String, Regex>>>` with
  `OnceLock<HashMap<&'static str, Regex>>` — all patterns from `AUTOMATON`
  compiled once at first parse, lock-free reads thereafter.
- `AssembleMode::Lc` marked `#[deprecated(since = "0.4.0")]`: non-conformant
  with LireCouleur 6 v6; prefer `AssembleMode::Std`.

---

## [0.1.0] — 2026-04-18

### Added
- Initial Rust port of LireCouleur 6 (`module.js`), 100% conformant on 4830-word oracle.
- Five-stage pipeline: `cleaner → parser → decoder post-processing → homographs → assembly`.
- `syllables(word)`, `syllables_with(word, …)`, `phonemes(word)`, `syllabify_text(text)`.
- 19 unit tests + 4830-word regression oracle.
- `AUTOMATON`: 43 letters × ~480 rules; 15 word lists; 16 homograph entries.
- CLI binary `syllabify` (requires `regex-full` feature).
- NOTES-v6.md documenting 10 v5→v6 algorithm changes and 4 porting pitfalls.
