//! Cycle-B Phase 2 — `codex32::Codex32String` MUST NOT leak its secret string
//! through `Debug`. Upstream `#[derive(...Debug)]` printed the full codex32
//! string (the L22-class footgun); we removed the derive and hand-rolled a
//! length-only redacting `Debug`.
//!
//! RULE Z-DEBUG (mirrors ms-codec's `Error`/`InspectReport` no-echo discipline):
//! the `Debug` output must not contain ANY ≥8-char contiguous window of the
//! secret data-part, and must contain the `[REDACTED` marker. The char-count is
//! non-sensitive (ms1 lengths are a small public set).

use ms_codec::codex32::Codex32String;

/// True iff `haystack` contains any `WINDOW`-char contiguous substring of
/// `needle`. The 8-char window is the no-echo oracle used across ms-codec's
/// redaction tests.
fn contains_window(haystack: &str, needle: &str, window: usize) -> bool {
    if needle.len() < window {
        // Too short to form a window — treat as "no leak possible".
        return false;
    }
    let nbytes = needle.as_bytes();
    nbytes
        .windows(window)
        .filter_map(|w| std::str::from_utf8(w).ok())
        .any(|sub| haystack.contains(sub))
}

#[test]
fn debug_does_not_echo_secret_string() {
    // A valid BIP-93 ms1 secret-at-S string (the §1 test vector). Its data-part
    // is the long `xxxxxxxx…` payload + checksum — any 8-char window of it
    // appearing in Debug would be a leak.
    let secret = "ms10testsxxxxxxxxxxxxxxxxxxxxxxxxxx4nzvca9cmczlw";
    let c32 = Codex32String::from_string(secret.into()).unwrap();

    let dbg = format!("{c32:?}");

    // 1. The full secret string must not appear verbatim.
    assert!(
        !dbg.contains(secret),
        "Codex32String Debug leaked the full secret string: {dbg}"
    );

    // 2. No 8-char window of the secret may appear in Debug.
    assert!(
        !contains_window(&dbg, secret, 8),
        "Codex32String Debug leaked an 8-char window of the secret: {dbg}"
    );

    // 3. The redaction marker must be present (proves we hand-rolled Debug, not
    //    accidentally suppressed it).
    assert!(
        dbg.contains("[REDACTED"),
        "Codex32String Debug missing the [REDACTED marker: {dbg}"
    );
}

#[test]
fn debug_reports_a_plausible_length_only() {
    // The char count is public (length is a small known set); confirm Debug
    // carries the length but nothing of the payload.
    let secret = "ms10testsxxxxxxxxxxxxxxxxxxxxxxxxxx4nzvca9cmczlw";
    let c32 = Codex32String::from_string(secret.into()).unwrap();
    let dbg = format!("{c32:?}");
    let len = secret.chars().count();
    assert!(
        dbg.contains(&len.to_string()),
        "expected the public char-count {len} in Debug: {dbg}"
    );
}
