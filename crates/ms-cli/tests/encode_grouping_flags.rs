//! `ms encode` / `ms split` mstring display-grouping flags.
//! `ms encode`: default unbroken, `--group-size N` groups stdout (single print,
//! 2026-11-22). `ms split`: default space/5 on stdout (unchanged).

use assert_cmd::Command;

mod support;

const Z12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
/// Canonical unbroken ms1 for the 12-word all-zeros phrase (wire canary).
const CANON: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

fn stdout_of(args: &[&str]) -> String {
    let out = support::run(args);
    assert!(
        out.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

/// Default: stdout is the canonical ms1 and stderr carries no copy of it.
/// Measured, the reason for the unbroken default: piped into `me sysw pack`, a
/// grouped ms1 exits **4** -- "record 0 ... is not a form this container can
/// place" -- while the unbroken form exits **0**.
#[test]
fn encode_stdout_is_the_canonical_ms1_and_stderr_has_no_copy() {
    let out = support::run(&["encode", "--phrase", Z12]);
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert_eq!(s, format!("{CANON}\n"), "stdout is the canonical artifact");
    let e = String::from_utf8(out.stderr).unwrap();
    assert!(
        !e.contains(CANON) && !e.contains("ms10e") && !e.contains("engraving card:"),
        "no copy of the ms1 on stderr:\n{e}"
    );
}

/// `--group-size` and `--separator` shape STDOUT, on request.
#[test]
fn group_size_groups_encode_stdout() {
    for (extra, want) in [
        (vec![], CANON.to_string()),
        (vec!["--group-size", "0"], CANON.to_string()),
        (
            vec!["--group-size", "5"],
            "ms10e ntrsq qqqqq qqqqq qqqqq qqqqq qqqqq qqcj9 sxraq 34v7f".to_string(),
        ),
        (
            vec!["--group-size", "5", "--separator", "space"],
            "ms10e ntrsq qqqqq qqqqq qqqqq qqqqq qqqqq qqcj9 sxraq 34v7f".to_string(),
        ),
    ] {
        let mut argv = vec!["encode", "--phrase", Z12];
        argv.extend(extra.iter().copied());
        assert_eq!(stdout_of(&argv), format!("{want}\n"), "{argv:?}");
    }
}

#[test]
fn encode_rejects_bad_separator() {
    // ms maps clap parse errors to exit 64 (main.rs).
    Command::cargo_bin("ms")
        .unwrap()
        .args(["encode", "--phrase", "-", "--separator", "bogus"])
        .write_stdin((Z12).to_string())
        .assert()
        .code(64);
}

#[test]
fn split_grouped_default_labels_on_stderr() {
    // Default-grouped split: stdout = N grouped share lines; labels → stderr.
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(["split", "--phrase", "-", "-k", "2", "-n", "3"])
        .write_stdin((Z12).to_string())
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
