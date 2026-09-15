//! The per-kind known-answer test (spec §10, §12 item 6).
//!
//! WHY THIS FILE EXISTS. Core's address vectors derive from the descriptor TEXT
//! and are structurally blind to a wrong digest function; `ms hashlock` checked
//! against this crate is self-consistency, not a KAT. Without these rows, a
//! `digest_hash256` written one word short passes every other gate in the cycle
//! and locks funds to a preimage that does not exist.
//!
//! IT COVERS THE DISPATCH TOO. Four correct functions plus one mis-wired arm of
//! `HashKind::digest` is the same failure with a different cause.

use ms_codec::hashlock::{
    digest_hash160, digest_hash256, digest_ripemd160, digest_sha256, HashKind,
};

const CORPUS: &str = include_str!("vectors/hashlock-v0.8.json");

fn hex(b: &[u8]) -> String {
    // `fold`, not `map(format!).collect()` -- clippy's format_collect fires
    // under `-D warnings`, which is a required CI context.
    b.iter()
        .fold(String::with_capacity(b.len() * 2), |mut acc, x| {
            use core::fmt::Write as _;
            let _ = write!(acc, "{x:02x}");
            acc
        })
}

fn hex32(s: &str) -> [u8; 32] {
    let v: Vec<u8> = (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("hex"))
        .collect();
    v.try_into().expect("32 bytes")
}

#[test]
fn every_row_pins_all_four_kinds_and_the_dispatch() {
    let v: serde_json::Value = serde_json::from_str(CORPUS).expect("corpus parses");
    let rows = v["derivation"].as_array().expect("derivation rows");
    let mut checked = 0usize;
    for r in rows {
        for stem in ["hardened", "sha256"] {
            let Some(xh) = r[format!("{stem}_x")].as_str() else {
                continue;
            };
            let x = hex32(xh);
            let want = |k: &str| -> String {
                r[format!("{stem}_h{k}")]
                    .as_str()
                    .unwrap_or_else(|| panic!("row missing {stem}_h{k}"))
                    .to_string()
            };
            let phrase = r["phrase"].as_str().unwrap_or("<no phrase>");

            assert_eq!(hex(&digest_sha256(&x)), want(""), "{phrase}/{stem}: sha256");
            assert_eq!(
                hex(&digest_hash256(&x)),
                want("_hash256"),
                "{phrase}/{stem}: hash256"
            );
            assert_eq!(
                hex(&digest_ripemd160(&x)),
                want("_ripemd160"),
                "{phrase}/{stem}: ripemd160"
            );
            assert_eq!(
                hex(&digest_hash160(&x)),
                want("_hash160"),
                "{phrase}/{stem}: hash160"
            );

            // THE DISPATCH, over the same rows.
            for (kind, suffix) in [
                (HashKind::Sha256, ""),
                (HashKind::Hash256, "_hash256"),
                (HashKind::Ripemd160, "_ripemd160"),
                (HashKind::Hash160, "_hash160"),
            ] {
                assert_eq!(
                    hex(kind.digest(&x).as_slice()),
                    want(suffix),
                    "{phrase}/{stem}: dispatch for {}",
                    kind.token()
                );
            }
            checked += 1;
        }
    }
    // A loop that silently iterated zero rows would pass.
    assert!(
        checked >= 8,
        "only {checked} row/stem pairs carried all four kinds"
    );
}
