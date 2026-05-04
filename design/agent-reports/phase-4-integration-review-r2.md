# Phase 4 (Integration Tests) — Opus Review r2

**Date:** 2026-05-04
**Reviewer:** feature-dev:code-reviewer (opus)
**Commits reviewed:** `d3d8459` (Phase 4 base) + `ae69f60` (r1 fixup)
**Tests:** 77 unit + integration tests pass; clippy --all-targets -D warnings + fmt --check clean.

## Verdict

**0 critical, 0 important.** Phase 4 reviewer-loop terminator condition met. Phase 5 is authorized.

## Critical

None.

## Important

None.

The three r1-Important resolutions verified sound:

- **r1-I1 (`decode_rejects_threshold_not_zero.rs`):** Test uses `Codex32String::from_seed("ms", 2, "entr", Fe::A, &data)` to bypass codex32-lib threshold validation, then asserts exit 2 / `kind: "ThresholdNotZero"`. Symmetric text + JSON tests. No regression risk.
- **r1-I2 (`decode_rejects_unknown_tag.rs`):** Tag "wxyz" correctly identified as bech32-alphabet-valid but absent from RESERVED_TAG_TABLE. JSON test correctly asserts `details.tag == "wxyz"` (exercising the structured detail field).
- **r1-I3 (plan task 1.8 sync):** Plan code block byte-equivalent (modulo doc-comment phrasing) to shipped `parse.rs`. One nit: shipped source has `normalize_phrase` as `fn` (private); plan shows `pub fn`. Visibility nit-only — not material.

The dev-dep `codex32 = { workspace = true }` is present in `ms-cli/Cargo.toml`, so the new tests compile cleanly.

## Low / Nit

1. **`ms-cli/Cargo.toml` missing publishable metadata** (`description`, `documentation`, `readme`, `keywords`, `categories`) that `ms-codec/Cargo.toml` already has. Phase 5 will need to add these before flipping `publish = true` and bumping `version`. Track as a Phase 5 checklist item.
2. **No `inspect` rejection-path test for UnknownTag.** Mostly redundant — `inspect` is the lenient/diagnostic path and per SPEC should *report* unknown tags via `failure_reasons`, not reject. Defer.
3. **r1's 4 unaddressed nits** (verify.rs `_mnemonic` underscore, verbose comment block, misnamed test fns) all cosmetic; correctly deferred.

## Affirmations

- Test count delta consistent: 73 + 4 = 77.
- New tests follow established pattern (text mode + JSON envelope, `predicate::str::contains` for resilience).
- JSON envelope assertions exercise all four §5.4 fields (`schema_version`, `error.kind`, `error.exit_code`, `error.details.tag`).
- `codex32 = "=0.1.0"` dev-dep was correctly pre-staged in Phase 1 (per plan task 1.2 r1-I4 resolution).
- Plan revision history r6 entry properly documents the parse.rs sync.

## Recommendation

Phase 4 converged. Proceed to Phase 5. Phase 5 should explicitly:
1. Add Cargo.toml metadata fields (description, documentation, readme, keywords, categories) — mirror ms-codec/Cargo.toml shape.
2. Bump version 0.0.0 → 0.1.0.
3. Flip publish=false → publish=true (or remove the line).
4. Run `cargo publish --dry-run` to verify clean packaging.
5. Tag `ms-cli-v0.1.0` (locally; user pushes).
