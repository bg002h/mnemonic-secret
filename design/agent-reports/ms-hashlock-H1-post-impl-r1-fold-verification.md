# H1 post-implementation round 1 — fold verification (sonnet, targeted)

**Reviewer:** independent fold-verification reviewer (sonnet), no sub-agents.
**Brief:** `design/agent-briefs/ms-hashlock-H1-post-impl-r1-fold-verification-brief.md`.
**Under verification:** branch `hashlock-h1`, fold commits `a150ba7..447eb09`
(nine commits), against round 0's `design/agent-reports/ms-hashlock-H1-post-impl-review.md`
(opus, `b776253` on `master`, 2C/3I/6M/2N at `a150ba7`).
**Method:** built and ran in a detached worktree at `447eb09`
(`/scratch/code/shibboleth/ms-worktrees/h1-verify`, since removed) with its own
`CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/h1-verify-target` (since
removed). All five mutations below were applied, the named test run, and the
file restored from a backup immediately after; `git status --short` was clean
in that worktree, in `ms-worktrees/hashlock-h1`, and in the main checkout at
every check. Nothing was committed anywhere. Every count below is measured at
`447eb09`, not carried from the report.

## The one question

Did the fold fix every Critical and Important of the round-0 review, with a
test that can FAIL for each, and without a regression or a false claim of its
own?

**Answer: yes — all 5 C/I rows FIXED, all 8 M/N rows closed as claimed, one
regression check clean. GREEN.**

## Finding table

| # | finding | verdict | evidence |
| --- | --- | --- | --- |
| C-1 | `--json` `PrivateKeyMaterial` advisory had no test that could fail | **FIXED** | `json_both_variants` now passes `--no-engraving-card` on both invocations and asserts `stderr == ADVISORY` (byte-exact const), not a substring disjunction. Ran green (11/11); deleting the advisory emission in `cmd/hashlock.rs` made exactly `json_both_variants` FAIL with `left: "", right: "warning: stdout carries private key material…"` — the failure is the advisory, not the card. Reverted. |
| C-2 | implementation report's final-gate count was false (535 claimed, 554 measured); explanation also false | **FIXED** | Re-measured at `447eb09`: `cargo nextest run --workspace --locked --no-fail-fast` → `559 tests run: 559 passed, 11 skipped`; `cargo test --workspace --locked` summed to `560 passed, 0 failed, 11 ignored`; clippy `-D warnings` exit 0; `cargo fmt --all -- --check` exit 0 (1.85.0) and under `+1.95.0` exit 0; `grep -c "ms-cli::bin/ms"` on the nextest log = 74. Every one of these matches the report's "Post-review fold — Final gate" section exactly. The false parenthetical ("doc-tests and the `--bin ms` unit tests it reports separately") no longer appears asserted as fact anywhere in the report — its only occurrence is inside a `> **CORRECTED 2026-09-04**` blockquote quoting it to refute it. |
| I-1 | terminal prompt said "then Enter"; Enter did not end the read | **FIXED** | pty test (`pty.fork()`, real terminal): typed the phrase + Enter → tool returned immediately with `hash:3cf5d421…`, exit 0 (no Ctrl-D needed). Control from a PIPE: `printf 'a\nb'` (embedded `\n`, no trailing newline) → refused as non-printable ASCII at position 1 — proves the pipe path is still `read_to_end`. Prompt text unchanged: `Type the hashlock phrase, then Enter.` (verbatim, matches `SPEC_ms_hashlock.md:442`). |
| I-2 | `--separator` had no value parser; unengravable plate strings possible | **FIXED** | `ms hashlock --separator ab --hashlock-phrase-stdin` and `ms encode --hex … --separator ab` give byte-identical refusal: `error: invalid value 'ab' for '--separator <SEPARATOR>': invalid separator "ab"; expected \`space\` (or the literal " ")`, both exit 64. |
| I-3 | omitted `--hashlock-phrase` value swallowed the next flag, derived a preimage at exit 0 | **FIXED** | Reviewer's exact counterexample now refused: `ms hashlock --allow-argv-secret --hashlock-phrase --json --no-engraving-card < /dev/null` → exit 64, no `hash:` line, `error: --hashlock-phrase was given "--json", which is a flag and not a value…`. `--hashlock-phrase=--json` (equals form) still admits the value → exit 0, `hash:329367945b…` (matches the PBKDF2-of-`--json` value from the pre-fix report). A legitimately dash-leading phrase via `--hashlock-phrase-stdin` (`-not a flag`) still works → exit 0. Disabling the guard's `v.starts_with('-')` check made `an_omitted_phrase_value_does_not_swallow_the_next_flag` FAIL with `left: Some(0), right: Some(64)` — exactly the fold's own claim. Reverted. |

