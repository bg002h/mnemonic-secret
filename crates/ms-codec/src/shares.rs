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

use crate::consts::{HRP, RESERVED_ID_BLOCKLIST};
use crate::envelope::payload_wire_bytes;
use crate::error::{Error, Result};
use crate::payload::Payload;
use crate::tag::Tag;
use codex32::{Codex32String, Fe};
use zeroize::Zeroizing;

/// The codex32 bech32 alphabet (32 chars). Index `s` (position 16) is the
/// secret-at-S index — never a distributed-share index.
const CODEX32_ALPHABET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

/// The 31 valid non-`s` share indices, taken from the bech32 alphabet in its
/// own order with `s` removed (deterministic, front-to-back). `n <= 31` is
/// enforced by `encode_shares`, so this pool never runs out.
fn non_s_index_pool() -> Vec<Fe> {
    CODEX32_ALPHABET
        .iter()
        .filter(|&&b| b != b's')
        .map(|&b| Fe::from_char(b as char).expect("alphabet char is a valid Fe"))
        .collect()
}

/// Generate a random 4-char codex32-alphabet `id`, re-rolling while it lands in
/// `RESERVED_ID_BLOCKLIST` (a v0.1 type-tag-shaped value). Uses `getrandom`
/// (0.3.x `getrandom::fill`) — no injected-RNG param (the `mk_codec::encode`
/// precedent).
fn random_id() -> String {
    loop {
        let mut raw = [0u8; 4];
        getrandom::fill(&mut raw).expect("getrandom::fill must not fail");
        let id: [u8; 4] = [
            CODEX32_ALPHABET[(raw[0] & 0x1f) as usize],
            CODEX32_ALPHABET[(raw[1] & 0x1f) as usize],
            CODEX32_ALPHABET[(raw[2] & 0x1f) as usize],
            CODEX32_ALPHABET[(raw[3] & 0x1f) as usize],
        ];
        if !RESERVED_ID_BLOCKLIST.contains(&id) {
            // Every byte is a codex32-alphabet ASCII char → always valid UTF-8.
            return String::from_utf8(id.to_vec()).expect("codex32 alphabet is ASCII");
        }
    }
}

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

