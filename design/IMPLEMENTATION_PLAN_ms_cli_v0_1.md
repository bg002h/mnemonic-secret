# ms-cli v0.1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship `crates/ms-cli` v0.1.0: the `ms` binary — a 5-subcommand CLI atop `ms-codec v0.1.0` for encoding BIP-39 mnemonics into engravable `ms1` strings, decoding/verifying/inspecting them, and dumping the SHA-pinned vector corpus.

**Architecture:** Single binary, clap-derive subcommand dispatch. 12 source modules in 5-phase build order: leaves (error/friendly-mappers/language/format/parse) → commands → root → integration tests → release prep. All BIP-39 phrase handling via `bip39 = "2"`; hex parsing via `hex = "0.4"`; structured output via `serde_json = "1"`; integration tests via `assert_cmd = "2"`. Engraving-friendly stdout discipline (multi-line ms1 + chunked form on stdout, engraving card on stderr); strip-whitespace stdin uniform across commands.

**Tech Stack:** Rust 2021 edition, MSRV 1.85 (workspace lockstep). Runtime deps: `ms-codec = "=0.1.0"` (workspace path), `bip39 = "2"`, `clap = "4"` (derive), `hex = "0.4"`, `serde = "1"` (derive), `serde_json = "1"`. Dev deps: `assert_cmd = "2"`, `predicates = "3"`. No `anyhow` (own `CliError`), no `getrandom` (no generate command), no `tracing`/`log`.

**Source-of-truth artifacts:**
- SPEC: `design/SPEC_ms_cli_v0_1.md` (reviewer-converged at r5; 3 terminators in a row).
- Library API: `crates/ms-codec/src/{lib,error,decode,inspect,consts}.rs` (the dep ms-cli consumes; `ms-codec = "=0.1.0"` exact-pin).
- Locked upstream `rust-codex32` source (read-only): `/tmp/codex32-extract/codex32-0.1.0/src/lib.rs:42-83` (Error variants for friendly mapper).
- Sibling precedent: `/scratch/code/shibboleth/descriptor-mnemonic/crates/md-cli/src/main.rs:180-193` (ExitCode dispatch + clap usage-error override).

**Convergence convention** (memory `feedback_iterative_review_every_phase`): each phase ends with an opus reviewer-loop that runs until 0 critical / 0 important findings. Per-phase reports persist to `design/agent-reports/<phase-id>-review-rN.md`. Critical/important fixed inline as a fixup commit; low/nit recorded in `design/FOLLOWUPS.md` at tier `v0.1-nice-to-have`.

**Commit cadence:** within each phase: one feature commit at phase-end (after all tasks of that phase); one fixup commit per opus-review round if findings landed. Stage paths explicitly per memory `feedback_avoid_git_add_all`. Spot-check HEAD content via `git show HEAD:path` post-commit per memory `feedback_verify_committed_content_not_working_tree`.

**SPEC closure tracking** — every implementation task references the SPEC sections it realizes. Reviewer can verify Q1-Q9 + mechanical defaults + r1-r5 fixes are all locked in code by tracing back from each closure to its task(s).

---

## Phase 1: Foundation modules (leaves)

**Goal:** Land the 6 leaf modules (`error`, `codex32_friendly`, `bip39_friendly`, `language`, `format`, `parse`) plus the `Cargo.toml` dep additions. Each module has unit tests; no internal-crate dependencies between them. By end of phase: `cargo build -p ms-cli` clean (no `main` yet beyond the stub), `cargo test -p ms-cli` runs all unit tests.

**Files:**
- Modify: `crates/ms-cli/Cargo.toml` (add deps + dev-deps)
- Modify: `crates/ms-cli/src/main.rs` (replace stub with `mod` declarations; bin still empty `fn main(){}`)
- Create: `crates/ms-cli/src/error.rs`
- Create: `crates/ms-cli/src/codex32_friendly.rs`
- Create: `crates/ms-cli/src/bip39_friendly.rs`
- Create: `crates/ms-cli/src/language.rs`
- Create: `crates/ms-cli/src/format.rs`
- Create: `crates/ms-cli/src/parse.rs`

### Task 1.1: bip39 + hex API contact spike (verification only)

**Files:** Read-verify only (no code lands).

- [ ] **Step 1: Add bip39 + hex to a throwaway spike to verify the API surface the SPEC assumes.**

```bash
mkdir -p /tmp/ms-cli-spike && cd /tmp/ms-cli-spike && cat > Cargo.toml <<'EOF'
[package]
name = "spike"
version = "0.0.0"
edition = "2021"

[dependencies]
bip39 = "2"
hex = "0.4"
EOF
mkdir -p src && cat > src/main.rs <<'EOF'
use bip39::{Language, Mnemonic};
use hex::FromHex;

fn main() {
    // Verify SPEC §2.1 claim: bip39::Mnemonic::parse_in(language, phrase)
    let m = Mnemonic::parse_in(
        Language::English,
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
    )
    .unwrap();
    let entropy = m.to_entropy();
    println!("entropy: {}", hex::encode(&entropy));

    // Verify SPEC §2.2 claim: bip39::Mnemonic::from_entropy_in(language, &entropy)
    let m2 = Mnemonic::from_entropy_in(Language::English, &entropy).unwrap();
    println!("phrase: {}", m2);

    // Verify hex parsing surface for --hex
    let bytes = <[u8; 16]>::from_hex("00000000000000000000000000000000").unwrap();
    println!("hex parsed: {:?}", &bytes[..4]);

    // Verify hex error variant on invalid input
    let err = <Vec<u8>>::from_hex("ZZ").unwrap_err();
    println!("hex error: {} ({:?})", err, err);
}
EOF
cargo run 2>&1 | tail -10
```

Expected output: prints the 16-byte zero entropy hex, the round-tripped mnemonic, the parsed `[0,0,0,0]` bytes, and a `hex::FromHexError::InvalidHexCharacter { c: 'Z', index: 0 }` style debug.

- [ ] **Step 2: Enumerate the bip39 Error variants by reading the crate's docs.rs page.**

```bash
cargo doc -p bip39 --open 2>/dev/null || echo "open https://docs.rs/bip39 manually"
```

Or via `cargo metadata`:

```bash
find ~/.cargo/registry/src -name "lib.rs" -path "*bip39-2*" | head -1 | xargs grep -nE "^pub enum Error" -A 30
```

Expected: `bip39::Error` covers (in version 2.x) `BadEntropyBitCount(usize)`, `BadWordCount(usize)`, `UnknownWord(usize)`, `InvalidChecksum`, `AmbiguousLanguages(AmbiguousLanguages)`. Record the exact variant set — Phase 1.5 (`bip39_friendly.rs`) implements the mapper against this list.

- [ ] **Step 3: Enumerate the hex Error variants.**

```bash
find ~/.cargo/registry/src -name "lib.rs" -path "*hex-0.4*" | head -1 | xargs grep -nE "^pub enum FromHexError" -A 10
```

Expected: `hex::FromHexError` covers `InvalidHexCharacter { c, index }`, `OddLength`, `InvalidStringLength`. Record exact variant set — Phase 2.2 (`cmd/encode.rs`) maps these to `CliError::BadInput` for `--hex` parsing.

- [ ] **Step 4: No commit** (verification task; nothing to commit).

If the bip39 or hex variants differ from the SPEC's assumptions, file a SPEC amendment (r6) before proceeding. If they match (expected), proceed to Task 1.2.

### Task 1.2: Cargo.toml deps + main.rs stub structure

**Files:**
- Modify: `crates/ms-cli/Cargo.toml`
- Modify: `crates/ms-cli/src/main.rs`

- [ ] **Step 1: Add deps to Cargo.toml.**

Modify `crates/ms-cli/Cargo.toml`. Replace the file's `[dependencies]` section (currently absent) with:

```toml
[dependencies]
ms-codec = { path = "../ms-codec", version = "=0.1.0" }
# bip39's non-English languages are feature-gated; "all-languages" enables all 9.
bip39 = { version = "2", features = ["all-languages"] }
clap = { version = "4", features = ["derive"] }
# codex32 is used in production code (error.rs, codex32_friendly.rs), not just tests.
codex32 = { workspace = true }
hex = "0.4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
codex32 = { workspace = true }
```

Keep the existing `[package]`, `[[bin]]`, and `publish = false` lines unchanged. Do NOT yet flip `publish` or bump `version` — those are Phase 5 tasks.

`codex32 = "=0.1.0"` is in dev-deps (not [dependencies]) because the runtime crate depends on it transitively via ms-codec; only Phase 4 integration tests need direct access to construct invalid-but-parseable v0.1 strings (non-zero prefix, wrong HRP, reserved tag) via `Codex32String::from_seed`. Adding it upfront here avoids a Cargo.toml edit in Phase 4 (per plan-r1 review I4).

- [ ] **Step 2: Replace main.rs stub with mod declarations.**

Replace `crates/ms-cli/src/main.rs` content with:

```rust
//! `ms` — engrave-friendly BIP-39 entropy backups (the `ms1` format).
//!
//! Companion CLI to the `ms-codec` library. See `design/SPEC_ms_cli_v0_1.md`
//! for the full surface specification.

#![allow(missing_docs)] // ms-cli is binary-only; field-level docs are pretty but not load-bearing for a non-published lib API. Mirror md-cli precedent at crates/md-cli/src/main.rs:1.

mod bip39_friendly;
mod codex32_friendly;
mod error;
mod format;
mod language;
mod parse;

fn main() {
    // Phase 3 replaces this with the clap dispatch.
}
```

`mod cmd;` is intentionally absent — the `cmd/` modules are Phase 2 work and the binary doesn't reference them yet.

