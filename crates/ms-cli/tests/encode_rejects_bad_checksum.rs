//! BIP-39 bad-checksum phrase → exit 1 with friendly message.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn encode_rejects_bad_bip39_checksum() {
    // Replace last word "about" with "ability" to break the BIP-39 checksum.
    let bad = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon ability";
    Command::cargo_bin("ms")
        .unwrap()
        .args(["encode", "--phrase", bad])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("BIP-39 checksum failure"));
}
