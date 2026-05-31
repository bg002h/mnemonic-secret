//! Integration tests for `ms derive` (read-only: master fingerprint + account xpub).
//!
//! The all-zeros 16-byte entropy (abandon×11-about) is the corpus. Expected
//! values are independently known: master fp `73c5da0a`; bip84 account xpub
//! `xpub6CatWdiZi…` (the same account the toolkit/mk oracles confirm). No
//! secrets (seed/xprv) ever reach stdout.

use std::process::Output;

use assert_cmd::Command;

const ZEROS_HEX: &str = "00000000000000000000000000000000";
const ABANDON: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const MASTER_FP_EN: &str = "73c5da0a";
const MASTER_FP_FR: &str = "7d53dc37";
const BIP84_ACCT_XPUB: &str = "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V";

fn ms(args: &[&str]) -> Output {
    Command::cargo_bin("ms").unwrap().args(args).output().unwrap()
}
fn out(o: &Output) -> String {
    String::from_utf8(o.stdout.clone()).unwrap()
}
fn err(o: &Output) -> String {
    String::from_utf8(o.stderr.clone()).unwrap()
}
fn code(o: &Output) -> i32 {
    o.status.code().unwrap()
}

/// Build an ms1 string for hex entropy via `ms encode`.
fn ms1_of(hex: &str) -> String {
    let o = ms(&["encode", "--hex", hex]);
    assert!(o.status.success(), "encode: {}", err(&o));
    out(&o).lines().next().unwrap().trim().to_string()
}

