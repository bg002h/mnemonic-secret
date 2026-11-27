//! The constellation exit-code spec's §7c rows for `ms`
//! (`SPEC_constellation_exit_codes.md`, mnemonic-engrave `design/`): one run per
//! class, through the real binary. Codes: 0 OK, 1 NO, 2 INVALID, 4 UNVERIFIED,
//! 8 REFUSED, 16 REPAIRED, 32 IO (1-63 are sums), 64 USAGE, 101 INTERNAL.

use std::process::Output;

use assert_cmd::Command;
use ms_codec::codex32::{Codex32String, Fe};

const MS0: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
/// MS0 with one substitution in the data part (correctable).
const MS0_C1: &str = "ms10entrsqqqqqqqqpqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
const PHRASE0: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
/// A valid BIP-39 phrase for different entropy (0x7f * 16).
const PHRASE_OTHER: &str =
    "legal winner thank year wave sausage worth useful legal winner thank yellow";
/// Twelve words whose BIP-39 checksum fails.
const PHRASE_BAD: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";

fn ms(args: &[&str], stdin: &str) -> Output {
    Command::cargo_bin("ms")
        .unwrap()
        .args(args)
        .write_stdin(stdin.to_string())
        .output()
        .unwrap()
}

fn code(o: &Output) -> i32 {
    o.status.code().expect("exited, not signalled")
}

fn file(dir: &tempfile::TempDir, name: &str, body: &str) -> String {
    let p = dir.path().join(name);
    std::fs::write(&p, body).unwrap();
    p.display().to_string()
}

/// A reserved-not-emitted tag (`seed`): a valid future format.
fn future_format() -> String {
    let mut data = vec![0x00u8];
    data.extend_from_slice(&[0xAAu8; 16]);
    Codex32String::from_seed("ms", 0, "seed", Fe::S, &data)
        .unwrap()
        .to_string()
}

/// An unknown tag with a valid checksum: parses, would not decode.
fn unknown_tag() -> String {
    let mut data = vec![0x00u8];
    data.extend_from_slice(&[0u8; 16]);
    Codex32String::from_seed("ms", 0, "zzzz", Fe::S, &data)
        .unwrap()
        .to_string()
}

// ── 1 NO ──

#[test]
fn verify_phrase_mismatch_is_no_1() {
    let d = tempfile::tempdir().unwrap();
    let card = file(&d, "c", MS0);
    let other = PHRASE_OTHER;
    let o = ms(&["verify", "--in", &card, "--phrase", "-"], other);
    assert_eq!(code(&o), 1, "{}", String::from_utf8_lossy(&o.stderr));
    let o = ms(&["verify", "--in", &card, "--phrase", "-", "--json"], other);
    assert_eq!(code(&o), 1);
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(
        v["error"]["exit_code"], 1,
        "R14: the JSON carries the status"
    );
}

// ── 2 INVALID ──

#[test]
fn bad_checksum_and_bad_length_are_invalid_2() {
    let bad = format!("{}q", &MS0[..MS0.len() - 1]);
    assert_eq!(code(&ms(&["decode", "-"], &bad)), 2);
    assert_eq!(code(&ms(&["decode", "-"], &MS0[..40])), 2);
    let o = ms(&["decode", "-", "--json"], &bad);
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["error"]["exit_code"], 2);
}

#[test]
fn bad_phrase_and_bad_hex_are_invalid_2() {
    assert_eq!(code(&ms(&["encode", "--phrase", "-"], PHRASE_BAD)), 2);
    assert_eq!(code(&ms(&["encode", "--hex", "-"], "0")), 2);
}

#[test]
fn future_format_is_invalid_2_and_verify_stops_saying_ok() {
    let s = future_format();
    assert_eq!(code(&ms(&["decode", "-"], &s)), 2);
    let o = ms(&["verify", "-"], &s);
    assert_eq!(code(&o), 2);
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(
        !out.contains("OK"),
        "R5: the message agrees with the code: {out}"
    );
}

#[test]
fn empty_input_is_invalid_2() {
    let d = tempfile::tempdir().unwrap();
    let empty = file(&d, "empty", "\n");
    let runs: &[&[&str]] = &[
        &["decode", "-"],
        &["verify", "-"],
        &["inspect", "-"],
        &["derive", "-"],
        &["encode", "--phrase", "-"],
        &["encode", "--hex", "-"],
        &["repair", "--ms1", "-"],
        &["combine", "-"],
        &["hashlock", "--hex", "-"],
        &["decode", "--in", &empty],
    ];
    for args in runs {
        let o = ms(args, "  \n");
        assert_eq!(code(&o), 2, "{args:?}");
        // Final review N-4: the message says the input is empty.
        let err = String::from_utf8_lossy(&o.stderr);
        assert!(err.contains("is empty"), "{args:?}: {err}");
    }
}

