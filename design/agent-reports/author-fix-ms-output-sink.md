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

## R1 fold (2026-11-13)

Folded into `design/FIX_ms_output_sink.md` and `scripts/ms-output-sink-fix-gate.sh`:

- The architect's read-only-stdout ruling: sections 3 and 4 replaced byte for byte by
  `fix.diff` and `rows.diff`, all 17 `old:`/`new:` pairs of `texts.md`, the moved counts
  (rows 10 to 11, RED commands 9 to 10, suite 678 to 679, sink.rs 94 to 120 lines, two new
  plants P19 and P20). The folded file was confirmed to differ from the head document only by
  those pairs and blocks before use.
- I-1 (section 9 item 7: `release.yml` runs on a PR, with a Windows leg), verbatim.
- I-2 is replaced by the ruling, as the brief says.
- Rulings: P-1 and P-2 rewritten as decided (section 10 and the intro); P-2 now names toolkit
  `ms-manual-exit-codes` as the owner of exit-3 manual text (the wording change of the ruling).
- Gate: the compare runs against `/scratch/cache/architect-output-sink-ro/ms`; counts and the
  "does NOT cover" line updated.
- Filed in `design/FOLLOWUPS.md`: 6 Minors and 3 Nits as `deferred-minor`; the document's
  FOLLOWUPS hunk still passes `git apply --check` on the head.

Gate last line: `GATE PASS`.

Points to note (not STOPs): M-5 is both folded (it is the ruling's wording change) and filed,
so the finding keeps a trace. The deferred-minor entries use the file's `Status`/`Surfaced`/
`What` form with no Tier line, as the other deferred-minor entries do, and sit before the last
entry so the document's end-of-file hunk still applies.

STOPs: none.
