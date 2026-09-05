# Push report — mnemonic-secret master via ci/staging — 8796d69

## Tip and commits pushed

- Tip SHA: `8796d69860248dabf5319dde65e6d96800939aee`
- Previous `origin/master`: `1e3d6df`
- Commits pushed (2, `origin/master..master` at start):
  - `8796d69` vendor: re-vendor for ms-codec 0.8.0's pbkdf2/hmac/sha2 (vendor-freshness gate)
  - `351a75e` report: ms push 1e3d6df via ci/staging -- four required contexts success, no bypass; vendor-freshness (non-required) FAILED, investigated next; verbatim
- Working tree was clean at start (no relevant untracked files beyond ignorable `scripts/__pycache__/`).

## Staging push and CI runs

`git push origin master:refs/heads/ci/staging` created branch `ci/staging` at `8796d69`.

Runs observed for commit `8796d69860248dabf5319dde65e6d96800939aee`:

- Run **33933154527** — workflow `rust` — status: completed, conclusion: **success**
- Run **33933154522** — workflow `vendor-freshness` — status: completed, conclusion: **success**

(No `fuzz-smoke` run was triggered for this push; only `rust` and `vendor-freshness` fired.)

## Check-run conclusions on the SHA

Full `check-runs` listing for `8796d69860248dabf5319dde65e6d96800939aee`:

| Check | Conclusion |
| --- | --- |
| g6 invariant (cross-repo mlock.rs) | success |
| musl compile/test (x86_64-unknown-linux-musl) | success |
| **test (ubuntu-latest)** | **success** |
| **clippy (ms-codec)** | **success** |
| miri (mlock unsafe) | success |
| test (macos-latest) | success |
| fmt (pinned 1.95.0) | success |
| musl compile/test (aarch64-unknown-linux-musl) | success |
| freebsd compile-gate (whole-crate) | success |
| history purge (recipes RUN under real shells) | success |
| **vendor/ satisfies Cargo.lock (offline)** | **success** |
| **test (ms-codec)** | **success** |
| test (release, ubuntu-latest, mlock einval) | success |
| **clippy** | **success** |

All four required contexts (`test (ubuntu-latest)`, `clippy`, `test (ms-codec)`, `clippy (ms-codec)`) are SUCCESS, and the previously-failing non-required `vendor/ satisfies Cargo.lock (offline)` is now SUCCESS as well — the vendor-freshness fix in `8796d69` is confirmed green on this SHA.

## Final push to master

Verbatim captured output (`/scratch/code/shibboleth/.tmp/push-ms-8796d69.log`):

```
To github.com:bg002h/mnemonic-secret.git
   1e3d6df..8796d69  master -> master
```

"Bypassed rule violations" did **NOT** appear in the output.

## Post-push verification

- `git fetch origin && git rev-parse origin/master` → `8796d69860248dabf5319dde65e6d96800939aee`
- Local `git rev-parse master` → `8796d69860248dabf5319dde65e6d96800939aee` (match)
- `git push origin --delete ci/staging` → `- [deleted]         ci/staging`
- `git ls-remote origin refs/heads/ci/staging` → empty output (ref gone)

## Verdict

**SUCCESS**
