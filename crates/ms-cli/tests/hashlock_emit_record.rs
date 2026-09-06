//! `--emit-record` (F-495): the `phrase:` record that lets a PHRASE-form plate
//! be cut from a host-derived phrase without hand-encoding hex.
//!
//! THE WIRE FORM BELONGS TO ANOTHER REPO. `phrase:` + hex of `<method>,<phrase>`
//! is `mnemonic_engrave::sysw::composer_records::phrase_record`, and `me sysw
//! pack --pack-preimage` is the only consumer. This verb composes the same two
//! fields, so the two spellings can drift — and the literals below are the
//! defence: they are copied BYTE FOR BYTE from that repo's committed corpus
//! `crates/me-cli/testdata/record_class_vectors.json`, rows `phrase-hardened`
//! and `phrase-sha256`, whose own suite asserts they classify as `Phrase`. If
//! either side changes the encoding, this file goes red.

use assert_cmd::Command;

const PHRASE: &str = "correct horse battery staple";

/// `record_class_vectors.json` row `phrase-hardened` — hex of
/// `hardened,correct horse battery staple`.
const REC_HARDENED: &str =
    "phrase:68617264656e65642c636f727265637420686f727365206261747465727920737461706c65";
/// Row `phrase-sha256` — hex of `sha256,correct horse battery staple`.
const REC_SHA256: &str =
    "phrase:7368613235362c636f727265637420686f727365206261747465727920737461706c65";

fn ms() -> Command {
    Command::cargo_bin("ms").unwrap()
}

/// The record is byte-identical to the consumer's corpus, in both methods.
///
/// MUTATION: emit `<phrase>,<method>` instead of `<method>,<phrase>`, or drop
/// the comma, or uppercase the hex -> neither literal matches and this fails.
#[test]
fn the_record_is_the_consumers_corpus_row_byte_for_byte() {
    for (method, want) in [("hardened", REC_HARDENED), ("sha256", REC_SHA256)] {
        let out = ms()
            .args([
                "hashlock",
                "--hashlock-phrase-stdin",
                "--method",
                method,
                "--emit-record",
            ])
            .write_stdin(PHRASE)
            .output()
            .unwrap();
        assert!(out.status.success(), "{method}: {out:?}");
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(
            err.contains(&format!("record (phrase): {want}")),
            "{method}: the card's record line is not the corpus row:\n{err}"
        );
    }
}

/// THE RECORD IS SECRET, so it never reaches stdout — the card says stdout
/// carries only the public digest, and that claim has to hold under the flag
/// that adds a phrase-bearing line.
///
/// MUTATION: write the record to stdout instead of the card -> stdout is two
/// lines and this fails.
#[test]
fn stdout_stays_the_public_digest_alone() {
    let out = ms()
        .args(["hashlock", "--hashlock-phrase-stdin", "--emit-record"])
        .write_stdin(PHRASE)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout, "hash:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12\n",
        "stdout carried more than the digest"
    );
    assert!(
        !stdout.contains("phrase:"),
        "the phrase record reached stdout: {stdout}"
    );
}

/// Without the flag nothing changes: no record line, and no phrase-bearing text
/// on the card beyond the character count that was always there.
///
/// MUTATION: retain and print the record unconditionally -> this fails.
#[test]
fn the_flag_is_what_adds_the_record() {
    let out = ms()
        .args(["hashlock", "--hashlock-phrase-stdin"])
        .write_stdin(PHRASE)
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        !err.contains("record (phrase):") && !err.contains(REC_HARDENED),
        "the record appeared without the flag:\n{err}"
    );
}

/// A source with no phrase is a usage error, not a silent omission: a flag that
/// asks for a record and produces none sends the operator looking for output
/// that was never coming.
///
/// MUTATION: ignore the flag instead of refusing -> both cases exit 0 and this
/// fails.
#[test]
fn a_source_without_a_phrase_refuses_the_flag() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("x.txt");
    let ms1_path = dir.path().join("plate.ms1");
    std::fs::write(
        &ms1_path,
        "ms10hashsq0p7jaf9gsjjpkjvll2l274w8a388xgqzlewp73scptwxgtjugspvs8tklufg89hqj\n",
    )
    .unwrap();
    for args in [
        vec![
            "hashlock",
            "--random",
            "--out",
            p.to_str().unwrap(),
            "--emit-record",
        ],
        // An ms1 plate, read through the private channel. NOT `--hex` on
        // argv: the argv guard refuses raw entropy there before the command
        // line is parsed, so that spelling would test the guard rather than
        // this flag (measured — the refusal never runs).
        vec![
            "hashlock",
            "--in",
            ms1_path.to_str().unwrap(),
            "--emit-record",
        ],
    ] {
        let out = ms().args(&args).output().unwrap();
        assert!(!out.status.success(), "{args:?} succeeded");
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(
            err.contains("--emit-record needs a phrase"),
            "{args:?}: not the usage refusal:\n{err}"
        );
    }
}

/// `--json` carries it too, in the object that already announces it holds the
/// secret — and only under the flag.
#[test]
fn json_carries_the_record_only_under_the_flag() {
    let out = ms()
        .args([
            "hashlock",
            "--hashlock-phrase-stdin",
            "--json",
            "--emit-record",
        ])
        .write_stdin(PHRASE)
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["phrase_record"], REC_HARDENED);

    let out = ms()
        .args(["hashlock", "--hashlock-phrase-stdin", "--json"])
        .write_stdin(PHRASE)
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(v.get("phrase_record").is_none(), "{v}");
}
