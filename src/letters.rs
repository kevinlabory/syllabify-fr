// SPDX-License-Identifier: GPL-3.0-or-later
//! Mise en évidence de lettres (ou successions de lettres) dans un mot.
//!
//! Port fidèle de la fonction `lettres` de `LireCouleur` 6
//! (`functionlc6.js:222-296`). Cas d'usage typique : limiter les
//! confusions de lettres (b/d/p/q, m/n/u…) ou différencier des
//! séquences proches comme `pir` / `pri`.
//!
//! ## Vue d'ensemble
//!
//! ```
//! use syllabify_fr::letters::{match_letters, render_letters_html, presets, RenderMode};
//!
//! let rules = presets::bdpq();
//! let spans = match_letters("dépit", &rules);
//! let html = render_letters_html("dépit", &spans, &rules, RenderMode::Inline);
//! assert!(html.contains("color:"));
//! ```
//!
//! ## Algorithme (fidèle LC6)
//!
//! Scan gauche→droite, à chaque position : essayer les règles dans
//! l'ordre **inverse** de leur déclaration (la dernière règle déclarée
//! est *outer*, prend précédence — comportement issu de la récursion
//! LC6 `recToHTML`).
//!
//! Pour les patterns d'**un seul caractère** appartenant à
//! `{a, c, e, i, o, u, y, n}`, l'égalité fait abstraction des
//! diacritiques (`a` matche `à`, `â`, `ä`…), comme LC6. Les autres
//! patterns sont comparés littéralement *case-insensitive*.

use crate::escape;
use std::borrow::Cow;

/// Style typographique appliqué à une règle.
///
/// Mapping vers CSS lors du rendu HTML (`RenderMode::Inline`) :
/// `color/background/bold/italic` → propriétés CSS classiques ;
/// `stroke` → contour façon LC6 (`font-weight:bold; -webkit-text-stroke:0.03em black;`) ;
/// `underline` → `text-decoration:underline`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
// Les 4 booléens (bold/italic/stroke/underline) reflètent 1:1 les flags de
// style LC6 (`functionlc6.js`). Les regrouper en bitflags casserait le mapping
// et l'ergonomie côté binding (presets/struct literals).
#[allow(clippy::struct_excessive_bools)]
pub struct LetterStyle {
    /// Couleur du texte (CSS, par ex. `"#ff0000"`).
    pub color: Option<String>,
    /// Couleur de fond (CSS).
    pub background: Option<String>,
    /// Gras.
    pub bold: bool,
    /// Italique.
    pub italic: bool,
    /// Contour gras (LC6 « stroke ») — `font-weight:bold; -webkit-text-stroke`.
    pub stroke: bool,
    /// Souligné.
    pub underline: bool,
    /// Classe CSS personnalisée (utilisée en `RenderMode::Classes`).
    pub class: Option<String>,
}

impl LetterStyle {
    /// Style minimal avec une couleur de texte.
    #[must_use]
    pub fn color(c: impl Into<String>) -> Self {
        Self {
            color: Some(c.into()),
            ..Self::default()
        }
    }
}

/// Une règle de coloriage : un *pattern* littéral et le style associé.
///
/// Les patterns peuvent être de simples lettres (`"b"`, `"q"`) ou des
/// séquences (`"pir"`, `"qu"`). Aucune syntaxe regex n'est interprétée
/// (par fidélité à LC6 et par sécurité — pas de `ReDoS` possible côté API).
#[derive(Debug, Clone)]
pub struct LetterRule {
    /// Pattern à matcher.
    pub pattern: Cow<'static, str>,
    /// Style appliqué au match.
    pub style: LetterStyle,
}

impl LetterRule {
    /// Construit une règle.
    #[must_use]
    pub fn new(pattern: impl Into<Cow<'static, str>>, style: LetterStyle) -> Self {
        Self {
            pattern: pattern.into(),
            style,
        }
    }
}

/// Un span résultant du matching : intervalle d'octets dans le mot et
/// identifiant de la règle qui a matché (= index dans `rules`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LetterSpan {
    /// Décalage de début (octets, valide pour slicer le mot d'origine).
    pub byte_start: usize,
    /// Décalage de fin exclusif (octets).
    pub byte_end: usize,
    /// Index de la règle dans `rules`.
    pub rule_id: usize,
}

/// Mode de rendu HTML.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum RenderMode {
    /// `<span style="color:#ff0000">b</span>` (par défaut).
    #[default]
    Inline,
    /// `<span class="lc-letter-N">b</span>` ou la classe custom du `LetterStyle`.
    Classes,
}

