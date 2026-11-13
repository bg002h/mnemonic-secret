//! What ms does when stdout cannot take its output (crash sweep F1 and the
//! family rule, grade-mk-crash-sweep-1 Q4; the ms/md output-error triage).
//!
//! - A reader that closed the pipe only stops the output: no text, no
//!   signal, the exit code is the verb's own.
//! - Any other write error (`/dev/full`, ENOSPC), from a verb or `--help`:
//!   "ms: cannot write the output: <error>" on stderr, exit 1; a verb that
//!   was already failing keeps its code. A success code (0, and `repair`'s
//!   4 = VERIFY-ME candidate) becomes 1.
//! - No file of the crate but `src/sink.rs` names stdout.
#![cfg(target_os = "linux")]

use std::fs::OpenOptions;
use std::os::fd::{FromRawFd, OwnedFd};
use std::os::unix::process::ExitStatusExt;
use std::process::{Command, Output, Stdio};

const A: &str = "--allow-argv-secret";
const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
// One substitution away from MS1: repair corrects it and exits 4.
const DAMAGED: &str = "ms10entrsqpqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
// A hashlock preimage plate: `decode` prints it through its own stdout writer.
const PREIMAGE: &str =
    "ms10hashsq02nfauqxa69u0g3zxl250sejulueyjpjd2xygd5snwfa2mjsrphxdd85w8w5fxwvh";
const SHARE_A: &str = "ms12fj8uqtju0yavfp4glm0acyu6cvquey82qm0j3ar9f2fpxt";
const SHARE_B: &str = "ms12fj8upvkrzw5jthgunaz5dwr2djqr6w7mqh6qvjfrcsxywt";
const FULL: &str = "ms: cannot write the output: No space left on device (os error 28)";

fn reserved_tag_ms1() -> String {
    use ms_codec::codex32::{Codex32String, Fe};
    let mut data = vec![0x00u8];
    data.extend_from_slice(&[0xAAu8; 16]);
    Codex32String::from_seed("ms", 0, "seed", Fe::S, &data)
        .unwrap()
        .to_string()
}

fn ms() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ms"))
}

fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

fn closed_pipe() -> OwnedFd {
    let mut fds = [0i32; 2];
    assert_eq!(unsafe { libc::pipe(fds.as_mut_ptr()) }, 0);
    unsafe { libc::close(fds[0]) }; // the reader is gone before ms starts
    unsafe { OwnedFd::from_raw_fd(fds[1]) }
}

fn on_full(a: &[String]) -> Output {
    let full = OpenOptions::new().write(true).open("/dev/full").unwrap();
    ms().args(a)
        .stdout(Stdio::from(full))
        .stderr(Stdio::piped())
        .output()
        .unwrap()
}

fn on_closed(a: &[String], stderr_too: bool) -> Output {
    let err = if stderr_too {
        Stdio::from(closed_pipe())
    } else {
        Stdio::piped()
    };
    ms().args(a)
        .stdout(Stdio::from(closed_pipe()))
        .stderr(err)
        .output()
        .unwrap()
}

fn full_case(a: &[String], code: i32) {
    let o = on_full(a);
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(!err.contains("panicked"), "{a:?}: panic: {err}");
    assert_eq!(
        o.status.code(),
        Some(code),
        "{a:?}: status {:?}: {err}",
        o.status
    );
    assert_eq!(err.lines().last(), Some(FULL), "{a:?}: stderr: {err}");
}

fn closed_case(a: &[String], code: i32, stderr_too: bool) {
    let o = on_closed(a, stderr_too);
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(
        o.status.signal(),
        None,
        "{a:?}: killed by a signal: {:?}",
        o.status
    );
    assert_eq!(
        o.status.code(),
        Some(code),
        "{a:?}: status {:?}: {err}",
        o.status
    );
    assert!(
        !err.contains("panicked") && !err.contains("cannot write"),
        "{a:?}: {err}"
    );
}

