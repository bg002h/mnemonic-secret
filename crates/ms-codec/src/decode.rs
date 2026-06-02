//! Public decoder. Applies SPEC §4 validity rules in order.
//!
//! v0.2.0: also hosts [`decode_with_correction`] — the BCH-error-correcting
//! decode entry point per plan §1 D22 + §2.B.2. Parse → polymod-residue →
//! (if non-zero) call [`crate::bch_decode::decode_regular_errors`] → apply
//! corrections → run the existing [`decode`] path → return
//! `(Tag, Payload, Vec<CorrectionDetail>)`. ms1 is single-chunk per codex32
//! spec, so there is no atomic-multi-chunk variant (cf. md-codec's
//! per-chunk-set version).

use crate::consts::{RESERVED_NOT_EMITTED_V01, TAG_ENTR, VALID_MNEM_STR_LENGTHS, VALID_STR_LENGTHS};
use crate::envelope;
use crate::error::{Error, Result};
use crate::payload::{Payload, PayloadKind};
use crate::tag::Tag;
use codex32::Codex32String;

/// Union of all emittable string lengths (entr ∪ mnem). Used as the
/// pre-dispatch gate in `decode` before kind-specific binding.
fn is_known_length(len: usize) -> bool {
    VALID_STR_LENGTHS.contains(&len) || VALID_MNEM_STR_LENGTHS.contains(&len)
}

/// Return the kind-appropriate allowed-length set for error reporting.
fn allowed_for_kind(kind: PayloadKind) -> &'static [usize] {
    match kind {
        PayloadKind::Entr => VALID_STR_LENGTHS,
        PayloadKind::Mnem => VALID_MNEM_STR_LENGTHS,
    }
}

/// Decode an ms1 string into `(Tag, Payload)`.
///
/// Rejects per SPEC §4 rules 1-10 (extended for v0.2 mnem):
///
/// - Rule 1: upstream codex32 parse failure (Codex32 variant).
/// - Rules 2-4, 8: wire-invariant violations (delegated to envelope::discriminate).
/// - Rules 5-7: tag-table membership rules (here).
/// - Rule 9: total string length not in the union {entr lengths} ∪ {mnem lengths}
///   (here, before parse); then bound to the discriminated kind post-dispatch.
/// - Rule 10: payload byte length mismatch for the tag (here, via Payload::validate()).
pub fn decode(s: &str) -> Result<(Tag, Payload)> {
    // §4 rule 9 (pre-dispatch): total string length must be in the union set.
    if !is_known_length(s.len()) {
        return Err(Error::UnexpectedStringLength {
            got: s.len(),
            allowed: VALID_STR_LENGTHS, // report the entr set as the primary allowed set
        });
    }

    // §4 rule 1: delegate parse + checksum to rust-codex32.
    let c = Codex32String::from_string(s.to_string())?;

    // §4 rules 2, 3, 4, 8 + tag-alphabet rule 5: envelope (returns typed Payload).
    let (tag, payload) = envelope::discriminate(&c)?;

    // §4 rule 9 (post-dispatch, bind to kind): length must be in the kind-appropriate set.
    let kind_allowed = allowed_for_kind(payload.kind());
    if !kind_allowed.contains(&s.len()) {
        return Err(Error::UnexpectedStringLength {
            got: s.len(),
            allowed: kind_allowed,
        });
    }

    // §4 rule 7: reserved-not-emitted tags.
    if RESERVED_NOT_EMITTED_V01.contains(tag.as_bytes()) {
        return Err(Error::ReservedTagNotEmittedInV01 {
            got: *tag.as_bytes(),
        });
    }

    // §4 rule 6: tag must be in the v0.2 accept set (currently {entr}).
    // SPEC v0.9.0 §1 item 2 — wrap the OWNED entropy buffer in `Zeroizing`
    // so the intermediate scrub runs on function exit. The public Payload
    // boundary is unwrapped per SPEC §3 OOS-2; caller wraps — see payload.rs.
    use zeroize::Zeroizing;
    let payload = match *tag.as_bytes() {
        x if x == TAG_ENTR => {
            match payload {
                Payload::Entr(data) => {
                    let scrubbed: Zeroizing<Vec<u8>> = Zeroizing::new(data);
                    let p = Payload::Entr((*scrubbed).clone());
                    // §4 rule 10: validate payload length.
                    p.validate()?;
                    p
                }
                Payload::Mnem { language, entropy } => {
                    let scrubbed: Zeroizing<Vec<u8>> = Zeroizing::new(entropy);
                    let p = Payload::Mnem { language, entropy: (*scrubbed).clone() };
                    // §4 rule 10: validate (language range + entropy length).
                    p.validate()?;
                    p
                }
            }
        }
        _ => {
            return Err(Error::UnknownTag {
                got: *tag.as_bytes(),
            });
        }
    };

    Ok((tag, payload))
}

