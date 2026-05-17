//! `ms repair` — BCH error-correction for ms1 strings.
//!
//! Realizes plan §2.B.3 (v0.22.x follow-ups Tranche B.5). Wraps
//! `ms_codec::decode_with_correction` (which performs full BCH(93,80,8)
//! correction up to t=4 and returns
//! `(Tag, Payload, Vec<CorrectionDetail>)`) and renders a per-input repair
//! report.
//!
//! Single-chunk context: ms1 is single-chunk per codex32 spec, so the
//! `--ms1` flag is non-repeating (cf. mk-cli's variadic `mk1_strings`).
//! Single-HRP context (always `ms`): no `--hrp` flag and no Levenshtein
//! "did you mean" suggestion — `decode_with_correction` validates HRP
//! internally against `"ms"`. HRP mismatches surface as exit 2 (via
//! `From<ms_codec::Error>` mapping `WrongHrp` → `CliError::FormatViolation`).
//!
//! Exit codes (D26 cross-CLI parity):
//!   - 0 — input was already valid (no corrections applied)
//!   - 5 — corrections were applied (REPAIR_APPLIED)
//!   - 2 — unrepairable input (`CliError::FormatViolation`) — propagated
//!     by `?`. The new v0.2.0 `ms_codec::Error::TooManyErrors` is mapped
//!     to `FormatViolation` in `error.rs` for parity with mk-cli's
//!     `CliError::Codec(_) → 2` rule.
//!
//! D9 secret-on-stdout: ms1 IS the seed-secret (BIP-39 entropy), so this
//! subcommand ALWAYS emits a stderr advisory when invoked (regardless of
//! whether corrections fired or whether `--json` was set — even
//! pass-through of a valid ms1 to stdout is sensitive material on stdout).
//! Byte-matches `mnemonic-toolkit/src/secret_advisory.rs::secret_on_stdout_warning`.
//!
//! Text output mirrors `mnemonic repair`'s text-form report shape (see
//! `mnemonic-toolkit/src/cmd/repair.rs::emit_repair_text`). JSON output
//! byte-matches the toolkit's standalone `RepairJson` schema (D27 — fields
//! `schema_version`, `kind`, `corrected_chunks`, `repairs`) so cross-CLI
//! parsers reuse the same struct.

use std::io::Write;

use clap::Args;
use ms_codec::CorrectionDetail;
use serde::Serialize;

use crate::error::{CliError, Result};
use crate::parse::read_input;

/// `ms repair` arguments.
#[derive(Args, Debug)]
pub struct RepairArgs {
    /// ms1 string to attempt to repair. Use `-` to read the string from
    /// stdin (a single line). Single-chunk per codex32 spec; non-repeating.
    #[arg(long, value_name = "MS1")]
    pub ms1: String,

    /// Emit a single JSON envelope on stdout instead of the text-form
    /// report. Schema byte-matches `mnemonic repair --json`'s
    /// `RepairJson` shape (cross-CLI parser reuse).
    #[arg(long)]
    pub json: bool,
}

/// Per-input repair report. Mirrors toolkit's `RepairDetail` shape so
/// JSON output is byte-identical to `mnemonic repair --json`.
#[derive(Debug, Clone)]
struct RepairDetail {
    chunk_index: usize,
    original_chunk: String,
    corrected_chunk: String,
    /// `(position, was, now)` — `position` is 0-indexed into the data-part
    /// (chars after the `ms` HRP + `1` separator).
    corrected_positions: Vec<(usize, char, char)>,
}

/// Run `ms repair`.
pub fn run(args: RepairArgs) -> Result<u8> {
    // `read_input` handles the `-` stdin sentinel (single line / trimmed).
    let original = read_input(Some(args.ms1.as_str()))?;

    // `decode_with_correction` performs BCH correction internally; HRP /
    // length / BCH-uncorrectable rejections surface as `ms_codec::Error`
    // and route to the appropriate exit code via `From<ms_codec::Error>`
    // in error.rs (`TooManyErrors` → `FormatViolation` → exit 2 per D26).
    let (_tag, _payload, corrections) = ms_codec::decode_with_correction(&original)?;

    let (corrected_chunk, corrected_positions) =
        reconstruct_corrected(&original, &corrections);

    // ms1 is single-chunk → exactly one RepairDetail with `chunk_index = 0`.
    let report = RepairDetail {
        chunk_index: 0,
        original_chunk: original.clone(),
        corrected_chunk: corrected_chunk.clone(),
        corrected_positions,
    };

    let corrected_chunks = vec![corrected_chunk];
    let reports = vec![report];

    if args.json {
        emit_json(&corrected_chunks, &reports)?;
    } else {
        emit_text(&corrected_chunks, &reports)?;
    }

    // D9: emit sensitive-secret stderr warning (always — even pass-through
    // of a valid ms1 to stdout is sensitive material on stdout). Byte-matches
    // mnemonic-toolkit's `secret_on_stdout_warning` for kind == Ms1.
    let _ = writeln!(
        std::io::stderr(),
        "warning: secret material on stdout — consider redirecting (e.g., '> file.txt' or '| age -e ...')"
    );

    let any_correction = reports.iter().any(|r| !r.corrected_positions.is_empty());
    Ok(if any_correction { 5 } else { 0 })
}

