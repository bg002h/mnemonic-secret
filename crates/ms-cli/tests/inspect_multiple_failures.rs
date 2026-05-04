//! Inspect on string with multiple violations reports both, sorted by rule number.

use assert_cmd::Command;
use codex32::{Codex32String, Fe};

#[test]
fn inspect_multiple_failures_sorted() {
    // Both wrong-hrp (rule 2) AND non-zero-prefix (rule 8).
    let mut data = vec![0x01u8];
    data.extend_from_slice(&[0xAAu8; 16]);
    let s = Codex32String::from_seed("mq", 0, "entr", Fe::S, &data)
        .unwrap()
        .to_string();

    let output = Command::cargo_bin("ms")
        .unwrap()
        .args(["inspect", &s])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();
    // Both reasons should appear; wrong-hrp first per rule-2-before-rule-8.
    let wrong_hrp_idx = stdout.find("wrong-hrp").expect("wrong-hrp reason present");
    let non_zero_idx = stdout
        .find("non-zero-prefix")
        .expect("non-zero-prefix reason present");
    assert!(
        wrong_hrp_idx < non_zero_idx,
        "reasons not in rule-number order"
    );
}
