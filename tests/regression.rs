// SPDX-License-Identifier: GPL-3.0-or-later
//! Test de régression : vérifie que `syllabify_fr::syllabify_text()` produit exactement
//! la même sortie que pylirecouleur 0.0.5 sur un corpus représentatif.

use serde_json::Value;
use std::fs;
use syllabify_fr::{syllabify_text, TextChunk};

#[derive(Debug)]
struct Mismatch {
    word: String,
    expected: String,
    actual: String,
}

fn load_oracle() -> Value {
    let data = fs::read_to_string("tests/oracle.json").expect("oracle.json manquant");
    serde_json::from_str(&data).expect("oracle.json mal formé")
}

/// Formate une liste de chunks pour comparaison textuelle lisible.
/// Un chunk mot devient "fa|mi|lle", un chunk brut reste tel quel entre [..].
fn fmt_chunks(chunks: &[TextChunk]) -> String {
    let mut s = String::new();
    for c in chunks {
        match c {
            TextChunk::Word(syls) => s.push_str(&syls.join("|")),
            TextChunk::Raw(r) => s.push_str(&format!("[{}]", r)),
            _ => {}
        }
        s.push(' ');
    }
    s.trim_end().to_string()
}

/// Oracle chunks (format JSON) → même format texte.
fn fmt_oracle_chunks(chunks: &Value) -> String {
    let arr = chunks.as_array().unwrap();
    let mut s = String::new();
    for c in arr {
        if let Some(syls) = c.as_array() {
            let ss: Vec<String> = syls
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect();
            s.push_str(&ss.join("|"));
        } else if let Some(raw) = c.as_str() {
            s.push_str(&format!("[{}]", raw));
        }
        s.push(' ');
    }
    s.trim_end().to_string()
}

#[test]
fn regression_syllabes() {
    let oracle = load_oracle();
    let obj = oracle.as_object().expect("oracle doit être un objet");

    let mut mismatches: Vec<Mismatch> = Vec::new();
    let mut total = 0usize;

    for (word, expected_data) in obj {
        if expected_data.get("error").is_some() {
            continue;
        }
        total += 1;

        let expected = fmt_oracle_chunks(&expected_data["chunks"]);
        let actual_chunks = syllabify_text(word);
        let actual = fmt_chunks(&actual_chunks);

        if actual != expected {
            mismatches.push(Mismatch {
                word: word.clone(),
                expected,
                actual,
            });
        }
    }

    let nb = mismatches.len();
    let pct = 100.0 * (total - nb) as f64 / total as f64;
    eprintln!(
        "\n=== Régression : {}/{} OK ({:.1}%) ===",
        total - nb,
        total,
        pct
    );
    if !mismatches.is_empty() {
        // Écrit aussi dans un fichier pour diagnostic offline
        let mut report = String::new();
        report.push_str(&format!("=== {} échecs / {} ===\n", nb, total));
        for m in &mismatches {
            let line = format!("{:30} attendu={} obtenu={}\n", m.word, m.expected, m.actual);
            eprintln!("{}", line.trim_end());
            report.push_str(&line);
        }
        let _ = fs::write("/tmp/syllabify_mismatches.txt", &report);
        panic!(
            "{} cas divergent de pylirecouleur (détails dans /tmp/syllabify_mismatches.txt)",
            nb
        );
    }
}
