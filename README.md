# syllabify-fr

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-4830%2F4830-brightgreen.svg)](tests/regression.rs)
[![Conformance](https://img.shields.io/badge/LC6_conformance-100%25-brightgreen.svg)](NOTES-v6.md)
[![Crates.io](https://img.shields.io/crates/v/syllabify-fr.svg)](https://crates.io/crates/syllabify-fr)
[![Docs.rs](https://docs.rs/syllabify-fr/badge.svg)](https://docs.rs/syllabify-fr)
[![CI](https://github.com/kevinlabory/syllabify-fr/actions/workflows/ci.yml/badge.svg)](https://github.com/kevinlabory/syllabify-fr/actions)
[![Coverage](https://codecov.io/gh/kevinlabory/syllabify-fr/branch/main/graph/badge.svg)](https://codecov.io/gh/kevinlabory/syllabify-fr)

Syllabification française pour l'apprentissage de la lecture, en particulier pour les enfants dyslexiques.

Portage en Rust de **[LireCouleur 6](https://lirecouleur.forge.apps.education.fr/)** (Marie-Pierre & Luc Brungard, GPL v3).

## Pourquoi pas un syllabifieur générique ?

Les outils typographiques (Hypher, hyphen-fr…) cherchent le **minimum** de coupures valides pour la mise en page, pas toutes les frontières syllabiques. Résultats inutilisables pour la lecture pédagogique :

- `chocolat` → `[chocolat]` (aucune coupure)
- `école` → `[école]` (évite de laisser un seul caractère)

LireCouleur segmente **chaque mot en toutes ses syllabes**, en distinguant finement les cas épineux : `-ent` muet vs prononcé, graphème `ille`, diérèses `lion/hier`, h muet/aspiré, liaisons avec apostrophe, homographes non homophones (`le couvent` vs `elles couvent`), etc.

## Exemples

```text
chocolat                       → cho-co-lat
famille                        → fa-mi-lle
fille                          → fi-lle
mille                          → mil-le
parlent  (verbe)               → par-lent      (-ent muet)
prudent  (adjectif)            → pru-dent      (-ent prononcé)
anticonstitutionnellement      → an-ti-cons-ti-tu-tion-nel-le-ment
lion                           → lion          (synérèse)
l'école                        → l'é-co-le
grand-père                     → grand-pè-re
homme                          → hom-me
œil                            → œil
```

### Homographes contextuels

```text
le couvent accueille les moines   → le cou-vent ac-cue-ille les moi-nes
                                                 [nom, -ent prononcé a~]

elles couvent leurs œufs          → el-les cou-vent leurs œufs
                                              [verbe, -ent muet]

il est là, vers l'est             → il est là, vers l'est
                                        [verbe, 1 phonème · direction, 3 phonèmes]
```

## Utilisation

### CLI

```bash
cargo build --release

./target/release/syllabify chocolat famille parlent
# cho-co-lat fa-mi-lle par-lent

./target/release/syllabify --text "le petit chat noir"
# le pe-tit chat noir

./target/release/syllabify --json chocolat
# ["cho","co","lat"]

echo -e "famille\nlion\nhier" | ./target/release/syllabify -
# fa-mi-lle
# lion
# hier
```

### Bibliothèque Rust

```rust
use syllabify_fr::{syllables, syllabify_text, TextChunk};

assert_eq!(syllables("famille"), vec!["fa", "mi", "lle"]);
assert_eq!(syllables("parlent"), vec!["par", "lent"]);

// Texte complet — les homographes sont désambiguïsés selon le mot précédent
for chunk in syllabify_text("le couvent accueille les moines") {
    match chunk {
        TextChunk::Word(syls) => print!("{}", syls.join("-")),
        TextChunk::Raw(s)     => print!("{}", s),
    }
}
// Affiche : le cou-vent ac-cue-ille les moi-nes
```

### Contrôle fin

```rust
use syllabify_fr::{syllables_with, AssembleMode, SyllableMode};

// Mode LC (segmentation phonologique, consonnes doubles groupées)
syllables_with("homme", false, AssembleMode::Lc, SyllableMode::Written);
// → ["ho", "mme"]

// Mode STD (défaut LC6, segmentation pédagogique)
syllables_with("homme", false, AssembleMode::Std, SyllableMode::Written);
// → ["hom", "me"]

// Syllabes orales (fusionne le q_caduc final)
syllables_with("école", false, AssembleMode::Std, SyllableMode::Oral);
// → ["é", "cole"]
```

### Liaisons inter-mots

Prédicats purs pour décider si une liaison orale est possible entre deux mots (utile pour le coloriage prosodique, pas pour la segmentation syllabique) :

```rust
use syllabify_fr::{liaison_amont, liaison_aval, liaison_possible};

assert!(liaison_aval("les"));              // déterminant pluriel → déclenche
assert!(liaison_amont("hôtel"));           // h muet → peut recevoir
assert!(!liaison_amont("homard"));         // h aspiré → bloque
assert!(liaison_possible("les", "hôtels")); // → on lira [le.zo.tɛl]
```

### Rendu HTML (coloriage syllabique)

Sortie prête à l'emploi pour un front-end : chaque syllabe dans un `<span class="syl syl-a">` / `<span class="syl syl-b">` alternés, chaque mot dans un `<span class="word">`, et les liaisons possibles entre mots matérialisées par `<span class="liaison" data-with="…">`.

```rust
use syllabify_fr::{render_word_html, render_html};

// Mot seul
render_word_html("chocolat");
// → <span class="word"><span class="syl syl-a">cho</span><span class="syl syl-b">co</span><span class="syl syl-a">lat</span></span>

// Texte complet avec ponctuation, homographes et liaisons
render_html("les hôtels, le couvent accueille");
// → <span class="word">…les…</span> <span class="liaison" data-with="z"></span><span class="word">…hôtels…</span>,
//   <span class="word">…le…</span> <span class="word">…cou…vent…</span> <span class="word">…ac…cue…ille…</span>
```

À la CSS du consommateur de définir les couleurs via les sélecteurs `.syl-a`, `.syl-b`, `.liaison`. Disponible aussi dans le binding WASM sous les noms `renderHtml` / `renderWordHtml`.

## Architecture

Pipeline en 5 étapes, fidèle à LireCouleur 6 :

1. **Nettoyage** — minuscules, apostrophes → `@`, ponctuation → espace
2. **Parser** — automate à états finis (1 règle par lettre avec lookahead/lookbehind regex + 10 règles spéciales type `regle_ient`, `regle_mots_ent`…)
3. **Post-traitements phonologiques** — `eu` ouvert/fermé, `o` ouvert/fermé, yod (`i+V → j`)
4. **Désambiguïsation contextuelle** — 16 homographes non homophones résolus selon le mot précédent
5. **Assemblage syllabique** — regroupement V/C/S, attaques complexes (`bl`, `tr`, `pl`…), fusion des diphtongues

## Données embarquées

Extraites du `module.js` de LireCouleur 6 (forge.apps.education.fr) :

- **Automate** : 43 lettres (y.c. accentuées), ~480 règles au total
- **Base linguistique** : 15 listes totalisant ~1200 entrées (217 verbes en -ier, 144 en -mer, 154 mots en -ent, etc.)
- **Homographes** : 16 mots avec ~30 variantes contextuelles

Pour régénérer `src/data.rs` depuis une nouvelle version de LireCouleur 6 :

```bash
# 1. extraire le JSON depuis module.js (Node.js)
node build/extract_v6_data.js

# 2. régénérer les structures Rust
python3 build/generate_data.py
```

## Tests

```bash
cargo test
```

- **19 tests unitaires** (classification, cleaner, parser, règles, homographes)
- **1 test de régression** sur **4 830 mots** : conformité stricte à LireCouleur 6

Pour régénérer l'oracle depuis LC6 :

```bash
# Nécessite Node.js + le module.js de LC6 à portée
LC6_PATH=/chemin/vers/lirecouleur/js/lirecouleur/module.js \
  node build/generate_oracle.js build/data/corpus.txt
```

## Licence

**GPL v3 ou ultérieure**, conformément à l'œuvre d'origine (LireCouleur par Marie-Pierre et Luc Brungard).

Les données embarquées (`src/data.rs`) dérivent directement de LireCouleur 6, lui-même GPL v3. Toute utilisation ou redistribution doit respecter les termes de la GPL v3.

Voir `LICENSE` pour le texte intégral.

## Historique

Voir [`NOTES-v6.md`](NOTES-v6.md) pour le détail de la migration v5 → v6 (corrections, pièges rencontrés, dette technique).

## Remerciements

Ce projet ne serait pas possible sans le travail pionnier et généreux de **Marie-Pierre Brungard** et **Luc Brungard**, qui maintiennent LireCouleur depuis plus de 15 ans au service des enseignants, orthophonistes, enfants dyslexiques et apprenants du français.

Site officiel : <https://lirecouleur.forge.apps.education.fr/>

## Roadmap

- [x] Port Rust fidèle à LireCouleur 6 (100% conformité sur 4 830 mots)
- [x] CLI
- [x] Homographes contextuels
- [x] `liaisonAmont` / `liaisonAval` (liaisons entre mots)
- [x] FFI C (`syllabify-fr-ffi`)
- [x] WASM (binding JS/Deno)
- [x] Python (PyO3)
- [x] Java / Kotlin / Android (JNI, `syllabify-fr-jni`)
- [x] Sortie HTML avec spans pour intégration web (coloriage syllabique)
