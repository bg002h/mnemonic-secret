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

/// What a pty run produced.
#[cfg(target_os = "linux")]
#[allow(dead_code)]
struct PtyRun {
    code: Option<i32>,
    /// The signal that killed the child, if one did.
    signal: Option<i32>,
    stdout: String,
    stderr: String,
    /// Everything the terminal DISPLAYED (the line discipline's echo).
    shown: Vec<u8>,
    /// Was ECHO on in the terminal's mode after the child was gone?
    echo_after: bool,
    /// F-687c: input still queued on the terminal after the child was gone,
    /// i.e. what the shell would read next and run.
    left_for_shell: Vec<u8>,
}

/// Run `bin argv` on a fresh pty that is the child's CONTROLLING terminal
/// (setsid + TIOCSCTTY, so a typed Ctrl-C really sends SIGINT). Waits for
/// `prompt` on stderr, then types `typed` on the master. On a timeout the
/// child is KILLED before the test fails, so no process is left blocked on
/// the pty (review M5).
#[cfg(target_os = "linux")]
fn run_on_a_terminal(
    bin: &std::path::Path,
    argv: &[&str],
    env: &[(&str, &str)],
    prompt: &str,
    typed: &[u8],
) -> PtyRun {
    run_on_a_terminal_writes(bin, argv, env, prompt, &[typed])
}

/// [`run_on_a_terminal`], typing each of `writes` as a separate write (the
/// second and later ones are type-ahead after the first).
#[cfg(target_os = "linux")]
fn run_on_a_terminal_writes(
    bin: &std::path::Path,
    argv: &[&str],
    env: &[(&str, &str)],
    prompt: &str,
    writes: &[&[u8]],
) -> PtyRun {
    use std::io::{Read, Write};
    use std::os::fd::{AsRawFd, FromRawFd};
    use std::os::unix::process::{CommandExt, ExitStatusExt};
    // SAFETY: posix_openpt/grantpt/unlockpt/ptsname on a fd we own.
    let (mut master, name) = unsafe {
        let m = libc::posix_openpt(libc::O_RDWR | libc::O_NOCTTY);
        assert!(m >= 0, "posix_openpt");
        assert_eq!(libc::grantpt(m), 0);
        assert_eq!(libc::unlockpt(m), 0);
        let name = std::ffi::CStr::from_ptr(libc::ptsname(m))
            .to_str()
            .unwrap()
            .to_string();
        (std::fs::File::from_raw_fd(m), name)
    };
    let open_slave = || {
        std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NOCTTY)
            .open(&name)
            .unwrap()
    };
    use std::os::unix::fs::OpenOptionsExt;
    // Our own handle on the slave, to read the terminal mode back afterwards.
    let keep = open_slave();
    let mut cmd = std::process::Command::new(bin);
    cmd.args(argv)
        .stdin(open_slave())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    for (k, v) in env {
        cmd.env(k, v);
    }
    // SAFETY: async-signal-safe calls only, between fork and exec.
    unsafe {
        cmd.pre_exec(|| {
            if libc::setsid() < 0 || libc::ioctl(0, libc::TIOCSCTTY as _, 0) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
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
        String::from_utf8_lossy(&got).into_owned()
    });
    if rx.recv_timeout(std::time::Duration::from_secs(20)).is_err() {
        let _ = child.kill();
        let _ = child.wait();
        panic!("the prompt {prompt:?} never appeared on stderr (child killed)");
    }
    for w in writes {
        // An EMPTY write is a 40 ms pause: it lets the child finish reading
        // the line and enter the drain before the next write arrives.
        if w.is_empty() {
            std::thread::sleep(std::time::Duration::from_millis(40));
            continue;
        }
        master.write_all(w).unwrap();
        master.flush().unwrap();
    }
    let out = child.wait_with_output().unwrap();
    let stderr = reader.join().unwrap();
    // SAFETY: fcntl/tcgetattr on fds we own.
    let echo_after = unsafe {
        let fd = master.as_raw_fd();
        libc::fcntl(
            fd,
            libc::F_SETFL,
            libc::fcntl(fd, libc::F_GETFL) | libc::O_NONBLOCK,
        );
        let mut t: libc::termios = std::mem::zeroed();
        assert_eq!(libc::tcgetattr(keep.as_raw_fd(), &mut t), 0, "tcgetattr");
        t.c_lflag & libc::ECHO != 0
    };
    // What would the shell read next? Switch our slave handle to
    // non-blocking, non-canonical reads and take everything queued.
    // SAFETY: termios/read on the slave fd we own.
    let left_for_shell = unsafe {
        let fd = keep.as_raw_fd();
        let mut t: libc::termios = std::mem::zeroed();
        libc::tcgetattr(fd, &mut t);
        t.c_lflag &= !libc::ICANON;
        t.c_cc[libc::VMIN] = 0;
        t.c_cc[libc::VTIME] = 0;
        libc::tcsetattr(fd, libc::TCSANOW, &t);
        let mut left = Vec::new();
        let mut b = [0u8; 256];
        loop {
            let n = libc::read(fd, b.as_mut_ptr().cast(), b.len());
            if n <= 0 {
                break;
            }
            left.extend_from_slice(&b[..n as usize]);
        }
        left
    };
    let mut shown = Vec::new();
    let mut buf = [0u8; 256];
    while let Ok(n) = master.read(&mut buf) {
        if n == 0 {
            break;
        }
        shown.extend_from_slice(&buf[..n]);
    }
    PtyRun {
        code: out.status.code(),
        signal: out.status.signal(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr,
        shown,
        echo_after,
        left_for_shell,
    }
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
        let r = run_on_a_terminal(&bin, &argv, &[], "Enter passphrase: ", b"TREZOR\n");
        assert_eq!(r.code, Some(0), "{pp:?}: {}", r.stderr);
        assert_eq!(
            fingerprint(&r.stdout).as_deref(),
            Some("b4e3f5ed"),
            "{}",
            r.stdout
        );
        assert!(!r.stderr.contains("input will be visible"), "{}", r.stderr);
        let shown = String::from_utf8_lossy(&r.shown);
        assert!(
            !shown.contains("TREZOR"),
            "the terminal echoed the passphrase: {shown:?}"
        );
        // Review M4: the terminal mode is RESTORED after a normal exit.
        assert!(r.echo_after, "{pp:?}: echo left OFF after the run");
    }
}

