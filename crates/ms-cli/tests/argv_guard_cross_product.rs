//! **P2 row 5 — the argv guard, gated by a GENERATED cross-product.**
//!
//! Not a hand list. Two earlier drafts of the donor's version of this work
//! enumerated first surfaces and then shapes, and both lists came up short; a
//! cross-product cannot be short, because the set is computed.
//!
//! **The arithmetic, and it is 92 rather than 56.** `ms` has **14**
//! secret-bearing channels across its eight material verbs — §6d's table gives
//! one per verb and understates `derive`, which has four. Nine are behind a
//! flag and five are positional. Four value spellings (canonical,
//! leading-space, trailing-space, UPPERCASE), times two join forms on the flag
//! channels (space-joined and `=`-joined) and one on the positional channels:
//!
//! ```text
//! 9 x 4 x 2  +  5 x 4  =  92
//! ```
//!
//! The first draft named 56, omitting the `=`-joined spelling entirely
//! (R0 round 0's C-1). F-302 records that `ms encode --phrase=<seed>` exits 0
//! and prints the artifact today, and that a guard whose gate is built from the
//! space-joined spellings alone would pass its own gate while leaking.
//!
//! ## The baseline, MEASURED before a line of the guard was written
//!
//! Generated and run against the tree's own build on 2026-08-27:
//!
//! | | |
//! | --- | --- |
//! | rows passing material at **exit 0** | **84** |
//! | rows already exiting non-zero | **8** |
//! | rows leaking material into stderr | **0 of 92** |
//!
//! So the leak half of the assertion was green everywhere before any code, and
//! only the exit-code half was a live gate on those 84.
//!
//! **THE 8 ARE WHY EVERY ROW ASSERTS THE GUARD'S OWN REFUSAL TEXT.** They are
//! UPPERCASE `--phrase` on `encode`, `verify`, `split` and `derive`, in both
//! join forms; each exits **1** with `error: unknown BIP-39 word at position 0`
//! and no material in stderr. Non-zero AND silent already — so *"exit non-zero
//! and no leak"* can never fail there, and clap's wordlist error would satisfy
//! it forever. Asserting a string only the guard emits is what makes those 8
//! rows gates instead of decoration; it costs nothing to apply to the other 84,
//! and applying it there removes the whole false-PASS class rather than eight
//! instances of it.
//!
//! ## The leak assertion is per WORD, never per character
//!
//! *"No whole material value, and no constituent word of 4+ characters, appears
//! in stderr, case-insensitively."* Not *"the material's own characters"* —
//! R0 round 0's M-6 showed that a 12-word English phrase makes a per-character
//! assertion unsatisfiable against any English sentence, the canonical refusal
//! included.
//!
//! ## What the 92 does NOT cover, stated because an unstated gap is worse
//!
//! The `--` end-of-options form and any shape where material is neither a whole
//! token nor an `=`-delimited half. The first is asserted separately below
//! rather than left as a claim about the raw-argv scan. Abbreviated long flags
//! are not a shape on `ms` (`ms encode --phr <seed>` exits 64,
//! `error: unexpected argument '--phr' found`), and no material channel has a
//! short alias — only `-h`, and `split`'s `-k`/`-n`, which carry no material.

use assert_cmd::Command;
use std::io::Write;

const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const HEX: &str = "00000000000000000000000000000000";
const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
/// Deliberately not ordinary English: the leak assertion is a case-insensitive
/// substring search for every 4+ character word of the material, so a passphrase
/// of common words would collide with the refusal's own prose and report a leak
/// that is not one.
const PASSPHRASE: &str = "zephyr frobnitz wibblewold";

/// A string only the guard emits. Asserting it is what stops clap's own errors
/// from satisfying these rows.
const GUARD_MARK: &str = "Refused BEFORE the command line was parsed";

fn ms() -> Command {
    Command::cargo_bin("ms").unwrap()
}

fn write_tmp(dir: &std::path::Path, name: &str, body: &str) -> std::path::PathBuf {
    let p = dir.join(name);
    let mut f = std::fs::File::create(&p).unwrap();
    f.write_all(body.as_bytes()).unwrap();
    p
}

#[derive(Clone)]
struct Channel {
    verb: &'static str,
    /// `None` for a positional channel.
    flag: Option<&'static str>,
    values: Vec<String>,
    /// Everything else the verb needs in order to reach the material at all —
    /// `-k`/`-n` on `split`, and a private `--in` for the ms1 a `--phrase` or
    /// `--passphrase` row would otherwise want on stdin.
    extra: Vec<String>,
}

fn spell(v: &str, how: &str) -> String {
    match how {
        "canonical" => v.to_string(),
        "leading-space" => format!(" {v}"),
        "trailing-space" => format!("{v} "),
        "UPPERCASE" => v.to_uppercase(),
        _ => unreachable!(),
    }
}

