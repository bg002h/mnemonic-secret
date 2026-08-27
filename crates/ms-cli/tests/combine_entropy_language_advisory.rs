//! L26 — `ms combine --to entropy` drops the wordlist language with NO advisory.
//!
//! The recovered hex is correct, but `--to entropy` carries only the entropy,
//! not the wordlist language — recovering it with English-defaulted software
//! derives a DIFFERENT seed. Fix: a stderr advisory on the `--to entropy` arm
//! ONLY (for non-English mnem shares). `--to phrase` (re-renders in-language)
//! and `--to ms1` (re-encodes the mnem payload) preserve the language → no
//! advisory. English → no advisory (self-recovers as the universal default).

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

mod support;

const ENGLISH_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

const ADVISORY_SUBSTR: &str = "encoding a japanese BIP-39 seed as raw entropy";

fn japanese_12_word() -> String {
    bip39::Mnemonic::from_entropy_in(bip39::Language::Japanese, &[0xABu8; 16])
        .expect("valid entropy length")
        .to_string()
}

/// Split a source into k-of-n shares (returns the share strings).
fn split_shares(source_args: &[&str], k: &str, n: &str) -> Vec<String> {
    let mut args = vec!["split"];
    args.extend_from_slice(source_args);
    args.extend_from_slice(&["-k", k, "-n", n, "--json"]);
    let o = support::run(&args);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let out = o.stdout;
    let v: Value = serde_json::from_slice(&out).unwrap();
    v["shares"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

/// RED→GREEN: combine Japanese shares `--to entropy` → correct hex on stdout
/// AND the advisory on stderr.
#[test]
fn japanese_to_entropy_emits_advisory() {
    let ja = japanese_12_word();
    let shares = split_shares(&["--language", "japanese", "--phrase", &ja], "2", "3");
    let expected_hex = "ab".repeat(16);
    Command::cargo_bin("ms")
        .unwrap()
        .args(["combine", "-", "--to", "entropy"])
        .write_stdin([shares[0].to_string(), shares[2].to_string()].join("\n"))
        .assert()
        .success()
        .stdout(predicate::str::contains(&expected_hex))
        .stderr(predicate::str::contains(ADVISORY_SUBSTR));
}

/// Arm-selective: Japanese `--to phrase` → NO advisory.
#[test]
fn japanese_to_phrase_no_advisory() {
    let ja = japanese_12_word();
    let shares = split_shares(&["--language", "japanese", "--phrase", &ja], "2", "3");
    Command::cargo_bin("ms")
        .unwrap()
        .args(["combine", "-", "--to", "phrase"])
        .write_stdin([shares[0].to_string(), shares[1].to_string()].join("\n"))
        .assert()
        .success()
        .stdout(predicate::str::contains(&ja))
        .stderr(predicate::str::contains(ADVISORY_SUBSTR).not());
}

/// Arm-selective: Japanese `--to ms1` → NO advisory.
#[test]
fn japanese_to_ms1_no_advisory() {
    let ja = japanese_12_word();
    let shares = split_shares(&["--language", "japanese", "--phrase", &ja], "2", "3");
    Command::cargo_bin("ms")
        .unwrap()
        .args(["combine", "-", "--to", "ms1"])
        .write_stdin([shares[0].to_string(), shares[1].to_string()].join("\n"))
        .assert()
        .success()
        .stderr(predicate::str::contains(ADVISORY_SUBSTR).not());
}

/// `--to ms1` language-byte preservation: the re-emitted ms1 decodes back to a
/// Mnem payload carrying the same Japanese language byte.
#[test]
fn japanese_to_ms1_preserves_language_byte() {
    let ja = japanese_12_word();
    let shares = split_shares(&["--language", "japanese", "--phrase", &ja], "2", "3");

    // Re-emitted ms1 via --json.
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(["combine", "-", "--to", "ms1", "--json"])
        .write_stdin([shares[0].to_string(), shares[1].to_string()].join("\n"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    let ms1 = v["ms1"].as_str().expect("ms1 field present");

    // Decode it back — must still be a Japanese mnem card.
    Command::cargo_bin("ms")
        .unwrap()
        .args(["decode", "-"])
        .write_stdin((ms1).to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains("language: japanese"))
        .stdout(predicate::str::contains(&ja));
}

/// English control: English shares `--to entropy` → NO advisory.
#[test]
fn english_to_entropy_no_advisory() {
    let shares = split_shares(&["--phrase", ENGLISH_12], "2", "3");
    Command::cargo_bin("ms")
        .unwrap()
        .args(["combine", "-", "--to", "entropy"])
        .write_stdin([shares[0].to_string(), shares[1].to_string()].join("\n"))
        .assert()
        .success()
        .stderr(predicate::str::contains("BIP-39 seed as raw entropy").not());
}

/// `--json` unchanged: `--to entropy --json` → stdout language null/absent,
/// advisory still on stderr.
#[test]
fn json_wire_shape_unchanged_advisory_on_stderr() {
    let ja = japanese_12_word();
    let shares = split_shares(&["--language", "japanese", "--phrase", &ja], "2", "3");
    let assert = Command::cargo_bin("ms")
        .unwrap()
        .args(["combine", "-", "--to", "entropy", "--json"])
        .write_stdin([shares[0].to_string(), shares[2].to_string()].join("\n"))
        .assert()
        .success()
        .stderr(predicate::str::contains(ADVISORY_SUBSTR));
    let out = assert.get_output().stdout.clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    // language is None (null or absent) for the entropy arm — unchanged wire shape.
    assert!(
        v.get("language").map(|x| x.is_null()).unwrap_or(true),
        "entropy --json language must stay null: {v}"
    );
}
