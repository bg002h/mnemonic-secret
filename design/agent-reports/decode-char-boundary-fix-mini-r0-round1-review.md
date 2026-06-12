# Mini-R0 Review — decode_with_correction char-boundary fix (round 1)

Reviewer: Fable 5 architect agent (af7a690b1099c8382), 2026-06-12.
Target: design/PLAN_decode_with_correction_char_boundary_fix.md @ mnemonic-secret 493c5de.
Persisted verbatim per CLAUDE.md convention.

## Verdict: GREEN

0 Critical / 0 Important. The bug, the fix's correctness, char-safety across all cases, no-regression, and the other-slices sweep are all empirically verified against the live crate @ 493c5de. The plan is correct and complete; proceed to implementation. Two Minor refinements below are non-blocking.

## Critical
- None.

## Important
- None.

## Minor
- **M1 (Q1 framing — leak-surface is leak-NEUTRAL, plan slightly mis-attributes why).** The plan says the no-`'1'` whole-string `got` is "a more honest got" and the fix "does NOT expand the leak surface." Correct conclusion, but the *reason* matters: per the leak FOLLOWUP (`ms-codec-error-display-echoes-input`), the actual `WrongHrp.got` leak vector is the **WITH-`'1'` path** (a data-char→`1` mutation makes `rfind('1')` land deep in the data-part → `got` becomes a long secret prefix). That path is **byte-identical old↔new** — this fix is orthogonal to it. The no-`'1'` arm (the only behavior delta) is reached only by strings with zero separators, which are not structured ms1 shares; the old `lower[..len-1]` already echoed the whole input minus one byte, so whole-string vs chopped is leak-neutral. Recommend the CHANGELOG/commit note phrase it as "leak-neutral (the leak vector is the unchanged WITH-`'1'` path; bounding it is the separate FOLLOWUP's job)" rather than "more honest." Non-blocking.
- **M2 (release-mechanics nit).** `ms1_decode` is currently held out of the *smoke matrix* only — it is still in the *build/compile gate*. So the re-enable is a one-line matrix add (`- ms1_decode`) plus removing the hold-out NOTE comment. No build-job change needed.

## Answers to open questions
**Q1 — `None => lower.clone()` vs empty vs bounded:** Keep `lower.clone()` (whole string). It does not worsen the echo (leak-neutral vs the old chopped value, and orthogonal to the real WITH-`'1'` leak vector which is unchanged). Empty `got` would lose the one useful diagnostic for a no-separator input for zero security gain. Proper bounding of `WrongHrp.got` belongs to the separate `ms-codec-error-display-echoes-input` cycle.

**Q2 — ms-cli version bump:** **Pin-only; do NOT bump ms-cli's version.** Git history: `c64b1eb` (ms-codec 0.4.0→0.4.1) and `e83a7fe` (0.4.1→0.4.2) both bumped *only* the `=0.4.x` exact pin in ms-cli/Cargo.toml line 20 — neither touched ms-cli's own version. `=0.4.2`→`=0.4.3` pin-only matches precedent exactly.

**Q3 — other char-boundary slices (adversarial sweep):** **None. `parse_ms1_symbols` is the SOLE offender — empirically proven.** The only `&str` slices in ms-codec on computed indices: decode.rs:154 (the fix), decode.rs:159 (`&lower[HRP_PREFIX.len()..]` — ASCII-constant index, char-safe), envelope.rs:78 (`&s[..sep]`), shares.rs:329 (`#[cfg(test)]`). `sep = rfind('1')` is always a char boundary ('1' is ASCII), AND every one of those runs *after* `Codex32String::from_string()` validation (string guaranteed pure-ASCII bech32). `parse_ms1_symbols` is unique in slicing a *raw, unvalidated, pre-codex32* user string. Proven by running all 4 public entries against 20 adversarial multibyte inputs with the buggy code: every panic came exclusively from `decode_with_correction`.

**Q4 — does `decode` truly never reach this for ALL no-`'1'` multibyte inputs:** **Yes, unconditionally.** `decode`'s first statement gates on byte-length (`is_known_length(s.len())`) before any parse/slice; no-`'1'` multibyte inputs fall outside the known-length set → immediate `Err(UnexpectedStringLength)`. Even a length-matching multibyte string hits `Codex32String::from_string` (rejects non-bech32) before any envelope slice. Structural and unconditional.

## Release mechanics sanity
- Version 0.4.2→0.4.3 PATCH: correct (bugfix, no API/wire change).
- CHANGELOG: both locations exist; root uses `## ms-codec [0.4.3]`, crate uses bare `## [0.4.3]`.
- Cargo.lock: updates for new ms-codec version.
- fuzz-smoke.yml re-enable: correct (matrix-add only per M2); the ms1_decode harness drives exactly the fixed path and is the regression gate.
- Defer-publish stance: correct and mandatory — do NOT `cargo publish` or bump the toolkit pin autonomously.

## Evidence log
```
BUG (buggy code @ HEAD): decode_with_correction(lossy[0xaa]"�") -> PANIC byte index 2 not char boundary inside '�'
  decode -> Err(UnexpectedStringLength{got:3}); inspect -> Err(Codex32(InvalidLength(3)))  # both clean
UTF-8 INVARIANT: scan U+0080..U+10FFFF for any char whose UTF-8 contains 0x31 -> 0 matches
  => rfind('1') can NEVER land inside a multibyte char → i always a boundary
OLD vs NEW got on WITH-'1' inputs: 0 mismatches (byte-identical)
FIX APPLIED, cargo build -p ms-codec EXIT 0; decode_with_correction("�")->Ok(Err(WrongHrp{got:"�"})) no panic;
  "xy1qqq"->WrongHrp{got:"xy"} unchanged
NO REGRESSION: cargo test -p ms-codec all pass; no test pins the no-'1' got value
SWEEP: 20 adversarial inputs × 4 entries, panics ONLY from decode_with_correction
Q2 precedent: c64b1eb & e83a7fe bumped ms-cli pin only, version unchanged
TREE LEFT AS FOUND: scratch removed; git diff crates/ clean
```

GREEN — implementation may begin. Carry M1's phrasing tweak and M2's matrix-only note as non-blocking polish.
