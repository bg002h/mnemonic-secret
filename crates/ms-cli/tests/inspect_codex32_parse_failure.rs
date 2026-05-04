//! Inspect on a string that fails BIP-93 parse → exit 1 with Codex32 error
//! per SPEC §2.3.1.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn inspect_bad_checksum_exits_1_with_friendly_error() {
    // Take a valid string and flip the last char to break BCH.
    let mut bytes = b"ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f".to_vec();
    let last = bytes.len() - 1;
    bytes[last] = if bytes[last] == b'q' { b'p' } else { b'q' };
    let bad = String::from_utf8(bytes).unwrap();

    Command::cargo_bin("ms")
        .unwrap()
        .args(["inspect", &bad])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("BCH checksum invalid"));
}

#[test]
fn inspect_bad_checksum_json_envelope() {
    let mut bytes = b"ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f".to_vec();
    let last = bytes.len() - 1;
    bytes[last] = if bytes[last] == b'q' { b'p' } else { b'q' };
    let bad = String::from_utf8(bytes).unwrap();

    Command::cargo_bin("ms")
        .unwrap()
        .args(["inspect", &bad, "--json"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("\"kind\":\"Codex32\""))
        .stdout(predicate::str::contains("\"schema_version\":\"1\""));
}