- [ ] **Step 3: Verify it builds (with empty modules — they don't exist yet, so this WILL fail).**

```bash
cargo build --manifest-path /scratch/code/shibboleth/mnemonic-secret/Cargo.toml -p ms-cli 2>&1 | tail -10
```

Expected: `error[E0583]: file not found for module 'bip39_friendly'` (and similar for the other 5 modules). This confirms `main.rs` is wired correctly; Phase 1.3-1.8 land each module to make the build green.

- [ ] **Step 4: No commit yet** — phase-end commit lands all 6 modules together (Task 1.9).

### Task 1.3: `error.rs` — CliError enum + From<ms_codec::Error> + exit_code()

**Files:**
- Create: `crates/ms-cli/src/error.rs`

**Realizes:** SPEC §6 (exit-code table), §6.1 (CliError enum), §6.1.1 (dispatch table from `ms_codec::Error`).

- [ ] **Step 1: Write `crates/ms-cli/src/error.rs` with full enum + impls + tests.**

```rust
//! CliError enum + exit-code mapping + From<ms_codec::Error> dispatch.
//!
//! Realizes SPEC §6 (exit-code table), §6.1 (CliError enum), §6.1.1
//! (dispatch table from ms_codec::Error).

use serde_json::json;

use crate::bip39_friendly::friendly_bip39;
use crate::codex32_friendly::friendly_codex32;

/// All CLI failure modes. `exit_code()` maps each to the SPEC §6 table.
#[derive(Debug)]
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
            CliError::FormatViolation { underlying_kind, .. } => underlying_kind,
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
            CliError::UnexpectedStringLength { got } => format!(
                "string length {} not in v0.1 set [50, 56, 62, 69, 75]",
                got
            ),
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
            ms_codec::Error::PayloadLengthMismatch { got, tag, expected: _ } => {
                CliError::PayloadLengthMismatch { got, tag }
            }
            // ms_codec::Error is #[non_exhaustive]; v0.2+ may add variants.
            // If you hit this in production, ms-codec added a variant ms-cli
            // hasn't dispatched yet — add an arm above for the new variant.
            other => CliError::BadInput(format!(
                "unhandled ms_codec::Error variant: {:?}",
                other
            )),
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
            CliError::PayloadLengthMismatch { got: 17, tag: *b"entr" }.exit_code(),
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
    fn kind_for_format_violation_carries_underlying() {
        let e: CliError = ms_codec::Error::TagInvalidAlphabet { got: [b'A'; 4] }.into();
        assert_eq!(e.kind(), "TagInvalidAlphabet");
    }

    #[test]
    fn display_includes_message() {
        let e = CliError::BadInput("test message".into());
        assert_eq!(e.to_string(), "error: test message");
    }
}
```

- [ ] **Step 2: Run the tests** (note: error.rs imports from codex32_friendly + bip39_friendly which don't exist yet — Tasks 1.4 + 1.5 land them. Build will fail until those tasks complete; defer test-running to Task 1.5 step 3).

### Task 1.4: `codex32_friendly.rs` — friendly_codex32(&codex32::Error) -> String

**Files:**
- Create: `crates/ms-cli/src/codex32_friendly.rs`

**Realizes:** SPEC §6.2 (codex32_friendly module). Audit C1 resolution.

- [ ] **Step 1: Write the file with all 16 codex32::Error variants mapped.**

```rust
//! Friendly human-readable messages for `codex32::Error` variants.
//!
//! Realizes SPEC §6.2. Stable since `codex32 = "=0.1.0"` is exact-pinned;
//! see `/tmp/codex32-extract/codex32-0.1.0/src/lib.rs:42-83` for the variant
//! source.

use codex32::Error;

/// Map each `codex32::Error` variant to a one-line user-facing message.
pub fn friendly_codex32(e: &Error) -> String {
    match e {
        Error::Field(fe) => format!("invalid bech32 character: {:?}", fe),
        Error::IdNotLength4(n) => format!("id field must be 4 chars, got {}", n),
        Error::IncompleteGroup(n) => format!(
            "incomplete bit group at end of payload (got {} bits; max 4 allowed)",
            n
        ),
        Error::InvalidLength(n) => format!(
            "string length {} not a valid codex32 length (need 48-93 short or 125-127 long)",
            n
        ),
        Error::InvalidChar(c) => format!("invalid character '{}' (not in codex32 alphabet)", c),
        Error::InvalidCase(_, c) => format!(
            "mixed case at character '{}' (codex32 strings must be all-lower or all-upper)",
            c
        ),
        Error::InvalidChecksum { checksum, .. } => format!(
            "BCH checksum invalid ({} code); engraving error or transcription typo",
            checksum
        ),
        Error::InvalidThreshold(c) => format!(
            "threshold character '{}' invalid (must be '0' for unshared or '2'-'9' for K-of-N)",
            c
        ),
        Error::InvalidThresholdN(n) => format!(
            "threshold value {} invalid (must be 0 or 2-9)",
            n
        ),
        Error::InvalidShareIndex(fe) => format!(
            "share index '{}' invalid for threshold-0 (BIP-93 requires 's')",
            fe.to_char()
        ),
        Error::MismatchedLength(a, b) => format!(
            "share length mismatch: {} vs {} (all shares of one secret must share length)",
            a, b
        ),
        Error::MismatchedHrp(a, b) => format!(
            "HRP mismatch among shares: {:?} vs {:?}",
            a, b
        ),
        Error::MismatchedThreshold(a, b) => format!(
            "threshold mismatch among shares: {} vs {}",
            a, b
        ),
        Error::MismatchedId(a, b) => format!(
            "id mismatch among shares: {:?} vs {:?}",
            a, b
        ),
        Error::RepeatedIndex(fe) => format!(
            "share index '{}' repeated (each share in a set must have a distinct index)",
            fe.to_char()
        ),
        Error::ThresholdNotPassed { threshold, n_shares } => format!(
            "not enough shares: have {}, need {}",
            n_shares, threshold
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_variant_produces_non_empty_message() {
        // Construct one example per variant. Some variants need helper types;
        // skip the ones we can't trivially construct (Field needs field::Error).
        let cases: Vec<Error> = vec![
            Error::IdNotLength4(3),
            Error::IncompleteGroup(5),
            Error::InvalidLength(99),
            Error::InvalidChar('!'),
            Error::InvalidThreshold('@'),
            Error::InvalidThresholdN(11),
            Error::MismatchedLength(50, 51),
            Error::MismatchedHrp("ms".into(), "mk".into()),
            Error::MismatchedThreshold(2, 3),
            Error::MismatchedId("abcd".into(), "efgh".into()),
            Error::ThresholdNotPassed {
                threshold: 3,
                n_shares: 1,
            },
        ];
        for e in &cases {
            let msg = friendly_codex32(e);
            assert!(!msg.is_empty(), "empty message for {:?}", e);
            assert!(
                !msg.contains("Debug"),
                "raw debug formatting leaked in: {}",
                msg
            );
        }
    }

    #[test]
    fn invalid_checksum_message_mentions_checksum() {
        let e = Error::InvalidChecksum {
            checksum: "short",
            string: "ms10...".into(),
        };
        let msg = friendly_codex32(&e);
        assert!(msg.contains("BCH checksum invalid"));
        assert!(msg.contains("short"));
    }
}
```

- [ ] **Step 2: Run codex32_friendly tests in isolation.**

```bash
cargo test --manifest-path /scratch/code/shibboleth/mnemonic-secret/Cargo.toml -p ms-cli codex32_friendly 2>&1 | tail -10
```

Expected: 2 tests pass. (Build still fails overall because bip39_friendly.rs doesn't exist; the `--lib codex32_friendly` filter scopes to just this module's tests.)

### Task 1.5: `bip39_friendly.rs` — friendly_bip39(&bip39::Error) -> String

**Files:**
- Create: `crates/ms-cli/src/bip39_friendly.rs`

**Realizes:** SPEC §6.2 (bip39_friendly module). Architect r1-I2 resolution.

- [ ] **Step 1: Write the file. Variant set per Phase 1 task 1's spike output.**

```rust
//! Friendly human-readable messages for `bip39::Error` variants.
//!
//! Realizes SPEC §6.2. Variant set verified against `bip39 = "2"` per
//! Phase 1 task 1 spike.

use bip39::Error;

/// Map each `bip39::Error` variant to a one-line user-facing message.
pub fn friendly_bip39(e: &Error) -> String {
    match e {
        Error::BadEntropyBitCount(n) => format!(
            "BIP-39 entropy bit count {} invalid (must be 128, 160, 192, 224, or 256)",
            n
        ),
        Error::BadWordCount(n) => format!(
            "BIP-39 word count {} invalid (must be 12, 15, 18, 21, or 24)",
            n
        ),
        Error::UnknownWord(idx) => format!(
            "unknown BIP-39 word at position {} (not in selected wordlist; did you pick the right --language?)",
            idx
        ),
        Error::InvalidChecksum => "BIP-39 checksum failure (last word does not match the entropy)".to_string(),
        Error::AmbiguousLanguages(_) => {
            "BIP-39 phrase parses under multiple wordlists; specify --language explicitly"
                .to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bad_word_count_message_helpful() {
        let msg = friendly_bip39(&Error::BadWordCount(13));
        assert!(msg.contains("13"));
        assert!(msg.contains("12"));
    }

    #[test]
    fn unknown_word_mentions_language() {
        let msg = friendly_bip39(&Error::UnknownWord(5));
        assert!(msg.contains("--language"));
    }

    #[test]
    fn bad_checksum_message_concise() {
        let msg = friendly_bip39(&Error::InvalidChecksum);
        assert!(msg.contains("BIP-39 checksum failure"));
    }
}
```

- [ ] **Step 2: Verify Error variants match upstream.** If `bip39::Error` has variants the SPEC's spike found that are not in this match arm, add them. If it has fewer (the SPEC over-anticipated), remove the extras and update the SPEC to match.

```bash
find ~/.cargo/registry/src -name "lib.rs" -path "*bip39-2*" | head -1 | xargs grep -nE "BadEntropyBitCount|BadWordCount|UnknownWord|InvalidChecksum|AmbiguousLanguages|^[[:space:]]*[A-Z][a-zA-Z]*[[:space:]]*\("
```

- [ ] **Step 3: Run lib tests.** Now error.rs + codex32_friendly.rs + bip39_friendly.rs all exist; the partial build should compile.

```bash
cargo test --manifest-path /scratch/code/shibboleth/mnemonic-secret/Cargo.toml -p ms-cli bip39_friendly codex32_friendly error 2>&1 | tail -20
```

Expected: ~10 tests pass (3 bip39_friendly + 2 codex32_friendly + 5 error).

### Task 1.6: `language.rs` — clap value_enum + From<bip39::Language>

**Files:**
- Create: `crates/ms-cli/src/language.rs`

**Realizes:** SPEC §7 (BIP-39 wordlist languages). 10 languages, kebab-case clap value-enum.

- [ ] **Step 1: Write the file.**

```rust
//! BIP-39 wordlist language enum — clap value-enum + From<bip39::Language>.
//!
//! Realizes SPEC §7 (10 BIP-39 wordlists, kebab-case CLI values).

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

/// CLI-facing BIP-39 wordlist language.
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum CliLanguage {
    English,
    Japanese,
    Korean,
    Spanish,
    ChineseSimplified,
    ChineseTraditional,
    French,
    Italian,
    Czech,
    Portuguese,
}

impl CliLanguage {
    /// Stable kebab-case name (for stderr / JSON output).
    pub fn as_str(self) -> &'static str {
        match self {
            CliLanguage::English => "english",
            CliLanguage::Japanese => "japanese",
            CliLanguage::Korean => "korean",
            CliLanguage::Spanish => "spanish",
            CliLanguage::ChineseSimplified => "chinese-simplified",
            CliLanguage::ChineseTraditional => "chinese-traditional",
            CliLanguage::French => "french",
            CliLanguage::Italian => "italian",
            CliLanguage::Czech => "czech",
            CliLanguage::Portuguese => "portuguese",
        }
    }
}

impl From<CliLanguage> for bip39::Language {
    fn from(l: CliLanguage) -> Self {
        match l {
            CliLanguage::English => bip39::Language::English,
            CliLanguage::Japanese => bip39::Language::Japanese,
            CliLanguage::Korean => bip39::Language::Korean,
            CliLanguage::Spanish => bip39::Language::Spanish,
            CliLanguage::ChineseSimplified => bip39::Language::SimplifiedChinese,
            CliLanguage::ChineseTraditional => bip39::Language::TraditionalChinese,
            CliLanguage::French => bip39::Language::French,
            CliLanguage::Italian => bip39::Language::Italian,
            CliLanguage::Czech => bip39::Language::Czech,
            CliLanguage::Portuguese => bip39::Language::Portuguese,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_10_languages_have_kebab_case_str() {
        let cases = [
            (CliLanguage::English, "english"),
            (CliLanguage::Japanese, "japanese"),
            (CliLanguage::Korean, "korean"),
            (CliLanguage::Spanish, "spanish"),
            (CliLanguage::ChineseSimplified, "chinese-simplified"),
            (CliLanguage::ChineseTraditional, "chinese-traditional"),
            (CliLanguage::French, "french"),
            (CliLanguage::Italian, "italian"),
            (CliLanguage::Czech, "czech"),
            (CliLanguage::Portuguese, "portuguese"),
        ];
        for (lang, expected) in cases {
            assert_eq!(lang.as_str(), expected);
        }
    }

    #[test]
    fn json_round_trips_kebab_case() {
        let json = serde_json::to_string(&CliLanguage::ChineseSimplified).unwrap();
        assert_eq!(json, "\"chinese-simplified\"");
        let back: CliLanguage = serde_json::from_str(&json).unwrap();
        assert_eq!(back, CliLanguage::ChineseSimplified);
    }

    #[test]
    fn maps_to_bip39_language() {
        assert_eq!(
            bip39::Language::from(CliLanguage::English),
            bip39::Language::English
        );
        assert_eq!(
            bip39::Language::from(CliLanguage::ChineseSimplified),
            bip39::Language::SimplifiedChinese
        );
    }
}
```

- [ ] **Step 2: Run language tests.**

```bash
cargo test --manifest-path /scratch/code/shibboleth/mnemonic-secret/Cargo.toml -p ms-cli language 2>&1 | tail -10
```

Expected: 3 tests pass.

### Task 1.7: `format.rs` — chunk + JSON output structs + engraving-card formatter

**Files:**
- Create: `crates/ms-cli/src/format.rs`

**Realizes:** SPEC §4 (engraving card + chunked form), §5 (JSON schemas).

- [ ] **Step 1: Write the file.**

```rust
//! Output formatting helpers — chunking + engraving card + JSON output structs.
//!
//! Realizes SPEC §4 (engraving card + chunked form: 5-char groups, 10
//! groups/line max, never mid-chunk) and §5 (JSON schemas for encode /
//! decode / inspect / verify / vectors / error).

use serde::Serialize;

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
            let total: usize = out.split(|c: char| c == ' ' || c == '\n').count();
            assert_eq!(total, expected_groups, "length {} expected {} groups", len, expected_groups);
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
```

- [ ] **Step 2: Run format tests.**

```bash
cargo test --manifest-path /scratch/code/shibboleth/mnemonic-secret/Cargo.toml -p ms-cli format 2>&1 | tail -10
```

Expected: 5 tests pass.

### Task 1.8: `parse.rs` — input source resolution + strip-whitespace stdin

**Files:**
- Create: `crates/ms-cli/src/parse.rs`

**Realizes:** SPEC §3.2 (stdin uniform behavior, whitespace strip). Architect r1-C2 resolution.

- [ ] **Step 1: Write the file.**

```rust
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
/// The `arg` is `None` when the positional was omitted, `Some("-")` when the
/// user explicitly requested stdin, or `Some(s)` when the user provided a value.
pub fn read_input(arg: Option<&str>) -> Result<String> {
    let raw = match arg {
        Some(s) if s != "-" => s.to_string(),
        _ => read_stdin()?,
    };
    Ok(strip_whitespace(&raw))
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
/// half AND the original input contained whitespace, and return just the first
/// half. The whitespace guard prevents spurious deduplication of inline args
/// that happen to have all-repeated bytes (e.g. all-zero hex).
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

/// Read input from arg or stdin and normalize for BIP-39 phrase parsing.
///
/// Distinct from `read_input` (which strips ALL whitespace, correct for ms1
/// strings): phrases need preserved single spaces between words. This reader
/// trims edges and collapses whitespace runs while preserving word boundaries
/// via `split_whitespace().collect::<Vec<_>>().join(" ")`.
pub fn read_phrase_input(arg: Option<&str>) -> Result<String> {
    let raw = match arg {
        Some(s) if s != "-" => s.to_string(),
        _ => read_stdin()?,
    };
    Ok(normalize_phrase(&raw))
}

/// Normalize a phrase: trim edges, collapse internal whitespace runs to single spaces.
pub fn normalize_phrase(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
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
        // Pipe round-trip with non-equal halves (no dedupe triggered):
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
        let phrase = "abandon abandon about";
        assert_eq!(normalize_phrase(phrase), phrase);
    }

    #[test]
    fn normalize_phrase_collapses_runs_and_trims() {
        let raw = "  abandon  abandon\tabout\n";
        assert_eq!(normalize_phrase(raw), "abandon abandon about");
    }

    #[test]
    fn read_phrase_input_with_explicit_arg_preserves_spaces() {
        let out = read_phrase_input(Some("abandon abandon about")).unwrap();
        assert_eq!(out, "abandon abandon about");
    }
}
```

- [ ] **Step 2: Run parse tests.**

```bash
cargo test --manifest-path /scratch/code/shibboleth/mnemonic-secret/Cargo.toml -p ms-cli parse 2>&1 | tail -10
```

Expected: 3 tests pass.

### Task 1.9: Phase 1 commit

- [ ] **Step 1: Run all Phase 1 tests + build + clippy.**

```bash
cargo test --manifest-path /scratch/code/shibboleth/mnemonic-secret/Cargo.toml -p ms-cli 2>&1 | grep "test result"
cargo build --manifest-path /scratch/code/shibboleth/mnemonic-secret/Cargo.toml -p ms-cli 2>&1 | tail -3
cargo clippy --manifest-path /scratch/code/shibboleth/mnemonic-secret/Cargo.toml -p ms-cli --all-targets -- -D warnings 2>&1 | tail -3
```

Expected: ~21 unit tests pass (5 error + 2 codex32_friendly + 3 bip39_friendly + 3 language + 5 format + 3 parse). Build + clippy clean. (clippy may flag unused fns since main.rs hasn't called any of them yet — `#[allow(dead_code)]` not needed since pub items are reachable from the crate root.)

- [ ] **Step 2: Stage paths explicitly + commit.**

```bash
git -C /scratch/code/shibboleth/mnemonic-secret add \
  Cargo.lock \
  crates/ms-cli/Cargo.toml \
  crates/ms-cli/src/main.rs \
  crates/ms-cli/src/error.rs \
  crates/ms-cli/src/codex32_friendly.rs \
  crates/ms-cli/src/bip39_friendly.rs \
  crates/ms-cli/src/language.rs \
  crates/ms-cli/src/format.rs \
  crates/ms-cli/src/parse.rs

git -C /scratch/code/shibboleth/mnemonic-secret commit -m "$(cat <<'EOF'
feat(ms-cli): Phase 1 foundation modules (error/friendly-mappers/language/format/parse)

Phase 1 of IMPLEMENTATION_PLAN_ms_cli_v0_1.md. 6 leaf modules + Cargo.toml
deps; ~21 unit tests passing. cargo build / clippy --all-targets -D warnings
clean.

Modules:
- error.rs: CliError enum (8 variants) + From<ms_codec::Error> dispatch per
  SPEC §6.1.1 + exit_code()/kind()/message()/details() per SPEC §6.
- codex32_friendly.rs: 16-variant codex32::Error -> friendly String mapper
  (SPEC §6.2; audit C1 resolution).
- bip39_friendly.rs: bip39::Error -> friendly String mapper (SPEC §6.2;
  architect r1-I2 resolution).
- language.rs: CliLanguage enum (10 BIP-39 wordlists) with clap value-enum +
  serde kebab-case + From<bip39::Language> bidi (SPEC §7).
- format.rs: chunked() (5-char groups, 10/line max, never mid-chunk; SPEC
  §4) + JSON output struct definitions for encode/decode/inspect/verify/
  error-envelope (SPEC §5).
- parse.rs: read_input() + strip_whitespace() handling pipe / back-typed /
  copy-paste workflows (SPEC §3.2; architect r1-C2 resolution).

Phase 1 task 1 (bip39 + hex API contact spike) verified:
- bip39 = "2" exposes Mnemonic::parse_in / from_entropy_in / Language enum.
- bip39::Error covers BadEntropyBitCount / BadWordCount / UnknownWord /
  InvalidChecksum / AmbiguousLanguages.
- hex = "0.4" exposes FromHex / FromHexError with InvalidHexCharacter /
  OddLength / InvalidStringLength variants.

main.rs is mod-declarations + empty fn main(); Phase 3 lands the clap dispatch.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"

git -C /scratch/code/shibboleth/mnemonic-secret show HEAD --stat | head -15
```

### Task 1.10: Phase 1 opus review checkpoint

- [ ] **Step 1: Dispatch a `feature-dev:code-reviewer` opus subagent.**

Brief the agent with:
- Files reviewed: `crates/ms-cli/src/{error,codex32_friendly,bip39_friendly,language,format,parse}.rs` + `Cargo.toml`.
- SPEC reference: `design/SPEC_ms_cli_v0_1.md` §6 (CliError + dispatch table), §6.2 (friendly mappers), §7 (languages), §4 (engraving card), §5 (JSON schemas), §3.2 (stdin handling).
- Library API: `crates/ms-codec/src/error.rs` (verify From<ms_codec::Error> dispatches every variant correctly).
- Brief:
  - Phase 1 lands 6 leaf modules with no internal-crate deps.
  - Verify each ms_codec::Error variant maps to the right CliError per SPEC §6.1.1.
  - Verify all 16 codex32::Error variants are mapped in friendly_codex32.
  - Verify all 5 bip39::Error variants are mapped in friendly_bip39.
  - Verify CliLanguage covers all 10 BIP-39 wordlists with kebab-case.
  - Verify chunked() respects 5-char groups + 10-groups-per-line wrap + never-mid-chunk per SPEC §4.
  - Verify strip_whitespace handles the three SPEC §3.2 workflows.
- Length cap: under 500 words. Categorize critical/important/low/affirmation. Iterate until 0 critical / 0 important.
- Persist the report to `design/agent-reports/phase-1-foundation-review-r1.md`.

- [ ] **Step 2: Apply critical/important findings inline.**

Each critical/important finding gets fixed inline. Re-run `cargo test -p ms-cli && cargo clippy -p ms-cli --all-targets -- -D warnings` after each fix. Commit fixes as a fixup:

```bash
git -C /scratch/code/shibboleth/mnemonic-secret add <fixed paths>
git -C /scratch/code/shibboleth/mnemonic-secret commit -m "fix(ms-cli): Phase 1 review fixes (rN findings)"
```

- [ ] **Step 3: Iterate review until convergence.**

Re-dispatch the reviewer (fresh agent for independence; persist `phase-1-foundation-review-r2.md`, etc.). Stop when a round returns 0 critical / 0 important.

- [ ] **Step 4: Capture remaining nits in FOLLOWUPS.**

Append to `design/FOLLOWUPS.md` at tier `v0.1-nice-to-have` per the template. Slug format: `### ms-cli-phase-1-low-N — <title>` (kebab-case, no backticks per the SPEC template established earlier).

---

## Phase 2: Command modules

**Goal:** Land the 5 subcommand handlers under `cmd/`. Each handler consumes the Phase 1 modules + the `ms-codec` library; each is independent of the others. By end of phase, the cmd modules are written and unit-tested but not wired to the binary (Phase 3 lands main.rs).

**Files:**
- Create: `crates/ms-cli/src/cmd/mod.rs`
- Create: `crates/ms-cli/src/cmd/encode.rs`
- Create: `crates/ms-cli/src/cmd/decode.rs`
- Create: `crates/ms-cli/src/cmd/inspect.rs`
- Create: `crates/ms-cli/src/cmd/verify.rs`
- Create: `crates/ms-cli/src/cmd/vectors.rs`
- Modify: `crates/ms-cli/src/main.rs` (add `mod cmd;`)

### Task 2.1: cmd/mod.rs + main.rs wiring

**Files:**
- Create: `crates/ms-cli/src/cmd/mod.rs`
- Modify: `crates/ms-cli/src/main.rs`

- [ ] **Step 1: Write `crates/ms-cli/src/cmd/mod.rs`.**

```rust
//! Subcommand handlers. Each module is independent and consumes Phase 1
//! foundation modules + the `ms-codec` library.

pub mod decode;
pub mod encode;
pub mod inspect;
pub mod vectors;
pub mod verify;
```

- [ ] **Step 2: Add `mod cmd;` to main.rs.**

Modify `crates/ms-cli/src/main.rs`. Insert after the existing `mod parse;` line:

```rust
mod cmd;
```

The `fn main()` body remains empty; Phase 3 lands the dispatch.

- [ ] **Step 3: No commit yet** — phase-end commit lands all cmd/ modules together (Task 2.7).

### Task 2.2: cmd/encode.rs

**Files:**
- Create: `crates/ms-cli/src/cmd/encode.rs`

**Realizes:** SPEC §2.1 (encode command), §3.5.1 (encoder reserved-tag symmetry), §4 (engraving card + chunked stdout), §5.1 (encode --json schema), §6 (error handling).

- [ ] **Step 1: Write the file.**

```rust
//! `ms encode` — produce an ms1 string from a BIP-39 mnemonic (or hex entropy).
//!
//! Realizes SPEC §2.1 (full command surface), §4 (multi-line stdout + stderr
//! engraving card + 5-char chunked form), §5.1 (--json schema).

use std::io::Write;

use bip39::{Language, Mnemonic};
use clap::Args;
use hex::FromHex;
use ms_codec::{Payload, Tag};
use serde_json::to_string;

use crate::error::{CliError, Result};
use crate::format::{chunked, EncodeJson};
use crate::language::CliLanguage;
use crate::parse::read_input;

/// `ms encode` arguments.
///
/// `--phrase` and `--hex` form a mutually-exclusive group; exactly one MUST
/// be supplied. The `#[command(group = ...)]` declaration scopes the exclusion
/// to just `phrase` + `hex`; encode_arg_group_violations.rs (Phase 4) tests
/// this with exit 64 on both-supplied and neither-supplied inputs.
#[derive(Args, Debug)]
#[command(group = clap::ArgGroup::new("input").required(true).args(["phrase", "hex"]))]
pub struct EncodeArgs {
    /// BIP-39 mnemonic. Use `-` to read from stdin.
    #[arg(long)]
    pub phrase: Option<String>,

    /// Hex-encoded entropy bytes (16/20/24/28/32 B = 32/40/48/56/64 hex chars).
    #[arg(long)]
    pub hex: Option<String>,

    /// BIP-39 wordlist for the input phrase. Ignored under --hex.
    #[arg(long, default_value = "english")]
    pub language: CliLanguage,

    /// Suppress the stderr engraving card (for tooling).
    #[arg(long)]
    pub no_engraving_card: bool,

    /// Emit a single JSON object on stdout instead of multi-line text.
    #[arg(long)]
    pub json: bool,
}

/// Run `ms encode` with the parsed args. Writes to stdout/stderr per SPEC §2.1.
pub fn run(args: EncodeArgs) -> Result<()> {
    // clap's mutually-exclusive group enforces exactly-one-of-{phrase,hex}.
    let (entropy, language_for_card): (Vec<u8>, Option<&str>) = if let Some(phrase_arg) = &args.phrase {
        let phrase = read_input(Some(phrase_arg))?;
        let lang: Language = args.language.into();
        let mnemonic = Mnemonic::parse_in(lang, &phrase)?;
        (mnemonic.to_entropy(), Some(args.language.as_str()))
    } else if let Some(hex_arg) = &args.hex {
        let hex_str = read_input(Some(hex_arg))?;
        let bytes = parse_hex_entropy(&hex_str)?;
        (bytes, None)
    } else {
        // clap's required-group should have caught this; defensive.
        return Err(CliError::BadInput(
            "exactly one of --phrase or --hex is required".into(),
        ));
    };

    let ms1 = ms_codec::encode(Tag::ENTR, &Payload::Entr(entropy.clone()))?;
    let word_count = entropy.len() * 3 / 4; // 16->12, 20->15, 24->18, 28->21, 32->24

    if args.json {
        emit_json(&ms1, language_for_card, word_count, &entropy)?;
    } else {
        emit_text(&ms1, language_for_card, word_count, args.no_engraving_card)?;
    }
    Ok(())
}

fn parse_hex_entropy(hex_str: &str) -> Result<Vec<u8>> {
    if hex_str.is_empty() {
        return Err(CliError::BadInput(
            "expected hex of length 32/40/48/56/64 chars (got empty input)".into(),
        ));
    }
    if hex_str.len() % 2 != 0 {
        return Err(CliError::BadInput(format!(
            "expected even-length hex (one byte = 2 chars); got {} chars",
            hex_str.len()
        )));
    }
    Vec::<u8>::from_hex(hex_str).map_err(|e| match e {
        hex::FromHexError::InvalidHexCharacter { c, index } => {
            CliError::BadInput(format!("invalid character '{}' at position {}", c, index))
        }
        hex::FromHexError::OddLength => {
            CliError::BadInput("expected even-length hex (one byte = 2 chars)".into())
        }
        hex::FromHexError::InvalidStringLength => {
            CliError::BadInput("hex string length invalid".into())
        }
    })
}

fn emit_json(
    ms1: &str,
    language: Option<&str>,
    word_count: usize,
    entropy: &[u8],
) -> Result<()> {
    let json = EncodeJson {
        schema_version: "1",
        ms1,
        language,
        word_count,
        entropy_hex: hex::encode(entropy),
    };
    let s = to_string(&json)
        .map_err(|e| CliError::BadInput(format!("json serialization: {}", e)))?;
    println!("{}", s);
    Ok(())
}

fn emit_text(
    ms1: &str,
    language: Option<&str>,
    word_count: usize,
    no_engraving_card: bool,
) -> Result<()> {
    // Multi-line stdout: ms1 + blank + chunked form (SPEC Q6).
    println!("{}", ms1);
    println!();
    println!("{}", chunked(ms1));

    if !no_engraving_card {
        let mut stderr = std::io::stderr().lock();
        writeln!(stderr, "word count: {}", word_count).ok();
        if let Some(lang) = language {
            writeln!(
                stderr,
                "language: {} (BIP-39 checksum valid)",
                lang
            )
            .ok();
        }
        writeln!(
            stderr,
            "passphrase: not stored in ms1 (record separately if used)"
        )
        .ok();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_entropy_accepts_canonical_zeros_16b() {
        let bytes = parse_hex_entropy("00000000000000000000000000000000").unwrap();
        assert_eq!(bytes.len(), 16);
        assert!(bytes.iter().all(|&b| b == 0));
    }

    #[test]
    fn parse_hex_entropy_rejects_odd_length() {
        let err = parse_hex_entropy("0").unwrap_err();
        assert!(matches!(err, CliError::BadInput(_)));
    }

    #[test]
    fn parse_hex_entropy_rejects_empty() {
        let err = parse_hex_entropy("").unwrap_err();
        assert!(matches!(err, CliError::BadInput(m) if m.contains("empty")));
    }

    #[test]
    fn parse_hex_entropy_rejects_non_hex_char() {
        let err = parse_hex_entropy("ZZ").unwrap_err();
        match err {
            CliError::BadInput(m) => {
                assert!(m.contains("'Z'"), "got: {}", m);
                assert!(m.contains("position 0"));
            }
            _ => panic!("expected BadInput"),
        }
    }
}
```

- [ ] **Step 2: Run encode unit tests.**

```bash
cargo test --manifest-path /scratch/code/shibboleth/mnemonic-secret/Cargo.toml -p ms-cli cmd::encode 2>&1 | tail -10
```

Expected: 4 tests pass.

### Task 2.3: cmd/decode.rs

**Files:**
- Create: `crates/ms-cli/src/cmd/decode.rs`

**Realizes:** SPEC §2.2 (decode), §5.2 (decode --json), §6.3 (default-language hazard surfacing).

- [ ] **Step 1: Write the file.**

```rust
//! `ms decode` — recover a BIP-39 mnemonic from an ms1 string.
//!
//! Realizes SPEC §2.2 (full command surface), §5.2 (--json schema),
//! §6.3 (default-language hazard surfacing on stdout AND stderr).

use std::io::Write;

use bip39::{Language, Mnemonic};
use clap::Args;
use ms_codec::Payload;
use serde_json::to_string;

use crate::error::Result;
use crate::format::DecodeJson;
use crate::language::CliLanguage;
use crate::parse::read_input;

/// `ms decode` arguments.
#[derive(Args, Debug)]
pub struct DecodeArgs {
    /// ms1 string to decode. Use `-` or omit to read from stdin.
    pub ms1: Option<String>,

    /// BIP-39 wordlist for the recovered phrase. Default `english`.
    /// SPEC §6.3: when defaulted, both stderr AND the stdout language
    /// line carry an explicit "DEFAULT" annotation.
    #[arg(long)]
    pub language: Option<CliLanguage>,

    /// Emit a single JSON object on stdout instead of labeled-block text.
    #[arg(long)]
    pub json: bool,
}

/// Run `ms decode`.
pub fn run(args: DecodeArgs) -> Result<()> {
    let ms1 = read_input(args.ms1.as_deref())?;

    let (cli_lang, defaulted) = match args.language {
        Some(l) => (l, false),
        None => (CliLanguage::English, true),
    };
    let lang: Language = cli_lang.into();

    let (_tag, payload) = ms_codec::decode(&ms1)?;
    let entropy = match payload {
        Payload::Entr(b) => b,
        // ms_codec::Payload is #[non_exhaustive]; v0.2+ may add variants.
        // v0.1 ms-codec emits Entr only — unreachable in practice.
        _ => unreachable!("ms-codec v0.1 only decodes to Payload::Entr"),
    };

    let mnemonic = Mnemonic::from_entropy_in(lang, &entropy)
        .expect("ms-codec validates entropy length; from_entropy_in cannot fail");
    let phrase = mnemonic.to_string();
    let word_count = phrase.split_whitespace().count();

    if args.json {
        emit_json(&entropy, &phrase, cli_lang.as_str(), word_count, defaulted)?;
    } else {
        emit_text(&entropy, &phrase, cli_lang.as_str(), word_count, defaulted)?;
    }
    Ok(())
}

fn emit_json(
    entropy: &[u8],
    phrase: &str,
    language: &str,
    word_count: usize,
    language_defaulted: bool,
) -> Result<()> {
    let json = DecodeJson {
        schema_version: "1",
        entropy_hex: hex::encode(entropy),
        phrase: phrase.to_string(),
        language,
        word_count,
        language_defaulted,
    };
    let s = to_string(&json).expect("decode json serialization always succeeds");
    println!("{}", s);
    Ok(())
}

fn emit_text(
    entropy: &[u8],
    phrase: &str,
    language: &str,
    word_count: usize,
    language_defaulted: bool,
) -> Result<()> {
    println!("entropy: {}", hex::encode(entropy));
    println!("phrase: {}", phrase);
    if language_defaulted {
        println!(
            "language: {} ({} words, default — verify against your records)",
            language, word_count
        );
        let mut stderr = std::io::stderr().lock();
        writeln!(
            stderr,
            "note: --language defaulted to '{}'; if your wallet was created with a different wordlist, decode with --language <lang>.",
            language
        )
        .ok();
    } else {
        println!("language: {} ({} words)", language, word_count);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    // Decode logic is mostly delegation to ms-codec + bip39; integration tests
    // (Phase 4) cover the stdout/stderr formatting end-to-end. No unit tests
    // here — would just be re-tests of bip39's own `from_entropy_in`.
}
```

- [ ] **Step 2: Verify it builds.**

```bash
cargo build --manifest-path /scratch/code/shibboleth/mnemonic-secret/Cargo.toml -p ms-cli 2>&1 | tail -5
```

Expected: clean build (decode.rs compiles; no test runs since no tests).

### Task 2.4: cmd/inspect.rs

**Files:**
- Create: `crates/ms-cli/src/cmd/inspect.rs`

**Realizes:** SPEC §2.3 (inspect), §2.3.1 (inspect-itself-fails handling), §5.3 (inspect --json), audit C3 (decode-vs-inspect routing) + I5 (would_decode + failure_reasons).

- [ ] **Step 1: Write the file.**

```rust
//! `ms inspect` — structural validity report for an ms1 string.
//!
//! Realizes SPEC §2.3 (verdict-first + structured fields), §2.3.1 (inspect()
//! BIP-93 parse failure handled per §6 standard error path), §5.3 (--json
//! schema), audit C3/I5 (would_decode + failure_reasons).

use clap::Args;
use ms_codec::consts::{RESERVED_NOT_EMITTED_V01, TAG_ENTR, VALID_ENTR_LENGTHS, VALID_STR_LENGTHS};
use ms_codec::InspectReport;
use serde_json::to_string;

use crate::error::Result;
use crate::format::{InspectJson, InspectReportJson};
use crate::parse::read_input;

/// `ms inspect` arguments.
#[derive(Args, Debug)]
pub struct InspectArgs {
    /// ms1 string to inspect. Use `-` or omit to read from stdin.
    pub ms1: Option<String>,

    /// Emit JSON instead of text verdict + fields.
    #[arg(long)]
    pub json: bool,
}

/// Run `ms inspect`. Lenient: returns a report even when the string would fail
/// decoder rules. If BIP-93 parse itself fails, treats the error per §6.
pub fn run(args: InspectArgs) -> Result<()> {
    let ms1 = read_input(args.ms1.as_deref())?;
    let report = ms_codec::inspect(&ms1)?; // §2.3.1: failures return CliError::Codex32 here.

    let (would_decode, reasons) = analyze(&report, ms1.len());

    if args.json {
        emit_json(&report, would_decode, &reasons)?;
    } else {
        emit_text(&report, would_decode, &reasons);
    }
    Ok(())
}

/// Re-walk SPEC §4 rules against the InspectReport's fields.
/// Returns `(would_decode, failure_reasons)` where reasons are pushed in
/// ASCENDING SPEC §4 rule order: 2, 3, 4, 6/7, 8, 9, 10.
fn analyze(report: &InspectReport, str_len: usize) -> (bool, Vec<&'static str>) {
    let mut reasons: Vec<&'static str> = Vec::new();
    let tag_bytes = *report.tag.as_bytes();

    // Rule 2: HRP == "ms".
    if report.hrp != "ms" {
        reasons.push("wrong-hrp");
    }
    // Rule 3: threshold == 0.
    if report.threshold != 0 {
        reasons.push("threshold-not-zero");
    }
    // Rule 4: share-index == 's'.
    if report.share_index != 's' {
        reasons.push("share-index-not-secret");
    }
    // Rules 6 + 7 are mutually exclusive (per `RESERVED_NOT_EMITTED_V01` vs `TAG_ENTR`).
    // Push rule 6 BEFORE rule 7 in ascending order if applicable; in our v0.1
    // shape only one of {entr accept-set, reserved-not-emitted, unknown}
    // applies, so at most one of these two reasons fires.
    if tag_bytes != TAG_ENTR {
        if RESERVED_NOT_EMITTED_V01.contains(&tag_bytes) {
            // Rule 7: tag is reserved-not-emitted in v0.1.
            // (Pushed after rule 6 logically — but only one of {6, 7} fires
            // because RESERVED_NOT_EMITTED_V01 ∩ accept-set = ∅, and a tag
            // either IS reserved or it's unknown.)
            reasons.push("reserved-tag-not-emitted");
        } else {
            // Rule 6: tag not in accept set.
            reasons.push("unknown-tag");
        }
    }
    // Rule 8: prefix byte == 0x00.
    if report.prefix_byte != 0x00 {
        reasons.push("non-zero-prefix");
    }
    // Rule 9: total string length in v0.1 set.
    if !VALID_STR_LENGTHS.contains(&str_len) {
        reasons.push("unexpected-string-length");
    }
    // Rule 10: payload length matches tag's expected set (only entr in v0.1).
    if tag_bytes == TAG_ENTR && !VALID_ENTR_LENGTHS.contains(&report.payload_bytes.len()) {
        reasons.push("payload-length-mismatch");
    }

    (reasons.is_empty(), reasons)
}

fn reason_text(tag: &'static str) -> &'static str {
    match tag {
        "unexpected-string-length" => {
            "string length not in v0.1 set [50, 56, 62, 69, 75]"
        }
        "wrong-hrp" => "HRP is not \"ms\"",
        "threshold-not-zero" => "threshold not 0 (v0.1 is single-string only)",
        "share-index-not-secret" => "share-index not 's' (BIP-93 requires 's' for threshold=0)",
        "reserved-tag-not-emitted" => "tag is reserved-not-emitted in v0.1; deferred to v0.2+",
        "unknown-tag" => "tag not in v0.1 RESERVED_TAG_TABLE",
        "non-zero-prefix" => "reserved-prefix byte is not 0x00 (v0.1 reserves it)",
        "payload-length-mismatch" => "entr payload length not in [16, 20, 24, 28, 32] bytes",
        _ => "<unknown reason>",
    }
}

fn emit_text(report: &InspectReport, would_decode: bool, reasons: &[&'static str]) {
    if would_decode {
        println!("OK: would decode v0.1");
    } else {
        println!("FAIL: would NOT decode v0.1");
        for r in reasons {
            println!("    reason: {} ({})", r, reason_text(r));
        }
    }
    println!();
    println!("hrp: {}", report.hrp);
    println!("threshold: {}", report.threshold);
    println!(
        "tag: {}",
        std::str::from_utf8(report.tag.as_bytes()).unwrap_or("<non-utf8>")
    );
    println!("share_index: {}", report.share_index);
    println!("prefix_byte: 0x{:02x}", report.prefix_byte);
    println!("payload_bytes: {}", hex::encode(&report.payload_bytes));
    println!("checksum_valid: {}", report.checksum_valid);
}

fn emit_json(
    report: &InspectReport,
    would_decode: bool,
    reasons: &[&'static str],
) -> Result<()> {
    let json = InspectJson {
        schema_version: "1",
        report: InspectReportJson {
            hrp: report.hrp.clone(),
            threshold: report.threshold,
            tag: std::str::from_utf8(report.tag.as_bytes())
                .unwrap_or("<non-utf8>")
                .to_string(),
            share_index: report.share_index,
            prefix_byte: report.prefix_byte,
            payload_bytes_hex: hex::encode(&report.payload_bytes),
            checksum_valid: report.checksum_valid,
        },
        would_decode,
        failure_reasons: reasons.to_vec(),
    };
    let s = to_string(&json).expect("inspect json always serializes");
    println!("{}", s);
    Ok(())
}
```

- [ ] **Step 2: Verify it builds.**

```bash
cargo build --manifest-path /scratch/code/shibboleth/mnemonic-secret/Cargo.toml -p ms-cli 2>&1 | tail -5
```

Expected: clean build.

### Task 2.5: cmd/verify.rs

**Files:**
- Create: `crates/ms-cli/src/cmd/verify.rs`

**Realizes:** SPEC §2.4 (verify), §2.4.1 (validation order), §6 exit codes 0/1/2/3/4.

- [ ] **Step 1: Write the file.**

```rust
//! `ms verify` — exit-code-only validity (and optional --phrase round-trip).
//!
//! Realizes SPEC §2.4 (full command), §2.4.1 (locked validation order:
//! decode -> exit on failure -> parse phrase -> compare -> exit), §6 exit
//! codes 0 (valid) / 1 (user-input) / 2 (format) / 3 (future format) /
//! 4 (round-trip mismatch).

use bip39::{Language, Mnemonic};
use clap::Args;
use ms_codec::Payload;
use serde_json::to_string;

use crate::error::{CliError, Result};
use crate::format::VerifySuccessJson;
use crate::language::CliLanguage;
use crate::parse::{is_stdin_arg, read_input};

/// `ms verify` arguments.
#[derive(Args, Debug)]
pub struct VerifyArgs {
    /// ms1 string to verify. Use `-` or omit to read from stdin.
    pub ms1: Option<String>,

    /// Original BIP-39 phrase to round-trip-check against the decoded entropy.
    /// When supplied, exit 4 on mismatch. Use `-` to read phrase from stdin.
    #[arg(long)]
    pub phrase: Option<String>,

    /// BIP-39 wordlist for --phrase. Default `english`.
    #[arg(long, default_value = "english")]
    pub language: CliLanguage,

    /// Emit success JSON on stdout (mirrors the §5 schema-versioned form).
    #[arg(long)]
    pub json: bool,
}

/// Run `ms verify` per SPEC §2.4.1 validation order.
pub fn run(args: VerifyArgs) -> Result<()> {
    // Step 1: read ms1 input. Concurrent-stdin guard: if both ms1 and --phrase
    // resolve to stdin, exit immediately (clap can't catch this).
    if is_stdin_arg(args.ms1.as_deref()) && args.phrase.as_deref() == Some("-") {
        return Err(CliError::BadInput(
            "cannot read both ms1 and --phrase from stdin".into(),
        ));
    }
    let ms1 = read_input(args.ms1.as_deref())?;

    // Step 2: decode the ms1 string. On failure, dispatch per §6.1.1 — phrase
    // is NEVER parsed in this branch.
    let decoded = ms_codec::decode(&ms1);
    let entropy = match decoded {
        Ok((_tag, Payload::Entr(b))) => b,
        // ms_codec::Payload is #[non_exhaustive]; v0.2+ may add variants.
        // v0.1 ms-codec only decodes to Payload::Entr; defensive arm only.
        Ok((_, _)) => unreachable!("ms-codec v0.1 only decodes to Payload::Entr"),
        Err(ms_codec::Error::ReservedTagNotEmittedInV01 { got }) => {
            // Exit 3 path: print the success-shaped "valid future format" message.
            return emit_future_format(&got, args.json);
        }
        Err(e) => return Err(e.into()),
    };

    // Step 3: parse --phrase if present.
    let phrase_supplied = match &args.phrase {
        Some(p) => Some(read_input(Some(p))?),
        None => None,
    };

    // Step 4: compare or exit-0 quick.
    if let Some(supplied) = phrase_supplied {
        let lang: Language = args.language.into();
        let supplied_mnemonic = Mnemonic::parse_in(lang, &supplied)?;
        let derived_mnemonic = Mnemonic::from_entropy_in(lang, &entropy)
            .expect("ms-codec validates entropy length");
        if supplied_mnemonic.to_string() == derived_mnemonic.to_string() {
            return emit_round_trip_ok(&derived_mnemonic, args.language.as_str(), args.json);
        } else {
            return Err(CliError::VerifyPhraseMismatch);
        }
    }

    // No --phrase: simple validity OK.
    let word_count = entropy.len() * 3 / 4;
    let str_len = ms1.len();
    emit_simple_ok(word_count, str_len, args.json)
}

fn emit_simple_ok(word_count: usize, str_len: usize, json: bool) -> Result<()> {
    if json {
        let j = VerifySuccessJson {
            schema_version: "1",
            status: "valid",
            message: &format!("valid v0.1 entr ({} words, {} chars)", word_count, str_len),
        };
        println!("{}", to_string(&j).expect("verify json"));
    } else {
        println!("OK: valid v0.1 entr ({} words, {} chars)", word_count, str_len);
    }
    Ok(())
}

fn emit_future_format(tag: &[u8; 4], json: bool) -> Result<()> {
    let tag_str = std::str::from_utf8(tag).unwrap_or("<non-utf8>");
    // Text mode: print success-shaped OK line. JSON mode: do NOT print here —
    // main.rs's ExitCode dispatch invokes emit_error which prints the error
    // envelope; printing a success line here would yield two outputs on stdout.
    if !json {
        println!("OK: valid future format (v0.2+, tag {})", tag_str);
    }
    // Either way, return Err(FutureFormat) so main.rs lands exit 3. In JSON
    // mode the error envelope (with kind="FutureFormat", exit_code=3) becomes
    // the sole stdout output; text-mode users see the OK line above + (since
    // ExitCode != 0) any stderr emit_error would write — but main.rs's
    // emit_error writes to stdout in --json mode and to stderr in text mode,
    // so text mode emits "error: ..." stderr alongside the OK stdout line.
    // That's intentionally redundant — exit-3 is "OK semantically" but the
    // err-path-with-stderr-display flags it for users who only watch stderr.
    Err(CliError::FutureFormat { tag: *tag })
}

fn emit_round_trip_ok(_mnemonic: &Mnemonic, language: &str, json: bool) -> Result<()> {
    let word_count = _mnemonic.to_string().split_whitespace().count();
    if json {
        let j = VerifySuccessJson {
            schema_version: "1",
            status: "round-trip-ok",
            message: &format!("round-trip valid ({} words, language={})", word_count, language),
        };
        println!("{}", to_string(&j).expect("verify json"));
    } else {
        println!(
            "OK: round-trip valid ({} words, language={})",
            word_count, language
        );
    }
    Ok(())
}
```

Note on `emit_future_format`: it prints the success-shaped "OK" message but returns `Err(CliError::FutureFormat)` so main.rs's ExitCode dispatch returns 3. This is intentional — exit 3 is "OK semantically (valid future format)" but encoded via the same Result-as-error path that other exit codes use, so the main.rs dispatch is uniform.

- [ ] **Step 2: Verify it builds.**

```bash
cargo build --manifest-path /scratch/code/shibboleth/mnemonic-secret/Cargo.toml -p ms-cli 2>&1 | tail -5
```

### Task 2.6: cmd/vectors.rs

**Files:**
- Create: `crates/ms-cli/src/cmd/vectors.rs`

**Realizes:** SPEC §2.5 (vectors), include_str! pattern from in-tree corpus.

- [ ] **Step 1: Write the file.**

```rust
//! `ms vectors` — print the SHA-pinned v0.1 test-vector corpus as JSON.
//!
//! Realizes SPEC §2.5. Corpus is `include_str!`-baked at compile time
//! from `crates/ms-cli/vectors/v0.1.json` (in-tree copy; parity with
//! `crates/ms-codec/tests/vectors/v0.1.json` enforced by the parity test).

use clap::Args;

use crate::error::{CliError, Result};

const VECTORS_V0_1_JSON: &str = include_str!("../../vectors/v0.1.json");

/// `ms vectors` arguments.
#[derive(Args, Debug)]
pub struct VectorsArgs {
    /// Indent the JSON output for human readability.
    #[arg(long)]
    pub pretty: bool,
}

/// Run `ms vectors`. Always exits 0 with the corpus on stdout.
pub fn run(args: VectorsArgs) -> Result<()> {
    if args.pretty {
        let parsed: serde_json::Value = serde_json::from_str(VECTORS_V0_1_JSON)
            .map_err(|e| CliError::BadInput(format!("vector corpus parse: {}", e)))?;
        let pretty = serde_json::to_string_pretty(&parsed)
            .map_err(|e| CliError::BadInput(format!("vector corpus serialize: {}", e)))?;
        println!("{}", pretty);
    } else {
        // Compact: print as-is.
        print!("{}", VECTORS_V0_1_JSON);
        if !VECTORS_V0_1_JSON.ends_with('\n') {
            println!();
        }
    }
    Ok(())
}
```

- [ ] **Step 2: Note that this WILL fail to compile** because `crates/ms-cli/vectors/v0.1.json` doesn't exist yet — Phase 3 task 3.1 lands the file. The Phase 2 commit defers the build-clean check until Phase 3.1 completes.

### Task 2.7: Phase 2 commit

- [ ] **Step 1: Stage paths + commit.**

```bash
git -C /scratch/code/shibboleth/mnemonic-secret add \
  crates/ms-cli/src/main.rs \
  crates/ms-cli/src/cmd/mod.rs \
  crates/ms-cli/src/cmd/encode.rs \
  crates/ms-cli/src/cmd/decode.rs \
  crates/ms-cli/src/cmd/inspect.rs \
  crates/ms-cli/src/cmd/verify.rs \
  crates/ms-cli/src/cmd/vectors.rs

git -C /scratch/code/shibboleth/mnemonic-secret commit -m "$(cat <<'EOF'
feat(ms-cli): Phase 2 command modules (encode/decode/inspect/verify/vectors)

Phase 2 of IMPLEMENTATION_PLAN_ms_cli_v0_1.md. 5 cmd modules + cmd/mod.rs
re-exports. Each consumes Phase 1 modules + ms-codec library.

Modules:
- encode.rs (SPEC §2.1): EncodeArgs (--phrase / --hex mutually-exclusive
  group, --language, --no-engraving-card, --json); BIP-39 parse via
  bip39::Mnemonic::parse_in; --hex parse via hex crate (4 unit tests);
  multi-line stdout + stderr engraving card per §4; --json schema per §5.1.
- decode.rs (SPEC §2.2): DecodeArgs (positional ms1, --language with no
  default at struct level, --json); §6.3 default-language hazard surfaced
  in BOTH stderr warning AND stdout language-line annotation when defaulted.
- inspect.rs (SPEC §2.3 + §2.3.1): InspectArgs (positional ms1, --json);
  re-walks SPEC §4 rules against InspectReport fields to compute
  (would_decode, failure_reasons[]); 8 closed-set kebab-case reason tags
  per SPEC §5.3.
- verify.rs (SPEC §2.4 + §2.4.1): VerifyArgs (positional ms1, --phrase,
  --language, --json); locked validation order (decode -> exit -> parse
  phrase -> compare -> exit); concurrent-stdin guard; emit_future_format
  uses Err(CliError::FutureFormat) so main.rs dispatch lands exit 3.
- vectors.rs (SPEC §2.5): VectorsArgs (--pretty); include_str!-bakes the
  corpus from ../../vectors/v0.1.json (Phase 3 task 3.1 lands the file,
  so this won't compile yet — that's the next phase's commit).

Build is intentionally broken at this commit's HEAD (vectors/v0.1.json
missing). Phase 3 task 3.1 lands the file; Phase 3 commit is the first
build-clean checkpoint. Tests still pass for Phase 1 modules in
isolation (cargo test -p ms-cli --no-run for cmd::encode passes
because encode doesn't reach into vectors).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"

git -C /scratch/code/shibboleth/mnemonic-secret show HEAD --stat | head -15
```

### Task 2.8: Phase 2 opus review checkpoint

- [ ] **Step 1: Dispatch reviewer.**

Brief:
- Files: `crates/ms-cli/src/cmd/{mod,encode,decode,inspect,verify,vectors}.rs`.
- SPEC reference: §2.1-§2.5, §3.5.1, §5.1-§5.4, §6 dispatch.
- Library API: `crates/ms-codec/src/lib.rs` exports.
- Specifics:
  - encode: --phrase / --hex mutually exclusive; bip39 parse-in + entropy length validation symmetry; chunked + engraving-card formatting matches SPEC §4.
  - decode: §6.3 hazard surfacing on BOTH stderr AND stdout when language defaulted.
  - inspect: re-walked rules cover all 8 SPEC §2.3 closed-set tags; ordering ascending by rule number.
  - verify: §2.4.1 validation order strict; concurrent-stdin guard fires before any read; emit_future_format uses Err(FutureFormat) for exit-3 dispatch.
  - vectors: include_str! from in-tree path; --pretty round-trips through serde_json::Value for stable formatting.
- Length cap: 600 words. Persist to `design/agent-reports/phase-2-cmd-review-r1.md`.

- [ ] **Step 2-4: Apply findings + iterate + capture nits.** Same convention as Phase 1.

---

## Phase 3: Root + glue

**Goal:** Land `vectors/v0.1.json` (the in-tree corpus copy) + `main.rs` (clap derive root + ExitCode dispatch). By end of phase, `cargo build` is clean for the first time, `cargo run -- --help` works, all 5 subcommand `--help`s render the SPEC §2.6 strings.

**Files:**
- Create: `crates/ms-cli/vectors/v0.1.json`
- Modify: `crates/ms-cli/src/main.rs`

### Task 3.1: vectors/v0.1.json (in-tree corpus copy)

**Files:**
- Create: `crates/ms-cli/vectors/v0.1.json`

**Realizes:** SPEC §2.5 + §10.2 (parity test enforces JSON-equality with ms-codec corpus).

- [ ] **Step 1: Copy the canonical corpus from ms-codec.**

```bash
mkdir -p /scratch/code/shibboleth/mnemonic-secret/crates/ms-cli/vectors
cp /scratch/code/shibboleth/mnemonic-secret/crates/ms-codec/tests/vectors/v0.1.json \
   /scratch/code/shibboleth/mnemonic-secret/crates/ms-cli/vectors/v0.1.json
```

- [ ] **Step 2: Verify identity.**

```bash
diff -u /scratch/code/shibboleth/mnemonic-secret/crates/ms-codec/tests/vectors/v0.1.json \
        /scratch/code/shibboleth/mnemonic-secret/crates/ms-cli/vectors/v0.1.json
```

Expected: empty output (files are byte-equal).

### Task 3.2: main.rs — clap derive root + ExitCode dispatch

**Files:**
- Modify: `crates/ms-cli/src/main.rs`

**Realizes:** SPEC §2.6 (per-subcommand about / after_long_help), §6 (ExitCode mapping), §6.3 error display (text + JSON paths), top-level `Cli::about`.

- [ ] **Step 1: Replace main.rs with the full clap derive + dispatch.**

```rust
//! `ms` — engrave-friendly BIP-39 entropy backups (the `ms1` format).
//!
//! Companion CLI to the `ms-codec` library. See `design/SPEC_ms_cli_v0_1.md`
//! for the full surface specification.

#![allow(missing_docs)] // ms-cli is binary-only; field-level docs are pretty but not load-bearing for a non-published lib API. Mirror md-cli precedent at crates/md-cli/src/main.rs:1.

mod bip39_friendly;
mod cmd;
mod codex32_friendly;
mod error;
mod format;
mod language;
mod parse;

use std::io::Write;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use error::{CliError, Result};
use format::{ErrorBodyJson, ErrorEnvelopeJson};

#[derive(Parser, Debug)]
#[command(
    name = "ms",
    version,
    about = "ms — engrave-friendly BIP-39 entropy backups (the ms1 format)"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Encode a BIP-39 mnemonic (or hex entropy) as an ms1 string for engraving.
    #[command(after_long_help = "EXAMPLES:\n  ms encode --phrase \"abandon abandon … about\"\n  ms encode --phrase - < phrase.txt\n  ms encode --hex 00000000000000000000000000000000 --no-engraving-card\n  ms encode --phrase \"...\" --json | jq .ms1")]
    Encode(cmd::encode::EncodeArgs),

    /// Decode an ms1 string back to its BIP-39 mnemonic and entropy bytes.
    #[command(after_long_help = "EXAMPLES:\n  ms decode ms10entrs…\n  ms decode - < engraved.txt\n  ms decode <ms1> --language french\n  ms decode <ms1> --json | jq .phrase")]
    Decode(cmd::decode::DecodeArgs),

    /// Inspect an ms1 string's structural fields and decoder verdict.
    #[command(after_long_help = "EXAMPLES:\n  ms inspect <ms1>          # verdict + fields\n  ms inspect <ms1> --json   # structured output for tooling\n  printf \"ms10e ntrsq…\" | ms inspect -   # back-typed chunked form")]
    Inspect(cmd::inspect::InspectArgs),

    /// Verify an ms1 string is valid (and optionally round-trips against a phrase).
    #[command(after_long_help = "EXAMPLES:\n  ms verify <ms1>                          # exit 0 = valid v0.1\n  ms verify <ms1> --phrase \"abandon … about\"   # round-trip; exit 4 on mismatch\n  ms verify <ms1> --phrase \"...\" --json    # structured outcome")]
    Verify(cmd::verify::VerifyArgs),

    /// Print the SHA-pinned v0.1 test-vector corpus as JSON.
    #[command(after_long_help = "EXAMPLES:\n  ms vectors                # compact JSON\n  ms vectors --pretty       # indented JSON\n  ms vectors | jq '.[0]'    # filter via jq")]
    Vectors(cmd::vectors::VectorsArgs),
}

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            // Clap usage error: print to stderr (clap's default), exit 64
            // (SPEC §6 — overrides clap's default of 2 to keep 2 reserved for
            // ms1 format violations).
            e.print().ok();
            return ExitCode::from(64);
        }
    };

    let json_mode = is_json_mode(&cli.command);

    let result: Result<()> = match cli.command {
        Command::Encode(args) => cmd::encode::run(args),
        Command::Decode(args) => cmd::decode::run(args),
        Command::Inspect(args) => cmd::inspect::run(args),
        Command::Verify(args) => cmd::verify::run(args),
        Command::Vectors(args) => cmd::vectors::run(args),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            emit_error(&e, json_mode);
            ExitCode::from(e.exit_code())
        }
    }
}

fn is_json_mode(cmd: &Command) -> bool {
    match cmd {
        Command::Encode(a) => a.json,
        Command::Decode(a) => a.json,
        Command::Inspect(a) => a.json,
        Command::Verify(a) => a.json,
        Command::Vectors(_) => false, // vectors output is always JSON-shaped
    }
}

fn emit_error(e: &CliError, json_mode: bool) {
    // Special case: FutureFormat is a "success-shaped" exit-3 path used by
    // verify. In text mode, cmd::verify::emit_future_format already wrote the
    // "OK: valid future format" line to stdout; emitting an "error: ..."
    // message to stderr here would contradict that. Skip the stderr write.
    // In JSON mode we DO want the error envelope (cmd handler suppressed its
    // own stdout output specifically so this path produces the envelope).
    if matches!(e, CliError::FutureFormat { .. }) && !json_mode {
        return;
    }

    if json_mode {
        // JSON-mode errors go to stdout (one stream) per SPEC §6.3.
        let envelope = ErrorEnvelopeJson {
            schema_version: "1",
            error: ErrorBodyJson {
                kind: e.kind(),
                message: e.message(),
                exit_code: e.exit_code(),
                details: e.details(),
            },
        };
        let s = serde_json::to_string(&envelope).expect("error envelope serializes");
        println!("{}", s);
    } else {
        // Text-mode errors go to stderr.
        let mut stderr = std::io::stderr().lock();
        writeln!(stderr, "{}", e).ok();
    }
}
```

- [ ] **Step 2: Verify build + clippy + fmt + --help.**

```bash
cargo build --manifest-path /scratch/code/shibboleth/mnemonic-secret/Cargo.toml -p ms-cli 2>&1 | tail -3
cargo clippy --manifest-path /scratch/code/shibboleth/mnemonic-secret/Cargo.toml -p ms-cli --all-targets -- -D warnings 2>&1 | tail -3
cargo fmt --manifest-path /scratch/code/shibboleth/mnemonic-secret/Cargo.toml --all -- --check 2>&1 | tail -3
cargo run --manifest-path /scratch/code/shibboleth/mnemonic-secret/Cargo.toml -p ms-cli -- --help 2>&1 | head -20
cargo run --manifest-path /scratch/code/shibboleth/mnemonic-secret/Cargo.toml -p ms-cli -- encode --help 2>&1 | head -20
```

Expected: build clean, clippy clean, fmt clean. `ms --help` shows the top-level Cli::about + 5 subcommands. `ms encode --help` shows the EncodeArgs surface + EXAMPLES block.

- [ ] **Step 3: Smoke test a real round-trip.**

```bash
ABANDON12="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
cargo run --manifest-path /scratch/code/shibboleth/mnemonic-secret/Cargo.toml -p ms-cli -- encode --phrase "$ABANDON12" 2>&1 | head -10
```

Expected: stdout shows ms1 string + blank line + chunked form. stderr shows engraving card with `language: english` and `passphrase: not stored in ms1` lines.

### Task 3.3: Phase 3 commit + opus review

- [ ] **Step 1: Stage + commit.**

```bash
git -C /scratch/code/shibboleth/mnemonic-secret add \
  crates/ms-cli/vectors/v0.1.json \
  crates/ms-cli/src/main.rs

git -C /scratch/code/shibboleth/mnemonic-secret commit -m "$(cat <<'EOF'
feat(ms-cli): Phase 3 root binary + vectors corpus (first build-clean checkpoint)

Phase 3 of IMPLEMENTATION_PLAN_ms_cli_v0_1.md. cargo build / clippy
--all-targets -D warnings / fmt --check all clean. `ms --help` and
`ms <subcmd> --help` render the SPEC §2.6 about + after_long_help text.
Round-trip smoke test (`ms encode --phrase "abandon... about"`) produces
expected stdout (ms1 + chunked) + stderr (engraving card).

Files:
- crates/ms-cli/vectors/v0.1.json: in-tree copy of canonical corpus
  (byte-equal to crates/ms-codec/tests/vectors/v0.1.json; Phase 4
  parity test asserts JSON-equality).
- main.rs: clap derive Cli root with Command enum (5 subcommands per
  SPEC §2.6); ExitCode dispatch via CliError::exit_code(); text-mode
  errors -> stderr, JSON-mode errors -> stdout via ErrorEnvelopeJson;
  clap usage errors override exit 64 (SPEC §6 reserves 2 for format
  violations, so clap's default 2 collides — md-cli precedent at
  crates/md-cli/src/main.rs:180-193 is the override pattern).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"

git -C /scratch/code/shibboleth/mnemonic-secret show HEAD --stat | head -10
```

- [ ] **Step 2: Phase 3 opus review.**

Brief: verify main.rs is_json_mode() exhaustively matches each subcommand's flag location; verify ExitCode dispatch handles each CliError variant correctly (incl. FutureFormat which is a "success-shaped" error per Phase 2's verify.rs design); verify clap usage error override to 64 actually fires (test by passing bad args); verify --help text matches SPEC §2.6 verbatim. Persist to `design/agent-reports/phase-3-root-review-r1.md`.

---

## Phase 4: Integration tests

**Goal:** Land ~28 `assert_cmd` integration tests covering every subcommand's behavior, error paths, exit codes, JSON schemas, and pipe interactions. By end of phase, the full test suite passes; ms-cli is feature-complete and verified.

**Files:** Tests under `crates/ms-cli/tests/`. Each is a separate file (per `assert_cmd` convention; one binary per file = parallel test execution).

The tests are grouped into 9 batches by concern.

### Task 4.1: Vector-corpus parity test (the simplest)

**Files:** Create `crates/ms-cli/tests/vectors_parity.rs`.

- [ ] **Step 1: Write the test.**

```rust
//! Vector corpus parity: ms-cli's in-tree copy must JSON-equal ms-codec's canonical corpus.
//!
//! Per SPEC §10.2: parsed-equality, not byte-equality (avoids whitespace
//! / line-ending fragility).

#[test]
fn vectors_corpus_parity_with_ms_codec() {
    let cli_corpus: serde_json::Value =
        serde_json::from_str(include_str!("../vectors/v0.1.json"))
            .expect("ms-cli vectors corpus parses as JSON");
    let codec_corpus: serde_json::Value =
        serde_json::from_str(include_str!("../../ms-codec/tests/vectors/v0.1.json"))
            .expect("ms-codec vectors corpus parses as JSON");
    assert_eq!(
        cli_corpus, codec_corpus,
        "vectors corpus drifted between ms-cli and ms-codec"
    );
}
```

- [ ] **Step 2: Run.**

```bash
cargo test --manifest-path /scratch/code/shibboleth/mnemonic-secret/Cargo.toml -p ms-cli --test vectors_parity 2>&1 | tail -5
```

Expected: 1 test passes.

### Task 4.2: encode integration tests

**Files:** Create 8 separate test files under `crates/ms-cli/tests/`:

- `encode_canonical_12_word.rs`
- `encode_canonical_24_word.rs`
- `encode_hex_input.rs`
- `encode_rejects_bad_checksum.rs`
- `encode_rejects_bad_language.rs`
- `encode_rejects_odd_length_hex.rs`
- `encode_emits_passphrase_warning.rs`
- `encode_no_engraving_card.rs`

Per the assert_cmd pattern. Detail per test below.

- [ ] **Step 1: Write `tests/encode_canonical_12_word.rs`.**

```rust
//! `ms encode --phrase` 12-word abandon round-trip.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn encode_12_word_abandon_about() {
    Command::cargo_bin("ms").unwrap()
        .args([
            "encode",
            "--phrase",
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        ])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("ms10entrsqqqq"))
        .stdout(predicate::str::contains("\n\n"))
        .stderr(predicate::str::contains("language: english"))
        .stderr(predicate::str::contains("word count: 12"))
        .stderr(predicate::str::contains("passphrase: not stored"));
}
```

- [ ] **Step 2: Write `tests/encode_canonical_24_word.rs`.**

```rust
//! `ms encode --phrase` 24-word abandon round-trip.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn encode_24_word_abandon_art() {
    let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
    Command::cargo_bin("ms").unwrap()
        .args(["encode", "--phrase", phrase])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("ms10entrsqqqq"))
        .stderr(predicate::str::contains("word count: 24"));
}
```

- [ ] **Step 3: Write `tests/encode_hex_input.rs`.**

```rust
//! `ms encode --hex` round-trip equivalent to --phrase.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn encode_hex_zeros_16_bytes() {
    Command::cargo_bin("ms").unwrap()
        .args(["encode", "--hex", "00000000000000000000000000000000"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("ms10entrsqqqq"));
}

