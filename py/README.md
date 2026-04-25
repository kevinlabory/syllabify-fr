# syllabify-fr (Python)

Python bindings for [syllabify-fr](https://github.com/kevinlabory/syllabify-fr) — a Rust port of
[LireCouleur 6](https://lirecouleur.forge.apps.education.fr/), a French syllabification library
for reading instruction (dyslexia support).

Built with [PyO3](https://pyo3.rs/) and [maturin](https://www.maturin.rs/).

## Installation

```bash
# From a wheel (after maturin build)
pip install syllabify_fr-*.whl

# Development install (requires maturin)
maturin develop -m py/Cargo.toml --features extension-module
```

## API

```python
import syllabify_fr

# Syllabify a word → list of syllables
syllabify_fr.syllables("chocolat")
# ['cho', 'co', 'lat']

# Syllabify full text → list of chunks (word or raw)
syllabify_fr.syllabify_text("le chat dort")
# [{'kind': 'word', 'syllables': ['le']},
#  {'kind': 'raw',  'text': ' '},
#  {'kind': 'word', 'syllables': ['chat']}, ...]

# Phonemes → list of (code, letters) tuples
syllabify_fr.phonemes("chat")
# [('s^', 'ch'), ('a', 'a'), ('#', 't')]

# HTML rendering
syllabify_fr.render_word_html("chocolat")
# '<span class="word"><span class="syl syl-a">cho</span>...</span>'

syllabify_fr.render_html("les hôtels")
# Full HTML with syllable spans and liaison markers
```

## Building from source

Requires [maturin](https://www.maturin.rs/) ≥ 1.0 and Rust ≥ 1.70.

```bash
# Build wheel for current Python
maturin build -m py/Cargo.toml --features extension-module --release

# Publish to PyPI
maturin publish -m py/Cargo.toml --features extension-module
```

## Running tests

```bash
# Install the package first
maturin develop -m py/Cargo.toml --features extension-module

# Run pytest
pytest py/tests/
```

## License

GPL-3.0-or-later — derived from [LireCouleur 6](https://lirecouleur.forge.apps.education.fr/)
by Marie-Pierre & Luc Brungard.

A proprietary application that distributes this package (including via PyPI) becomes GPL v3
at distribution. Contact the original authors for a relicensing agreement if needed.
