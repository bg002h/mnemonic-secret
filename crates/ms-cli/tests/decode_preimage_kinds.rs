//! F-534: `ms decode` on a preimage answers for EVERY hash kind.
//!
//! It printed one `digest:` line, computed with sha256, under a label naming no
//! function. With four kinds in the world that is the silence
//! SPEC_hashlock_kinds §13.4 forbids of `ms hashlock` next door: an operator
//! verifying a PLATE gets an answer for sha256 with nothing saying a choice was
//! made for them.
//!
//! A preimage ms1 carries NO kind by construction, so there is no `--kind` to
//! add here and a lookup is the only honest answer -- the same reasoning that
//! gave `ms hashlock` its four-digest fallback.

use assert_cmd::Command;

fn ms() -> Command {
    Command::cargo_bin("ms").unwrap()
}

/// The corpus's derivation[0] preimage and its four digests
/// (hashlock-v0.8.json: hardened_x and hardened_h*). Transcribed from the
/// corpus, NOT recomputed here -- a value this test derives itself would agree
/// with the code under test by construction.
const X: &str = "c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016";
const SHA256: &str = "3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12";
const HASH256: &str = "98a20fc25dbcdf236fb0307e3f82cad47fca2e807f3ef82c31993549641cd488";
const RIPEMD160: &str = "09e7bb5051d89788fb4e4b374126721dbcc2946b";
const HASH160: &str = "b5b72c0e6896ff59dfa99e0d1052a9c0214cd0bd";

/// Build the preimage ms1 the way an operator would, through the CLI.
fn preimage_ms1(dir: &tempfile::TempDir) -> String {
    let out = dir.path().join("p.ms1");
    ms().args([
        "hashlock",
        "--hex",
        "-",
        "--out",
        out.to_str().unwrap(),
        "--no-engraving-card",
    ])
    .write_stdin(X)
    .assert()
    .success();
    std::fs::read_to_string(&out).unwrap().trim().to_string()
}

/// MUTATION: drop any kind from the loop -> its row fails.
/// MUTATION: compute every row with digest_sha256 -> three rows fail.
#[test]
fn decode_prints_the_preimages_digest_under_every_kind() {
    let dir = tempfile::tempdir().unwrap();
    let ms1 = preimage_ms1(&dir);
    let r = ms()
        .args(["decode", "-"])
        .write_stdin(ms1)
        .assert()
        .success();
    let out = String::from_utf8_lossy(&r.get_output().stdout).to_string();

    for (token, want) in [
        ("sha256", SHA256),
        ("hash256", HASH256),
        ("ripemd160", RIPEMD160),
        ("hash160", HASH160),
    ] {
        assert!(
            out.contains(want),
            "decode does not print the {token} digest {want}:\n{out}"
        );
        assert!(out.contains(token), "decode does not NAME {token}:\n{out}");
    }
    // AND IT SAYS WHY THERE ARE FOUR. Without this the operator is left to
    // guess which line is theirs, which is the same silence in a longer form.
    assert!(
        out.contains("carries no hash kind"),
        "decode lists four digests and does not say a preimage carries no kind:\n{out}"
    );
    // The bare `digest:` label is gone: it implied an answer.
    assert!(
        !out.contains("digest:    "),
        "the single sha256-only digest line is still there:\n{out}"
    );
}

/// The JSON object gains `digests_by_kind` and KEEPS `digest`.
///
/// Keeping it is deliberate: a consumer parsing this object predates the other
/// three kinds, and removing the key would break it for a reason it cannot see.
/// `digest_kind` says what that retained key means, so it stops being a bare
/// assertion.
#[test]
fn the_json_object_carries_every_kind_and_keeps_the_old_key() {
    let dir = tempfile::tempdir().unwrap();
    let ms1 = preimage_ms1(&dir);
    let r = ms()
        .args(["decode", "-", "--json"])
        .write_stdin(ms1)
        .assert()
        .success();
    let v: serde_json::Value =
        serde_json::from_slice(&r.get_output().stdout).expect("decode --json is one object");
    assert_eq!(v["digest"], SHA256, "the pre-existing key must not move");
    assert_eq!(v["digest_kind"], "sha256", "and it must say what it is");
    for (token, want) in [
        ("sha256", SHA256),
        ("hash256", HASH256),
        ("ripemd160", RIPEMD160),
        ("hash160", HASH160),
    ] {
        assert_eq!(
            v["digests_by_kind"][token], want,
            "digests_by_kind[{token}]"
        );
    }
}
