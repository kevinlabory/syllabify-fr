// SPDX-License-Identifier: GPL-3.0-or-later
//! Possibilités de liaison inter-mots. Port de `liaisonAmont` / `liaisonAval`
//! de LireCouleur 6 (`module.js` ll. 1041-1057).
//!
//! Deux prédicats purs, sans transformation phonologique :
//!
//! - [`liaison_amont`] : le mot *peut-il recevoir* une liaison ?
//! - [`liaison_aval`] : le mot *déclenche-t-il* une liaison ?
//!
//! Un helper [`liaison_possible`] combine les deux et donne la réponse usuelle.
//!
//! ```
//! use syllabify_fr::{liaison_amont, liaison_aval, liaison_possible};
//!
//! assert!(liaison_aval("les"));
//! assert!(liaison_amont("hôtel"));        // h muet
//! assert!(!liaison_amont("homard"));      // h aspiré
//! assert!(liaison_possible("les", "hôtels"));
//! ```

use crate::data::LIAISONS_AVAL;
use crate::decoder::{extract_phonemes_word, SyllableMode};
use crate::phoneme::{classify, PhonClass};

/// Le mot *peut-il recevoir* une liaison en amont ?
///
/// Vrai si son premier phonème est une voyelle ou `#_h_muet` (h muet).
/// Faux pour un h aspiré (phonème `#`) ou une consonne initiale.
pub fn liaison_amont(word: &str) -> bool {
    let phonemes = extract_phonemes_word(word, false, SyllableMode::Written);
    match phonemes.first() {
        Some(p) if p.code == "#_h_muet" => true,
        Some(p) => classify(&p.code) == PhonClass::Vowel,
        None => false,
    }
}

/// Le mot *déclenche-t-il* une liaison en aval ?
///
/// Vrai s'il appartient à la liste fermée LC6 (40 mots : déterminants,
/// pronoms sujets, adverbes, prépositions, numéraux). Insensible à la casse.
pub fn liaison_aval(word: &str) -> bool {
    let lower: String = word.chars().flat_map(|c| c.to_lowercase()).collect();
    LIAISONS_AVAL.iter().any(|w| *w == lower)
}

/// Y a-t-il possibilité de liaison entre `previous` et `next` ?
///
/// Équivalent à `liaison_aval(previous) && liaison_amont(next)`.
pub fn liaison_possible(previous: &str, next: &str) -> bool {
    liaison_aval(previous) && liaison_amont(next)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amont_h_muet() {
        assert!(liaison_amont("hôtel"));
        assert!(liaison_amont("homme"));
    }

    #[test]
    fn amont_h_aspire() {
        assert!(!liaison_amont("homard"));
        assert!(!liaison_amont("haricot"));
        assert!(!liaison_amont("héros"));
    }

    #[test]
    fn amont_voyelle_initiale() {
        assert!(liaison_amont("arbre"));
        assert!(liaison_amont("enfant"));
        assert!(liaison_amont("œuf"));
    }

    #[test]
    fn amont_consonne_initiale() {
        assert!(!liaison_amont("chat"));
        assert!(!liaison_amont("papa"));
    }

    #[test]
    fn amont_semi_voyelle_initiale() {
        // 'hier' commence par yod (semi-voyelle) : pas de liaison
        assert!(!liaison_amont("hier"));
    }

    #[test]
    fn aval_liste() {
        assert!(liaison_aval("les"));
        assert!(liaison_aval("des"));
        assert!(liaison_aval("trois"));
        assert!(liaison_aval("sans"));
    }

    #[test]
    fn aval_case_insensible() {
        assert!(liaison_aval("LES"));
        assert!(liaison_aval("Les"));
    }

    #[test]
    fn aval_hors_liste() {
        assert!(!liaison_aval("chat"));
        assert!(!liaison_aval("grand"));
    }

    #[test]
    fn aval_liste_complete_40_mots() {
        // Garde-fou : la liste LC6 contient exactement 40 mots.
        // Si ce test casse, c'est que la régénération a introduit un écart à confirmer.
        assert_eq!(LIAISONS_AVAL.len(), 40);
    }

    #[test]
    fn possible_cas_canoniques() {
        assert!(liaison_possible("les", "hôtels"));
        assert!(liaison_possible("des", "enfants"));
        assert!(liaison_possible("trois", "amis"));
    }

    #[test]
    fn possible_bloque_par_amont() {
        // 'les' déclenche bien, mais 'chats' commence par consonne
        assert!(!liaison_possible("les", "chats"));
        // 'les' déclenche, mais 'héros' commence par h aspiré
        assert!(!liaison_possible("les", "héros"));
    }

    #[test]
    fn possible_bloque_par_aval() {
        // 'le' n'est pas dans la liste de liaison (pas de consonne finale liable)
        assert!(!liaison_possible("le", "hôtel"));
        // 'chat' n'est pas un mot de liaison
        assert!(!liaison_possible("chat", "hôtels"));
    }
}
