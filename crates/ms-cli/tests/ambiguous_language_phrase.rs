//! F-579: a valid English mnemonic whose every word is ALSO French made
//! `ms encode` and `ms split` panic (exit 101) instead of working.
//!
//! `bip39::Mnemonic::to_entropy` calls `to_entropy_array`, which re-detects the
//! language with `language_of_iter(self.words()).unwrap()` and throws away the
//! `lang` that `parse_in` was given -- on the crate's own comment that the
//! method "can only be called on values that were already previously
//! validated". Validation was against a KNOWN language; re-detection is a
//! different question, and for the 100 words in the English/French
//! intersection it is ambiguous.
//!
//! `abandon x23 + surface` is such a phrase, and it is the reasonably complex
//! wallet's own tier-3 seed -- so this crashed on a fixture in-tree. `--language
//! english` did not help, because the flag is discarded by the re-detection.
//! Found by the 2026-09-16 journey walk, which it nearly ended at step 3.
//!
//! MUTATION: restore `mnemonic.to_entropy()` in
//! `cmd::encode::resolve_secret_payload` -> every test here fails with exit 101.

use assert_cmd::Command;
use predicates::prelude::*;

/// 23x abandon + surface. Valid English; every word is also in the French list.
const AMBIGUOUS: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon surface";

/// The entropy behind it, computed INDEPENDENTLY from the BIP-39 wordlist
/// (31 zero bytes then 0x06) and checksum-verified, not read back from `ms`.
/// A round trip alone would agree with a wrong answer.
const AMBIGUOUS_ENTROPY: &str = "0000000000000000000000000000000000000000000000000000000000000006";

#[test]
fn encode_does_not_panic_on_an_english_phrase_that_is_also_french() {
    Command::cargo_bin("ms")
        .unwrap()
        .args(["encode", "--phrase", "-", "--group-size", "0"])
        .write_stdin(AMBIGUOUS)
        .assert()
        .success()
        .stdout(predicate::str::starts_with("ms1"));
}

#[test]
fn split_does_not_panic_on_the_same_phrase() {
    // `split` shares `encode::resolve_secret_payload`, so it panicked too.
    Command::cargo_bin("ms")
        .unwrap()
        .args(["split", "--phrase", "-", "-k", "2", "-n", "3"])
        .write_stdin(AMBIGUOUS)
        .assert()
        .success();
}

#[test]
fn the_entropy_is_the_one_the_wordlist_says_not_merely_a_round_trip() {
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(["encode", "--phrase", "-", "--group-size", "0"])
        .write_stdin(AMBIGUOUS)
        .assert()
        .success();
    let ms1 = String::from_utf8(out.get_output().stdout.clone()).unwrap();

    Command::cargo_bin("ms")
        .unwrap()
        .args(["decode"])
        .write_stdin(ms1.trim().to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains(AMBIGUOUS_ENTROPY))
        .stdout(predicate::str::contains("surface"));
}

/// The control that keeps the three above from passing on a stub: an ordinary,
/// unambiguous phrase must still produce the BIP-39 test vector's entropy.
#[test]
fn an_unambiguous_phrase_still_encodes_to_its_known_vector() {
    let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(["encode", "--phrase", "-", "--group-size", "0"])
        .write_stdin(phrase)
        .assert()
        .success();
    let ms1 = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    Command::cargo_bin("ms")
        .unwrap()
        .args(["decode"])
        .write_stdin(ms1.trim().to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains("0".repeat(64)));
}
