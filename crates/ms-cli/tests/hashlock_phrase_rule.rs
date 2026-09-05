//! The phrase rule (SPEC_ms_hashlock §4.3) driven through the BINARY on both
//! channels, and the byte-exact rows no codec vector can see (correctness
//! I-6.1): the mutation is swapping in `read_phrase_input`/`read_input` on
//! either channel.

use assert_cmd::Command;

fn ms() -> Command {
    Command::cargo_bin("ms").unwrap()
}

fn record_via_stdin(phrase: &[u8], method: &str) -> (Option<i32>, String, String) {
    let out = ms()
        .args([
            "hashlock",
            "--hashlock-phrase-stdin",
            "--method",
            method,
            "--no-engraving-card",
        ])
        .write_stdin(phrase.to_vec())
        .output()
        .unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

fn record_via_argv(phrase: &str, method: &str) -> (Option<i32>, String, String) {
    let out = ms()
        .args([
            "hashlock",
            "--allow-argv-secret",
            "--hashlock-phrase",
            phrase,
            "--method",
            method,
            "--no-engraving-card",
        ])
        .write_stdin("")
        .output()
        .unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

/// Byte-exact through BOTH channels, equal to the codec's own answer.
#[test]
fn byte_exact_rows_on_both_channels() {
    for phrase in [
        "  a  b ",
        "a-b,c",
        "correct-horse,battery staple",
        "Correct Horse Battery Staple",
    ] {
        let expect = {
            let x = ms_codec::hashlock::preimage_sha256(phrase.as_bytes());
            let h = ms_codec::hashlock::digest(&x);
            format!("hash:{}", hex::encode(h))
        };
        let (code, so, se) = record_via_stdin(phrase.as_bytes(), "sha256");
        assert_eq!(code, Some(0), "stdin {phrase:?}: {se}");
        assert_eq!(
            so.trim(),
            expect,
            "stdin channel changed the bytes of {phrase:?}"
        );
        let (code, so, se) = record_via_argv(phrase, "sha256");
        assert_eq!(code, Some(0), "argv {phrase:?}: {se}");
        assert_eq!(
            so.trim(),
            expect,
            "argv channel changed the bytes of {phrase:?}"
        );
    }
}

#[test]
fn stdin_strips_exactly_one_newline() {
    let a = record_via_stdin(b"abc\n", "sha256").1;
    let b = record_via_stdin(b"abc", "sha256").1;
    let c = record_via_stdin(b"abc\n\n", "sha256").1;
    assert_eq!(a, b, "one trailing LF is stripped");
    assert_ne!(b, c, "two trailing LFs keep one");
    let d = record_via_stdin(b"abc\r\n", "sha256").1;
    assert_eq!(a, d, "CRLF is one newline");
}

#[test]
fn refusals_in_four_spellings_on_both_channels_name_the_ms1_route() {
    let plate = String::from_utf8(
        ms().args(["hashlock", "--hex", "-", "--json"])
            .write_stdin("c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    let plate: String = serde_json::from_str::<serde_json::Value>(&plate).unwrap()["preimage_ms1"]
        .as_str()
        .unwrap()
        .to_string();
    let grouped5: String = plate
        .as_bytes()
        .chunks(5)
        .map(|c| std::str::from_utf8(c).unwrap())
        .collect::<Vec<_>>()
        .join(" ");
    let grouped2: String = plate
        .as_bytes()
        .chunks(2)
        .map(|c| std::str::from_utf8(c).unwrap())
        .collect::<Vec<_>>()
        .join(" ");
    for (name, s) in [
        ("lowercase", plate.clone()),
        ("UPPERCASE", plate.to_ascii_uppercase()),
        ("grouped", grouped5),
        ("padded", format!("  {plate}  ")),
        (
            "grouped-by-2 (112 chars: the shape test precedes the cap)",
            grouped2,
        ),
    ] {
        let (code, _, se) = record_via_stdin(s.as_bytes(), "sha256");
        assert_eq!(code, Some(1), "stdin {name}: {se}");
        assert!(
            se.contains("--in"),
            "stdin {name} must name the ms1 route:\n{se}"
        );
        assert!(
            !se.contains("100 characters") || !name.starts_with("grouped-by-2"),
            "cap fired before the shape test:\n{se}"
        );
        // The argv channel: the guard's shape layer catches a plate string
        // FIRST (it is ms1 material on argv) and names --in itself.
        let out = ms()
            .args(["hashlock", "--hashlock-phrase", &s])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(1), "argv {name}");
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("--in"),
            "argv {name} must name the ms1 route"
        );
    }
}

#[test]
fn hex64_either_case_is_redirected_to_hex_on_stdin_and_short_hex_is_accepted() {
    let lower = "c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016";
    for s in [lower.to_string(), lower.to_ascii_uppercase()] {
        let (code, _, se) = record_via_stdin(s.as_bytes(), "sha256");
        assert_eq!(code, Some(1), "{se}");
        assert!(
            se.contains("--hex") && se.contains("64 hex characters"),
            "{se}"
        );
    }
    let (code, _, se) = record_via_stdin(b"beef", "sha256");
    assert_eq!(code, Some(0), "{se}");
}

/// The 64-hex guard is EXACTLY 64: a longer all-hex phrase and a 64-character
/// phrase with one non-hex character are both accepted (R0 r0 tests I-4).
#[test]
fn hex_looking_phrases_of_other_lengths_are_accepted() {
    let eighty = "deadbeef".repeat(10);
    assert_eq!(record_via_stdin(eighty.as_bytes(), "sha256").0, Some(0));
    let mut sixty_four =
        "c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016".to_string();
    sixty_four.replace_range(63..64, "z");
    assert_eq!(record_via_stdin(sixty_four.as_bytes(), "sha256").0, Some(0));
}

#[test]
fn printable_ascii_boundary_and_cap() {
    assert_eq!(record_via_stdin(b" ~", "sha256").0, Some(0));
    for bad in [
        b"a\tb".to_vec(),
        b"a\x7f".to_vec(),
        vec![0xff],
        "caf\u{e9}".as_bytes().to_vec(),
    ] {
        let (code, _, se) = record_via_stdin(&bad, "sha256");
        assert_eq!(code, Some(1), "{bad:?}: {se}");
        assert!(
            se.contains("printable ASCII"),
            "the rule must be named:\n{se}"
        );
    }
    assert_eq!(
        record_via_stdin("a".repeat(100).as_bytes(), "sha256").0,
        Some(0)
    );
    let (code, _, se) = record_via_stdin("a".repeat(101).as_bytes(), "sha256");
    assert_eq!(code, Some(1));
    assert!(se.contains("100"), "{se}");
    let (code, _, se) = record_via_stdin(b"", "sha256");
    assert_eq!(code, Some(1));
    assert!(se.contains("empty"), "{se}");
}

/// The 100/101 lockstep rows (spec §8), driven with the CORPUS's own phrases
/// rather than an `"a"`-repeat, because these are the rows H2's device side is
/// pinned against. Both halves are asserted, as the name promises: 100 derives
/// through the CLI to the corpus's `hardened_h`, and 101 is REFUSED by the CLI
/// while the CODEC still derives the corpus's `hardened_x` for it -- the cap is
/// the CLI's rule, not the codec's, and a drift in either direction shows here.
/// (The 101 refusal is also reached by `printable_ascii_boundary_and_cap`; the
/// overlap is deliberate, this is the lockstep row.)
#[test]
fn lockstep_100_and_101() {
    const P100: &str = "hashlock phrase row: one hundred printable characters kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk";
    const P101: &str = "hashlock phrase row: one hundred printable characters kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk!";
    assert_eq!(P100.len(), 100);
    assert_eq!(P101.len(), 101);

    let (code, so, se) = record_via_stdin(P100.as_bytes(), "hardened");
    assert_eq!(code, Some(0), "100 characters must be accepted: {se}");
    assert_eq!(
        so.trim(),
        "hash:70a5395386c769019faa4996aa61510f7760a1b32d6980173ccc57b3e68b4525",
        "the corpus's 100-character hardened_h"
    );
    // ...and it is the codec's own answer, not a second constant.
    let x = ms_codec::hashlock::preimage_hardened(P100.as_bytes());
    assert_eq!(
        so.trim(),
        format!("hash:{}", hex::encode(ms_codec::hashlock::digest(&x)))
    );

    let (code, so, se) = record_via_stdin(P101.as_bytes(), "hardened");
    assert_eq!(code, Some(1), "101 characters must be refused: {se}");
    assert!(so.is_empty(), "a refused phrase produced a record:\n{so}");
    assert!(se.contains("101") && se.contains("100"), "{se}");
    // The CODEC still derives it -- the cap lives in the CLI. The corpus row
    // says so ("the codec derives it; the CLI refuses it") and this pins it.
    let x = ms_codec::hashlock::preimage_hardened(P101.as_bytes());
    assert_eq!(
        hex::encode(&x[..]),
        "abe28ff3905421a9f8caae476f3685555bb94a11c55e767d3bd979e9e46f6a57"
    );
}
