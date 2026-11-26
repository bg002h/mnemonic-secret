//! `ms derive --origin-path NAME` is the flag; `--template NAME` is its old
//! spelling, kept as a HIDDEN alias for several releases.
//!
//! "Template" in the constellation means a BIP-388 wallet descriptor template
//! (`wsh(sortedmulti(2,@0/**,@1/**))`). What `ms derive` takes is a named
//! derivation scheme that picks a key-origin path (`bip84` -> `m/84'/0'/0'`),
//! so the flag is `--origin-path`. Behaviour is unchanged: same values, same
//! output, same exit codes. Only the spelling `ms` prints and documents moved.

use std::process::Output;

use assert_cmd::Command;

mod support;

const ZEROS_HEX: &str = "00000000000000000000000000000000";

fn ms(args: &[&str]) -> Output {
    support::run(args)
}

/// The old spelling still parses and gives the same stdout, stderr and exit
/// code as the new one, for every value, in text and JSON mode.
#[test]
fn the_old_template_spelling_is_byte_identical_to_origin_path() {
    for name in [
        "bip44",
        "bip49",
        "bip84",
        "bip86",
        "bip48-p2wsh",
        "bip48-p2sh-p2wsh",
        "bip48",
        "bip48-p2tr",
        "bg002h-tr",
        "bg002h-wsh",
    ] {
        for json in [false, true] {
            let mut new_args = vec!["derive", "--hex", ZEROS_HEX, "--origin-path", name];
            let mut old_args = vec!["derive", "--hex", ZEROS_HEX, "--template", name];
            if json {
                new_args.push("--json");
                old_args.push("--json");
            }
            let new = ms(&new_args);
            let old = ms(&old_args);
            assert_eq!(
                new.status.code(),
                Some(0),
                "--origin-path {name} (json={json}) failed: {}",
                String::from_utf8_lossy(&new.stderr)
            );
            assert_eq!(
                old.status.code(),
                new.status.code(),
                "exit code differs for {name}"
            );
            assert_eq!(old.stdout, new.stdout, "stdout differs for {name}");
            assert_eq!(old.stderr, new.stderr, "stderr differs for {name}");
        }
    }
}

/// A bad value is refused the same way under either spelling.
#[test]
fn a_bad_value_exits_the_same_under_either_spelling() {
    let new = ms(&["derive", "--hex", ZEROS_HEX, "--origin-path", "bip99"]);
    let old = ms(&["derive", "--hex", ZEROS_HEX, "--template", "bip99"]);
    assert_ne!(new.status.code(), Some(0));
    assert_eq!(old.status.code(), new.status.code());
    assert_eq!(old.stdout, new.stdout);
    assert_eq!(old.stderr, new.stderr);
}

/// `ms derive --help` names `--origin-path`, calls its values derivation
/// schemes, and does not show the hidden old spelling.
#[test]
fn derive_help_shows_origin_path_and_not_template() {
    for flag in ["-h", "--help"] {
        let out = Command::cargo_bin("ms")
            .unwrap()
            .args(["derive", flag])
            .output()
            .expect("ms runs");
        assert!(out.status.success());
        let help = String::from_utf8(out.stdout).unwrap();
        assert!(help.contains("--origin-path"), "{help}");
        assert!(!help.contains("--template"), "{help}");
        assert!(!help.to_lowercase().contains("template"), "{help}");
        assert!(help.contains("derivation scheme"), "{help}");
    }
}

/// Top-level help names the new flag in the `derive` line.
#[test]
fn top_level_help_names_origin_path() {
    let out = Command::cargo_bin("ms")
        .unwrap()
        .arg("--help")
        .output()
        .expect("ms runs");
    let help = String::from_utf8(out.stdout).unwrap();
    assert!(help.contains("--origin-path"), "{help}");
    assert!(!help.contains("--template"), "{help}");
}

/// The bare-bip48 assumption note names the flag by its new spelling, even
/// when the operator typed the old one.
#[test]
fn the_bip48_note_prints_origin_path() {
    for flag in ["--origin-path", "--template"] {
        let o = ms(&["derive", "--hex", ZEROS_HEX, flag, "bip48"]);
        let e = String::from_utf8_lossy(&o.stderr);
        assert!(
            e.contains("--origin-path bip48 named no script type"),
            "{e}"
        );
        assert!(e.contains("--origin-path bip48-p2sh-p2wsh"), "{e}");
        assert!(!e.contains("--template"), "{e}");
    }
}

/// The `--phrase-stdin` refusal's recipe prints the new spelling.
#[test]
fn the_phrase_stdin_remedy_prints_origin_path() {
    let o = Command::cargo_bin("ms")
        .unwrap()
        .args(["derive", "--phrase-stdin"])
        .output()
        .expect("ms runs");
    let e = String::from_utf8_lossy(&o.stderr);
    assert!(
        e.contains("ms derive --in card.ms1 --origin-path <NAME>"),
        "{e}"
    );
    assert!(!e.contains("--template"), "{e}");
}
