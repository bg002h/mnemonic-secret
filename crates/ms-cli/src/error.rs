//! CliError enum + exit-code mapping + From<ms_codec::Error> dispatch.
//!
//! Realizes SPEC §6 (exit-code table), §6.1 (CliError enum), §6.1.1
//! (dispatch table from ms_codec::Error).

use serde_json::json;

use crate::bip39_friendly::friendly_bip39;
use crate::codex32_friendly::friendly_codex32;

/// All CLI failure modes. `exit_code()` maps each to its class in the
/// constellation exit-code spec's §3 (SPEC §6 here).
///
/// L5: `Debug` is hand-rolled (NOT derived) — `Codex32(ms_codec::codex32::Error)` carries
/// the raw inner error, and `ms_codec::codex32::Error::InvalidChecksum { string }` echoes
/// the full input ms1 (secret-equivalent). A derived Debug would leak it on any
/// future `{:?}`/`unwrap`/`expect`/`panic`. The impl below delegates to the
/// sanitized `kind()`+`message()` (`Codex32` → `friendly_codex32`, which drops
/// `InvalidChecksum.string`).
#[non_exhaustive]
pub enum CliError {
    /// An input is wrong for this verb (bad hex, bad phrase, empty input, the
    /// wrong kind of plate): exit 2, INVALID.
    BadInput(String),
    /// A usage error the verb itself detects from the command line alone
    /// (source arithmetic, a gate a flag must satisfy, two readers on stdin):
    /// exit 64, the same code clap's errors get.
    Usage(String),
    /// A safety or policy refusal a flag, channel or destination would waive
    /// (a digest-shaped phrase, an `--out` that exists): exit 8, REFUSED.
    Refused(String),
    /// Something needed from the system failed or is missing (an input file,
    /// stdin, an unset `@env:` variable, OS randomness, a `gen-man` directory):
    /// exit 32, IO.
    Io(String),
    /// A failure no class covers (a serializer that cannot fail, a codec
    /// variant ms-cli has not dispatched): exit 101, INTERNAL (R13).
    Internal(String),
    /// A failed `--out` write (R2 of FIX_bughunt1119_ms_out_writes): exit 32, printed
    /// on stderr as `ms: cannot write the output: <FILE>: <error>` plus the state
    /// sentence when a second look confirms it, in both modes (never an envelope).
    OutputWrite(String),
    /// `--out` names the file standard output is on (R3 of the same FIX): exit 8,
    /// printed on stderr in both modes, because an envelope on stdout would be
    /// written into that very file.
    StdoutIsOut(String),
    /// BIP-39 phrase parse / checksum failure.
    Bip39(bip39::Error),
    /// codex32 parse / BCH-checksum failure (delegated from ms_codec).
    Codex32(ms_codec::codex32::Error),
    /// String length not in v0.1 set (delegated from ms_codec).
    UnexpectedStringLength { got: usize },
    /// Payload byte length mismatch (delegated from ms_codec).
    PayloadLengthMismatch { got: usize, tag: [u8; 4] },
    /// Format violation — wrong HRP/threshold/share/tag/prefix.
    /// Carries the originating ms-codec variant name + structured fields.
    FormatViolation {
        /// e.g., "WrongHrp", "ReservedPrefixViolation"
        underlying_kind: &'static str,
        /// Human-readable one-line message.
        message: String,
        /// Structured fields preserving the underlying variant's data.
        details: Option<serde_json::Value>,
    },
    /// Valid-but-future-version format (`ReservedTagNotEmittedInV01`).
    FutureFormat { tag: [u8; 4] },
    /// Verify round-trip phrase mismatch.
    VerifyPhraseMismatch,
}

