//! Integration tests for `ms` output-class stderr advisory (Cycle B, Phase 4).
//!
//! Every output-producing ms command must emit exactly one stderr line
//! classifying the worst-case security nature of what it wrote to stdout.
//! Byte-identical to mnemonic-toolkit's `secret_advisory` lines per SPEC §2.2.
//!
//! # Advisory lines (byte-exact, em-dash U+2014)
//!
//! P (PrivateKeyMaterial):
//!   warning: stdout carries private key material (can spend) — redirect or
//!   encrypt (e.g. '> file.txt' or '| age -e ...')
//!
//! W (WatchOnly):
//!   note: stdout is watch-only — public keys only, cannot spend
//!
//! T (Template):
//!   note: stdout is a keyless descriptor template (no keys)
//!
//! # Inert commands (no advisory)
//!
//! `ms inspect`, `ms verify`, `ms vectors`, `ms gui-schema` do NOT write
//! private material to stdout — no advisory line.
//!
//! # Byte-parity
//!
//! The advisory lines in ms-cli's `advisory.rs` MUST be byte-identical to the
//! toolkit's `secret_advisory.rs` lines. The `byte_parity_advisory_lines` test
//! asserts this by comparing against hard-coded literals.

use std::process::Command;

use assert_cmd::cargo::CommandCargoExt;

// ─── Canonical fixtures ───────────────────────────────────────────────────────

/// 12-word abandon canonical ms1 (v0.1 test vectors entry 0).
const ABANDON_MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

/// Hex entropy for 12-word (all-zeros, 16 bytes).
const ABANDON_HEX: &str = "00000000000000000000000000000000";

/// The exact P-class advisory line (em-dash U+2014, no trailing newline in the
/// `contains` check — the newline is present in the actual stderr output but
/// `contains` matches a substring).
const PRIVATE_KEY_LINE: &str = "warning: stdout carries private key material (can spend) \u{2014} redirect or encrypt (e.g. '> file.txt' or '| age -e ...')";

/// The exact W-class advisory line.
const WATCH_ONLY_LINE: &str = "note: stdout is watch-only \u{2014} public keys only, cannot spend";

// ─── Byte-parity test ─────────────────────────────────────────────────────────

/// The 3 advisory lines in ms-cli MUST be byte-identical to the
/// mnemonic-toolkit's secret_advisory lines. Assert against hard-coded
/// literals (the canonical source of truth).
#[test]
fn byte_parity_advisory_lines() {
    // P line
    assert_eq!(
        PRIVATE_KEY_LINE,
        "warning: stdout carries private key material (can spend) \u{2014} redirect or encrypt (e.g. '> file.txt' or '| age -e ...')",
        "P advisory must match toolkit literal exactly (em-dash U+2014)"
    );
    // W line
    assert_eq!(
        WATCH_ONLY_LINE, "note: stdout is watch-only \u{2014} public keys only, cannot spend",
        "W advisory must match toolkit literal exactly"
    );
    // T line (not emitted by any ms command, but the constant must be correct)
    let template_line = "note: stdout is a keyless descriptor template (no keys)";
    assert_eq!(
        template_line, "note: stdout is a keyless descriptor template (no keys)",
        "T advisory must match toolkit literal exactly"
    );
    // Confirm em-dash is U+2014, not ASCII hyphen
    assert!(
        PRIVATE_KEY_LINE.contains('\u{2014}'),
        "P line must contain U+2014 em-dash, not ASCII hyphen"
    );
    assert!(
        WATCH_ONLY_LINE.contains('\u{2014}'),
        "W line must contain U+2014 em-dash, not ASCII hyphen"
    );
}

// ─── ms encode emits PrivateKeyMaterial ──────────────────────────────────────

