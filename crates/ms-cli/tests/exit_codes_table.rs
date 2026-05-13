//! Parametric: exit code per CliError variant. Locks SPEC §6 table.

use assert_cmd::Command;

#[test]
fn exit_code_table_user_input() {
    // Odd-length hex → exit 1.
    Command::cargo_bin("ms")
        .unwrap()
        .args(["encode", "--hex", "0"])
        .assert()
        .failure()
        .code(1);
}

#[test]
fn exit_code_table_format_violation() {
    Command::cargo_bin("ms")
        .unwrap()
        .args([
            "decode",
            "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7p",
        ]) // bad cksum → Codex32 → exit 1
        .assert()
        .failure()
        .code(1);
}

#[test]
fn exit_code_table_clap_usage() {
    // No subcommand → exit 64.
    Command::cargo_bin("ms")
        .unwrap()
        .arg("--frob-flag-that-doesnt-exist")
        .assert()
        .failure()
        .code(64);
}

// ── v0.2.1 fix: clap soft-error terminations exit 0 ────────────────────
//
// Pre-fix, `ms --version` and `ms --help` exited 64 because the
// `Cli::try_parse()` catch-all in `src/main.rs` mapped EVERY
// `clap::Error` to `ExitCode::from(64)`. But clap returns soft-error
// variants for help/version display:
//
//   --version → ErrorKind::DisplayVersion
//   --help    → ErrorKind::DisplayHelp
//
// These are not usage errors; the output is to stdout, not stderr, and
// the canonical Unix convention is exit 0. The fix branches on
// `e.kind()` and returns `ExitCode::SUCCESS` for those two variants,
// preserving the SPEC §6 carve-out (exit 64 for real parse errors)
// for everything else. These two cells pin the post-fix invariant.

#[test]
fn version_flag_exits_zero_and_prints_version() {
    let out = Command::cargo_bin("ms")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .code(0)
        .get_output()
        .clone();
    let stdout = String::from_utf8(out.stdout).expect("stdout valid utf-8");
    assert!(
        stdout.starts_with("ms "),
        "expected stdout to start with `ms `; got {stdout:?}"
    );
}

#[test]
fn help_flag_exits_zero_and_prints_help() {
    let out = Command::cargo_bin("ms")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .code(0)
        .get_output()
        .clone();
    let stdout = String::from_utf8(out.stdout).expect("stdout valid utf-8");
    // Clap's --help output includes the `Usage:` header from the
    // top-level `#[command(name = "ms", ...)]` derive.
    assert!(
        stdout.contains("Usage:") && stdout.contains("ms"),
        "expected clap --help stdout to contain `Usage:` and `ms`; got {stdout:?}"
    );
}
