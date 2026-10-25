
# Re-run after: ci: guard the release build jobs with ci/docs-only.sh

PR #14. Copies ci-skip-constellation reference/after-b/B-sec (SPEC-after-b.md).

## Change

```
 .github/workflows/release.yml | 30 ++++++++++++++++++++++++------
 1 file changed, 24 insertions(+), 6 deletions(-)
```

## Gate

`make-after-b-reference --check: PASS (12 files)`

## Local checks

```
ci/docs-only.sh: OK
ci/docs-only.test.sh: OK
ci/code-docs-check.sh: OK
ci/code-docs-check.test.sh: OK
docs-only.test.sh: 73 passed, 0 failed
code-docs-check.test.sh: 41 passed, 0 failed
code-docs check: ok (170 files scanned, 134 docs files)
PASS B-sec .github/workflows/release.yml: exactly the §4.2 edits to build
after-b-check B-sec: PASS
```

## L1 (code head e4bf92b1a9ef025c17e29b8df8735022a33170e9)

```
run 40232307748    release                        pull_request   success    ok
run 40232307750    rust                           pull_request   success    ok
  release/build ms (windows-x86_64)                success    ok
  release/release test (ubuntu-latest)             success    ok
  release/build ms (macos-aarch64)                 success    ok
  release/build ms (macos-x86_64)                  success    ok
  release/assemble + release                       skipped    ok (allowed skip)
  rust/clippy                                      success    ok
  rust/freebsd compile-gate (whole-crate)          success    ok
  rust/test (ubuntu-latest)                        success    ok
  rust/history purge (recipes RUN under real shells) success    ok
  rust/clippy (ms-codec)                           success    ok
  rust/g6 invariant (cross-repo mlock.rs)          success    ok
  rust/test (macos-latest)                         success    ok
  rust/test (ms-codec)                             success    ok
  rust/fmt (pinned 1.95.0)                         success    ok
  rust/test (release, ubuntu-latest, mlock einval) success    ok
  rust/miri (mlock unsafe)                         success    ok
  rust/musl compile/test (x86_64-unknown-linux-musl) success    ok
  rust/musl compile/test (aarch64-unknown-linux-musl) success    ok
pr-checks: PASS (mnemonic-secret e4bf92b1a9ef025c17e29b8df8735022a33170e9)
```