#[test]
fn full_stdout_exits_1_with_the_line() {
    for a in [
        s(&["gui-schema"]),
        s(&["vectors"]),
        s(&["vectors", "--pretty"]),
        s(&["encode", A, "--phrase", PHRASE]),
        s(&["encode", A, "--phrase", PHRASE, "--json"]),
        s(&["decode", A, MS1]),
        s(&["decode", A, "--json", MS1]),
        s(&["inspect", A, MS1]),
        s(&["inspect", A, "--json", MS1]),
        s(&["inspect", A, SHARE_A]),
        s(&["verify", A, MS1]),
        s(&["verify", A, "--json", MS1]),
        s(&["verify", A, MS1, "--phrase", PHRASE]),
        s(&["repair", A, "--ms1", MS1]),
        s(&["repair", A, "--ms1", MS1, "--json"]),
        s(&["split", A, "-k", "2", "-n", "3", "--phrase", PHRASE]),
        s(&[
            "split", A, "-k", "2", "-n", "3", "--phrase", PHRASE, "--json",
        ]),
        s(&["combine", A, SHARE_A, SHARE_B]),
        s(&["combine", A, SHARE_A, SHARE_B, "--to", "entropy"]),
        s(&["combine", A, SHARE_A, SHARE_B, "--to", "ms1"]),
        s(&["combine", A, SHARE_A, SHARE_B, "--json"]),
        s(&["derive", A, "--phrase", PHRASE, "--template", "bip84"]),
        s(&["derive", A, "--phrase", PHRASE, "--json"]),
        s(&[
            "hashlock",
            A,
            "--hashlock-phrase",
            "correct horse battery staple long enough",
            "--kind",
            "sha256",
        ]),
        s(&[
            "hashlock",
            A,
            "--hashlock-phrase",
            "correct horse battery staple long enough",
            "--kind",
            "sha256",
            "--json",
        ]),
        s(&["--help"]),
        s(&["--version"]),
        s(&["decode", "--help"]),
        s(&["help"]),
    ] {
        full_case(&a, 1);
    }
}

/// Verbs that swallowed the write error (`.ok()` / `let _ =` on a direct
/// `std::io::stdout()` writer, or clap's `.print().ok()`) and exited 0 with the
/// output lost, one case each so the first failure does not mask the rest.
#[test]
fn full_stdout_is_not_a_silent_success() {
    let h = "correct horse battery staple long enough";
    let cases = [
        s(&["split", A, "-k", "2", "-n", "3", "--phrase", PHRASE]),
        s(&["derive", A, "--phrase", PHRASE, "--template", "bip84"]),
        s(&["hashlock", A, "--hashlock-phrase", h, "--kind", "sha256"]),
        s(&[
            "hashlock",
            A,
            "--hashlock-phrase",
            h,
            "--kind",
            "sha256",
            "--json",
        ]),
        s(&["decode", A, PREIMAGE]),
        s(&["decode", A, "--json", PREIMAGE]),
        s(&["decode", A, MS1]),
        s(&["--help"]),
        s(&["--version"]),
    ];
    let mut wrong = Vec::new();
    for a in &cases {
        let o = on_full(a);
        if o.status.code() != Some(1) {
            wrong.push(format!("{a:?} -> {:?}", o.status.code()));
        }
    }
    assert!(wrong.is_empty(), "stdout lost without exit 1: {wrong:#?}");
}

#[test]
fn full_stdout_after_a_repair_exits_1_not_4() {
    let a = s(&["repair", A, "--ms1", DAMAGED]);
    let o = ms().args(&a).output().unwrap();
    assert_eq!(o.status.code(), Some(4), "fixture: repair should exit 4");
    full_case(&a, 1);
}

#[test]
fn full_stdout_keeps_a_failing_code() {
    // A --json failure prints its envelope on stdout; the failing code stays.
    full_case(&s(&["decode", A, "--json", "ms1notastring"]), 1);
    // verify's phrase mismatch is a failing 4 (an Err), unlike repair's
    // candidate 4 (an Ok): it stays 4.
    let other = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";
    let a = s(&["verify", A, "--json", MS1, "--phrase", other]);
    let o = ms().args(&a).output().unwrap();
    assert_eq!(
        o.status.code(),
        Some(4),
        "fixture: a phrase mismatch exits 4"
    );
    full_case(&a, 4);
    // A reserved-tag (future format) ms1: verify's text OK line is lost, the 3 stays.
    let a = s(&["verify", A, &reserved_tag_ms1()]);
    let o = ms().args(&a).output().unwrap();
    assert_eq!(o.status.code(), Some(3), "fixture: a future format exits 3");
    full_case(&a, 3);
    // A usage error in --json mode prints its envelope on stdout: 64 stays.
    let a = s(&["hashlock", "--random", "--json"]);
    let o = ms().args(&a).output().unwrap();
    assert_eq!(o.status.code(), Some(64), "fixture: {o:?}");
    assert!(
        String::from_utf8_lossy(&o.stdout).contains("\"Usage\""),
        "fixture: {o:?}"
    );
    full_case(&a, 64);
}

/// A verb whose `--out FILE` artifact was written and whose stdout record was
/// then lost exits 1; the file stays.
#[test]
fn full_stdout_after_an_out_file_exits_1_and_keeps_the_file() {
    let f = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("output_errors_hashlock.ms1");
    let _ = std::fs::remove_file(&f);
    let a = s(&[
        "hashlock",
        A,
        "--hashlock-phrase",
        "correct horse battery staple long enough",
        "--kind",
        "sha256",
        "--out",
        f.to_str().unwrap(),
    ]);
    let o = on_full(&a);
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        err.contains("THIS CARD CARRIES THE PREIMAGE"),
        "the stderr card is still printed: {err}"
    );
    full_case(&a, 1);
    let body = std::fs::read_to_string(&f).unwrap();
    assert!(body.starts_with("ms10hash"), "the --out file: {body:?}");
}

