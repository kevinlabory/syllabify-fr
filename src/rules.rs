// SPDX-License-Identifier: GPL-3.0-or-later
//! Règles spéciales utilisées par l'automate : port des fonctions `regle_*` de `parser.py`.
//!
//! Toutes ces règles prennent :
//! - `word` : le mot à analyser, sous forme de [`Word`] (texte + index
//!   caractère → octet, cf. `parser.rs`) — indexation unicode-safe par
//!   caractère **et** découpage en `&str` sans allocation ;
//! - `pos_mot` : position 1-indexée (Python : la lettre actuelle est
//!   `word.chars()[pos_mot - 1]`).
//!
//! Elles renvoient `true` si la règle s'applique (donc que la lettre courante doit produire
//! le phonème spécial associé).
//!
//! Ces règles tournent dans le hot path du parser : les chemins fréquents
//! (tests de suffixe, recherches dans les listes) sont sans allocation. Les
//! seules allocations restantes construisent un pseudo-infinitif et sont
//! confinées aux chemins froids (mots en `-ient` / `-ment`), commentées
//! comme telles.

use crate::data::{
    EXCEPTIONS_FINAL_ER, EXCEPTIONS_FINAL_TIEN, MOTS_ENT, MOTS_S_FINAL, MOTS_T_FINAL,
    POSSIBLES_AVOIR, POSSIBLES_NC_AI_FINAL, VERBES_ENTER, VERBES_IER, VERBES_MER,
};
use crate::parser::Word;

/// Replie un caractère accentué sur sa lettre de base (équivalent `no_accent`).
fn fold_accent(c: char) -> char {
    match c {
        'à' | 'ä' | 'â' => 'a',
        'é' | 'è' | 'ê' | 'ë' | 'œ' => 'e',
        'î' | 'ï' => 'i',
        'ô' | 'ö' => 'o',
        'û' | 'ù' => 'u',
        'ç' => 'c',
        other => other.to_lowercase().next().unwrap_or(other),
    }
}

/// Supprime les accents (équivalent `no_accent`).
fn no_accent(s: &str) -> String {
    s.chars().map(fold_accent).collect()
}

/// Retire le `@` en deuxième position (marqueur d'apostrophe élidée :
/// `l@arbre` → `arbre`). Sans allocation.
fn strip_elision(s: &str) -> &str {
    let mut it = s.char_indices();
    it.next();
    match it.next() {
        Some((_, '@')) => it.next().map_or("", |(i, _)| &s[i..]),
        _ => s,
    }
}

/// Retire le `s` final s'il y en a un. Sans allocation.
fn strip_final_s(s: &str) -> &str {
    s.strip_suffix('s').unwrap_or(s)
}

/// `regle_ient` : le mot se termine par `[consonne]ient` et son infinitif (-ier) est dans `verbes_ier` ?
pub fn regle_ient(word: &Word, pos_mot: usize) -> bool {
    let n = word.len();
    if n < 5 {
        return false;
    }
    let last_five = &word.chars()[n - 5..];
    let consonnes = "bcçdfghjklnmpqrstvwxz";
    if !consonnes.contains(last_five[0])
        || last_five[1] != 'i'
        || last_five[2] != 'e'
        || last_five[3] != 'n'
        || last_five[4] != 't'
    {
        return false;
    }
    // pos_mot doit viser la partie finale
    if pos_mot < n - 4 {
        return false;
    }

    // Allocation justifiée (chemin froid : mots en -[consonne]ient uniquement) :
    // pseudo-infinitif `mot[..-2] + "r"`, testé brut puis désaccentué.
    let mut pseudo = String::with_capacity(word.text().len());
    pseudo.push_str(word.slice(0, n - 2));
    pseudo.push('r');
    if VERBES_IER.binary_search(&pseudo.as_str()).is_ok() {
        return true;
    }
    let pseudo_na = no_accent(&pseudo);
    VERBES_IER.binary_search(&strip_elision(&pseudo_na)).is_ok()
}

/// `regle_mots_ent` : mot se termine par `-ent` muet ?
pub fn regle_mots_ent(word: &Word, pos_mot: usize) -> bool {
    let chars = word.chars();
    let len = word.len();
    // Regex : ^[consonne]ent(s?)$
    if (len == 4 || len == 5)
        && "bcdfghjklmnpqrstvwxz".contains(chars[0])
        && chars[1] == 'e'
        && chars[2] == 'n'
        && chars[3] == 't'
        && (len == 4 || chars[4] == 's')
    {
        return true;
    }

    let text = word.text();
    let (comparateur, comp_len) = text.strip_suffix('s').map_or((text, len), |s| (s, len - 1));

    // Python : pos_mot + 2 < len(comparateur) → return False
    if pos_mot + 2 < comp_len {
        return false;
    }

    let comparateur = strip_elision(comparateur);

    if MOTS_ENT.binary_search(&comparateur).is_ok() {
        return true;
    }
    // Allocation justifiée (chemin froid : mots en -ent hors MOTS_ENT) :
    // concaténation `-er` pour tester l'infinitif.
    let pseudo_verbe = format!("{comparateur}er");
    VERBES_ENTER.binary_search(&pseudo_verbe.as_str()).is_ok()
}