impl RenderMode {
    /// Résout un mode de rendu depuis son nom : `"inline"` ou `"classes"`.
    ///
    /// Rend `None` sur un nom inconnu. Comme pour [`presets::by_name`], le
    /// traitement du cas inconnu appartient à l'appelant : les bindings WASM,
    /// C et JNI font `unwrap_or_default()` (donc [`RenderMode::Inline`]), le
    /// binding Python en fait une `ValueError`.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "inline" => Some(Self::Inline),
            "classes" => Some(Self::Classes),
            _ => None,
        }
    }
}

enum Matcher {
    /// Lettre simple à matcher en ignorant les diacritiques (8-set LC6).
    Folded(char),
    /// Pattern lowercase à comparer littéralement.
    Literal(Vec<char>),
}

/// Reproduit les 8 filtres pré-définis de LC6 (`functionlc6.js:223-232`).
fn folded_set(c: char) -> Option<&'static [char]> {
    match c {
        'a' => Some(&['a', 'à', 'á', 'â', 'ã', 'ä', 'å']),
        'c' => Some(&['c', 'ç']),
        'e' => Some(&['e', 'é', 'è', 'ê', 'ë']),
        'i' => Some(&['i', 'ì', 'í', 'î', 'ï']),
        'o' => Some(&['o', 'ò', 'ó', 'õ', 'ö', 'ø']),
        'u' => Some(&['u', 'ù', 'ú', 'û', 'ü']),
        'y' => Some(&['y', 'ÿ']),
        'n' => Some(&['n', 'ñ']),
        _ => None,
    }
}

fn lower(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

fn compile(pattern: &str) -> Option<Matcher> {
    if pattern.is_empty() {
        return None;
    }
    let chars: Vec<char> = pattern.chars().map(lower).collect();
    if chars.len() == 1 && folded_set(chars[0]).is_some() {
        return Some(Matcher::Folded(chars[0]));
    }
    Some(Matcher::Literal(chars))
}

/// Calcule les spans coloriés pour `word` selon `rules`.
///
/// Les spans sont retournés dans l'ordre du texte, sans chevauchement.
/// Les positions correspondent à des indices d'octets dans `word`,
/// directement utilisables pour slicer (`&word[span.byte_start..span.byte_end]`).
#[must_use]
pub fn match_letters(word: &str, rules: &[LetterRule]) -> Vec<LetterSpan> {
    if word.is_empty() || rules.is_empty() {
        return Vec::new();
    }
    let matchers: Vec<Option<Matcher>> = rules.iter().map(|r| compile(&r.pattern)).collect();
    if matchers.iter().all(Option::is_none) {
        return Vec::new();
    }

    let chars: Vec<(usize, char)> = word.char_indices().collect();
    let lowered: Vec<char> = chars.iter().map(|&(_, c)| lower(c)).collect();
    let n = chars.len();

    let mut spans = Vec::new();
    let mut i = 0;
    while i < n {
        let mut hit: Option<(usize, usize)> = None;
        // Règles en ordre inverse : la dernière déclarée gagne (LC6 outer-first).
        for (rid, m) in matchers.iter().enumerate().rev() {
            let Some(m) = m else { continue };
            let len = match m {
                Matcher::Folded(c) => folded_set(*c)
                    .filter(|set| set.contains(&lowered[i]))
                    .map(|_| 1),
                Matcher::Literal(p) => {
                    if i + p.len() <= n && lowered[i..i + p.len()] == p[..] {
                        Some(p.len())
                    } else {
                        None
                    }
                }
            };
            if let Some(len) = len {
                hit = Some((rid, len));
                break;
            }
        }

        if let Some((rid, len)) = hit {
            let start = chars[i].0;
            let end = if i + len < n {
                chars[i + len].0
            } else {
                word.len()
            };
            spans.push(LetterSpan {
                byte_start: start,
                byte_end: end,
                rule_id: rid,
            });
            i += len;
        } else {
            i += 1;
        }
    }
    spans
}

/// Rend `word` en HTML, en enveloppant chaque span d'un `<span>` stylé.
///
/// Le texte hors-span est échappé HTML mais sorti tel quel. Le texte
/// dans-span est échappé puis enveloppé. Le mode `Inline` produit une
/// balise `style=""` ; le mode `Classes` produit `class="…"` (la classe
/// custom du style si fournie, sinon `lc-letter-{rule_id}`).
#[must_use]
pub fn render_letters_html(
    word: &str,
    spans: &[LetterSpan],
    rules: &[LetterRule],
    mode: RenderMode,
) -> String {
    let mut out = String::with_capacity(word.len() * 2);
    let mut cursor = 0;

    for span in spans {
        out.push_str(&escape::html(&word[cursor..span.byte_start]));

        let style = &rules[span.rule_id].style;
        let inner = escape::html(&word[span.byte_start..span.byte_end]);

        match mode {
            RenderMode::Inline => {
                let css = inline_style(style);
                if css.is_empty() {
                    out.push_str(&inner);
                } else {
                    out.push_str("<span style=\"");
                    out.push_str(&css);
                    out.push_str("\">");
                    out.push_str(&inner);
                    out.push_str("</span>");
                }
            }
            RenderMode::Classes => {
                let class = class_for(style, span.rule_id);
                out.push_str("<span class=\"");
                out.push_str(&class);
                out.push_str("\">");
                out.push_str(&inner);
                out.push_str("</span>");
            }
        }

        cursor = span.byte_end;
    }
    out.push_str(&escape::html(&word[cursor..]));
    out
}

fn inline_style(s: &LetterStyle) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(c) = &s.color {
        parts.push(format!("color:{c}"));
    }
    if let Some(b) = &s.background {
        parts.push(format!("background-color:{b}"));
    }
    if s.stroke {
        // LC6 functionlc6.js:36-37 — bold + webkit text-stroke.
        parts.push("font-weight:bold".into());
        parts.push("-webkit-text-stroke:0.03em black".into());
    } else if s.bold {
        parts.push("font-weight:bold".into());
    }
    if s.italic {
        parts.push("font-style:italic".into());
    }
    if s.underline {
        parts.push("text-decoration:underline".into());
    }
    parts.join(";")
}