impl CliError {
    /// SPEC §6 exit-code mapping: the constellation classes of §3.
    pub fn exit_code(&self) -> u8 {
        use crate::exit_code as x;
        match self {
            CliError::BadInput(_)
            | CliError::Bip39(_)
            | CliError::Codex32(_)
            | CliError::UnexpectedStringLength { .. }
            | CliError::PayloadLengthMismatch { .. }
            | CliError::FormatViolation { .. }
            | CliError::FutureFormat { .. } => x::INVALID,
            CliError::Usage(_) => x::USAGE,
            CliError::Refused(_) | CliError::StdoutIsOut(_) => x::REFUSED,
            CliError::Io(_) | CliError::OutputWrite(_) => x::IO,
            CliError::Internal(_) => x::INTERNAL,
            CliError::VerifyPhraseMismatch => x::NO,
        }
    }

    /// Stable kebab-case-style discriminant for JSON `kind` field (SPEC §5.4).
    pub fn kind(&self) -> &'static str {
        match self {
            CliError::BadInput(_) => "BadInput",
            CliError::Usage(_) => "Usage",
            CliError::Refused(_) | CliError::StdoutIsOut(_) => "Refused",
            CliError::Io(_) | CliError::OutputWrite(_) => "Io",
            CliError::Internal(_) => "Internal",
            CliError::Bip39(_) => "Bip39",
            CliError::Codex32(_) => "Codex32",
            CliError::UnexpectedStringLength { .. } => "UnexpectedStringLength",
            CliError::PayloadLengthMismatch { .. } => "PayloadLengthMismatch",
            CliError::FormatViolation {
                underlying_kind, ..
            } => underlying_kind,
            CliError::FutureFormat { .. } => "FutureFormat",
            CliError::VerifyPhraseMismatch => "VerifyPhraseMismatch",
        }
    }

    /// Friendly human-readable message (stderr text-mode + JSON `message`).
    pub fn message(&self) -> String {
        match self {
            CliError::BadInput(m) => m.clone(),
            CliError::Usage(m)
            | CliError::Refused(m)
            | CliError::Io(m)
            | CliError::Internal(m)
            | CliError::StdoutIsOut(m)
            | CliError::OutputWrite(m) => m.clone(),
            CliError::Bip39(e) => friendly_bip39(e),
            CliError::Codex32(e) => friendly_codex32(e),
            CliError::UnexpectedStringLength { got } => {
                format!("string length {} not in v0.1 set [50, 56, 62, 69, 75]", got)
            }
            CliError::PayloadLengthMismatch { got, tag } => format!(
                "tag {:?} payload length {} not in expected set [16, 20, 24, 28, 32]",
                std::str::from_utf8(tag).unwrap_or("<non-utf8>"),
                got
            ),
            CliError::FormatViolation { message, .. } => message.clone(),
            CliError::FutureFormat { tag } => format!(
                "tag {:?} reserved-not-emitted in v0.1; deferred to v0.2+",
                std::str::from_utf8(tag).unwrap_or("<non-utf8>")
            ),
            CliError::VerifyPhraseMismatch => {
                "phrase mismatch (decoded does not match --phrase)".to_string()
            }
        }
    }

    /// True for the two errors printed on stderr in both modes and never as a JSON
    /// envelope on stdout (`OutputWrite`, `StdoutIsOut`).
    pub fn stderr_only(&self) -> bool {
        matches!(self, CliError::OutputWrite(_) | CliError::StdoutIsOut(_))
    }

    /// Structured `details` field for JSON output (SPEC §6.1.1 dispatch table).
    pub fn details(&self) -> Option<serde_json::Value> {
        match self {
            CliError::UnexpectedStringLength { got } => Some(json!({
                "got": got,
                "allowed": [50, 56, 62, 69, 75],
            })),
            CliError::PayloadLengthMismatch { got, tag } => Some(json!({
                "tag": std::str::from_utf8(tag).unwrap_or("<non-utf8>"),
                "got": got,
                "expected": [16, 20, 24, 28, 32],
            })),
            CliError::FormatViolation { details, .. } => details.clone(),
            CliError::FutureFormat { tag } => Some(json!({
                "tag": std::str::from_utf8(tag).unwrap_or("<non-utf8>"),
            })),
            _ => None,
        }
    }
}