/// `regle_ment` : le mot se termine par `-ment` à prononcer [a~] ?
pub fn regle_ment(word: &Word, pos_mot: usize) -> bool {
    let text = word.text();
    if !text.ends_with("ment") {
        return false;
    }
    let n = word.len();
    if pos_mot < n - 3 {
        return false;
    }

    // Allocation justifiée (chemin froid : mots en -ment uniquement) :
    // pseudo_infinitif = no_accent(mot[:-2] + 'r').
    let mut pseudo = no_accent(word.slice(0, n - 2));
    pseudo.push('r');
    if VERBES_MER.binary_search(&strip_elision(&pseudo)).is_ok() {
        return false;
    }
    // Cas spécial : dorment (verbe dormir)
    if n > 6 && text.ends_with("dorment") {
        return false;
    }
    true
}

/// `regle_verbe_mer` : l'inverse de `regle_ment`.
pub fn regle_verbe_mer(word: &Word, pos_mot: usize) -> bool {
    if !word.text().ends_with("ment") {
        return false;
    }
    if pos_mot < word.len() - 3 {
        return false;
    }
    !regle_ment(word, pos_mot)
}

/// `regle_er` : le mot se termine par -er et n'est pas une exception type "amer", "cher".
pub fn regle_er(word: &Word, _pos_mot: usize) -> bool {
    let m_sing = strip_elision(strip_final_s(word.text()));
    if !m_sing.ends_with("er") {
        return false;
    }
    // Note: le Python original a une logique légèrement ambiguë ici (il retourne True
    // "si dans les exceptions" alors que le commentaire dit "pas une exception"),
    // on respecte strictement le code source.
    EXCEPTIONS_FINAL_ER.binary_search(&m_sing).is_ok()
}

/// `regle_nc_ai_final` : nom commun terminé par -ai prononcé `[è]` plutôt que `[é]`.
pub fn regle_nc_ai_final(word: &Word, pos_mot: usize) -> bool {
    let m_seul = strip_elision(word.text());
    if POSSIBLES_NC_AI_FINAL.binary_search(&m_seul).is_ok() {
        return pos_mot == word.len() - 1;
    }
    false
}

/// `regle_avoir` : forme conjuguée de "avoir" au passé simple / participe passé / subj. imparfait.
pub fn regle_avoir(word: &Word, pos_mot: usize) -> bool {
    if POSSIBLES_AVOIR.binary_search(&word.text()).is_ok() {
        return pos_mot < 2;
    }
    false
}

/// `regle_s_final` : le mot se termine par un `s` qui se prononce.
pub fn regle_s_final(word: &Word, _pos_mot: usize) -> bool {
    let m_seul = strip_elision(word.text());
    MOTS_S_FINAL.binary_search(&m_seul).is_ok()
}

/// `regle_t_final` : le mot se termine par un `t` qui se prononce.
pub fn regle_t_final(word: &Word, _pos_mot: usize) -> bool {
    let m_sing = strip_elision(strip_final_s(word.text()));
    MOTS_T_FINAL.binary_search(&m_sing).is_ok()
}

/// `regle_tien` : le mot se termine par `-tien` où le 't' se prononce `[t]`.
pub fn regle_tien(word: &Word, pos_mot: usize) -> bool {
    let text = word.text();
    let (m_sing, n) = text
        .strip_suffix('s')
        .map_or((text, word.len()), |s| (s, word.len() - 1));
    if n < 4 {
        return false;
    }
    if !m_sing.ends_with("tien") {
        return false;
    }
    if pos_mot < n - 4 {
        return false;
    }
    EXCEPTIONS_FINAL_TIEN.binary_search(&m_sing).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_accent() {
        assert_eq!(no_accent("élève"), "eleve");
        assert_eq!(no_accent("garçon"), "garcon");
        assert_eq!(no_accent("œuf"), "euf");
    }

    #[test]
    fn strip_helpers_are_zero_copy_and_exact() {
        assert_eq!(strip_elision("l@arbre"), "arbre");
        assert_eq!(strip_elision("arbre"), "arbre");
        assert_eq!(strip_elision("l@"), "");
        assert_eq!(strip_elision("a"), "a");
        assert_eq!(strip_elision(""), "");
        assert_eq!(strip_final_s("chats"), "chat");
        assert_eq!(strip_final_s("chat"), "chat");
    }

    #[test]
    fn regle_mots_ent_sur_prudent() {
        // prudent : len 7, -ent → PAS muet (pas verbe, adjectif)
        // MOTS_ENT contient "prudent", "violent", "agent", "moment", etc.
        // pos_mot = len-2 = 5 (position du 'e' de 'ent')
        assert!(regle_mots_ent(&Word::new("prudent"), 5));
    }

    #[test]
    fn regle_mots_ent_sur_parlent() {
        // parlent : verbe 3pp → "ent" muet → regle_mots_ent doit retourner false
        assert!(!regle_mots_ent(&Word::new("parlent"), 5));
    }

    #[test]
    fn regle_ment_et_verbe_mer_sont_complementaires() {
        // "vraiment" : adverbe en -ment → [a~] ; "dorment" : verbe → muet.
        let vraiment = Word::new("vraiment");
        assert!(regle_ment(&vraiment, 6));
        assert!(!regle_verbe_mer(&vraiment, 6));
        let dorment = Word::new("dorment");
        assert!(!regle_ment(&dorment, 5));
        assert!(regle_verbe_mer(&dorment, 5));
    }
}
