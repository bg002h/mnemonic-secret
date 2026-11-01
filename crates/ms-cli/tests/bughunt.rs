//! Bug-hunt triage rows for the CLI (2026-10-31 hunt, report
//! `design/agent-reports/bughunt-2026-10-31-ms.md`). Each test asserts the
//! CORRECT behaviour, so it fails on unmodified code iff the reported bug is
//! real. Names are `bughunt_ms_NN_<slug>` (NN = the report's number). Material
//! rides stdin (`-`), never argv, as the argv guard requires.

use assert_cmd::Command;
use ms_codec::codex32::{Codex32String, Fe};
use ms_codec::{encode_shares, Payload, Tag, Threshold};
use serde_json::Value;
use std::process::Output;
use zeroize::Zeroizing;

/// The REAL `src/mlock.rs`, compiled into this test crate by path (ms-cli is a
/// bin-only crate and `mlock` is a private module, so an integration test
/// cannot otherwise reach it). Not a copy: edits to the file are what runs.
#[path = "../src/mlock.rs"]
#[allow(dead_code)]
mod mlock;

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
// #2 PinnedPageRange::drop munlocks unconditionally; mlock is not
// reference-counted, so dropping one pin unlocks a page another live pin
// still needs. (Secret-handling class: never C/I per the 2026-08-27 ruling.)
// ---------------------------------------------------------------------------

/// `Locked:` kB of the VMA containing `addr`, from /proc/self/smaps.
fn locked_kb_of_vma_containing(addr: usize) -> u64 {
    let smaps = std::fs::read_to_string("/proc/self/smaps").expect("read smaps");
    let mut in_vma = false;
    for line in smaps.lines() {
        let first = line.split_whitespace().next().unwrap_or("");
        if let Some((lo, hi)) = first.split_once('-') {
            if let (Ok(lo), Ok(hi)) = (usize::from_str_radix(lo, 16), usize::from_str_radix(hi, 16))
            {
                in_vma = lo <= addr && addr < hi;
                continue;
            }
        }
        if in_vma && line.starts_with("Locked:") {
            return line.split_whitespace().nth(1).unwrap().parse().unwrap();
        }
    }
    panic!("no VMA containing {addr:#x} in /proc/self/smaps");
}

#[test]
#[ignore = "deferred-minor: ms-mlock-unpin-not-refcounted"]
fn bughunt_ms_02_mlock_unpin_not_refcounted() {
    let page = mlock::page_size_for_test();
    // One private anonymous mapping of two pages: a page nothing else touches.
    let base = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            2 * page,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1,
            0,
        )
    };
    assert_ne!(base, libc::MAP_FAILED, "mmap");
    let buf: &[u8] = unsafe { std::slice::from_raw_parts(base as *const u8, page) };

    // Two distinct secrets on the SAME page.
    let a = &buf[0..32];
    let b = &buf[32..64];
    let pin_a = mlock::pin_pages_for(a);
    let pin_b = mlock::pin_pages_for(b);
    let both = locked_kb_of_vma_containing(base as usize);
    assert!(
        both > 0,
        "precondition: page is locked while both pins live (mlock must succeed: RLIMIT_MEMLOCK)"
    );

    drop(pin_a); // B's pin is still live
    let after = locked_kb_of_vma_containing(base as usize);

    drop(pin_b);
    unsafe { libc::munmap(base, 2 * page) };

    assert!(
        after > 0,
        "dropping pin A unlocked the page while pin B (same page) is still live: \
         Locked went {both} kB -> {after} kB"
    );
}

// ---------------------------------------------------------------------------
// #4 `ms combine --to ms1` on preimage shares is silently ignored.
// Ruling (FIX_bughunt_ms.md): `--to ms1` prints the `hash` single and
// `--to entropy` the 64-hex preimage (text and --json); the default
// `--to phrase` keeps printing the record as `ms decode` does.
// ---------------------------------------------------------------------------
/// Preimage shares of X = 0x11 x 32 (the CLI has no preimage source for
/// split, F-468) and the `hash` single `encode` gives for X.
fn preimage_shares_and_single() -> (String, String) {
    let x = Payload::Preimage(Zeroizing::new([0x11u8; 32]));
    let shares = encode_shares(Tag::HASH, Threshold::new(2).unwrap(), 2, &x).unwrap();
    (shares.join("\n"), ms_codec::encode(Tag::HASH, &x).unwrap())
}
const X11_HEX: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const PRIVATE_KEY_ADVISORY: &str = "warning: stdout carries private key material (can spend) \u{2014} redirect or encrypt (e.g. '> file.txt' or '| age -e ...')";
const PREIMAGE_AS_ENTROPY_WARNING: &str = "warning: these 64 hex characters are a hashlock PREIMAGE, not seed entropy \u{2014} the first spend of the hash path publishes them in the clear, so never use them as a seed (ms encode, ms split or ms derive --hex); --to ms1 prints the preimage plate string";