/// Ctrl-C at the prompt: dies of SIGINT with echo back on. Ctrl-D at an empty
/// prompt: the empty warning starts on its own line (review N1).
#[cfg(target_os = "linux")]
#[test]
fn ctrl_c_and_ctrl_d_at_the_prompt_leave_a_working_terminal() {
    let bin = assert_cmd::cargo::cargo_bin("ms");
    let card = card_file();
    let path = card.path().to_str().unwrap().to_string();
    let argv = ["derive", "--in", path.as_str(), "--passphrase", "-"];
    let r = run_on_a_terminal(&bin, &argv, &[], "Enter passphrase: ", b"TRE\x03");
    assert_eq!(
        r.signal,
        Some(libc::SIGINT),
        "code {:?}: {}",
        r.code,
        r.stderr
    );
    assert!(r.stdout.is_empty());
    assert!(r.echo_after, "Ctrl-C left echo OFF");
    let r = run_on_a_terminal(&bin, &argv, &[], "Enter passphrase: ", b"\x04");
    assert_eq!(r.code, Some(0), "{}", r.stderr);
    assert_eq!(fingerprint(&r.stdout).as_deref(), Some("73c5da0a"));
    assert!(
        r.stderr
            .contains("Enter passphrase: \nwarning: --passphrase from stdin is empty"),
        "{:?}",
        r.stderr
    );
    assert!(r.echo_after);
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
    // Review N2: the message shows the value as typed, not trimmed.
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("was given \"- \""),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
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

/// F-687c (operator ruling 2026-09-25): on a terminal, input pending after the
/// prompted line -- a multi-line paste or type-ahead -- is read, shown on
/// stderr and discarded, so the shell never runs it.
#[cfg(target_os = "linux")]
#[test]
fn pasted_and_typed_ahead_lines_are_drained_and_shown() {
    let bin = assert_cmd::cargo::cargo_bin("ms");
    let card = card_file();
    let path = card.path().to_str().unwrap().to_string();
    let argv = ["derive", "--in", path.as_str(), "--passphrase", "-"];
    let p = "Enter passphrase: ";
    let label = "note: discarded";
    for (what, writes, lines, text) in [
        // F-687d: each discarded line is MASKED (<= 8 chars shown as is,
        // two-space indent); see drain_preview.json.
        ("paste", vec![&b"TREZOR\nline2\nline3\n"[..]], 2usize, "  line2\n  line3"),
        ("type-ahead", vec![&b"TREZOR\n"[..], b"ls -la\n"], 1, "  ls -la"),
        ("partial line", vec![&b"TREZOR\npartial"[..]], 1, "  partial"),
        (
            "a pasted 12-word seed",
            vec![&b"TREZOR\nabandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about\n"[..]],
            1,
            "  abandon \u{2026} (12 words, 93 chars)",
        ),
        ("an escape sequence", vec![&b"TREZOR\n\x1b[2J\n"[..]], 1, "  ?"),
        (
            "multi-byte text",
            vec!["TREZOR\n\u{43f}\u{430}\u{440}\u{43e}\u{43b}\u{44c}-\u{441}\u{435}\u{43a}\u{440}\u{435}\u{442}\n".as_bytes()],
            1,
            "  \u{43f}\u{430}\u{440}\u{43e}\u{43b}\u{44c}-\u{441}\u{2026} (1 word, 13 chars)",
        ),
    ] {
        let r = run_on_a_terminal_writes(&bin, &argv, &[], p, &writes);
        assert_eq!(r.code, Some(0), "{what}: {}", r.stderr);
        assert_eq!(
            fingerprint(&r.stdout).as_deref(),
            Some("b4e3f5ed"),
            "{what}"
        );
        assert!(
            r.stderr.contains(&format!(
                "{label} {lines} line(s) typed after the passphrase (not run, not used):\n{text}\n"
            )),
            "{what}: {:?}",
            r.stderr
        );
        assert!(
            r.left_for_shell.is_empty(),
            "{what}: left for the shell: {:?}",
            r.left_for_shell
        );
        // Masked: never the seed's tail, never a raw escape byte.
        assert!(!r.stderr.contains("about"), "{what}: the full line leaked: {}", r.stderr);
        assert!(!r.stderr.contains('\u{1b}'), "{what}: a raw ESC reached stderr");
        assert!(r.echo_after, "{what}: echo left off");
    }
    let r = run_on_a_terminal(&bin, &argv, &[], p, b"TREZOR\n");
    assert_eq!(r.code, Some(0), "{}", r.stderr);
    assert!(!r.stderr.contains(label), "{}", r.stderr);
    assert!(r.left_for_shell.is_empty());
    assert!(r.echo_after);
    // A pipe is unchanged: no drain, no note.
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(argv)
        .write_stdin("TREZOR\nline2\n")
        .output()
        .unwrap();
    assert!(!String::from_utf8_lossy(&out.stderr).contains(label));
}

/// F-687c: a Ctrl-C arriving DURING the drain (40 ms after Enter; the drain
/// listens for 100 ms) still restores the terminal and exits by SIGINT. With
/// ISIG off in the drain the ^C would be read as data and the run would exit
/// 0 -- which is what makes this test fail on that mutation.
#[cfg(target_os = "linux")]
#[test]
fn ctrl_c_during_the_drain_leaves_a_working_terminal() {
    let (bin, argv, env) = drain_target();
    let argv: Vec<&str> = argv.iter().map(String::as_str).collect();
    let env: Vec<(&str, &str)> = env.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    let r = run_on_a_terminal_writes(
        &bin,
        &argv,
        &env,
        "Enter passphrase: ",
        &[b"TREZOR\n", b"", b"\x03"],
    );
    assert_eq!(
        r.signal,
        Some(libc::SIGINT),
        "code {:?}: {}",
        r.code,
        r.stderr
    );
    assert!(r.stdout.is_empty(), "{}", r.stdout);
    assert!(r.echo_after, "a signal during the drain left echo OFF");
}

#[cfg(target_os = "linux")]
fn drain_target() -> (std::path::PathBuf, Vec<String>, Vec<(String, String)>) {
    // The card file must outlive the run: leak it for the test process.
    let card = Box::leak(Box::new(card_file()));
    (
        assert_cmd::cargo::cargo_bin("ms"),
        [
            "derive",
            "--in",
            card.path().to_str().unwrap(),
            "--passphrase",
            "-",
        ]
        .iter()
        .map(|x| x.to_string())
        .collect(),
        vec![],
    )
}
