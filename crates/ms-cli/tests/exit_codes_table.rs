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
