# ms-codec v0.1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship `crates/ms-codec` v0.1.0: a Rust library encoding BIP-39 entropy as `ms1`-prefixed BIP-93 codex32 strings via Andrew Poelstra's `rust-codex32 = "=0.1.0"` (no fork), with the v0.2-share-encoding-migration `0x00` reserved-prefix byte locked in from day 1.

**Architecture:** Thin layer atop `rust-codex32`. All BCH plumbing delegated upstream. The crate adds: type-tag semantics on BIP-93's `id` field (v0.1: `entr` only), reserved-prefix byte on the payload (v0.1: `0x00`; v0.2 promotes to type discriminator), strict decoder enforcement of the SPEC §4 validity rules, public `Tag`/`Payload`/`InspectReport` types with `#[non_exhaustive]` discipline. The single rust-codex32 contact module (`envelope.rs`) wire-position re-parses the validated string to extract hrp/threshold/id/share_index because `rust-codex32 v0.1.0`'s `Parts` struct has non-`pub` fields (verified against `/tmp/codex32-extract/codex32-0.1.0/src/lib.rs:383-392` during SPEC drafting).

**Tech Stack:** Rust 2021 edition, MSRV 1.85 (lockstep with md-codec; do not lead). Single runtime dep: `codex32 = "=0.1.0"` (CC0). Dev deps: `proptest` (round-trip), `bip39` (BIP-39 integration test), `serde` + `serde_json` (vector corpus). CI: stable + beta + MSRV three-row matrix; `cargo build`, `cargo test`, `cargo clippy --all-targets -D warnings`, `cargo fmt --check`.

**Source-of-truth artifacts:**
- SPEC: `design/SPEC_ms_v0_1.md` (reviewer-converged at r2 = 0 critical / 0 important; r3 nits applied inline).
- BRAINSTORM: `design/BRAINSTORM_ms_v0_1.md` (r6 amendment integrated).
- MIGRATION: `MIGRATION.md` (v0.1 → v0.2 contract; SPEC §5 mirrors verbatim).
- Pinned upstream source (read-only reference): `/tmp/codex32-extract/codex32-0.1.0/src/lib.rs`, `/tmp/codex32-extract/codex32-0.1.0/src/checksum.rs`, `/tmp/codex32-extract/codex32-0.1.0/src/field.rs`.
- BIP-93 canonical text (read-only reference): `/tmp/bip-0093.mediawiki`.

**Convergence convention (per memory `feedback_iterative_review_every_phase`):** every phase ends with an opus reviewer-loop iteration that runs until a round returns 0 critical / 0 important findings. Per-phase reports persist to `design/agent-reports/<phase-id>-review-rN.md`. Critical/important findings → fixed inline in a fixup commit. Low/nit → recorded in `design/FOLLOWUPS.md` at appropriate tier. Affirmations confirm decisions and terminate the loop.

**Commit cadence (per memory):** within each phase: one feature commit at phase-end (after all tasks); one fixup commit after each opus review round if findings landed. Stage paths explicitly per memory `feedback_avoid_git_add_all`.