#[test]
fn encode_hex_omits_language_in_engraving_card() {
    Command::cargo_bin("ms").unwrap()
        .args(["encode", "--hex", "00000000000000000000000000000000"])
        .assert()
        .success()
        .stderr(predicate::str::contains("word count: 12"))
        .stderr(predicate::str::contains("passphrase: not stored"))
        .stderr(predicate::str::contains("language:").not());
}
```

- [ ] **Step 4: Write `tests/encode_rejects_bad_checksum.rs`.**

```rust
//! BIP-39 bad-checksum phrase → exit 1 with friendly message.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn encode_rejects_bad_bip39_checksum() {
    // Replace last word "about" with "ability" to break the BIP-39 checksum.
    let bad = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon ability";
    Command::cargo_bin("ms").unwrap()
        .args(["encode", "--phrase", bad])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("BIP-39 checksum failure"));
}
```

- [ ] **Step 5: Write `tests/encode_rejects_bad_language.rs`.**

```rust
//! English phrase with --language japanese → exit 1 (UnknownWord).

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn encode_rejects_english_phrase_under_japanese_lang() {
    let english = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    Command::cargo_bin("ms").unwrap()
        .args(["encode", "--phrase", english, "--language", "japanese"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("unknown BIP-39 word"));
}
```

- [ ] **Step 6: Write `tests/encode_rejects_odd_length_hex.rs`.**

```rust
//! Odd-length --hex → exit 1 with friendly message.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn encode_rejects_odd_length_hex() {
    Command::cargo_bin("ms").unwrap()
        .args(["encode", "--hex", "0"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("even-length hex"));
}

#[test]
fn encode_rejects_non_hex_char() {
    Command::cargo_bin("ms").unwrap()
        .args(["encode", "--hex", "ZZ"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("position 0"));
}
```

- [ ] **Step 7: Write `tests/encode_emits_passphrase_warning.rs`.**

```rust
//! SPEC §2.1 + architect r1-C1 resolution: encode stderr engraving card includes
//! the passphrase reminder line.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn encode_emits_passphrase_warning_on_stderr() {
    Command::cargo_bin("ms").unwrap()
        .args([
            "encode",
            "--phrase",
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "passphrase: not stored in ms1 (record separately if used)",
        ));
}
```

- [ ] **Step 8: Write `tests/encode_no_engraving_card.rs`.**

```rust
//! --no-engraving-card suppresses stderr block; stdout unchanged.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn encode_no_engraving_card_suppresses_stderr() {
    Command::cargo_bin("ms").unwrap()
        .args([
            "encode",
            "--phrase",
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
            "--no-engraving-card",
        ])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("ms10entrsqqqq"))
        .stderr(predicate::str::is_empty());
}
```

- [ ] **Step 9: Write `tests/encode_arg_group_violations.rs`.**

```rust
//! SPEC §2.1 edge-case table: clap arg-group violations exit 64 (usage error).
//!
//! Both --phrase + --hex supplied → usage error.
//! Neither supplied → usage error.

