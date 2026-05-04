//! Inspect on canonical valid string → verdict OK + fields.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn inspect_valid_canonical_v01_string() {
    Command::cargo_bin("ms")
        .unwrap()
        .args([
            "inspect",
            "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f",
        ])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("OK: would decode v0.1"))
        .stdout(predicate::str::contains("hrp: ms"))
        .stdout(predicate::str::contains("threshold: 0"))
        .stdout(predicate::str::contains("tag: entr"))
        .stdout(predicate::str::contains("share_index: s"))
        .stdout(predicate::str::contains("prefix_byte: 0x00"))
        .stdout(predicate::str::contains("checksum_valid: true"));
}

#[test]
fn inspect_valid_string_json_schema() {
    Command::cargo_bin("ms")
        .unwrap()
        .args([
            "inspect",
            "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f",
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"schema_version\":\"1\""))
        .stdout(predicate::str::contains("\"would_decode\":true"))
        .stdout(predicate::str::contains("\"failure_reasons\":[]"));
}