// ---------------------------------------------------------------------------
// v0.2.0: BCH-error-correcting decode (plan §1 D22 + §2.B.2).
// ---------------------------------------------------------------------------

/// Per-correction report emitted by [`decode_with_correction`]. One entry
/// per repaired character. `position` is 0-indexed into the codex32
/// data-part (i.e. the characters following the `ms1` HRP + separator);
/// `was` is the original (corrupted) char from the input; `now` is the
/// corrected char.
///
/// ms1 is single-chunk per codex32 spec, so there is no `chunk_index`
/// field (cf. md-codec's `CorrectionDetail`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorrectionDetail {
    /// 0-indexed position of the corrected character within the codex32
    /// data-part (post-HRP-and-separator).
    pub position: usize,
    /// The original (corrupted) character at this position.
    pub was: char,
    /// The corrected character at this position.
    pub now: char,
}

/// Local codex32 alphabet (BIP 173 lowercase). Each char = one 5-bit
/// symbol. Mirrors md-codec's `chunk.rs` local copy — kept private here so
/// this module doesn't widen the codex32 public surface.
const CODEX32_ALPHABET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

/// BIP 173 HRP for ms1 strings (HRP + separator).
const HRP_PREFIX: &str = "ms1";

/// Parse an ms1 string into its 5-bit data-part symbol vector. Returns
/// the data-with-checksum symbols (i.e. all symbols after `ms1`). The
/// returned symbol count includes the 13-symbol BCH checksum tail.
///
/// Returns [`Error::WrongHrp`] if the string does not start with `ms1`,
/// or [`Error::Codex32`] (via a `codex32::Error::InvalidChar`) if any
/// data-part character is not in the codex32 alphabet.
fn parse_ms1_symbols(s: &str) -> Result<Vec<u8>> {
    let lower = s.to_ascii_lowercase();
    if !lower.starts_with(HRP_PREFIX) {
        // Find the actual HRP (everything up to and including the last '1'
        // separator) so the error reports the observed HRP instead of "".
        let hrp_end = lower.rfind('1').map(|i| i + 1).unwrap_or(lower.len());
        let got = lower[..hrp_end.saturating_sub(1)].to_string();
        return Err(Error::WrongHrp { got });
    }
    let rest = &lower[HRP_PREFIX.len()..];
    let mut symbols: Vec<u8> = Vec::with_capacity(rest.len());
    // Non-alphabet characters can't appear in a valid v0.1 string. We
    // can't fabricate a `codex32::Error` value here (the upstream crate
    // doesn't expose a constructor for `InvalidChar`), so we use
    // `UnexpectedStringLength` as a stand-in: the existing `decode` path
    // would have rejected the string for the same reason on a different
    // axis. Toolkit-side helper at B.7 absorbs into `UnparseableInput`
    // per plan §2.B.4 D29 error-mapping table.
    for c in rest.chars() {
        let lc = c as u8;
        let sym = CODEX32_ALPHABET
            .iter()
            .position(|&b| b == lc)
            .ok_or(Error::UnexpectedStringLength {
                got: s.len(),
                allowed: VALID_STR_LENGTHS,
            })? as u8;
        symbols.push(sym);
    }
    Ok(symbols)
}

/// Re-encode a 5-bit data-part symbol vector as a complete ms1 string.
fn encode_ms1_string(data_with_checksum: &[u8]) -> String {
    let mut out = String::with_capacity(HRP_PREFIX.len() + data_with_checksum.len());
    out.push_str(HRP_PREFIX);
    for &v in data_with_checksum {
        out.push(CODEX32_ALPHABET[(v & 0x1F) as usize] as char);
    }
    out
}