#[test]
fn bughunt_ms_04_combine_to_ms1_ignored_for_preimage() {
    let (input, single) = preimage_shares_and_single();
    let o = ms(&["combine", "--to", "ms1", "-"], &input);
    assert_eq!(o.status.code(), Some(0), "{}", err_s(&o));
    assert_eq!(out_s(&o), format!("{single}\n"));
    assert!(
        single.starts_with("ms10hashs") && single.len() == 75,
        "{single}"
    );
}

#[test]
fn bughunt_ms_04b_combine_to_entropy_preimage() {
    let (input, _) = preimage_shares_and_single();
    let o = ms(&["combine", "--to", "entropy", "-"], &input);
    assert_eq!(o.status.code(), Some(0), "{}", err_s(&o));
    assert_eq!(out_s(&o), format!("{X11_HEX}\n"));
    assert_eq!(
        err_s(&o),
        format!("{PREIMAGE_AS_ENTROPY_WARNING}\n{PRIVATE_KEY_ADVISORY}\n")
    );
}

#[test]
fn bughunt_ms_04c_combine_to_ms1_json_preimage() {
    let (input, single) = preimage_shares_and_single();
    let o = ms(&["combine", "--to", "ms1", "--json", "-"], &input);
    assert_eq!(o.status.code(), Some(0), "{}", err_s(&o));
    assert_eq!(
        out_s(&o),
        format!(
            "{{\"schema_version\":\"1\",\"kind\":\"preimage\",\"ms1\":\"{single}\",\"entropy_hex\":\"{X11_HEX}\"}}\n"
        )
    );
}

#[test]
fn bughunt_ms_04d_combine_to_entropy_json_preimage() {
    let (input, _) = preimage_shares_and_single();
    let o = ms(&["combine", "--to", "entropy", "--json", "-"], &input);
    assert_eq!(o.status.code(), Some(0), "{}", err_s(&o));
    assert_eq!(
        out_s(&o),
        format!(
            "{{\"schema_version\":\"1\",\"kind\":\"preimage\",\"entropy_hex\":\"{X11_HEX}\"}}\n"
        )
    );
    assert_eq!(err_s(&o), format!("{PRIVATE_KEY_ADVISORY}\n"));
}

/// Control for 04b: an entr share set of the SAME 32 bytes prints the same hex
/// with no preimage warning, so the warning is keyed on the kind, not the bytes.
#[test]
fn bughunt_ms_04e_combine_to_entropy_entr_has_no_preimage_warning() {
    let x = Payload::Entr(vec![0x11u8; 32]);
    let input = encode_shares(Tag::ENTR, Threshold::new(2).unwrap(), 2, &x)
        .unwrap()
        .join("\n");
    let o = ms(&["combine", "--to", "entropy", "-"], &input);
    assert_eq!(o.status.code(), Some(0), "{}", err_s(&o));
    assert_eq!(out_s(&o), format!("{X11_HEX}\n"));
    assert_eq!(err_s(&o), format!("{PRIVATE_KEY_ADVISORY}\n"));
}

// ---------------------------------------------------------------------------
// #5 `ms inspect` reports every k>=2 string as "would combine" with no checks.
// Clearest case: a share at index `s` (the secret-at-S) -- `ms combine`
// refuses it (SecretShareSuppliedToCombine). Ruling (FIX_bughunt_ms.md):
// FAIL with reason `share-index-secret`, exit 0; no other share reason.
// ---------------------------------------------------------------------------
const SECRET_SHARE_TEXT: &str = "FAIL: would NOT combine\n    reason: share-index-secret (share-index 's' is the secret itself, not a distributed share; ms combine refuses it)\n\nhrp: ms\nthreshold: 2\nid: zzzz\nindex: s\nchecksum_valid: true\nkind: share\n";
const SECRET_SHARE_JSON: &str = "{\"failure_reasons\":[\"share-index-secret\"],\"report\":{\"checksum_valid\":true,\"hrp\":\"ms\",\"kind\":\"share\",\"share_index\":\"s\",\"tag\":\"zzzz\",\"threshold\":2},\"schema_version\":\"1\",\"would_combine\":false,\"would_decode\":false}\n";

