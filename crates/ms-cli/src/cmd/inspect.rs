//! `ms inspect` — structural validity report for an ms1 string.
//!
//! Realizes SPEC §2.3 (verdict-first + structured fields), §2.3.1 (inspect()
//! BIP-93 parse failure handled per §6 standard error path), §5.3 (--json
//! schema), audit C3/I5 (would_decode + failure_reasons).

use clap::Args;
use ms_codec::consts::{
    MNEM_LANGUAGE_NAMES, RESERVED_NOT_EMITTED_V01, TAG_ENTR, VALID_ENTR_LENGTHS,
    VALID_MNEM_STR_LENGTHS, VALID_STR_LENGTHS,
};
use ms_codec::{InspectKind, InspectReport};
use serde_json::to_string;
use zeroize::Zeroizing;

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
    // cycle-15 Lane M (slug #5): the ms1 intake IS secret material (BIP-39
    // entropy) — scrub it on drop. `inspect()` borrows `&str` (Deref).
    let ms1: Zeroizing<String> = Zeroizing::new(read_input(args.ms1.as_deref())?);
    let report = ms_codec::inspect(&ms1)?; // §2.3.1: failures return CliError::Codex32 here.

    // A threshold ∈ 2..=9 string is one share of a K-of-N share-set — a
    // first-class read, NOT a malformed v0.1 single-string. Report it as such
    // (kind: share, threshold/id/index) and SKIP the v0.1 rule-walk: a
    // distributed share's data()[0] is an interpolated value, not a payload
    // prefix, so prefix_byte / payload_bytes / the entr/mnem kind are garbage.
    if is_share(&report) {
        if args.json {
            emit_share_json(&report)?;
        } else {
            emit_share_text(&report);
        }
        return Ok(0);
    }

    let (would_decode, reasons) = analyze(&report, ms1.len());

    if args.json {
        emit_json(&report, would_decode, &reasons)?;
    } else {
        emit_text(&report, would_decode, &reasons);
    }
    Ok(0)
}

/// True iff the inspected string is one share of a K-of-N set: the codex32
/// threshold field is a share threshold (`2..=9`). Threshold `0` is the v0.1
/// single-string; `1` is invalid-per-codex32 (never constructible).
fn is_share(report: &InspectReport) -> bool {
    (2..=9).contains(&report.threshold)
}

/// Text report for a lone K-of-N share. Reports kind/threshold/id/index +
/// "would combine (needs k)". Suppresses prefix_byte / payload_bytes / the
/// entr/mnem kind — a distributed share's data()[0] is interpolated, not a
/// payload-kind prefix.
fn emit_share_text(report: &InspectReport) {
    println!(
        "OK: K-of-N share (would combine: needs {} shares)",
        report.threshold
    );
    println!();
    println!("hrp: {}", report.hrp);
    println!("threshold: {}", report.threshold);
    println!(
        "id: {}",
        std::str::from_utf8(report.tag.as_bytes()).unwrap_or("<non-utf8>")
    );
    println!("index: {}", report.share_index);
    println!("checksum_valid: {}", report.checksum_valid);
    println!("kind: share");
}

