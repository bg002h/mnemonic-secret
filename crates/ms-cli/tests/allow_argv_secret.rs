//! **P2 row 6 — `--allow-argv-secret` is a CHANNEL, not a flag.**
//!
//! Its own parse happens on raw argv, before `Cli::try_parse_from`, because it
//! cannot be honoured by a layer that has already handed the material to clap.
//!
//! **THE MATERIAL IS SUBSTITUTED OUT, NOT REMOVED**, and the difference is the
//! whole of R0 round 0's I-2. Removing the admitted token strands `encode` and
//! `split`, whose required `ArgGroup` then has no member — measured before this
//! work: `ms encode` exits **64** with `error: the following required arguments
//! were not provided:` followed by the group's own usage line, and removing only
//! the value gives `ms encode --phrase` → **64**, `error: a value is required
//! for '--phrase <PHRASE>' but none was supplied`. So the layer substitutes: the
//! admitted value becomes `-`, the stdin sentinel `ms` already parses on every
//! one of its fourteen channels, the override token itself is dropped, and the
//! material is seeded into the side channel `read_input` / `read_phrase_input` /
//! `read_shares` consult **before** stdin.
//!
//! `-` is not the material, so nothing §6d forbids is re-presented to clap.

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

/// **THE GATE.** The override proceeds, and produces byte-identical output to
/// the private channel — an equality, not a bare exit 0.
#[test]
fn the_override_proceeds_and_matches_the_private_channel_byte_for_byte() {
    let dir = tempfile::tempdir().unwrap();
    let seed = write_tmp(dir.path(), "seed.txt", PHRASE);

    let via_override = ms()
        .args(["encode", "--allow-argv-secret", "--phrase", PHRASE])
        .output()
        .unwrap();
    let via_in = ms()
        .args(["encode", "--in", &seed.display().to_string()])
        .output()
        .unwrap();

    assert_eq!(
        via_override.status.code(),
        Some(0),
        "the override must proceed: {}",
        String::from_utf8_lossy(&via_override.stderr)
    );
    assert_eq!(via_in.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&via_override.stdout),
        String::from_utf8_lossy(&via_in.stdout),
        "the override must produce the SAME artifact, not merely an artifact"
    );
}

/// **THE ASSERTION A `-`-SUBSTITUTING IMPLEMENTATION GETS WRONG.**
///
/// With stdin unreadable, an implementation whose `-` is a REAL stdin read
/// cannot survive: it gets EOF, the phrase is empty, and BIP-39 parsing fails.
/// Only a genuine side channel exits 0 here.
#[test]
fn the_admitted_material_does_not_come_from_stdin() {
    // The control first, and it is what makes the assertion mean something:
    // with the same empty stdin, the REAL stdin form fails.
    let control = ms()
        .args(["encode", "--phrase", "-"])
        .write_stdin("")
        .output()
        .unwrap();
    assert_ne!(
        control.status.code(),
        Some(0),
        "control: an empty stdin must NOT produce an artifact, or this test cannot \
         tell a side channel from a stdin read"
    );

    let out = ms()
        .args(["encode", "--allow-argv-secret", "--phrase", PHRASE])
        .write_stdin("")
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "the admitted material must reach the verb WITHOUT stdin: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("ms1"));
}

/// **THE ASSERTION THE NAIVE IMPLEMENTATION FAILS.** If the admitted token were
/// left in argv for clap, an unrelated later parse error would echo it. clap
/// must name the unknown flag and never the value.
#[test]
fn an_unrelated_parse_error_never_echoes_the_admitted_value() {
    let out = ms()
        .args([
            "encode",
            "--allow-argv-secret",
            "--nosuchflag",
            "--phrase",
            PHRASE,
        ])
        .write_stdin("")
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&out.stderr);
    assert_ne!(out.status.code(), Some(0));
    assert!(
        err.contains("--nosuchflag"),
        "clap must name the flag it could not parse: {err}"
    );
    for word in PHRASE.split_whitespace() {
        assert!(
            !err.to_lowercase().contains(word),
            "the admitted value was echoed back (`{word}`): {err}"
        );
    }
}

