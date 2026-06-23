//! SPEC §4 rule 6 (UnknownTag): integration coverage.
//!
//! Plan task 4.10 reviewer brief flagged this as a missing test. "wxyz" is
//! codex32-alphabet-valid (w/x/y/z all in qpzry9x8gf2tvdw0s3jn54khce6mua7l)
//! but NOT in RESERVED_TAG_TABLE. Decode should fire UnknownTag
//! (FormatViolation -> exit 2).

use assert_cmd::Command;
use ms_codec::codex32::{Codex32String, Fe};
use predicates::prelude::*;

#[test]
fn decode_rejects_unknown_tag() {
    let mut data = vec![0x00u8];
    data.extend_from_slice(&[0x00u8; 16]);
    let s = Codex32String::from_seed("ms", 0, "wxyz", Fe::S, &data)
        .unwrap()
        .to_string();
    Command::cargo_bin("ms")
        .unwrap()
        .args(["decode", &s])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("unknown tag"));
}

#[test]
fn decode_rejects_unknown_tag_json_envelope() {
    let mut data = vec![0x00u8];
    data.extend_from_slice(&[0x00u8; 16]);
    let s = Codex32String::from_seed("ms", 0, "wxyz", Fe::S, &data)
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
    assert_eq!(v["error"]["kind"], "UnknownTag");
    assert_eq!(v["error"]["details"]["tag"], "wxyz");
}
