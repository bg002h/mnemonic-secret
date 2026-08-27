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

use crate::advisory::{emit_output_class_advisory, OutputClass};
use crate::error::{CliError, Result};
use crate::format::{render_grouped, EncodeJson};
use crate::language::CliLanguage;
use crate::parse::{read_input, read_phrase_input, Source};

/// `ms encode` arguments.
///
/// `--phrase` and `--hex` form a mutually-exclusive group; exactly one MUST
/// be supplied. The `#[command(group = ...)]` declaration scopes the exclusion
/// to just `phrase` + `hex`; encode_arg_group_violations.rs (Phase 4) tests
/// this with exit 64 on both-supplied and neither-supplied inputs.
#[derive(Args, Debug)]
#[command(group = clap::ArgGroup::new("input").required(true).args(["phrase", "hex", "in_path"]))]
pub struct EncodeArgs {
    /// Write the canonical artifact to FILE, **owner-only (0600)**.
    ///
    /// Through the shared crate's `write::write_private`, which sets the mode on
    /// the OPEN FILE as well as passing it to `OpenOptions` -- the latter binds
    /// on CREATE only, so an existing 0644 target would otherwise stay 0644 and
    /// the tool would report success. It OVERWRITES (§6b), and truncates, so a
    /// shrinking rewrite leaves no tail of the old file.
    ///
    /// **Not to be confused with `ms gen-man --out <DIR>`**, which is shipped,
    /// exampled, driven by this repo's `man-release.yml` and by
    /// `scripts/install.sh` in mnemonic-toolkit, and means a DIRECTORY. P2 does
    /// not rename it; F-282 records that one binary now carries two meanings.
    #[arg(long, value_name = "FILE")]
    pub out: Option<std::path::PathBuf>,

    /// Proceed even though secret material is on argv.
    ///
    /// **Read off RAW argv before the parser, and it is a CHANNEL rather than a
    /// flag** (§6d): the admitted value is replaced by `-` and routed to the
    /// verb through a side channel, so it is never handed back to clap. It is
    /// declared here so `--help` documents it, and destructured nowhere --
    /// consulting it after parsing would be a decision reached too late.
    ///
    /// For a single-user air-gapped box, or an amnesic Tails session.
    #[arg(long)]
    pub allow_argv_secret: bool,

    /// BIP-39 mnemonic. Use `-` to read from stdin.
    #[arg(long)]
    pub phrase: Option<String>,

    /// Hex-encoded entropy bytes (16/20/24/28/32 B = 32/40/48/56/64 hex chars).
    #[arg(long)]
    pub hex: Option<String>,

