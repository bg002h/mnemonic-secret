//! F-687 — `--passphrase -` is stdin and `--passphrase @env:VAR` is the
//! environment; a literal argv passphrase keeps working with one stderr note.
//!
//! Driven by `vectors/passphrase_channels.json`, which mnemonic-toolkit carries
//! byte-identically: the rule is constellation-wide and other implementations
//! follow the vector file, not this harness.
//!
//! Before F-687, on ms 0.19.1, `ms derive --passphrase -` derived with the
//! one-character passphrase `-` (fingerprint `66d564d1`) at exit 0 even with
//! `TREZOR` on stdin, and `--passphrase @env:VAR` was refused as argv material.

use std::io::Write as _;

use assert_cmd::Command;
use serde_json::Value;

const VECTORS: &str = include_str!("../vectors/passphrase_channels.json");

/// "abandon ×11 about" as an ms1 card, so the seed never touches argv.
const CARD: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

fn card_file() -> tempfile::NamedTempFile {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    writeln!(f, "{CARD}").unwrap();
    f
}

fn fingerprint(stdout: &str) -> Option<String> {
    stdout
        .lines()
        .find(|l| l.starts_with("master_fingerprint:"))
        .map(|l| l.split_whitespace().last().unwrap().to_string())
}

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run_case(case: &Value, card: &std::path::Path) -> Run {
    let mut cmd = Command::cargo_bin("ms").unwrap();
    cmd.arg("derive").arg("--in").arg(card);
    if case["allow_argv_secret"].as_bool().unwrap_or(false) {
        cmd.arg("--allow-argv-secret");
    }
    for a in case["args"].as_array().unwrap() {
        cmd.arg(a.as_str().unwrap());
    }
    if let Some(env) = case["env"].as_object() {
        for (k, v) in env {
            cmd.env(k, v.as_str().unwrap());
        }
    }
    for k in case["unset"].as_array().into_iter().flatten() {
        cmd.env_remove(k.as_str().unwrap());
    }
    cmd.write_stdin(case["stdin"].as_str().unwrap().as_bytes().to_vec());
    let out = cmd.output().unwrap();
    Run {
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

#[test]
fn every_vector_case_holds() {
    let v: Value = serde_json::from_str(VECTORS).unwrap();
    let note = v["argv_note"].as_str().unwrap();
    assert_eq!(
        note,
        "warning: secret material on argv (--passphrase) \u{2014} read it privately with \
         --passphrase - or --passphrase-stdin (stdin), or --passphrase @env:VAR \
         (environment variable)"
    );
    let card = card_file();
    let cases = v["cases"].as_array().unwrap();
    assert!(
        cases.len() >= 37,
        "the vector file lost cases: {}",
        cases.len()
    );
    let mut failures = Vec::new();
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let r = run_case(case, card.path());
        let notes = r.stderr.lines().filter(|l| *l == note).count() as u64;
        if notes != case["argv_notes"].as_u64().unwrap() {
            failures.push(format!("{name}: {notes} argv notes; stderr:\n{}", r.stderr));
        }
        if let Some(fp) = case["expect"]["fingerprint"].as_str() {
            if r.code != 0 || fingerprint(&r.stdout).as_deref() != Some(fp) {
                failures.push(format!(
                    "{name}: want {fp}, got rc {} fp {:?}; stderr:\n{}",
                    r.code,
                    fingerprint(&r.stdout),
                    r.stderr
                ));
            }
        } else {
            let needle = case["expect"]["error_contains"].as_str().unwrap();
            if r.code == 0 || !r.stderr.contains(needle) || !r.stdout.is_empty() {
                failures.push(format!(
                    "{name}: want an error naming {needle:?}, got rc {} stdout {:?} stderr {:?}",
                    r.code, r.stdout, r.stderr
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n---\n"));
}

/// The literal form's stdout is byte-identical to the stdin form's: the note
/// goes to stderr and nowhere else, and never carries the value.
#[test]
fn a_literal_changes_stderr_only() {
    let card = card_file();
    let base = || {
        let mut c = Command::cargo_bin("ms").unwrap();
        c.arg("derive")
            .arg("--in")
            .arg(card.path())
            .args(["--template", "bip84"]);
        c
    };
    let stdin_out = base()
        .arg("--passphrase-stdin")
        .write_stdin("TREZOR")
        .output()
        .unwrap();
    let lit_out = base()
        .args(["--allow-argv-secret", "--passphrase", "TREZOR"])
        .output()
        .unwrap();
    assert!(stdin_out.status.success() && lit_out.status.success());
    assert_eq!(stdin_out.stdout, lit_out.stdout);
    let lit_err = String::from_utf8(lit_out.stderr).unwrap();
    assert!(!lit_err.contains("TREZOR"), "{lit_err}");
    let stdin_err = String::from_utf8(stdin_out.stderr).unwrap();
    // Exactly one extra stderr line, and it is the note.
    let extra: Vec<&str> = lit_err
        .lines()
        .filter(|l| !stdin_err.lines().any(|s| s == *l))
        .collect();
    assert_eq!(extra.len(), 1, "{lit_err}");
    assert!(extra[0].contains("--passphrase @env:VAR"), "{}", extra[0]);
}

/// One stdin per invocation: `--passphrase -` beside an ms1 on stdin is
/// refused before anything is read — the same guard `--passphrase-stdin` has.
#[test]
fn dash_beside_another_stdin_input_is_refused() {
    for pp in [&["--passphrase", "-"][..], &["--passphrase-stdin"][..]] {
        let out = Command::cargo_bin("ms")
            .unwrap()
            .args(["derive", "-"])
            .args(pp)
            .write_stdin(format!("{CARD}\n"))
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(1), "{pp:?}");
        assert!(out.stdout.is_empty());
        let err = String::from_utf8(out.stderr).unwrap();
        assert!(err.contains("one stdin per invocation"), "{pp:?}: {err}");
    }
}

/// The override substitutes an admitted `--passphrase <p>` with a `-`
/// placeholder; that placeholder must NOT count as a stdin reader (it would
/// refuse `ms derive - --allow-argv-secret --passphrase TREZOR`) or be read as
/// stdin (it would take the ms1 as the passphrase).
#[test]
fn the_overrides_placeholder_dash_is_not_stdin() {
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args([
            "derive",
            "-",
            "--allow-argv-secret",
            "--passphrase",
            "TREZOR",
        ])
        .write_stdin(format!("{CARD}\n"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        fingerprint(&String::from_utf8(out.stdout).unwrap()).as_deref(),
        Some("b4e3f5ed")
    );
}

/// `@env:` on `--passphrase` is a channel, not material: the argv guard no
/// longer refuses it (it did before F-687), in either spelling.
#[test]
fn the_guard_admits_the_env_channel_without_the_override() {
    let card = card_file();
    for args in [
        &["--passphrase", "@env:F687_GUARD"][..],
        &["--passphrase=@env:F687_GUARD"][..],
    ] {
        let out = Command::cargo_bin("ms")
            .unwrap()
            .arg("derive")
            .arg("--in")
            .arg(card.path())
            .args(args)
            .env("F687_GUARD", "TREZOR")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    // ...but only on `--passphrase`: `--hex @env:X` is still refused as material.
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(["derive", "--hex", "@env:F687_GUARD"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
}

// ---------------------------------------------------------------------------
// Fold 1 (review f687-review.md).
// ---------------------------------------------------------------------------

/// M1: `--in` naming stdin — by name, or the same file as fd 0 — is a stdin
/// reader. Before fold 1, `ms derive --in /dev/stdin --passphrase -` derived
/// with the EMPTY passphrase (73c5da0a) at exit 0.
#[test]
fn an_in_path_that_is_stdin_is_a_second_stdin_reader() {
    for path in ["/dev/stdin", "/dev/fd/0"] {
        for pp in [&["--passphrase", "-"][..], &["--passphrase-stdin"][..]] {
            let out = Command::cargo_bin("ms")
                .unwrap()
                .args(["derive", "--in", path])
                .args(pp)
                .write_stdin(format!("{CARD}\n"))
                .output()
                .unwrap();
            assert_eq!(out.status.code(), Some(1), "{path} {pp:?}");
            assert!(out.stdout.is_empty());
            assert!(String::from_utf8_lossy(&out.stderr).contains("one stdin per invocation"));
        }
    }
    // By inode: fd 0 IS the card file (a real `< card.ms1` redirect, not a
    // pipe), and the same file is named as --in.
    let card = card_file();
    let out = std::process::Command::new(assert_cmd::cargo::cargo_bin("ms"))
        .arg("derive")
        .arg("--in")
        .arg(card.path())
        .args(["--passphrase", "-"])
        .stdin(std::fs::File::open(card.path()).unwrap())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "{out:?}");
}

/// M2: the guard and the resolver agree on `-`: a padded dash is argv
/// material (refused), never the stdin channel, in every spelling.
#[test]
fn a_padded_dash_is_material_in_every_spelling() {
    let card = card_file();
    for args in [
        &["--passphrase", " -"][..],
        &["--passphrase", "\t-"][..],
        &["--passphrase", "- "][..],
        &["--passphrase=- "][..],
        &["--passphrase=-\n"][..],
    ] {
        let out = Command::cargo_bin("ms")
            .unwrap()
            .arg("derive")
            .arg("--in")
            .arg(card.path())
            .args(args)
            .write_stdin("TREZOR")
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(1), "{args:?}");
        assert!(out.stdout.is_empty(), "{args:?}");
        assert!(String::from_utf8_lossy(&out.stderr).contains("BIP-39 passphrase"));
    }
}

/// N1: non-UTF-8 `@env:` value → "not valid UTF-8"; non-UTF-8 argv → 64,
/// not a panic, and not echoed.
#[cfg(unix)]
#[test]
fn non_utf8_input_is_refused_by_name_not_panicked() {
    use std::os::unix::ffi::OsStrExt;
    let bad = std::ffi::OsStr::from_bytes(b"TRE\xffZOR");
    let card = card_file();
    let out = Command::cargo_bin("ms")
        .unwrap()
        .arg("derive")
        .arg("--in")
        .arg(card.path())
        .args(["--passphrase", "@env:F687_PP"])
        .env("F687_PP", bad)
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{err}");
    assert!(
        err.contains("F687_PP") && err.contains("not valid UTF-8"),
        "{err}"
    );
    let out = Command::cargo_bin("ms")
        .unwrap()
        .arg("derive")
        .arg("--in")
        .arg(card.path())
        .args(["--allow-argv-secret", "--passphrase"])
        .arg(bad)
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(64), "{err}");
    assert!(
        err.contains("not valid UTF-8") && !err.contains("ZOR"),
        "{err}"
    );
}
