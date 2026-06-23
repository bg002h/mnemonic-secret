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
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        !stdout.contains("\n\n"),
        "print-once: no blank line; got {stdout:?}"
    );
    let line = stdout.lines().next().unwrap();
    assert_eq!(
        line.chars().nth(5),
        Some(' '),
        "default space/5; got {line:?}"
    );
    assert!(
        line.chars()
            .filter(|c| *c != ' ')
            .collect::<String>()
            .starts_with("ms10entrsqqqq"),
        "space-stripped form starts with the canonical prefix; got {line:?}"
    );
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("language: english"));
    assert!(stderr.contains("word count: 12"));
    assert!(stderr.contains("passphrase: not stored"));
}
