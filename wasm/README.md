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

## Publish to npm

```bash
# Build with @dyscolor scope (generates wasm/pkg/ with name @dyscolor/syllabify-fr-wasm)
wasm-pack build wasm/ --target bundler --scope dyscolor --release

# Publish (first time: npm login + the @dyscolor scope must exist on your npm account)
cd wasm/pkg
npm publish --access public
```

Publication is automated via GitHub Actions (`.github/workflows/publish-npm.yml`)
on every `v*` tag push. Requires the `NPM_TOKEN` secret to be set in the repository
(Settings → Secrets and variables → Actions).

## Usage (ESM / browser)

```js
import init, {
  syllables,
  syllabifyText,
  phonemes,
  renderHtml,
  renderWordHtml,
} from './pkg/syllabify_fr_wasm.js';

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

renderWordHtml('chocolat');
// → '<span class="word"><span class="syl syl-a">cho</span>…</span>'

renderHtml('les hôtels');
// → '<span class="word">…les…</span> <span class="liaison" data-with="z"></span><span class="word">…hôtels…</span>'
```

`syllabifyText` disambiguates non-homophonic homographs (`couvent`, `est`, `fils`, `violent`, …) from the preceding word. `syllables` works on isolated words only. `renderHtml` handles text + homographs + liaison markers in one pass; target CSS selectors are `.syl-a`, `.syl-b`, `.word`, `.liaison`.

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
