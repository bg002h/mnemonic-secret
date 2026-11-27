//! FIX #46 (bh-ms-I6, bh-ms-I9, bh-ms-R11): printed commands must run as printed.
//!
//! Rows are those of `design/FIX_bughunt1119_ms_printed_commands.md` section 4, one
//! test function per row, named as the row. Test vectors only (16 zero bytes of
//! entropy; all-zero-data codex32 strings with id `test`).

use assert_cmd::Command;
use std::path::{Path, PathBuf};
use std::process::Command as Proc;

const ZERO: &str = "00000000000000000000000000000000";
const C12: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
const S2: &str = "ms12testsqqqqqqqqqqqqqqqqqqqqqqqqqqqqkwe5qlrp6ykwa";
const S9: &str = "ms19testsqqqqqqqqqqqqqqqqqqqqqqqqqqqqx7chst33fgtms";
const A2: &str = "ms12testaqqqqqqqqqqqqqqqqqqqqqqqqqqqqzfvk7urqpuwd0";
const A9: &str = "ms19testaqqqqqqqqqqqqqqqqqqqqqqqqqqqqjed4wg3sjsncz";

fn ms() -> Command {
    Command::cargo_bin("ms").unwrap()
}

fn bin_dir() -> PathBuf {
    assert_cmd::cargo::cargo_bin("ms")
        .parent()
        .unwrap()
        .to_path_buf()
}

fn spath() -> String {
    format!("{}:/usr/bin:/bin", bin_dir().display())
}

fn secret_text(t: char) -> String {
    format!(
        "this is the secret share of a K-of-N set (threshold '{t}', index 's'); it is the secret itself, not a distributed share, and `ms combine` does not take it"
    )
}

fn share_text(t: char, i: char) -> String {
    format!(
        "this is one share of a K-of-N set (threshold '{t}', index '{i}'); use `ms combine` to recombine K shares"
    )
}

/// Run `ms <args>` in `dir` (rc 2 expected: an invalid input), take the stderr line starting with
/// `ms `, assert it equals `want`, run it in `sh -c` in `dir`; return the run.
fn remedy(dir: &Path, name: &str, want: &str, args: &[&str]) -> std::process::Output {
    std::fs::write(dir.join(name), format!("{ZERO}\n")).unwrap();
    let out = ms()
        .current_dir(dir)
        .args(args)
        .write_stdin("")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "first run exit");
    let err = String::from_utf8(out.stderr).unwrap();
    let line = err
        .lines()
        .map(str::trim)
        .find(|l| l.starts_with("ms "))
        .unwrap_or_else(|| panic!("no printed command in stderr: {err:?}"))
        .to_string();
    assert_eq!(line, want, "printed command");
    Proc::new("sh")
        .arg("-c")
        .arg(&line)
        .current_dir(dir)
        .env_clear()
        .env("PATH", spath())
        .env("HOME", dir)
        .output()
        .unwrap()
}

fn assert_runs_encode(run: &std::process::Output) {
    assert_eq!(
        run.status.code(),
        Some(0),
        "printed line exit; stderr={}",
        String::from_utf8_lossy(&run.stderr)
    );
    let so = String::from_utf8(run.stdout.clone()).unwrap();
    assert_eq!(so.lines().last(), Some(C12), "last stdout line is the card");
}

fn count_shares(text: &str) -> usize {
    text.lines().filter(|l| l.starts_with("ms1")).count()
}

#[test]
fn printed_cmd_split_remedy_runs() {
    let d = tempfile::tempdir().unwrap();
    let run = remedy(
        d.path(),
        "entropy.hex",
        "ms split -k 2 -n 3 --hex - < 'entropy.hex'",
        &["split", "-k", "2", "-n", "3", "--in", "entropy.hex"],
    );
    assert_eq!(run.status.code(), Some(0));
    assert_eq!(count_shares(&String::from_utf8(run.stdout).unwrap()), 3);
}

