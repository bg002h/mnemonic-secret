//! `ms verify` — exit-code-only validity (and optional --phrase round-trip).
//!
//! Realizes SPEC §2.4 (full command), §2.4.1 (locked validation order:
//! decode -> exit on failure -> parse phrase -> compare -> exit), §6 exit
//! codes 0 (valid) / 1 (user-input) / 2 (format) / 3 (future format) /
//! 4 (round-trip mismatch).

use bip39::{Language, Mnemonic};
use clap::Args;
use ms_codec::Payload;
use serde_json::to_string;

use crate::error::{CliError, Result};
use crate::format::VerifySuccessJson;
use crate::language::CliLanguage;
use crate::parse::{is_stdin_arg, read_input, read_phrase_input};

/// `ms verify` arguments.
#[derive(Args, Debug)]
pub struct VerifyArgs {
    /// ms1 string to verify. Use `-` or omit to read from stdin.
    pub ms1: Option<String>,

    /// Original BIP-39 phrase to round-trip-check against the decoded entropy.
    /// When supplied, exit 4 on mismatch. Use `-` to read phrase from stdin.
    #[arg(long)]
    pub phrase: Option<String>,

    /// BIP-39 wordlist for --phrase. Default `english`.
    #[arg(long, default_value = "english")]
    pub language: CliLanguage,

    /// Emit success JSON on stdout (mirrors the §5 schema-versioned form).
    #[arg(long)]
    pub json: bool,
}

/// Run `ms verify` per SPEC §2.4.1 validation order.
pub fn run(args: VerifyArgs) -> Result<()> {
    // Step 1: read ms1 input. Concurrent-stdin guard: if both ms1 and --phrase
    // resolve to stdin, exit immediately (clap can't catch this).
    if is_stdin_arg(args.ms1.as_deref()) && args.phrase.as_deref() == Some("-") {
        return Err(CliError::BadInput(
            "cannot read both ms1 and --phrase from stdin".into(),
        ));
    }
    let ms1 = read_input(args.ms1.as_deref())?;

    // Step 2: decode the ms1 string. On failure, dispatch per §6.1.1 — phrase
    // is NEVER parsed in this branch.
    let decoded = ms_codec::decode(&ms1);
    let entropy = match decoded {
        Ok((_tag, Payload::Entr(b))) => b,
        // ms_codec::Payload is #[non_exhaustive]; v0.2+ may add variants.
        // v0.1 ms-codec only decodes to Payload::Entr; defensive arm only.
        Ok((_, _)) => unreachable!("ms-codec v0.1 only decodes to Payload::Entr"),
        Err(ms_codec::Error::ReservedTagNotEmittedInV01 { got }) => {
            // Exit 3 path: print the success-shaped "valid future format" message.
            return emit_future_format(&got, args.json);
        }
        Err(e) => return Err(e.into()),
    };

    // Step 3: parse --phrase if present.
    let phrase_supplied = match &args.phrase {
        Some(p) => Some(read_phrase_input(Some(p))?),
        None => None,
    };

    // Step 4: compare or exit-0 quick.
    if let Some(supplied) = phrase_supplied {
        let lang: Language = args.language.into();
        let supplied_mnemonic = Mnemonic::parse_in(lang, &supplied)?;
        let derived_mnemonic =
            Mnemonic::from_entropy_in(lang, &entropy).expect("ms-codec validates entropy length");
        if supplied_mnemonic.to_string() == derived_mnemonic.to_string() {
            return emit_round_trip_ok(&derived_mnemonic, args.language.as_str(), args.json);
        } else {
            return Err(CliError::VerifyPhraseMismatch);
        }
    }

    // No --phrase: simple validity OK.
    let word_count = entropy.len() * 3 / 4;
    let str_len = ms1.len();
    emit_simple_ok(word_count, str_len, args.json)
}

fn emit_simple_ok(word_count: usize, str_len: usize, json: bool) -> Result<()> {
    if json {
        let j = VerifySuccessJson {
            schema_version: "1",
            status: "valid",
            message: &format!("valid v0.1 entr ({} words, {} chars)", word_count, str_len),
        };
        println!("{}", to_string(&j).expect("verify json"));
    } else {
        println!(
            "OK: valid v0.1 entr ({} words, {} chars)",
            word_count, str_len
        );
    }
    Ok(())
}

fn emit_future_format(tag: &[u8; 4], json: bool) -> Result<()> {
    let tag_str = std::str::from_utf8(tag).unwrap_or("<non-utf8>");
    // Text mode: print success-shaped OK line. JSON mode: do NOT print here —
    // main.rs's ExitCode dispatch invokes emit_error which prints the error
    // envelope; printing a success line here would yield two outputs on stdout.
    if !json {
        println!("OK: valid future format (v0.2+, tag {})", tag_str);
    }
    // Either way, return Err(FutureFormat) so main.rs lands exit 3. In JSON
    // mode the error envelope (with kind="FutureFormat", exit_code=3) becomes
    // the sole stdout output; text-mode users see the OK line above + (since
    // ExitCode != 0) any stderr emit_error would write — but main.rs's
    // emit_error writes to stdout in --json mode and to stderr in text mode,
    // so text mode emits "error: ..." stderr alongside the OK stdout line.
    // That's intentionally redundant — exit-3 is "OK semantically" but the
    // err-path-with-stderr-display flags it for users who only watch stderr.
    Err(CliError::FutureFormat { tag: *tag })
}

fn emit_round_trip_ok(_mnemonic: &Mnemonic, language: &str, json: bool) -> Result<()> {
    let word_count = _mnemonic.to_string().split_whitespace().count();
    if json {
        let j = VerifySuccessJson {
            schema_version: "1",
            status: "round-trip-ok",
            message: &format!(
                "round-trip valid ({} words, language={})",
                word_count, language
            ),
        };
        println!("{}", to_string(&j).expect("verify json"));
    } else {
        println!(
            "OK: round-trip valid ({} words, language={})",
            word_count, language
        );
    }
    Ok(())
}
