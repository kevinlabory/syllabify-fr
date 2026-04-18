// SPDX-License-Identifier: GPL-3.0-or-later
//! Prétraitement de texte.
//!
//! - Minuscules
//! - Apostrophes (`'`, `´`, `'`) → `@` (marqueur de liaison élidée)
//! - Suppression des `\r`
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

/// Nettoie le texte pour analyse.
pub fn clean(text: &str, substitute: char) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        if ch == '\r' {
            continue;
        }
        for lower in ch.to_lowercase() {
            let replaced = match lower {
                '\'' | '´' | '\u{2019}' => '@',
                c if est_significatif(c) => c,
                _ => substitute,
            };
            out.push(replaced);
        }
    }
    out
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
    fn retour_chariot_supprime() {
        assert_eq!(clean("chat\r\nchien", ' '), "chat chien");
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