#[test]
fn printed_cmd_split_remedy_long_flags_runs() {
    let d = tempfile::tempdir().unwrap();
    let run = remedy(
        d.path(),
        "entropy.hex",
        "ms split -k 3 -n 5 --hex - < 'entropy.hex'",
        &[
            "split",
            "--in",
            "entropy.hex",
            "--shares",
            "5",
            "--threshold",
            "3",
        ],
    );
    assert_eq!(run.status.code(), Some(0));
    assert_eq!(count_shares(&String::from_utf8(run.stdout).unwrap()), 5);
}

#[test]
fn printed_cmd_encode_remedy_text() {
    let d = tempfile::tempdir().unwrap();
    let run = remedy(
        d.path(),
        "entropy.hex",
        "ms encode --hex - < 'entropy.hex'",
        &["encode", "--in", "entropy.hex"],
    );
    assert_runs_encode(&run);
}

#[test]
fn printed_cmd_encode_remedy_path_with_space_runs() {
    let d = tempfile::tempdir().unwrap();
    let run = remedy(
        d.path(),
        "two words.hex",
        "ms encode --hex - < 'two words.hex'",
        &["encode", "--in", "two words.hex"],
    );
    assert_runs_encode(&run);
}

#[test]
fn printed_cmd_encode_remedy_path_with_quote_runs() {
    let d = tempfile::tempdir().unwrap();
    let run = remedy(
        d.path(),
        "it's.hex",
        "ms encode --hex - < 'it'\\''s.hex'",
        &["encode", "--in", "it's.hex"],
    );
    assert_runs_encode(&run);
}

#[test]
fn printed_cmd_split_remedy_path_with_space_runs() {
    let d = tempfile::tempdir().unwrap();
    let run = remedy(
        d.path(),
        "two words.hex",
        "ms split -k 2 -n 3 --hex - < 'two words.hex'",
        &["split", "-k", "2", "-n", "3", "--in", "two words.hex"],
    );
    assert_eq!(run.status.code(), Some(0));
    assert_eq!(count_shares(&String::from_utf8(run.stdout).unwrap()), 3);
}

#[test]
fn printed_cmd_encode_remedy_keeps_out() {
    let d = tempfile::tempdir().unwrap();
    let run = remedy(
        d.path(),
        "entropy.hex",
        "ms encode --out='my card.ms1' --hex - < 'entropy.hex'",
        &["encode", "--in", "entropy.hex", "--out", "my card.ms1"],
    );
    assert_eq!(run.status.code(), Some(0));
    assert!(
        run.stdout.is_empty(),
        "the printed run writes nothing to stdout"
    );
    let card = std::fs::read_to_string(d.path().join("my card.ms1")).unwrap();
    assert_eq!(card, format!("{C12}\n"));
}

