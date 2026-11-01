fold r2: e67eb6a; gate PASS; 0 STOP

# Fold r2: `design/FIX_bughunt_ms.md` (folds `bughunt-fix-review-r2-ms.md`, c22a867)

Commit `e67eb6a` on `bughunt/triage`, local only, not pushed. Worktree `/scratch/code/shibboleth/ms-worktrees/bughunt-triage`. The first attempt's STOP (`bughunt-fix-fold-r2-ms-STOP.md`) is answered by the coordinator decisions: slugs, titles and tiers came from the brief, so no STOP.

## Changed

- **I-1:** FIX 5.6.6 New block and ninth edit of `toolkit-manual-edits.py` take the review's New text. FIX 2.6 gains the capitals/lowercase transcript block, prose above the fence.
- **9.4:** `toolkit-repair-indel-search-rejects-capitals`. I-1 was graded Important; it is filed deferred-minor per the coordinator's routing. The primer text is fixed. The tool behaviour is not.
- **9.2:** `fix-2-6-mk1-card-displays-truncated` (M-1), `primer-leave-out-two-strings-silent` (M-2), `fix-2-6-ms-split-transcript-abridged` (N-1). All deferred-minor, nothing fixed. **N-2 has no entry**; the I-1 New text closes it, and the commit message says so.
- **Script sha256:** `f1bd63ae…` to `e56d598ec4c486be260bbe5364eafbfa9ab4ffe32e57daaf9b8427cc164124ee`. The FIX header is re-recorded (its one occurrence).
- Backup: `/scratch/cache/bughunt/fold-r2/toolkit-manual-edits.r3-backup.py` (sha `f1bd63ae…`). Helper that extracts the texts from the report and asserts single occurrence: `/scratch/cache/bughunt/fold-r2/fold_r2.py`.

## Gate

| check | result |
|---|---|
| sha256 of script = FIX header | PASS (`e56d598e…`) |
| 2.6 capitals command, `mnemonic 0.105.1` (stdout empty, stderr, `exit=2`) vs block | PASS, exact |
| 2.6 lowercase command (stdout, stderr, `exit=5`) vs block | PASS, exact |
| script on scratch toolkit worktree at `c2b09861`: 10 edits, each once | PASS (`applied 10 edits`) |
| `make lint` after | 10 FAILs, all `44-mk-cli.md` |
| `make lint` on unedited `c2b09861` (the before-state) | same 10 FAIL lines (`diff` empty), PASS as before |
| `make anchor-check` after | PASS: `9 danglers … match baseline` |
| verbatim: I-1 Old/New, 2.6 block, M-1 fix lines, M-2 fix, N-1 fix, 9.4 quote | match report (whitespace-normalised; the report wraps lines) |
| four headings vs brief fence | exact, once each |
| fence balance in FIX | even (106 `~~~` lines) |

Outputs: `/scratch/cache/bughunt/fold-r2/` (`lint-after.txt`, `lint-unedited-c2b09861.txt`, `anchor-after.txt`, `script-apply.txt`, `gate2/transcript-actual.txt`). The scratch worktree is removed.

## Notes for the coordinator

- The old `lint-base.txt` wording differs from today's `c2b09861` output (`expected … 0.14.0`). The unedited-`c2b09861` run shows this is pre-existing, not from this fold.
- The 2.6 harness first reported a mismatch on case 0. It was only the blank separator line between the two commands; the outputs match.
- Nothing pushed. This report is uncommitted.
