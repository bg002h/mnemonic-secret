//! `ms split --out FILE` refuses an existing FILE (owner ruling 2026-11-13):
//! each run draws a new share set, so an overwritten set cannot be re-made.
//! Modelled on `hashlock --random --out` (out.rs `write_artifact_create_new`).

#![cfg(unix)]

use assert_cmd::Command;

const HEX: &str = "00000000000000000000000000000000\n";

fn ms() -> Command {
    Command::cargo_bin("ms").unwrap()
}

fn refusal(path: &std::path::Path) -> String {
    format!(
        "error: --out {} already exists; a new share set will not overwrite it (choose another file, or move the old one first)\n",
        path.display()
    )
}

fn split_to(path: &std::path::Path, extra: &[&str]) -> std::process::Output {
    ms().args(["split", "--hex", "-", "-k", "2", "-n", "3", "--out"])
        .arg(path)
        .args(extra)
        .write_stdin(HEX)
        .output()
        .unwrap()
}

fn assert_refused(out: &std::process::Output, path: &std::path::Path) {
    assert_eq!(
        out.status.code(),
        Some(64),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stderr), refusal(path));
    assert!(out.stdout.is_empty(), "no share may reach stdout");
}

#[test]
fn an_existing_share_file_is_refused_and_left_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("shares.txt");
    assert!(split_to(&p, &[]).status.success());
    let monday = std::fs::read(&p).unwrap();
    let out = split_to(&p, &[]);
    assert_refused(&out, &p);
    assert_eq!(
        std::fs::read(&p).unwrap(),
        monday,
        "the first set must survive"
    );
}

#[test]
fn an_empty_existing_file_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("empty.txt");
    std::fs::write(&p, "").unwrap();
    assert_refused(&split_to(&p, &[]), &p);
    assert_eq!(std::fs::read(&p).unwrap(), b"");
}

#[test]
fn a_directory_is_refused_as_existing() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("d");
    std::fs::create_dir(&p).unwrap();
    assert_refused(&split_to(&p, &[]), &p);
}

#[test]
fn a_dangling_symlink_is_refused_and_its_target_not_created() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("link");
    let target = dir.path().join("nowhere");
    std::os::unix::fs::symlink(&target, &p).unwrap();
    assert_refused(&split_to(&p, &[]), &p);
    assert!(!target.exists(), "the link's target must not be created");
}

#[test]
fn a_symlink_to_a_file_is_refused_and_the_file_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("target.txt");
    std::fs::write(&target, "keep\n").unwrap();
    let p = dir.path().join("link");
    std::os::unix::fs::symlink(&target, &p).unwrap();
    assert_refused(&split_to(&p, &[]), &p);
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "keep\n");
}

#[test]
fn dev_null_is_refused_as_existing() {
    let p = std::path::Path::new("/dev/null");
    assert_refused(&split_to(p, &[]), p);
}

/// The check runs before the secret is read: a missing --in is not reached.
#[test]
fn the_refusal_comes_before_the_secret_is_read() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("shares.txt");
    std::fs::write(&p, "old\n").unwrap();
    let missing = dir.path().join("no-such-phrase.txt");
    let out = ms()
        .args(["split", "--in"])
        .arg(&missing)
        .args(["-k", "2", "-n", "3", "--out"])
        .arg(&p)
        .output()
        .unwrap();
    assert_refused(&out, &p);
}

/// Under --json the refusal is the error envelope, exit 64, and no share.
#[test]
fn json_mode_refusal_is_the_usage_envelope() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("shares.txt");
    std::fs::write(&p, "old\n").unwrap();
    let out = split_to(&p, &["--json"]);
    assert_eq!(out.status.code(), Some(64));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["error"]["kind"], "Usage");
    assert_eq!(v["error"]["exit_code"], 64);
    assert_eq!(
        format!("error: {}\n", v["error"]["message"].as_str().unwrap()),
        refusal(&p)
    );
    assert!(v.get("shares").is_none());
    assert_eq!(std::fs::read_to_string(&p).unwrap(), "old\n");
}

/// Pins that pass before and after: a missing parent is still a write
/// failure (exit 1), and hashlock's refusal text is byte-unchanged.
#[test]
fn a_missing_parent_directory_is_still_exit_1() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("nodir").join("x.txt");
    let out = split_to(&p, &[]);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr)
        .starts_with(&format!("error: failed to write --out {}: ", p.display())));
}

#[test]
fn hashlock_random_refusal_text_is_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("preimage.txt");
    std::fs::write(&p, "old\n").unwrap();
    let out = ms()
        .args(["hashlock", "--random", "--no-engraving-card", "--out"])
        .arg(&p)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(64));
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        format!(
            "error: --out {} already exists; a --random preimage will not overwrite it (choose another file, or move the old one first)\n",
            p.display()
        )
    );
}

/// A dangling symlink is refused before the secret is read, not only at the
/// O_EXCL create (`symlink_metadata`, not `metadata`).
#[test]
fn a_dangling_symlink_is_refused_before_the_secret_is_read() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("link");
    std::os::unix::fs::symlink(dir.path().join("nowhere"), &p).unwrap();
    let missing = dir.path().join("no-such-phrase.txt");
    let out = ms()
        .args(["split", "--in"])
        .arg(&missing)
        .args(["-k", "2", "-n", "3", "--out"])
        .arg(&p)
        .output()
        .unwrap();
    assert_refused(&out, &p);
}
