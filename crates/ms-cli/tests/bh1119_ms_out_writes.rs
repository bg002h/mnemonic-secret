//! FIX_bughunt1119_ms_out_writes (bh-ms-I13, bh-ms-I1): a failed `--out` write keeps
//! the old file, and `--out` is never the file that standard output is on.
//! One test per TRACE row of the FIX document's section 5, named exactly as the row.
//! Test data only: entropy 00x16 and 11x16, preimages 01x32 and 02x32.

#![cfg(unix)]

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{symlink, PermissionsExt};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Output, Stdio};

const A: &str = "00000000000000000000000000000000\n";
const B: &str = "11111111111111111111111111111111\n";
const H: &str = "0101010101010101010101010101010101010101010101010101010101010101\n";
const H2: &str = "0202020202020202020202020202020202020202020202020202020202020202\n";
/// The card of 00x16, with its newline.
const CARD0: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f\n";

fn bad() -> String {
    // CARD0 with `q` at character 21 replaced by `p`: repair corrects it (exit 4).
    let mut s: Vec<char> = CARD0.chars().collect();
    assert_eq!(s[20], 'q');
    s[20] = 'p';
    s.into_iter().collect()
}

/// E27(X): the failed-write line for a file-size limit.
fn e27(x: &str) -> String {
    format!("ms: cannot write the output: {x}: File too large (os error 27)")
}

/// SAME(X): the refusal when `--out` and standard output are one file.
fn same(x: &str) -> String {
    format!(
        "error: --out {x} and standard output are the same file (the same path, a symlink or a hard link), so what ms prints on stdout would go into the file it writes. With >, the redirect empties that file before ms starts; with >>, the file is untouched. Check that it still holds what you expect. Write to a new name.\n"
    )
}

#[derive(Default)]
struct Run<'a> {
    args: Vec<&'a str>,
    stdin: &'a str,
    stdout: Option<File>,
    stderr: Option<File>,
    fsize8: bool,
}

fn ms_in(dir: &Path, r: Run) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_ms"));
    c.current_dir(dir).args(&r.args).stdin(Stdio::piped());
    c.stdout(r.stdout.map_or_else(Stdio::piped, Stdio::from));
    c.stderr(r.stderr.map_or_else(Stdio::piped, Stdio::from));
    if r.fsize8 {
        // SAFETY: only async-signal-safe libc calls between fork and exec.
        unsafe {
            c.pre_exec(|| {
                let rl = libc::rlimit {
                    rlim_cur: 8,
                    rlim_max: 8,
                };
                libc::setrlimit(libc::RLIMIT_FSIZE, &rl);
                libc::signal(libc::SIGXFSZ, libc::SIG_IGN);
                Ok(())
            });
        }
    }
    let mut child = c.spawn().unwrap();
    let mut sin = child.stdin.take().unwrap();
    let _ = sin.write_all(r.stdin.as_bytes());
    drop(sin);
    child.wait_with_output().unwrap()
}

fn plain(dir: &Path, args: &[&str], stdin: &str) -> Output {
    ms_in(
        dir,
        Run {
            args: args.to_vec(),
            stdin,
            ..Default::default()
        },
    )
}

fn limited(dir: &Path, args: &[&str], stdin: &str) -> Output {
    ms_in(
        dir,
        Run {
            args: args.to_vec(),
            stdin,
            fsize8: true,
            ..Default::default()
        },
    )
}

/// Stdout opened on `f` as `>` does (create, truncate).
fn on_truncated(f: &Path) -> File {
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(f)
        .unwrap()
}

/// Stdout opened on `f` as `>>` does (append).
fn on_appended(f: &Path) -> File {
    OpenOptions::new().append(true).open(f).unwrap()
}

fn rc(o: &Output) -> Option<i32> {
    o.status.code()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

fn verify_rc(dir: &Path, f: &str) -> Option<i32> {
    plain(dir, &["verify", "--in", f], "").status.code()
}

fn no_tmp(dir: &Path) {
    let tmps: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".tmp"))
        .collect();
    assert!(tmps.is_empty(), "temporaries left: {tmps:?}");
}

