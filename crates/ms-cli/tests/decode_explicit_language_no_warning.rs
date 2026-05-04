//! Explicit --language removes both stderr and stdout warnings.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn decode_explicit_english_removes_warnings() {
    Command::cargo_bin("ms")
        .unwrap()
        .args([
            "decode",
            "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f",
            "--language",
            "english",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("default —").not())
        .stderr(predicate::str::contains("defaulted").not());
}
