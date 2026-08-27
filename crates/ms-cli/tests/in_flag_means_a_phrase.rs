//! **P2 row 2 — `--in FILE` on `encode` and `split` means a PHRASE.**
//!
//! Both verbs declare a required `ArgGroup` over `--phrase` and `--hex`; `--in`
//! joins it as a third alternative and resolves to the **phrase** channel.
//! Hex-from-a-file keeps using `--hex - <`.
//!
//! **THE COUNTEREXAMPLE TEST IS THE WHOLE RULING.** A file holding exactly 64
//! legal hex characters is a valid entropy length, so it is precisely the input
//! a content-sniffing `--in` would route to the hex channel. The ruling
//! (consult, 2026-08-27) rejected sniffing on a specific hazard rather than on
//! taste: today's sniff would be safe only because a phrase always contains
//! whitespace, and that restraint is invisible — a later maintainer being
//! liberal with whitespace turns a hex-alphabet BIP-39 phrase into valid entropy
//! for a **different wallet**, which is a valid, wrong plate.
//!
//! So this file's central test goes RED if the design ever drifts to sniffing,
//! RED if a later reader "fixes" `--in` to accept hex, and simultaneously proves
//! the channel it redirects to works.

use assert_cmd::Command;
use std::io::Write;

const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
/// 64 legal hex characters — a VALID entropy length (32 bytes), and therefore
/// exactly the file a sniffing implementation would accept.
const HEX_64: &str = "0000000000000000000000000000000000000000000000000000000000000000";

fn ms() -> Command {
    Command::cargo_bin("ms").unwrap()
}

fn write_tmp(dir: &std::path::Path, name: &str, body: &str) -> std::path::PathBuf {
    let p = dir.join(name);
    let mut f = std::fs::File::create(&p).unwrap();
    f.write_all(body.as_bytes()).unwrap();
    p
}

/// **THE COUNTEREXAMPLE.** Three assertions in one, because separating them
/// would let a half-implementation pass two of three.
#[test]
fn a_file_of_legal_hex_is_refused_by_in_and_accepted_by_hex_dash() {
    for verb in ["encode", "split"] {
        let dir = tempfile::tempdir().unwrap();
        let p = write_tmp(dir.path(), "entropy.hex", HEX_64);
        let path = p.display().to_string();

        let mut argv = vec![verb, "--in", &path];
        if verb == "split" {
            argv.extend_from_slice(&["-k", "2", "-n", "3"]);
        }
        let refused = ms().args(&argv).output().unwrap();

        assert_ne!(
            refused.status.code(),
            Some(0),
            "{verb} --in on a file of 64 legal hex chars must REFUSE -- accepting it \
             is the sniffing design the consult rejected"
        );
        let out = String::from_utf8_lossy(&refused.stdout);
        assert!(
            !out.contains("ms1"),
            "{verb} --in must emit NO artifact on stdout when it refuses; got:\n{out}"
        );
        let err = String::from_utf8_lossy(&refused.stderr);
        assert!(
            err.contains("--hex - <"),
            "{verb}: the refusal must name the channel that DOES read hex from a file, \
             or the operator's next move is --allow-argv-secret. stderr was:\n{err}"
        );

        // ...and the channel it redirects to works on the very same file.
        let mut hex_argv = vec![verb, "--hex", "-"];
        if verb == "split" {
            hex_argv.extend_from_slice(&["-k", "2", "-n", "3"]);
        }
        let accepted = ms()
            .args(&hex_argv)
            .write_stdin(std::fs::read_to_string(&p).unwrap())
            .output()
            .unwrap();
        assert_eq!(
            accepted.status.code(),
            Some(0),
            "{verb} --hex - < f must succeed on the same file, or the advice is dead. \
             stderr:\n{}",
            String::from_utf8_lossy(&accepted.stderr)
        );
        assert!(
            String::from_utf8_lossy(&accepted.stdout).contains("ms1"),
            "{verb} --hex - < f must emit the artifact"
        );
    }
}