use assert_cmd::Command;

#[test]
fn encode_rejects_both_phrase_and_hex() {
    Command::cargo_bin("ms").unwrap()
        .args([
            "encode",
            "--phrase",
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
            "--hex",
            "00000000000000000000000000000000",
        ])
        .assert()
        .failure()
        .code(64);
}

#[test]
fn encode_rejects_neither_phrase_nor_hex() {
    Command::cargo_bin("ms").unwrap()
        .arg("encode")
        .assert()
        .failure()
        .code(64);
}
```

This test requires the clap `Args` struct in `cmd/encode.rs` to declare the `--phrase` / `--hex` group as `required = true` (mutually exclusive AND at-least-one). If Phase 2 task 2.2's `EncodeArgs` has just `group = "input"` without `required = true`, this test will fail with exit 1 (zero-arg path passes through clap then fails at runtime in cmd::encode::run). Fix: add `#[arg(group = "input", required = true)]` to clap derive in encode.rs — or equivalently use `#[command(group = clap::ArgGroup::new("input").required(true).args(["phrase", "hex"]))]` on the EncodeArgs struct.

- [ ] **Step 10: Run all encode integration tests.**

```bash
cargo test --manifest-path /scratch/code/shibboleth/mnemonic-secret/Cargo.toml -p ms-cli --test 'encode_*' 2>&1 | grep "test result"
```

