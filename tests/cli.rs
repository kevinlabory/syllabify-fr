// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests d'intégration du binaire `syllabify`.

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
