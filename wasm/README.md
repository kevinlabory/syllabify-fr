# syllabify-fr-wasm

WebAssembly bindings for [`syllabify-fr`](../README.md) — French syllabification for reading instruction, port of [LireCouleur 6](https://lirecouleur.forge.apps.education.fr/).

## Licence

**GPL v3 or later.** Derived from LireCouleur 6 (Marie-Pierre & Luc Brungard, GPL v3). Any application distributing this module must comply with GPL v3 terms.

## Build

```bash
wasm-pack build wasm/ --target web --release
# Outputs wasm/pkg/ (ESM + .d.ts + wasm binary)
```

Other targets: `--target bundler` (Vite/Webpack), `--target nodejs`.

## Usage (ESM / browser)

```js
import init, { syllables, syllabifyText, phonemes } from './pkg/syllabify_fr_wasm.js';

await init();

syllables('chocolat');
// → ['cho', 'co', 'lat']

syllabifyText('le couvent accueille les moines');
// → [
//     { kind: 'word', syllables: ['le'] },
//     { kind: 'raw', text: ' ' },
//     { kind: 'word', syllables: ['cou', 'vent'] },  // nom : -ent prononcé
//     { kind: 'raw', text: ' ' },
//     { kind: 'word', syllables: ['ac', 'cue', 'ille'] },
//     ...
//   ]

phonemes('hier');
// → [['j', 'i'], ['e^_comp', 'e'], ['r', 'r']]
```

`syllabifyText` disambiguates non-homophonic homographs (`couvent`, `est`, `fils`, `violent`, …) from the preceding word. `syllables` works on isolated words only.

## Size

~75 KB gzipped (`.wasm` binary), plus ~3 KB for the JS glue.

## Tests

```bash
wasm-pack test --node wasm/
```

Core-library regression (4830 words, must stay at 100%):

```bash
cargo test --no-default-features --features regex-lite
```
