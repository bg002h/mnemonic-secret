# Author report: FIX_ms_output_sink.md

Branch `fix/ms-output-sink` from master 548da55a4b709442be3391ec8d44a1b74f1aa61b. Document commit 1dd1941
(`design/FIX_ms_output_sink.md`, `scripts/ms-output-sink-fix-gate.sh`); this report is the last commit.

## Gate (run once, output in /scratch/cache/ms-output-sink-gate.log)
```
RED 9 ok (rc=100): cargo nextest run --locked -j 8 -E 'test(/(^|::)a_reserved_tag_exits_3_with_the_error_line$/)'
guard ok on the base: verify_says_ok_on_stdout_and_nothing_on_stderr
RED whole set: 10 tests run: 1 passed, 9 failed
self-test ok: the checker rejects a broken row
GREEN rows: 10 tests run: 10 passed
GREEN suite: 678 tests run: 678 passed, 23 skipped
GREEN doc, fmt, clippy: ok
GATE PASS
rc=0
```
Also printed earlier in the same run: `cmp: blocks identical to the grade's files`, RED 1 to 9 ok (rc=100 each),
`guard ok on the base`. Code blocks (8) sha256 0e1ea194be0d5770e25f4a95fc10366c2a9a8a990f1c881a51fa687fe9c03f62;
rows blocks (2) sha256 23079e7199db0df08cf4a76ef6f751e0992a45300a53e81ceb9653baf49a2bd8.
First gate run failed on my own mistake (whole-set RED without `--no-fail-fast` stopped at 9/10); fixed in the
script and the document, rerun green.

## Case table (document section 2)
Closed reader: verb's own code (0; repair 4; failing verify --json 1), no signal. Other stdout error: Ok code (0,
repair 4) becomes 1 with `ms: cannot write the output: <error>`; Err codes kept (decode --json bad card 1, verify
mismatch 4, verify reserved tag 3, hashlock --random --json 64). --help/--version/help: 0 closed, 1 full. Stderr
error ignored. `--out FILE` then stdout lost: 1, file stays, stderr card still printed. E4: decode, derive,
hashlock, repair on a reserved tag exit 3 with the error line on stderr; verify unchanged.

## STOPs and proposals
No STOP open (S1 ruled, transcribed). Two proposals, both marked P-n (proposed) in section 10:
- P-1: the grade's FOLLOWUPS entries carry `resolved <COMMIT>`; who fills it and when is unsaid. Proposed: the builder, in a following commit.
- P-2: the grade rules no manual text for E4's new stderr line; 43-ms.md has no sentence about it. Proposed: none.

## Not run / not covered
Plants (18, not re-run); CI legs macOS, musl, release, miri, FreeBSD, history-purge, g6, fuzz; the toolkit manual
section; mutation-gate.sh; no catalog row added for the new script (no catalog in this repo; the coordinator catalog is not mine to edit).
No production code on the branch. Nothing under /tmp used for builds.

Counts: Critical 0, Important 0, Minor 0, Nit 0
