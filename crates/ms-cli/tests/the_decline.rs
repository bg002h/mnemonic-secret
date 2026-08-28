//! **P2 row 14 — the DECLINE, asserted.**
//!
//! `ms` takes **4 of the crate's 11** public items and declines 7. The four are
//! checked by `tests/crate_items_resolve.rs`; this file is the other half —
//! **a backstop that pins what `ms` keeps**, so a later phase cannot adopt a
//! declined item as tidying and cannot delete these as redundant.
//!
//! Two consumers now decline the same three modules for different reasons, and
//! that is the finding F-276 records: `mt` took 5 of 11, `ms` takes 4, and
//! neither took `exit`, `observation` or `records`.
//!
//! | declined | why, for `ms` |
//! | --- | --- |
//! | `exit::write_block` | its `Terminal` arm refuses unconditionally, and §6e's retraction FORBIDS `ms` that refusal by name — refusing a terminal directs the operator to `--out FILE`, which they must then read to hand-engrave, so *"a screen-only exposure becomes a screen exposure plus a disk artifact"* |
//! | `exit::WriteBlock` | goes with it: `Terminal(PayloadKind)` would be unconstructible and `WorldReadable(u32)` unreachable, because P2 builds no stdout mode gate |
//! | `channel::destination` | `ms` has nothing to MAP the non-`File` arms onto. `mt` adopted it because it has a world-readable-stdout gate and a terminal policy; `ms` has neither, so `--out` needs only `Option::is_some` |
//! | `fd::stdout_mode` | exists to feed a stdout gate, and P2 builds none |
//! | `observation::PayloadKind` | `ms` already carries a **superset**: `OutputClass` has three variants and `ms` uses two. A watch-only account xpub is not `CarriesNoSecret`, documented as *"measured to hold nothing: a 65,536-byte fill image"* |
//! | `records::split_record_stream` | it strips nothing, and `read_shares` strips display separators per line — swapping it in loses the grouped-card re-ingest, which is the whole point of a share typed back off metal |
//! | `records::no_records_guard` | its message advises *"pass them on argv, with --in, or on stdin"*, and after P2 `ms` REFUSES argv. It would print advice this phase exists to make unfollowable |

use assert_cmd::Command;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;

const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

fn ms() -> Command {
    Command::cargo_bin("ms").unwrap()
}

fn seed_file(dir: &std::path::Path) -> String {
    let p = dir.join("seed.txt");
    let mut f = std::fs::File::create(&p).unwrap();
    f.write_all(PHRASE.as_bytes()).unwrap();
    p.display().to_string()
}

/// **`ms` keeps its PERMISSIVE TERMINAL.** `ms encode` to a real pty still exits
/// 0 and still prints the artifact. An adoption of `exit::write_block`, whose
/// `Destination::Terminal` arm refuses unconditionally, goes RED here.
///
/// `#[ignore]`d because a pty is not portable across `ms`'s test matrix
/// (macOS, aarch64-musl under QEMU) and `script(1)`'s arguments differ between
/// GNU and BSD; `rust.yml`'s `history-purge` job runs it with the shells.
#[test]
#[ignore = "needs a pty; run by rust.yml's history-purge job"]
fn a_terminal_destination_is_still_permitted() {
    let dir = tempfile::tempdir().unwrap();
    let seed = seed_file(dir.path());
    let log = dir.path().join("tty.log");
    let bin = assert_cmd::cargo::cargo_bin("ms");
    let st = std::process::Command::new("/usr/bin/script")
        .arg("-qec")
        .arg(format!("{} encode --in {seed}", bin.display()))
        .arg(log.display().to_string())
        .output()
        .expect("`script` (util-linux) gives ms a real pty");
    assert!(
        st.status.success(),
        "ms encode to a TERMINAL must still exit 0 -- §6e retracts the refusal for \
         `ms` by name, because it turns a screen-only exposure into a screen \
         exposure PLUS a disk artifact. stderr:\n{}",
        String::from_utf8_lossy(&st.stderr)
    );
    let seen = std::fs::read_to_string(&log).unwrap();
    assert!(
        seen.contains(MS1),
        "...and must still PRINT the artifact, not merely exit 0. tty log:\n{seen}"
    );
}