fn enc(dir: &Path, hex: &str, out: &str) {
    let o = plain(
        dir,
        &["encode", "--hex", "-", "--out", out, "--no-engraving-card"],
        hex,
    );
    assert_eq!(rc(&o), Some(0), "{}", err(&o));
}

fn read(dir: &Path, f: &str) -> String {
    String::from_utf8(std::fs::read(dir.join(f)).unwrap()).unwrap()
}

#[test]
fn o1_encode_failed_overwrite_keeps_previous_card() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    enc(p, A, "F");
    let o = limited(
        p,
        &["encode", "--hex", "-", "--out", "F", "--no-engraving-card"],
        B,
    );
    assert_eq!(rc(&o), Some(1), "{}", err(&o));
    assert_eq!(err(&o), format!("{}. F is unchanged.\n", e27("F")));
    assert!(o.stdout.is_empty());
    assert_eq!(read(p, "F"), CARD0);
    assert_eq!(verify_rc(p, "F"), Some(0));
    no_tmp(p);
}

#[test]
fn o2_hashlock_failed_overwrite_keeps_previous_plate() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let a = [
        "hashlock",
        "--hex",
        "-",
        "--kind",
        "sha256",
        "--no-engraving-card",
        "--out",
        "P",
    ];
    let o = plain(p, &a, H);
    assert_eq!(rc(&o), Some(0), "{}", err(&o));
    let before = read(p, "P");
    let o = limited(p, &a, H2);
    assert_eq!(rc(&o), Some(1), "{}", err(&o));
    assert_eq!(err(&o), format!("{}. P is unchanged.\n", e27("P")));
    assert_eq!(read(p, "P"), before);
}

#[test]
fn o3_repair_failed_overwrite_keeps_previous_card() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    std::fs::write(p.join("BAD"), bad()).unwrap();
    enc(p, A, "C");
    let o = limited(p, &["repair", "--in", "BAD", "--out", "C"], "");
    assert_eq!(rc(&o), Some(1), "{}", err(&o));
    assert_eq!(err(&o), format!("{}. C is unchanged.\n", e27("C")));
    assert_eq!(read(p, "C"), CARD0);
}

#[test]
fn o4_absent_target_failed_write_leaves_nothing() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let o = limited(
        p,
        &["encode", "--hex", "-", "--out", "N", "--no-engraving-card"],
        A,
    );
    assert_eq!(rc(&o), Some(1), "{}", err(&o));
    assert_eq!(err(&o), format!("{}. Nothing was written.\n", e27("N")));
    assert!(!p.join("N").exists());
    no_tmp(p);
}

#[test]
fn o5_missing_directory_text_and_exit() {
    let d = tempfile::tempdir().unwrap();
    let o = plain(
        d.path(),
        &[
            "encode",
            "--hex",
            "-",
            "--out",
            "nodir/x.ms1",
            "--no-engraving-card",
        ],
        A,
    );
    assert_eq!(rc(&o), Some(1), "{}", err(&o));
    assert_eq!(
        err(&o),
        "ms: cannot write the output: nodir/x.ms1: No such file or directory (os error 2). Nothing was written.\n"
    );
}

