//! Bug hunt 2026-11-25, ms#3 — a passphrase from stdin or `@env:` kept an
//! embedded newline (or other control character) without a word.
//!
//! The strip-one-trailing-newline rule is deliberate and shared with
//! mnemonic-toolkit (vectors/passphrase_channels.json), so the VALUE is not
//! changed: changing it would change the wallet. What changes is that a
//! passphrase that still holds a control character after that rule draws one
//! stderr warning, which never echoes the value.

use assert_cmd::Command;

/// `ms encode --hex 000102…0f` (bug report ms#2/ms#3).
const CARD: &str = "ms10entrsqqqqzqsrqszsvpcgpy9qkrqdpc8sh9lf92n5fr33s";
const WARN: &str = "contains a line break or other control character";

fn derive_stdin(passphrase: &str) -> (Option<i32>, String, String) {
    let dir = tempfile::tempdir().unwrap();
    let card = dir.path().join("card.ms1");
    std::fs::write(&card, format!("{CARD}\n")).unwrap();
    let o = Command::cargo_bin("ms")
        .unwrap()
        .args([
            "derive",
            "--in",
            card.to_str().unwrap(),
            "--passphrase-stdin",
        ])
        .write_stdin(passphrase.to_string())
        .output()
        .unwrap();
    (
        o.status.code(),
        String::from_utf8_lossy(&o.stdout).into_owned(),
        String::from_utf8_lossy(&o.stderr).into_owned(),
    )
}

fn derive_env(value: &str) -> (Option<i32>, String, String) {
    let dir = tempfile::tempdir().unwrap();
    let card = dir.path().join("card.ms1");
    std::fs::write(&card, format!("{CARD}\n")).unwrap();
    let o = Command::cargo_bin("ms")
        .unwrap()
        .args([
            "derive",
            "--in",
            card.to_str().unwrap(),
            "--passphrase",
            "@env:MS_BH1125_PP",
        ])
        .env("MS_BH1125_PP", value)
        .output()
        .unwrap();
    (
        o.status.code(),
        String::from_utf8_lossy(&o.stdout).into_owned(),
        String::from_utf8_lossy(&o.stderr).into_owned(),
    )
}

/// The fingerprints the bug report measured (and checked independently).
#[test]
fn values_are_unchanged() {
    for (pp, fp) in [
        ("TREZOR\n", "3dd37943"),
        ("TREZOR\n\n", "86d72e39"),
        ("TREZOR\nX\n", "a865e667"),
    ] {
        let (rc, out, err) = derive_stdin(pp);
        assert_eq!(rc, Some(0), "{pp:?}: {err}");
        assert!(out.contains(fp), "{pp:?}: want {fp}: {out}");
    }
}

#[test]
fn extra_blank_line_on_stdin_warns() {
    let (rc, _, err) = derive_stdin("TREZOR\n\n");
    assert_eq!(rc, Some(0), "{err}");
    assert_eq!(err.matches(WARN).count(), 1, "{err}");
    assert!(err.contains("stdin"), "{err}");
    assert!(!err.contains("TREZOR"), "never echoes the value: {err}");
}

#[test]
fn two_lines_on_stdin_warn() {
    let (rc, _, err) = derive_stdin("TREZOR\nX\n");
    assert_eq!(rc, Some(0), "{err}");
    assert_eq!(err.matches(WARN).count(), 1, "{err}");
}

#[test]
fn tab_or_lone_cr_warns() {
    for pp in ["TRE\tZOR", "TREZOR\r"] {
        let (rc, _, err) = derive_stdin(pp);
        assert_eq!(rc, Some(0), "{pp:?}: {err}");
        assert_eq!(err.matches(WARN).count(), 1, "{pp:?}: {err}");
    }
}

#[test]
fn env_with_interior_newline_warns() {
    let (rc, _, err) = derive_env("TRE\nZOR");
    assert_eq!(rc, Some(0), "{err}");
    assert_eq!(err.matches(WARN).count(), 1, "{err}");
    assert!(err.contains("environment variable MS_BH1125_PP"), "{err}");
    assert!(!err.contains("ZOR"), "never echoes the value: {err}");
}

#[test]
fn ordinary_passphrases_do_not_warn() {
    for pp in [
        "TREZOR",
        "TREZOR\n",
        "TREZOR\r\n",
        " spaced pass phrase ",
        "naïve",
    ] {
        let (rc, _, err) = derive_stdin(pp);
        assert_eq!(rc, Some(0), "{pp:?}: {err}");
        assert!(!err.contains(WARN), "{pp:?}: {err}");
    }
    let (_, _, err) = derive_env("TREZOR\n");
    assert!(!err.contains(WARN), "{err}");
}
