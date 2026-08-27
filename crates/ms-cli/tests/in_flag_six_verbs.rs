//! **P2 row 1 — `--in FILE` on the six verbs whose `--in` binding is unambiguous.**
//!
//! `decode`, `verify`, `inspect`, `repair`, `derive`, `combine`. §6d's per-verb
//! table gives each of these exactly one channel for `--in` to bind to — the
//! positional (`--ms1` on `repair`, the share list on `combine`) — which is why
//! they need no ruling and `encode`/`split` do.
//!
//! **The gate is an EQUALITY, never a bare success.** stdout *and* stderr must
//! be byte-equal to the same invocation fed on stdin, at the same exit code. A
//! `--in` that clap accepted and the verb silently ignored would satisfy
//! "exit 0" and fail this.
//!
//! **Plus two controls, and they are the ones a naive implementation fails:**
//!
//! 1. `--in <a nonexistent path>` must fail NAMING the path and must not fall
//!    back to stdin — otherwise a typo silently reads a terminal, or worse,
//!    silently reads whatever the previous stage of a pipeline wrote.
//! 2. `--in f` together with the verb's own `-` must REFUSE, matching the two
//!    contention refusals `ms` already ships. A channel that silently wins over
//!    another is how an operator engraves the wrong card.

use assert_cmd::Command;
use std::io::Write;

const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

fn ms() -> Command {
    Command::cargo_bin("ms").unwrap()
}

fn write_tmp(dir: &std::path::Path, name: &str, body: &str) -> std::path::PathBuf {
    let p = dir.join(name);
    let mut f = std::fs::File::create(&p).unwrap();
    f.write_all(body.as_bytes()).unwrap();
    p
}

/// Run `args` twice — once with the material on stdin, once with `--in` — and
/// assert the two runs are indistinguishable on stdout, stderr and exit code.
///
/// `stdin_args` is the invocation that reads stdin; `in_args` is the same
/// invocation with the stdin channel replaced by `--in <path>`.
fn assert_in_equals_stdin(stdin_args: &[&str], in_args: &[&str], body: &str, label: &str) {
    let dir = tempfile::tempdir().unwrap();
    let p = write_tmp(dir.path(), "material.txt", body);

    let via_stdin = ms()
        .args(stdin_args)
        .write_stdin(body.to_string())
        .output()
        .unwrap();

    let mut in_argv: Vec<String> = in_args.iter().map(|s| s.to_string()).collect();
    in_argv.push("--in".to_string());
    in_argv.push(p.display().to_string());
    let via_in = ms().args(&in_argv).output().unwrap();

    assert_eq!(
        via_stdin.status.code(),
        via_in.status.code(),
        "{label}: exit codes differ. stdin run stderr:\n{}\n--in run stderr:\n{}",
        String::from_utf8_lossy(&via_stdin.stderr),
        String::from_utf8_lossy(&via_in.stderr)
    );
    assert_eq!(
        via_stdin.status.code(),
        Some(0),
        "{label}: both runs must SUCCEED, or the equality is between two failures. \
         stderr:\n{}",
        String::from_utf8_lossy(&via_in.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&via_stdin.stdout),
        String::from_utf8_lossy(&via_in.stdout),
        "{label}: stdout differs between the stdin run and the --in run"
    );
    assert_eq!(
        String::from_utf8_lossy(&via_stdin.stderr),
        String::from_utf8_lossy(&via_in.stderr),
        "{label}: stderr differs between the stdin run and the --in run"
    );
}

#[test]
fn in_on_decode_equals_the_stdin_run() {
    assert_in_equals_stdin(&["decode", "-"], &["decode"], MS1, "decode");
}

#[test]
fn in_on_verify_equals_the_stdin_run() {
    assert_in_equals_stdin(&["verify", "-"], &["verify"], MS1, "verify");
}

#[test]
fn in_on_inspect_equals_the_stdin_run() {
    assert_in_equals_stdin(&["inspect", "-"], &["inspect"], MS1, "inspect");
}

#[test]
fn in_on_derive_equals_the_stdin_run() {
    assert_in_equals_stdin(&["derive", "-"], &["derive"], MS1, "derive");
}

/// `repair`'s channel is the `--ms1` FLAG rather than a positional, and it is
/// the one verb here that writes `--out` while exiting non-zero — so the
/// equality is asserted on a clean (exit 0) input, and the exit-4 shape is the
/// private-write work's business.
#[test]
fn in_on_repair_equals_the_stdin_run() {
    assert_in_equals_stdin(&["repair", "--ms1", "-"], &["repair"], MS1, "repair");
}

