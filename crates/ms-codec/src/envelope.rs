//! THE v0.2-MIGRATION SEAM. This is the only module that contacts `rust-codex32`.
//!
//! Why isolated: SPEC §2.2 + §10. When K-of-N share encoding ships in v0.2, only
//! this module changes — `discriminate()` adds prefix-byte dispatch, `package()`
//! gains the `Threshold` parameter. The rest of the crate is untouched.
//!
//! Why wire-position re-parse: `rust-codex32 v0.1.0`'s `Parts` struct (verified
//! at `src/lib.rs:383-392` of the upstream crate) has non-`pub` fields; only
//! `Parts::data() -> Vec<u8>` is publicly accessible. We cannot read
//! `parts.hrp` / `parts.threshold` / `parts.id` / `parts.share_index` from
//! outside the upstream crate. The re-parse below replays what
//! `rust-codex32`'s own `parts_inner` does internally (it's a fast O(n) string
//! parse on a string already proven valid by `Codex32String::from_string`).
//! Re-parse cost is negligible — the upstream `Parts<'s>` is `Copy`.
//!
//! Wire positions (relative to the `1` separator at index `sep`):
//!
//! ```text
//! threshold:   sep + 1                  (1 char; v0.1 = '0')
//! id:          sep + 2 .. sep + 6       (4 chars; type tag in v0.1)
//! share-index: sep + 6                  (1 char; v0.1 = 's')
//! payload:     sep + 7 .. s.len() - 13  (variable; -13 strips short cksum)
//! checksum:    s.len() - 13 .. s.len()  (13 chars; short only in v0.1)
//! ```
//!
//! For v0.1 we never see long-checksum strings (rejected by SPEC §4 rule 9
//! before this module is reached); `CHECKSUM_LEN_SHORT = 13` is hard-coded.

use crate::consts::{
    CHECKSUM_LEN_SHORT, HRP, RESERVED_PREFIX, SEPARATOR, SHARE_INDEX_V01, THRESHOLD_V01,
};
use crate::error::{Error, Result};
use crate::tag::Tag;
use codex32::{Codex32String, Fe};

/// Wire-position offsets relative to the separator index.
const THRESHOLD_OFFSET: usize = 1;
const ID_START_OFFSET: usize = 2;
const ID_END_OFFSET: usize = 6;
const SHARE_INDEX_OFFSET: usize = 6;
const PAYLOAD_START_OFFSET: usize = 7;

/// Wire fields extracted from a BIP-93-validated ms1 string.
#[derive(Debug, Clone, Copy)]
pub(crate) struct WireFields<'s> {
    pub hrp: &'s str,
    pub threshold_byte: u8,
    pub id_bytes: [u8; 4],
    pub share_index_byte: u8,
}

/// Re-parse a string already validated by `Codex32String::from_string` to
/// extract wire-position fields. Caller MUST pass only strings that successfully
/// round-tripped through `rust-codex32` parsing.
///
/// Returns `Err(Error::UnexpectedStringLength)` if the string is too short to
/// contain the fixed wire prefix (defensive only; unreachable for inputs that
/// passed BIP-93 parsing).
pub(crate) fn extract_wire_fields(s: &str) -> Result<WireFields<'_>> {
    let sep = s
        .rfind(SEPARATOR)
        .ok_or_else(|| Error::WrongHrp { got: s.to_string() })?;
    // The fixed wire prefix after the separator is 7 chars (threshold + 4-char
    // id + share-index) + 13-char short checksum = 20. Any v0.1-shaped string
    // therefore needs at least sep + 20 bytes.
    if s.len() < sep + PAYLOAD_START_OFFSET + CHECKSUM_LEN_SHORT {
        return Err(Error::UnexpectedStringLength {
            got: s.len(),
            allowed: crate::consts::VALID_STR_LENGTHS,
        });
    }
    let bytes = s.as_bytes();
    let id_slice = &bytes[sep + ID_START_OFFSET..sep + ID_END_OFFSET];
    Ok(WireFields {
        hrp: &s[..sep],
        threshold_byte: bytes[sep + THRESHOLD_OFFSET],
        id_bytes: [id_slice[0], id_slice[1], id_slice[2], id_slice[3]],
        share_index_byte: bytes[sep + SHARE_INDEX_OFFSET],
    })
}

