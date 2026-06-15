//! `ms encode --language japanese --phrase <ja>` routes to mnem payload (0x02).
//! `ms encode --phrase <english>` stays entr payload (0x00, byte-identical to v0.1).
//! `ms encode --hex <hex>` stays entr payload (0x00).

use assert_cmd::Command;
use predicates::prelude::*;

/// Build a valid 12-word Japanese mnemonic from 16 entropy bytes (0xAB repeated).
fn japanese_12_word_phrase() -> String {
    let entropy = [0xABu8; 16];
    bip39::Mnemonic::from_entropy_in(bip39::Language::Japanese, &entropy)
        .expect("valid entropy length")
        .to_string()
}

/// ms1 lengths for mnem (0x02 payload = prefix + lang + entropy):
/// entropy 16 B → mnem string len 51
const MNEM_12_WORD_LEN: usize = 51;

#[test]
fn encode_japanese_phrase_produces_mnem_ms1_of_expected_length() {
    let ja = japanese_12_word_phrase();
    Command::cargo_bin("ms")
        .unwrap()
        .args(["encode", "--language", "japanese", "--phrase", &ja, "--group-size", "0"])
        .assert()
        .success()
        .stdout(predicate::function(|s: &str| {
            let first_line = s.lines().next().unwrap_or("");
            first_line.len() == MNEM_12_WORD_LEN
        }));
}

#[test]
fn encode_japanese_phrase_decode_round_trip() {
    let ja = japanese_12_word_phrase();
    // Encode to ms1
    let encode_out = Command::cargo_bin("ms")
        .unwrap()
        .args(["encode", "--language", "japanese", "--phrase", &ja, "--group-size", "0"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let ms1 = std::str::from_utf8(&encode_out)
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .to_string();
    assert_eq!(ms1.len(), MNEM_12_WORD_LEN, "expected mnem ms1 length");

    // Decode back — should recover the Japanese phrase
    Command::cargo_bin("ms")
        .unwrap()
        .args(["decode", &ms1])
        .assert()
        .success()
        .stdout(predicate::str::contains(&ja));
}

#[test]
fn encode_english_phrase_stays_entr_payload_length() {
    // 12-word English → entr ms1 = 50 chars
    let english = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    Command::cargo_bin("ms")
        .unwrap()
        .args(["encode", "--phrase", english, "--group-size", "0"])
        .assert()
        .success()
        .stdout(predicate::function(|s: &str| {
            let first_line = s.lines().next().unwrap_or("");
            first_line.len() == 50
        }));
}

#[test]
fn encode_hex_stays_entr_payload_length() {
    // 16 bytes hex → entr ms1 = 50 chars
    let hex32 = "ab".repeat(16);
    Command::cargo_bin("ms")
        .unwrap()
        .args(["encode", "--hex", &hex32, "--group-size", "0"])
        .assert()
        .success()
        .stdout(predicate::function(|s: &str| {
            let first_line = s.lines().next().unwrap_or("");
            first_line.len() == 50
        }));
}
