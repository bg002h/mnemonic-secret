//! Input-source resolution: arg | stdin (with strip-whitespace).
//!
//! Realizes SPEC §3.2. Stdin reader strips ALL whitespace before parsing,
//! handling three workflows with one mechanism: pipe round-trip,
//! engraver-typed-back chunked form, and terminal copy-paste artifacts.

use std::io::{self, Read};

use zeroize::Zeroizing;

use crate::error::{CliError, Result};

/// Read input from either the supplied arg (if `Some` and not `"-"`) or stdin.
/// The returned String is whitespace-stripped (per `char::is_whitespace`).
///
/// Use for ms1 string inputs where ALL whitespace is removed so that chunked /
/// pipe-round-trip / copy-paste forms all reach the same canonical string.
///
/// The `arg` is `None` when the positional was omitted, `Some("-")` when the
/// user explicitly requested stdin, or `Some(s)` when the user provided a value.
pub fn read_input(arg: Option<&str>) -> Result<String> {
    let raw: String = match arg {
        Some(s) if s != "-" => s.to_string(),
        _ => (*read_stdin()?).clone(),
    };
    Ok(strip_whitespace(&raw))
}

/// Read a BIP-39 phrase from either the supplied arg or stdin.
/// The returned String is edge-trimmed and internal whitespace runs are
/// collapsed to single spaces — preserving the space-separated word structure
/// that `bip39::Mnemonic::parse_in` requires.
///
/// SPEC v0.9.0 §1 item 2 — returns `Zeroizing<String>` so callers can
/// move the secret-bearing buffer to a scrub-on-drop binding.
pub fn read_phrase_input(arg: Option<&str>) -> Result<Zeroizing<String>> {
    let raw: Zeroizing<String> = match arg {
        Some(s) if s != "-" => Zeroizing::new(s.to_string()),
        _ => read_stdin()?,
    };
    Ok(Zeroizing::new(normalize_phrase(&raw)))
}

/// Normalize a BIP-39 phrase: trim edges and collapse whitespace runs.
fn normalize_phrase(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Read a BIP-39 passphrase from stdin, preserving ALL bytes except a single
/// trailing `\r?\n`. A passphrase may intentionally contain leading/trailing
/// spaces, tabs, or internal whitespace, so the `strip_whitespace`/`read_input`
/// path (which would mangle it and dedup doubled strings) MUST NOT be used here.
/// Mirrors mnemonic-toolkit's `read_stdin_passphrase`.
pub(crate) fn read_stdin_passphrase() -> Result<Zeroizing<String>> {
    let mut s: Zeroizing<String> = read_stdin()?;
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
    Ok(s)
}

pub(crate) fn read_stdin() -> Result<Zeroizing<String>> {
    // SPEC v0.9.0 §1 item 2 — wrap the raw stdin buffer so the byte
    // sequence scrubs on drop. The trimmed copy emitted by callers is
    // their responsibility to wrap.
    let mut buf: Zeroizing<String> = Zeroizing::new(String::new());
    io::stdin()
        .read_to_string(&mut buf)
        .map_err(|e| CliError::BadInput(format!("failed to read stdin: {}", e)))?;
    // Cycle B Phase 3b Site 5 — pin the heap pages of the freshly-read
    // stdin buffer for the function-local scope. Per SPEC §2 row 5: scope-
    // bound to the buffer's lifetime within read_stdin. The buffer's heap
    // data pointer is stable across the move into the caller via Ok(buf);
    // however the pin is bound to this function's scope and drops at
    // return — that is the SPEC-locked tradeoff (post-substitution
    // normalize_phrase produces a fresh allocation; future hardening
    // could pin the normalized buffer at the caller site if desired).
    let _entropy_pin = crate::mlock::pin_pages_for(buf.as_bytes());
    Ok(buf)
}

/// Strip mstring display separators from `s`: ALL Unicode whitespace PLUS `-`
/// and `,` (SPEC §3.2). Delegates to `format::strip_display_separators` so a
/// grouped (space/hyphen/comma) or unbroken card both re-ingest. Plain filter,
/// NO doubling-dedup — that heuristic is removed now that every emit point is
/// print-once (§10), so the `<ms1>\n\n<chunked>` doubling can no longer occur.
pub fn strip_whitespace(s: &str) -> String {
    crate::format::strip_display_separators(s)
}

/// Returns `true` if the supplied arg resolves to stdin (None or "-").
pub fn is_stdin_arg(arg: Option<&str>) -> bool {
    matches!(arg, None | Some("-"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_whitespace_handles_all_three_workflows() {
        // Pipe round-trip with non-equal halves (no dedupe triggered — lengths differ
        // after strip): just verifies whitespace removal.
        let pipe = "ms10entrsqqqq\n\nms10e ntrsq qqqq qqqq";
        assert_eq!(strip_whitespace(pipe), "ms10entrsqqqqms10entrsqqqqqqqqq");

        // Engraver-typed-back chunked form.
        let typed = "ms10e ntrsq qqqqq\nqqqqq cj9sx";
        assert_eq!(strip_whitespace(typed), "ms10entrsqqqqqqqqqqqcj9sx");

        // Terminal copy-paste artifacts: leading/trailing whitespace + tabs.
        let pasted = "\t  ms10entrsqqqq  \n";
        assert_eq!(strip_whitespace(pasted), "ms10entrsqqqq");
    }

    #[test]
    fn strip_whitespace_strips_hyphen_and_comma_too() {
        // mstring-grouping P2: now strips `-` and `,` in addition to whitespace
        // (no doubling-dedup — emit is print-once).
        assert_eq!(strip_whitespace("ms10e,ntrs,qqqq"), "ms10entrsqqqq");
        assert_eq!(strip_whitespace("ms10e-ntrs-qqqq"), "ms10entrsqqqq");
        assert_eq!(strip_whitespace("ms10e ntrs\tqqqq"), "ms10entrsqqqq");
        assert_eq!(strip_whitespace("ms-10, e nt"), "ms10ent");
    }

    #[test]
    fn is_stdin_arg_recognizes_none_and_dash() {
        assert!(is_stdin_arg(None));
        assert!(is_stdin_arg(Some("-")));
        assert!(!is_stdin_arg(Some("ms10...")));
    }

    #[test]
    fn read_input_with_explicit_arg_returns_stripped() {
        // Note: can't easily test stdin path in a unit test; integration tests
        // (Phase 4) cover the stdin path via `assert_cmd`'s `write_stdin`.
        let out = read_input(Some("  ms10  ")).unwrap();
        assert_eq!(out, "ms10");
    }

    #[test]
    fn normalize_phrase_preserves_word_spaces() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        assert_eq!(normalize_phrase(phrase), phrase);
    }

    #[test]
    fn normalize_phrase_collapses_runs_and_trims() {
        let phrase = "  abandon  abandon  about  ";
        assert_eq!(normalize_phrase(phrase), "abandon abandon about");
    }

    #[test]
    fn read_phrase_input_with_explicit_arg_preserves_spaces() {
        let out = read_phrase_input(Some("abandon abandon about")).unwrap();
        assert_eq!(out.as_str(), "abandon abandon about");
    }
}
