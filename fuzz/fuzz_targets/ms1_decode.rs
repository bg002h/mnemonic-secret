//! Fuzz target: ms1 single-string decode surface.
//!
//! ms phase of the constellation stress-fuzz program (Cycle C). Drives the
//! whole-input string through THREE decode entry points on the same bytes:
//! `decode`, `decode_with_correction`, and `inspect`.
//!
//! Oracles:
//! 1. Never-panic / clean-error (implicit: any panic/abort = libFuzzer
//!    failure). `inspect` participates here only — call it and assert it does
//!    not panic; its report value is intentionally ignored (it is the lenient
//!    diagnostic surface, with no round-trip contract).
//! 2. Decode → re-encode fixed-point (R0 [Q5]/[I6]): on `decode` `Ok((tag, p))`,
//!    `encode(tag, &p)` then `decode` again and assert `(Tag, Payload)` equal
//!    (both `PartialEq`). A re-encode `Err` on a decode-accepted value is a
//!    REAL FINDING — the decode/encode-asymmetry class the charter targets —
//!    so it panics in-target rather than being swallowed.
//! 3. Apply-details idempotence (R0 [I1]) on `decode_with_correction`
//!    `Ok((tag, p, details))`: apply each `CorrectionDetail.now` at its
//!    `position` into the data-part, re-run `decode_with_correction`, and
//!    assert the decoded `(Tag, Payload)` is unchanged AND the new details
//!    vector is EMPTY (a corrected card needs no further correction).
//!
//! COORDINATE (ms-specific, R0 [I1] + round-2/round-3): ms `CorrectionDetail`
//! is `{position, was, now}` — NO `chunk_index` (single data-part). `position`
//! is the 0-indexed offset into the codex32 DATA-PART, i.e. the chars AFTER the
//! `ms1` HRP+separator (decode.rs:113-123 / :154 `parse_ms1_symbols` indexes
//! post-`ms1`). Applying `now` therefore offsets past the `ms1` prefix and
//! indexes the `position`-th data-part char directly (ms data-parts carry no
//! visual separators in the symbol count, unlike md). The apply is BOUNDS-SAFE:
//! details come from fuzzed input, so an out-of-range `position` (or a string
//! that no longer starts with `ms1`) early-returns rather than panics — a
//! false crash would otherwise be a harness bug, not a finding.
#![no_main]

use libfuzzer_sys::fuzz_target;
use ms_codec::{CorrectionDetail, decode, decode_with_correction, inspect};

/// HRP prefix every ms1 string begins with (case-insensitively).
const HRP_PREFIX: &str = "ms1";

/// Apply `detail.now` at the post-HRP data-part offset `detail.position` within
/// `s`. Returns `None` (skip the idempotence check for this input) when the
/// string does not start with `ms1` or the position is out of range — the
/// decoder lowercases before parsing, so we mirror that here. BOUNDS-SAFE: no
/// indexing can panic on arbitrary fuzzed input.
fn apply_correction(s: &str, detail: &CorrectionDetail) -> Option<String> {
    // `decode_with_correction` lowercases via `parse_ms1_symbols`; match it so
    // the HRP check and the position walk line up with how the detail was
    // reported.
    let lower = s.to_ascii_lowercase();
    if !lower.starts_with(HRP_PREFIX) {
        return None;
    }
    let data = &lower[HRP_PREFIX.len()..];

    // Find the BYTE offset of the `position`-th char of the data-part. ms1
    // data-parts count every char as one symbol (no visual-separator skipping),
    // so this is a direct char-index walk.
    let target_byte = data
        .char_indices()
        .nth(detail.position)
        .map(|(byte_off, _)| byte_off)?;
    let target_char = data[target_byte..].chars().next()?;

    let mut out = String::with_capacity(s.len());
    out.push_str(HRP_PREFIX);
    out.push_str(&data[..target_byte]);
    out.push(detail.now);
    out.push_str(&data[target_byte + target_char.len_utf8()..]);
    Some(out)
}

fuzz_target!(|data: &[u8]| {
    // ms1 strings are ASCII; U+FFFD collapse from lossy conversion just wastes
    // a sliver of input space (R0 [M7]).
    let s = String::from_utf8_lossy(data);

    // --- Oracle 2: decode → re-encode fixed-point. ---
    if let Ok((tag, payload)) = decode(&s) {
        let reencoded = ms_codec::encode(tag, &payload)
            .expect("FINDING: decode-accepted (tag, payload) failed to re-encode");
        let (tag2, payload2) =
            decode(&reencoded).expect("FINDING: re-encoded ms1 string failed to decode");
        assert_eq!(
            (tag, &payload),
            (tag2, &payload2),
            "FINDING: decode/re-encode/decode is not a fixed point"
        );
    }

    // --- Oracle 3: apply-details idempotence on decode_with_correction. ---
    if let Ok((tag, payload, details)) = decode_with_correction(&s) {
        if !details.is_empty() {
            // Apply every reported correction to the input string.
            let mut corrected = s.to_string();
            let mut applied_all = true;
            for detail in &details {
                match apply_correction(&corrected, detail) {
                    Some(fixed) => corrected = fixed,
                    // Coordinate not applicable against this string state
                    // (out-of-range on fuzzed input) — skip idempotence here;
                    // a false crash would be a harness bug, not a finding.
                    None => {
                        applied_all = false;
                        break;
                    }
                }
            }
            if applied_all {
                let (tag2, payload2, details2) = decode_with_correction(&corrected).expect(
                    "FINDING: applying reported corrections produced an undecodable string",
                );
                assert_eq!(
                    (tag, &payload),
                    (tag2, &payload2),
                    "FINDING: apply-details idempotence — corrected string decodes to a different (Tag, Payload)"
                );
                assert!(
                    details2.is_empty(),
                    "FINDING: apply-details idempotence — corrected string still reports corrections: {details2:?}"
                );
            }
        }
    }

    // --- Oracle 1: inspect must not panic (its report has no round-trip
    //     contract; the value is intentionally ignored). ---
    let _ = inspect(&s);
});