**Phase 1 task 1 is non-negotiable:** before any code lands, verify `rust-codex32 v0.1.0`'s actual `Parts` accessor surface against this plan's assumptions (SPEC §10.1). The wire-position re-parse strategy is locked but the upstream API is the source of truth — if the upstream surface has changed since SPEC drafting (it shouldn't have; we exact-pinned), this phase adapts.

**TDD step-title convention:** several tasks below have a "Step 1: Write the failing tests" step that contains both the test module and the implementation in a single file edit. The TDD discipline is preserved because (a) tests are written deliberately to drive design, not retrofitted to passing impl; (b) the first `cargo test` run after the file is created either passes (and we proceed) or fails (and we fix impl before moving on — the canonical red→green loop); (c) for pure type definitions where the test is essentially tautological with the const definition (e.g., `assert_eq!(Tag::ENTR.as_str(), "entr")`), no real "red" phase is meaningful. Where a behavioral test could genuinely fail before impl, we structure the file so the test references functions that don't exist yet, forcing the impl to land in the same edit. This is a convention; reviewers may flag it but it does not warrant a step-title rewrite per task.

---

## Plan revision history

(Tracks this plan's iterative reviewer-loop convergence.)

- **r1** — 2026-05-03 initial draft via `superpowers:writing-plans` skill. 7 phases, ~30 unit + integration tests projected.
- **r3** — 2026-05-03 reviewer loop terminated (r2 returned 0 critical / 0 important, 5 nits / 7 affirmations). Two r2 nits taken inline: rule_2 wrong-HRP test tightened to assert `WrongHrp` deterministically (the `UnexpectedStringLength` branch was dead code since HRP swap doesn't change string length); added a `consts.rs::tests::valid_str_lengths_match_entr_lengths_via_bijection` test that locks the formula `9 + ceil((bytes+1)*8/5) + 13 = total` so any future drift between `VALID_ENTR_LENGTHS` and `VALID_STR_LENGTHS` fails CI loudly. Three r2 nits left as-is (forward-compat 255-byte sweep is fast enough; meta-narrative consistency between "important" and "nit-applied" labels is cosmetic; FOLLOWUPS entry quality affirmation needed no action).

- **r2** — 2026-05-03 integrated 2 critical + 3 important findings from r1 plan-review:
  - **Critical #1:** Phase 5 `tests/negative.rs` rule_4 test was unreachable (48-char string failed rule 9 length-check before reaching upstream parse). Replaced with a 50-char `Codex32String::from_seed("ms", 0, "entr", Fe::C, ...)` that triggers BIP-93's own threshold-0/share-S enforcement at upstream parse, surfaced via our `Error::Codex32(_)`.
  - **Critical #2:** Phase 7 Task 7.6 Step 5 mixed a deliberate version-bump (`0.1.0-dev` → `0.1.0`) into a CI-gate task. Split out into new Task 7.6.5 with its own commit (`chore(ms-codec): bump to v0.1.0`) so the release log is unambiguous. Task 7.7's release-prep commit narrative updated accordingly.
  - **Important #3:** Phase 5 rule_3 test had dead-code conditional (`if !VALID_STR_LENGTHS.contains(&s.len())` was always false for 16-B + 0x00 prefix in threshold-2 form). Dropped the conditional; assertion is now deterministic.
  - **Important #4:** `error.rs` defined `From<codex32::Error>` but call sites used `.map_err(Error::Codex32)` inconsistently. Updated `envelope::package`, `decode::decode`, `inspect::inspect` to use the `?` operator to leverage the From impl.
  - **Important #5:** Test step titles ("Write the failing tests") were technically misleading where impl + tests land in the same file edit. Documented the convention up front (above) rather than rewriting each task.
  - **Nits applied inline:** Added `Tag::from_raw_bytes` named constructor (alphabet-validation-bypassing) for `inspect.rs`'s "surface raw bytes" use case; made `Tag.0` field private to enforce that all encoder/decoder paths go through `try_new`. Phase 4 doc-test count reconciled (26 unit + 1 doc-test).
  - **Nits deferred to FOLLOWUPS:** Phase 1 Task 1.7 nit-format slug style; Phase 7 Task 7.5 README single-step granularity; Phase 5 rule_2 SPEC-mandate-ordering wording; consts.rs naming-style consistency.

---

## Phase 1: Foundation — types, constants, errors

**Goal:** Land all type definitions and constants with no runtime behavior. By end of phase, `cargo build` passes; tests are scaffolded but mostly empty (no encoder/decoder yet).

**Files:**
- Read-verify: `/tmp/codex32-extract/codex32-0.1.0/src/lib.rs` (Phase 1 task 1)
- Modify: `crates/ms-codec/Cargo.toml` (add proptest dev-dep)
- Create: `crates/ms-codec/src/consts.rs`
- Create: `crates/ms-codec/src/error.rs`
- Create: `crates/ms-codec/src/tag.rs`
- Create: `crates/ms-codec/src/payload.rs`
- Modify: `crates/ms-codec/src/lib.rs` (replace placeholder; add module decls + re-exports)

### Task 1.1: Verify rust-codex32 v0.1.0 Parts accessor surface (the spike)

**Files:**
- Read: `/tmp/codex32-extract/codex32-0.1.0/src/lib.rs:383-430` (Parts struct + impl block)

- [ ] **Step 1: Read the upstream Parts definition**

```bash
sed -n '380,430p' /tmp/codex32-extract/codex32-0.1.0/src/lib.rs
```

Expected output: a `pub struct Parts<'s>` with private fields (no `pub` keyword on `hrp`, `threshold`, `id`, `share_index`, `payload`, `checksum`), and a single `pub fn data(&self) -> Vec<u8>` impl method. If the fields are `pub`, this plan's wire-position re-parse strategy is unnecessary — fall back to direct field access in envelope.rs and update SPEC §10.1 to document the change. If the fields are non-`pub` as expected, proceed to Step 2.

- [ ] **Step 2: Confirm by attempting field access in a throwaway program**

```bash
mkdir -p /tmp/ms-codec-spike && cd /tmp/ms-codec-spike && cat > Cargo.toml <<'EOF'
[package]
name = "spike"
version = "0.0.0"
edition = "2021"

[dependencies]
codex32 = "=0.1.0"
EOF
mkdir -p src && cat > src/main.rs <<'EOF'
use codex32::Codex32String;

fn main() {
    let s = Codex32String::from_string(
        "ms10testsxxxxxxxxxxxxxxxxxxxxxxxxxx4nzvca9cmczlw".into()
    ).unwrap();
    let parts = s.parts();
    let _ = parts.data(); // pub method, OK
    let _ = parts.hrp;    // expected to fail with E0616 "field is private"
}
EOF
cargo build 2>&1 | tail -10
```

Expected: compile error `error[E0616]: field hrp of struct codex32::Parts is private` (or similar). This confirms the SPEC §10.1 strategy is required.

- [ ] **Step 3: Lock the wire-position re-parse offsets**

Document the offsets in a comment at the top of `envelope.rs` (Phase 2 task 2.1). Offsets per SPEC §2.5 wire-field-assignments table, computed relative to the position of the `1` separator (call it `sep`):

```text
threshold:    sep + 1                   (1 char)
id:           sep + 2  .. sep + 6       (4 chars, codex32 alphabet)
share-index:  sep + 6                   (1 char, must be 's' for v0.1)
payload:      sep + 7  .. s.len() - 13  (variable; -13 strips short checksum)
checksum:     s.len() - 13 .. s.len()   (13 chars, short)
```

For v0.1 we never see long-checksum strings (rejected by §4 rule 9); checksum length is fixed at 13. **Do not compute against the long bracket** — it's a future-version concern.

- [ ] **Step 4: No commit** (this is a verification task; nothing to commit)

If Step 1 contradicted the SPEC §10.1 assumption, file a SPEC amendment (r4) before proceeding to Task 1.2. If Step 1 confirmed the assumption (expected case), proceed.

### Task 1.2: Add proptest dev-dep + write `consts.rs`

**Files:**
- Modify: `crates/ms-codec/Cargo.toml`
- Create: `crates/ms-codec/src/consts.rs`

- [ ] **Step 1: Add proptest dev-dep**

Modify `crates/ms-codec/Cargo.toml`. After the `[dependencies]` block, add:

```toml
[dev-dependencies]
proptest = "1"
```

- [ ] **Step 2: Verify Cargo.toml parses**

```bash
cargo check -p ms-codec 2>&1 | tail -5
```

Expected: clean `Compiling`/`Finished` output (or "no targets specified" if no lib content yet — also acceptable). No `error: failed to parse` lines.

- [ ] **Step 3: Write `crates/ms-codec/src/consts.rs`**

```rust
//! v0.1 wire-format constants.

/// HRP for ms1 strings (BIP-93 codex32 HRP).
pub const HRP: &str = "ms";

/// BIP-93 separator character.
pub const SEPARATOR: char = '1';

/// v0.1 reserved-prefix byte (becomes the v0.2 type discriminator).
pub const RESERVED_PREFIX: u8 = 0x00;

/// v0.1 emit-side threshold value (ASCII).
pub const THRESHOLD_V01: u8 = b'0';

/// v0.1 emit-side share-index value (ASCII; "s" denotes the unshared secret per BIP-93).
pub const SHARE_INDEX_V01: u8 = b's';

/// Short codex32 checksum length in characters.
pub const CHECKSUM_LEN_SHORT: usize = 13;

/// Allowed v0.1 entr entropy byte lengths (bijective with BIP-39 word counts {12,15,18,21,24}).
pub const VALID_ENTR_LENGTHS: &[usize] = &[16, 20, 24, 28, 32];

/// Allowed v0.1 total ms1 string lengths (HRP+sep+threshold+id+share+payload+cksum).
/// Computed: 9 fixed + ceil((entropy_bytes + 1) * 8 / 5) payload symbols + 13 cksum.
pub const VALID_STR_LENGTHS: &[usize] = &[50, 56, 62, 69, 75];

/// 4-byte type tag — v0.1 emit (also accept).
pub const TAG_ENTR: [u8; 4] = *b"entr";

/// 4-byte type tags reserved-not-emitted in v0.1 (decoder rejects).
pub const RESERVED_NOT_EMITTED_V01: &[[u8; 4]] = &[
    *b"seed",
    *b"xprv",
    *b"mnem",
    *b"prvk",
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Lock the bijection between VALID_ENTR_LENGTHS and VALID_STR_LENGTHS so
    /// that a future edit to one without the other fails CI loudly.
    /// Formula per SPEC §2.4: total = 9 fixed (HRP+sep+threshold+id+share) +
    /// ceil((entropy_bytes + 1) * 8 / 5) payload symbols + 13 short checksum.
    #[test]
    fn valid_str_lengths_match_entr_lengths_via_bijection() {
        assert_eq!(VALID_ENTR_LENGTHS.len(), VALID_STR_LENGTHS.len());
        for (i, &entropy_bytes) in VALID_ENTR_LENGTHS.iter().enumerate() {
            let data_bits = (entropy_bytes + 1) * 8; // +1 for the 0x00 prefix byte
            let payload_symbols = (data_bits + 4) / 5; // ceil(bits/5)
            let total = 9 + payload_symbols + CHECKSUM_LEN_SHORT;
            assert_eq!(
                total, VALID_STR_LENGTHS[i],
                "entropy {} B -> expected str.len {}, got {} (bijection drift)",
                entropy_bytes, VALID_STR_LENGTHS[i], total
            );
        }
    }
}
```

- [ ] **Step 4: Wire `consts` into lib.rs**

Modify `crates/ms-codec/src/lib.rs`. Replace the file's contents (the existing scaffold) with:

```rust
//! `ms-codec` — reference implementation of the **ms1** backup format (HRP `ms`).
//!
//! Status: pre-v0.1.0. Wire format and public API are specified in
//! [`design/SPEC_ms_v0_1.md`](../../design/SPEC_ms_v0_1.md). See also
//! [`MIGRATION.md`](../../MIGRATION.md) for the v0.1 → v0.2 contract.
//!
//! v0.1 emits BIP-39 entropy only (16/20/24/28/32 B). Direct BIP-32 master seed
//! and xpriv payloads are reserved-not-emitted in v0.1 and deferred to v0.2+
//! with separate framing (they overflow BIP-93 codex32's length brackets when
//! prepended with the v0.2-migration prefix byte).

#![cfg_attr(not(test), deny(missing_docs))]

pub mod consts;
```

- [ ] **Step 5: Verify it builds**

```bash
cargo build -p ms-codec 2>&1 | tail -5
```

Expected: clean `Compiling ms-codec ... Finished`. No warnings (the `deny(missing_docs)` attribute is gated on `not(test)` so it won't fire here yet).

### Task 1.3: Write `error.rs`

**Files:**
- Create: `crates/ms-codec/src/error.rs`
- Modify: `crates/ms-codec/src/lib.rs` (add `pub mod error`)

- [ ] **Step 1: Write `crates/ms-codec/src/error.rs`**

```rust
//! ms-codec error taxonomy. Variants mirror SPEC §4 decoder validity rules
//! plus the encoder-side validation surface from SPEC §3.5 / §3.5.1.

use std::fmt;

/// ms-codec error type.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Upstream codex32 parse / checksum failure (delegated from rust-codex32).
    Codex32(codex32::Error),
    /// HRP was not "ms" (SPEC §4 rule 2).
    WrongHrp { got: String },
    /// Threshold was not 0 (SPEC §4 rule 3).
    ThresholdNotZero { got: u8 },
    /// Share-index was not 's' — BIP-93 requires 's' for threshold=0 (SPEC §4 rule 4).
    ShareIndexNotSecret { got: char },
    /// Tag bytes were not in the codex32 alphabet (SPEC §4 rule 5).
    TagInvalidAlphabet { got: [u8; 4] },
    /// Tag was structurally valid but not in RESERVED_TAG_TABLE (SPEC §4 rule 6).
    UnknownTag { got: [u8; 4] },
    /// Tag was in RESERVED_TAG_TABLE but reserved-not-emitted in v0.1 (SPEC §4 rule 7,
    /// SPEC §3.5.1 encoder symmetry).
    ReservedTagNotEmittedInV01 { got: [u8; 4] },
    /// Reserved-prefix byte was not 0x00 (SPEC §4 rule 8).
    ReservedPrefixViolation { got: u8 },
    /// Total string length was outside the v0.1 emittable set (SPEC §4 rule 9).
    UnexpectedStringLength { got: usize, allowed: &'static [usize] },
    /// Payload byte length did not match the tag's spec (SPEC §3.5, §4 rule 10).
    PayloadLengthMismatch {
        tag: [u8; 4],
        expected: &'static [usize],
        got: usize,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Codex32(e) => write!(f, "codex32 parse error: {:?}", e),
            Error::WrongHrp { got } => write!(f, "wrong HRP: got {:?}, expected \"ms\"", got),
            Error::ThresholdNotZero { got } => {
                write!(f, "threshold not 0 (got '{}'); v0.1 is single-string only", *got as char)
            }
            Error::ShareIndexNotSecret { got } => {
                write!(f, "share-index not 's' (got '{}'); BIP-93 requires 's' for threshold=0", got)
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
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Codex32(_) => None, // codex32::Error doesn't impl std::error::Error in v0.1.0
            _ => None,
        }
    }
}

impl From<codex32::Error> for Error {
    fn from(e: codex32::Error) -> Self {
        Error::Codex32(e)
    }
}

/// Result alias for ms-codec.
pub type Result<T> = std::result::Result<T, Error>;
```

- [ ] **Step 2: Wire `error` into lib.rs**

Modify `crates/ms-codec/src/lib.rs`. Append after `pub mod consts;`:

```rust
pub mod error;

pub use error::{Error, Result};
```

- [ ] **Step 3: Verify it builds**

```bash
cargo build -p ms-codec 2>&1 | tail -5
```

Expected: clean `Compiling ... Finished`. No warnings.

### Task 1.4: Write `tag.rs` with TDD

**Files:**
- Create: `crates/ms-codec/src/tag.rs`
- Modify: `crates/ms-codec/src/lib.rs`

- [ ] **Step 1: Write the failing tests at the top of a new `tag.rs`**

Create `crates/ms-codec/src/tag.rs` with the test module first (TDD discipline):

```rust
//! Tag type — 4-byte codex32-alphabet validated type tag.

use crate::consts::TAG_ENTR;
use crate::error::{Error, Result};

/// codex32 alphabet (BIP-173 lowercase bech32 charset).
const CODEX32_ALPHABET: &[u8] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

/// 4-byte type tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tag([u8; 4]);

impl Tag {
    /// The v0.1 emit-tag for BIP-39 entropy.
    pub const ENTR: Tag = Tag(TAG_ENTR);

    /// Construct a Tag from raw 4-byte input WITHOUT alphabet validation.
    /// Reserved for tooling (e.g., `inspect()`) that needs to surface whatever
    /// bytes were observed on the wire, including alphabet violators. Encoder
    /// + decoder paths MUST go through `try_new` instead.
    pub fn from_raw_bytes(b: [u8; 4]) -> Self {
        Tag(b)
    }

    /// Construct a Tag from a 4-character string slice. Returns
    /// `Error::TagInvalidAlphabet` if any character is outside the codex32 alphabet.
    pub fn try_new(s: &str) -> Result<Self> {
        let bytes = s.as_bytes();
        if bytes.len() != 4 {
            return Err(Error::TagInvalidAlphabet {
                got: [
                    bytes.get(0).copied().unwrap_or(0),
                    bytes.get(1).copied().unwrap_or(0),
                    bytes.get(2).copied().unwrap_or(0),
                    bytes.get(3).copied().unwrap_or(0),
                ],
            });
        }
        let mut out = [0u8; 4];
        for (i, b) in bytes.iter().enumerate() {
            if !CODEX32_ALPHABET.contains(b) {
                return Err(Error::TagInvalidAlphabet { got: [bytes[0], bytes[1], bytes[2], bytes[3]] });
            }
            out[i] = *b;
        }
        Ok(Tag(out))
    }

    /// Borrow the underlying 4 bytes.
    pub fn as_bytes(&self) -> &[u8; 4] {
        &self.0
    }

    /// View the tag as a string slice.
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).expect("Tag construction validates alphabet, which is ASCII")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entr_const_matches_string() {
        assert_eq!(Tag::ENTR.as_str(), "entr");
    }

    #[test]
    fn try_new_accepts_alphabet_chars() {
        // All four lowercase BIP-39-style tags should parse.
        for s in ["entr", "seed", "xprv", "mnem", "prvk"] {
            let t = Tag::try_new(s).expect(s);
            assert_eq!(t.as_str(), s);
        }
    }

    #[test]
    fn try_new_rejects_uppercase() {
        // codex32 alphabet is lowercase; uppercase bytes are rejected.
        assert!(matches!(
            Tag::try_new("ENTR"),
            Err(Error::TagInvalidAlphabet { .. })
        ));
    }

    #[test]
    fn try_new_rejects_out_of_alphabet_chars() {
        // 'b' and 'i' and 'o' are NOT in the codex32 alphabet (excluded for OCR safety).
        for s in ["beer", "iron", "oboe"] {
            assert!(
                matches!(Tag::try_new(s), Err(Error::TagInvalidAlphabet { .. })),
                "expected reject for {:?}",
                s
            );
        }
    }

    #[test]
    fn try_new_rejects_wrong_length() {
        for s in ["", "a", "ab", "abc", "abcde"] {
            assert!(
                matches!(Tag::try_new(s), Err(Error::TagInvalidAlphabet { .. })),
                "expected reject for {:?}",
                s
            );
        }
    }
}
```

- [ ] **Step 2: Wire `tag` into lib.rs**

Modify `crates/ms-codec/src/lib.rs`. Append:

```rust
pub mod tag;

pub use tag::Tag;
```

- [ ] **Step 3: Run the tests — they must pass on first try (since impl is in the same file)**

```bash
cargo test -p ms-codec --lib tag::tests 2>&1 | tail -15
```

Expected: 5 tests pass. If any fail, the impl needs adjustment in this task before moving on. **Do not proceed to Task 1.5 until all 5 pass.**

### Task 1.5: Write `payload.rs` with TDD

**Files:**
- Create: `crates/ms-codec/src/payload.rs`
- Modify: `crates/ms-codec/src/lib.rs`

- [ ] **Step 1: Write `crates/ms-codec/src/payload.rs`**

```rust
//! Payload type — v0.1: Entr (BIP-39 entropy) only.

use crate::consts::VALID_ENTR_LENGTHS;
use crate::error::{Error, Result};
use crate::tag::Tag;

/// v0.1 payload kind. Future kinds (Mnem, Seed, Xprv) will arrive in v0.2+
/// with their own framing per SPEC §1, §3.3, §8.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PayloadKind {
    /// BIP-39 entropy (16/20/24/28/32 B).
    Entr,
}

/// v0.1 payload.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Payload {
    /// BIP-39 entropy. Length MUST be in {16, 20, 24, 28, 32} bytes
    /// (bijective with BIP-39 word counts {12, 15, 18, 21, 24}).
    ///
    /// **Caller responsibility:** ms-codec does NOT check the statistical
    /// quality of these bytes. Callers are responsible for sourcing entropy
    /// from a vetted CSPRNG, or from a BIP-39 mnemonic the user already trusts.
    /// FIPS-style entropy-quality checks would slow encoding and provide false
    /// assurance — they cannot detect attacker-supplied "pseudo-random" seeds
    /// crafted to pass standard randomness tests. See SPEC §3.6.
    Entr(Vec<u8>),
}

impl Payload {
    /// Validate the payload's intrinsic structure (byte length for Entr).
    /// Encoder MUST call this before emitting; decoder calls it after extracting
    /// the payload bytes following the reserved-prefix byte.
    pub fn validate(&self) -> Result<()> {
        match self {
            Payload::Entr(data) => {
                if !VALID_ENTR_LENGTHS.contains(&data.len()) {
                    return Err(Error::PayloadLengthMismatch {
                        tag: *Tag::ENTR.as_bytes(),
                        expected: VALID_ENTR_LENGTHS,
                        got: data.len(),
                    });
                }
                Ok(())
            }
        }
    }

    /// The PayloadKind discriminant.
    pub fn kind(&self) -> PayloadKind {
        match self {
            Payload::Entr(_) => PayloadKind::Entr,
        }
    }

    /// Borrow the inner byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Payload::Entr(data) => data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entr_accepts_all_bip39_lengths() {
        for len in [16usize, 20, 24, 28, 32] {
            let p = Payload::Entr(vec![0u8; len]);
            p.validate().unwrap_or_else(|e| panic!("expected ok for len {}, got {:?}", len, e));
        }
    }

    #[test]
    fn entr_rejects_off_by_one_lengths() {
        for len in [15usize, 17, 19, 21, 23, 25, 31, 33] {
            let p = Payload::Entr(vec![0u8; len]);
            assert!(
                matches!(p.validate(), Err(Error::PayloadLengthMismatch { .. })),
                "expected reject for len {}",
                len
            );
        }
    }

    #[test]
    fn entr_rejects_zero_length() {
        let p = Payload::Entr(vec![]);
        assert!(matches!(p.validate(), Err(Error::PayloadLengthMismatch { .. })));
    }

    #[test]
    fn kind_returns_entr() {
        assert_eq!(Payload::Entr(vec![0u8; 16]).kind(), PayloadKind::Entr);
    }
}
```

- [ ] **Step 2: Wire `payload` into lib.rs**

Modify `crates/ms-codec/src/lib.rs`. Append:

```rust
pub mod payload;

pub use payload::{Payload, PayloadKind};
```

- [ ] **Step 3: Run the tests**

```bash
cargo test -p ms-codec --lib payload::tests 2>&1 | tail -15
```

Expected: 4 tests pass.

### Task 1.6: Phase 1 commit

- [ ] **Step 1: Verify clean state**

```bash
git -C /scratch/code/shibboleth/mnemonic-secret status --short
cargo test -p ms-codec --lib 2>&1 | tail -5
```

Expected status: 5 files modified/created (Cargo.toml, src/lib.rs, src/consts.rs, src/error.rs, src/tag.rs, src/payload.rs). Expected tests: 9 passing (5 tag + 4 payload).

- [ ] **Step 2: Stage paths explicitly + commit**

```bash
git -C /scratch/code/shibboleth/mnemonic-secret add \
  crates/ms-codec/Cargo.toml \
  crates/ms-codec/src/lib.rs \
  crates/ms-codec/src/consts.rs \
  crates/ms-codec/src/error.rs \
  crates/ms-codec/src/tag.rs \
  crates/ms-codec/src/payload.rs

git -C /scratch/code/shibboleth/mnemonic-secret commit -m "$(cat <<'EOF'
feat(ms-codec): Phase 1 foundation — consts, error, Tag, Payload

Phase 1 of IMPLEMENTATION_PLAN_ms_v0_1.md. Lands all type definitions
and constants with no runtime encoder/decoder behavior yet. cargo build
clean; 9 unit tests passing (5 tag + 4 payload).

Modules added:
- consts.rs:        HRP "ms", reserved prefix 0x00, valid entr/string
                    lengths, RESERVED_TAG_TABLE bytes
- error.rs:         Error enum (10 variants) mirroring SPEC §4 decoder
                    rules + §3.5.1 encoder symmetry; #[non_exhaustive];
                    From<codex32::Error>
- tag.rs:           Tag([u8; 4]) with codex32-alphabet validation;
                    Tag::ENTR const (only v0.1 emit-tag)
- payload.rs:       Payload enum (v0.1: Entr only; #[non_exhaustive]);
                    validate() enforces {16,20,24,28,32}-byte length
                    set; PayloadKind discriminant

Phase 1 task 1 verified rust-codex32 v0.1.0 Parts struct fields are
non-pub (only data() publicly accessible); SPEC §10.1 wire-position
re-parse strategy is required and is locked for Phase 2's envelope.rs.

Per SPEC §3.6, Payload::Entr doc-comment explicitly states caller-side
CSPRNG responsibility and dissuades implementers from adding
FIPS-style entropy-quality checks.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"

git -C /scratch/code/shibboleth/mnemonic-secret show HEAD --stat | head -15
```

Expected: clean commit with 6 files changed.

### Task 1.7: Phase 1 opus review

- [ ] **Step 1: Dispatch a `feature-dev:code-reviewer` (or equivalent) opus subagent**

Brief the agent with:
- Files reviewed: `crates/ms-codec/src/{consts,error,tag,payload}.rs` + `crates/ms-codec/src/lib.rs` + `crates/ms-codec/Cargo.toml`.
- Reference docs: `design/SPEC_ms_v0_1.md` (especially §2 string layer, §3 payload semantics, §4 decoder rules), `design/BRAINSTORM_ms_v0_1.md` for rationale.
- Brief: this is Phase 1 of the ms-codec v0.1 implementation. Verify type definitions match SPEC; verify error variant set covers SPEC §4 rules 2-10; verify Tag/Payload `#[non_exhaustive]` is present (one-way door per SPEC §3.4 / §10.3); verify `Payload::Entr` doc-comment includes the CSPRNG responsibility note from SPEC §3.6; verify `Payload::validate()` is reachable from both encoder and decoder paths.
- Length cap: under 500 words. Categorize critical/important/low/affirmation. Iterate until 0 critical / 0 important.
- Persist the report to `design/agent-reports/phase-1-foundation-review-r1.md`.

- [ ] **Step 2: Apply critical/important findings**

Each critical/important finding gets fixed inline. Re-run `cargo test -p ms-codec --lib` after each fix. Commit fixes as a fixup:

```bash
git -C /scratch/code/shibboleth/mnemonic-secret add <fixed paths>
git -C /scratch/code/shibboleth/mnemonic-secret commit -m "fix(ms-codec): Phase 1 review fixes (rN findings)"
```

- [ ] **Step 3: Iterate review until convergence**

Re-dispatch the reviewer (fresh agent for independence; persist `phase-1-foundation-review-r2.md`, etc.). Stop when a round returns 0 critical / 0 important. Record the convergence round in the phase commit log.

- [ ] **Step 4: Capture remaining nits in FOLLOWUPS**

For any low/nit findings not applied inline, append to `design/FOLLOWUPS.md` at tier `v0.1-nice-to-have`:

```markdown
### `phase-1-low-N` — <one-line title>

- **Surfaced:** Phase 1 review (`design/agent-reports/phase-1-foundation-review-rN.md` finding XX)
- **Where:** `crates/ms-codec/src/<file>.rs:<line>`
- **What:** <1-3 sentences>
- **Why deferred:** <reason>
- **Status:** `open`
- **Tier:** `v0.1-nice-to-have`
```

---

## Phase 2: Envelope — the rust-codex32 contact module

**Goal:** Land `envelope.rs` — the only module that touches `rust-codex32`. Implements `discriminate(s) -> (Tag, &[u8])` (decode-side) and `package(tag, payload_bytes) -> Codex32String` (encode-side), plus the wire-position re-parse helpers that extract hrp/threshold/id/share-index from a validated string. By end of phase, `cargo test` passes for envelope-level unit tests.

**Files:**
- Create: `crates/ms-codec/src/envelope.rs`
- Modify: `crates/ms-codec/src/lib.rs` (add `mod envelope`)

### Task 2.1: Write `envelope.rs` skeleton + wire-position constants

**Files:**
- Create: `crates/ms-codec/src/envelope.rs`

- [ ] **Step 1: Create the file with module-level docs and the wire-position constants**

```rust
//! THE v0.2-MIGRATION SEAM. This is the only module that contacts `rust-codex32`.
//!
//! Why isolated: SPEC §2.2 + §10. When K-of-N share encoding ships in v0.2, only
//! this module changes — `discriminate()` adds prefix-byte dispatch, `package()`
//! gains the `Threshold` parameter. The rest of the crate is untouched.
//!
//! Why wire-position re-parse: `rust-codex32 v0.1.0`'s `Parts` struct
//! (`/tmp/codex32-extract/codex32-0.1.0/src/lib.rs:383-392`) has non-`pub` fields;
//! only `Parts::data() -> Vec<u8>` is publicly accessible. We cannot read
//! `parts.hrp` / `parts.threshold` / `parts.id` / `parts.share_index` from
//! outside the upstream crate. The re-parse below replays what
//! `rust-codex32`'s own `parts_inner` does internally (it's a fast O(n) string
//! parse on a string already proven valid by `Codex32String::from_string`).
//! Re-parse cost is negligible — the upstream `Parts<'s>` is `Copy`.
//!
//! Wire positions (relative to the `1` separator at index `sep`):
//!
//! ```text
//! threshold:   sep + 1                  (1 char; v0.1 = '0')
//! id:          sep + 2 .. sep + 6       (4 chars; type tag in v0.1)
//! share-index: sep + 6                  (1 char; v0.1 = 's')
//! payload:     sep + 7 .. s.len() - 13  (variable; -13 strips short cksum)
//! checksum:    s.len() - 13 .. s.len()  (13 chars; short only in v0.1)
//! ```
//!
//! For v0.1 we never see long-checksum strings (rejected by SPEC §4 rule 9
//! before this module is reached); `CHECKSUM_LEN = 13` is hard-coded.

use crate::consts::{CHECKSUM_LEN_SHORT, HRP, RESERVED_PREFIX, SEPARATOR, SHARE_INDEX_V01, THRESHOLD_V01};
use crate::error::{Error, Result};
use crate::tag::Tag;
use codex32::{Codex32String, Fe};

/// Wire-position offsets relative to the separator index.
const THRESHOLD_OFFSET: usize = 1;
const ID_START_OFFSET: usize = 2;
const ID_END_OFFSET: usize = 6;
const SHARE_INDEX_OFFSET: usize = 6;
const PAYLOAD_START_OFFSET: usize = 7;

// (impl follows in Task 2.2/2.3/2.4)
```

### Task 2.2: Implement wire-position re-parse helpers with TDD

**Files:**
- Modify: `crates/ms-codec/src/envelope.rs`

- [ ] **Step 1: Append the helper functions + tests**

```rust
/// Wire fields extracted from a BIP-93-validated ms1 string.
#[derive(Debug, Clone, Copy)]
pub(crate) struct WireFields<'s> {
    pub hrp: &'s str,
    pub threshold_byte: u8,
    pub id_bytes: [u8; 4],
    pub share_index_byte: u8,
    pub payload_5bit_chars: &'s str,
}

/// Re-parse a string already validated by `Codex32String::from_string` to extract
/// wire-position fields. Caller MUST pass only strings that successfully round-tripped
/// through `rust-codex32` parsing.
///
/// Returns `Err(Error::UnexpectedStringLength)` if the string is too short to contain
/// the fixed wire prefix (defensive only; unreachable for inputs that passed BIP-93 parsing).
pub(crate) fn extract_wire_fields(s: &str) -> Result<WireFields<'_>> {
    let sep = s.rfind(SEPARATOR).ok_or(Error::WrongHrp { got: s.to_string() })?;
    if s.len() < sep + PAYLOAD_START_OFFSET + CHECKSUM_LEN_SHORT {
        return Err(Error::UnexpectedStringLength {
            got: s.len(),
            allowed: crate::consts::VALID_STR_LENGTHS,
        });
    }
    let bytes = s.as_bytes();
    let id_slice = &bytes[sep + ID_START_OFFSET..sep + ID_END_OFFSET];
    Ok(WireFields {
        hrp: &s[..sep],
        threshold_byte: bytes[sep + THRESHOLD_OFFSET],
        id_bytes: [id_slice[0], id_slice[1], id_slice[2], id_slice[3]],
        share_index_byte: bytes[sep + SHARE_INDEX_OFFSET],
        payload_5bit_chars: &s[sep + PAYLOAD_START_OFFSET..s.len() - CHECKSUM_LEN_SHORT],
    })
}

#[cfg(test)]
mod tests_extract {
    use super::*;

    #[test]
    fn bip93_test_vector_1_extracts_correctly() {
        // From rust-codex32 src/lib.rs bip_vector_1 test (BIP-93 vector 1):
        // hrp="ms", threshold=0, id="test", share_index='s', payload=26 'x' chars, cksum=13 chars
        let s = "ms10testsxxxxxxxxxxxxxxxxxxxxxxxxxx4nzvca9cmczlw";
        let fields = extract_wire_fields(s).unwrap();
        assert_eq!(fields.hrp, "ms");
        assert_eq!(fields.threshold_byte, b'0');
        assert_eq!(&fields.id_bytes, b"test");
        assert_eq!(fields.share_index_byte, b's');
        assert_eq!(fields.payload_5bit_chars.len(), 26);
        assert!(fields.payload_5bit_chars.chars().all(|c| c == 'x'));
    }

    #[test]
    fn rejects_too_short_string() {
        // "ms1" alone is below the minimum.
        assert!(matches!(
            extract_wire_fields("ms1"),
            Err(Error::UnexpectedStringLength { .. })
        ));
    }
}
```

- [ ] **Step 2: Run the tests**

```bash
cargo test -p ms-codec --lib envelope::tests_extract 2>&1 | tail -10
```

Expected: 2 tests pass.

### Task 2.3: Implement `discriminate()` (decode-side seam) with TDD

**Files:**
- Modify: `crates/ms-codec/src/envelope.rs`

- [ ] **Step 1: Append discriminate + tests**

```rust
/// Decode-side v0.2-migration seam. Given a BIP-93-validated codex32 string,
/// extract (Tag, payload_bytes_without_prefix). Enforces the v0.1 wire-format
/// invariants: HRP="ms", threshold='0', share-index='s', prefix byte == 0x00.
/// Tag/payload-length validation against RESERVED_TAG_TABLE happens in `decode.rs`.
///
/// In v0.2 this function gains prefix-byte dispatch (`0x00` → v0.1 entr fallback,
/// `0x01` → v0.2 entr-share path, `0x02..` → kind-specific dispatch) per SPEC §5
/// invariant #2.
pub(crate) fn discriminate(c: &Codex32String) -> Result<(Tag, Vec<u8>)> {
    let s = c.to_string();
    let fields = extract_wire_fields(&s)?;

    // Wire-invariant checks (SPEC §4 rules 2, 3, 4).
    if fields.hrp != HRP {
        return Err(Error::WrongHrp { got: fields.hrp.to_string() });
    }
    if fields.threshold_byte != THRESHOLD_V01 {
        return Err(Error::ThresholdNotZero { got: fields.threshold_byte });
    }
    if fields.share_index_byte != SHARE_INDEX_V01 {
        return Err(Error::ShareIndexNotSecret { got: fields.share_index_byte as char });
    }

    // Tag construction (SPEC §4 rule 5; rule 6/7 happen later in decode.rs).
    let tag_bytes = fields.id_bytes;
    let tag_str = std::str::from_utf8(&tag_bytes)
        .map_err(|_| Error::TagInvalidAlphabet { got: tag_bytes })?;
    let tag = Tag::try_new(tag_str)?;

    // Payload extraction via the upstream Parts::data().
    let payload_with_prefix = c.parts().data();
    if payload_with_prefix.is_empty() {
        return Err(Error::ReservedPrefixViolation { got: 0 }); // unreachable for valid strings, defensive
    }

    // Reserved-prefix-byte check (SPEC §4 rule 8).
    if payload_with_prefix[0] != RESERVED_PREFIX {
        return Err(Error::ReservedPrefixViolation { got: payload_with_prefix[0] });
    }

    Ok((tag, payload_with_prefix[1..].to_vec()))
}

#[cfg(test)]
mod tests_discriminate {
    use super::*;

    fn build_v01_entr(entropy: &[u8]) -> Codex32String {
        // [0x00 reserved-prefix] || entropy
        let mut data = vec![RESERVED_PREFIX];
        data.extend_from_slice(entropy);
        Codex32String::from_seed(HRP, 0, "entr", Fe::S, &data).unwrap()
    }

    #[test]
    fn v01_entr_16_round_trips_through_discriminate() {
        let entropy = vec![0xAAu8; 16];
        let c = build_v01_entr(&entropy);
        let (tag, recovered) = discriminate(&c).unwrap();
        assert_eq!(tag, Tag::ENTR);
        assert_eq!(recovered, entropy);
    }

    #[test]
    fn v01_entr_32_round_trips_through_discriminate() {
        let entropy = vec![0x55u8; 32];
        let c = build_v01_entr(&entropy);
        let (tag, recovered) = discriminate(&c).unwrap();
        assert_eq!(tag, Tag::ENTR);
        assert_eq!(recovered, entropy);
    }

    #[test]
    fn discriminate_rejects_non_zero_prefix() {
        // Hand-build a string with prefix 0x01.
        let mut data = vec![0x01u8];
        data.extend_from_slice(&[0xAAu8; 16]);
        let c = Codex32String::from_seed(HRP, 0, "entr", Fe::S, &data).unwrap();
        assert!(matches!(
            discriminate(&c),
            Err(Error::ReservedPrefixViolation { got: 0x01 })
        ));
    }

    #[test]
    fn discriminate_rejects_wrong_hrp() {
        let mut data = vec![RESERVED_PREFIX];
        data.extend_from_slice(&[0xAAu8; 16]);
        let c = Codex32String::from_seed("mq", 0, "entr", Fe::S, &data).unwrap();
        assert!(matches!(discriminate(&c), Err(Error::WrongHrp { .. })));
    }
}
```

- [ ] **Step 2: Run the tests**

```bash
cargo test -p ms-codec --lib envelope::tests_discriminate 2>&1 | tail -15
```

Expected: 4 tests pass. If any fail (esp. the 32-byte case), debug the bit-packing in rust-codex32's `from_seed` against `Parts::data()`.

### Task 2.4: Implement `package()` (encode-side seam) with TDD

**Files:**
- Modify: `crates/ms-codec/src/envelope.rs`

- [ ] **Step 1: Append package + tests**

```rust
/// Encode-side v0.2-migration seam. Given (tag, payload_bytes), build a
/// BIP-93-validated codex32 string with the v0.1 prefix-byte and wire-field
/// fixed values (threshold=0, share-index='s'). The payload bytes here are
/// the raw secret WITHOUT the reserved-prefix byte; this function prepends 0x00.
///
/// In v0.2 this function gains a `Threshold` parameter (per SPEC §5 invariant #4)
/// and the prefix byte becomes the type discriminator.
pub(crate) fn package(tag: Tag, payload_bytes: &[u8]) -> Result<Codex32String> {
    // [0x00 reserved-prefix] || payload
    let mut data = Vec::with_capacity(1 + payload_bytes.len());
    data.push(RESERVED_PREFIX);
    data.extend_from_slice(payload_bytes);

    // Delegate to rust-codex32. v0.1 always uses threshold=0, share=Fe::S.
    // `?` leverages the From<codex32::Error> for Error impl in error.rs.
    Ok(Codex32String::from_seed(HRP, 0, tag.as_str(), Fe::S, &data)?)
}

#[cfg(test)]
mod tests_package {
    use super::*;

    #[test]
    fn package_round_trips_through_discriminate() {
        for len in [16usize, 20, 24, 28, 32] {
            let entropy = vec![0xAAu8; len];
            let c = package(Tag::ENTR, &entropy).unwrap();
            let (tag, recovered) = discriminate(&c).unwrap();
            assert_eq!(tag, Tag::ENTR);
            assert_eq!(recovered, entropy);
        }
    }

    #[test]
    fn package_produces_str_lengths_in_v01_set() {
        let expected_lengths = crate::consts::VALID_STR_LENGTHS;
        for (i, len) in [16usize, 20, 24, 28, 32].iter().enumerate() {
            let entropy = vec![0xAAu8; *len];
            let c = package(Tag::ENTR, &entropy).unwrap();
            let s = c.to_string();
            assert_eq!(
                s.len(),
                expected_lengths[i],
                "length mismatch for {}-B entropy: got {}, expected {}",
                len,
                s.len(),
                expected_lengths[i]
            );
        }
    }
}
```

- [ ] **Step 2: Wire envelope into lib.rs**

Modify `crates/ms-codec/src/lib.rs`. Append:

```rust
mod envelope; // crate-private; v0.2-migration seam
```

- [ ] **Step 3: Run all envelope tests**

```bash
cargo test -p ms-codec --lib envelope 2>&1 | tail -20
```

Expected: 8 tests pass (2 extract + 4 discriminate + 2 package).

### Task 2.5: Phase 2 commit + opus review

- [ ] **Step 1: Commit**

```bash
git -C /scratch/code/shibboleth/mnemonic-secret add \
  crates/ms-codec/src/envelope.rs \
  crates/ms-codec/src/lib.rs

git -C /scratch/code/shibboleth/mnemonic-secret commit -m "$(cat <<'EOF'
feat(ms-codec): Phase 2 envelope (rust-codex32 contact + v0.2-migration seam)

Phase 2 of IMPLEMENTATION_PLAN_ms_v0_1.md. Lands envelope.rs — the only
module that contacts rust-codex32. 8 unit tests passing.

Implements:
- WireFields + extract_wire_fields: wire-position re-parse of a
  BIP-93-validated string (rust-codex32 v0.1.0 Parts fields are
  non-pub; we replay parts_inner's parse externally).
- discriminate(c): decode-side seam. Extracts (Tag, payload_bytes
  without 0x00 prefix). Enforces SPEC §4 wire invariants 2, 3, 4, 8
  (HRP=ms, threshold=0, share=s, prefix=0x00). Tag-table validation
  (rules 5/6/7) defers to decode.rs.
- package(tag, bytes): encode-side seam. Prepends 0x00 and delegates
  to Codex32String::from_seed.

In v0.2, only this module changes: discriminate() adds prefix-byte
dispatch (0x00 -> v0.1 fallback, 0x01 -> entr-share, 0x02+ ->
kind-specific) per SPEC §5 invariant #2; package() gains the
Threshold parameter per invariant #4. Rest of crate untouched.

Module is crate-private (mod envelope; not pub mod) so the v0.2
migration is invisible to callers.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

- [ ] **Step 2: Phase 2 opus review**

Same convention as Task 1.7. Brief: verify envelope.rs's wire-position re-parse safely matches rust-codex32's internal parse (cite line numbers in `/tmp/codex32-extract/codex32-0.1.0/src/lib.rs:178-200`); verify discriminate covers SPEC §4 rules 2-4 + 8; verify package's prefix-byte prepend is correct; verify the v0.2 seam isolation is genuine (no rust-codex32 imports outside envelope.rs). Persist to `design/agent-reports/phase-2-envelope-review-r1.md`. Iterate until convergence.

---

## Phase 3: Encode + Decode

**Goal:** Land `encode.rs` and `decode.rs` — the public API surface for round-tripping. Both delegate to envelope. Decoder applies the full SPEC §4 rule set (the wire-invariant subset is in envelope; tag-table + length-set + payload-length are here). Encoder applies SPEC §3.5 + §3.5.1 (length validation + reserved-tag rejection).

**Files:**
- Create: `crates/ms-codec/src/encode.rs`
- Create: `crates/ms-codec/src/decode.rs`
- Modify: `crates/ms-codec/src/lib.rs`

### Task 3.1: Write `encode.rs` with TDD

**Files:**
- Create: `crates/ms-codec/src/encode.rs`

- [ ] **Step 1: Write encode.rs**

```rust
//! Public encoder. v0.1 entr-only; future kinds in v0.2+ via the envelope seam.

use crate::consts::RESERVED_NOT_EMITTED_V01;
use crate::envelope;
use crate::error::{Error, Result};
use crate::payload::Payload;
use crate::tag::Tag;

/// Encode a `(Tag, Payload)` as a v0.1 ms1 string.
///
/// Per SPEC §3.5 + §3.5.1:
/// - Encoder validates `Payload` length first (rejects out-of-set entr lengths).
/// - Encoder rejects reserved-not-emitted tags symmetrically with the decoder
///   (SPEC §4 rule 7), preventing a v0.1 ms-codec from emitting a string that
///   v0.1 ms-codec itself cannot decode.
pub fn encode(tag: Tag, payload: &Payload) -> Result<String> {
    // §3.5.1: encoder symmetry on reserved-not-emitted tags.
    if RESERVED_NOT_EMITTED_V01.contains(tag.as_bytes()) {
        return Err(Error::ReservedTagNotEmittedInV01 { got: *tag.as_bytes() });
    }
    // §3.5: payload length validation.
    payload.validate()?;
    // Hand off to envelope.
    let c = envelope::package(tag, payload.as_bytes())?;
    Ok(c.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::VALID_STR_LENGTHS;

    #[test]
    fn encode_entr_all_lengths_succeed() {
        for (i, len) in [16usize, 20, 24, 28, 32].iter().enumerate() {
            let p = Payload::Entr(vec![0xAAu8; *len]);
            let s = encode(Tag::ENTR, &p).unwrap();
            assert_eq!(s.len(), VALID_STR_LENGTHS[i]);
            assert!(s.starts_with("ms10entrs"), "got {}", s);
        }
    }

    #[test]
    fn encode_rejects_seed_tag() {
        let p = Payload::Entr(vec![0u8; 16]);
        let seed_tag = Tag::try_new("seed").unwrap();
        assert!(matches!(
            encode(seed_tag, &p),
            Err(Error::ReservedTagNotEmittedInV01 { .. })
        ));
    }

    #[test]
    fn encode_rejects_xprv_tag() {
        let p = Payload::Entr(vec![0u8; 16]);
        let xprv_tag = Tag::try_new("xprv").unwrap();
        assert!(matches!(
            encode(xprv_tag, &p),
            Err(Error::ReservedTagNotEmittedInV01 { .. })
        ));
    }

    #[test]
    fn encode_rejects_off_by_one_entr_length() {
        let p = Payload::Entr(vec![0u8; 17]);
        assert!(matches!(
            encode(Tag::ENTR, &p),
            Err(Error::PayloadLengthMismatch { .. })
        ));
    }
}
```

- [ ] **Step 2: Wire encode into lib.rs**

```rust
pub mod encode;

pub use encode::encode;
```

- [ ] **Step 3: Run tests**

```bash
cargo test -p ms-codec --lib encode::tests 2>&1 | tail -10
```

Expected: 4 tests pass.

### Task 3.2: Write `decode.rs` with TDD

**Files:**
- Create: `crates/ms-codec/src/decode.rs`

- [ ] **Step 1: Write decode.rs**

```rust
//! Public decoder. Applies SPEC §4 validity rules in order.

use crate::consts::{RESERVED_NOT_EMITTED_V01, TAG_ENTR, VALID_STR_LENGTHS};
use crate::envelope;
use crate::error::{Error, Result};
use crate::payload::Payload;
use crate::tag::Tag;
use codex32::Codex32String;

/// Decode a v0.1 ms1 string into `(Tag, Payload)`.
///
/// Rejects per SPEC §4 rules 1-10:
/// 1. Upstream codex32 parse failure (Codex32 variant).
/// 2-4, 8. Wire-invariant violations (delegated to envelope::discriminate).
/// 5-7. Tag-table membership rules (here).
/// 9. Total string length not in v0.1-emittable set (here).
/// 10. Payload byte length mismatch for the tag (here, via Payload::validate()).
pub fn decode(s: &str) -> Result<(Tag, Payload)> {
    // §4 rule 9: total string length must be in the v0.1 set.
    if !VALID_STR_LENGTHS.contains(&s.len()) {
        return Err(Error::UnexpectedStringLength {
            got: s.len(),
            allowed: VALID_STR_LENGTHS,
        });
    }

    // §4 rule 1: delegate parse + checksum to rust-codex32. `?` leverages the
    // From<codex32::Error> for Error impl in error.rs.
    let c = Codex32String::from_string(s.to_string())?;

    // §4 rules 2, 3, 4, 8 + tag-alphabet rule 5: envelope.
    let (tag, payload_bytes) = envelope::discriminate(&c)?;

    // §4 rule 7: reserved-not-emitted tags.
    if RESERVED_NOT_EMITTED_V01.contains(tag.as_bytes()) {
        return Err(Error::ReservedTagNotEmittedInV01 { got: *tag.as_bytes() });
    }

    // §4 rule 6: tag must be in the v0.1 accept set (currently {entr}).
    let payload = match *tag.as_bytes() {
        x if x == TAG_ENTR => {
            let p = Payload::Entr(payload_bytes);
            // §4 rule 10: validate payload length against the tag's expected set.
            p.validate()?;
            p
        }
        _ => {
            return Err(Error::UnknownTag { got: *tag.as_bytes() });
        }
    };

    Ok((tag, payload))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode;

    #[test]
    fn round_trip_entr_all_lengths() {
        for len in [16usize, 20, 24, 28, 32] {
            let entropy = (0..len as u8).map(|i| i.wrapping_mul(7)).collect::<Vec<_>>();
            let p = Payload::Entr(entropy.clone());
            let s = encode::encode(Tag::ENTR, &p).unwrap();
            let (tag, recovered) = decode(&s).unwrap();
            assert_eq!(tag, Tag::ENTR);
            assert_eq!(recovered, p);
        }
    }

    #[test]
    fn decode_rejects_unexpected_length() {
        // 51 chars is not a v0.1 emittable length.
        let s = "ms10entrsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
        assert!(matches!(
            decode(s),
            Err(Error::UnexpectedStringLength { .. })
        ));
    }

    #[test]
    fn decode_rejects_short_seed_string_with_reserved_tag() {
        // Hand-build a 50-char string with id="seed" — 16-B entropy worth.
        // The string-length check passes; tag-rule 7 fails.
        let mut data = vec![0x00u8];
        data.extend_from_slice(&[0xAAu8; 16]);
        let c = Codex32String::from_seed("ms", 0, "seed", codex32::Fe::S, &data).unwrap();
        let s = c.to_string();
        assert_eq!(s.len(), 50, "expected str.len 50 for 16-B + prefix");
        assert!(matches!(
            decode(&s),
            Err(Error::ReservedTagNotEmittedInV01 { .. })
        ));
    }
}
```

- [ ] **Step 2: Wire decode into lib.rs**

```rust
pub mod decode;

pub use decode::decode;
```

- [ ] **Step 3: Run tests**

```bash
cargo test -p ms-codec --lib decode::tests 2>&1 | tail -10
```

Expected: 3 tests pass. The round_trip test should pass for all 5 lengths.

### Task 3.3: Phase 3 commit + opus review

- [ ] **Step 1: Commit**

```bash
git -C /scratch/code/shibboleth/mnemonic-secret add \
  crates/ms-codec/src/encode.rs \
  crates/ms-codec/src/decode.rs \
  crates/ms-codec/src/lib.rs

git -C /scratch/code/shibboleth/mnemonic-secret commit -m "$(cat <<'EOF'
feat(ms-codec): Phase 3 encode + decode public API

Phase 3 of IMPLEMENTATION_PLAN_ms_v0_1.md. Lands the public encode/decode
surface. 7 unit tests passing (4 encode + 3 decode), full round-trip
verified for all 5 entr lengths {16, 20, 24, 28, 32}.

encode.rs:
- encode(tag, payload) -> Result<String>
- §3.5.1 encoder symmetry: rejects reserved-not-emitted tags
  (Tag::try_new("seed"/"xprv"/"mnem"/"prvk")) with
  Error::ReservedTagNotEmittedInV01 — prevents emitting strings
  v0.1 ms-codec itself cannot decode.
- §3.5 payload length validation via Payload::validate().
- Delegates wire construction to envelope::package.

decode.rs:
- decode(s) -> Result<(Tag, Payload)>
- Applies SPEC §4 rules in order: rule 9 (total length) before parse;
  rule 1 (parse) via rust-codex32; rules 2/3/4/8 + alphabet rule 5
  via envelope::discriminate; rules 6/7 (tag table) and rule 10
  (payload length) here.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

- [ ] **Step 2: Opus review**

Brief: verify SPEC §4 rule ordering in decode.rs matches the SPEC; verify all 10 §4 rules are exercised by some test (mentally trace each rule to a specific test); verify encode/decode are genuine inverses (round-trip property holds for all 5 entr lengths). Persist to `design/agent-reports/phase-3-encode-decode-review-r1.md`. Iterate until convergence.

---

## Phase 4: Inspect + lib.rs polish

**Goal:** Land `inspect.rs` (`InspectReport` for debugging / future ms-cli) and finalize `lib.rs` re-exports + crate-level docs.

**Files:**
- Create: `crates/ms-codec/src/inspect.rs`
- Modify: `crates/ms-codec/src/lib.rs`

### Task 4.1: Write `inspect.rs` with TDD

**Files:**
- Create: `crates/ms-codec/src/inspect.rs`

- [ ] **Step 1: Write inspect.rs**

```rust
//! Structural inspection of an ms1 string for debugging / future ms-cli.

use crate::envelope;
use crate::error::{Error, Result};
use crate::tag::Tag;
use codex32::Codex32String;

/// Structural dump of a parsed ms1 string. `#[non_exhaustive]` per SPEC §10
/// — v0.2+ may add fields (share-index detail, threshold-layer hints,
/// derivation metadata).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct InspectReport {
    /// Expected "ms" in v0.1.
    pub hrp: String,
    /// Expected 0 in v0.1.
    pub threshold: u8,
    /// The parsed type tag (id field).
    pub tag: Tag,
    /// Expected 's' in v0.1.
    pub share_index: char,
    /// 0x00 in v0.1 (reserved); becomes type discriminator in v0.2+.
    pub prefix_byte: u8,
    /// Payload bytes after the prefix byte.
    pub payload_bytes: Vec<u8>,
    /// BCH verification result. True if the upstream codex32 parser accepted.
    pub checksum_valid: bool,
}

/// Inspect an ms1 string. Less strict than `decode()`: returns a report even
/// for strings that would fail decoder validity rules (e.g., wrong threshold,
/// reserved-not-emitted tag, non-zero prefix byte) — caller can examine the
/// fields to diagnose what's wrong. Still requires a valid BIP-93 parse.
pub fn inspect(s: &str) -> Result<InspectReport> {
    // `?` leverages From<codex32::Error> for Error.
    let c = Codex32String::from_string(s.to_string())?;
    let s_owned = c.to_string();
    let fields = envelope::extract_wire_fields(&s_owned)?;

    // For tag construction in inspect we accept whatever bytes were on the wire
    // (alphabet-valid or not) — surfacing the raw observation is the point.
    let tag = match std::str::from_utf8(&fields.id_bytes) {
        Ok(t) => Tag::try_new(t).unwrap_or_else(|_| Tag::from_raw_bytes(fields.id_bytes)),
        Err(_) => Tag::from_raw_bytes(fields.id_bytes),
    };

    let payload_with_prefix = c.parts().data();
    let (prefix_byte, payload_bytes) = if payload_with_prefix.is_empty() {
        (0u8, Vec::new())
    } else {
        (payload_with_prefix[0], payload_with_prefix[1..].to_vec())
    };

    Ok(InspectReport {
        hrp: fields.hrp.to_string(),
        threshold: fields.threshold_byte - b'0', // ASCII to digit
        tag,
        share_index: fields.share_index_byte as char,
        prefix_byte,
        payload_bytes,
        checksum_valid: true, // if from_string accepted, BCH was valid
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{encode, payload::Payload};

    #[test]
    fn inspect_v01_entr_returns_expected_fields() {
        let entropy = vec![0xAAu8; 16];
        let s = encode::encode(Tag::ENTR, &Payload::Entr(entropy.clone())).unwrap();
        let r = inspect(&s).unwrap();
        assert_eq!(r.hrp, "ms");
        assert_eq!(r.threshold, 0);
        assert_eq!(r.tag, Tag::ENTR);
        assert_eq!(r.share_index, 's');
        assert_eq!(r.prefix_byte, 0x00);
        assert_eq!(r.payload_bytes, entropy);
        assert!(r.checksum_valid);
    }

    #[test]
    fn inspect_returns_report_for_decoder_rejects() {
        // A non-zero-prefix string: decode() rejects, inspect() returns the report.
        let mut data = vec![0x01u8];
        data.extend_from_slice(&[0xAAu8; 16]);
        let c = Codex32String::from_seed("ms", 0, "entr", codex32::Fe::S, &data).unwrap();
        let r = inspect(&c.to_string()).unwrap();
        assert_eq!(r.prefix_byte, 0x01); // would fail decode rule 8, inspect surfaces it
    }
}
```

- [ ] **Step 2: Wire inspect + finalize lib.rs**

Replace `crates/ms-codec/src/lib.rs` with the final version:

```rust
//! `ms-codec` — reference implementation of the **ms1** backup format (HRP `ms`).
//!
//! ms1 is a Bitcoin self-custody backup format for BIP-39 entropy, layered atop
//! BIP-93 codex32 via Andrew Poelstra's `rust-codex32` crate (CC0). Designed for
//! steel-plate engraving alongside sibling formats `mk1` (xpubs) and `md1`
//! (descriptors). Every wire-format decision is judged against "does this make
//! a steel-plate backup more correct, or less?"
//!
//! See [`SPEC_ms_v0_1.md`](../../design/SPEC_ms_v0_1.md) for the full wire-format
//! specification and [`MIGRATION.md`](../../MIGRATION.md) for the v0.1 → v0.2
//! K-of-N share-encoding migration contract.
//!
//! # Quickstart
//!
//! ```
//! use ms_codec::{encode, decode, Payload, Tag};
//!
//! let entropy = vec![0xAAu8; 16]; // 12-word BIP-39 entropy
//! let s = encode(Tag::ENTR, &Payload::Entr(entropy.clone())).unwrap();
//! assert_eq!(s.len(), 50); // 12-word entr = 50-char ms1 string
//!
//! let (tag, payload) = decode(&s).unwrap();
//! assert_eq!(tag, Tag::ENTR);
//! assert_eq!(payload, Payload::Entr(entropy));
//! ```
//!
//! # v0.1 scope
//!
//! - **In:** BIP-39 entropy (16/20/24/28/32 B). Tag: `entr`.
//! - **Out:** Direct BIP-32 master seed (64 B) and serialized xpriv (78 B) —
//!   reserved-not-emitted in v0.1; deferred to v0.2+ with separate framing
//!   (they overflow BIP-93 codex32's length brackets when prepended with
//!   the v0.2-migration prefix byte). The master-seed backup use case is
//!   preserved via the application-layer routing
//!   `BIP-39 phrase → entropy → ms1 entr → engrave → recover → BIP-39 mnemonic
//!   → PBKDF2 → master seed`. See SPEC §1.2.

#![cfg_attr(not(test), deny(missing_docs))]

pub mod consts;
pub mod decode;
pub mod encode;
pub mod error;
pub mod inspect;
pub mod payload;
pub mod tag;

mod envelope; // crate-private; v0.2-migration seam

pub use decode::decode;
pub use encode::encode;
pub use error::{Error, Result};
pub use inspect::{inspect, InspectReport};
pub use payload::{Payload, PayloadKind};
pub use tag::Tag;
```

- [ ] **Step 3: Run all tests**

```bash
cargo test -p ms-codec 2>&1 | tail -20
```

Expected: 26 unit tests pass (5 tag + 4 payload + 8 envelope + 4 encode + 3 decode + 2 inspect) plus 1 doc-test from the lib.rs Quickstart.

### Task 4.2: Phase 4 commit + opus review

- [ ] **Step 1: Commit**

```bash
git -C /scratch/code/shibboleth/mnemonic-secret add \
  crates/ms-codec/src/inspect.rs \
  crates/ms-codec/src/lib.rs

git -C /scratch/code/shibboleth/mnemonic-secret commit -m "$(cat <<'EOF'
feat(ms-codec): Phase 4 inspect + lib.rs polish

Phase 4 of IMPLEMENTATION_PLAN_ms_v0_1.md. Lands inspect.rs and
finalizes the public API surface in lib.rs. ~26 unit + 1 doc test
passing.

inspect.rs:
- InspectReport (#[non_exhaustive] per SPEC §10): hrp, threshold, tag,
  share_index, prefix_byte, payload_bytes, checksum_valid.
- inspect(s) -> Result<InspectReport>: less strict than decode() —
  returns the report even for strings that would fail decoder validity
  rules. Caller examines the fields to diagnose what's wrong.

lib.rs:
- Quickstart doc-test for the round-trip path.
- Re-exports: encode, decode, inspect, Tag, Payload, PayloadKind,
  InspectReport, Error, Result.
- envelope is crate-private (mod, not pub mod) so the v0.2 seam is
  invisible to callers.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

- [ ] **Step 2: Opus review**

Brief: verify InspectReport's field set matches SPEC §10 sketch; verify inspect() is genuinely more permissive than decode() (returns reports for cases that decode() rejects); verify the Quickstart doc-test compiles and passes; verify all SPEC §10 re-exports are present and consistent. Persist to `design/agent-reports/phase-4-inspect-review-r1.md`.

---

## Phase 5: Negative-vector + forward-compat tests

**Goal:** Land integration tests under `crates/ms-codec/tests/` covering the SPEC §10.2 negative-vector and forward-compat smoke requirements. One negative case per §4 rule.

**Files:**
- Create: `crates/ms-codec/tests/negative.rs`
- Create: `crates/ms-codec/tests/forward_compat.rs`
- Create: `crates/ms-codec/tests/round_trip.rs`

### Task 5.1: Write `tests/negative.rs`

**Files:**
- Create: `crates/ms-codec/tests/negative.rs`

- [ ] **Step 1: Write tests/negative.rs**

```rust
//! One negative test per SPEC §4 decoder rule. Each test hand-constructs an
//! ms1 string that violates exactly one rule and asserts the corresponding
//! Error variant.

use codex32::{Codex32String, Fe};
use ms_codec::{decode, Error};

const VALID_PREFIX: u8 = 0x00;
const ENTROPY_16: &[u8] = &[0xAAu8; 16];

fn build_with(hrp: &str, threshold: usize, id: &str, share: Fe, prefix: u8, payload: &[u8]) -> String {
    let mut data = vec![prefix];
    data.extend_from_slice(payload);
    Codex32String::from_seed(hrp, threshold, id, share, &data)
        .unwrap()
        .to_string()
}

#[test]
fn rule_1_invalid_checksum_rejected() {
    // Take a valid string and flip the last char to break BCH.
    let s = build_with("ms", 0, "entr", Fe::S, VALID_PREFIX, ENTROPY_16);
    let mut bytes = s.into_bytes();
    let last = bytes.len() - 1;
    bytes[last] = if bytes[last] == b'q' { b'p' } else { b'q' };
    let bad = String::from_utf8(bytes).unwrap();
    assert!(matches!(decode(&bad), Err(Error::Codex32(_))));
}

#[test]
fn rule_2_wrong_hrp_rejected() {
    // Build with HRP "mq" instead of "ms". HRP byte length is the same (2);
    // total string length is identical to the "ms" case (50). Length check
    // passes, upstream parse passes, our envelope::discriminate fires
    // WrongHrp deterministically.
    let s = build_with("mq", 0, "entr", Fe::S, VALID_PREFIX, ENTROPY_16);
    assert_eq!(s.len(), 50, "sanity: HRP swap doesn't change string length");
    assert!(matches!(decode(&s), Err(Error::WrongHrp { .. })));
}

#[test]
fn rule_3_threshold_not_zero_rejected() {
    // Threshold = 2 with share_index = Fe::A produces a valid-length string
    // (9 fixed + 28 payload + 13 cksum = 50, in VALID_STR_LENGTHS). Length
    // check passes; upstream from_string accepts threshold=2 + share=A
    // (parts_inner rejects threshold=0 + share!=S only); our envelope
    // discriminate fires ThresholdNotZero deterministically.
    let s = build_with("ms", 2, "entr", Fe::A, VALID_PREFIX, ENTROPY_16);
    assert_eq!(s.len(), 50, "sanity: 16-B + 0x00 prefix in threshold-2 form is 50 chars");
    assert!(matches!(decode(&s), Err(Error::ThresholdNotZero { .. })));
}

#[test]
fn rule_4_share_index_not_secret_rejected() {
    // For threshold=0 with share_index != Fe::S, BIP-93 itself rejects at
    // upstream parse (rust-codex32 v0.1.0 lib.rs:202-204:
    // `if ret.threshold == 0 && ret.share_index != Fe::S { return InvalidShareIndex(...) }`).
    // Build a valid-length, valid-checksum string with share=Fe::C and confirm
    // our decoder surfaces Error::Codex32 wrapping the upstream error.
    let s = build_with("ms", 0, "entr", Fe::C, VALID_PREFIX, ENTROPY_16);
    assert_eq!(s.len(), 50, "sanity: valid v0.1 length so the rule 9 length-check passes");
    assert!(matches!(decode(&s), Err(Error::Codex32(_))));
}

#[test]
fn rule_5_tag_invalid_alphabet_unreachable_via_decode() {
    // Tag bytes outside the codex32 alphabet would be rejected at upstream parse
    // (rust-codex32 validates every char in the data part is in the alphabet).
    // Our rule 5 path is therefore defensive-only. Skip with a comment note.
}

#[test]
fn rule_6_unknown_tag_rejected() {
    // Build with id="abcd" (alphabet-valid but not in RESERVED_TAG_TABLE).
    let s = build_with("ms", 0, "abcd", Fe::S, VALID_PREFIX, ENTROPY_16);
    assert!(matches!(decode(&s), Err(Error::UnknownTag { .. })));
}

#[test]
fn rule_7_reserved_not_emitted_tags_rejected() {
    for reserved in ["seed", "xprv", "mnem", "prvk"] {
        let s = build_with("ms", 0, reserved, Fe::S, VALID_PREFIX, ENTROPY_16);
        let err = decode(&s).unwrap_err();
        assert!(
            matches!(err, Error::ReservedTagNotEmittedInV01 { got: _ }),
            "tag {:?}: expected ReservedTagNotEmittedInV01, got {:?}",
            reserved,
            err
        );
    }
}

#[test]
fn rule_8_reserved_prefix_violation_rejected() {
    // Build with prefix byte = 0x01 instead of 0x00.
    let s = build_with("ms", 0, "entr", Fe::S, 0x01, ENTROPY_16);
    assert!(matches!(
        decode(&s),
        Err(Error::ReservedPrefixViolation { got: 0x01 })
    ));
}

#[test]
fn rule_9_unexpected_string_length_rejected() {
    // 51 chars: not a v0.1 emittable length. Use random-looking valid base32 chars.
    let s = "ms10entrsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
    assert_eq!(s.len(), 51);
    assert!(matches!(
        decode(s),
        Err(Error::UnexpectedStringLength { got: 51, .. })
    ));
}

#[test]
fn rule_10_payload_length_mismatch_unreachable_via_decode() {
    // Rule 10 (Payload::validate post-extraction) cannot be reached for valid inputs
    // because rule 9 (string length) fires first. The two rules are
    // length-set-equivalent: VALID_STR_LENGTHS bijects with VALID_ENTR_LENGTHS via
    // the 22-fixed-char prefix. Defensive-only path.
}
```

- [ ] **Step 2: Run negative tests**

```bash
cargo test -p ms-codec --test negative 2>&1 | tail -15
```

Expected: 10 tests pass (rules 5 + 10 are no-op tests that just document unreachability).

### Task 5.2: Write `tests/forward_compat.rs`

**Files:**
- Create: `crates/ms-codec/tests/forward_compat.rs`

- [ ] **Step 1: Write tests/forward_compat.rs**

```rust
//! SPEC §10.2 forward-compat smoke test: encode a v0.1 string, manually flip
//! the prefix byte to 0x01, confirm decoder rejects with
//! Error::ReservedPrefixViolation. Locks the v0.1 ↔ v0.2 contract.

use codex32::{Codex32String, Fe};
use ms_codec::{decode, encode, Error, Payload, Tag};

#[test]
fn flipping_prefix_byte_to_v02_value_rejects_at_v01_decoder() {
    // Encode a real v0.1 string.
    let entropy = vec![0xAAu8; 16];
    let _s_v01 = encode(Tag::ENTR, &Payload::Entr(entropy.clone())).unwrap();

    // Hand-build the same wire shape but with prefix byte = 0x01 (the future v0.2
    // entr discriminator). v0.1 decoder MUST reject this — that's the migration
    // contract from SPEC §5 invariant #1.
    let mut data = vec![0x01u8];
    data.extend_from_slice(&entropy);
    let c = Codex32String::from_seed("ms", 0, "entr", Fe::S, &data).unwrap();
    let s_v02_shaped = c.to_string();

    assert_eq!(s_v02_shaped.len(), 50);
    assert!(matches!(
        decode(&s_v02_shaped),
        Err(Error::ReservedPrefixViolation { got: 0x01 })
    ));
}

#[test]
fn all_non_zero_prefix_bytes_rejected_in_v01() {
    // Defense-in-depth: every non-zero prefix value is rejected, not just 0x01.
    let entropy = [0xAAu8; 16];
    for prefix in 1u8..=255 {
        let mut data = vec![prefix];
        data.extend_from_slice(&entropy);
        let c = Codex32String::from_seed("ms", 0, "entr", Fe::S, &data).unwrap();
        let err = decode(&c.to_string()).unwrap_err();
        assert!(
            matches!(err, Error::ReservedPrefixViolation { got } if got == prefix),
            "prefix 0x{:02x}: expected ReservedPrefixViolation, got {:?}",
            prefix,
            err
        );
    }
}
```

- [ ] **Step 2: Run forward-compat tests**

```bash
cargo test -p ms-codec --test forward_compat 2>&1 | tail -10
```

Expected: 2 tests pass.

### Task 5.3: Write `tests/round_trip.rs` (proptest)

**Files:**
- Create: `crates/ms-codec/tests/round_trip.rs`

- [ ] **Step 1: Write tests/round_trip.rs**

```rust
//! Property-based round-trip tests: encode → decode → assert equal payload + tag,
//! across all 5 entr byte lengths.

use ms_codec::{decode, encode, Payload, Tag};
use proptest::prelude::*;

fn entropy_strategy(len: usize) -> impl Strategy<Value = Vec<u8>> {
    proptest::collection::vec(any::<u8>(), len..=len)
}

proptest! {
    #[test]
    fn round_trip_entr_16(entropy in entropy_strategy(16)) {
        let p = Payload::Entr(entropy.clone());
        let s = encode(Tag::ENTR, &p).unwrap();
        let (tag, recovered) = decode(&s).unwrap();
        prop_assert_eq!(tag, Tag::ENTR);
        prop_assert_eq!(recovered, Payload::Entr(entropy));
    }

    #[test]
    fn round_trip_entr_20(entropy in entropy_strategy(20)) {
        let p = Payload::Entr(entropy.clone());
        let s = encode(Tag::ENTR, &p).unwrap();
        let (tag, recovered) = decode(&s).unwrap();
        prop_assert_eq!(tag, Tag::ENTR);
        prop_assert_eq!(recovered, Payload::Entr(entropy));
    }

    #[test]
    fn round_trip_entr_24(entropy in entropy_strategy(24)) {
        let p = Payload::Entr(entropy.clone());
        let s = encode(Tag::ENTR, &p).unwrap();
        let (tag, recovered) = decode(&s).unwrap();
        prop_assert_eq!(tag, Tag::ENTR);
        prop_assert_eq!(recovered, Payload::Entr(entropy));
    }

    #[test]
    fn round_trip_entr_28(entropy in entropy_strategy(28)) {
        let p = Payload::Entr(entropy.clone());
        let s = encode(Tag::ENTR, &p).unwrap();
        let (tag, recovered) = decode(&s).unwrap();
        prop_assert_eq!(tag, Tag::ENTR);
        prop_assert_eq!(recovered, Payload::Entr(entropy));
    }

    #[test]
    fn round_trip_entr_32(entropy in entropy_strategy(32)) {
        let p = Payload::Entr(entropy.clone());
        let s = encode(Tag::ENTR, &p).unwrap();
        let (tag, recovered) = decode(&s).unwrap();
        prop_assert_eq!(tag, Tag::ENTR);
        prop_assert_eq!(recovered, Payload::Entr(entropy));
    }
}
```

- [ ] **Step 2: Run round-trip proptests**

```bash
cargo test -p ms-codec --test round_trip 2>&1 | tail -10
```

Expected: 5 proptests pass (256 cases each by default = 1280 total).

### Task 5.4: Phase 5 commit + opus review

- [ ] **Step 1: Commit**

```bash
git -C /scratch/code/shibboleth/mnemonic-secret add \
  crates/ms-codec/tests/negative.rs \
  crates/ms-codec/tests/forward_compat.rs \
  crates/ms-codec/tests/round_trip.rs

git -C /scratch/code/shibboleth/mnemonic-secret commit -m "$(cat <<'EOF'
test(ms-codec): Phase 5 negative + forward-compat + round-trip tests

Phase 5 of IMPLEMENTATION_PLAN_ms_v0_1.md. Adds the integration test
surface from SPEC §10.2.

tests/negative.rs (10 tests):
- One test per SPEC §4 decoder rule (1-10).
- Rules 5 and 10 are documented as defensive-only / unreachable from
  the public decode() entry point (rule 5 is rejected at upstream
  parse; rule 10 is shadowed by rule 9 since string-length and
  payload-length sets are bijective).
- Rule 7 covers all 4 reserved-not-emitted tags (seed, xprv, mnem, prvk).

tests/forward_compat.rs (2 tests):
- Single-byte flip 0x00 → 0x01 (the future v0.2 entr discriminator):
  decoder rejects with ReservedPrefixViolation. Locks the v0.1 ↔ v0.2
  contract from SPEC §5 invariant #1.
- Defense-in-depth: every prefix byte 0x01..0xFF rejected.

tests/round_trip.rs (5 proptests):
- proptest, 256 cases each: encode → decode → assert equal for each
  entr byte length {16, 20, 24, 28, 32}. 1280 random inputs total.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

- [ ] **Step 2: Opus review**

Brief: verify each SPEC §4 rule has at least one negative-test exercise (or is documented as unreachable with a justification); verify the forward-compat test exhaustively probes the prefix-byte space; verify proptest case counts are sufficient (default 256 should be enough; bump to 1024 if reviewer flags). Persist to `design/agent-reports/phase-5-tests-review-r1.md`.

---

## Phase 6: BIP-39 integration test

**Goal:** Land a real BIP-39 phrase round-trip test using the `bip39` crate as a dev-dep. Catches any entropy-bit-misalignment regression that might exist between rust-codex32's bit-packing and our wire format.

**Files:**
- Modify: `crates/ms-codec/Cargo.toml`
- Create: `crates/ms-codec/tests/bip39_integration.rs`

### Task 6.1: Add bip39 dev-dep

**Files:**
- Modify: `crates/ms-codec/Cargo.toml`

- [ ] **Step 1: Append bip39 to dev-dependencies**

Modify `crates/ms-codec/Cargo.toml`:

```toml
[dev-dependencies]
proptest = "1"
bip39 = "2"
```

- [ ] **Step 2: Verify**

```bash
cargo build -p ms-codec --tests 2>&1 | tail -5
```

Expected: clean.

### Task 6.2: Write tests/bip39_integration.rs with TDD

**Files:**
- Create: `crates/ms-codec/tests/bip39_integration.rs`

- [ ] **Step 1: Write the test**

```rust
//! SPEC §10.2 BIP-39 round-trip integration: take an English BIP-39 mnemonic,
//! extract entropy, encode as ms1 entr, decode, re-derive the mnemonic,
//! confirm string-exact match. Catches any entropy-bit-misalignment regression.

use bip39::{Language, Mnemonic};
use ms_codec::{decode, encode, Payload, Tag};

#[test]
fn bip39_12_word_round_trip_english() {
    let phrase = "abandon abandon abandon abandon abandon abandon \
                  abandon abandon abandon abandon abandon about";
    let mnemonic = Mnemonic::parse_in(Language::English, phrase).unwrap();
    let entropy = mnemonic.to_entropy();
    assert_eq!(entropy.len(), 16, "12 words = 128 bits = 16 bytes");

    let s = encode(Tag::ENTR, &Payload::Entr(entropy.clone())).unwrap();
    assert_eq!(s.len(), 50, "12-word entr = 50-char ms1 string");

    let (tag, recovered_payload) = decode(&s).unwrap();
    assert_eq!(tag, Tag::ENTR);
    let recovered_entropy = match recovered_payload {
        Payload::Entr(b) => b,
    };
    assert_eq!(recovered_entropy, entropy);

    let recovered_mnemonic =
        Mnemonic::from_entropy_in(Language::English, &recovered_entropy).unwrap();
    assert_eq!(recovered_mnemonic.to_string(), phrase);
}

#[test]
fn bip39_24_word_round_trip_english() {
    let phrase = "abandon abandon abandon abandon abandon abandon \
                  abandon abandon abandon abandon abandon abandon \
                  abandon abandon abandon abandon abandon abandon \
                  abandon abandon abandon abandon abandon art";
    let mnemonic = Mnemonic::parse_in(Language::English, phrase).unwrap();
    let entropy = mnemonic.to_entropy();
    assert_eq!(entropy.len(), 32);

    let s = encode(Tag::ENTR, &Payload::Entr(entropy.clone())).unwrap();
    assert_eq!(s.len(), 75);

    let (_tag, recovered_payload) = decode(&s).unwrap();
    let recovered_entropy = match recovered_payload {
        Payload::Entr(b) => b,
    };
    assert_eq!(recovered_entropy, entropy);

    let recovered_mnemonic =
        Mnemonic::from_entropy_in(Language::English, &recovered_entropy).unwrap();
    assert_eq!(recovered_mnemonic.to_string(), phrase);
}

#[test]
fn bip39_random_entropy_round_trips_at_all_word_counts() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Deterministic pseudo-random entropy from a fixed seed (no rand dep needed).
    fn det_bytes(seed: u64, len: usize) -> Vec<u8> {
        let mut out = Vec::with_capacity(len);
        let mut h = seed;
        while out.len() < len {
            let mut hasher = DefaultHasher::new();
            h.hash(&mut hasher);
            let v = hasher.finish().to_le_bytes();
            out.extend_from_slice(&v);
            h = h.wrapping_add(0x9E3779B97F4A7C15);
        }
        out.truncate(len);
        out
    }

    for (word_count, byte_len) in [(12usize, 16usize), (15, 20), (18, 24), (21, 28), (24, 32)] {
        let entropy = det_bytes(0xDEADBEEF + word_count as u64, byte_len);
        let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy).unwrap();
        let original_phrase = mnemonic.to_string();
        assert_eq!(original_phrase.split_whitespace().count(), word_count);

        let s = encode(Tag::ENTR, &Payload::Entr(entropy.clone())).unwrap();
        let (_tag, recovered_payload) = decode(&s).unwrap();
        let recovered_entropy = match recovered_payload {
            Payload::Entr(b) => b,
        };
        assert_eq!(recovered_entropy, entropy);

        let recovered_mnemonic =
            Mnemonic::from_entropy_in(Language::English, &recovered_entropy).unwrap();
        assert_eq!(recovered_mnemonic.to_string(), original_phrase);
    }
}
```

- [ ] **Step 2: Run BIP-39 integration tests**

```bash
cargo test -p ms-codec --test bip39_integration 2>&1 | tail -10
```

Expected: 3 tests pass. If any fail, the most likely culprit is bit-packing misalignment between `rust-codex32`'s `from_seed` / `Parts::data()` and our prefix-byte handling; verify by adding a debug print of the round-tripped bytes vs the input bytes.

### Task 6.3: Phase 6 commit + opus review

- [ ] **Step 1: Commit**

```bash
git -C /scratch/code/shibboleth/mnemonic-secret add \
  crates/ms-codec/Cargo.toml \
  crates/ms-codec/tests/bip39_integration.rs

git -C /scratch/code/shibboleth/mnemonic-secret commit -m "$(cat <<'EOF'
test(ms-codec): Phase 6 BIP-39 integration round-trip

Phase 6 of IMPLEMENTATION_PLAN_ms_v0_1.md. Adds bip39 = "2" as a
dev-dep and 3 integration tests proving the SPEC §1.2 recovery routing
works end-to-end:

  BIP-39 mnemonic (English)
    -> entropy bytes
    -> ms1 entr string (encode)
    -> decode
    -> recovered entropy bytes (byte-exact)
    -> re-derived mnemonic (string-exact match against original)

Tests:
- 12-word "abandon... about" canonical BIP-39 vector
- 24-word "abandon... art" canonical 256-bit vector
- All 5 word counts (12/15/18/21/24) with deterministic
  pseudo-random entropy

Catches any entropy-bit-misalignment regression between
rust-codex32's bit-packing and our prefix-byte handling.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

- [ ] **Step 2: Opus review**

Brief: verify the test exhaustively covers all 5 BIP-39 word counts; verify it asserts string-exact match (not just byte-exact entropy); verify the canonical "abandon... about" / "abandon... art" vectors are used; flag if any non-English wordlists should also be tested. Persist to `design/agent-reports/phase-6-bip39-integration-review-r1.md`.

---

## Phase 7: Vector corpus + release prep

**Goal:** Land the SHA-pinned test-vector corpus, add CHANGELOG / README, run all CI gates, do a `cargo publish --dry-run`. End-state: ms-codec is ready to tag `ms-codec-v0.1.0`.

**Files:**
- Create: `crates/ms-codec/tests/vectors.rs`
- Create: `crates/ms-codec/tests/vectors/v0.1.json`
- Modify: `CHANGELOG.md`
- Modify: `README.md`
- Modify: `crates/ms-codec/Cargo.toml` (add `serde` + `serde_json` dev-deps)

### Task 7.1: Generate the v0.1 vector corpus

**Files:**
- Create: `crates/ms-codec/tests/vectors/v0.1.json`

- [ ] **Step 1: Write a one-shot binary that emits canonical vectors**

Create `/tmp/ms-codec-vectorgen/Cargo.toml`:

```toml
[package]
name = "vectorgen"
version = "0.0.0"
edition = "2021"

[dependencies]
ms-codec = { path = "/scratch/code/shibboleth/mnemonic-secret/crates/ms-codec" }
bip39 = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

And `/tmp/ms-codec-vectorgen/src/main.rs`:

```rust
use bip39::{Language, Mnemonic};
use ms_codec::{encode, Payload, Tag};
use serde::Serialize;

#[derive(Serialize)]
struct Vector {
    description: String,
    mnemonic: String,
    entropy_hex: String,
    ms1: String,
}

fn main() {
    let vectors = vec![
        ("12-word abandon canonical", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"),
        ("24-word abandon canonical", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art"),
    ];

    let out: Vec<Vector> = vectors
        .iter()
        .map(|(desc, phrase)| {
            let m = Mnemonic::parse_in(Language::English, phrase).unwrap();
            let entropy = m.to_entropy();
            let s = encode(Tag::ENTR, &Payload::Entr(entropy.clone())).unwrap();
            Vector {
                description: desc.to_string(),
                mnemonic: phrase.to_string(),
                entropy_hex: entropy.iter().map(|b| format!("{:02x}", b)).collect(),
                ms1: s,
            }
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&out).unwrap());
}
```

Run it:

```bash
cd /tmp/ms-codec-vectorgen && cargo run --quiet > /scratch/code/shibboleth/mnemonic-secret/crates/ms-codec/tests/vectors/v0.1.json
```

- [ ] **Step 2: Inspect the generated file**

```bash
cat /scratch/code/shibboleth/mnemonic-secret/crates/ms-codec/tests/vectors/v0.1.json
```

Expected: a JSON array of 2 vectors. Eyeball the mnemonic / entropy_hex / ms1 fields for each.

### Task 7.2: Write tests/vectors.rs replay test

**Files:**
- Create: `crates/ms-codec/tests/vectors.rs`
- Modify: `crates/ms-codec/Cargo.toml`

- [ ] **Step 1: Add serde + serde_json dev-deps**

Modify `crates/ms-codec/Cargo.toml`:

```toml
[dev-dependencies]
proptest = "1"
bip39 = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

- [ ] **Step 2: Write tests/vectors.rs**

```rust
//! Versioned vector corpus replay. Pinned at v0.1.0 release per RELEASE_PROCESS.md.

use ms_codec::{decode, encode, Payload, Tag};
use serde::Deserialize;

#[derive(Deserialize)]
struct Vector {
    description: String,
    mnemonic: String,
    entropy_hex: String,
    ms1: String,
}

fn load_v01_corpus() -> Vec<Vector> {
    let raw = include_str!("vectors/v0.1.json");
    serde_json::from_str(raw).expect("v0.1.json parse")
}

fn decode_hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn v01_corpus_round_trips() {
    let corpus = load_v01_corpus();
    assert!(!corpus.is_empty(), "v0.1.json must have at least one vector");

    for v in &corpus {
        let entropy = decode_hex(&v.entropy_hex);
        let s = encode(Tag::ENTR, &Payload::Entr(entropy.clone()))
            .unwrap_or_else(|e| panic!("{}: encode failed: {:?}", v.description, e));
        assert_eq!(s, v.ms1, "{}: encoded ms1 mismatch", v.description);

        let (tag, payload) = decode(&v.ms1)
            .unwrap_or_else(|e| panic!("{}: decode failed: {:?}", v.description, e));
        assert_eq!(tag, Tag::ENTR);
        assert_eq!(payload, Payload::Entr(entropy));
    }
}
```

- [ ] **Step 3: Run vector tests**

```bash
cargo test -p ms-codec --test vectors 2>&1 | tail -10
```

Expected: 1 test passes; iterates over both corpus vectors.

### Task 7.3: SHA-pin the vector corpus

- [ ] **Step 1: Compute SHA-256**

```bash
sha256sum /scratch/code/shibboleth/mnemonic-secret/crates/ms-codec/tests/vectors/v0.1.json
```

Record the hash. This will go in CHANGELOG.md.

### Task 7.4: Write CHANGELOG.md

**Files:**
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Replace CHANGELOG.md content**

```markdown
# Changelog

All notable changes to `ms-codec` are documented here. Format mirrors the
descriptor-mnemonic / mnemonic-key sibling repos (per-crate prefix in section
header; "What's new" / "What didn't change" / "Migration notes" subsections).

## ms-codec [0.1.0] — 2026-MM-DD

### What's new

- Initial release. Reference implementation of the **ms1** backup format
  (HRP `ms`) for BIP-39 entropy.
- Wire format: BIP-93 codex32 used directly via Andrew Poelstra's
  `rust-codex32 = "=0.1.0"` (CC0). No fork.
- v0.1 payload kind: `entr` (BIP-39 entropy, 16/20/24/28/32 B = BIP-39
  word counts {12, 15, 18, 21, 24}).
- v0.1 emitted strings: 50/56/62/69/75 chars (short codex32 checksum only).
- Public API: `encode(Tag, &Payload) -> Result<String>`,
  `decode(&str) -> Result<(Tag, Payload)>`, `inspect(&str) -> Result<InspectReport>`.
- `Tag::ENTR` const; `Payload::Entr(Vec<u8>)`; `InspectReport` for debugging.
- Decoder applies the full SPEC §4 validity rule set (10 rules);
  encoder mirrors the reserved-not-emitted-tag rejection (SPEC §3.5.1).
- v0.2 share-encoding migration designed up-front via the `0x00`
  reserved-prefix byte; v0.1 strings remain forward-readable by v0.2 decoders.

### What didn't change

(N/A — initial release.)

### Migration notes

(N/A — initial release. See `MIGRATION.md` for the v0.1 → v0.2 contract.)

### Wire-format SHA pin

The canonical test vectors at `crates/ms-codec/tests/vectors/v0.1.json`
are SHA-256-pinned at this release. Subsequent corpus changes that alter
the SHA require a SemVer minor bump.

```text
sha256(crates/ms-codec/tests/vectors/v0.1.json) = <PASTE-HASH-FROM-TASK-7.3>
```
```

Replace `<PASTE-HASH-FROM-TASK-7.3>` with the actual SHA from Task 7.3 Step 1. Replace `2026-MM-DD` with the actual release date.

### Task 7.5: Refresh README.md

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Read current README**

```bash
cat /scratch/code/shibboleth/mnemonic-secret/README.md
```

- [ ] **Step 2: Replace with the v0.1.0 release version**

```markdown
# mnemonic-secret

[![CC0 1.0 Universal][license-badge]][license-link]
[![MSRV: 1.85][msrv-badge]][msrv-link]

Reference implementation of the **ms1** backup format — BIP-93 codex32 directly
applied to BIP-39 entropy for steel-plate engraving with strong BCH error correction.
Sibling to [`descriptor-mnemonic`][md-repo] (md1, wallet descriptors) and
[`mnemonic-key`][mk-repo] (mk1, xpubs).

[license-badge]: https://img.shields.io/badge/License-CC0_1.0-blue.svg
[license-link]: LICENSE
[msrv-badge]: https://img.shields.io/badge/MSRV-1.85-blue.svg
[msrv-link]: rust-toolchain.toml
[md-repo]: https://github.com/bg002h/descriptor-mnemonic
[mk-repo]: https://github.com/bg002h/mnemonic-key

## What it does

Encode the entropy of a BIP-39 seed phrase as a `ms1`-prefixed BIP-93 codex32
string designed to engrave on metal. The encoded string self-checks for up to 8
character substitutions and self-corrects up to 4 — far stronger than BIP-39's
own 4-bit checksum, which is too weak to localize errors on engraved media.

## Quickstart

```rust
use ms_codec::{encode, decode, Payload, Tag};

// Take the entropy of a 12-word BIP-39 mnemonic.
let entropy = vec![0u8; 16]; // ... the 16 raw bytes from your mnemonic
let s = encode(Tag::ENTR, &Payload::Entr(entropy.clone())).unwrap();
assert_eq!(s.len(), 50);

// Engrave `s`. Recover later:
let (tag, payload) = decode(&s).unwrap();
assert_eq!(tag, Tag::ENTR);
assert_eq!(payload, Payload::Entr(entropy));
```

To recover a BIP-39 mnemonic from the decoded entropy, use the [`bip39`][bip39] crate:

```rust
use bip39::{Language, Mnemonic};
let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy).unwrap();
println!("{}", mnemonic);
```

[bip39]: https://crates.io/crates/bip39

## Scope

| | v0.1 (this release) | v0.2 (planned) | v0.2+ |
|---|---|---|---|
| BIP-39 entropy `entr` | ✓ emit + accept | + K-of-N share encoding | |
| BIP-32 master seed `seed` | reserved-not-emitted | | + own framing |
| BIP-32 xpriv `xprv` | reserved-not-emitted | | + own framing |
| K-of-N shares | not yet | ✓ for `entr` | + for other kinds |

The BIP-32 master seed backup use case is preserved at the application layer:
`BIP-39 phrase → entropy → ms1 entr → engrave → recover entropy → BIP-39 mnemonic
→ PBKDF2 → 64-B BIP-32 master seed`. Direct `seed` and `xprv` payloads are
deferred to v0.2+ because they overflow BIP-93 codex32's length brackets when
prepended with the v0.2-migration prefix byte. See [`design/SPEC_ms_v0_1.md`](design/SPEC_ms_v0_1.md)
§1.3 for full discussion.

## Documentation

- [`design/SPEC_ms_v0_1.md`](design/SPEC_ms_v0_1.md) — wire format, decoder rules, BIP-93 anchoring.
- [`design/BRAINSTORM_ms_v0_1.md`](design/BRAINSTORM_ms_v0_1.md) — the rationale chain.
- [`MIGRATION.md`](MIGRATION.md) — v0.1 → v0.2 contract.
- [`design/RELEASE_PROCESS.md`](design/RELEASE_PROCESS.md) — release discipline.

## Family

`ms-codec` is one of three sibling format crates plus a future toolkit:

- **md-codec** ([repo][md-repo]) — wallet descriptors / templates (`md1`, HRP `md`).
- **mk-codec** ([repo][mk-repo]) — xpubs (`mk1`, HRP `mk`).
- **ms-codec** (this crate) — secret material (`ms1`, HRP `ms`).
- **mnemonic-toolkit** (planned) — top-level integration: take a BIP-39 phrase,
  emit a complete ms1+mk1+md1 engravable bundle.

## License

CC0 1.0 Universal. See [LICENSE](LICENSE).
```

### Task 7.6: Run all CI gates

- [ ] **Step 1: cargo build**

```bash
cargo build -p ms-codec --all-targets 2>&1 | tail -5
```

Expected: clean.

- [ ] **Step 2: cargo test (full suite)**

```bash
cargo test -p ms-codec --all-targets 2>&1 | tail -20
```

Expected: ~30 tests pass (5 tag + 4 payload + 8 envelope + 4 encode + 3 decode + 2 inspect + 10 negative + 2 forward-compat + 5 round-trip + 3 bip39 + 1 vectors + 1 doc-test). All proptests run their default 256 cases.

- [ ] **Step 3: cargo clippy --all-targets -D warnings**

```bash
cargo clippy -p ms-codec --all-targets -- -D warnings 2>&1 | tail -10
```

Expected: clean. If clippy fires, fix inline.

- [ ] **Step 4: cargo fmt --check**

```bash
cargo fmt --check -p ms-codec 2>&1 | tail -10
```

Expected: clean. If unformatted code, run `cargo fmt -p ms-codec` and re-stage.

- [ ] **Step 5: cargo publish --dry-run** (post-version-bump in Task 7.6.5)

After Task 7.6.5 lands the version bump:

```bash
cargo publish -p ms-codec --dry-run 2>&1 | tail -20
```

Expected: clean dry-run packaging. Address any "missing field" complaints (license, description, etc.) inline.

### Task 7.6.5: Version bump 0.1.0-dev → 0.1.0

**Files:**
- Modify: `crates/ms-codec/Cargo.toml`

The version bump is a deliberate semantic act and gets its own commit so the release log is unambiguous about when v0.1.0 was tagged.

- [ ] **Step 1: Bump version**

Modify `crates/ms-codec/Cargo.toml`:

```toml
version = "0.1.0"
```

- [ ] **Step 2: Verify it compiles + tests still pass**

```bash
cargo test -p ms-codec --all-targets 2>&1 | tail -10
```

Expected: all tests still pass; no version-related warnings.

- [ ] **Step 3: Commit the version bump alone**

```bash
git -C /scratch/code/shibboleth/mnemonic-secret add crates/ms-codec/Cargo.toml
git -C /scratch/code/shibboleth/mnemonic-secret commit -m "$(cat <<'EOF'
chore(ms-codec): bump to v0.1.0

Removes -dev suffix in preparation for v0.1.0 release tag.
Cargo.toml-only change; no source or test edits.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

### Task 7.7: Phase 7 commit + opus review

- [ ] **Step 1: Commit the release-prep set** (excludes the version bump from Task 7.6.5)

```bash
git -C /scratch/code/shibboleth/mnemonic-secret add \
  CHANGELOG.md \
  README.md \
  crates/ms-codec/tests/vectors.rs \
  crates/ms-codec/tests/vectors/v0.1.json

# Note: the serde/serde_json dev-dep additions to Cargo.toml from Task 7.2
# Step 1 are also part of this commit if they were not separately staged earlier.
git -C /scratch/code/shibboleth/mnemonic-secret add crates/ms-codec/Cargo.toml

git -C /scratch/code/shibboleth/mnemonic-secret commit -m "$(cat <<'EOF'
release(ms-codec): v0.1.0 release prep — vector corpus + CHANGELOG + README

Phase 7 of IMPLEMENTATION_PLAN_ms_v0_1.md. Final release-prep commit
(version bump to 0.1.0 landed separately in the preceding chore commit).
~30 tests passing across the full suite (lib + 6 integration test files
+ 1 doc-test). All CI gates green: cargo build, cargo test, cargo
clippy --all-targets -D warnings, cargo fmt --check, cargo publish
--dry-run.

What's in this commit:
- Cargo.toml: added serde + serde_json dev-deps for vector corpus.
- tests/vectors.rs + tests/vectors/v0.1.json: 2 canonical BIP-39
  test vectors (12-word abandon-about, 24-word abandon-art),
  SHA-256-pinned at this release per RELEASE_PROCESS.md.
- CHANGELOG.md: ms-codec [0.1.0] entry with What's-new / What-didn't-
  change / Migration-notes / Wire-format-SHA-pin sections per the
  per-crate-prefix convention from md-codec / mk-codec.
- README.md: replaced placeholder with v0.1.0 release content
  (Quickstart, scope table, family pointer, license).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

- [ ] **Step 2: Opus review**

Brief: verify CHANGELOG follows the md-codec / mk-codec convention; verify README accurately reflects v0.1 scope; verify cargo publish --dry-run output had no missing-field warnings; verify the SHA-pin is recorded; verify version bump to 0.1.0 (no -dev suffix). Persist to `design/agent-reports/phase-7-release-prep-review-r1.md`.

### Task 7.8: Tag the release

- [ ] **Step 1: Tag**

```bash
git -C /scratch/code/shibboleth/mnemonic-secret tag -a ms-codec-v0.1.0 -m "ms-codec v0.1.0"
git -C /scratch/code/shibboleth/mnemonic-secret tag --list ms-codec-v0.1.0 -n
```

Expected: tag created with the v0.1.0 message. **Do not push the tag** — that's a user-explicit-approval action per the session conventions.

---

## Phase-completion summary

After Phase 7's opus-review convergence, the v0.1.0 release is locally tagged but not pushed. The remaining steps are user-gated:

1. `git push origin main && git push origin ms-codec-v0.1.0` — publish to GitHub.
2. `cargo publish -p ms-codec` — publish to crates.io.

Update the cross-repo FOLLOWUPS entries (`ms1-v01-payload-bracket-overflow-prefix-byte-incompatibility`) in mk1 and md1 to add `Status: resolved by ms-codec v0.1.0 release at <commit-sha>` once the tag pushes.

---

## Self-review checklist (run before handing off)

Performed during plan drafting:

**1. Spec coverage:** every SPEC §4 decoder rule has a negative test in Phase 5; every SPEC §10.2 test category (round-trip, BIP-93 cross-validation, negative vectors, vector corpus, forward-compat smoke, BIP-39 round-trip integration) has a Phase 5/6/7 task. SPEC §3.5 + §3.5.1 encoder validation is exercised by Phase 3 encode tests. SPEC §11 BIP-93 anchoring informs the inline test data (BIP-93 vector 1 string in envelope::tests_extract::bip93_test_vector_1_extracts_correctly).

**2. Placeholder scan:** every code block contains real, runnable code. Every commit message is final text. Two exceptions noted: CHANGELOG.md `2026-MM-DD` placeholder (filled at release time) and the SHA-256 pin `<PASTE-HASH-FROM-TASK-7.3>` (filled when Task 7.3 runs).

**3. Type consistency:** `Tag` / `Payload` / `Error` variant names are consistent across phases. `Codex32String::from_seed` / `Codex32String::from_string` / `Codex32String::parts` / `Parts::data` are the only upstream API names referenced; verified during Phase 1 task 1 spike against `/tmp/codex32-extract/codex32-0.1.0/src/lib.rs`. `Fe::S` / `Fe::A` are the upstream share-index constants.

---

## Execution handoff

Plan complete and saved to `design/IMPLEMENTATION_PLAN_ms_v0_1.md`. The session is in autonomous mode per user directive; recommended execution path:

- **Inline execution** via `superpowers:executing-plans` skill, batched by phase, with the per-phase opus-review checkpoints already specified in each phase's final task.

Phase 1 task 1 (the rust-codex32 API contact spike) is the gating verification that confirms the SPEC §10.1 wire-position re-parse strategy is required. Once that lands, Phases 1-7 proceed in sequence.