/// Decode-side v0.2-migration seam. Given a BIP-93-validated codex32 string,
/// extract `(Tag, payload_bytes_without_prefix)`. Enforces v0.1 wire-format
/// invariants: HRP="ms", threshold='0', share-index='s', prefix byte == 0x00.
/// Tag/payload-length validation against RESERVED_TAG_TABLE happens in `decode.rs`.
///
/// In v0.2 this function gains prefix-byte dispatch (`0x00` → v0.1 entr fallback,
/// `0x01` → v0.2 entr-share path, `0x02..` → kind-specific dispatch) per SPEC §5
/// invariant #2.
pub(crate) fn discriminate(c: &Codex32String) -> Result<(Tag, Vec<u8>)> {
    let s = c.to_string();
    let fields = extract_wire_fields(&s)?;

    // Wire-invariant checks (SPEC §4 rules 2, 3, 4).
    if fields.hrp != HRP {
        return Err(Error::WrongHrp {
            got: fields.hrp.to_string(),
        });
    }
    if fields.threshold_byte != THRESHOLD_V01 {
        return Err(Error::ThresholdNotZero {
            got: fields.threshold_byte,
        });
    }
    if fields.share_index_byte != SHARE_INDEX_V01 {
        return Err(Error::ShareIndexNotSecret {
            got: fields.share_index_byte as char,
        });
    }

    // Tag construction (SPEC §4 rule 5; rule 6/7 happen later in decode.rs).
    let tag_bytes = fields.id_bytes;
    let tag_str = std::str::from_utf8(&tag_bytes)
        .map_err(|_| Error::TagInvalidAlphabet { got: tag_bytes })?;
    let tag = Tag::try_new(tag_str)?;

    // Payload extraction via the upstream Parts::data(). For any string that
    // passed `extract_wire_fields` (s.len >= sep + 7 + 13 = at least 22 chars)
    // and `Codex32String::from_string` (s.len >= 48 for short codex32), the
    // payload is at least 26 codex32 symbols ≈ 16 raw bytes, so it cannot be
    // empty. No defensive `is_empty` arm needed.
    let payload_with_prefix = c.parts().data();

    // Reserved-prefix-byte check (SPEC §4 rule 8).
    if payload_with_prefix[0] != RESERVED_PREFIX {
        return Err(Error::ReservedPrefixViolation {
            got: payload_with_prefix[0],
        });
    }

    Ok((tag, payload_with_prefix[1..].to_vec()))
}

/// Encode-side v0.2-migration seam. Given `(tag, payload_bytes)`, build a
/// BIP-93-validated codex32 string with the v0.1 prefix-byte and wire-field
/// fixed values (threshold=0, share-index='s'). The payload bytes here are
/// the raw secret WITHOUT the reserved-prefix byte; this function prepends 0x00.
///
/// In v0.2 this function gains a `Threshold` parameter (per SPEC §5 invariant #4)
/// and the prefix byte becomes the type discriminator.
pub(crate) fn package(tag: Tag, payload_bytes: &[u8]) -> Result<Codex32String> {
    // [0x00 reserved-prefix] || payload
    let mut data = Vec::with_capacity(1 + payload_bytes.len());
    data.push(RESERVED_PREFIX);
    data.extend_from_slice(payload_bytes);

    // Delegate to rust-codex32. v0.1 always uses threshold=0, share=Fe::S.
    // `?` leverages the From<codex32::Error> for Error impl in error.rs.
    Ok(Codex32String::from_seed(
        HRP,
        0,
        tag.as_str(),
        Fe::S,
        &data,
    )?)
}