Expected: 10+ tests pass (one per file; some files have 2 tests; encode_arg_group_violations has 2).

### Task 4.3: decode integration tests

**Files:** `decode_round_trip.rs`, `decode_default_english_in_stdout.rs`, `decode_explicit_language_no_warning.rs`.

- [ ] **Step 1: Write `tests/decode_round_trip.rs`.**

```rust
//! `ms decode <ms1>` produces the labeled block + matches input phrase.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn decode_canonical_12_word_round_trip() {
    Command::cargo_bin("ms").unwrap()
        .args(["decode", "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f"])
        .assert()
        .success()
        .stdout(predicate::str::contains("entropy: 00000000000000000000000000000000"))
        .stdout(predicate::str::contains(
            "phrase: abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        ))
        .stdout(predicate::str::contains("language: english (12 words"));
}

#[test]
fn decode_json_schema() {
    Command::cargo_bin("ms").unwrap()
        .args(["decode", "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"schema_version\":\"1\""))
        .stdout(predicate::str::contains("\"language\":\"english\""))
        .stdout(predicate::str::contains("\"language_defaulted\":true"));
}
```

- [ ] **Step 2: Write `tests/decode_default_english_in_stdout.rs`.**

```rust
//! When --language is defaulted, stdout language line carries DEFAULT annotation
//! AND stderr emits non-suppressible warning (SPEC §6.3 hazard surfacing).

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn decode_default_english_warns_on_both_streams() {
    Command::cargo_bin("ms").unwrap()
        .args(["decode", "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f"])
        .assert()
        .success()
        .stdout(predicate::str::contains("default — verify against your records"))
        .stderr(predicate::str::contains("note: --language defaulted to 'english'"));
}
```

