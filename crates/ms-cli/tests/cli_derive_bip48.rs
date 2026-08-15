//! `ms derive --template bip48-*` — multisig account xpubs (BIP-48).
//!
//! # Why these templates exist
//!
//! An operator building a multisig backup knows one thing about their key:
//! which SCRIPT TYPE the wallet uses. They should not also have to know that
//! native segwit multisig lives at `m/48'/0'/0'/2'` and nested segwit at
//! `.../1'`. Before this landed, `ms derive --template` offered only
//! bip44/49/84/86 — all single-sig — and accepted no literal path, so there was
//! NO way to get a multisig account xpub out of `ms` at all. The one tool in
//! the constellation that turns a seed into an account xpub could not serve the
//! format the constellation exists to back up.
//!
//! # BIP-48, quoted
//!
//! `m / purpose' / coin_type' / account' / script_type' / change / address_index`
//!
//! and it registers exactly two script types — `1'` nested segwit (p2sh-p2wsh)
//! and `2'` native segwit (p2wsh), the latter being the recommended default.
//! There is no registered Taproot value, so none is offered here; inventing one
//! would put funds at a path no other wallet looks at.
//!
//! # Provenance of the pins below
//!
//! Derived through the SeedHammer II fork's INDEPENDENT Go implementation
//! (`bip39.MnemonicSeed` -> `hdkeychain.NewMaster` -> `bip32.Derive` ->
//! `Neuter`), which shares no code with this crate. The methodology is
//! self-checked: the same run reproduced
//! `m/48'/0'/0'/2'` = `xpub6DkFAXWQ2dHxq2vat…` byte-for-byte against a value
//! that is ENGRAVED on steel in that fork's committed gate record
//! (`oracle/gaterecords/S0-trace-a.record.json`) and independently decoded by
//! `mk decode`. A helper that reproduces an already-published value is one that
//! can be trusted for the values that are new here.
//!
//! Corpus is the all-zeros 16-byte entropy (abandon×11-about), as in
//! `cli_derive.rs`; master fp `73c5da0a`. Public BIP-39 test material — never
//! put funds behind it.

use std::process::Output;

use assert_cmd::Command;

const ZEROS_HEX: &str = "00000000000000000000000000000000";
const ABANDON: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const MASTER_FP: &str = "73c5da0a";

// m/48'/0'/0'/2' — native segwit multisig, account 0. Also the account the
// SeedHammer II engraves as cosigner card A@0.
const P2WSH_ACCT0: &str = "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf";
// m/48'/0'/0'/1' — nested segwit multisig, account 0.
const P2SH_P2WSH_ACCT0: &str = "xpub6DkFAXWQ2dHxnMKoSBogHrw1rgNJKR4umdbnNVNTYeCGcduxWnNUHgGptqEQWPKRmeW4Zn4FHSbLMBKEWYaMDYu47Ytg6DdFnPNt8hwn5mE";
// m/48'/0'/1'/2' — native segwit multisig, account 1. Pins that --account
// lands on the ACCOUNT level and not on script_type.
const P2WSH_ACCT1: &str = "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk";
// m/48'/1'/0'/2' — testnet, so coin_type is 1'.
const P2WSH_TESTNET_ACCT0: &str = "tpubDFH9dgzveyD8zTbPUFuLrGmCydNvxehyNdUXKJAQN8x4aZ4j6UZqGfnqFrD4NqyaTVGKbvEW54tsvPTK2UoSbCC1PJY8iCNiwTL3RWZEheQ";

fn ms(args: &[&str]) -> Output {
    Command::cargo_bin("ms")
        .unwrap()
        .args(args)
        .output()
        .unwrap()
}
fn out(o: &Output) -> String {
    String::from_utf8(o.stdout.clone()).unwrap()
}
fn err(o: &Output) -> String {
    String::from_utf8(o.stderr.clone()).unwrap()
}
fn code(o: &Output) -> i32 {
    o.status.code().unwrap_or(-1)
}

