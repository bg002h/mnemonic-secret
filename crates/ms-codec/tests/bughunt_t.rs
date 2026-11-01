//! Bug-hunt triage rows, round 2 (three suspicions from the grading report's
//! "Not graded" section, `design/agent-reports/bughunt-grade-ms.md`). Each test
//! asserts the CORRECT behaviour, so it fails on unmodified code iff the
//! suspicion is real. Names are `bughunt_ms_tN_<slug>`. Public API only.

use ms_codec::{decode_with_correction, encode, Payload, Tag};

/// A valid 12-word entr string (50 chars): `ms10entrsqz4242...`.
fn valid_entr16() -> String {
    encode(Tag::ENTR, &Payload::Entr(vec![0xAA; 16])).expect("encode entr16")
}

/// `s` with the characters at the given byte indices replaced by `?`.
fn with_erasures(s: &str, at: &[usize]) -> String {
    let mut b = s.as_bytes().to_vec();
    for &p in at {
        b[p] = b'?';
    }
    String::from_utf8(b).expect("ascii")
}

// ---------------------------------------------------------------------------
// T1 BIP-93 line 158: "Implementations interpret "?" as an erasure." A valid
// ms1 string with ONE character replaced by `?` must be recovered to the
// original string by `decode_with_correction`. Every data-part position
// (indices 3..50, the payload and the checksum) is tried.
// ---------------------------------------------------------------------------
#[test]
fn bughunt_ms_t1_question_mark_erasure() {
    let original = valid_entr16();
    assert_eq!(
        original.len(),
        50,
        "precondition: a valid 50-char entr string"
    );
    assert!(
        decode_with_correction(&original).is_ok(),
        "precondition: the undamaged string decodes"
    );

    let mut failures: Vec<String> = Vec::new();
    for p in 3..original.len() {
        let damaged = with_erasures(&original, &[p]);
        match decode_with_correction(&damaged) {
            Ok((tag, payload, _)) => {
                let back = encode(tag, &payload).expect("re-encode");
                if back != original {
                    failures.push(format!("index {p}: recovered a DIFFERENT string {back}"));
                }
            }
            Err(e) => failures.push(format!("index {p}: refused: {e}")),
        }
    }
    assert!(
        failures.is_empty(),
        "one `?` (an erasure, BIP-93 line 158) was not recovered at {} of {} positions; first: {}",
        failures.len(),
        original.len() - 3,
        failures[0]
    );
}

// ---------------------------------------------------------------------------
// T1b The same BIP-93 sentence at its stated strength (line 152: "up to 8
// unreadable characters (called erasures)"; the toolkit manual's BCH primer
// repeats it). Eight `?` at scattered positions must be recovered. Checked
// independently: a GF(2) linear solve over the BIP-93 reference polymod
// recovers these positions uniquely (oracle in /scratch/cache/bughunt/ms2/).
// ---------------------------------------------------------------------------
#[test]
fn bughunt_ms_t1b_eight_erasures_recovered() {
    let original = valid_entr16();
    let damaged = with_erasures(&original, &[3, 9, 15, 21, 27, 33, 39, 45]);
    assert_eq!(
        damaged.matches('?').count(),
        8,
        "precondition: eight erasures"
    );
    match decode_with_correction(&damaged) {
        Ok((tag, payload, _)) => {
            let back = encode(tag, &payload).expect("re-encode");
            assert_eq!(
                back, original,
                "eight erasures recovered to a different string"
            );
        }
        Err(e) => panic!("eight `?` erasures (within BIP-93's stated 8) were refused: {e}"),
    }
}