/// Split a secret (`entr` or `mnem`) into `n` codex32 K-of-N shares.
///
/// - `threshold == ZERO`: `n` MUST be 1; returns a single string **byte-identical**
///   to `encode(tag, secret)` — the v0.1 single-string construction
///   (`from_seed(HRP, 0, tag, Fe::S, [prefix]||payload)`, deterministic). The
///   `id` stays the type `tag` (NOT random) — load-bearing for byte-identity.
/// - `threshold == k ∈ 2..=9`: validate `k <= n <= 31` (else `InvalidShareCount`).
///   A random 4-char `id` (not in `RESERVED_ID_BLOCKLIST`) keys the share-set.
///   The secret-at-S (`Fe::S`) holds the real payload; `k-1` random **defining
///   shares** at fixed canonical non-`s` indices + `interpolate_at` for the
///   remaining `n-(k-1)` indices produce the `n` **distributed** shares. The
///   secret-at-S is NEVER returned (it is the recovery target only).
///
/// Works identically for `entr` and `mnem` (byte-agnostic); language survives a
/// `mnem` split (it rides the secret-at-S wire bytes).
pub fn encode_shares(
    tag: Tag,
    threshold: Threshold,
    n: usize,
    secret: &Payload,
) -> Result<Vec<String>> {
    secret.validate()?;
    let bytes = payload_wire_bytes(secret);

    if threshold == Threshold::ZERO {
        // Unshared single-string: must be n==1; byte-identical to encode().
        if n != 1 {
            return Err(Error::InvalidShareCount { k: 0, n });
        }
        let single = Codex32String::from_seed(HRP, 0, tag.as_str(), Fe::S, &bytes[..])?;
        return Ok(vec![single.to_string()]);
    }

    let k = threshold.get();
    let k_usize = k as usize;
    // Bounds (SPEC §1): 2 <= k <= n <= 31 (31 valid non-`s` indices).
    if !(k_usize <= n && n <= 31) {
        return Err(Error::InvalidShareCount { k, n });
    }

    let id = random_id();
    let pool = non_s_index_pool();

    // 1. secret-at-S carries the real payload at index `s`, threshold `k`.
    let secret_s = Codex32String::from_seed(HRP, k_usize, &id, Fe::S, &bytes[..])?;

    // 2. k-1 random DEFINING shares at the first k-1 pool indices. Each gets a
    //    CSPRNG payload of the SAME byte length as the secret (Zeroizing scrub).
    //    The defining set [secret_s, def_1..def_{k-1}] is k points → fully
    //    determines the Shamir polynomial.
    let mut defining: Vec<Codex32String> = Vec::with_capacity(k_usize);
    defining.push(secret_s);
    for pool_idx in pool.iter().take(k_usize - 1) {
        let mut filler: Zeroizing<Vec<u8>> = Zeroizing::new(vec![0u8; bytes.len()]);
        getrandom::fill(&mut filler[..]).expect("getrandom::fill must not fail");
        let share = Codex32String::from_seed(HRP, k_usize, &id, *pool_idx, &filler[..])?;
        defining.push(share);
    }

    // 3. The n DISTRIBUTED shares: the k-1 defining shares (indices 0..k-1) plus
    //    interpolation-derived shares at the remaining n-(k-1) pool indices.
    //    The secret-at-S (defining[0]) is NEVER distributed.
    let mut distributed: Vec<String> = Vec::with_capacity(n);
    for share in defining.iter().skip(1) {
        distributed.push(share.to_string());
    }
    for pool_idx in pool.iter().take(n).skip(k_usize - 1) {
        let derived = Codex32String::interpolate_at(&defining, *pool_idx)?;
        distributed.push(derived.to_string());
    }

    debug_assert_eq!(distributed.len(), n);
    Ok(distributed)
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

    // --- encode_shares tests (Task 1.3) ---

    use crate::encode::encode;
    use crate::payload::Payload;
    use crate::tag::Tag;
    use codex32::{Codex32String, Fe};

    fn entr_p() -> Payload {
        Payload::Entr(vec![0xCDu8; 16])
    }
    fn mnem_p() -> Payload {
        Payload::Mnem { language: 1, entropy: vec![0xCDu8; 16] }
    }

    /// Re-parse a share string and return (threshold_char, share_index_char, id).
    fn share_header(s: &str) -> (char, char, String) {
        let sep = s.rfind('1').unwrap();
        let b = s.as_bytes();
        let threshold = b[sep + 1] as char;
        let id: String = s[sep + 2..sep + 6].to_string();
        let index = b[sep + 6] as char;
        (threshold, index, id)
    }

    #[test]
    fn zero_share_is_byte_identical_to_encode_entr() {
        let p = entr_p();
        let shares = encode_shares(Tag::ENTR, Threshold::ZERO, 1, &p).unwrap();
        assert_eq!(shares, vec![encode(Tag::ENTR, &p).unwrap()]);
    }

    #[test]
    fn zero_share_is_byte_identical_to_encode_mnem() {
        let p = mnem_p();
        let shares = encode_shares(Tag::ENTR, Threshold::ZERO, 1, &p).unwrap();
        assert_eq!(shares, vec![encode(Tag::ENTR, &p).unwrap()]);
    }

    #[test]
    fn zero_share_requires_n_eq_1() {
        let p = entr_p();
        assert!(matches!(
            encode_shares(Tag::ENTR, Threshold::ZERO, 2, &p),
            Err(Error::InvalidShareCount { k: 0, n: 2 })
        ));
    }

    #[test]
    fn encode_shares_2_of_3_shape() {
        let p = entr_p();
        let shares = encode_shares(Tag::ENTR, Threshold::new(2).unwrap(), 3, &p).unwrap();
        assert_eq!(shares.len(), 3);
        // Each parses, threshold char '2', distinct non-`s` indices, same id.
        let mut indices = Vec::new();
        let mut ids = Vec::new();
        for s in &shares {
            Codex32String::from_string(s.clone()).expect("each share must parse");
            let (thr, idx, id) = share_header(s);
            assert_eq!(thr, '2', "threshold char");
            assert_ne!(idx, 's', "distributed share must not be index s");
            indices.push(idx);
            ids.push(id);
        }
        // Distinct indices.
        let mut sorted = indices.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), indices.len(), "indices must be distinct");
        // Same id across the set.
        assert!(ids.windows(2).all(|w| w[0] == w[1]), "id must be shared");
    }

    #[test]
    fn encode_shares_rejects_n_below_k() {
        let p = entr_p();
        assert!(matches!(
            encode_shares(Tag::ENTR, Threshold::new(2).unwrap(), 1, &p),
            Err(Error::InvalidShareCount { k: 2, n: 1 })
        ));
    }

    #[test]
    fn encode_shares_rejects_n_32() {
        let p = entr_p();
        assert!(matches!(
            encode_shares(Tag::ENTR, Threshold::new(2).unwrap(), 32, &p),
            Err(Error::InvalidShareCount { k: 2, n: 32 })
        ));
    }

    #[test]
    fn encode_shares_id_not_in_blocklist() {
        // Statistical: across many splits, the random id never lands in the blocklist.
        let p = entr_p();
        for _ in 0..64 {
            let shares = encode_shares(Tag::ENTR, Threshold::new(2).unwrap(), 2, &p).unwrap();
            let (_, _, id) = share_header(&shares[0]);
            let id_bytes: [u8; 4] = id.as_bytes().try_into().unwrap();
            assert!(
                !crate::consts::RESERVED_ID_BLOCKLIST.contains(&id_bytes),
                "id {id:?} must not be in RESERVED_ID_BLOCKLIST"
            );
        }
    }

    /// Inline round-trip (combine_shares lands in Task 1.4): any k of the n
    /// distributed shares, interpolated at S, recover the secret wire bytes.
    #[test]
    fn encode_shares_round_trip_via_interpolate_entr_and_mnem() {
        for p in [entr_p(), mnem_p()] {
            let secret_wire = crate::envelope::payload_wire_bytes(&p);
            for k in 2u8..=9 {
                let n = (k as usize) + 2; // exercise interpolation-derived shares
                let shares = encode_shares(Tag::ENTR, Threshold::new(k).unwrap(), n, &p).unwrap();
                assert_eq!(shares.len(), n);
                let parsed: Vec<Codex32String> = shares
                    .iter()
                    .map(|s| Codex32String::from_string(s.clone()).unwrap())
                    .collect();
                // First k and last k subsets both recover the secret.
                for subset in [&parsed[..k as usize], &parsed[n - k as usize..]] {
                    let recovered = Codex32String::interpolate_at(subset, Fe::S).unwrap();
                    assert_eq!(
                        recovered.parts().data(),
                        secret_wire[..],
                        "k={k} n={n} kind={:?} must recover secret wire bytes",
                        p.kind()
                    );
                }
            }
        }
    }
}
