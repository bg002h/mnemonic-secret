//! **P2 row 10 — the whitespace-only separator (§6c).**
//!
//! `--separator` loses its `hyphen` and `comma` keywords, and their literal
//! forms, on `encode` and `split` alike — one `parse_separator` serves both, so
//! it cannot bind to one of them.
//!
//! **INTAKE IS NOT NARROWED, and that is the half that matters for funds.** A
//! plate already engraved from a hyphen-grouped card must still decode, so
//! `is_display_separator` keeps stripping `-` and `,` and `render_grouped` keeps
//! its `char` parameter. Narrowing emission is a uniformity decision; narrowing
//! intake would strand metal.
//!
//! **Measured before this row: all three exited 0**, and all three round-trip
//! through `ms decode` — which is precisely why the argument is a cross-tool one
//! and not a per-tool one.
//!
//! The conformance pin is untouched. `design/display-grouping-vectors.tsv` is
//! SHA-pinned and checked in CI; its 22 rows include 2 that render with `hyphen`
//! and 3 with `comma`, and they exercise `render_grouped` DIRECTLY — a pure
//! `(&str, usize, char)` function — never through the CLI. The pin is only
//! endangered if an implementer also narrows `render_grouped` or
//! `is_display_separator`, and the control below says they did not.

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

/// **THE GATE.** Both retired spellings, both keyword and literal, on both
/// verbs. `space` still works.
#[test]
fn hyphen_and_comma_are_refused_and_space_still_works() {
    let dir = tempfile::tempdir().unwrap();
    let seed = write_tmp(dir.path(), "seed.txt", PHRASE);
    let path = seed.display().to_string();

    for verb in ["encode", "split"] {
        let extra: &[&str] = if verb == "split" {
            &["-k", "2", "-n", "3"]
        } else {
            &[]
        };
        for retired in ["hyphen", "-", "comma", ","] {
            let mut argv = vec![verb, "--in", &path, "--separator", retired];
            argv.extend_from_slice(extra);
            let out = ms().args(&argv).write_stdin("").output().unwrap();
            assert_eq!(
                out.status.code(),
                Some(64),
                "{verb} --separator {retired:?} must be a usage error now; stderr:\n{}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
        for kept in ["space", " "] {
            let mut argv = vec![verb, "--in", &path, "--separator", kept];
            argv.extend_from_slice(extra);
            let out = ms().args(&argv).write_stdin("").output().unwrap();
            assert_eq!(
                out.status.code(),
                Some(0),
                "{verb} --separator {kept:?} must still work; stderr:\n{}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
    }
}

/// **Control 1 — INTAKE was not narrowed.** A plate engraved from a
/// hyphen-grouped or comma-grouped card still decodes. If this ever goes red,
/// metal that already exists has been stranded.
#[test]
fn already_engraved_hyphen_and_comma_cards_still_decode() {
    let dir = tempfile::tempdir().unwrap();
    for (name, sep) in [("hyphen.txt", '-'), ("comma.txt", ',')] {
        let grouped: String = MS1
            .chars()
            .enumerate()
            .flat_map(|(i, c)| {
                if i > 0 && i % 5 == 0 {
                    vec![sep, c]
                } else {
                    vec![c]
                }
            })
            .collect();
        let f = write_tmp(dir.path(), name, &grouped);
        let out = ms()
            .args(["decode", "--in", &f.display().to_string()])
            .output()
            .unwrap();
        assert_eq!(
            out.status.code(),
            Some(0),
            "a {sep}-grouped card must still decode: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(String::from_utf8_lossy(&out.stdout).contains(PHRASE));
    }
}

/// **Control 2 — the conformance pin is not dragged along.** The vectors file is
/// byte-unchanged and its checksum still verifies. Asserted here as well as in
/// CI so a developer machine catches it at the same moment CI would.
#[test]
fn the_display_grouping_conformance_pin_is_untouched() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("design");
    let out = std::process::Command::new("sha256sum")
        .arg("-c")
        .arg("display-grouping-vectors.tsv.sha256")
        .current_dir(&root)
        .output()
        .expect("sha256sum");
    assert!(
        out.status.success(),
        "the SHA-pinned conformance vectors changed: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let tsv = std::fs::read_to_string(root.join("display-grouping-vectors.tsv")).unwrap();
    let hyphen = tsv.lines().filter(|l| l.contains("hyphen")).count();
    let comma = tsv.lines().filter(|l| l.contains("comma")).count();
    assert!(
        hyphen > 0 && comma > 0,
        "the pin still exercises the two retired separators through render_grouped \
         directly ({hyphen} hyphen rows, {comma} comma rows) -- which is why \
         retiring the CLI keyword does not touch it"
    );
}
