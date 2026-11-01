//! Bug-hunt triage rows (2026-10-31 hunt, report `design/agent-reports/
//! bughunt-2026-10-31-ms.md`). Each test asserts the CORRECT behaviour, so it
//! fails on unmodified code iff the reported bug is real. Names are
//! `bughunt_ms_NN_<slug>` (NN = the report's number). Public API only.

use ms_codec::codex32::Codex32String;
use ms_codec::{
    decode, decode_with_correction, encode, encode_shares, inspect, Error, Payload, Tag, Threshold,
};
use zeroize::Zeroizing;

/// The codex32 / bech32 alphabet, symbol value = index.
const ALPHABET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

/// A valid 12-word entr string (50 chars): `ms10entrsqz4242...`.
fn valid_entr16() -> String {
    encode(Tag::ENTR, &Payload::Entr(vec![0xAA; 16])).expect("encode entr16")
}

/// Replace the char at byte index `p` of an ASCII string with the alphabet
/// symbol `shift` positions further along (cyclic).
fn shift_char(s: &str, p: usize, shift: usize) -> String {
    let mut b = s.as_bytes().to_vec();
    let i = ALPHABET
        .iter()
        .position(|&c| c == b[p])
        .expect("alphabet char");
    b[p] = ALPHABET[(i + shift) % 32];
    String::from_utf8(b).expect("ascii")
}

// ---------------------------------------------------------------------------
// #1 encode_shares(threshold 0) skips encode()'s tag checks.
// Correct: the same refusal encode() gives (ReservedTagNotEmittedInV01 or
// TagKindMismatch); the docstring promises "byte-identical to encode()".
// ---------------------------------------------------------------------------
#[test]
fn bughunt_ms_01_zero_threshold_skips_tag_checks() {
    let cases: Vec<(&str, Tag, Payload)> = vec![
        (
            "seed tag + Entr",
            Tag::try_new("seed").unwrap(),
            Payload::Entr(vec![0; 16]),
        ),
        ("hash tag + Entr", Tag::HASH, Payload::Entr(vec![0; 16])),
        (
            "entr tag + Preimage",
            Tag::ENTR,
            Payload::Preimage(Zeroizing::new([0u8; 32])),
        ),
    ];
    let mut failures: Vec<String> = Vec::new();
    for (label, tag, payload) in cases {
        // Reference: encode() refuses every one of these (precondition).
        assert!(
            encode(tag, &payload).is_err(),
            "{label}: precondition: encode() must refuse"
        );
        match encode_shares(tag, Threshold::ZERO, 1, &payload) {
            Ok(v) => failures.push(format!(
                "{label}: encode_shares(ZERO) returned Ok({} string(s) of len {}); decode() of it is_ok={}",
                v.len(),
                v[0].len(),
                decode(&v[0]).is_ok()
            )),
            Err(Error::ReservedTagNotEmittedInV01 { .. }) | Err(Error::TagKindMismatch { .. }) => {}
            Err(other) => failures.push(format!("{label}: wrong refusal: {other}")),
        }
    }
    assert!(
        failures.is_empty(),
        "encode_shares(ZERO) must refuse like encode():\n{}",
        failures.join("\n")
    );
}

