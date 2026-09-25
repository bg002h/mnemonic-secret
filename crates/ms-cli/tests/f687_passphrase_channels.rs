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
        cases.len() >= 42,
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
        // F-687b ruling 2: exactly `empty_warnings` empty-channel warnings.
        let empties = r
            .stderr
            .lines()
            .filter(|l| {
                l.starts_with("warning: --passphrase from ")
                    && l.ends_with(" is empty; proceeding with the EMPTY passphrase")
            })
            .count() as u64;
        if empties != case["empty_warnings"].as_u64().unwrap() {
            failures.push(format!(
                "{name}: {empties} empty warnings; stderr:\n{}",
                r.stderr
            ));
        }
        // F-687b ruling 3: stdin is a pipe in every case, so never a prompt.
        if r.stderr.contains(v["prompt"].as_str().unwrap().trim_end()) {
            failures.push(format!("{name}: prompted on a non-terminal:\n{}", r.stderr));
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
        } else if let Some(code) = case["expect"]["exit_code"].as_i64() {
            if i64::from(r.code) != code || !r.stdout.is_empty() {
                failures.push(format!(
                    "{name}: want exit {code}, got rc {} stdout {:?} stderr {:?}",
                    r.code, r.stdout, r.stderr
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

// ---------------------------------------------------------------------------
// F-687b ruling 3: the terminal prompt. A real pseudo-terminal is the only way
// to exercise it: every other test pipes stdin, which is the no-prompt case.
// ---------------------------------------------------------------------------

/// Run `bin argv` with stdin on a fresh pty slave; wait for `prompt` on
/// stderr, then type `typed` on the master. Returns (exit code, stdout,
/// stderr, everything the terminal displayed = the pty echo).
#[cfg(target_os = "linux")]
fn run_on_a_terminal(
    bin: &std::path::Path,
    argv: &[&str],
    env: &[(&str, &str)],
    prompt: &str,
    typed: &[u8],
) -> (i32, String, String, Vec<u8>) {
    use std::io::{Read, Write};
    use std::os::fd::FromRawFd;
    // SAFETY: posix_openpt/grantpt/unlockpt/ptsname on a fd we own.
    let (mut master, slave) = unsafe {
        let m = libc::posix_openpt(libc::O_RDWR | libc::O_NOCTTY);
        assert!(m >= 0, "posix_openpt");
        assert_eq!(libc::grantpt(m), 0);
        assert_eq!(libc::unlockpt(m), 0);
        let name = std::ffi::CStr::from_ptr(libc::ptsname(m))
            .to_str()
            .unwrap()
            .to_string();
        let slave = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&name)
            .unwrap();
        (std::fs::File::from_raw_fd(m), slave)
    };
    let mut cmd = std::process::Command::new(bin);
    cmd.args(argv)
        .stdin(slave)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    for (k, v) in env {
        cmd.env(k, v);
    }
    let mut child = cmd.spawn().unwrap();
    let mut err = child.stderr.take().unwrap();
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    let prompt_owned = prompt.to_string();
    let reader = std::thread::spawn(move || {
        let mut got = Vec::new();
        let mut b = [0u8; 1];
        let mut sent = false;
        while err.read(&mut b).unwrap_or(0) == 1 {
            got.push(b[0]);
            if !sent && String::from_utf8_lossy(&got).contains(&prompt_owned) {
                let _ = tx.send(());
                sent = true;
            }
        }
        if !sent {
            let _ = tx.send(());
        }
        String::from_utf8_lossy(&got).into_owned()
    });
    rx.recv_timeout(std::time::Duration::from_secs(20))
        .expect("the prompt never appeared on stderr");
    master.write_all(typed).unwrap();
    let out = child.wait_with_output().unwrap();
    let stderr = reader.join().unwrap();
    // Everything the terminal would have DISPLAYED (the line discipline's
    // echo), read non-blocking now that the child is gone.
    // SAFETY: fcntl on the master fd we own.
    unsafe {
        use std::os::fd::AsRawFd;
        let fd = master.as_raw_fd();
        libc::fcntl(
            fd,
            libc::F_SETFL,
            libc::fcntl(fd, libc::F_GETFL) | libc::O_NONBLOCK,
        );
    }
    let mut shown = Vec::new();
    let mut buf = [0u8; 256];
    while let Ok(n) = master.read(&mut buf) {
        if n == 0 {
            break;
        }
        shown.extend_from_slice(&buf[..n]);
    }
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr,
        shown,
    )
}

/// F-687b ruling 3: on a terminal, prompt on stderr, echo off, read ONE line.
#[cfg(target_os = "linux")]
#[test]
fn a_terminal_gets_a_prompt_and_no_echo() {
    let bin = assert_cmd::cargo::cargo_bin("ms");
    let card = card_file();
    let path = card.path().to_str().unwrap().to_string();
    for pp in [&["--passphrase", "-"][..], &["--passphrase-stdin"][..]] {
        let mut argv = vec!["derive", "--in", path.as_str()];
        argv.extend(pp);
        let (code, stdout, stderr, shown) =
            run_on_a_terminal(&bin, &argv, &[], "Enter passphrase: ", b"TREZOR\n");
        assert_eq!(code, 0, "{pp:?}: {stderr}");
        assert_eq!(
            fingerprint(&stdout).as_deref(),
            Some("b4e3f5ed"),
            "{stdout}"
        );
        assert!(!stderr.contains("input will be visible"), "{stderr}");
        let shown = String::from_utf8_lossy(&shown);
        assert!(
            !shown.contains("TREZOR"),
            "the terminal echoed the passphrase: {shown:?}"
        );
    }
}

/// F-691: the flag-shape test is exact on `--passphrase`: `"- "` as a
/// separate argument is a usage error (64) as in mnemonic-toolkit, even under
/// the override; `--passphrase=- ` is the literal.
#[test]
fn a_dash_space_argument_is_a_usage_error_under_the_override() {
    let card = card_file();
    let out = Command::cargo_bin("ms")
        .unwrap()
        .arg("derive")
        .arg("--in")
        .arg(card.path())
        .args(["--allow-argv-secret", "--passphrase", "- "])
        .write_stdin("TREZOR")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(64));
    assert!(out.stdout.is_empty());
    // A LEADING space does not begin with `-`: the literal, as in the toolkit.
    let out = Command::cargo_bin("ms")
        .unwrap()
        .arg("derive")
        .arg("--in")
        .arg(card.path())
        .args(["--allow-argv-secret", "--passphrase", " -"])
        .output()
        .unwrap();
    assert_eq!(
        fingerprint(&String::from_utf8_lossy(&out.stdout)).as_deref(),
        Some("e20c1882")
    );
}