fn class_for(s: &LetterStyle, rule_id: usize) -> String {
    s.class
        .clone()
        .unwrap_or_else(|| format!("lc-letter-{rule_id}"))
}

/// Presets prêts à l'emploi pour les confusions de lettres les plus courantes.
///
/// Inspirés de la documentation LC6 (`doc/utilisations.md:31`). LC6 ne
/// hard-code aucun groupe — c'est l'enseignant qui les configure.
/// Ces presets sont du sucre pour la découvrabilité ; le moteur reste
/// générique.
pub mod presets {
    use super::{LetterRule, LetterStyle};

    /// Confusion classique **b / d / p / q** — quatre couleurs distinctes.
    #[must_use]
    pub fn bdpq() -> Vec<LetterRule> {
        vec![
            LetterRule::new("b", LetterStyle::color("#1a73e8")), // bleu
            LetterRule::new("d", LetterStyle::color("#1e8e3e")), // vert
            LetterRule::new("p", LetterStyle::color("#d93025")), // rouge
            LetterRule::new("q", LetterStyle::color("#f9ab00")), // jaune-orangé
        ]
    }

    /// Confusion **m / n / u** — trois lettres avec le même nombre de jambages.
    #[must_use]
    pub fn mnu() -> Vec<LetterRule> {
        vec![
            LetterRule::new("m", LetterStyle::color("#1a73e8")),
            LetterRule::new("n", LetterStyle::color("#1e8e3e")),
            LetterRule::new("u", LetterStyle::color("#d93025")),
        ]
    }

    /// Distinction des séquences **pir / pri** (cf. `doc/utilisations.md:31`).
    #[must_use]
    pub fn pir_pri() -> Vec<LetterRule> {
        vec![
            LetterRule::new("pir", LetterStyle::color("#1a73e8")),
            LetterRule::new("pri", LetterStyle::color("#d93025")),
        ]
    }

