//! bh-ms-I12 (bughunt 2026-11-19): `ms inspect` must not say a share would
//! combine when its string length is not the length of any ms1 secret.
//! Material rides stdin (`-`), never argv, as the argv guard requires.

use assert_cmd::Command;
use ms_codec::codex32::{Codex32String, Fe};
use serde_json::Value;
use std::process::Output;

/// Every string length an ms1 secret can have (entr, mnem, preimage), written
/// out here rather than taken from `ms_codec::consts`.
const SECRET_LENGTHS: [usize; 10] = [50, 51, 56, 58, 62, 64, 69, 70, 75, 77];

fn ms(args: &[&str], stdin: &str) -> Output {
    Command::cargo_bin("ms")
        .unwrap()
        .args(args)
        .write_stdin(stdin.to_string())
        .output()
        .unwrap()
}
fn out_s(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}
fn err_s(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

/// A 2-of-n share with id `test` carrying `n` zero data bytes.
fn share(n: usize, idx: Fe) -> String {
    Codex32String::from_seed("ms", 2, "test", idx, &vec![0u8; n])
        .unwrap()
        .to_string()
}

const BAD_LEN_TEXT_A: &str = "FAIL: would NOT combine\n    reason: unexpected-string-length (string length not in valid set for this kind ([50,56,62,69,75] entr / [51,58,64,70,77] mnem / [75] preimage))\n\nhrp: ms\nthreshold: 2\nid: test\nindex: a\nchecksum_valid: true\nkind: share\n";
const BAD_LEN_JSON_A: &str = "{\"failure_reasons\":[\"unexpected-string-length\"],\"report\":{\"checksum_valid\":true,\"hrp\":\"ms\",\"kind\":\"share\",\"share_index\":\"a\",\"tag\":\"test\",\"threshold\":2},\"schema_version\":\"1\",\"would_combine\":false,\"would_decode\":false}\n";
const BAD_LEN_TEXT_S: &str = "FAIL: would NOT combine\n    reason: share-index-secret (share-index 's' is the secret itself, not a distributed share; ms combine refuses it)\n    reason: unexpected-string-length (string length not in valid set for this kind ([50,56,62,69,75] entr / [51,58,64,70,77] mnem / [75] preimage))\n\nhrp: ms\nthreshold: 2\nid: test\nindex: s\nchecksum_valid: true\nkind: share\n";
const BAD_LEN_JSON_S: &str = "{\"failure_reasons\":[\"share-index-secret\",\"unexpected-string-length\"],\"report\":{\"checksum_valid\":true,\"hrp\":\"ms\",\"kind\":\"share\",\"share_index\":\"s\",\"tag\":\"test\",\"threshold\":2},\"schema_version\":\"1\",\"would_combine\":false,\"would_decode\":false}\n";

/// A 2-of-n share with id `test` carrying the given data bytes.
fn share_data(data: &[u8], idx: Fe) -> String {
    Codex32String::from_seed("ms", 2, "test", idx, data)
        .unwrap()
        .to_string()
}

#[test]
fn bh1119_ms_i12_combine_oracle() {
    // The bad pair: 40 data bytes (string length 86); combine refuses it.
    let bad = format!("{}\n{}\n", share(40, Fe::A), share(40, Fe::C));
    let c = ms(&["combine", "--to", "entropy", "-"], &bad);
    assert_eq!(c.status.code(), Some(1), "{}", err_s(&c));
    assert_eq!(out_s(&c), "");
    assert_eq!(
        err_s(&c),
        "error: tag \"entr\" payload length 39 not in expected set [16, 20, 24, 28, 32]\n"
    );
    // Entr control: 17 data bytes (length 50) recover 16 zero bytes.
    let entr = format!("{}\n{}\n", share(17, Fe::A), share(17, Fe::C));
    let c = ms(&["combine", "--to", "entropy", "-"], &entr);
    assert_eq!(c.status.code(), Some(0), "{}", err_s(&c));
    assert!(out_s(&c).starts_with(&format!("{}\n", "00".repeat(16))), "{}", out_s(&c));
    // Mnem control: [0x02, 0x00 (English), 16 zero bytes] (length 51).
    let mut d = vec![0x02u8, 0x00];
    d.extend_from_slice(&[0u8; 16]);
    let s = share_data(&d, Fe::A);
    assert_eq!(s.len(), 51);
    let mnem = format!("{}\n{}\n", s, share_data(&d, Fe::C));
    let c = ms(&["combine", "--to", "entropy", "-"], &mnem);
    assert_eq!(c.status.code(), Some(0), "{}", err_s(&c));
    assert!(out_s(&c).starts_with(&format!("{}\n", "00".repeat(16))), "{}", out_s(&c));
}

#[test]
fn bh1119_ms_i12_inspect_share_length_json() {
    let s = share(40, Fe::A);
    eprintln!("share len {} : {s}", s.len());
    let o = ms(&["inspect", "--json", "-"], &s);
    assert_eq!(o.status.code(), Some(0), "{}", err_s(&o));
    assert_eq!(out_s(&o), BAD_LEN_JSON_A);
}

#[test]
fn bh1119_ms_i12_inspect_share_length_text() {
    let s = share(40, Fe::A);
    let o = ms(&["inspect", "-"], &s);
    assert_eq!(o.status.code(), Some(0), "{}", err_s(&o));
    assert_eq!(out_s(&o), BAD_LEN_TEXT_A);
}

#[test]
fn bh1119_ms_i12_inspect_secret_share_bad_length_both_reasons() {
    let s = share(40, Fe::S);
    let j = ms(&["inspect", "--json", "-"], &s);
    assert_eq!(j.status.code(), Some(0), "{}", err_s(&j));
    assert_eq!(out_s(&j), BAD_LEN_JSON_S);
    let t = ms(&["inspect", "-"], &s);
    assert_eq!(t.status.code(), Some(0), "{}", err_s(&t));
    assert_eq!(out_s(&t), BAD_LEN_TEXT_S);
}

#[test]
fn bh1119_ms_i12_inspect_share_verdict_tracks_length() {
    let mut wrong = Vec::new();
    // Data sizes whose share string has a codex32 length (48-93 short, 125-127
    // long); other lengths are refused at parse (exit 1), unchanged.
    for n in (16..=44usize).chain(63..=64) {
        let s = share(n, Fe::A);
        let want = SECRET_LENGTHS.contains(&s.len());
        let jo = ms(&["inspect", "--json", "-"], &s);
        let to = ms(&["inspect", "-"], &s);
        assert_eq!((jo.status.code(), to.status.code()), (Some(0), Some(0)), "n={n} len={}", s.len());
        let j: Value = serde_json::from_slice(&jo.stdout).unwrap();
        let t = out_s(&to);
        let json_ok = j["would_combine"] == Value::Bool(want) && j["would_decode"] == Value::Bool(want);
        let text_ok = t.starts_with("OK: K-of-N share (would combine: needs 2 shares)\n") == want
            && t.starts_with("FAIL: would NOT combine\n") == !want;
        if !(json_ok && text_ok) {
            wrong.push((n, s.len()));
        }
    }
    assert!(wrong.is_empty(), "(data bytes, string length) with a wrong verdict: {wrong:?}");
}
