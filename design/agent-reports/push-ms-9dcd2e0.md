# Push report: mnemonic-secret master → 9dcd2e0 via ci/staging

## Tip SHA
`9dcd2e0dc0fef57ef94274359181a9f706a10fc6`

## Preflight (before any push)
- `PATH=/home/bcg/.cargo/bin:$PATH cargo --version`: `cargo 1.85.0 (d73d2caf9 2024-12-31)` (confirmed the pinned toolchain, not system 1.98.0).
- `git status --porcelain`: empty (clean tree).
- `git rev-parse master`: `9dcd2e0dc0fef57ef94274359181a9f706a10fc6`.
- `git rev-parse origin/master` (after `git fetch origin`): `c48f4bd593f80e411950dabf9d02fd1aaf1716e3`.
- `git log --oneline c48f4bd593f80e411950dabf9d02fd1aaf1716e3..9dcd2e0dc0fef57ef94274359181a9f706a10fc6` (the 7 unpushed commits):
  ```
  9dcd2e0 release: ms-codec 0.10.0 / ms-cli 0.19.0, with the migration record
  a7b4e16 ms hashlock: --kind, with a four-digest lookup when it is absent
  7cdcd71 hashlock: qr_text names the kind on its own line
  f2a8ecd hashlock: the per-kind KAT, functions and dispatch, mutation-verified
  f837343 hashlock: KAT rows for the three new kinds, computed in python3
  49cf439 hashlock: four digest functions, the named dispatch, and the rename collateral
  7a0e96f report: engrave push c48f4bd5 via ci/staging -- test (rust) success on run 34047250997, no bypass; verbatim
  ```
  Matches the brief: phase 2 of the hashlock-kinds cycle (ms-codec 0.10.0 / ms-cli 0.19.0) plus one earlier report commit, 7 commits total.

## Controller-measured local validation (before staging push)
- `./ci/repro/vendor-freshness.sh` → `vendor-freshness: OK -- vendor/ satisfies Cargo.lock.` (exit 0). This release adds the vendored `ripemd` dependency and moves `Cargo.lock`; this check is not a required context, so confirming it locally covers what the ritual would otherwise miss until tag time.
- `cargo nextest run --locked --all-targets` → **584 tests run, 584 passed, 0 failed, 11 skipped** (`Summary [0.263s] 584 tests run: 584 passed, 11 skipped`), matching the brief's expectation exactly. `grep -c FAIL` on the captured log: 0.

## Staging push and CI run
- Reconfirmed `git rev-parse master` = `9dcd2e0...` immediately before staging (master had not moved).
- `git push origin master:refs/heads/ci/staging` → `* [new branch]      master -> ci/staging`.
- `gh run list --repo bg002h/mnemonic-secret --commit 9dcd2e0dc0fef57ef94274359181a9f706a10fc6` (full SHA) → three runs created immediately: workflow `rust` (databaseId **34988663424**), `fuzz-smoke` (34988663410), `vendor-freshness` (34988663370), all `queued`/`in_progress` at first check.
- `gh run watch 34988663424 --repo bg002h/mnemonic-secret --exit-status` → exited 0 (all jobs green). Run URL: https://github.com/bg002h/mnemonic-secret/actions/runs/34988663424

## Per-job conclusions (`gh run view 34988663424 --json jobs`)
```
test (release, ubuntu-latest, mlock einval)   success
freebsd compile-gate (whole-crate)            success
miri (mlock unsafe)                           success
clippy                                        success
test (macos-latest)                           success
test (ubuntu-latest)                          success
test (ms-codec)                               success
fmt (pinned 1.95.0)                           success
musl compile/test (x86_64-unknown-linux-musl) success
history purge (recipes RUN under real shells) success
musl compile/test (aarch64-unknown-linux-musl) success
g6 invariant (cross-repo mlock.rs)            success
clippy (ms-codec)                             success
```
Required four -- `test (ubuntu-latest)`, `clippy`, `test (ms-codec)`, `clippy (ms-codec)` -- all **SUCCESS**. All 13 jobs on the `rust` workflow run succeeded.

Sibling runs on the same SHA also confirmed via `gh run view --json conclusion,status`:
- `fuzz-smoke` (34988663410): `completed / success`.
- `vendor-freshness` (34988663370): `completed / success` -- matches the local `ci/repro/vendor-freshness.sh` OK result above.

## Real push to master
`git push origin master` -- output verbatim:
```
To github.com:bg002h/mnemonic-secret.git
   c48f4bd..9dcd2e0  master -> master
```

### Bypass check
No "Bypassed rule violations" line in the push output (checked directly against the captured stdout/stderr above). **Not bypassed** -- the push was accepted on the strength of the passing required contexts already attached to the SHA via `ci/staging`.

## Post-push verification
- `git push origin --delete ci/staging` → `- [deleted]         ci/staging`.
- `git fetch origin && git rev-parse origin/master` → `9dcd2e0dc0fef57ef94274359181a9f706a10fc6` (matches local master tip exactly).
- `git status --short --branch` (post-push) → `## master...origin/master` (up to date, no ahead/behind).

## Verdict
**SUCCESS -- CLEAN** (no bypass).