/// BCH-error-correcting decode for a single ms1 string.
///
/// Per plan §1 Q1 lock — full-decode semantics: this is the single entry
/// point that callers needing both "did anything get repaired?" AND "the
/// fully-decoded `(Tag, Payload)`" should use.
///
/// Algorithm:
/// 1. Parse the input as ms1 (`ms1` HRP + codex32 data-part) into a
///    5-bit symbol vector.
/// 2. Compute the BCH polymod residue
///    (`hrp_expand("ms") || data_with_checksum`) XOR'd against
///    [`crate::bch::MS_REGULAR_CONST`].
/// 3. Residue `== 0` ⇒ clean codeword; pass through to the existing
///    [`decode`] entry point unchanged.
/// 4. Residue `!= 0` ⇒ invoke
///    [`crate::bch_decode::decode_regular_errors`]. If `None`, return
///    `Err(Error::TooManyErrors { bound: 8 })` per plan §2.B.4 D29
///    error-mapping table.
/// 5. Apply corrections to the symbol vector, re-verify via polymod (a
///    defensive catch for pathological 5+-error patterns that fool BM
///    into returning a degree-≤4 locator with 4 valid roots), and record
///    one [`CorrectionDetail`] per repaired character.
/// 6. Re-encode the corrected symbol vector as an ms1 string and forward
///    it to the existing [`decode`] entry point.
///
/// Per Q1 lock + D29 error-mapping table, any §4-rule error from the
/// full decode (orphan variants like `ThresholdNotZero`,
/// `ReservedTagNotEmittedInV01`, etc.) surfaces directly; toolkit-side
/// `repair_via_ms_codec` (B.7) absorbs these into
/// `RepairError::PostCorrectionDecodeFailed`.
///
/// Returns `(Tag, Payload, Vec<CorrectionDetail>)` on success. The
/// correction-detail vector is in ascending `position` order; an empty
/// vector means the input was already a valid codeword.
pub fn decode_with_correction(s: &str) -> Result<(Tag, Payload, Vec<CorrectionDetail>)> {
    // Parse data-part symbols. Length checks live in `decode` proper
    // (rule 9 is enforced there after we've potentially corrected, since
    // BCH correction does not change the string length).
    let symbols = parse_ms1_symbols(s)?;

    // Polymod residue against ms1's target constant.
    let mut input = crate::bch::hrp_expand("ms");
    input.extend_from_slice(&symbols);
    let residue = crate::bch::polymod_run(&input) ^ crate::bch::MS_REGULAR_CONST;

    if residue == 0 {
        // Already a valid codeword; pass through to the existing decoder.
        let (tag, payload) = decode(s)?;
        return Ok((tag, payload, Vec::new()));
    }

    // Attempt BCH correction.
    let (positions, magnitudes) = crate::bch_decode::decode_regular_errors(residue, symbols.len())
        .ok_or(Error::TooManyErrors { bound: 8 })?;

    // Apply corrections; record (was, now) chars per position.
    let mut corrected = symbols.clone();
    let mut details: Vec<CorrectionDetail> = Vec::with_capacity(positions.len());
    for (&pos, &mag) in positions.iter().zip(&magnitudes) {
        if pos >= corrected.len() {
            // Defensive: chien_search bounded pos to [0, L); but a
            // pathological 5+-error pattern could in principle skirt
            // that.
            return Err(Error::TooManyErrors { bound: 8 });
        }
        let was_byte = corrected[pos];
        let now_byte = was_byte ^ mag;
        let was = CODEX32_ALPHABET[(was_byte & 0x1F) as usize] as char;
        let now = CODEX32_ALPHABET[(now_byte & 0x1F) as usize] as char;
        details.push(CorrectionDetail {
            position: pos,
            was,
            now,
        });
        corrected[pos] = now_byte;
    }

    // Defensive re-verify (catches pathological 5+-error patterns that
    // happen to produce a degree-≤4 locator with 4 valid roots).
    let mut verify_input = crate::bch::hrp_expand("ms");
    verify_input.extend_from_slice(&corrected);
    let verify_residue =
        crate::bch::polymod_run(&verify_input) ^ crate::bch::MS_REGULAR_CONST;
    if verify_residue != 0 {
        return Err(Error::TooManyErrors { bound: 8 });
    }

    // Hand the corrected string to the existing decoder. Any §4-rule
    // error surfaces directly per Q1 lock; toolkit helper at B.7 absorbs.
    let corrected_str = encode_ms1_string(&corrected);
    let (tag, payload) = decode(&corrected_str)?;
    Ok((tag, payload, details))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode;

    #[test]
    fn round_trip_entr_all_lengths() {
        for len in [16usize, 20, 24, 28, 32] {
            let entropy = (0..len as u8)
                .map(|i| i.wrapping_mul(7))
                .collect::<Vec<_>>();
            let p = Payload::Entr(entropy.clone());
            let s = encode::encode(Tag::ENTR, &p).unwrap();
            let (tag, recovered) = decode(&s).unwrap();
            assert_eq!(tag, Tag::ENTR);
            assert_eq!(recovered, p);
        }
    }

    #[test]
    fn decode_rejects_unexpected_length() {
        // 52 chars is outside both the entr set [50,56,62,69,75]
        // and the mnem set [51,58,64,70,77].
        let s = "ms10entrsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
        assert_eq!(s.len(), 52, "test string must be 52 chars");
        assert!(matches!(
            decode(s),
            Err(Error::UnexpectedStringLength { .. })
        ));
    }

    #[test]
    fn decode_rejects_short_seed_string_with_reserved_tag() {
        // Hand-build a 50-char string with id="seed" — 16-B entropy worth.
        // The string-length check passes; tag-rule 7 fails.
        let mut data = vec![0x00u8];
        data.extend_from_slice(&[0xAAu8; 16]);
        let c = Codex32String::from_seed("ms", 0, "seed", codex32::Fe::S, &data).unwrap();
        let s = c.to_string();
        assert_eq!(s.len(), 50, "expected str.len 50 for 16-B + prefix");
        assert!(matches!(
            decode(&s),
            Err(Error::ReservedTagNotEmittedInV01 { .. })
        ));
    }
}
