// SPDX-License-Identifier: GPL-3.0-or-later
//! Prétraitement de texte.
//!
//! - Minuscules
//! - Apostrophes (`'`, `´`, `'`) → `@` (marqueur de liaison élidée)
//! - `\r` traité comme ponctuation (un caractère de sortie par caractère d'entrée)
//! - Tirets `-` et underscores `_` préservés (transparents au parser, comme LC6)
//! - Autres caractères non significatifs → espace

/// Caractères retenus : alphabet latin + lettres françaises accentuées + `@` + `-` + `_`.
fn est_significatif(c: char) -> bool {
    matches!(c,
        'a'..='z' | 'A'..='Z' | '@' | '-' | '_'
        | 'à' | 'ä' | 'â' | 'é' | 'è' | 'ê' | 'ë'
        | 'î' | 'ï' | 'ô' | 'ö' | 'û' | 'ù' | 'ç' | 'œ'
    )
}

/// Normalise un caractère déjà passé en minuscules.
fn normalize(lower: char, substitute: char) -> char {
    match lower {
        '\'' | '´' | '\u{2019}' => '@',
        c if est_significatif(c) => c,
        _ => substitute,
    }
}

/// Nettoie le texte pour analyse.
///
/// Garantit **un caractère de sortie par caractère d'entrée** : les indices du
/// résultat restent valides dans `text`. `decoder::extract_syllables` en a
/// besoin pour redécouper le texte d'origine aux positions trouvées dans le
/// texte nettoyé. Pour cela, `\r` devient `substitute` (il était supprimé
/// auparavant) et seul le premier caractère de `to_lowercase` est gardé
/// (`İ` donne `i`, pas `i` + U+0307).
pub fn clean(text: &str, substitute: char) -> String {
    text.chars()
        .map(|ch| {
            // Invariant : `to_lowercase` rend au moins un caractère ; on garde le premier.
            let lower = ch.to_lowercase().next().unwrap_or(ch);
            normalize(lower, substitute)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minuscules() {
        assert_eq!(clean("CHAT", ' '), "chat");
    }

    #[test]
    fn apostrophes() {
        assert_eq!(clean("l'école", ' '), "l@école");
        assert_eq!(clean("l\u{2019}école", ' '), "l@école");
    }

    #[test]
    fn ponctuation_devient_espace() {
        assert_eq!(clean("chat, chien.", ' '), "chat  chien ");
    }

    #[test]
    fn garde_un_caractere_par_caractere() {
        for t in ["chat\r\nchien", ",o~İ@", "ÉCOLE İstanbul"] {
            assert_eq!(clean(t, ' ').chars().count(), t.chars().count());
        }
    }

    #[test]
    fn retour_chariot_devient_espace() {
        assert_eq!(clean("chat\r\nchien", ' '), "chat  chien");
    }

    #[test]
    fn accents_preserves() {
        assert_eq!(clean("élève", ' '), "élève");
    }

    #[test]
    fn tiret_preserve() {
        // Le tiret reste dans le texte (pour les mots composés style "grand-père")
        assert_eq!(clean("grand-père", ' '), "grand-père");
    }
}