/// **Control 1.** With `-` already supplied, the override changes nothing.
#[test]
fn the_override_over_an_explicit_dash_behaves_as_without_it() {
    let with = ms()
        .args(["encode", "--allow-argv-secret", "--phrase", "-"])
        .write_stdin(PHRASE.to_string())
        .output()
        .unwrap();
    let without = ms()
        .args(["encode", "--phrase", "-"])
        .write_stdin(PHRASE.to_string())
        .output()
        .unwrap();
    assert_eq!(with.status.code(), without.status.code());
    assert_eq!(
        String::from_utf8_lossy(&with.stdout),
        String::from_utf8_lossy(&without.stdout)
    );
    assert_eq!(
        String::from_utf8_lossy(&with.stderr),
        String::from_utf8_lossy(&without.stderr)
    );
}

/// **Control 2.** The override is not a way to make a required group optional.
#[test]
fn the_override_alone_still_fails_the_required_group() {
    let out = ms()
        .args(["encode", "--allow-argv-secret"])
        .write_stdin("")
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(64),
        "no input channel was named, so the group is still unsatisfied: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// The override reaches every material channel, positional and flag-keyed, in
/// both join forms — not just `encode --phrase`.
#[test]
fn the_override_reaches_every_shape() {
    let dir = tempfile::tempdir().unwrap();
    let shares = split_two_shares();
    let cases: Vec<Vec<String>> = vec![
        vec!["decode".into(), "--allow-argv-secret".into(), MS1.into()],
        vec!["inspect".into(), "--allow-argv-secret".into(), MS1.into()],
        vec!["verify".into(), "--allow-argv-secret".into(), MS1.into()],
        vec!["derive".into(), "--allow-argv-secret".into(), MS1.into()],
        vec![
            "repair".into(),
            "--allow-argv-secret".into(),
            "--ms1".into(),
            MS1.into(),
        ],
        vec![
            "encode".into(),
            "--allow-argv-secret".into(),
            format!("--phrase={PHRASE}"),
        ],
        vec![
            "encode".into(),
            "--allow-argv-secret".into(),
            "--hex".into(),
            "00000000000000000000000000000000".into(),
        ],
        vec![
            "split".into(),
            "--allow-argv-secret".into(),
            "--phrase".into(),
            PHRASE.into(),
            "-k".into(),
            "2".into(),
            "-n".into(),
            "3".into(),
        ],
        vec![
            "combine".into(),
            "--allow-argv-secret".into(),
            shares[0].clone(),
            shares[1].clone(),
        ],
    ];
    for argv in cases {
        let out = ms().args(&argv).write_stdin("").output().unwrap();
        assert_eq!(
            out.status.code(),
            Some(0),
            "{argv:?} must proceed under the override: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    drop(dir);
}

/// **The override binds only where it is DECLARED.** `vectors`, `gui-schema`
/// and `gen-man` carry no material, so there is nothing to opt into — clap
/// rejects the flag there, which is the correct answer rather than a silent
/// acceptance.
#[test]
fn the_override_is_not_declared_on_the_verbs_that_carry_no_material() {
    for verb in ["vectors", "gui-schema"] {
        let out = ms()
            .args([verb, "--allow-argv-secret"])
            .write_stdin("")
            .output()
            .unwrap();
        assert_eq!(
            out.status.code(),
            Some(64),
            "{verb} does not declare the override"
        );
    }
}

/// **The `--allow-argv-secret` name appears in `--help` on all eight material
/// verbs**, or an operator meeting the refusal cannot find the escape it names.
#[test]
fn the_override_is_documented_on_every_material_verb() {
    for verb in [
        "encode", "decode", "inspect", "verify", "repair", "split", "combine", "derive",
    ] {
        let out = ms().args([verb, "--help"]).output().unwrap();
        assert!(
            String::from_utf8_lossy(&out.stdout).contains("--allow-argv-secret"),
            "{verb} --help must document the override"
        );
    }
}

/// **The freed-stdin work's oracle, reinstated live.** Row 3 pinned the
/// one-command argv fingerprint to a measured constant because the guard refuses
/// that invocation; the override is what makes it runnable again, so the
/// constant is re-derived rather than trusted.
#[test]
fn the_pinned_one_command_fingerprint_is_still_what_the_binary_derives() {
    let out = ms()
        .args([
            "derive",
            "--allow-argv-secret",
            "--phrase",
            PHRASE,
            "--passphrase",
            "correct horse battery staple",
        ])
        .write_stdin("")
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("master_fingerprint:  6090b661"),
        "freed_stdin.rs pins this constant; got:\n{}",
        String::from_utf8_lossy(&out.stdout)
    );
}

fn split_two_shares() -> Vec<String> {
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
            "0",
        ])
        .write_stdin(PHRASE.to_string())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .take(2)
        .map(str::to_string)
        .collect()
}
