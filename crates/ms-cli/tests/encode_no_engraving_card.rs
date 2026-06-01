//! --no-engraving-card suppresses the engraving-card stderr block (word count,
//! language, passphrase notice); stdout unchanged. The output-class P advisory
//! is NOT suppressed — it is unconditional on any ms encode invocation.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn encode_no_engraving_card_suppresses_engraving_block() {
    let out = Command::cargo_bin("ms").unwrap()
        .args([
            "encode",
            "--phrase",
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
            "--no-engraving-card",
        ])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("ms10entrsqqqq"))
        .get_output()
        .stderr
        .clone();
    let stderr = String::from_utf8(out).expect("stderr utf-8");
    // Engraving card fields must NOT appear.
    assert!(
        !stderr.contains("word count:"),
        "--no-engraving-card must suppress word count line; got stderr={stderr:?}"
    );
    assert!(
        !stderr.contains("passphrase:"),
        "--no-engraving-card must suppress passphrase notice; got stderr={stderr:?}"
    );
    // Output-class P advisory MUST appear (unconditional — ms encode always
    // emits private key material regardless of --no-engraving-card).
    assert!(
        stderr.contains("warning: stdout carries private key material"),
        "P advisory must fire even with --no-engraving-card; got stderr={stderr:?}"
    );
}
