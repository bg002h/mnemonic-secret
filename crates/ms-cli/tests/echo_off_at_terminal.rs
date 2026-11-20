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
#[allow(clippy::zombie_processes)] // every panic path is a failed test
fn on_pty(args: &[&str], typed: &str, eot: bool) -> (String, String, Option<i32>) {
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
    (
        String::from_utf8_lossy(&shown).into_owned(),
        out,
        status.and_then(|s| s.code()),
    )
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

/// MUTATION: drop the echo-off guard `let _echo = (crate::passphrase_input::path_is_stdin(path) && io::stdin().is_terminal()).then(crate::passphrase_input::EchoOff::new);`
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
