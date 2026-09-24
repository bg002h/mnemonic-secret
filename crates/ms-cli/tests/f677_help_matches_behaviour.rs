//! F-677 / F-670: help and hint text that disagreed with what `ms` does.
//!
//! - `ms encode --group-size` said it grouped "the emitted ms1 string"; since
//!   §6a it shapes only the stderr engraving card.
//! - Help EXAMPLES put the secret on argv, which the argv guard has refused
//!   since ms-cli 0.17.0 -- a taught one-liner that fails, whose only visible
//!   way forward is `--allow-argv-secret`. Every example is now RUN here.
//! - `ms split --out FILE` warned "stdout carries private key material" with
//!   stdout empty (the F-589 defect, on a second verb).
//! - The hashlock card's `for md compose:` fragment could not run against
//!   md-cli 0.20.x: a key-less path needs `--wrapper wsh --experimental
//!   --md-only` (F-670).

use std::path::Path;
use std::process::Command as Proc;

use assert_cmd::Command;

const Z12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const CANON: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
const WARN: &str = "stdout carries private key material";

fn ms() -> Command {
    Command::cargo_bin("ms").expect("ms binary")
}

fn long_help(verb: &str) -> String {
    let out = ms().args([verb, "--help"]).output().unwrap();
    assert!(out.status.success(), "ms {verb} --help failed");
    String::from_utf8(out.stdout).unwrap()
}

// ---------------------------------------------------------------------------
// encode --group-size
// ---------------------------------------------------------------------------

/// The help names what the flag shapes (the card) and what it does not
/// (stdout, --out, --json). The behaviour half is pinned by
/// `encode_grouping_flags.rs`; this pins the words to it.
#[test]
fn encode_group_size_help_describes_the_card_not_stdout() {
    let h = long_help("encode");
    let para: String = h
        .split("--group-size")
        .nth(1)
        .and_then(|s| s.split("--separator").next())
        .expect("no --group-size entry in `ms encode --help`")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        !para.contains("in the emitted ms1 string"),
        "the old wording is back: {para}"
    );
    for want in [
        "engraving card",
        "stdout, `--out` and `--json` always carry the canonical, unbroken ms1",
        "`--no-engraving-card` this flag has no effect",
    ] {
        assert!(para.contains(want), "missing {want:?} in: {para}");
    }

    // And the claim is true: a non-default group size leaves stdout unbroken.
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("p.txt"), Z12).unwrap();
    let out = ms()
        .current_dir(dir.path())
        .args(["encode", "--in", "p.txt", "--group-size", "3"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), format!("{CANON}\n"));
}

// ---------------------------------------------------------------------------
// Every help EXAMPLE runs.
// ---------------------------------------------------------------------------

/// Subcommands listed by `ms --help`, minus `help`.
fn subcommands() -> Vec<String> {
    let out = ms().arg("--help").output().unwrap();
    let h = String::from_utf8(out.stdout).unwrap();
    let section = h
        .split("Commands:")
        .nth(1)
        .and_then(|s| s.split("\n\n").next())
        .expect("no Commands: section");
    section
        .lines()
        .filter_map(|l| l.split_whitespace().next().map(str::to_string))
        .filter(|v| v != "help")
        .collect()
}

/// The `ms ...` lines of a verb's EXAMPLES block.
fn examples(verb: &str) -> Vec<String> {
    let h = long_help(verb);
    let Some(block) = h.split("EXAMPLES:\n").nth(1) else {
        return Vec::new();
    };
    block
        .split("\n\n")
        .next()
        .unwrap()
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("ms ") || l.contains("| ms "))
        .map(str::to_string)
        .collect()
}

