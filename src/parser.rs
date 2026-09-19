// SPDX-License-Identifier: GPL-3.0-or-later
//! Parser : transforme un mot en suite de phonèmes.
//! Équivalent de `parser.py` dans pylirecouleur.

use crate::data::{LetterEntry, RuleKind, Special, AUTOMATON};
use crate::rules;
// Features are additive in Cargo workspaces, so if both are unified we pick
// regex-full (the native default). Only a WASM build with default-features=false
// + features=["regex-lite"] actually hits the regex-lite path.
#[cfg(feature = "regex-full")]
use regex::Regex;
#[cfg(all(feature = "regex-lite", not(feature = "regex-full")))]
use regex_lite::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;

#[cfg(not(any(feature = "regex-full", feature = "regex-lite")))]
compile_error!("one of the features `regex-full` or `regex-lite` must be enabled");

/// Un phonème produit par le parser, avant les post-traitements du décodeur.
///
/// `code` est l'étiquette phonétique LC6 (ex : `"a"`, `"s^"` pour /ʃ/, `"#_h_muet"`).
/// `step` est le nombre de caractères consommés du mot d'entrée par cette
/// règle de l'automate (1 pour `'a'`, 2 pour `"ch"`, 3 pour `"ill"` selon le
/// contexte). La somme des `step` égale toujours le nombre de caractères
/// du mot après nettoyage par [`crate::cleaner`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phoneme {
    /// Code phonétique LC6. Tous les codes sont des littéraux : ceux de
    /// `data.rs` pour les règles de l'automate, ceux de `decoder.rs` pour
    /// les réécritures des post-traitements. Aucun n'est construit à
    /// l'exécution — d'où `&'static str`, plutôt qu'un `Cow` dont la
    /// variante `Owned` n'était jamais produite.
    pub code: &'static str,
    /// Nombre de caractères consommés du mot d'entrée.
    pub step: usize,
}

/// Les regex de l'automate, compilées une fois sous les formes dont le parser
/// a besoin. Les maps sont immuables une fois construites : les lectures
/// concurrentes n'ont besoin d'aucun verrou.
struct Regexes {
    /// `^(?:plus)` — le lookahead ne s'intéresse qu'à un match démarrant
    /// exactement au début du suffixe. Ancrer évite au moteur de balayer tout
    /// le suffixe à la recherche d'un match plus loin, qu'on rejetterait.
    start: HashMap<&'static str, Regex>,
    /// `minus` tel quel — nécessaire pour vérifier qu'un match couvre
    /// *exactement* une sous-chaîne donnée.
    ///
    /// Pas de forme ancrée ici. Un pré-filtre `(?:minus)$` a été implémenté
    /// puis retiré : mesuré, il coûtait plus qu'il ne rapportait sur les mots
    /// courts — le cas courant — parce qu'il s'ajoute à la boucle au lieu de
    /// la remplacer (cf. CHANGELOG 0.10.1).
    raw: HashMap<&'static str, Regex>,
}

fn regexes() -> &'static Regexes {
    static CACHE: OnceLock<Regexes> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut start = HashMap::new();
        let mut raw = HashMap::new();
        for (_, entry) in AUTOMATON {
            for rule in entry.rules {
                if let RuleKind::Context {
                    plus,
                    minus,
                    has_plus,
                    has_minus,
                } = rule.kind
                {
                    if has_plus && !plus.is_empty() {
                        start.entry(plus).or_insert_with(|| {
                            Regex::new(&format!("^(?:{plus})")).expect("invalid regex in data.rs")
                        });
                    }
                    if has_minus && !minus.is_empty() {
                        raw.entry(minus).or_insert_with(|| {
                            Regex::new(minus).expect("invalid regex in data.rs")
                        });
                    }
                }
            }
        }
        Regexes { start, raw }
    })
}

// Index char → LetterEntry built once at first parse, replacing a linear scan
// over AUTOMATON's 41 entries on every letter.
fn letter_index() -> &'static HashMap<char, &'static LetterEntry> {
    static IDX: OnceLock<HashMap<char, &'static LetterEntry>> = OnceLock::new();
    IDX.get_or_init(|| {
        let mut map = HashMap::with_capacity(AUTOMATON.len());
        for (k, entry) in AUTOMATON {
            if let Some(c) = k.chars().next() {
                map.insert(c, entry);
            }
        }
        map
    })
}

/// Lookup dans l'automate pour une lettre donnée.
fn lookup_letter(letter: char) -> Option<&'static LetterEntry> {
    letter_index().get(&letter).copied()
}

/// Mot en cours d'analyse : le texte et son index caractère → octet.
///
/// Construit une seule fois par [`parse`]. L'automate et les règles
/// (`rules.rs`) raisonnent en positions de **caractères** (sémantique
/// Python/LC6 : `pos_mot` 1-indexé), mais les regex consomment des `&str`.
/// La table `offsets` permet de découper `text` en `&str` sans allocation
/// via [`Word::slice`] — c'est ce qui rend le hot path du parser zero-alloc.
#[derive(Debug)]
pub struct Word<'a> {
    text: &'a str,
    chars: Vec<char>,
    /// `offsets[i]` = octet de début du caractère `i`, plus une sentinelle
    /// finale `offsets[len] == text.len()` pour que `slice(i, len)` soit valide.
    offsets: Vec<usize>,
}

