//! Integration tests for `ms derive` (read-only: master fingerprint + account xpub).
//!
//! The all-zeros 16-byte entropy (abandon×11-about) is the corpus. Expected
//! values are independently known: master fp `73c5da0a`; bip84 account xpub
//! `xpub6CatWdiZi…` (the same account the toolkit/mk oracles confirm). No
//! secrets (seed/xprv) ever reach stdout.

use std::process::Output;

use assert_cmd::Command;

mod support;

const ZEROS_HEX: &str = "00000000000000000000000000000000";
const ABANDON: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const MASTER_FP_EN: &str = "73c5da0a";
const MASTER_FP_FR: &str = "7d53dc37";
const BIP84_ACCT_XPUB: &str = "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V";
// T1-b (#10, SPEC_test_hardening_T1_ms_funds_safety.md): independent-oracle
// account-0 xpubs for bip44/49/86, cross-checked at write time (2026-07-10)
// via TWO independent from-scratch derivations (bip32utils, a pure-Python
// BIP32 lib; and a hand-rolled HMAC-SHA512+secp256k1 derivation using only
// the `ecdsa` package's curve arithmetic) — neither touches rust-bitcoin or
// this crate's `purpose()`. bip86 additionally matches the published BIP-86
// spec test vector verbatim (github.com/bitcoin/bips bip-0086.mediawiki,
// "abandon x11 about" account-0 xpub). bip84 already carried an oracled pin
// (BIP84_ACCT_XPUB above); both independent derivations reproduced it
// byte-identically, confirming the oracle methodology.
const BIP44_ACCT_XPUB: &str = "xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj";
const BIP49_ACCT_XPUB: &str = "xpub6C6nQwHaWbSrzs5tZ1q7m5R9cPK9eYpNMFesiXsYrgc1P8bvLLAet9JfHjYXKjToD8cBRswJXXbbFpXgwsswVPAZzKMa1jUp2kVkGVUaJa7";
const BIP86_ACCT_XPUB: &str = "xpub6BgBgsespWvERF3LHQu6CnqdvfEvtMcQjYrcRzx53QJjSxarj2afYWcLteoGVky7D3UKDP9QyrLprQ3VCECoY49yfdDEHGCtMMj92pReUsQ";

fn ms(args: &[&str]) -> Output {
    // P2: material never rides on argv. `support::run` rewrites the invocation
    // onto `--in FILE` / `-` / `--passphrase-stdin` -- the channels an operator
    // uses -- rather than appending `--allow-argv-secret`, which would leave the
    // suite exercising a path the operator never takes.
    support::run(args)
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
    assert!(
        out(&from_phrase).contains(MASTER_FP_EN),
        "{}",
        out(&from_phrase)
    );
}

