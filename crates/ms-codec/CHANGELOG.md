# Changelog — ms-codec

All notable changes to the `ms-codec` crate are documented here. The format
follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] — 2026-05-29

### Fixed

- **`decode_with_correction` now error-corrects all entropy lengths, not just
  16-byte (12-word) seeds.** The hand-rolled BCH path used a wrong
  `POLYMOD_INIT` (`0x23181b3`) and an empirically-lifted `MS_REGULAR_CONST`
  (`0x962958058f2c192a`) calibrated to a single 12-word vector, so `polymod_run`
  was length-variant for valid codewords and `decode_with_correction` returned
  `TooManyErrors` on CLEAN 20/24/28/32-byte ms1 strings. Corrected to the
  standard codex32 short-code start state (`POLYMOD_INIT = 1`) and the true
  "SECRETSHARE32" target (`MS_REGULAR_CONST = 0x10ce0795c2fd1e62a`). The
  generator and Berlekamp-Massey/Chien/Forney decoder were already correct and
  are unchanged. Downstream impact: the toolkit's `ms repair`, `repair
  --max-indel`, and `Ms1IndelOracle` now work for 15/18/21/24-word seeds.
  Root cause + evidence: `design/BUG_decode_with_correction_length_divergence.md`.

### Added

- `tests/bch_all_lengths.rs` — all-five-length BCH regression suite (the
  constant-derivation + single-target-residue gate that would have caught the
  bug; clean-passthrough; 1–4-error correction with position checks; the 5–8-
  error miscorrection sweep; and the indel reject-contract). Replaces the prior
  12-word-only test monoculture that hid the defect.

## [0.1.1] — 2026-05-07

BIP test vector audit close-out (Phase 10 of the v0.7.1 audit cycle). No
wire-format changes; pure test-coverage extension.

### Added

- 4 new entries in `tests/vectors/v0.1.json` (custom corpus grows 2 → 6):
  - 15-word all-zero entropy (BIP-39 `[0; 20]`).
  - 18-word all-zero entropy (BIP-39 `[0; 24]`).
  - 21-word all-zero entropy (BIP-39 `[0; 28]`).
  - 15-word non-zero entropy (`0123456789abcdef0123456789abcdef01234567`)
    — catches entropy-bit-ordering regressions zero-entropy vectors miss.
- `tests/bip93_cross_format.rs` — 2 new tests pinning BIP-93 §Test Vector
  93.4 (256-bit `leet`) cross-format conformance:
  - Payload extraction via upstream `rust-codex32` is byte-stable.
  - Re-encoding 93.4's 32-byte payload as ms-codec entr round-trips and
    the resulting ms1 string is parseable by upstream codex32 (proves
    ms-codec is a proper sub-format of BIP-93 codex32 at the byte level
    for the `entr` length bucket).
  - BIP-93 spec: <https://github.com/bitcoin/bips/blob/master/bip-0093.mediawiki>.

### Internal

- Audit matrix `design/agent-reports/v0_7_1-bip-test-vector-audit-matrix.md`
  updated: BIP-93 §93.4 row flipped MISSING → COVERED; custom-corpus row
  flipped 2 → 6 entries; AMBIGUOUS Discovery #2 (non-zero entropy missing)
  marked RESOLVED.
- Workspace-internal: `ms-cli` ms-codec dep pin bumped `=0.1.0` → `=0.1.1`,
  ms-cli's mirrored `vectors/v0.1.json` extended to match (parity test
  `vectors_corpus_parity_with_ms_codec` still passes).

## [0.1.0] — 2026-05-03

Initial public release of `ms-codec`. See `design/SPEC_ms_v0_1.md` for the
full wire-format specification.
