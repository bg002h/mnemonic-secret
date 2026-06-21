//! H5 — `ms verify` on a valid NON-English (mnem) ms1.
//!
//! TODAY: `verify.rs:64` panics (`unreachable!("ms-codec v0.1 only decodes to
//! Payload::Entr")`) on any `Payload::Mnem`, and the `--phrase` round-trip leg
//! parses under `args.language` (CLI flag) not the wire byte. The fix routes
//! ONLY the `Ok((tag,payload))` decode arm through `payload_entropy_and_language`
//! (the exit-3 `ReservedTagNotEmittedInV01` future-format leg + the generic
//! `Err` leg are preserved verbatim), and re-types `--language` to
//! `Option<CliLanguage>` so omission ≠ explicit `--language english` (no
//! spurious disagreement note).

use std::process::Output;

use assert_cmd::Command;
use codex32::{Codex32String, Fe};

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

fn phrase_of(lang: bip39::Language, entropy: &[u8]) -> String {
    bip39::Mnemonic::from_entropy_in(lang, entropy)
        .expect("valid entropy length")
        .to_string()
}

/// Build a mnem ms1 (carries the wire language byte) via `ms encode`.
fn mnem_ms1(language: &str, phrase: &str) -> String {
    let o = ms(&["encode", "--language", language, "--phrase", phrase, "--group-size", "0"]);
    assert!(o.status.success(), "encode: {}", err(&o));
    out(&o).lines().next().unwrap().trim().to_string()
}

const ENGLISH_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const ENGLISH_ENTR_MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

/// RED→GREEN (no --phrase): a Japanese mnem ms1 → verify. TODAY panics; AFTER exit 0.
#[test]
fn japanese_mnem_no_phrase_exit_0() {
    let ja = phrase_of(bip39::Language::Japanese, &[0xABu8; 16]);
    let card = mnem_ms1("japanese", &ja);
    let o = ms(&["verify", &card]);
    assert_eq!(code(&o), 0, "stderr: {}", err(&o));
}

/// --phrase round-trip under the WIRE (Japanese) language, NO --language → exit 0.
#[test]
fn japanese_phrase_round_trip_wire_honored() {
    let ja = phrase_of(bip39::Language::Japanese, &[0xABu8; 16]);
    let card = mnem_ms1("japanese", &ja);
    let o = ms(&["verify", "--phrase", &ja, &card]);
    assert_eq!(code(&o), 0, "stderr: {}", err(&o));
}

/// True-negative preserved: an English phrase against a Japanese card → NOT exit 0.
#[test]
fn english_phrase_against_japanese_card_fails() {
    let ja = phrase_of(bip39::Language::Japanese, &[0xABu8; 16]);
    let card = mnem_ms1("japanese", &ja);
    let o = ms(&["verify", "--phrase", ENGLISH_12, &card]);
    assert_ne!(code(&o), 0, "must not be a false GREEN; stdout: {}", out(&o));
}

/// Disagreement note (EXPLICIT flag): `--language english` on a Japanese card →
/// exit 0 + stderr disagreement note.
#[test]
fn explicit_english_on_japanese_card_emits_note() {
    let ja = phrase_of(bip39::Language::Japanese, &[0xABu8; 16]);
    let card = mnem_ms1("japanese", &ja);
    let o = ms(&["verify", "--language", "english", &card]);
    assert_eq!(code(&o), 0, "stderr: {}", err(&o));
    let e = err(&o);
    assert!(e.contains("note:"), "expected disagreement note: {e}");
    assert!(e.contains("japanese"), "note names wire language: {e}");
    assert!(e.contains("english"), "note names supplied --language: {e}");
}

/// NO spurious note when flag OMITTED (Option-ization guard): bare verify of a
/// non-English card → exit 0 AND NO `note: this ms1 carries wordlist language`.
#[test]
fn bare_no_flag_no_spurious_note() {
    let ja = phrase_of(bip39::Language::Japanese, &[0xABu8; 16]);
    let card = mnem_ms1("japanese", &ja);
    let o = ms(&["verify", &card]);
    assert_eq!(code(&o), 0, "stderr: {}", err(&o));
    assert!(
        !err(&o).contains("this ms1 carries wordlist language"),
        "omitted --language must NOT emit a disagreement note: {}",
        err(&o)
    );
}

/// Round-trip label pin: `--phrase` round-trip (no --language) → the success
/// label shows the WIRE language (japanese), NOT english.
#[test]
fn round_trip_label_shows_wire_language() {
    let ja = phrase_of(bip39::Language::Japanese, &[0xABu8; 16]);
    let card = mnem_ms1("japanese", &ja);
    let o = ms(&["verify", "--phrase", &ja, &card]);
    assert_eq!(code(&o), 0, "stderr: {}", err(&o));
    let s = out(&o);
    assert!(s.contains("language=japanese"), "round-trip label = wire japanese: {s}");
    assert!(!s.contains("language=english"), "must not show english: {s}");
}

/// EXIT-3 NO-REGRESSION: a reserved-tag future-format string still exits 3 via
/// `emit_future_format` (the Err-leg is preserved verbatim; the Ok-arm refactor
/// did NOT drop it). Built like the existing `verify_future_format.rs` fixture.
#[test]
fn reserved_tag_still_exits_3() {
    let mut data = vec![0x00u8];
    data.extend_from_slice(&[0xAAu8; 16]);
    let s = Codex32String::from_seed("ms", 0, "seed", Fe::S, &data)
        .unwrap()
        .to_string();
    let o = ms(&["verify", &s]);
    assert_eq!(code(&o), 3, "reserved tag must exit 3; stderr: {}", err(&o));
    assert!(
        out(&o).contains("OK: valid future format (v0.2+, tag seed)"),
        "exit-3 future-format message: {}",
        out(&o)
    );
}

/// Positive control: English entr ms1 (no --phrase, and with --phrase) → exit 0,
/// english label unchanged.
#[test]
fn english_entr_positive_control() {
    let o1 = ms(&["verify", ENGLISH_ENTR_MS1]);
    assert_eq!(code(&o1), 0, "stderr: {}", err(&o1));

    let o2 = ms(&["verify", ENGLISH_ENTR_MS1, "--phrase", ENGLISH_12]);
    assert_eq!(code(&o2), 0, "stderr: {}", err(&o2));
    assert!(
        out(&o2).contains("language=english"),
        "english entr round-trip label: {}",
        out(&o2)
    );
}
