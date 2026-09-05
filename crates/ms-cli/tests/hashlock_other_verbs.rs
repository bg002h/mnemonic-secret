//! The other verbs on the preimage kind (SPEC_ms_hashlock §5), and the
//! structural pins that keep the next kind from re-opening the
//! `#[non_exhaustive]` hazard (§3).

use assert_cmd::Command;

const HEX32: &str = "c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016";
const H: &str = "3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12";

fn ms() -> Command {
    Command::cargo_bin("ms").unwrap()
}

fn plate() -> String {
    let out = ms()
        .args(["hashlock", "--hex", "-", "--json"])
        .write_stdin(HEX32)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    v["preimage_ms1"].as_str().unwrap().to_string()
}

/// MUTATION: leave decode.rs's catch-all as `unreachable!` -> exit 101.
#[test]
fn decode_prints_kind_hex_and_digest_and_never_words() {
    let out = ms()
        .args(["decode", "-"])
        .write_stdin(plate())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let so = String::from_utf8_lossy(&out.stdout);
    // STRUCTURAL, not a word blocklist (R0 r0 tests I-3): exactly three
    // labelled lines, so any extra line -- words, a phrase, anything -- fails.
    let lines: Vec<&str> = so.lines().collect();
    assert_eq!(
        lines.len(),
        3,
        "decode's text output for a preimage is exactly three lines:\n{so}"
    );
    assert!(
        lines[0].starts_with("kind:") && lines[0].contains("preimage"),
        "{so}"
    );
    assert!(
        lines[1].starts_with("preimage:") && lines[1].contains(HEX32),
        "{so}"
    );
    assert!(
        lines[2].starts_with("digest:") && lines[2].contains(H),
        "{so}"
    );
}

#[test]
fn decode_json_carries_kind_and_digest() {
    let out = ms()
        .args(["decode", "-", "--json"])
        .write_stdin(plate())
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["kind"], "preimage");
    assert_eq!(v["preimage_hex"], HEX32);
    assert_eq!(v["digest"], H);
    assert_eq!(
        v.as_object().unwrap().len(),
        3,
        "exactly kind, preimage_hex, digest: {v}"
    );
}

/// MUTATION: leave inspect.rs's rule-6/8 copies untouched -> `unknown-tag`
/// and `non-zero-prefix` fire on a valid preimage single.
#[test]
fn inspect_reports_the_kind_with_no_false_reason() {
    let out = ms()
        .args(["inspect", "-"])
        .write_stdin(plate())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let so = String::from_utf8_lossy(&out.stdout);
    assert!(so.contains("preimage"), "{so}");
    assert!(
        !so.contains("unknown-tag") && !so.contains("non-zero-prefix"),
        "{so}"
    );
    assert!(!so.contains("would NOT decode"), "{so}");
}

/// MUTATION: place the refusal AFTER `payload_entropy_and_language` -> exit
/// 101 from the helper's `unreachable!`.
#[test]
fn derive_and_verify_refuse_with_the_executable_remedy() {
    for verb in ["derive", "verify"] {
        let out = ms()
            .args([verb, "-"])
            .write_stdin(plate())
            .output()
            .unwrap();
        assert_ne!(out.status.code(), Some(101), "{verb} panicked");
        assert!(!out.status.success(), "{verb} must refuse a preimage");
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(
            err.contains("ms hashlock"),
            "{verb}: the remedy must be executable:\n{err}"
        );
        assert!(!err.contains(HEX32), "{verb} echoed the preimage:\n{err}");
    }
}

