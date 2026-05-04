//! Public decoder. Applies SPEC §4 validity rules in order.

use crate::consts::{RESERVED_NOT_EMITTED_V01, TAG_ENTR, VALID_STR_LENGTHS};
use crate::envelope;
use crate::error::{Error, Result};
use crate::payload::Payload;
use crate::tag::Tag;
use codex32::Codex32String;

/// Decode a v0.1 ms1 string into `(Tag, Payload)`.
///
/// Rejects per SPEC §4 rules 1-10:
///
/// - Rule 1: upstream codex32 parse failure (Codex32 variant).
/// - Rules 2-4, 8: wire-invariant violations (delegated to envelope::discriminate).
/// - Rules 5-7: tag-table membership rules (here).
/// - Rule 9: total string length not in v0.1-emittable set (here, before parse).
/// - Rule 10: payload byte length mismatch for the tag (here, via Payload::validate()).
pub fn decode(s: &str) -> Result<(Tag, Payload)> {
    // §4 rule 9: total string length must be in the v0.1 set.
    if !VALID_STR_LENGTHS.contains(&s.len()) {
        return Err(Error::UnexpectedStringLength {
            got: s.len(),
            allowed: VALID_STR_LENGTHS,
        });
    }

    // §4 rule 1: delegate parse + checksum to rust-codex32. `?` leverages the
    // From<codex32::Error> for Error impl in error.rs.
    let c = Codex32String::from_string(s.to_string())?;

    // §4 rules 2, 3, 4, 8 + tag-alphabet rule 5: envelope.
    let (tag, payload_bytes) = envelope::discriminate(&c)?;

    // §4 rule 7: reserved-not-emitted tags.
    if RESERVED_NOT_EMITTED_V01.contains(tag.as_bytes()) {
        return Err(Error::ReservedTagNotEmittedInV01 {
            got: *tag.as_bytes(),
        });
    }

    // §4 rule 6: tag must be in the v0.1 accept set (currently {entr}).
    let payload = match *tag.as_bytes() {
        x if x == TAG_ENTR => {
            let p = Payload::Entr(payload_bytes);
            // §4 rule 10: validate payload length against the tag's expected set.
            p.validate()?;
            p
        }
        _ => {
            return Err(Error::UnknownTag {
                got: *tag.as_bytes(),
            });
        }
    };

    Ok((tag, payload))
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
        // 51 chars is not a v0.1 emittable length.
        let s = "ms10entrsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
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
