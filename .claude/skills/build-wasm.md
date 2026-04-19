---
name: build-wasm
description: Build le crate syllabify-fr-wasm avec wasm-pack (--target web --release) et mesure la taille gzippée du binaire. Échoue si le .wasm dépasse 300 Ko gzippés — le budget qui justifie l'usage en remplacement de Hypher sur dyscolor.com.
---

# build-wasm

## Commandes

```bash
wasm-pack build wasm/ --target web --release
RAW=$(wc -c < wasm/pkg/syllabify_fr_wasm_bg.wasm | tr -d ' ')
GZ=$(gzip -c wasm/pkg/syllabify_fr_wasm_bg.wasm | wc -c | tr -d ' ')
echo "raw:     $RAW bytes"
echo "gzipped: $GZ bytes"
test "$GZ" -lt 300000 && echo "PASS: under 300 KB budget" || echo "FAIL: over budget"
```

## Rapport attendu

| Métrique | Valeur | Budget | % utilisé |
|---|---|---|---|
| Raw `.wasm` | X o | — | — |
| Gzippé | X o | 300 000 o | X % |
| Smoke tests | 4/4 | — | — |

Baseline de référence (commit initial WASM) : **~75 Ko gzippés** (regex-lite + serialization js_sys manuelle, sans serde).

## Si hors budget

1. Identifier les symboles lourds :
   ```bash
   twiggy top -n 30 wasm/pkg/syllabify_fr_wasm_bg.wasm
   ```
2. Suspects probables par ordre de fréquence :
   - Retour sur `regex` full au lieu de `regex-lite` (+80 Ko gzippés)
   - Ajout involontaire de `serde` + `serde-wasm-bindgen` (+30 Ko)
   - Expansion de `src/data.rs` après régénération (mots/règles ajoutés)
   - Débordement de `wasm-bindgen` glue (Array/Object abus)
3. Vérifier que `[profile.release.package.syllabify-fr-wasm]` utilise toujours `opt-level = "z"` dans le `Cargo.toml` racine.

## Smoke tests (complément)

```bash
wasm-pack test --node wasm/
```

Les 4 tests de `wasm/tests/smoke.rs` doivent passer. Un échec ici indique une régression fonctionnelle, pas un problème de taille.

**Le budget 300 Ko est une ligne dure** : au-delà, le projet perd son avantage compétitif face aux alternatives typographiques légères sur le front-end dyscolor.com.