| # | finding | verdict | evidence |
| --- | --- | --- | --- |
| M-1 | `ms inspect` prints a preimage with no output-class advisory | **FILED, not fixed (correct per 2026-08-27 ruling)** | `design/FOLLOWUPS.md` entry `ms-inspect-prints-a-preimage-with-no-output-class-advisory` exists with a re-run reproduction and owning phase `0.18.0 release`. Secret-handling class, non-gating. |
| M-2 | terminal echoes the phrase during `--hashlock-phrase-stdin` | **FILED, not fixed (correct per 2026-08-27 ruling)** | `design/FOLLOWUPS.md` entry `hashlock-phrase-stdin-echoes-the-phrase-at-a-terminal` exists with a re-run reproduction and owning phase `0.18.0 release`. Secret-handling class, non-gating. |
| M-3 | three test names/doc comments over-promised their bodies | **FIXED** | (a) `gui_schema_emits_spec_v7_json.rs:254` renamed to `…_the_total_is_67`, matches its `assert_eq!(total, 67)`. (b) `lockstep_100_and_101` now drives both the 100-char (`hash:70a5395386…`, corpus `hardened_h`) and 101-char (exit 1, message names both numbers) rows, cross-checked against `ms_codec::hashlock::preimage_hardened` directly — ran green. (c) `out_is_owner_only` now asserts mode 0600 on all three writer paths (phrase, `--hex`, `--random`); mutating `write_artifact_create_new`'s `opts.mode(0o600)` → `0o644` made the **whole workspace** run `559 tests run: 558 passed, 1 failed` with the single failure being exactly `ms-cli::hashlock_sources out_is_owner_only`. Reverted. |
| M-4 | stale line citation in the new CI step | **FIXED** | `.github/workflows/rust.yml:128` now reads `# The job runs \`cargo test\`, not nextest (the step above, measured) —`; no line-number citation remains to go stale. |
| M-5 | `write_artifact`'s doc comment was orphaned onto the new function | **FIXED** | `crates/ms-cli/src/out.rs`: `write_artifact_create_new` (line 25) and `write_artifact` (line 58) each now carry their own doc comment; no paragraph describes both. |
| M-6 | two tests wrote to fixed paths in the shared `/tmp` | **FIXED** | `grep -rn '"/tmp' crates/ms-cli/tests` returns only `out_flag_private_write.rs` (pre-existing, asserts a file must NOT exist — explicitly out of scope per the fold's own commit message). Neither hashlock test file has a fixed `/tmp` path; both use `tempfile::tempdir()`. |
| N-1 | `--group-size` was `u8` on hashlock, `u16` on encode/split | **FIXED** | `crates/ms-cli/src/cmd/hashlock.rs:79` now declares `pub group_size: u16` (was `u8`), matching `encode.rs:86`. `ms hashlock … --group-size 256` → exit 0 (previously a clap parse error). |
| N-2 | no 76-character `0x03` string exists | **NOTHING TO DO (correctly left alone)** | Reviewer's own arithmetic stands; no change was needed or made. |

## Regression check — `argv_guard.rs`

`git diff a150ba7..447eb09 -- crates/ms-cli/src/argv_guard.rs` is small and
purely additive/narrowing:

- A new `Decision::Usage(String)` variant (exit 64), distinct from `Refuse`.
- `decide()` maps `substitute`'s new `Err` to `Usage` instead of assuming success.
- `substitute`'s signature changed from `Vec<String>` to
  `Result<Vec<String>, String>`; the only new branch is
  `if v != "-" && v.starts_with('-') { return Err(...) }`, reached solely inside
  the pre-existing `SECRET_FLAGS` branch when a value is present and
  flag-shaped. Every other path in `substitute` — the bare `-` sentinel, a
  non-flag-shaped admitted value, a valueless trailing flag left for clap, the
  non-secret-flag pass-through — is textually unchanged.
- One new match arm in an existing unit test (`Decision::Usage(m) => panic!(...)`),
  a defensive assertion that a seed-on-argv must still be `Refuse`, not `Usage`.

No pre-existing decision path changed behavior; the only behavior that changed
is the one the finding names (a flag-shaped swallowed value, previously
silently admitted, now a usage error).

`cargo test -p ms-cli --test argv_guard_cross_product --test allow_argv_secret --test exit_codes_table --locked`
→ 9 + 6 + 6 = 21 passed, 0 failed. Full-workspace `cargo nextest run
--workspace --locked --no-fail-fast` at `447eb09` (unmutated): `559 tests run:
559 passed, 11 skipped`.

## Counts

- Critical: 2 reviewed, **2 FIXED**, 0 not fixed, 0 false claims.
- Important: 3 reviewed, **3 FIXED**, 0 not fixed, 0 false claims.
- Minor: 6 reviewed — 4 FIXED (M-3, M-4, M-5, M-6), 2 correctly FILED not fixed
  (M-1, M-2, per the non-gating secret-handling ruling).
- Nit: 2 reviewed — 1 FIXED (N-1), 1 correctly left alone (N-2).
- Regression: none found in `argv_guard.rs` or the full suite.
- No test added by the fold was found to be unable to fail on its own defect;
  every mutation attempted (C-1, I-3, M-3(c)) killed exactly the test the
  fold's commit claims it kills, with the same failure shape (same assertion,
  same `left`/`right` values where quoted).

## Verdict: GREEN
