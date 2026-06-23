//! `ms encode` / `ms split` mstring display-grouping flags (P2).
//! Default = space/5 print-once; `--group-size 0` unbroken; `--separator`.

use assert_cmd::Command;

const Z12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
/// Canonical unbroken ms1 for the 12-word all-zeros phrase (wire canary).
const CANON: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

fn stdout_of(args: &[&str]) -> String {
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn encode_default_groups_space_5_print_once() {
    let s = stdout_of(&["encode", "--phrase", Z12]);
    // print-once: no blank line / no second copy.
    assert!(
        !s.contains("\n\n"),
        "print-once: stdout must not contain \\n\\n; got {s:?}"
    );
    let line = s.lines().next().unwrap();
    assert_eq!(
        line.chars().nth(5),
        Some(' '),
        "expected a space after the first 5 chars; got {line:?}"
    );
    let unbroken: String = line.chars().filter(|c| *c != ' ').collect();
    assert_eq!(
        unbroken, CANON,
        "space-stripped grouped form must equal canonical ms1"
    );
}

#[test]
fn encode_unbroken_group_size_0() {
    let s = stdout_of(&["encode", "--phrase", Z12, "--group-size", "0"]);
    let line = s.lines().next().unwrap();
    assert_eq!(line, CANON);
}

#[test]
fn encode_separator_hyphen() {
    let s = stdout_of(&["encode", "--phrase", Z12, "--separator", "hyphen"]);
    let line = s.lines().next().unwrap();
    assert_eq!(
        line.chars().nth(5),
        Some('-'),
        "expected hyphen at idx 5; got {line:?}"
    );
}

#[test]
fn encode_rejects_bad_separator() {
    // ms maps clap parse errors to exit 64 (main.rs).
    Command::cargo_bin("ms")
        .unwrap()
        .args(["encode", "--phrase", Z12, "--separator", "bogus"])
        .assert()
        .code(64);
}

#[test]
fn split_grouped_default_labels_on_stderr() {
    // Default-grouped split: stdout = N grouped share lines; labels → stderr.
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(["split", "--phrase", Z12, "-k", "2", "-n", "3"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(
        lines.len(),
        3,
        "stdout = exactly 3 share lines; got {stdout:?}"
    );
    for l in &lines {
        assert!(l.starts_with("ms1"), "share line: {l:?}");
        assert!(
            l.contains(' '),
            "default-grouped share must contain a space: {l:?}"
        );
    }
    assert!(
        !stdout.contains("share "),
        "labels must NOT be on stdout; got {stdout:?}"
    );
    assert!(
        stderr.contains("share 1 of 3"),
        "label on stderr; got {stderr:?}"
    );
}