impl<'a> Word<'a> {
    pub fn new(text: &'a str) -> Self {
        let mut chars = Vec::with_capacity(text.len());
        let mut offsets = Vec::with_capacity(text.len() + 1);
        for (i, c) in text.char_indices() {
            offsets.push(i);
            chars.push(c);
        }
        offsets.push(text.len());
        Self {
            text,
            chars,
            offsets,
        }
    }

    /// Nombre de caractères.
    pub fn len(&self) -> usize {
        self.chars.len()
    }

    /// Le mot entier.
    pub fn text(&self) -> &'a str {
        self.text
    }

    /// Les caractères, pour l'indexation par position.
    pub fn chars(&self) -> &[char] {
        &self.chars
    }

    /// Sous-chaîne couvrant les caractères `[start, end)`, sans allocation.
    pub fn slice(&self, start: usize, end: usize) -> &'a str {
        &self.text[self.offsets[start]..self.offsets[end]]
    }
}

/// Évalue une règle contextuelle : lookahead (`plus`) et lookbehind (`minus`).
///
/// `pos_mot` est la position 1-indexée comme dans le Python : la lettre
/// actuelle est `word.chars()[pos_mot - 1]`.
///
/// Reproduit la logique de `Parser.check` : pour `-` qui commence par `^` sans
/// préfixe, test que `pos_mot == 1` ; pour `-` qui commence par `^...`, test
/// pattern qui mange tout le préfixe ; pour `-` ordinaire, test que le pattern
/// s'ajuste au bord droit du préfixe (boucle k).
///
/// Les regex reçoivent des sous-chaînes de `word` obtenues par
/// [`Word::slice`] : aucune allocation, et exactement les mêmes chaînes que
/// l'implémentation historique (qui les reconstruisait depuis `&[char]`).
fn check_context(
    plus: &str,
    minus: &str,
    has_plus: bool,
    has_minus: bool,
    word: &Word,
    pos_mot: usize,
) -> bool {
    let cache = regexes();
    let mut found_s = true;
    let mut found_p = true;

    if has_plus {
        let suffix = word.slice(pos_mot, word.len());
        // `^(?:plus)` matche ⟺ un match de `plus` démarre en 0, c'est-à-dire
        // exactement ce que testait `find(suffix).start() == 0`. Le
        // leftmost-first décide *quel* match est rendu, jamais s'il en existe
        // un en position 0 : l'équivalence tient pour tous les patterns.
        found_s = cache.start.get(plus).is_some_and(|re| re.is_match(suffix));
    }

    if has_minus {
        let prefix = word.slice(0, pos_mot - 1);
        found_p = false;
        if minus.starts_with('^') {
            if minus.len() == 1 {
                // minus == "^" : début du mot vide → la lettre est en position 0
                found_p = pos_mot == 1;
            } else if let Some(re) = cache.raw.get(minus) {
                // minus == "^..." : le début du mot doit matcher tout le préfixe.
                // mat.start()/end() sont en OCTETS, comme prefix.len().
                if let Some(mat) = re.find(prefix) {
                    found_p = mat.start() == 0 && mat.end() == prefix.len();
                }
            }
        } else if let Some(re) = cache.raw.get(minus) {
            // Pattern sans ^ : on cherche une correspondance qui « finit » au bord
            // droit du préfixe. Python : boucle k de pos_mot-2 descendant vers -1,
            // pattern.match(mot, k, pos_mot) → le match doit couvrir [k, pos_mot-1].
            //
            for k in (0..pos_mot - 1).rev() {
                let sub = word.slice(k, pos_mot - 1);
                if let Some(mat) = re.find(sub) {
                    if mat.start() == 0 && mat.end() == sub.len() {
                        found_p = true;
                        break;
                    }
                }
            }
        }
    }

    found_p && found_s
}

/// Applique une règle spéciale.
fn check_special(sp: Special, word: &Word, pos_mot: usize) -> bool {
    match sp {
        Special::RegleIent => rules::regle_ient(word, pos_mot),
        Special::RegleMotsEnt => rules::regle_mots_ent(word, pos_mot),
        Special::RegleMent => rules::regle_ment(word, pos_mot),
        Special::RegleVerbeMer => rules::regle_verbe_mer(word, pos_mot),
        Special::RegleEr => rules::regle_er(word, pos_mot),
        Special::RegleNcAiFinal => rules::regle_nc_ai_final(word, pos_mot),
        Special::RegleAvoir => rules::regle_avoir(word, pos_mot),
        Special::RegleSFinal => rules::regle_s_final(word, pos_mot),
        Special::RegleTFinal => rules::regle_t_final(word, pos_mot),
        Special::RegleTien => rules::regle_tien(word, pos_mot),
    }
}

