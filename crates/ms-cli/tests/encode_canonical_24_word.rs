//! `ms encode --phrase` 24-word abandon round-trip.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn encode_24_word_abandon_art() {
    let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
    Command::cargo_bin("ms")
        .unwrap()
        .args(["encode", "--phrase", phrase])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("ms10entrsqqqq"))
        .stderr(predicate::str::contains("word count: 24"));
}
