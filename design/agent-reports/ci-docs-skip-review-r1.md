r1: 0C/0I/0M/1N

# L3 delta review r1: mnemonic-secret PR #13 (feat/ci-docs-skip, head 21ceae99)

Cursory delta review per brief `ci-skip-constellation/briefs/l3-mnemonic-secret-r1.md`.
Settled items (shared files, SPEC, gate, L1/L2) were not re-reviewed.
Diff examined: `git diff b728f160..21ceae99`.

## Deltas checked

1. **docs-only-paths / exempt.** `ci/docs-only-paths.txt` gains one `-` rule,
   `^docs/verify-reproducibility\.md$`. The file is named only in an `::error::`
   message in `ci/repro/vendor-freshness.sh`, which runs in the unguarded
   `vendor-freshness.yml`. Over-protective but fail-safe (a change to it runs
   everything), and it matches the SPEC §3 table. `ci/code-docs-exempt.txt`
   exempts only `scripts/plan-handwire-ms-hashlock.py`; grep shows it is called
   only by `scripts/plan-build-gate-ms.sh`, and no workflow or CI script
   references either script. No exempt file is run by a guarded step. OK.
2. **Insertion sites.** `rust.yml` job `clippy`: the new step sits directly after
   `Self-test the docs-only guard`, with no `if:` (the self-test has none).
   `release.yml` job `test`: directly after the self-test, with
   `if: runner.os == 'Linux' && github.event.inputs.tag == ''` as its second
   line, identical to the self-test's. Both match the SPEC §2.3 table. OK.
3. **release.yml `test` job.** `ref:` is
   `${{ github.event.inputs.tag || (startsWith(github.ref, 'refs/tags/') && github.ref) || github.sha }}`;
   `name:` is `release test (${{ matrix.os }})` plus ` (tag)` on a tag run;
   `CHECK_NAME: release test (${{ matrix.os }})`. All as SPEC §2.1.
   Old-name references: the live required contexts
   (`gh api repos/bg002h/mnemonic-secret/branches/master/protection/required_status_checks`)
   are `test (ubuntu-latest)`, `clippy`, `test (ms-codec)`, `clippy (ms-codec)`,
   `test (macos-latest)`. Both `test (<os>)` contexts come from `rust.yml`'s
   `test` matrix `[ubuntu-latest, macos-latest]`, which runs on every PR and on
   pushes to main/master/ci/** with no path filter. So the rename removes no
   producer that a required check depends on. No rulesets. The only `needs:` on
   the job is `needs: [test, build]` (release.yml:261), which uses the job id,
   and the id is unchanged. `scripts/push-via-staging.sh:164` mentions
   `test (ubuntu-latest)` only in a comment about the required (rust.yml)
   context. The other hits are historical briefs and reports. OK.
4. **PR description.** It says the PR changes `.github/workflows/` and `ci/`,
   describes A′, the code-docs check and the release rename, cites the SPEC, and
   ends with the Claude Code footer. The PR is a draft. OK (see N1).
5. **parity_smoke.rs.** Exactly one line changed, inside a format-string
   message: `see design/FOLLOWUPS.md \` became `see the FOLLOWUPS entry \`, the
   SPEC §3 step 4 text verbatim. It is only a message: nothing reads the file.
   Nothing else in the file changed. OK.

## Findings

- **N1** (nit) The PR description says it changes `.github/workflows/` and
  `ci/`, but it does not mention the one-line message reword in
  `crates/ms-codec/tests/parity_smoke.rs`.
