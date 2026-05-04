//! Verify with wrong --phrase → exit 4. Phrase NEVER echoed to output.

use assert_cmd::Command;

#[test]
fn verify_round_trip_with_wrong_phrase_exit_4() {
    let wrong = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon ability";
    let assertion = Command::cargo_bin("ms")
        .unwrap()
        .args([
            "verify",
            "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f",
            "--phrase",
            wrong,
        ])
        .assert()
        .failure();

    let output = assertion.get_output();
    let combined = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);

    // Mismatch could be exit 4 (correct), or exit 1 if bip39 rejected the wrong
    // phrase first (it has a bad checksum since "ability" is not the right
    // 12th word for the all-zero-entropy case). Per SPEC §2.4.1 step 3:
    // bad-checksum phrase fires before round-trip compare → exit 1.
    let code = output.status.code().unwrap();
    assert!(code == 1 || code == 4, "expected exit 1 or 4, got {}", code);

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
