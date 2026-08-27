//! SPEC §2.1 edge-case table: clap arg-group violations exit 64 (usage error).
//!
//! Both --phrase + --hex supplied → usage error.
//! Neither supplied → usage error.

use assert_cmd::Command;

#[test]
fn encode_rejects_both_phrase_and_hex() {
    Command::cargo_bin("ms").unwrap()
        .args([
            "encode",
            "--phrase",
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
            "--hex",
            "00000000000000000000000000000000",
        ])
        .assert()
        .failure()
        .code(64);
}

#[test]
fn encode_rejects_neither_phrase_nor_hex() {
    Command::cargo_bin("ms")
        .unwrap()
        .arg("encode")
        .assert()
        .failure()
        .code(64);
}

// --- P2 row 2: the group gained a THIRD member, `--in` ----------------------
//
// Extended rather than rewritten: the two assertions above are unchanged and
// still pin the original pair. What is new is that `--in` is inside the same
// exclusion, so every PAIR of the three collides and the neither-supplied case
// still exits 64 while now offering `--in` in its usage line.

#[test]
fn encode_rejects_in_together_with_phrase_or_hex() {
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("seed.txt");
    std::fs::write(&f, "abandon abandon about").unwrap();
    let path = f.display().to_string();
    for other in [["--phrase", "-"], ["--hex", "-"]] {
        Command::cargo_bin("ms")
            .unwrap()
            .args(["encode", other[0], other[1], "--in", &path])
            .assert()
            .failure()
            .code(64);
    }
}

#[test]
fn the_neither_supplied_usage_line_now_offers_in() {
    let out = Command::cargo_bin("ms")
        .unwrap()
        .arg("encode")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(64));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("--in"),
        "an operator told only about --phrase and --hex reaches for argv; got:\n{err}"
    );
}
