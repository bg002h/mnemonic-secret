# Final pre-ship review — ms-codec BCH-checksum length-divergence fix (commit `25fed85`)

Opus code-architect "bless" pass. Branch `ms-codec-test-hardening`, target `0.2.1`. Persisted by controller (review agent had no Write tool). The prior fix-design R0 (`ms-bch-fix-R0-review.md`) was GREEN; this pass confirms the COMMITTED result + folds.

## Confirmations
1. **Fix is exactly two constants; only `bch.rs` changed in `src/`.** `bch.rs:52` `POLYMOD_INIT = 0x1`; `bch.rs:46` `MS_REGULAR_CONST = 0x10ce0795c2fd1e62a`. `GEN_REGULAR`/`polymod_step`/`polymod_run`/`hrp_expand`/`bch_create_checksum_regular`/`bch_verify_regular` UNCHANGED; `bch_decode.rs` BM/Chien/Forney constant-agnostic; `decode.rs` structurally unchanged. **Controller-verified:** `git diff 25fed85^ 25fed85 -- 'crates/ms-codec/src/*.rs' ':!bch.rs'` is EMPTY. Old values appear only in docs/CHANGELOG/BUG-doc.
2. **Doc folds correct, no new false claims.** `bch.rs` MS_REGULAR_CONST/POLYMOD_INIT docs accurately describe SECRETSHARE32/init=1; M1 dangling toolkit cross-check corrected; `README.md:62` hex updated (I1); `tests/bch_decode.rs:18-25` header de-falsified (I2).
3. **`tests/bch_all_lengths.rs` (I3) non-vacuous + locks the fix.** `polymod_lands_on_single_target_for_every_length` would FAIL against the old `POLYMOD_INIT=0x23181b3` (length-variant residue) for all non-16-byte lengths — the exact gate that would have caught it. `handrolled_checksum_matches_codex32_encoded_tail` cross-checks the real encoder. clean-passthrough + 1-4-error (position-set) all 5 lengths. 5-8-error sweep asserts `!= Ok(original)` (correct for non-perfect BCH). indel insert→`is_err()`, delete→`matches!(TooManyErrors|UnexpectedStringLength)` — both robust (residue computed before the rule-9 length gate, `decode.rs:188-207`).
4. **Versioning/lockstep.** ms-codec `0.2.1`; CHANGELOG `[0.2.1]` accurate; ms-cli `0.4.2` + pin `=0.2.1`; **Cargo.lock controller-verified** ms-codec 0.2.1 / ms-cli 0.4.2.
5. **Ship-ready.** No clap surface change → no GUI/manual lockstep. CI (clippy `-p ms-cli` + tests, no fmt gate) — **controller-verified 189 tests / 0 fail, clippy PASS**. Post-publish toolkit re-pin correctly deferred.

## CRITICAL — None.   ## IMPORTANT — None.
## MINOR
- **M-a** (R0 M2/M3, unfolded): `bch_decode.rs:30` doc cites `bch.rs:39-41` for the bare consts; now `:52-54`. Prose-only decayed line-ref, non-blocking.
- **M-b**: `design/FOLLOWUPS.md` + BUG doc retain the old hex as historical record — intentional, no action.

## VERDICT: GREEN — clear to merge + publish 0.2.1, then the deferred toolkit re-pin PATCH after crates.io publish.
