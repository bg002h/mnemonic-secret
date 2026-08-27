//! `ms encode --phrase` 12-word abandon round-trip.

use assert_cmd::Command;

#[test]
fn encode_12_word_abandon_about() {
    // mstring-grouping P2: encode text is now print-once, default space/5
    // (was `<ms1>\n\n<chunked>` print-twice). stderr engraving card unchanged.
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
    // §6a/§6b: stdout is the CANONICAL ms1, always ungrouped. The default
    // space/5 grouping moved to the stderr engraving card.
    let line = stdout.lines().next().unwrap();
    assert_eq!(
        line, "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f",
        "stdout is the artifact, not a display form; got {line:?}"
    );
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("engraving card: ms10e ntrsq"),
        "the default space/5 grouping is on the card now; got {stderr:?}"
    );
    assert!(stderr.contains("language: english"));
    assert!(stderr.contains("word count: 12"));
    assert!(stderr.contains("passphrase: not stored"));
}