/// Une étape : retourne le phonème produit et le nombre de caractères consommés.
/// Retour (code vide, 1) signifie « caractère non décodable », on avance d'un cran.
fn one_step(word: &Word, pos: usize) -> Phoneme {
    let letter = word.chars()[pos];
    let Some(entry) = lookup_letter(letter) else {
        return Phoneme { code: "", step: 1 };
    };

    for rule in entry.rules {
        let applies = match rule.kind {
            RuleKind::Context {
                plus,
                minus,
                has_plus,
                has_minus,
            } => check_context(plus, minus, has_plus, has_minus, word, pos + 1),
            RuleKind::Special(sp) => check_special(sp, word, pos + 1),
        };
        if applies {
            return Phoneme {
                code: rule.phoneme,
                step: rule.step,
            };
        }
    }

    // Fin de mot : règle '@'
    if pos == word.len() - 1 {
        if let Some((phon, step)) = entry.end_of_word {
            return Phoneme { code: phon, step };
        }
    }

    // Règle par défaut '*'
    if let Some((phon, step)) = entry.default {
        return Phoneme { code: phon, step };
    }

    // Rien trouvé : caractère non décodable
    Phoneme { code: "", step: 1 }
}

/// Décode un mot en suite de phonèmes.
pub fn parse(word: &str) -> Vec<Phoneme> {
    let word = Word::new(word);
    let mut code: Vec<Phoneme> = Vec::new();
    let mut pos = 0;

    // Note v6 : le dictionnaire d'exceptions explicite (metz, zeus, ouranos…)
    // a été supprimé ; les cas sont désormais gérés par des règles d'automate
    // ou par le mécanisme `HOMOGRAPHES` (utilisé au niveau texte).

    while pos < word.len() {
        let ph = one_step(&word, pos);
        pos += ph.step;
        code.push(ph);
    }

    code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_slice_is_char_indexed_and_zero_copy() {
        let w = Word::new("école");
        assert_eq!(w.len(), 5);
        assert_eq!(w.slice(0, 1), "é");
        assert_eq!(w.slice(1, 5), "cole");
        assert_eq!(w.slice(5, 5), "");
        assert_eq!(w.slice(0, 5), w.text());
        assert_eq!(Word::new("").len(), 0);
    }

    /// Corpus de sous-chaînes pour les tests d'équivalence : fragments réels
    /// de mots français, plus les cas limites qui piègent le leftmost-first
    /// (`"ae"` pour `(e?)`, `"xb"` pour `(^b|cob|cip)`).
    const EQUIV_CORPUS: &[&str] = &[
        "",
        "a",
        "e",
        "s",
        "x",
        "b",
        "ae",
        "xb",
        "ent",
        "ents",
        "ient",
        "ment",
        "ill",
        "ille",
        "tion",
        "oi",
        "oin",
        "eau",
        "eu",
        "er",
        "ien",
        "hier",
        "chocolat",
        "famille",
        "parlent",
        "prudent",
        "œuf",
        "école",
        "élève",
        "monsieur",
        "tranquille",
        "anticonstitutionnellement",
    ];

    /// Invariant de l'ancrage (0.10.1) : `^(?:p)` matche **si et seulement si**
    /// le match leftmost de `p` démarre en 0 — ce que testait l'ancien
    /// `find(suffix).start() == 0`. Le leftmost-first décide *quel* match est
    /// rendu, jamais s'il en existe un en position 0, donc l'équivalence vaut
    /// pour tous les patterns, `$` compris. Vérifié ici exhaustivement sur les
    /// patterns `plus` réels de l'automate.
    #[test]
    fn anchored_lookahead_is_equivalent_to_legacy_find() {
        for (_, entry) in AUTOMATON {
            for rule in entry.rules {
                let RuleKind::Context { plus, has_plus, .. } = rule.kind else {
                    continue;
                };
                if !has_plus || plus.is_empty() {
                    continue;
                }
                let legacy = Regex::new(plus).expect("pattern invalide dans data.rs");
                let anchored = Regex::new(&format!("^(?:{plus})")).expect("ancrage invalide");
                for s in EQUIV_CORPUS {
                    assert_eq!(
                        anchored.is_match(s),
                        legacy.find(s).is_some_and(|m| m.start() == 0),
                        "divergence d'ancrage : pattern={plus:?} input={s:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn parse_chat() {
        let ph = parse("chat");
        let codes: Vec<&str> = ph.iter().map(|p| p.code).collect();
        assert_eq!(codes, &["s^", "a", "#"]);
    }

    #[test]
    fn parse_ecole() {
        let ph = parse("école");
        let codes: Vec<&str> = ph.iter().map(|p| p.code).collect();
        // é c o l e → e, k, o, l, q_caduc
        assert_eq!(codes, &["e", "k", "o", "l", "q_caduc"]);
    }
}