#[test]
fn printed_cmd_split_remedy_keeps_out_and_recombines() {
    let d = tempfile::tempdir().unwrap();
    let run = remedy(
        d.path(),
        "two words.hex",
        "ms split -k 2 -n 3 --out='shares.txt' --hex - < 'two words.hex'",
        &[
            "split",
            "-k",
            "2",
            "-n",
            "3",
            "--in",
            "two words.hex",
            "--out",
            "shares.txt",
        ],
    );
    assert_eq!(run.status.code(), Some(0));
    let shares = std::fs::read_to_string(d.path().join("shares.txt")).unwrap();
    assert_eq!(count_shares(&shares), 3);
    let two: String = shares.lines().take(2).collect::<Vec<_>>().join("\n") + "\n";
    std::fs::write(d.path().join("two.txt"), two).unwrap();
    let out = ms()
        .current_dir(d.path())
        .args(["combine", "--in", "two.txt", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let so = String::from_utf8(out.stdout).unwrap();
    assert!(
        so.contains(&format!("\"entropy_hex\":\"{ZERO}\"")),
        "combine output: {so}"
    );
}

/// The user-manpath line of `ms gen-man --help`, cut before the comment.
fn gen_man_example() -> String {
    let out = ms().args(["gen-man", "--help"]).output().unwrap();
    let help = String::from_utf8(out.stdout).unwrap();
    let line = help
        .lines()
        .map(str::trim)
        .find(|l| l.starts_with("ms gen-man --out \"$"))
        .expect("user-manpath example line")
        .to_string();
    line.split("   #").next().unwrap().trim().to_string()
}

fn run_example(env: &[(&str, &Path)], home: &Path, dir: &Path) -> std::process::Output {
    let mut c = Proc::new("sh");
    c.arg("-c")
        .arg(gen_man_example())
        .current_dir(dir)
        .env_clear()
        .env("PATH", spath())
        .env("HOME", home);
    for (k, v) in env {
        c.env(k, v);
    }
    c.output().unwrap()
}

#[test]
fn printed_cmd_gen_man_example_runs_without_xdg() {
    let d = tempfile::tempdir().unwrap();
    let out = run_example(&[], d.path(), d.path());
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(d.path().join(".local/share/man/man1/ms.1").exists());
}

#[test]
fn printed_cmd_gen_man_example_runs_with_empty_xdg() {
    let d = tempfile::tempdir().unwrap();
    let out = run_example(&[("XDG_DATA_HOME", Path::new(""))], d.path(), d.path());
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(d.path().join(".local/share/man/man1/ms.1").exists());
}

#[test]
fn printed_cmd_gen_man_example_honours_xdg() {
    let d = tempfile::tempdir().unwrap();
    let xdg = d.path().join("xdg");
    let home = d.path().join("home");
    let out = run_example(&[("XDG_DATA_HOME", &xdg)], &home, d.path());
    assert_eq!(out.status.code(), Some(0));
    assert!(xdg.join("man/man1/ms.1").exists());
    assert!(!home.exists(), "HOME must not be created");
}

#[test]
fn printed_cmd_hex_channel_control() {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("entropy.hex"), format!("{ZERO}\n")).unwrap();
    let out = ms()
        .current_dir(d.path())
        .args(["encode", "--hex", "-"])
        .write_stdin(format!("{ZERO}\n"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let so = String::from_utf8(out.stdout).unwrap();
    assert_eq!(so.lines().last(), Some(C12));
}

fn range_row(want: &str, args: &[&str]) {
    let d = tempfile::tempdir().unwrap();
    std::fs::write(d.path().join("entropy.hex"), format!("{ZERO}\n")).unwrap();
    let out = ms()
        .current_dir(d.path())
        .args(args)
        .write_stdin("")
        .output()
        .unwrap();
    // K and N are settings: a bad one is a usage error (exit-code spec decision 2).
    assert_eq!(out.status.code(), Some(64));
    assert_eq!(
        String::from_utf8(out.stderr)
            .unwrap()
            .trim_end_matches('\n'),
        want
    );
    assert!(out.stdout.is_empty());
}

#[test]
fn printed_cmd_split_bad_threshold_refused_before_read() {
    range_row(
        "error: invalid threshold 1; K-of-N shares require k in 2..=9",
        &["split", "-k", "1", "-n", "3", "--in", "entropy.hex"],
    );
}

#[test]
fn printed_cmd_split_bad_share_count_refused_before_read() {
    range_row(
        "error: invalid share count n=3 for threshold k=5; require k <= n <= 31",
        &["split", "-k", "5", "-n", "3", "--in", "entropy.hex"],
    );
}

/// Exit 2, stderr exactly `want`, nothing on stdout.
fn share_row(want: &str, stdin: &str, args: &[&str]) {
    let out = ms()
        .args(args)
        .write_stdin(format!("{stdin}\n"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(
        String::from_utf8(out.stderr)
            .unwrap()
            .trim_end_matches('\n'),
        want
    );
    assert!(out.stdout.is_empty(), "stdout must be empty");
}

#[test]
fn printed_cmd_decode_secret_share_names_secret() {
    share_row(
        &format!("error: {}", secret_text('2')),
        S2,
        &["decode", "-"],
    );
}

#[test]
fn printed_cmd_repair_secret_share_names_secret() {
    share_row(
        &format!("error: {}", secret_text('2')),
        S2,
        &["repair", "--ms1", "-"],
    );
}

#[test]
fn printed_cmd_decode_secret_share_threshold_9_names_secret() {
    share_row(
        &format!("error: {}", secret_text('9')),
        S9,
        &["decode", "-"],
    );
}

#[test]
fn printed_cmd_decode_secret_share_uppercase_names_secret() {
    share_row(
        &format!("error: {}", secret_text('2')),
        &S2.to_uppercase(),
        &["decode", "-"],
    );
}

#[test]
fn printed_cmd_decode_secret_share_json_message() {
    let out = ms()
        .args(["decode", "--json", "-"])
        .write_stdin(format!("{S2}\n"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let both = format!(
        "{}{}",
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap()
    );
    let want = format!(
        "{{\"schema_version\":\"1\",\"error\":{{\"kind\":\"IsShareNotSingleString\",\"message\":\"{}\",\"exit_code\":2,\"details\":{{\"index\":\"s\",\"threshold\":\"2\"}}}}}}",
        secret_text('2')
    );
    assert_eq!(both.trim_end_matches('\n'), want);
}

#[test]
fn printed_cmd_decode_distributed_share_keeps_combine_advice() {
    share_row(
        &format!("error: {}", share_text('2', 'a')),
        A2,
        &["decode", "-"],
    );
}

#[test]
fn printed_cmd_repair_distributed_share_keeps_combine_advice() {
    // A clean distributed share is a valid string for `repair` (exit-code final
    // review I-2): echoed at 0, and the stderr line still points at combine.
    let out = ms()
        .args(["repair", "--ms1", "-"])
        .write_stdin(format!("{A9}\n"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&out.stdout).contains(A9));
    let err = String::from_utf8(out.stderr).unwrap();
    assert!(
        err.contains("repair: this is share 'a' of a 9-of-N set, not a single card; recover the secret with `ms combine` and 9 shares"),
        "{err}"
    );
}

#[test]
fn printed_cmd_combine_refuses_secret_share_control() {
    share_row(
        "error: the secret share (index 's') must not be combined; supply only distributed shares (the secret is the recovery target)",
        S2,
        &["combine", "-"],
    );
}

#[test]
fn printed_cmd_distributed_share_advice_combines() {
    let d = tempfile::tempdir().unwrap();
    let out = ms()
        .current_dir(d.path())
        .args(["split", "-k", "2", "-n", "3", "--hex", "-"])
        .write_stdin(format!("{ZERO}\n"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let shares = String::from_utf8(out.stdout).unwrap();
    let shares: Vec<&str> = shares.lines().filter(|l| l.starts_with("ms1")).collect();
    let first: String = shares[0].chars().filter(|c| !c.is_whitespace()).collect();
    let idx = first.chars().nth(8).unwrap();
    assert_ne!(idx, 's');
    share_row(
        &format!("error: {}", share_text('2', idx)),
        &first,
        &["decode", "-"],
    );
    let two: String = shares
        .iter()
        .take(2)
        .copied()
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    std::fs::write(d.path().join("two.txt"), two).unwrap();
    let c = ms()
        .current_dir(d.path())
        .args(["combine", "--in", "two.txt", "--json"])
        .output()
        .unwrap();
    assert_eq!(c.status.code(), Some(0));
    assert!(String::from_utf8(c.stdout)
        .unwrap()
        .contains(&format!("\"entropy_hex\":\"{ZERO}\"")));
}
