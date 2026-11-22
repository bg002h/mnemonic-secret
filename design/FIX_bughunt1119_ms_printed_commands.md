# FIX: the `--in` hex note prints a command that fails (bh-ms-I6) and the `gen-man` help example writes to `/man/man1` (bh-ms-I9)

Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-22. FIX table row #46 of
`coordinator/agent-reports/architect-bughunt-1119-questions.md` (L158). **Small track**, grade I for both keys
(`coordinator/agent-reports/bughunt-1119/grade-ms.md` L37 and L40; fix shape at L64-65). No T-ruling and no product
ruling (B1-B10, P1-P10) names this row ("none" in the table). Base: `origin/master` `423f1734` (ms-cli 0.21.0).
Sources: `design/agent-reports/bug-hunt-2026-11-19.md` L136-152 (I6) and L172-184 (I9); triage
`coordinator/agent-reports/bughunt-1119/triage-ms-t2.md` L12 (I6) and `triage-ms-t3.md` L14 (I9).

Every statement of current behaviour below was measured on a build of `423f1734`; every statement of fixed behaviour
was measured on a scratch build with section 3 applied and then reverted (not committed). The scratch diffs stay
outside the repo as reference only: `/scratch/cache/ms-printed-commands-red/scratch-fix-src-v2.diff` (sha256
`da1087f0...46ac`), `scratch-fix-tests-v2.diff` (`7c4d9beb...616c`) and `bh1119_printed_commands.rs.scratch2`
(`f1e8bc8f...6b71`); the builder writes from sections 3 and 5, not from those files. Workspace suite on the base:
`758 tests run: 758 passed, 23 skipped`; on the scratch build with the section 5 tests: `769 tests run: 769 passed,
23 skipped` (758 + 11 new functions; the changed f677 function passes). `cargo clippy --locked -p ms-cli
--all-targets -- -D warnings` is clean on the scratch build.

## 1. Summary

1. **bh-ms-I6.** `encode.rs:329-345` `phrase_parse_error` prints, when `--in FILE` does not parse as a phrase,
   `ms <verb> --hex - < <path>`. For `split` it drops `-k`/`-n` (the printed line exits 64, clap: required
   `--threshold`/`--shares`); the path is unquoted (a name with a space or a `'` breaks the shell line); and a
   requested `--out FILE` is dropped (the printed line sends the secret to stdout instead of the file the holder named).
2. **bh-ms-I9.** `main.rs:139` `ms gen-man --help` prints `ms gen-man --out "$XDG_DATA_HOME/man/man1"`. With
   `XDG_DATA_HOME` unset or empty that is `/man/man1`: exit 1 (`Permission denied`) for a normal user, a write under
   `/` for one who can create it. The help-example test (`f677_help_matches_behaviour.rs:209`) sets the variable on
   every run, which hid this.

## 2. Spec delta (holder-visible behaviour)

No SPEC file carries either text (`git grep "never sniffs"` and the gen-man help find only the code), so the delta is
stated here and in the CHANGELOG.

**R1 (I6, split K/N).** On `ms split`, the note's command carries the threshold and share count the holder gave, as
`-k <K> -n <N>` right after the verb, whichever spelling (`-k`/`--threshold`, `-n`/`--shares`) and order was used.
PROPOSED (architect confirms): the short spelling, the same one the argv-guard remedy uses (`argv_guard.rs:531`,
`-k K -n N`). For K or N out of range see Q2.

