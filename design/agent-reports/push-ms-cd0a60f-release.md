# Push + release: mnemonic-secret master `cd0a60f`, tags `ms-codec-v0.8.0` / `ms-cli-v0.18.0`

Date: 2026-09-04. Ritual: `ci/staging` push, then two release tags on the same SHA.

## Precondition check

- Tree: clean (`git status --short` empty).
- Tip before push: `cd0a60f` — matched expected.
- 3 unpushed commits (`git log origin/master..master`):
  - `cd0a60f` release: ms-codec 0.8.0 + ms-cli 0.18.0 -- corpus SHA pinned; H0 merged (fork c4a64fc, me 024dd08); released regardless of the device (ruling L26)
  - `a1e0a6f` rulings L24-L27 (operator, 2026-09-05): TagKindMismatch refused; --random requires --out; release regardless of the device; --hashlock-phrase - refused
  - `89231de` report + record: ms push 8796d69 via ci/staging -- four required contexts + vendor-freshness success, no bypass; H1 plan record notes the missing re-vendor step

## Staging run (`ci/staging`, full SHA `cd0a60fd0c2ce5ea2da6953c77dbb07056bc2ab2`)

Single workflow triggered: `rust`, run **33940697969**, conclusion **success**.

Check-run conclusions for this SHA (`gh api .../check-runs`):

| context | conclusion |
|---|---|
| test (ubuntu-latest) | success |
| clippy | success |
| test (ms-codec) | success |
| clippy (ms-codec) | success |
| test (macos-latest) | success |
| musl compile/test (x86_64/aarch64) | success |
| freebsd compile-gate | success |
| g6 invariant (cross-repo mlock.rs) | success |
| history purge | success |
| test (release, ubuntu-latest, mlock einval) | success |
| miri (mlock unsafe) | success |
| fmt (pinned 1.95.0) | success |

**`vendor/ satisfies Cargo.lock (offline)` did not appear for this SHA.** Investigated: `vendor-freshness.yml` is path-filtered (`Cargo.lock`, `Cargo.toml`, `crates/**/Cargo.toml`, `vendor/**`, `ci/repro/vendor-freshness.sh`, `.github/workflows/vendor-freshness.yml`); `git diff --name-only HEAD~3 HEAD` touches only `CHANGELOG.md`, `design/IMPLEMENTATION_PLAN_ms_hashlock_H1.md`, `design/SPEC_ms_hashlock.md`, `design/agent-reports/push-ms-8796d69.md` — none match, so the workflow correctly did not trigger. Confirmed via `gh api repos/bg002h/mnemonic-secret/branches/master/protection` that the actual enforced `required_status_checks.contexts` are exactly `["test (ubuntu-latest)", "clippy", "test (ms-codec)", "clippy (ms-codec)"]` — `vendor-freshness` is not a branch-protection-required context. All four enforced contexts: **success**.

## Master push

```
$ git push origin master
To github.com:bg002h/mnemonic-secret.git
   8796d69..cd0a60f  master -> master
```

No "Bypassed rule violations" text present — **satisfied, not bypassed**.

`git push origin --delete ci/staging` succeeded (`- [deleted] ci/staging`).

`git fetch origin && git rev-parse origin/master` == local tip == `cd0a60fd0c2ce5ea2da6953c77dbb07056bc2ab2`. Match confirmed.

## Tags

Both created as annotated tags on the pushed tip and verified via `git rev-list -n1 <tag>` to point at `cd0a60fd0c2ce5ea2da6953c77dbb07056bc2ab2` before pushing:

- `ms-codec-v0.8.0` — "ms-codec 0.8.0: the hashlock preimage kind"
- `ms-cli-v0.18.0` — "ms-cli 0.18.0: ms hashlock"

`git push origin ms-codec-v0.8.0 ms-cli-v0.18.0` → both reported `[new tag]`. Only these two tags were pushed (no `--tags`).

## Release-workflow runs triggered by the tags

`man-release.yml` triggers only on `ms-cli-v*` (push tags); there is no workflow that fires a *release* build on an `ms-codec-v*` tag. `fuzz-smoke.yml` is path-filtered but fired for both tag pushes (a new tag ref has no "before" commit, so GitHub treats all paths as changed on tag creation — expected quirk, not an error).

| run id | workflow | headBranch | conclusion |
|---|---|---|---|
| 33941268901 | man-release | ms-cli-v0.18.0 | success |
| 33941270115 | fuzz-smoke | ms-codec-v0.8.0 | success |
| 33941268613 | fuzz-smoke | ms-cli-v0.18.0 | success |

Per-job conclusions:

**33941268901 (man-release)**
- derive git-source pins — success
- ms-man.tar.gz release asset — success
- repro / build-container (resolve BUILT-DIGEST) — success
- repro / repro-aarch64-musl (aarch64-unknown-linux-musl) — **skipped**
- repro / repro-substrate (x86_64-unknown-linux-musl) — success
- repro / repro-x86_64-musl (x86_64-unknown-linux-musl) — success
- musl-binary (x86_64-unknown-linux-musl) — success
- musl-binary (aarch64-unknown-linux-musl) — success

**33941270115 (fuzz-smoke, ms-codec-v0.8.0)**
- cargo fuzz build (compile gate) — success
- cargo fuzz run (60s smoke) — skipped

**33941268613 (fuzz-smoke, ms-cli-v0.18.0)**
- cargo fuzz build (compile gate) — success
- cargo fuzz run (60s smoke) — skipped

No job outside success/skipped — no failing-log excerpt needed.

## Release assets

`gh release view ms-cli-v0.18.0`:
```json
{"tag":"ms-cli-v0.18.0","assets":["ms-0.18.0-aarch64-linux-musl.tar.gz","ms-0.18.0-x86_64-linux-musl.tar.gz","ms-man.tar.gz","PROVENANCE.aarch64.txt","PROVENANCE.x86_64.txt","SHA256SUMS.aarch64","SHA256SUMS.x86_64"]}
```

`gh release view ms-codec-v0.8.0` → **release not found** (expected: no workflow creates a GitHub Release for an `ms-codec-v*` tag; that tag exists solely as the version marker, with `cargo publish` explicitly out of scope for this task).

## Not done (per task scope)

`cargo publish` was not run. Nothing else was committed or pushed. No other repo was touched.

## Verdict

**SUCCESS**
