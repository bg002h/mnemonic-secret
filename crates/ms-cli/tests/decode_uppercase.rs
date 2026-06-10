//! `ms decode <UPPERCASE_MS1>` succeeds — BIP-173 all-uppercase (QR form)
//! acceptance, the CI-executed coverage for the envelope canonicalization
//! cycle (`design/PLAN_ms1_envelope_uppercase.md`; CI runs only
//! `cargo test -p ms-cli`).

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn decode_uppercase_ms1_succeeds() {
    // The all-uppercase twin of decode_round_trip.rs's canonical 12-word vector.
    Command::cargo_bin("ms")
        .unwrap()
        .args(["decode", "MS10ENTRSQQQQQQQQQQQQQQQQQQQQQQQQQQQQCJ9SXRAQ34V7F"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "entropy: 00000000000000000000000000000000",
        ))
        .stdout(predicate::str::contains(
            "phrase: abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        ));
}
