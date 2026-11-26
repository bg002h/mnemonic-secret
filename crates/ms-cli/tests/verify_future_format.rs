//! Verify on string with reserved-not-emitted tag → exit 2 (INVALID), no OK line.

use assert_cmd::Command;
use ms_codec::codex32::{Codex32String, Fe};
use predicates::prelude::*;

#[test]
fn verify_reserved_seed_tag_exits_2() {
    let mut data = vec![0x00u8];
    data.extend_from_slice(&[0xAAu8; 16]);
    let s = Codex32String::from_seed("ms", 0, "seed", Fe::S, &data)
        .unwrap()
        .to_string();

    Command::cargo_bin("ms")
        .unwrap()
        .args(["verify", "-"])
        .write_stdin(s.to_string())
        .assert()
        .failure()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("reserved-not-emitted"));
}