#[test]
fn fingerprint_from_ms1() {
    let card = ms1_of(ZEROS_HEX);
    let o = ms(&["derive", &card]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert!(out(&o).contains(MASTER_FP_EN), "{}", out(&o));
}

#[test]
fn hex_and_phrase_parity() {
    let from_hex = ms(&["derive", "--hex", ZEROS_HEX]);
    let from_phrase = ms(&["derive", "--phrase", ABANDON]);
    assert!(out(&from_hex).contains(MASTER_FP_EN), "{}", out(&from_hex));
    assert!(out(&from_phrase).contains(MASTER_FP_EN), "{}", out(&from_phrase));
}

#[test]
fn account_xpub_bip84_matches_oracle() {
    let o = ms(&["derive", "--hex", ZEROS_HEX, "--template", "bip84"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    let s = out(&o);
    assert!(s.contains(BIP84_ACCT_XPUB), "{s}");
    assert!(s.contains("m/84'/0'/0'"), "{s}");
}

#[test]
fn account_index_changes_xpub() {
    let a0 = ms(&["derive", "--hex", ZEROS_HEX, "--template", "bip84", "--account", "0", "--json"]);
    let a1 = ms(&["derive", "--hex", ZEROS_HEX, "--template", "bip84", "--account", "1", "--json"]);
    let v0: serde_json::Value = serde_json::from_str(&out(&a0)).unwrap();
    let v1: serde_json::Value = serde_json::from_str(&out(&a1)).unwrap();
    assert_ne!(v0["account_xpub"], v1["account_xpub"]);
    assert_eq!(v1["account_path"], "m/84'/0'/1'");
}

#[test]
fn no_template_no_account_line() {
    let o = ms(&["derive", "--hex", ZEROS_HEX]);
    let s = out(&o);
    assert!(s.contains(MASTER_FP_EN));
    assert!(!s.contains("account_xpub"), "{s}");
}

#[test]
fn language_is_load_bearing() {
    let en = ms(&["derive", "--hex", ZEROS_HEX, "--language", "english"]);
    let fr = ms(&["derive", "--hex", ZEROS_HEX, "--language", "french"]);
    assert!(out(&en).contains(MASTER_FP_EN));
    assert!(out(&fr).contains(MASTER_FP_FR));
    assert_ne!(MASTER_FP_EN, MASTER_FP_FR);
}

#[test]
fn default_language_annotated() {
    let o = ms(&["derive", "--hex", ZEROS_HEX]);
    assert!(out(&o).contains("DEFAULT"), "stdout: {}", out(&o));
    assert!(err(&o).contains("defaulted"), "stderr: {}", err(&o));
    // explicit language → no DEFAULT
    let ex = ms(&["derive", "--hex", ZEROS_HEX, "--language", "english"]);
    assert!(!out(&ex).contains("DEFAULT"), "{}", out(&ex));
}

#[test]
fn passphrase_changes_fingerprint() {
    let plain = ms(&["derive", "--hex", ZEROS_HEX]);
    let with_pp = ms(&["derive", "--hex", ZEROS_HEX, "--passphrase", "TREZOR"]);
    assert!(out(&plain).contains(MASTER_FP_EN));
    assert!(!out(&with_pp).contains(MASTER_FP_EN), "passphrase must change fp: {}", out(&with_pp));
}

#[test]
fn passphrase_stdin_reads_stdin() {
    let o = Command::cargo_bin("ms").unwrap()
        .args(["derive", "--hex", ZEROS_HEX, "--passphrase-stdin"])
        .write_stdin("TREZOR").output().unwrap();
    assert_eq!(o.status.code().unwrap(), 0, "{}", String::from_utf8_lossy(&o.stderr));
    assert!(!String::from_utf8(o.stdout).unwrap().contains(MASTER_FP_EN), "passphrase applied");
}

#[test]
fn passphrase_stdin_preserves_multiword_matches_inline() {
    // C1 regression: a multi-word passphrase via stdin must NOT be whitespace-
    // stripped — it must equal the inline --passphrase result for the same bytes.
    let inline = ms(&["derive", "--hex", ZEROS_HEX, "--passphrase", "a b c", "--json"]);
    let from_stdin = Command::cargo_bin("ms").unwrap()
        .args(["derive", "--hex", ZEROS_HEX, "--passphrase-stdin", "--json"])
        .write_stdin("a b c\n").output().unwrap();
    let vi: serde_json::Value = serde_json::from_str(&out(&inline)).unwrap();
    let vs: serde_json::Value = serde_json::from_str(&String::from_utf8(from_stdin.stdout).unwrap()).unwrap();
    assert_eq!(vi["master_fingerprint"], vs["master_fingerprint"], "stdin passphrase must match inline");
    // and differ from the no-passphrase fp (proves it was actually applied).
    assert_ne!(vi["master_fingerprint"], MASTER_FP_EN);
}

#[test]
fn single_stdin_guard() {
    // ms1 from stdin + --passphrase-stdin → BadInput (one stdin).
    let card = ms1_of(ZEROS_HEX);
    let o = Command::cargo_bin("ms").unwrap()
        .args(["derive", "--passphrase-stdin"])
        .write_stdin(card).output().unwrap();
    assert_eq!(o.status.code().unwrap(), 1, "{}", String::from_utf8_lossy(&o.stderr));
}

#[test]
fn network_testnet_tpub_same_fingerprint() {
    let main = ms(&["derive", "--hex", ZEROS_HEX, "--template", "bip84", "--json"]);
    let test = ms(&["derive", "--hex", ZEROS_HEX, "--template", "bip84", "--network", "testnet", "--json"]);
    let vm: serde_json::Value = serde_json::from_str(&out(&main)).unwrap();
    let vt: serde_json::Value = serde_json::from_str(&out(&test)).unwrap();
    assert_eq!(vm["master_fingerprint"], vt["master_fingerprint"], "fp network-independent");
    assert!(vt["account_xpub"].as_str().unwrap().starts_with("tpub"), "{}", vt["account_xpub"]);
    assert_eq!(vt["account_path"], "m/84'/1'/0'");
}

#[test]
fn input_exclusivity() {
    let card = ms1_of(ZEROS_HEX);
    let o = ms(&["derive", &card, "--hex", ZEROS_HEX]); // ms1 + --hex
    assert_eq!(code(&o), 64, "{}", err(&o)); // clap conflict → ms-cli catch-all 64
    assert_ne!(code(&ms(&["derive", "--hex", "zz"])), 0); // bad hex
}

#[test]
fn json_shape() {
    let o = ms(&["derive", "--hex", ZEROS_HEX, "--template", "bip84", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&out(&o)).unwrap();
    assert_eq!(v["schema_version"], "1");
    assert_eq!(v["master_fingerprint"], MASTER_FP_EN);
    assert_eq!(v["network"], "mainnet");
    assert_eq!(v["account_xpub"], BIP84_ACCT_XPUB);
    assert_eq!(v["language_defaulted"], true);
    // no-template → account fields omitted (skip_serializing_if)
    let nt = ms(&["derive", "--hex", ZEROS_HEX, "--json"]);
    let vnt: serde_json::Value = serde_json::from_str(&out(&nt)).unwrap();
    assert!(vnt.get("account_xpub").is_none(), "omitted without --template");
}

#[test]
fn no_secret_on_stdout() {
    // PUBLIC-only boundary: stdout never carries an xprv/tprv or a 64-byte seed.
    let o = ms(&["derive", "--hex", ZEROS_HEX, "--template", "bip84", "--network", "testnet"]);
    let s = out(&o);
    assert!(!s.contains("xprv"), "{s}");
    assert!(!s.contains("tprv"), "{s}");
}

#[test]
fn inline_secret_argv_advisory() {
    let o = ms(&["derive", "--hex", ZEROS_HEX]);
    assert!(err(&o).contains("secret material on argv (--hex)"), "{}", err(&o));
}
