//! `ms combine` — recombine K-of-N codex32 shares into the original secret.
//!
//! Realizes SPEC_ms_v0_2_kofn §3 (`ms combine`). Takes K (or more) distributed
//! share strings; `ms_codec::combine_shares` recovers the secret-at-S and the
//! payload kind (entr / mnem). Emits per `--to`: a BIP-39 phrase (default;
//! mnem → on-wire language, entr → English), raw entropy hex, or a single ms1
//! string. The recovered secret is PrivateKeyMaterial (advisory) and
//! Zeroizing-wrapped.

use bip39::{Language, Mnemonic};
use clap::{Args, ValueEnum};
use ms_codec::{Payload, Tag};
use serde_json::to_string;
use zeroize::Zeroizing;

use crate::advisory::{OutputClass, emit_output_class_advisory};
use crate::error::{CliError, Result};
use crate::format::CombineJson;
use crate::language::CliLanguage;

/// Output form for the recovered secret.
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum CombineTo {
    /// BIP-39 mnemonic phrase (mnem → wire language; entr → English).
    Phrase,
    /// Raw entropy bytes as hex.
    Entropy,
    /// A single unshared ms1 string (re-encodes the recovered secret).
    Ms1,
}

/// `ms combine` arguments.
#[derive(Args, Debug)]
pub struct CombineArgs {
    /// The distributed share strings to recombine (K or more, distinct indices).
    #[arg(required = true)]
    pub shares: Vec<String>,

    /// Output form for the recovered secret. Default `phrase`.
    #[arg(long, value_enum, default_value = "phrase")]
    pub to: CombineTo,

    /// Emit a single JSON object on stdout instead of text.
    #[arg(long)]
    pub json: bool,
}

/// Read the share strings: positional `args` minus a leading `-`, which means
/// "read one share per line from stdin" (parallel to `ms split | ms combine -`
/// and mk-cli's `read_mk1_strings`). Each share (positional OR stdin line) is
/// stripped of mstring display separators (SPEC §3.2 / §15 C1+C3) so a grouped
/// or unbroken card both re-ingest. Shares are secret-equivalent → Zeroizing.
fn read_shares(args: &[String]) -> Result<Zeroizing<Vec<String>>> {
    let mut out: Vec<String> = Vec::with_capacity(args.len());
    let mut consumed_stdin = false;
    for a in args {
        if a == "-" && !consumed_stdin {
            consumed_stdin = true;
            let buf = crate::parse::read_stdin()?;
            for line in buf.lines() {
                let s = crate::format::strip_display_separators(line);
                if !s.is_empty() {
                    out.push(s);
                }
            }
        } else if a == "-" {
            // Already consumed stdin; ignore additional `-` markers.
        } else {
            out.push(crate::format::strip_display_separators(a));
        }
    }
    if out.is_empty() {
        return Err(CliError::BadInput(
            "expected at least one share (positional or via stdin with '-')".into(),
        ));
    }
    Ok(Zeroizing::new(out))
}

/// Run `ms combine`. Writes the recovered secret to stdout per the `--to` form.
pub fn run(mut args: CombineArgs) -> Result<u8> {
    // The share strings themselves are secret-equivalent — wrap them. `-` reads
    // one share per line from stdin; display separators are stripped per share.
    let shares: Zeroizing<Vec<String>> = read_shares(&std::mem::take(&mut args.shares))?;

    // Recover the secret. combine_shares surfaces the §2 error taxonomy
    // (SecretShareSuppliedToCombine / Codex32(ThresholdNotPassed/Mismatched*/
    // RepeatedIndex)) which the From<ms_codec::Error> mapping + codex32_friendly
    // render into clean messages.
    let (_tag, payload): (Tag, Payload) = ms_codec::combine_shares(&shares)?;

    // Extract the entropy + language for rendering. The on-wire language byte is
    // authoritative for mnem; entr renders as English.
    let (entropy, language, kind): (Zeroizing<Vec<u8>>, CliLanguage, &'static str) = match &payload
    {
        Payload::Entr(b) => (Zeroizing::new(b.clone()), CliLanguage::English, "entr"),
        Payload::Mnem {
            language: wire_code,
            entropy,
        } => {
            let lang = CliLanguage::from_code(*wire_code).unwrap_or(CliLanguage::English);
            (Zeroizing::new(entropy.clone()), lang, "mnem")
        }
        // ms_codec::Payload is #[non_exhaustive]; guard against future variants.
        _ => unreachable!("combine_shares returned an unknown Payload variant"),
    };

    match args.to {
        CombineTo::Phrase => emit_phrase(&entropy, language, kind, args.json)?,
        CombineTo::Entropy => emit_entropy(&entropy, kind, args.json)?,
        CombineTo::Ms1 => emit_ms1(&payload, &entropy, kind, args.json)?,
    }

    emit_output_class_advisory(OutputClass::PrivateKeyMaterial, &mut std::io::stderr().lock());
    Ok(0)
}

/// Render the recovered secret as a BIP-39 phrase in its language.
fn emit_phrase(
    entropy: &[u8],
    language: CliLanguage,
    kind: &'static str,
    json: bool,
) -> Result<()> {
    let lang: Language = language.into();
    // SAFETY: third-party-blocked — `bip39::Mnemonic` has no Drop+Zeroize;
    // FOLLOWUP `rust-bip39-mnemonic-zeroize-upstream`.
    let mnemonic = Mnemonic::from_entropy_in(lang, entropy)
        .expect("combine_shares validates entropy length; from_entropy_in cannot fail");
    let phrase: Zeroizing<String> = Zeroizing::new(mnemonic.to_string());
    let word_count = phrase.split_whitespace().count();

    if json {
        let json = CombineJson {
            schema_version: "1",
            kind: kind.to_string(),
            ms1: None,
            entropy_hex: hex::encode(entropy),
            phrase: Some(phrase.to_string()),
            language: Some(language.as_str()),
            word_count: Some(word_count),
        };
        let s =
            to_string(&json).map_err(|e| CliError::BadInput(format!("json serialization: {e}")))?;
        println!("{s}");
    } else {
        println!("entropy: {}", hex::encode(entropy));
        println!("phrase: {}", *phrase);
        println!("language: {} ({} words)", language.as_str(), word_count);
        println!("kind: {kind}");
    }
    Ok(())
}

/// Render just the raw entropy hex.
fn emit_entropy(entropy: &[u8], kind: &'static str, json: bool) -> Result<()> {
    if json {
        let json = CombineJson {
            schema_version: "1",
            kind: kind.to_string(),
            ms1: None,
            entropy_hex: hex::encode(entropy),
            phrase: None,
            language: None,
            word_count: None,
        };
        let s =
            to_string(&json).map_err(|e| CliError::BadInput(format!("json serialization: {e}")))?;
        println!("{s}");
    } else {
        println!("{}", hex::encode(entropy));
    }
    Ok(())
}

/// Re-encode the recovered secret as a single (unshared) ms1 string.
fn emit_ms1(payload: &Payload, entropy: &[u8], kind: &'static str, json: bool) -> Result<()> {
    let ms1 = ms_codec::encode(Tag::ENTR, payload)?;
    if json {
        let json = CombineJson {
            schema_version: "1",
            kind: kind.to_string(),
            ms1: Some(ms1.clone()),
            entropy_hex: hex::encode(entropy),
            phrase: None,
            language: None,
            word_count: None,
        };
        let s =
            to_string(&json).map_err(|e| CliError::BadInput(format!("json serialization: {e}")))?;
        println!("{s}");
    } else {
        println!("{ms1}");
    }
    Ok(())
}
