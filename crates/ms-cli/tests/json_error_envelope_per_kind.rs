//! For each CliError `kind`, verify JSON-mode error output matches §5.4 schema.

use assert_cmd::Command;
use ms_codec::codex32::{Codex32String, Fe};
use serde_json::Value;

fn run_and_parse(args: &[&str]) -> Value {
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(args)
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&out).expect("error envelope is valid JSON")
}

#[test]
fn bad_input_json_envelope() {
    // Odd-length hex → BadInput.
    let v = run_and_parse(&["encode", "--hex", "0", "--json"]);
    assert_eq!(v["schema_version"], "1");
    assert_eq!(v["error"]["kind"], "BadInput");
    assert_eq!(v["error"]["exit_code"], 1);
}

#[test]
fn bip39_json_envelope() {
    let v = run_and_parse(&[
        "encode",
        "--phrase",
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon ability",
        "--json",
    ]);
    assert_eq!(v["error"]["kind"], "Bip39");
    assert_eq!(v["error"]["exit_code"], 1);
}

#[test]
fn codex32_json_envelope() {
    // Bad checksum string.
    let v = run_and_parse(&[
        "decode",
        "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7p", // last char flipped
        "--json",
    ]);
    assert_eq!(v["error"]["kind"], "Codex32");
    assert_eq!(v["error"]["exit_code"], 1);
}

#[test]
fn unexpected_string_length_json_envelope() {
    // 52 chars: outside both the entr {50,56,62,69,75} and mnem {51,58,64,70,77} length sets.
    let v = run_and_parse(&[
        "decode",
        "ms10entrsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx", // 52 chars
        "--json",
    ]);
    assert_eq!(v["error"]["kind"], "UnexpectedStringLength");
    assert_eq!(v["error"]["exit_code"], 1);
    assert_eq!(v["error"]["details"]["got"], 52);
}

#[test]
fn format_violation_json_envelope() {
    // Wrong HRP.
    let mut data = vec![0x00u8];
    data.extend_from_slice(&[0xAAu8; 16]);
    let s = Codex32String::from_seed("mq", 0, "entr", Fe::S, &data)
        .unwrap()
        .to_string();
    let v = run_and_parse(&["decode", &s, "--json"]);
    assert_eq!(v["error"]["kind"], "WrongHrp");
    assert_eq!(v["error"]["exit_code"], 2);
}

#[test]
fn future_format_json_envelope() {
    let mut data = vec![0x00u8];
    data.extend_from_slice(&[0xAAu8; 16]);
    let s = Codex32String::from_seed("ms", 0, "seed", Fe::S, &data)
        .unwrap()
        .to_string();
    let v = run_and_parse(&["verify", &s, "--json"]);
    assert_eq!(v["error"]["kind"], "FutureFormat");
    assert_eq!(v["error"]["exit_code"], 3);
    assert_eq!(v["error"]["details"]["tag"], "seed");
}
