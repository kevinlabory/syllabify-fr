// SPDX-License-Identifier: GPL-3.0-or-later
//! CLI : `syllabify <mot>` ou `syllabify --text "phrase entière"`.

use std::env;
use std::io::{self, BufRead, Write};
use std::process::ExitCode;

use syllabify_fr::{syllabify_text, syllables, TextChunk};

fn print_usage() {
    eprintln!("syllabify-fr — syllabification française (LireCouleur port)");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  syllabify <mot>                  Syllabifie un mot, séparateur '-'");
    eprintln!("  syllabify --text \"une phrase\"    Syllabifie un texte entier");
    eprintln!("  syllabify --json <mot>           Sortie JSON (tableau de syllabes)");
    eprintln!("  syllabify -                      Lit stdin, un mot par ligne");
    eprintln!();
    eprintln!("EXEMPLES:");
    eprintln!("  syllabify chocolat                        # cho-co-lat");
    eprintln!("  syllabify --text \"le petit chat noir\"     # le pe-tit chat noir");
    eprintln!("  echo \"famille\" | syllabify -              # fa-mi-lle");
}

fn syllabify_to_dashes(word: &str) -> String {
    syllables(word).join("-")
}

fn syllabify_to_json(word: &str) -> String {
    let s = syllables(word);
    // JSON array simple, on échappe juste les " et \
    let escaped: Vec<String> = s
        .iter()
        .map(|s| format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")))
        .collect();
    format!("[{}]", escaped.join(","))
}

fn syllabify_text_to_string(text: &str) -> String {
    let mut out = String::new();
    for chunk in syllabify_text(text) {
        match chunk {
            TextChunk::Word(syls) => out.push_str(&syls.join("-")),
            TextChunk::Raw(s) => out.push_str(&s),
        }
    }
    out
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() || args[0] == "-h" || args[0] == "--help" {
        print_usage();
        return ExitCode::from(if args.is_empty() { 1 } else { 0 });
    }

    match args[0].as_str() {
        "--text" => {
            if args.len() < 2 {
                eprintln!("erreur: --text requiert un argument");
                return ExitCode::from(2);
            }
            let text = args[1..].join(" ");
            println!("{}", syllabify_text_to_string(&text));
        }
        "--json" => {
            if args.len() < 2 {
                eprintln!("erreur: --json requiert un argument");
                return ExitCode::from(2);
            }
            println!("{}", syllabify_to_json(&args[1]));
        }
        "-" => {
            let stdin = io::stdin();
            let stdout = io::stdout();
            let mut out = stdout.lock();
            for line in stdin.lock().lines() {
                let line = match line {
                    Ok(l) => l,
                    Err(_) => break,
                };
                let word = line.trim();
                if word.is_empty() {
                    writeln!(out, "").ok();
                    continue;
                }
                writeln!(out, "{}", syllabify_to_dashes(word)).ok();
            }
        }
        _ => {
            // syllabifier chacun des args comme un mot
            let outs: Vec<String> = args.iter().map(|w| syllabify_to_dashes(w)).collect();
            println!("{}", outs.join(" "));
        }
    }
    ExitCode::SUCCESS
}
