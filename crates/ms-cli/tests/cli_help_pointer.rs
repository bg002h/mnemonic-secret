//! Help-footer guard: `ms --help` points lost-passphrase users at btcrecover.
//!
//! Mirror of the `mnemonic` CLI footer (recon decision 2026-05-25): a BIP-39
//! passphrase has no internal verifier, so `ms` cannot brute-force it from the
//! entropy alone — that is an external-derivation-oracle attack, which
//! btcrecover performs. Assert the load-bearing substrings only (name +
//! maintained repo + date stamp); exact rendering is clap-version-sensitive.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn top_level_help_points_to_btcrecover_for_passphrase_recovery() {
    Command::cargo_bin("ms")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("btcrecover"))
        .stdout(predicate::str::contains(
            "https://github.com/3rdIteration/btcrecover",
        ))
        .stdout(predicate::str::contains("2026-05-25"));
}
