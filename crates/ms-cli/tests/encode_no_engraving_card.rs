//! --no-engraving-card suppresses stderr block; stdout unchanged.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn encode_no_engraving_card_suppresses_stderr() {
    Command::cargo_bin("ms").unwrap()
        .args([
            "encode",
            "--phrase",
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
            "--no-engraving-card",
        ])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("ms10entrsqqqq"))
        .stderr(predicate::str::is_empty());
}
