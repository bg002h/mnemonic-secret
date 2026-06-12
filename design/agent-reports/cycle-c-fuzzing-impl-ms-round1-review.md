# Implementation Review — Cycle C phase 2 ms-codec (round 1)

Reviewer: Fable 5 architect agent (a9eb63b7f2c054b2d), 2026-06-12.
Target: uncommitted ms-phase fuzz infra @ mnemonic-secret (HEAD 2251741).
Persisted verbatim per CLAUDE.md convention.

## Verdict: GREEN

0 Critical / 0 Important. The no-leak oracle is sound: the fixture is a genuine recombineable 2-of-3, the window-scan is post-HRP and correctly thresholded, the exclusion set is exactly `Codex32(_) | WrongHrp{..}` and provably minimal against the real 16-variant enum, and non-vacuity was independently proven (exclusion-removed → real leak found; intact → 48K execs clean). The decode_with_correction panic is a real, correctly-scoped, correctly-filed robustness finding.

## Critical
- none

## Important
- none

## Minor
- **gen_corpus mirrors the no-leak mutation logic in `mutate_fixture` (a hand-copy of the target's body).** Deliberate "kept in lockstep" duplication (commented), but a drift hazard: if the target's mutation layout changes and the mirror doesn't, the gate silently stops exercising the real path. Non-blocking — the mirror only asserts no-panic; the leak scan is the target's job. Worth a shared helper if ever refactored.
- **Set-id region (`22hy2`) is shared across all 3 fixture shares**, so an 8-char window starting there is not unique-per-share and the panic message attributes the hit to the first matching share. Does not weaken detection (any 8 contiguous data-part chars echoed = a leak regardless of attribution); cosmetic.
- **`from_utf8_lossy` in the no-leak bit-flip/truncate arms** can turn a mutated byte into a multi-byte U+FFFD, so "position" is byte-not-char in a few mutations. Harmless (fixture is ASCII; surfaces handle arbitrary strings); acceptable per R0 [M7].

## Deliverable-conformance table
| Item | Conforms? | Notes |
|---|---|---|
| 3 targets compile (gnu) | Yes | Clean rebuild, zero warnings |
| ms1_decode: decode + dwc + inspect | Yes | All three on same input; inspect value ignored, panic-only oracle |
| ms1_decode: fixed-point via encode(tag,&payload) | Yes | Value `PartialEq` compare (not bytes); re-encode Err = `.expect` panic = finding |
| ms1_decode: apply-details idempotence | Yes | ms `{position,was,now}`, no chunk_index, position post-`ms1`; out-of-range → `None` early-return (bounds-safe) |
| ms1_combine: \n-split 2..=8 | Yes | `take(MAX_PARTS=8)`, lossy utf8; below-threshold let through (clean Err) |
| ms1_combine: re-encode fixed-point | Yes | encode→decode→value-equal; re-encode Err = finding |
| no-leak: real valid 2-of-3 fixture | Yes | Each share = `IsShareNotSingleString`; all pairs + full set recombine to `Entr([0xAB;16])` |
| no-leak: data-part window scan (≥8, post-HRP) | Yes | `data_part` strips `ms1`; WINDOW=8; Display + Debug both scanned; short-needle guard present |
| no-leak: exclusion = exactly Codex32 \| WrongHrp | Yes | Verified minimal against all 16 ms Error variants + 3 codex32 String variants (all via `From`→`Codex32`, excluded) |
| no-leak: non-vacuous | Yes | Exclusion-removed scratch run found a real leak in seconds; intact run clean over 48,536 execs |
| gen_corpus gate (same-call-as-target) | Yes | decode/combine seeds gated; combine \n-between-only (no trailing); deterministic (2 runs, no churn) |
| fuzz-smoke.yml: build all 3, smoke = combine + no_secret_leak | Yes | ms1_decode held out (commented, cites FOLLOWUP); gnu pinned both jobs; actionlint clean; ms-first-CI header |
| Lock: codex32 == root (0.1.0) | Yes | Both crates.io 0.1.0; ms-codec path-dep 0.4.2 |
| Isolation: root fmt ignores fuzz/ | Yes | Only pre-existing ms-cli diff reported; fuzz/ invisible; crates/ untouched |
| .gitignore: target/artifacts/coverage ignored; corpus+lock committed | Yes | `git check-ignore` confirms target ignored, corpus/Cargo.lock untracked-not-ignored |
| FOLLOWUP filed | Yes | `decode-with-correction-panics-on-non-char-boundary-hrp-slice` — accurate |

## Real-finding assessment
**Real, correctly scoped, correctly filed.** Scratch-reproduced against the live crate:
- `decode_with_correction(from_utf8_lossy(&[0xaa]))` panics at `crates/ms-codec/src/decode.rs:151:24`: "end byte index 2 is not a char boundary; it is inside '�' (bytes 0..3)" — exact message + line the FOLLOWUP cites. Root cause confirmed: `parse_ms1_symbols` does `lower[..hrp_end.saturating_sub(1)]` when there's no `'1'` (so `hrp_end = len`), slicing inside the 3-byte U+FFFD.
- `decode("�")` → clean `Err(UnexpectedStringLength { got: 3 })` (length gate first); `inspect("�")` → clean `Err(Codex32(InvalidLength(3)))`. "decode_with_correction ONLY" scope accurate.
- The `ms1_decode` target re-finds it instantly — exactly why holding it out of the smoke matrix (vs leaving it red or guarding the input) is the right call per the standing-red-masks-reds lesson; the build gate still compile-checks all 3, so it can't silently rot, and it rejoins the matrix as the regression gate the moment the fix lands.
- NOT a secret leak — aborts before producing any payload; pure robustness/DoS (never-panic charter class).

## Evidence log
```
BUILD: clean rebuild all 3 gnu targets, 0 warnings
NO-LEAK: fixture decode→IsShareNotSingleString(q/p/z); combine {0,1}{0,2}{1,2}+all3 → Entr([0xAB;16]) ✓ REAL 2-of-3
  data_part strips "ms1" (dp 47, full 50), no ms1-prefix false-positive; WINDOW=8 Display+Debug
  exclusion minimal: ms Error 16 variants; only Codex32(_)+WrongHrp{got} ≥8-char input; codex32 String variants all via From→Codex32 (excluded)
  NON-VACUITY: exclusion removed (scratch) → fuzzer PANICKED in seconds with real leak finding (hit="22hy2qfw" Codex32(InvalidChecksum{string:"ds122hy2qfw..."})), exit 77; restored
  INTACT: 48,536 execs / 61s CLEAN
REAL FINDING: dwc("�") panics decode.rs:151:24; decode→clean Err(UnexpectedStringLength got=3); inspect→clean Err(Codex32(InvalidLength(3))); ms1_decode re-finds instantly
ms1_combine: 2,013,944 execs / 31s CLEAN
CORPUS: gen_corpus PASS; md5 identical 2 runs; combine seed 3 shares \n-between no trailing; corpus+lock untracked-not-ignored; fuzz/target ignored
LOCK: fuzz codex32==root==0.1.0 crates.io; ms-codec path-dep 0.4.2
ISOLATION: crates/ untouched; root fmt reports ONLY pre-existing ms-cli/src/cmd/combine.rs; fuzz sources rustfmt +1.85.0 --edition 2024 clean; actionlint clean
CI: build all 3; smoke=[ms1_combine, ms1_no_secret_leak]; ms1_decode held out (cites FOLLOWUP); gnu pinned; ms-first-CI header
TREE LEFT AS FOUND: M .gitignore, M design/FOLLOWUPS.md, ?? fuzz/, ?? .github/workflows/fuzz-smoke.yml
```

GREEN — cleared to commit.
