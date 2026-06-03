//! K-of-N codex32 Shamir share encoding (ms v0.2).
//!
//! A secret (`entr` or `mnem`) splits into N shares, any K of which recombine
//! to the original — using codex32's *native* threshold(k)+index Shamir
//! mechanism, NOT a payload byte (SPEC_ms_v0_2_kofn §1). The codex32 header
//! threshold char is the share-vs-single discriminator; the prefix byte
//! (`0x00`=entr / `0x02`=mnem) remains the payload-KIND discriminator, recovered
//! only on the secret-at-S after interpolation.
//!
//! v0.1/mnem single-strings stay byte-identical: `encode_shares(tag, ZERO, 1, &p)`
//! reduces to the exact `package()`/`encode()` construction (the Phase-0 gate).

use crate::error::{Error, Result};

/// A codex32 share threshold.
///
/// `ZERO` is the unshared v0.1 single-string sentinel (codex32 threshold `0`,
/// share-index `s`); `new(k)` accepts a K-of-N share threshold `k in 2..=9`
/// (codex32 `from_seed` accepts threshold `0` or `2..=9` only — `1` is invalid).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Threshold(u8);

impl Threshold {
    /// The unshared single-string sentinel (threshold `0`). A const, NOT
    /// `new(0)` — `new` only admits the K-of-N share range `2..=9`.
    pub const ZERO: Threshold = Threshold(0);

    /// Construct a K-of-N share threshold. `k` MUST be in `2..=9`, else
    /// `Error::InvalidThreshold(k)`.
    pub fn new(k: u8) -> Result<Threshold> {
        if (2..=9).contains(&k) {
            Ok(Threshold(k))
        } else {
            Err(Error::InvalidThreshold(k))
        }
    }

    /// The threshold value (`0` for `ZERO`, `2..=9` for a share threshold).
    pub fn get(self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_accepts_2_through_9() {
        for k in 2u8..=9 {
            let t = Threshold::new(k).unwrap_or_else(|e| panic!("new({k}) should be Ok, got {e:?}"));
            assert_eq!(t.get(), k);
        }
    }

    #[test]
    fn new_rejects_zero() {
        assert!(matches!(Threshold::new(0), Err(Error::InvalidThreshold(0))));
    }

    #[test]
    fn new_rejects_one() {
        assert!(matches!(Threshold::new(1), Err(Error::InvalidThreshold(1))));
    }

    #[test]
    fn new_rejects_ten() {
        assert!(matches!(Threshold::new(10), Err(Error::InvalidThreshold(10))));
    }

    #[test]
    fn zero_const_get_is_zero() {
        assert_eq!(Threshold::ZERO.get(), 0);
    }

    #[test]
    fn new_five_get_is_five() {
        assert_eq!(Threshold::new(5).unwrap().get(), 5);
    }
}