#[test]
fn o6_read_only_directory_refuses_and_keeps_file() {
    if unsafe { libc::geteuid() } == 0 {
        return;
    }
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    std::fs::create_dir(p.join("ro")).unwrap();
    enc(p, A, "ro/F");
    std::fs::set_permissions(p.join("ro"), std::fs::Permissions::from_mode(0o500)).unwrap();
    let o = plain(
        p,
        &[
            "encode",
            "--hex",
            "-",
            "--out",
            "ro/F",
            "--no-engraving-card",
        ],
        B,
    );
    std::fs::set_permissions(p.join("ro"), std::fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(rc(&o), Some(1), "{}", err(&o));
    assert_eq!(
        err(&o),
        "ms: cannot write the output: ro/F: Permission denied (os error 13). ro/F is unchanged.\n"
    );
    assert_eq!(read(p, "ro/F"), CARD0);
}

#[test]
fn o7_symlink_out_replaces_target_keeps_link() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    std::fs::write(p.join("T"), "OLD\n").unwrap();
    std::fs::set_permissions(p.join("T"), std::fs::Permissions::from_mode(0o644)).unwrap();
    symlink("T", p.join("L")).unwrap();
    let o = plain(
        p,
        &["encode", "--hex", "-", "--out", "L", "--no-engraving-card"],
        A,
    );
    assert_eq!(rc(&o), Some(0), "{}", err(&o));
    assert!(std::fs::symlink_metadata(p.join("L"))
        .unwrap()
        .file_type()
        .is_symlink());
    assert_eq!(read(p, "T"), CARD0);
    assert_eq!(
        std::fs::metadata(p.join("T")).unwrap().permissions().mode() & 0o777,
        0o600
    );
}

#[test]
fn o8_stdio_alias_onto_a_file_names_no_state() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let old = "OLD BACKUP LINE\n";
    std::fs::write(p.join("S2"), old).unwrap();
    let o = ms_in(
        p,
        Run {
            args: vec![
                "encode",
                "--hex",
                "-",
                "--out",
                "/dev/stdout",
                "--no-engraving-card",
            ],
            stdin: A,
            stdout: Some(on_appended(&p.join("S2"))),
            fsize8: true,
            ..Default::default()
        },
    );
    assert_eq!(rc(&o), Some(1), "{}", err(&o));
    let base = e27("/dev/stdout");
    let s2_intact = read(p, "S2") == old;
    let want = if s2_intact {
        format!("{base}. /dev/stdout is unchanged.\n")
    } else {
        format!("{base}\n")
    };
    assert_eq!(err(&o), want);
}

#[test]
fn o9_create_new_failed_write_text() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let o = limited(
        p,
        &["split", "-k", "2", "-n", "3", "--hex", "-", "--out", "SH"],
        A,
    );
    assert_eq!(rc(&o), Some(1), "{}", err(&o));
    let e = err(&o);
    assert_eq!(
        e.lines().last().unwrap(),
        format!("{}. Nothing was written.", e27("SH"))
    );
    assert!(!p.join("SH").exists());
}

#[test]
fn o9b_create_new_retry_after_failed_write() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let a = ["split", "-k", "2", "-n", "3", "--hex", "-", "--out", "SH"];
    let o = limited(p, &a, A);
    assert_eq!(rc(&o), Some(1), "{}", err(&o));
    let o = plain(p, &a, A);
    assert_eq!(rc(&o), Some(0), "{}", err(&o));
    let md = std::fs::metadata(p.join("SH")).unwrap();
    assert_eq!(read(p, "SH").lines().count(), 3);
    assert_eq!(md.len(), 153);
    assert_eq!(md.permissions().mode() & 0o777, 0o600);
}

#[test]
fn o9c_random_preimage_failed_write_leaves_nothing() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let o = limited(
        p,
        &["hashlock", "--random", "--no-engraving-card", "--out", "R"],
        "",
    );
    assert_eq!(rc(&o), Some(1), "{}", err(&o));
    let e = err(&o);
    assert_eq!(
        e.lines().last().unwrap(),
        format!("{}. Nothing was written.", e27("R"))
    );
    assert!(!p.join("R").exists());
}

/// Run `args` with stdout on `target` (opened by `open`), expect the SAME refusal.
fn refused_same(p: &Path, args: &[&str], stdin: &str, stdout: File, typed: &str) -> Output {
    let o = ms_in(
        p,
        Run {
            args: args.to_vec(),
            stdin,
            stdout: Some(stdout),
            ..Default::default()
        },
    );
    assert_eq!(rc(&o), Some(64), "{}", err(&o));
    assert_eq!(err(&o), same(typed));
    o
}

