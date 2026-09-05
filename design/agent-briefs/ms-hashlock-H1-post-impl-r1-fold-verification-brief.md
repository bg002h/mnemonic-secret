You are the INDEPENDENT fold-verification reviewer (sonnet tier, targeted) for round 1 of the H1 post-implementation review in mnemonic-secret. Round 0: `design/agent-reports/ms-hashlock-H1-post-impl-review.md` (opus, committed `b776253` on ms master): 2C/3I/6M/2N on branch `hashlock-h1` at `a150ba7`. The fold: nine commits `a150ba7..447eb09` on the same branch (worktree `/scratch/code/shibboleth/ms-worktrees/hashlock-h1`), by the same implementer; the "Post-review fold" section of `design/agent-reports/ms-hashlock-H1-implementation-report.md` on the branch maps each finding to its commit and its proof.

ONE QUESTION: did the fold fix every Critical and Important of the review — FIXED / PARTIAL / NOT FIXED / DECLINED-with-reason, one line each — with a test that can FAIL for each, and without a regression or a false claim of its own?

Read-only on master and on the branch; commit nothing; no sub-agents; never read any `.jsonl`. Build and run in your OWN detached worktree with its OWN target dir: `git -C /scratch/code/shibboleth/mnemonic-secret worktree add --detach /scratch/code/shibboleth/ms-worktrees/h1-verify 447eb09`; every cargo command with `PATH=$HOME/.cargo/bin:$PATH TMPDIR=/scratch/code/shibboleth/.tmp CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/h1-verify-target`. Mutate only there, revert after each mutation (`touch` a file you restore from a backup), remove the worktree and the target dir when done.

## Already settled — do not re-derive
- The controller re-ran CI's commands at `447eb09` in an isolated target dir: nextest 559/559, `cargo test` 560 passed / 0 failed, clippy `-D warnings` exit 0, `cargo fmt --check` exit 0 on both the repo toolchain and `+1.95.0`. Do not report compile/lint/fmt findings.
- The four controller defaults and Task 11 were out of the fold's scope; do not review them.
- Secret-handling findings never gate (operator ruling): M-1 and M-2 were filed as follow-ups, not fixed — confirm the entries exist with a reproduction and an owning phase, then move on.

## Verify (execute; quote output)
1. **C-1** (`--json` advisory test could not fail): find the new assertion; run the test; then delete the `PrivateKeyMaterial` advisory emission in the code and run again — it must FAIL, and the failure must be the advisory, not the card. Confirm the invocation passes `--no-engraving-card`.
2. **C-2** (false report count): re-measure `cargo nextest run --workspace --locked --no-fail-fast` and `cargo test --workspace --locked` at `447eb09`; every count in the report's "Post-review fold" and "Final gate" sections must match what you measure; the false sentence about `--bin ms` unit tests must be gone.
3. **I-1** (terminal read ends at newline): read the new terminal branch; with a pty (python `pty.fork` as the review did) type a phrase and Enter — `ms hashlock --hashlock-phrase-stdin` must return with a `hash:` line; from a PIPE, `printf 'a\nb'` must still be refused by the phrase rule (a `\n` inside the phrase), proving the pipe path stayed `read_to_end`. The prompt text must still be the spec's verbatim `Type the hashlock phrase, then Enter.`
4. **I-2** (`--separator`): `ms hashlock --separator ab --hashlock-phrase-stdin <<< x` must be refused with the SAME message `ms encode --separator ab` gives; quote both.
5. **I-3** (flag swallowed as a secret): run the review's exact counterexample `ms hashlock --allow-argv-secret --hashlock-phrase --json --no-engraving-card < /dev/null` — exit 64, no `hash:` line, the message names `--hashlock-phrase`; then mutate the guard's new check away and confirm the test the fold added FAILS. Confirm `--hashlock-phrase=--json` (equals form) and a legitimately dash-leading phrase under `--hashlock-phrase-stdin` (`-not a flag`) behave as the spec's phrase rule says.
6. **Minors**: one line each — M-3 (names match bodies; the `out_is_owner_only` extension: mutate `mode(0o600)` → `0o644` in `write_artifact_create_new` and confirm exactly that test fails), M-4, M-5 (the hand-wire script and the tree agree), M-6 (no fixed `/tmp` path remains: `grep -rn '"/tmp' crates/ms-cli/tests`), N-1.
7. **Regressions**: the fold touched `argv_guard.rs`'s `substitute` and added `Decision::Usage`; run the full argv-guard test binary and the exit-codes table test; diff `a150ba7..447eb09 -- crates/ms-cli/src/argv_guard.rs` and say whether any pre-existing decision path changed.

## Severity
A C/I marked FIXED but not fixed = Critical. A test the fold added that cannot fail on its defect = Critical. A regression or a count that does not reproduce = Important. Wording = Minor.

## Report (your final action)
Write `/scratch/code/shibboleth/mnemonic-secret/design/agent-reports/ms-hashlock-H1-post-impl-r1-fold-verification.md` (in the MAIN checkout at that absolute path; create; must not exist): the finding table (5 C/I rows, then the 8 M/N), the executed checks with output, the regression check, closing counts and GREEN / NOT GREEN. Return a two-line summary plus the path.
