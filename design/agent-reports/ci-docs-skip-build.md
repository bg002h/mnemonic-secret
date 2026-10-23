
# Re-run after: ci: skip a docs-only push to a PR that already passed (rule A')

PR #13. Applies ci-skip-constellation SPEC.md (GREEN at e8f831d), Part A.

## Change

```
 .github/workflows/release.yml         |  13 ++-
 .github/workflows/rust.yml            |   6 ++
 ci/code-docs-check.sh                 | 165 ++++++++++++++++++++++++++++++++++
 ci/code-docs-check.test.sh            | 128 ++++++++++++++++++++++++++
 ci/code-docs-exempt.txt               |   8 ++
 ci/docs-only-paths.txt                |   2 +
 ci/docs-only.sh                       | 158 +++++++++++++++++++++++---------
 ci/docs-only.test.sh                  | 103 +++++++++++++++++++--
 ci/shared-ci.sha256                   |   4 +
 crates/ms-codec/tests/parity_smoke.rs |   2 +-
 10 files changed, 536 insertions(+), 53 deletions(-)
```

## Gate

`gate-public: 0 failure(s)`

## Local checks

```
ci/docs-only.sh: OK
ci/docs-only.test.sh: OK
ci/code-docs-check.sh: OK
ci/code-docs-check.test.sh: OK
docs-only.test.sh: 73 passed, 0 failed
code-docs-check.test.sh: 41 passed, 0 failed
code-docs check: ok (170 files scanned, 132 docs files)
```

## L1 (code head f0a0676bc165f2225bc31134ff80999dc36950f7)

```
run 40038675448    release                        pull_request   success    ok
run 40038675437    rust                           pull_request   success    ok
  release/build ms (windows-x86_64)                success    ok
  release/build ms (macos-x86_64)                  success    ok
  release/build ms (macos-aarch64)                 success    ok
  release/release test (ubuntu-latest)             success    ok
  release/assemble + release                       skipped    ok (allowed skip)
  rust/test (release, ubuntu-latest, mlock einval) success    ok
  rust/test (ubuntu-latest)                        success    ok
  rust/test (macos-latest)                         success    ok
  rust/miri (mlock unsafe)                         success    ok
  rust/fmt (pinned 1.95.0)                         success    ok
  rust/clippy (ms-codec)                           success    ok
  rust/g6 invariant (cross-repo mlock.rs)          success    ok
  rust/test (ms-codec)                             success    ok
  rust/freebsd compile-gate (whole-crate)          success    ok
  rust/history purge (recipes RUN under real shells) success    ok
  rust/musl compile/test (x86_64-unknown-linux-musl) success    ok
  rust/clippy                                      success    ok
  rust/musl compile/test (aarch64-unknown-linux-musl) success    ok
pr-checks: PASS (mnemonic-secret f0a0676bc165f2225bc31134ff80999dc36950f7)
```

## L2 (docs head ce185f8715ea0cf3a3a215be1b05d8ba026ee310, previous head f0a0676bc165f2225bc31134ff80999dc36950f7)