const ENC_JSON: [&str; 7] = [
    "encode",
    "--hex",
    "-",
    "--out",
    "F",
    "--json",
    "--no-engraving-card",
];

#[test]
fn o10_encode_json_stdout_is_out_refused() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    refused_same(p, &ENC_JSON, A, on_truncated(&p.join("F")), "F");
    assert_eq!(std::fs::metadata(p.join("F")).unwrap().len(), 0);
}

#[test]
fn o11_encode_json_stdout_hard_link_refused() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    File::create(p.join("F")).unwrap();
    std::fs::hard_link(p.join("F"), p.join("F2")).unwrap();
    refused_same(p, &ENC_JSON, A, on_truncated(&p.join("F2")), "F");
    assert_eq!(std::fs::metadata(p.join("F")).unwrap().len(), 0);
}

#[test]
fn o12_encode_json_stdout_symlink_refused() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    File::create(p.join("F")).unwrap();
    symlink("F", p.join("LF")).unwrap();
    refused_same(p, &ENC_JSON, A, on_truncated(&p.join("LF")), "F");
    assert_eq!(std::fs::metadata(p.join("F")).unwrap().len(), 0);
}

const HL: [&str; 8] = [
    "hashlock",
    "--hex",
    "-",
    "--kind",
    "sha256",
    "--no-engraving-card",
    "--out",
    "P",
];

#[test]
fn o13_hashlock_stdout_is_out_refused() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    refused_same(p, &HL, H, on_truncated(&p.join("P")), "P");
    assert_eq!(std::fs::metadata(p.join("P")).unwrap().len(), 0);
}

#[test]
fn o14_hashlock_json_stdout_is_out_refused() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let mut a = HL.to_vec();
    a.push("--json");
    refused_same(p, &a, H, on_truncated(&p.join("P")), "P");
    assert_eq!(std::fs::metadata(p.join("P")).unwrap().len(), 0);
}

#[test]
fn o15_repair_stdout_is_out_refused() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    std::fs::write(p.join("BAD"), bad()).unwrap();
    refused_same(
        p,
        &["repair", "--in", "BAD", "--out", "C"],
        "",
        on_truncated(&p.join("C")),
        "C",
    );
    assert_eq!(std::fs::metadata(p.join("C")).unwrap().len(), 0);
}

#[test]
fn o16_text_encode_out_with_stdout_on_it_still_writes() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let o = ms_in(
        p,
        Run {
            args: vec!["encode", "--hex", "-", "--out", "F", "--no-engraving-card"],
            stdin: A,
            stdout: Some(on_truncated(&p.join("F"))),
            ..Default::default()
        },
    );
    assert_eq!(rc(&o), Some(0), "{}", err(&o));
    assert_eq!(read(p, "F"), CARD0);
    assert_eq!(verify_rc(p, "F"), Some(0));
}

#[test]
fn o17_stderr_on_the_out_file_keeps_the_card() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let o = ms_in(
        p,
        Run {
            args: vec!["encode", "--hex", "-", "--out", "F"],
            stdin: A,
            stderr: Some(on_truncated(&p.join("F"))),
            ..Default::default()
        },
    );
    assert_eq!(rc(&o), Some(0));
    assert_eq!(read(p, "F"), CARD0);
    assert_eq!(verify_rc(p, "F"), Some(0));
}

#[test]
fn o18_split_out_with_stdout_on_it_refused_as_existing() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let o = ms_in(
        p,
        Run {
            args: vec!["split", "-k", "2", "-n", "3", "--hex", "-", "--out", "SH"],
            stdin: A,
            stdout: Some(on_truncated(&p.join("SH"))),
            ..Default::default()
        },
    );
    assert_eq!(rc(&o), Some(64), "{}", err(&o));
    assert_eq!(
        err(&o),
        "error: --out SH already exists; a new share set will not overwrite it (choose another file, or move the old one first)\n"
    );
}