const SPELLINGS: [&str; 4] = ["canonical", "leading-space", "trailing-space", "UPPERCASE"];

fn channels(card: &str, shares: &[String]) -> (Vec<Channel>, Vec<Channel>) {
    let s = |v: &str| vec![v.to_string()];
    let card_in = vec!["--in".to_string(), card.to_string()];
    let kn = vec![
        "-k".to_string(),
        "2".to_string(),
        "-n".to_string(),
        "3".to_string(),
    ];
    let flag = vec![
        Channel {
            verb: "encode",
            flag: Some("--phrase"),
            values: s(PHRASE),
            extra: vec![],
        },
        Channel {
            verb: "encode",
            flag: Some("--hex"),
            values: s(HEX),
            extra: vec![],
        },
        Channel {
            verb: "verify",
            flag: Some("--phrase"),
            values: s(PHRASE),
            extra: card_in.clone(),
        },
        Channel {
            verb: "repair",
            flag: Some("--ms1"),
            values: s(MS1),
            extra: vec![],
        },
        Channel {
            verb: "split",
            flag: Some("--phrase"),
            values: s(PHRASE),
            extra: kn.clone(),
        },
        Channel {
            verb: "split",
            flag: Some("--hex"),
            values: s(HEX),
            extra: kn,
        },
        Channel {
            verb: "derive",
            flag: Some("--hex"),
            values: s(HEX),
            extra: vec![],
        },
        Channel {
            verb: "derive",
            flag: Some("--phrase"),
            values: s(PHRASE),
            extra: vec![],
        },
        Channel {
            verb: "derive",
            flag: Some("--passphrase"),
            values: s(PASSPHRASE),
            extra: card_in,
        },
    ];
    let positional = vec![
        Channel {
            verb: "decode",
            flag: None,
            values: s(MS1),
            extra: vec![],
        },
        Channel {
            verb: "verify",
            flag: None,
            values: s(MS1),
            extra: vec![],
        },
        Channel {
            verb: "inspect",
            flag: None,
            values: s(MS1),
            extra: vec![],
        },
        Channel {
            verb: "combine",
            flag: None,
            values: shares.to_vec(),
            extra: vec![],
        },
        Channel {
            verb: "derive",
            flag: None,
            values: s(MS1),
            extra: vec![],
        },
    ];
    (flag, positional)
}

struct Row {
    argv: Vec<String>,
    values: Vec<String>,
    label: String,
}

fn rows(card: &str, shares: &[String]) -> Vec<Row> {
    let (flag, positional) = channels(card, shares);
    let mut out = Vec::new();
    for ch in &flag {
        for sp in SPELLINGS {
            for join in ["space", "equals"] {
                let mut argv = vec![ch.verb.to_string()];
                for v in &ch.values {
                    let val = spell(v, sp);
                    match join {
                        "space" => argv.extend([ch.flag.unwrap().to_string(), val]),
                        _ => argv.push(format!("{}={}", ch.flag.unwrap(), val)),
                    }
                }
                argv.extend(ch.extra.iter().cloned());
                out.push(Row {
                    argv,
                    values: ch.values.clone(),
                    label: format!("{} {} / {sp} / {join}", ch.verb, ch.flag.unwrap()),
                });
            }
        }
    }
    for ch in &positional {
        for sp in SPELLINGS {
            let mut argv = vec![ch.verb.to_string()];
            argv.extend(ch.values.iter().map(|v| spell(v, sp)));
            argv.extend(ch.extra.iter().cloned());
            out.push(Row {
                argv,
                values: ch.values.clone(),
                label: format!("{} <positional> / {sp}", ch.verb),
            });
        }
    }
    out
}

/// The whole value, plus every constituent word of 4+ characters, lowercased.
fn leak_needles(values: &[String]) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    for value in values {
        v.push(value.to_lowercase());
        for w in value.split_whitespace() {
            if w.chars().count() >= 4 {
                v.push(w.to_lowercase());
            }
        }
    }
    v
}

