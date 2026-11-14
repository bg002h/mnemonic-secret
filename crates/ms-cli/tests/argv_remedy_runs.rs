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
use std::{fs, path::Path};

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

// ---- Every printed line RUN, in a directory holding only the material that tripped the guard
// (plus the files the holder's own command named), with FILE/K/N filled in.

const PHRASE12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const CARD: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
const PASS: &str = "hunter2hunter2";

#[derive(Clone, Copy)]
enum M {
    Phrase,
    Hex,
    Card,
    Plate,
    Pass,
    HlPhrase,
}

/// Write the material under the name its channel uses; return (file name, content).
fn material(dir: &Path, m: M) -> (&'static str, String) {
    let (name, s) = match m {
        M::Phrase => ("seed.txt", PHRASE12.to_string()),
        M::Hex => ("entropy.hex", HEX.to_string()),
        M::Card => ("card.ms1", CARD.to_string()),
        M::Plate => {
            let f = dir.join("card.ms1");
            assert!(ms()
                .args(["hashlock", "--random", "--out"])
                .arg(&f)
                .output()
                .unwrap()
                .status
                .success());
            (
                "card.ms1",
                fs::read_to_string(&f).unwrap().trim().to_string(),
            )
        }
        M::Pass => ("pass.txt", PASS.to_string()),
        M::HlPhrase => ("phrase.txt", "correct horse battery staple".to_string()),
    };
    fs::write(dir.join(name), &s).unwrap();
    if matches!(m, M::Hex) {
        fs::write(dir.join("preimage.hex"), &s).unwrap();
    }
    (name, s)
}

