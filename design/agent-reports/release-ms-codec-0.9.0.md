# Release report: ms-codec 0.9.0

Sonnet-tier release agent, 2026-09-06, following
`/scratch/code/shibboleth/mnemonic-engrave/design/agent-briefs/hashlock-H6-release-ms-codec-0.9.0-brief.md`
(and its two addenda) and `design/RELEASE_PROCESS.md`, per the H1/0.8.0
precedent (`design/agent-reports/decision-crates-io-publish-ms-codec-0.8.0.md`,
`design/agent-reports/push-ms-e96676c.md`). No `.jsonl` file was opened.

## Starting state

- `git -C /scratch/code/shibboleth/mnemonic-secret status --short` → clean.
- `git -C /scratch/code/shibboleth/mnemonic-secret rev-parse HEAD` →
  `1a4f4aa8a3b4b29e5e6f7a479b4c57bc0e073fea` (`h6-a` fast-forwarded, matches
  the brief; the pre-publish opus review is GREEN 0C/0I, engrave
  `design/agent-reports/hashlock-H6-A-pre-publish-review.md`).
- CHANGELOG's `## ms-codec [0.9.0] — 2026-09-05` heading and body were already
  in place (addendum 2) — left unchanged, per instruction.

## Records commit (ADDENDUM item 3 / item 9 / N-3), BEFORE the push

Two files edited, one commit, on `master`:

- `MIGRATION.md` — new `## v0.8 → v0.9` section documenting the five public
  additions (`validate_phrase`, `PhraseRefusal`, `looks_like_ms1`, `qr_text`,
  `HASHLOCK_PHRASE_MAX_CHARS`) per the v0.1 → v0.2 precedent's structure and
  detail level; states the corpus SHA move
  (`a46c197a...` → `4f1819cdd0...`), the internal hex-check convergence, and
  the byte-identity / no-breaking-change claim.
- `design/FOLLOWUPS.md` — two new entries under `## Open items`:
  - `ms-codec-0-9-0-phrase-rule-two-consumers` (tier `cross-repo`): names the
    two waiting consumers — `mnemonic-engrave`'s Task 2 pin bump
    (`crates/me-cli/Cargo.toml:53`, `=0.8.0` → `=0.9.0`) and the SeedHammer
    fork's Task 5b re-vendor + `hashlock/testdata/hashlock-v0.8.provenance.json`
    re-pin to the new corpus SHA. This repo's FOLLOWUPS.md uses slug ids
    throughout (verified: `grep -c "^### " design/FOLLOWUPS.md` → 68 entries,
    zero `### F-` headers), so the entry is filed as a slug, not an F-number —
    the brief's "next free F-number" language does not match this file's own
    convention, which the entry follows instead.
  - `qr-text-no-realloc-property-unguarded` (tier `v1+`): the pre-publish
    review's N-3 finding verbatim (no-reallocation property on `qr_text`'s
    `Zeroizing<String>` buffer has no regression guard; fix would be a
    four-line `assert_eq!(got.capacity(), got.len())` in
    `the_worst_case_is_194_bytes`, `crates/ms-codec/tests/hashlock_qr_text.rs`);
    filed as secret-handling class, non-blocking per the 2026-08-27 operator
    ruling. **The assertion itself was NOT added** — the tree under release is
    reviewed as-is, per the addendum's explicit instruction.
- Verified before committing: `git diff --stat` → `MIGRATION.md | 46
  ++++++++++++++++++++++++++++++++++++++++++++++`, `design/FOLLOWUPS.md | 20
  ++++++++++++++++++++`, `2 files changed, 66 insertions(+)` — exactly the two
  named files, no other change.
- Commit message written to
  `/scratch/code/shibboleth/.tmp/commit-msg-ms-records-0.9.0.txt`, committed via
  `git commit -F`:
  ```
  git add MIGRATION.md design/FOLLOWUPS.md
  git commit -F /scratch/code/shibboleth/.tmp/commit-msg-ms-records-0.9.0.txt
  ```
  Result: `[master 990df82] H6 Task 1 (ms): release records -- MIGRATION.md
  v0.8->v0.9 and two FOLLOWUPS entries` — `2 files changed, 66 insertions(+)`.
- `git rev-parse HEAD` → `990df82e2ec40c00d6fd91f0d2ae36351557861b`. This is
  the release commit the tag lands on.

## Vendor-freshness gate

```
$ bash ci/repro/vendor-freshness.sh
vendor-freshness: resolving Cargo.lock against committed vendor/ (offline, locked; mnemonic-io-lib rev 6c24e62823e6c1ac02aa3862cd6020674bf58544) ...
vendor-freshness: OK — vendor/ satisfies Cargo.lock.
```
Exit 0. The records commit touched only docs, so `Cargo.lock` did not move;
no re-vendor was needed.

