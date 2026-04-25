// SPDX-License-Identifier: GPL-3.0-or-later
//! Règles spéciales utilisées par l'automate : port des fonctions `regle_*` de `parser.py`.
//!
//! Toutes ces règles prennent :
//! - `word`: le mot à analyser, sous forme de chars (pour l'indexation unicode-safe)
//! - `pos_mot`: position 1-indexée (Python : la lettre actuelle est word[pos_mot - 1])
//!
//! Elles renvoient `true` si la règle s'applique (donc que la lettre courante doit produire
//! le phonème spécial associé).

use crate::data::{
    EXCEPTIONS_FINAL_ER, EXCEPTIONS_FINAL_TIEN, MOTS_ENT, MOTS_S_FINAL, MOTS_T_FINAL,
    POSSIBLES_AVOIR, POSSIBLES_NC_AI_FINAL, VERBES_ENTER, VERBES_IER, VERBES_MER,
};

/// Supprime les accents (équivalent `no_accent`).
fn no_accent(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' => 'i',
            'ô' | 'ö' => 'o',
            'û' | 'ù' => 'u',
            'ç' => 'c',
            'œ' => 'e',
            other => other.to_lowercase().next().unwrap_or(other),
        })
        .collect()
}

fn as_string(word: &[char]) -> String {
    word.iter().collect()
}

/// Retire le '@' initial (marqueur d'apostrophe élidée).
fn strip_elision(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() > 1 && chars[1] == '@' {
        chars[2..].iter().collect()
    } else {
        s.to_string()
    }
}

/// Vérifie qu'un mot se termine par `suffix`.
fn ends_with(word: &[char], suffix: &str) -> bool {
    let sc: Vec<char> = suffix.chars().collect();
    if word.len() < sc.len() {
        return false;
    }
    word[word.len() - sc.len()..] == sc[..]
}

/// Retire le `s` final s'il y en a un.
fn strip_final_s(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.last().copied() == Some('s') {
        chars[..chars.len() - 1].iter().collect()
    } else {
        s.to_string()
    }
}

/// `regle_ient` : le mot se termine par `[consonne]ient` et son infinitif (-ier) est dans verbes_ier ?
pub fn regle_ient(word: &[char], pos_mot: usize) -> bool {
    // Le mot se termine-t-il par "[consonne]ient" ?
    if word.len() < 5 {
        return false;
    }
    let last_five = &word[word.len() - 5..];
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
    if pos_mot < word.len() - 4 {
        return false;
    }

    // Construire le pseudo-infinitif : mot[:-2] + 'r'
    let mut pseudo: String = word[..word.len() - 2].iter().collect();
    pseudo.push('r');

    if VERBES_IER.binary_search(&pseudo.as_str()).is_ok() {
        return true;
    }
    let mut pseudo_na = no_accent(&pseudo);
    if pseudo_na.chars().nth(1) == Some('@') {
        pseudo_na = pseudo_na.chars().skip(2).collect();
    }
    VERBES_IER.binary_search(&pseudo_na.as_str()).is_ok()
}

/// `regle_mots_ent` : mot se termine par `-ent` muet ?
pub fn regle_mots_ent(word: &[char], pos_mot: usize) -> bool {
    let mot_str = as_string(word);
    // Regex : ^[consonne]ent(s?)$
    let len = word.len();
    if (len == 4 || len == 5)
        && "bcdfghjklmnpqrstvwxz".contains(word[0])
        && word[1] == 'e'
        && word[2] == 'n'
        && word[3] == 't'
        && (len == 4 || word[4] == 's')
    {
        return true;
    }

    let comparateur = if mot_str.ends_with('s') {
        &mot_str[..mot_str.len() - 1]
    } else {
        &mot_str[..]
    };

    // Python : pos_mot + 2 < len(comparateur) → return False
    let comp_len = comparateur.chars().count();
    if pos_mot + 2 < comp_len {
        return false;
    }

    let comparateur = strip_elision(comparateur);

    if MOTS_ENT.binary_search(&comparateur.as_str()).is_ok() {
        return true;
    }
    let pseudo_verbe = format!("{}er", comparateur);
    VERBES_ENTER.binary_search(&pseudo_verbe.as_str()).is_ok()
}

/// `regle_ment` : le mot se termine par `-ment` à prononcer [a~] ?
pub fn regle_ment(word: &[char], pos_mot: usize) -> bool {
    if !ends_with(word, "ment") {
        return false;
    }
    if pos_mot < word.len() - 3 {
        return false;
    }

    // pseudo_infinitif = no_accent(mot[:-2] + 'r')
    let base: String = word[..word.len() - 2].iter().collect();
    let mut pseudo = no_accent(&format!("{}r", base));
    if pseudo.chars().nth(1) == Some('@') {
        pseudo = pseudo.chars().skip(2).collect();
    }
    if VERBES_MER.binary_search(&pseudo.as_str()).is_ok() {
        return false;
    }
    // Cas spécial : dorment (verbe dormir)
    if word.len() > 6 && ends_with(word, "dorment") {
        return false;
    }
    true
}

