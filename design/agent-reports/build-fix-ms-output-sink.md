# Build report: ms output-sink fix (FIX_ms_output_sink.md)

Branch `fix/ms-output-sink`, base head 6d0c18f. Commits: RED 97bb2fe, GREEN cf996a7,
`<COMMIT>` fill 6d0c18f (first builder); the code-docs exemption 798f839 (finisher).

## Applied
- RED: the 11 rows of `crates/ms-cli/tests/output_errors.rs` and
  `future_format_names_the_problem.rs`.
- GREEN: `crates/ms-cli/src/sink.rs` (new), `main.rs` and the decode, derive,
  hashlock and split verbs routed through the sink, the CHANGELOG `### Fixed`
  entry and two FOLLOWUPS entries.
- Fill: both FOLLOWUPS `resolved <COMMIT>` set to cf996a7.
- Finisher: two lines of exempt entry for `scripts/ms-output-sink-fix-gate.sh`
  appended to `ci/code-docs-exempt.txt` (the only `ci/` file touched).

## STOP that happened, and its ruling
The first builder stopped on `bash ci/code-docs-check.sh` (exit 1): the local gate script
`scripts/ms-output-sink-fix-gate.sh` names `design/` files, and fails the same way on the
branch before the build. Ruled by the coordinator: the script is run by hand and by no CI
step, so it takes an entry in `ci/code-docs-exempt.txt`, as `scripts/push-via-staging.sh`
does. Applied in 798f839; nothing else edited.

## Results (logs under /scratch/cache/build-fix-output-sink/ms/)
From the first builder's logs (summary lines grepped, not rerun):
- RED, each of ten rows: failed with its `## expect:` text (red-1..red-10); guard row
  passed (red-guard); whole set `11 tests run: 1 passed, 10 failed, 0 skipped` (red-set.log).
- GREEN rows: `11 tests run: 11 passed, 0 skipped` (green-rows.log).
- Workspace suite: `679 tests run: 679 passed, 23 skipped` (suite.log).
- ci-nextest-mscli `469 passed, 18 skipped`; ci-nextest-mscodec `210 passed, 5 skipped`;
  miri job, repro, vectors, doc, fmt and clippy logs end clean (fmt, clippy, doc,
  ci-* logs). These logs carry no rc line; I did not rerun them. Their exit codes were
  reported to me by the dispatch only as passing.

Run by the finisher on head 798f839, rc 0 each:
- `sha256sum --check --strict ci/shared-ci.sha256` and `bash ci/code-docs-check.test.sh`
  (`41 passed, 0 failed`): rc 0 (finish-ci-shared.log).
- `bash ci/code-docs-check.sh`: rc 0, `ok (177 files scanned, 155 docs files)`
  (finish-code-docs.log).
- `bash ci/docs-only.test.sh`: rc 0, `73 passed, 0 failed` (finish-docs-only.log).

## Left to CI (first builder's list)
- the macOS test leg;
- the release-mode mlock einval test and release.yml (its Ubuntu test, and the macOS and
  Windows build legs with smoke tests, which run only once the PR is out of draft);
- both musl legs;
- g6-invariant (needs a checkout of the mnemonic-toolkit sibling, not set up);
- fuzz-smoke.yml and vendor-freshness.yml as their path filters select.

## STOPs
None open.

Counts: Critical 0, Important 0, Minor 0.