#[test]
fn bip48_p2wsh_matches_the_independent_oracle() {
    let o = ms(&["derive", "--hex", ZEROS_HEX, "--template", "bip48-p2wsh"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    let s = out(&o);
    assert!(s.contains(MASTER_FP), "{s}");
    assert!(s.contains(P2WSH_ACCT0), "{s}");
    assert!(s.contains("m/48'/0'/0'/2'"), "{s}");
}

#[test]
fn bip48_p2sh_p2wsh_matches_the_independent_oracle() {
    let o = ms(&[
        "derive",
        "--hex",
        ZEROS_HEX,
        "--template",
        "bip48-p2sh-p2wsh",
    ]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    let s = out(&o);
    assert!(s.contains(P2SH_P2WSH_ACCT0), "{s}");
    assert!(s.contains("m/48'/0'/0'/1'"), "{s}");
}

/// The two script types must not collapse onto one path. A copy-paste in the
/// enum would otherwise be invisible: both would still "work".
#[test]
fn the_two_script_types_derive_different_keys() {
    let a = out(&ms(&[
        "derive",
        "--hex",
        ZEROS_HEX,
        "--template",
        "bip48-p2wsh",
    ]));
    let b = out(&ms(&[
        "derive",
        "--hex",
        ZEROS_HEX,
        "--template",
        "bip48-p2sh-p2wsh",
    ]));
    assert!(a.contains(P2WSH_ACCT0) && b.contains(P2SH_P2WSH_ACCT0));
    assert_ne!(a, b, "both script types produced identical output");
}

/// --account must land on the ACCOUNT level, not on script_type. Getting this
/// wrong yields a valid-looking xpub at a path holding no funds.
#[test]
fn account_index_moves_the_account_level_not_the_script_type() {
    let o = ms(&[
        "derive",
        "--hex",
        ZEROS_HEX,
        "--template",
        "bip48-p2wsh",
        "--account",
        "1",
    ]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    let s = out(&o);
    assert!(s.contains(P2WSH_ACCT1), "{s}");
    assert!(s.contains("m/48'/0'/1'/2'"), "{s}");
}

#[test]
fn testnet_uses_coin_type_one() {
    let o = ms(&[
        "derive",
        "--hex",
        ZEROS_HEX,
        "--template",
        "bip48-p2wsh",
        "--network",
        "testnet",
    ]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    let s = out(&o);
    assert!(s.contains(P2WSH_TESTNET_ACCT0), "{s}");
    assert!(s.contains("m/48'/1'/0'/2'"), "{s}");
}

/// The phrase and the hex are the same secret, so they must derive the same
/// account — the path a real operator takes is the phrase.
#[test]
fn a_phrase_derives_the_same_account_as_the_hex() {
    let o = ms(&["derive", "--phrase", ABANDON, "--template", "bip48-p2wsh"]);
    assert_eq!(code(&o), 0, "{}", err(&o));
    assert!(out(&o).contains(P2WSH_ACCT0), "{}", out(&o));
}

/// No Taproot script_type is registered by BIP-48, so offering one would invent
/// a path. Refusal must be the behaviour, not a silent fallback to p2wsh.
#[test]
fn an_unregistered_script_type_is_refused() {
    let o = ms(&["derive", "--hex", ZEROS_HEX, "--template", "bip48-p2tr"]);
    assert_ne!(
        code(&o),
        0,
        "bip48-p2tr must be refused; stdout={}",
        out(&o)
    );
}

/// A bare `bip48` names no script type. Refusing it teaches the choice; picking
/// one silently would put a multisig cosigner key at a path the operator did
/// not choose.
#[test]
fn a_bare_bip48_is_refused_rather_than_guessed() {
    let o = ms(&["derive", "--hex", ZEROS_HEX, "--template", "bip48"]);
    assert_ne!(
        code(&o),
        0,
        "bare bip48 must be refused; stdout={}",
        out(&o)
    );
    let e = err(&o);
    assert!(
        e.contains("bip48-p2wsh") && e.contains("bip48-p2sh-p2wsh"),
        "the refusal must list the two registered script types; stderr={e}"
    );
}

/// The existing single-sig template names must not have been renamed as
/// collateral damage of adding a hyphenated variant.
#[test]
fn the_single_sig_template_names_are_unchanged() {
    for t in ["bip44", "bip49", "bip84", "bip86"] {
        let o = ms(&["derive", "--hex", ZEROS_HEX, "--template", t]);
        assert_eq!(code(&o), 0, "template {t} stopped working: {}", err(&o));
    }
}
