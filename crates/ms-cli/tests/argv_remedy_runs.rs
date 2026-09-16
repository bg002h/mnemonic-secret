//! **F-581 — the argv guard's prescribed channels, RUN.**
//!
//! The guard identified the verb and the material precisely and then printed
//! ONE generic remedy for every verb:
//!
//! ```text
//!     ms <verb> --in FILE      # read it from a file
//!     ms <verb> -              # or pipe it on stdin
//! ```
//!
//! For `derive` BOTH are wrong, measured on the shipped binary:
//!
//! ```text
//! ms derive --in seed.txt  -> error: string length 164 not in v0.1 set […]
//! ms derive - < seed.txt   -> error: string length 164 not in v0.1 set […]
//! ```
//!
//! because `--in` and stdin on `derive` read an ms1, not a phrase. An operator
//! who runs the prescribed command, sees it fail, and retries on the command
//! line has been taught by the guard to defeat the guard.
//!
//! **This is the third time this class has shipped.** `me`'s
//! `tests/ms_remedy_runs.rs` exists because a remedy pipeline was shipped
//! broken twice (F-301), with a source comment above it asserting it was
//! "verified to pipe into pack". The lesson there was the same one here: a
//! remedy nobody executed is a guess in the imperative mood.
//!
//! So this file EXECUTES what the guard prints, rather than matching its text.

use assert_cmd::Command;
use std::fs;

const PHRASE: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
const HEX: &str = "0000000000000000000000000000000000000000000000000000000000000001";

fn ms() -> Command {
    Command::cargo_bin("ms").expect("ms binary")
}

/// Trigger the guard and return the `ms …` command lines it prescribes.
fn prescribed(args: &[&str]) -> Vec<String> {
    let out = ms().args(args).output().expect("ms runs");
    assert!(
        !out.status.success(),
        "the guard did not refuse, so there is no remedy to run"
    );
    String::from_utf8_lossy(&out.stderr)
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("ms "))
        // Drop the history-purge block: those are shell recipes, not ms calls.
        .filter(|l| !l.contains("HISTFILE"))
        .map(|l| l.split('#').next().unwrap().trim().to_string())
        .collect()
}

