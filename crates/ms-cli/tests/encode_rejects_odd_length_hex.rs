//! Odd-length --hex → exit 1 with friendly message.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn encode_rejects_odd_length_hex() {
    Command::cargo_bin("ms")
        .unwrap()
        .args(["encode", "--hex", "0"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("even-length hex"));
}

#[test]
fn encode_rejects_non_hex_char() {
    Command::cargo_bin("ms")
        .unwrap()
        .args(["encode", "--hex", "ZZ"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("position 0"));
}
