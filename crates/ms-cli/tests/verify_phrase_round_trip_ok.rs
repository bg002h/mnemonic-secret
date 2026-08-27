//! Verify with --phrase matching the encoded entropy → exit 0.

use assert_cmd::Command;
use predicates::prelude::*;

const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

/// **Two secret values in one invocation, and P2 is what makes that
/// performable privately at all.** Before `--in` the only way to supply the
/// second was argv; `ms verify - --phrase -` is the contention refusal the
/// freed-stdin work routes around.
#[test]
fn verify_round_trip_with_correct_phrase() {
    let dir = tempfile::tempdir().unwrap();
    let card = dir.path().join("card.ms1");
    std::fs::write(&card, MS1).unwrap();
    Command::cargo_bin("ms")
        .unwrap()
        .args([
            "verify",
            "--in",
            &card.display().to_string(),
            "--phrase",
            "-",
        ])
        .write_stdin(PHRASE.to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "OK: round-trip valid (12 words, language=english)",
        ));
}
