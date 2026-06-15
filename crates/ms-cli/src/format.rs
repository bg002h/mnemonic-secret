//! Output formatting helpers — chunking + engraving card + JSON output structs.
//!
//! Realizes SPEC §4 (engraving card + chunked form: 5-char groups, 10
//! groups/line max, never mid-chunk) and §5 (JSON schemas for encode /
//! decode / inspect / verify / vectors / error).

use serde::Serialize;

/// True for any display separator on intake: ALL Unicode whitespace + `-` + `,`
/// (SPEC §3.2, mstring display-grouping). None appear in the codex32 alphabet or
/// the `ms`/`1` structural chars, so stripping is unambiguous.
pub fn is_display_separator(c: char) -> bool {
    c.is_whitespace() || c == '-' || c == ','
}

/// Insert `separator` after every `group_size` chars (SPEC §3.1). `group_size == 0`
/// returns the input unchanged. Single line (legacy wrap@10 removed).
pub fn render_grouped(s: &str, group_size: usize, separator: char) -> String {
    if group_size == 0 {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len() + s.len() / group_size);
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && i % group_size == 0 {
            out.push(separator);
        }
        out.push(ch);
    }
    out
}

/// Strip every display separator (SPEC §3.2) — used on intake before decode.
/// Idempotent; strips ONLY separators (plain filter, NO doubling-dedup — that
/// heuristic is removed now that emit is print-once, §10).
pub fn strip_display_separators(s: &str) -> String {
    s.chars().filter(|&c| !is_display_separator(c)).collect()
}

/// Parse `--separator`: keyword (`space|hyphen|comma`) or literal (`" "|-|,`).
/// SPEC §5. clap value-parser; rejection is an exit-64 parse error.
pub fn parse_separator(s: &str) -> Result<char, String> {
    match s {
        "space" | " " => Ok(' '),
        "hyphen" | "-" => Ok('-'),
        "comma" | "," => Ok(','),
        other => Err(format!(
            "invalid separator {other:?}; expected one of: space|hyphen|comma (or the literal char)"
        )),
    }
}

/// Structured output for `ms encode --json` (SPEC §5.1).
/// `language` is `None` for `--hex` invocations.
#[derive(Serialize)]
pub struct EncodeJson<'a> {
    pub schema_version: &'static str,
    pub ms1: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<&'a str>,
    pub word_count: usize,
    pub entropy_hex: String,
}

/// Structured output for `ms split --json` (SPEC_ms_v0_2_kofn §3).
/// `language` is `None` for an `entr` (English-phrase / `--hex`) share-set.
#[derive(Serialize)]
pub struct SplitJson<'a> {
    pub schema_version: &'static str,
    pub shares: Vec<String>,
    pub k: u8,
    pub n: usize,
    pub id: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<&'a str>,
}

/// Structured output for `ms combine --json` (SPEC_ms_v0_2_kofn §3).
/// `entropy_hex` is always present; `phrase`/`language`/`word_count` are present
/// when the recovered secret renders to a BIP-39 phrase (`--to phrase`).
#[derive(Serialize)]
pub struct CombineJson<'a> {
    pub schema_version: &'static str,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ms1: Option<String>,
    pub entropy_hex: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phrase: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub word_count: Option<usize>,
}

/// Structured output for `ms decode --json` (SPEC §5.2).
#[derive(Serialize)]
pub struct DecodeJson<'a> {
    pub schema_version: &'static str,
    pub entropy_hex: String,
    pub phrase: String,
    pub language: &'a str,
    pub word_count: usize,
    pub language_defaulted: bool,
}

/// Structured output for `ms derive --json` (read-only: fingerprint + xpub).
#[derive(Serialize)]
pub struct DeriveJson<'a> {
    pub schema_version: &'static str,
    pub master_fingerprint: String,
    pub network: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_xpub: Option<String>,
    pub language: &'a str,
    pub language_defaulted: bool,
}

/// Inspect's `report` field (SPEC §5.3).
#[derive(Serialize)]
pub struct InspectReportJson {
    pub hrp: String,
    pub threshold: u8,
    pub tag: String,
    pub share_index: char,
    pub prefix_byte: u8,
    pub payload_bytes_hex: String,
    pub checksum_valid: bool,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

/// Structured output for `ms inspect --json` (SPEC §5.3).
#[derive(Serialize)]
pub struct InspectJson {
    pub schema_version: &'static str,
    pub report: InspectReportJson,
    pub would_decode: bool,
    pub failure_reasons: Vec<&'static str>,
}

/// Structured output for `ms verify --json` (success cases).
#[derive(Serialize)]
pub struct VerifySuccessJson<'a> {
    pub schema_version: &'static str,
    pub status: &'a str, // "valid" | "valid-future-format" | "round-trip-ok"
    pub message: &'a str,
}

