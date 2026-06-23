//! `ms inspect <share>` — a lone K-of-N share is a first-class read (Task 2.3).
//!
//! A threshold∈2..9 string is one share of a share-set. `inspect` must report
//! `kind: share` + threshold/id/index and a "would combine (needs k)" verdict —
//! NOT a `FAIL`/`threshold-not-zero`. The distributed share's data()[0] is an
//! interpolated value, NOT a payload-kind prefix, so prefix_byte / payload_bytes
//! / the entr/mnem kind are SUPPRESSED.

use assert_cmd::Command;
use codex32::{Codex32String, Fe};
use predicates::prelude::*;
use serde_json::Value;

/// A genuine threshold=2 share at index Fe::P, id "tst7".
fn a_share() -> String {
    let mut data = vec![0x00u8];
    data.extend_from_slice(&[0xAAu8; 16]);
    Codex32String::from_seed("ms", 2, "tst7", Fe::P, &data)
        .unwrap()
        .to_string()
}

#[test]
fn inspect_share_text_reports_kind_share_no_fail() {
    let s = a_share();
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(["inspect", &s])
        .assert()
        .success() // exit 0 — a share is a valid read, not a failure
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).unwrap();
    assert!(
        text.contains("kind: share"),
        "expected 'kind: share' in:\n{text}"
    );
    assert!(
        text.contains("threshold: 2"),
        "expected 'threshold: 2' in:\n{text}"
    );
    assert!(text.contains("tst7"), "expected the id in:\n{text}");
    assert!(
        text.contains("index: p") || text.contains("share_index: p"),
        "expected index p in:\n{text}"
    );
    // NOT a v0.1 failure path.
    assert!(
        !text.contains("FAIL"),
        "must not report FAIL for a share:\n{text}"
    );
    assert!(
        !text.contains("threshold-not-zero"),
        "must not push threshold-not-zero:\n{text}"
    );
    // The garbage prefix-byte / payload-bytes interpretation is suppressed.
    assert!(
        !text.contains("prefix_byte"),
        "prefix_byte must be suppressed:\n{text}"
    );
    assert!(
        !text.contains("payload_bytes"),
        "payload_bytes must be suppressed:\n{text}"
    );
}

#[test]
fn inspect_share_text_says_would_combine() {
    let s = a_share();
    Command::cargo_bin("ms")
        .unwrap()
        .args(["inspect", &s])
        .assert()
        .success()
        .stdout(predicate::str::contains("combine"));
}

#[test]
fn inspect_share_json_reports_kind_share() {
    let s = a_share();
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(["inspect", &s, "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).expect("valid json");
    assert_eq!(v["schema_version"], "1");
    assert_eq!(v["report"]["kind"], "share");
    assert_eq!(v["report"]["threshold"], 2);
    assert_eq!(v["report"]["tag"], "tst7"); // the id field
    assert_eq!(v["report"]["share_index"], "p");
    // No would-not-decode FAIL; no threshold-not-zero reason.
    let reasons = v["would_decode"].as_bool();
    assert_eq!(
        reasons,
        Some(true),
        "a share is a valid read (would_combine)"
    );
    // Garbage payload fields suppressed in JSON too.
    assert!(v["report"].get("prefix_byte").is_none() || v["report"]["prefix_byte"].is_null());
    assert!(
        v["report"].get("payload_bytes_hex").is_none()
            || v["report"]["payload_bytes_hex"].is_null()
    );
}

#[test]
fn inspect_v01_single_string_still_works() {
    // Regression: a normal v0.1 single string still inspects as before.
    let mut data = vec![0x00u8];
    data.extend_from_slice(&[0xAAu8; 16]);
    let s = Codex32String::from_seed("ms", 0, "entr", Fe::S, &data)
        .unwrap()
        .to_string();
    Command::cargo_bin("ms")
        .unwrap()
        .args(["inspect", &s])
        .assert()
        .success()
        .stdout(predicate::str::contains("threshold: 0"))
        .stdout(predicate::str::contains("OK: would decode"));
}
