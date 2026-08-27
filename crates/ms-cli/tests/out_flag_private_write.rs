//! **P2 row 8 — the private write: `--out FILE` on `encode`, `split` and
//! `repair`.**
//!
//! `ms` has never created a file with a mode. Measured before this work:
//! `git grep -n 'fs::write\|OpenOptions\|set_permissions\|0o600\|0o077\|0o044\|st_mode'`
//! scoped to `crates/` returned **zero hits**, and `ms encode > backup.txt`
//! under the default umask 022 creates **0644** and exits 0. `--out` is the
//! first mode-aware code `ms` has, and it goes through the shared crate's
//! `write::write_private`.
//!
//! ## The gate is the PRE-EXISTING target, not the fresh one
//!
//! `OpenOptions::mode()` binds **on create only**. A `--out` implemented with it
//! alone leaves an existing `0644` file at `0644` and reports success — and that
//! is the case an operator re-running a command actually hits, not an exotic
//! one. The fresh-file half passes under any implementation that sets a mode at
//! all, so on its own it proves nothing.
//!
//! ## `repair --out` had an unspecified meaning, and it is RULED
//!
//! `ms repair` does not emit a bare artifact: it prints `# Repair report`, then
//! `#   ms1 chunk 0: …`, then the corrected `ms1`. So `--out` there receives
//! **the artifact line alone** and the report stays on stdout — a payload
//! beginning `# Repair report` is not an `ms1`, and the correction record is what
//! an operator needs to SEE before trusting a repaired card. **A `--out` that
//! wrote the whole stdout passes a mode-only gate and fails the byte-pin below**
//! (R0 round 0's I-5).
//!
//! ## And `--out` is already taken on this binary
//!
//! `ms gen-man --out <DIR>` is shipped, documented, exampled twice in `--help`,
//! driven by this repo's `man-release.yml` workflow and by `scripts/install.sh`
//! in **mnemonic-toolkit**. P2 does not rename it — that breaks a release
//! workflow and a script in another repository for a cosmetic gain, in a phase
//! whose row is funds-safety work. F-282 records the collision; the control here
//! records that the two meanings coexist.

#![cfg(unix)]

use assert_cmd::Command;
use mnemonic_io_lib::fd::mode_of;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;

const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
/// `MS1` with data-part position 0 flipped. Measured: `ms repair` corrects it,
/// exits **4**, and prints two `#` report lines then the corrected string.
const MS1_ONE_ERROR: &str = "ms1eentrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

fn ms() -> Command {
    Command::cargo_bin("ms").unwrap()
}

fn write_tmp(dir: &std::path::Path, name: &str, body: &str) -> std::path::PathBuf {
    let p = dir.join(name);
    let mut f = std::fs::File::create(&p).unwrap();
    f.write_all(body.as_bytes()).unwrap();
    p
}

fn mode_at(p: &std::path::Path) -> u32 {
    mode_of(&std::fs::metadata(p).unwrap()).expect("a regular file has a mode")
}

