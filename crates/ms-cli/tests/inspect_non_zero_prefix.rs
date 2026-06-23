//! Inspect on non-zero-prefix string → verdict FAIL with rule 8.

use assert_cmd::Command;
use ms_codec::codex32::{Codex32String, Fe};
use predicates::prelude::*;

fn build_with_prefix_0x01() -> String {
    let mut data = vec![0x01u8];
    data.extend_from_slice(&[0xAAu8; 16]);
    Codex32String::from_seed("ms", 0, "entr", Fe::S, &data)
        .unwrap()
        .to_string()
}

#[test]
fn inspect_non_zero_prefix_reports_rule_8() {
    let s = build_with_prefix_0x01();
    Command::cargo_bin("ms")
        .unwrap()
        .args(["inspect", &s])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("FAIL: would NOT decode v0.1"))
        .stdout(predicate::str::contains("non-zero-prefix"));
}