impl std::fmt::Debug for CliError {
    /// Hand-rolled (NOT derived) so Debug NEVER prints the raw inner error.
    /// `ms_codec::codex32::Error::InvalidChecksum` carries the secret ms1 `string`; the
    /// derived Debug would leak it. `kind()` is a stable non-secret discriminant;
    /// `message()` is sanitized (Codex32 → `friendly_codex32`, which drops
    /// `InvalidChecksum.string`).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CliError::{} {{ {} }}", self.kind(), self.message())
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // T6's shape, shared with md, mk and mt: `ms: cannot write the output: ...`.
            CliError::OutputWrite(m) => write!(f, "ms: {}", m),
            _ => write!(f, "error: {}", self.message()),
        }
    }
}

impl std::error::Error for CliError {}

impl From<bip39::Error> for CliError {
    fn from(e: bip39::Error) -> Self {
        CliError::Bip39(e)
    }
}

impl From<ms_codec::Error> for CliError {
    /// SPEC §6.1.1 dispatch table.
    fn from(e: ms_codec::Error) -> Self {
        match e {
            ms_codec::Error::Codex32(c) => CliError::Codex32(c),
            ms_codec::Error::MnemUnknownLanguage(code) => CliError::FormatViolation {
                underlying_kind: "MnemUnknownLanguage",
                message: format!(
                    "unknown mnem wordlist-language code {code} (known codes are 0-9)"
                ),
                details: Some(json!({ "got": code })),
            },
            ms_codec::Error::WrongHrp { got } => CliError::FormatViolation {
                underlying_kind: "WrongHrp",
                message: format!("wrong HRP: got {:?}, expected \"ms\"", got),
                details: Some(json!({ "got": got })),
            },
            ms_codec::Error::ThresholdNotZero { got } => CliError::FormatViolation {
                underlying_kind: "ThresholdNotZero",
                message: format!(
                    "threshold not 0 (got '{}'); v0.1 is single-string only",
                    got as char
                ),
                details: Some(json!({ "got": (got as char).to_string() })),
            },
            ms_codec::Error::ShareIndexNotSecret { got } => CliError::FormatViolation {
                underlying_kind: "ShareIndexNotSecret",
                message: format!(
                    "share-index not 's' (got '{}'); BIP-93 requires 's' for threshold=0",
                    got
                ),
                details: Some(json!({ "got": got.to_string() })),
            },
            ms_codec::Error::TagInvalidAlphabet { got } => CliError::FormatViolation {
                underlying_kind: "TagInvalidAlphabet",
                message: format!("tag bytes not in codex32 alphabet: {:?}", got),
                details: Some(json!({ "got_hex": hex::encode(got) })),
            },
            ms_codec::Error::UnknownTag { got } => CliError::FormatViolation {
                underlying_kind: "UnknownTag",
                message: format!(
                    "unknown tag {:?}; not a member of RESERVED_TAG_TABLE",
                    std::str::from_utf8(&got).unwrap_or("<non-utf8>")
                ),
                details: Some(json!({
                    "tag": std::str::from_utf8(&got).unwrap_or("<non-utf8>")
                })),
            },
            ms_codec::Error::ReservedTagNotEmittedInV01 { got } => {
                CliError::FutureFormat { tag: got }
            }
            ms_codec::Error::ReservedPrefixViolation { got } => CliError::FormatViolation {
                underlying_kind: "ReservedPrefixViolation",
                message: format!("reserved-prefix byte was 0x{:02x}, expected 0x00", got),
                details: Some(json!({ "got": got })),
            },
            ms_codec::Error::UnexpectedStringLength { got, allowed: _ } => {
                CliError::UnexpectedStringLength { got }
            }
            ms_codec::Error::PayloadLengthMismatch {
                got,
                tag,
                expected: _,
            } => CliError::PayloadLengthMismatch { got, tag },
            // v0.2.0 BCH error-correction variant. `bound = 8` is the
            // BCH(93,80,8) singleton bound. Maps to a FormatViolation so
            // the SPEC §6 exit-2 slot covers BCH-uncorrectable input —
            // matches D26 (`ms repair` unrepairable → exit 2).
            ms_codec::Error::TooManyErrors { bound } => CliError::FormatViolation {
                underlying_kind: "TooManyErrors",
                message: format!("more than {} errors; uncorrectable", bound),
                details: Some(json!({ "bound": bound })),
            },

            // ── v0.2 K-of-N share variants (SPEC_ms_v0_2_kofn §3) ──
            //
            // A single-string `decode` was handed one share of a K-of-N set.
            // This is a FORMAT VIOLATION (exit 2, the §6 ms1-shape class): the
            // string is well-formed codex32 but not a v0.1 single-string. The
            // message (carried from ms_codec's Display) directs the user to
            // `ms combine`, except for index `s` (the secret itself).
            ms_codec::Error::IsShareNotSingleString { threshold, index } => {
                CliError::FormatViolation {
                    underlying_kind: "IsShareNotSingleString",
                    message: if index.eq_ignore_ascii_case(&'s') {
                        format!(
                            "this is the secret share of a K-of-N set (threshold '{}', index '{}'); \
                             it is the secret itself, not a distributed share, and `ms combine` does not take it",
                            threshold, index
                        )
                    } else {
                        format!(
                            "this is one share of a K-of-N set (threshold '{}', index '{}'); \
                             use `ms combine` to recombine K shares",
                            threshold, index
                        )
                    },
                    details: Some(json!({
                        "threshold": threshold.to_string(),
                        "index": index.to_string(),
                    })),
                }
            }
            // The secret-at-S (index 's') was supplied to `combine`. The secret
            // is the recovery TARGET, never a combine input — also a format
            // violation (exit 2): the wrong KIND of share for this operation.
            ms_codec::Error::SecretShareSuppliedToCombine => CliError::FormatViolation {
                underlying_kind: "SecretShareSuppliedToCombine",
                message: "the secret share (index 's') must not be combined; \
                          supply only distributed shares (the secret is the recovery target)"
                    .to_string(),
                details: None,
            },
            // A same-id (same hrp/id/threshold/length) but cross-polynomial set
            // was supplied to `combine` — at least one over-threshold share does
            // not lie on the polynomial the first k define. A funds-safety /
            // format violation (exit 2): combining it would yield a SILENT WRONG
            // secret. Routed explicitly so it does NOT fall through to the
            // `other =>` Internal (exit 101) wildcard below.
            ms_codec::Error::InconsistentShareSet => CliError::FormatViolation {
                underlying_kind: "InconsistentShareSet",
                message: "one or more shares are not from the same split; the \
                          supplied shares do not all lie on a single Shamir \
                          polynomial"
                    .to_string(),
                details: None,
            },
            // Bad `-k` / `-n` arguments to `ms split`: K and N are SETTINGS,
            // judged from the command line alone, so a usage error (64;
            // exit-code spec decision 2).
            ms_codec::Error::InvalidThreshold(k) => CliError::Usage(format!(
                "invalid threshold {}; K-of-N shares require k in 2..=9",
                k
            )),
            ms_codec::Error::InvalidShareCount { k, n } => CliError::Usage(format!(
                "invalid share count n={} for threshold k={}; require k <= n <= 31",
                n, k
            )),

            ms_codec::Error::PreimageLengthMismatch { got } => CliError::FormatViolation {
                underlying_kind: "PreimageLengthMismatch",
                message: format!("preimage payload is {got} bytes after the prefix; a hashlock preimage is exactly 32 bytes (64 hex characters)"),
                details: Some(json!({ "got": got })),
            },
            ms_codec::Error::TagKindMismatch { tag, prefix } => CliError::FormatViolation {
                underlying_kind: "TagKindMismatch",
                message: format!(
                    "the id {:?} names a different kind than the prefix byte 0x{prefix:02x} carries; refusing rather than reading one kind as another",
                    std::str::from_utf8(&tag).unwrap_or("<non-utf8>")
                ),
                details: Some(json!({ "tag": std::str::from_utf8(&tag).unwrap_or("<non-utf8>"), "prefix": prefix })),
            },
            ms_codec::Error::RandomnessUnavailable => CliError::Io(
                "the OS random source is unavailable; no preimage was produced".to_string(),
            ),
            // ms_codec::Error is #[non_exhaustive]; v0.2+ may add variants.
            // If you hit this in production, ms-codec added a variant ms-cli
            // hasn't dispatched yet — add an arm above for the new variant.
            // Unclassified, so INTERNAL (101, R13).
            other => CliError::Internal(format!("unhandled ms_codec::Error variant: {:?}", other)),
        }
    }
}

