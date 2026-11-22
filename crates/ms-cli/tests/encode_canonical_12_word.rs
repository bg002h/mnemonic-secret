//! `ms encode --phrase` 12-word abandon round-trip.

use assert_cmd::Command;

#[test]
fn encode_12_word_abandon_about() {
    // Print-once: the ms1 appears once, on stdout.
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args([
            "encode",
            "--phrase",
            "-"]).write_stdin(("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about").to_string())
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        !stdout.contains("\n\n"),
        "print-once: no blank line; got {stdout:?}"
    );
    // Single print (2026-11-22): stdout is the CANONICAL ms1, ungrouped by
    // default, and stderr carries no copy of it.
    let line = stdout.lines().next().unwrap();
    assert_eq!(
        line, "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f",
        "stdout is the artifact, not a display form; got {line:?}"
    );
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        !stderr.contains("ms10e") && !stderr.contains("engraving card:"),
        "single print: no copy of the ms1 on stderr; got {stderr:?}"
    );
    assert!(stderr.contains("language: english"));
    assert!(stderr.contains("word count: 12"));
    assert!(stderr.contains("passphrase: not stored"));
}
