//! CliError enum + exit-code mapping + From<ms_codec::Error> dispatch.
//!
//! Realizes SPEC §6 (exit-code table), §6.1 (CliError enum), §6.1.1
//! (dispatch table from ms_codec::Error).

use serde_json::json;

use crate::bip39_friendly::friendly_bip39;
use crate::codex32_friendly::friendly_codex32;

/// All CLI failure modes. `exit_code()` maps each to the SPEC §6 table.
///
/// L5: `Debug` is hand-rolled (NOT derived) — `Codex32(codex32::Error)` carries
/// the raw inner error, and `codex32::Error::InvalidChecksum { string }` echoes
/// the full input ms1 (secret-equivalent). A derived Debug would leak it on any
/// future `{:?}`/`unwrap`/`expect`/`panic`. The impl below delegates to the
/// sanitized `kind()`+`message()` (`Codex32` → `friendly_codex32`, which drops
/// `InvalidChecksum.string`).
#[non_exhaustive]
pub enum CliError {
    /// User-input error: bad hex, missing args, runtime input failure.
    BadInput(String),
    /// BIP-39 phrase parse / checksum failure.
    Bip39(bip39::Error),
    /// codex32 parse / BCH-checksum failure (delegated from ms_codec).
    Codex32(codex32::Error),
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
    /// SPEC §6 exit-code mapping.
    pub fn exit_code(&self) -> u8 {
        match self {
            CliError::BadInput(_)
            | CliError::Bip39(_)
            | CliError::Codex32(_)
            | CliError::UnexpectedStringLength { .. }
            | CliError::PayloadLengthMismatch { .. } => 1,
            CliError::FormatViolation { .. } => 2,
            CliError::FutureFormat { .. } => 3,
            CliError::VerifyPhraseMismatch => 4,
        }
    }

    /// Stable kebab-case-style discriminant for JSON `kind` field (SPEC §5.4).
    pub fn kind(&self) -> &'static str {
        match self {
            CliError::BadInput(_) => "BadInput",
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
    /// `codex32::Error::InvalidChecksum` carries the secret ms1 `string`; the
    /// derived Debug would leak it. `kind()` is a stable non-secret discriminant;
    /// `message()` is sanitized (Codex32 → `friendly_codex32`, which drops
    /// `InvalidChecksum.string`).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CliError::{} {{ {} }}", self.kind(), self.message())
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error: {}", self.message())
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
            // `ms combine`.
            ms_codec::Error::IsShareNotSingleString { threshold, index } => {
                CliError::FormatViolation {
                    underlying_kind: "IsShareNotSingleString",
                    message: format!(
                        "this is one share of a K-of-N set (threshold '{}', index '{}'); \
                         use `ms combine` to recombine K shares",
                        threshold, index
                    ),
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
            // `other =>` BadInput (exit 1) wildcard below.
            ms_codec::Error::InconsistentShareSet => CliError::FormatViolation {
                underlying_kind: "InconsistentShareSet",
                message: "one or more shares are not from the same split; the \
                          supplied shares do not all lie on a single Shamir \
                          polynomial"
                    .to_string(),
                details: None,
            },
            // Bad `-k` / `-n` arguments to `ms split` — user-input errors
            // (exit 1, the BadInput class). There is no exit-64 `CliError`
            // variant (clap parse-level 64s never reach `From<ms_codec::Error>`);
            // the BadInput user-input class is the existing-taxonomy fit.
            ms_codec::Error::InvalidThreshold(k) => CliError::BadInput(format!(
                "invalid threshold {}; K-of-N shares require k in 2..=9",
                k
            )),
            ms_codec::Error::InvalidShareCount { k, n } => CliError::BadInput(format!(
                "invalid share count n={} for threshold k={}; require k <= n <= 31",
                n, k
            )),

            // ms_codec::Error is #[non_exhaustive]; v0.2+ may add variants.
            // If you hit this in production, ms-codec added a variant ms-cli
            // hasn't dispatched yet — add an arm above for the new variant.
            other => CliError::BadInput(format!("unhandled ms_codec::Error variant: {:?}", other)),
        }
    }
}

/// Result alias for ms-cli.
pub type Result<T> = std::result::Result<T, CliError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_code_table_per_variant() {
        assert_eq!(CliError::BadInput("x".into()).exit_code(), 1);
        assert_eq!(CliError::UnexpectedStringLength { got: 51 }.exit_code(), 1);
        assert_eq!(
            CliError::PayloadLengthMismatch {
                got: 17,
                tag: *b"entr"
            }
            .exit_code(),
            1
        );
        assert_eq!(
            CliError::FormatViolation {
                underlying_kind: "WrongHrp",
                message: "x".into(),
                details: None,
            }
            .exit_code(),
            2
        );
        assert_eq!(CliError::FutureFormat { tag: *b"seed" }.exit_code(), 3);
        assert_eq!(CliError::VerifyPhraseMismatch.exit_code(), 4);
    }