#[test]
fn o19_hashlock_random_out_with_stdout_on_it_refused() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    refused_same(
        p,
        &["hashlock", "--random", "--no-engraving-card", "--out", "R"],
        "",
        on_truncated(&p.join("R")),
        "R",
    );
    assert_eq!(std::fs::metadata(p.join("R")).unwrap().len(), 0);
}

#[test]
fn o20_fifo_out_written_directly() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let ff = p.join("FF");
    let c = std::ffi::CString::new(ff.to_str().unwrap()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(c.as_ptr(), 0o600) }, 0);
    let ff2 = ff.clone();
    let reader = std::thread::spawn(move || {
        let mut s = String::new();
        File::open(ff2).unwrap().read_to_string(&mut s).unwrap();
        s
    });
    let o = plain(
        p,
        &["encode", "--hex", "-", "--out", "FF", "--no-engraving-card"],
        A,
    );
    assert_eq!(rc(&o), Some(0), "{}", err(&o));
    assert_eq!(reader.join().unwrap(), CARD0);
}

#[test]
fn o21_json_out_with_piped_stdout_writes_both() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let o = plain(p, &ENC_JSON, A);
    assert_eq!(rc(&o), Some(0), "{}", err(&o));
    assert_eq!(read(p, "F"), CARD0);
    assert!(String::from_utf8_lossy(&o.stdout).contains("schema_version"));
}

#[test]
fn o22_encode_json_stdout_appended_to_out_keeps_card() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    enc(p, A, "F");
    refused_same(p, &ENC_JSON, B, on_appended(&p.join("F")), "F");
    assert_eq!(read(p, "F"), CARD0);
    assert_eq!(verify_rc(p, "F"), Some(0));
}

#[test]
fn o23_hashlock_random_json_stdout_appended_keeps_preimage() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let o = plain(
        p,
        &["hashlock", "--random", "--no-engraving-card", "--out", "R"],
        "",
    );
    assert_eq!(rc(&o), Some(0), "{}", err(&o));
    let before = read(p, "R");
    refused_same(
        p,
        &[
            "hashlock",
            "--random",
            "--json",
            "--no-engraving-card",
            "--out",
            "R",
        ],
        "",
        on_appended(&p.join("R")),
        "R",
    );
    assert_eq!(read(p, "R"), before);
    let o = plain(
        p,
        &[
            "hashlock",
            "--kind",
            "sha256",
            "--in",
            "R",
            "--no-engraving-card",
        ],
        "",
    );
    assert_eq!(rc(&o), Some(0), "{}", err(&o));
}

#[test]
fn o24_split_json_stdout_appended_keeps_shares() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    let o = plain(
        p,
        &["split", "-k", "2", "-n", "3", "--hex", "-", "--out", "SH"],
        A,
    );
    assert_eq!(rc(&o), Some(0), "{}", err(&o));
    let before = read(p, "SH");
    refused_same(
        p,
        &[
            "split", "-k", "2", "-n", "3", "--hex", "-", "--json", "--out", "SH",
        ],
        A,
        on_appended(&p.join("SH")),
        "SH",
    );
    assert_eq!(read(p, "SH"), before);
    let o = plain(p, &["combine", "--in", "SH"], "");
    assert_eq!(rc(&o), Some(0), "{}", err(&o));
}

#[test]
fn o25_json_failed_overwrite_line_on_stderr_no_envelope() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path();
    enc(p, A, "F");
    let o = limited(p, &ENC_JSON, B);
    assert_eq!(rc(&o), Some(1), "{}", err(&o));
    assert!(o.stdout.is_empty(), "no envelope on stdout");
    assert_eq!(err(&o), format!("{}. F is unchanged.\n", e27("F")));
    assert_eq!(read(p, "F"), CARD0);
}
