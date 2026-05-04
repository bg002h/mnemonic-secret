//! `ms encode --phrase` 12-word abandon round-trip.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn encode_12_word_abandon_about() {
    Command::cargo_bin("ms").unwrap()
        .args([
            "encode",
            "--phrase",
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        ])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("ms10entrsqqqq"))
        .stdout(predicate::str::contains("\n\n"))
        .stderr(predicate::str::contains("language: english"))
        .stderr(predicate::str::contains("word count: 12"))
        .stderr(predicate::str::contains("passphrase: not stored"));
}
