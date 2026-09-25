//! **F-679 — an `--allow-argv-secret`-admitted value is not a stdin read.**
//!
//! The override substitutes each admitted argv value with `-` before clap
//! parses, and routes the material through a side channel. Two verbs guard
//! "both inputs from stdin" by testing for that same `-`, so on ms 0.19.0
//! `ms verify --allow-argv-secret --phrase <p> <ms1> </dev/null` exited 1 with
//! `cannot read both ms1 and --phrase from stdin` while reading nothing from
//! stdin. `combine` had the mirror image: the first `-` drained the admitted
//! shares, so a genuine `-` for stdin was silently ignored.
//!
//! Every row runs with an explicit stdin (empty where nothing should be read),
//! so a row that passes by reading stdin cannot hide.

use assert_cmd::Command;
use serde_json::Value;
use std::io::Write;

mod support;

const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
const HEX: &str = "00000000000000000000000000000000";
const PASSPHRASE: &str = "correct horse battery staple";

fn ms() -> Command {
    Command::cargo_bin("ms").unwrap()
}

fn write_tmp(dir: &std::path::Path, name: &str, body: &str) -> std::path::PathBuf {
    let p = dir.join(name);
    let mut f = std::fs::File::create(&p).unwrap();
    f.write_all(body.as_bytes()).unwrap();
    p
}

fn run(args: &[&str], stdin: &str) -> std::process::Output {
    ms().args(args)
        .write_stdin(stdin.to_string())
        .output()
        .unwrap()
}

fn ok(o: &std::process::Output, what: &str) -> String {
    assert_eq!(
        o.status.code(),
        Some(0),
        "{what}: stderr:\n{}",
        String::from_utf8_lossy(&o.stderr)
    );
    String::from_utf8_lossy(&o.stdout).into_owned()
}

// ---------------------------------------------------------------- verify

/// The reproduced defect: both values admitted, stdin empty.
#[test]
fn verify_admitted_ms1_and_admitted_phrase() {
    let o = run(
        &["verify", "--allow-argv-secret", "--phrase", PHRASE, MS1],
        "",
    );
    assert!(ok(&o, "argv+argv").contains("round-trip valid"));
}

/// Admitted phrase, ms1 genuinely on stdin.
#[test]
fn verify_admitted_phrase_with_ms1_on_stdin() {
    let o = run(
        &["verify", "--allow-argv-secret", "--phrase", PHRASE, "-"],
        MS1,
    );
    assert!(ok(&o, "argv phrase + stdin ms1").contains("round-trip valid"));
    // Omitted positional is the same stdin read.
    let o = run(&["verify", "--allow-argv-secret", "--phrase", PHRASE], MS1);
    assert!(ok(&o, "argv phrase + omitted ms1").contains("round-trip valid"));
}

/// Admitted ms1, phrase genuinely on stdin.
#[test]
fn verify_admitted_ms1_with_phrase_on_stdin() {
    let o = run(
        &["verify", "--allow-argv-secret", MS1, "--phrase", "-"],
        PHRASE,
    );
    assert!(ok(&o, "argv ms1 + stdin phrase").contains("round-trip valid"));
}

/// `--in` plus an admitted phrase keeps working.
#[test]
fn verify_in_file_with_admitted_phrase() {
    let dir = tempfile::tempdir().unwrap();
    let card = write_tmp(dir.path(), "card.ms1", MS1);
    let o = run(
        &[
            "verify",
            "--allow-argv-secret",
            "--in",
            &card.display().to_string(),
            "--phrase",
            PHRASE,
        ],
        "",
    );
    assert!(ok(&o, "--in + argv phrase").contains("round-trip valid"));
}

/// The guard's real purpose survives: two GENUINE stdin reads are refused,
/// with and without the override flag present.
#[test]
fn verify_genuine_stdin_twice_is_still_refused() {
    for args in [
        vec!["verify", "-", "--phrase", "-"],
        vec!["verify", "--phrase", "-"],
        vec!["verify", "--allow-argv-secret", "-", "--phrase", "-"],
        vec!["verify", "--allow-argv-secret", "--phrase", "-"],
    ] {
        let o = run(&args, MS1);
        assert_eq!(o.status.code(), Some(1), "{args:?}");
        assert!(
            String::from_utf8_lossy(&o.stderr)
                .contains("cannot read both ms1 and --phrase from stdin"),
            "{args:?}: {}",
            String::from_utf8_lossy(&o.stderr)
        );
    }
}

// ---------------------------------------------------------------- derive

