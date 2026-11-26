//! What ms does when stdout cannot take its output (crash sweep F1 and the
//! family rule, grade-mk-crash-sweep-1 Q4; the ms/md output-error triage).
//!
//! - A reader that closed the pipe only stops the output: no text, no
//!   signal, the exit code is the verb's own.
//! - Any other write error (`/dev/full`, ENOSPC), from a verb or `--help`:
//!   "ms: cannot write the output: <error>" on stderr, and 32 (IO) is ADDED to
//!   the verb's own code (the constellation exit-code spec's R8): 0 becomes
//!   32, `repair`'s 20 becomes 52, a failing 2 becomes 34. 64 never combines.
//! - A stdout that cannot write (read-only, access mode 3, or closed at exec,
//!   `>&-`) is the same: EBADF, exit 32; a verb that writes nothing to stdout
//!   is not touched, and nothing meant for stdout lands in an `--out` file.
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
fn full_stdout_exits_32_with_the_line() {
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
        full_case(&a, 32);
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
        if o.status.code() != Some(32) {
            wrong.push(format!("{a:?} -> {:?}", o.status.code()));
        }
    }
    assert!(wrong.is_empty(), "stdout lost without exit 32: {wrong:#?}");
}

#[test]
fn full_stdout_after_a_repair_adds_32_to_20() {
    let a = s(&["repair", A, "--ms1", DAMAGED]);
    let o = ms().args(&a).output().unwrap();
    assert_eq!(o.status.code(), Some(20), "fixture: repair should exit 20");
    full_case(&a, 52);
}

#[test]
fn full_stdout_adds_32_to_a_failing_code() {
    // A --json failure prints its envelope on stdout; 32 is added to its 2.
    full_case(&s(&["decode", A, "--json", "ms1notastring"]), 34);
    // verify's phrase mismatch is 1, NO: the lost envelope makes it 33.
    let other = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";
    let a = s(&["verify", A, "--json", MS1, "--phrase", other]);
    let o = ms().args(&a).output().unwrap();
    assert_eq!(
        o.status.code(),
        Some(1),
        "fixture: a phrase mismatch exits 1"
    );
    full_case(&a, 33);
    // A reserved-tag (future format) ms1 under --json: the envelope is lost, 2 + 32.
    let a = s(&["verify", A, "--json", &reserved_tag_ms1()]);
    let o = ms().args(&a).output().unwrap();
    assert_eq!(o.status.code(), Some(2), "fixture: a future format exits 2");
    full_case(&a, 34);
    // A usage error in --json mode prints its envelope on stdout: 64 never
    // combines, so it stays.
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
/// then lost exits 32; the file stays.
#[test]
fn full_stdout_after_an_out_file_exits_32_and_keeps_the_file() {
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
        err.contains("THE PREIMAGE -- the secret -- is in "),
        "the stderr card is still printed: {err}"
    );
    full_case(&a, 32);
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
    closed_case(&s(&["repair", A, "--ms1", DAMAGED]), 20, false);
    // "x" is not a BIP-39 phrase: an invalid input, 2.
    closed_case(&s(&["verify", A, "--json", MS1, "--phrase", "x"]), 2, false);
}

#[test]
fn closed_reader_on_stdout_and_stderr_exits_the_verbs_code() {
    // encode and decode print stderr advisories after stdout; repair 20 prints
    // its UNVERIFIED line on stderr. Both streams go to a pipe with no reader.
    closed_case(&s(&["encode", A, "--phrase", PHRASE]), 0, true);
    closed_case(&s(&["decode", A, MS1]), 0, true);
    closed_case(&s(&["repair", A, "--ms1", DAMAGED]), 20, true);
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
fn read_only_stdout_exits_32_with_the_line() {
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
        assert_eq!(o.status.code(), Some(32), "{a:?}: {err}");
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
fn no_access_mode_stdout_exits_32_with_the_line() {
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
    assert_eq!(o.status.code(), Some(32), "{err}");
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

/// Runs ms with fd 1 really closed at exec (`>&-`). std's runtime plugs a
/// closed standard fd with `/dev/null` before `main`, so this lost the output
/// and exited 0 (Defect 1 of the 2026-11-26 exit-code survey).
fn on_closed_stdout(a: &[String]) -> Output {
    Command::new("sh")
        .arg("-c")
        .arg(r#"exec "$0" "$@" >&-"#)
        .arg(env!("CARGO_BIN_EXE_ms"))
        .args(a)
        .stderr(Stdio::piped())
        .output()
        .unwrap()
}

#[test]
fn closed_stdout_exits_32_with_the_line() {
    const BADF: &str = "ms: cannot write the output: Bad file descriptor (os error 9)";
    for a in [
        s(&["vectors"]),
        s(&["gui-schema"]),
        s(&["decode", A, MS1]),
        s(&["encode", A, "--phrase", PHRASE]),
        s(&["repair", A, "--ms1", DAMAGED]),
        s(&["--help"]),
        s(&["--version"]),
    ] {
        let o = on_closed_stdout(&a);
        let err = String::from_utf8_lossy(&o.stderr);
        assert!(!err.contains("panicked"), "{a:?}: {err}");
        // R8: 32 is added to the verb's own code (repair's 20 becomes 52).
        let want = if a[0] == "repair" { 52 } else { 32 };
        assert_eq!(o.status.code(), Some(want), "{a:?}: {err}");
        assert_eq!(err.lines().last(), Some(BADF), "{a:?}: {err}");
    }
}

/// With stdout closed, a verb whose output goes to `--out` succeeds and the
/// file holds exactly that output: no stdout write lands in it.
#[test]
fn closed_stdout_with_out_file_succeeds_and_nothing_goes_astray() {
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("card.ms1");
    let o = on_closed_stdout(&s(&[
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
    assert_eq!(
        std::fs::read_to_string(&f).unwrap().trim_end(),
        MS1,
        "the --out file"
    );

    // hashlock writes the plate to --out AND its `hash:` record to stdout:
    // the record is lost (exit 32, the line) and never reaches the file.
    let f = dir.path().join("preimage.ms1");
    let o = on_closed_stdout(&s(&[
        "hashlock",
        A,
        "--hashlock-phrase",
        "correct horse battery staple long enough",
        "--kind",
        "sha256",
        "--out",
        f.to_str().unwrap(),
    ]));
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(32), "{err}");
    assert_eq!(
        err.lines().last(),
        Some("ms: cannot write the output: Bad file descriptor (os error 9)"),
        "{err}"
    );
    let body = std::fs::read_to_string(&f).unwrap();
    assert!(body.starts_with("ms10hash"), "the --out file: {body:?}");
    assert!(
        !body.contains("hash:"),
        "a stdout record went astray: {body:?}"
    );
}