/// `derive` is the case the old text got wrong in both of its two lines.
#[test]
fn the_remedy_derive_prescribes_actually_derives() {
    let dir = tempfile::tempdir().unwrap();
    let seed = dir.path().join("seed.txt");
    let card = dir.path().join("card.ms1");
    fs::write(&seed, PHRASE).unwrap();

    let lines = prescribed(&["derive", "--phrase", PHRASE]);
    assert!(
        !lines.is_empty(),
        "the guard prescribed no ms command at all"
    );
    // It must NOT prescribe the channels that cannot read a phrase.
    for l in &lines {
        assert!(
            !(l.starts_with("ms derive --in seed") || l == "ms derive -"),
            "the guard still prescribes a channel that reads an ms1: {l}"
        );
    }

    // Run every prescribed `ms` line in order, with the placeholder filenames
    // bound to real files. If the guard's advice works, these all succeed.
    let mut ran = 0;
    for l in &lines {
        let argv: Vec<String> = l
            .split(" < ")
            .next()
            .unwrap()
            .split_whitespace()
            .skip(1)
            .map(|w| match w {
                "seed.txt" => seed.display().to_string(),
                "card.ms1" => card.display().to_string(),
                other => other.to_string(),
            })
            .collect();
        // The passphrase variant needs a stdin file the remedy only names; the
        // two core steps are what this test binds.
        if argv.iter().any(|a| a == "--passphrase-stdin") {
            continue;
        }
        let out = ms().args(&argv).output().expect("ms runs");
        assert!(
            out.status.success(),
            "the guard prescribed `{l}`, which FAILED:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
        ran += 1;
    }
    assert!(
        ran >= 2,
        "expected the two-step recipe, ran {ran} command(s)"
    );
    assert!(
        card.exists(),
        "the prescribed recipe never produced the card"
    );
}

/// `hashlock` with hex: `--in` does not read hex there either.
#[test]
fn the_remedy_hashlock_prescribes_actually_hashes() {
    let lines = prescribed(&["hashlock", "--hex", HEX]);
    assert!(!lines.is_empty(), "no remedy prescribed for hashlock");
    let l = &lines[0];
    assert!(
        l.contains("--hex -"),
        "hashlock's remedy does not name the hex channel: {l}"
    );
    // The line carries a SHELL REDIRECT (`< preimage.hex`); everything from
    // `<` on is the shell's, not argv's. Splitting on whitespace alone passed
    // the filename as an argument and `ms` correctly refused two sources.
    let argv: Vec<&str> = l
        .split(" < ")
        .next()
        .unwrap()
        .split_whitespace()
        .skip(1)
        .collect();
    ms().args(&argv).write_stdin(HEX).assert().success();
}

/// The control: `encode`'s generic advice was always correct, and must stay.
#[test]
fn the_remedy_encode_prescribes_still_works() {
    let dir = tempfile::tempdir().unwrap();
    let seed = dir.path().join("seed.txt");
    fs::write(&seed, PHRASE).unwrap();

    let lines = prescribed(&["encode", "--phrase", PHRASE]);
    assert!(
        lines.iter().any(|l| l.contains("--in FILE")),
        "encode lost its (correct) --in advice: {lines:?}"
    );
    ms().args([
        "encode",
        "--in",
        &seed.display().to_string(),
        "--group-size",
        "0",
    ])
    .assert()
    .success();
}

/// F-597 — the `--phrase-stdin` trap, and its remedy RUN.
///
/// `--phrase-stdin` is a name an operator reaches for and `derive` does not
/// have. Clap answered it with *"tip: a similar argument exists:
/// '--passphrase-stdin'"* — string-distance matching pointing at a DIFFERENT
/// SECRET. Following that tip feeds a seed phrase into the BIP-39 passphrase
/// slot and derives a different wallet, with no error anywhere.
///
/// Declaring `--phrase-stdin` hidden takes it out of clap's suggestion pool and
/// gives the refusal somewhere to put the real recipe. This test is in THIS
/// file, not beside the other derive tests, because it makes the same demand
/// the file exists for: the prescribed lines are EXECUTED, never matched. A
/// refusal that teaches a broken command is the class F-301 and F-581 already
/// shipped twice.
#[test]
fn the_phrase_stdin_trap_refuses_and_its_remedy_runs() {
    let dir = tempfile::tempdir().unwrap();
    let phrase = dir.path().join("phrase.txt");
    let card = dir.path().join("card.ms1");
    fs::write(&phrase, PHRASE).unwrap();

    let out = ms()
        .args(["derive", "--phrase-stdin", "--template", "bg002h-tr"])
        .output()
        .expect("ms runs");
    assert!(!out.status.success(), "--phrase-stdin was ACCEPTED");
    let err = String::from_utf8_lossy(&out.stderr).to_string();

    // The clap suggestion must be gone, and the warning about it present.
    assert!(
        !err.contains("a similar argument exists"),
        "clap still suggests a neighbour for --phrase-stdin:\n{err}"
    );
    assert!(
        err.contains("DO NOT reach for --passphrase-stdin"),
        "the refusal does not warn about the wrong-secret trap:\n{err}"
    );

    // And the two lines it prescribes must actually work, in order.
    let lines: Vec<String> = err
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("ms "))
        .map(str::to_string)
        .collect();
    assert_eq!(lines.len(), 2, "expected a two-step recipe, got {lines:?}");
    assert!(lines[0].starts_with("ms encode --in"), "{:?}", lines[0]);
    assert!(lines[1].starts_with("ms derive --in"), "{:?}", lines[1]);

    ms().args(["encode", "--in"])
        .arg(&phrase)
        .arg("--out")
        .arg(&card)
        .assert()
        .success();
    let derived = ms()
        .args(["derive", "--in"])
        .arg(&card)
        .args(["--template", "bg002h-tr"])
        .assert()
        .success();
    let stdout = String::from_utf8(derived.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("account_xpub:") && stdout.contains("xpub"),
        "the prescribed recipe ran but produced no account xpub:\n{stdout}"
    );
}
