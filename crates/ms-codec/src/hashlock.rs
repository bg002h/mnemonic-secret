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

/// Which hash the SCRIPT commits to. Crate-local by design: the spec forbids a
/// shared type across repo boundaries (§5), so every consumer defines its own
/// and maps onto these functions.
///
/// NOT the same axis as the preimage METHOD (`preimage_hardened` vs
/// `preimage_sha256`). The two share the token `sha256` and mean different
/// things; four separate reviews of this cycle each found a defect caused by
/// that collision. Where both could be read, name both or neither.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashKind {
    /// `sha256(X)`.
    Sha256,
    /// `sha256d(X)` = `sha256(sha256(X))`.
    Hash256,
    /// `ripemd160(X)`, the bare primitive.
    Ripemd160,
    /// `hash160(X)` = `ripemd160(sha256(X))`.
    Hash160,
}

/// A digest and its width. The width is a CONSEQUENCE of the kind, never a
/// separate thing to keep in sync.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DigestBytes {
    /// A 32-byte digest: `sha256` or `hash256`.
    B32([u8; 32]),
    /// A 20-byte digest: `ripemd160` or `hash160`.
    B20([u8; 20]),
}

impl DigestBytes {
    /// The digest bytes, at their kind's width.
    pub fn as_slice(&self) -> &[u8] {
        match self {
            DigestBytes::B32(b) => &b[..],
            DigestBytes::B20(b) => &b[..],
        }
    }
}

impl HashKind {
    /// THE ONE NAMED DISPATCH in this crate. Spec §10 requires the KAT to
    /// exercise the dispatch and not only the four functions, because four
    /// correct functions plus one mis-wired arm is the same lost-funds outcome
    /// with a different cause.
    pub fn digest(self, preimage: &[u8; 32]) -> DigestBytes {
        match self {
            HashKind::Sha256 => DigestBytes::B32(digest_sha256(preimage)),
            HashKind::Hash256 => DigestBytes::B32(digest_hash256(preimage)),
            HashKind::Ripemd160 => DigestBytes::B20(digest_ripemd160(preimage)),
            HashKind::Hash160 => DigestBytes::B20(digest_hash160(preimage)),
        }
    }

    /// The lowercase miniscript fragment name. Case is rejected, never folded.
    pub fn token(self) -> &'static str {
        match self {
            HashKind::Sha256 => "sha256",
            HashKind::Hash256 => "hash256",
            HashKind::Ripemd160 => "ripemd160",
            HashKind::Hash160 => "hash160",
        }
    }

    /// The Script opcode the spending script actually contains.
    ///
    /// Spec §3 F1: every hash fragment lowers to
    /// `OP_SIZE <32> OP_EQUALVERIFY <hashop> <h> OP_EQUAL`, and `<hashop>` is
    /// THE VARIABLE -- only it and the digest width move. The engraving card
    /// names this opcode, and it named `OP_SHA256` under every kind for one
    /// release: a false statement about the object in the operator's hand, on
    /// the one axis this whole cycle exists to disambiguate, in the line a
    /// kind-confused operator would use to check themselves.
    pub fn opcode(self) -> &'static str {
        match self {
            HashKind::Sha256 => "OP_SHA256",
            HashKind::Hash256 => "OP_HASH256",
            HashKind::Ripemd160 => "OP_RIPEMD160",
            HashKind::Hash160 => "OP_HASH160",
        }
    }
}

/// H = SHA-256(X): what a sha256 policy carries and the plate shows. Public.
pub fn digest_sha256(preimage: &[u8; 32]) -> [u8; 32] {
    let mut h = [0u8; 32];
    h.copy_from_slice(&Sha256::digest(preimage));
    h
}

/// H = SHA-256(SHA-256(X)) -- `sha256d`. THE DANGEROUS ONE: written one word
/// short as `sha256(x)` it is still 32 bytes, still type-checks, still lowers,
/// and Core still agrees with the address. Only the KAT catches it.
pub fn digest_hash256(preimage: &[u8; 32]) -> [u8; 32] {
    let once = Sha256::digest(preimage);
    let mut h = [0u8; 32];
    h.copy_from_slice(&Sha256::digest(once));
    h
}

/// H = RIPEMD-160(X) -- the BARE primitive, not hash160.
pub fn digest_ripemd160(preimage: &[u8; 32]) -> [u8; 20] {
    use ripemd::Ripemd160;
    let mut h = [0u8; 20];
    h.copy_from_slice(&Ripemd160::digest(preimage));
    h
}

