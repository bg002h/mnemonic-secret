//! H4 — `ms derive` on a valid NON-English (mnem) ms1.
//!
//! TODAY: `derive.rs` panics (`unreachable!("ms-codec v0.1 decodes only
//! Payload::Entr")`) on any `Payload::Mnem`. A naive `--language`-default fix
//! would derive the WRONG fingerprint (English) for a non-English seed —
//! BIP-39 seed = PBKDF2 over the language-specific sentence.
//!
//! The fix routes the ms1 branch through `payload_entropy_and_language`, which
//! honors the WIRE language byte. Oracle (same all-zeros 16-byte entropy):
//! English fp `73c5da0a` ≠ French fp `7d53dc37` (both in `cli_derive.rs`).

use std::process::Output;

use assert_cmd::Command;

const ZEROS_HEX: &str = "00000000000000000000000000000000";
const MASTER_FP_EN: &str = "73c5da0a";
const MASTER_FP_FR: &str = "7d53dc37";

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
    o.status.code().unwrap()
}

/// 12-word phrase for the given bip39 language from the given entropy.
fn phrase_of(lang: bip39::Language, entropy: &[u8]) -> String {
    bip39::Mnemonic::from_entropy_in(lang, entropy)
        .expect("valid entropy length")
        .to_string()
}

/// Build a mnem ms1 (carries the wire language byte) via `ms encode`.
fn mnem_ms1(language: &str, phrase: &str) -> String {
    let o = ms(&[
        "encode",
        "--language",
        language,
        "--phrase",
        phrase,
        "--group-size",
        "0",
    ]);
    assert!(o.status.success(), "encode: {}", err(&o));
    out(&o).lines().next().unwrap().trim().to_string()
}

/// Build an entr ms1 (no language byte) via `ms encode --hex`.
fn entr_ms1(hex: &str) -> String {
    let o = ms(&["encode", "--hex", hex, "--group-size", "0"]);
    assert!(o.status.success(), "encode: {}", err(&o));
    out(&o).lines().next().unwrap().trim().to_string()
}

/// Funds-safety core: a French mnem ms1 derives the CORRECT French fp, NOT the
/// wrong English fp a naive `--language`-default patch would emit. TODAY: panics.
#[test]
fn french_mnem_ms1_derives_correct_french_fp() {
    let fr = phrase_of(bip39::Language::French, &[0u8; 16]);
    let card = mnem_ms1("french", &fr);
    let o = ms(&["derive", &card]);
    assert_eq!(code(&o), 0, "stderr: {}", err(&o));
    let s = out(&o);
    assert!(
        s.contains(MASTER_FP_FR),
        "expected French fp {MASTER_FP_FR}, got: {s}"
    );
    assert!(
        !s.contains(MASTER_FP_EN),
        "must NOT emit wrong English fp {MASTER_FP_EN} (naive-patch bug): {s}"
    );
}

/// Japanese variant + derive-from-card == derive-from-phrase parity.
#[test]
fn japanese_mnem_card_matches_derive_from_phrase() {
    let ja = phrase_of(bip39::Language::Japanese, &[0xABu8; 16]);
    let card = mnem_ms1("japanese", &ja);

    let from_card = ms(&["derive", &card]);
    assert_eq!(code(&from_card), 0, "stderr: {}", err(&from_card));

    let from_phrase = ms(&["derive", "--phrase", &ja, "--language", "japanese"]);
    assert_eq!(code(&from_phrase), 0, "stderr: {}", err(&from_phrase));

    // Both print the same master fingerprint line.
    let fp_line = |s: &str| {
        s.lines()
            .find(|l| l.contains("master_fingerprint"))
            .unwrap_or("")
            .to_string()
    };
    let a = fp_line(&out(&from_card));
    let b = fp_line(&out(&from_phrase));
    assert!(!a.is_empty(), "no fp line from card: {}", out(&from_card));
    assert_eq!(a, b, "card vs phrase fp mismatch");
}

