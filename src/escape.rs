// SPDX-License-Identifier: GPL-3.0-or-later
//! Échappement HTML partagé par les deux moteurs de rendu : [`crate::html`]
//! (spans syllabiques) et [`crate::letters`] (confusions de lettres).
//!
//! Couvre les cinq caractères significatifs en HTML5, en contexte texte
//! **comme** en contexte attribut (`& < > " '`). Fait main plutôt que via
//! une dépendance : le jeu est fermé et la fonction tient en dix lignes.

/// Échappe les cinq caractères significatifs en HTML.
pub fn html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::html;

    #[test]
    fn echappe_les_cinq_entites() {
        assert_eq!(
            html(r#"<a href="x">&'</a>"#),
            "&lt;a href=&quot;x&quot;&gt;&amp;&#39;&lt;/a&gt;"
        );
    }

    #[test]
    fn laisse_le_reste_intact() {
        assert_eq!(html("élève — ok"), "élève — ok");
    }
}
