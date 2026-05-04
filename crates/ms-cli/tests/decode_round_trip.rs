//! `ms decode <ms1>` produces the labeled block + matches input phrase.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn decode_canonical_12_word_round_trip() {
    Command::cargo_bin("ms").unwrap()
        .args(["decode", "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f"])
        .assert()
        .success()
        .stdout(predicate::str::contains("entropy: 00000000000000000000000000000000"))
        .stdout(predicate::str::contains(
            "phrase: abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        ))
        .stdout(predicate::str::contains("language: english (12 words"));
}

#[test]
fn decode_json_schema() {
    Command::cargo_bin("ms")
        .unwrap()
        .args([
            "decode",
            "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f",
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"schema_version\":\"1\""))
        .stdout(predicate::str::contains("\"language\":\"english\""))
        .stdout(predicate::str::contains("\"language_defaulted\":true"));
}