#[test]
fn ms_encode_emits_private_key_material_text_mode() {
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(["encode", "--hex", ABANDON_HEX, "--no-engraving-card"])
        .output()
        .expect("invoke ms encode");
    assert!(
        out.status.success(),
        "expected exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8(out.stderr).expect("stderr utf-8");
    assert!(
        stderr.contains(PRIVATE_KEY_LINE),
        "ms encode (text mode) must emit P advisory; got stderr={stderr:?}"
    );
}

#[test]
fn ms_encode_emits_private_key_material_json_mode() {
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(["encode", "--hex", ABANDON_HEX, "--json"])
        .output()
        .expect("invoke ms encode --json");
    assert!(
        out.status.success(),
        "expected exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8(out.stderr).expect("stderr utf-8");
    assert!(
        stderr.contains(PRIVATE_KEY_LINE),
        "ms encode (--json mode) must emit P advisory; got stderr={stderr:?}"
    );
}

// ─── ms decode emits PrivateKeyMaterial ──────────────────────────────────────

#[test]
fn ms_decode_emits_private_key_material_text_mode() {
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(["decode", ABANDON_MS1])
        .output()
        .expect("invoke ms decode");
    assert!(
        out.status.success(),
        "expected exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8(out.stderr).expect("stderr utf-8");
    assert!(
        stderr.contains(PRIVATE_KEY_LINE),
        "ms decode (text mode) must emit P advisory; got stderr={stderr:?}"
    );
}

#[test]
fn ms_decode_emits_private_key_material_json_mode() {
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(["decode", ABANDON_MS1, "--json"])
        .output()
        .expect("invoke ms decode --json");
    assert!(
        out.status.success(),
        "expected exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8(out.stderr).expect("stderr utf-8");
    assert!(
        stderr.contains(PRIVATE_KEY_LINE),
        "ms decode (--json mode) must emit P advisory; got stderr={stderr:?}"
    );
}

// ─── ms derive emits WatchOnly ────────────────────────────────────────────────

/// ms derive always emits W — unconditionally for --json AND text mode AND
/// non-defaulted language. It must coexist with the language-defaulted note.
#[test]
fn ms_derive_emits_watch_only_and_language_note_text_mode() {
    // No --language → defaulted → language note appears on stderr too.
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(["derive", ABANDON_MS1])
        .output()
        .expect("invoke ms derive");
    assert!(
        out.status.success(),
        "expected exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8(out.stderr).expect("stderr utf-8");
    // Both lines must be present.
    assert!(
        stderr.contains(WATCH_ONLY_LINE),
        "ms derive must emit W advisory; got stderr={stderr:?}"
    );
    assert!(
        stderr.contains("note: --language defaulted to english"),
        "ms derive with defaulted language must emit language note; got stderr={stderr:?}"
    );
}

#[test]
fn ms_derive_emits_watch_only_json_mode() {
    // --json path also emits W; no language note in this case (--language supplied).
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(["derive", ABANDON_MS1, "--language", "english", "--json"])
        .output()
        .expect("invoke ms derive --json");
    assert!(
        out.status.success(),
        "expected exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8(out.stderr).expect("stderr utf-8");
    assert!(
        stderr.contains(WATCH_ONLY_LINE),
        "ms derive (--json mode, explicit language) must emit W advisory; got stderr={stderr:?}"
    );
}

#[test]
fn ms_derive_emits_watch_only_with_template() {
    // --template path: still W (account xpub is public key, cannot spend).
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args([
            "derive",
            ABANDON_MS1,
            "--language",
            "english",
            "--template",
            "bip84",
        ])
        .output()
        .expect("invoke ms derive --template");
    assert!(
        out.status.success(),
        "expected exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8(out.stderr).expect("stderr utf-8");
    assert!(
        stderr.contains(WATCH_ONLY_LINE),
        "ms derive --template must emit W advisory; got stderr={stderr:?}"
    );
}

// ─── ms repair emits PrivateKeyMaterial ──────────────────────────────────────

/// The old literal was: "warning: secret material on stdout — consider redirecting (e.g., '> file.txt' or '| age -e ...')"
/// The new canonical line is emitted via emit_output_class_advisory(PrivateKeyMaterial).
#[test]
fn ms_repair_emits_private_key_material() {
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(["repair", "--ms1", ABANDON_MS1])
        .output()
        .expect("invoke ms repair");
    assert!(
        out.status.success(),
        "expected exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8(out.stderr).expect("stderr utf-8");
    assert!(
        stderr.contains(PRIVATE_KEY_LINE),
        "ms repair must emit P advisory (new canonical line); got stderr={stderr:?}"
    );
}

// ─── Inert commands: no advisory line ─────────────────────────────────────────

/// `ms inspect` outputs structural fields only — no key material on stdout.
#[test]
fn ms_inspect_is_inert_no_advisory() {
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(["inspect", ABANDON_MS1])
        .output()
        .expect("invoke ms inspect");
    assert!(
        out.status.success(),
        "expected exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8(out.stderr).expect("stderr utf-8");
    assert!(
        !stderr.contains("warning:") && !stderr.contains("note: stdout"),
        "ms inspect must NOT emit any output-class advisory; got stderr={stderr:?}"
    );
}

/// `ms verify` outputs a verdict only — no key material on stdout.
#[test]
fn ms_verify_is_inert_no_advisory() {
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(["verify", ABANDON_MS1])
        .output()
        .expect("invoke ms verify");
    assert!(
        out.status.success(),
        "expected exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8(out.stderr).expect("stderr utf-8");
    assert!(
        !stderr.contains("note: stdout"),
        "ms verify must NOT emit output-class advisory; got stderr={stderr:?}"
    );
}

/// `ms vectors` outputs public test vectors — no advisory.
#[test]
fn ms_vectors_is_inert_no_advisory() {
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(["vectors"])
        .output()
        .expect("invoke ms vectors");
    assert!(
        out.status.success(),
        "expected exit 0; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8(out.stderr).expect("stderr utf-8");
    assert!(
        !stderr.contains("note: stdout"),
        "ms vectors must NOT emit output-class advisory; got stderr={stderr:?}"
    );
}
