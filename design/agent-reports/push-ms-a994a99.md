# Push report: mnemonic-secret master → a994a99 via ci/staging

## Tip SHA
`a994a998e059a31f656a55616d12e15f02686126`

## Preflight (before any push)
- `git -C /scratch/code/shibboleth/mnemonic-secret status --short`: clean (no output).
- `git -C /scratch/code/shibboleth/mnemonic-secret rev-parse master`: `a994a998e059a31f656a55616d12e15f02686126`
- `git -C /scratch/code/shibboleth/mnemonic-secret rev-parse origin/master` (before staging push): `990df82e2ec40c00d6fd91f0d2ae36351557861b`
- `git -C /scratch/code/shibboleth/mnemonic-secret log --oneline origin/master..master`:
  ```
  a994a99 records: the engraving card no longer says the method line is on no plate -- H6 cuts it on the HASHLOCK PHRASE plate (H6 Task 13 record 4; engrave F-501)
  4755061 report: ms-codec 0.9.0 release -- ci/staging push 990df82 four contexts success, tag pushed, crates.io max_version 0.9.0; verbatim
  ```
  2 unpushed commits, tip `a994a99`, matching the brief exactly (the ms-codec 0.9.0 release report plus the one-line hashlock.rs wording fix).

## Staging push and CI run
- `git -C /scratch/code/shibboleth/mnemonic-secret push origin master:refs/heads/ci/staging` → new branch `ci/staging` created.
- `gh run list --repo bg002h/mnemonic-secret --commit a994a998e059a31f656a55616d12e15f02686126 --json databaseId,workflowName,status,conclusion,headSha` → **databaseId 34033401445**, workflow `rust`, headSha matches exactly.
- `gh run watch 34033401445 --repo bg002h/mnemonic-secret --exit-status` → exited with code 0 (all jobs green).

## Required contexts (via `gh api repos/bg002h/mnemonic-secret/commits/<full-sha>/check-runs`)
```
clippy (ms-codec)                                          success
clippy                                                     success
fmt (pinned 1.95.0)                                        success
freebsd compile-gate (whole-crate)                         success
g6 invariant (cross-repo mlock.rs)                         success
history purge (recipes RUN under real shells)              success
miri (mlock unsafe)                                        success
musl compile/test (aarch64-unknown-linux-musl)             success
musl compile/test (x86_64-unknown-linux-musl)              success
test (macos-latest)                                        success
test (ms-codec)                                            success
test (release, ubuntu-latest, mlock einval)                success
test (ubuntu-latest)                                       success
```
Required four -- `test (ubuntu-latest)`, `clippy`, `test (ms-codec)`, `clippy (ms-codec)` -- all **SUCCESS**. Every other context on the SHA was also success (13 total). No `vendor-freshness` job ran on this SHA, consistent with this commit touching only `design/` and a wording fix in `crates/ms-cli/src/cmd/hashlock.rs` (no `Cargo.lock` change).

## Real push to master
`git -C /scratch/code/shibboleth/mnemonic-secret push origin master 2>&1 | tee /scratch/code/shibboleth/.tmp/push-ms-a994a99.log` -- output verbatim:
```
To github.com:bg002h/mnemonic-secret.git
   990df82..a994a99  master -> master
```

### Bypass check
`grep -i "bypass" /scratch/code/shibboleth/.tmp/push-ms-a994a99.log` → no match (grep exit code 1). No "Bypassed rule violations" message. **Not bypassed** -- the push was accepted on the strength of the passing required contexts already attached to the SHA.

## Post-push verification
- `git -C /scratch/code/shibboleth/mnemonic-secret push origin --delete ci/staging` → `- [deleted] ci/staging`.
- `git -C /scratch/code/shibboleth/mnemonic-secret fetch origin && git -C /scratch/code/shibboleth/mnemonic-secret rev-parse origin/master` → `a994a998e059a31f656a55616d12e15f02686126` (matches local tip exactly).
- `git -C /scratch/code/shibboleth/mnemonic-secret ls-remote origin refs/heads/ci/staging` → empty (staging ref absent, as required).

## Verdict
**SUCCESS**
