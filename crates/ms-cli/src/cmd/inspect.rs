//! `ms inspect` — structural validity report for an ms1 string.
//!
//! Realizes SPEC §2.3 (verdict-first + structured fields), §2.3.1 (inspect()
//! BIP-93 parse failure handled per §6 standard error path), §5.3 (--json
//! schema), audit C3/I5 (would_decode + failure_reasons).

use clap::Args;
use ms_codec::consts::{RESERVED_NOT_EMITTED_V01, TAG_ENTR, VALID_ENTR_LENGTHS, VALID_STR_LENGTHS};
use ms_codec::InspectReport;
use serde_json::to_string;

use crate::error::Result;
use crate::format::{InspectJson, InspectReportJson};
use crate::parse::read_input;

/// `ms inspect` arguments.
#[derive(Args, Debug)]
pub struct InspectArgs {
    /// ms1 string to inspect. Use `-` or omit to read from stdin.
    pub ms1: Option<String>,

    /// Emit JSON instead of text verdict + fields.
    #[arg(long)]
    pub json: bool,
}

/// Run `ms inspect`. Lenient: returns a report even when the string would fail
/// decoder rules. If BIP-93 parse itself fails, treats the error per §6.
pub fn run(args: InspectArgs) -> Result<u8> {
    let ms1 = read_input(args.ms1.as_deref())?;
    let report = ms_codec::inspect(&ms1)?; // §2.3.1: failures return CliError::Codex32 here.

    let (would_decode, reasons) = analyze(&report, ms1.len());

    if args.json {
        emit_json(&report, would_decode, &reasons)?;
    } else {
        emit_text(&report, would_decode, &reasons);
    }
    Ok(0)
}

/// Re-walk SPEC §4 rules against the InspectReport's fields.
/// Returns `(would_decode, failure_reasons)` where reasons are pushed in
/// ASCENDING SPEC §4 rule order: 2, 3, 4, 6/7, 8, 9, 10.
fn analyze(report: &InspectReport, str_len: usize) -> (bool, Vec<&'static str>) {
    let mut reasons: Vec<&'static str> = Vec::new();
    let tag_bytes = *report.tag.as_bytes();

    // Rule 2: HRP == "ms".
    if report.hrp != "ms" {
        reasons.push("wrong-hrp");
    }
    // Rule 3: threshold == 0.
    if report.threshold != 0 {
        reasons.push("threshold-not-zero");
    }
    // Rule 4: share-index == 's'.
    if report.share_index != 's' {
        reasons.push("share-index-not-secret");
    }
    // Rules 6 + 7 are mutually exclusive (per `RESERVED_NOT_EMITTED_V01` vs `TAG_ENTR`).
    // Push rule 6 BEFORE rule 7 in ascending order if applicable; in our v0.1
    // shape only one of {entr accept-set, reserved-not-emitted, unknown}
    // applies, so at most one of these two reasons fires.
    if tag_bytes != TAG_ENTR {
        if RESERVED_NOT_EMITTED_V01.contains(&tag_bytes) {
            // Rule 7: tag is reserved-not-emitted in v0.1.
            // (Pushed after rule 6 logically — but only one of {6, 7} fires
            // because RESERVED_NOT_EMITTED_V01 ∩ accept-set = ∅, and a tag
            // either IS reserved or it's unknown.)
            reasons.push("reserved-tag-not-emitted");
        } else {
            // Rule 6: tag not in accept set.
            reasons.push("unknown-tag");
        }
    }
    // Rule 8: prefix byte == 0x00.
    if report.prefix_byte != 0x00 {
        reasons.push("non-zero-prefix");
    }
    // Rule 9: total string length in v0.1 set.
    if !VALID_STR_LENGTHS.contains(&str_len) {
        reasons.push("unexpected-string-length");
    }
    // Rule 10: payload length matches tag's expected set (only entr in v0.1).
    if tag_bytes == TAG_ENTR && !VALID_ENTR_LENGTHS.contains(&report.payload_bytes.len()) {
        reasons.push("payload-length-mismatch");
    }

    (reasons.is_empty(), reasons)
}

fn reason_text(tag: &'static str) -> &'static str {
    match tag {
        "unexpected-string-length" => "string length not in v0.1 set [50, 56, 62, 69, 75]",
        "wrong-hrp" => "HRP is not \"ms\"",
        "threshold-not-zero" => "threshold not 0 (v0.1 is single-string only)",
        "share-index-not-secret" => "share-index not 's' (BIP-93 requires 's' for threshold=0)",
        "reserved-tag-not-emitted" => "tag is reserved-not-emitted in v0.1; deferred to v0.2+",
        "unknown-tag" => "tag not in v0.1 RESERVED_TAG_TABLE",
        "non-zero-prefix" => "reserved-prefix byte is not 0x00 (v0.1 reserves it)",
        "payload-length-mismatch" => "entr payload length not in [16, 20, 24, 28, 32] bytes",
        _ => "<unknown reason>",
    }
}

fn emit_text(report: &InspectReport, would_decode: bool, reasons: &[&'static str]) {
    if would_decode {
        println!("OK: would decode v0.1");
    } else {
        println!("FAIL: would NOT decode v0.1");
        for r in reasons {
            println!("    reason: {} ({})", r, reason_text(r));
        }
    }
    println!();
    println!("hrp: {}", report.hrp);
    println!("threshold: {}", report.threshold);
    println!(
        "tag: {}",
        std::str::from_utf8(report.tag.as_bytes()).unwrap_or("<non-utf8>")
    );
    println!("share_index: {}", report.share_index);
    println!("prefix_byte: 0x{:02x}", report.prefix_byte);
    println!("payload_bytes: {}", hex::encode(&report.payload_bytes));
    println!("checksum_valid: {}", report.checksum_valid);
}

fn emit_json(report: &InspectReport, would_decode: bool, reasons: &[&'static str]) -> Result<()> {
    let json = InspectJson {
        schema_version: "1",
        report: InspectReportJson {
            hrp: report.hrp.clone(),
            threshold: report.threshold,
            tag: std::str::from_utf8(report.tag.as_bytes())
                .unwrap_or("<non-utf8>")
                .to_string(),
            share_index: report.share_index,
            prefix_byte: report.prefix_byte,
            payload_bytes_hex: hex::encode(&report.payload_bytes),
            checksum_valid: report.checksum_valid,
        },
        would_decode,
        failure_reasons: reasons.to_vec(),
    };
    let s = to_string(&json).expect("inspect json always serializes");
    println!("{}", s);
    Ok(())
}
