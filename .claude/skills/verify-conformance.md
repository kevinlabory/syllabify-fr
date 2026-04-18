---
name: verify-conformance
description: Exécute la régression LC6 sous les deux backends regex (regex-full défaut + regex-lite). Gate obligatoire avant toute PR qui touche parser.rs, decoder.rs, rules.rs, data.rs, cleaner.rs, homographs.rs, ou le feature-gate regex.
---

# verify-conformance

Lance les deux passes de régression et rapporte le résultat sous forme de tableau compact.

## Commandes

```bash
cargo test
cargo test --no-default-features --features regex-lite --lib --tests
```

## Attendu

Les deux passes doivent afficher :
- `19 passed` sur les tests unitaires
- `1 passed` sur `regression_syllabes` (qui vérifie 4830/4830 mots)

Format de rapport :

| Backend | Unit | Régression | Doctest | Statut |
|---|---|---|---|---|
| `regex-full` (défaut) | 19/19 | 4830/4830 | 1/1 | ✅ |
| `regex-lite` | 19/19 | 4830/4830 | — | ✅ |

## Diagnostic d'échec

- **Une seule passe rouge** → divergence entre backends regex. Causes probables : classe de caractères multi-byte, ancre `$` en multiline, alternance vide. Inspecter les patterns concernés dans `src/data.rs` et tester manuellement avec `regex-lite` vs `regex`.
- **Les deux passes rouges** → régression algorithmique réelle. Lire `/tmp/syllabify_mismatches.txt` (écrit par le test sur échec), puis enchaîner avec `/diagnose-divergence`.
- **Divergence > 0 mais < 50 mots** → presque toujours un piège documenté dans `NOTES-v6.md` § "Pièges rencontrés".

**L'invariant 4830/4830 est le contrat de licence et de correction avec LireCouleur upstream. Ne jamais merger avec un chiffre inférieur.**