/// Result alias for ms-cli.
pub type Result<T> = std::result::Result<T, CliError>;

#[cfg(test)]
mod tests {
    use super::*;

    /// R13: every variant maps to its constellation class (exit-code spec §3).
    /// The match is exhaustive on purpose: a new variant fails to compile here
    /// until it is given a class.
    #[test]
    fn exit_code_table_per_variant() {
        use crate::exit_code as x;
        let rows: Vec<(CliError, u8)> = vec![
            (CliError::BadInput("x".into()), x::INVALID),
            (CliError::Usage("x".into()), x::USAGE),
            (CliError::Refused("x".into()), x::REFUSED),
            (CliError::Io("x".into()), x::IO),
            (CliError::Internal("x".into()), x::INTERNAL),
            (CliError::OutputWrite("x".into()), x::IO),
            (CliError::StdoutIsOut("x".into()), x::REFUSED),
            (CliError::Bip39(bip39::Error::BadWordCount(0)), x::INVALID),
            (
                CliError::Codex32(ms_codec::codex32::Error::InvalidChar('!')),
                x::INVALID,
            ),
            (CliError::UnexpectedStringLength { got: 51 }, x::INVALID),
            (
                CliError::PayloadLengthMismatch {
                    got: 17,
                    tag: *b"entr",
                },
                x::INVALID,
            ),
            (
                CliError::FormatViolation {
                    underlying_kind: "WrongHrp",
                    message: "x".into(),
                    details: None,
                },
                x::INVALID,
            ),
            (CliError::FutureFormat { tag: *b"seed" }, x::INVALID),
            (CliError::VerifyPhraseMismatch, x::NO),
        ];
        for (e, want) in &rows {
            assert_eq!(e.exit_code(), *want, "{e:?}");
            assert_eq!(expected_class(e), *want, "{e:?}");
        }
        // Every variant has a row: `expected_class` is exhaustive, so a new
        // variant must be given a class there, and its discriminant set must
        // equal the rows'.
        for (i, (a, _)) in rows.iter().enumerate() {
            for (b, _) in &rows[i + 1..] {
                assert_ne!(
                    std::mem::discriminant(a),
                    std::mem::discriminant(b),
                    "{a:?} has two rows"
                );
            }
        }
        let declared = enum_variants(include_str!("error.rs"), "pub enum CliError {");
        assert_eq!(
            rows.len(),
            declared.len(),
            "one row per variant: {declared:?}"
        );
    }

