# Review: baseline GAP-B hardening, ms (PR #28, base 9b3332e, head fd10f23)

Reviewer: Claude Sonnet 5.5 (claude-sonnet-5-5), medium. Author: Sonnet implementer.

1. Production code: `git diff --stat 9b3332e..fd10f23` = tests/gapb_hardening.rs and design/agent-reports/harden-baseline-gapb.md only. OK.
2. Mutants re-applied by hand (scratch edits, each reverted; worktree clean afterwards):
   - CHARS_INV `-1` -> `1` at bytes 0, 9, 31, 32, 33, 47, 58, 64, 73 (I), 79 (O), 91, 95, 105 (i), 111 (o), 127: all 15 FAIL chars_inv test.
   - mod.rs:199 `< 94` -> `<= 94`: from_string_length_94 FAILS (got InvalidChecksum, expected InvalidLength(94)).
   - bch_decode.rs:238 `m += 1` -> `m *= 1`: bm test FAILS (len=16 pos=[19,46] must repair: uncorrectable).
   Baseline: 3 passed. All 17 reproduced; no claim unreproduced.
3. Expected values: 94 is outside BIP-93's regular range 48..=93 (spec text, not code echo); BM test asserts the original payload and the exact injected position set, independent of the decoder. chars_inv asserts the refusing char; count 72 = 128 - 55 alphabet bytes - '1' checks out. The '1' EQUIV (split at last '1') is consistent with the code.
4. Robustness: fixed-seed xorshift, no timing; debug-profile run of all 3 tests 0.045 s total (profile shows optimized+debuginfo per repo config).
5. EQUIV: only the CHARS_INV '1' entry (49), reason holds.

Minor: author reports all 72 CHARS_INV entries failing; I sampled 15 of 72 only (the script claim is plausible, logs not re-run).
Nit: the report's "0.1 s" for the BM test vs 0.045 s total here (immaterial).

C 0 / I 0 / M 1 / N 1
