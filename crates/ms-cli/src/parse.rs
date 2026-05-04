//! Input-source resolution: arg | stdin (with strip-whitespace).
//!
//! Realizes SPEC §3.2. Stdin reader strips ALL whitespace before parsing,
//! handling three workflows with one mechanism: pipe round-trip,
//! engraver-typed-back chunked form, and terminal copy-paste artifacts.

use std::io::{self, Read};

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
    let raw = match arg {
        Some(s) if s != "-" => s.to_string(),
        _ => read_stdin()?,
    };
    Ok(strip_whitespace(&raw))
}

/// Read a BIP-39 phrase from either the supplied arg or stdin.
/// The returned String is edge-trimmed and internal whitespace runs are
/// collapsed to single spaces — preserving the space-separated word structure
/// that `bip39::Mnemonic::parse_in` requires.
pub fn read_phrase_input(arg: Option<&str>) -> Result<String> {
    let raw = match arg {
        Some(s) if s != "-" => s.to_string(),
        _ => read_stdin()?,
    };
    Ok(normalize_phrase(&raw))
}

/// Normalize a BIP-39 phrase: trim edges and collapse whitespace runs.
fn normalize_phrase(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn read_stdin() -> Result<String> {
    let mut buf = String::new();
    io::stdin()
        .read_to_string(&mut buf)
        .map_err(|e| CliError::BadInput(format!("failed to read stdin: {}", e)))?;
    Ok(buf)
}

/// Strip ALL Unicode whitespace from `s` (per `char::is_whitespace`).
///
/// SPEC §3.2 doubling-detection: `ms encode` stdout is the multi-line form
/// `<ms1>\n\n<chunked-form>` where `<chunked-form>` is the same ms1 with
/// spaces interspersed. Strip-whitespace collapses these into `<ms1><ms1>`.
/// Detect even-length stripped output where the first half equals the second
/// half AND the original input contained whitespace (i.e., the doubling can
/// only arise from stripping — a bare inline arg with no whitespace cannot
/// produce a spurious double), and return just the first half. This covers
/// the encode-piped-to-decoder case without breaking the multi-line
/// back-typed-chunked-form case (a single ms1 with spaces, NOT a doubled ms1)
/// or inline args that happen to be all-repeated bytes (e.g. all-zero hex).
pub fn strip_whitespace(s: &str) -> String {
    let had_whitespace = s.chars().any(|c| c.is_whitespace());
    let stripped: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    if had_whitespace {
        let len = stripped.len();
        if len > 0 && len % 2 == 0 {
            let half = len / 2;
            if stripped.is_char_boundary(half) && stripped[..half] == stripped[half..] {
                return stripped[..half].to_string();
            }
        }
    }
    stripped
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
    fn strip_whitespace_dedupes_doubled_content() {
        // Simulates `ms encode --phrase X | ms decode -` input:
        // encode stdout is "<ms1>\n\n<chunked>"; chunked is ms1 with spaces.
        // After strip_whitespace, content is doubled — dedupe to single copy.
        let canonical = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
        let chunked = "ms10e ntrsq qqqqq qqqqq qqqqq qqqqq qqqqq qqcj9 sxraq 34v7f";
        let encode_stdout = format!("{}\n\n{}", canonical, chunked);
        assert_eq!(strip_whitespace(&encode_stdout), canonical);

        // Single-line ms1 (no doubling) — pass through.
        assert_eq!(strip_whitespace(canonical), canonical);

        // Multi-line back-typed chunked form (single ms1 across lines) — strip ok.
        let back_typed = "ms10e ntrsq qqqqq qqqqq qqqqq qqqqq\nqqqqq qqcj9 sxraq 34v7f";
        assert_eq!(strip_whitespace(back_typed), canonical);
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
        assert_eq!(out, "abandon abandon about");
    }
}
