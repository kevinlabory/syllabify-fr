# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Rust port of **LireCouleur 6** (Marie-Pierre & Luc Brungard, GPL v3), a French syllabifier designed for reading instruction — especially for dyslexic children. Unlike typographic hyphenators (Hypher, hyphen-fr…) which minimize break points, this library segments every word into **all** its syllables.

The port must stay **100% conformant** to LC6 on the 4830-word regression oracle (`tests/oracle.json`). Any change to parser, decoder, or data must keep `cargo test` green.

## Commands

```bash
cargo build --release            # builds lib + CLI binary `syllabify`
cargo test                       # 19 unit + 1 regression (4830 words) — must stay 100%
cargo test --test regression     # regression only; writes /tmp/syllabify_mismatches.txt on failure
cargo test <name>                # single unit test by name
./target/release/syllabify chocolat           # cho-co-lat
./target/release/syllabify --text "…"         # syllabifies full text (with homograph disambiguation)
./target/release/syllabify --json mot         # JSON array output
echo "famille" | ./target/release/syllabify -  # stdin mode, one word per line
```

Regenerating embedded data from a new LC6 release (requires Node.js + Python 3, and access to LC6's `module.js`):

```bash
node build/extract_v6_data.js       # module.js → build/data/*.json
python3 build/generate_data.py      # JSON → src/data.rs (hand-edit output is a no-go)
LC6_PATH=/path/to/lirecouleur/js/lirecouleur/module.js \
  node build/generate_oracle.js build/data/corpus.txt   # regenerates tests/oracle.json
```

## Architecture

Five-stage pipeline, fidelity to LC6 is the design constraint at every stage:

1. **`cleaner.rs`** — lowercase, apostrophe → `@`, punctuation → space. Hyphens and underscores are **kept in-word** (a v6 change; see NOTES-v6.md Piège 4) so `grand-père` is one token.
2. **`parser.rs`** — finite-state automaton. For each letter (1-indexed à la Python), selects a rule via regex lookahead (`plus`) / lookbehind (`minus`), or one of ~10 special rules (`regle_ient`, `regle_mots_ent`, `regle_verbes_ier`, …) in `rules.rs`. Produces `Phoneme { code, step }`. Unknown characters become empty phonemes (`code=""`, `step=1`) that must be **skipped** downstream (NOTES-v6.md Piège 2).
3. **`decoder.rs` — post-processing** — `post_process_e` (eu open/closed), `post_process_o` (o open/closed), `post_traitement_yod` (v6: `i + V` → replace `i` with `j`, **not** fuse — NOTES-v6.md §1). `post_traitement_w` is intentionally a no-op (removed in v6).
4. **`homographs.rs`** — `lookup(word, previous_word)` short-circuits the automaton for 16 non-homophonic homographs (`couvent`, `est`, `fils`, `violent`, `excellent`, …). Called from `decoder::extract_syllables` when walking a full text.
5. **`decoder.rs` — `assemble_syllables`** — groups V/C/S into syllables, handles complex onsets (`bl`, `tr`, `pl`, …), and in `AssembleMode::Std` (the v6 default) splits **only true consonants** (`PhonClass::Consonant`) on double letters. **Never split semi-vowels** like `j(ll)` — that was Piège 1 in the v5→v6 port.

### Data (`src/data.rs`, ~66KB, generated)

- `AUTOMATON`: 43 letters × ~480 rules total
- 15 word lists (~1200 entries): `MOTS_ENT`, `VERBES_IER`, `VERBES_MER`, `EXCEPTIONS_FINAL_ER`, `POSSIBLES_AVOIR`, etc.
- `HOMOGRAPHES`: 16 entries with ~30 context variants
- `DETERMINANT` / `PRONOM`: used by homograph context matching

**Do not hand-edit `src/data.rs`** — regenerate via `build/generate_data.py`. Piège 3 (NOTES-v6.md): for the `*` default rule, `generate_data.py` must always emit `default_rule` (ignore the JSON `+` condition) — LC6's `module.js` ignores it too.

### Public API (`lib.rs`)

- `syllables(word)` — defaults: `Std` mode, `Written` syllables, `novice_reader=false`
- `syllables_with(word, novice_reader, AssembleMode, SyllableMode)` — full control
- `phonemes(word)` — `(code, letters)` pairs
- `syllabify_text(text) -> Vec<TextChunk>` — `Word(Vec<String>)` or `Raw(String)`; runs homograph disambiguation against the previous word

### Modes

- `AssembleMode::Std` (default, pedagogical — `homme → hom-me`) vs `Lc` (phonological — `ho-mme`). `Lc` is no longer 100%-aligned with LC6; treat it as a legacy option.
- `SyllableMode::Written` vs `Oral` (final `q_caduc` fused with preceding syllable: `école → é-cole`).

## What is explicitly **not** ported

See NOTES-v6.md §"Points non portés". `regle_en_final` and `dernierTraitement` remain out of scope — don't add these without confirming. `liaisonAmont`/`liaisonAval` are now ported in `src/liaisons.rs` as pure predicates (no syllabic side-effect).

## Technical debt to be aware of

- None tracked at the moment. The previously-listed `Mutex<HashMap>` regex cache
  in `parser.rs` is now an `OnceLock<HashMap>`; `AssembleMode::Lc` carries
  `#[deprecated]` and its drift from LC6 v6 is documented inline.

Always consult **NOTES-v6.md** before modifying parser/decoder behavior — it documents four concrete pitfalls that bit during the port and explains *why* specific lines look the way they do.
