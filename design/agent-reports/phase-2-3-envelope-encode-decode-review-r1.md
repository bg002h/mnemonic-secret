# Phase 2+3 (Envelope + Encode/Decode) — Opus Review r1

**Date:** 2026-05-03
**Reviewer:** feature-dev:code-reviewer (opus, async)
**Commit reviewed:** `eef00f7` ("feat(ms-codec): Phase 2+3 envelope + encode/decode (combined commit)")
**Files:** `crates/ms-codec/src/{envelope,encode,decode,lib}.rs`
**Tests:** 26 unit tests pass; clippy --all-targets -D warnings clean.

## Verdict

**Zero critical, zero important findings.** Phase 2+3 is ready to proceed to Phase 4 without iteration. Wire offsets are byte-correct against `rust-codex32 v0.1.0`'s `parts_inner`; rule ordering matches SPEC §4 and the plan's defensive-optimization notes; the v0.2-migration seam is clean.

## Critical findings

None.

## Important findings

None.

## Low / Nit (deferrable)

1. **`envelope.rs:108` defensive arm yields a misleading error variant** (confidence 60). When `payload_with_prefix.is_empty()` (unreachable for valid v0.1 strings, but the code path exists), it returns `Error::ReservedPrefixViolation { got: 0 }`. A reader chasing that error in logs would be confused — `got: 0` is what a *valid* prefix byte looks like. Consider returning `Error::UnexpectedStringLength` or a dedicated invariant-broken variant. Defer to FOLLOWUPS; not worth blocking Phase 4.

2. **`extract_wire_fields` length check could be expressed against `VALID_STR_LENGTHS` minimum** (confidence 35). The current arithmetic `s.len() < sep + PAYLOAD_START_OFFSET + CHECKSUM_LEN_SHORT` is correct but cryptic at the call site. A comment "minimum sep+20 for any v0.1-shaped string" would help future readers. Stylistic.

3. **`tag.rs:30-39` `try_new` error variant for length-mismatch is `TagInvalidAlphabet`** (confidence 40). When `s.len() != 4`, returning `TagInvalidAlphabet` is technically inaccurate — the bytes might be alphabet-valid but the wrong count. Spec §4 only enumerates rule 5 for alphabet, not a separate length rule. Pre-existing from Phase 1; not introduced by this phase.

## Affirmations

- **Wire offsets exactly match `parts_inner`** (confidence 95). Verified against `codex32-0.1.0/src/lib.rs:178-205`.
- **`discriminate` rule ordering matches SPEC §4** (confidence 90): HRP → threshold → share-index → tag-alphabet → prefix-byte. Rule 6/7 correctly deferred to `decode.rs`.
- **`encode` ordering matches §3.5.1** (confidence 95): reserved-not-emitted fires before payload validation.
- **`decode` rule-9-before-rule-1 ordering correct** (confidence 95): A 96-char string (valid BIP-93 long-checksum) is rejected before parse.
- **`decode` rule-7-before-rule-6 ordering safe** (confidence 95): All three branches (entr / reserved / unknown) resolve to the spec-mandated error.
- **`package` ↔ `discriminate` symmetry** (confidence 90): round-trip test exercises all 5 entr lengths.
- **`From<codex32::Error>` usage clean** (confidence 90): `?` operator centralizes wrapping.
- **`decode_rejects_short_seed_string_with_reserved_tag` exercises rule 7 specifically** (confidence 90).
- **`cargo clippy --all-targets -D warnings` clean + 26 unit tests passing** (confidence 100).

## Nit handling

Nits 1, 2 added to FOLLOWUPS at tier `v0.1-nice-to-have`. Nit 3 inherits the `phase-1-low-1` open item (same diagnostic-bytes concern raised in Phase 1 review).