/// `combine`'s channel is the variadic `<SHARES>...`, and `--in` reads **one
/// share per line**, display separators stripped, exactly as the stdin path
/// already does.
#[test]
fn in_on_combine_equals_the_stdin_run() {
    let shares = split_two_shares();
    let body = format!("{}\n{}\n", shares[0], shares[1]);
    assert_in_equals_stdin(&["combine", "-"], &["combine"], &body, "combine");
}

/// `--in` on `combine` strips display separators per line, like the stdin path.
/// A grouped card typed back off metal must re-ingest.
#[test]
fn in_on_combine_accepts_grouped_shares() {
    let dir = tempfile::tempdir().unwrap();
    let shares = split_two_shares_grouped();
    let p = write_tmp(
        dir.path(),
        "shares.txt",
        &format!("{}\n{}\n", shares[0], shares[1]),
    );
    let out = ms()
        .args(["combine", "--in", &p.display().to_string()])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "grouped shares must re-ingest through --in: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        String::from_utf8_lossy(&out.stdout).contains(PHRASE),
        "the recovered phrase must be the original"
    );
}

// ---------------------------------------------------------------------------
// The two controls.
// ---------------------------------------------------------------------------

/// **Control 1 — a missing `--in` file fails NAMING the path, and never falls
/// back to stdin.** Valid material is placed on stdin, so an implementation
/// that silently falls back exits 0 and this goes RED.
#[test]
fn a_missing_in_file_names_the_path_and_does_not_fall_back_to_stdin() {
    for verb in ["decode", "verify", "inspect", "derive", "repair"] {
        let out = ms()
            .args([verb, "--in", "/nonexistent/ms-p2-no-such-file.txt"])
            .write_stdin(MS1.to_string())
            .output()
            .unwrap();
        assert_ne!(
            out.status.code(),
            Some(0),
            "{verb}: a nonexistent --in must NOT fall back to stdin. stdout was:\n{}",
            String::from_utf8_lossy(&out.stdout)
        );
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(
            err.contains("/nonexistent/ms-p2-no-such-file.txt"),
            "{verb}: the refusal must name the path it could not read; got:\n{err}"
        );
    }
}

/// **Control 2 — `--in` and the verb's own `-` REFUSE rather than one silently
/// winning.** Mirrors the two contention refusals `ms` already ships.
#[test]
fn in_together_with_dash_refuses_on_every_verb() {
    let dir = tempfile::tempdir().unwrap();
    let p = write_tmp(dir.path(), "card.ms1", MS1);
    let path = p.display().to_string();
    let cases: Vec<Vec<&str>> = vec![
        vec!["decode", "-", "--in", &path],
        vec!["verify", "-", "--in", &path],
        vec!["inspect", "-", "--in", &path],
        vec!["derive", "-", "--in", &path],
        vec!["repair", "--ms1", "-", "--in", &path],
        vec!["combine", "-", "--in", &path],
    ];
    for args in cases {
        let out = ms()
            .args(&args)
            .write_stdin(MS1.to_string())
            .output()
            .unwrap();
        assert_ne!(
            out.status.code(),
            Some(0),
            "{args:?}: two input channels at once must REFUSE, not pick one"
        );
    }
}

/// **Control 3 — `--in` and an explicit positional VALUE refuse too.** The
/// value form is the shape an operator reaches for after a copy-paste, and a
/// silent winner there is the same defect as with `-`.
#[test]
fn in_together_with_a_positional_value_refuses() {
    let dir = tempfile::tempdir().unwrap();
    let p = write_tmp(dir.path(), "card.ms1", MS1);
    let out = ms()
        .args(["decode", MS1, "--in", &p.display().to_string()])
        .output()
        .unwrap();
    assert_ne!(
        out.status.code(),
        Some(0),
        "a positional value and --in together must REFUSE"
    );
}

// ---------------------------------------------------------------------------

fn split_two_shares_with(group_size: &str) -> Vec<String> {
    let dir = tempfile::tempdir().unwrap();
    let p = write_tmp(dir.path(), "seed.txt", PHRASE);
    let out = ms()
        .args([
            "split",
            "--phrase",
            "-",
            "-k",
            "2",
            "-n",
            "3",
            "--group-size",
            group_size,
        ])
        .write_stdin(std::fs::read_to_string(&p).unwrap())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "split must succeed");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .take(2)
        .map(str::to_string)
        .collect()
}

fn split_two_shares() -> Vec<String> {
    split_two_shares_with("0")
}

fn split_two_shares_grouped() -> Vec<String> {
    split_two_shares_with("5")
}
