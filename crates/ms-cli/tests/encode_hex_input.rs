//! `ms encode --hex` round-trip equivalent to --phrase.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn encode_hex_zeros_16_bytes() {
    Command::cargo_bin("ms")
        .unwrap()
        .args(["encode", "--hex", "00000000000000000000000000000000"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("ms10entrsqqqq"));
}

#[test]
fn encode_hex_omits_language_in_engraving_card() {
    Command::cargo_bin("ms")
        .unwrap()
        .args(["encode", "--hex", "00000000000000000000000000000000"])
        .assert()
        .success()
        .stderr(predicate::str::contains("word count: 12"))
        .stderr(predicate::str::contains("passphrase: not stored"))
        .stderr(predicate::str::contains("language:").not());
}