#[test]
fn inspect_would_not_decode_prints_the_report_and_exits_2() {
    let o = ms(&["inspect", "-"], &unknown_tag());
    assert_eq!(code(&o), 2, "decision 8");
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(out.contains("FAIL: would NOT decode"), "{out}");
    assert!(out.contains("tag: zzzz"), "the whole report: {out}");
    let o = ms(&["inspect", "-", "--json"], &unknown_tag());
    assert_eq!(code(&o), 2);
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["would_decode"], false);
    // A clean card still inspects to 0.
    assert_eq!(code(&ms(&["inspect", "-"], MS0)), 0);
}

// ── 8 REFUSED ──

#[test]
fn secret_on_argv_is_refused_8() {
    assert_eq!(code(&ms(&["decode", MS0], "")), 8);
    assert_eq!(code(&ms(&["encode", "--phrase", PHRASE0], "")), 8);
    // The guard runs before clap (R2): a bad flag does not turn it into 64.
    assert_eq!(
        code(&ms(&["encode", "--nosuchflag", "--phrase", PHRASE0], "")),
        8
    );
}

#[test]
fn out_refusals_are_refused_8() {
    let d = tempfile::tempdir().unwrap();
    let phrase = file(&d, "p", PHRASE0);
    let existing = file(&d, "shares", "x\n");
    let o = ms(
        &[
            "split", "--in", &phrase, "-k", "2", "-n", "3", "--out", &existing,
        ],
        "",
    );
    assert_eq!(code(&o), 8, "{}", String::from_utf8_lossy(&o.stderr));
}

#[test]
fn digest_shaped_phrase_is_refused_8() {
    let digest = "a".repeat(64);
    let o = ms(&["hashlock", "--hashlock-phrase-stdin"], &digest);
    assert_eq!(code(&o), 8, "{}", String::from_utf8_lossy(&o.stderr));
}

// ── 20 REPAIRED + UNVERIFIED, 0, 2 ──

#[test]
fn repair_codes() {
    let o = ms(&["repair", "--ms1", "-"], MS0_C1);
    assert_eq!(code(&o), 20, "a correction with no oracle: 16 + 4");
    assert!(String::from_utf8_lossy(&o.stdout).contains(MS0));
    let o = ms(&["repair", "--ms1", "-", "--json"], MS0_C1);
    assert_eq!(code(&o), 20);
    assert_eq!(code(&ms(&["repair", "--ms1", "-"], MS0)), 0);
    // Six substitutions: uncorrectable.
    let six = "ms10entrsqpppppppqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
    assert_eq!(code(&ms(&["repair", "--ms1", "-"], six)), 2);
}

// ── 64 USAGE ──

#[test]
fn usage_is_64() {
    assert_eq!(code(&ms(&["bogus"], "")), 64);
    let d = tempfile::tempdir().unwrap();
    let phrase = file(&d, "p", PHRASE0);
    // Two readers on stdin: judged from the command line alone.
    assert_eq!(code(&ms(&["verify", "-", "--phrase", "-"], "")), 64);
    // K and N are settings.
    assert_eq!(
        code(&ms(&["split", "--in", &phrase, "-k", "1", "-n", "3"], "")),
        64
    );
}

/// R2: a setting is judged before any input is read or any refusal runs.
/// Each run below would otherwise read garbage (2) or meet a refusal (8).
#[test]
fn usage_is_judged_before_input_and_refusals() {
    // An `@env:` NAME is part of the command line.
    let o = ms(&["derive", "-", "--passphrase", "@env:bad-name"], "garbage");
    assert_eq!(code(&o), 64, "{}", String::from_utf8_lossy(&o.stderr));
    // An account index is a setting: hardened indices stop at 2^31 - 1.
    let o = ms(
        &[
            "derive",
            "-",
            "--template",
            "bip84",
            "--account",
            "2147483648",
        ],
        "garbage",
    );
    assert_eq!(code(&o), 64, "{}", String::from_utf8_lossy(&o.stderr));
    let o = ms(
        &[
            "derive",
            "-",
            "--template",
            "bip84",
            "--account",
            "2147483647",
        ],
        MS0,
    );
    assert_eq!(code(&o), 0, "{}", String::from_utf8_lossy(&o.stderr));
    // Two hashlock sources with stdout aliasing `--out`: the count is a usage
    // error (64) before the alias refusal (8).
    let d = tempfile::tempdir().unwrap();
    let out = d.path().join("o");
    let f = std::fs::File::create(&out).unwrap();
    let o = std::process::Command::new(env!("CARGO_BIN_EXE_ms"))
        .args(["hashlock", "--hex", "-", "--random", "--out"])
        .arg(&out)
        .stdout(f)
        .output()
        .unwrap();
    assert_eq!(code(&o), 64, "{}", String::from_utf8_lossy(&o.stderr));
}

