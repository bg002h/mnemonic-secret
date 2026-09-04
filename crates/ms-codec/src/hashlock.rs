//! The hashlock preimage derivation (SPEC_ms_hashlock §2).
//!
//! THE RULE LIVES HERE, in the codec, beside the kind that carries its
//! output: one crate, one corpus, one SHA pin, one provenance pin for the Go
//! port. `ms hashlock` is a thin verb over these four functions.
//!
//! Two methods, the operator's choice (brainstorm L5): `preimage_hardened`
//! is PBKDF2-HMAC-SHA256 with a fixed salt, 100,000 iterations and dkLen 32
//! (L4); `preimage_sha256` is one SHA-256 of the phrase bytes. Both take the
//! phrase as BYTES, exactly as given -- no trimming, folding or normalising
//! happens here or in any caller (§4.3). `digest` is SHA-256 of X, the value
//! the policy carries; it is public the moment the policy is engraved and is
//! therefore NOT zeroized.
//!
//! THE SALT IS FIXED AND HAS NO PARAMETER (L13). Changing it after any vector
//! ships is a new method, not a tweak: every engraved policy's preimage was
//! derived under this exact byte string.

use pbkdf2::pbkdf2_hmac;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::error::{Error, Result};

/// The fixed salt (ASCII, copyable by hand, domain-separated from BIP-39's
/// `"mnemonic"` and from `me`'s 16-byte random seal salt).
pub const HASHLOCK_SALT: &[u8] = b"ms-hashlock-v1";
/// PBKDF2 iteration count -- the operator's cap, chosen so a signer at a
/// tenth of the SH2's measured rate still derives in reasonable time.
pub const HASHLOCK_ITERATIONS: u32 = 100_000;
/// Derived-key length: a miniscript `sha256(H)` preimage is exactly 32 bytes.
pub const HASHLOCK_DKLEN: usize = 32;

/// X = PBKDF2-HMAC-SHA256(phrase, HASHLOCK_SALT, HASHLOCK_ITERATIONS, 32).
pub fn preimage_hardened(phrase: &[u8]) -> Zeroizing<[u8; 32]> {
    let mut x = Zeroizing::new([0u8; HASHLOCK_DKLEN]);
    pbkdf2_hmac::<Sha256>(phrase, HASHLOCK_SALT, HASHLOCK_ITERATIONS, &mut *x);
    x
}

/// X = SHA-256(phrase). The brainwallet construction; the CLI warns on it at
/// every length (L12) and this function does not judge.
pub fn preimage_sha256(phrase: &[u8]) -> Zeroizing<[u8; 32]> {
    let mut x = Zeroizing::new([0u8; 32]);
    x.copy_from_slice(&Sha256::digest(phrase));
    x
}

/// X from the OS CSPRNG, failing closed: an error, never a zeroed buffer.
/// Lives here rather than in the CLI so the whole preimage surface -- and its
/// randomness contract -- is one crate's (R0 r0 correctness I-2).
pub fn preimage_random() -> Result<Zeroizing<[u8; 32]>> {
    let mut x = Zeroizing::new([0u8; 32]);
    getrandom::fill(&mut *x).map_err(|_| Error::RandomnessUnavailable)?;
    Ok(x)
}

/// H = SHA-256(X): what the policy carries and the plate shows. Public.
pub fn digest(preimage: &[u8; 32]) -> [u8; 32] {
    let mut h = [0u8; 32];
    h.copy_from_slice(&Sha256::digest(preimage));
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hardened_output_is_zeroizing_and_32() {
        let x = preimage_hardened(b"x");
        assert_eq!(x.len(), 32);
        // Two calls agree: the salt and count are constants, not state.
        assert_eq!(&preimage_hardened(b"x")[..], &x[..]);
    }
}
