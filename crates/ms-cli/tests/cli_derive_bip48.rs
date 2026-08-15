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

/// The existing single-sig template names must not have been renamed as
/// collateral damage of adding a hyphenated variant.
#[test]
fn the_single_sig_template_names_are_unchanged() {
    for t in ["bip44", "bip49", "bip84", "bip86"] {
        let o = ms(&["derive", "--hex", ZEROS_HEX, "--template", t]);
        assert_eq!(code(&o), 0, "template {t} stopped working: {}", err(&o));
    }
}

// ─── permissive input, loud output ───────────────────────────────────────────
//
// A bare `bip48` used to be REFUSED, on the reasoning that BIP-48 registers two
// script types and choosing one silently could place a cosigner key at a path
// the operator did not pick.
//
// That was the wrong call, and this file's own neighbour proves it: `--language`
// makes a MORE dangerous assumption — the wordlist changes both the fingerprint
// and the xpub — and `ms` does not refuse it. It assumes english, annotates
// `(DEFAULT)` on stdout, emits a loud note on stderr, and exposes
// `language_defaulted` in JSON. Refusing here broke an established house pattern
// for a smaller assumption.
//
// The rule the product wants: be permissive on input, expressive on output, and
// say loudly when a common assumption had to be made. BIP-48 itself names 2'
// (p2wsh) the recommended default, so assuming it is not a guess — it is the
// BIP's own answer, announced.

const P2WSH_SCRIPT_TYPE_NOTE: &str = "script_type";

#[test]
fn a_bare_bip48_is_accepted_and_assumes_the_bip_recommended_default() {
    let o = ms(&["derive", "--hex", ZEROS_HEX, "--template", "bip48"]);
    assert_eq!(
        code(&o),
        0,
        "a bare bip48 must be ACCEPTED; stderr={}",
        err(&o)
    );
    let s = out(&o);
    // It must land on 2' (p2wsh) — the script type BIP-48 recommends.
    assert!(s.contains(P2WSH_ACCT0), "{s}");
    assert!(s.contains("m/48'/0'/0'/2'"), "{s}");
}

#[test]
fn the_assumption_is_announced_on_stdout_and_stderr() {
    let o = ms(&["derive", "--hex", ZEROS_HEX, "--template", "bip48"]);
    let s = out(&o);
    let e = err(&o);
    assert!(
        s.contains("DEFAULT"),
        "stdout must annotate the assumed script type, as --language does; stdout={s}"
    );
    assert!(
        e.to_lowercase().contains("p2wsh") && e.contains(P2WSH_SCRIPT_TYPE_NOTE),
        "stderr must say loudly which script type was assumed; stderr={e}"
    );
    // The note must be actionable: it has to name the alternative, or the
    // operator cannot tell they had a choice.
    assert!(
        e.contains("bip48-p2sh-p2wsh"),
        "the note must name the other registered script type; stderr={e}"
    );
}

/// An EXPLICIT script type is not an assumption, so it must NOT be annotated.
/// A tool that cries "DEFAULT" when the operator chose is a tool whose warnings
/// get ignored.
#[test]
fn an_explicit_script_type_announces_nothing() {
    let o = ms(&["derive", "--hex", ZEROS_HEX, "--template", "bip48-p2wsh"]);
    let s = out(&o);
    let e = err(&o);
    assert!(
        !s.contains("DEFAULT") || !s.contains(P2WSH_SCRIPT_TYPE_NOTE),
        "an explicitly chosen script type must not be annotated as a default; stdout={s}"
    );
    assert!(
        !e.contains("bip48-p2sh-p2wsh"),
        "no assumption note is due when the operator chose; stderr={e}"
    );
}

/// Machine-readable, mirroring `language_defaulted`. A caller parsing --json
/// must be able to see that an assumption was made without scraping stderr.
#[test]
fn json_carries_the_assumption_flag() {
    let bare = out(&ms(&[
        "derive",
        "--hex",
        ZEROS_HEX,
        "--template",
        "bip48",
        "--json",
    ]));
    assert!(
        bare.contains("\"script_type_defaulted\":true"),
        "--json must expose the assumption; got {bare}"
    );
    let explicit = out(&ms(&[
        "derive",
        "--hex",
        ZEROS_HEX,
        "--template",
        "bip48-p2wsh",
        "--json",
    ]));
    assert!(
        explicit.contains("\"script_type_defaulted\":false"),
        "--json must report false when the operator chose; got {explicit}"
    );
}

/// Single-sig templates make no script-type assumption at all, so the flag must
/// be false rather than vacuously true.
#[test]
fn single_sig_templates_report_no_script_type_assumption() {
    let s = out(&ms(&[
        "derive",
        "--hex",
        ZEROS_HEX,
        "--template",
        "bip84",
        "--json",
    ]));
    assert!(s.contains("\"script_type_defaulted\":false"), "got {s}");
}
