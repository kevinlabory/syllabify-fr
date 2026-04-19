// SPDX-License-Identifier: GPL-3.0-or-later
//! # syllabify-fr
//!
//! Syllabification française destinée à l'apprentissage de la lecture.
//! Portage en Rust de [LireCouleur 6](https://lirecouleur.forge.apps.education.fr/),
//! créé par Marie-Pierre et Luc Brungard (GPL v3).
//!
//! ## Usage rapide
//!
//! ```
//! use syllabify_fr::syllables;
//!
//! // Mode STD (défaut, comme LC6) : sépare les consonnes doubles
//! assert_eq!(syllables("famille"), vec!["fa", "mi", "lle"]);
//! assert_eq!(syllables("parlent"), vec!["par", "lent"]);
//! assert_eq!(syllables("homme"),   vec!["hom", "me"]);
//! ```

pub mod cleaner;
pub mod data;
pub mod decoder;
pub mod homographs;
pub mod html;
pub mod liaisons;
pub mod parser;
pub mod phoneme;
pub mod rules;

pub use decoder::{AssembleMode, SyllableMode, TextChunk};
pub use html::{render_html, render_word_html};
pub use liaisons::{liaison_amont, liaison_aval, liaison_possible};

/// Syllabifie un mot seul avec les paramètres par défaut
/// (mode STD comme LireCouleur 6, syllabes écrites).
pub fn syllables(word: &str) -> Vec<String> {
    syllables_with(word, false, AssembleMode::Std, SyllableMode::Written)
}

/// Variante avec contrôle fin des paramètres.
///
/// * `novice_reader` : désactive les post-traitements subtils (yod, o ouvert/fermé).
/// * `assemble_mode` : `Std` (consonnes doubles séparées, défaut pédagogique) ou
///   `Lc` (consonnes doubles groupées, segmentation savante).
/// * `syl_mode` : syllabes écrites ou orales.
pub fn syllables_with(
    word: &str,
    novice_reader: bool,
    assemble_mode: AssembleMode,
    syl_mode: SyllableMode,
) -> Vec<String> {
    let phonemes = decoder::extract_phonemes_word(word, novice_reader, syl_mode);
    let (sylls, nphons) = decoder::assemble_syllables(&phonemes, assemble_mode, syl_mode);
    sylls
        .iter()
        .map(|syl| syl.iter().map(|&i| nphons[i].letters.clone()).collect::<String>())
        .collect()
}

/// Extrait les phonèmes d'un mot : liste de (code, lettres).
pub fn phonemes(word: &str) -> Vec<(String, String)> {
    decoder::extract_phonemes_word(word, false, SyllableMode::Written)
        .into_iter()
        .map(|p| (p.code, p.letters))
        .collect()
}

/// Syllabifie un texte entier, en préservant la ponctuation et les espaces.
/// Les homographes non homophones sont désambiguïsés selon le mot précédent.
pub fn syllabify_text(text: &str) -> Vec<TextChunk> {
    decoder::extract_syllables(text, false, AssembleMode::Std, SyllableMode::Written)
}