**R2 (I6, quoting).** Every path in the note's command is written as one POSIX single-quoted word: `'` + the path +
`'`, with each `'` in the path written `'\''`. Always, also for a plain name (`'entropy.hex'`). The path is the one
the holder gave (`Path::display()`, unchanged from today; relative stays relative). The prose sentence before it
(`If <path> holds hex entropy`) is unchanged and unquoted.

**R3 (I6, output destination).** When the holder gave `--out FILE`, the note's command carries `--out '<FILE>'`
(quoted per R2) after the verb (and after `-k`/`-n` on split), before `--hex -`. This is the consultant's "retain K/N
and any requested output destination" (bug report L150). No other flag is carried (see Q1).

**R4 (I9).** The second `ms gen-man --help` EXAMPLES line becomes
`  ms gen-man --out "${XDG_DATA_HOME:-$HOME/.local/share}/man/man1"   # user manpath` (the expression
`scripts/install.sh` uses in mnemonic-toolkit, L223). The first line and the comment text are unchanged. The
generated `ms-gen-man.1` page carries the same line (clap_mangen).

Exit codes: unchanged (the note's verb still exits 1 with `error: BIP-39 ...`; `gen-man` unchanged).

### Exact printed texts (stderr, after the fix; 10 leading spaces as today)

| invocation (after `ms`) | note's command line, today | after the fix |
|---|---|---|
| `encode --in entropy.hex` | `ms encode --hex - < entropy.hex` | `ms encode --hex - < 'entropy.hex'` |
| `encode --in "two words.hex"` | `ms encode --hex - < two words.hex` | `ms encode --hex - < 'two words.hex'` |
| `encode --in "it's.hex"` | `ms encode --hex - < it's.hex` | `ms encode --hex - < 'it'\''s.hex'` |
| `encode --in entropy.hex --out "my card.ms1"` | `ms encode --hex - < entropy.hex` | `ms encode --out 'my card.ms1' --hex - < 'entropy.hex'` |
| `split -k 2 -n 3 --in entropy.hex` | `ms split --hex - < entropy.hex` | `ms split -k 2 -n 3 --hex - < 'entropy.hex'` |
| `split --in entropy.hex --shares 5 --threshold 3` | `ms split --hex - < entropy.hex` | `ms split -k 3 -n 5 --hex - < 'entropy.hex'` |
| `split -k 2 -n 3 --in "two words.hex" --out shares.txt` | `ms split --hex - < two words.hex` | `ms split -k 2 -n 3 --out 'shares.txt' --hex - < 'two words.hex'` |

The note's first line is unchanged: `note: --in reads a PHRASE and never sniffs the file's contents. If <path> holds
hex entropy, use the hex channel:`.

## 3. Code delta (production; prose, no code)

- `crates/ms-cli/src/cmd/encode.rs`: `resolve_secret_payload` (L119-125) and `phrase_parse_error` (L329-345) take the
  verb as `&str` instead of `&'static str` (it is now built per call). `phrase_parse_error` writes the path in the
  command line through a new private-to-crate helper that applies R2; the prose `{path}` stays `p.display()`. A second
  crate helper builds the note's verb part: the verb, then ` --out '<FILE>'` when `--out` was given (R3). `run`
  (L246-252) passes that helper's result for `encode` with `args.out` instead of the literal `"encode"`.
- `crates/ms-cli/src/cmd/split.rs` L115-121: passes the same helper's result for `split -k {k} -n {n}` with `args.out`,
  instead of the literal `"split"`.
- `crates/ms-cli/src/main.rs` L139: the `after_long_help` string per R4.
- `CHANGELOG.md` under `## ms-cli [Unreleased]`, a `### Fixed` entry, PROPOSED (architect confirms):
  `- **The \`--in\` hex note prints a command that runs.** On \`ms split\` it carries \`-k\`/\`-n\`; a given \`--out\`
  is carried; paths are single-quoted. \`ms gen-man --help\`'s user-manpath example falls back to
  \`$HOME/.local/share\` when \`XDG_DATA_HOME\` is unset or empty (bh-ms-I6, bh-ms-I9).`

## 4. RED rows

The rows are run by `scripts/ms-printed-commands-fix-gate.sh` (committed with this document): each in a fresh
directory; test vector only (16 zero bytes as hex, card `ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f`). Each I6
row runs `ms <args>`, takes the stderr line that starts with `ms `, and runs it in `sh -c` exactly as printed. Printed
commands as run:

    scripts/ms-printed-commands-fix-gate.sh red   <ms built at 423f1734> /scratch/cache/ms-printed-commands-red/gate-red
    scripts/ms-printed-commands-fix-gate.sh green <ms built with section 3> /scratch/cache/ms-printed-commands-red/gate-green

Results: red on `423f1734`: `rows failed: 0`, rc 0 (10 RED rows showed their defect, 2 controls held); green on the
scratch build: `rows failed: 0`, rc 0 (12 rows held). Self-test: green mode on `423f1734` gives `rows failed: 10`,
rc 1; red mode on the scratch build gives `rows failed: 10`, rc 1.

| row (test function) | command (after `ms`) | observed at `423f1734` (RED) | expected after the fix |
|---|---|---|---|
| printed_cmd_split_remedy_runs | `split -k 2 -n 3 --in entropy.hex`, then the printed line | printed `ms split --hex - < entropy.hex`; it exits 64 `error: the following required arguments were not provided:` | printed `ms split -k 2 -n 3 --hex - < 'entropy.hex'`; exits 0 with 3 shares on stdout |
| printed_cmd_split_remedy_long_flags_runs | `split --in entropy.hex --shares 5 --threshold 3`, then the printed line | printed `ms split --hex - < entropy.hex`; exit 64 as above | printed `ms split -k 3 -n 5 --hex - < 'entropy.hex'`; exits 0 with 5 shares |
| printed_cmd_encode_remedy_text | `encode --in entropy.hex` | printed `ms encode --hex - < entropy.hex` (runs; the row pins the text) | printed `ms encode --hex - < 'entropy.hex'`; exits 0, last stdout line the card |
| printed_cmd_encode_remedy_path_with_space_runs | `encode --in "two words.hex"`, then the printed line | `sh: line 1: two: No such file or directory`, exit 1 | printed `ms encode --hex - < 'two words.hex'`; exits 0, the card |
| printed_cmd_encode_remedy_path_with_quote_runs | `encode --in "it's.hex"`, then the printed line | `sh: -c: line 1: unexpected EOF while looking for matching` `` ` ``, exit 2 | printed `ms encode --hex - < 'it'\''s.hex'`; exits 0, the card |
| printed_cmd_split_remedy_path_with_space_runs | `split -k 2 -n 3 --in "two words.hex"`, then the printed line | `sh: line 1: two: No such file or directory`, exit 1 | printed `ms split -k 2 -n 3 --hex - < 'two words.hex'`; exits 0, 3 shares |
| printed_cmd_encode_remedy_keeps_out | `encode --in entropy.hex --out "my card.ms1"`, then the printed line | printed `ms encode --hex - < entropy.hex`; exits 0 but the card goes to stdout and `my card.ms1` is never written | printed `ms encode --out 'my card.ms1' --hex - < 'entropy.hex'`; exits 0, stdout of the first run empty, `my card.ms1` holds the card and a newline |
| printed_cmd_split_remedy_keeps_out_and_recombines | `split -k 2 -n 3 --in "two words.hex" --out shares.txt`, then the printed line, then `combine --in two.txt --json` on its first two shares | `sh: line 1: two: No such file or directory`, exit 1 | printed `ms split -k 2 -n 3 --out 'shares.txt' --hex - < 'two words.hex'`; exits 0; `shares.txt` holds 3 shares; combine of two gives `"entropy_hex":"00000000000000000000000000000000"` |
| printed_cmd_gen_man_example_runs_without_xdg | the user-manpath line of `gen-man --help`, in `sh -c` with an empty environment but PATH and HOME | `error: cannot create --out dir: Permission denied (os error 13)`, exit 1 | exits 0; `$HOME/.local/share/man/man1/ms.1` exists |
| printed_cmd_gen_man_example_runs_with_empty_xdg | the same with `XDG_DATA_HOME=` | as above, exit 1 | exits 0; `$HOME/.local/share/man/man1/ms.1` exists |

Control rows (hold before and after):

| row (test function) | command | expected, both builds |
|---|---|---|
| printed_cmd_gen_man_example_honours_xdg | the user-manpath line with `XDG_DATA_HOME=<dir>/xdg`, `HOME=<dir>/home` | exit 0; `<dir>/xdg/man/man1/ms.1` exists; `<dir>/home` not created |
| printed_cmd_hex_channel_control | `encode --hex - < entropy.hex` | exit 0, last stdout line the card |

## 5. Tests that change

- **Changed:** `crates/ms-cli/tests/f677_help_matches_behaviour.rs` `every_help_example_runs_and_none_puts_material_on_argv`
  (L179-238): each EXAMPLES line runs twice, once with `XDG_DATA_HOME` set to the run's temp dir as today (L209) and
  once with it removed from the environment (`HOME` still the temp dir); the failure message names which; the floor
  `ran >= 30` (L237) becomes `ran >= 60`. Measured: on `423f1734` it fails at the gen-man line,
  `` `ms gen-man --out "$XDG_DATA_HOME/man/man1"   # user manpath (XDG_DATA_HOME unset)` exited Some(1) ``; on the
  scratch build it passes.
- **Unchanged and still passing:** `in_flag_means_a_phrase.rs:69` asserts the note contains `--hex - <` (still true).
  No other test pins the note's text or the gen-man help line (`git grep` on `never sniffs`, `hex - <`,
  `XDG_DATA_HOME` under `crates/`).
