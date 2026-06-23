//! Friendly human-readable messages for `ms_codec::codex32::Error` variants.
//!
//! Realizes SPEC §6.2. Stable since `codex32 = "=0.1.0"` is exact-pinned;
//! see `ms-codec/src/codex32/mod.rs (vendored, Cycle-B)` for the variant
//! source.

use ms_codec::codex32::Error;

/// Map each `ms_codec::codex32::Error` variant to a one-line user-facing message.
pub fn friendly_codex32(e: &Error) -> String {
    match e {
        Error::Field(fe) => format!("invalid bech32 character: {:?}", fe),
        Error::IdNotLength4(n) => format!("id field must be 4 chars, got {}", n),
        Error::IncompleteGroup(n) => format!(
            "incomplete bit group at end of payload (got {} bits; max 4 allowed)",
            n
        ),
        Error::InvalidLength(n) => format!(
            "string length {} not a valid codex32 length (need 48-93 short or 125-127 long)",
            n
        ),
        Error::InvalidChar(c) => format!("invalid character '{}' (not in codex32 alphabet)", c),
        Error::InvalidCase(_, c) => format!(
            "mixed case at character '{}' (codex32 strings must be all-lower or all-upper)",
            c
        ),
        Error::InvalidChecksum { checksum, .. } => format!(
            "BCH checksum invalid ({} code); engraving error or transcription typo",
            checksum
        ),
        Error::InvalidThreshold(c) => format!(
            "threshold character '{}' invalid (must be '0' for unshared or '2'-'9' for K-of-N)",
            c
        ),
        Error::InvalidThresholdN(n) => format!("threshold value {} invalid (must be 0 or 2-9)", n),
        Error::InvalidShareIndex(fe) => format!(
            "share index '{}' invalid for threshold-0 (BIP-93 requires 's')",
            fe.to_char()
        ),
        Error::MismatchedLength(a, b) => format!(
            "share length mismatch: {} vs {} (all shares of one secret must share length)",
            a, b
        ),
        Error::MismatchedHrp(a, b) => format!("HRP mismatch among shares: {:?} vs {:?}", a, b),
        Error::MismatchedThreshold(a, b) => {
            format!("threshold mismatch among shares: {} vs {}", a, b)
        }
        Error::MismatchedId(a, b) => format!("id mismatch among shares: {:?} vs {:?}", a, b),
        Error::RepeatedIndex(fe) => format!(
            "share index '{}' repeated (each share in a set must have a distinct index)",
            fe.to_char()
        ),
        Error::ThresholdNotPassed {
            threshold,
            n_shares,
        } => format!("not enough shares: have {}, need {}", n_shares, threshold),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_variant_produces_non_empty_message() {
        // Construct one example per variant. Some variants need helper types;
        // skip the ones we can't trivially construct (Field needs field::Error).
        let cases: Vec<Error> = vec![
            Error::IdNotLength4(3),
            Error::IncompleteGroup(5),
            Error::InvalidLength(99),
            Error::InvalidChar('!'),
            Error::InvalidThreshold('@'),
            Error::InvalidThresholdN(11),
            Error::MismatchedLength(50, 51),
            Error::MismatchedHrp("ms".into(), "mk".into()),
            Error::MismatchedThreshold(2, 3),
            Error::MismatchedId("abcd".into(), "efgh".into()),
            Error::ThresholdNotPassed {
                threshold: 3,
                n_shares: 1,
            },
        ];
        for e in &cases {
            let msg = friendly_codex32(e);
            assert!(!msg.is_empty(), "empty message for {:?}", e);
            assert!(
                !msg.contains("Debug"),
                "raw debug formatting leaked in: {}",
                msg
            );
        }
    }

    #[test]
    fn invalid_checksum_message_mentions_checksum() {
        let e = Error::InvalidChecksum {
            checksum: "short",
            string: "ms10...".into(),
        };
        let msg = friendly_codex32(&e);
        assert!(msg.contains("BCH checksum invalid"));
        assert!(msg.contains("short"));
    }
}