    /// The spec's class for each `CliError` variant, written independently of
    /// `exit_code()`. Exhaustive: a new variant does not compile until it is
    /// classed here; the row count is checked against the declared enum.
    fn expected_class(e: &CliError) -> u8 {
        use crate::exit_code as x;
        match e {
            CliError::BadInput(_) => x::INVALID,
            CliError::Usage(_) => x::USAGE,
            CliError::Refused(_) => x::REFUSED,
            CliError::Io(_) => x::IO,
            CliError::Internal(_) => x::INTERNAL,
            CliError::OutputWrite(_) => x::IO,
            CliError::StdoutIsOut(_) => x::REFUSED,
            CliError::Bip39(_) => x::INVALID,
            CliError::Codex32(_) => x::INVALID,
            CliError::UnexpectedStringLength { .. } => x::INVALID,
            CliError::PayloadLengthMismatch { .. } => x::INVALID,
            CliError::FormatViolation { .. } => x::INVALID,
            CliError::FutureFormat { .. } => x::INVALID,
            CliError::VerifyPhraseMismatch => x::NO,
        }
    }

    /// R13: every `ms_codec::Error` variant ms-cli dispatches lands in its class.
    #[test]
    fn from_ms_codec_every_variant_has_its_class() {
        use crate::exit_code as x;
        use ms_codec::Error as E;
        let rows: Vec<(E, u8)> = vec![
            (
                E::Codex32(ms_codec::codex32::Error::InvalidChar('!')),
                x::INVALID,
            ),
            (E::MnemUnknownLanguage(10), x::INVALID),
            (E::WrongHrp { got: "mq".into() }, x::INVALID),
            (E::ThresholdNotZero { got: b'2' }, x::INVALID),
            (E::ShareIndexNotSecret { got: 'a' }, x::INVALID),
            (E::TagInvalidAlphabet { got: [b'A'; 4] }, x::INVALID),
            (E::UnknownTag { got: *b"zzzz" }, x::INVALID),
            (E::ReservedTagNotEmittedInV01 { got: *b"seed" }, x::INVALID),
            (E::ReservedPrefixViolation { got: 1 }, x::INVALID),
            (
                E::UnexpectedStringLength {
                    got: 51,
                    allowed: &[],
                },
                x::INVALID,
            ),
            (
                E::PayloadLengthMismatch {
                    got: 17,
                    tag: *b"entr",
                    expected: &[],
                },
                x::INVALID,
            ),
            (E::TooManyErrors { bound: 8 }, x::INVALID),
            (
                E::IsShareNotSingleString {
                    threshold: '2',
                    index: 'a',
                },
                x::INVALID,
            ),
            (E::SecretShareSuppliedToCombine, x::INVALID),
            (E::InconsistentShareSet, x::INVALID),
            (E::InvalidThreshold(1), x::USAGE),
            (E::InvalidShareCount { k: 2, n: 1 }, x::USAGE),
            (E::PreimageLengthMismatch { got: 31 }, x::INVALID),
            (
                E::TagKindMismatch {
                    tag: *b"hash",
                    prefix: 0,
                },
                x::INVALID,
            ),
            (E::RandomnessUnavailable, x::IO),
        ];
        let mut tabled: Vec<&'static str> = Vec::new();
        for (e, want) in rows {
            tabled.push(codec_variant_name(&e));
            let c: CliError = e.into();
            assert!(!matches!(c, CliError::Internal(_)), "{c:?}");
            assert_eq!(c.exit_code(), want, "{c:?}");
        }
        // `ms_codec::Error` is `#[non_exhaustive]`, so no match here can be
        // exhaustive. ms-codec is in this workspace: read its `pub enum Error`
        // and require one row per variant, so a new codec variant fails this
        // test instead of silently reaching the 101 wildcard.
        tabled.sort_unstable();
        let mut declared = codec_error_variants_from_source();
        declared.sort_unstable();
        assert_eq!(tabled, declared, "one row per ms_codec::Error variant");
    }

