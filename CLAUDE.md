# CLAUDE.md

Ce fichier guide Claude Code (claude.ai/code) pour le travail sur ce dépôt.

## Projet

Portage Rust de **LireCouleur 6** (Marie-Pierre & Luc Brungard, GPL v3), un
syllabifieur français conçu pour l'apprentissage de la lecture — notamment chez
les enfants dyslexiques. Contrairement aux séparateurs typographiques (Hypher,
hyphen-fr…) qui minimisent les points de coupure, cette bibliothèque segmente
chaque mot en **toutes** ses syllabes.

Le portage doit rester **100 % conforme** à LC6 sur l'oracle de régression de
4830 mots (`tests/oracle.json`). Toute modification du parser, du décodeur ou
des données doit garder `cargo test --test regression` au vert ; c'est
l'invariant non-négociable.

## Commandes

```bash
cargo build --release            # bibliothèque + binaire CLI `syllabify`
cargo test                       # ~124 tests sur 8 suites (lib, cli, edge, properties,
                                 # regression, ffi, jni, doctests)
cargo test --test regression     # oracle 4830 mots seul ; écrit les divergences
                                 # dans /tmp/syllabify_mismatches.txt en cas d'échec
cargo test <nom>                 # un seul test par nom
./target/release/syllabify chocolat            # cho-co-lat
./target/release/syllabify --text "…"          # syllabe le texte (désambiguïsation des homographes)
./target/release/syllabify --json mot          # sortie JSON (échappement RFC 8259 via serde_json)
echo "famille" | ./target/release/syllabify -  # mode stdin, un mot par ligne
```

Régénération des données embarquées depuis une nouvelle version LC6 (nécessite
Node.js + Python 3, et accès au `module.js` de LC6) :

```bash
node build/extract_v6_data.js       # module.js → build/data/*.json
python3 build/generate_data.py      # JSON → src/data.rs (la sortie n'est pas
                                    # éditable à la main)
LC6_PATH=/path/to/lirecouleur/js/lirecouleur/module.js \
  node build/generate_oracle.js build/data/corpus.txt   # régénère tests/oracle.json
```

## Architecture

Pipeline en cinq étapes, la fidélité à LC6 est la contrainte de design à chaque
étape :

1. **`cleaner.rs`** — passage en minuscules, apostrophe → `@`, ponctuation →
   espace. Les tirets et underscores sont **conservés en-mot** (changement v6 ;
   cf. NOTES-v6.md Piège 4), donc `grand-père` est un seul token.
2. **`parser.rs`** — automate à états finis. Le mot est porté par
   `parser::Word` (texte + table d'offsets caractère → octet) : les règles
   gardent l'indexation par caractère façon Python tout en découpant des
   `&str` sans allocation via `Word::slice`. Pour chaque lettre (indexée à
   partir de 1 façon Python), sélectionne une règle via regex lookahead
   (`plus`) / lookbehind (`minus`), ou l'une des ~10 règles spéciales
   (`regle_ient`, `regle_mots_ent`, `regle_verbes_ier`, …) dans `rules.rs`.
   Produit `Phoneme { code, step }`. Les caractères inconnus deviennent des
   phonèmes vides (`code=""`, `step=1`) qui doivent être **filtrés** en aval
   (NOTES-v6.md Piège 2).
3. **`decoder.rs` — post-traitements** — `post_process_e` (eu ouvert/fermé),
   `post_process_o` (o ouvert/fermé), `post_traitement_yod` (v6 : `i + V` →
   remplace `i` par `j`, **pas** de fusion — NOTES-v6.md §1).
   `post_traitement_w` est volontairement un no-op (supprimé en v6).
4. **`homographs.rs`** — `lookup(word, previous_word)` court-circuite
   l'automate pour 16 homographes non homophones (`couvent`, `est`, `fils`,
   `violent`, `excellent`, …). Appelé depuis `decoder::extract_syllables`
   lors du parcours d'un texte complet.
5. **`decoder.rs` — `assemble_syllables`** — regroupe V/C/S en syllabes,
   traite les attaques complexes (`bl`, `tr`, `pl`, …), et en `AssembleMode::Std`
   (défaut v6) ne sépare que les **vraies consonnes** (`PhonClass::Consonant`)
   sur les doubles lettres. **Ne jamais séparer une semi-voyelle** comme
   `j(ll)` — c'était le Piège 1 du portage v5→v6.

### Données (`src/data.rs`, ~66 KB, généré)

