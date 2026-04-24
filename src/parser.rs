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

/// Un phonème produit par le parser : (code phonétique, nombre de caractères consommés).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phoneme {
    pub code: String,
    pub step: usize,
}

// Compile every regex pattern that appears in AUTOMATON exactly once, at first parse.
// The resulting HashMap is immutable: concurrent reads need no lock.
fn regex_cache() -> &'static HashMap<&'static str, Regex> {
    use crate::data::RuleKind;
    static CACHE: OnceLock<HashMap<&'static str, Regex>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut map = HashMap::new();
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
                        map.entry(plus)
                            .or_insert_with(|| Regex::new(plus).expect("invalid regex in data.rs"));
                    }
                    if has_minus && !minus.is_empty() {
                        map.entry(minus).or_insert_with(|| {
                            Regex::new(minus).expect("invalid regex in data.rs")
                        });
                    }
                }
            }
        }
        map
    })
}

fn get_regex(pattern: &str) -> Option<&'static Regex> {
    regex_cache().get(pattern)
}

/// Lookup dans l'automate pour une lettre donnée.
fn lookup_letter(letter: char) -> Option<&'static LetterEntry> {
    let s = letter.to_string();
    for (k, entry) in AUTOMATON {
        if *k == s.as_str() {
            return Some(entry);
        }
    }
    None
}

/// Évalue une règle contextuelle : lookahead (`plus`) et lookbehind (`minus`).
///
/// `word` est le mot sous forme de chars.
/// `pos_mot` est la position (1-indexée comme dans le Python : la lettre actuelle est word[pos_mot-1]).
///
/// Reproduit la logique de `Parser.check` : pour `-` qui commence par `^` sans préfixe,
/// test que pos_mot == 1 ; pour `-` qui commence par `^...`, test pattern qui mange tout le préfixe ;
/// pour `-` ordinaire, test que le pattern s'ajuste au bord droit du préfixe (boucle k).
fn check_context(
    plus: &str,
    minus: &str,
    has_plus: bool,
    has_minus: bool,
    word: &[char],
    pos_mot: usize,
) -> bool {
    let mut found_s = true;
    let mut found_p = true;

    // La chaîne suffixe à partir de pos_mot
    let suffix: String = word[pos_mot..].iter().collect();

    if has_plus {
        match get_regex(plus) {
            Some(re) => {
                found_s = re.find(&suffix).is_some_and(|m| m.start() == 0);
            }
            None => found_s = false,
        }
    }

    if has_minus {
        let prefix: String = word[..pos_mot - 1].iter().collect();
        found_p = false;
        if minus.starts_with('^') {
            // ^ = début de chaîne
            if minus.len() == 1 {
                // minus == "^" : match début du mot vide → pos_mot == 1 veut dire lettre en position 0
                found_p = pos_mot == 1;
            } else {
                // minus == "^...": le début du mot doit matcher tout le préfixe
                if let Some(re) = get_regex(minus) {
                    if let Some(mat) = re.find(&prefix) {
                        // IMPORTANT : mat.start()/end() sont en BYTES, comparer à prefix.len() (bytes).
                        found_p = mat.start() == 0 && mat.end() == prefix.len();
                    }
                }
            }
        } else {
            // Pattern sans ^ : on cherche une correspondance qui "finit" au bord droit (= à pos_mot-1)
            // Dans le Python : boucle k de pos_mot-2 descendant vers -1,
            //   pattern.match(mot, k, pos_mot) : le match doit couvrir exactement [k, pos_mot-1]
            if let Some(re) = get_regex(minus) {
                let prefix_len = prefix.chars().count();
                // Tester tous les points de départ possibles
                for k in (0..prefix_len).rev() {
                    // Construire le slice [k, prefix_len]
                    let sub: String = word[k..pos_mot - 1].iter().collect();
                    if let Some(mat) = re.find(&sub) {
                        // IMPORTANT : mat.start()/end() sont en BYTES.
                        // On doit donc comparer aux bytes de sub, pas aux chars.
                        if mat.start() == 0 && mat.end() == sub.len() {
                            found_p = true;
                            break;
                        }
                    }
                }
            }
        }
    }

    found_p && found_s
}

/// Applique une règle spéciale.
fn check_special(sp: Special, word: &[char], pos_mot: usize) -> bool {
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
/// Retour (String vide, 1) signifie "caractère non décodable", on avance d'un cran.
fn one_step(word: &[char], pos: usize) -> Phoneme {
    let letter = word[pos];
    let entry = match lookup_letter(letter) {
        Some(e) => e,
        None => {
            return Phoneme {
                code: String::new(),
                step: 1,
            }
        }
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
                code: rule.phoneme.to_string(),
                step: rule.step,
            };
        }
    }

    // Fin de mot : règle '@'
    if pos == word.len() - 1 {
        if let Some((phon, step)) = entry.end_of_word {
            return Phoneme {
                code: phon.to_string(),
                step,
            };
        }
    }

    // Règle par défaut '*'
    if let Some((phon, step)) = entry.default {
        return Phoneme {
            code: phon.to_string(),
            step,
        };
    }

    // Rien trouvé : caractère non décodable
    Phoneme {
        code: String::new(),
        step: 1,
    }
}

/// Décode un mot en suite de phonèmes.
pub fn parse(word: &str) -> Vec<Phoneme> {
    let chars: Vec<char> = word.chars().collect();
    let mut code: Vec<Phoneme> = Vec::new();
    let mut pos = 0;

    // Note v6 : le dictionnaire d'exceptions explicite (metz, zeus, ouranos…)
    // a été supprimé ; les cas sont désormais gérés par des règles d'automate
    // ou par le mécanisme `HOMOGRAPHES` (utilisé au niveau texte).

    while pos < chars.len() {
        let ph = one_step(&chars, pos);
        pos += ph.step;
        code.push(ph);
    }

    code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_chat() {
        let ph = parse("chat");
        let codes: Vec<&str> = ph.iter().map(|p| p.code.as_str()).collect();
        assert_eq!(codes, &["s^", "a", "#"]);
    }

    #[test]
    fn parse_ecole() {
        let ph = parse("école");
        let codes: Vec<&str> = ph.iter().map(|p| p.code.as_str()).collect();
        // é c o l e → e, k, o, l, q_caduc
        assert_eq!(codes, &["e", "k", "o", "l", "q_caduc"]);
    }
}
