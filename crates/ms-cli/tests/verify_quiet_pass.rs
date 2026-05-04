//! Verify on valid v0.1 string → exit 0 with one-line OK summary.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn verify_valid_v01_exit_0() {
    Command::cargo_bin("ms")
        .unwrap()
        .args([
            "verify",
            "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "OK: valid v0.1 entr (12 words, 50 chars)",
        ));
}
