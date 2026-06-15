//! `ms encode` — produce an ms1 string from a BIP-39 mnemonic (or hex entropy).
//!
//! Realizes SPEC §2.1 (full command surface), §4 (multi-line stdout + stderr
//! engraving card + 5-char chunked form), §5.1 (--json schema).

use std::io::Write;

use bip39::{Language, Mnemonic};
use clap::Args;
use hex::FromHex;
use ms_codec::{Payload, Tag};
use serde_json::to_string;

use crate::advisory::{OutputClass, emit_output_class_advisory};
use crate::error::{CliError, Result};
use crate::format::{render_grouped, EncodeJson};
use crate::language::CliLanguage;
use crate::parse::{read_input, read_phrase_input};

/// `ms encode` arguments.
///
/// `--phrase` and `--hex` form a mutually-exclusive group; exactly one MUST
/// be supplied. The `#[command(group = ...)]` declaration scopes the exclusion
/// to just `phrase` + `hex`; encode_arg_group_violations.rs (Phase 4) tests
/// this with exit 64 on both-supplied and neither-supplied inputs.
#[derive(Args, Debug)]
#[command(group = clap::ArgGroup::new("input").required(true).args(["phrase", "hex"]))]
pub struct EncodeArgs {
    /// BIP-39 mnemonic. Use `-` to read from stdin.
    #[arg(long)]
    pub phrase: Option<String>,

    /// Hex-encoded entropy bytes (16/20/24/28/32 B = 32/40/48/56/64 hex chars).
    #[arg(long)]
    pub hex: Option<String>,

    /// BIP-39 wordlist for the input phrase. Ignored under --hex.
    #[arg(long, default_value = "english")]
    pub language: CliLanguage,

    /// Suppress the stderr engraving card (for tooling).
    #[arg(long)]
    pub no_engraving_card: bool,

    /// Insert a separator every N characters in the emitted ms1 string
    /// (0 = unbroken). SPEC §3. Display only; --json stays unbroken.
    #[arg(long, default_value_t = 5)]
    pub group_size: u16,

    /// Separator: space|hyphen|comma (keyword) or the literal " "|-|, . SPEC §5.
    #[arg(long, default_value = "space", value_parser = crate::format::parse_separator)]
    pub separator: char,

    /// Emit a single JSON object on stdout instead of multi-line text.
    #[arg(long)]
    pub json: bool,
}

