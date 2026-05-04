//! SPEC §2.1 edge-case table: clap arg-group violations exit 64 (usage error).
//!
//! Both --phrase + --hex supplied → usage error.
//! Neither supplied → usage error.

use assert_cmd::Command;

#[test]
fn encode_rejects_both_phrase_and_hex() {
    Command::cargo_bin("ms").unwrap()
        .args([
            "encode",
            "--phrase",
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
            "--hex",
            "00000000000000000000000000000000",
        ])
        .assert()
        .failure()
        .code(64);
}

#[test]
fn encode_rejects_neither_phrase_nor_hex() {
    Command::cargo_bin("ms")
        .unwrap()
        .arg("encode")
        .assert()
        .failure()
        .code(64);
}