/// Disagreement note: explicit `--language english` on a French card → wire wins
/// (French fp), stderr carries the `note:` ignoring `--language english`.
#[test]
fn explicit_wrong_language_wire_wins_with_note() {
    let fr = phrase_of(bip39::Language::French, &[0u8; 16]);
    let card = mnem_ms1("french", &fr);
    let o = ms(&["derive", "--language", "english", &card]);
    assert_eq!(code(&o), 0, "stderr: {}", err(&o));
    assert!(
        out(&o).contains(MASTER_FP_FR),
        "wire (French) fp: {}",
        out(&o)
    );
    let e = err(&o);
    assert!(e.contains("note:"), "expected disagreement note: {e}");
    assert!(e.contains("french"), "note names wire language: {e}");
    assert!(e.contains("english"), "note names supplied --language: {e}");
}

/// LABEL PIN (mislabel-card guard): a French card with NO `--language` →
/// `language: french` (NOT `english (DEFAULT)`), and NO bogus english-default
/// note on stderr.
#[test]
fn french_card_labels_french_not_default() {
    let fr = phrase_of(bip39::Language::French, &[0u8; 16]);
    let card = mnem_ms1("french", &fr);

    // Text mode.
    let o = ms(&["derive", &card]);
    assert_eq!(code(&o), 0, "stderr: {}", err(&o));
    let s = out(&o);
    assert!(s.contains(MASTER_FP_FR), "{s}");
    assert!(
        s.contains("language:            french"),
        "expected 'language: french': {s}"
    );
    assert!(!s.contains("(DEFAULT)"), "must not be DEFAULT-labeled: {s}");
    assert!(
        !err(&o).contains("--language defaulted to english"),
        "no bogus english-default note: {}",
        err(&o)
    );

    // JSON mode.
    let oj = ms(&["derive", "--json", &card]);
    assert_eq!(code(&oj), 0, "stderr: {}", err(&oj));
    let sj = out(&oj);
    assert!(
        sj.contains("\"language\":\"french\""),
        "json language=french: {sj}"
    );
    assert!(
        sj.contains("\"language_defaulted\":false"),
        "json defaulted=false: {sj}"
    );
}

/// Positive control: an English Entr card (no language byte) → fp 73c5da0a,
/// `language: english (DEFAULT)` + the english-default note, NO disagreement note.
#[test]
fn english_entr_card_default_label_preserved() {
    let card = entr_ms1(ZEROS_HEX);
    let o = ms(&["derive", &card]);
    assert_eq!(code(&o), 0, "stderr: {}", err(&o));
    let s = out(&o);
    assert!(s.contains(MASTER_FP_EN), "{s}");
    assert!(
        s.contains("(DEFAULT)"),
        "Entr card keeps DEFAULT label: {s}"
    );
    let e = err(&o);
    assert!(
        e.contains("--language defaulted to english"),
        "Entr card keeps english-default note: {e}"
    );
    assert!(
        !e.contains("ignoring --language"),
        "Entr card must not emit a disagreement note: {e}"
    );
}

/// Contrast pin (Entr-vs-Mnem label split): `ms encode --language english`
/// canonicalizes to an *Entr* card (English = universal default, no language
/// byte), so the Entr DEFAULT-label path is the english branch — already pinned
/// by `english_entr_card_default_label_preserved`. The non-DEFAULT-without-flag
/// branch is exercised by the French/Japanese mnem tests above
/// (`effective_lang_defaulted == false` for any wire `Mnem`). A non-English
/// *Mnem* card with a matching explicit `--language` (agreement, no note) pins
/// the no-DEFAULT label without relying on a non-constructible english Mnem.
#[test]
fn french_card_explicit_matching_language_no_default_label() {
    let fr = phrase_of(bip39::Language::French, &[0u8; 16]);
    let card = mnem_ms1("french", &fr);
    let o = ms(&["derive", "--language", "french", &card]);
    assert_eq!(code(&o), 0, "stderr: {}", err(&o));
    let s = out(&o);
    assert!(s.contains(MASTER_FP_FR), "{s}");
    assert!(s.contains("language:            french"), "{s}");
    assert!(
        !s.contains("(DEFAULT)"),
        "real-wire french card is not DEFAULT-labeled: {s}"
    );
    assert!(
        !err(&o).contains("--language defaulted to english"),
        "no english-default note for a real-wire card: {}",
        err(&o)
    );
    assert!(
        !err(&o).contains("ignoring --language"),
        "agreement (french == wire) → no disagreement note: {}",
        err(&o)
    );
}
