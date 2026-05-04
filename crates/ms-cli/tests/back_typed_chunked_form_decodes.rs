//! Engraver-typed-back chunked form via stdin (with spaces + newlines).

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn back_typed_chunked_form_with_spaces_and_newlines() {
    let typed_back = "ms10e ntrsq qqqqq qqqqq qqqqq qqqqq qqqqq qqcj9 sxraq 34v7f";

    Command::cargo_bin("ms")
        .unwrap()
        .args(["decode", "-"])
        .write_stdin(typed_back)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "entropy: 00000000000000000000000000000000",
        ));
}