    #[test]
    fn from_ms_codec_dispatches_correctly() {
        let e: CliError = ms_codec::Error::WrongHrp { got: "mq".into() }.into();
        assert_eq!(e.kind(), "WrongHrp");
        assert_eq!(e.exit_code(), 2);

        let e: CliError = ms_codec::Error::ReservedTagNotEmittedInV01 { got: *b"seed" }.into();
        assert_eq!(e.kind(), "FutureFormat");
        assert_eq!(e.exit_code(), 3);

        let e: CliError = ms_codec::Error::UnexpectedStringLength {
            got: 51,
            allowed: &[],
        }
        .into();
        assert_eq!(e.kind(), "UnexpectedStringLength");
        assert_eq!(e.exit_code(), 1);
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
        // funds-safety violation — NOT fall through the wildcard to BadInput
        // (exit 1).
        let e: CliError = ms_codec::Error::InconsistentShareSet.into();
        assert_eq!(e.kind(), "InconsistentShareSet");
        assert_eq!(e.exit_code(), 2);
        assert!(e.message().contains("same split"));
    }

    #[test]
    fn invalid_threshold_maps_to_bad_input_exit_1() {
        let e: CliError = ms_codec::Error::InvalidThreshold(1).into();
        assert_eq!(e.kind(), "BadInput");
        assert_eq!(e.exit_code(), 1);
        assert!(e.message().contains("2..=9"));
    }

    #[test]
    fn invalid_share_count_maps_to_bad_input_exit_1() {
        let e: CliError = ms_codec::Error::InvalidShareCount { k: 2, n: 1 }.into();
        assert_eq!(e.kind(), "BadInput");
        assert_eq!(e.exit_code(), 1);
        assert!(e.message().contains("k <= n <= 31"));
    }

    // ── L5 — CliError Debug must NOT echo the secret ms1 string ──

    #[test]
    fn debug_does_not_echo_codex32_invalid_checksum_secret() {
        // codex32::Error::InvalidChecksum.string carries the full input ms1
        // (secret-equivalent). The derived Debug would leak it; the hand-rolled
        // Debug delegates to the sanitized kind()+message().
        let e = CliError::Codex32(codex32::Error::InvalidChecksum {
            checksum: "long",
            string: "ms1secret_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx".into(),
        });
        let dbg = format!("{:?}", e);
        assert!(
            !dbg.contains("ms1secret_"),
            "Debug leaked the secret string: {dbg}"
        );
        assert!(dbg.contains("Codex32"), "Debug keeps the sanitized kind: {dbg}");
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
        let e = CliError::Codex32(codex32::Error::InvalidChar('!'));
        let dbg = format!("{:?}", e);
        assert!(dbg.contains("Codex32"), "sanitized kind present: {dbg}");
        assert!(!dbg.is_empty());
    }

    #[test]
    fn codex32_share_errors_route_through_friendly() {
        // ThresholdNotPassed surfaces via Error::Codex32 → friendly_codex32.
        let e: CliError = ms_codec::Error::Codex32(codex32::Error::ThresholdNotPassed {
            threshold: 3,
            n_shares: 1,
        })
        .into();
        assert_eq!(e.kind(), "Codex32");
        assert_eq!(e.exit_code(), 1);
        assert!(e.message().contains("not enough shares"));
    }
}