// ── 32 IO ──

#[test]
fn io_failures_are_32() {
    let d = tempfile::tempdir().unwrap();
    let missing = d.path().join("nope.txt").display().to_string();
    assert_eq!(code(&ms(&["decode", "--in", &missing], "")), 32);
    let phrase = file(&d, "p", PHRASE0);
    let unwritable = d.path().join("nodir").join("o").display().to_string();
    assert_eq!(
        code(&ms(&["encode", "--in", &phrase, "--out", &unwritable], "")),
        32
    );
    let card = file(&d, "c", MS0);
    let o = Command::cargo_bin("ms")
        .unwrap()
        .args([
            "derive",
            "--in",
            &card,
            "--passphrase",
            "@env:MS_EXIT_TEST_UNSET_VAR",
        ])
        .env_remove("MS_EXIT_TEST_UNSET_VAR")
        .output()
        .unwrap();
    assert_eq!(code(&o), 32, "{}", String::from_utf8_lossy(&o.stderr));
}

/// R8: a failed stdout write ADDS 32; it does not replace the verb's bits.
#[cfg(target_os = "linux")]
#[test]
fn stdout_failure_adds_32() {
    let full = || {
        std::fs::OpenOptions::new()
            .write(true)
            .open("/dev/full")
            .unwrap()
    };
    let d = tempfile::tempdir().unwrap();
    let phrase = file(&d, "p", PHRASE0);
    let o = std::process::Command::new(env!("CARGO_BIN_EXE_ms"))
        .args(["encode", "--in", &phrase])
        .stdout(full())
        .output()
        .unwrap();
    assert_eq!(code(&o), 32);
    let c1 = file(&d, "c1", MS0_C1);
    let o = std::process::Command::new(env!("CARGO_BIN_EXE_ms"))
        .args(["repair", "--in", &c1])
        .stdout(full())
        .output()
        .unwrap();
    assert_eq!(code(&o), 52, "20 + 32");
}

// ── R11 ──

#[test]
fn root_help_ends_with_the_legend() {
    let o = ms(&["--help"], "");
    assert_eq!(code(&o), 0);
    let flat = String::from_utf8_lossy(&o.stdout)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        flat.ends_with(
            "Exit status: 0 = done, every check passed. 64 = bad command line, nothing ran. \
             101 = internal error. 130 = stopped by the operator. Other codes from 128 up = \
             killed by a signal. Otherwise the sum (1 to 63) of: 1 a check said no; 2 an input \
             is invalid; 4 verify the result before trusting it; 8 refused (network mismatch, \
             secret on the command line, unsafe output); 16 a correction was applied; 32 a \
             file, device or helper failed."
        ),
        "{flat}"
    );
}

/// Decision 6: a correction whose result is not a card prints the corrected
/// string with the warning and exits 16 + 4 + 2 = 22; `--out` is not written.
#[test]
fn repair_of_a_corrected_non_card_is_22() {
    let good = unknown_tag();
    // One substitution in the data part (index 12 of the string).
    let mut chars: Vec<char> = good.chars().collect();
    chars[12] = if chars[12] == 'q' { 'p' } else { 'q' };
    let damaged: String = chars.into_iter().collect();
    let d = tempfile::tempdir().unwrap();
    let out = d.path().join("o.ms1");
    let o = ms(
        &["repair", "--ms1", "-", "--out", &out.display().to_string()],
        &damaged,
    );
    assert_eq!(code(&o), 22, "{}", String::from_utf8_lossy(&o.stderr));
    let stdout = String::from_utf8_lossy(&o.stdout);
    assert!(
        stdout.contains(&good),
        "the corrected string is shown: {stdout}"
    );
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        err.contains("UNVERIFIED") && err.contains("NOT an ms1 card"),
        "{err}"
    );
    assert!(!out.exists(), "a non-card is never filed");
    // The `--out` line follows its reason (cause, then consequence).
    let reason = err.find("NOT an ms1 card").unwrap();
    let not_written = err.find("was not written").expect("--out line");
    assert!(reason < not_written, "{err}");
    // R5 in JSON: the envelope says "not a card" too, not a bare candidate.
    let o = ms(&["repair", "--ms1", "-", "--json"], &damaged);
    assert_eq!(code(&o), 22);
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["verdict"], "candidate", "{v}");
    assert_eq!(v["not_a_card"]["kind"], "UnknownTag", "{v}");
    assert!(v["not_a_card"]["message"].as_str().is_some(), "{v}");
    // A correction that decodes carries no such field (shape unchanged at 20).
    let o = ms(&["repair", "--ms1", "-", "--json"], MS0_C1);
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert!(v.get("not_a_card").is_none(), "{v}");
    // The same string with a clean checksum fails atomically: 2, nothing printed.
    let o = ms(&["repair", "--ms1", "-"], &good);
    assert_eq!(code(&o), 2);
    assert!(o.stdout.is_empty());
}