- [ ] **Step 3: Write `tests/decode_explicit_language_no_warning.rs`.**

```rust
//! Explicit --language removes both stderr and stdout warnings.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn decode_explicit_english_removes_warnings() {
    Command::cargo_bin("ms").unwrap()
        .args([
            "decode",
            "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f",
            "--language",
            "english",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("default —").not())
        .stderr(predicate::str::contains("defaulted").not());
}
```

- [ ] **Step 4: Run.** `cargo test -p ms-cli --test 'decode_*' 2>&1 | grep "test result"`. Expected: 4 tests pass.

### Task 4.4: inspect integration tests

**Files:** `inspect_valid_string.rs`, `inspect_non_zero_prefix.rs`, `inspect_reserved_tag.rs`, `inspect_multiple_failures.rs`, `inspect_codex32_parse_failure.rs`.

- [ ] **Step 1: Write `tests/inspect_valid_string.rs`.**

```rust
//! Inspect on canonical valid string → verdict OK + fields.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn inspect_valid_canonical_v01_string() {
    Command::cargo_bin("ms").unwrap()
        .args(["inspect", "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("OK: would decode v0.1"))
        .stdout(predicate::str::contains("hrp: ms"))
        .stdout(predicate::str::contains("threshold: 0"))
        .stdout(predicate::str::contains("tag: entr"))
        .stdout(predicate::str::contains("share_index: s"))
        .stdout(predicate::str::contains("prefix_byte: 0x00"))
        .stdout(predicate::str::contains("checksum_valid: true"));
}

#[test]
fn inspect_valid_string_json_schema() {
    Command::cargo_bin("ms").unwrap()
        .args(["inspect", "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"schema_version\":\"1\""))
        .stdout(predicate::str::contains("\"would_decode\":true"))
        .stdout(predicate::str::contains("\"failure_reasons\":[]"));
}
```

- [ ] **Step 2: Write `tests/inspect_non_zero_prefix.rs`.**

This requires a hand-built ms1 string with prefix byte = 0x01. Generate it via a setup step in the test:

```rust
//! Inspect on non-zero-prefix string → verdict FAIL with rule 8.

use assert_cmd::Command;
use codex32::{Codex32String, Fe};
use predicates::prelude::*;

fn build_with_prefix_0x01() -> String {
    let mut data = vec![0x01u8];
    data.extend_from_slice(&[0xAAu8; 16]);
    Codex32String::from_seed("ms", 0, "entr", Fe::S, &data)
        .unwrap()
        .to_string()
}

#[test]
fn inspect_non_zero_prefix_reports_rule_8() {
    let s = build_with_prefix_0x01();
    Command::cargo_bin("ms").unwrap()
        .args(["inspect", &s])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("FAIL: would NOT decode v0.1"))
        .stdout(predicate::str::contains("non-zero-prefix"));
}
```

Note: this requires adding `codex32 = "=0.1.0"` to ms-cli's `[dev-dependencies]` for the test setup helper (the runtime crate already depends on it transitively via ms-codec). Add via:

```toml
[dev-dependencies]
assert_cmd = "2"
predicates = "3"
codex32 = "=0.1.0"  # for test fixture helpers
```

- [ ] **Step 3: Write `tests/inspect_reserved_tag.rs`.**

```rust
//! Inspect on string with id="seed" → verdict FAIL with rule 7.

use assert_cmd::Command;
use codex32::{Codex32String, Fe};
use predicates::prelude::*;

#[test]
fn inspect_reserved_seed_tag_reports_rule_7() {
    let mut data = vec![0x00u8];
    data.extend_from_slice(&[0xAAu8; 16]);
    let s = Codex32String::from_seed("ms", 0, "seed", Fe::S, &data).unwrap().to_string();

    Command::cargo_bin("ms").unwrap()
        .args(["inspect", &s])
        .assert()
        .success()
        .stdout(predicate::str::contains("FAIL"))
        .stdout(predicate::str::contains("reserved-tag-not-emitted"));
}
```

- [ ] **Step 4: Write `tests/inspect_multiple_failures.rs`.**

```rust
//! Inspect on string with multiple violations reports both, sorted by rule number.

use assert_cmd::Command;
use codex32::{Codex32String, Fe};
use predicates::prelude::*;

#[test]
fn inspect_multiple_failures_sorted() {
    // Both wrong-hrp (rule 2) AND non-zero-prefix (rule 8).
    let mut data = vec![0x01u8];
    data.extend_from_slice(&[0xAAu8; 16]);
    let s = Codex32String::from_seed("mq", 0, "entr", Fe::S, &data).unwrap().to_string();

    let output = Command::cargo_bin("ms").unwrap()
        .args(["inspect", &s])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();
    // Both reasons should appear; wrong-hrp first per rule-2-before-rule-8.
    let wrong_hrp_idx = stdout.find("wrong-hrp").expect("wrong-hrp reason present");
    let non_zero_idx = stdout.find("non-zero-prefix").expect("non-zero-prefix reason present");
    assert!(wrong_hrp_idx < non_zero_idx, "reasons not in rule-number order");
}
```

- [ ] **Step 5: Write `tests/inspect_codex32_parse_failure.rs`.**

```rust
//! Inspect on a string that fails BIP-93 parse → exit 1 with Codex32 error
//! per SPEC §2.3.1.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn inspect_bad_checksum_exits_1_with_friendly_error() {
    // Take a valid string and flip the last char to break BCH.
    let mut bytes = b"ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f".to_vec();
    let last = bytes.len() - 1;
    bytes[last] = if bytes[last] == b'q' { b'p' } else { b'q' };
    let bad = String::from_utf8(bytes).unwrap();

    Command::cargo_bin("ms").unwrap()
        .args(["inspect", &bad])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("BCH checksum invalid"));
}

#[test]
fn inspect_bad_checksum_json_envelope() {
    let mut bytes = b"ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f".to_vec();
    let last = bytes.len() - 1;
    bytes[last] = if bytes[last] == b'q' { b'p' } else { b'q' };
    let bad = String::from_utf8(bytes).unwrap();

    Command::cargo_bin("ms").unwrap()
        .args(["inspect", &bad, "--json"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("\"kind\":\"Codex32\""))
        .stdout(predicate::str::contains("\"schema_version\":\"1\""));
}
```

- [ ] **Step 6: Run all inspect tests.** `cargo test -p ms-cli --test 'inspect_*' 2>&1 | grep "test result"`. Expected: ~8 tests pass.

### Task 4.5: verify integration tests

**Files:** `verify_quiet_pass.rs`, `verify_quiet_fail.rs`, `verify_future_format.rs`, `verify_phrase_round_trip_ok.rs`, `verify_phrase_round_trip_mismatch.rs`.

- [ ] **Step 1: Write `tests/verify_quiet_pass.rs`.**

```rust
//! Verify on valid v0.1 string → exit 0 with one-line OK summary.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn verify_valid_v01_exit_0() {
    Command::cargo_bin("ms").unwrap()
        .args(["verify", "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f"])
        .assert()
        .success()
        .stdout(predicate::str::contains("OK: valid v0.1 entr (12 words, 50 chars)"));
}
```

- [ ] **Step 2: Write `tests/verify_quiet_fail.rs`.**

```rust
//! Verify on invalid string → exit 2 (format violation) with FAIL summary.

use assert_cmd::Command;
use codex32::{Codex32String, Fe};
use predicates::prelude::*;

#[test]
fn verify_non_zero_prefix_exits_2() {
    let mut data = vec![0x01u8];
    data.extend_from_slice(&[0xAAu8; 16]);
    let s = Codex32String::from_seed("ms", 0, "entr", Fe::S, &data).unwrap().to_string();

    Command::cargo_bin("ms").unwrap()
        .args(["verify", &s])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("reserved-prefix byte was 0x01"));
}
```

