//! F-589: `ms encode --out FILE` warned "stdout carries private key material"
//! when STDOUT WAS EMPTY — measured at 0 bytes. A warning that cries wolf on
//! the safe path erodes the identical warning on the unsafe one, which is the
//! only reason the line exists.
//!
//! THE CONDITION IS NOT `--out`. All three modes were measured before the fix
//! was written:
//!
//! | invocation            | stdout                              | material |
//! |-----------------------|-------------------------------------|----------|
//! | `--out FILE`, text    | 0 bytes                             | no       |
//! | no `--out`, text      | 76 bytes (the ms1)                  | yes      |
//! | `--out FILE --json`   | 225 bytes, with `entropy_hex`+`ms1` | yes      |
//!
//! Keying on `--out` alone would have suppressed a TRUE warning on the `--json`
//! path — closing a small defect by opening a larger one.
//!
//! The advisory TEXT is untouched: it is byte-identical to mnemonic-toolkit's
//! and a parity test holds it that way. Only the decision to emit moved.

use assert_cmd::Command;
use std::fs;

const PHRASE: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
const WARN: &str = "stdout carries private key material";

fn ms() -> Command {
    Command::cargo_bin("ms").expect("ms binary")
}

#[test]
fn the_stdout_advisory_fires_only_when_stdout_carries_material() {
    let dir = tempfile::tempdir().unwrap();
    let seed = dir.path().join("seed.txt");
    fs::write(&seed, PHRASE).unwrap();

    // 1. --out FILE, text: stdout is empty, so the warning is FALSE.
    let out = ms()
        .args(["encode", "--in", &seed.display().to_string()])
        .args(["--out", &dir.path().join("a.ms1").display().to_string()])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(
        out.stdout.is_empty(),
        "this case assumes an empty stdout; the fixture drifted: {:?}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        !String::from_utf8_lossy(&out.stderr).contains(WARN),
        "warned that stdout carries key material when stdout is EMPTY:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );

    // 2. No --out, text: the ms1 goes to stdout, so the warning is TRUE.
    let bare = ms()
        .args(["encode", "--in", &seed.display().to_string()])
        .output()
        .unwrap();
    assert!(bare.status.success());
    assert!(!bare.stdout.is_empty(), "the ms1 should be on stdout");
    assert!(
        String::from_utf8_lossy(&bare.stderr).contains(WARN),
        "the warning was lost on the path where it is TRUE:\n{}",
        String::from_utf8_lossy(&bare.stderr)
    );

    // 3. --out FILE AND --json: stdout STILL carries entropy_hex, so the
    //    warning is TRUE. This is the case a naive `--out` check would have
    //    silenced, and it is the one that matters most.
    let j = ms()
        .args(["encode", "--in", &seed.display().to_string()])
        .args(["--out", &dir.path().join("b.ms1").display().to_string()])
        .arg("--json")
        .output()
        .unwrap();
    assert!(j.status.success());
    let sout = String::from_utf8_lossy(&j.stdout);
    assert!(
        sout.contains("entropy_hex"),
        "this case exists because --json puts entropy on stdout; it no longer does:\n{sout}"
    );
    assert!(
        String::from_utf8_lossy(&j.stderr).contains(WARN),
        "--json puts entropy_hex on stdout and the warning was suppressed:\n{}",
        String::from_utf8_lossy(&j.stderr)
    );
}
