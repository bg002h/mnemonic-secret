# Push report: mnemonic-secret master → fb98d73 via ci/staging

## Tip SHA
`fb98d73e431da8e409a5d08d049bb5682d62895f`

## Preflight (before any push)
- `git status --short`: clean (no output; no untracked `scripts/__pycache__/` present either)
- `git log --oneline origin/master..master`:
  ```
  fb98d73 records: ms-codec 0.8.0 PUBLISHED to crates.io (max_version 0.8.0 at 2026-09-05T03:28:43Z, from the tag's tree, --locked); follow-up: RELEASE_PROCESS.md gets the real publish step
  a4e3b4a decision: fable architect (operator's stand-in) -- publish ms-codec 0.8.0 to crates.io: YES, from the tag's tree, --locked, ms-codec only; verbatim
  503ed93 report: ms push 1990648 via ci/staging -- four contexts success, no bypass; verbatim
  ```
  3 unpushed commits, tip `fb98d73`, as expected.

## Staging push and CI run
- `git push origin master:refs/heads/ci/staging` → new branch `ci/staging` created.
- CI run on this exact SHA: **databaseId 33942824875**, workflow `rust`.
- `gh run watch 33942824875 --repo bg002h/mnemonic-secret` → exited with code 0 (all jobs green).

## Required contexts (via `gh api .../commits/<full-sha>/check-runs`)
```
history purge (recipes RUN under real shells)          success
clippy                                                   success
fmt (pinned 1.95.0)                                      success
test (release, ubuntu-latest, mlock einval)              success
freebsd compile-gate (whole-crate)                       success
g6 invariant (cross-repo mlock.rs)                       success
test (ms-codec)                                          success
test (ubuntu-latest)                                     success
musl compile/test (x86_64-unknown-linux-musl)            success
musl compile/test (aarch64-unknown-linux-musl)            success
miri (mlock unsafe)                                      success
test (macos-latest)                                      success
clippy (ms-codec)                                        success
```
Required four — `test (ubuntu-latest)`, `clippy`, `test (ms-codec)`, `clippy (ms-codec)` — all **SUCCESS**. (Every other context on the SHA was also success; no vendor-freshness job ran, consistent with these commits touching only `design/`.)

## Real push to master
`git push origin master 2>&1 | tee /scratch/code/shibboleth/.tmp/push-ms-fb98d73.log` — output verbatim:
```
To github.com:bg002h/mnemonic-secret.git
   1990648..fb98d73  master -> master
```

### Bypass check
`grep -i "bypass" push-ms-fb98d73.log` → no match (grep exit code 1). No "Bypassed rule violations" message. **Not bypassed** — the push was accepted on the strength of the passing `test (rust + go)`-equivalent contexts already attached to the SHA.

## Post-push verification
- `git push origin --delete ci/staging` → `- [deleted] ci/staging`.
- `git fetch origin && git rev-parse origin/master` → `fb98d73e431da8e409a5d08d049bb5682d62895f` (matches local tip exactly).
- `git ls-remote origin refs/heads/ci/staging` → empty (staging ref absent, as required).

## Verdict
**SUCCESS**
