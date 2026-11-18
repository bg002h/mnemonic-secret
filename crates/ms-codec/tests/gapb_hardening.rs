//! Kill tests for baseline GAP-B mutation survivors (tests only).
//!
//! * `chars_inv_*`: every non-alphabet ASCII byte standing where a `p` was is
//!   refused with `InvalidChar` (kills the 70 `delete -` mutants on CHARS_INV
//!   in `codex32/field.rs`; each would map the byte to value 1 = 'p').
//! * `from_string_length_94_*`: BIP-93 regular strings are 48..=93 chars; 94 is
//!   not a legal length (kills `<` -> `<=` at `codex32/mod.rs:199:64`).
//! * `bm_*`: seeded 2..=4-error sweep over all five lengths with exact position
//!   recovery (kills `m += 1` -> `m *= 1` at `bch_decode.rs:238:15`).

use ms_codec::codex32::{Codex32String, Error as CxError};
use ms_codec::{decode_with_correction, encode, Payload, Tag};

const ABC: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

/// A valid lowercase ms1 string whose data part contains a `p` after the
/// `ms1` prefix, with the byte index of that `p`.
fn string_with_p() -> (String, usize) {
    for seed in 0u8..=255 {
        let e: Vec<u8> = (0..16u8)
            .map(|i| i.wrapping_mul(37).wrapping_add(seed))
            .collect();
        let s = encode(Tag::ENTR, &Payload::Entr(e)).unwrap();
        if let Some(i) = s[3..].find('p') {
            return (s, i + 3);
        }
    }
    panic!("no seed produced a 'p'");
}

#[test]
fn chars_inv_non_alphabet_bytes_refused_as_invalid_char() {
    let (s, idx) = string_with_p();
    // Sanity: the unmodified string parses.
    Codex32String::from_string(s.clone()).expect("base string parses");
    let mut checked = 0;
    for b in 0u8..128 {
        let c = b as char;
        if b == b'1' || ABC.contains(&(c.to_ascii_lowercase() as u8)) {
            continue;
        }
        // Lowercase letters stay in a lowercase string; everything else goes
        // into an all-uppercase string so the case check does not fire first.
        let mut bytes = if c.is_ascii_lowercase() {
            s.clone().into_bytes()
        } else {
            s.to_ascii_uppercase().into_bytes()
        };
        bytes[idx] = b;
        let bad = String::from_utf8(bytes).unwrap();
        match Codex32String::from_string(bad) {
            Err(CxError::InvalidChar(got)) => assert_eq!(got, c, "byte {b}"),
            other => panic!("byte {b} ({c:?}): expected InvalidChar, got {other:?}"),
        }
        checked += 1;
    }
    // 128 bytes - 55 alphabet bytes (9 digits + 23 letters x 2 cases) - '1'.
    assert_eq!(checked, 72);
}

#[test]
fn from_string_length_94_is_invalid_length() {
    let s = format!("ms1{}", "q".repeat(91));
    assert_eq!(s.len(), 94);
    match Codex32String::from_string(s) {
        Err(CxError::InvalidLength(94)) => {}
        other => panic!("expected InvalidLength(94), got {other:?}"),
    }
}

/// xorshift64, fixed seed: deterministic sweep.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

#[test]
fn bm_repairs_every_random_2_to_4_error_pattern_with_exact_positions() {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    for len in [16usize, 20, 24, 28, 32] {
        let e: Vec<u8> = (0..len as u8).collect();
        let clean = encode(Tag::ENTR, &Payload::Entr(e.clone())).unwrap();
        let dp = clean.len() - 3;
        for _ in 0..400 {
            let k = 2 + (rng.next() % 3) as usize;
            let mut pos: Vec<usize> = Vec::new();
            while pos.len() < k {
                let p = (rng.next() % dp as u64) as usize;
                if !pos.contains(&p) {
                    pos.push(p);
                }
            }
            let mut chars: Vec<u8> = clean.clone().into_bytes();
            for &p in &pos {
                let cur = ABC.iter().position(|&b| b == chars[3 + p]).unwrap() as u64;
                let mask = 1 + rng.next() % 31;
                chars[3 + p] = ABC[(cur ^ mask) as usize];
            }
            let bad = String::from_utf8(chars).unwrap();
            let (_, payload, corr) = decode_with_correction(&bad)
                .unwrap_or_else(|err| panic!("len={len} pos={pos:?} must repair: {err:?}"));
            assert_eq!(payload, Payload::Entr(e.clone()), "len={len} pos={pos:?}");
            let got: std::collections::BTreeSet<usize> = corr.iter().map(|c| c.position).collect();
            let want: std::collections::BTreeSet<usize> = pos.iter().copied().collect();
            assert_eq!(got, want, "len={len}");
        }
    }
}
