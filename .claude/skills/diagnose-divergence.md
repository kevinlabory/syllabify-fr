---
name: diagnose-divergence
description: Lit /tmp/syllabify_mismatches.txt (écrit par regression_syllabes en cas d'échec), classifie chaque mismatch contre les 4 pièges documentés dans NOTES-v6.md et propose un diagnostic avant toute modification de code.
---

# diagnose-divergence

Avant tout fix : **lire `/tmp/syllabify_mismatches.txt`** (format : `mot  attendu=a|b|c obtenu=x|y|z`).

## Grille de classification

Pour chaque mot divergent, tenter de matcher contre ces patterns connus :

| Symptôme observé | Piège probable (NOTES-v6.md) | Fichier à inspecter | Fix type |
|---|---|---|---|
| Consonnes doubles découpées à tort (ex: `j(ll)` → `j(l)+j(l)` pour `fille`) | **Piège 1** : dédoublement des semi-voyelles | `decoder.rs` étape 1 `assemble_syllables` | `classify() == Consonant` (pas `|| SemiVowel`) |
| Tirets ou caractères non reconnus collés à la syllabe voisine (`grand-père` → `gran-d-pè-re`) | **Piège 2** : phonème vide non skippé | `decoder.rs` étape 2 | `if ph.code.is_empty() { continue; }` |
| Mot en `œ` mal segmenté, règle `*` non appliquée | **Piège 3** : condition `+` de la règle `*` | `build/generate_data.py` | `*` émet toujours `default_rule`, sans test de condition |
| Mot composé à tiret découpé en 2 tokens | **Piège 4** : cleaner mange le `-` | `cleaner.rs::est_significatif()` | `-` et `_` préservés |
| Homographe (`couvent`/`est`/`fils`/`violent`/`parent`…) mal lu | Désambiguïsation échouée | `homographs.rs::lookup` | vérifier normalisation `previous_word` (lowercase + `\u{2019}` → `'`) |
| Divergence **uniquement sous regex-lite** | Incompatibilité regex-lite sur un pattern de `data.rs` | `src/data.rs` | identifier le pattern via bisection mot à mot |
| `-ent` prononcé vs muet incorrect | Listes `MOTS_ENT` / `VERBES_*` incomplètes ou mal consultées | `rules.rs::regle_mots_ent` | régénérer `data.rs` depuis LC6 |
| `h` initial mal traité (muet vs aspiré) | Règles `h1`–`h8` | `data.rs` lettre `h` | régénérer `data.rs` depuis LC6 |

## Méthode

1. **Compter et grouper** les mots par type probable avant tout fix — une cause structurelle résout souvent plusieurs mots à la fois.
2. **Proposer un diagnostic** par groupe au format :
   > N mots affectés par [piège X] : [liste]. Cause probable : [description]. Fix candidat : [fichier:ligne + change]. Confirmer avant modification ?
3. **Ne jamais modifier `src/data.rs` à la main** — si la cause pointe vers les données, la correction doit passer par `build/extract_v6_data.js` puis `build/generate_data.py`.
4. **Ne jamais modifier `tests/oracle.json` à la main** — c'est la source de vérité immuable issue de LC6.

## Si aucun piège documenté ne colle

Hypothèses dans l'ordre :
- **LC6 a évolué** en amont : régénérer `data.rs` et/ou `oracle.json`.
- **Ordre des post-traitements perturbé** : `post_process_e` → `post_traitement_yod` → `post_process_o` doit rester dans cet ordre exact (voir `decoder.rs::extract_phonemes_word`).
- **Nouvelle classe phonème** mal propagée dans `phoneme.rs::classify` : vérifier qu'un nouveau code n'est pas tombé dans le default `Vowel`.

## Tabou absolu

- Ne jamais "corriger" localement en patchant un cas particulier qui casse l'alignement avec LC6.
- Ne jamais skipper un test pour le faire passer (`#[ignore]`) sans ouvrir une issue et documenter la raison.