#[cfg(test)]
mod tests_extract {
    use super::*;

    #[test]
    fn bip93_test_vector_1_extracts_correctly() {
        // From rust-codex32 src/lib.rs bip_vector_1 test (BIP-93 vector 1):
        // hrp="ms", threshold=0, id="test", share_index='s', payload=26 'x' chars.
        let s = "ms10testsxxxxxxxxxxxxxxxxxxxxxxxxxx4nzvca9cmczlw";
        let fields = extract_wire_fields(s).unwrap();
        assert_eq!(fields.hrp, "ms");
        assert_eq!(fields.threshold_byte, b'0');
        assert_eq!(&fields.id_bytes, b"test");
        assert_eq!(fields.share_index_byte, b's');
    }

    #[test]
    fn rejects_too_short_string() {
        // "ms1" alone is below the minimum.
        assert!(matches!(
            extract_wire_fields("ms1"),
            Err(Error::UnexpectedStringLength { .. })
        ));
    }
}

#[cfg(test)]
mod tests_discriminate {
    use super::*;

    fn build_v01_entr(entropy: &[u8]) -> Codex32String {
        let mut data = vec![RESERVED_PREFIX];
        data.extend_from_slice(entropy);
        Codex32String::from_seed(HRP, 0, "entr", Fe::S, &data).unwrap()
    }

    #[test]
    fn v01_entr_16_round_trips_through_discriminate() {
        let entropy = vec![0xAAu8; 16];
        let c = build_v01_entr(&entropy);
        let (tag, recovered) = discriminate(&c).unwrap();
        assert_eq!(tag, Tag::ENTR);
        assert_eq!(recovered, entropy);
    }

    #[test]
    fn v01_entr_32_round_trips_through_discriminate() {
        let entropy = vec![0x55u8; 32];
        let c = build_v01_entr(&entropy);
        let (tag, recovered) = discriminate(&c).unwrap();
        assert_eq!(tag, Tag::ENTR);
        assert_eq!(recovered, entropy);
    }

    #[test]
    fn discriminate_rejects_non_zero_prefix() {
        let mut data = vec![0x01u8];
        data.extend_from_slice(&[0xAAu8; 16]);
        let c = Codex32String::from_seed(HRP, 0, "entr", Fe::S, &data).unwrap();
        assert!(matches!(
            discriminate(&c),
            Err(Error::ReservedPrefixViolation { got: 0x01 })
        ));
    }

    #[test]
    fn discriminate_rejects_wrong_hrp() {
        let mut data = vec![RESERVED_PREFIX];
        data.extend_from_slice(&[0xAAu8; 16]);
        let c = Codex32String::from_seed("mq", 0, "entr", Fe::S, &data).unwrap();
        assert!(matches!(discriminate(&c), Err(Error::WrongHrp { .. })));
    }
}

#[cfg(test)]
mod tests_package {
    use super::*;

    #[test]
    fn package_round_trips_through_discriminate() {
        for len in [16usize, 20, 24, 28, 32] {
            let entropy = vec![0xAAu8; len];
            let c = package(Tag::ENTR, &entropy).unwrap();
            let (tag, recovered) = discriminate(&c).unwrap();
            assert_eq!(tag, Tag::ENTR);
            assert_eq!(recovered, entropy);
        }
    }

    #[test]
    fn package_produces_str_lengths_in_v01_set() {
        let expected_lengths = crate::consts::VALID_STR_LENGTHS;
        for (i, len) in [16usize, 20, 24, 28, 32].iter().enumerate() {
            let entropy = vec![0xAAu8; *len];
            let c = package(Tag::ENTR, &entropy).unwrap();
            let s = c.to_string();
            assert_eq!(
                s.len(),
                expected_lengths[i],
                "length mismatch for {}-B entropy: got {}, expected {}",
                len,
                s.len(),
                expected_lengths[i]
            );
        }
    }
}
