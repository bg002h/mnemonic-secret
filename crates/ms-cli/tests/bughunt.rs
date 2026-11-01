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
// Report: should print the recovered ms1, or fail clearly. Open product
// question (print vs refuse): assert only that the reported wrong behaviour --
// exit 0 with the --to flag ignored -- is absent.
// ---------------------------------------------------------------------------
#[test]
fn bughunt_ms_04_combine_to_ms1_ignored_for_preimage() {
    let x = [0x11u8; 32];
    let shares = encode_shares(
        Tag::HASH,
        Threshold::new(2).unwrap(),
        2,
        &Payload::Preimage(Zeroizing::new(x)),
    )
    .unwrap();
    let input = shares.join("\n");

    let default = ms(&["combine", "-"], &input);
    assert!(
        default.status.success(),
        "precondition: {}",
        err_s(&default)
    );

    let o = ms(&["combine", "--to", "ms1", "-"], &input);
    if o.status.success() {
        // Accepting it means printing the recovered ms1 (a `hash`-tag single
        // that decodes back to the same preimage), not the --to-less record.
        let so = out_s(&o);
        assert_ne!(
            so,
            out_s(&default),
            "`--to ms1` exited 0 and printed exactly what the default prints (flag silently ignored):\n{so}"
        );
        match ms_codec::decode(so.trim()) {
            Ok((_, Payload::Preimage(p))) => {
                assert_eq!(*p, x, "re-encoded ms1 must carry the same preimage")
            }
            other => panic!(
                "`--to ms1` exited 0 but stdout is not an ms1 preimage single: {so:?} -> {other:?}"
            ),
        }
    }
    // A non-zero exit is the other acceptable outcome (a clear refusal).
}

// ---------------------------------------------------------------------------
// #5 `ms inspect` reports every k>=2 string as "would combine" with no checks.
// Clearest case: a share at index `s` (the secret-at-S) -- `ms combine`
// refuses it (SecretShareSuppliedToCombine). Which other reasons to list (wrong
// HRP, unknown id) is an open product question and is NOT asserted here.
// ---------------------------------------------------------------------------
#[test]
fn bughunt_ms_05_inspect_would_combine_unchecked() {
    let s = Codex32String::from_seed("ms", 2, "zzzz", Fe::S, &[0u8; 17])
        .unwrap()
        .to_string();

    // Reference: combine refuses an index-`s` share.
    let c = ms(&["combine", "-"], &s);
    assert!(
        !c.status.success(),
        "precondition: combine must refuse an `s` share"
    );
    assert!(
        err_s(&c).contains("secret share"),
        "precondition: refusal is SecretShareSuppliedToCombine: {}",
        err_s(&c)
    );

    let mut failures: Vec<String> = Vec::new();

    let j = ms(&["inspect", "--json", "-"], &s);
    assert!(j.status.success(), "inspect --json: {}", err_s(&j));
    let v: Value = serde_json::from_slice(&j.stdout).unwrap();
    if v["would_combine"] == Value::Bool(true) {
        failures.push(format!(
            "inspect --json says would_combine=true, failure_reasons={} (combine refuses this string)",
            v["failure_reasons"]
        ));
    }

    let t = ms(&["inspect", "-"], &s);
    let first = out_s(&t).lines().next().unwrap_or("").to_string();
    if first.starts_with("OK:") {
        failures.push(format!(
            "inspect text verdict is {first:?} (combine refuses this string)"
        ));
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

// ---------------------------------------------------------------------------
// #7 The string-length error always lists the entr set, even when that
// contradicts itself. The report's example (a 56-char mnem string) is not
// constructible (PayloadLengthMismatch fires first); the contradiction IS
// reachable through `ms repair` on a right-length string with a non-alphabet
// character (decode.rs parse_ms1_symbols uses UnexpectedStringLength as a
// stand-in). Correct: never say "length N not in [..N..]".
// ---------------------------------------------------------------------------
#[test]
fn bughunt_ms_07_string_length_error_contradicts_itself() {
    // 50 chars (a valid entr-16 length) with 'b' (not in the codex32 alphabet).
    let bad = "ms10entrsqz4b42424242424242424242424q8lurdrzjnyn08";
    assert_eq!(bad.len(), 50, "precondition: a length in the valid set");
    let o = ms(&["repair", "--json", "--ms1", "-"], bad);
    assert!(
        !o.status.success(),
        "repair of a non-alphabet string must fail"
    );
    let v: Value = serde_json::from_slice(&o.stdout).expect("json error envelope");
    let e = &v["error"];
    if e["kind"] == "UnexpectedStringLength" {
        let got = e["details"]["got"].as_u64().unwrap();
        let allowed: Vec<u64> = e["details"]["allowed"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_u64().unwrap())
            .collect();
        assert!(
            !allowed.contains(&got),
            "self-contradictory error: {}",
            e["message"]
        );
    }
}

// ---------------------------------------------------------------------------
// #8 (CLI side) "more than 8 errors; uncorrectable" when the decoder's limit is
// 4. The holder sees this text from `ms repair`; the codec row is
// `bughunt_ms_08_too_many_errors_bound_is_8`. Fix shape is open (reword, or
// `bound: 4`): assert only that a 5-substitution string is not said to have
// "more than 8" errors.
// ---------------------------------------------------------------------------
#[test]
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