fn secret_share_k2() -> String {
    Codex32String::from_seed("ms", 2, "zzzz", Fe::S, &[0u8; 17])
        .unwrap()
        .to_string()
}

#[test]
fn bughunt_ms_05_inspect_would_combine_unchecked() {
    let s = secret_share_k2();
    // Reference: combine refuses an index-`s` share.
    let c = ms(&["combine", "-"], &s);
    assert_eq!(c.status.code(), Some(2), "precondition: {}", err_s(&c));
    assert!(
        err_s(&c).contains("secret share"),
        "precondition: {}",
        err_s(&c)
    );

    let j = ms(&["inspect", "--json", "-"], &s);
    assert_eq!(j.status.code(), Some(0), "{}", err_s(&j));
    assert_eq!(out_s(&j), SECRET_SHARE_JSON);

    let t = ms(&["inspect", "-"], &s);
    assert_eq!(t.status.code(), Some(0), "{}", err_s(&t));
    assert_eq!(out_s(&t), SECRET_SHARE_TEXT);
}

#[test]
fn bughunt_ms_05b_inspect_uppercase_secret_share() {
    let s = secret_share_k2().to_uppercase();
    let j = ms(&["inspect", "--json", "-"], &s);
    assert_eq!(j.status.code(), Some(0), "{}", err_s(&j));
    assert_eq!(out_s(&j), SECRET_SHARE_JSON);
    let t = ms(&["inspect", "-"], &s);
    assert_eq!(out_s(&t), SECRET_SHARE_TEXT);
}

// ---------------------------------------------------------------------------
// #7 The string-length error always lists the entr set, even when that
// contradicts itself. The report's example (a 56-char mnem string) is not
// constructible (PayloadLengthMismatch fires first); the contradiction IS
// reachable through `ms repair` on a right-length string with a non-alphabet
// character (decode.rs parse_ms1_symbols used UnexpectedStringLength as a
// stand-in). Ruling (FIX_bughunt_ms.md): `invalid character '<c>'`, the
// character as typed, exit 1, JSON kind Codex32.
// ---------------------------------------------------------------------------
/// `ms repair --json` on `input`: (exit code, error kind, message, details).
fn repair_error(input: &str) -> (Option<i32>, Value, Value, Value) {
    let o = ms(&["repair", "--json", "--ms1", "-"], input);
    let v: Value = serde_json::from_slice(&o.stdout).expect("json error envelope");
    (
        o.status.code(),
        v["error"]["kind"].clone(),
        v["error"]["message"].clone(),
        v["error"]["details"].clone(),
    )
}

#[test]
fn bughunt_ms_07_string_length_error_contradicts_itself() {
    // 50 chars (a valid entr-16 length) with 'b' (not in the codex32 alphabet).
    let bad = "ms10entrsqz4b42424242424242424242424q8lurdrzjnyn08";
    assert_eq!(bad.len(), 50, "precondition: a length in the valid set");
    let (code, kind, msg, details) = repair_error(bad);
    assert_eq!(code, Some(1));
    assert_eq!(kind, "Codex32");
    assert_eq!(msg, "invalid character 'b' (not in codex32 alphabet)");
    assert_eq!(details, Value::Null);
}

