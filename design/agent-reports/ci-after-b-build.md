
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

## L2 (docs head b115891080d8b4b2ba11fd32d28d937a81ba9247, previous head e4bf92b1a9ef025c17e29b8df8735022a33170e9)

- run 40232755271 job 121287787080 `release/release test (ubuntu-latest)`: `docs-only: all 1 files changed since the previous PR head e4bf92b1a9ef025c17e29b8df8735022a33170e9 are documentation, the merged base dccd989da9964fbcd0711bd09f99e901e5a98d4f is an ancestor of it, and 'release test (ubuntu-latest)' passed on it; skipping the guarded steps`
- run 40232755271 job 121287787168 `release/build ms (macos-x86_64)`: `docs-only: all 1 files changed since the previous PR head e4bf92b1a9ef025c17e29b8df8735022a33170e9 are documentation, the merged base dccd989da9964fbcd0711bd09f99e901e5a98d4f is an ancestor of it, and 'build ms (macos-x86_64)' passed on it; skipping the guarded steps`
- run 40232755271 job 121287787306 `release/build ms (windows-x86_64)`: `docs-only: all 1 files changed since the previous PR head e4bf92b1a9ef025c17e29b8df8735022a33170e9 are documentation, the merged base dccd989da9964fbcd0711bd09f99e901e5a98d4f is an ancestor of it, and 'build ms (windows-x86_64)' passed on it; skipping the guarded steps`
- run 40232755271 job 121287787311 `release/build ms (macos-aarch64)`: `docs-only: all 1 files changed since the previous PR head e4bf92b1a9ef025c17e29b8df8735022a33170e9 are documentation, the merged base dccd989da9964fbcd0711bd09f99e901e5a98d4f is an ancestor of it, and 'build ms (macos-aarch64)' passed on it; skipping the guarded steps`
- run 40232755266 job 121287786914 `rust/test (release, ubuntu-latest, mlock einval)`: `docs-only: all 1 files changed since the previous PR head e4bf92b1a9ef025c17e29b8df8735022a33170e9 are documentation, the merged base dccd989da9964fbcd0711bd09f99e901e5a98d4f is an ancestor of it, and 'test (release, ubuntu-latest, mlock einval)' passed on it; skipping the guarded steps`
- run 40232755266 job 121287787133 `rust/test (macos-latest)`: `docs-only: all 1 files changed since the previous PR head e4bf92b1a9ef025c17e29b8df8735022a33170e9 are documentation, the merged base dccd989da9964fbcd0711bd09f99e901e5a98d4f is an ancestor of it, and 'test (macos-latest)' passed on it; skipping the guarded steps`
- run 40232755266 job 121287787140 `rust/clippy (ms-codec)`: `docs-only: all 1 files changed since the previous PR head e4bf92b1a9ef025c17e29b8df8735022a33170e9 are documentation, the merged base dccd989da9964fbcd0711bd09f99e901e5a98d4f is an ancestor of it, and 'clippy (ms-codec)' passed on it; skipping the guarded steps`
- run 40232755266 job 121287787146 `rust/test (ms-codec)`: `docs-only: all 1 files changed since the previous PR head e4bf92b1a9ef025c17e29b8df8735022a33170e9 are documentation, the merged base dccd989da9964fbcd0711bd09f99e901e5a98d4f is an ancestor of it, and 'test (ms-codec)' passed on it; skipping the guarded steps`
- run 40232755266 job 121287787147 `rust/fmt (pinned 1.95.0)`: `docs-only: all 1 files changed since the previous PR head e4bf92b1a9ef025c17e29b8df8735022a33170e9 are documentation, the merged base dccd989da9964fbcd0711bd09f99e901e5a98d4f is an ancestor of it, and 'fmt (pinned 1.95.0)' passed on it; skipping the guarded steps`
- run 40232755266 job 121287787148 `rust/miri (mlock unsafe)`: `docs-only: all 1 files changed since the previous PR head e4bf92b1a9ef025c17e29b8df8735022a33170e9 are documentation, the merged base dccd989da9964fbcd0711bd09f99e901e5a98d4f is an ancestor of it, and 'miri (mlock unsafe)' passed on it; skipping the guarded steps`
- run 40232755266 job 121287787189 `rust/history purge (recipes RUN under real shells)`: `docs-only: all 1 files changed since the previous PR head e4bf92b1a9ef025c17e29b8df8735022a33170e9 are documentation, the merged base dccd989da9964fbcd0711bd09f99e901e5a98d4f is an ancestor of it, and 'history purge (recipes RUN under real shells)' passed on it; skipping the guarded steps`
- run 40232755266 job 121287787217 `rust/freebsd compile-gate (whole-crate)`: `docs-only: all 1 files changed since the previous PR head e4bf92b1a9ef025c17e29b8df8735022a33170e9 are documentation, the merged base dccd989da9964fbcd0711bd09f99e901e5a98d4f is an ancestor of it, and 'freebsd compile-gate (whole-crate)' passed on it; skipping the guarded steps`
- run 40232755266 job 121287787229 `rust/test (ubuntu-latest)`: `docs-only: all 1 files changed since the previous PR head e4bf92b1a9ef025c17e29b8df8735022a33170e9 are documentation, the merged base dccd989da9964fbcd0711bd09f99e901e5a98d4f is an ancestor of it, and 'test (ubuntu-latest)' passed on it; skipping the guarded steps`
- run 40232755266 job 121287787245 `rust/musl compile/test (x86_64-unknown-linux-musl)`: `docs-only: all 1 files changed since the previous PR head e4bf92b1a9ef025c17e29b8df8735022a33170e9 are documentation, the merged base dccd989da9964fbcd0711bd09f99e901e5a98d4f is an ancestor of it, and 'musl compile/test (x86_64-unknown-linux-musl)' passed on it; skipping the guarded steps`
- run 40232755266 job 121287787250 `rust/musl compile/test (aarch64-unknown-linux-musl)`: `docs-only: all 1 files changed since the previous PR head e4bf92b1a9ef025c17e29b8df8735022a33170e9 are documentation, the merged base dccd989da9964fbcd0711bd09f99e901e5a98d4f is an ancestor of it, and 'musl compile/test (aarch64-unknown-linux-musl)' passed on it; skipping the guarded steps`
- run 40232755266 job 121287787379 `rust/clippy`: `docs-only: all 1 files changed since the previous PR head e4bf92b1a9ef025c17e29b8df8735022a33170e9 are documentation, the merged base dccd989da9964fbcd0711bd09f99e901e5a98d4f is an ancestor of it, and 'clippy' passed on it; skipping the guarded steps`
