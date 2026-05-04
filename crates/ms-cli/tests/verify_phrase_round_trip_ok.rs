//! Verify with --phrase matching the encoded entropy → exit 0.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn verify_round_trip_with_correct_phrase() {
    Command::cargo_bin("ms").unwrap()
        .args([
            "verify",
            "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f",
            "--phrase",
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("OK: round-trip valid (12 words, language=english)"));
}