#[test]
fn bughunt_ms_07b_repair_names_char_as_typed() {
    for (input, want) in [
        (
            "MS10ENTRSQZ4B42424242424242424242424Q8LURDRZJNYN08",
            "invalid character 'B' (not in codex32 alphabet)",
        ),
        (
            "ms10entrsqz4?42424242424242424242424q8lurdrzjnyn08",
            "invalid character '?' (not in codex32 alphabet)",
        ),
        (
            "ms10entrsqz4\u{133}42424242424242424242424q8lurdrzjnyn08",
            "invalid character '\u{133}' (not in codex32 alphabet)",
        ),
    ] {
        let (code, kind, msg, _) = repair_error(input);
        assert_eq!(
            (code, kind.as_str(), msg.as_str()),
            (Some(1), Some("Codex32"), Some(want)),
            "{input:?}"
        );
    }
    // Text mode: the same message on stderr.
    let o = ms(
        &["repair", "--ms1", "-"],
        "ms10entrsqz4?42424242424242424242424q8lurdrzjnyn08",
    );
    assert_eq!(o.status.code(), Some(1));
    assert_eq!(
        err_s(&o),
        "error: invalid character '?' (not in codex32 alphabet)\n"
    );
    assert_eq!(out_s(&o), "");
}

// ---------------------------------------------------------------------------
// #8 (CLI side) "more than 8 errors; uncorrectable" when the decoder's limit is
// 4. The holder sees this text from `ms repair`; the codec row is
// `bughunt_ms_08_too_many_errors_bound_is_8`. Fix shape is open (reword, or
// `bound: 4`): assert only that a 5-substitution string is not said to have
// "more than 8" errors.
// ---------------------------------------------------------------------------
#[test]
#[ignore = "deferred-minor: ms-too-many-errors-says-more-than-8"]
fn bughunt_ms_08_cli_repair_message_claims_more_than_8() {
    const ALPHABET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";
    let mut bad = b"ms10entrsqz4242424242424242424242424q8lurdrzjnyn08".to_vec();
    for p in [4usize, 9, 14, 19, 24] {
        let i = ALPHABET.iter().position(|&c| c == bad[p]).unwrap();
        bad[p] = ALPHABET[(i + 1) % 32];
    }
    let bad = String::from_utf8(bad).unwrap();
    let o = ms(&["repair", "--json", "--ms1", "-"], &bad);
    let v: Value = serde_json::from_slice(&o.stdout).expect("json error envelope");
    assert_eq!(
        v["error"]["kind"], "TooManyErrors",
        "precondition: 5 substitutions are uncorrectable: {v}"
    );
    let msg = v["error"]["message"].as_str().unwrap();
    assert!(
        !msg.contains("more than 8"),
        "`ms repair` on a string with 5 substitutions says: {msg:?} (the decoder corrects at most 4)"
    );
}

// ---------------------------------------------------------------------------
// #9 MnemUnknownLanguage falls through to the catch-all BadInput (exit 1,
// "unhandled ms_codec::Error variant"). Report: FormatViolation, exit 2.
// ---------------------------------------------------------------------------
#[test]
#[ignore = "deferred-minor: ms-cli-mnem-unknown-language-catch-all"]
fn bughunt_ms_09_mnem_unknown_language_unhandled() {
    // prefix 0x02 (mnem), language byte 10 (valid are 0..=9), 16 B entropy.
    let mut payload = vec![0x02u8, 10u8];
    payload.extend_from_slice(&[0u8; 16]);
    let s = Codex32String::from_seed("ms", 0, "entr", Fe::S, &payload)
        .unwrap()
        .to_string();
    assert_eq!(s.len(), 51, "precondition: a valid mnem length");

    let o = ms(&["decode", "-"], &s);
    assert!(!o.status.success(), "decode must refuse language byte 10");
    assert!(
        !err_s(&o).contains("unhandled"),
        "falls through to the catch-all: {}",
        err_s(&o)
    );
    assert_eq!(
        o.status.code(),
        Some(2),
        "report: a format violation, exit 2 (got {:?}): {}",
        o.status.code(),
        err_s(&o)
    );
}

// ---------------------------------------------------------------------------
// #10 `ms gui-schema` marks combine's `shares` positional non-repeating.
// ---------------------------------------------------------------------------
#[test]
fn bughunt_ms_10_gui_schema_combine_shares_not_repeating() {
    let o = ms(&["gui-schema"], "");
    assert!(o.status.success(), "{}", err_s(&o));
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    let combine = v["subcommands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["name"] == "combine")
        .expect("combine subcommand");
    let shares = combine["positionals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"] == "shares")
        .expect("shares positional");
    assert_eq!(
        shares["repeating"].as_bool(),
        Some(true),
        "combine's `shares` positional (Vec<String>, K or more) must be repeating: {shares}"
    );
}
