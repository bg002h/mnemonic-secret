//! `ms decode` — recover a BIP-39 mnemonic from an ms1 string.
//!
//! Realizes SPEC §2.2 (full command surface), §5.2 (--json schema),
//! §6.3 (default-language hazard surfacing on stdout AND stderr).

use std::io::Write;

use bip39::{Language, Mnemonic};
use clap::Args;
use ms_codec::Payload;
use serde_json::to_string;

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
pub fn run(args: DecodeArgs) -> Result<()> {
    let ms1 = read_input(args.ms1.as_deref())?;

    let (cli_lang, defaulted) = match args.language {
        Some(l) => (l, false),
        None => (CliLanguage::English, true),
    };
    let lang: Language = cli_lang.into();

    let (_tag, payload) = ms_codec::decode(&ms1)?;
    let entropy = match payload {
        Payload::Entr(b) => b,
        // ms_codec::Payload is #[non_exhaustive]; v0.2+ may add variants.
        // v0.1 ms-codec emits Entr only — unreachable in practice.
        _ => unreachable!("ms-codec v0.1 only decodes to Payload::Entr"),
    };

    let mnemonic = Mnemonic::from_entropy_in(lang, &entropy)
        .expect("ms-codec validates entropy length; from_entropy_in cannot fail");
    let phrase = mnemonic.to_string();
    let word_count = phrase.split_whitespace().count();

    if args.json {
        emit_json(&entropy, &phrase, cli_lang.as_str(), word_count, defaulted)?;
    } else {
        emit_text(&entropy, &phrase, cli_lang.as_str(), word_count, defaulted)?;
    }
    Ok(())
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
