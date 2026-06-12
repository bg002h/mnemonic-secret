//! Fuzz target: `combine_shares` (K-of-N codex32 Shamir recombination).
//!
//! ms phase of the constellation stress-fuzz program (Cycle C).
//!
//! Structured multi-share input uses a SENTINEL-BYTE splitter (R0 [M2]): split
//! the fuzz input on `\n` (0x0A, outside the bech32 alphabet) into 2..=8 parts
//! (truncate excess). A libFuzzer insert/delete then moves ONE share boundary
//! locally instead of re-shearing all of them. `combine_shares` takes
//! `&[String]` (shares.rs:186), so the parts are owned `String`s.
//!
//! Oracles:
//! 1. Never-panic / clean-error (implicit).
//! 2. Decode → re-encode fixed-point (R0 [Q5]/[I6]): on `Ok((tag, p))`,
//!    `encode(tag, &p)` then `decode` and assert the recovered `(Tag, Payload)`
//!    is equal. `combine_shares` always returns `Tag::ENTR` with the recovered
//!    payload (shares.rs:182-185); the encoder's reserved-tag symmetry
//!    (encode.rs:16-25) makes that re-encodable. A re-encode `Err` on a
//!    combine-accepted value is a REAL FINDING (R0 [I6]).
#![no_main]

use libfuzzer_sys::fuzz_target;
use ms_codec::{combine_shares, decode};

/// Maximum share count accepted by the splitter; extra parts are dropped.
const MAX_PARTS: usize = 8;

fuzz_target!(|data: &[u8]| {
    // Sentinel split on `\n`; cap at MAX_PARTS by truncation. `combine_shares`
    // requires at least the share-set threshold (>=2) of shares, so anything
    // below 2 parts cannot combine — but we still let it through to exercise
    // the below-threshold rejection path (a clean error, not a finding).
    let parts: Vec<String> = data
        .split(|&b| b == b'\n')
        .take(MAX_PARTS)
        .map(|chunk| String::from_utf8_lossy(chunk).into_owned())
        .collect();

    if let Ok((tag, payload)) = combine_shares(&parts) {
        // Re-encode the recovered secret. Err here = finding.
        let reencoded = ms_codec::encode(tag, &payload)
            .expect("FINDING: combine-accepted (tag, payload) failed to re-encode");
        let (tag2, payload2) =
            decode(&reencoded).expect("FINDING: re-encoded combined secret failed to decode");
        assert_eq!(
            (tag, &payload),
            (tag2, &payload2),
            "FINDING: combine/re-encode/decode is not a fixed point"
        );
    }
});