/// A directory holding every file an example names, with real content.
fn fixtures(dir: &Path, verb: &str) {
    let w = |name: &str, body: &str| std::fs::write(dir.join(name), body).unwrap();
    // `ms hashlock`'s phrase.txt is a hashlock phrase, not a BIP-39 seed.
    if verb == "hashlock" {
        w("phrase.txt", "correct horse battery staple");
    } else {
        w("phrase.txt", Z12);
    }
    let ja = bip39::Mnemonic::from_entropy_in(bip39::Language::Japanese, &[0u8; 16]).unwrap();
    w("phrase-ja.txt", &ja.to_string());
    w("entropy.hex", "00000000000000000000000000000000");
    w("card.ms1", &format!("{CANON}\n"));
    let grouped: String = CANON
        .chars()
        .enumerate()
        .flat_map(|(i, c)| (i > 0 && i % 5 == 0).then_some(' ').into_iter().chain([c]))
        .collect();
    w("typed-back.txt", &format!("{grouped}\n"));
    // One substitution, well inside BCH's t=4.
    let mut broken: Vec<char> = CANON.chars().collect();
    broken[20] = if broken[20] == 'q' { 'p' } else { 'q' };
    w(
        "broken.txt",
        &format!("{}\n", broken.iter().collect::<String>()),
    );
    // Built from its own seed file: under `hashlock`, phrase.txt is not one.
    w(".seed-for-shares.txt", Z12);
    let split = ms()
        .current_dir(dir)
        .args([
            "split",
            "--in",
            ".seed-for-shares.txt",
            "-k",
            "2",
            "-n",
            "3",
            "--out",
            "shares.txt",
        ])
        .output()
        .unwrap();
    assert!(
        split.status.success(),
        "fixture: split failed:\n{}",
        String::from_utf8_lossy(&split.stderr)
    );
}

/// **Every EXAMPLES line in `ms`'s help is executed**, with the files it names
/// present. The guard's refusal is exit 1 and names "on ARGV"; a taught line
/// must never hit it. A `| jq FILTER` tail is replaced by parsing stdout as
/// JSON here (and, for a bare `.field` filter, checking the field exists), so
/// the suite does not depend on jq being installed.
#[test]
fn every_help_example_runs_and_none_puts_material_on_argv() {
    let bin = assert_cmd::cargo::cargo_bin("ms");
    let bindir = bin.parent().unwrap();
    let path = format!(
        "{}:{}",
        bindir.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    let verbs = subcommands();
    assert!(verbs.len() >= 12, "subcommand list looks short: {verbs:?}");
    let mut ran = 0;
    for verb in &verbs {
        let lines = examples(verb);
        // `derive` carries no EXAMPLES block; every other verb must, or this
        // test would pass by finding nothing.
        if verb != "derive" {
            assert!(!lines.is_empty(), "ms {verb} --help has no examples");
        }
        for line in lines {
            let (cmd, jq) = match line.split_once(" | jq ") {
                Some((c, f)) => (c.to_string(), Some(f.trim().trim_matches('\'').to_string())),
                None => (line.clone(), None),
            };
            let dir = tempfile::tempdir().unwrap();
            fixtures(dir.path(), verb);
            let out = Proc::new("bash")
                .args(["-o", "pipefail", "-c", &cmd])
                .current_dir(dir.path())
                .env("PATH", &path)
                .env("XDG_DATA_HOME", dir.path())
                .env("HOME", dir.path())
                .output()
                .unwrap();
            let code = out.status.code();
            let stderr = String::from_utf8_lossy(&out.stderr);
            assert!(
                !stderr.contains("on ARGV"),
                "`{line}` is refused by ms's own argv guard:\n{stderr}"
            );
            // `ms repair` on a damaged card is exit 4 (VERIFY-ME) by design.
            let ok = code == Some(0) || (verb == "repair" && code == Some(4));
            assert!(ok, "`{line}` exited {code:?}:\n{stderr}");
            if let Some(filter) = jq {
                let v: serde_json::Value = serde_json::from_slice(&out.stdout)
                    .unwrap_or_else(|e| panic!("`{line}`: stdout is not JSON ({e})"));
                if let Some(field) = filter.strip_prefix('.') {
                    if field.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                        assert!(
                            !v[field].is_null(),
                            "`{line}`: the JSON has no {field:?} for jq to select"
                        );
                    }
                }
            }
            ran += 1;
        }
    }
    assert!(ran >= 30, "only {ran} examples ran");
}