/// **THE GATE.** An existing world-readable target is TIGHTENED, not inherited.
#[test]
fn an_existing_0644_target_is_tightened_to_0600_and_holds_the_new_artifact() {
    let dir = tempfile::tempdir().unwrap();
    let seed = write_tmp(dir.path(), "seed.txt", PHRASE);

    let stale = dir.path().join("card.ms1");
    std::fs::write(&stale, b"stale contents that are longer than the new ones").unwrap();
    std::fs::set_permissions(&stale, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(
        mode_at(&stale),
        0o644,
        "the control: the target really is 0644 before the call"
    );

    let out = ms()
        .args(["encode", "--in", &seed.display().to_string()])
        .args(["--out", &stale.display().to_string()])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    assert_eq!(
        mode_at(&stale),
        0o600,
        "`0o600` binds on CREATE, so an implementation that only passes it to \
         OpenOptions leaves this at 0644 and reports success"
    );
    assert_eq!(
        std::fs::read_to_string(&stale).unwrap(),
        format!("{MS1}\n"),
        "the file must hold the NEW artifact -- tightening a file the tool failed \
         to write would pass a permissions-only assertion, and a shrinking \
         overwrite without truncate leaves the tail of the old one"
    );
}

/// A file `--out` creates fresh is owner-only too. Weaker than the assertion
/// above and kept because it is the common case.
#[test]
fn a_fresh_out_file_is_owner_only() {
    let dir = tempfile::tempdir().unwrap();
    let seed = write_tmp(dir.path(), "seed.txt", PHRASE);
    let fresh = dir.path().join("fresh.ms1");
    ms().args(["encode", "--in", &seed.display().to_string()])
        .args(["--out", &fresh.display().to_string()])
        .assert()
        .success();
    assert_eq!(mode_at(&fresh), 0o600);
}

/// `ms split --out` writes the N shares, one per line, owner-only — and any K
/// of them still recombine, so what landed is the artifact and not a report.
#[test]
fn split_out_writes_recombinable_shares_owner_only() {
    let dir = tempfile::tempdir().unwrap();
    let seed = write_tmp(dir.path(), "seed.txt", PHRASE);
    let shares = dir.path().join("shares.txt");
    ms().args(["split", "--in", &seed.display().to_string()])
        .args(["-k", "2", "-n", "3"])
        .args(["--out", &shares.display().to_string()])
        .assert()
        .success();
    assert_eq!(mode_at(&shares), 0o600);
    let body = std::fs::read_to_string(&shares).unwrap();
    assert_eq!(
        body.lines().count(),
        3,
        "three shares, one per line: {body}"
    );

    let two = dir.path().join("two.txt");
    std::fs::write(
        &two,
        body.lines().take(2).collect::<Vec<_>>().join("\n") + "\n",
    )
    .unwrap();
    ms().args(["combine", "--in", &two.display().to_string()])
        .assert()
        .success()
        .stdout(predicates::str::contains(PHRASE));
}

/// **`repair --out` is RULED: the artifact line ALONE, byte-pinned.**
///
/// It is also the one verb that writes `--out` while exiting non-zero — 4,
/// VERIFY-ME — and the file is written all the same, because a correction the
/// operator must confirm is still the artifact they asked for.
#[test]
fn repair_out_holds_the_corrected_ms1_alone_while_stdout_keeps_the_report() {
    let dir = tempfile::tempdir().unwrap();
    let broken = write_tmp(dir.path(), "broken.ms1", MS1_ONE_ERROR);
    let fixed = dir.path().join("fixed.ms1");

    let out = ms()
        .args(["repair", "--in", &broken.display().to_string()])
        .args(["--out", &fixed.display().to_string()])
        .output()
        .unwrap();

    assert_eq!(
        out.status.code(),
        Some(4),
        "a correction is a VERIFY-ME candidate and the exit code says so: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(&fixed).unwrap(),
        format!("{MS1}\n"),
        "BYTE-PINNED. A --out that wrote the whole stdout passes a mode-only gate \
         and fails here: a payload beginning `# Repair report` is not an ms1"
    );
    assert_eq!(mode_at(&fixed), 0o600);

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("# Repair report"),
        "the report stays on the stream the operator is READING: {stdout}"
    );
    assert!(
        stdout.contains("# "),
        "both comment lines stay on stdout: {stdout}"
    );
}

/// **The collision control.** `--out` means a DIRECTORY on `gen-man` and a FILE
/// on the other three, and after P2 both meanings coexist on one binary.
#[test]
fn gen_man_out_still_means_a_directory() {
    let dir = tempfile::tempdir().unwrap();
    let man = dir.path().join("man");
    ms().args(["gen-man", "--out", &man.display().to_string()])
        .assert()
        .success();
    assert!(man.is_dir(), "gen-man --out must still create a DIRECTORY");
    assert!(
        man.join("ms.1").exists(),
        "and fill it with man pages: {:?}",
        std::fs::read_dir(&man).unwrap().count()
    );
}

/// `--out` OVERWRITES, per §6b's explicit ruling — no refusal, no `.1` suffix.
/// A shrinking overwrite leaves no tail of the previous file.
#[test]
fn out_overwrites_and_leaves_no_stale_tail() {
    let dir = tempfile::tempdir().unwrap();
    let seed = write_tmp(dir.path(), "seed.txt", PHRASE);
    let target = dir.path().join("card.ms1");
    std::fs::write(&target, "x".repeat(4096)).unwrap();
    ms().args(["encode", "--in", &seed.display().to_string()])
        .args(["--out", &target.display().to_string()])
        .assert()
        .success();
    assert_eq!(
        std::fs::read_to_string(&target).unwrap(),
        format!("{MS1}\n")
    );
}

/// The three verbs that gained `--out` document it; the report verbs did not
/// gain it (§6a rules `decode`, `verify` and `inspect` report verbs, and
/// `combine`/`derive` emit labelled reports rather than a canonical artifact).
/// F-285 records that `ms decode` and `ms combine` still write a recovered seed
/// to an unprotected stdout.
#[test]
fn out_is_on_exactly_the_three_artifact_verbs() {
    for verb in ["encode", "split", "repair"] {
        let help = ms().args([verb, "--help"]).output().unwrap();
        assert!(
            String::from_utf8_lossy(&help.stdout).contains("--out"),
            "{verb} --help must document --out"
        );
    }
    for verb in ["decode", "verify", "inspect", "combine", "derive"] {
        let out = ms()
            .args([verb, "--out", "/tmp/ms-p2-should-not-exist"])
            .write_stdin(MS1.to_string())
            .output()
            .unwrap();
        assert_eq!(
            out.status.code(),
            Some(64),
            "{verb} must NOT have gained --out in P2"
        );
        assert!(!std::path::Path::new("/tmp/ms-p2-should-not-exist").exists());
    }
}
