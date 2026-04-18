# Notes de migration v5 → v6

Ce document trace les changements faits lors du portage de **pylirecouleur 0.0.5** (v5, oct. 2022) vers **LireCouleur 6** (v6, tronc actuel de `forge.apps.education.fr`).

## Résultat

**4830 / 4830 mots (100 %)** conformes à LC6 v6 sur un corpus de stress mixte (mots CP/CE1, conjugaisons, mots rares, mots composés à trait d'union).

## Changements de données (automate et listes)

### Dans l'automate

1. **Règle `a/m`** : `m[mbp]` → `m[bp]` (un `amm` comme dans "flamme" ne nasalise plus — correction orthographique).
2. **Règle `o/in`** : phonème `u` → `w` pour `oin` (comme dans "coin", "loin").
3. **8 nouvelles règles `h1`–`h8`** sur la lettre `h` : distinction `#` (h aspiré) vs `#_h_muet` (h muet), basée sur des patterns lookahead précis.
4. **~30 règles ajoutées** : `abbaye`, `hier`, `dessus_dessous`, `que_gue_final`, `oignon`, `voeux`, `sept`, `aujourdhui`, `tomn` (automne), `damn`, `cet`, `es_1/es_2`, `est_1/est_2`, `wag`, `wisig`, `wurt`, etc.

### Dictionnaires supprimés

Le `dictionary.json` v5 (18 mots irréguliers : metz, zeus, dieux grecs) a été **supprimé** : ses cas sont désormais gérés soit par des règles d'automate, soit par le mécanisme `HOMOGRAPHES`.

### Nouvelles listes v6

- `exceptions_en_final` (7 mots : abdomen, dolmen, gentlemen, golden, pollen, spécimen, zen)
- `homographesNonHomophones` (16 entrées : couvent, est, fils, violent, excellent, content, parent, etc.)
- `determinant` (27 entrées : mon, ton, son, ma, ta, sa, mes, tes, ses, nos, vos, leurs, le, la, l', les, des, un, une, du, ce, cette, cet, ces, notre, votre, leur)
- `pronom` (17 entrées : je, j', tu, il, elle, on, nous, vous, ils, elles, y, s'y, me, te, se)

## Changements algorithmiques

### 1. `post_traitement_yod` simplifié (v5 → v6)

- **v5** : fusionnait `i + voyelle` en phonème composé `j_<voyelle>` (ex: `lion` → `l + j_o~`).
- **v6** : remplace juste le phonème `i` par `j` (ex: `lion` → `l + j + o~`). Plus propre.

### 2. `post_traitement_w` supprimé

Le post-traitement qui fusionnait `u + voyelle` en `w_<voyelle>` n'existe plus en v6. La fonction reste présente dans l'API Rust en no-op pour compat.

### 3. Classe phonème `#_h_muet`

Nouveau code phonème classé `Silent`, utilisé pour marquer explicitement un `h` muet. Permet à `liaisonAmont` de savoir qu'un mot qui commence par un h muet **autorise** la liaison (ex: "les hôtels" → `les-z-hôtels`).

### 4. Désambiguïsation contextuelle (homographes)

Nouvelle fonction `homographs::lookup(word, previous_word)` qui court-circuite l'automate si un mot est un homographe non homophone et que le mot précédent fait partie de son contexte. Exemples :

```
"le couvent" (lieu, nom)     → codage avec a~ nasal
"elles couvent" (verbe)      → codage avec q_caduc + verb_3p (muet)
"il est" (verbe)              → phonème e^_comp, step=3 (tout le mot lu)
"vers l'est" (direction, nom) → e^_comp + s + t (3 phonèmes distincts)
```

### 5. Mode par défaut Std

La v6 utilise `Std` par défaut (dédouble les consonnes : `homme → hom-me`, `abaisse → a-bais-se`), là où pylirecouleur 0.0.5 utilisait `Lc` (consonnes groupées : `ho-mme`, `a-bai-sse`). `Std` correspond à la segmentation pédagogique dominante à l'école.

## Pièges rencontrés et corrections

### Piège 1 : dédoublement abusif des semi-voyelles

En mode Std, j'ai d'abord dédoublé systématiquement les consonnes ET les semi-voyelles doublées (`j(ll) → j(l) + j(l)`). C'est **faux** : LC6 ne dédouble que les consonnes "vraies" (classe `c`). Un `j` (yod, classe `s`) reste indivisible.

**Fix** (`decoder.rs`, étape 1 de `assemble_syllables`) : vérifier `classify() == Consonant` plutôt que `Consonant || SemiVowel`.

### Piège 2 : phonèmes vides dans sylph

Un caractère non reconnu par l'automate (ex: `-` dans `grand-père`) produit un phonème `{code: "", step: 1}`. Ces phonèmes doivent être **complètement ignorés** lors de la construction de `sylph` (c'est ce que fait le check `estPhoneme()` en JS). Sans ça, le `-` reste collé à la syllabe voisine.

**Fix** (`decoder.rs`, étape 2) : skipper les phonèmes vides :
```rust
if ph.code.is_empty() { continue; }
```

### Piège 3 : règle `*` conditionnelle pour `œ`

Le JSON v6 contient pour `œ` une règle `"*"` avec condition `{"+": "u"}`. En lecture naïve, cela voudrait dire "ne s'applique que si suivi d'un `u`". **Mais LC6 ignore cette condition** (cf. `module.js` l. 811-816) : la règle `*` est **toujours** appliquée comme règle par défaut, sans test de condition.

**Fix** (`build/generate_data.py`) : pour `*`, toujours émettre `default_rule`, jamais de règle conditionnelle.

### Piège 4 : tokenisation des tirets

Le cleaner v5 remplaçait `-` par espace, ce qui tokenisait `grand-père` en trois mots séparés. En v6, le tiret doit rester dans le mot pour être consommé comme phonème vide par l'automate.

**Fix** (`cleaner.rs`) : `-` et `_` ajoutés à `est_significatif()`.

## Points non portés (explicitement laissés pour plus tard)

Ces fonctionnalités LC6 **ne sont pas** dans le crate :

- `liaisonAmont` / `liaisonAval` — détection des possibilités de liaison entre mots, utile pour le coloriage prosodique (pas pour la simple segmentation syllabique). Faible priorité.
- `regle_en_final` — méthode présente dans LC6 mais non invoquée par l'automate de base ; probablement utilisée uniquement pour les homographes. Déjà couverte par notre mécanisme d'homographes.
- `dernierTraitement` — cache d'état interne à LC6, pas nécessaire en Rust.

## Dettes techniques identifiées

1. **Perf du parser** : le cache de regex est global avec un `Mutex`, inutile pour un usage mono-thread. Passer à un `OnceCell` thread-local ou pré-compiler toutes les regex à `build.rs` time.
2. **Gestion des `+` conditionnels de `*`** : même si LC6 ignore la condition pour `œ`, d'autres lettres pourraient à terme avoir une règle `*` conditionnelle utilisée. Vérifier le comportement exact si LC6 évolue.
3. **Mode `Lc`** : j'ai conservé le mode LC5 (consonnes groupées) comme option, mais il n'est plus aligné 100% à LC6 (son équivalent LC6 serait un mode "phonologique pur" qui n'existe pas en v6). À étiqueter clairement comme "mode historique".

## Sources

- **LireCouleur v6** : <https://forge.apps.education.fr/lirecouleur/lirecouleur.forge.apps.education.fr>
- **pylirecouleur v5** : <https://pypi.org/project/pylirecouleur/> (Marie-Pierre Brungard, extraction Python 2022)
- **Auteurs originaux** : Marie-Pierre Brungard & Luc Brungard, GPL v3
