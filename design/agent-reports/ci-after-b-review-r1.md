r1: 0C/0I/0M/2N

# L3 review r1: mnemonic-secret PR #14 (B-sec), head f4ea970de1a263d1539e3ff391ab37b00f23f07f

Reviewer: Opus, independent whole-diff adversarial pass, per
`ci-skip-constellation/briefs/l3-ab-B-sec-r1.md`.

**The question:** can the change skip `build`'s build steps on an event whose
build output is used, or on a change that is not documentation, or break the
release (tag) path? **Answer: no.** No Critical, no Important.

Diff reviewed: `git diff origin/master...f4ea970` (`.github/workflows/release.yml`
+30/-6, plus the build report). Read in full against `ci/docs-only.sh` (unchanged)
and SPEC-after-b §4, §6, §7.

## Evidence run

- `gh api repos/bg002h/mnemonic-secret/commits/e4bf92b…/check-runs -f check_name=…
  -f filter=latest`: `build ms (macos-x86_64)` returns exactly that run
  (`completed/success`); `build ms (macos` and `build ms` return nothing. The
  `check_name` filter is an exact match, so a ` (tag)` run, or another leg's run,
  is never returned for a leg's `CHECK_NAME`.
- No other workflow in the repo (`fuzz-smoke`, `man-release`, `rust`,
  `vendor-freshness`) has a job whose name renders to `build ms (…)`
  (grep of every `name:`), so no other workflow's run can stand in.
- `origin/master` dccd989 already carries `build ms (macos-aarch64|macos-x86_64|windows-x86_64)`
  check runs (old workflow, see N2).
- No `include_str!`/`include_bytes!`/build input in `crates/` names a path under
  `design/`, `docs/` or a root `*.md` (the three `+` rules), so no build or
  smoke-test step reads a docs file.

## 1. Each guarded step, and what consumes the output

Every step after `Docs-only change?` carries `steps.docs.outputs.docs_only != 'true'`
(toolchain, cross ×2, cargo build, smoke test, stage, upload). There is no cache
save, attestation, signing or required check in this job. The only consumer of
`ms-<target>` artifacts is `assemble` (`Download all artifacts`, `pack`), whose
`if:` is `startsWith(github.ref,'refs/tags/') || inputs.tag != '' || inputs.sign_dry_run == 'true'`.
Per event:

| event | guard step runs? | `ci/docs-only.sh` result | build steps | `assemble` |
| --- | --- | --- | --- | --- |
| pull_request → master | yes | may skip (base rule / A′) | may skip | skipped (ref is `refs/pull/…`) |
| push master / `ci/**` | yes | may skip | may skip | skipped |
| push tag `ms-cli-v*` | yes (`inputs` null == `''`) | `push to refs/tags/…` → full | run | runs, gets artifacts |
| dispatch, `tag` set | no (`if:` false) → output empty | n/a | run | runs |
| dispatch, no tag (incl. `sign_dry_run`) | yes | `event workflow_dispatch` → full | run | runs on `sign_dry_run`, gets artifacts |
| dispatch from a tag ref, no tag input | yes | full (event) | run | runs |

So every event on which `assemble` runs is a full build; every event on which
the build can skip has no `assemble`. The release path is unchanged except the
checkout ref (below), which on tag and tagged-dispatch runs resolves exactly as
before.

## 2. `CHECK_NAME` versus the real check-run name

`CHECK_NAME: build ms (${{ matrix.name }})` renders per leg to the same string as
the job's untagged `name:`; the L2 run's three log lines quote the leg's own name.
Exact-match lookup (evidence above) means: a different leg's run never satisfies
it (names differ in `matrix.name`), a ` (tag)` run never satisfies it, and a
docs-skipped green run of the same leg satisfies it by the settled induction.

## 3. Permissions

Job-level `contents: read, checks: read` covers every step: checkout (contents
read), the guard (fetch + compare API need contents read; check-runs need checks
read). `upload-artifact@v4` uses the runtime token, not `GITHUB_TOKEN`. No step
uses OIDC, attestations or `contents: write`; those remain workflow-level for
`assemble`, which has no job-level block and is untouched. Repo is PUBLIC.

## 4. ` (tag)` suffix and checkout `ref:`

| event | checked out | check run attached to | name |
| --- | --- | --- | --- |
| pull_request | `github.sha` (the test-merge commit; before: `refs/pull/N/merge` re-resolved at checkout) | PR head | `build ms (x)` |
| push master / `ci/**` | `github.sha` (before: branch tip at checkout time) | `github.sha` | `build ms (x)` |
| push tag | `github.ref` (tag) — unchanged | tagged commit | `build ms (x) (tag)` |
| dispatch with `tag` | `inputs.tag` — unchanged | dispatch sha (a different tree) | `build ms (x) (tag)` |
| dispatch, no tag, branch ref | `github.sha` (before: branch tip at checkout time) | `github.sha` | `build ms (x)` |
| dispatch, no tag, tag ref | `github.ref` | tagged commit | `build ms (x) (tag)` |

Every untagged-name run now builds the tree of the commit its check run sits on
(or that commit's test merge, for a PR), which is what the guard's "passed on R"
assumes. The one run whose check-run commit differs from its built tree (dispatch
with `tag`) carries ` (tag)` and cannot be returned for an unsuffixed `CHECK_NAME`.
The reverse cannot happen: tagged runs never consult the guard (tag push → full;
dispatch with tag → guard step skipped).

Old-workflow runs under the unchanged untagged names (checkout `github.ref` at
checkout time, so possibly a newer tree than their sha): cannot produce a wrong
skip under the new workflow. The merge of this PR changes `.github/workflows/release.yml`
(no `+` rule matches), so its own push diff, any PR diff from a pre-merge base, and
any A′ diff from a pre-merge PR head all include a non-docs file and run full.

## 5. PR description

Accurate: names the three guard cases, the permissions, the `github.sha`
checkout and the ` (tag)` suffix, says it changes `.github/workflows/`, cites the
spec and the byte-identical reference, ends with the Claude Code footer. See N1.

## Minors and nits

- N1: PR description says "Each job now has `contents: read` and `checks: read` only" and "release build jobs", but B-sec guards one job (`build`, three legs); template wording, not wrong in effect.
- N2: SPEC-after-b §6.2's premise ("no check run under the new build names") does not hold for B-sec's untagged legs: the names are unchanged and dccd989 already carries `build ms (*)` runs. The conclusion ("runs in full, once") still holds, because this PR's merge diff includes `.github/workflows/release.yml`. Spec text only; no change needed here.
