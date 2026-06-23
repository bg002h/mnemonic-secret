//! Verify on string with reserved-not-emitted tag → exit 3.

use assert_cmd::Command;
use ms_codec::codex32::{Codex32String, Fe};
use predicates::prelude::*;

#[test]
fn verify_reserved_seed_tag_exits_3() {
    let mut data = vec![0x00u8];
    data.extend_from_slice(&[0xAAu8; 16]);
    let s = Codex32String::from_seed("ms", 0, "seed", Fe::S, &data)
        .unwrap()
        .to_string();

    Command::cargo_bin("ms")
        .unwrap()
        .args(["verify", &s])
        .assert()
        .failure()
        .code(3)
        .stdout(predicate::str::contains(
            "OK: valid future format (v0.2+, tag seed)",
        ));
}
