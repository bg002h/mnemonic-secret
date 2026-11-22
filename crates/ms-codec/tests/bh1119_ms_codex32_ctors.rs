//! FIX_bughunt1119_ms_codex32_ctors (bh-ms-I2; bh-ms-I3 graded M by T24).
//! Every test asserts the behaviour the FIX document specifies.

use ms_codec::codex32::{Codex32String as C, Error, Fe};

/// The all-zero 12-word signet test card (BIP-39 entropy 16 x 0x00).
const CARD: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
/// BIP-93 test vector 5 (long code, 15-character checksum).
const LONG: &str = "MS100C8VSM32ZXFGUHPCHTLUPZRY9X8GF2TVDW0S3JN54KHCE6MUA7LQPZYGSFJD6AN074RXVCEMLH8WU3TK925ACDEFGHJKLMNPQRSTUVWXY06FHPV80UNDVARHRAK";

#[test]
fn i2_lowercase_body_completes_to_card() {
    let built = C::from_unchecksummed_string(CARD[..CARD.len() - 13].into()).unwrap();
    assert_eq!(built.to_string(), CARD);
}

#[test]
fn i2_uppercase_body_completes_to_uppercase_card() {
    let up = CARD.to_ascii_uppercase();
    let built = C::from_unchecksummed_string(up[..up.len() - 13].into()).unwrap();
    assert_eq!(built.to_string(), up);
}

#[test]
fn i2_long_body_completes_to_bip93_vector_5() {
    let built = C::from_unchecksummed_string(LONG[..LONG.len() - 15].into()).unwrap();
    assert_eq!(built.to_string(), LONG);
    let low = LONG.to_ascii_lowercase();
    let built = C::from_unchecksummed_string(low[..low.len() - 15].into()).unwrap();
    assert_eq!(built.to_string(), low);
}

#[test]
fn i2_completed_length_below_short_bracket_is_refused() {
    // 30-char body + 13 = 43, below the short bracket (48..=93).
    let body = format!("ms10entrs{}", "q".repeat(21));
    assert_eq!(body.len(), 30);
    let r = C::from_unchecksummed_string(body);
    assert!(matches!(r, Err(Error::InvalidLength(43))), "30-char body: {r:?}");
}

#[test]
fn i2_completed_length_between_brackets_is_refused() {
    // 91-char body + 15 = 106, between the short (48..=93) and long (125..=127) brackets.
    let body = format!("ms10entrs{}", "q".repeat(82));
    assert_eq!(body.len(), 91);
    let r = C::from_unchecksummed_string(body);
    assert!(matches!(r, Err(Error::InvalidLength(106))), "91-char body: {r:?}");
}

#[test]
fn i2_controls_unchanged_refusals() {
    let mixed = format!("ms10entrS{}", "q".repeat(28));
    assert!(matches!(C::from_unchecksummed_string(mixed), Err(Error::InvalidCase(..))));
    let badchar = format!("ms10entrsb{}", "q".repeat(27));
    assert!(C::from_unchecksummed_string(badchar).is_err());
    let idx = format!("ms10entra{}", "q".repeat(28));
    assert!(matches!(C::from_unchecksummed_string(idx), Err(Error::InvalidShareIndex(Fe::A))));
}

#[test]
fn i3_from_seed_unchecked_builder_contract() {
    let a = C::from_seed("ms", 0, "test", Fe::A, &[0x42; 16]).unwrap();
    assert!(matches!(C::from_string(a.to_string()), Err(Error::InvalidShareIndex(Fe::A))));
    let b = C::from_seed("ms", 0, "test", Fe::S, &[0x42; 45]).unwrap();
    assert!(matches!(C::from_string(b.to_string()), Err(Error::InvalidLength(94))));
    let c = C::from_seed("MS", 0, "TEST", Fe::S, &[0x42; 16]);
    assert!(matches!(c, Err(Error::InvalidCase(..))), "uppercase hrp: {c:?}");
    for n in [16usize, 20, 24, 28, 32, 64] {
        let ok = C::from_seed("ms", 0, "test", Fe::S, &vec![0x42; n]).unwrap();
        assert!(C::from_string(ok.to_string()).is_ok(), "{n}");
    }
}

#[test]
fn i2_bracket_edges() {
    let body = |n: usize| format!("ms10entrs{}", "q".repeat(n - 9));
    for (n, ok) in [(35, true), (80, true), (110, true), (112, true)] {
        let r = C::from_unchecksummed_string(body(n)).map(|c| c.to_string());
        assert!(ok && r.as_ref().is_ok_and(|s| C::from_string(s.clone()).is_ok()), "{n}: {r:?}");
    }
    for (n, len) in [(34usize, 47usize), (81, 96), (109, 124), (113, 128)] {
        let r = C::from_unchecksummed_string(body(n));
        assert!(matches!(r, Err(Error::InvalidLength(l)) if l == len), "{n}: {r:?}");
    }
}