/// Trigger with `args` (`@` = the material, literally on argv); run every printed line.
fn row(m: M, args: &[&str], aux: &[(&str, &str)]) {
    let d = tempfile::tempdir().unwrap();
    for (n, c) in aux {
        fs::write(d.path().join(n), c).unwrap();
    }
    if !aux.iter().any(|(n, _)| *n == "pass.txt") {
        fs::write(d.path().join("pass.txt"), PASS).unwrap();
    }
    let (name, s) = material(d.path(), m);
    let argv: Vec<&str> = args
        .iter()
        .map(|a| if *a == "@" { s.as_str() } else { a })
        .collect();
    let o = ms().current_dir(d.path()).args(&argv).output().unwrap();
    let err = String::from_utf8_lossy(&o.stderr).to_string();
    assert!(
        !o.status.success() && err.contains("on ARGV"),
        "guard did not fire: {err}"
    );
    let lines: Vec<&str> = err
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("ms ") && !l.contains("HISTFILE"))
        .collect();
    assert!(!lines.is_empty(), "no printed line");
    let mut bad = Vec::new();
    for l in &lines {
        let body = l.split(" #").next().unwrap().trim();
        let (cmd, redir) = match body.split_once(" < ") {
            Some((c, r)) => (c, Some(r.trim())),
            None => (body, None),
        };
        let toks: Vec<String> = cmd
            .split_whitespace()
            .skip(1)
            .map(|w| match w {
                "FILE" => name.to_string(),
                "K" => "2".into(),
                "N" => "3".into(),
                o => o.into(),
            })
            .collect();
        let mut c = ms();
        c.current_dir(d.path()).args(&toks);
        match redir {
            Some(r) => {
                c.write_stdin(fs::read(d.path().join(r)).unwrap_or_default());
            }
            None if toks.iter().any(|t| t == "-") => {
                c.write_stdin(s.clone());
            }
            None => {}
        }
        let r = c.output().unwrap();
        if !r.status.success() {
            bad.push(format!(
                "printed `{l}` FAILED rc={:?}: {}",
                r.status.code(),
                String::from_utf8_lossy(&r.stderr)
                    .lines()
                    .next()
                    .unwrap_or("")
            ));
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

// A: the stdin line and split's missing -k/-n
#[test]
fn a_encode_phrase() {
    row(M::Phrase, &["encode", "--phrase", "@"], &[]);
}
#[test]
fn a_split_phrase() {
    row(
        M::Phrase,
        &["split", "-k", "2", "-n", "3", "--phrase", "@"],
        &[],
    );
}
#[test]
fn a_split_hex() {
    row(M::Hex, &["split", "-k", "2", "-n", "3", "--hex", "@"], &[]);
}
#[test]
fn a_repair_ms1() {
    row(M::Card, &["repair", "--ms1", "@"], &[]);
}
// B: verify's phrase
#[test]
fn b_verify_phrase() {
    row(
        M::Phrase,
        &["verify", "--in", "card.ms1", "--phrase", "@"],
        &[("card.ms1", CARD)],
    );
}
// E: a verb given material it has no channel for
#[test]
fn e_verify_hex() {
    row(
        M::Hex,
        &["verify", "--in", "card.ms1", "--hex", "@"],
        &[("card.ms1", CARD)],
    );
}
#[test]
fn e_decode_phrase() {
    row(M::Phrase, &["decode", "@"], &[]);
}
#[test]
fn e_inspect_hex() {
    row(M::Hex, &["inspect", "--hex", "@"], &[]);
}
#[test]
fn e_combine_phrase() {
    row(M::Phrase, &["combine", "--phrase", "@"], &[]);
}
#[test]
fn e_repair_hex() {
    row(M::Hex, &["repair", "--hex", "@"], &[]);
}
#[test]
fn e_encode_ms1() {
    row(M::Card, &["encode", "--phrase", "@"], &[]);
}
#[test]
fn e_split_ms1() {
    row(
        M::Card,
        &["split", "-k", "2", "-n", "3", "--phrase", "@"],
        &[],
    );
}
#[test]
fn e_encode_passphrase() {
    row(
        M::Pass,
        &["encode", "--in", "seed.txt", "--passphrase", "@"],
        &[("seed.txt", PHRASE12)],
    );
}
#[test]
fn e_verify_hashlock_phrase() {
    row(
        M::HlPhrase,
        &["verify", "--in", "card.ms1", "--hashlock-phrase", "@"],
        &[("card.ms1", CARD)],
    );
}
// M-2: derive --hex
#[test]
fn m2_derive_hex() {
    row(M::Hex, &["derive", "--hex", "@"], &[]);
}
// Controls: pass on master and with the fix (the plant check of the harness)
#[test]
fn k_encode_hex() {
    row(M::Hex, &["encode", "--hex", "@"], &[]);
}
#[test]
fn k_derive_phrase() {
    row(M::Phrase, &["derive", "--phrase", "@"], &[]);
}
#[test]
fn k_decode_ms1() {
    row(M::Card, &["decode", "@"], &[]);
}
#[test]
fn k_verify_ms1() {
    row(M::Card, &["verify", "@"], &[]);
}
#[test]
fn k_hashlock_hex() {
    row(M::Hex, &["hashlock", "--hex", "@"], &[]);
}
#[test]
fn k_hashlock_plate() {
    row(M::Plate, &["hashlock", "@"], &[]);
}
#[test]
fn k_hashlock_phrase() {
    row(M::HlPhrase, &["hashlock", "--hashlock-phrase", "@"], &[]);
}

// C: texts naming `ms hashlock <ms1>` / "the ms1 argument"
fn backticked_hashlock(e: &str) -> Vec<String> {
    e.split('`')
        .skip(1)
        .step_by(2)
        .filter(|s| s.starts_with("ms hashlock"))
        .map(String::from)
        .collect()
}
fn run_plate_cmd(dir: &Path, cmd: &str, plate: &str) -> Option<String> {
    let toks: Vec<String> = cmd
        .split_whitespace()
        .skip(1)
        .map(|w| match w {
            "<ms1>" => plate.to_string(),
            "FILE" => "card.ms1".into(),
            o => o.into(),
        })
        .collect();
    let mut c = ms();
    c.current_dir(dir).args(&toks);
    if toks.iter().any(|t| t == "-") {
        c.write_stdin(plate.to_string());
    }
    let r = c.output().unwrap();
    (!r.status.success()).then(|| {
        format!(
            "printed `{cmd}` FAILED rc={:?}: {}",
            r.status.code(),
            String::from_utf8_lossy(&r.stderr)
                .lines()
                .next()
                .unwrap_or("")
        )
    })
}
#[test]
fn c_payload_lang() {
    for verb in ["derive", "verify"] {
        let d = tempfile::tempdir().unwrap();
        let (_, p) = material(d.path(), M::Plate);
        let o = ms()
            .current_dir(d.path())
            .args([verb, "--in", "card.ms1"])
            .output()
            .unwrap();
        let e = String::from_utf8_lossy(&o.stderr).to_string();
        let cmds = backticked_hashlock(&e);
        assert!(!o.status.success() && !cmds.is_empty(), "{verb}: {e}");
        let bad: Vec<String> = cmds
            .iter()
            .filter_map(|c| run_plate_cmd(d.path(), c, &p))
            .collect();
        assert!(bad.is_empty(), "{verb}: {}", bad.join("\n"));
    }
}
#[test]
fn c_phrase_stdin_remedy() {
    let d = tempfile::tempdir().unwrap();
    let (_, p) = material(d.path(), M::Plate);
    let o = ms()
        .current_dir(d.path())
        .args(["hashlock", "--hashlock-phrase-stdin"])
        .write_stdin(p.clone())
        .output()
        .unwrap();
    let e = String::from_utf8_lossy(&o.stderr).to_string();
    assert!(
        !o.status.success() && e.contains("not a hashlock phrase"),
        "{e}"
    );
    // The remedy names channels; each named one must read the plate.
    let mut bad = Vec::new();
    if e.contains("ms1 argument") {
        bad.extend(run_plate_cmd(d.path(), "ms hashlock <ms1>", &p));
    }
    for c in ["ms hashlock --in FILE", "ms hashlock -"] {
        bad.extend(run_plate_cmd(d.path(), c, &p));
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}
// D: the brainwallet warning's alternative, completed to a command
#[test]
fn d_random_fragment() {
    let d = tempfile::tempdir().unwrap();
    let o = ms()
        .current_dir(d.path())
        .args(["hashlock", "--hashlock-phrase-stdin", "--method", "sha256"])
        .write_stdin("correct horse battery staple\n")
        .output()
        .unwrap();
    let e = String::from_utf8_lossy(&o.stderr).to_string();
    let line = e
        .lines()
        .find(|l| l.contains("diceware"))
        .expect("warning printed");
    let frag = line.rsplit("or ").next().unwrap().trim_end_matches('.');
    let toks: Vec<String> = std::iter::once("hashlock".to_string())
        .chain(frag.split_whitespace().map(|w| {
            if w == "FILE" {
                "r.ms1".into()
            } else {
                w.into()
            }
        }))
        .collect();
    let r = ms().current_dir(d.path()).args(&toks).output().unwrap();
    assert!(
        r.status.success(),
        "printed `ms {}` FAILED rc={:?}: {}",
        toks.join(" "),
        r.status.code(),
        String::from_utf8_lossy(&r.stderr)
            .lines()
            .next()
            .unwrap_or("")
    );
}
