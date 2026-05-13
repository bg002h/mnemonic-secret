//! `ms gui-schema` emits SPEC §7 JSON matching the `mnemonic-gui` schema-mirror contract.
//!
//! Tests the SPEC §7 invariants (Phase C.2 of `mnemonic-gui` v0.2):
//! - exits 0
//! - parseable JSON
//! - `version == 1`
//! - `cli == "ms"`
//! - `encode`, `decode`, `verify` subcommands present
//! - `encode --phrase` and `encode --hex` present as flags
//! - `encode --language` is `dropdown` with hyphenated `chinese-simplified` /
//!   `chinese-traditional` choices (not `simplifiedchinese` / `traditionalchinese`).

use assert_cmd::Command;
use serde_json::Value;

fn run_gui_schema() -> Value {
    let out = Command::cargo_bin("ms")
        .unwrap()
        .arg("gui-schema")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).expect("gui-schema stdout is utf-8");
    serde_json::from_str(&s).expect("gui-schema stdout parses as JSON")
}

fn find_subcommand<'a>(root: &'a Value, name: &str) -> &'a Value {
    let subs = root["subcommands"]
        .as_array()
        .expect("subcommands is array");
    subs.iter()
        .find(|s| s["name"].as_str() == Some(name))
        .unwrap_or_else(|| panic!("subcommand '{}' not present", name))
}

fn find_flag<'a>(sub: &'a Value, name: &str) -> &'a Value {
    let flags = sub["flags"].as_array().expect("flags is array");
    flags
        .iter()
        .find(|f| f["name"].as_str() == Some(name))
        .unwrap_or_else(|| panic!("flag '{}' not present on subcommand", name))
}

#[test]
fn gui_schema_exits_zero_and_emits_parseable_json() {
    // success() in run_gui_schema asserts exit 0; the helper also runs the
    // JSON parse — both invariants in one go.
    let v = run_gui_schema();
    assert!(v.is_object(), "root must be an object");
}

#[test]
fn gui_schema_version_is_1() {
    let v = run_gui_schema();
    assert_eq!(
        v["version"].as_u64(),
        Some(1),
        "SPEC §7: version field must be 1"
    );
}

#[test]
fn gui_schema_cli_is_ms() {
    let v = run_gui_schema();
    assert_eq!(
        v["cli"].as_str(),
        Some("ms"),
        "SPEC §7: cli field must be \"ms\""
    );
}

#[test]
fn gui_schema_contains_encode_decode_verify() {
    let v = run_gui_schema();
    for name in ["encode", "decode", "verify"] {
        find_subcommand(&v, name); // panics on missing
    }
}

#[test]
fn gui_schema_does_not_self_reference() {
    // The `gui-schema` subcommand itself MUST NOT appear in the JSON it
    // emits — GUI doesn't surface it, and including it would create a
    // recursive surface entry.
    let v = run_gui_schema();
    let subs = v["subcommands"].as_array().unwrap();
    assert!(
        !subs
            .iter()
            .any(|s| s["name"].as_str() == Some("gui-schema")),
        "gui-schema must not appear in its own subcommands list"
    );
}

#[test]
fn gui_schema_encode_has_phrase_and_hex_flags() {
    let v = run_gui_schema();
    let encode = find_subcommand(&v, "encode");
    find_flag(encode, "--phrase");
    find_flag(encode, "--hex");
}

#[test]
fn gui_schema_encode_language_is_dropdown_with_hyphenated_chinese() {
    let v = run_gui_schema();
    let encode = find_subcommand(&v, "encode");
    let language = find_flag(encode, "--language");
    assert_eq!(
        language["kind"].as_str(),
        Some("dropdown"),
        "--language must be a dropdown"
    );
    let choices: Vec<&str> = language["choices"]
        .as_array()
        .expect("dropdown choices must be an array")
        .iter()
        .map(|c| c.as_str().expect("choice must be a string"))
        .collect();
    // SPEC §7: hyphenated kebab-case matches the bip39-wordlist convention
    // used end-to-end through stdout / --json output. The bug case the
    // GUI would catch is the bip39::Language `SimplifiedChinese` debug name
    // leaking as `simplifiedchinese` — assert the kebab form is what we emit.
    assert!(
        choices.contains(&"chinese-simplified"),
        "expected \"chinese-simplified\" in dropdown choices; got {:?}",
        choices
    );
    assert!(
        choices.contains(&"chinese-traditional"),
        "expected \"chinese-traditional\" in dropdown choices; got {:?}",
        choices
    );
    assert!(
        !choices.iter().any(|c| c == &"simplifiedchinese"),
        "found bip39 debug name \"simplifiedchinese\" — should be \"chinese-simplified\""
    );
}

#[test]
fn gui_schema_decode_positional_ms1_is_optional() {
    // SPEC §2.2: `ms decode [MS1]` — positional is optional (stdin fallback).
    let v = run_gui_schema();
    let decode = find_subcommand(&v, "decode");
    let positionals = decode["positionals"].as_array().expect("positionals array");
    let ms1 = positionals
        .iter()
        .find(|p| p["name"].as_str() == Some("ms1"))
        .expect("ms1 positional present");
    assert_eq!(
        ms1["required"].as_bool(),
        Some(false),
        "ms decode ms1 positional must be optional"
    );
    assert_eq!(
        ms1["repeating"].as_bool(),
        Some(false),
        "ms decode ms1 positional must not be repeating"
    );
}

#[test]
fn gui_schema_json_flags_are_boolean_kind() {
    let v = run_gui_schema();
    for sub_name in ["encode", "decode", "inspect", "verify"] {
        let sub = find_subcommand(&v, sub_name);
        let json = find_flag(sub, "--json");
        assert_eq!(
            json["kind"].as_str(),
            Some("boolean"),
            "--json on {} must be kind=boolean",
            sub_name
        );
        assert!(
            json["choices"].is_null(),
            "--json on {} must have null choices (kind=boolean)",
            sub_name
        );
    }
}

#[test]
fn gui_schema_vectors_subcommand_present_with_pretty_flag() {
    // SPEC §2.5: `ms vectors --pretty` boolean flag.
    let v = run_gui_schema();
    let vectors = find_subcommand(&v, "vectors");
    let pretty = find_flag(vectors, "--pretty");
    assert_eq!(pretty["kind"].as_str(), Some("boolean"));
}

#[test]
fn gui_schema_inspect_subcommand_present() {
    let v = run_gui_schema();
    let inspect = find_subcommand(&v, "inspect");
    // inspect has only --json flag and one ms1 positional.
    find_flag(inspect, "--json");
    let positionals = inspect["positionals"]
        .as_array()
        .expect("positionals array");
    assert!(
        positionals
            .iter()
            .any(|p| p["name"].as_str() == Some("ms1")),
        "inspect must have an ms1 positional"
    );
}
