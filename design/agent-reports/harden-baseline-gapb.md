# Baseline GAP-B hardening, ms (base 9b3332e06e29547ce55fa3e257780e8310114bde)

Model: Claude Sonnet 5.5 (claude-sonnet-5-5). Tests only: `crates/ms-codec/tests/gapb_hardening.rs`.
Logs: /scratch/cache/ms-harden-gapb-logs/ (chars_inv_mutants.log, mut94.log, mutbm.log, base.log, nextest.log).

Base run (all PASS): `PASS from_string_length_94_is_invalid_length`, `PASS chars_inv_non_alphabet_bytes_refused_as_invalid_char`,
`PASS bm_repairs_every_random_2_to_4_error_pattern_with_exact_positions` (3 tests run: 3 passed).

## Group 1: field.rs CHARS_INV (70 graded survivors)
Test `chars_inv_non_alphabet_bytes_refused_as_invalid_char`: takes a valid ms1 string containing a `p`, replaces it with each of
the 72 non-alphabet ASCII bytes other than '1' (lowercase letters in the lowercase string, all others in the all-uppercase
string), asserts `Codex32String::from_string` returns `InvalidChar(that char)`. Scripted loop turned each `-1` entry in
CHARS_INV (lines 78..85) into `1`, one at a time, reverting after. Result: all 72 entries (the 70 survivors plus the
already-caught 'B' and 'b') FAIL the test (`FAIL ... chars_inv_non_alphabet_bytes_refused_as_invalid_char`, rc=100 each; see
chars_inv_mutants.log). The entry for '1' (81:9) passes under the mutant (rc=0): equivalent, as graded (the data is split at
the last '1', so it never reaches `from_char`). KILLED 70 (72 verified).

## Group 2: codex32/mod.rs:199:64 `<` -> `<=`
Test `from_string_length_94_is_invalid_length`: "ms1" + 91 'q' (94 chars) must give `InvalidLength(94)` (BIP-93 regular
strings are 48..=93). Mutant: `FAIL ms-codec::gapb_hardening from_string_length_94_is_invalid_length` (the mutant reaches the
checksum step and returns InvalidChecksum). KILLED 1.

## Group 3: bch_decode.rs:238:15 `m += 1` -> `m *= 1`
Test `bm_repairs_every_random_2_to_4_error_pattern_with_exact_positions`: fixed-seed xorshift64, 400 patterns of 2..=4
errors per length x 5 lengths (2000 patterns, 0.1 s debug), asserts the payload and exact position set. Mutant:
`FAIL ms-codec::gapb_hardening bm_repairs_every_random_2_to_4_error_pattern_with_exact_positions`. KILLED 1.

## Gates (PATH cargo 1.85.0, --locked, CARGO_TARGET_DIR=/scratch/cache/target-ms-harden-gapb)
- `cargo +1.95.0 fmt --all -- --check`: clean (after fmt of the new file).
- `cargo clippy --locked -p ms-codec --all-targets -- -D warnings`: clean.
- `cargo nextest run --locked -j 8 -p ms-codec --no-fail-fast`: 213 passed, 5 skipped.
- `cargo test --locked -p ms-codec --doc`: 1 passed.
- vendor-freshness.yml scripts (sha256 check, vendor-freshness.sh, vendor-freshness.test.sh): pass.
- Not run: the ms-cli `test` job, `test-release-mlock-einval` (uses --release, forbidden here), miri, fuzz-smoke, man-release,
  release.yml. No ms-cli or production code changed (tests only); miri/fuzz not run for lack of a stated need.

The 72 GAP-B survivors in the grader list: 70 (CHARS_INV) + 1 + 1.

KILLED 72 / EQUIV 0 / STOP 0 / BASE-FAIL 0