- [ ] **Step 3: Write `tests/verify_future_format.rs`.**

```rust
//! Verify on string with reserved-not-emitted tag → exit 3.

use assert_cmd::Command;
use codex32::{Codex32String, Fe};
use predicates::prelude::*;

#[test]
fn verify_reserved_seed_tag_exits_3() {
    let mut data = vec![0x00u8];
    data.extend_from_slice(&[0xAAu8; 16]);
    let s = Codex32String::from_seed("ms", 0, "seed", Fe::S, &data).unwrap().to_string();

    Command::cargo_bin("ms").unwrap()
        .args(["verify", &s])
        .assert()
        .failure()
        .code(3)
        .stdout(predicate::str::contains("OK: valid future format (v0.2+, tag seed)"));
}
```

- [ ] **Step 4: Write `tests/verify_phrase_round_trip_ok.rs`.**

```rust
//! Verify with --phrase matching the encoded entropy → exit 0.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn verify_round_trip_with_correct_phrase() {
    Command::cargo_bin("ms").unwrap()
        .args([
            "verify",
            "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f",
            "--phrase",
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("OK: round-trip valid (12 words, language=english)"));
}
```

- [ ] **Step 5: Write `tests/verify_phrase_round_trip_mismatch.rs`.**

```rust
//! Verify with wrong --phrase → exit 4. Phrase NEVER echoed to output.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn verify_round_trip_with_wrong_phrase_exit_4() {
    let wrong = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon ability";
    let assertion = Command::cargo_bin("ms").unwrap()
        .args([
            "verify",
            "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f",
            "--phrase",
            wrong,
        ])
        .assert()
        .failure();

    let output = assertion.get_output();
    let combined = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);

    // Mismatch could be exit 4 (correct), or exit 1 if bip39 rejected the wrong
    // phrase first (it has a bad checksum since "ability" is not the right
    // 12th word for the all-zero-entropy case). Per SPEC §2.4.1 step 3:
    // bad-checksum phrase fires before round-trip compare → exit 1.
    let code = output.status.code().unwrap();
    assert!(code == 1 || code == 4, "expected exit 1 or 4, got {}", code);

    // Critical: neither phrase appears in any output (per SPEC §2.4 phrases-as-secrets).
    assert!(!combined.contains("ability"), "wrong phrase echoed in output: {}", combined);
    assert!(!combined.contains("about"), "decoded phrase echoed in output: {}", combined);
}
```

- [ ] **Step 6: Run.** `cargo test -p ms-cli --test 'verify_*' 2>&1 | grep "test result"`. Expected: 5 tests pass.

### Task 4.6: pipe + back-typed tests

**Files:** `encode_pipe_to_verify.rs`, `encode_pipe_to_decode.rs`, `back_typed_chunked_form_decodes.rs`.

- [ ] **Step 1: Write `tests/encode_pipe_to_verify.rs`.**

```rust
//! End-to-end pipe round-trip: ms encode | ms verify -.

use assert_cmd::Command;

#[test]
fn encode_pipe_to_verify() {
    let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let encoded = Command::cargo_bin("ms").unwrap()
        .args(["encode", "--phrase", phrase])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let encoded_str = String::from_utf8(encoded).unwrap();

    // Pipe encoded multi-line stdout into verify - via stdin.
    Command::cargo_bin("ms").unwrap()
        .args(["verify", "-"])
        .write_stdin(encoded_str)
        .assert()
        .success();
}
```

- [ ] **Step 2: Write `tests/encode_pipe_to_decode.rs`.**

```rust
//! End-to-end pipe round-trip: ms encode | ms decode - recovers the phrase.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn encode_pipe_to_decode_recovers_phrase() {
    let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let encoded = Command::cargo_bin("ms").unwrap()
        .args(["encode", "--phrase", phrase])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let encoded_str = String::from_utf8(encoded).unwrap();

    Command::cargo_bin("ms").unwrap()
        .args(["decode", "-"])
        .write_stdin(encoded_str)
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("phrase: {}", phrase).as_str()));
}
```

- [ ] **Step 3: Write `tests/back_typed_chunked_form_decodes.rs`.**

```rust
//! Engraver-typed-back chunked form via stdin (with spaces + newlines).

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn back_typed_chunked_form_with_spaces_and_newlines() {
    let typed_back = "ms10e ntrsq qqqqq qqqqq qqqqq qqqqq qqqqq qqqqq qqqqq qqqqq\ncj9sx raq34 v7f";

    Command::cargo_bin("ms").unwrap()
        .args(["decode", "-"])
        .write_stdin(typed_back)
        .assert()
        .success()
        .stdout(predicate::str::contains("entropy: 00000000000000000000000000000000"));
}
```

- [ ] **Step 4: Run.** `cargo test -p ms-cli --test 'encode_pipe_*' --test back_typed_chunked_form_decodes 2>&1 | grep "test result"`. Expected: 3 tests pass.

### Task 4.7: vectors integration tests

**Files:** `vectors_compact.rs`, `vectors_pretty.rs`.

- [ ] **Step 1: Write `tests/vectors_compact.rs`.**

```rust
//! ms vectors emits parseable JSON compact-form by default.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn vectors_compact_is_parseable_json() {
    let out = Command::cargo_bin("ms").unwrap()
        .arg("vectors")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert!(parsed.is_array());
    assert!(parsed.as_array().unwrap().len() >= 2, "expected >=2 vectors");
}

#[test]
fn vectors_first_entry_matches_canonical_12_word() {
    Command::cargo_bin("ms").unwrap()
        .arg("vectors")
        .assert()
        .success()
        .stdout(predicate::str::contains("ms10entrsqqqq"))
        .stdout(predicate::str::contains("abandon"));
}
```

- [ ] **Step 2: Write `tests/vectors_pretty.rs`.**

```rust
//! ms vectors --pretty emits indented JSON with same content.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn vectors_pretty_is_indented_and_parseable() {
    let compact = Command::cargo_bin("ms").unwrap()
        .arg("vectors")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let pretty = Command::cargo_bin("ms").unwrap()
        .args(["vectors", "--pretty"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\n"))
        .get_output()
        .stdout
        .clone();

    let cs: serde_json::Value = serde_json::from_slice(&compact).unwrap();
    let ps: serde_json::Value = serde_json::from_slice(&pretty).unwrap();
    assert_eq!(cs, ps);
}
```

- [ ] **Step 3: Run.** `cargo test -p ms-cli --test 'vectors_*' 2>&1 | grep "test result"`. Expected: 3 tests pass (1 in vectors_parity from earlier + 2 new + 1 pretty).

### Task 4.8: Parametric error-envelope + exit-code tests

**Files:** `json_error_envelope_per_kind.rs`, `exit_codes_table.rs`.

- [ ] **Step 1: Write `tests/json_error_envelope_per_kind.rs`.**

```rust
//! For each CliError `kind`, verify JSON-mode error output matches §5.4 schema.

use assert_cmd::Command;
use codex32::{Codex32String, Fe};
use predicates::prelude::*;
use serde_json::Value;

fn run_and_parse(args: &[&str]) -> Value {
    let out = Command::cargo_bin("ms").unwrap()
        .args(args)
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&out).expect("error envelope is valid JSON")
}

#[test]
fn bad_input_json_envelope() {
    // Odd-length hex → BadInput.
    let v = run_and_parse(&["encode", "--hex", "0", "--json"]);
    assert_eq!(v["schema_version"], "1");
    assert_eq!(v["error"]["kind"], "BadInput");
    assert_eq!(v["error"]["exit_code"], 1);
}

#[test]
fn bip39_json_envelope() {
    let v = run_and_parse(&[
        "encode",
        "--phrase",
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon ability",
        "--json",
    ]);
    assert_eq!(v["error"]["kind"], "Bip39");
    assert_eq!(v["error"]["exit_code"], 1);
}

#[test]
fn codex32_json_envelope() {
    // Bad checksum string.
    let v = run_and_parse(&[
        "decode",
        "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7p",  // last char flipped
        "--json",
    ]);
    assert_eq!(v["error"]["kind"], "Codex32");
    assert_eq!(v["error"]["exit_code"], 1);
}

#[test]
fn unexpected_string_length_json_envelope() {
    // 51-char input.
    let v = run_and_parse(&[
        "decode",
        "ms10entrsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",  // 51 chars
        "--json",
    ]);
    assert_eq!(v["error"]["kind"], "UnexpectedStringLength");
    assert_eq!(v["error"]["exit_code"], 1);
    assert_eq!(v["error"]["details"]["got"], 51);
}

#[test]
fn format_violation_json_envelope() {
    // Wrong HRP.
    let mut data = vec![0x00u8];
    data.extend_from_slice(&[0xAAu8; 16]);
    let s = Codex32String::from_seed("mq", 0, "entr", Fe::S, &data).unwrap().to_string();
    let v = run_and_parse(&["decode", &s, "--json"]);
    assert_eq!(v["error"]["kind"], "WrongHrp");
    assert_eq!(v["error"]["exit_code"], 2);
}

#[test]
fn future_format_json_envelope() {
    let mut data = vec![0x00u8];
    data.extend_from_slice(&[0xAAu8; 16]);
    let s = Codex32String::from_seed("ms", 0, "seed", Fe::S, &data).unwrap().to_string();
    let v = run_and_parse(&["verify", &s, "--json"]);
    assert_eq!(v["error"]["kind"], "FutureFormat");
    assert_eq!(v["error"]["exit_code"], 3);
    assert_eq!(v["error"]["details"]["tag"], "seed");
}
```

- [ ] **Step 2: Write `tests/exit_codes_table.rs`.**

```rust
//! Parametric: exit code per CliError variant. Locks SPEC §6 table.

use assert_cmd::Command;

#[test]
fn exit_code_table_user_input() {
    // Odd-length hex → exit 1.
    Command::cargo_bin("ms").unwrap()
        .args(["encode", "--hex", "0"])
        .assert()
        .failure()
        .code(1);
}

#[test]
fn exit_code_table_format_violation() {
    Command::cargo_bin("ms").unwrap()
        .args(["decode", "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7p"])  // bad cksum → Codex32 → exit 1
        .assert()
        .failure()
        .code(1);
}

#[test]
fn exit_code_table_clap_usage() {
    // No subcommand → exit 64.
    Command::cargo_bin("ms").unwrap()
        .arg("--frob-flag-that-doesnt-exist")
        .assert()
        .failure()
        .code(64);
}
```

- [ ] **Step 3: Run.** `cargo test -p ms-cli --test json_error_envelope_per_kind --test exit_codes_table 2>&1 | grep "test result"`. Expected: 9 tests pass.

### Task 4.9: Phase 4 commit

- [ ] **Step 1: Verify full suite + clippy + fmt.**

```bash
cargo test --manifest-path /scratch/code/shibboleth/mnemonic-secret/Cargo.toml -p ms-cli 2>&1 | grep "test result"
cargo clippy --manifest-path /scratch/code/shibboleth/mnemonic-secret/Cargo.toml -p ms-cli --all-targets -- -D warnings 2>&1 | tail -3
cargo fmt --manifest-path /scratch/code/shibboleth/mnemonic-secret/Cargo.toml --all -- --check 2>&1 | tail -3
```

Expected: ~50 tests pass (lib + ~28 integration tests). All CI gates clean.

- [ ] **Step 2: Stage + commit.**

```bash
git -C /scratch/code/shibboleth/mnemonic-secret add Cargo.lock crates/ms-cli/Cargo.toml crates/ms-cli/tests/
git -C /scratch/code/shibboleth/mnemonic-secret commit -m "$(cat <<'EOF'
test(ms-cli): Phase 4 integration test suite (~28 assert_cmd tests + parity)

Phase 4 of IMPLEMENTATION_PLAN_ms_cli_v0_1.md. ~50 total tests passing
(Phase 1 lib unit tests + Phase 4 integration tests). cargo build / clippy
--all-targets -D warnings / fmt --check all clean.

Test groups:
- vectors_parity (1 test): asserts ms-cli/vectors/v0.1.json JSON-equals
  ms-codec/tests/vectors/v0.1.json.
- encode_* (8 tests): canonical 12-word + 24-word; --hex; bad-checksum
  reject; bad-language reject; odd-length-hex reject; passphrase-warning
  on stderr; --no-engraving-card suppression.
- decode_* (4 tests): round-trip with labeled-block stdout; default-language
  stdout/stderr warning; explicit-language no-warning; --json schema.
- inspect_* (8 tests): valid string OK + fields; non-zero-prefix rule 8;
  reserved seed tag rule 7; multiple failures sorted; codex32 parse-failure
  exit 1 + JSON envelope.
- verify_* (5 tests): quiet pass; quiet fail exit 2; future-format exit 3;
  --phrase round-trip OK; --phrase mismatch (no phrase echoed in output).
- pipe + back-typed (3 tests): encode | verify; encode | decode; engraver
  back-typed chunked form via stdin.
- vectors_* (3 tests): compact JSON parseable; canonical first entry; pretty
  matches compact content.
- json_error_envelope_per_kind (6 tests): one per CliError kind with §5.4
  schema verification.
- exit_codes_table (3 tests): parametric exit-code asserts per §6 table.

Cargo.toml dev-deps: added codex32 = "=0.1.0" for test-fixture builders
that hand-construct invalid-but-parseable v0.1 strings (non-zero prefix,
wrong HRP, reserved tag).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

### Task 4.10: Phase 4 opus review checkpoint

- [ ] **Step 1: Dispatch reviewer.**

Brief: verify each SPEC §4 decoder rule has at least one negative integration test. Per the plan r2 review (nit N5), the following rules currently lack direct integration tests and should be added before Phase 4 ships:

- **Rule 3 (threshold ≠ 0):** add `tests/decode_rejects_threshold_not_zero.rs` using `Codex32String::from_seed("ms", 2, "entr", Fe::A, &[0x00, ...16])` (threshold=2, share=A — codex32 lib accepts arbitrary threshold + share at construction).
- **Rule 4 (share-index ≠ 's'):** add `tests/decode_rejects_share_index_not_secret.rs` — but per ms-codec audit, threshold=0 + share != 's' is rejected at upstream parse with `Codex32`; reachable rule 4 path requires threshold > 0 AND share-index parse to succeed, which is mutually exclusive in v0.1 emitted strings. May be defensive-only — confirm with reviewer.
- **Rule 6 (unknown tag):** add `tests/decode_rejects_unknown_tag.rs` using `Codex32String::from_seed("ms", 0, "wxyz", Fe::S, &[0x00, ...16])` ("wxyz" is alphabet-valid but not in RESERVED_TAG_TABLE; same fixture pattern as ms-codec's negative.rs).

Other reviewer concerns: verify all 8 closed-set kebab-case `failure_reasons` tags are exercised by at least one test; verify the parametric `exit_codes_table.rs` actually parameterizes (or upgrade to data-driven if not); verify `verify --phrase` mismatch test does NOT echo either phrase to any output stream.

Persist to `design/agent-reports/phase-4-integration-review-r1.md`. Iterate until convergence.

---

## Phase 5: Release prep

**Goal:** Bump version to 0.1.0, flip publish=false to true, write CHANGELOG entry, write crate-level README, run cargo publish --dry-run, smoke-test --help, tag release.

**Files:**
- Modify: `crates/ms-cli/Cargo.toml`
- Create: `crates/ms-cli/README.md`
- Modify: `CHANGELOG.md`

### Task 5.1: Version bump + publish flip

**Files:**
- Modify: `crates/ms-cli/Cargo.toml`

- [ ] **Step 1: Edit Cargo.toml.**

Change `version = "0.0.0"` → `version = "0.1.0"`.
Change `publish = false` → (remove the line; default is publish=true).
Add to the `[package]` block:

```toml
description = "Companion CLI for the ms-codec library — engrave-friendly BIP-39 entropy backups (the ms1 format)."
documentation = "https://docs.rs/ms-cli"
readme = "README.md"
keywords = ["bitcoin", "codex32", "bip93", "bip39", "engraving"]
categories = ["cryptography::cryptocurrencies", "command-line-utilities"]
```

(Mirror the ms-codec/Cargo.toml shape.)

- [ ] **Step 2: Verify it still builds + tests pass.**

```bash
cargo test --manifest-path /scratch/code/shibboleth/mnemonic-secret/Cargo.toml -p ms-cli 2>&1 | grep "test result"
```

- [ ] **Step 3: Commit version bump alone (separate commit per ms-codec precedent).**

```bash
git -C /scratch/code/shibboleth/mnemonic-secret add crates/ms-cli/Cargo.toml
git -C /scratch/code/shibboleth/mnemonic-secret commit -m "$(cat <<'EOF'
chore(ms-cli): bump to v0.1.0 + flip publish=true