/// **`ms` has no stdout mode gate, and P2 gives it none.** Writing the artifact
/// to a world-readable file still exits 0. A gate smuggled in — refusal OR
/// warning — goes RED here.
///
/// This is the one decline that is a live funds-safety question rather than a
/// settled one, and it is filed rather than argued: F-281 asks whether `ms`
/// should gate a world-readable stdout at all, with F-275's `mt decode`
/// precedent (warn-and-proceed, not refusal) attached.
#[test]
fn a_world_readable_stdout_is_still_permitted() {
    let dir = tempfile::tempdir().unwrap();
    let seed = seed_file(dir.path());
    let target = dir.path().join("backup.txt");
    let f = std::fs::File::create(&target).unwrap();
    std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o644)).unwrap();

    // std::process::Command, because the point is a REAL 0644 file on fd 1 --
    // `assert_cmd` pipes stdout and a pipe is 0600, which would make this test
    // pass against a tool that did have a mode gate.
    let out = std::process::Command::new(assert_cmd::cargo::cargo_bin("ms"))
        .args(["encode", "--in", &seed])
        .stdout(std::process::Stdio::from(f))
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "a 0644 stdout must NOT be refused: P2 builds no stdout mode gate. \
         stderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        std::fs::read_to_string(&target).unwrap().contains(MS1),
        "and the artifact must have been written"
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        !err.to_lowercase().contains("world-readable") && !err.contains("0644"),
        "not even a WARNING: §7's P2 row enumerates this phase's content and lists \
         none. F-281 carries the question. Got:\n{err}"
    );
}

/// **`read_shares` was not displaced by `records::split_record_stream`.**
/// `split_record_stream` strips nothing; `read_shares` strips display
/// separators per line. Grouped shares fed to `ms combine -` must still
/// recombine — that is the whole point of a share an operator typed back off
/// metal.
#[test]
fn grouped_shares_still_re_ingest_through_combine() {
    let dir = tempfile::tempdir().unwrap();
    let seed = seed_file(dir.path());
    let out = ms()
        .args(["split", "--in", &seed, "-k", "2", "-n", "3"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let grouped: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .take(2)
        .map(str::to_string)
        .collect();
    assert!(
        grouped[0].contains(' '),
        "the control: `ms split`'s stdout is GROUPED by default and P2 does not \
         change that (§6a binds the stdout rule to `encode` alone; F-284). If this \
         ever fails, this test stopped exercising the stripping. Got: {:?}",
        grouped[0]
    );

    ms().args(["combine", "-"])
        .write_stdin(grouped.join("\n"))
        .assert()
        .success()
        .stdout(predicates::str::contains(PHRASE));
}

/// **`OutputClass` was not displaced by `observation::PayloadKind`.** All three
/// of `ms`'s classes are still reachable in the vocabulary, and the two it uses
/// still fire on the right verbs. The byte-parity test against
/// `mnemonic-toolkit` is `cli_output_class.rs::byte_parity_advisory_lines`,
/// which runs in the same suite; this asserts the BEHAVIOUR that pins it.
#[test]
fn the_three_class_output_vocabulary_is_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let seed = seed_file(dir.path());
    let card = dir.path().join("card.ms1");
    std::fs::write(&card, MS1).unwrap();

    let private = ms().args(["encode", "--in", &seed]).output().unwrap();
    assert!(String::from_utf8_lossy(&private.stderr)
        .contains("warning: stdout carries private key material"));

    let watch_only = ms()
        .args(["derive", "--in", &card.display().to_string()])
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&watch_only.stderr).contains("note: stdout is watch-only"),
        "`derive` emits the WATCH-ONLY line -- a class `PayloadKind` cannot \
         express, because a watch-only account xpub is not `CarriesNoSecret`"
    );
}

/// **`gen-man --out DIR` is `ms`'s own and stays.** Renaming it would break
/// this repo's `man-release.yml` and `scripts/install.sh` in
/// **mnemonic-toolkit**. F-282.
#[test]
fn gen_man_keeps_its_directory_valued_out() {
    let dir = tempfile::tempdir().unwrap();
    let man = dir.path().join("man");
    ms().args(["gen-man", "--out", &man.display().to_string()])
        .assert()
        .success();
    assert!(man.is_dir() && man.join("ms.1").exists());
}

/// **`ms`'s own empty-input behaviour is unchanged** — `no_records_guard` was
/// not adopted, and its message would advise a channel this phase refuses.
#[test]
fn empty_input_keeps_ms_own_message_and_never_advises_argv() {
    let out = ms()
        .args(["combine", "-"])
        .write_stdin("")
        .output()
        .unwrap();
    assert_ne!(out.status.code(), Some(0));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("expected at least one share"),
        "`ms`'s own wording, not the crate's: {err}"
    );
    assert!(
        !err.contains("pass them on argv"),
        "the crate's guard advises argv, which P2 REFUSES -- adopting it would \
         print advice this phase exists to make unfollowable: {err}"
    );
}
