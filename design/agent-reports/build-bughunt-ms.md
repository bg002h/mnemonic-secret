build: ms 7b6169b; tk 451ca5f8; gate PASS; 0 STOP

Implementer: Sonnet 5.5 (this session). Brief: `coordinator/briefs/bughunt-ms-build.md`. Spec: `design/FIX_bughunt_ms.md` (the doc wins over the reference patch; no difference between them was found, so no STOP). Nothing pushed. This report is uncommitted.

Toolchain: `~/.cargo/bin` first on PATH, cargo 1.85.0 (the pin) in both repos. All logs are in `/scratch/cache/bughunt/build/` (`red-nextest.txt`, `ms-gate/`, `tk-gate/{base,edited}/`, `rows-*.txt`, `after-built.txt`).

## Commits

ms, worktree `/scratch/code/shibboleth/ms-worktrees/bughunt-triage`, branch `bughunt/triage`, `f78c403` to `7b6169b` (7 commits):

| step | commit | closes (FOLLOWUPS 9.1 `bughunt-ms-fix`) |
|---|---|---|
| 1 RED rows | `700ce9b` | Build impact (6): every row of FIX section 6 (tightened, new, ignored) |
| 2 code (#1) | `7c1b499` | Build impact (1): `shares.rs` ZERO branch + rustdoc |
| 2 code (#7, #3) | `f92706d` | Build impact (2): `decode.rs` `parse_ms1_symbols` + comment |
| 2 code (#4) | `80a016a` | Build impact (3): `combine.rs`, preimage warning, `emit_ms1` tag |
| 2 code (#5, T2) | `d97f575` | Build impact (4): `inspect.rs` `share-index-secret`, `mnem-unknown-language`, doc comment |
| 4 specs | `4c058ae` | Build impact (5): the four spec replacements (script asserts one occurrence each) |
| 7 FOLLOWUPS | `7b6169b` | Build impact (7): 9.1, 9.2, 9.3, 9.5 appended in that order, plus both coordinator additions (the "Journey input" bullet verbatim after 9.5's "Design direction" bullet; `primer-lowercase-rule-after-subst-hint` last in 9.2, body from review r3 M-1). Both diffed against the brief: identical. |

Step 6 (`cargo fmt`): nothing to change; `cargo fmt --all --check` and `cargo +1.95.0 fmt --all -- --check` both exit 0. FOLLOWUPS statuses are as the FIX gives them (`bughunt-ms-fix` stays `open`; the coordinator closes it).

Production and test code is reference patch r3 (sha256 `26c20263…` verified), split into per-fix commits; the patch had been checked against FIX sections 3 and 6 first.

tk, worktree `/scratch/code/shibboleth/tk-worktrees/bughunt-ms-manual`, branch `docs/bughunt-ms-manual`, `c2b09861` to `451ca5f8` (2 commits, local only):
- `e0877cb3`: ran `toolkit-manual-edits.py` (sha256 `e56d598e…` checked first): 10 edits applied (43-ms.md x3, 65-bch-codex-primer.md x7).
- `451ca5f8`: both 9.4 entries (`toolkit-ms-codec-bump-bughunt-ms-fixes`, `toolkit-repair-indel-search-rejects-capitals`) appended to `design/FOLLOWUPS.md`. Closes Build impact (8) and (9); no toolkit code or test change.

Versions unchanged.

## RED rows (step 1)

On unmodified production code, `-E 'binary(bughunt) | binary(bughunt_t)'`: 22 run, 10 passed, 12 failed, 12 skipped. The 12 failures are codec 01, 03, 07; CLI 04, 04b, 04c, 04d, 05, 05b, 07, 07b; CLI t2. Each failed for the reason FIX section 6 gives under "RED today" (checked against the assertion output in `red-nextest.txt`). The 10 passes: 8 mlock unit tests, `bughunt_ms_10`, the 04e control.

## Gate, ms (every rust.yml job run locally; one run each, logs in `ms-gate/`)

| job / step | result |
|---|---|
| fmt: `cargo +1.95.0 fmt --all -- --check` and 1.85 `cargo fmt --all --check` | exit 0, exit 0 |
| test-ms-codec: preflight (openssl kdf, python3), `nextest -p ms-codec` | 210 run, 210 passed, 5 skipped |
| `cargo test -p ms-codec --doc` | 1 passed |
| hashlock repro by name (`hashlock_repro_three_ways`) | `test result: ok. 1 passed` |
| clippy-ms-codec (`-p ms-codec --all-targets -D warnings`) | exit 0 |
| test (ubuntu): `nextest -p ms-cli --no-fail-fast` | 458 run, 458 passed, 18 skipped |
| G2.1 eperm / G2.3 einval / G2.4 off (`--include-ignored`) | 1 passed each |
| test-release-mlock-einval (`--release`, g2_3) | 1 passed |
| miri (`cargo +nightly miri test -p ms-cli --bin ms mlock::`) | 8 passed, 3 ignored |
| clippy job: `docs-only.test.sh` 73 passed; shared-ci sha256; `code-docs-check.test.sh` 41 passed; `code-docs-check.sh` ok; display-grouping vector pin OK; `clippy --all-targets -p ms-cli -D warnings` | all exit 0 |
| history-purge `--include-ignored` / the_decline `--include-ignored` | 7 passed / 6 passed |
| g6-invariant (`SIBLING_REPO_PATH` = local toolkit checkout) | 2 passed |
| freebsd compile-gate (`cargo check --target x86_64-unknown-freebsd -p ms-cli`) | exit 0 |
| whole workspace: `cargo nextest run --locked --workspace` | 668 run, 668 passed, 23 skipped (equals the FIX's count) |
| `cargo test --locked --doc --workspace` | 1 passed |
| `cargo clippy --locked --all-targets --workspace -- -D warnings` | exit 0 |
| `bash ci/repro/vendor-freshness.sh` (vendor-freshness.yml) | OK |
| fuzz-smoke.yml `build` job (`cargo +nightly-2026-04-27 fuzz build`) | exit 0 (2 pre-existing lifetime warnings in vendored `codex32/mod.rs`; I reverted the `fuzz/Cargo.lock` rewrite the build makes) |

Section 6 rows by exact name (counts equal the FIX's):
- `binary(bughunt) & test(/^bughunt_ms_/)`: 13 run, 13 passed, 17 skipped.
- Same filter with `--run-ignored ignored-only`: 6 run, 6 failed (02, 06, 08 codec, 08 CLI, 09, x1: the deferred rows stay red).
- `binary(bughunt)`: 21 run, 21 passed, 9 skipped.
- `binary(bughunt_t)`: 1 run, 1 passed, 3 skipped. With ignored-only: 3 run, 3 failed (t1, t1b, t3). t1 and t1b now fail with `refused: codex32 parse error: InvalidChar('?')`, as the FIX states.
- Unreachable census: `unreachable_catch_all_count_is_pinned` passes (4).

FIX section 2 byte for byte: `check_after.py` copy pointed at the built `target/debug/ms`: 4 blocks, 15 commands, 0 diffs (`after-built.txt`). Negative control, same helper on the installed unfixed `ms`: 14 of 15 diff (the 15th is a tautology of the helper), so the check can fail. Also confirmed: `ms combine --in` and `--to phrase` print the same bytes as each other and as the unfixed build, text and `--json`; `ms combine --to ms1` equals `ms hashlock --kind sha256 --hex - --out` byte for byte; `ms hashlock --in` reads it (exit 0, `hash:02d449…`); `ms decode` prints the preimage record.

## Gate, toolkit (the `manual` job's steps; "base" = unedited `c2b09861`, "edited" = the working tree holding all edits, run before the two commits; its content equals `451ca5f8`)

| step | base | edited |
|---|---|---|
| `ci/doc-gate-guard.test.sh` | rc 0 | rc 0 |
| `ci/docs-only.test.sh`; shared-ci sha256 + `code-docs-check.test.sh` + `code-docs-check.sh` | rc 0 | rc 0 |
| `make audit` (CI's flags) | rc 2: markdownlint 0, cspell 0 issues, lychee 0 errors; flag-coverage 10 FAILs, all stale markers in `44-mk-cli.md` | rc 2: the same 10 FAIL lines (diffed, identical); none in `43-ms.md` or `65-bch-codex-primer.md` |
| `ci/doc-flag-lint.test.sh` (mutation self-test) | rc 1: 12 passed, 3 failed | rc 1: 12 passed, 3 failed; the same three cases (baseline, R4 reverse, G0), all caused by the baseline FAILs |
| `make verify-examples` (not reached by `make audit` once lint fails, so run separately) | rc 2: 9 `mk` transcript drifts | rc 2: the same 9 |
| `make anchor-check` | OK, 9 danglers match baseline | OK, 9 danglers match baseline |
| `make pdf MERMAID_FILTER=skip` / `make html` | rc 0 / rc 0 | rc 0 / rc 0 (PDF 545 pages) |

No new failure from the edits.

## Could not do, or not exactly as CI runs it

- musl native job: `musl-gcc` is not installed here (`x86_64-linux-musl-gcc` not found, nextest exits 101 as CI would on a bare box). Approximation: `CC_x86_64_unknown_linux_musl=gcc cargo nextest run -p ms-cli --target x86_64-unknown-linux-musl --no-fail-fast`: 458 run, 458 passed. Not run: the aarch64-musl and macOS matrix legs, and `release.yml` (its pull_request cross-build matrix), which this box cannot run. The FIX's gate paragraph does not list them.
- The toolkit baseline already fails locally for `mk`-related reasons (10 lint FAILs, 9 transcript drifts, 3 mutation-test cases). They are unchanged by the edits; I did not touch `44-mk-cli.md`.
- The mutation gate (`coordinator/scripts/mutation-gate.sh`) and the whole-diff review are the coordinator's next step, not part of this brief.

## STOPs

None. Every holder-visible point (texts, refusals, exit codes, test rows, FOLLOWUPS entries) was decided by the FIX doc and the brief.

## Files

- ms report: `/scratch/code/shibboleth/ms-worktrees/bughunt-triage/design/agent-reports/build-bughunt-ms.md` (uncommitted)
- Gate scripts (reusable): `/scratch/cache/bughunt/build/ms-gate.sh`, `tk-gate.sh`, `tk-gate-extra.sh`, `check_after_built.py`, `apply_specs.py`, `append_followups.py`, `append_tk_followups.py`