    /// Résout un preset depuis son nom.
    ///
    /// Noms acceptés : `"bdpq"`, `"mnu"` et `"pir-pri"` — ce dernier avec
    /// `"pir_pri"` en alias, les identifiants transitant par les bindings C
    /// et JNI s'écrivant souvent sans tiret.
    ///
    /// Rend `None` sur un nom inconnu, à charge pour l'appelant de décider :
    /// les bindings WASM, C et JNI font `unwrap_or_default()` (aucune règle,
    /// donc le mot rendu échappé mais sans span), le binding Python en fait
    /// une `ValueError`. Cette fonction existe pour que les cinq
    /// consommateurs partagent la même table de noms.
    ///
    /// ```
    /// use syllabify_fr::letters::presets;
    ///
    /// assert!(presets::by_name("bdpq").is_some());
    /// assert!(presets::by_name("pir_pri").is_some());
    /// assert!(presets::by_name("inconnu").is_none());
    /// ```
    #[must_use]
    pub fn by_name(name: &str) -> Option<Vec<LetterRule>> {
        match name {
            "bdpq" => Some(bdpq()),
            "mnu" => Some(mnu()),
            "pir-pri" | "pir_pri" => Some(pir_pri()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_inputs_return_empty() {
        assert!(match_letters("", &presets::bdpq()).is_empty());
        assert!(match_letters("abc", &[]).is_empty());
    }

    #[test]
    fn bdpq_matches_simple_letters() {
        let rules = presets::bdpq();
        let spans = match_letters("dépit", &rules);
        // d (id 1) at 0-1, p (id 2) at 3-4 (é = 2 octets)
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].rule_id, 1);
        assert_eq!(&"dépit"[spans[0].byte_start..spans[0].byte_end], "d");
        assert_eq!(spans[1].rule_id, 2);
        assert_eq!(&"dépit"[spans[1].byte_start..spans[1].byte_end], "p");
    }

    #[test]
    fn diacritic_folding_for_single_letter() {
        // pattern 'a' matche 'à'
        let rules = vec![LetterRule::new("a", LetterStyle::color("#fff"))];
        let spans = match_letters("là", &rules);
        assert_eq!(spans.len(), 1);
        assert_eq!(&"là"[spans[0].byte_start..spans[0].byte_end], "à");
    }

    #[test]
    fn case_insensitive_matching() {
        let rules = vec![LetterRule::new("B", LetterStyle::color("#fff"))];
        let spans = match_letters("Bobby", &rules);
        // 'B' et 'b' et 'b' (l'avant-dernier doublon)
        assert_eq!(spans.len(), 3);
    }

    #[test]
    fn last_rule_wins_over_overlap() {
        // pattern court 'p' avant pattern long 'pir' :
        // la dernière règle déclarée ('pir') a la priorité (LC6 outer-first).
        let rules = vec![
            LetterRule::new("p", LetterStyle::color("#aaa")),
            LetterRule::new("pir", LetterStyle::color("#bbb")),
        ];
        let spans = match_letters("pirate", &rules);
        // pir (rule_id 1) en premier, puis rien d'autre (le 'p' est consommé)
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].rule_id, 1);
        assert_eq!(spans[0].byte_end - spans[0].byte_start, 3);
    }

    #[test]
    fn multibyte_word_byte_offsets_are_correct() {
        let rules = presets::bdpq();
        let spans = match_letters("pédagogue", &rules);
        // p à 0-1, d à 3-4 (é fait 2 octets)
        assert_eq!(spans[0].byte_start, 0);
        assert_eq!(spans[0].byte_end, 1);
        assert_eq!(spans[1].byte_start, 3);
        assert_eq!(spans[1].byte_end, 4);
    }

    #[test]
    fn render_inline_emits_color() {
        let rules = presets::bdpq();
        let spans = match_letters("bd", &rules);
        let html = render_letters_html("bd", &spans, &rules, RenderMode::Inline);
        assert!(html.contains("color:#1a73e8")); // b
        assert!(html.contains("color:#1e8e3e")); // d
    }

    #[test]
    fn render_classes_uses_default_class_name() {
        let rules = presets::bdpq();
        let spans = match_letters("bd", &rules);
        let html = render_letters_html("bd", &spans, &rules, RenderMode::Classes);
        assert!(html.contains("class=\"lc-letter-0\""));
        assert!(html.contains("class=\"lc-letter-1\""));
    }

    #[test]
    fn render_classes_uses_custom_class_when_provided() {
        let rules = vec![LetterRule::new(
            "b",
            LetterStyle {
                class: Some("danger".into()),
                ..Default::default()
            },
        )];
        let spans = match_letters("bob", &rules);
        let html = render_letters_html("bob", &spans, &rules, RenderMode::Classes);
        assert!(html.contains("class=\"danger\""));
    }

    #[test]
    fn html_escapes_special_characters() {
        let rules = vec![LetterRule::new("b", LetterStyle::color("#fff"))];
        // texte hors-span : '<' doit être échappé
        let html = render_letters_html("a<b", &[], &rules, RenderMode::Inline);
        assert!(html.contains("&lt;"));
    }

    #[test]
    fn pir_pri_preset_distinguishes_sequences() {
        let rules = presets::pir_pri();
        let spans = match_letters("pirpri", &rules);
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].rule_id, 0); // 'pir' premier dans le mot → règle 0
        assert_eq!(spans[1].rule_id, 1); // puis 'pri' → règle 1
    }

    #[test]
    fn stroke_emits_webkit_text_stroke() {
        let rules = vec![LetterRule::new(
            "a",
            LetterStyle {
                stroke: true,
                ..Default::default()
            },
        )];
        let spans = match_letters("a", &rules);
        let html = render_letters_html("a", &spans, &rules, RenderMode::Inline);
        assert!(html.contains("-webkit-text-stroke"));
        assert!(html.contains("font-weight:bold"));
    }

    #[test]
    fn empty_pattern_is_ignored() {
        let rules = vec![
            LetterRule::new("", LetterStyle::color("#fff")),
            LetterRule::new("b", LetterStyle::color("#000")),
        ];
        let spans = match_letters("abc", &rules);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].rule_id, 1);
    }
}