    /// Read the BIP-39 PHRASE from FILE (never hex — use `--hex - < FILE`).
    ///
    /// **`--in` means a phrase, and refuses to sniff.** A sniffing `--in` would
    /// be safe today only because a phrase always contains whitespace, and that
    /// restraint is invisible: a later maintainer being liberal with whitespace
    /// turns a hex-alphabet BIP-39 phrase into valid entropy for a DIFFERENT
    /// wallet — a valid, wrong plate. The phrase-only rule has no input that
    /// both parses as BIP-39 and reads as entropy.
    #[arg(long = "in", value_name = "FILE")]
    pub in_path: Option<std::path::PathBuf>,

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
    in_path: Option<&std::path::Path>,
    language: CliLanguage,
    verb: &'static str,
) -> Result<(Payload, Option<&'static str>)> {
    use zeroize::Zeroizing;

    // clap's mutually-exclusive group enforces exactly-one-of-{phrase,hex}.
    // `entropy` is the secret byte buffer; `language_for_card` is the 2nd
    // element returned to the caller. Both are bound here (lint anchor:
    // `let (entropy, language_for_card): (Zeroizing<Vec<u8>>`).
    // `--in` and `--phrase` are ONE channel -- the phrase channel -- reached
    // from two places. clap's required group already guarantees at most one is
    // present, so this is a routing decision and not a precedence one.
    let phrase_source: Option<crate::parse::Source> = match (in_path, phrase) {
        (Some(p), _) => Some(Source::new(None, Some(p))),
        (None, Some(a)) => Some(Source::new(Some(a), None).on("--phrase")),
        (None, None) => None,
    };

    let (entropy, language_for_card): (Zeroizing<Vec<u8>>, Option<&'static str>) =
        if let Some(src) = phrase_source {
            let phrase: Zeroizing<String> = read_phrase_input(src)?;
            let lang: Language = language.into();
            // SAFETY: third-party-blocked — `bip39::Mnemonic` has no Drop+
            // Zeroize; tracked at FOLLOWUP `rust-bip39-mnemonic-zeroize-upstream`
            // (companion of the mnemonic-toolkit cycle entry).
            let mnemonic = Mnemonic::parse_in(lang, phrase.as_str())
                .map_err(|e| phrase_parse_error(e, in_path, verb))?;
            (
                Zeroizing::new(mnemonic.to_entropy()),
                Some(language.as_str()),
            )
        } else if let Some(hex_arg) = hex {
            let hex_str = Zeroizing::new(read_input(Source::new(Some(hex_arg), None).on("--hex"))?);
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
    let payload = if language != CliLanguage::English && phrase_source.is_some() {
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
    let hex_arg: Option<Zeroizing<String>> = std::mem::take(&mut args.hex).map(Zeroizing::new);

    // Shared entropy-resolution + AUTO-route (also used by `ms split`).
    let (payload, language_for_card) = resolve_secret_payload(
        phrase_arg.as_ref().map(|p| p.as_str()),
        hex_arg.as_ref().map(|h| h.as_str()),
        args.in_path.as_deref(),
        args.language,
        "encode",
    )?;

    let ms1 = ms_codec::encode(Tag::ENTR, &payload)?;

    // Re-derive the output `entropy` view from the resolved Payload (the
    // entropy bytes, sans prefix/language). word_count + entropy_hex use it.
    let entropy: Zeroizing<Vec<u8>> = Zeroizing::new(payload.as_bytes().to_vec());
    let word_count = entropy.len() * 3 / 4; // 16->12, 20->15, 24->18, 28->21, 32->24

    // `--out` receives the CANONICAL artifact -- ungrouped, newline-terminated
    // -- because it exists so the next tool can read the file. When it is given,
    // the text form no longer repeats the artifact on stdout: the operator asked
    // for it in a 0600 file, and printing it as well would put the same secret on
    // a stream that is usually redirected into a 0644 one. The stderr engraving
    // card is unaffected, and `--json` (a REPORT, not the artifact) still goes to
    // stdout.
    if let Some(path) = args.out.as_deref() {
        crate::out::write_artifact(path, &format!("{ms1}\n"))?;
    }

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
            args.out.is_some(),
        )?;
    }
    emit_output_class_advisory(
        OutputClass::PrivateKeyMaterial,
        &mut std::io::stderr().lock(),
    );
    Ok(0)
}

/// The BIP-39 parse failure — **and, when the material arrived through
/// `--in`, an executable redirect printed beside it.**
///
/// **The error KIND and exit code are unchanged, deliberately.** An earlier
/// draft returned `BadInput` here so the redirect could be part of the message,
/// which made the same file report `Bip39` through `--phrase -` and `BadInput`
/// through `--in` — one input, two error kinds, and `--json`'s envelope
/// disagreeing with itself depending on which private channel was used. The
/// note goes to stderr instead, which is where `ms`'s other advisories go, so
/// `error.kind` stays a property of the INPUT.
///
/// §6h: remedy text names channels that exist and commands that RUN. An
/// operator who points `--in` at a file of hex otherwise gets a wordlist error
/// and no way forward except `--allow-argv-secret`, which is the exposure this
/// phase exists to close.
fn phrase_parse_error(
    e: bip39::Error,
    in_path: Option<&std::path::Path>,
    verb: &'static str,
) -> CliError {
    if let Some(p) = in_path {
        let mut stderr = std::io::stderr().lock();
        let _ = writeln!(
            stderr,
            "note: --in reads a PHRASE and never sniffs the file's contents. If \
             {path} holds hex entropy, use the hex channel:\n      \
             \x20   ms {verb} --hex - < {path}",
            path = p.display()
        );
    }
    CliError::Bip39(e)
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
    // cycle-15 Lane M (slug #8, defense-in-depth): scrub the serialized
    // entropy-bearing JSON buffer on drop.
    let s: zeroize::Zeroizing<String> = zeroize::Zeroizing::new(
        to_string(&json).map_err(|e| CliError::BadInput(format!("json serialization: {}", e)))?,
    );
    println!("{}", *s);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn emit_text(
    ms1: &str,
    language: Option<&str>,
    word_count: usize,
    no_engraving_card: bool,
    group_size: usize,
    separator: char,
    artifact_went_to_a_file: bool,
) -> Result<()> {
    // Print-once stdout: the ms1 in the flag-controlled grouped form (SPEC §6).
    if !artifact_went_to_a_file {
        println!("{}", render_grouped(ms1, group_size, separator));
    }

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