- **New integration test file** `crates/ms-cli/tests/bh1119_printed_commands.rs`: one `#[test]` per row of section 4
  (both tables), named exactly as the row (12 functions). Each I6 row runs the built `ms` (`assert_cmd`) in its own
  `tempfile::tempdir()`, asserts exit 1, takes the trimmed stderr line starting with `ms `, asserts it equals the
  "after the fix" text, runs it with `sh -c` in the same directory with the built `ms` first on PATH, and asserts the
  "expected after the fix" result. The gen-man rows run the help's line (the trimmed help line starting with
  `ms gen-man --out "$`, cut at `   #`) in `sh -c` with `env_clear()` plus PATH and HOME (and `XDG_DATA_HOME` as the
  row says). The builder runs the file on the base first and records that exactly the 10 RED functions fail and the
  2 controls pass before writing the code. (Measured on the scratch file: the 11 functions it held pass on the scratch
  build; 8 of them, the first version, were seen RED on the base; the hex-channel control is new here.)

## 6. Toolkit manual lockstep

None. No flag or exit code changes, and the manual quotes neither changed text: mnemonic-toolkit `origin/master`
`04c56bfa` `docs/manual/src/40-cli-reference/43-ms.md` has no `never sniffs` note and no `XDG_DATA_HOME` example
(its `ms gen-man` section, L867-896, shows only the synopsis `ms gen-man --out <DIR>`); its `--in` row (L84, `use
--hex - < FILE`) stays true. No mnemonic-gui mirror change (no flag change).

