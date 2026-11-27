//! A reserved-tag (future format) ms1 is an input this version cannot use: it
//! exits 2, INVALID, and says why (architect crash-sweep-exit-sets Q4 E4). Under
//! the constellation exit-code spec (R5) `verify` no longer prints an "OK" line
//! beside a failing code: it names the problem on stderr like every other verb.

use assert_cmd::Command;
use ms_codec::codex32::{Codex32String, Fe};

const LINE: &str = "error: tag \"seed\" reserved-not-emitted in v0.1; deferred to v0.2+";

fn reserved_tag_ms1() -> String {
    let mut data = vec![0x00u8];
    data.extend_from_slice(&[0xAAu8; 16]);
    Codex32String::from_seed("ms", 0, "seed", Fe::S, &data)
        .unwrap()
        .to_string()
}

#[test]
fn a_reserved_tag_exits_2_with_the_error_line() {
    let mut wrong = Vec::new();
    for args in [
        vec!["decode", "-"],
        vec!["derive", "-"],
        vec!["hashlock", "-"],
        vec!["repair", "--ms1", "-"],
        vec!["verify", "-"],
    ] {
        let o = Command::cargo_bin("ms")
            .unwrap()
            .args(&args)
            .write_stdin(reserved_tag_ms1())
            .output()
            .unwrap();
        let err = String::from_utf8_lossy(&o.stderr);
        if o.status.code() != Some(2) || !o.stdout.is_empty() || err.lines().last() != Some(LINE) {
            wrong.push(format!(
                "{args:?}: {:?}, stdout {:?}, stderr {err:?}",
                o.status.code(),
                String::from_utf8_lossy(&o.stdout)
            ));
        }
    }
    assert!(wrong.is_empty(), "exit 2 without the line: {wrong:#?}");
}

#[test]
fn verify_no_longer_says_ok() {
    let o = Command::cargo_bin("ms")
        .unwrap()
        .args(["verify", "-"])
        .write_stdin(reserved_tag_ms1())
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(2));
    assert!(
        o.stdout.is_empty(),
        "stdout: {:?}",
        String::from_utf8_lossy(&o.stdout)
    );
}