- `AUTOMATON` : 43 lettres × ~480 règles au total. **Alphabet effectif** :
  a-z + `'` + `@` + `_` + à â ç è é ê ë î ï ô ö ù û œ. **Pas** dans
  `AUTOMATON` : ä ÿ æ ü (rares en français, mots d'emprunt type *müesli*).
  Pour ces caractères, `cleaner::est_significatif` les retient mais le
  parser produit un phonème vide qui se fait silencieusement filtrer
  → la lettre disparaît du résultat (cf.
  `tests/edge_cases.rs::chars_outside_automaton_are_dropped`,
  `tests/properties.rs::syllables_preserve_letter_count_for_french`).
- 15 listes de mots (~1200 entrées) : `MOTS_ENT`, `VERBES_IER`, `VERBES_MER`,
  `EXCEPTIONS_FINAL_ER`, `POSSIBLES_AVOIR`, etc.
- `HOMOGRAPHES` : 16 entrées avec ~30 variantes de contexte.
- `DETERMINANT` / `PRONOM` : utilisés par le matching de contexte des homographes.

**Ne pas éditer `src/data.rs` à la main** — régénérer via `build/generate_data.py`.
Piège 3 (NOTES-v6.md) : pour la règle par défaut `*`, `generate_data.py` doit
toujours émettre `default_rule` (ignorer la condition `+` du JSON) — le
`module.js` de LC6 l'ignore aussi.

### API publique (`lib.rs`)

Re-exports et modules publics :

- Modules : `html`, `letters`, `liaisons`.
- Types re-exportés : `AssembleMode`, `SyllableMode`, `TextChunk`.
- Fonctions racine :
  - `syllables(word)` — défauts : `Std`, `Written`, `novice_reader=false`.
  - `syllables_with(word, novice_reader, AssembleMode, SyllableMode)` —
    contrôle complet.
  - `phonemes(word) -> Vec<(String, String)>` — paires `(code, lettres)`.
  - `syllabify_text(text) -> Vec<TextChunk>` — `Word(Vec<String>)` ou
    `Raw(String)` ; effectue la désambiguïsation des homographes sur le mot
    précédent.
  - `render_html`, `render_word_html`, `liaison_amont`, `liaison_aval`,
    `liaison_possible`.
- Résolution par nom, partagée par les cinq consommateurs (CLI + 4 bindings)
  depuis 0.10.0 : `letters::presets::by_name(&str) -> Option<Vec<LetterRule>>`
  et `letters::RenderMode::from_name(&str) -> Option<RenderMode>`. Les deux
  rendent `None` sur un nom inconnu — c'est délibéré : chaque appelant garde
  son contrat (`unwrap_or_default()` côté WASM/C/JNI, `ValueError` côté
  Python). **Ajouter un preset = l'ajouter dans `by_name`**, les bindings
  en héritent sans modification.

Les modules `parser`, `decoder`, `homographs`, `phoneme`, `cleaner`, `data`,
`rules` sont `pub(crate)` : leurs types internes (`Phoneme`, `DecodedPhoneme`,
…) peuvent évoluer sans casser l'API publique — `code` est ainsi passé de
`String` (≤ 0.8.2) à `Cow<'static, str>` (0.8.3) puis à `&'static str`
(0.10.0) sans qu'aucune de ces étapes ne soit breaking.

### Modes

- `AssembleMode::Std` (défaut, pédagogique — `homme → hom-me`) vs `Lc`
  (phonologique — `ho-mme`). `Lc` n'est plus aligné à 100 % sur LC6 ;
  marqué `#[deprecated(since = "0.4.0")]`, à traiter comme option héritée.
- `SyllableMode::Written` vs `Oral` (le `q_caduc` final est fusionné avec
  la syllabe précédente : `école → é-cole`).

## Parité bindings (checklist obligatoire)

Toute modification de la **surface publique** de `syllabify-fr` (nouveau
`pub mod`, `pub fn`, `pub struct` re-exporté, nouvelle fonction de la crate
root) **doit** être propagée aux bindings *avant* de bumper la version
du workspace :

- `wasm/src/lib.rs` — `#[wasm_bindgen]`
- `ffi/src/lib.rs` — `pub unsafe extern "C"` (renvoie `*mut c_char`,
  libérer avec `syllabify_free`). En cas de nouvelle export :
  ajouter à `ffi/cbindgen.toml` `[export].include` et au header
  manuel `ffi/include/syllabify_fr.h`.
- `py/src/lib.rs` — `#[pyfunction]` + `m.add_function(...)` dans
  `#[pymodule]`
- `jni/src/lib.rs` —
  `pub extern "system" fn Java_com_dyscolor_syllabify_SyllabifyFr_…`
- `swift/Sources/SyllabifyFr/SyllabifyFr.swift` — wrapper Swift
  idiomatique au-dessus du C ABI (consomme `ffi/include/syllabify_fr.h`
  via le module `CSyllabifyFr`). Le binding Swift est **dérivé** du
  FFI : pas de nouvelle dette JSON/HTML à maintenir, mais une fonction
  C non exposée côté Swift est invisible aux apps iOS.

Choisir une signature uniforme (presets/strings plutôt que types Rust
custom) pour réduire le coût marginal par binding. Côté CI : `clippy`
tourne en `--workspace` (cf. `ci.yml`/`release.yml`), donc les régressions
de style sur les bindings sont attrapées. La couverture codecov reste
exclue des bindings (testés dans leur écosystème natif).

**Garantie de stabilité d'API — partage des responsabilités.** La surface
publique de la *crate core* `syllabify-fr` est gardée **automatiquement**
par le job `semver` de `ci.yml` (`cargo-semver-checks` contre la baseline
crates.io) : un changement cassant sans bump de version adéquat fait
échouer la PR. Les **4 bindings** ne sont pas sur crates.io
(npm / PyPI / Maven), donc `cargo-semver-checks` ne les voit pas — leur
parité reste garantie par la **checklist manuelle ci-dessus**. Règle
pratique : un nouvel item `pub` dans la lib core → semver-checks impose
un bump correct ; le propager aux bindings reste de ta responsabilité.

Précédent concret : v0.8.0 a livré `letters` côté lib + CLI mais a
oublié les 4 bindings, rendant le `.d.ts` npm v0.8.0 identique au v0.7.0.
Corrigé en v0.8.1.

## Idiomes Rust attendus

Issues d'un audit B+ ; les frictions repérées ont été corrigées en
0.8.2–0.8.4, les règles passives ci-dessous en préservent l'esprit.
Vérifiées à la lecture (pas de tooling automatique au-delà de
`clippy::pedantic`, activé sur la crate root depuis 0.8.2).

- **Pas de `Vec<String>` intermédiaire pour scanner des codes phonèmes** —
  préférer `Vec<&str>` ou itérer. Pattern à conserver :
  `decoder::post_process_e/o` construit un `Vec<&str>` qui emprunte `pp`
  sans clone.
- **`unwrap()` interdit en code prod sans commentaire d'invariant** —
  préférer `let-else { unreachable!("invariant : …") }` ou
  `.expect("invariant : …")` qui documente la pré-condition. Pattern en
  place dans `decoder::assemble_syllables`.
- **Bindings = uniformité de signature, pas de logique métier** — si un
  binding doit transformer/normaliser, factoriser dans la lib core.
  Cas d'école résolu en 0.10.0 : `preset_rules` était recopié dans les
  4 bindings, avec des signatures divergentes (`Vec` vide côté C/JNI,
  `Option` côté Python/WASM) ; tout passe désormais par
  `letters::presets::by_name`.
- **Choisir le type le plus simple qui décrit la donnée : `&'static str`
  si elle est toujours statique, `Cow<'static, str>` seulement si la
  variante `Owned` est réellement atteignable, `String` en dernier
  recours.** `LetterRule::pattern` (`letters.rs`) justifie son `Cow` —
  `LetterRule::new` accepte un preset statique *comme* un `String`
  construit à l'exécution. `Phoneme::code` / `DecodedPhoneme::code` ne le
  justifiaient pas : leur `Owned` n'était produit nulle part, d'où le
  passage à `&'static str` en 0.10.0 (qui a rendu `code` `Copy` et fait
  tomber au passage quatre `.clone()` et un contournement d'emprunt dans
  `post_process_o`). Un `Cow` dont `Owned` est inatteignable est un
  discriminant payé pour rien.
- **`#[non_exhaustive]` sur tout enum/struct public exposé via le
  pipeline.** Déjà appliqué à `TextChunk`, `AssembleMode`, `SyllableMode`,
  `RenderMode`, `LetterStyle` — ne pas régresser.
- **`#[must_use]` sur toute fonction pure.** Présent sur les fonctions
  racine de `lib.rs`, sur `homographs::lookup`, `presets::by_name` et
  `RenderMode::from_name`.
- **Hot path = pas d'alloc.** `parser::one_step`,
  `decoder::assemble_syllables` tournent par mot. Tout `String::new()`,
  `Vec::new()`, `format!()` ajouté ici doit être justifié en commentaire.
- **Pipeline modifié → `cargo test --test regression` avant ET après.**
  L'oracle 4830 mots est non-négociable. Si une optimisation idiomatique
  change le résultat sur 1 mot, c'est l'optimisation qui perd.
- **Pas de `Result`/`thiserror` sur la lib core, c'est volontaire.** La
  lib est *totale* : seuls les `expect()` sur AUTOMATON statique
  (`parser.rs:55, 59`) peuvent échouer, et c'est un bug data, pas une
  erreur runtime à propager.
- **JSON émis vers un consommateur externe → `serde_json`, jamais
  d'échappement maison.** Depuis 0.8.4, CLI / FFI / JNI utilisent
  `serde_json::to_string(&json!(...))` ; les anciens helpers maison
  laissaient passer les caractères de contrôle U+0000..U+001F (JSON
  invalide). Property tests adversariaux : `tests/cli.rs`,
  `ffi/src/lib.rs::tests`, `jni/src/lib.rs::tests`.
- **Idiomes 2024/2025 préférés** : `let-else` plutôt que
  `match { Some(x) => x, None => return }` ; `if let && chain`
  (Rust 1.85+, disponible) plutôt que `if let { if let { … } }`
  imbriqué ; `OnceLock` plutôt que `Mutex<…>` pour un cache lazy en
  lecture-fréquente.

## Ce qui n'est explicitement **pas** porté

Cf. NOTES-v6.md §« Points non portés ». `regle_en_final` et
`dernierTraitement` restent hors-scope — ne pas les ajouter sans
confirmation. `liaisonAmont`/`liaisonAval` sont portés dans
`src/liaisons.rs` sous forme de prédicats purs (pas d'effet de bord
syllabique).

## Dette technique à connaître

- **Optimisation regex du parser : piste explorée, mesurée, abandonnée.**
  Après la passe zero-alloc de 0.10.0 (−55 % à −73 %), le coût restait
  superlinéaire : rapport **×5,93** entre un mot de 25 lettres et un de 8,
  contre ×3,12 pour du linéaire. La boucle `k` de `parser::check_context`
  essaie toutes les positions de départ, soit O(n) recherches regex par
  règle dans le pire cas.

  Deux optimisations ont été implémentées et mesurées (criterion, macOS
  arm64, baseline v0.10.0), puis **abandonnées** — ne pas les retenter
  sans lire ce qui suit :

  | Variante | `chocolat` (8 l.) | mot de 25 l. | `syllabify_text/sentence` |
  |---|---|---|---|
  | Lookahead ancré `^(?:plus)` | **+9,2 %** | −3,8 % | −0,7 % |
  | … + pré-filtre `(?:minus)$` | **+9,6 %** | −40,0 % | **+13,1 %** |

  Trois enseignements :
  1. **Ancrer le lookahead coûte ~9 % sur les mots courts**, contre toute
     intuition. Le moteur `regex` optimise très agressivement les
     recherches *non ancrées* à préfixe littéral (memchr/Teddy) ; `^(?:…)`
     semble désactiver ce chemin rapide, et sur des haystacks de quelques
     caractères cette optimisation pèse plus que le balayage qu'elle
     évite. L'équivalence sémantique, elle, est exacte : `^(?:p)` matche
     ssi un match de `p` démarre en 0 — c'est un problème de performance,
     pas de justesse.
  2. **Le gain sur les mots longs venait du pré-filtre**, pas de l'ancrage
     (−40 % contre −3,8 %). Mais il *s'ajoute* à la boucle au lieu de la
     remplacer : quand il passe, on paie une recherche complète sur le
     préfixe **puis** la boucle entière. D'où +13 % sur du texte
     ordinaire — le cas courant de cette bibliothèque.
  3. **Remplacer purement la boucle par `(?:minus)$` serait un bug.**
     Vérifié sur les patterns réels : `(e?)` matche le vide (la forme
     ancrée matcherait donc *toujours*), et le `^` de `(^b|cob|cip)`
     désigne dans la boucle le début de la **sous-chaîne** examinée, pas
     celui du mot — sur le préfixe `xb`, la boucle trouve un match, la
     forme ancrée non.

  Conclusion : **le parser est à son bon compromis depuis 0.10.0.** Une
  vraie amélioration devrait *remplacer* la boucle du lookbehind sans
  ancrer le lookahead, et se juger sur `syllabify_text/sentence` — pas sur
  un mot de 25 lettres, qui n'est pas le cas d'usage.
- `jni/src/lib.rs` garde `#[allow(deprecated)]` sur `find_class` /
  `new_object_array` / `set_object_array_element` : la migration vers
  `JObjectArray::<T>::new` / `set_element` est un refactor type-generic
  non-trivial, resté hors-scope du bump jni 0.21 → 0.22.
- `AssembleMode::Lc` porte `#[deprecated]` et sa dérive vis-à-vis de
  LC6 v6 est documentée inline.

Toujours consulter **NOTES-v6.md** avant de modifier le comportement du
parser ou du décodeur — il documente quatre pièges concrets rencontrés
pendant le portage et explique *pourquoi* certaines lignes ont la forme
qu'elles ont.
