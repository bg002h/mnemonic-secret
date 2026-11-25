//! Bug hunt 2026-11-25, ms#2 — `--language` given with `--hex` was dropped
//! silently by `ms encode` / `ms split`.
//!
//! `--hex` mints a language-less Entr card, so an explicit non-English
//! `--language` was accepted, discarded, and the backup restored by default to
//! the English wallet (a different seed). The fix refuses that pair (exit 64,
//! before anything is read) and points at the phrase channel. What a card
//! encodes to is unchanged: `--hex` alone, and `--hex --language english`,
//! still mint the same Entr card.
//!
//! `ms combine` on an Entr share set now carries the same
//! "default — verify against your records" annotation `ms decode` prints.

use assert_cmd::Command;

const E: &str = "000102030405060708090a0b0c0d0e0f";
/// `ms encode --hex` of `E` (see the bug report's reproduction).
const E_ENTR: &str = "ms10entrsqqqqzqsrqszsvpcgpy9qkrqdpc8sh9lf92n5fr33s";

fn ms(args: &[&str], stdin: &str) -> std::process::Output {
    Command::cargo_bin("ms")
        .unwrap()
        .args(args)
        .write_stdin(stdin.to_string())
        .output()
        .unwrap()
}

#[test]
fn encode_hex_with_non_english_language_is_refused() {
    let o = ms(&["encode", "--hex", "-", "--language", "japanese"], E);
    assert_eq!(o.status.code(), Some(64), "{o:?}");
    assert!(o.stdout.is_empty(), "nothing minted: {o:?}");
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("--language japanese"), "{err}");
    assert!(err.contains("--hex"), "{err}");
    assert!(
        err.contains("--phrase"),
        "points at the phrase channel: {err}"
    );
}

#[test]
fn encode_hex_with_non_english_language_is_refused_json_envelope() {
    let o = ms(
        &["encode", "--hex", "-", "--language", "french", "--json"],
        E,
    );
    assert_eq!(o.status.code(), Some(64), "{o:?}");
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(!out.contains("ms10"), "no card in the envelope: {out}");
    assert!(!out.contains(E), "no entropy echoed: {out}");
}

#[test]
fn split_hex_with_non_english_language_is_refused() {
    let o = ms(
        &[
            "split",
            "--hex",
            "-",
            "--language",
            "japanese",
            "-k",
            "2",
            "-n",
            "2",
        ],
        E,
    );
    assert_eq!(o.status.code(), Some(64), "{o:?}");
    assert!(o.stdout.is_empty(), "no shares minted: {o:?}");
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("--language japanese"), "{err}");
    assert!(err.contains("--phrase"), "{err}");
}

/// The card `--hex` mints is unchanged, with or without an explicit English.
#[test]
fn encode_hex_card_is_unchanged() {
    for args in [
        &["encode", "--hex", "-"][..],
        &["encode", "--hex", "-", "--language", "english"][..],
    ] {
        let o = ms(args, E);
        assert_eq!(o.status.code(), Some(0), "{args:?}: {o:?}");
        assert_eq!(
            String::from_utf8_lossy(&o.stdout).trim(),
            E_ENTR,
            "{args:?}"
        );
    }
}

/// A non-English phrase still mints a Mnem card (the language survives).
#[test]
fn encode_phrase_with_non_english_language_still_works() {
    let ja = bip39::Mnemonic::from_entropy_in(bip39::Language::Japanese, &[0u8; 16])
        .unwrap()
        .to_string();
    let o = ms(
        &[
            "encode",
            "--phrase",
            "-",
            "--language",
            "japanese",
            "--json",
        ],
        &ja,
    );
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    assert!(String::from_utf8_lossy(&o.stdout).contains("\"language\":\"japanese\""));
}

fn split_hex_shares() -> Vec<String> {
    let o = ms(&["split", "--hex", "-", "-k", "2", "-n", "2", "--json"], E);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    v["shares"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn combine_entr_marks_english_as_default() {
    let shares = split_hex_shares();
    let o = ms(&["combine", "-"], &shares.join("\n"));
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(
        out.contains("language: english (12 words, default — verify against your records)"),
        "{out}"
    );
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("carry no wordlist language"), "{err}");
}

/// A Mnem share set names its wire language plainly: no default annotation.
#[test]
fn combine_mnem_names_its_language_plainly() {
    let ja = bip39::Mnemonic::from_entropy_in(bip39::Language::Japanese, &[0u8; 16])
        .unwrap()
        .to_string();
    let o = ms(
        &[
            "split",
            "--phrase",
            "-",
            "--language",
            "japanese",
            "-k",
            "2",
            "-n",
            "2",
            "--json",
        ],
        &ja,
    );
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    let shares: Vec<&str> = v["shares"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap())
        .collect();
    let o = ms(&["combine", "-"], &shares.join("\n"));
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(out.contains("language: japanese (12 words)\n"), "{out}");
    assert!(!String::from_utf8_lossy(&o.stderr).contains("carry no wordlist language"));
}

/// The refusal's printed command runs as printed and keeps the language.
#[test]
fn refusal_command_runs_as_printed() {
    for verb_args in [
        &["encode", "--hex", "-", "--language", "japanese"][..],
        &[
            "split",
            "--hex",
            "-",
            "--language",
            "japanese",
            "-k",
            "2",
            "-n",
            "2",
        ][..],
    ] {
        let o = ms(verb_args, E);
        assert_eq!(o.status.code(), Some(64), "{o:?}");
        let err = String::from_utf8_lossy(&o.stderr);
        let cmd = err
            .split("instead: ")
            .nth(1)
            .and_then(|t| t.split(" (or").next())
            .unwrap_or_else(|| panic!("no printed command: {err}"));
        let words: Vec<&str> = cmd.split_whitespace().collect();
        assert_eq!(words[0], "ms", "{cmd}");
        let ja =
            bip39::Mnemonic::from_entropy_in(bip39::Language::Japanese, &hex::decode(E).unwrap())
                .unwrap()
                .to_string();
        let o = ms(&words[1..], &ja);
        assert_eq!(o.status.code(), Some(0), "{cmd}: {o:?}");
        let out = String::from_utf8_lossy(&o.stdout);
        let back_verb = if verb_args[0] == "encode" {
            "decode"
        } else {
            "combine"
        };
        let o = ms(&[back_verb, "-"], &out);
        assert_eq!(o.status.code(), Some(0), "{back_verb}: {o:?}");
        let back = String::from_utf8_lossy(&o.stdout);
        assert!(back.contains("language: japanese (12 words)\n"), "{back}");
    }
}
