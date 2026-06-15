// SPDX-License-Identifier: GPL-3.0-or-later
//! Property tests : invariants vérifiés sur des entrées générées aléatoirement.
//!
//! Complément à `tests/regression.rs` (oracle 4830 mots) et `tests/edge_cases.rs`
//! (cas limites énumérés). On vérifie ici que l'API publique survit à des
//! entrées arbitraires sans paniquer, et que les invariants structurels tiennent.

use proptest::prelude::*;
use syllabify_fr::{
    liaison_amont, liaison_aval, liaison_possible, phonemes, render_html, render_word_html,
    syllabify_text, syllables, TextChunk,
};

/// Stratégie : mots français réalistes (alphabet latin + accents fréquents).
fn realistic_word() -> impl Strategy<Value = String> {
    proptest::string::string_regex("[a-zàâäéèêëîïôöùûüÿœæç]{1,30}").unwrap()
}

/// Stratégie : phrases avec espaces et ponctuation.
fn realistic_text() -> impl Strategy<Value = String> {
    proptest::string::string_regex("[a-zàâäéèêëîïôöùûüÿœæç '.,!?-]{0,200}").unwrap()
}

proptest! {
    /// `syllables` ne panique jamais sur un mot d'alphabet français raisonnable.
    #[test]
    fn syllables_never_panics_on_realistic_word(w in realistic_word()) {
        let _ = syllables(&w);
    }

    /// Aucune syllabe vide produite pour un mot non vide d'alphabet français.
    #[test]
    fn syllables_no_empty_syllable_for_nonempty_word(w in realistic_word()) {
        let result = syllables(&w);
        for syl in &result {
            prop_assert!(!syl.is_empty(), "syllable vide trouvée dans {:?}: {:?}", w, result);
        }
    }

    /// Conservation des lettres : pour un mot ASCII alphabétique, la concaténation
    /// des syllabes a le même nombre de caractères que le mot d'entrée
    /// (l'API préserve la casse, cf. `tests/edge_cases.rs::uppercase_preserves_case_in_output`).
    #[test]
    fn syllables_preserve_letter_count_for_ascii(w in "[a-z]{1,30}") {
        let result = syllables(&w);
        let joined: String = result.iter().flat_map(|s| s.chars()).collect();
        prop_assert_eq!(joined.chars().count(), w.chars().count(),
            "longueur changée pour {:?}: {:?}", w, result);
    }

    /// **Invariant `lower()` français** : `cleaner::clean` utilise `char::to_lowercase`
    /// qui peut multi-étendre (ex : `İ` turc → `i\u{307}`, `ß` allemand → `ss`).
    /// Sur l'alphabet réellement décrit par `AUTOMATON` (a-z + à â ç è é ê ë î ï
    /// ô ö ù û œ), `to_lowercase` est 1-pour-1 et chaque caractère a une règle —
    /// l'hypothèse implicite portée par `decoder::extract_phonemes_word` tient.
    ///
    /// Note : `ä`, `ÿ`, `æ`, `ü` ne sont **pas** dans `AUTOMATON` et seraient
    /// perdus à la syllabation (cf. `tests/edge_cases.rs::chars_outside_automaton_are_dropped`).
    #[test]
    fn syllables_preserve_letter_count_for_french(
        w in "[a-zàâçèéêëîïôöùûœ]{1,30}"
    ) {
        let result = syllables(&w);
        let joined: String = result.iter().flat_map(|s| s.chars()).collect();
        prop_assert_eq!(joined.chars().count(), w.chars().count(),
            "longueur changée pour {:?}: {:?}", w, result);
    }

    /// Idem en casse mixte : `char::to_lowercase` doit aussi être 1-pour-1 sur
    /// les majuscules françaises de l'alphabet `AUTOMATON` (ex : `É` → `é`).
    #[test]
    fn syllables_preserve_letter_count_for_mixed_case_french(
        w in "[a-zA-ZàâçèéêëîïôöùûœÀÂÇÈÉÊËÎÏÔÖÙÛŒ]{1,30}"
    ) {
        let result = syllables(&w);
        let joined: String = result.iter().flat_map(|s| s.chars()).collect();
        prop_assert_eq!(joined.chars().count(), w.chars().count(),
            "longueur changée pour {:?}: {:?}", w, result);
    }

    /// **Invariant `step`** : la somme des longueurs des `letters` retournées
    /// par `phonemes()` égale le nombre de caractères du mot d'entrée
    /// (sur l'alphabet `AUTOMATON`).
    ///
    /// Concrètement : chaque phonème consomme `step` caractères, la concaténation
    /// des `letters` reconstruit le mot. Documentation : `parser::Phoneme.step`.
    #[test]
    fn phonemes_letters_partition_input_length(
        w in "[a-zàâçèéêëîïôöùûœ]{1,30}"
    ) {
        let pairs = phonemes(&w);
        let total: usize = pairs.iter().map(|(_, letters)| letters.chars().count()).sum();
        prop_assert_eq!(total, w.chars().count(),
            "somme des step != nb chars pour {:?}: {:?}", w, pairs);
    }

    /// `syllables` ne panique jamais sur n'importe quelle `String` UTF-8 valide.
    /// C'est le test "adversarial" : émojis, contrôles, surrogates, etc.
    #[test]
    fn syllables_never_panics_on_arbitrary_utf8(s in any::<String>()) {
        let _ = syllables(&s);
    }

    /// Mots longs : on cap à 100 chars. Le parser semble être en O(n²)
    /// ou pire sur certaines règles à cause du regex cache et des
    /// lookaheads ; au-delà ça devient trop coûteux pour un test
    /// lancé à chaque `cargo test`. Le fuzz couvrira les cas extrêmes.
    #[test]
    fn syllables_never_panics_on_long_word(w in "[a-z]{50,100}") {
        let _ = syllables(&w);
    }

    /// `phonemes` ne panique jamais sur entrée arbitraire.
    #[test]
    fn phonemes_never_panics(s in any::<String>()) {
        let _ = phonemes(&s);
    }

    /// `syllabify_text` ne panique jamais et produit toujours un résultat structurellement cohérent.
    #[test]
    fn syllabify_text_never_panics(s in realistic_text()) {
        let chunks = syllabify_text(&s);
        for chunk in &chunks {
            if let TextChunk::Word(syls) = chunk {
                for syl in syls {
                    prop_assert!(!syl.is_empty(), "syllable vide dans {:?}: {:?}", s, chunks);
                }
            }
        }
    }

    /// `syllabify_text` adversarial.
    #[test]
    fn syllabify_text_never_panics_on_arbitrary_utf8(s in any::<String>()) {
        let _ = syllabify_text(&s);
    }

    /// Les trois prédicats de liaison ne paniquent jamais.
    #[test]
    fn liaison_predicates_never_panic(a in any::<String>(), b in any::<String>()) {
        let _ = liaison_amont(&a);
        let _ = liaison_aval(&a);
        let _ = liaison_possible(&a, &b);
    }

    /// Le rendu HTML ne panique jamais et produit toujours un balisage qui se ferme.
    #[test]
    fn render_word_html_never_panics_and_balanced(w in realistic_word()) {
        let html = render_word_html(&w);
        let opens = html.matches("<span").count();
        let closes = html.matches("</span>").count();
        prop_assert_eq!(opens, closes, "balises non équilibrées pour {:?}: {}", w, html);
    }

    #[test]
    fn render_html_never_panics_on_arbitrary_utf8(s in any::<String>()) {
        let _ = render_html(&s);
    }

    /// XSS : tout `<` dans l'entrée doit être échappé (`&lt;`) dans la sortie.
    /// On utilise une stratégie qui injecte du HTML brut entre des mots.
    #[test]
    fn render_html_escapes_angle_brackets(s in "[a-z]+ <script>alert\\(1\\)</script> [a-z]+") {
        let html = render_html(&s);
        prop_assert!(!html.contains("<script>"),
            "balise <script> non échappée pour {:?}: {}", s, html);
    }
}
