//! `ms split` integration coverage (Task 2.1).
//!
//! Splits a secret (entr or mnem) into N codex32 K-of-N shares; any K recombine
//! via `ms combine`. The share SET is secret-equivalent (PrivateKeyMaterial
//! advisory). Bounds: 2 ≤ k ≤ n ≤ 31.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

const ENGLISH_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fn japanese_12_word() -> String {
    bip39::Mnemonic::from_entropy_in(bip39::Language::Japanese, &[0xABu8; 16])
        .expect("valid entropy length")
        .to_string()
}

/// Re-parse a share string → (threshold_char, share_index_char, id).
fn share_header(s: &str) -> (char, char, String) {
    let sep = s.rfind('1').unwrap();
    let b = s.as_bytes();
    let threshold = b[sep + 1] as char;
    let id: String = s[sep + 2..sep + 6].to_string();
    let index = b[sep + 6] as char;
    (threshold, index, id)
}

#[test]
fn split_english_phrase_emits_n_shares_text() {
    // `--group-size 0` keeps shares unbroken so the bare-share parse holds
    // (default is now space/5; labels moved to stderr — mstring-grouping P2).
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args([
            "split",
            "--phrase",
            ENGLISH_12,
            "-k",
            "2",
            "-n",
            "3",
            "--group-size",
            "0",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    // Each non-empty stdout line is a bare share string (no labels on stdout).
    let shares: Vec<&str> = s
        .lines()
        .filter(|l| l.starts_with("ms1") && !l.contains(' '))
        .collect();
    assert_eq!(shares.len(), 3, "expected 3 share strings; got:\n{s}");
    let mut indices = Vec::new();
    let mut ids = Vec::new();
    for sh in &shares {
        let (thr, idx, id) = share_header(sh);
        assert_eq!(thr, '2', "threshold char");
        assert_ne!(idx, 's', "distributed share must not be index s");
        indices.push(idx);
        ids.push(id);
    }
    let mut sorted = indices.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), 3, "distinct indices");
    assert!(ids.windows(2).all(|w| w[0] == w[1]), "shared id");
}

#[test]
fn split_emits_private_key_material_advisory() {
    Command::cargo_bin("ms")
        .unwrap()
        .args(["split", "--phrase", ENGLISH_12, "-k", "2", "-n", "3"])
        .assert()
        .success()
        .stderr(predicate::str::contains("private key material"));
}

#[test]
fn split_json_shape() {
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args([
            "split",
            "--hex",
            &"ab".repeat(16),
            "-k",
            "3",
            "-n",
            "5",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).expect("valid json");
    assert_eq!(v["schema_version"], "1");
    assert_eq!(v["k"], 3);
    assert_eq!(v["n"], 5);
    assert_eq!(v["kind"], "entr");
    assert_eq!(v["shares"].as_array().unwrap().len(), 5);
    assert!(v["id"].is_string());
    // hex → entr → no language field.
    assert!(v.get("language").is_none() || v["language"].is_null());
}

#[test]
fn split_japanese_phrase_json_has_language_and_mnem_kind() {
    let ja = japanese_12_word();
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args([
            "split",
            "--language",
            "japanese",
            "--phrase",
            &ja,
            "-k",
            "2",
            "-n",
            "3",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).expect("valid json");
    assert_eq!(v["kind"], "mnem");
    assert_eq!(v["language"], "japanese");
    assert_eq!(v["shares"].as_array().unwrap().len(), 3);
}

#[test]
fn split_k_below_2_rejected() {
    Command::cargo_bin("ms")
        .unwrap()
        .args(["split", "--phrase", ENGLISH_12, "-k", "1", "-n", "3"])
        .assert()
        .failure()
        .code(1); // InvalidThreshold → BadInput (exit 1)
}

#[test]
fn split_n_below_k_rejected() {
    Command::cargo_bin("ms")
        .unwrap()
        .args(["split", "--phrase", ENGLISH_12, "-k", "3", "-n", "2"])
        .assert()
        .failure()
        .code(1); // InvalidShareCount → BadInput (exit 1)
}

#[test]
fn split_n_above_31_rejected() {
    Command::cargo_bin("ms")
        .unwrap()
        .args(["split", "--phrase", ENGLISH_12, "-k", "2", "-n", "32"])
        .assert()
        .failure()
        .code(1); // InvalidShareCount → BadInput (exit 1)
}
