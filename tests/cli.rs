// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests d'intégration du binaire `syllabify`.
//!
//! Gated behind `cli` feature : avec `--no-default-features --features regex-lite`,
//! le binaire n'est pas compilé (`required-features = ["cli"]`), donc ces tests
//! seraient des panics. On désactive le fichier entier dans ce cas.

#![cfg(feature = "cli")]

use assert_cmd::Command;
use predicates::str::contains;

fn syllabify() -> Command {
    Command::cargo_bin("syllabify").unwrap()
}

#[test]
fn positional_word() {
    syllabify()
        .arg("chocolat")
        .assert()
        .success()
        .stdout("cho-co-lat\n");
}

#[test]
fn multiple_positional_words() {
    syllabify()
        .args(["chocolat", "famille"])
        .assert()
        .success()
        .stdout("cho-co-lat fa-mi-lle\n");
}

#[test]
fn text_mode() {
    syllabify()
        .args(["--text", "le petit chat noir"])
        .assert()
        .success()
        .stdout("le pe-tit chat noir\n");
}

#[test]
fn json_mode() {
    syllabify()
        .args(["--json", "chocolat"])
        .assert()
        .success()
        .stdout("[\"cho\",\"co\",\"lat\"]\n");
}

#[test]
fn json_rejects_multi_words() {
    syllabify()
        .args(["--json", "chocolat", "famille"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn json_text_mutually_exclusive() {
    syllabify()
        .args(["--json", "--text", "foo"])
        .assert()
        .failure();
}

#[test]
fn stdin_mode() {
    syllabify()
        .arg("-")
        .write_stdin("famille\nchocolat\n")
        .assert()
        .success()
        .stdout("fa-mi-lle\ncho-co-lat\n");
}

#[test]
fn novice_reader_flag_accepted() {
    syllabify()
        .args(["--novice-reader", "chocolat"])
        .assert()
        .success();
}

#[test]
fn oral_mode_changes_output() {
    let written = syllabify().arg("école").output().unwrap();
    let oral = syllabify().args(["--oral", "école"]).output().unwrap();
    assert_ne!(
        written.stdout, oral.stdout,
        "le mode oral doit modifier la sortie pour `école`"
    );
}

#[test]
fn version_flag() {
    syllabify()
        .arg("--version")
        .assert()
        .success()
        .stdout(contains("syllabify"));
}

#[test]
fn help_flag() {
    syllabify()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("Usage:"));
}

#[test]
fn completions_bash() {
    syllabify()
        .args(["--completions", "bash"])
        .assert()
        .success()
        .stdout(contains("_syllabify"));
}

#[test]
fn completions_zsh() {
    syllabify()
        .args(["--completions", "zsh"])
        .assert()
        .success()
        .stdout(contains("syllabify"));
}

#[test]
fn no_args_shows_help_and_exits_with_error() {
    syllabify().assert().failure().code(2);
}

#[test]
fn highlight_letters_bdpq_emits_inline_spans() {
    syllabify()
        .args(["--highlight-letters", "bdpq", "dépit"])
        .assert()
        .success()
        .stdout(contains("color:#1e8e3e")) // d
        .stdout(contains("color:#d93025")); // p
}

#[test]
fn highlight_letters_pir_pri() {
    syllabify()
        .args(["--highlight-letters", "pir-pri", "pirate"])
        .assert()
        .success()
        .stdout(contains("color:#1a73e8"))
        .stdout(contains(">pir</span>"));
}

#[test]
fn highlight_letters_rejects_unknown_preset() {
    syllabify()
        .args(["--highlight-letters", "nope", "mot"])
        .assert()
        .failure();
}

/// Audit #3 — `--json` produit du JSON RFC 8259-conforme sur entrées
/// adversariales (caractères de contrôle, guillemets, backslashes).
/// La sortie doit être parseable par `serde_json`.
#[test]
fn json_output_is_valid_on_adversarial_inputs() {
    // On utilise stdin pour pouvoir injecter les caractères de contrôle
    // sans qu'ils soient interprétés par le shell.
    let adversarial = "a\u{0001}\u{0007}\u{0008}\u{000B}\u{000C}\u{001F}\"\\b";
    let output = syllabify()
        .args(["--json", adversarial])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).expect("stdout not utf-8");
    let trimmed = stdout.trim_end();
    // Aucun control char brut.
    assert!(
        !trimmed.chars().any(|c| (c as u32) < 0x20),
        "raw control char in --json output: {trimmed:?}"
    );
    // Round-trip via serde_json.
    let _: serde_json::Value = serde_json::from_str(trimmed)
        .unwrap_or_else(|e| panic!("invalid JSON from --json: {e} :: {trimmed:?}"));
}