/// Shares 0 and 1 of a 2-of-3 split of MS0's entropy (index order as encoded).
fn shares() -> Vec<String> {
    ms_codec::encode_shares(
        ms_codec::Tag::ENTR,
        ms_codec::Threshold::new(2).unwrap(),
        3,
        &ms_codec::Payload::Entr(vec![0u8; 16]),
    )
    .unwrap()
    .into_iter()
    .map(|s| s.to_string())
    .collect()
}

/// A share plate is a valid string of the format (final review I-2): a clean
/// share is echoed at 0, a corrected one is printed, filed and exits 20 (no
/// oracle), and the message names it a share. 22 stays for a non-card,
/// non-share string (see `repair_of_a_corrected_non_card_is_22`).
#[test]
fn repair_of_a_share_plate_is_0_or_20() {
    let set = shares();
    let share = &set[0];
    assert!(!share.starts_with("ms12s"), "a distributed share: {share}");
    let o = ms(&["repair", "--ms1", "-"], share);
    assert_eq!(code(&o), 0, "{}", String::from_utf8_lossy(&o.stderr));
    assert!(String::from_utf8_lossy(&o.stdout).contains(share.as_str()));
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        err.contains("share") && !err.contains("NOT an ms1"),
        "{err}"
    );

    let mut chars: Vec<char> = share.chars().collect();
    chars[12] = if chars[12] == 'q' { 'p' } else { 'q' };
    let damaged: String = chars.into_iter().collect();
    let d = tempfile::tempdir().unwrap();
    let out = d.path().join("share.ms1");
    let o = ms(
        &["repair", "--ms1", "-", "--out", &out.display().to_string()],
        &damaged,
    );
    assert_eq!(code(&o), 20, "{}", String::from_utf8_lossy(&o.stderr));
    assert!(String::from_utf8_lossy(&o.stdout).contains(share.as_str()));
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        err.contains("UNVERIFIED") && err.contains("share") && !err.contains("NOT an ms1"),
        "{err}"
    );
    assert_eq!(std::fs::read_to_string(&out).unwrap(), format!("{share}\n"));
    // JSON: the ordinary candidate envelope, marked as a share.
    let o = ms(&["repair", "--ms1", "-", "--json"], &damaged);
    assert_eq!(code(&o), 20);
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["verdict"], "candidate", "{v}");
    assert!(v.get("not_a_card").is_none(), "{v}");
    assert_eq!(v["share"]["threshold"], 2, "{v}");
    // The corrected string combines with a second share to the original secret.
    let o = ms(&["combine", "-"], &format!("{share}\n{}\n", set[1]));
    assert_eq!(code(&o), 0, "{}", String::from_utf8_lossy(&o.stderr));
}

/// R11 per verb (final review M-4): each verb's help, short and long, has one
/// `Exit status:` line naming the codes that verb can return.
#[test]
fn each_verb_help_states_its_exit_status() {
    let want: &[(&str, &[&str])] = &[
        ("derive", &["0 ", "2 ", "8 ", "32 ", "64 "]),
        ("encode", &["0 ", "2 ", "8 ", "32 ", "64 "]),
        ("decode", &["0 ", "2 ", "8 ", "32 ", "64 "]),
        ("hashlock", &["0 ", "2 ", "8 ", "32 ", "64 "]),
        ("inspect", &["0 ", "2 would not decode", "8 ", "32 ", "64 "]),
        ("verify", &["0 ", "1 ", "2 ", "8 ", "32 ", "64 "]),
        ("vectors", &["0 ", "32 ", "64 "]),
        ("gui-schema", &["0 ", "32 ", "64 "]),
        ("gen-man", &["0 ", "32 ", "64 "]),
        (
            "repair",
            &["0 ", "20 ", "22 ", "2 ", "8 ", "32 ", "52 ", "64 "],
        ),
        ("split", &["0 ", "2 ", "8 ", "32 ", "64 "]),
        ("combine", &["0 ", "2 ", "8 ", "32 ", "64 "]),
    ];
    for (verb, codes) in want {
        for flag in ["-h", "--help"] {
            let o = ms(&[verb, flag], "");
            assert_eq!(code(&o), 0);
            let out = String::from_utf8_lossy(&o.stdout);
            let lines: Vec<&str> = out
                .lines()
                .filter(|l| l.starts_with("Exit status:"))
                .collect();
            assert_eq!(lines.len(), 1, "{verb} {flag}: {out}");
            for c in *codes {
                assert!(
                    lines[0].contains(&format!(" {c}")),
                    "{verb} {flag} lacks `{c}`: {}",
                    lines[0]
                );
            }
        }
    }
}
