# Implementation Review — ms-codec error no-echo fix (round 1)

Reviewer: Fable 5 architect agent (ae0e7136926b8e066), 2026-06-12.
Target: uncommitted ms-codec 0.4.4 error-no-echo fix @ mnemonic-secret master.
Persisted verbatim per CLAUDE.md convention.

## Verdict: GREEN

## Critical
- none

## Important
- none

## Minor
- **[m1] `fuzz/Cargo.lock` still pinned `ms-codec 0.4.3`** (regenerates to 0.4.4 on fuzz build; not a correctness issue — the fuzz target compiled/ran clean against 0.4.4 via the path dep). FOLDED by the orchestrator: `cargo update -p ms-codec --precise 0.4.4` in fuzz/ → 0.4.4.
- **[m2] Residual ms-cli `codex32_friendly.rs:44,48` HRP/ID echo** — acceptable/by-design (provenance-bounded: HRP="ms", id=4 chars from `interpolate_at` on a valid `Codex32String`, never the data-part); noted in both CHANGELOGs; out of `ms_codec::Error` scope; fuzz oracle won't fire. No action.
- **[m3] `WrongHrp{got}` renders `got` verbatim via `{:?}`** — the construction-time 4-char cap is the only thing between it and a leak (intended design per R0 [I1]; render-only was rejected). All 3 production sites cap correctly; a future path constructing WrongHrp with an uncapped got would re-leak — the fuzz oracle is the regression gate. Informational.

## Security-closure assessment
**All Display+Debug leak paths are closed.** Evidence:
- **Non-vacuity PROVEN by revert-an-arm:** reverting the Codex32 manual match to `write!("{e:?}")` → 4/5 no-echo tests FAIL (catch 8-char windows `qpzry9x8`, `0entrsqg`); reverting the `got` cap to `.to_string()` → both `wrong_hrp_no_separator_does_not_leak` and the enshrining `…multibyte_does_not_panic` FAIL. Both restored byte-identical.
- **Leaky-variant interception complete + correctly ordered:** codex32-0.1.0 Error = 16 variants, exactly 3 carry a String (InvalidChecksum.string, MismatchedHrp, MismatchedId); all 3 matched FIRST, the `safe => "{safe:?}"` catch-all only receives the 13 safe variants. Independent scratch test constructed all 3 with a 49-char secret → Display+Debug clean.
- **Custom Debug non-leaking:** `#[derive(Debug)]` removed; hand-impl delegates to sanitized Display; `got` ≤4 chars after construction so Debug can't echo what isn't stored.
- **Fuzz exclusion really deleted + clean run:** `is_known_echo` + arms + call-guard GONE (grep 0 matches, deletion not comment-out); 90s run = 37747 runs, no finding. Oracle now scans Codex32(_) and WrongHrp{..} — permanent regression gate.

## Deliverable-conformance table
| Item | Conforms? | Notes |
|---|---|---|
| 1. Build + suite green | YES | `cargo build --workspace` 0 warnings; `cargo test -p ms-codec -p ms-cli` all pass. |
| 2. 5 red-first tests non-vacuous | YES | Scan BOTH Display+Debug; 49–50 distinct-char secret; correct 8-window slide; revert-an-arm proves they catch regressions. |
| 3. Codex32 match complete, 3 leaky FIRST | YES | 3 leaky matched before `safe => {:?}` catch-all; field::Error re-checked, no missed String. |
| 4. Custom Debug non-leaking | YES | derive gone; delegates to Display; got ≤4 chars; InvalidChecksum-50char Debug clean. |
| 5. 3 WrongHrp sites cap (char-counted) | YES | decode.rs:167, envelope.rs:67, envelope.rs:127 — all `.chars().take(4).collect()`; no byte-slice; no missed site. |
| 6. Fuzz exclusion deleted + clean | YES | deleted (not commented); 90s = 37747 runs clean. |
| 7. Enshrining test updated | YES | asserts 4-char cap; no-panic preserved; short-HRP survivors pass. |
| 8. No error-semantics regression | YES | Display wording unchanged (only stored got shortens); no test pins got>4. |
| 9. Release mechanics | YES | 0.4.4; ms-cli pin =0.4.4; both CHANGELOGs accurate; `cargo metadata --locked` OK; no ms-cli CHANGELOG. |
| 10. Downstream untouched | YES | ms-cli src untouched (only pin); toolkit separate repo; residual HRP/ID echo noted. |

## Evidence log
```
codex32-0.1.0 Error = 16 variants; leaky = InvalidChecksum.string / MismatchedHrp / MismatchedId only; field::Error SAFE
non-vacuity #1: revert Codex32 arm→{e:?} ⇒ 4/5 fail (qpzry9x8, 0entrsqg); restored
non-vacuity #2: revert got cap→.to_string() ⇒ wrong_hrp_no_separator + multibyte fail; restored
3 WrongHrp sites: decode.rs:167, envelope.rs:67, envelope.rs:127 — all chars().take(4)
custom Debug: derive removed; impl fmt::Debug → write!("Error(\"{self}\")"); 3 leaky + 50-char InvalidChecksum clean Display+Debug
fuzz: is_known_echo grep=0; 90s run 37747 runs no finding; corpus restored
release: 0.4.4 / pin =0.4.4; Cargo.lock 0.4.4; metadata --locked OK; both CHANGELOGs accurate
ms-cli src untouched; tree left exactly as found
```

GREEN — cleared for commit/publish. Only follow-up: the trivial fuzz/Cargo.lock → 0.4.4 bump (m1, folded).
