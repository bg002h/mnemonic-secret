//! Shared `payload_entropy_and_language` helper for `derive` + `verify`.
//!
//! ms-codec `decode()` / `combine_shares()` return `Payload::Mnem` for any ms1
//! built from a non-English BIP-39 phrase. The wire `Mnem.language` byte is the
//! AUTHORITATIVE wordlist language — `--language` is advisory-only for mnem
//! cards (BIP-39 seed = PBKDF2 over the language-specific sentence, so the same
//! entropy under two wordlists yields two different seeds → two different
//! wallets). This helper centralizes the recovery of `(entropy, effective_lang,
//! effective_lang_defaulted)` so `derive`/`verify` reach parity with the proven
//! `decode.rs:63-89` policy and the `#[non_exhaustive]` future-variant guard
//! lives once.

use std::io::Write;

use ms_codec::Payload;
use zeroize::Zeroizing;

use crate::language::CliLanguage;

/// Recover `(entropy, effective wordlist language, effective-language-defaulted)`
/// from a decoded `Payload`. This is the 3-tuple `derive`/`verify` consume to
/// reach parity with `decode` (the 2-tuple language part is `decode.rs:63`'s
/// `(effective_lang, effective_lang_defaulted)`; this adds the entropy).
///
/// - `Entr`: language + defaulted are whatever the caller resolved from
///   `--language`/default → pass `(cli_lang, cli_lang_defaulted)` through.
/// - `Mnem`: the WIRE language byte is AUTHORITATIVE (`CliLanguage::from_code`);
///   `--language` is advisory-only; `effective_lang_defaulted = false` (a real
///   wire language exists, never "defaulted"), mirroring `decode.rs`. On
///   Mnem/cli disagreement (cli explicit AND wire != cli) emit the `decode.rs`
///   note to `stderr`.
///
/// `payload` is consumed by value; the moved entropy `Vec` is immediately
/// re-wrapped in `Zeroizing` (scrub-on-drop).
// P1: no non-test consumer wired yet (derive/verify consume it in P2/P3).
// The `#[allow(dead_code)]` is removed in P2 once `derive` calls it.
#[allow(dead_code)]
pub(crate) fn payload_entropy_and_language(
    payload: Payload,
    cli_lang: CliLanguage,
    cli_lang_defaulted: bool,
    stderr: &mut impl Write,
) -> (Zeroizing<Vec<u8>>, CliLanguage, bool) {
    match payload {
        Payload::Entr(b) => (Zeroizing::new(b), cli_lang, cli_lang_defaulted),
        Payload::Mnem {
            language: wire_code,
            entropy,
        } => {
            let wire_cli = CliLanguage::from_code(wire_code).unwrap_or(CliLanguage::English);
            // Wire wins; warn only if the user EXPLICITLY supplied --language
            // that disagrees. Byte-for-byte the decode.rs note string.
            if !cli_lang_defaulted && wire_cli != cli_lang {
                let _ = writeln!(
                    stderr,
                    "note: this ms1 carries wordlist language '{}'; ignoring --language {}",
                    wire_cli.as_str(),
                    cli_lang.as_str()
                );
            }
            (Zeroizing::new(entropy), wire_cli, false)
        }
        // ms_codec::Payload is #[non_exhaustive]; guard against future variants.
        _ => unreachable!("ms-codec decode returned unknown Payload variant"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Entr pass-through (defaulted): caller's (cli_lang, defaulted) flow through; no note.
    #[test]
    fn entr_pass_through_defaulted() {
        let mut buf: Vec<u8> = Vec::new();
        let (entropy, lang, defaulted) = payload_entropy_and_language(
            Payload::Entr(vec![0u8; 16]),
            CliLanguage::English,
            true,
            &mut buf,
        );
        assert_eq!(&entropy[..], &[0u8; 16]);
        assert_eq!(lang, CliLanguage::English);
        assert!(defaulted);
        assert!(buf.is_empty(), "no note for Entr");
    }

    /// Entr pass-through (explicit non-default): (French, false) flow through; no note.
    #[test]
    fn entr_pass_through_explicit_french() {
        let mut buf: Vec<u8> = Vec::new();
        let (entropy, lang, defaulted) = payload_entropy_and_language(
            Payload::Entr(vec![0u8; 16]),
            CliLanguage::French,
            false,
            &mut buf,
        );
        assert_eq!(&entropy[..], &[0u8; 16]);
        assert_eq!(lang, CliLanguage::French);
        assert!(!defaulted);
        assert!(buf.is_empty(), "no note for Entr");
    }

    /// Mnem wire-wins, flag omitted (defaulted): returns wire French, defaulted=false, no note.
    #[test]
    fn mnem_wire_wins_no_flag() {
        let mut buf: Vec<u8> = Vec::new();
        let (entropy, lang, defaulted) = payload_entropy_and_language(
            Payload::Mnem {
                language: 6, // french
                entropy: vec![0u8; 16],
            },
            CliLanguage::English,
            true, // defaulted (flag omitted)
            &mut buf,
        );
        assert_eq!(&entropy[..], &[0u8; 16]);
        assert_eq!(lang, CliLanguage::French);
        assert!(!defaulted, "a real wire language is never 'defaulted'");
        assert!(buf.is_empty(), "no disagreement note when flag omitted");
    }

    /// Mnem disagreement (explicit english vs wire french): wire wins + note on stderr.
    #[test]
    fn mnem_disagreement_note() {
        let mut buf: Vec<u8> = Vec::new();
        let (_entropy, lang, defaulted) = payload_entropy_and_language(
            Payload::Mnem {
                language: 6, // french
                entropy: vec![0u8; 16],
            },
            CliLanguage::English,
            false, // explicit --language english
            &mut buf,
        );
        assert_eq!(lang, CliLanguage::French);
        assert!(!defaulted);
        let s = String::from_utf8(buf).unwrap();
        assert_eq!(
            s,
            "note: this ms1 carries wordlist language 'french'; ignoring --language english\n"
        );
    }

    /// Mnem agreement (explicit french == wire french): no note.
    #[test]
    fn mnem_agreement_no_note() {
        let mut buf: Vec<u8> = Vec::new();
        let (_entropy, lang, defaulted) = payload_entropy_and_language(
            Payload::Mnem {
                language: 6, // french
                entropy: vec![0u8; 16],
            },
            CliLanguage::French,
            false, // explicit --language french
            &mut buf,
        );
        assert_eq!(lang, CliLanguage::French);
        assert!(!defaulted);
        assert!(buf.is_empty(), "wire == cli → no note");
    }
}
