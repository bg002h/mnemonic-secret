//! A reserved-tag (future format) ms1 exits 3 and says why (architect
//! crash-sweep-exit-sets Q4 E4). Only `verify` prints its own OK line on
//! stdout, so only `verify` (text mode) leaves stderr without the error line.

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
fn a_reserved_tag_exits_3_with_the_error_line() {
    let mut wrong = Vec::new();
    for args in [
        vec!["decode", "-"],
        vec!["derive", "-"],
        vec!["hashlock", "-"],
        vec!["repair", "--ms1", "-"],
    ] {
        let o = Command::cargo_bin("ms")
            .unwrap()
            .args(&args)
            .write_stdin(reserved_tag_ms1())
            .output()
            .unwrap();
        let err = String::from_utf8_lossy(&o.stderr);
        if o.status.code() != Some(3) || !o.stdout.is_empty() || err.lines().last() != Some(LINE) {
            wrong.push(format!(
                "{args:?}: {:?}, stdout {:?}, stderr {err:?}",
                o.status.code(),
                String::from_utf8_lossy(&o.stdout)
            ));
        }
    }
    assert!(wrong.is_empty(), "exit 3 without the line: {wrong:#?}");
}

#[test]
fn verify_says_ok_on_stdout_and_nothing_on_stderr() {
    let o = Command::cargo_bin("ms")
        .unwrap()
        .args(["verify", "-"])
        .write_stdin(reserved_tag_ms1())
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(3));
    assert_eq!(
        String::from_utf8_lossy(&o.stdout),
        "OK: valid future format (v0.2+, tag seed)\n"
    );
    assert!(
        o.stderr.is_empty(),
        "stderr: {:?}",
        String::from_utf8_lossy(&o.stderr)
    );
}
