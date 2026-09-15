//! SPEC_hashlock_H6 §8.6: the QR text a hashlock PHRASE plate carries, pinned
//! by corpus rows BEFORE a parser for it exists.
//!
//! `ms hashlock` learning to READ this text is a follow-on (H6 §13). The text
//! is fixed here so the plate and the future parser cannot be designed against
//! two different strings, and so the fork's plate builder has something to
//! assert against that is not a literal it transcribed itself.

use ms_codec::hashlock::{qr_text, HashKind};
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
    /// Which hashlock KIND the row pins. A different axis from `method`, which
    /// selects the PREIMAGE derivation -- the two share the token `sha256` and
    /// mean different things (SPEC_hashlock_kinds §5).
    kind: String,
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
    // AND EVERY KIND IS PRESENT. A floor alone is not coverage: with `>= 7` and
    // ten rows, all three per-kind rows could be deleted and this suite stayed
    // green -- measured. Phase 4 keys its Go port on these rows, so their
    // absence must be loud.
    for kind in ["sha256", "hash256", "ripemd160", "hash160"] {
        assert!(
            c.qr_text.iter().any(|r| r.kind == kind),
            "the corpus carries no qr_text row for kind {kind}"
        );
    }
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
        let kind = match row.kind.as_str() {
            "sha256" => HashKind::Sha256,
            "hash256" => HashKind::Hash256,
            "ripemd160" => HashKind::Ripemd160,
            "hash160" => HashKind::Hash160,
            other => panic!("row {}: unknown kind {other}", row.name),
        };
        let got = qr_text(hardened, kind, &row.phrase);
        assert_eq!(*got, row.qr_text, "row {}", row.name);
        assert_eq!(got.len(), row.bytes, "row {}: byte count", row.name);
        assert!(
            !got.ends_with('\n'),
            "row {}: the text has a trailing newline",
            row.name
        );
        assert_eq!(
            got.lines().count(),
            4,
            "row {}: the text is four LF-separated lines (head, method, hash, phrase)",
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

/// Spec §13.1: the plate is read years later by someone with neither the tool
/// nor this firmware, so it spells its parameters out. Without the kind it is
/// one step short, and the device tells the operator to store this plate APART
/// from the md1 card that holds the missing step.
///
/// THE TWO AXES, in one assertion each. `hash:` is WHICH HASH THE SCRIPT
/// COMMITS TO; `method:` is HOW THE PREIMAGE WAS DERIVED. They share the token
/// `sha256` and mean different things, and every defect this cycle produced
/// came from reading one as the other -- so this test pins that adding the
/// first did not disturb the second.
#[test]
fn qr_text_names_the_kind_on_its_own_line() {
    let t = qr_text(true, HashKind::Ripemd160, "correct horse battery staple");
    assert!(
        t.contains("\nhash: ripemd160\n"),
        "the kind is not on its own line:\n{}",
        &*t
    );
    assert!(
        t.contains("method: pbkdf2-hmac-sha256"),
        "the METHOD line must survive unchanged -- it is a different axis:\n{}",
        &*t
    );
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
    let t = qr_text(true, HashKind::Sha256, "x");
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
        qr_text(false, HashKind::Sha256, "x")
            .lines()
            .nth(1)
            .unwrap(),
        "method: sha256"
    );
}

/// The 100-character cap produces the 210-byte worst case the plate and the
/// QR-version raise are both sized for.
///
/// RE-KEYED ON `ripemd160` (F-507 / SPEC_hashlock_kinds §13.1). `ripemd160` is
/// the longest of the four kind tokens -- nine characters against seven, seven
/// and six -- so once the kind is on the plate, the sha256 case is no longer the
/// worst case and a test named for the worst case must track the real one. The
/// sha256 rows are kept beside it because they are the common case, not because
/// they bound anything.
#[test]
fn the_worst_case_is_210_bytes() {
    let t = qr_text(true, HashKind::Ripemd160, &"0".repeat(100));
    assert_eq!(t.len(), 210, "hardened + ripemd160 is the true worst case");
    assert_eq!(
        qr_text(false, HashKind::Ripemd160, &"0".repeat(100)).len(),
        151
    );
    // The common case, for reference; 194 and 135 before the kind line.
    assert_eq!(qr_text(true, HashKind::Sha256, &"0".repeat(100)).len(), 207);
    assert_eq!(
        qr_text(false, HashKind::Sha256, &"0".repeat(100)).len(),
        148
    );
}
