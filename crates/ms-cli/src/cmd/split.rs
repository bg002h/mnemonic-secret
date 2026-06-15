//! `ms split` — split a secret (entr / mnem) into N codex32 K-of-N shares.
//!
//! Realizes SPEC_ms_v0_2_kofn §3 (`ms split`). The secret source uses the same
//! `--phrase`/`--hex`/`--language` forms as `ms encode` (via the shared
//! `encode::resolve_secret_payload` helper), so a non-English phrase splits as a
//! `mnem` share-set (the wordlist language survives the split). Emits N share
//! strings; any K recombine via `ms combine`. The whole N-share SET is
//! secret-equivalent → `PrivateKeyMaterial` advisory.

use std::io::Write;

use clap::Args;
use ms_codec::{PayloadKind, Tag, Threshold};
use serde_json::to_string;
use zeroize::Zeroizing;

use crate::advisory::{OutputClass, emit_output_class_advisory};
use crate::cmd::encode::resolve_secret_payload;
use crate::error::{CliError, Result};
use crate::format::{SplitJson, render_grouped};
use crate::language::CliLanguage;

/// `ms split` arguments.
///
/// `--phrase` and `--hex` form a required mutually-exclusive group (mirrors
/// `ms encode`). `-k`/`-n` carry the K-of-N threshold + share count.
#[derive(Args, Debug)]
#[command(group = clap::ArgGroup::new("split_input").required(true).args(["phrase", "hex"]))]
pub struct SplitArgs {
    /// BIP-39 mnemonic to split. Use `-` to read from stdin.
    #[arg(long)]
    pub phrase: Option<String>,

    /// Hex-encoded entropy bytes to split (16/20/24/28/32 B = 32/40/48/56/64 hex chars).
    #[arg(long)]
    pub hex: Option<String>,

    /// BIP-39 wordlist for the input phrase. Ignored under --hex.
    #[arg(long, default_value = "english")]
    pub language: CliLanguage,

    /// Threshold K — the minimum number of shares needed to recombine (2..=9).
    #[arg(short = 'k', long = "threshold")]
    pub k: u8,

    /// Total number of shares N to produce (K ≤ N ≤ 31).
    #[arg(short = 'n', long = "shares")]
    pub n: usize,

    /// Insert a separator every N characters in each emitted share string
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

/// Run `ms split`. Writes the N shares to stdout per SPEC_ms_v0_2_kofn §3.
pub fn run(mut args: SplitArgs) -> Result<u8> {
    // Consume + Zeroizing-wrap the clap-owned secret fields at run() entry
    // (mirrors encode::run; clap-derive can't emit Zeroizing<String>).
    let phrase_arg: Option<Zeroizing<String>> =
        std::mem::take(&mut args.phrase).map(Zeroizing::new);
    let hex_arg: Option<Zeroizing<String>> = std::mem::take(&mut args.hex).map(Zeroizing::new);

    // Shared entropy-resolution + AUTO-route (English/hex → entr; non-English
    // phrase → mnem). The 2nd tuple element (language_for_card) is ignored —
    // split derives the JSON `language` field from the Payload itself.
    let (payload, _language_for_card) = resolve_secret_payload(
        phrase_arg.as_ref().map(|p| p.as_str()),
        hex_arg.as_ref().map(|h| h.as_str()),
        args.language,
    )?;

    // K-of-N split. `Threshold::new` rejects k ∉ 2..=9 (→ InvalidThreshold →
    // BadInput exit 1); `encode_shares` rejects n ∉ k..=31 (→ InvalidShareCount).
    let threshold = Threshold::new(args.k)?;
    let shares = ms_codec::encode_shares(Tag::ENTR, threshold, args.n, &payload)?;

    // Each share carries the share-set's shared random id + threshold char;
    // re-read the id off the first share for the report.
    let id = share_id(&shares[0]);
    let (kind, language): (&'static str, Option<&'static str>) = match payload.kind() {
        PayloadKind::Entr => ("entr", None),
        PayloadKind::Mnem => ("mnem", Some(language_str_for_payload(&payload, args.language))),
        // PayloadKind is #[non_exhaustive]; guard against future kinds.
        _ => ("unknown", None),
    };

    if args.json {
        emit_json(&shares, args.k, args.n, &id, kind, language)?;
    } else {
        emit_text(&shares, args.group_size as usize, args.separator);
    }

    // The N-share SET is secret-equivalent (any K reconstruct the secret).
    emit_output_class_advisory(OutputClass::PrivateKeyMaterial, &mut std::io::stderr().lock());
    Ok(0)
}

/// The 4-char codex32 id field of a share string (between the `1` separator +
/// threshold char and the share-index char).
fn share_id(share: &str) -> String {
    match share.rfind('1') {
        Some(sep) if share.len() >= sep + 6 => share[sep + 2..sep + 6].to_string(),
        // Defensive: any parseable codex32 short string has >= 48 chars, so the
        // id slice is always present. Fall back to empty rather than panic.
        _ => String::new(),
    }
}

/// The wordlist language name for a `mnem` payload. The on-wire language byte is
/// authoritative; fall back to the CLI `--language` if the byte is out of range
/// (cannot happen for a freshly-split payload — `validate()` already gated it).
fn language_str_for_payload(payload: &ms_codec::Payload, cli_lang: CliLanguage) -> &'static str {
    if let ms_codec::Payload::Mnem { language, .. } = payload {
        CliLanguage::from_code(*language)
            .unwrap_or(cli_lang)
            .as_str()
    } else {
        cli_lang.as_str()
    }
}

fn emit_json(
    shares: &[String],
    k: u8,
    n: usize,
    id: &str,
    kind: &str,
    language: Option<&str>,
) -> Result<()> {
    let json = SplitJson {
        schema_version: "1",
        shares: shares.to_vec(),
        k,
        n,
        id: id.to_string(),
        kind: kind.to_string(),
        language,
    };
    let s = to_string(&json).map_err(|e| CliError::BadInput(format!("json serialization: {e}")))?;
    println!("{s}");
    Ok(())
}

/// Text form (print-once, SPEC §6 + §15 C1/C2): stdout carries the N share
/// strings one per line in the flag-controlled grouped form (machine-pipeable
/// into `ms combine -`); all human labels ("share i of n") move to stderr
/// (mirrors the engraving-card panel). Emit ALL stdout shares first, then the
/// stderr labels.
fn emit_text(shares: &[String], group_size: usize, separator: char) {
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for share in shares {
        let _ = writeln!(out, "{}", render_grouped(share, group_size, separator));
    }
    let stderr = std::io::stderr();
    let mut err = stderr.lock();
    for (i, _share) in shares.iter().enumerate() {
        let _ = writeln!(err, "share {} of {}:", i + 1, shares.len());
    }
}