- run 40039230748 job 120706111584 `release/release test (ubuntu-latest)`: `docs-only: all 1 files changed since the previous PR head f0a0676bc165f2225bc31134ff80999dc36950f7 are documentation, the merged base b728f160db1bd84af99e6338106d303c75b8a2d6 is an ancestor of it, and 'release test (ubuntu-latest)' passed on it; skipping the guarded steps`
- run 40039230633 job 120706111197 `rust/test (ms-codec)`: `docs-only: all 1 files changed since the previous PR head f0a0676bc165f2225bc31134ff80999dc36950f7 are documentation, the merged base b728f160db1bd84af99e6338106d303c75b8a2d6 is an ancestor of it, and 'test (ms-codec)' passed on it; skipping the guarded steps`
- run 40039230633 job 120706111381 `rust/test (ubuntu-latest)`: `docs-only: all 1 files changed since the previous PR head f0a0676bc165f2225bc31134ff80999dc36950f7 are documentation, the merged base b728f160db1bd84af99e6338106d303c75b8a2d6 is an ancestor of it, and 'test (ubuntu-latest)' passed on it; skipping the guarded steps`
- run 40039230633 job 120706111395 `rust/clippy (ms-codec)`: `docs-only: all 1 files changed since the previous PR head f0a0676bc165f2225bc31134ff80999dc36950f7 are documentation, the merged base b728f160db1bd84af99e6338106d303c75b8a2d6 is an ancestor of it, and 'clippy (ms-codec)' passed on it; skipping the guarded steps`
- run 40039230633 job 120706111472 `rust/clippy`: `docs-only: all 1 files changed since the previous PR head f0a0676bc165f2225bc31134ff80999dc36950f7 are documentation, the merged base b728f160db1bd84af99e6338106d303c75b8a2d6 is an ancestor of it, and 'clippy' passed on it; skipping the guarded steps`
- run 40039230633 job 120706111480 `rust/test (release, ubuntu-latest, mlock einval)`: `docs-only: all 1 files changed since the previous PR head f0a0676bc165f2225bc31134ff80999dc36950f7 are documentation, the merged base b728f160db1bd84af99e6338106d303c75b8a2d6 is an ancestor of it, and 'test (release, ubuntu-latest, mlock einval)' passed on it; skipping the guarded steps`
- run 40039230633 job 120706111481 `rust/miri (mlock unsafe)`: `docs-only: all 1 files changed since the previous PR head f0a0676bc165f2225bc31134ff80999dc36950f7 are documentation, the merged base b728f160db1bd84af99e6338106d303c75b8a2d6 is an ancestor of it, and 'miri (mlock unsafe)' passed on it; skipping the guarded steps`
- run 40039230633 job 120706111489 `rust/musl compile/test (aarch64-unknown-linux-musl)`: `docs-only: all 1 files changed since the previous PR head f0a0676bc165f2225bc31134ff80999dc36950f7 are documentation, the merged base b728f160db1bd84af99e6338106d303c75b8a2d6 is an ancestor of it, and 'musl compile/test (aarch64-unknown-linux-musl)' passed on it; skipping the guarded steps`
- run 40039230633 job 120706111509 `rust/musl compile/test (x86_64-unknown-linux-musl)`: `docs-only: all 1 files changed since the previous PR head f0a0676bc165f2225bc31134ff80999dc36950f7 are documentation, the merged base b728f160db1bd84af99e6338106d303c75b8a2d6 is an ancestor of it, and 'musl compile/test (x86_64-unknown-linux-musl)' passed on it; skipping the guarded steps`
- run 40039230633 job 120706111514 `rust/test (macos-latest)`: `docs-only: all 1 files changed since the previous PR head f0a0676bc165f2225bc31134ff80999dc36950f7 are documentation, the merged base b728f160db1bd84af99e6338106d303c75b8a2d6 is an ancestor of it, and 'test (macos-latest)' passed on it; skipping the guarded steps`
- run 40039230633 job 120706111578 `rust/history purge (recipes RUN under real shells)`: `docs-only: all 1 files changed since the previous PR head f0a0676bc165f2225bc31134ff80999dc36950f7 are documentation, the merged base b728f160db1bd84af99e6338106d303c75b8a2d6 is an ancestor of it, and 'history purge (recipes RUN under real shells)' passed on it; skipping the guarded steps`
- run 40039230633 job 120706111595 `rust/freebsd compile-gate (whole-crate)`: `docs-only: all 1 files changed since the previous PR head f0a0676bc165f2225bc31134ff80999dc36950f7 are documentation, the merged base b728f160db1bd84af99e6338106d303c75b8a2d6 is an ancestor of it, and 'freebsd compile-gate (whole-crate)' passed on it; skipping the guarded steps`
- run 40039230633 job 120706112060 `rust/fmt (pinned 1.95.0)`: `docs-only: all 1 files changed since the previous PR head f0a0676bc165f2225bc31134ff80999dc36950f7 are documentation, the merged base b728f160db1bd84af99e6338106d303c75b8a2d6 is an ancestor of it, and 'fmt (pinned 1.95.0)' passed on it; skipping the guarded steps`
