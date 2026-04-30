//! Démonstration de l'API syllabify-fr.
//!
//! `cargo run --example basic`

use syllabify_fr::{
    liaison_possible, phonemes, render_html, render_word_html, syllabify_text, syllables,
    syllables_with, AssembleMode, SyllableMode, TextChunk,
};

fn main() {
    // ── Syllabification de mots ──────────────────────────────────────────────
    println!("=== syllables ===");
    for word in &[
        "chocolat",
        "famille",
        "parlent",
        "œil",
        "grand-père",
        "anticonstitutionnellement",
    ] {
        println!("  {:30} → {}", word, syllables(word).join("-"));
    }

    // ── Phonèmes ─────────────────────────────────────────────────────────────
    println!("\n=== phonemes ===");
    for (code, letters) in phonemes("chocolat") {
        print!("[{code}:{letters}] ");
    }
    println!();

    // ── Texte complet avec homographe ─────────────────────────────────────────
    println!("\n=== syllabify_text ===");
    for text in &["le petit chat noir", "le couvent", "elles couvent"] {
        let chunks = syllabify_text(text);
        let rendered: Vec<String> = chunks
            .iter()
            .map(|c| match c {
                TextChunk::Word(s) => s.join("-"),
                TextChunk::Raw(s) => s.clone(),
                _ => String::new(),
            })
            .collect();
        println!("  {:25} → {}", text, rendered.join(""));
    }

    // ── Modes alternatifs ────────────────────────────────────────────────────
    println!("\n=== modes ===");
    let word = "école";
    println!(
        "  {word} Written → {}",
        syllables_with(word, false, AssembleMode::Std, SyllableMode::Written).join("-")
    );
    println!(
        "  {word} Oral    → {}",
        syllables_with(word, false, AssembleMode::Std, SyllableMode::Oral).join("-")
    );

    // ── Liaisons ─────────────────────────────────────────────────────────────
    println!("\n=== liaisons ===");
    let pairs = [("les", "hôtels"), ("les", "héros"), ("un", "enfant")];
    for (a, b) in pairs {
        println!("  «{}» + «{}» → liaison: {}", a, b, liaison_possible(a, b));
    }

    // ── HTML ──────────────────────────────────────────────────────────────────
    println!("\n=== render_word_html ===");
    println!("  {}", render_word_html("chocolat"));

    println!("\n=== render_html ===");
    println!("  {}", render_html("les hôtels"));
}
