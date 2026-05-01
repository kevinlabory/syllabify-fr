// SPDX-License-Identifier: GPL-3.0-or-later
//! CLI : `syllabify <mot>` ou `syllabify --text "phrase entière"`.

use std::io::{self, BufRead, Write};
use std::process::ExitCode;

use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use syllabify_fr::{syllabify_text, syllables_with, AssembleMode, SyllableMode, TextChunk};

#[derive(Parser, Debug)]
#[command(
    name = "syllabify",
    version,
    about = "Syllabification française (port LireCouleur 6)",
    long_about = "Segmente chaque mot français en toutes ses syllabes — utile pour l'apprentissage de la lecture, notamment chez les enfants dyslexiques. Contrairement aux séparateurs typographiques (Hypher, hyphen-fr…) qui minimisent les points de coupure, syllabify segmente l'intégralité du mot."
)]
struct Cli {
    /// Mots à syllabifier. Utiliser `-` pour lire stdin (un mot par ligne).
    #[arg(value_name = "MOT")]
    words: Vec<String>,

    /// Syllabifier un texte complet (avec désambiguïsation des homographes).
    #[arg(long, conflicts_with = "json")]
    text: bool,

    /// Sortie JSON (tableau de syllabes).
    #[arg(long)]
    json: bool,

    /// Désactive les post-traitements subtils (yod, o ouvert/fermé).
    #[arg(long = "novice-reader")]
    novice_reader: bool,

    /// Mode oral (q caduc final fusionné : `école` → `é-cole`).
    #[arg(long)]
    oral: bool,

    /// Génère les completions shell sur stdout, puis quitte.
    #[arg(long, value_enum, value_name = "SHELL", exclusive = true)]
    completions: Option<Shell>,
}

fn syllable_mode(oral: bool) -> SyllableMode {
    if oral {
        SyllableMode::Oral
    } else {
        SyllableMode::Written
    }
}

fn syllabify_to_dashes(word: &str, novice: bool, oral: bool) -> String {
    syllables_with(word, novice, AssembleMode::Std, syllable_mode(oral)).join("-")
}

fn syllabify_to_json(word: &str, novice: bool, oral: bool) -> String {
    let s = syllables_with(word, novice, AssembleMode::Std, syllable_mode(oral));
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
            _ => {}
        }
    }
    out
}

fn run_stdin(novice: bool, oral: bool) {
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
            writeln!(out).ok();
            continue;
        }
        writeln!(out, "{}", syllabify_to_dashes(word, novice, oral)).ok();
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if let Some(shell) = cli.completions {
        let mut cmd = Cli::command();
        let name = cmd.get_name().to_string();
        generate(shell, &mut cmd, name, &mut io::stdout());
        return ExitCode::SUCCESS;
    }

    if cli.text {
        if cli.words.is_empty() {
            eprintln!("erreur: --text requiert au moins un argument");
            return ExitCode::from(2);
        }
        let text = cli.words.join(" ");
        println!("{}", syllabify_text_to_string(&text));
        return ExitCode::SUCCESS;
    }

    if cli.words.len() == 1 && cli.words[0] == "-" {
        run_stdin(cli.novice_reader, cli.oral);
        return ExitCode::SUCCESS;
    }

    if cli.words.is_empty() {
        Cli::command().print_help().ok();
        println!();
        return ExitCode::from(2);
    }

    if cli.json {
        if cli.words.len() > 1 {
            eprintln!("erreur: --json n'accepte qu'un seul mot");
            return ExitCode::from(2);
        }
        println!(
            "{}",
            syllabify_to_json(&cli.words[0], cli.novice_reader, cli.oral)
        );
    } else {
        let outs: Vec<String> = cli
            .words
            .iter()
            .map(|w| syllabify_to_dashes(w, cli.novice_reader, cli.oral))
            .collect();
        println!("{}", outs.join(" "));
    }
    ExitCode::SUCCESS
}
