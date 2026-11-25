//! Bug hunt 2026-11-25, ms#1 — a K-of-N share with one wrong character had no
//! repair path in ms-cli: `ms combine` failed the whole set without naming
//! the share, and `ms repair` sent the holder back to `ms combine`.
//!
//! Now `ms repair` corrects a damaged share and emits it as an UNVERIFIED
//! candidate (exit 4, `verdict: "candidate"`), never as a verified recovery,
//! and `ms combine` names the share that fails its checksum and points at
//! `ms repair`. A share that is already valid is refused by `ms repair` as
//! before (exit 2, "use `ms combine`").

use assert_cmd::Command;

const E: &str = "000102030405060708090a0b0c0d0e0f";

fn ms(args: &[&str], stdin: &str) -> std::process::Output {
    Command::cargo_bin("ms")
        .unwrap()
        .args(args)
        .write_stdin(stdin.to_string())
        .output()
        .unwrap()
}

fn shares(k: &str, n: &str) -> Vec<String> {
    let o = ms(&["split", "--hex", "-", "-k", k, "-n", n, "--json"], E);
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    v["shares"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

/// Change the character at string offset `at` to another alphabet symbol.
fn damage(s: &str, at: usize) -> String {
    let mut c: Vec<char> = s.chars().collect();
    c[at] = if c[at] == 'q' { 'p' } else { 'q' };
    c.into_iter().collect()
}

fn write(dir: &tempfile::TempDir, name: &str, body: &str) -> String {
    let p = dir.path().join(name);
    std::fs::write(&p, body).unwrap();
    p.to_str().unwrap().to_string()
}

#[test]
fn repair_corrects_a_damaged_share_as_an_unverified_candidate() {
    let s = shares("2", "2");
    let bad = damage(&s[0], 20);
    let dir = tempfile::tempdir().unwrap();
    let f = write(&dir, "one.txt", &format!("{bad}\n"));
    let o = ms(&["repair", "--in", &f], "");
    assert_eq!(o.status.code(), Some(4), "candidate, not recovered: {o:?}");
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(out.starts_with("# Repair report\n"), "{out}");
    assert!(out.contains("1 correction at position 17"), "{out}");
    assert_eq!(out.lines().last().unwrap(), s[0], "{out}");
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("correction UNVERIFIED"), "{err}");
    assert!(err.contains("K-of-N share"), "{err}");
    assert!(err.contains("ms combine"), "{err}");
    assert!(
        !err.contains("use `ms combine` to recombine"),
        "no loop: {err}"
    );

    // The candidate recombines with the other share to the original secret.
    let o = ms(
        &["combine", "-", "--to", "entropy"],
        &format!("{}\n{}\n", s[0], s[1]),
    );
    assert!(String::from_utf8_lossy(&o.stdout).contains(E));
}

#[test]
fn repair_json_says_candidate() {
    let s = shares("2", "3");
    let bad = damage(&s[2], 30);
    let dir = tempfile::tempdir().unwrap();
    let f = write(&dir, "one.txt", &format!("{bad}\n"));
    let o = ms(&["repair", "--in", &f, "--json"], "");
    assert_eq!(o.status.code(), Some(4), "{o:?}");
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["kind"], "ms1");
    assert_eq!(v["verdict"], "candidate");
    assert_eq!(v["corrected_chunks"][0], s[2].as_str());
    assert_eq!(v["repairs"][0]["original_chunk"], bad.as_str());
    assert_eq!(v["repairs"][0]["corrected_positions"][0]["position"], 27);
}

#[test]
fn repair_out_receives_the_corrected_share() {
    let s = shares("2", "2");
    let bad = damage(&s[1], 12);
    let dir = tempfile::tempdir().unwrap();
    let f = write(&dir, "one.txt", &format!("{bad}\n"));
    let out = dir.path().join("fixed.txt");
    let o = ms(&["repair", "--in", &f, "--out", out.to_str().unwrap()], "");
    assert_eq!(o.status.code(), Some(4), "{o:?}");
    assert_eq!(
        std::fs::read_to_string(&out).unwrap(),
        format!("{}\n", s[1])
    );
}

/// A valid share is refused exactly as before: there is nothing to repair.
#[test]
fn repair_of_a_valid_share_is_unchanged() {
    let s = shares("2", "2");
    let dir = tempfile::tempdir().unwrap();
    let f = write(&dir, "one.txt", &format!("{}\n", s[0]));
    let o = ms(&["repair", "--in", &f], "");
    assert_eq!(o.status.code(), Some(2), "{o:?}");
    assert!(o.stdout.is_empty());
    assert!(String::from_utf8_lossy(&o.stderr).contains("use `ms combine` to recombine K shares"));
}

/// Too many errors in a share: refused (exit 2), nothing printed on stdout.
#[test]
fn repair_of_a_share_beyond_correction_is_refused() {
    let s = shares("2", "2");
    let mut bad = s[0].clone();
    for at in 10..20 {
        bad = damage(&bad, at);
    }
    let dir = tempfile::tempdir().unwrap();
    let f = write(&dir, "one.txt", &format!("{bad}\n"));
    let o = ms(&["repair", "--in", &f], "");
    assert_ne!(o.status.code(), Some(0), "{o:?}");
    assert_ne!(o.status.code(), Some(4), "{o:?}");
    assert!(o.stdout.is_empty(), "{o:?}");
}

#[test]
fn combine_names_the_damaged_share_and_points_at_repair() {
    let s = shares("2", "3");
    let bad = damage(&s[1], 20);
    let o = ms(&["combine", "-"], &format!("{}\n{}\n", s[0], bad));
    assert_eq!(o.status.code(), Some(1), "error kind unchanged: {o:?}");
    assert!(o.stdout.is_empty());
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("share 2 of 2"), "{err}");
    assert!(!err.contains("share 1 of 2"), "{err}");
    assert!(err.contains("ms repair"), "{err}");
    assert!(!err.contains(&bad), "never echoes the share: {err}");
}
