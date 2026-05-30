# BUG — `ms_codec::decode_with_correction` only works for 16-byte (12-word) seeds

**Found:** 2026-05-29, during the ms-codec test-hardening cycle (Theme-1 grounding probe).
**Severity:** HIGH — BCH error-correction + indel-repair silently broken for 15/18/21/24-word ms seeds (20/24/28/32-byte entropy). The toolkit `ms repair`, `repair --max-indel`, and `Ms1IndelOracle` all delegate to `decode_with_correction` and inherit the break.
**Status:** root-caused (high confidence); fix approach pending user decision. The codec is on branch `ms-codec-test-hardening` (off `master`).

## Symptom
`decode_with_correction(s)` returns `Err(TooManyErrors{bound:8})` on a freshly-encoded, CLEAN ms1 string for every entropy length EXCEPT 16 bytes. The non-correcting `decode(s)` (which delegates to rust-codex32) round-trips all 5 lengths fine.

## Evidence (controller-run throwaway probes)
**Probe 1** — clean + 1-error decode_with_correction per length:
```
len=16 (50 chars): CLEAN ok 0 corrections; 1-err recovered
len=20/24/28/32:   CLEAN -> Err(TooManyErrors{bound:8})   (and 1-err -> Err)
```
**Probe 2** — hand-rolled `bch_create_checksum_regular` vs the checksum codex32 actually wrote into the encoded string:
```
len=16 data_syms=34: hand-rolled bch_verify=TRUE,  bch_create==codex32_checksum=TRUE
len=20 data_syms=40: bch_verify=FALSE, bch_create==codex32=FALSE
len=24/28/32:        bch_verify=FALSE, bch_create==codex32=FALSE
```
**Probe 3** — raw `polymod_run(hrp_expand("ms") || symbols)` for CLEAN valid codewords (a correct BCH verify yields ONE fixed target for all valid codewords):
```
SECRETSHARE32 packed big-endian = 0x10ce0795c2fd1e62a  (bit64=1)  <- the TRUE codex32 short target
MS_REGULAR_CONST                = 0x962958058f2c192a   (bit64=0)  <- the hand-rolled const
raw(16B) = 0x962958058f2c192a  (== MS_REGULAR_CONST)              <- the single calibration vector
raw(20B) = 0x1ddc8a0f6200f5e2d
raw(24B) = 0x15b9aac2ecb92c931
raw(28B) = 0x400125ef092d41f8
raw(32B) = 0xffd053fa2844803b
```
The valid-codeword residues are **length-variant** and **none equals SECRETSHARE32** — so the hand-rolled polymod does NOT compute the codex32 residue at all.

## Root cause (confirmed)
- ms **encodes** via the rust-codex32 crate. `envelope.rs:1` — *"THE v0.2-MIGRATION SEAM. This is the only module that contacts rust-codex32."* `encode` → `envelope::package` → `Codex32String::from_unchecksummed_string` → the **standard codex32 short checksum** (init residue = value 1, target = "SECRETSHARE32"). All 5 ms string lengths (50–75 < 81) use the short code.
- ms **error-corrects** (`decode_with_correction`, `decode.rs:188-246`) via the **hand-rolled** `bch.rs`: `polymod_run` with `POLYMOD_INIT = 0x23181b3` and verify-target `MS_REGULAR_CONST = 0x962958058f2c192a`.
- `MS_REGULAR_CONST` was **empirically lifted from one 12-word vector's raw polymod** (`bch.rs:33-37` cite a now-deleted toolkit `ms_nums_target_is_stable_across_distinct_valid_strings` cell; the doc claim "codex32's SECRETSHARE32 Fe-vec packed" is FALSE — probe3 shows `0x9629… ≠ 0x10ce…`). The hand-rolled scheme is NOT codex32-equivalent; for valid codewords it produces length-variant residues, so it only validates the single calibration length (16-byte). The `bch_decode.rs:19-25` header comment ("other v0.1 lengths … do not satisfy the polymod invariant the BCH decoder assumes") is the bug wearing a comment.
- `GEN_REGULAR`, `POLYMOD_INIT`, `REGULAR_SHIFT`, `REGULAR_MASK`, `polymod_step` are **byte-identical to mk-codec** (`mnemonic-key/.../mk-codec/src/string_layer/bch.rs`). So **mk-codec may share the latent bug** if mk never exercised multiple data-part lengths (mk1 is chunked). REQUIRES AUDIT.

## First architect diagnosis — REFUTED
The initial architect pass concluded "MS_REGULAR_CONST dropped bit 64 of a fixed target; just restore the 65th bit." Probe3 refutes this: the valid-codeword residues are length-variant (`0x9629/0x1ddc/0x15b9/0x4001/0xffd0`), not a single fixed target with one dropped bit; `(MS_REGULAR_CONST | 1<<64)` matches none of them, and none equals SECRETSHARE32. No single constant fix works — the polymod machinery itself does not compute the codex32 residue (likely `POLYMOD_INIT` and/or the whole formulation, calibrated for a different/empirical scheme).

## Fix options
- **(A) Delegate to rust-codex32 (recommended, safest).** Make `decode_with_correction` obtain the residue/syndrome from the authoritative rust-codex32 engine (the same crate `encode`/`decode` already use), correct for ALL lengths. Caveat: the BCH error-CORRECTION (Berlekamp-Massey + Chien + Forney in `bch_decode.rs`) must be driven by a syndrome consistent with codex32's field/generator — needs careful design + review.
- **(B) Fix the hand-rolled constants.** Derive the TRUE codex32-standard `POLYMOD_INIT`/target and replace them, then verify the whole BCH correction path is codex32-consistent. Riskier (more bit-level GF(32) surface; the constants were wrong precisely because they were hand-derived).

## Toolkit + sibling impact
- Toolkit `ms-codec = "0.2.0"` (crates.io, not git). Fix → ms-codec PATCH/MINOR + publish + toolkit `cargo update -p ms-codec` re-pin + toolkit PATCH (binary behavior changes: ms repair now works for 15/18/21/24-word). No GUI/manual flag-surface change.
- mk-codec audit (shared machinery). md-codec uses its own derivation — likely unaffected but worth a parallel all-lengths gate.

## Regression-test scope (post-fix)
- A constant/scheme gate: `decode_with_correction` on a clean encode-produced string of EVERY length → `Ok` with 0 corrections.
- Theme-1 corrupt→correct→decode property spanning ALL 5 lengths (the monoculture-on-12-word was the root miss).
- Promote the `bch_decode.rs` cells (currently hard-bound to `VALID_MS1_12W`) to sweep all 5 lengths.
