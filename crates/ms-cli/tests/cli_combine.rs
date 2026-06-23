//! `ms combine` integration coverage (Task 2.2).
//!
//! Recombines K-of-N codex32 shares (produced by `ms split`) back into the
//! original secret. Default `--to phrase`; also `--to entropy` / `--to ms1`.
//! Errors (below-threshold / index-s / duplicate / mismatched) surface via the
//! Task-2.0 mapping + codex32_friendly. The recovered secret is
//! PrivateKeyMaterial (advisory) and Zeroizing-wrapped.

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

/// Split a source into k-of-n shares, returning the share strings.
fn split_shares(source_args: &[&str], k: &str, n: &str) -> Vec<String> {
    let mut args = vec!["split"];
    args.extend_from_slice(source_args);
    args.extend_from_slice(&["-k", k, "-n", n, "--json"]);
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(&args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    v["shares"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

/// Insert a comma every 5 chars (a separator neither ms-codec nor the legacy
/// `strip_whitespace` removed — genuinely exercises `strip_display_separators`).
fn comma5(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && i % 5 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

#[test]
fn combine_accepts_comma_grouped_positional_shares() {
    // mstring-grouping P2 (SPEC §15 C3): grouped positional shares re-ingest.
    let shares = split_shares(&["--phrase", ENGLISH_12], "2", "3");
    let g0 = comma5(&shares[0]);
    let g2 = comma5(&shares[2]);
    Command::cargo_bin("ms")
        .unwrap()
        .args(["combine", &g0, &g2])
        .assert()
        .success()
        .stdout(predicate::str::contains(ENGLISH_12));
}

#[test]
fn combine_dash_stdin_round_trips() {
    // mstring-grouping P2 (SPEC §15 C1): `ms combine -` reads one share per line
    // from stdin; comma-grouped lines strip + re-ingest.
    let shares = split_shares(&["--phrase", ENGLISH_12], "2", "3");
    let stdin = format!("{}\n{}\n", comma5(&shares[0]), comma5(&shares[1]));
    Command::cargo_bin("ms")
        .unwrap()
        .args(["combine", "-"])
        .write_stdin(stdin)
        .assert()
        .success()
        .stdout(predicate::str::contains(ENGLISH_12));
}

#[test]
fn combine_english_round_trip_to_phrase() {
    let shares = split_shares(&["--phrase", ENGLISH_12], "2", "3");
    // Any 2 of 3 recover the english phrase (default --to phrase).
    Command::cargo_bin("ms")
        .unwrap()
        .args(["combine", &shares[0], &shares[2]])
        .assert()
        .success()
        .stdout(predicate::str::contains(ENGLISH_12));
}

#[test]
fn combine_japanese_round_trip_preserves_language() {
    let ja = japanese_12_word();
    let shares = split_shares(&["--language", "japanese", "--phrase", &ja], "2", "3");
    // mnem share-set → recovered phrase is in the wire language (japanese).
    Command::cargo_bin("ms")
        .unwrap()
        .args(["combine", &shares[1], &shares[2]])
        .assert()
        .success()
        .stdout(predicate::str::contains(&ja))
        .stdout(predicate::str::contains("japanese"));
}

#[test]
fn combine_to_entropy_emits_hex() {
    let shares = split_shares(&["--hex", &"ab".repeat(16)], "2", "3");
    Command::cargo_bin("ms")
        .unwrap()
        .args(["combine", &shares[0], &shares[1], "--to", "entropy"])
        .assert()
        .success()
        .stdout(predicate::str::contains("abababababababababababababababab"));
}

#[test]
fn combine_to_ms1_emits_single_string_that_decodes() {
    let shares = split_shares(&["--phrase", ENGLISH_12], "2", "3");
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(["combine", &shares[0], &shares[1], "--to", "ms1"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let ms1 = s
        .lines()
        .find(|l| l.starts_with("ms1") && !l.contains(' '))
        .expect("an ms1 line");
    // The recovered single ms1 must decode back to the english phrase.
    Command::cargo_bin("ms")
        .unwrap()
        .args(["decode", ms1])
        .assert()
        .success()
        .stdout(predicate::str::contains(ENGLISH_12));
}

#[test]
fn combine_json_shape_entr() {
    let shares = split_shares(&["--hex", &"ab".repeat(16)], "2", "3");
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(["combine", &shares[0], &shares[1], "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["schema_version"], "1");
    assert_eq!(v["kind"], "entr");
    assert_eq!(v["entropy_hex"], "abababababababababababababababab");
    // default --to phrase → phrase present (english for entr).
    assert!(v["phrase"].is_string());
    assert_eq!(v["language"], "english");
}

#[test]
fn combine_below_threshold_friendly_error() {
    let shares = split_shares(&["--phrase", ENGLISH_12], "3", "4");
    // Only 2 of a 3-of-4 set → ThresholdNotPassed friendly message.
    Command::cargo_bin("ms")
        .unwrap()
        .args(["combine", &shares[0], &shares[1]])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not enough shares"));
}

#[test]
fn combine_secret_share_index_s_rejected() {
    // Build the secret-at-S directly (index 's', threshold 2) + one real share.
    let shares = split_shares(&["--phrase", ENGLISH_12], "2", "3");
    // Construct a secret-at-S string with matching id/threshold so only the
    // index-s axis differs.
    let (_, _, id) = {
        let sh = &shares[0];
        let sep = sh.rfind('1').unwrap();
        (
            sh.as_bytes()[sep + 1] as char,
            sh.as_bytes()[sep + 6] as char,
            sh[sep + 2..sep + 6].to_string(),
        )
    };
    let mut data = vec![0x00u8];
    data.extend_from_slice(&[0x00u8; 16]);
    let secret_s = codex32::Codex32String::from_seed("ms", 2, &id, codex32::Fe::S, &data)
        .unwrap()
        .to_string();
    Command::cargo_bin("ms")
        .unwrap()
        .args(["combine", &secret_s, &shares[0]])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("secret share"));
}

#[test]
fn combine_duplicate_index_rejected() {
    let shares = split_shares(&["--phrase", ENGLISH_12], "2", "3");
    // Same share twice → RepeatedIndex friendly message.
    Command::cargo_bin("ms")
        .unwrap()
        .args(["combine", &shares[0], &shares[0]])
        .assert()
        .failure()
        .stderr(predicate::str::contains("repeated"));
}