/// Structured output for the JSON-mode error envelope (SPEC §5.4).
#[derive(Serialize)]
pub struct ErrorEnvelopeJson {
    pub schema_version: &'static str,
    pub error: ErrorBodyJson,
}

#[derive(Serialize)]
pub struct ErrorBodyJson {
    pub kind: &'static str,
    pub message: String,
    pub exit_code: u8,
    pub details: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_grouped_separators_and_unbroken() {
        assert_eq!(render_grouped("abcdefghij", 5, ' '), "abcde fghij");
        assert_eq!(render_grouped("abcdefghij", 5, '-'), "abcde-fghij");
        assert_eq!(render_grouped("abcdefghij", 5, ','), "abcde,fghij");
        assert_eq!(render_grouped("abcdefghij", 0, ' '), "abcdefghij");
        assert_eq!(render_grouped("abcde", 5, ' '), "abcde");
        assert_eq!(render_grouped("abcdefg", 3, '-'), "abc-def-g");
        assert_eq!(render_grouped("", 5, ' '), "");
    }

    #[test]
    fn strip_display_separators_ws_hyphen_comma() {
        assert_eq!(strip_display_separators("ab cd-ef,gh"), "abcdefgh");
        assert_eq!(strip_display_separators("ms10\tentrs\r\nqq"), "ms10entrsqq");
        let once = strip_display_separators("a b-c,d");
        assert_eq!(strip_display_separators(&once), once);
    }

    #[test]
    fn parse_separator_keyword_and_literal() {
        assert_eq!(parse_separator("space").unwrap(), ' ');
        assert_eq!(parse_separator(" ").unwrap(), ' ');
        assert_eq!(parse_separator("hyphen").unwrap(), '-');
        assert_eq!(parse_separator("comma").unwrap(), ',');
        assert!(parse_separator("bogus").is_err());
    }

    #[test]
    fn encode_json_serializes_correctly() {
        let j = EncodeJson {
            schema_version: "1",
            ms1: "ms10entrs...",
            language: Some("english"),
            word_count: 12,
            entropy_hex: "00".repeat(16),
        };
        let s = serde_json::to_string(&j).unwrap();
        assert!(s.starts_with("{\"schema_version\":\"1\""));
        assert!(s.contains("\"ms1\":\"ms10entrs...\""));
        assert!(s.contains("\"language\":\"english\""));
    }

    #[test]
    fn encode_json_omits_language_for_hex_input() {
        let j = EncodeJson {
            schema_version: "1",
            ms1: "ms10...",
            language: None,
            word_count: 12,
            entropy_hex: "00".repeat(16),
        };
        let s = serde_json::to_string(&j).unwrap();
        assert!(!s.contains("language"));
    }
}

/// Same canonical display-grouping vectors as the toolkit + the other siblings
/// (copy is checksum-pinned in CI). Proves ms-cli's render/strip match
/// byte-for-byte. SPEC §8. Bin-crate unit test (ms-cli is bin-only).
#[cfg(test)]
mod conformance {
    use super::{render_grouped, strip_display_separators};

    fn decode(f: &str) -> String {
        if f == "<empty>" {
            return String::new();
        }
        f.replace("<sp>", " ")
            .replace("<tab>", "\t")
            .replace("<lf>", "\n")
            .replace("<cr>", "\r")
    }

    fn sep(k: &str) -> char {
        match k {
            "space" => ' ',
            "hyphen" => '-',
            "comma" => ',',
            "none" => ' ',
            o => panic!("unknown separator keyword: {o}"),
        }
    }

    #[test]
    fn conformance_vectors_pass() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../design/display-grouping-vectors.tsv"
        );
        let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
        let mut lines = text.lines();
        assert_eq!(
            lines.next().expect("header"),
            "op\tinput\tgroup_size\tseparator\texpected\tnote",
            "vector header drift"
        );
        let mut n = 0usize;
        for (i, line) in lines.enumerate() {
            if line.is_empty() {
                continue;
            }
            let c: Vec<&str> = line.split('\t').collect();
            assert_eq!(c.len(), 6, "row {} not 6 fields: {line:?}", i + 2);
            let (op, input, gs, s, exp, note) =
                (c[0], decode(c[1]), c[2], c[3], decode(c[4]), c[5]);
            let gs: usize = gs
                .parse()
                .unwrap_or_else(|_| panic!("row {}: bad group_size", i + 2));
            let got = match op {
                "render" => render_grouped(&input, gs, sep(s)),
                "strip" => strip_display_separators(&input),
                o => panic!("row {}: unknown op {o:?}", i + 2),
            };
            assert_eq!(got, exp, "row {} ({note})", i + 2);
            n += 1;
        }
        assert!(n >= 20, "expected >=20 rows, got {n}");
    }
}
