//! SPEC §4 rule 3 (ThresholdNotZero): integration coverage.
//!
//! Plan task 4.10 reviewer brief flagged this as a missing test. Hand-build
//! a string with threshold=2 + share=Fe::A (codex32 lib accepts arbitrary
//! threshold + share at construction). Decode should fire ThresholdNotZero
//! (FormatViolation -> exit 2).

use assert_cmd::Command;
use codex32::{Codex32String, Fe};
use predicates::prelude::*;

#[test]
fn decode_rejects_threshold_not_zero() {
    let mut data = vec![0x00u8];
    data.extend_from_slice(&[0x00u8; 16]);
    let s = Codex32String::from_seed("ms", 2, "entr", Fe::A, &data)
        .unwrap()
        .to_string();
    Command::cargo_bin("ms")
        .unwrap()
        .args(["decode", &s])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("threshold"));
}

#[test]
fn decode_rejects_threshold_not_zero_json_envelope() {
    let mut data = vec![0x00u8];
    data.extend_from_slice(&[0x00u8; 16]);
    let s = Codex32String::from_seed("ms", 2, "entr", Fe::A, &data)
        .unwrap()
        .to_string();
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(["decode", &s, "--json"])
        .assert()
        .failure()
        .code(2)
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("error envelope is valid JSON");
    assert_eq!(v["schema_version"], "1");
    assert_eq!(v["error"]["kind"], "ThresholdNotZero");
    assert_eq!(v["error"]["exit_code"], 2);
}
