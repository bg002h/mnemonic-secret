//! `ms decode` — recover a BIP-39 mnemonic from an ms1 string.
//!
//! Realizes SPEC §2.2 (full command surface), §5.2 (--json schema),
//! §6.3 (default-language hazard surfacing on stdout AND stderr).

use std::io::Write;

use bip39::{Language, Mnemonic};
use clap::Args;
use ms_codec::Payload;
use serde_json::to_string;

use crate::advisory::{OutputClass, emit_output_class_advisory};
use crate::error::Result;
use crate::format::DecodeJson;
use crate::language::CliLanguage;
use crate::parse::read_input;

/// `ms decode` arguments.
#[derive(Args, Debug)]
pub struct DecodeArgs {
    /// ms1 string to decode. Use `-` or omit to read from stdin.
    pub ms1: Option<String>,

    /// BIP-39 wordlist for the recovered phrase. Default `english`.
    /// SPEC §6.3: when defaulted, both stderr AND the stdout language
    /// line carry an explicit "DEFAULT" annotation.
    #[arg(long)]
    pub language: Option<CliLanguage>,

    /// Emit a single JSON object on stdout instead of labeled-block text.
    #[arg(long)]
    pub json: bool,
}

/// Run `ms decode`.
pub fn run(args: DecodeArgs) -> Result<u8> {
    use zeroize::Zeroizing;
    // Note: `ms1` is the codex32 string, not directly secret-bearing,
    // but it's encrypted-form-equivalent (an attacker with this string
    // can recover the entropy). Wrap defensively.
    let ms1: Zeroizing<String> = Zeroizing::new(read_input(args.ms1.as_deref())?);

    let (cli_lang, defaulted) = match args.language {
        Some(l) => (l, false),
        None => (CliLanguage::English, true),
    };

    let (_tag, payload) = ms_codec::decode(&ms1)?;
    // SPEC v0.9.0 §1 item 2 — wrap the entropy Vec at the consumer
    // boundary per `payload.rs` caller-wrap contract.
    //
    // For Payload::Mnem the wire language byte is authoritative: if the user
    // passed --language AND it disagrees with the wire, the wire wins and we
    // print a stderr note (exit 0). "Explicitly passed" is detectable because
    // args.language is Option<CliLanguage> — Some means user-set, None means
    // defaulted.
    //
    // Two-step: borrow first to compute language + emit warning, then consume
    // to extract entropy. The standalone typed binding `let entropy:
    // Zeroizing<Vec<u8>>` is the greppable discipline anchor required by the
    // `lint_zeroize_discipline` test (every_canonical_zeroize_row_has_evidence_anchor).
    let (effective_lang, effective_lang_defaulted) = match &payload {
        Payload::Entr(_) => (cli_lang, defaulted),
        Payload::Mnem { language: wire_code, .. } => {
            let wire_cli_lang = CliLanguage::from_code(*wire_code).unwrap_or(CliLanguage::English);
            // Wire wins; warn only if user EXPLICITLY supplied --language that disagrees.
            if !defaulted && wire_cli_lang != cli_lang {
                let mut stderr = std::io::stderr().lock();
                writeln!(
                    stderr,
                    "note: this ms1 carries wordlist language '{}'; ignoring --language {}",
                    wire_cli_lang.as_str(),
                    cli_lang.as_str()
                )
                .ok();
            }
            (wire_cli_lang, false)
        }
        // ms_codec::Payload is #[non_exhaustive]; guard against future variants.
        _ => unreachable!("ms-codec decode returned unknown Payload variant"),
    };
    let entropy: Zeroizing<Vec<u8>> = match payload {
        Payload::Entr(b) => Zeroizing::new(b),
        Payload::Mnem { entropy, .. } => Zeroizing::new(entropy),
        _ => unreachable!("ms-codec decode returned unknown Payload variant"),
    };

    let lang: Language = effective_lang.into();
    // SAFETY: third-party-blocked — `bip39::Mnemonic` has no Drop+Zeroize;
    // FOLLOWUP `rust-bip39-mnemonic-zeroize-upstream`.
    let mnemonic = Mnemonic::from_entropy_in(lang, &entropy[..])
        .expect("ms-codec validates entropy length; from_entropy_in cannot fail");
    let phrase: Zeroizing<String> = Zeroizing::new(mnemonic.to_string());
    let word_count = phrase.split_whitespace().count();

    if args.json {
        emit_json(
            &entropy[..],
            &phrase,
            effective_lang.as_str(),
            word_count,
            effective_lang_defaulted,
        )?;
    } else {
        emit_text(
            &entropy[..],
            &phrase,
            effective_lang.as_str(),
            word_count,
            effective_lang_defaulted,
        )?;
    }
    emit_output_class_advisory(OutputClass::PrivateKeyMaterial, &mut std::io::stderr().lock());
    Ok(0)
}

fn emit_json(
    entropy: &[u8],
    phrase: &str,
    language: &str,
    word_count: usize,
    language_defaulted: bool,
) -> Result<()> {
    let json = DecodeJson {
        schema_version: "1",
        entropy_hex: hex::encode(entropy),
        phrase: phrase.to_string(),
        language,
        word_count,
        language_defaulted,
    };
    let s = to_string(&json).expect("decode json serialization always succeeds");
    println!("{}", s);
    Ok(())
}

fn emit_text(
    entropy: &[u8],
    phrase: &str,
    language: &str,
    word_count: usize,
    language_defaulted: bool,
) -> Result<()> {
    println!("entropy: {}", hex::encode(entropy));
    println!("phrase: {}", phrase);
    if language_defaulted {
        println!(
            "language: {} ({} words, default — verify against your records)",
            language, word_count
        );
        let mut stderr = std::io::stderr().lock();
        writeln!(
            stderr,
            "note: --language defaulted to '{}'; if your wallet was created with a different wordlist, decode with --language <lang>.",
            language
        )
        .ok();
    } else {
        println!("language: {} ({} words)", language, word_count);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    // Decode logic is mostly delegation to ms-codec + bip39; integration tests
    // (Phase 4) cover the stdout/stderr formatting end-to-end. No unit tests
    // here — would just be re-tests of bip39's own `from_entropy_in`.
}
