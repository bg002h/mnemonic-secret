//! Inspect on string with id="seed" → verdict FAIL with rule 7.

use assert_cmd::Command;
use codex32::{Codex32String, Fe};
use predicates::prelude::*;

#[test]
fn inspect_reserved_seed_tag_reports_rule_7() {
    let mut data = vec![0x00u8];
    data.extend_from_slice(&[0xAAu8; 16]);
    let s = Codex32String::from_seed("ms", 0, "seed", Fe::S, &data)
        .unwrap()
        .to_string();

    Command::cargo_bin("ms")
        .unwrap()
        .args(["inspect", &s])
        .assert()
        .success()
        .stdout(predicate::str::contains("FAIL"))
        .stdout(predicate::str::contains("reserved-tag-not-emitted"));
}
