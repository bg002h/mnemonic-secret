//! English phrase with --language japanese → exit 1 (UnknownWord).

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn encode_rejects_english_phrase_under_japanese_lang() {
    let english = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    Command::cargo_bin("ms")
        .unwrap()
        .args(["encode", "--phrase", english, "--language", "japanese"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("unknown BIP-39 word"));
}
