// SPDX-License-Identifier: GPL-3.0-or-later
//! Classification des phonèmes en voyelles, consonnes, semi-voyelles.
//! Équivalent de `syllaphon` dans `constant.py`.

/// Classe phonologique d'un phonème.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhonClass {
    /// Voyelle
    Vowel,
    /// Consonne
    Consonant,
    /// Semi-voyelle (yod, etc.)
    SemiVowel,
    /// Silencieux ('#') ou marqueur verbe 3e p. pluriel ('verb_3p')
    Silent,
}

/// Retourne la classe d'un phonème.
/// Les phonèmes composés (`j_a`, `w_o`, `y_i`) sont considérés comme voyelles
/// (ils sont constitués par les post-traitements yod/w).
pub fn classify(phon: &str) -> PhonClass {
    if phon.starts_with("j_") || phon.starts_with("w_") || phon.starts_with("y_") {
        return PhonClass::Vowel;
    }
    match phon {
        // voyelles
        "a" | "q" | "q_caduc" | "i" | "o" | "o_comp" | "o_ouvert" | "u" | "y" | "e" | "e_comp"
        | "e^" | "e^_comp" | "a~" | "e~" | "x~" | "o~" | "x" | "x^" | "wa" | "w5" => {
            PhonClass::Vowel
        }
        // consonnes
        "p" | "t" | "k" | "b" | "d" | "g" | "f" | "f_ph" | "s" | "s^" | "v" | "z" | "z^" | "l"
        | "r" | "m" | "n" | "k_qu" | "z^_g" | "g_u" | "s_c" | "s_t" | "z_s" | "ks" | "gz" => {
            PhonClass::Consonant
        }
        // semi-voyelles
        "j" | "g~" | "n~" | "w" => PhonClass::SemiVowel,
        // marqueurs
        "#" | "#_h_muet" | "verb_3p" => PhonClass::Silent,
        _ => PhonClass::Silent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classes_de_base() {
        assert_eq!(classify("a"), PhonClass::Vowel);
        assert_eq!(classify("p"), PhonClass::Consonant);
        assert_eq!(classify("j"), PhonClass::SemiVowel);
        assert_eq!(classify("#"), PhonClass::Silent);
        assert_eq!(classify("verb_3p"), PhonClass::Silent);
    }

    #[test]
    fn phonemes_composes_sont_voyelles() {
        assert_eq!(classify("j_a"), PhonClass::Vowel);
        assert_eq!(classify("w_a"), PhonClass::Vowel);
        assert_eq!(classify("y_i"), PhonClass::Vowel);
    }
}
