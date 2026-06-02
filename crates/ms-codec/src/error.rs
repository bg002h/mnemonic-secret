//! ms-codec error taxonomy. Variants mirror SPEC §4 decoder validity rules
//! plus the encoder-side validation surface from SPEC §3.5 / §3.5.1.

use std::fmt;

/// ms-codec error type.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Upstream codex32 parse / checksum failure (delegated from rust-codex32).
    Codex32(codex32::Error),
    /// Mnem wordlist-language byte was not in the valid range 0..=9 (SPEC v0.2 §3).
    MnemUnknownLanguage(u8),
    /// HRP was not "ms" (SPEC §4 rule 2).
    WrongHrp {
        /// The HRP that was observed.
        got: String,
    },
    /// Threshold was not 0 (SPEC §4 rule 3).
    ThresholdNotZero {
        /// The threshold-position byte (ASCII digit) that was observed.
        got: u8,
    },
    /// Share-index was not 's' — BIP-93 requires 's' for threshold=0 (SPEC §4 rule 4).
    ShareIndexNotSecret {
        /// The share-index character that was observed.
        got: char,
    },
    /// Tag bytes were not in the codex32 alphabet (SPEC §4 rule 5).
    TagInvalidAlphabet {
        /// The 4-byte id-field bytes that failed alphabet validation.
        got: [u8; 4],
    },
    /// Tag was structurally valid but not in RESERVED_TAG_TABLE (SPEC §4 rule 6).
    UnknownTag {
        /// The 4-byte tag that was not recognized.
        got: [u8; 4],
    },
    /// Tag was in RESERVED_TAG_TABLE but reserved-not-emitted in v0.1 (SPEC §4 rule 7,
    /// SPEC §3.5.1 encoder symmetry).
    ReservedTagNotEmittedInV01 {
        /// The 4-byte reserved tag (one of seed/xprv/mnem/prvk in v0.1).
        got: [u8; 4],
    },
    /// Reserved-prefix byte was not 0x00 (SPEC §4 rule 8).
    ReservedPrefixViolation {
        /// The non-zero prefix byte that was observed.
        got: u8,
    },
    /// Total string length was outside the v0.1 emittable set (SPEC §4 rule 9).
    UnexpectedStringLength {
        /// The total string length that was observed.
        got: usize,
        /// The set of v0.1-emittable lengths.
        allowed: &'static [usize],
    },
    /// Payload byte length did not match the tag's spec (SPEC §3.5, §4 rule 10).
    PayloadLengthMismatch {
        /// The 4-byte tag whose length set was checked against.
        tag: [u8; 4],
        /// The set of valid byte lengths for this tag.
        expected: &'static [usize],
        /// The observed payload byte length (after stripping the prefix byte).
        got: usize,
    },
    /// BCH error-correction (`bch_decode`) reported the input is uncorrectable
    /// — the number of symbol errors exceeds the regular code's `t = 4`
    /// correction capacity (singleton bound `d = 8`). Surfaced by
    /// [`crate::decode_with_correction`] when `bch_decode::decode_regular_errors`
    /// returns `None`, or when a post-correction re-verification step fails
    /// (catches pathological 5+-error patterns that fool the decoder into
    /// producing a "consistent" but invalid locator). Added v0.2.0 per plan
    /// §1 D29 + §2.B.2.
    ///
    /// `bound = 8` is the BCH(93,80,8) singleton bound. ms1 is single-chunk
    /// only — no `chunk_index` field (cf. md-codec's `TooManyErrors` which
    /// carries chunk-set context).
    TooManyErrors {
        /// Singleton bound for the BCH regular code (always 8).
        bound: u8,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Codex32(e) => write!(f, "codex32 parse error: {:?}", e),
            Error::MnemUnknownLanguage(code) => {
                write!(f, "unknown mnem wordlist-language code: {0}", code)
            }
            Error::WrongHrp { got } => write!(f, "wrong HRP: got {:?}, expected \"ms\"", got),
            Error::ThresholdNotZero { got } => {
                write!(
                    f,
                    "threshold not 0 (got '{}'); v0.1 is single-string only",
                    *got as char
                )
            }
            Error::ShareIndexNotSecret { got } => {
                write!(
                    f,
                    "share-index not 's' (got '{}'); BIP-93 requires 's' for threshold=0",
                    got
                )
            }
            Error::TagInvalidAlphabet { got } => {
                write!(f, "tag bytes not in codex32 alphabet: {:?}", got)
            }
            Error::UnknownTag { got } => write!(
                f,
                "unknown tag {:?}; not a member of RESERVED_TAG_TABLE",
                std::str::from_utf8(got).unwrap_or("<non-utf8>")
            ),
            Error::ReservedTagNotEmittedInV01 { got } => write!(
                f,
                "tag {:?} reserved-not-emitted in v0.1; deferred to v0.2+",
                std::str::from_utf8(got).unwrap_or("<non-utf8>")
            ),
            Error::ReservedPrefixViolation { got } => {
                write!(f, "reserved-prefix byte was 0x{:02x}, expected 0x00", got)
            }
            Error::UnexpectedStringLength { got, allowed } => {
                write!(f, "string length {} outside v0.1 set {:?}", got, allowed)
            }
            Error::PayloadLengthMismatch { tag, expected, got } => write!(
                f,
                "tag {:?} payload length {} not in expected set {:?}",
                std::str::from_utf8(tag).unwrap_or("<non-utf8>"),
                got,
                expected
            ),
            Error::TooManyErrors { bound } => {
                write!(f, "more than {} errors; uncorrectable", bound)
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // codex32::Error doesn't impl std::error::Error in v0.1.0; chain stops here.
        None
    }
}

impl From<codex32::Error> for Error {
    fn from(e: codex32::Error) -> Self {
        Error::Codex32(e)
    }
}

/// Result alias for ms-codec.
pub type Result<T> = std::result::Result<T, Error>;
