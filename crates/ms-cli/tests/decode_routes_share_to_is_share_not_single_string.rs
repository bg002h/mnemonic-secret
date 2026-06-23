//! SPEC_ms_v0_2_kofn §3 (decode threshold-routing): integration coverage.
//!
//! v0.2 RE-SPEC of the former `decode_rejects_threshold_not_zero` test. A
//! threshold=2 codex32 string IS one share of a K-of-N share-set — NOT a
//! malformed v0.1 single-string. ms-codec `decode` now routes such a string to
//! `Error::IsShareNotSingleString` (the user should run `ms combine`), NOT the
//! v0.1 `ThresholdNotZero` hard-reject.
//!
//! ms-cli surfaces `IsShareNotSingleString` as a FormatViolation-class error
//! (exit 2), with a message directing the user to `ms combine`, and JSON
//! `kind: "IsShareNotSingleString"`. This is the Task-2.0 mapping (proven
//! correct against the share semantics, NOT a re-capture of the prior
//! exit-1 "unhandled variant" wildcard behavior).
//!
//! Hand-build a threshold=2 + share=Fe::A string (codex32 lib accepts arbitrary
//! threshold + share at construction — that string is a genuine, well-formed
//! K-of-N share).

use assert_cmd::Command;
use ms_codec::codex32::{Codex32String, Fe};
use predicates::prelude::*;

/// A threshold=2 / index=A string is a genuine share; `decode` must route it to
/// `ms combine`, not reject it as a malformed single-string.
#[test]
fn decode_routes_share_to_is_share_not_single_string() {
    let mut data = vec![0x00u8];
    data.extend_from_slice(&[0x00u8; 16]);
    let s = Codex32String::from_seed("ms", 2, "tst7", Fe::A, &data)
        .unwrap()
        .to_string();
    Command::cargo_bin("ms")
        .unwrap()
        .args(["decode", &s])
        .assert()
        .failure()
        .code(2)
        // The Display message names this as one share of a K-of-N set and
        // points the user at `ms combine`.
        .stderr(predicate::str::contains("ms combine"))
        .stderr(predicate::str::contains("share"));
}

#[test]
fn decode_routes_share_to_is_share_not_single_string_json_envelope() {
    let mut data = vec![0x00u8];
    data.extend_from_slice(&[0x00u8; 16]);
    let s = Codex32String::from_seed("ms", 2, "tst7", Fe::A, &data)
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
    assert_eq!(v["error"]["kind"], "IsShareNotSingleString");
    assert_eq!(v["error"]["exit_code"], 2);
    // Structured details carry the observed threshold + index chars.
    assert_eq!(v["error"]["details"]["threshold"], "2");
    assert_eq!(v["error"]["details"]["index"], "a");
}
