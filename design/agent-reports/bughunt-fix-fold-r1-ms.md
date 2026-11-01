fold r1: 05b4c64; gate PASS; 0 STOP

# Fold r1 report: `design/FIX_bughunt_ms.md` (ms bug-hunt), Sonnet xhigh

Branch `bughunt/triage`, local only, nothing pushed. One commit, `05b4c64`, on top
of `7400dfd`. Only `design/FIX_bughunt_ms.md` changed. Gate outputs are in
`/scratch/cache/bughunt/fold-r1/`. Scratch worktrees (`fold-r1-gate`,
`fold-r1-tk`) are removed.

## What changed, per section

| FIX section | change |
|---|---|
| header | reference patch is now r3 (sha256 `26c2026336da72ba760742fba0125290650a333d0ee40bfcaea9c6f32abdc043`), described as r2 (`b1f7d1be...3cd0e`, untouched on disk) plus the architect delta (alone `66a4316e...f6fa`, verified equal to the architect's stated value); `git apply --check` of r3 passes. Script sha256 now `f1bd63aeac7be38f414b8d866c054f3b220fa98ffe031b3adceb0a490c66c58c` (was `08306322...aa09`). Check table: nextest row 667 -> 668. RED-copy sentence: 20 run / 9 passed -> 21 run / 10 passed / 11 failed, "the 10 passes are the 8 mlock tests plus #10 and the control 04e". |
| 1 item 2 | architect edit 1 |
| 2.2 | architect edits 2 (table entropy row), 3 (stderr bullet + W in a `~~~text` fence + "Bare hex ..." paragraph), 4 (W line inserted in the `--to entropy` after-fix block), 5 (spec row) |
| 2.6 | intro reworded; the existing transcript kept; a second set of transcripts for every command the new 5.6.6 prints (typo + `--max-subst 1`; hashlock plate; K-of-N share + `ms combine`; mk1 and md1 with all strings, with a wrong guess, with only the damaged string; one-string md1; 5 left out); two new "measured" bullets citing the review's matrix |
| 3 item 3 | architect edit 6: new bullet inserted between the `emit_ms1` bullet and the Constraint bullet (placement was not specified) |
| 5.1 | architect edit 7 (new fragment) |
| 5.6.6 | review's replacement block (extracted programmatically from the review, not retyped). The closing sentence "The two commands printed in 5.6.6 are the ones run in 2.6" became "Every command printed in 5.6.6 is one of those run in 2.6" |
| 6 | architect edit 8: 04b and 04d assertions extended, new row 04e; edit 9: counts 13/13/17, 21/21/9, whole workspace 668/668/23. Section 7 "(667/667 above)" -> "(668/668 above)" |
| 8 item 3, 9.1 (3) | architect edit 10 |
| 9.2 | nine new `deferred-minor` entries, all "none — ownerless residue" except M-5 (owning phase: the toolkit's ms-codec bump): `ms-encode-shares-zero-validate-order` (M-1), `toolkit-primer-length-damage-bullet-contradicts-indel` (M-2), `toolkit-primer-long-code-erasures-15` (M-3), `fix-primer-insert-instruction-eats-heading` (M-4), `toolkit-ms-codec-bump-followup-text-incomplete` (M-5), `ms-repair-exit-table-char-outside-alphabet` (M-6), `primer-printed-commands-have-no-executed-pin` (M-7), `ms-invalid-char-message-invisible-char` (N-1), `fix-9-5-cite-toolkit-indel-erasure-item` (N-2). Fix texts and measurements are the review's. No Minor or Nit was fixed (M-4's instruction in 5.6.6 is left as written; the script does it right). |

`toolkit-manual-edits.py`: ninth edit replaced with the review's block; first
edit's `entropy` fragment replaced per architect edit 7. Overwritten in place;
the r2 version is kept at `fold-r1/toolkit-manual-edits.r2-backup.py`.

Reference patch r3: built in a scratch worktree (r2 applied, then the delta
applied with `git apply`), regenerated with `diff -u` in r2's format. The method
was proven first: regenerating r2 that way is byte-identical to the r2 file.
r3 differs from r2 only by the delta's hunks (plus offset shifts). It
round-trips: applying r3 to the fold head and regenerating gives r3 again.

## Gate (scratch worktree detached at the fold head, r3 applied after `git apply --check`)

`cargo 1.85.0` (the `rust-toolchain` pin; `~/.cargo/bin` first on PATH; the
system `/usr/bin/cargo` is 1.98.1 and was not used).

| item | result |
|---|---|
| `cargo nextest run --locked --workspace` | 668 run, 668 passed, 23 skipped (matches the architect's 668) |
| `cargo test --locked --doc --workspace` | 1 passed |
| `cargo fmt --all --check`; `cargo +1.95.0 fmt --all -- --check` | exit 0; exit 0 |
| `cargo clippy --locked --all-targets --workspace -- -D warnings` | exit 0 |
| `bash ci/code-docs-check.sh`; `bash ci/docs-only.test.sh` | ok (174 files, 143 docs files); 73 passed, 0 failed |
| 14 section 6 GREEN rows, each by exact name (`test(/(^\|::)<row>$/)`) | each: 1 run, 1 passed (includes 04b, 04d tightened, 04e new, tightened 04/05/07, T2) |
| group counts, patched | `binary(bughunt) & test(/^bughunt_ms_/)` 13 run, 13 passed, 17 skipped; ignored-only same filter 6 run, 6 failed; `binary(bughunt)` 21 run, 21 passed, 9 skipped; `binary(bughunt_t)` 1 run, 1 passed, 3 skipped; ignored-only 3 run, 3 failed |
| same rows over unmodified production code (new tests only) | 12 RED rows fail (01, 03, 07 codec; 04, 04b, 04c, 04d, 05, 05b, 07, 07b CLI; T2); 04e (control) passes, as the architect's text says; #10 passes (pin). `binary(bughunt)`: 21 run, 10 passed, 11 failed. 04b fails on stdout (left = the 8-line record) and 04d on stdout (left = `ms decode`'s object). |
| FIX 2.2-2.5 after-fix output vs the patched `ms` | 4 blocks, 15 commands, every stdout line, stderr line and exit code identical (`check_after.py`, `after-fix-check.txt`). The same harness against the installed `ms 0.20.1` reports 14 of 15 differing, so it can fail. |
| manual script on toolkit `c2b09861` | `applied 10 edits` (the script asserts each old text occurs once); `make lint`: 10 FAILs before, 10 after, the sorted lists identical, all `44-mk-cli.md` stale markers; `make anchor-check`: OK, 9 danglers = baseline; pandoc output has the `h3 id="unreadable-characters-what-to-type"`, both list items and the code block |

2.6 re-run: every result matched the review's advice-lens matrix (installed
`mnemonic 0.105.1`, `ms 0.20.1`): typo as printed exit 2, with `--max-subst 1`
exit 4; hash plate: search exit 2, `ms repair` exit 4, `mnemonic repair` exit 2;
share: all routes exit 2, `ms combine` of two clean shares exit 0; mk1 and md1
with all strings exit 5 (guess and leave-out), only the damaged string exit 2;
one-string md1: search exit 2, route 2 exit 4; 5 left out exit 2.

## What I could not do or did differently

- `make lint` with the Makefile defaults gives 226 FAILs in a scratch worktree
  (the defaults `cargo run --manifest-path ../descriptor-mnemonic/...` point at
  sibling repos that are not beside `ms-worktrees/`). I ran it with
  `MNEMONIC_BIN=mnemonic MD_BIN=md MS_BIN=ms MK_BIN=mk` (the installed tools),
  which gives the same 10 FAILs as before the edits and as the earlier record.
- Edits beyond the architect's list, all count or sentence consequences:
  header table (668), header RED-copy sentence (21/10/11), section 7 (668/668),
  5.6.6's closing sentence, and 2.6's intro. None changes a holder-visible text.
- The architect's three planted mutants were not re-run (not in the brief).
- The K-of-N share in the 2.6 transcript is a one-run value (`ms split` ids are
  random); the transcript says so.
- No `agent-tally.md` row written (the controller's file; tokens not known to me).
