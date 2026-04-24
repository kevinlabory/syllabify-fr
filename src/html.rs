// SPDX-License-Identifier: GPL-3.0-or-later
//! Rendu HTML avec balises `<span>` autour de chaque syllabe, destiné à
//! l'intégration web (coloriage syllabique pédagogique type dyscolor.com).
//!
//! Conventions :
//!
//! - chaque syllabe → `<span class="syl syl-a">…</span>` ou `syl-b`, alternées
//!   à l'intérieur de chaque mot (la première syllabe de chaque mot est toujours `syl-a`) ;
//! - chaque mot est enveloppé par `<span class="word">…</span>` ;
//! - le texte brut (espaces, ponctuation) est conservé et échappé ;
//! - une liaison possible entre deux mots adjacents (séparés uniquement par
//!   de l'espace) est matérialisée par `<span class="liaison" data-with="z"></span>`
//!   inséré entre les deux mots, où `data-with` est la consonne de liaison
//!   inférée de la dernière lettre du mot précédent.
//!
//! Le HTML produit est auto-suffisant : à la CSS du consommateur de définir
//! les styles sur `.syl-a`, `.syl-b`, `.word`, `.liaison`.
//!
//! ```
//! use syllabify_fr::{render_word_html, render_html};
//!
//! assert_eq!(
//!     render_word_html("chocolat"),
//!     r#"<span class="word"><span class="syl syl-a">cho</span><span class="syl syl-b">co</span><span class="syl syl-a">lat</span></span>"#
//! );
//!
//! // Liaison détectée entre 'les' et 'hôtels'
//! assert!(render_html("les hôtels").contains(r#"<span class="liaison" data-with="z""#));
//! ```

use crate::decoder::TextChunk;
use crate::{liaison_possible, syllabify_text, syllables};

/// Rend un mot unique en HTML avec syllabes enveloppées par des `<span>` alternés.
///
/// Le mot complet est enveloppé dans `<span class="word">…</span>`.
/// Utilise le mode standard (pédagogique) — `homme` → `hom-me`.
pub fn render_word_html(word: &str) -> String {
    render_word_spans(&syllables(word))
}

/// Rend un texte complet en HTML.
///
/// Les homographes sont désambiguïsés selon le mot précédent (comme
/// `syllabify_text`). Les liaisons possibles entre mots adjacents sont
/// marquées par des spans `<span class="liaison" data-with="…">`.
///
/// Le texte brut entre les mots (espaces, ponctuation) est préservé et
/// échappé HTML. Une liaison n'est émise que si l'intervalle entre deux
/// mots est constitué *uniquement* d'espaces (pas de virgule, point, etc.).
pub fn render_html(text: &str) -> String {
    let chunks = syllabify_text(text);
    let mut out = String::with_capacity(text.len() * 4);
    let mut previous_word_raw: Option<String> = None;

    for chunk in &chunks {
        match chunk {
            TextChunk::Raw(s) => {
                // Si on voit autre chose que des espaces, le contexte de
                // liaison avec le mot précédent est cassé.
                if !s.chars().all(char::is_whitespace) {
                    previous_word_raw = None;
                }
                out.push_str(&escape(s));
            }
            TextChunk::Word(sylls) => {
                let word_raw: String = sylls.concat();
                if let Some(prev) = &previous_word_raw {
                    if liaison_possible(prev, &word_raw) {
                        let consonant = liaison_consonant_for(prev);
                        out.push_str(&format!(
                            r#"<span class="liaison" data-with="{}"></span>"#,
                            consonant
                        ));
                    }
                }
                out.push_str(&render_word_spans(sylls));
                previous_word_raw = Some(word_raw);
            }
        }
    }

    out
}

fn render_word_spans(sylls: &[String]) -> String {
    if sylls.iter().all(|s| s.is_empty()) {
        return String::new();
    }
    let mut s = String::from(r#"<span class="word">"#);
    for (i, syl) in sylls.iter().enumerate() {
        let class = if i % 2 == 0 { "syl syl-a" } else { "syl syl-b" };
        s.push_str(&format!(
            r#"<span class="{}">{}</span>"#,
            class,
            escape(syl)
        ));
    }
    s.push_str("</span>");
    s
}

/// Consonne de liaison pour un mot précédent donné, inférée de sa dernière
/// lettre. Retourne `"z"` par défaut (cas majoritaire : `-s`, `-x`, `-z` et
/// déterminants pluriels), ce qui correspond à la majorité des mots de la
/// liste `LIAISONS_AVAL`.
fn liaison_consonant_for(prev: &str) -> &'static str {
    let last = prev
        .chars()
        .rev()
        .flat_map(|c| c.to_lowercase())
        .next()
        .unwrap_or(' ');
    match last {
        's' | 'x' | 'z' => "z",
        'd' | 't' => "t",
        'n' => "n",
        'p' => "p",
        'r' => "r",
        'g' => "k",
        _ => "z",
    }
}

