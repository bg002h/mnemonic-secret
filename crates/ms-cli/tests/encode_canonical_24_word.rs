//! `ms encode --phrase` 24-word abandon round-trip.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn encode_24_word_abandon_art() {
    let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
    // `--group-size 0` keeps the exact unbroken prefix (default is now space/5).
    Command::cargo_bin("ms")
        .unwrap()
        .args(["encode", "--phrase", "-", "--group-size", "0"])
        .write_stdin((phrase).to_string())
        .assert()
        .success()
        .stdout(predicate::str::starts_with("ms10entrsqqqq"))
        .stderr(predicate::str::contains("word count: 24"));
}
