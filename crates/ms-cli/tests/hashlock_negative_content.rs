//! Eleven refusals, and in none of them does the phrase or the preimage
//! appear on stdout, stderr or in the `--json` error envelope
//! (SPEC_ms_hashlock §11; Minor class by the 2026-08-27 ruling, recorded
//! because the brainstorm agreed the matrix). MUTATION: a refusal built with
//! `format!("... {phrase}")`.

use assert_cmd::Command;

const SECRET_PHRASE: &str = "zebra quantum lantern violet";
const SECRET_HEX: &str = "c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016";

fn ms() -> Command {
    Command::cargo_bin("ms").unwrap()
}

fn assert_silent(args: &[&str], stdin: &[u8], secrets: &[&str], label: &str) {
    for json in [false, true] {
        let mut a = args.to_vec();
        if json {
            a.push("--json");
        }
        let out = ms().args(&a).write_stdin(stdin.to_vec()).output().unwrap();
        assert!(!out.status.success(), "{label} (json={json}) must refuse");
        assert_ne!(out.status.code(), Some(101), "{label} panicked");
        let all = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        for s in secrets {
            assert!(
                !all.contains(s),
                "{label} (json={json}) echoed material:\n{all}"
            );
        }
    }
}

#[test]
fn eleven_refusals_never_echo() {
    let tab_phrase = format!("{SECRET_PHRASE}\t");
    let long_phrase = format!("{SECRET_PHRASE}{}", "x".repeat(100));
    let plate = format!("ms10hashsq{}", "q".repeat(65));
    let plate = plate.as_str();
    assert_silent(
        &["hashlock", "--hashlock-phrase-stdin"],
        b"",
        &[SECRET_PHRASE],
        "empty",
    );
    assert_silent(
        &["hashlock", "--hashlock-phrase-stdin"],
        "caf\u{e9} zebra quantum".as_bytes(),
        &["zebra quantum"],
        "non-ascii",
    );
    assert_silent(
        &["hashlock", "--hashlock-phrase-stdin"],
        tab_phrase.as_bytes(),
        &[SECRET_PHRASE],
        "control byte",
    );
    assert_silent(
        &["hashlock", "--hashlock-phrase-stdin"],
        long_phrase.as_bytes(),
        &[SECRET_PHRASE],
        "over 100",
    );
    assert_silent(
        &["hashlock", "--hashlock-phrase-stdin"],
        SECRET_HEX.as_bytes(),
        &[SECRET_HEX],
        "64-hex",
    );
    assert_silent(
        &["hashlock", "--hashlock-phrase-stdin"],
        plate.as_bytes(),
        &[plate],
        "ms1-shaped",
    );
    assert_silent(
        &["hashlock", "--hex", "-"],
        b"abcd",
        &["abcd"],
        "--hex wrong length",
    );
    assert_silent(
        &["hashlock", "-"],
        b"ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f",
        &["ms10entrsqqqqq"],
        "wrong ms1 kind",
    );
    assert_silent(&["hashlock"], b"", &[SECRET_PHRASE], "zero sources");
    assert_silent(
        &["hashlock", "--hashlock-phrase-stdin", "--hex", SECRET_HEX],
        SECRET_PHRASE.as_bytes(),
        &[SECRET_PHRASE, SECRET_HEX],
        "two sources",
    );
    assert_silent(
        &[
            "hashlock",
            "--hex",
            SECRET_HEX,
            "--method",
            "sha256",
            "--allow-argv-secret",
        ],
        b"",
        &[SECRET_HEX],
        "--method with a supplied X",
    );
}
