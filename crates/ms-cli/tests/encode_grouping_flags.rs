//! `ms encode` / `ms split` mstring display-grouping flags (P2).
//! Default = space/5 print-once; `--group-size 0` unbroken; `--separator`.

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

/// **MOVED, not deleted: the grouping is now on the CARD.** §6a/§6b make
/// `ms encode`'s stdout the canonical ms1 and nothing else, always ungrouped,
/// and `--group-size` / `--separator` bind to the stderr engraving card alone.
/// The reason is cross-tool rather than cosmetic and was measured: piped into
/// `me sysw pack`, the grouped default exits **4** -- "record 0 ... is not a
/// form this container can place" -- and writes no payload, while the ungrouped
/// form exits **0** and writes a 102-byte payload at 0600.
#[test]
fn encode_stdout_is_the_canonical_ms1_and_the_card_carries_the_grouping() {
    let out = support::run(&["encode", "--phrase", Z12]);
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert_eq!(
        s,
        format!("{CANON}\n"),
        "stdout is the canonical artifact, ungrouped, and nothing else"
    );

    let e = String::from_utf8(out.stderr).unwrap();
    let card = e
        .lines()
        .find(|l| l.starts_with("engraving card: "))
        .unwrap_or_else(|| panic!("no engraving-card line in stderr:\n{e}"));
    let grouped = card.trim_start_matches("engraving card: ");
    assert_eq!(
        grouped.chars().nth(5),
        Some(' '),
        "the card keeps the default space/5 grouping; got {grouped:?}"
    );
    assert_eq!(
        grouped.chars().filter(|c| *c != ' ').collect::<String>(),
        CANON,
        "and the grouped form is the same artifact"
    );
}

/// The flags still WORK; they just work on the card.
#[test]
fn group_size_zero_collapses_the_card_and_leaves_stdout_alone() {
    let out = support::run(&["encode", "--phrase", Z12, "--group-size", "0"]);
    let s = String::from_utf8(out.stdout).unwrap();
    let e = String::from_utf8(out.stderr).unwrap();
    assert_eq!(s, format!("{CANON}\n"));
    assert!(
        e.contains(&format!("engraving card: {CANON}")),
        "at group-size 0 the card is the unbroken string too:\n{e}"
    );
}

/// **`--no-engraving-card` now throws away the form an engraver reads**, and so
/// does any `2>/dev/null`. §6c names this as a real change rather than a
/// cosmetic one, and this pins it so nobody re-discovers it on a plate.
#[test]
fn no_engraving_card_removes_the_only_grouped_form_there_is() {
    let out = support::run(&["encode", "--phrase", Z12, "--no-engraving-card"]);
    let s = String::from_utf8(out.stdout).unwrap();
    let e = String::from_utf8(out.stderr).unwrap();
    assert_eq!(s, format!("{CANON}\n"));
    assert!(
        !e.contains("engraving card:"),
        "the card is suppressed:\n{e}"
    );
    assert!(
        !e.contains("ms10e ntrsq"),
        "and with it the grouped form, which exists nowhere else now:\n{e}"
    );
}

/// `ms encode`'s stdout does not move, whatever the display flags say.
#[test]
fn no_display_flag_can_change_encode_stdout() {
    for extra in [
        vec![],
        vec!["--group-size", "0"],
        vec!["--group-size", "7"],
        vec!["--separator", "space"],
    ] {
        let mut argv = vec!["encode", "--phrase", Z12];
        argv.extend(extra.iter().copied());
        assert_eq!(
            stdout_of(&argv),
            format!("{CANON}\n"),
            "{argv:?} changed the ARTIFACT, which is not a display property"
        );
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