fn escape(s: &str) -> String {
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
    use super::*;

    #[test]
    fn mot_simple_3_syllabes() {
        let html = render_word_html("chocolat");
        assert_eq!(
            html,
            r#"<span class="word"><span class="syl syl-a">cho</span><span class="syl syl-b">co</span><span class="syl syl-a">lat</span></span>"#
        );
    }

    #[test]
    fn mot_alternance_demarre_a() {
        assert!(render_word_html("famille")
            .starts_with(r#"<span class="word"><span class="syl syl-a">fa</span>"#));
    }

    #[test]
    fn texte_preserve_ponctuation() {
        let html = render_html("le chat,");
        assert!(html.contains(r#">le</span>"#));
        assert!(html.contains(" "));
        assert!(html.ends_with(","));
    }

    #[test]
    fn liaison_les_hotels_emet_span() {
        let html = render_html("les hôtels");
        assert!(
            html.contains(r#"<span class="liaison" data-with="z"></span>"#),
            "liaison 'z' attendue, got: {}",
            html
        );
        // Ordre : mot1, espace, span liaison, mot2
        let pos_first_word = html.find("les").unwrap();
        let pos_liaison = html.find(r#"class="liaison""#).unwrap();
        let pos_second_word = html.find("ô").unwrap();
        assert!(pos_first_word < pos_liaison);
        assert!(pos_liaison < pos_second_word);
    }

    #[test]
    fn liaison_absente_h_aspire() {
        // 'les héros' : h aspiré, pas de liaison
        let html = render_html("les héros");
        assert!(!html.contains(r#"class="liaison""#));
    }

    #[test]
    fn liaison_absente_consonne_initiale() {
        let html = render_html("les chats");
        assert!(!html.contains(r#"class="liaison""#));
    }

    #[test]
    fn liaison_consonne_t_pour_tout() {
        // 'tout' est dans LIAISONS_AVAL et finit par 't' → liaison en 't'
        let html = render_html("tout ami");
        assert!(html.contains(r#"data-with="t""#), "got: {}", html);
    }

    #[test]
    fn liaison_consonne_n_pour_en() {
        // 'en' → liaison en 'n' (denasalisation — attestée en français)
        let html = render_html("en automne");
        assert!(html.contains(r#"data-with="n""#), "got: {}", html);
    }

    #[test]
    fn liaison_bloquee_par_virgule() {
        // 'les, hôtels' : virgule entre les deux → pas de liaison
        let html = render_html("les, hôtels");
        assert!(!html.contains(r#"class="liaison""#), "got: {}", html);
    }

    #[test]
    fn homographes_contexte_respecte() {
        // syllabify_text fait la désambiguïsation ; on vérifie que le rendu
        // l'honore bien : 'couvent' après 'le' = nom (cou-vent), après 'elles' = verbe
        let html_nom = render_html("le couvent");
        let html_verbe = render_html("elles couvent");
        // nom : deux syllabes cou/vent
        assert!(
            html_nom.contains(r#">cou</span><span class="syl syl-b">vent</span>"#),
            "got: {}",
            html_nom
        );
        // verbe : la 2e syllabe 'vent' est prononcée muet, mais la graphie reste 'vent'
        // (la différence est phonétique, pas graphique — donc même rendu textuel).
        assert!(html_verbe.contains(">cou</span>"));
    }

    #[test]
    fn html_echappe_caracteres_speciaux() {
        // L'input contient un caractère à échapper dans du texte brut
        let html = render_html("a < b");
        assert!(html.contains("&lt;"));
    }

    #[test]
    fn texte_vide() {
        assert_eq!(render_html(""), "");
    }

    #[test]
    fn mot_vide_ne_crash_pas() {
        // syllables("") retourne [] → render_word_html retourne ""
        assert_eq!(render_word_html(""), "");
    }
}
