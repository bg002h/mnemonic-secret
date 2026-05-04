//! Verify on invalid string → exit 2 (format violation) with FAIL summary.

use assert_cmd::Command;
use codex32::{Codex32String, Fe};
use predicates::prelude::*;

#[test]
fn verify_non_zero_prefix_exits_2() {
    let mut data = vec![0x01u8];
    data.extend_from_slice(&[0xAAu8; 16]);
    let s = Codex32String::from_seed("ms", 0, "entr", Fe::S, &data)
        .unwrap()
        .to_string();

    Command::cargo_bin("ms")
        .unwrap()
        .args(["verify", &s])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("reserved-prefix byte was 0x01"));
}
