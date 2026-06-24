//! `ms gen-man --out <DIR>` emits clap_mangen roff man pages — one per
//! (sub)command — into the output directory.
//!
//! Per SPEC_constellation_man_pages.md §8 P1:
//! - a non-empty `*.1` set is produced;
//! - the root page (`ms.1`) carries a `.TH` roff header;
//! - each subcommand yields a distinct hyphen-joined page filename;
//! - EXACT page-set: equals walking the UNBUILT `Cli::command()` tree
//!   minus `is_hide_set()` (and minus the auto `help`);
//! - NEGATIVE canary: ZERO `*-help*.1` pages (the tripwire for an accidental
//!   pre-`.build()` that would materialize the clap `help` shadow tree, C-1);
//! - NO assertion that any page contains the `global=true` flag — ms has none,
//!   and clap_mangen 0.3 renders global args in zero pages anyway (C-2).

use std::collections::BTreeSet;
use std::fs;

use assert_cmd::Command;

// ms-cli is a binary crate, so integration tests cannot import its `Cli` type
// directly. The EXACT page set is therefore reconstructed from the binary's own
// `gui-schema` subcommand inventory — the same `clap::Command` reflection the
// binary uses — plus the two visible developer subcommands (`gui-schema`,
// `gen-man`) that emit man pages but are filtered out of the schema JSON.

/// Authoritative expected page set: `ms.1` (root) + `ms-<sub>.1` for every
/// subcommand the binary's own clap tree exposes (via `gui-schema`), filtering
/// `help`. ms has no nested subcommands, so every page is a single hyphen join.
fn expected_from_schema() -> BTreeSet<String> {
    let out = Command::cargo_bin("ms")
        .unwrap()
        .arg("gui-schema")
        .output()
        .unwrap()
        .stdout;
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    let mut set = BTreeSet::new();
    set.insert("ms.1".to_string());
    for sub in v["subcommands"].as_array().unwrap() {
        let name = sub["name"].as_str().unwrap();
        set.insert(format!("ms-{name}.1"));
    }
    // gui-schema omits `gui-schema` and `gen-man`? Verify against the produced
    // set below; gui-schema currently filters `gui-schema` + `help` from its
    // own JSON, so we add the two visible developer subcommands that DO emit
    // man pages but are excluded from the schema JSON.
    set.insert("ms-gui-schema.1".to_string());
    set.insert("ms-gen-man.1".to_string());
    set
}

fn produced_pages(dir: &std::path::Path) -> BTreeSet<String> {
    fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".1"))
        .collect()
}

#[test]
fn gen_man_produces_nonempty_page_set() {
    let tmp = tempdir();
    Command::cargo_bin("ms")
        .unwrap()
        .args(["gen-man", "--out", tmp.to_str().unwrap()])
        .assert()
        .success();
    let pages = produced_pages(&tmp);
    assert!(!pages.is_empty(), "gen-man produced no *.1 pages");
}

#[test]
fn root_page_has_th_header() {
    let tmp = tempdir();
    Command::cargo_bin("ms")
        .unwrap()
        .args(["gen-man", "--out", tmp.to_str().unwrap()])
        .assert()
        .success();
    let root = tmp.join("ms.1");
    assert!(root.exists(), "root page ms.1 not produced");
    let body = fs::read_to_string(&root).unwrap();
    assert!(
        body.contains(".TH"),
        "root page ms.1 lacks a .TH roff header"
    );
}

#[test]
fn one_distinct_page_per_subcommand() {
    let tmp = tempdir();
    Command::cargo_bin("ms")
        .unwrap()
        .args(["gen-man", "--out", tmp.to_str().unwrap()])
        .assert()
        .success();
    let pages = produced_pages(&tmp);
    // a handful of known subcommand pages must each exist as a distinct file
    for expect in [
        "ms-encode.1",
        "ms-decode.1",
        "ms-split.1",
        "ms-combine.1",
        "ms-gen-man.1",
    ] {
        assert!(pages.contains(expect), "missing page {expect}");
    }
}

#[test]
fn exact_page_set_matches_unbuilt_tree() {
    let tmp = tempdir();
    Command::cargo_bin("ms")
        .unwrap()
        .args(["gen-man", "--out", tmp.to_str().unwrap()])
        .assert()
        .success();
    let produced = produced_pages(&tmp);
    let expected = expected_from_schema();
    assert_eq!(
        produced, expected,
        "produced man-page set != expected set derived from the clap tree"
    );
}

#[test]
fn negative_canary_no_help_pages() {
    let tmp = tempdir();
    Command::cargo_bin("ms")
        .unwrap()
        .args(["gen-man", "--out", tmp.to_str().unwrap()])
        .assert()
        .success();
    let produced = produced_pages(&tmp);
    let help_pages: Vec<_> = produced
        .iter()
        .filter(|n| {
            *n == "ms-help.1"
                || (n.starts_with("ms-help-") || n.contains("-help-")) && n.ends_with(".1")
        })
        .collect();
    assert!(
        help_pages.is_empty(),
        "found spurious *-help*.1 shadow pages (accidental pre-.build()?): {help_pages:?}"
    );
}

/// Minimal unique temp dir under the cargo target tmp (no external dep).
fn tempdir() -> std::path::PathBuf {
    use std::sync::atomic::{AtomicU32, Ordering};
    static CTR: AtomicU32 = AtomicU32::new(0);
    let n = CTR.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    let dir = std::env::temp_dir().join(format!("ms-gen-man-{pid}-{n}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}
