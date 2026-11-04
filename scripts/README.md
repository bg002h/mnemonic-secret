Full catalog: https://github.com/bg002h/mnemonic-refugium/blob/coordinator/coordinator/scripts/README.md

| script | path | purpose | usage |
|---|---|---|---|
| code-docs-check.sh | ci/code-docs-check.sh | Fails if a scanned code file names a file that ci/docs-only.sh treats as documentation. | `bash ci/code-docs-check.sh` |
| code-docs-check.test.sh | ci/code-docs-check.test.sh | Scenario tests for code-docs-check.sh, one throwaway git repo per row. | `bash ci/code-docs-check.test.sh` |
| docs-only.sh | ci/docs-only.sh | Writes docs_only=true to $GITHUB_OUTPUT only when every changed file is documentation (env GITHUB_EVENT_NAME, GITHUB_BASE_REF, GITHUB_OUTPUT). | `bash ci/docs-only.sh` |
| docs-only.test.sh | ci/docs-only.test.sh | Scenario tests for docs-only.sh. | `bash ci/docs-only.test.sh` |
| vendor-freshness.sh | ci/repro/vendor-freshness.sh | Fails iff the committed vendor/ cannot satisfy Cargo.lock under the --offline --locked source-replacement config (two-block, fork-free form). | `bash ci/repro/vendor-freshness.sh` |
| plan-build-gate-ms.sh | scripts/plan-build-gate-ms.sh | Compiles and runs the Rust and JSON blocks of a mnemonic-secret implementation plan in a scratch copy; exit 3 if nothing is extracted (env MS_REPO, PRE_H1, TMPDIR). | `bash scripts/plan-build-gate-ms.sh design/IMPLEMENTATION_PLAN_ms_hashlock_H1.md` |
| plan-handwire-ms-hashlock.py | scripts/plan-handwire-ms-hashlock.py | Applies the H1 plan's fragment edits to a scratch copy; refuses a second run (sentinel .handwired); called by plan-build-gate-ms.sh. | `python3 scripts/plan-handwire-ms-hashlock.py <scratch-copy>` |
| push-via-staging.sh | scripts/push-via-staging.sh | The ci/staging push ritual: a SHA earns the required checks on ci/staging before the protected-branch push; repo-agnostic. | `bash scripts/push-via-staging.sh [branch]` |