/// **THE GATE.** All 92 rows, generated.
#[test]
fn every_argv_channel_refuses_with_the_guards_own_text_and_leaks_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let card = write_tmp(dir.path(), "card.ms1", MS1).display().to_string();
    let shares = split_two_shares();

    let all = rows(&card, &shares);
    assert_eq!(
        all.len(),
        92,
        "9 flag channels x 4 spellings x 2 join forms + 5 positional x 4 = 92; a \
         different count means a channel was added or dropped without the \
         arithmetic being revisited"
    );

    let mut failures: Vec<String> = Vec::new();
    for row in &all {
        let out = ms().args(&row.argv).write_stdin("").output().unwrap();
        let err = String::from_utf8_lossy(&out.stderr).to_string();

        if out.status.code() == Some(0) {
            failures.push(format!("[{}] exited 0 -- material was ACCEPTED", row.label));
            continue;
        }
        if !err.contains(GUARD_MARK) {
            failures.push(format!(
                "[{}] exited {:?} but WITHOUT the guard's own refusal -- something \
                 else refused it, and this row proves nothing. stderr:\n{err}",
                row.label,
                out.status.code()
            ));
        }
        let lower = err.to_lowercase();
        for needle in leak_needles(&row.values) {
            if lower.contains(&needle) {
                failures.push(format!("[{}] LEAKED `{needle}` into stderr", row.label));
            }
        }
        if !String::from_utf8_lossy(&out.stdout).is_empty() {
            failures.push(format!("[{}] wrote to stdout while refusing", row.label));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} rows failed:\n{}",
        failures.len(),
        all.len(),
        failures.join("\n")
    );
}

/// **The gap the 92 does not cover, closed rather than described.** The
/// raw-argv scan reaches `--` because it implements no end-of-options; nothing
/// in the cross-product proves that, so this does. Measured before the guard:
/// `ms decode -- <ms1>` exits 0.
#[test]
fn end_of_options_does_not_buy_past_the_guard() {
    for argv in [
        vec!["decode", "--", MS1],
        vec!["encode", "--", "--phrase", PHRASE],
    ] {
        let out = ms().args(&argv).write_stdin("").output().unwrap();
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(
            err.contains(GUARD_MARK),
            "{argv:?}: `--` must not buy past a scan of RAW argv. stderr:\n{err}"
        );
    }
}

/// **The clap-echo control, and it is the one the naive implementation fails.**
/// With an unknown flag present, clap's error names the offending token — so a
/// guard that ran after the parser would refuse the secret while printing it.
#[test]
fn an_unknown_flag_does_not_let_clap_echo_the_material() {
    let out = ms()
        .args(["encode", "--nosuchflag", "--phrase", PHRASE])
        .write_stdin("")
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains(GUARD_MARK),
        "the guard must decide BEFORE clap; got:\n{err}"
    );
    for needle in leak_needles(&[PHRASE.to_string()]) {
        assert!(
            !err.to_lowercase().contains(&needle),
            "clap echoed `{needle}`:\n{err}"
        );
    }
}

/// **The near-miss control.** A FILENAME that starts with the HRP is not
/// material, and refusing it would refuse every dated backup an operator keeps.
#[test]
fn a_filename_containing_the_hrp_is_still_accepted() {
    let out = ms()
        .args(["verify", "--in", "ms1-2026-08-23-backup.txt"])
        .write_stdin("")
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        !err.contains(GUARD_MARK),
        "the guard refused a filename; it must classify by CHARSET, not by prefix:\n{err}"
    );
    // It still fails, for the right reason: there is no such file.
    assert!(err.contains("ms1-2026-08-23-backup.txt"));
}

/// **The negative control.** If these were refused, the guard would be keying
/// on the binary's name rather than on the material.
#[test]
fn the_verbs_that_carry_no_material_still_run() {
    let dir = tempfile::tempdir().unwrap();
    let man = dir.path().join("man");
    let cases: Vec<Vec<String>> = vec![
        vec!["vectors".into()],
        vec!["gui-schema".into()],
        vec!["gen-man".into(), "--out".into(), man.display().to_string()],
    ];
    for argv in cases {
        let out = ms().args(&argv).write_stdin("").output().unwrap();
        assert_eq!(
            out.status.code(),
            Some(0),
            "{argv:?} carries no material and must still exit 0. stderr:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

/// The stdin idiom is untouched: `-` on every channel still reaches the code.
#[test]
fn the_stdin_sentinel_is_not_refused_on_any_channel() {
    let cases: Vec<(Vec<&str>, &str)> = vec![
        (vec!["encode", "--phrase", "-"], PHRASE),
        (vec!["encode", "--hex", "-"], HEX),
        (vec!["decode", "-"], MS1),
        (vec!["verify", "-"], MS1),
        (vec!["inspect", "-"], MS1),
        (vec!["repair", "--ms1", "-"], MS1),
        (vec!["derive", "-"], MS1),
        (vec!["split", "--phrase", "-", "-k", "2", "-n", "3"], PHRASE),
    ];
    for (argv, stdin) in cases {
        let out = ms()
            .args(&argv)
            .write_stdin(stdin.to_string())
            .output()
            .unwrap();
        assert_eq!(
            out.status.code(),
            Some(0),
            "{argv:?} must still work. stderr:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
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
    assert_eq!(out.status.code(), Some(0), "split must succeed");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .take(2)
        .map(str::to_string)
        .collect()
}