/// H = RIPEMD-160(SHA-256(X)) -- `hash160`. Same width as ripemd160 and a
/// different preimage relation (spec §3 F4).
pub fn digest_hash160(preimage: &[u8; 32]) -> [u8; 20] {
    use ripemd::Ripemd160;
    let inner = Sha256::digest(preimage);
    let mut h = [0u8; 20];
    h.copy_from_slice(&Ripemd160::digest(inner));
    h
}

// NO `digest` ALIAS. An earlier draft kept the old name as a `#[deprecated]`
// shim "so phase 3 keeps compiling". Every internal call site then becomes a
// deprecation warning, and `-D warnings` is a REQUIRED CI context. Measured by
// building that counterfactual (2026-09-15): `clippy -p ms-codec
// --all-targets` reports 14, `-p ms-cli --all-targets` 4. Without
// `--all-targets` ms-codec reports 1, which is why a gate that drops the flag
// does not see this at all. (An earlier record said "nine"; it did not
// reproduce under any flag combination, and this file's own new unit tests are
// two of the 14.)
//
// `digest` is renamed to `digest_sha256` and its call sites in THIS repo move
// with it. Phase 3 (`me-cli`) is a different repo pinned to a git rev, so it
// does not break until it chooses to bump.

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

    /// The four functions are FOUR FUNCTIONS (spec §3 F4), and the two that
    /// share a width are the pair with no structural signal — so they are
    /// asserted to DIFFER, not merely to compute. The KAT in
    /// `tests/hashlock_kat.rs` pins the VALUES against `python3 hashlib`;
    /// this pins the STRUCTURE, and needs no corpus to do it.
    #[test]
    fn the_four_digests_are_four_different_functions() {
        let x = [0xabu8; 32];
        let s = digest_sha256(&x);
        let d = digest_hash256(&x);
        let r = digest_ripemd160(&x);
        let h = digest_hash160(&x);
        assert_ne!(
            s, d,
            "hash256 is sha256d, not sha256 — one word short is the whole defect"
        );
        assert_ne!(
            r, h,
            "hash160 is ripemd160(sha256(x)); ripemd160 is the bare primitive"
        );
        assert_eq!(s.len(), 32);
        assert_eq!(d.len(), 32);
        assert_eq!(r.len(), 20);
        assert_eq!(h.len(), 20);
    }

    /// The dispatch is the thing every caller uses, so it is tested as such:
    /// four correct functions behind one mis-wired arm is the same lost-funds
    /// outcome with a different cause.
    #[test]
    fn the_dispatch_selects_the_matching_function() {
        let x = [0x11u8; 32];
        assert_eq!(
            HashKind::Sha256.digest(&x),
            DigestBytes::B32(digest_sha256(&x))
        );
        assert_eq!(
            HashKind::Hash256.digest(&x),
            DigestBytes::B32(digest_hash256(&x))
        );
        assert_eq!(
            HashKind::Ripemd160.digest(&x),
            DigestBytes::B20(digest_ripemd160(&x))
        );
        assert_eq!(
            HashKind::Hash160.digest(&x),
            DigestBytes::B20(digest_hash160(&x))
        );
    }

    /// The tokens are the miniscript fragment names, lowercase. They are the
    /// operand name on the engraving card's `for md compose:` line and the
    /// `hash:<kind>:` tag in the record, so a typo here composes a wallet
    /// nobody can spend.
    #[test]
    fn tokens_are_the_lowercase_fragment_names() {
        assert_eq!(HashKind::Sha256.token(), "sha256");
        assert_eq!(HashKind::Hash256.token(), "hash256");
        assert_eq!(HashKind::Ripemd160.token(), "ripemd160");
        assert_eq!(HashKind::Hash160.token(), "hash160");
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
pub fn qr_text(hardened: bool, kind: HashKind, phrase: &str) -> Zeroizing<String> {
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
    // ITS OWN LINE, never appended to `method:` -- H6 §6.5 pins that line at 73
    // characters and the plate refuses an eleventh row at every font rung.
    let kind_line = format!("\nhash: {}", kind.token());
    let mut out: Zeroizing<String> = Zeroizing::new(String::with_capacity(
        HEAD.len() + method.len() + kind_line.len() + LABEL.len() + phrase.len(),
    ));
    out.push_str(HEAD);
    out.push_str(&method);
    out.push_str(&kind_line);
    out.push_str(LABEL);
    out.push_str(phrase);
    out
}