/// `regle_verbe_mer` : l'inverse de regle_ment.
pub fn regle_verbe_mer(word: &[char], pos_mot: usize) -> bool {
    if !ends_with(word, "ment") {
        return false;
    }
    if pos_mot < word.len() - 3 {
        return false;
    }
    !regle_ment(word, pos_mot)
}

/// `regle_er` : le mot se termine par -er et n'est pas une exception type "amer", "cher".
pub fn regle_er(word: &[char], _pos_mot: usize) -> bool {
    let mot = as_string(word);
    let m_sing = strip_final_s(&mot);
    let m_sing = strip_elision(&m_sing);
    if !m_sing.ends_with("er") {
        return false;
    }
    // Note: le Python original a une logique légèrement ambiguë ici (il retourne True
    // "si dans les exceptions" alors que le commentaire dit "pas une exception"),
    // on respecte strictement le code source.
    EXCEPTIONS_FINAL_ER.binary_search(&m_sing.as_str()).is_ok()
}

/// `regle_nc_ai_final` : nom commun terminé par -ai prononcé `[è]` plutôt que `[é]`.
pub fn regle_nc_ai_final(word: &[char], pos_mot: usize) -> bool {
    let mot = as_string(word);
    let m_seul = strip_elision(&mot);
    if POSSIBLES_NC_AI_FINAL
        .binary_search(&m_seul.as_str())
        .is_ok()
    {
        return pos_mot == word.len() - 1;
    }
    false
}

/// `regle_avoir` : forme conjuguée de "avoir" au passé simple / participe passé / subj. imparfait.
pub fn regle_avoir(word: &[char], pos_mot: usize) -> bool {
    let mot = as_string(word);
    if POSSIBLES_AVOIR.binary_search(&mot.as_str()).is_ok() {
        return pos_mot < 2;
    }
    false
}

/// `regle_s_final` : le mot se termine par un `s` qui se prononce.
pub fn regle_s_final(word: &[char], _pos_mot: usize) -> bool {
    let mot = as_string(word);
    let m_seul = strip_elision(&mot);
    MOTS_S_FINAL.binary_search(&m_seul.as_str()).is_ok()
}

/// `regle_t_final` : le mot se termine par un `t` qui se prononce.
pub fn regle_t_final(word: &[char], _pos_mot: usize) -> bool {
    let mot = as_string(word);
    let m_sing = strip_final_s(&mot);
    let m_sing = strip_elision(&m_sing);
    MOTS_T_FINAL.binary_search(&m_sing.as_str()).is_ok()
}

/// `regle_tien` : le mot se termine par `-tien` où le 't' se prononce `[t]`.
pub fn regle_tien(word: &[char], pos_mot: usize) -> bool {
    let mot = as_string(word);
    let m_sing = strip_final_s(&mot);
    let chars: Vec<char> = m_sing.chars().collect();
    if chars.len() < 4 {
        return false;
    }
    let last4: String = chars[chars.len() - 4..].iter().collect();
    if last4 != "tien" {
        return false;
    }
    if pos_mot < chars.len() - 4 {
        return false;
    }
    EXCEPTIONS_FINAL_TIEN
        .binary_search(&m_sing.as_str())
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chars(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    #[test]
    fn test_no_accent() {
        assert_eq!(no_accent("élève"), "eleve");
        assert_eq!(no_accent("garçon"), "garcon");
        assert_eq!(no_accent("œuf"), "euf");
    }

    #[test]
    fn test_ends_with() {
        let w = chars("parlent");
        assert!(ends_with(&w, "ent"));
        assert!(!ends_with(&w, "ant"));
    }

    #[test]
    fn regle_mots_ent_sur_prudent() {
        // prudent : len 7, -ent → PAS muet (pas verbe, adjectif)
        // mais MOTS_ENT contient-il "prudent" ?
        // Dans la base : "prudent", "violent", "agent", "moment", etc. devraient y être.
        let w = chars("prudent");
        // pos_mot = len-2 = 5 (position du 'e' de 'ent')
        assert!(regle_mots_ent(&w, 5));
    }

    #[test]
    fn regle_mots_ent_sur_parlent() {
        // parlent : verbe 3pp → "ent" muet → regle_mots_ent doit retourner false
        let w = chars("parlent");
        assert!(!regle_mots_ent(&w, 5));
    }
}
