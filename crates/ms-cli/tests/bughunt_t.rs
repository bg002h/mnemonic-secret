//! Bug-hunt triage rows for the CLI, round 2 (three suspicions from the grading
//! report's "Not graded" section, `design/agent-reports/bughunt-grade-ms.md`).
//! Each test asserts the CORRECT behaviour, so it fails on unmodified code iff
//! the suspicion is real. Names are `bughunt_ms_tN_<slug>`. Material rides stdin
//! (`-`), never argv, as the argv guard requires.

use assert_cmd::Command;
use ms_codec::codex32::{Codex32String, Fe};
use std::process::Output;

/// Run `ms <args>` with `stdin` on the private channel.
fn ms(args: &[&str], stdin: &str) -> Output {
    Command::cargo_bin("ms")
        .unwrap()
        .args(args)
        .write_stdin(stdin.to_string())
        .output()
        .unwrap()
}

fn out_s(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}
fn err_s(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

// ---------------------------------------------------------------------------
// T2 `ms inspect` on a checksum-valid mnem string whose language code is 10 or
// more says "OK: would decode" while `ms decode` refuses it (codec
// `Payload::validate`: `language >= 10` -> MnemUnknownLanguage). Ruling
// (FIX_bughunt_ms.md T2): FAIL with reason `mnem-unknown-language`, exit 0.
// ---------------------------------------------------------------------------
#[test]
fn bughunt_ms_t2_inspect_unknown_language_says_ok() {
    // prefix 0x02 (mnem), language byte 10, 16 zero entropy bytes -> 51 chars.
    let mut data = vec![0x02u8, 10];
    data.extend_from_slice(&[0u8; 16]);
    let s = Codex32String::from_seed("ms", 0, "entr", Fe::S, &data)
        .unwrap()
        .to_string();
    assert_eq!(s.len(), 51, "precondition: a valid mnem string length");

    // Reference: decode refuses it.
    let d = ms(&["decode", "-"], &s);
    assert!(
        !d.status.success(),
        "precondition: decode must refuse language code 10: {}",
        out_s(&d)
    );
    assert!(
        err_s(&d).contains("unknown mnem wordlist-language code"),
        "precondition: the refusal names the language code: {}",
        err_s(&d)
    );

    let j = ms(&["inspect", "--json", "-"], &s);
    assert_eq!(j.status.code(), Some(0), "inspect --json: {}", err_s(&j));
    assert_eq!(out_s(&j), T2_JSON);

    let t = ms(&["inspect", "-"], &s);
    assert_eq!(t.status.code(), Some(0), "inspect: {}", err_s(&t));
    assert_eq!(out_s(&t), T2_TEXT);
}

const T2_JSON: &str = "{\"schema_version\":\"1\",\"report\":{\"hrp\":\"ms\",\"threshold\":0,\"tag\":\"entr\",\"share_index\":\"s\",\"prefix_byte\":2,\"payload_bytes_hex\":\"0a00000000000000000000000000000000\",\"checksum_valid\":true,\"kind\":\"mnem\",\"language\":\"unknown\"},\"would_decode\":false,\"failure_reasons\":[\"mnem-unknown-language\"]}\n";
const T2_TEXT: &str = "FAIL: would NOT decode v0.1\n    reason: mnem-unknown-language (unknown mnem wordlist-language code; known codes are 0-9)\n\nhrp: ms\nthreshold: 0\ntag: entr\nshare_index: s\nprefix_byte: 0x02\npayload_bytes: 0a00000000000000000000000000000000\nchecksum_valid: true\nkind: mnem\nlanguage: unknown\n";

// ---------------------------------------------------------------------------
// T3 `ms combine` on a checksum-valid 2-of-2 share set whose HRP is `xx`, not
// `ms`. `ms decode` refuses a single string with a non-`ms` HRP (WrongHrp,
// exit 2); `combine` has no HRP check and prints the recovered secret. Correct:
// combine refuses (non-zero exit). The exact code and text are an open product
// question and are NOT asserted.
// ---------------------------------------------------------------------------

/// A 2-of-2 share set under `hrp`: the secret-at-S carries `data`; the
/// defining share (index `a`) is fixed filler; the second share (index `c`) is
/// interpolated. Mirrors `encode_shares`' construction with a chosen HRP.
fn two_of_two(hrp: &str, data: &[u8]) -> Vec<String> {
    let secret = Codex32String::from_seed(hrp, 2, "zzzz", Fe::S, data).unwrap();
    let idx_a = Fe::from_char('a').unwrap();
    let idx_c = Fe::from_char('c').unwrap();
    let share_a =
        Codex32String::from_seed(hrp, 2, "zzzz", idx_a, &vec![0x5Au8; data.len()]).unwrap();
    let share_a_str = share_a.to_string();
    let share_c = Codex32String::interpolate_at(&[secret, share_a], idx_c).unwrap();
    vec![share_a_str, share_c.to_string()]
}

#[test]
#[ignore = "deferred-minor: ms-combine-accepts-non-ms-hrp"]
fn bughunt_ms_t3_combine_accepts_non_ms_hrp() {
    // prefix 0x00 (entr) + 16 entropy bytes.
    let mut data = vec![0x00u8];
    data.extend_from_slice(&[0x42u8; 16]);

    // Control: the same construction under HRP `ms` combines.
    let ok = two_of_two("ms", &data);
    let c = ms(&["combine", "--to", "entropy", "-"], &ok.join("\n"));
    assert!(
        c.status.success(),
        "precondition: the HRP `ms` set must combine: {}",
        err_s(&c)
    );
    assert_eq!(
        out_s(&c).trim(),
        "42".repeat(16),
        "precondition: recovered entropy"
    );

    // Reference: decode refuses a single string with a non-`ms` HRP.
    let single = Codex32String::from_seed("xx", 0, "entr", Fe::S, &data)
        .unwrap()
        .to_string();
    let d = ms(&["decode", "-"], &single);
    assert!(
        !d.status.success(),
        "precondition: decode must refuse HRP `xx`: {}",
        out_s(&d)
    );

    let bad = two_of_two("xx", &data);
    assert!(bad[0].starts_with("xx1") && bad[1].starts_with("xx1"));
    let o = ms(&["combine", "--to", "entropy", "-"], &bad.join("\n"));
    assert!(
        !o.status.success(),
        "`ms combine` accepted a share set with HRP `xx` (exit 0) and printed: {:?}",
        out_s(&o)
    );
}
