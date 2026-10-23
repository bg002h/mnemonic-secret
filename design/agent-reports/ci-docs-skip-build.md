
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
