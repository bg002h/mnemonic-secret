# R0 Review — ms-codec BCH-checksum length-divergence fix

Opus code-architect. Post-fix review on branch `ms-codec-test-hardening` + mk/md sibling audit. Source-read only; two confirmatory probes delegated to controller (both since run — see end). Persisted by controller (review agent had no Write tool).

## Confirmations (file:line)
Fix shape exact: `bch.rs` `MS_REGULAR_CONST = 0x10ce0795c2fd1e62a`, `POLYMOD_INIT = 0x1`; `GEN_REGULAR`/`REGULAR_SHIFT=60`/`REGULAR_MASK`/`polymod_step`/`hrp_expand` UNCHANGED.
- **(a)** `POLYMOD_INIT=1` is the correct codex32/bech32 init for the `hrp_expand("ms") || data` convention (matches rust-codex32's encode engine). Old `0x23181b3` made `polymod_run` length-variant for valid codewords.
- **(b)** `MS_REGULAR_CONST` = SECRETSHARE32 `[16,25,24,3,25,11,16,23,29,3,25,17,10]` packed `Σ vᵢ<<(5·(12−i))` = `0x10ce0795c2fd1e62a` (bit 64 set).
- **(c)** `bch_decode.rs` BM/Chien/Forney is CONSTANT-AGNOSTIC — `decode_regular_errors(residue_xor_const, len)` unpacks 13 coeffs of `residue_xor_const`, evaluates at `BETA^j` (`BETA` :148, `REGULAR_J_START=77` :152, GF(1024) tower); references NEITHER `POLYMOD_INIT` nor `MS_REGULAR_CONST`. Fixing the caller's residue fixes the whole path. Decoder unit tests construct residue via `polymod_run ^ MS_REGULAR_CONST` → self-update.
- **(d)** No other code encodes the old values wrongly. STALE: `README.md:62` (old hex — I1); `tests/bch_decode.rs:18-25` header (false "other lengths don't satisfy invariant" — I2); `bch.rs:12-15` dangling toolkit cross-check (deleted copy — M1).

## CRITICAL — None.
## IMPORTANT (same-PR folds; 0C/0I gate requires before tag)
- **I1** `README.md:62` still prints old `0x962958058f2c192a`. Fix to `0x10ce0795c2fd1e62a`.
- **I2** `tests/bch_decode.rs:18-25` documents the broken behavior as intended ("other v0.1 lengths do not satisfy the polymod invariant") — now FALSE. Rewrite when promoting cells.
- **I3** Add regression gates: (5a) `MS_REGULAR_CONST == pack_be(SECRETSHARE32)` AND `polymod_run(hrp_expand("ms")||symbols(encode(len))) == MS_REGULAR_CONST` for EVERY length (the gate that would've caught it — 0.1.1 corpus HAD 15/18/21-word vectors but never ran them through `decode_with_correction`); (5b) `decode_with_correction(clean) → 0 corrections` every length; (5c) promote `bch_decode.rs` cells to all 5 lengths + rewrite I2 header; (5d) Theme-1 corrupt→correct property all 5 lengths.

## MINOR
- **M1** `bch.rs:12-15` dangling toolkit byte-exact claim (copy deleted, held OLD value). Drop/rephrase.
- **M2/M3** `bch.rs:48` + `bch_decode.rs:30` cite `bch.rs:39-41` for the bare consts; now `:50-52`.

## mk-codec: FINE (no probe needed)
No external `codex32` crate. mk encodes AND corrects via its OWN hand-rolled `bch_create_checksum_regular`/`_long` + same `polymod_run` + same target. `MK_REGULAR_CONST=0x1062435f91072fa5c` = top-65 bits of SHA-256("shibbolethnumskey") (reproducer `consts.rs:71-91`), NOT lifted. Single self-consistent scheme → systematic-encoding identity holds for ANY init at EVERY length; shared `POLYMOD_INIT=0x23181b3` self-cancels. Already exercised at data lengths 10/8/7/0 (regular) + 16 (long). mk never round-trips through rust-codex32.

## md-codec: FINE (no probe needed)
Same structure — own `codex32.rs` module over own hand-rolled `bch`; `MD_REGULAR_CONST=0x0815c07747a3392e7` = top-65 bits SHA-256("shibbolethnums"). Variable chunk lengths already round-trip (12-/13-symbol parts). `POLYMOD_INIT` self-cancels.

**Net:** bug is UNIQUE to ms — the only codec that encodes through external rust-codex32 (init=1, SECRETSHARE32) while error-correcting through the hand-rolled module (cross-scheme mismatch). mk/md hand-roll both halves with matched NUMS constants.

## Versioning + toolkit
- ms-codec → **0.2.1 PATCH** (pure bug fix; `MS_REGULAR_CONST` stays `pub const u128`, only its always-wrong value changes; `POLYMOD_INIT` bare-private). CHANGELOG `## [0.2.1]` + `### Fixed`, cite the BUG doc.
- Toolkit pins `ms-codec="0.2.0"` (crates.io) → publish 0.2.1 → `cargo update -p ms-codec` re-pin + toolkit PATCH (ms repair / `--max-indel` / `Ms1IndelOracle` now work at 20/24/28/32-byte). Toolkit's vendored `MS_NUMS_TARGET` already deleted — pin-only.
- **No GUI/manual flag-surface change** (no clap change). Manual prose stays accurate.

## Confirmatory probes (controller-run, post-fix — PASS)
- Packing: `pack_be([16,25,24,3,25,11,16,23,29,3,25,17,10]) == 0x10ce0795c2fd1e62a` ✓ (probe3).
- All-length residue: `polymod_run(hrp_expand("ms")||symbols)` with init=1 == `0x10ce0795c2fd1e62a` for all 5 lengths ✓ (probe5); `decode_with_correction` clean+1-4 errors recover all 5 lengths ✓ (verify); full suite 75/0 ✓.

## VERDICT: GREEN (0C; I1/I2/I3 + M1/M2/M3 are same-PR doc/test folds, mandatory before tag).
