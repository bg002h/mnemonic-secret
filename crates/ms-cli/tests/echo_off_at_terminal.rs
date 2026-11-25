//! A secret typed on a terminal must not be echoed back (FOLLOWUPS
//! `hashlock-phrase-stdin-echoes-the-phrase-at-a-terminal`). These tests drive
//! the real binary on a pty and read what the terminal itself displays (the
//! master side), which is exactly what a shoulder-surfer or scrollback sees.

#![cfg(unix)]

use std::io::{Read, Write};
use std::os::fd::FromRawFd;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const PHRASE: &str = "correct horse battery staple";
const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

/// Run `ms args` with stdin and stderr on a pty, type `typed` (then EOT when
/// `eot`), and return (everything the terminal displayed, stdout, exit code).
fn on_pty(args: &[&str], typed: &str, eot: bool) -> (String, String, Option<i32>) {
    let r = on_pty_full(args, typed, eot);
    (r.shown, r.stdout, r.code)
}

/// What a pty run shows: terminal output, stdout, exit code, and whether the
/// pty's ECHO bit is on after the child exited (read from the master side,
/// which reports the slave's termios on Linux).
struct PtyRun {
    shown: String,
    stdout: String,
    code: Option<i32>,
    echo_after: bool,
}

#[allow(clippy::zombie_processes)] // every panic path is a failed test
fn on_pty_full(args: &[&str], typed: &str, eot: bool) -> PtyRun {
    // SAFETY: plain pty setup; every fd is checked and owned below.
    let (master, slave) = unsafe {
        let m = libc::posix_openpt(libc::O_RDWR | libc::O_NOCTTY);
        assert!(m >= 0, "posix_openpt");
        assert_eq!(libc::grantpt(m), 0);
        assert_eq!(libc::unlockpt(m), 0);
        let name = libc::ptsname(m);
        assert!(!name.is_null());
        let s = libc::open(name, libc::O_RDWR | libc::O_NOCTTY);
        assert!(s >= 0, "open slave");
        (m, s)
    };
    let slave_in = unsafe { Stdio::from(std::fs::File::from_raw_fd(libc::dup(slave))) };
    let slave_err = unsafe { Stdio::from(std::fs::File::from_raw_fd(libc::dup(slave))) };
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("ms"));
    cmd.args(args)
        .stdin(slave_in)
        .stderr(slave_err)
        .stdout(Stdio::piped());
    // A new session, so the slave can be the child's controlling terminal.
    unsafe {
        cmd.pre_exec(|| {
            libc::setsid();
            // The slave becomes the controlling terminal, so `/dev/tty` opens.
            libc::ioctl(0, libc::TIOCSCTTY as _, 0);
            Ok(())
        });
    }
    let mut child = cmd.spawn().unwrap();
    unsafe { libc::close(slave) };
    let mut master = unsafe { std::fs::File::from_raw_fd(master) };
    // Give the child time to put the terminal in its reading mode.
    std::thread::sleep(Duration::from_millis(700));
    master.write_all(typed.as_bytes()).unwrap();
    if eot {
        std::thread::sleep(Duration::from_millis(300));
        master.write_all(&[0x04]).unwrap();
    }
    let mut shown = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut stdout = child.stdout.take().unwrap();
    let mut status = None;
    // Non-blocking read of the master until the child exits.
    unsafe {
        use std::os::fd::AsRawFd;
        let fd = master.as_raw_fd();
        let fl = libc::fcntl(fd, libc::F_GETFL);
        libc::fcntl(fd, libc::F_SETFL, fl | libc::O_NONBLOCK);
    }
    while Instant::now() < deadline {
        let mut buf = [0u8; 4096];
        match master.read(&mut buf) {
            Ok(n) if n > 0 => shown.extend_from_slice(&buf[..n]),
            _ => {}
        }
        if let Some(s) = child.try_wait().unwrap() {
            status = Some(s);
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    if status.is_none() {
        let _ = child.kill();
        let _ = child.wait();
    }
    loop {
        let mut buf = [0u8; 4096];
        match master.read(&mut buf) {
            Ok(n) if n > 0 => shown.extend_from_slice(&buf[..n]),
            _ => break,
        }
    }
    let mut out = String::new();
    let _ = stdout.read_to_string(&mut out);
    let echo_after = {
        use std::os::fd::AsRawFd;
        // SAFETY: tcgetattr on the open master fd into a zeroed termios.
        let mut t: libc::termios = unsafe { std::mem::zeroed() };
        assert_eq!(unsafe { libc::tcgetattr(master.as_raw_fd(), &mut t) }, 0);
        t.c_lflag & libc::ECHO != 0
    };
    PtyRun {
        shown: String::from_utf8_lossy(&shown).into_owned(),
        stdout: out,
        code: status.and_then(|s| s.code()),
        echo_after,
    }
}

/// MUTATION: drop the echo-off guard in `read_phrase_stdin` -> the terminal
/// displays the phrase as it is typed and this fails.
#[test]
fn hashlock_phrase_stdin_does_not_echo_at_a_terminal() {
    let (shown, stdout, code) = on_pty(
        &["hashlock", "--hashlock-phrase-stdin"],
        &format!("{PHRASE}\n"),
        false,
    );
    assert_eq!(code, Some(0), "shown:\n{shown}");
    assert!(stdout.contains("hash:"), "stdout:\n{stdout}");
    assert!(
        !shown.contains(PHRASE),
        "the terminal displayed the phrase:\n{shown}"
    );
}

/// MUTATION: drop the echo-off guard in `parse::read_stdin` -> the ms1 string
/// typed after `-` is displayed and this fails.
#[test]
fn ms1_read_from_stdin_does_not_echo_at_a_terminal() {
    let (shown, stdout, code) = on_pty(&["decode", "-"], &format!("{MS1}\n"), true);
    assert_eq!(code, Some(0), "shown:\n{shown}");
    assert!(!stdout.is_empty());
    assert!(
        !shown.contains(MS1),
        "the terminal displayed the ms1 string:\n{shown}"
    );
}

/// MUTATION: drop the echo-off guard `let _echo = ...` (the `is_terminal()` branch)
/// in `parse::read_in_file` -> the ms1 string typed into `--in /dev/stdin` is
/// displayed and this fails.
#[test]
fn ms1_read_in_dev_stdin_does_not_echo_at_a_terminal() {
    let (shown, stdout, code) =
        on_pty(&["decode", "--in", "/dev/stdin"], &format!("{MS1}\n"), true);
    assert_eq!(code, Some(0), "shown:\n{shown}");
    assert!(!stdout.is_empty());
    assert!(
        !shown.contains(MS1),
        "the terminal displayed the ms1 string:\n{shown}"
    );
}

/// MUTATION: `--in /dev/tty` is not covered by the echo-off guard in
/// `parse::read_in_file` -> the ms1 string typed on the terminal is displayed.
#[test]
fn ms1_read_in_dev_tty_does_not_echo_at_a_terminal() {
    let (shown, stdout, code) = on_pty(&["decode", "--in", "/dev/tty"], &format!("{MS1}\n"), true);
    assert_eq!(code, Some(0), "shown:\n{shown}");
    assert!(!stdout.is_empty());
    assert!(
        !shown.contains(MS1),
        "the terminal displayed the ms1 string:\n{shown}"
    );
}

/// The ECHO bit is back on after `--in /dev/tty` was read and failed to parse.
#[test]
fn echo_is_restored_after_a_bad_ms1_on_dev_tty() {
    let r = on_pty_full(&["decode", "--in", "/dev/tty"], "not-an-ms1-string\n", true);
    assert_ne!(r.code, Some(0), "shown:\n{}", r.shown);
    assert!(
        r.echo_after,
        "ECHO still off after exit; shown:\n{}",
        r.shown
    );
}

/// Restore after an error exit: a bad ms1 typed at `decode -`.
/// MUTATION: drop the guard's restore in `EchoOff::drop` -> ECHO stays off.
#[test]
fn echo_is_restored_after_a_bad_ms1_on_decode_stdin() {
    let r = on_pty_full(&["decode", "-"], "not-an-ms1-string\n", true);
    assert_ne!(r.code, Some(0), "shown:\n{}", r.shown);
    assert!(
        !r.shown.contains("not-an-ms1-string"),
        "echoed:\n{}",
        r.shown
    );
    assert!(
        r.echo_after,
        "ECHO still off after exit; shown:\n{}",
        r.shown
    );
}

/// `combine -` at a terminal: nothing typed is displayed, and ECHO is back
/// on after the (failing) read.
#[test]
fn combine_stdin_does_not_echo_and_restores_echo() {
    let r = on_pty_full(&["combine", "-"], "not-a-share-string\n", true);
    assert_ne!(r.code, Some(0), "shown:\n{}", r.shown);
    assert!(
        !r.shown.contains("not-a-share-string"),
        "echoed:\n{}",
        r.shown
    );
    assert!(
        r.echo_after,
        "ECHO still off after exit; shown:\n{}",
        r.shown
    );
}

/// The non-tty path: a piped stdin reads and decodes as before, with no pty
/// involved and no echo notice on stderr.
#[test]
fn decode_stdin_pipe_is_unchanged_without_a_terminal() {
    let o = assert_cmd::Command::cargo_bin("ms")
        .unwrap()
        .args(["decode", "-"])
        .write_stdin(format!("{MS1}\n"))
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(0));
    assert!(!o.stdout.is_empty());
    assert!(
        !String::from_utf8_lossy(&o.stderr).contains("echo"),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
}
