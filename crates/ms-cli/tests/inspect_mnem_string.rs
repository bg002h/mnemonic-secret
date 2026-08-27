//! `ms inspect` on a valid mnem (0x02) string:
//! - exit 0
//! - reports kind: mnem + language: japanese
//! - does NOT print non-zero-prefix / unexpected-string-length /
//!   payload-length-mismatch / FAIL

use assert_cmd::Command;
use predicates::prelude::*;

mod support;

/// Build a valid Japanese mnem ms1 from 16 entropy bytes (0xAB repeated).
fn japanese_mnem_ms1() -> String {
    let ja = bip39::Mnemonic::from_entropy_in(bip39::Language::Japanese, &[0xABu8; 16])
        .unwrap()
        .to_string();
    let encode_out = Command::cargo_bin("ms")
        .unwrap()
        .args(["encode", "--language", "japanese", "--phrase", "-"])
        .write_stdin(ja.to_string())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    std::str::from_utf8(&encode_out)
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .to_string()
}

#[test]
fn inspect_mnem_string_text_mode_ok() {
    let ms1 = japanese_mnem_ms1();
    Command::cargo_bin("ms")
        .unwrap()
        .args(["inspect", "-"])
        .write_stdin(ms1.to_string())
        .assert()
        .success()
        .stdout(predicate::str::starts_with("OK:"))
        .stdout(predicate::str::contains("kind: mnem"))
        .stdout(predicate::str::contains("language: japanese"))
        .stdout(predicate::str::contains("non-zero-prefix").not())
        .stdout(predicate::str::contains("unexpected-string-length").not())
        .stdout(predicate::str::contains("payload-length-mismatch").not())
        .stdout(predicate::str::contains("FAIL").not());
}

#[test]
fn inspect_mnem_string_json_mode_ok() {
    let ms1 = japanese_mnem_ms1();
    let o = support::run(&["inspect", "--json", &ms1]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let so = String::from_utf8_lossy(&o.stdout);
    for want in [
        "\"would_decode\":true",
        "\"failure_reasons\":[]",
        "\"kind\":\"mnem\"",
        "\"language\":\"japanese\"",
    ] {
        assert!(so.contains(want), "missing {want} in {so}");
    }
}

/// Existing v0.1 entr string → inspect unchanged (kind: entr, no language field).
#[test]
fn inspect_entr_string_unchanged() {
    Command::cargo_bin("ms")
        .unwrap()
        .args(["inspect", "-"])
        .write_stdin(("ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f").to_string())
        .assert()
        .success()
        .stdout(predicate::str::starts_with("OK:"))
        .stdout(predicate::str::contains("kind: entr"));
}
