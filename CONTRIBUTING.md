# Contributing to syllabify-fr

## Prérequis

- Rust stable ≥ 1.70 (`rustup update stable`)
- `cargo` — aucune dépendance système requise

## Lancer les tests

```bash
cargo test                    # tests unitaires + regression + doctests
cargo test --test regression  # oracle seul (4830 mots LC6)
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## Règle d'or

**`cargo test --test regression` doit rester à 4830/4830.**

Ce projet est un portage de LireCouleur 6 (LC6) ; la conformité à l'algorithme de référence est non négociable. Toute modification du parser, du decoder ou des données doit passer l'oracle sans régression. En cas d'échec, les mots en désaccord sont écrits dans `/tmp/syllabify_mismatches.txt`.

## Architecture

Voir [`CLAUDE.md`](CLAUDE.md) pour le détail du pipeline en 5 étapes et les pièges connus (`NOTES-v6.md`).

`src/data.rs` (~66 KB) est généré — ne pas l'éditer à la main, voir la section "Regenerating embedded data" dans `CLAUDE.md`.

## Soumettre une PR

1. Forker le dépôt et créer une branche descriptive (`fix/...`, `feat/...`)
2. S'assurer que `cargo test`, `cargo clippy` et `cargo fmt --check` passent
3. Décrire brièvement le changement et son impact sur l'oracle dans la PR
4. Les PRs qui cassent la conformité LC6 ne seront pas mergées

## Licence

En soumettant une contribution, vous acceptez qu'elle soit distribuée sous la licence [GPL-3.0-or-later](LICENSE).