/// Build the corrected ms1 string + `(position, was, now)` triples from
/// the `CorrectionDetail` vector emitted by `decode_with_correction`.
/// Re-encodes by substituting characters at the reported data-part
/// positions in the original string; ms-codec already verified
/// post-correction polymod, so the substituted string is a valid codeword.
fn reconstruct_corrected(
    original: &str,
    corrections: &[CorrectionDetail],
) -> (String, Vec<(usize, char, char)>) {
    // ms1 HRP + separator = "ms1" (3 chars). The data-part begins at byte
    // offset 3 in the lowercased input. `decode_with_correction` already
    // validated the HRP; this `rfind('1')` is a sanity check matching
    // mk-cli's `reconstruct_corrected` shape.
    let sep_pos = original
        .rfind('1')
        .expect("ms1 input passed BCH decode; must contain bech32 separator '1'");
    let (prefix, rest) = original.split_at(sep_pos);
    // Lowercase the data part for substitution (`decode_with_correction`
    // lowercases internally, so the `was`/`now` chars are lowercase).
    let mut data_chars: Vec<char> = rest[1..].chars().map(|c| c.to_ascii_lowercase()).collect();

    let mut corrected_positions: Vec<(usize, char, char)> =
        Vec::with_capacity(corrections.len());
    for c in corrections {
        // Defensive: ms-codec's `decode_with_correction` bounds-checks
        // before applying corrections (errors `TooManyErrors` otherwise),
        // so `c.position` is in-range. Mirror mk-cli's defensive pattern.
        if c.position < data_chars.len() {
            data_chars[c.position] = c.now;
        }
        corrected_positions.push((c.position, c.was, c.now));
    }

    let mut out = String::with_capacity(prefix.len() + 1 + data_chars.len());
    out.push_str(&prefix.to_ascii_lowercase());
    out.push('1');
    for c in &data_chars {
        out.push(*c);
    }

    (out, corrected_positions)
}

/// Text-form report: `# Repair report` header, per-chunk correction lines,
/// then corrected chunks one per line. Mirrors toolkit's
/// `cmd::repair::emit_repair_text` shape byte-exact (modulo the `ms1`-only
/// `kind_str`).
fn emit_text(corrected_chunks: &[String], reports: &[RepairDetail]) -> Result<()> {
    let any_correction = reports.iter().any(|r| !r.corrected_positions.is_empty());
    if any_correction {
        println!("# Repair report");
        for r in reports {
            if r.corrected_positions.is_empty() {
                continue;
            }
            let n = r.corrected_positions.len();
            let plural = if n == 1 { "correction" } else { "corrections" };
            let mut line = format!("#   ms1 chunk {}: {} {} at ", r.chunk_index, n, plural);
            for (i, (pos, was, now)) in r.corrected_positions.iter().enumerate() {
                if i > 0 {
                    line.push_str(", ");
                }
                line.push_str(&format!("position {pos}: '{was}' -> '{now}'"));
            }
            println!("{line}");
        }
    }
    for chunk in corrected_chunks {
        println!("{chunk}");
    }
    Ok(())
}

// JSON envelope — schema MUST byte-match toolkit's standalone `RepairJson`
// at `mnemonic-toolkit/src/cmd/repair.rs:162-183` (D27 cross-CLI parser
// reuse). Field order is part of the schema (serde preserves struct field
// order in the default JSON serializer).
#[derive(Serialize)]
struct RepairJson<'a> {
    schema_version: &'static str,
    kind: &'static str,
    corrected_chunks: &'a [String],
    repairs: Vec<RepairJsonDetail<'a>>,
}

#[derive(Serialize)]
struct RepairJsonDetail<'a> {
    chunk_index: usize,
    original_chunk: &'a str,
    corrected_chunk: &'a str,
    corrected_positions: Vec<RepairJsonPosition>,
}

#[derive(Serialize)]
struct RepairJsonPosition {
    position: usize,
    was: String,
    now: String,
}

fn emit_json(corrected_chunks: &[String], reports: &[RepairDetail]) -> Result<()> {
    let envelope = RepairJson {
        schema_version: "1",
        kind: "ms1",
        corrected_chunks,
        repairs: reports
            .iter()
            // Mirror toolkit: only include entries for chunks that
            // actually had corrections applied.
            .filter(|r| !r.corrected_positions.is_empty())
            .map(|r| RepairJsonDetail {
                chunk_index: r.chunk_index,
                original_chunk: &r.original_chunk,
                corrected_chunk: &r.corrected_chunk,
                corrected_positions: r
                    .corrected_positions
                    .iter()
                    .map(|(p, w, n)| RepairJsonPosition {
                        position: *p,
                        was: w.to_string(),
                        now: n.to_string(),
                    })
                    .collect(),
            })
            .collect(),
    };
    let body = serde_json::to_string(&envelope)
        .map_err(|e| CliError::BadInput(format!("repair JSON serialize: {e}")))?;
    println!("{body}");
    Ok(())
}