/// MUTATION: leave combine.rs's catch-all -> exit 101 on a preimage share set.
#[test]
fn combine_prints_a_recovered_preimage_as_decode_does() {
    // Shares are made through the codec (the CLI has no ms1 source for split:
    // F-468), then combined through the CLI.
    use ms_codec::{encode_shares, Payload, Tag, Threshold};
    let mut x = [0u8; 32];
    for (i, b) in x.iter_mut().enumerate() {
        *b = u8::from_str_radix(&HEX32[2 * i..2 * i + 2], 16).unwrap();
    }
    let shares = encode_shares(
        Tag::HASH,
        Threshold::new(2).unwrap(),
        2,
        &Payload::Preimage(zeroize::Zeroizing::new(x)),
    )
    .unwrap();
    let out = ms()
        .args(["combine", "-"])
        .write_stdin(shares.join("\n"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let so = String::from_utf8_lossy(&out.stdout);
    assert!(so.contains(HEX32) && so.contains(H), "{so}");
}

/// C-1 (R0 r0 fidelity): a forged single whose id and prefix disagree is a
/// FormatViolation (exit 2) with the spec's wording, on decode and on inspect --
/// never "unhandled ms_codec::Error variant" at exit 1, never "would decode".
/// MUTATION: delete the TagKindMismatch arm in From<ms_codec::Error>.
#[test]
fn tag_kind_mismatch_is_a_format_violation_on_decode_and_a_reason_on_inspect() {
    use ms_codec::codex32::{Codex32String, Fe};
    let mut seed = vec![0x00u8];
    seed.extend_from_slice(&[0xab; 32]);
    let forged = Codex32String::from_seed("ms", 0, "hash", Fe::S, &seed)
        .unwrap()
        .to_string();
    let out = ms()
        .args(["decode", "-"])
        .write_stdin(forged.clone())
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("names a different kind than the prefix byte"),
        "{err}"
    );
    assert!(!err.contains("unhandled"), "{err}");
    let out = ms()
        .args(["inspect", "-"])
        .write_stdin(forged)
        .output()
        .unwrap();
    let so = String::from_utf8_lossy(&out.stdout);
    assert!(
        so.contains("tag-kind-mismatch") && so.contains("would NOT decode"),
        "{so}"
    );
}

/// I-3 (R0 r0 fidelity): split.rs's PayloadKind catch-all was swept too.
#[test]
fn split_kind_match_has_a_preimage_arm() {
    let s = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/cmd/split.rs"),
    )
    .unwrap();
    assert!(
        s.contains("PayloadKind::Preimage =>"),
        "split.rs's kind match must name the preimage kind"
    );
}

/// repair is unchanged and benign on the kind (adversarial M-3).
#[test]
fn repair_on_an_undamaged_preimage_plate_is_a_no_op() {
    let p = plate();
    let out = ms()
        .args(["repair", "--ms1", "-"])
        .write_stdin(p.clone())
        .output()
        .unwrap();
    assert_ne!(out.status.code(), Some(101));
}

/// The committed catch-all count (§3): the next kind re-triggers the sweep
/// mechanically. MUTATION: add a fifth `_ => unreachable!` -> this fails.
#[test]
fn unreachable_catch_all_count_is_pinned() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut n = 0;
    fn walk(p: &std::path::Path, n: &mut usize) {
        for e in std::fs::read_dir(p).unwrap() {
            let e = e.unwrap();
            if e.path().is_dir() {
                walk(&e.path(), n);
            } else if e.path().extension().map(|x| x == "rs").unwrap_or(false) {
                *n += std::fs::read_to_string(e.path())
                    .unwrap()
                    .matches("_ => unreachable!")
                    .count();
            }
        }
    }
    walk(&root, &mut n);
    assert_eq!(n, 4, "the ms-cli `_ => unreachable!` census moved: every catch-all over Payload/PayloadKind/InspectKind needs a Preimage arm before this number changes");
}

/// The SECRET_FLAGS doc comment was corrected while the line was edited (tests N-4).
#[test]
fn secret_flags_doc_comment_counts_five() {
    let s = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/argv_guard.rs"),
    )
    .unwrap();
    assert!(
        !s.contains("The nine flag-keyed"),
        "stale doc comment above SECRET_FLAGS"
    );
    assert!(s.contains("const SECRET_FLAGS: [&str; 5]"));
}
