//! Integration tests for `ms repair` (v0.4.0; Tranche B.5 of v0.22.x
//! follow-ups cycle per plan §4.B.3).
//!
//! Covers all 4 cells locked in the plan:
//!   1. `repair_already_valid_input_exits_0`
//!   2. `repair_one_substitution_exits_5`
//!   3. `repair_unrepairable_exits_2`
//!   4. `repair_json_envelope_shape` — schema byte-match with toolkit's
//!      `RepairJson` (cross-CLI parser reuse)
//!
//! Test fixture: the 12-word abandon canonical ms1 from
//! `crates/ms-codec/tests/vectors/v0.1.json` (entry 0). Single-chunk per
//! codex32 spec; total length 50 chars; data part is 47 chars
//! (post-`ms1` HRP). Mirrors the mk-cli `cli_repair.rs` shape, adapted
//! for ms1's single-chunk single-HRP context + D9 secret-on-stdout
//! advisory.

use std::process::{Command, Stdio};

use assert_cmd::cargo::CommandCargoExt;

/// Canonical 12-word abandon ms1 from `crates/ms-codec/tests/vectors/v0.1.json`
/// entry 0 (`description: "12-word abandon canonical (BIP-39 [0; 16])"`).
/// Total length 50 chars; data part (post-`ms1`) is 47 chars.
const ABANDON_MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

/// Local copy of the codex32 alphabet (BIP 173 lowercase). Used to flip
/// a single character at a data-part position to a guaranteed-different
/// alphabet char.
const CODEX32_ALPHABET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

/// Flip the codex32 character at position `pos` (0-indexed into the data
/// part, i.e. chars after `ms1`). Returns the corrupted string. Replacement
/// is the next codex32-alphabet char (cyclically) — guarantees the result
/// is parseable but BCH-invalid. Mirrors mk-cli's `flip_at` helper.
fn flip_at(chunk: &str, pos: usize) -> String {
    // ms1 strings have a 3-char HRP+separator ("ms1"); the data part begins
    // at byte offset 3.
    let (prefix, rest) = chunk.split_at(3);
    let mut chars: Vec<char> = rest.chars().collect();
    let was = chars[pos];
    let alphabet_str = std::str::from_utf8(CODEX32_ALPHABET).unwrap();
    let was_idx = alphabet_str.find(was).unwrap();
    let new_idx = (was_idx + 1) % 32;
    chars[pos] = alphabet_str.chars().nth(new_idx).unwrap();
    let mut out = String::from(prefix);
    for c in chars {
        out.push(c);
    }
    out
}

fn flip_many(chunk: &str, positions: &[usize]) -> String {
    positions
        .iter()
        .fold(chunk.to_string(), |acc, &p| flip_at(&acc, p))
}

