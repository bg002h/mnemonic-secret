# R0 round 0 — TESTS lens (sonnet) — `IMPLEMENTATION_PLAN_ms_hashlock_H1.md`

**Scope.** ONE QUESTION: for every guarantee the plan's tests claim to protect, would the
plan's own named MUTATION actually make that test fail — and which spec'd guarantee has NO
test that can fail on it? Plan reviewed at mnemonic-secret master `36d314daa98cb1a6d9212b47d1f44cfc04be47b8`
(identical to current master `33c9b350213033a2616918e04bc016995f962c14` for the plan file and
`scripts/plan-build-gate-ms.sh` — `git diff` between the two SHAs on those paths is empty).

**Method.** `cp -r` of the repo to `/scratch/code/shibboleth/.tmp/tests-lens-ms`; ran
`scripts/plan-build-gate-ms.sh design/IMPLEMENTATION_PLAN_ms_hashlock_H1.md` once
(`TMPDIR=/scratch/code/shibboleth/.tmp`), producing the wired scratch tree at
`/scratch/code/shibboleth/.tmp/plan-build-gate-ms` — build clean, 64/64 hashlock tests green,
clippy clean, fmt clean, codeword distance 17, downgrade row exit 2 / `reserved-prefix byte was
0x03`. Every mutation below was applied to that tree with a Python driver
(`cp` a pristine backup → apply one exact-anchor edit → `cargo build --workspace --all-targets`
→ `cargo nextest run` on the named test(s) → revert from the backup), one mutation at a time. No
sub-agents, no `.jsonl` reads, nothing committed. The tree was rebuilt and re-verified 64/64 green
after every batch and again as the final action below.

---

## 1. Mutation table — plan-declared (every `MUTATION:` comment in the plan, 18 distinct)

