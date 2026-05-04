//! Payload type — v0.1: Entr (BIP-39 entropy) only.

use crate::consts::VALID_ENTR_LENGTHS;
use crate::error::{Error, Result};
use crate::tag::Tag;

/// v0.1 payload kind. Future kinds (Mnem, Seed, Xprv) will arrive in v0.2+
/// with their own framing per SPEC §1, §3.3, §8.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PayloadKind {
    /// BIP-39 entropy (16/20/24/28/32 B).
    Entr,
}

/// v0.1 payload.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Payload {
    /// BIP-39 entropy. Length MUST be in {16, 20, 24, 28, 32} bytes
    /// (bijective with BIP-39 word counts {12, 15, 18, 21, 24}).
    ///
    /// **Caller responsibility:** ms-codec does NOT check the statistical
    /// quality of these bytes. Callers are responsible for sourcing entropy
    /// from a vetted CSPRNG, or from a BIP-39 mnemonic the user already trusts.
    /// FIPS-style entropy-quality checks would slow encoding and provide false
    /// assurance — they cannot detect attacker-supplied "pseudo-random" seeds
    /// crafted to pass standard randomness tests. See SPEC §3.6.
    Entr(Vec<u8>),
}

impl Payload {
    /// Validate the payload's intrinsic structure (byte length for Entr).
    /// Encoder MUST call this before emitting; decoder calls it after extracting
    /// the payload bytes following the reserved-prefix byte.
    pub fn validate(&self) -> Result<()> {
        match self {
            Payload::Entr(data) => {
                if !VALID_ENTR_LENGTHS.contains(&data.len()) {
                    return Err(Error::PayloadLengthMismatch {
                        tag: *Tag::ENTR.as_bytes(),
                        expected: VALID_ENTR_LENGTHS,
                        got: data.len(),
                    });
                }
                Ok(())
            }
        }
    }

    /// The PayloadKind discriminant.
    pub fn kind(&self) -> PayloadKind {
        match self {
            Payload::Entr(_) => PayloadKind::Entr,
        }
    }

    /// Borrow the inner byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Payload::Entr(data) => data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entr_accepts_all_bip39_lengths() {
        for len in [16usize, 20, 24, 28, 32] {
            let p = Payload::Entr(vec![0u8; len]);
            p.validate()
                .unwrap_or_else(|e| panic!("expected ok for len {}, got {:?}", len, e));
        }
    }

    #[test]
    fn entr_rejects_off_by_one_lengths() {
        for len in [15usize, 17, 19, 21, 23, 25, 31, 33] {
            let p = Payload::Entr(vec![0u8; len]);
            assert!(
                matches!(p.validate(), Err(Error::PayloadLengthMismatch { .. })),
                "expected reject for len {}",
                len
            );
        }
    }

    #[test]
    fn entr_rejects_zero_length() {
        let p = Payload::Entr(vec![]);
        assert!(matches!(
            p.validate(),
            Err(Error::PayloadLengthMismatch { .. })
        ));
    }

    #[test]
    fn kind_returns_entr() {
        assert_eq!(Payload::Entr(vec![0u8; 16]).kind(), PayloadKind::Entr);
    }
}