// ──────────────────────────────────────────────────────────────────────────
// Cell 1: already-valid input → exit 0, no corrections, pass-through.
// ──────────────────────────────────────────────────────────────────────────
#[test]
fn repair_already_valid_input_exits_0() {
    let mut cmd = Command::cargo_bin("ms").expect("ms binary");
    let out = cmd
        .args(["repair", "--ms1", ABANDON_MS1])
        .output()
        .expect("invoke ms repair");
    let code = out.status.code().expect("exited normally");
    assert_eq!(
        code,
        0,
        "expected exit 0 for clean input; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).expect("stdout utf-8");
    assert!(
        !stdout.contains("# Repair report"),
        "clean input must not emit a Repair report; got stdout={stdout:?}"
    );
    // The corrected chunk equals the input (pass-through, one per line).
    assert!(
        stdout.lines().any(|line| line == ABANDON_MS1),
        "expected pass-through of valid input on stdout; got {stdout:?}"
    );
    // D9 secret-on-stdout advisory MUST fire even on pass-through, since
    // ms1 itself is secret material (BIP-39 entropy). Byte-match toolkit's
    // `secret_on_stdout_warning` line.
    let stderr = String::from_utf8(out.stderr).expect("stderr utf-8");
    assert!(
        stderr.contains("warning: stdout carries private key material"),
        "expected D9 output-class advisory on stderr; got {stderr:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────
// Cell 2: one substitution → exit 5, 1 correction reported, ms1 restored.
// ──────────────────────────────────────────────────────────────────────────
#[test]
fn repair_one_substitution_exits_5() {
    // Flip 1 char inside the entropy region (data-part pos 9 is well inside;
    // the abandon ms1 has 47 data-part chars and 13 chars of BCH tail).
    let corrupted = flip_at(ABANDON_MS1, 9);

    let mut cmd = Command::cargo_bin("ms").expect("ms binary");
    let out = cmd
        .args(["repair", "--ms1", &corrupted])
        .output()
        .expect("invoke ms repair");
    let code = out.status.code().expect("exited normally");
    assert_eq!(
        code,
        5,
        "expected exit 5 (REPAIR_APPLIED); stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).expect("stdout utf-8");
    assert!(
        stdout.contains("# Repair report"),
        "expected `# Repair report` header; got {stdout:?}"
    );
    assert!(
        stdout.contains("ms1 chunk 0: 1 correction at position 9"),
        "expected per-chunk correction line at position 9; got {stdout:?}"
    );
    // Corrected chunk is the original abandon ms1 (restored).
    assert!(
        stdout.lines().any(|line| line == ABANDON_MS1),
        "expected corrected chunk to match the original valid ms1; got {stdout:?}"
    );
    // D9 advisory MUST also fire on the correction-applied path.
    let stderr = String::from_utf8(out.stderr).expect("stderr utf-8");
    assert!(
        stderr.contains("warning: stdout carries private key material"),
        "expected D9 output-class advisory on stderr; got {stderr:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────
// Cell 3: 5+ substitutions exceed t=4 capacity → exit 2 (FormatViolation
// via `ms_codec::Error::TooManyErrors`).
// ──────────────────────────────────────────────────────────────────────────
#[test]
fn repair_unrepairable_exits_2() {
    // Spread positions so the BCH locator-degree exceeds 4; 5 flips
    // distributed across the data part (47 chars).
    let irreparable = flip_many(ABANDON_MS1, &[3, 11, 19, 27, 35]);

    let mut cmd = Command::cargo_bin("ms").expect("ms binary");
    let out = cmd
        .args(["repair", "--ms1", &irreparable])
        .output()
        .expect("invoke ms repair");
    let code = out.status.code().expect("exited normally");
    assert_eq!(
        code,
        2,
        "expected exit 2 (FormatViolation::TooManyErrors); stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    // ms-cli's Display for the FormatViolation surface is the message
    // assembled in `From<ms_codec::Error>`. The load-bearing assertion is
    // the exit code (D26); message substring is a defensive sanity check.
    assert!(
        stderr.contains("uncorrectable") || stderr.contains("errors"),
        "expected BCH-uncorrectable error message on stderr; got {stderr:?}"
    );
}

// ──────────────────────────────────────────────────────────────────────────
// Cell 4: JSON envelope shape — `repair --ms1 <bad> --json` emits a
// `RepairJson`-shaped envelope (schema_version=1, kind=ms1,
// corrected_chunks, repairs). Schema byte-matches
// `mnemonic-toolkit/src/cmd/repair.rs::RepairJson` (D27 cross-CLI parser
// reuse).
// ──────────────────────────────────────────────────────────────────────────
#[test]
fn repair_json_envelope_shape() {
    let corrupted = flip_at(ABANDON_MS1, 9);

    let mut cmd = Command::cargo_bin("ms").expect("ms binary");
    let out = cmd
        .args(["repair", "--ms1", &corrupted, "--json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("invoke ms repair --json");
    let code = out.status.code().expect("exited normally");
    assert_eq!(
        code,
        5,
        "expected exit 5 for JSON-mode repair; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8(out.stdout).expect("stdout utf-8");
    let envelope: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout parses as JSON");

    // Schema mirror: byte-match with toolkit's `RepairJson` shape (D27).
    assert_eq!(
        envelope["schema_version"],
        serde_json::Value::String("1".into()),
        "schema_version must equal \"1\" (string)"
    );
    assert_eq!(
        envelope["kind"],
        serde_json::Value::String("ms1".into()),
        "kind must equal \"ms1\""
    );

    let corrected_chunks = envelope["corrected_chunks"]
        .as_array()
        .expect("corrected_chunks must be a JSON array");
    assert_eq!(corrected_chunks.len(), 1, "ms1 single-chunk → one corrected_chunk");
    assert_eq!(
        corrected_chunks[0],
        serde_json::Value::String(ABANDON_MS1.into()),
        "corrected_chunk must equal the original valid ms1"
    );

    let repairs = envelope["repairs"]
        .as_array()
        .expect("repairs must be a JSON array");
    assert_eq!(
        repairs.len(),
        1,
        "one corrupted input → one repair entry"
    );
    let r0 = &repairs[0];
    assert_eq!(
        r0["chunk_index"],
        serde_json::Value::from(0u32),
        "ms1 single-chunk → chunk_index is always 0"
    );
    assert_eq!(
        r0["original_chunk"],
        serde_json::Value::String(corrupted.clone())
    );
    assert_eq!(
        r0["corrected_chunk"],
        serde_json::Value::String(ABANDON_MS1.into())
    );

    let positions = r0["corrected_positions"]
        .as_array()
        .expect("corrected_positions must be a JSON array");
    assert_eq!(positions.len(), 1, "single-flip → one position entry");
    let p0 = &positions[0];
    assert_eq!(p0["position"], serde_json::Value::from(9u32));
    assert!(p0["was"].is_string(), "was must be a string");
    assert!(p0["now"].is_string(), "now must be a string");
    assert_ne!(
        p0["was"], p0["now"],
        "was != now for a real correction"
    );

    // D9 advisory MUST also fire in JSON mode (sensitive material is on
    // stdout regardless of representation).
    let stderr = String::from_utf8(out.stderr).expect("stderr utf-8");
    assert!(
        stderr.contains("warning: stdout carries private key material"),
        "expected D9 output-class advisory on stderr in JSON mode; got {stderr:?}"
    );
}

// Bonus dimension covered by the spawn pipeline: stdin via `-`. Not a
// plan-required cell (the plan locks 4 cells), but defensively included
// to confirm the `-` sentinel + `read_input` plumbing works in the new
// subcommand. If this becomes flaky in CI, demote to `#[ignore]`.
#[test]
fn repair_stdin_input_via_dash() {
    use std::io::Write as _;

    let corrupted = flip_at(ABANDON_MS1, 9);
    let stdin_body = format!("{corrupted}\n");

    let mut child = Command::cargo_bin("ms")
        .expect("ms binary")
        .args(["repair", "--ms1", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn ms repair --ms1 -");
    child
        .stdin
        .as_mut()
        .expect("stdin pipe")
        .write_all(stdin_body.as_bytes())
        .expect("write stdin");
    let out = child.wait_with_output().expect("wait ms repair --ms1 -");
    let code = out.status.code().expect("exited normally");
    assert_eq!(
        code,
        5,
        "expected exit 5 for stdin-with-corrupted-input; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).expect("stdout utf-8");
    assert!(
        stdout.lines().any(|line| line == ABANDON_MS1),
        "expected restored ms1 on stdout; got {stdout:?}"
    );
}
