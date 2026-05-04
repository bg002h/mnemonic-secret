//! When --language is defaulted, stdout language line carries DEFAULT annotation
//! AND stderr emits non-suppressible warning (SPEC §6.3 hazard surfacing).

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn decode_default_english_warns_on_both_streams() {
    Command::cargo_bin("ms")
        .unwrap()
        .args([
            "decode",
            "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "default — verify against your records",
        ))
        .stderr(predicate::str::contains(
            "note: --language defaulted to 'english'",
        ));
}
