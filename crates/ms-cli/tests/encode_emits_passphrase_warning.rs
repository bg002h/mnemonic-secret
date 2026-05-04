//! SPEC §2.1 + architect r1-C1 resolution: encode stderr engraving card includes
//! the passphrase reminder line.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn encode_emits_passphrase_warning_on_stderr() {
    Command::cargo_bin("ms").unwrap()
        .args([
            "encode",
            "--phrase",
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "passphrase: not stored in ms1 (record separately if used)",
        ));
}
