//! Structural inspection of an ms1 string for debugging / future ms-cli.

use crate::envelope;
use crate::error::Result;
use crate::tag::Tag;
use codex32::Codex32String;

/// Structural dump of a parsed ms1 string. `#[non_exhaustive]` per SPEC §10
/// — v0.2+ may add fields (share-index detail, threshold-layer hints,
/// derivation metadata).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct InspectReport {
    /// Expected "ms" in v0.1.
    pub hrp: String,
    /// Expected 0 in v0.1.
    pub threshold: u8,
    /// The parsed type tag (id field).
    pub tag: Tag,
    /// Expected 's' in v0.1.
    pub share_index: char,
    /// 0x00 in v0.1 (reserved); becomes type discriminator in v0.2+.
    pub prefix_byte: u8,
    /// Payload bytes after the prefix byte.
    pub payload_bytes: Vec<u8>,
    /// BCH verification result. True if the upstream codex32 parser accepted.
    pub checksum_valid: bool,
}

/// Inspect an ms1 string. Less strict than `decode()`: returns a report even
/// for strings that would fail decoder validity rules (e.g., wrong threshold,
/// reserved-not-emitted tag, non-zero prefix byte) — caller can examine the
/// fields to diagnose what's wrong. Still requires a valid BIP-93 parse.
pub fn inspect(s: &str) -> Result<InspectReport> {
    // `?` leverages From<codex32::Error> for Error.
    let c = Codex32String::from_string(s.to_string())?;
    let s_owned = c.to_string();
    let fields = envelope::extract_wire_fields(&s_owned)?;

    // For tag construction in inspect we accept whatever bytes were on the wire
    // (alphabet-valid or not) — surfacing the raw observation is the point.
    let tag = match std::str::from_utf8(&fields.id_bytes) {
        Ok(t) => Tag::try_new(t).unwrap_or_else(|_| Tag::from_raw_bytes(fields.id_bytes)),
        Err(_) => Tag::from_raw_bytes(fields.id_bytes),
    };

    let payload_with_prefix = c.parts().data();
    let (prefix_byte, payload_bytes) = if payload_with_prefix.is_empty() {
        (0u8, Vec::new())
    } else {
        (payload_with_prefix[0], payload_with_prefix[1..].to_vec())
    };

    Ok(InspectReport {
        hrp: fields.hrp.to_string(),
        threshold: fields.threshold_byte - b'0', // ASCII to digit
        tag,
        share_index: fields.share_index_byte as char,
        prefix_byte,
        payload_bytes,
        checksum_valid: true, // if from_string accepted, BCH was valid
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{encode, payload::Payload};

    #[test]
    fn inspect_v01_entr_returns_expected_fields() {
        let entropy = vec![0xAAu8; 16];
        let s = encode::encode(Tag::ENTR, &Payload::Entr(entropy.clone())).unwrap();
        let r = inspect(&s).unwrap();
        assert_eq!(r.hrp, "ms");
        assert_eq!(r.threshold, 0);
        assert_eq!(r.tag, Tag::ENTR);
        assert_eq!(r.share_index, 's');
        assert_eq!(r.prefix_byte, 0x00);
        assert_eq!(r.payload_bytes, entropy);
        assert!(r.checksum_valid);
    }

    #[test]
    fn inspect_returns_report_for_decoder_rejects() {
        // A non-zero-prefix string: decode() rejects, inspect() returns the report.
        let mut data = vec![0x01u8];
        data.extend_from_slice(&[0xAAu8; 16]);
        let c = Codex32String::from_seed("ms", 0, "entr", codex32::Fe::S, &data).unwrap();
        let r = inspect(&c.to_string()).unwrap();
        assert_eq!(r.prefix_byte, 0x01); // would fail decode rule 8, inspect surfaces it
    }
}
