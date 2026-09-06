//! SPEC_hashlock_H6 §8.6: the QR text a hashlock PHRASE plate carries, pinned
//! by corpus rows BEFORE a parser for it exists.
//!
//! `ms hashlock` learning to READ this text is a follow-on (H6 §13). The text
//! is fixed here so the plate and the future parser cannot be designed against
//! two different strings, and so the fork's plate builder has something to
//! assert against that is not a literal it transcribed itself.

use ms_codec::hashlock::qr_text;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Corpus {
    qr_text: Vec<Row>,
}

#[derive(Debug, Deserialize)]
struct Row {
    name: String,
    method: String,
    phrase: String,
    qr_text: String,
    bytes: usize,
}

fn corpus() -> Corpus {
    let raw = include_str!("vectors/hashlock-v0.8.json");
    serde_json::from_str(raw).expect("the hashlock corpus parses")
}

/// Every row's `qr_text` is what `qr_text()` builds, byte for byte.
///
/// MUTATION: emit a trailing newline -> every row fails.
/// MUTATION: put the `phrase:` line before the `method:` line -> every row
/// fails.
/// MUTATION: render the salt from a literal instead of `HASHLOCK_SALT` -> the
/// hardened rows still pass TODAY and fail the moment the salt moves, which is
/// the property the parameter-reading rule exists for; the
/// `parameters_come_from_the_constants` test below is the one that fails now.
#[test]
fn qr_text_matches_every_corpus_row() {
    let c = corpus();
    assert!(c.qr_text.len() >= 7, "the corpus lost its qr_text rows");
    for row in &c.qr_text {
        let hardened = match row.method.as_str() {
            "hardened" => true,
            "sha256" => false,
            other => panic!("row {}: unknown method {other}", row.name),
        };
        // `qr_text` returns `Zeroizing<String>` (the phrase is in it), so the
        // comparison derefs to the `String` inside. Every other assertion below
        // reaches `str`'s inherent methods through the same `Deref` and needed
        // no change.
        let got = qr_text(hardened, &row.phrase);
        assert_eq!(*got, row.qr_text, "row {}", row.name);
        assert_eq!(got.len(), row.bytes, "row {}: byte count", row.name);
        assert!(
            !got.ends_with('\n'),
            "row {}: the text has a trailing newline",
            row.name
        );
        assert_eq!(
            got.lines().count(),
            3,
            "row {}: the text is three LF-separated lines",
            row.name
        );
        let last = got.lines().next_back().unwrap();
        assert_eq!(
            last.strip_prefix("phrase: ").unwrap(),
            row.phrase,
            "row {}: the phrase is the LAST line, verbatim",
            row.name
        );
    }
}

/// The method line reads its parameters from the CONSTANTS, so a parameter
/// change cannot leave a plate lying about how to reproduce the derivation.
///
/// MUTATION: hard-code `iterations=100000` in `qr_text` -> this test still
/// passes today and the assertion below on the rendered numbers is what makes
/// the coupling visible; MUTATION: change `HASHLOCK_ITERATIONS` without
/// changing the corpus -> `qr_text_matches_every_corpus_row` fails, which is
/// the intended coupling.
#[test]
fn parameters_come_from_the_constants() {
    let t = qr_text(true, "x");
    let method = t.lines().nth(1).unwrap();
    assert!(
        method.contains(&format!(
            "iterations={}",
            ms_codec::hashlock::HASHLOCK_ITERATIONS
        )),
        "{method}"
    );
    assert!(
        method.contains(&format!(
            "salt={}",
            std::str::from_utf8(ms_codec::hashlock::HASHLOCK_SALT).unwrap()
        )),
        "{method}"
    );
    assert!(
        method.contains(&format!("dklen={}", ms_codec::hashlock::HASHLOCK_DKLEN)),
        "{method}"
    );
    assert_eq!(
        method.len(),
        73,
        "the hardened method line is 73 characters; H6 §6.5 pins the plate's \
         worst case on it and a 79th character puts the phrase plate over budget"
    );
    assert_eq!(
        qr_text(false, "x").lines().nth(1).unwrap(),
        "method: sha256"
    );
}

/// The 100-character cap produces the 194-byte worst case the plate and the
/// QR-version raise are both sized for.
#[test]
fn the_worst_case_is_194_bytes() {
    let t = qr_text(true, &"0".repeat(100));
    assert_eq!(t.len(), 194);
    assert_eq!(qr_text(false, &"0".repeat(100)).len(), 135);
}
