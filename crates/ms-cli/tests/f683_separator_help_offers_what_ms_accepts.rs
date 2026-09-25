//! F-683: `--help` offered `space|hyphen|comma` after §6c had retired
//! `hyphen` and `comma` from emission, so the help taught two values that exit
//! 64. The offered list now comes from the value parser itself
//! (`format::SEPARATORS`), which is also what `gui-schema` reads.
//!
//! This test does not trust the list: it takes each value `gui-schema`
//! advertises and RUNS it on every verb that has `--separator`, and checks the
//! help's offer line against the same list.

use assert_cmd::Command;

const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fn ms() -> Command {
    Command::cargo_bin("ms").unwrap()
}

/// Every verb whose schema carries `--separator`, with its `choices`.
fn separator_verbs() -> Vec<(String, serde_json::Value)> {
    let out = ms().arg("gui-schema").output().unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let mut found = Vec::new();
    for sub in v["subcommands"].as_array().unwrap() {
        for f in sub["flags"].as_array().unwrap() {
            if f["name"] == "--separator" {
                found.push((sub["name"].as_str().unwrap().to_string(), f.clone()));
            }
        }
    }
    found
}

#[test]
fn every_separator_verb_is_the_known_three() {
    let names: Vec<String> = separator_verbs().into_iter().map(|(n, _)| n).collect();
    assert_eq!(
        names,
        ["encode", "hashlock", "split"],
        "a verb gained or lost --separator; extend the run below to cover it"
    );
}

#[test]
fn gui_schema_offers_exactly_what_ms_accepts() {
    for (verb, flag) in separator_verbs() {
        assert_eq!(flag["kind"], "dropdown", "{verb}: {flag}");
        assert_eq!(
            flag["choices"],
            serde_json::json!(["space"]),
            "{verb}: the schema must offer only the emitted separator: {flag}"
        );
    }
}

#[test]
fn each_offered_value_runs_and_the_help_offers_nothing_else() {
    let dir = tempfile::tempdir().unwrap();
    let seed = dir.path().join("seed.txt");
    std::fs::write(&seed, PHRASE).unwrap();
    let seed = seed.display().to_string();

    for (verb, flag) in separator_verbs() {
        let help = ms().args([&verb, "--help"]).output().unwrap();
        let help = String::from_utf8(help.stdout).unwrap();
        let entry: String = help
            .split("--separator <SEPARATOR>")
            .nth(1)
            .unwrap_or_else(|| panic!("{verb}: no --separator entry"))
            .split("\n      -")
            .next()
            .unwrap()
            .to_string();
        assert!(
            entry.contains("[possible values: space]"),
            "{verb}: help must list the offered values: {entry}"
        );
        assert!(
            !entry.contains("space|hyphen|comma") && !entry.contains("|-|,"),
            "{verb}: help still offers a retired separator: {entry}"
        );

        for choice in flag["choices"].as_array().unwrap() {
            let choice = choice.as_str().unwrap();
            let mut argv: Vec<&str> = vec![&verb, "--separator", choice];
            match verb.as_str() {
                "encode" => argv.extend(["--in", &seed]),
                "split" => argv.extend(["--in", &seed, "-k", "2", "-n", "3"]),
                "hashlock" => argv.extend(["--kind", "sha256", "--hashlock-phrase-stdin"]),
                other => panic!("no invocation for {other}"),
            }
            let stdin = if verb == "hashlock" {
                "correct horse battery staple river\n"
            } else {
                ""
            };
            let out = ms().args(&argv).write_stdin(stdin).output().unwrap();
            assert_eq!(
                out.status.code(),
                Some(0),
                "ms {argv:?} was offered and must run; stderr:\n{}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
    }
}
