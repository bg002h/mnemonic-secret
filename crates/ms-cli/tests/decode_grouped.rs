//! `ms decode` accepts a comma-grouped ms1 card (mstring-grouping P2).
//! Comma is the SPEC §3.2 separator neither ms-codec's decode nor the legacy
//! `strip_whitespace` removed — so this genuinely exercises the new
//! `strip_display_separators` intake.

use assert_cmd::Command;
use predicates::prelude::*;

const Z12: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fn comma5(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && i % 5 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

#[test]
fn decode_accepts_comma_grouped() {
    // Encode unbroken, then comma-group it and decode — must recover the entropy.
    let enc = Command::cargo_bin("ms")
        .unwrap()
        .args(["encode", "--phrase", Z12, "--group-size", "0"])
        .output()
        .unwrap();
    let ms1 = String::from_utf8(enc.stdout)
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .to_string();
    let grouped = comma5(&ms1);
    Command::cargo_bin("ms")
        .unwrap()
        .args(["decode", &grouped])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "entropy: 00000000000000000000000000000000",
        ));
}