// ---------------------------------------------------------------------------
// #3 parse_ms1_symbols does `c as u8`: a non-ASCII char aliases to an
// alphabet symbol, and the corrector "repairs" it. Correct: reject.
// ---------------------------------------------------------------------------
#[test]
fn bughunt_ms_03_non_ascii_read_as_symbol() {
    let good = valid_entr16();
    // Byte 12 (data part) is '2' in this string: neither '3' nor '0'.
    assert_eq!(good.as_bytes()[12], b'2', "precondition");
    let mut failures: Vec<String> = Vec::new();
    // U+0133 has low byte 0x33 = '3'; U+0130 has low byte 0x30 = '0'.
    for (label, alias) in [
        ("U+0133 (reads as '3')", '\u{133}'),
        ("U+0130 (reads as '0')", '\u{130}'),
    ] {
        let mut bad = String::new();
        bad.push_str(&good[..12]);
        bad.push(alias);
        bad.push_str(&good[13..]);
        if let Ok((_, _, d)) = decode_with_correction(&bad) {
            failures.push(format!(
                "{label} accepted by decode_with_correction and repaired: {:?}",
                d.iter()
                    .map(|c| (c.position, c.was, c.now))
                    .collect::<Vec<_>>()
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "non-ASCII chars must be rejected, not read as alphabet symbols:\n{}",
        failures.join("\n")
    );
}

// ---------------------------------------------------------------------------
// #6 Codex32String::from_string panics on a long HRP with a short data part.
// Correct: return Err (never panic). Also reachable via the public inspect().
// ---------------------------------------------------------------------------
const EVIL_LONG_HRP: &str = "laaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa12e0ds56ptqt4x";

#[test]
fn bughunt_ms_06_long_hrp_short_data_panics() {
    assert_eq!(EVIL_LONG_HRP.len(), 54, "precondition: in the 48..94 range");
    let mut failures: Vec<String> = Vec::new();
    match std::panic::catch_unwind(|| {
        Codex32String::from_string(EVIL_LONG_HRP.to_string()).is_err()
    }) {
        Ok(true) => {}
        Ok(false) => failures.push("Codex32String::from_string returned Ok".into()),
        Err(_) => failures.push("Codex32String::from_string PANICKED".into()),
    }
    match std::panic::catch_unwind(|| inspect(EVIL_LONG_HRP).is_err()) {
        Ok(true) => {}
        Ok(false) => failures.push("ms_codec::inspect returned Ok".into()),
        Err(_) => failures.push("ms_codec::inspect PANICKED".into()),
    }
    assert!(
        failures.is_empty(),
        "a 54-char string with a 40-char HRP and a 13-char data part must give Err, not panic:\n{}",
        failures.join("\n")
    );
}

// ---------------------------------------------------------------------------
// #8 TooManyErrors{bound: 8} but the decoder corrects at most 4.
// The report leaves the fix open (bound: 4 or reword): assert only that the
// message does not claim "more than 8" for a 5-error string.
// ---------------------------------------------------------------------------
#[test]
fn bughunt_ms_08_too_many_errors_bound_is_8() {
    let good = valid_entr16();
    let mut bad = good.clone();
    for p in [4usize, 9, 14, 19, 24] {
        bad = shift_char(&bad, p, 1);
    }
    let err = match decode_with_correction(&bad) {
        Err(e @ Error::TooManyErrors { .. }) => e,
        other => {
            panic!("precondition: a 5-substitution string must give TooManyErrors, got {other:?}")
        }
    };
    let msg = err.to_string();
    assert!(
        !msg.contains("more than 8"),
        "a string with 5 substitutions is reported as: {msg:?} (the decoder corrects at most 4; 5 errors is not 'more than 8')"
    );
}

// ---------------------------------------------------------------------------
// X1 (not in the report's numbered list): does the correction path accept a
// mixed-case string? BIP-93 line "a codex32 string MUST be entirely uppercase
// or entirely lowercase"; the project pins mixed-case rejection on decode and
// combine (tests/uppercase_envelope.rs U4). Asserts: a mixed-case string with
// one substitution is NOT silently repaired. See the triage report for the
// BIP-93 quotation and why the BIP text alone does not settle it.
// ---------------------------------------------------------------------------
#[test]
fn bughunt_ms_x1_mixed_case_correction() {
    let good = valid_entr16();
    // Upper-case one lowercase letter after the HRP (index 8 = the share-index
    // char 's'); the HRP stays lowercase, so the string is genuinely mixed case.
    let idx = 8usize;
    assert!(good.as_bytes()[idx].is_ascii_lowercase(), "precondition");
    let upper_at = |s: &str| -> String {
        let mut b = s.as_bytes().to_vec();
        b[idx] = b[idx].to_ascii_uppercase();
        String::from_utf8(b).unwrap()
    };

    // Precondition: mixed case on a CLEAN string is rejected by decode() and
    // by decode_with_correction() alike.
    let mixed_clean = upper_at(&good);
    assert_ne!(mixed_clean, good, "precondition: actually mixed");
    assert!(
        decode(&mixed_clean).is_err(),
        "precondition: decode() rejects mixed case"
    );
    assert!(
        decode_with_correction(&mixed_clean).is_err(),
        "decode_with_correction must reject a CLEAN mixed-case string"
    );

    // The case under test: mixed case + one real substitution elsewhere.
    let mixed_bad = upper_at(&shift_char(&good, 30, 1));
    let r = decode_with_correction(&mixed_bad);
    assert!(
        r.is_err(),
        "a mixed-case string with one substitution was accepted and repaired: {:?}",
        r.as_ref().map(|(_, _, d)| d
            .iter()
            .map(|c| (c.position, c.was, c.now))
            .collect::<Vec<_>>())
    );
}
