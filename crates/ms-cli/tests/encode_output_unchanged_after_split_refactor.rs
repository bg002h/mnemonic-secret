//! Task 2.1 byte-identity gate: extracting `resolve_secret_payload` from
//! `encode::run` MUST NOT change `ms encode`'s text or `--json` output for any
//! of the three input shapes {english phrase, non-english phrase, hex}.
//!
//! These are the EXACT bytes `ms encode` emitted before the refactor (captured
//! from the pre-refactor binary). The split command reuses the same helper, so
//! this is the encode-side guard that the helper preserves the
//! English-phrase/hex → Entr and non-English-phrase → Mnem auto-route plus the
//! `language` card/json field.

use assert_cmd::Command;

const ENGLISH_12: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// 12-word Japanese mnemonic from 16 bytes of 0xAB (mirrors encode_mnem_japanese.rs).
fn japanese_12_word() -> String {
    bip39::Mnemonic::from_entropy_in(bip39::Language::Japanese, &[0xABu8; 16])
        .expect("valid entropy length")
        .to_string()
}

fn stdout_of(args: &[&str]) -> String {
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(out).expect("stdout is utf-8")
}

#[test]
fn english_phrase_text_unchanged() {
    let s = stdout_of(&["encode", "--phrase", ENGLISH_12]);
    assert_eq!(
        s,
        "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f\n\n\
         ms10e ntrsq qqqqq qqqqq qqqqq qqqqq qqqqq qqcj9 sxraq 34v7f\n"
    );
}

#[test]
fn english_phrase_json_unchanged() {
    let s = stdout_of(&["encode", "--phrase", ENGLISH_12, "--json"]);
    assert_eq!(
        s,
        "{\"schema_version\":\"1\",\"ms1\":\"ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f\",\"language\":\"english\",\"word_count\":12,\"entropy_hex\":\"00000000000000000000000000000000\"}\n"
    );
}

#[test]
fn japanese_phrase_text_unchanged() {
    let ja = japanese_12_word();
    let s = stdout_of(&["encode", "--language", "japanese", "--phrase", &ja]);
    // mnem (0x02) ms1, 51-char first line; chunked second line.
    // 51-char ms1 → 11 chunks; line-wrap at 10 chunks/line puts the 11th
    // (`l`) on its own line.
    assert_eq!(
        s,
        "ms10entrsqgq6h2at4w46h2at4w46h2at4w46k0mt2va9nwh4ql\n\n\
         ms10e ntrsq gq6h2 at4w4 6h2at 4w46h 2at4w 46k0m t2va9 nwh4q\nl\n"
    );
}

#[test]
fn japanese_phrase_json_unchanged() {
    let ja = japanese_12_word();
    let s = stdout_of(&["encode", "--language", "japanese", "--phrase", &ja, "--json"]);
    assert_eq!(
        s,
        "{\"schema_version\":\"1\",\"ms1\":\"ms10entrsqgq6h2at4w46h2at4w46h2at4w46k0mt2va9nwh4ql\",\"language\":\"japanese\",\"word_count\":12,\"entropy_hex\":\"abababababababababababababababab\"}\n"
    );
}

#[test]
fn hex_text_unchanged() {
    let hex = "ab".repeat(16);
    let s = stdout_of(&["encode", "--hex", &hex]);
    assert_eq!(
        s,
        "ms10entrsqz46h2at4w46h2at4w46h2at4w4sna8r2pfm392lu\n\n\
         ms10e ntrsq z46h2 at4w4 6h2at 4w46h 2at4w 4sna8 r2pfm 392lu\n"
    );
}

#[test]
fn hex_json_unchanged_omits_language() {
    let hex = "ab".repeat(16);
    let s = stdout_of(&["encode", "--hex", &hex, "--json"]);
    // hex → Entr → no `language` field.
    assert_eq!(
        s,
        "{\"schema_version\":\"1\",\"ms1\":\"ms10entrsqz46h2at4w46h2at4w46h2at4w4sna8r2pfm392lu\",\"word_count\":12,\"entropy_hex\":\"abababababababababababababababab\"}\n"
    );
}