#[test]
fn closed_reader_exits_with_the_verbs_code() {
    for a in [
        s(&["gui-schema"]),
        s(&["vectors"]),
        s(&["vectors", "--pretty"]),
        s(&["encode", A, "--phrase", PHRASE]),
        s(&["decode", A, MS1]),
        s(&["derive", A, "--phrase", PHRASE, "--template", "bip84"]),
        s(&["split", A, "-k", "2", "-n", "3", "--phrase", PHRASE]),
        s(&[
            "hashlock",
            A,
            "--hashlock-phrase",
            "correct horse battery staple long enough",
            "--kind",
            "sha256",
        ]),
        s(&["--help"]),
        s(&["--version"]),
    ] {
        closed_case(&a, 0, false);
    }
    closed_case(&s(&["repair", A, "--ms1", DAMAGED]), 4, false);
    closed_case(&s(&["verify", A, "--json", MS1, "--phrase", "x"]), 1, false);
}

#[test]
fn closed_reader_on_stdout_and_stderr_exits_the_verbs_code() {
    // encode and decode print stderr advisories after stdout; repair 4 prints
    // its UNVERIFIED line on stderr. Both streams go to a pipe with no reader.
    closed_case(&s(&["encode", A, "--phrase", PHRASE]), 0, true);
    closed_case(&s(&["decode", A, MS1]), 0, true);
    closed_case(&s(&["repair", A, "--ms1", DAMAGED]), 4, true);
}

#[test]
fn only_the_sink_names_stdout() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut bad = Vec::new();
    let mut stack = vec![dir];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "rs") && !p.ends_with("src/sink.rs") {
                let t = std::fs::read_to_string(&p).unwrap();
                for n in ["stdout(", "std::print", "std::eprint", "core::print"] {
                    if t.contains(n) {
                        bad.push(format!("{}: {n}", p.display()));
                    }
                }
            }
        }
    }
    assert!(bad.is_empty(), "stdout named outside src/sink.rs: {bad:?}");
}

/// std's stdout reports EBADF as a successful write, so a stdout opened
/// read-only (`ms vectors 1</dev/null`) lost the output and exited 0.
#[test]
fn read_only_stdout_exits_1_with_the_line() {
    const BADF: &str = "ms: cannot write the output: Bad file descriptor (os error 9)";
    let on_read_only = |a: &[String]| {
        let ro = std::fs::File::open("/dev/null").unwrap();
        ms().args(a)
            .stdout(Stdio::from(ro))
            .stderr(Stdio::piped())
            .output()
            .unwrap()
    };
    for a in [
        s(&["vectors"]),
        s(&["gui-schema"]),
        s(&["decode", A, MS1]),
        s(&["--help"]),
        s(&["--version"]),
    ] {
        let o = on_read_only(&a);
        let err = String::from_utf8_lossy(&o.stderr);
        assert_eq!(o.status.code(), Some(1), "{a:?}: {err}");
        assert_eq!(err.lines().last(), Some(BADF), "{a:?}: {err}");
    }
    // A verb that writes nothing to stdout keeps its code and prints no line.
    let f = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("output_errors_ro.ms1");
    let _ = std::fs::remove_file(&f);
    let o = on_read_only(&s(&[
        "encode",
        A,
        "--phrase",
        PHRASE,
        "--out",
        f.to_str().unwrap(),
    ]));
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(0), "{err}");
    assert!(!err.contains("cannot write"), "{err}");
}

/// Linux accepts access mode 3 (`O_WRONLY | O_RDWR`): the fd can neither read
/// nor write, and a write gets EBADF, which std reports as a success.
#[test]
fn no_access_mode_stdout_exits_1_with_the_line() {
    const BADF: &str = "ms: cannot write the output: Bad file descriptor (os error 9)";
    let fd = unsafe { libc::open(c"/dev/null".as_ptr(), libc::O_ACCMODE) };
    assert!(fd >= 0);
    let o = ms()
        .args(["vectors"])
        .stdout(Stdio::from(unsafe { OwnedFd::from_raw_fd(fd) }))
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(1), "{err}");
    assert_eq!(err.lines().last(), Some(BADF), "{err}");
}

/// A stdout opened read-write (a terminal, `1<>file`) is writable: the probe
/// must not treat it as read-only.
#[test]
fn read_write_stdout_keeps_the_output_and_exits_0() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("out");
    let rw = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&p)
        .unwrap();
    let o = ms()
        .arg("gui-schema")
        .stdout(Stdio::from(rw))
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(0), "{err}");
    assert!(!err.contains("cannot write"), "{err}");
    assert!(std::fs::metadata(&p).unwrap().len() > 0, "output lost");
}