## 7. Coverage map (rule -> rows)

| rule | rows |
|---|---|
| R1 | printed_cmd_split_remedy_runs, printed_cmd_split_remedy_long_flags_runs, printed_cmd_split_remedy_path_with_space_runs, printed_cmd_split_remedy_keeps_out_and_recombines |
| R2 | printed_cmd_encode_remedy_text, printed_cmd_encode_remedy_path_with_space_runs, printed_cmd_encode_remedy_path_with_quote_runs, printed_cmd_split_remedy_path_with_space_runs (and every row's exact text) |
| R3 | printed_cmd_encode_remedy_keeps_out, printed_cmd_split_remedy_keeps_out_and_recombines |
| R4 | printed_cmd_gen_man_example_runs_without_xdg, printed_cmd_gen_man_example_runs_with_empty_xdg, printed_cmd_gen_man_example_honours_xdg (control); f677 `every_help_example_runs_and_none_puts_material_on_argv` (changed) |
| hex channel itself | printed_cmd_hex_channel_control |

## 8. Gate (builder runs once to a file and greps it)

Rust pin `cargo 1.85.0`, `$HOME/.cargo/bin` first in PATH, `--locked`, no `--release`, worktree-own `target/`:

    cargo nextest run --locked --workspace --build-jobs 6 -j 6 --no-fail-fast
    cargo test --locked --doc --workspace
    cargo clippy --locked --all-targets --workspace -- -D warnings
    cargo +1.95.0 fmt --all -- --check
    bash ci/code-docs-check.sh && bash ci/docs-only.test.sh
    scripts/ms-printed-commands-fix-gate.sh green target/debug/ms /scratch/cache/ms-printed-commands-gate-green

Expected: the suite is the base's 758 plus the 12 new functions, all passing; the gate script prints
`rows failed: 0`. Not covered by this gate: the macOS, musl, release, miri, FreeBSD and fuzz CI legs (CI on the PR
head), shells other than `sh`/`bash` (fish and csh do not expand `${VAR:-default}`; see Q3), and review.

## 9. Open questions (not decided here; the architect rules)

- **Q1. Other flags in the note's command.** The holder may also have given `--json`, `--language`,
  `--group-size`, `--separator`, `--no-engraving-card` (encode) or `--allow-argv-secret`. Options: (a) carry only
  `-k`/`-n` and `--out` (this document, the consultant's list); (b) carry every flag given except `--in`, in a fixed
  order; (c) as (a) plus `--json`. Recommend (a): none of the others changes which secret is encoded or where it is
  stored, and the shorter line is easier to read; file a Minor FOLLOWUPS line if (b) is wanted later.
- **Q2. K or N out of range.** `ms split -k 1 -n 3 --in entropy.hex` prints the note before the threshold is checked
  (`split.rs:125` runs after `resolve_secret_payload`), so after R1 the printed line `ms split -k 1 -n 3 --hex - <
  'entropy.hex'` exits 1 `error: invalid threshold 1; K-of-N shares require k in 2..=9` (also `-k 5 -n 3`:
  `error: invalid share count n=3 for threshold k=5; require k <= n <= 31`). Options: (a) carry K and N as given and
  accept that the line then reproduces the holder's own range error (a refusal, exit 1, nothing written); (b) check K
  and N (the existing `Threshold::new` and the `k <= n <= 31` check, same texts and exit 1) before the secret is read,
  as `--out` is already refused first (`split.rs:107-110`, before the secret is read), so the note is only ever printed with valid K/N;
  (c) print the note without `-k`/`-n` when they are out of range. Recommend (b): every printed line then runs, and a
  doubly wrong invocation sees the range error first. It changes the error order (not any text or exit code); rows
  would be `printed_cmd_split_bad_threshold_refused_before_read` (`split -k 1 -n 3 --in entropy.hex`: exit 1, stderr
  `error: invalid threshold 1; K-of-N shares require k in 2..=9`, no `note:` line) and the same for `-k 5 -n 3`.
- **Q3. Shell dialect.** R4's `${XDG_DATA_HOME:-$HOME/.local/share}` is POSIX (sh, bash, zsh, dash) and is the
  grader's and consultant's shape; fish does not expand it (today's `$XDG_DATA_HOME` form does expand in fish but
  fails when unset). R2's `'it'\''s'` quoting also reads correctly in fish. Options: (a) POSIX shells only, as every
  other ms help example is run under bash (this document); (b) a second example line for fish. Recommend (a).

Edge cases decided here (state them so the reviewer can object): a non-UTF-8 path is printed through
`Path::display()` as today (lossy), so the printed line names a different file and the shell fails with `No such
file` (fails closed); a path holding a newline is printed inside the quotes across two lines (the line still runs if
copied whole). Both are unchanged in kind from today.

## 10. FOLLOWUPS

None appended in the PR. The report lists the lines to direct-push after merge.