/// The repair examples specifically -- the F-677 item -- read the damaged card
/// from a file or stdin, never from argv.
#[test]
fn repair_examples_use_a_private_channel() {
    let lines = examples("repair");
    assert!(!lines.is_empty());
    for l in &lines {
        let private = l.contains("--in ") || l.contains("--ms1 - <");
        assert!(private, "repair example on argv: {l}");
        assert!(!l.contains("ms10"), "repair example carries an ms1: {l}");
    }
}

// ---------------------------------------------------------------------------
// split --out: the advisory fires only when stdout carries shares.
// ---------------------------------------------------------------------------

#[test]
fn split_stdout_advisory_fires_only_when_stdout_carries_shares() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("p.txt"), Z12).unwrap();
    let run = |extra: &[&str]| {
        ms().current_dir(dir.path())
            .args(["split", "--in", "p.txt", "-k", "2", "-n", "3"])
            .args(extra)
            .output()
            .unwrap()
    };

    // 1. --out FILE, text: stdout is empty, so the warning would be false.
    let a = run(&["--out", "a.txt"]);
    assert!(a.status.success());
    assert!(a.stdout.is_empty(), "fixture drifted: stdout not empty");
    assert!(
        !String::from_utf8_lossy(&a.stderr).contains(WARN),
        "warned that stdout carries key material when stdout is EMPTY:\n{}",
        String::from_utf8_lossy(&a.stderr)
    );
    let body = std::fs::read_to_string(dir.path().join("a.txt")).unwrap();
    assert_eq!(body.lines().count(), 3, "the shares went to the file");

    // 2. No --out: the shares are on stdout; the warning is true.
    let b = run(&[]);
    assert!(b.status.success());
    assert!(!b.stdout.is_empty());
    assert!(String::from_utf8_lossy(&b.stderr).contains(WARN));

    // 3. --out FILE --json: the JSON report on stdout carries every share.
    let c = run(&["--out", "c.txt", "--json"]);
    assert!(c.status.success());
    assert!(String::from_utf8_lossy(&c.stdout).contains("\"shares\""));
    assert!(
        String::from_utf8_lossy(&c.stderr).contains(WARN),
        "--json puts the shares on stdout and the warning was suppressed"
    );
}

// ---------------------------------------------------------------------------
// F-670: the md compose fragment carries what md-cli 0.20.x requires.
// ---------------------------------------------------------------------------

/// Measured against md-cli 0.20.2: `md compose` + this fragment (placeholder
/// replaced by `2of3`) exits 0 and prints the wsh template; dropping
/// `--md-only` exits 1 ("Pass --md-only"), dropping `--experimental` exits 1
/// ("this policy needs --experimental"), and `--wrapper tr` refuses a key-less
/// path outright. md is not a dependency of this repo, so the run lives in the
/// F-677 implementation report; this pins the text.
#[test]
fn the_md_compose_fragment_carries_wsh_experimental_and_md_only() {
    for kind in ["sha256", "hash256", "ripemd160", "hash160"] {
        let out = ms()
            .args(["hashlock", "--hashlock-phrase-stdin", "--kind", kind])
            .write_stdin("correct horse battery staple")
            .output()
            .unwrap();
        assert!(out.status.success());
        let se = String::from_utf8_lossy(&out.stderr);
        let line = se
            .lines()
            .find(|l| l.starts_with("for md compose:"))
            .unwrap_or_else(|| panic!("{kind}: no `for md compose:` line:\n{se}"));
        let frag: Vec<&str> = line["for md compose:".len()..].split_whitespace().collect();
        let pos = |t: &str| frag.iter().position(|x| *x == t);
        assert_eq!(
            frag.get(pos("--wrapper").expect("no --wrapper") + 1),
            Some(&"wsh"),
            "{kind}: {line}"
        );
        assert!(pos("--experimental").is_some(), "{kind}: {line}");
        assert!(pos("--md-only").is_some(), "{kind}: {line}");
        assert!(
            frag.iter()
                .any(|t| t.starts_with(&format!("keyless,{kind}="))),
            "{kind}: {line}"
        );
    }
}
