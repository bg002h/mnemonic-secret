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

// ─── The PHRASE RULE (SPEC_ms_hashlock §4.3) ────────────────────────────────
//
// IT LIVES HERE, IN THE CODEC, for the reason the module header already gives
// for the derivation: one crate, one corpus, one SHA pin, one provenance pin
// for the Go port. Until H6 the rule was `ms-cli`'s `validate_phrase`, private
// to that binary; `me sysw pack`'s `phrase:` record must apply the SAME rule
// byte for byte (SPEC_hashlock_H6 §3.1) and `me` depends on `ms-codec`, not on
// `ms-cli`. Leaving it where it was would have produced a THIRD copy of a rule
// whose whole point is that the host and the device cannot disagree about what
// a phrase is.
//
// `ms-cli`'s `validate_phrase` now delegates here and keeps only its own
// message rendering, so there is still exactly one implementation.

/// The phrase cap. Its own constant on each side, lockstep-pinned; NOT the
/// device's plate-legibility `passphrase.MaxLen`.
pub const HASHLOCK_PHRASE_MAX_CHARS: usize = 100;

/// The shortest string `looks_like_ms1` will call ms1-shaped.
const MIN_MS1_LEN: usize = 48;

const BECH32_CHARSET: &str = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";

/// Why a phrase was refused. One variant per rule, in the order the rule
/// checks them; the CALLER renders the sentence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhraseRefusal {
    /// No bytes at all.
    Empty,
    /// A byte outside `0x20..=0x7E`, with the byte and its position.
    NotPrintableAscii {
        /// The offending byte.
        byte: u8,
        /// Its zero-based position.
        at: usize,
    },
    /// An ms1 string — a preimage plate, not a phrase.
    Ms1Shaped,
    /// Over `HASHLOCK_PHRASE_MAX_CHARS`.
    TooLong {
        /// The length that was measured.
        chars: usize,
    },
    /// Exactly 64 hex characters — a preimage in hex, not a phrase.
    Hex64,
}

/// `looks_like_ms1` over the NORMALISED token: trimmed, ASCII-lowercased,
/// display separators (whitespace, `-`, `,`) stripped, then at least 48
/// characters, an `ms1` prefix and only bech32 characters.
///
/// NO CHECKSUM, deliberately. A GROUPED plate is what `ms hashlock`'s
/// engraving card prints and therefore what an operator retypes, and a
/// checksum test would answer false for it — so the guard would miss the one
/// spelling it exists to catch.
pub fn looks_like_ms1(raw: &str) -> bool {
    let t: String = raw
        .trim()
        .to_ascii_lowercase()
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-' && *c != ',')
        .collect();
    t.len() >= MIN_MS1_LEN
        && t.starts_with("ms1")
        && t[3..].chars().all(|c| BECH32_CHARSET.contains(c))
}

/// The rule. ORDER MATTERS and is the spec's: empty, printable ASCII,
/// ms1-shape (BEFORE the cap, so a grouped plate string gets the `--in`
/// remedy and not "too long"), the cap, 64-hex.
///
/// It changes nothing: no trim, no case fold, no normalisation. The shape test
/// works on a copy.
pub fn validate_phrase(bytes: &[u8]) -> core::result::Result<(), PhraseRefusal> {
    if bytes.is_empty() {
        return Err(PhraseRefusal::Empty);
    }
    if let Some((at, &byte)) = bytes
        .iter()
        .enumerate()
        .find(|(_, b)| !(0x20..=0x7e).contains(*b))
    {
        return Err(PhraseRefusal::NotPrintableAscii { byte, at });
    }
    // All bytes are printable ASCII now, so this is a &str.
    let s = core::str::from_utf8(bytes).expect("printable ASCII is UTF-8");
    if looks_like_ms1(s) {
        return Err(PhraseRefusal::Ms1Shaped);
    }
    if s.len() > HASHLOCK_PHRASE_MAX_CHARS {
        return Err(PhraseRefusal::TooLong { chars: s.len() });
    }
    if s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(PhraseRefusal::Hex64);
    }
    Ok(())
}

/// The QR text a hashlock PHRASE plate carries (SPEC_hashlock_H6 §8.6), byte
/// for byte: three labelled lines, LF-separated, NO trailing newline, the
/// phrase LAST.
///
/// The phrase is last so a reader knows where it ends: it may itself contain
/// `:` and spaces, and everything after `phrase: ` on the final line is the
/// phrase, verbatim, with real `0x20` spaces.
///
/// The method line names the ALGORITHM in full — not the `--method` selector —
/// so a reader with the plate and no tool can reproduce the derivation. Its
/// parameters are read from `HASHLOCK_SALT`, `HASHLOCK_ITERATIONS` and
/// `HASHLOCK_DKLEN` and never from a literal, so a parameter change cannot
/// leave the plate lying.
///
/// `hashlock v1` is the VERSION TAG of this TEXT, not of the derivation. A
/// future parameter set gets `hashlock v2`.
///
/// IT RETURNS `Zeroizing<String>` BECAUSE THE PHRASE IS IN IT. Every other
/// phrase-bearing value in this workspace is protected -- `read_phrase_from`
/// and `read_phrase_stdin` return `Zeroizing<Vec<u8>>`, `preimage_hardened`
/// and `preimage_sha256` return `Zeroizing<[u8; 32]>`, and the kind carries
/// `Payload::Preimage(Zeroizing<[u8; 32]>)` -- and a plain `String` here would
/// have been the one hole in that surface, holding the phrase in the clear on
/// the heap until the allocator happened to reuse the page.
///
/// **The buffer is `Zeroizing` from the FIRST byte, and it is allocated once.**
/// Wrapping a finished `format!` would be no protection at all: the `format!`
/// would build an unprotected `String` containing the phrase and the wrap would
/// only guard the copy. The exact capacity is reserved up front so no `push_str`
/// can reallocate and abandon an unwiped buffer part-way through. The `method`
/// line is deliberately NOT protected -- it is three compile-time constants and
/// carries nothing of the phrase.
pub fn qr_text(hardened: bool, phrase: &str) -> Zeroizing<String> {
    const HEAD: &str = "hashlock v1\n";
    const LABEL: &str = "\nphrase: ";
    let method = if hardened {
        format!(
            "method: pbkdf2-hmac-sha256 iterations={HASHLOCK_ITERATIONS} salt={} dklen={HASHLOCK_DKLEN}",
            core::str::from_utf8(HASHLOCK_SALT).expect("the salt is ASCII"),
        )
    } else {
        "method: sha256".to_string()
    };
    let mut out: Zeroizing<String> = Zeroizing::new(String::with_capacity(
        HEAD.len() + method.len() + LABEL.len() + phrase.len(),
    ));
    out.push_str(HEAD);
    out.push_str(&method);
    out.push_str(LABEL);
    out.push_str(phrase);
    out
}
