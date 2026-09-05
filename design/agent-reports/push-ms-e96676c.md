# Push report: mnemonic-secret master → e96676c via ci/staging

## Tip SHA
`e96676c0004bc1a7fea6f5408f53ea4f577f3b18`

## Preflight (before any push)
- `git -C /scratch/code/shibboleth/mnemonic-secret status --short`: clean (no output).
- `git -C /scratch/code/shibboleth/mnemonic-secret rev-parse master`: `e96676c0004bc1a7fea6f5408f53ea4f577f3b18`
- `git -C /scratch/code/shibboleth/mnemonic-secret rev-parse origin/master` (before staging push): `fb98d73e431da8e409a5d08d049bb5682d62895f`
- `git -C /scratch/code/shibboleth/mnemonic-secret log --oneline origin/master..master`:
  ```
  e96676c report: ms push fb98d73 via ci/staging -- four contexts success on run 33942824875, no bypass; verbatim
  ```
  1 unpushed commit, tip `e96676c`, as expected (a push report over `fb98d73`).

## Staging push and CI run
- `git -C /scratch/code/shibboleth/mnemonic-secret push origin master:refs/heads/ci/staging` → new branch `ci/staging` created.
- `gh run list --repo bg002h/mnemonic-secret --commit e96676c0004bc1a7fea6f5408f53ea4f577f3b18 --json databaseId,workflowName,status,conclusion,headSha` → **databaseId 33946142767**, workflow `rust`, headSha matches exactly.
- `gh run watch 33946142767 --repo bg002h/mnemonic-secret --exit-status` → exited with code 0 (all jobs green).

## Required contexts (via `gh api repos/bg002h/mnemonic-secret/commits/<full-sha>/check-runs`)
```
test (macos-latest)                                      success
freebsd compile-gate (whole-crate)                        success
musl compile/test (x86_64-unknown-linux-musl)             success
history purge (recipes RUN under real shells)             success
clippy                                                     success
clippy (ms-codec)                                          success
musl compile/test (aarch64-unknown-linux-musl)             success
g6 invariant (cross-repo mlock.rs)                         success
test (ubuntu-latest)                                       success
miri (mlock unsafe)                                        success
test (release, ubuntu-latest, mlock einval)                success
fmt (pinned 1.95.0)                                        success
test (ms-codec)                                            success
```
Required four -- `test (ubuntu-latest)`, `clippy`, `test (ms-codec)`, `clippy (ms-codec)` -- all **SUCCESS**. Every other context on the SHA was also success. No `vendor-freshness` job ran on this SHA, consistent with this commit touching only `design/`.

## Real push to master
`git -C /scratch/code/shibboleth/mnemonic-secret push origin master 2>&1 | tee /scratch/code/shibboleth/.tmp/push-ms-e96676c.log` -- output verbatim:
```
To github.com:bg002h/mnemonic-secret.git
   fb98d73..e96676c  master -> master
```

### Bypass check
`grep -i "bypass" /scratch/code/shibboleth/.tmp/push-ms-e96676c.log` → no match (grep exit code 1). No "Bypassed rule violations" message. **Not bypassed** -- the push was accepted on the strength of the passing required contexts already attached to the SHA.

## Post-push verification
- `git -C /scratch/code/shibboleth/mnemonic-secret push origin --delete ci/staging` → `- [deleted] ci/staging`.
- `git -C /scratch/code/shibboleth/mnemonic-secret fetch origin && git -C /scratch/code/shibboleth/mnemonic-secret rev-parse origin/master` → `e96676c0004bc1a7fea6f5408f53ea4f577f3b18` (matches local tip exactly).
- `git -C /scratch/code/shibboleth/mnemonic-secret ls-remote origin refs/heads/ci/staging` → empty (staging ref absent, as required).

## Verdict
**SUCCESS**