/// JSON report for a lone K-of-N share. `would_decode: true` carries the
/// "would combine" semantics (a share is a valid read). The garbage
/// prefix_byte / payload_bytes_hex / entr-mnem kind are omitted.
fn emit_share_json(report: &InspectReport) -> Result<()> {
    let json = serde_json::json!({
        "schema_version": "1",
        "report": {
            "hrp": report.hrp,
            "threshold": report.threshold,
            "tag": std::str::from_utf8(report.tag.as_bytes()).unwrap_or("<non-utf8>"),
            "share_index": report.share_index.to_string(),
            "checksum_valid": report.checksum_valid,
            "kind": "share",
        },
        "would_decode": true,
        "would_combine": true,
        "failure_reasons": Vec::<&str>::new(),
    });
    println!(
        "{}",
        serde_json::to_string(&json).expect("inspect share json serializes")
    );
    Ok(())
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
    // Rule 8: prefix byte must be a recognised kind (0x00 = entr, 0x02 = mnem).
    // Only flag non-zero-prefix if the kind is Unknown (not a recognised v0.2 type).
    if report.kind == InspectKind::Unknown {
        reasons.push("non-zero-prefix");
    }
    // Rule 9: total string length must be in the valid set for the detected kind.
    let valid_lengths: &[usize] = match report.kind {
        InspectKind::Mnem => VALID_MNEM_STR_LENGTHS,
        _ => VALID_STR_LENGTHS,
    };
    if !valid_lengths.contains(&str_len) {
        reasons.push("unexpected-string-length");
    }
    // Rule 10: payload length matches the expected set for the detected kind.
    // - entr: payload = entropy bytes ∈ {16,20,24,28,32}
    // - mnem: payload = [lang_byte][entropy] = entropy_len + 1 ∈ {17,21,25,29,33}
    match report.kind {
        InspectKind::Entr if tag_bytes == TAG_ENTR => {
            if !VALID_ENTR_LENGTHS.contains(&report.payload_bytes.len()) {
                reasons.push("payload-length-mismatch");
            }
        }
        InspectKind::Mnem => {
            // payload_bytes = [lang_byte, entropy...]; valid if len - 1 ∈ VALID_ENTR_LENGTHS.
            let entropy_len = report.payload_bytes.len().saturating_sub(1);
            if !VALID_ENTR_LENGTHS.contains(&entropy_len) {
                reasons.push("payload-length-mismatch");
            }
        }
        _ => {}
    }

    (reasons.is_empty(), reasons)
}

fn reason_text(tag: &'static str) -> &'static str {
    match tag {
        "unexpected-string-length" => {
            "string length not in valid set for this kind ([50,56,62,69,75] entr / [51,58,64,70,77] mnem)"
        }
        "wrong-hrp" => "HRP is not \"ms\"",
        "threshold-not-zero" => "threshold not 0 (v0.1 is single-string only)",
        "share-index-not-secret" => "share-index not 's' (BIP-93 requires 's' for threshold=0)",
        "reserved-tag-not-emitted" => "tag is reserved-not-emitted in v0.1; deferred to v0.2+",
        "unknown-tag" => "tag not in v0.1 RESERVED_TAG_TABLE",
        "non-zero-prefix" => "prefix byte is not a recognised kind (0x00=entr, 0x02=mnem)",
        "payload-length-mismatch" => {
            "payload length not valid for kind (entr: [16,20,24,28,32] B; mnem: [17,21,25,29,33] B)"
        }
        _ => "<unknown reason>",
    }
}

fn emit_text(report: &InspectReport, would_decode: bool, reasons: &[&'static str]) {
    if would_decode {
        let version = match report.kind {
            InspectKind::Mnem => "v0.2",
            _ => "v0.1",
        };
        println!("OK: would decode {}", version);
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
    println!("kind: {}", report.kind.as_str());
    if let Some(lang_code) = report.language {
        let name = MNEM_LANGUAGE_NAMES
            .get(lang_code as usize)
            .copied()
            .unwrap_or("unknown");
        println!("language: {}", name);
    }
}

fn emit_json(report: &InspectReport, would_decode: bool, reasons: &[&'static str]) -> Result<()> {
    let language_name: Option<String> = report.language.map(|code| {
        MNEM_LANGUAGE_NAMES
            .get(code as usize)
            .copied()
            .unwrap_or("unknown")
            .to_string()
    });
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
            kind: report.kind.as_str().to_string(),
            language: language_name,
        },
        would_decode,
        failure_reasons: reasons.to_vec(),
    };
    // cycle-15 Lane M (slug #8, defense-in-depth): the serialized JSON carries
    // `payload_bytes_hex` (the entropy) — scrub the buffer on drop.
    let s: Zeroizing<String> =
        Zeroizing::new(to_string(&json).expect("inspect json always serializes"));
    println!("{}", *s);
    Ok(())
}