fn fingerprint(stdout: &str) -> String {
    stdout
        .lines()
        .find(|l| l.to_ascii_lowercase().contains("fingerprint"))
        .unwrap_or_else(|| panic!("no fingerprint line in:\n{stdout}"))
        .to_string()
}

/// The private oracle: `--in card --passphrase-stdin`.
fn derive_oracle() -> String {
    let dir = tempfile::tempdir().unwrap();
    let card = write_tmp(dir.path(), "card.ms1", MS1);
    let o = run(
        &[
            "derive",
            "--in",
            &card.display().to_string(),
            "--passphrase-stdin",
        ],
        PASSPHRASE,
    );
    fingerprint(&ok(&o, "oracle"))
}

/// Every admitted entropy source, with the passphrase genuinely on stdin, and
/// the passphrase must actually reach the derivation (compared to the oracle).
#[test]
fn derive_admitted_entropy_with_passphrase_on_stdin() {
    let want = derive_oracle();
    for args in [
        vec!["derive", "--allow-argv-secret", MS1, "--passphrase-stdin"],
        vec![
            "derive",
            "--allow-argv-secret",
            "--hex",
            HEX,
            "--passphrase-stdin",
        ],
        vec![
            "derive",
            "--allow-argv-secret",
            "--phrase",
            PHRASE,
            "--passphrase-stdin",
        ],
    ] {
        let o = run(&args, PASSPHRASE);
        assert_eq!(fingerprint(&ok(&o, &format!("{args:?}"))), want, "{args:?}");
    }
}

/// `--in` with the passphrase on stdin, flag present, still works.
#[test]
fn derive_in_file_with_override_present() {
    let want = derive_oracle();
    let dir = tempfile::tempdir().unwrap();
    let card = write_tmp(dir.path(), "card.ms1", MS1);
    let o = run(
        &[
            "derive",
            "--allow-argv-secret",
            "--in",
            &card.display().to_string(),
            "--passphrase-stdin",
        ],
        PASSPHRASE,
    );
    assert_eq!(fingerprint(&ok(&o, "--in")), want);
}

#[test]
fn derive_genuine_stdin_twice_is_still_refused() {
    for args in [
        vec!["derive", "-", "--passphrase-stdin"],
        vec!["derive", "--passphrase-stdin"],
        vec!["derive", "--hex", "-", "--passphrase-stdin"],
        vec!["derive", "--phrase", "-", "--passphrase-stdin"],
        vec!["derive", "--allow-argv-secret", "-", "--passphrase-stdin"],
        vec![
            "derive",
            "--allow-argv-secret",
            "--hex",
            "-",
            "--passphrase-stdin",
        ],
    ] {
        let o = run(&args, MS1);
        assert_eq!(o.status.code(), Some(1), "{args:?}");
        assert!(
            String::from_utf8_lossy(&o.stderr).contains("one stdin per invocation"),
            "{args:?}: {}",
            String::from_utf8_lossy(&o.stderr)
        );
    }
}

// ---------------------------------------------------------------- combine

fn shares_2_of_3() -> Vec<String> {
    let o = support::run(&["split", "--phrase", PHRASE, "-k", "2", "-n", "3", "--json"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    v["shares"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

/// Both shares admitted, stdin empty.
#[test]
fn combine_admitted_shares_only() {
    let s = shares_2_of_3();
    let o = run(&["combine", "--allow-argv-secret", &s[0], &s[1]], "");
    assert!(ok(&o, "argv+argv").contains(PHRASE));
}

/// One share admitted, one GENUINELY on stdin via an explicit `-`. On 0.19.0
/// the first `-` drained the admitted share and the user's `-` was dropped.
#[test]
fn combine_admitted_share_plus_genuine_stdin() {
    let s = shares_2_of_3();
    for args in [
        vec!["combine", "--allow-argv-secret", &s[0], "-"],
        vec!["combine", "--allow-argv-secret", "-", &s[0]],
    ] {
        let o = run(&args, &format!("{}\n", s[1]));
        assert!(ok(&o, "argv + stdin").contains(PHRASE));
    }
}

/// `--in` still works with the flag present.
#[test]
fn combine_in_file_with_override_present() {
    let s = shares_2_of_3();
    let dir = tempfile::tempdir().unwrap();
    let f = write_tmp(dir.path(), "shares.txt", &format!("{}\n{}\n", s[0], s[2]));
    let o = run(
        &[
            "combine",
            "--allow-argv-secret",
            "--in",
            &f.display().to_string(),
        ],
        "",
    );
    assert!(ok(&o, "--in").contains(PHRASE));
}