## ci/staging push ritual

- Preflight: `git status --short` clean; `git rev-parse master` →
  `990df82e2ec40c00d6fd91f0d2ae36351557861b`; `git rev-parse origin/master`
  (before staging) → `e96676c0004bc1a7fea6f5408f53ea4f577f3b18`.
- `git push origin master:refs/heads/ci/staging` → `* [new branch] master ->
  ci/staging` (log:
  `/scratch/code/shibboleth/.tmp/staging-push-ms-990df82.log`).
- `gh run list --repo bg002h/mnemonic-secret --commit 990df82... --json
  databaseId,workflowName,status,conclusion,headSha` (after an 8s wait — the
  first call returned `[]`) → three runs on this exact SHA: `vendor-freshness`
  (34027682706), `fuzz-smoke` (34027682714), `rust` (34027682698, the one
  carrying the required contexts).
- `gh run watch 34027682698 --repo bg002h/mnemonic-secret --exit-status` →
  every job shown green (`test (ubuntu-latest)`, `test (macos-latest)`, `test
  (ms-codec)`, `clippy`, `clippy (ms-codec)`, `fmt (pinned 1.95.0)`, `miri
  (mlock unsafe)`, `history purge`, `g6 invariant (cross-repo mlock.rs)`,
  `freebsd compile-gate (whole-crate)`, both `musl compile/test` jobs, `test
  (release, ubuntu-latest, mlock einval)`); **WATCH EXIT: 0**.
- Per-job conclusions via `gh api
  repos/bg002h/mnemonic-secret/commits/990df82.../check-runs --jq
  '.check_runs[] | "\(.name)\t\(.conclusion)"'`:
  ```
  cargo fuzz run (60s smoke)                              skipped
  musl compile/test (aarch64-unknown-linux-musl)          success
  freebsd compile-gate (whole-crate)                      success
  g6 invariant (cross-repo mlock.rs)                      success
  clippy                                                   success
  test (release, ubuntu-latest, mlock einval)             success
  musl compile/test (x86_64-unknown-linux-musl)           success
  test (ms-codec)                                          success
  fmt (pinned 1.95.0)                                     success
  test (ubuntu-latest)                                     success
  test (macos-latest)                                      success
  history purge (recipes RUN under real shells)           success
  miri (mlock unsafe)                                      success
  cargo fuzz build (compile gate)                          success
  clippy (ms-codec)                                        success
  vendor/ satisfies Cargo.lock (offline)                  success
  ```
  **Required four — `test (ubuntu-latest)`, `clippy`, `test (ms-codec)`,
  `clippy (ms-codec)` — all SUCCESS.** Every other context on the SHA also
  success (`cargo fuzz run` is a non-required `skipped`).
- Tip check before the real push: `git status --short` empty, `git rev-parse
  master` still `990df82e2ec40c00d6fd91f0d2ae36351557861b` — the tip did not
  move between the staging push and this point.
- `git push origin master` → `e96676c..990df82  master -> master` (log:
  `/scratch/code/shibboleth/.tmp/push-ms-990df82.log`).
- `grep -i "bypass" /scratch/code/shibboleth/.tmp/push-ms-990df82.log` → no
  match (grep exit 1). **No "Bypassed rule violations" line — not bypassed.**
  The push was accepted on the strength of the already-passing required
  contexts.
- `git push origin --delete ci/staging` → `- [deleted] ci/staging`.
- `git fetch origin && git rev-parse origin/master` →
  `990df82e2ec40c00d6fd91f0d2ae36351557861b` (matches local tip exactly).
- `git ls-remote origin refs/heads/ci/staging` → empty (staging ref absent).

## Tag

```
$ git tag -l "ms-codec-v0.9.0"        # empty -- no prior tag
$ git tag -a ms-codec-v0.9.0 -m "ms-codec 0.9.0 -- the hashlock PHRASE rule and qr_text move in from ms-cli" 990df82e2ec40c00d6fd91f0d2ae36351557861b
$ git show --no-patch --format="%H %D" ms-codec-v0.9.0
990df82e2ec40c00d6fd91f0d2ae36351557861b HEAD -> master, tag: ms-codec-v0.9.0, origin/master, origin/HEAD
```
Tag lands exactly on the release commit (which itself carries `MIGRATION.md`
+ `design/FOLLOWUPS.md`, per the addendum's requirement that the released
commit carry them).

```
$ git push origin ms-codec-v0.9.0
 * [new tag]         ms-codec-v0.9.0 -> ms-codec-v0.9.0
```

