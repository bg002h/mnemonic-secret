//! Verify with a wrong --phrase fails without echoing either phrase.

use assert_cmd::Command;

#[test]
fn verify_round_trip_with_wrong_phrase_exit_2() {
    let wrong = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon ability";
    // The card from a file, the phrase on stdin: one reader per channel (`-`
    // for both would be a usage error, 64, before either is read).
    let dir = tempfile::tempdir().unwrap();
    let card = dir.path().join("card.ms1");
    std::fs::write(
        &card,
        "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f\n",
    )
    .unwrap();
    let assertion = Command::cargo_bin("ms")
        .unwrap()
        .args(["verify", "--in", card.to_str().unwrap(), "--phrase", "-"])
        .write_stdin((wrong).to_string())
        .assert()
        .failure();

    let output = assertion.get_output();
    let combined = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);

    // "ability" is not the right 12th word for the all-zero entropy, so the
    // phrase's BIP-39 checksum fails before the round-trip compare (SPEC
    // §2.4.1 step 3): an invalid input, exit 2. A well-formed different phrase
    // would be a mismatch, exit 1 (`verify_phrase_mismatch_is_no_1`).
    let code = output.status.code().unwrap();
    assert_eq!(code, 2, "expected exit 2, got {}", code);

    // Critical: neither phrase appears in any output (per SPEC §2.4 phrases-as-secrets).
    assert!(
        !combined.contains("ability"),
        "wrong phrase echoed in output: {}",
        combined
    );
    assert!(
        !combined.contains("about"),
        "decoded phrase echoed in output: {}",
        combined
    );
}
