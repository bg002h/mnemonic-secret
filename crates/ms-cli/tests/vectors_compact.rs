//! ms vectors emits parseable JSON compact-form by default.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn vectors_compact_is_parseable_json() {
    let out = Command::cargo_bin("ms")
        .unwrap()
        .arg("vectors")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert!(parsed.is_array());
    assert!(
        parsed.as_array().unwrap().len() >= 2,
        "expected >=2 vectors"
    );
}

#[test]
fn vectors_first_entry_matches_canonical_12_word() {
    Command::cargo_bin("ms")
        .unwrap()
        .arg("vectors")
        .assert()
        .success()
        .stdout(predicate::str::contains("ms10entrsqqqq"))
        .stdout(predicate::str::contains("abandon"));
}