| # | Test | Mutation (plan's own words) | Result |
|---|------|------------------------------|--------|
| A1 | `hashlock_phrase_on_argv_is_refused_without_the_allow_flag_and_never_echoed` | drop `--hashlock-phrase` from `SECRET_FLAGS` | **FAILS-AS-CLAIMED.** (First attempt hit a compiler array-length mismatch since `SECRET_FLAGS: [&str; 5]` still declared 5; fixed to `[&str; 4]` — a real but incidental type-level guard, see M-4.) With the guard gone, `ms hashlock --hashlock-phrase "correct horse battery staple"` exits 0 and the phrase's derived card prints to stderr — `assert_eq!(out.status.code(), Some(1))` fails (`left: Some(0), right: Some(1)`). |
| A2 | `allow_argv_secret_admits_the_phrase_through_the_side_channel` | leave `hashlock` out of `override_applies` | **FAILS-AS-CLAIMED.** `--allow-argv-secret --hashlock-phrase …` no longer opts in; the invocation is refused instead of succeeding. |
| A3 | `admitted_phrase_does_not_read_stdin` | build the Source without consulting the admitted side channel | **FAILS-AS-CLAIMED.** Forced the phrase-argv branch to always `read_phrase_stdin()`; with stdin at `""` the derivation refuses `empty` instead of deriving from the admitted flag value. |
| A4 | `zero_sources_exits_64_listing_five` | zero sources defaults to stdin/ms1 instead of refusing | **FAILS-AS-CLAIMED.** `assert_eq!(out.status.code(), Some(64))` fails once zero-source falls through to the `Ms1` source instead of `CliError::Usage`. |
| A5 | `every_two_source_pair_exits_64` | check only a subset of pairs | **Not a production-code mutation — see note below.** |
| A6 | `method_with_a_supplied_preimage_exits_64_for_all_three_sources` | `--method` silently ignored with a supplied X | **FAILS-AS-CLAIMED.** `refuse_method` turned into a no-op; all three (`--hex`, `--random`, ms1) proceed instead of exit 64. |
| A7 | `random_requires_out_file_and_json_alone_does_not_satisfy_it` | gate on `--out \|\| --json` | **FAILS-AS-CLAIMED.** `ms hashlock --random --json` (no `--out`) now exits 0 instead of 64. |
| A8 | `random_twice_differs` | a fixed buffer | **FAILS-AS-CLAIMED.** Two `--random` invocations produce byte-identical output; `assert_ne!` fails. |
| A9 | `random_out_refuses_to_overwrite_but_other_sources_overwrite` | truncating writer for `--random`'s `--out` | **FAILS-AS-CLAIMED.** A second `--random --out` onto the same path now exits 0 and clobbers the file instead of refusing. |
| A10 | `decode_prints_kind_hex_and_digest_and_never_words` | leave `decode.rs`'s catch-all as `unreachable!` | **FAILS-AS-CLAIMED.** `ms decode` on a preimage plate panics, exit 101, exactly as the plan predicts. |
| A11 | `inspect_reports_the_kind_with_no_false_reason` | leave `inspect.rs`'s rule-6/8 copies untouched | **FAILS-AS-CLAIMED.** Restoring the pre-hashlock tag check (`tag_bytes != TAG_ENTR` alone) makes a valid preimage single print `FAIL: would NOT decode v0.1` with reason `unknown-tag`. |
| A12/A12b | `derive_and_verify_refuse_with_the_executable_remedy` | place the refusal AFTER `payload_entropy_and_language`'s catch-all | **A12 (reorder-but-not-past-catchall) is a semantic no-op — PASS, correctly, since `Entr` and `Preimage` patterns don't overlap.** **A12b (actually move the `Preimage` arm past the `_ => unreachable!` catch-all) — FAILS-AS-CLAIMED**: `ms derive`/`ms verify` on a preimage plate panic, exit 101. |
| A13 | `combine_prints_a_recovered_preimage_as_decode_does` | leave `combine.rs`'s catch-all | **FAILS-AS-CLAIMED.** Guarding the `Preimage` arm with `if false` (simulating "arm never added") makes a recovered preimage share set panic on combine. |
| A14 | `unreachable_catch_all_count_is_pinned` | add a fifth `_ => unreachable!` | **FAILS-AS-CLAIMED.** Census moves 4→5, `assert_eq!(n, 4)` fails as designed. |
| A15 | `stdout_is_exactly_the_record_under_out_and_under_sha256` | `--out` suppresses stdout (encode's shape) | **FAILS-AS-CLAIMED.** Under `--out`, stdout becomes empty instead of carrying the `hash:` record; the composer-feed contract breaks. |
| A16 | `hardened_warns_under_20_only` | hardened threshold at 19 or 21 | **FAILS-AS-CLAIMED** (tested 21: a 20-char phrase now spuriously warns). |
| A17 | `sha256_warns_at_every_length` | sha256 gated on length | **FAILS-AS-CLAIMED.** A 100-char sha256 phrase stops warning once the warning is length-gated. |
| A18 | `record_line_shape_is_what_me_sysw_pack_reads` | uppercase hex in `hash:` | **FAILS-AS-CLAIMED.** The `!b.is_ascii_uppercase()` assertion catches it immediately. |

**Note on A5.** `every_two_source_pair_exits_64`'s `MUTATION:` comment ("check only a subset of
pairs") describes a hypothetical *test-authoring* lapse, not a production-code bug — there is no
code path to mutate that corresponds to it. I verified by direct inspection (not a runtime
mutation) that the **shipped** test's nested loop over 5 source-arg-vectors enumerates all
C(5,2)=10 pairs, including the named stdin-contention pair (`--hashlock-phrase-stdin` at index 1
against the bare `-` positional at index 3). This is VERIFIED-BY-INSPECTION, not a PASS-STILL —
the shipped test genuinely does cover all 10 pairs.

**18/18 plan-declared mutations produced the claimed result** (17 direct FAILS-AS-CLAIMED + 1
semantic non-mutation confirmed sound by inspection). Two required a construction fix on my side
(A1's array-length type, A17's non-exhaustive match) to actually reach the runtime test rather than
tripping over an incidental compile error of my own mutation's making — noted, not a plan defect.

---

## 2. Mutation table — reviewer-added (from the spec's guarantees, brief §2)

All run against the broad filter `binary(/hashlock/) | test(/hashlock/)` (all 64 hashlock tests);
"catches" names the specific test(s) that failed.

| # | Mutation | Result / catching test |
|---|----------|------------------------|
| B1 | swap the two derivation methods at the CLI dispatch site | **FAILS.** Caught by `json_both_variants`, `stdout_is_exactly_the_record_under_out_and_under_sha256`, `byte_exact_rows_on_both_channels`, `allow_argv_secret_admits_the_phrase_through_the_side_channel`, `lockstep_100_and_101` (5 tests). |
| B2 | `HASHLOCK_ITERATIONS` 100_000 → 10_000 | **FAILS.** Caught by `anchor_rows_both_methods_pin_x_and_h` and `hashlock_repro_three_ways` (independent literal-anchored). |
| B3 | `HASHLOCK_SALT` one byte off (`v1`→`v2`) | **FAILS.** Same two tests. |
| B4 | `HASHLOCK_DKLEN` 32→16 | **BUILD_FAILED at compile time** (`Zeroizing<[u8;16]>` vs the function's declared `Zeroizing<[u8;32]>` return type) — caught by the type system, not by a test. See M-4. |
| B5 | `digest()` call site ignores X, hashes a fixed value | **FAILS.** Caught by `json_both_variants` (hash_record mismatch) and others. |
| B6 | `strip_one_trailing_newline` strips two | **FAILS.** Caught by `strip_exactly_one_newline` (unit test: `"abc\n\n"` must become `"abc\n"`, not `"abc"`). |
| B7 | printable-ASCII range implemented as `is_ascii()` | **FAILS.** Caught by `printable_boundary_is_pinned_on_both_sides` (TAB no longer refused) — this is also Task 6's own named RED-step mutation; see §5. |
| B8 | phrase cap off-by-one (`>` → `>=`) | **FAILS.** Caught by `cap_at_100` (100 chars now spuriously refused). |
| B9 | `looks_like_ms1` without the case fold | **FAILS.** Caught by `ms1_shape_in_four_spellings_and_before_the_cap` and `refusals_in_four_spellings_on_both_channels_name_the_ms1_route` (the UPPERCASE spelling). |
| B10 | 64-hex guard widened to `>= 64` (only the *lower* boundary at 63 is deliberately pinned) | **FAILS — but incidentally.** Caught by `lockstep_100_and_101`, `sha256_warns_at_every_length`, `cap_at_100`, `printable_ascii_boundary_and_cap` — **all of which use degenerate filler phrases (`"a".repeat(100)`, `"b".repeat(n)`) that happen to be all-hex-digit strings.** There is no test that deliberately asserts a realistic (non-degenerate) phrase longer than 64 all-hex-looking characters is *accepted*. See I-4. |
| B11 | ms1-shape check moved to run AFTER the length cap | **FAILS.** Caught by `ms1_shape_in_four_spellings_and_before_the_cap` (the 112-char grouped-by-2 row) and `refusals_in_four_spellings_on_both_channels_name_the_ms1_route`. |
| B12 | `PREIMAGE_PREFIX` 0x03 → 0x01 | **FAILS.** Caught trivially by `constants_are_the_specs`'s own literal assertion. |
| B13 | reintroduce the fixed-range slice index (`&data[1..33]` + `.unwrap()`) that panics on a short payload — the exact class SPEC_ms_hashlock §1 says the length-check-before-construction rule fixes | **FAILS.** Caught by `preimage_length_rows_through_decode_name_their_error` (now panics instead of returning `PreimageLengthMismatch`). |
| B14 | `TagKindMismatch` never raised on decode (remove the standalone rule-6b pre-check) | **FAILS.** Caught by `id_and_prefix_must_agree_both_directions` (the `hash`-tag-over-`0x00`-payload direction silently decodes instead of refusing; the other direction remains protected by the per-tag arm's own catch-all, incidentally). |
| B15 | `TagKindMismatch` never raised on encode | **FAILS.** Caught by the same test, encode side. |
| B16 | `RESERVED_ID_BLOCKLIST` without `hash` | **FAILS** — but tautologically: `constants_are_the_specs` asserts the constant's own length/contents directly, so this is a self-referential pin rather than a behavioral test (no test actually generates share ids and confirms none collides with `hash`). See M-3. |
| B17 | `--method` silently ignored, isolated to `--hex` only (partial-fix variant, distinct from A6's all-three removal) | **FAILS.** Caught by `method_with_a_supplied_preimage_exits_64_for_all_three_sources`'s first loop iteration. |
| B18 | `--json`'s `method` key populated even for a supplied X (`None` arm no longer empty) | **FAILS.** Caught by `json_both_variants`'s `v.get("method").is_none()` assertion. |
| B19 | `emit_preimage` (the `ms decode` renderer) prints an extra line of prose text not containing any of the test's four blocklisted words | **PASS-STILL. Real finding — see I-3.** |
| B20 | delete `constants_equal_the_literals` (the "repro test built from constants" concern) | **PASS-STILL in isolation** (expected — nothing else changed) but **the compound check (delete the pin test AND independently mutate `HASHLOCK_SALT`) still FAILS**, via `hashlock_repro_three_ways`'s "Rust vs literal" assertion. This shows the *real* protection against a constant mutation is the test's independent hardcoded `EXPECTED_X`/`EXPECTED_H` literals, not `constants_equal_the_literals` — which is a genuinely redundant (belt-and-suspenders) check for this mutation class. Its actual job (guarding against a *future* refactor where `python_x()`/`openssl_x()` are rewritten to source SALT/ITER/DKLEN from the crate's own constants, at which point the cross-tool comparison would become self-referential) was not itself falsifiable without rewriting the test in the way the design deliberately avoids — verified by inspection instead: `EXPECTED_X`/`EXPECTED_H` in `hashlock_repro.rs` are `const &str` literals, never computed from the crate. |
| — | `--random`'s `--out` truncating; `--out` suppressing stdout; `--random` gated on `--out \|\| --json` | Covered as A9, A15, A7 above (plan already named these). |
| — | uppercase hex in `hash:` | Covered as A18. |
| — | four `unreachable!` arms, one at a time | `decode.rs`'s early-return arm = A10; `combine.rs` = A13; `payload_lang.rs` = A12b. The spec's table lists a *second* `decode.rs` site (`:112`) as a separate "functional" arm, but its `Payload::Preimage`-shaped catch-all is **provably unreachable for this kind**: the first match's arm does `return emit_preimage(...)`, so control never reaches the second match for a `Preimage` payload. Confirmed by reading, not run — mutating it is a no-op (dead code), so it is not a meaningful mutation target for THIS kind (it exists to guard a hypothetical future `0x04`). |
| — | derive/verify refusal placement | Covered as A12b. |
| — | inspect still pushing `unknown-tag` | Covered as A11. |
| — | `#[cfg(test)]` module count pin | **NONE.** No such construct exists anywhere in the plan (grepped `module count`, `cfg(test)` — the only hits are three unrelated `#[cfg(test)] mod tests` block openers). Not applicable. |

**Follow-up confirmation (not a mutation, a robustness check):** does the catch-all *count* test
(`unreachable_catch_all_count_is_pinned`) survive a spelling change rather than just an addition?
Rewrote `combine.rs`'s arm from `_ => unreachable!(...)` to `_ => todo!(...)` (same panic-on-reach
semantics, different literal text) — the census drops 4→3 and the test correctly FAILS
(`left: 3, right: 4`). The count test is sound against both additions and substitutions of the
pattern it tracks; a scan of all of `crates/ms-cli/src` also confirms there is no OTHER unrelated
`"_ => unreachable!"` occurrence that could inflate its baseline.

---

## 3. Three false-PASS structural analyses (brief item 3)

**(a) The reproduction test (`hashlock_repro_three_ways`), shim resistance.**
- *Fake `python3` that ignores its arguments and echoes the expected hex*: built a shim, prepended
  it to `PATH`, ran the test — **it PASSES.** `rust_x`, `py`, `ssl` and `EXPECTED_X` all still
  compare equal (`py` is now a constant, `ssl` is the real openssl, both happen to match the
  literal). **This is a real false-PASS**, inherent to any subprocess-comparison design: a
  compromised/shadowed system tool defeats the entire cross-tool check silently. Classified
  Important (I-3 below) rather than Critical — it requires a compromised environment (not a code
  mutation), and it is a CI-fidelity guarantee rather than a runtime security boundary. Worth
  recording as a known limitation of "shell out and compare" test designs generally.
- *`openssl` absent/non-functional, `python3` present*: built a failing `openssl` stub, prepended
  it to `PATH` (full PATH otherwise intact) — **the test FAILS LOUDLY** (`panicked at
  hashlock_repro.rs:83: openssl kdf failed: …`), never `ok`, never skipped. This half of the
  spec's claim ("FAILS if either tool is absent — never `#[ignore]`, never a `cfg` gate") holds.

**(b) The catch-all count test.** Counts the right pattern (`"_ => unreachable!"`, exact substring,
across all of `crates/ms-cli/src`); no unrelated occurrence inflates the baseline (confirmed by
grep); a `_ => todo!()` substitution for one of the four arms is caught (count moves 4→3, assert
fails). No gap found here.

**(c) The `Zeroizing` type pin (`preimage_field_is_zeroizing` in `hashlock_kind.rs`).** This is a
REAL test, not prose — it is a **compile-time** assertion masquerading as a runtime one: `let _: &Zeroizing<[u8; 32]> = z;`
after `if let Payload::Preimage(z) = &p`. If the variant's field were ever changed to a bare
`[u8; 32]`, `z`'s inferred type would no longer match the annotation and the WHOLE CRATE fails to
build (confirmed independently via the DKLEN experiment above and via direct reasoning about match
ergonomics — a bare-array field means `z: &[u8; 32]`, and the type ascription to
`&Zeroizing<[u8; 32]>` would then be a type error). The self-review's claim that this was upgraded
from prose to "a type-level assertion the compiler enforces" is accurate.

---

## 4. Corpus sufficiency for the Go port (brief item 4)

**`crates/ms-codec/tests/vectors/hashlock-v0.8.json` is not currently loaded by ANY test** — grepped
every `.rs` file under `crates/ms-codec/tests/` and `crates/ms-cli/tests/` for
`include_str!`/`from_str` referencing `hashlock-v0.8.json`: zero hits (contrast with the existing
`v0.1.json` corpus, which IS loaded by `vectors.rs` and `vectors_parity.rs`). The file exists purely
as documentation/future-vendoring material at gate-GREEN time.

**`kind`, `refusals`, `lengths_by_door`, `lockstep` sections** carry concrete values (no
placeholders) except the `refusals` section's four `<the kind[0].ms1 string, …>` bracketed
descriptions — but the underlying guarantees they describe (ms1-shape refusal in four spellings, on
both channels) **are** independently, concretely tested in `hashlock_phrase_rule.rs` and
`hashlock_phrase.rs`'s own unit tests. So a Go port missing THESE specific corpus values would still
be caught by the Rust-side test suite; only the JSON-vendoring artifact needs values filled before
H2's fork-side pin test can consume it.

**`derivation` section: 9 of its 10 rows are literal `"…"` placeholders** — only the anchor phrase
(`"correct horse battery staple"`) is filled. These 9 unfilled rows are EXACTLY the boundary set
spec §8 names as required: 1 character, 20 characters, 64/65 (HMAC block boundary), 100/101, the
"  a  b " spaces row, and the "correct-horse,battery staple" / "a-b,c" hyphen-comma rows. Task 4
Step 1 explicitly says "the implementer fills every `"…"`" — disclosed, not hidden — **but no plan
step, test, or CI check anywhere verifies completion**: nothing greps the shipped corpus for a
remaining `"…"`, and nothing loads the JSON to cross-check it against `preimage_hardened`/
`preimage_sha256`. The PARALLEL Rust-side pin (`hashlock_derivation.rs`'s `ROWS` const, which IS
executed) independently has only the same one anchor row, with an inline comment ("The implementer
fills the remaining rows…") deferring the rest — no task step compels expanding it either.

**Net effect for the Go-port question asked:** at the state this plan builds to gate-GREEN, a
behaviour-faithful Go port that (a) trims a trailing space, (b) folds case on the phrase, or (c)
accepts a tab would face:
- **(a) trailing space** — the `"  a  b "` corpus row exists by NAME but carries no value to compare
  against (all four hex fields are `"…"`), so a Go port cannot be checked against it until filled.
  The Rust-side `bytes_are_used_verbatim` unit test DOES independently catch a trailing-space
  mutation on the Rust side (not corpus-vendored, so it protects Rust but nothing that reads only
  the JSON corpus).
- **(b) case-folding the phrase** — **NO row, NO test, anywhere in the plan — Rust or corpus —
  exercises this.** Confirmed by grep across every hashlock test file: no test compares
  `preimage_hardened("ABC")` against `preimage_hardened("abc")` or any case-varied pair through the
  derivation path (the only case-related tests concern ms1-shape-refusal case-folding and 64-hex
  refusal case-folding, a different guarantee). Spec §4.3 explicitly requires "no case folding" as
  part of the byte-verbatim phrase rule — this is a spec'd guarantee with zero test coverage. **I-1.**
- **(c) accepting a tab** — covered: `printable_boundary_is_pinned_on_both_sides` and the corpus's
  `refusals` row both pin TAB refusal concretely (this one is fine).

---

## 5. The RED steps (brief item 5) — five tasks checked

| Task | Plan's "Expected" claim | Verified |
|------|--------------------------|----------|
| **Task 1** (consts/tag/error, line 595) | `cargo test -p ms-codec --test hashlock_kind` FAILS to compile: `PREIMAGE_PREFIX`, `TAG_HASH` etc. do not exist | **CONFIRMED.** Reverted `consts.rs`'s Task-1 additions on the wired tree; `cargo build -p ms-codec --tests` produced exactly `unresolved import crate::consts::PREIMAGE_PREFIX` / `TAG_HASH` / `VALID_PREIMAGE_STR_LENGTHS` (4 errors). Reverted back; workspace rebuilt clean. |
| **Task 2** (payload/dispatch, line 802) | FAILS to compile: `Payload::Preimage`, `InspectKind::Preimage` | **CONFIRMED.** Reverted just the `Payload::Preimage` variant addition; `cargo build -p ms-codec --tests` produced exactly `E0599: no variant or associated item named 'Preimage' found for enum 'Payload'` (6 occurrences, 2 build targets). Reverted back; rebuilt clean. |
| **Task 5** (argv guard, line 1705) | "every test FAILS — `ms hashlock` is not a subcommand" | **CONTRADICTED for 3 of 11 tests — Minor.** Renamed the clap subcommand away from `hashlock` (`#[command(name = "hashlock_disabled_for_red_step_check")]`) to simulate pre-Task-7 state, keeping Task 5's argv-guard fragments in place (as the plan sequence has them already shipped by this point). Result: 8/11 tests fail as claimed (`error: unrecognized subcommand 'hashlock'`), but **3 continue to PASS**: `hashlock_phrase_on_argv_is_refused_without_the_allow_flag_and_never_echoed` (the argv guard refuses BEFORE clap ever runs, independent of subcommand registration — reasonable, not a defect), and, more interestingly, `method_with_a_supplied_preimage_exits_64_for_all_three_sources` and `every_two_source_pair_exits_64` — **because this CLI maps clap's own "unrecognized subcommand" parse error to exit 64, the SAME code `CliError::Usage` uses.** Verified directly: `ms hashlock_disabled_for_red_step_check --hex - --method sha256` (renamed) exits 64 with `error: unrecognized subcommand 'hashlock'`, and the correctly-refused case (`ms hashlock --hex - --method sha256` on the real binary) ALSO exits 64. Two of the plan's own exit-64-only assertions cannot, on their own, distinguish "the guard/verb correctly refused" from "the subcommand doesn't exist at all." Reverted; rebuilt clean, 64/64. |
| **Task 6** (phrase rule, line 1992) | `MUTATE (0x20..=0x7e) to is_ascii() → printable_boundary_is_pinned_on_both_sides FAILS on TAB` | **CONFIRMED** (= mutation B7 above): exact test named, exact failure mode (TAB no longer refused). |
| **Task 8** (other verbs, line 2630) | `decode_*`, `combine_*` exit 101; `inspect_*` fails on `unknown-tag`; `derive_and_verify_*` exit 101; `unreachable_catch_all_count_is_pinned` still passes (4) | **CONFIRMED**, composed from mutations already run above: A10 (decode exit 101), A13 (combine exit 101), A11 (inspect `unknown-tag`), A12b (derive/verify exit 101), and the baseline pristine run (count test passes at 4). |

---

## 6. Closing counts

- **Critical: 0.**
- **Important: 4.**
  - **I-1** — Phrase case-folding during derivation (spec §4.3: "no case folding") has **no test
    anywhere** — Rust or corpus. A regression that lowercased/uppercased the phrase before hashing
    would ship undetected. *Funds-adjacent (preimage-derivation-exactness) guarantee with no test.*
  - **I-2** — The hashlock corpus's `derivation` array is 9-of-10 rows unfilled `"…"` placeholders,
    nothing loads/validates the JSON file at all, and the parallel Rust-side pin
    (`hashlock_derivation.rs`'s `ROWS`) independently has only the one anchor row — so at
    gate-GREEN, **none** of spec §8's named boundary derivation rows (1-char, 20-char, 64/65-byte
    HMAC boundary, 100/101, spaces, hyphen-comma) has any executable pin, disclosed as a TODO but
    with no step/test/CI enforcing completion before merge or before the corpus's SHA is pinned
    into the CHANGELOG (Task 11).
  - **I-3** — `ms decode`'s "never words" guarantee (spec §5) is enforced by a fixed 4-substring
    blocklist (`["abandon","sentence","entry","word"]`), not a structural check. Empirically
    demonstrated PASS-STILL: injecting an arbitrary extra line of prose text not containing those
    four substrings passes `decode_prints_kind_hex_and_digest_and_never_words` undetected (mutation
    B19). Separately, the reproduction test's cross-tool comparison is defeated by a shimmed
    `python3` that echoes the expected value — a related false-PASS class, recorded against the
    same guarantee family (test fidelity, not funds-safety).
  - **I-4** — The 64-hex phrase guard's upper boundary (spec §4.3: "exactly 64 hex" is refused,
    everything else accepted) is protected only incidentally: every test that currently exercises a
    phrase longer than 64 characters happens to use a degenerate single-character-repeated filler
    (`"a".repeat(100)`, `"b".repeat(n)`) that is coincidentally all-hex-digit. No test deliberately
    confirms a realistic 65–100 character phrase that happens to look like hex is accepted rather
    than mis-redirected to `--hex`.
- **Minor: 3.**
  - **M-1** — Task 5's RED-step claim ("every test FAILS") is false for 3/11 tests, traced to this
    CLI mapping clap's own subcommand-not-found error to the same exit code (64) as
    `CliError::Usage`; two of the plan's exit-64-only assertions can't discriminate the two causes.
  - **M-2** — the plan's `MUTATION:` comment on `every_two_source_pair_exits_64` doesn't correspond
    to a real code mutation (test-authoring-lapse framing); resolved by direct inspection, not a
    defect (the shipped test genuinely covers all 10 pairs).
  - **M-3** — `RESERVED_ID_BLOCKLIST` (B16) and `PREIMAGE_PREFIX` (B12) mutations are caught only by
    tautological self-referential assertions (`constants_are_the_specs` re-asserting the constant's
    own literal value), not by any behavioral/generative test — sufficient for the named mutation,
    weaker than it looks.
- **Nit: 1.**
  - **N-1** — `HASHLOCK_DKLEN` (16) and the `SECRET_FLAGS` array-length mutation (A1's first
    attempt) are caught by the Rust type system at compile time rather than by any test — stronger
    protection than a test, but means "no test can fail on this" is the wrong frame for these two;
    noted for completeness rather than filed as a gap.

**18/18** plan-declared (Group A) mutations behaved as claimed (17 direct + 1 resolved by
inspection). **19/20** reviewer-added (Group B) mutations were caught by an existing test or the
compiler; **1 (B19) was a genuine PASS-STILL**, folded into I-3 above. All five sampled RED steps
were checked against the actual wired tree; four confirmed exact, one (Task 5) partially
contradicted (M-1). The scratch tree was restored to pristine (64/64 hashlock tests green, full
workspace `cargo build --workspace --all-targets` clean) after every mutation and as the final
action before this report.
