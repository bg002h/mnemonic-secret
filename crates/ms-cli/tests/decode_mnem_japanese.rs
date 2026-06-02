//! `ms decode` mnem-arm (0x02 payload) tests:
//! (a) No --language → emits wire language (japanese), no warning, exit 0.
//! (b) --language english (disagrees with wire) → wire wins, stderr warning, exit 0.
//! (c) Existing entr string → unchanged behaviour.

use assert_cmd::Command;
use predicates::prelude::*;

/// Build a valid Japanese mnem ms1 from 16 entropy bytes (0xAB repeated).
fn japanese_mnem_ms1() -> String {
    let ja = bip39::Mnemonic::from_entropy_in(bip39::Language::Japanese, &[0xABu8; 16])
        .unwrap()
        .to_string();
    let encode_out = Command::cargo_bin("ms")
        .unwrap()
        .args(["encode", "--language", "japanese", "--phrase", &ja])
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
        .args(["decode", &ms1])
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
    Command::cargo_bin("ms")
        .unwrap()
        .args(["decode", "--language", "english", &ms1])
        .assert()
        .success()
        .stdout(predicate::str::contains(&phrase)) // wire language (japanese) phrase
        .stderr(predicate::str::contains("japanese"))  // warning names wire language
        .stderr(predicate::str::contains("english"));  // warning names user-supplied language
}

/// (c) Existing entr string decoded → unchanged (English default, no wire-wins warning).
#[test]
fn decode_entr_string_unchanged() {
    Command::cargo_bin("ms")
        .unwrap()
        .args(["decode", "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f"])
        .assert()
        .success()
        .stdout(predicate::str::contains("abandon abandon"))
        .stdout(predicate::str::contains("language: english"));
}

/// JSON mode: decode mnem ms1 with no --language → language is wire language.
#[test]
fn decode_mnem_json_emits_wire_language() {
    let ms1 = japanese_mnem_ms1();
    Command::cargo_bin("ms")
        .unwrap()
        .args(["decode", "--json", &ms1])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"language\":\"japanese\""))
        .stdout(predicate::str::contains("\"language_defaulted\":false"));
}