#[test]
fn account_xpub_bip84_matches_oracle() {
    let o = ms(&["derive", "--hex", ZEROS_HEX, "--template", "bip84"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    let s = out(&o);
    assert!(s.contains(MASTER_FP_EN), "{s}");
    assert!(s.contains(BIP84_ACCT_XPUB), "{s}");
    assert!(s.contains("m/84'/0'/0'"), "{s}");
}

/// T1-b (#10, funds-safety): pin the bip44 end-to-end derive result (master
/// fingerprint + account xpub + path) against an INDEPENDENT oracle (see
/// BIP44_ACCT_XPUB doc comment) — NOT computed via this crate's `purpose()`.
/// `Template::purpose()` is private; this e2e pin is the load-bearing check
/// per SPEC T1-b (a wrong constant corrupts the derived xpub, which this test
/// catches; a wrong constant does NOT corrupt the path string, which mirrors
/// whatever `purpose()` returns — so the xpub is what actually matters).
#[test]
fn account_xpub_bip44_matches_independent_oracle() {
    let o = ms(&["derive", "--hex", ZEROS_HEX, "--template", "bip44"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    let s = out(&o);
    assert!(s.contains(MASTER_FP_EN), "{s}");
    assert!(s.contains(BIP44_ACCT_XPUB), "{s}");
    assert!(s.contains("m/44'/0'/0'"), "{s}");
}

/// T1-b (#10): bip49 end-to-end pin against an independent oracle (see
/// BIP49_ACCT_XPUB doc comment).
#[test]
fn account_xpub_bip49_matches_independent_oracle() {
    let o = ms(&["derive", "--hex", ZEROS_HEX, "--template", "bip49"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    let s = out(&o);
    assert!(s.contains(MASTER_FP_EN), "{s}");
    assert!(s.contains(BIP49_ACCT_XPUB), "{s}");
    assert!(s.contains("m/49'/0'/0'"), "{s}");
}

/// T1-b (#10): bip86 end-to-end pin against the PUBLISHED BIP-86 spec test
/// vector (see BIP86_ACCT_XPUB doc comment) — the strongest-available oracle
/// (an official BIP test vector, not a third-party re-derivation).
#[test]
fn account_xpub_bip86_matches_bip86_spec_vector() {
    let o = ms(&["derive", "--hex", ZEROS_HEX, "--template", "bip86"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    let s = out(&o);
    assert!(s.contains(MASTER_FP_EN), "{s}");
    assert!(s.contains(BIP86_ACCT_XPUB), "{s}");
    assert!(s.contains("m/86'/0'/0'"), "{s}");
}

#[test]
fn account_index_changes_xpub() {
    let a0 = ms(&[
        "derive",
        "--hex",
        ZEROS_HEX,
        "--template",
        "bip84",
        "--account",
        "0",
        "--json",
    ]);
    let a1 = ms(&[
        "derive",
        "--hex",
        ZEROS_HEX,
        "--template",
        "bip84",
        "--account",
        "1",
        "--json",
    ]);
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
    // P2: entropy AND a passphrase are two secret channels and there is one
    // stdin, so the entropy goes through the ms1 card -- the two-command route
    // the argv refusal itself advises.
    let card = ms1_of(ZEROS_HEX);
    let plain = ms(&["derive", &card]);
    let with_pp = ms(&["derive", &card, "--passphrase", "TREZOR"]);
    assert!(out(&plain).contains(MASTER_FP_EN));
    assert!(
        !out(&with_pp).contains(MASTER_FP_EN),
        "passphrase must change fp: {}",
        out(&with_pp)
    );
}

#[test]
fn passphrase_stdin_reads_stdin() {
    let card = ms1_of(ZEROS_HEX);
    let o = support::run_stdin(&["derive", &card, "--passphrase-stdin"], "TREZOR");
    assert_eq!(
        o.status.code().unwrap(),
        0,
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    assert!(
        !String::from_utf8(o.stdout).unwrap().contains(MASTER_FP_EN),
        "passphrase applied"
    );
}

#[test]
fn passphrase_stdin_preserves_multiword_matches_inline() {
    // C1 regression: a multi-word passphrase via stdin must NOT be whitespace-
    // stripped — it must equal the inline --passphrase result for the same bytes.
    let card = ms1_of(ZEROS_HEX);
    let inline = ms(&["derive", &card, "--passphrase", "a b c", "--json"]);
    let from_stdin = support::run_stdin(
        &["derive", &card, "--passphrase-stdin", "--json"],
        "a b c\n",
    );
    let vi: serde_json::Value = serde_json::from_str(&out(&inline)).unwrap();
    let vs: serde_json::Value =
        serde_json::from_str(&String::from_utf8(from_stdin.stdout).unwrap()).unwrap();
    assert_eq!(
        vi["master_fingerprint"], vs["master_fingerprint"],
        "stdin passphrase must match inline"
    );
    // and differ from the no-passphrase fp (proves it was actually applied).
    assert_ne!(vi["master_fingerprint"], MASTER_FP_EN);
}

#[test]
fn single_stdin_guard() {
    // ms1 from stdin + --passphrase-stdin → BadInput (one stdin).
    let card = ms1_of(ZEROS_HEX);
    let o = Command::cargo_bin("ms")
        .unwrap()
        .args(["derive", "--passphrase-stdin"])
        .write_stdin(card)
        .output()
        .unwrap();
    assert_eq!(
        o.status.code().unwrap(),
        1,
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
}

#[test]
fn network_testnet_tpub_same_fingerprint() {
    let main = ms(&[
        "derive",
        "--hex",
        ZEROS_HEX,
        "--template",
        "bip84",
        "--json",
    ]);
    let test = ms(&[
        "derive",
        "--hex",
        ZEROS_HEX,
        "--template",
        "bip84",
        "--network",
        "testnet",
        "--json",
    ]);
    let vm: serde_json::Value = serde_json::from_str(&out(&main)).unwrap();
    let vt: serde_json::Value = serde_json::from_str(&out(&test)).unwrap();
    assert_eq!(
        vm["master_fingerprint"], vt["master_fingerprint"],
        "fp network-independent"
    );
    assert!(
        vt["account_xpub"].as_str().unwrap().starts_with("tpub"),
        "{}",
        vt["account_xpub"]
    );
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
    let o = ms(&[
        "derive",
        "--hex",
        ZEROS_HEX,
        "--template",
        "bip84",
        "--json",
    ]);
    let v: serde_json::Value = serde_json::from_str(&out(&o)).unwrap();
    assert_eq!(v["schema_version"], "1");
    assert_eq!(v["master_fingerprint"], MASTER_FP_EN);
    assert_eq!(v["network"], "mainnet");
    assert_eq!(v["account_xpub"], BIP84_ACCT_XPUB);
    assert_eq!(v["language_defaulted"], true);
    // no-template → account fields omitted (skip_serializing_if)
    let nt = ms(&["derive", "--hex", ZEROS_HEX, "--json"]);
    let vnt: serde_json::Value = serde_json::from_str(&out(&nt)).unwrap();
    assert!(
        vnt.get("account_xpub").is_none(),
        "omitted without --template"
    );
}

#[test]
fn no_secret_on_stdout() {
    // PUBLIC-only boundary: stdout never carries an xprv/tprv or a 64-byte seed.
    let o = ms(&[
        "derive",
        "--hex",
        ZEROS_HEX,
        "--template",
        "bip84",
        "--network",
        "testnet",
    ]);
    let s = out(&o);
    assert!(!s.contains("xprv"), "{s}");
    assert!(!s.contains("tprv"), "{s}");
}

/// **REWRITTEN, not deleted.** This test's subject was the stderr advisory
/// `derive` printed while proceeding: `ms derive --hex <entropy>` exited 0 and
/// warned. P2 replaces warn-and-proceed with a refusal that decides before the
/// parser, so the subject is now the refusal — and the assertion is stronger,
/// because a warning that still exits 0 is what this phase exists to remove.
#[test]
fn inline_secret_on_argv_is_now_refused_not_merely_advised() {
    let o = Command::cargo_bin("ms")
        .unwrap()
        .args(["derive", "--hex", ZEROS_HEX])
        .output()
        .unwrap();
    assert_eq!(
        o.status.code(),
        Some(1),
        "argv material must be REFUSED, not warned about: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    let e = String::from_utf8_lossy(&o.stderr);
    assert!(
        e.contains("Refused BEFORE the command line was parsed"),
        "{e}"
    );
    assert!(
        !e.contains(ZEROS_HEX),
        "the refusal must name the CLASS and the LENGTH, never the value: {e}"
    );
    assert!(
        String::from_utf8_lossy(&o.stdout).is_empty(),
        "nothing was read and nothing was written"
    );
}

/// Wave-2 ms lane (slug `ms-cli-derive-xpriv-master-not-zeroized`, in-repo
/// leg) — the derived `master`/`acct_xpriv` `Xpriv` values are now confined in
/// a binary-private move-only `ScrubbedXpriv` newtype that byte-scrubs on drop.
/// `master_fingerprint` and `account_xpub` are materialized (`.to_string()`)
/// BEFORE either wrapper drops, so the scrub is output-invisible. This pins the
/// byte-identical-output invariant across the scrub rewire: a regression in the
/// rewire (reading a stale/zeroed value, or moving the materialize past the
/// scrub) surfaces here as a golden mismatch. Anchors on `MASTER_FP_EN` +
/// `BIP84_ACCT_XPUB` (text + `--json`, fingerprint-only + account-xpub paths).
#[test]
fn scrub_rewire_leaves_output_byte_identical() {
    // (a) fingerprint-only (text) — master fp unchanged.
    let fp_text = ms(&["derive", "--hex", ZEROS_HEX, "--language", "english"]);
    assert_eq!(code(&fp_text), 0, "{}", err(&fp_text));
    assert!(
        out(&fp_text).contains(&format!("master_fingerprint:  {MASTER_FP_EN}")),
        "fingerprint-only text output changed by scrub rewire: {}",
        out(&fp_text)
    );

    // (b) fingerprint-only (--json) — exact field value unchanged.
    let fp_json = ms(&[
        "derive",
        "--hex",
        ZEROS_HEX,
        "--language",
        "english",
        "--json",
    ]);
    let vj: serde_json::Value = serde_json::from_str(&out(&fp_json)).unwrap();
    assert_eq!(vj["master_fingerprint"], MASTER_FP_EN);
    assert!(
        vj.get("account_xpub").is_none(),
        "no account without --template: {}",
        out(&fp_json)
    );

    // (c) account-xpub bip84 (text) — both fp and account xpub unchanged.
    let acct_text = ms(&[
        "derive",
        "--hex",
        ZEROS_HEX,
        "--language",
        "english",
        "--template",
        "bip84",
    ]);
    assert_eq!(code(&acct_text), 0, "{}", err(&acct_text));
    let at = out(&acct_text);
    assert!(at.contains(MASTER_FP_EN), "account text fp changed: {at}");
    assert!(
        at.contains(BIP84_ACCT_XPUB),
        "account xpub changed by scrub rewire: {at}"
    );

    // (d) account-xpub bip84 (--json) — exact field values unchanged.
    let acct_json = ms(&[
        "derive",
        "--hex",
        ZEROS_HEX,
        "--language",
        "english",
        "--template",
        "bip84",
        "--json",
    ]);
    let va: serde_json::Value = serde_json::from_str(&out(&acct_json)).unwrap();
    assert_eq!(va["master_fingerprint"], MASTER_FP_EN);
    assert_eq!(va["account_xpub"], BIP84_ACCT_XPUB);
    assert_eq!(va["account_path"], "m/84'/0'/0'");
}
