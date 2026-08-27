//! **P2 row 3 — `--in` frees stdin, and two shipped refusals become
//! satisfiable.**
//!
//! No new code: this file is the assertion that the two preceding entries
//! composed. `ms` ships two refusals that exist only because there is ONE
//! stdin —
//!
//! ```text
//! ms verify - --phrase -            -> rc 1, "cannot read both ms1 and --phrase from stdin"
//! ms derive - --passphrase-stdin    -> rc 1, "cannot read both the entropy source and --passphrase from stdin (one stdin per invocation)"
//! ```
//!
//! — and both are correct. What they cost is that the round-trip checks an
//! operator plainly wants could not be performed **privately at all**: the only
//! way to supply the second value was argv. So `--in` is not only a hardening
//! measure here; it is the first private way to do two things at once.
//!
//! **THE THIRD SHAPE IS THE ONE `--in` DOES NOT FREE, and it is asserted here
//! rather than left implied** (R0 round 0's I-3). §6d binds `--in` on `derive`
//! to the **ms1 positional**, and the positional is ms1-only — measured:
//! `ms derive - < <a file holding a BIP-39 phrase>` exits 1 with
//! `string length 82 not in v0.1 set [50, 56, 62, 69, 75]`. So an operator with
//! a paper seed phrase AND a passphrase — the recovery shape, not the card
//! shape — has no one-command private form after P2. A private route DOES
//! remain, and it is two commands; F-303 carries the one-command form.

use assert_cmd::Command;
use std::io::Write;

const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
const PASSPHRASE: &str = "correct horse battery staple";

fn ms() -> Command {
    Command::cargo_bin("ms").unwrap()
}

fn write_tmp(dir: &std::path::Path, name: &str, body: &str) -> std::path::PathBuf {
    let p = dir.join(name);
    let mut f = std::fs::File::create(&p).unwrap();
    f.write_all(body.as_bytes()).unwrap();
    p
}

/// `ms verify --in card.txt --phrase -` exits 0 and reports the round trip.
/// Without `--in` the same intent is the shipped refusal at rc 1.
#[test]
fn verify_can_read_a_card_and_a_phrase_in_one_invocation() {
    let dir = tempfile::tempdir().unwrap();
    let card = write_tmp(dir.path(), "card.ms1", MS1);

    let freed = ms()
        .args([
            "verify",
            "--in",
            &card.display().to_string(),
            "--phrase",
            "-",
        ])
        .write_stdin(PHRASE.to_string())
        .output()
        .unwrap();
    assert_eq!(
        freed.status.code(),
        Some(0),
        "stderr:\n{}",
        String::from_utf8_lossy(&freed.stderr)
    );
    assert!(
        String::from_utf8_lossy(&freed.stdout).contains("round-trip valid"),
        "it must REPORT the round trip, not merely exit 0"
    );

    // The shape this replaces: no --in, so both channels want stdin.
    let contended = ms()
        .args(["verify", "--phrase", "-"])
        .write_stdin(MS1.to_string())
        .output()
        .unwrap();
    assert_eq!(contended.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&contended.stderr)
        .contains("cannot read both ms1 and --phrase from stdin"));
}

/// `ms derive --in card.txt --passphrase-stdin` exits 0 and applies the
/// passphrase. Applying it is asserted by DIFFERENCE against the no-passphrase
/// run — a fingerprint that ignored the passphrase would still exit 0.
#[test]
fn derive_can_read_a_card_and_a_passphrase_in_one_invocation() {
    let dir = tempfile::tempdir().unwrap();
    let card = write_tmp(dir.path(), "card.ms1", MS1);
    let path = card.display().to_string();

    let with_pass = ms()
        .args(["derive", "--in", &path, "--passphrase-stdin"])
        .write_stdin(PASSPHRASE.to_string())
        .output()
        .unwrap();
    assert_eq!(
        with_pass.status.code(),
        Some(0),
        "stderr:\n{}",
        String::from_utf8_lossy(&with_pass.stderr)
    );

    let without = ms().args(["derive", "--in", &path]).output().unwrap();
    assert_eq!(without.status.code(), Some(0));
    assert_ne!(
        String::from_utf8_lossy(&with_pass.stdout),
        String::from_utf8_lossy(&without.stdout),
        "the passphrase must reach the derivation -- identical output means it was READ \
         and then dropped, which exits 0 either way"
    );
}

