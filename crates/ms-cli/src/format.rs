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

/// Chunk a string into 5-char groups, wrapping at 10 groups per line max.
/// Never splits mid-chunk; trailing partial group is allowed.
///
/// SPEC §4: 5 chars per chunk, max 10 chunks/line (= 59 chars wide
/// including 9 separators), wrap at chunk boundary always.
pub fn chunked(ms1: &str) -> String {
    const CHUNK: usize = 5;
    const GROUPS_PER_LINE: usize = 10;

    let groups: Vec<&str> = ms1
        .as_bytes()
        .chunks(CHUNK)
        .map(|c| std::str::from_utf8(c).expect("ASCII codex32 chars only"))
        .collect();

    let mut out = String::new();
    for (i, line_groups) in groups.chunks(GROUPS_PER_LINE).enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&line_groups.join(" "));
    }
    out
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
    fn chunked_50_char_string_is_one_line_of_10_groups() {
        let ms1 = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
        assert_eq!(ms1.len(), 50);
        let out = chunked(ms1);
        assert_eq!(out.lines().count(), 1);
        let groups: Vec<&str> = out.split(' ').collect();
        assert_eq!(groups.len(), 10);
        assert!(groups.iter().all(|g| g.len() == 5));
    }

    #[test]
    fn chunked_75_char_string_is_two_lines_10_plus_5() {
        let ms1 = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqcwugpdxtfme2w";
        assert_eq!(ms1.len(), 75);
        let out = chunked(ms1);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 2);
        let line1_groups: Vec<&str> = lines[0].split(' ').collect();
        assert_eq!(line1_groups.len(), 10);
        let line2_groups: Vec<&str> = lines[1].split(' ').collect();
        assert_eq!(line2_groups.len(), 5);
    }

    #[test]
    fn chunked_each_v01_length_produces_expected_layout() {
        // SPEC §2.4 length set: 50 / 56 / 62 / 69 / 75
        // Each is 10/11.2/12.4/13.8/15 groups; line-wrap at chunk boundary.
        for (len, expected_groups) in [(50, 10), (56, 12), (62, 13), (69, 14), (75, 15)] {
            let s: String = "x".repeat(len);
            let out = chunked(&s);
            let total: usize = out.split([' ', '\n']).count();
            assert_eq!(
                total, expected_groups,
                "length {} expected {} groups",
                len, expected_groups
            );
        }
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
