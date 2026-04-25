// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests de cas limites — comportements documentés pour les entrées hors-corpus.
//!
//! Ces tests ne vérifient pas la conformité LC6 (c'est l'oracle) mais s'assurent
//! que les entrées atypiques ne paniquent pas et que leurs sorties restent stables.

use syllabify_fr::{phonemes, syllabify_text, syllables};

#[test]
fn empty_string_returns_one_empty_syllable() {
    // Comportement défini : une chaîne vide produit une syllabe vide,
    // pas un Vec vide, car le pipeline consomme toujours au moins un token.
    assert_eq!(syllables(""), vec!["".to_string()]);
}

#[test]
fn single_letter() {
    assert_eq!(syllables("a"), vec!["a"]);
    assert_eq!(syllables("y"), vec!["y"]);
}

#[test]
fn digits_are_opaque() {
    // Les chiffres n'ont pas de règle phonétique : traités comme un bloc unique.
    assert_eq!(syllables("123"), vec!["123"]);
}

#[test]
fn uppercase_preserves_case_in_output() {
    // Le nettoyeur opère en interne en minuscules pour la phonétisation,
    // mais les syllabes restituées utilisent les lettres originales.
    assert_eq!(syllables("CHOCOLAT"), vec!["CHO", "CO", "LAT"]);
}

#[test]
fn word_with_apostrophe() {
    // Les apostrophes sont conservées dans le token : l'arbre = 1 mot.
    assert_eq!(syllables("l'arbre"), vec!["l'ar", "bre"]);
}

#[test]
fn hyphenated_word() {
    // Les traits d'union sont conservés en-mot (comportement v6, cf. NOTES-v6.md §4).
    assert_eq!(syllables("grand-père"), vec!["grand", "pè", "re"]);
}

#[test]
fn phonemes_empty_string() {
    // Pas de phonème pour une chaîne vide.
    assert_eq!(phonemes(""), vec![]);
}

#[test]
fn syllabify_text_empty_string() {
    // syllabify_text sur chaîne vide → pas de chunk.
    assert_eq!(syllabify_text(""), vec![]);
}

#[test]
fn single_punctuation_is_raw_chunk() {
    use syllabify_fr::TextChunk;
    let chunks = syllabify_text("!");
    assert!(matches!(&chunks[0], TextChunk::Raw(s) if s == "!"));
}