/// **The control that keeps the refusals alive.** `--in` routes around the
/// contention; it must not have removed the guard.
#[test]
fn both_channels_on_stdin_are_still_refused() {
    let both_ms1 = ms()
        .args(["verify", "-", "--phrase", "-"])
        .write_stdin(MS1.to_string())
        .output()
        .unwrap();
    assert_eq!(both_ms1.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&both_ms1.stderr)
        .contains("cannot read both ms1 and --phrase from stdin"));

    let both_derive = ms()
        .args(["derive", "-", "--passphrase-stdin"])
        .write_stdin(MS1.to_string())
        .output()
        .unwrap();
    assert_eq!(both_derive.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&both_derive.stderr).contains("one stdin per invocation"));
}

/// **THE THIRD SHAPE — a phrase plus a passphrase on `derive`.**
///
/// `--in` on `derive` reads an `ms1`, so this shape is NOT freed. What must
/// hold is that a private route exists and is EQUIVALENT, not merely runnable:
/// the two-command route derives the same master fingerprint the one-command
/// argv form does.
#[test]
fn the_phrase_plus_passphrase_shape_has_a_private_two_command_route() {
    let dir = tempfile::tempdir().unwrap();
    let seed = write_tmp(dir.path(), "seed.txt", PHRASE);
    let pass = write_tmp(dir.path(), "pass.txt", PASSPHRASE);
    let card = dir.path().join("card.ms1");

    // Command 1 -- the phrase never touches argv, and the card lands at 0600.
    //
    // Row 3 asserted this route in its `--out`-free form, because `--out` is the
    // private write's to build and row 3 builds no new code. The private write
    // has landed, so this is now the route the refusal itself prints.
    let enc = ms()
        .args(["encode", "--in", &seed.display().to_string()])
        .args(["--out", &card.display().to_string()])
        .output()
        .unwrap();
    assert_eq!(
        enc.status.code(),
        Some(0),
        "encode --in --out: {}",
        String::from_utf8_lossy(&enc.stderr)
    );

    // Command 2 -- the passphrase never touches argv either.
    let der = ms()
        .args(["derive", "--in", &card.display().to_string()])
        .arg("--passphrase-stdin")
        .write_stdin(std::fs::read_to_string(&pass).unwrap())
        .output()
        .unwrap();
    assert_eq!(
        der.status.code(),
        Some(0),
        "derive --in --passphrase-stdin: {}",
        String::from_utf8_lossy(&der.stderr)
    );

    // ...and it is the SAME wallet. The one-command argv form is the oracle;
    // after the override lands it is reached through `--allow-argv-secret`,
    // because the guard refuses argv material by then.
    let oracle = one_command_argv_oracle();
    assert_eq!(
        fingerprint_of(&String::from_utf8_lossy(&der.stdout)),
        fingerprint_of(&oracle),
        "the two-command route must derive the SAME master fingerprint, or it is a \
         different wallet and the advice is worse than none"
    );

    // And the shape that is NOT freed: `--in` on derive reads an ms1, so a
    // phrase file through it is a length error, not a phrase.
    let wrong_kind = ms()
        .args(["derive", "--in", &seed.display().to_string()])
        .output()
        .unwrap();
    assert_ne!(
        wrong_kind.status.code(),
        Some(0),
        "`--in` on derive is ms1-only; a phrase file must NOT quietly work, or the \
         two-command route was never needed and F-303 is mis-scoped"
    );
}

/// The one-command argv form's fingerprint, as the oracle.
///
/// **Measured, then pinned.** Before the guard landed,
/// `ms derive --phrase <PHRASE> --passphrase <PASSPHRASE>` exited 0 and printed
/// `master_fingerprint:  6090b661`; the same run through the two-command route
/// printed the same value. The guard now refuses that invocation, so the live
/// oracle is reinstated by the override work, which re-runs it under
/// `--allow-argv-secret` and asserts it still equals this constant.
const ONE_COMMAND_ARGV_FINGERPRINT: &str = "6090b661";

fn one_command_argv_oracle() -> String {
    format!("master_fingerprint:  {ONE_COMMAND_ARGV_FINGERPRINT}")
}

fn fingerprint_of(stdout: &str) -> String {
    let line = stdout
        .lines()
        .find(|l| l.starts_with("master_fingerprint:"))
        .unwrap_or_else(|| panic!("no master_fingerprint line in:\n{stdout}"));
    line.split_whitespace().nth(1).unwrap().to_string()
}
