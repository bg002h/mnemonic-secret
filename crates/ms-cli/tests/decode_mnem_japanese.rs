//! `ms decode` mnem-arm (0x02 payload) tests:
//! (a) No --language → emits wire language (japanese), no warning, exit 0.
//! (b) --language english (disagrees with wire) → wire wins, stderr warning, exit 0.
//! (c) Existing entr string → unchanged behaviour.

use assert_cmd::Command;
use predicates::prelude::*;

mod support;

/// Build a valid Japanese mnem ms1 from 16 entropy bytes (0xAB repeated).
fn japanese_mnem_ms1() -> String {
    let ja = bip39::Mnemonic::from_entropy_in(bip39::Language::Japanese, &[0xABu8; 16])
        .unwrap()
        .to_string();
    let encode_out = Command::cargo_bin("ms")
        .unwrap()
        .args(["encode", "--language", "japanese", "--phrase", "-"])
        .write_stdin(ja.to_string())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    std::str::from_utf8(&encode_out)
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .to_string()
}

fn expected_japanese_phrase() -> String {
    bip39::Mnemonic::from_entropy_in(bip39::Language::Japanese, &[0xABu8; 16])
        .unwrap()
        .to_string()
}

/// (a) Decode mnem string with NO --language → emits Japanese phrase, NO warning, exit 0.
#[test]
fn decode_mnem_no_language_arg_emits_wire_language_japanese() {
    let ms1 = japanese_mnem_ms1();
    let phrase = expected_japanese_phrase();
    Command::cargo_bin("ms")
        .unwrap()
        .args(["decode", "-"])
        .write_stdin(ms1.to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains(&phrase))
        .stdout(predicate::str::contains("language: japanese"))
        .stderr(predicate::str::contains("note:").not()); // no wire-language-mismatch warning
}

/// (b) --language english disagrees with wire (japanese) → wire wins, stderr warning, exit 0.
#[test]
fn decode_mnem_wrong_language_arg_wire_wins_with_warning() {
    let ms1 = japanese_mnem_ms1();
    let phrase = expected_japanese_phrase();
    let o = support::run(&["decode", "--language", "english", &ms1]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let so = String::from_utf8_lossy(&o.stdout);
    let se = String::from_utf8_lossy(&o.stderr);
    assert!(
        so.contains(&phrase),
        "wire language (japanese) phrase: {so}"
    );
    assert!(se.contains("japanese"), "warning names wire language: {se}");
    assert!(
        se.contains("english"),
        "warning names the user-supplied language: {se}"
    );
}

/// (c) Existing entr string decoded → unchanged (English default, no wire-wins warning).
#[test]
fn decode_entr_string_unchanged() {
    Command::cargo_bin("ms")
        .unwrap()
        .args(["decode", "-"])
        .write_stdin(("ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f").to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains("abandon abandon"))
        .stdout(predicate::str::contains("language: english"));
}

/// JSON mode: decode mnem ms1 with no --language → language is wire language.
#[test]
fn decode_mnem_json_emits_wire_language() {
    let ms1 = japanese_mnem_ms1();
    let o = support::run(&["decode", "--json", &ms1]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let so = String::from_utf8_lossy(&o.stdout);
    assert!(so.contains("\"language\":\"japanese\""), "{so}");
    assert!(so.contains("\"language_defaulted\":false"), "{so}");
}