## Pre-publish check

```
$ curl -s -H "User-Agent: mnemonic-secret-release-agent (goss.brian@gmail.com)" https://crates.io/api/v1/crates/ms-codec | python3 -c "..."
0.8.0 0.8.0
```
`max_version`/`max_stable_version` both `0.8.0` before publish — no prior
partial 0.9.0 attempt (the plain `curl` without a `User-Agent` header returned
an empty body / JSON decode error; crates.io requires one).

## Publish, from a detached worktree at the tag

```
$ git -C /scratch/code/shibboleth/mnemonic-secret worktree add --detach /scratch/code/shibboleth/ms-worktrees/release-0.9.0 ms-codec-v0.9.0
HEAD is now at 990df82 H6 Task 1 (ms): release records -- MIGRATION.md v0.8->v0.9 and two FOLLOWUPS entries
```

Dry-run first (env: `PATH=$HOME/.cargo/bin:$PATH TMPDIR=/scratch/code/shibboleth/.tmp CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/h6-release-target`):
```
$ cargo publish -p ms-codec --locked --dry-run
   Packaging ms-codec v0.9.0 (.../ms-worktrees/release-0.9.0/crates/ms-codec)
   Packaged 50 files, 414.9KiB (125.1KiB compressed)
   Verifying ms-codec v0.9.0 ...
   Compiling ms-codec v0.9.0 (.../h6-release-target/package/ms-codec-0.9.0)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.74s
   Uploading ms-codec v0.9.0 ...
warning: aborting upload due to dry run
```
Clean (no error above the intentional dry-run abort).

Real publish, same env and worktree:
```
$ cargo publish -p ms-codec --locked
   Packaging ms-codec v0.9.0 (.../ms-worktrees/release-0.9.0/crates/ms-codec)
   Packaged 50 files, 414.9KiB (125.1KiB compressed)
   Verifying ms-codec v0.9.0 ...
   Compiling ms-codec v0.9.0 (.../h6-release-target/package/ms-codec-0.9.0)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.30s
   Uploading ms-codec v0.9.0 (.../ms-worktrees/release-0.9.0/crates/ms-codec)
    Uploaded ms-codec v0.9.0 to registry `crates-io`
note: waiting for `ms-codec v0.9.0` to be available at registry `crates-io`.
   Published ms-codec v0.9.0 at registry `crates-io`
```
(log: `/scratch/code/shibboleth/.tmp/cargo-publish-ms-codec-0.9.0.log`)

## Post-publish verification

```
$ curl -s -H "User-Agent: mnemonic-secret-release-agent (goss.brian@gmail.com)" https://crates.io/api/v1/crates/ms-codec | python3 -c "..."
max_version: 0.9.0
max_stable_version: 0.9.0
```

Worktree cleanup:
```
$ git worktree remove /scratch/code/shibboleth/ms-worktrees/release-0.9.0
$ git worktree list
/scratch/code/shibboleth/mnemonic-secret   990df82 [master]
/scratch/code/shibboleth/ms-worktrees/h6-a 1a4f4aa [h6-a]
```
`master` at `990df82`, working tree clean, matches `origin/master`.

## Verdict

**SUCCESS.** Records commit `990df82` (MIGRATION.md v0.8→v0.9 + two
FOLLOWUPS entries) landed before the push, carries the release; pushed via
the `ci/staging` ritual with all four required contexts (and every other
context on the SHA) green; real push accepted with no bypass; tag
`ms-codec-v0.9.0` at `990df82`, pushed; `cargo publish -p ms-codec --locked`
succeeded from a detached worktree at the tag after a clean dry-run;
`max_version` on crates.io verified `0.9.0`.

## What I read (read-only, beyond the brief)

`design/RELEASE_PROCESS.md`; `design/agent-reports/decision-crates-io-publish-ms-codec-0.8.0.md`,
`push-ms-e96676c.md`; `MIGRATION.md` (full, for the v0.1→v0.2 precedent
structure); `CHANGELOG.md` (top ~90 lines); `design/FOLLOWUPS.md` (header +
tail); `crates/ms-codec/src/hashlock.rs` (grep for the five public items);
`crates/ms-codec/tests/hashlock_qr_text.rs` (grep for the N-3 test name);
`crates/ms-codec/Cargo.toml` (version line); engrave
`design/agent-reports/hashlock-H6-A-pre-publish-review.md` (N-3 finding
text, verbatim); engrave
`design/IMPLEMENTATION_PLAN_hashlock_H6_preimage_plates.md` (Task 1 Step 6,
Task 2 Step 0, the fork's Task 5b mention). No `.jsonl` file was opened.
