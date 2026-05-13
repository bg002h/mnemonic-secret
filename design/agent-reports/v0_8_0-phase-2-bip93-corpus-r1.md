# v0.8.0 Phase 2 — ms-codec BIP-93 — R1 architect review + disposition

**Date:** 2026-05-13
**Reviewer:** `feature-dev:code-reviewer` (Sonnet 4.6), dispatched
per plan Phase 2 reviewer-loop discipline.
**Phase commit reviewed:** `7101c16` on branch
`v0_8_0-bip93-inline-vectors` — adds
`crates/ms-codec/tests/bip93_inline_vectors.rs` (5 valid cells + 2
invariant cells + 1 parametric invalid-corpus cell) and 2
`design/FOLLOWUPS.md` entries.

## R1 verdict

**1I / 0C** — fold applied across two sibling repos.

## R1 findings

### I-1 — `bip-vector-adoption-v0_8` companion entries absent in toolkit + mk-codec (confidence 88)

The ms-codec entry's `Companion:` field stated entries exist in
`mnemonic-toolkit/design/FOLLOWUPS.md`,
`descriptor-mnemonic/design/FOLLOWUPS.md`, and
`mnemonic-key/design/FOLLOWUPS.md`. At review time:

- `descriptor-mnemonic` entry present (landed Phase 1 fold at
  `b464f3f`).
- `mnemonic-toolkit` entry **was not yet present** at review-dispatch
  time. Subsequently added in Phase 3 commit `d269dda` (which
  landed during the architect's review). Verified post-review:
  `grep -c 'bip-vector-adoption-v0_8' /scratch/code/shibboleth/mnemonic-toolkit/design/FOLLOWUPS.md`
  returns 2 (one own-entry + one companion cross-cite).
- `mnemonic-key` entry **absent**. SPEC §5 explicitly names
  mnemonic-key as a "no-scope / symmetry only" companion repo;
  the entry was missed because no other Phase work touches mk-codec.

**Fold:**

1. `mnemonic-toolkit/design/FOLLOWUPS.md` — entry already added at
   Phase 3 commit `d269dda` (predated this R1 disposition by
   minutes but not at review-dispatch time; architect was working
   from `7101c16` ms-secret HEAD which had no visibility into
   later toolkit commits).

2. `mnemonic-key/design/FOLLOWUPS.md` — added a no-scope companion
   entry on a new branch `v0_8_0-bip-vector-adoption-companion`,
   to be committed alongside this report's persistence. Entry body
   quotes the SPEC §5 carve-out verbatim and notes that mk-codec's
   v0.7.1 matrix carries the relevant coverage with no new gap for
   the cycle.

### Nit findings (non-blocking, recorded for cycle audit trail)

- **N-1 — Vector 3 alternate encoding coverage.** Same-cycle
  extension opportunity. Pinning all 4 BIP-93 §93.3 alternate
  canonical encodings against the same master seed would make the
  test independent of `rust-codex32`'s own coverage of alternates.
  Not a defect; documented inline rationale stands. Defer.

- **N-2 — `hex_to_bytes` vs `hex_decode` duplication.** Two test
  files (`bip93_inline_vectors.rs` and `bip93_cross_format.rs`)
  define identical hex parsers. Rust test files cannot share
  helpers without a `tests/common/mod.rs` module. Below confidence
  threshold for a fold; defer.

## R1 checklist passes

The reviewer verified vector 2's `S`-share interpretation against
`codex32-0.1.0`'s own internal tests (lines 478–485 of upstream
`src/lib.rs`): identical pinning shape. V1–V5 strings, seeds, and
the `data()[..N]` slice are all confirmed against upstream. The
parametric invalid-corpus `is_err()` shape is plan-endorsed. The
`Display`-round-trip case-preserving assertion is confirmed by
the upstream `Display` impl (`fmt::Display::fmt(&self.0, f)`).

## R2 self-clear

I-1 fold landed in two sibling repos (toolkit via Phase 3 commit,
mk-codec via separate companion branch). All cited companions now
exist on disk. **Phase 2 close gate: CLEAR.** This report is the
canonical Phase 2 R1 record; Phase 4 audit-matrix successor will
cross-cite it.