/// Resolve a secret source (`--phrase` / `--hex`) to a `(Payload, language_for_card)`
/// pair — the shared entropy-resolution + AUTO-route used by `ms encode` AND
/// `ms split` (Task 2.1). Extracted verbatim from the former `encode::run`
/// inline logic.
///
/// - A non-English `--phrase` → `Payload::Mnem { language, entropy }` (the
///   wordlist language survives onto the wire).
/// - An English `--phrase` OR `--hex` → `Payload::Entr(entropy)` (byte-identical
///   to v0.1).
///
/// The 2nd tuple element is `language_for_card`: `Some(language.as_str())` for a
/// phrase (the BIP-39 wordlist used to parse it), `None` for `--hex`. A bare
/// `Payload` cannot tell an English phrase from `--hex` (both → `Payload::Entr`),
/// so callers that surface a `language` field MUST use this 2nd element rather
/// than re-deriving from the `Payload`.
///
/// Exactly one of `phrase` / `hex` MUST be `Some` (clap's required group enforces
/// this for both `encode` and `split`); both-`None` is a defensive `BadInput`.
pub(crate) fn resolve_secret_payload(
    phrase: Option<&str>,
    hex: Option<&str>,
    language: CliLanguage,
) -> Result<(Payload, Option<&'static str>)> {
    use zeroize::Zeroizing;

    // clap's mutually-exclusive group enforces exactly-one-of-{phrase,hex}.
    // `entropy` is the secret byte buffer; `language_for_card` is the 2nd
    // element returned to the caller. Both are bound here (lint anchor:
    // `let (entropy, language_for_card): (Zeroizing<Vec<u8>>`).
    let (entropy, language_for_card): (Zeroizing<Vec<u8>>, Option<&'static str>) =
        if let Some(phrase_arg) = phrase {
            let phrase: Zeroizing<String> = read_phrase_input(Some(phrase_arg))?;
            let lang: Language = language.into();
            // SAFETY: third-party-blocked — `bip39::Mnemonic` has no Drop+
            // Zeroize; tracked at FOLLOWUP `rust-bip39-mnemonic-zeroize-upstream`
            // (companion of the mnemonic-toolkit cycle entry).
            let mnemonic = Mnemonic::parse_in(lang, phrase.as_str())?;
            (Zeroizing::new(mnemonic.to_entropy()), Some(language.as_str()))
        } else if let Some(hex_arg) = hex {
            let hex_str = Zeroizing::new(read_input(Some(hex_arg))?);
            let bytes = Zeroizing::new(parse_hex_entropy(&hex_str)?);
            (bytes, None)
        } else {
            // clap's required-group should have caught this; defensive.
            return Err(CliError::BadInput(
                "exactly one of --phrase or --hex is required".into(),
            ));
        };

    // Route to Payload::Mnem when input is a phrase in a non-English language;
    // English phrases and --hex always stay Payload::Entr (byte-identical to v0.1).
    // ms_codec::Payload::Entr(Vec<u8>) / Payload::Mnem are the public-API
    // caller-wrap-contract shapes; clone the wrapped buffer's contents into the
    // public Vec at the call boundary. The original `entropy` Zeroizing<Vec<u8>>
    // scrubs on drop at function exit.
    let payload = if language != CliLanguage::English && phrase.is_some() {
        Payload::Mnem {
            language: language.code(),
            entropy: (*entropy).clone(),
        }
    } else {
        Payload::Entr((*entropy).clone())
    };
    Ok((payload, language_for_card))
}

/// Run `ms encode` with the parsed args. Writes to stdout/stderr per SPEC §2.1.
pub fn run(mut args: EncodeArgs) -> Result<u8> {
    use zeroize::Zeroizing;
    // SPEC v0.9.0 §1 item 2 — consume + immediately wrap the clap-owned
    // secret fields (phrase / hex) at `run()` entry. clap-derive does not
    // natively emit `Zeroizing<String>`, so we `mem::take` the Option
    // contents, wrapping the captured String. The clap-owned `Option<String>`
    // slots are left as `None` (its allocation freed; the actual bytes are
    // now in the Zeroizing wrapper and will be scrubbed on drop).
    let phrase_arg: Option<Zeroizing<String>> =
        std::mem::take(&mut args.phrase).map(Zeroizing::new);
    let hex_arg: Option<Zeroizing<String>> =
        std::mem::take(&mut args.hex).map(Zeroizing::new);

    // Shared entropy-resolution + AUTO-route (also used by `ms split`).
    let (payload, language_for_card) = resolve_secret_payload(
        phrase_arg.as_ref().map(|p| p.as_str()),
        hex_arg.as_ref().map(|h| h.as_str()),
        args.language,
    )?;

    let ms1 = ms_codec::encode(Tag::ENTR, &payload)?;

    // Re-derive the output `entropy` view from the resolved Payload (the
    // entropy bytes, sans prefix/language). word_count + entropy_hex use it.
    let entropy: Zeroizing<Vec<u8>> = Zeroizing::new(payload.as_bytes().to_vec());
    let word_count = entropy.len() * 3 / 4; // 16->12, 20->15, 24->18, 28->21, 32->24

    if args.json {
        emit_json(&ms1, language_for_card, word_count, &entropy[..])?;
    } else {
        emit_text(
            &ms1,
            language_for_card,
            word_count,
            args.no_engraving_card,
            args.group_size as usize,
            args.separator,
        )?;
    }
    emit_output_class_advisory(OutputClass::PrivateKeyMaterial, &mut std::io::stderr().lock());
    Ok(0)
}

pub(crate) fn parse_hex_entropy(hex_str: &str) -> Result<Vec<u8>> {
    if hex_str.is_empty() {
        return Err(CliError::BadInput(
            "expected hex of length 32/40/48/56/64 chars (got empty input)".into(),
        ));
    }
    if hex_str.len() % 2 != 0 {
        return Err(CliError::BadInput(format!(
            "expected even-length hex (one byte = 2 chars); got {} chars",
            hex_str.len()
        )));
    }
    Vec::<u8>::from_hex(hex_str).map_err(|e| match e {
        hex::FromHexError::InvalidHexCharacter { c, index } => {
            CliError::BadInput(format!("invalid character '{}' at position {}", c, index))
        }
        hex::FromHexError::OddLength => {
            CliError::BadInput("expected even-length hex (one byte = 2 chars)".into())
        }
        hex::FromHexError::InvalidStringLength => {
            CliError::BadInput("hex string length invalid".into())
        }
    })
}

fn emit_json(ms1: &str, language: Option<&str>, word_count: usize, entropy: &[u8]) -> Result<()> {
    let json = EncodeJson {
        schema_version: "1",
        ms1,
        language,
        word_count,
        entropy_hex: hex::encode(entropy),
    };
    let s =
        to_string(&json).map_err(|e| CliError::BadInput(format!("json serialization: {}", e)))?;
    println!("{}", s);
    Ok(())
}

fn emit_text(
    ms1: &str,
    language: Option<&str>,
    word_count: usize,
    no_engraving_card: bool,
    group_size: usize,
    separator: char,
) -> Result<()> {
    // Print-once stdout: the ms1 in the flag-controlled grouped form (SPEC §6).
    println!("{}", render_grouped(ms1, group_size, separator));

    if !no_engraving_card {
        let mut stderr = std::io::stderr().lock();
        writeln!(stderr, "word count: {}", word_count).ok();
        if let Some(lang) = language {
            writeln!(stderr, "language: {} (BIP-39 checksum valid)", lang).ok();
        }
        writeln!(
            stderr,
            "passphrase: not stored in ms1 (record separately if used)"
        )
        .ok();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_entropy_accepts_canonical_zeros_16b() {
        let bytes = parse_hex_entropy("00000000000000000000000000000000").unwrap();
        assert_eq!(bytes.len(), 16);
        assert!(bytes.iter().all(|&b| b == 0));
    }

    #[test]
    fn parse_hex_entropy_rejects_odd_length() {
        let err = parse_hex_entropy("0").unwrap_err();
        assert!(matches!(err, CliError::BadInput(_)));
    }

    #[test]
    fn parse_hex_entropy_rejects_empty() {
        let err = parse_hex_entropy("").unwrap_err();
        assert!(matches!(err, CliError::BadInput(m) if m.contains("empty")));
    }

    #[test]
    fn parse_hex_entropy_rejects_non_hex_char() {
        let err = parse_hex_entropy("ZZ").unwrap_err();
        match err {
            CliError::BadInput(m) => {
                assert!(m.contains("'Z'"), "got: {}", m);
                assert!(m.contains("position 0"));
            }
            _ => panic!("expected BadInput"),
        }
    }
}