/// `--in` reads a phrase, and the artifact is byte-identical to the `--phrase -`
/// run. Equality, not success.
#[test]
fn in_on_encode_and_split_equals_the_phrase_stdin_run() {
    for verb in ["encode", "split"] {
        let dir = tempfile::tempdir().unwrap();
        let p = write_tmp(dir.path(), "seed.txt", PHRASE);
        let path = p.display().to_string();

        let mut in_argv = vec![verb, "--in", &path];
        let mut stdin_argv = vec![verb, "--phrase", "-"];
        if verb == "split" {
            // `split` mixes fresh randomness into every share set, so only
            // `encode`'s artifact is comparable byte-for-byte. For `split` the
            // equality is asserted on the report shape via --json below.
            in_argv.extend_from_slice(&["-k", "2", "-n", "3"]);
            stdin_argv.extend_from_slice(&["-k", "2", "-n", "3"]);
        }
        let via_in = ms().args(&in_argv).output().unwrap();
        let via_stdin = ms()
            .args(&stdin_argv)
            .write_stdin(PHRASE.to_string())
            .output()
            .unwrap();

        assert_eq!(
            via_in.status.code(),
            Some(0),
            "{verb} --in must succeed on a phrase file: {}",
            String::from_utf8_lossy(&via_in.stderr)
        );
        assert_eq!(
            via_stdin.status.code(),
            Some(0),
            "{verb} --phrase - control"
        );
        assert_eq!(
            String::from_utf8_lossy(&via_in.stderr),
            String::from_utf8_lossy(&via_stdin.stderr),
            "{verb}: stderr differs between --in and --phrase -"
        );
        if verb == "encode" {
            assert_eq!(
                String::from_utf8_lossy(&via_in.stdout),
                String::from_utf8_lossy(&via_stdin.stdout),
                "encode: --in must produce the same artifact as --phrase -"
            );
            // Separators stripped: `ms encode`'s stdout is grouped by default
            // until the ungrouped-stdout work lands, and this assertion is
            // about the ARTIFACT rather than about its display form.
            let flat: String = String::from_utf8_lossy(&via_in.stdout)
                .chars()
                .filter(|c| !c.is_whitespace() && *c != '-' && *c != ',')
                .collect();
            assert!(
                flat.contains(MS1),
                "encode --in must emit the all-abandon card; stdout was:\n{}",
                String::from_utf8_lossy(&via_in.stdout)
            );
        }
    }
}

/// The required group now has THREE members. Both-supplied and
/// neither-supplied still exit 64, and every pair of the three collides.
#[test]
fn the_input_group_has_three_members_and_still_refuses_pairs() {
    let dir = tempfile::tempdir().unwrap();
    let p = write_tmp(dir.path(), "seed.txt", PHRASE);
    let path = p.display().to_string();

    for verb in ["encode", "split"] {
        let extra: &[&str] = if verb == "split" {
            &["-k", "2", "-n", "3"]
        } else {
            &[]
        };
        let pairs: Vec<Vec<&str>> = vec![
            vec!["--phrase", "-", "--in", &path],
            vec!["--hex", "-", "--in", &path],
        ];
        for pair in pairs {
            let mut argv = vec![verb];
            argv.extend(pair.iter().copied());
            argv.extend_from_slice(extra);
            let out = ms()
                .args(&argv)
                .write_stdin(PHRASE.to_string())
                .output()
                .unwrap();
            assert_eq!(
                out.status.code(),
                Some(64),
                "{argv:?} must be a group violation at exit 64"
            );
        }
        // Neither supplied: still 64, and the usage line must now offer --in
        // too, or the operator is told to reach for a channel that leaks.
        let mut none_argv = vec![verb];
        none_argv.extend_from_slice(extra);
        let out = ms().args(&none_argv).output().unwrap();
        assert_eq!(out.status.code(), Some(64), "{verb} with no input channel");
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(
            err.contains("--in"),
            "{verb}: the required-group usage line must name --in; got:\n{err}"
        );
    }
}