    /// The variant's name; the wildcard arm keeps this compiling on a new variant
    /// (the source comparison above is what catches it).
    fn codec_variant_name(e: &ms_codec::Error) -> &'static str {
        use ms_codec::Error as E;
        match e {
            E::Codex32(_) => "Codex32",
            E::MnemUnknownLanguage(_) => "MnemUnknownLanguage",
            E::WrongHrp { .. } => "WrongHrp",
            E::ThresholdNotZero { .. } => "ThresholdNotZero",
            E::ShareIndexNotSecret { .. } => "ShareIndexNotSecret",
            E::TagInvalidAlphabet { .. } => "TagInvalidAlphabet",
            E::UnknownTag { .. } => "UnknownTag",
            E::ReservedTagNotEmittedInV01 { .. } => "ReservedTagNotEmittedInV01",
            E::PreimageLengthMismatch { .. } => "PreimageLengthMismatch",
            E::TagKindMismatch { .. } => "TagKindMismatch",
            E::RandomnessUnavailable => "RandomnessUnavailable",
            E::ReservedPrefixViolation { .. } => "ReservedPrefixViolation",
            E::UnexpectedStringLength { .. } => "UnexpectedStringLength",
            E::PayloadLengthMismatch { .. } => "PayloadLengthMismatch",
            E::TooManyErrors { .. } => "TooManyErrors",
            E::InvalidShareCount { .. } => "InvalidShareCount",
            E::InvalidThreshold(_) => "InvalidThreshold",
            E::IsShareNotSingleString { .. } => "IsShareNotSingleString",
            E::SecretShareSuppliedToCombine => "SecretShareSuppliedToCombine",
            E::InconsistentShareSet => "InconsistentShareSet",
            _ => "<untabled>",
        }
    }

    fn codec_error_variants_from_source() -> Vec<&'static str> {
        enum_variants(
            include_str!("../../ms-codec/src/error.rs"),
            "pub enum Error {",
        )
    }

    /// Variant names of the enum opened by `head` in `src` (4-space-indented
    /// identifiers inside the enum body, which closes at a column-0 `}`).
    fn enum_variants(src: &'static str, head: &str) -> Vec<&'static str> {
        let body = src.split_once(head).expect("enum declared").1;
        let body = body.split_once("\n}").expect("enum body closes").0;
        body.lines()
            .filter_map(|l| l.strip_prefix("    "))
            .filter(|l| l.starts_with(|c: char| c.is_ascii_uppercase()))
            .map(|l| {
                let end = l
                    .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                    .unwrap_or(l.len());
                &l[..end]
            })
            .collect()
    }

    #[test]
    fn from_ms_codec_dispatches_correctly() {
        let e: CliError = ms_codec::Error::WrongHrp { got: "mq".into() }.into();
        assert_eq!(e.kind(), "WrongHrp");
        assert_eq!(e.exit_code(), 2);

        let e: CliError = ms_codec::Error::ReservedTagNotEmittedInV01 { got: *b"seed" }.into();
        assert_eq!(e.kind(), "FutureFormat");
        assert_eq!(e.exit_code(), 2);

        let e: CliError = ms_codec::Error::UnexpectedStringLength {
            got: 51,
            allowed: &[],
        }
        .into();
        assert_eq!(e.kind(), "UnexpectedStringLength");
        assert_eq!(e.exit_code(), 2);
    }

    #[test]
    fn details_carries_structure_for_format_violations() {
        let e: CliError = ms_codec::Error::ReservedPrefixViolation { got: 0x01 }.into();
        let details = e.details().expect("FormatViolation has details");
        assert_eq!(details["got"], 1);
    }

    #[test]
    fn too_many_errors_maps_to_format_violation_exit_2() {
        // v0.2.0: ms_codec::Error::TooManyErrors dispatches to
        // FormatViolation (kind="TooManyErrors") so D26 'unrepairable → exit 2'
        // holds for `ms repair`. Bound is the BCH(93,80,8) singleton bound.
        let e: CliError = ms_codec::Error::TooManyErrors { bound: 8 }.into();
        assert_eq!(e.kind(), "TooManyErrors");
        assert_eq!(e.exit_code(), 2);
        let details = e.details().expect("FormatViolation has details");
        assert_eq!(details["bound"], 8);
    }

    #[test]
    fn kind_for_format_violation_carries_underlying() {
        let e: CliError = ms_codec::Error::TagInvalidAlphabet { got: [b'A'; 4] }.into();
        assert_eq!(e.kind(), "TagInvalidAlphabet");
    }

    #[test]
    fn display_includes_message() {
        let e = CliError::BadInput("test message".into());
        assert_eq!(e.to_string(), "error: test message");
    }

    // ── v0.2 K-of-N share-variant dispatch (Task 2.0) ──

    #[test]
    fn is_share_not_single_string_maps_to_format_violation_exit_2() {
        let e: CliError = ms_codec::Error::IsShareNotSingleString {
            threshold: '2',
            index: 'a',
        }
        .into();
        assert_eq!(e.kind(), "IsShareNotSingleString");
        assert_eq!(e.exit_code(), 2);
        assert!(e.message().contains("ms combine"));
        let details = e.details().expect("FormatViolation has details");
        assert_eq!(details["threshold"], "2");
        assert_eq!(details["index"], "a");
    }

    #[test]
    fn secret_share_supplied_to_combine_maps_to_format_violation_exit_2() {
        let e: CliError = ms_codec::Error::SecretShareSuppliedToCombine.into();
        assert_eq!(e.kind(), "SecretShareSuppliedToCombine");
        assert_eq!(e.exit_code(), 2);
        assert!(e.message().contains("secret share"));
    }

    #[test]
    fn inconsistent_share_set_maps_to_format_violation_exit_2() {
        // M6: a same-id cross-polynomial set must surface as an exit-2 format/
        // funds-safety violation — NOT fall through the wildcard to Internal
        // (exit 101).
        let e: CliError = ms_codec::Error::InconsistentShareSet.into();
        assert_eq!(e.kind(), "InconsistentShareSet");
        assert_eq!(e.exit_code(), 2);
        assert!(e.message().contains("same split"));
    }

    #[test]
    fn invalid_threshold_maps_to_usage_exit_64() {
        let e: CliError = ms_codec::Error::InvalidThreshold(1).into();
        assert_eq!(e.kind(), "Usage");
        assert_eq!(e.exit_code(), 64);
        assert!(e.message().contains("2..=9"));
    }

    #[test]
    fn invalid_share_count_maps_to_usage_exit_64() {
        let e: CliError = ms_codec::Error::InvalidShareCount { k: 2, n: 1 }.into();
        assert_eq!(e.kind(), "Usage");
        assert_eq!(e.exit_code(), 64);
        assert!(e.message().contains("k <= n <= 31"));
    }

    // ── L5 — CliError Debug must NOT echo the secret ms1 string ──

    #[test]
    fn debug_does_not_echo_codex32_invalid_checksum_secret() {
        // ms_codec::codex32::Error::InvalidChecksum.string carries the full input ms1
        // (secret-equivalent). The derived Debug would leak it; the hand-rolled
        // Debug delegates to the sanitized kind()+message().
        let e = CliError::Codex32(ms_codec::codex32::Error::InvalidChecksum {
            checksum: "long",
            string: "ms1secret_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx".into(),
        });
        let dbg = format!("{:?}", e);
        assert!(
            !dbg.contains("ms1secret_"),
            "Debug leaked the secret string: {dbg}"
        );
        assert!(
            dbg.contains("Codex32"),
            "Debug keeps the sanitized kind: {dbg}"
        );
        assert!(!dbg.is_empty(), "Debug stays informative: {dbg}");
        // Display is already sanitized (pin).
        assert!(
            !format!("{}", e).contains("ms1secret_"),
            "Display must not leak the secret"
        );
    }

    #[test]
    fn debug_non_invalid_checksum_arm_no_input_echo() {
        // M-4 forward-looking hardening: a non-InvalidChecksum codex32 arm also
        // must not echo input through Debug. InvalidChar carries only a single
        // structural char, never the full secret.
        let e = CliError::Codex32(ms_codec::codex32::Error::InvalidChar('!'));
        let dbg = format!("{:?}", e);
        assert!(dbg.contains("Codex32"), "sanitized kind present: {dbg}");
        assert!(!dbg.is_empty());
    }

    #[test]
    fn codex32_share_errors_route_through_friendly() {
        // ThresholdNotPassed surfaces via Error::Codex32 → friendly_codex32.
        let e: CliError = ms_codec::Error::Codex32(ms_codec::codex32::Error::ThresholdNotPassed {
            threshold: 3,
            n_shares: 1,
        })
        .into();
        assert_eq!(e.kind(), "Codex32");
        assert_eq!(e.exit_code(), 2);
        assert!(e.message().contains("not enough shares"));
    }
}