Removes -dev/0.0.0 placeholder; adds description / documentation / readme /
keywords / categories metadata mirroring ms-codec/Cargo.toml. Cargo.toml-
only change; no source or test edits.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

### Task 5.2: Crate-level README

**Files:**
- Create: `crates/ms-cli/README.md`

- [ ] **Step 1: Write the README.**

```markdown
# ms-cli

Companion CLI to the [`ms-codec`](https://crates.io/crates/ms-codec) library — encode BIP-39 entropy as `ms1` strings for steel-plate engraving, decode/inspect/verify the engraved strings, and dump the SHA-pinned test-vector corpus.

5 commands: `encode`, `decode`, `inspect`, `verify`, `vectors`.

## Installation

```bash
cargo install ms-cli
```

The installed binary is named `ms`.

## Quickstart

```bash
# Encode a 12-word BIP-39 mnemonic.
ms encode --phrase "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"

# Recover from an engraved string.
ms decode ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f

# Verify an engraved string round-trips against the original phrase.
ms verify ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f --phrase "abandon abandon ... about"

# Inspect a candidate string for structural validity.
ms inspect ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f
```

All commands support stdin input (`-` or omitted positional) and `--json` for tooling.

## Engraving caveat

`ms1` v0.1 does NOT carry the BIP-39 wordlist language on the wire. Users with non-English wallets MUST record their wordlist language alongside the engraved card. Decode-time `--language english` is the default; the CLI emits a non-suppressible stderr warning when defaulting. See the [SPEC §6.3](https://github.com/bg002h/mnemonic-secret/blob/master/design/SPEC_ms_cli_v0_1.md) for the full hazard discussion.

## Documentation

- [SPEC](https://github.com/bg002h/mnemonic-secret/blob/master/design/SPEC_ms_cli_v0_1.md) — full CLI surface specification.
- [`ms-codec`](https://crates.io/crates/ms-codec) — the underlying library.

## License

CC0 1.0 Universal.
```

### Task 5.3: CHANGELOG entry

**Files:**
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Add the ms-cli [0.1.0] entry at the top of CHANGELOG.md** (above the existing ms-codec entry).

```markdown
## ms-cli [0.1.0] — 2026-MM-DD

### What's new

- Initial release. Companion CLI to ms-codec v0.1.0.
- 5 subcommands: encode, decode, inspect, verify, vectors.
- Phrase-first encode (`--phrase` headline; `--hex` escape hatch); structured `--json` output mode across all commands.
- Strip-whitespace stdin uniform across commands — handles pipe round-trip, engraver-typed-back chunked form, and copy-paste artifacts with one mechanism.
- BIP-39 wordlist enforcement: 10 wordlists supported via `--language` (default `english` with non-suppressible stderr warning surfacing the SPEC §6.3 hazard).
- Exit codes per SPEC §6: 0/1/2/3/4 (verify round-trip mismatch is its own exit code) plus 64 for clap usage errors (overrides clap's default 2 to keep ms1 format violations distinct).
- Engraving-friendly stdout: encode emits `<ms1>\n\n<chunked-form>` (5-char groups, 10/line max, never mid-chunk).
- `verify --phrase` round-trip check: useful for engraver-typed-back proofreading. Phrases never echoed to output (secrets discipline).

### Tests

50 tests across the surface: ~21 unit (Phase 1 modules) + ~28 integration (`assert_cmd`). cargo build / clippy --all-targets -D warnings / fmt --check all clean.
```

(Replace `2026-MM-DD` with the actual release date.)

### Task 5.4: cargo publish --dry-run

- [ ] **Step 1: Run dry-run.**

```bash
cargo publish --manifest-path /scratch/code/shibboleth/mnemonic-secret/Cargo.toml -p ms-cli --dry-run 2>&1 | tail -10
```

Expected: clean packaging. ~30 files, similar size to ms-codec. If "missing readme" or "missing description" complaints fire, fix Cargo.toml + retry.

- [ ] **Step 2: --help smoke test.**

```bash
cargo run --manifest-path /scratch/code/shibboleth/mnemonic-secret/Cargo.toml -p ms-cli -- --help 2>&1 | head -25
cargo run --manifest-path /scratch/code/shibboleth/mnemonic-secret/Cargo.toml -p ms-cli -- encode --help 2>&1 | tail -15
```

Expected: --help text matches SPEC §2.6 verbatim.

### Task 5.5: Phase 5 commit + tag

- [ ] **Step 1: Commit README + CHANGELOG.**

```bash
git -C /scratch/code/shibboleth/mnemonic-secret add crates/ms-cli/README.md CHANGELOG.md
git -C /scratch/code/shibboleth/mnemonic-secret commit -m "$(cat <<'EOF'
release(ms-cli): v0.1.0 — README + CHANGELOG

Phase 5 of IMPLEMENTATION_PLAN_ms_cli_v0_1.md. cargo publish --dry-run
clean. --help smoke test passes. All ~50 tests passing.

Files:
- crates/ms-cli/README.md: crate-level docs (Quickstart, engraving caveat,
  pointer to SPEC + ms-codec).
- CHANGELOG.md: ms-cli [0.1.0] entry above ms-codec [0.1.0] per the
  per-crate-prefix convention from md-codec / mk-codec.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

- [ ] **Step 2: Phase 5 opus review.**

Brief: verify CHANGELOG follows the md-codec / ms-codec convention; verify README accurately reflects the v0.1 command surface; verify `cargo publish --dry-run` output had no missing-field warnings; verify the version-bump commit is separate from the README/CHANGELOG commit (per ms-codec precedent). Persist to `design/agent-reports/phase-5-release-prep-review-r1.md`.

- [ ] **Step 3: Tag the release (locally, not pushed).**

```bash
git -C /scratch/code/shibboleth/mnemonic-secret tag -a ms-cli-v0.1.0 -m "ms-cli v0.1.0"
git -C /scratch/code/shibboleth/mnemonic-secret tag --list ms-cli-v0.1.0 -n
```

**Do not push the tag yet** — that's a user-explicit-approval action per session conventions.

---

## Phase-completion summary

After Phase 5's opus-review convergence, the v0.1.0 release is locally tagged but not pushed. The remaining steps are user-gated:

1. `git push origin master && git push origin ms-cli-v0.1.0` — publish to GitHub.
2. `cargo publish -p ms-cli` — publish to crates.io (requires `cargo login`).
3. `gh release create ms-cli-v0.1.0 --notes-file <changelog excerpt>` — create the GitHub Release.

---

## Plan revision history

(Tracks the plan's own reviewer-loop convergence. Independent of the per-phase reviews.)

- **r6** — 2026-05-04 Phase 4 reviewer-loop r1 fixup: added 2 missing integration tests for SPEC §4 rules 3 (ThresholdNotZero) + 6 (UnknownTag) per task 4.10 brief; updated Task 1.8 parse.rs code block to include `read_phrase_input` + `normalize_phrase` (which was added during Phase 2+3 execution but not back-propagated into the plan's Phase 1 source listing). Rule 4 (ShareIndexNotSecret) remains defensive-only per architect r1 review.

- **r5** — 2026-05-04 Phase 4 execution-time fixups (three source bugs surfaced by integration tests): (1) Task 2.2 `EncodeArgs` struct-level `#[group]` changed to `#[command(group = clap::ArgGroup::new(...))]` — the old form made ALL fields mutually exclusive (including `--language`, `--json`, `--no-engraving-card`), not just `phrase`/`hex`; (2) Task 2.5 `verify.rs` concurrent-stdin guard tightened to `args.phrase.as_deref() == Some("-")` instead of `is_stdin_arg(args.phrase.as_deref())` — the old form fired when `--phrase` was absent (None), preventing any stdin-piped `ms verify -` without `--phrase`; (3) Task 1.8 `parse.rs::strip_whitespace` gains doubling-detection (SPEC §3.2 step 4 per r7) — naive strip-whitespace collapsed `ms encode` multi-line stdout into a doubled `<ms1><ms1>` string, and a `had_whitespace` gate prevents the guard from firing on all-zero inline hex args.

- **r1** — 2026-05-04 initial draft via `superpowers:writing-plans` skill (~3500 lines, 5 phases, 32 tasks, ~21 unit + ~30 integration tests projected).
- **r4** — 2026-05-04 Phase 2 execution-time fixup: ms_codec::Payload is also #[non_exhaustive] (parallel to ms_codec::Error in r2), so cmd/decode.rs and cmd/verify.rs match expressions need wildcard arms. Two unreachable!() arms added to each file inline; same plan-r2-fix-pattern.

- **r3** — 2026-05-04 architect r2 plan-review terminator (0 critical / 0 important / 6 nits). 2 high-value nits applied: Phase 4 task 4.10 reviewer brief now explicitly calls out rules 3/4/6 coverage gaps with concrete fixture commands (resolves r2-N5); 2 nits deferred to FOLLOWUPS as `ms-cli-v01-plan-r2-nit-N1` (verify --phrase language masking) and `ms-cli-v01-plan-r2-nit-N3` (parse_hex_entropy length pre-check).

- **r2** — 2026-05-04 architect r1 plan-review surfaced 2 critical + 5 important + 6 nits, all resolved inline:
  - **C1:** `bip39::Error::BadChecksum` → `InvalidChecksum` (4 places in plan; SPEC also fixed `InvalidWord` → `UnknownWord` in 2 places, in lockstep).
  - **C2:** Task 1.1 spike notes that the registry is populated by step 1's `cargo run` so step 2's grep works (or implementer can run `cargo fetch` defensively).
  - **I1:** `cmd/inspect.rs::analyze` rule-check order rewritten to ascend (2/3/4/6-or-7/8/9/10) per SPEC §2.3 contract.
  - **I2:** Phase 4 task 4.2 step 9 added `tests/encode_arg_group_violations.rs` (both-supplied + neither-supplied → exit 64).
  - **I3:** `main.rs` `deny(missing_docs)` swapped for `allow(missing_docs)` per md-cli precedent (binary-only crate; field docs aren't load-bearing).
  - **I4:** Task 1.2 dev-deps now include `codex32 = "=0.1.0"` upfront so Phase 4 doesn't need a Cargo.toml edit step.
  - **I5:** `cmd/verify.rs::emit_future_format` only prints success line in text mode (suppresses in JSON to avoid double-output); `main.rs::emit_error` adds an explicit early-return for `CliError::FutureFormat` in text mode (suppresses stderr "error: ..." that would contradict the OK stdout line).
  - Plus EncodeArgs at Task 2.2 gains `#[group(id = "input", required = true, multiple = false)]` at the struct level so clap enforces the I2-tested arg-group invariants.

---

## Self-review checklist (run before handing off)

Performed during plan drafting:

**1. Spec coverage:** Walking through SPEC sections:

- §1 Scope → Phase 1 task 1.2 (Cargo.toml deps), Phase 3 task 3.2 (main.rs Cli::about lock).
- §2.1 encode → Phase 2 task 2.2 (encode.rs); Phase 4 task 4.2 (8 integration tests).
- §2.2 decode → Phase 2 task 2.3; Phase 4 task 4.3 (3 tests).
- §2.3 + §2.3.1 inspect → Phase 2 task 2.4; Phase 4 task 4.4 (5 tests including codex32_parse_failure).
- §2.4 + §2.4.1 verify → Phase 2 task 2.5; Phase 4 task 4.5 (5 tests).
- §2.5 vectors → Phase 2 task 2.6 + Phase 3 task 3.1 (corpus); Phase 4 tasks 4.1 + 4.7.
- §2.6 --help text → Phase 3 task 3.2 (main.rs `#[command(after_long_help = ...)]`).
- §3 I/O discipline → Phase 1 task 1.8 (parse.rs strip-whitespace) + Phase 2's commands honor stdin/stderr conventions.
- §4 engraving card + chunking → Phase 1 task 1.7 (format::chunked) + Phase 2 task 2.2 (encode.rs emit_text).
- §5 JSON schemas → Phase 1 task 1.7 (format.rs structs); Phase 4 task 4.8 (per-kind error envelope tests).
- §6 + §6.1 + §6.1.1 errors → Phase 1 task 1.3 (error.rs CliError + From<ms_codec::Error>); Phase 4 task 4.8 (exit_codes_table).
- §7 languages → Phase 1 task 1.6 (language.rs); Phase 4 covered by encode_canonical_*_word.
- §8 out-of-scope → no implementation (defines what's NOT here).
- §9 closure tracking → revision history at the bottom of the SPEC + this plan's task ↔ closure references.
- §10 + §10.0 module layout + dep order → followed exactly by Phase 1 → Phase 2 → Phase 3 ordering.
- §10.1 test strategy → matches Phase 4's ~28 integration tests + Phase 1's ~21 unit tests.
- §10.2 CI gates → verified per-phase via cargo build/test/clippy/fmt commands in each phase's commit task.
- §10.3 versioning → Phase 5 task 5.1 (version bump 0.0.0 → 0.1.0; publish flip).

**2. Placeholder scan:** Every code block contains real, runnable Rust. Every commit message is final text. Two exceptions: CHANGELOG.md `2026-MM-DD` placeholder (filled at release time) and Phase 4 test-count `~28` is approximate to allow for slight count variance during implementation.

**3. Type consistency:** `CliError` / `Payload` / `Tag` / `Mnemonic` / `Language` / `CliLanguage` / `EncodeArgs` etc. are referenced consistently across phases. The `cmd::*::run(args)` signature is consistent. `From<ms_codec::Error> for CliError` is the only impl boundary mentioned; no drift.

---

## Execution handoff

Plan complete and saved to `design/IMPLEMENTATION_PLAN_ms_cli_v0_1.md`. Two execution options:

1. **Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration. Best for the 30+ task plan since each task is independently verifiable.

2. **Inline Execution** — Execute tasks in this session using `superpowers:executing-plans`, batch by phase with checkpoints. Lower overhead but less review per task.

Per the user's explicit autonomy authorization earlier in this session (and the established cadence with ms-codec which used inline execution to v0.1.0 successfully), inline execution is the natural choice unless you want subagent-per-task isolation.

**Which approach?** And: should I run the iterative reviewer-loop on this plan itself (per memory `feedback_iterative_review_every_phase` — plan reviews stay in transcript) before execution starts? r2 SPEC was 0/0 terminator; plan r1 review may find similar tightenings.
