# FIX: `--allow-argv-secret` rewrites `--in`/`--out` file names (bh-ms-C1) and trims the `--hashlock-phrase=` value (bh-ms-C2)

Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-22. FIX table row #3 of
`coordinator/agent-reports/architect-bughunt-1119-questions.md` (L115). **Full track, class B (funds path):**
both keys are Critical (graded in `coordinator/agent-reports/bughunt-1119/grade-ms.md` L30-31, upheld by
`fable-critical-review.md` L18-19). Up to three review rounds, no separate plan. Base: `origin/master` `9c0f5f3`
(ms-cli 0.20.5). Sources: `design/agent-reports/bug-hunt-2026-11-19.md` L27-55 (consultant, C1 and C2),
`coordinator/agent-reports/bughunt-1119/triage-ms-t1.md` L11-12 (RED/GREEN), the grade and the Fable review above.
No architect T-ruling names this row ("none" in the table); no product ruling (B1-B10) bears on it.

Every statement of current behaviour below was measured on a build of `9c0f5f3`; every statement of fixed behaviour
was measured on a scratch build with the change of section 3 applied and then reverted (not committed). The scratch
diff is kept outside the repo as a reference only: `/scratch/cache/ms-argv-red/scratch-fix.diff` (sha256
`bc28e871e267f17fe76d0d766be134bc54adf7b7a4814b14038b618fb38ef20c`); the builder writes the code from section 3,
not from that file. On the scratch build `cargo nextest run --locked --workspace` gave `725 tests run: 725 passed,
23 skipped`, the same set as the base: no existing test changes.

## 1. Summary

1. **bh-ms-C1.** With `--allow-argv-secret`, the guard's substitution pass (`argv_guard.rs::substitute`) classifies
   every token by shape, including the FILE value of `--in` and `--out`. A file name of 32, 40, 48, 56 or 64 hex
   characters, or one shaped like an ms1 string, is taken for material and replaced by `-`. `--in -` and `--out -`
   then name a file literally called `-` in the current directory (no verb maps `--in -`/`--out -` to stdin/stdout).
   So `ms encode --in <name>` backs up whatever phrase is in `./-` (a different wallet, exit 0); `ms encode --out
   <name>` writes the card to `./-`, the named file never appears, and the next such run replaces it (exit 0);
   `ms verify --in <name>` reports `OK` for a file that does not exist. The `=` spellings (`--in=<name>`,
   `--out=<name>`) become a bare `-` positional and exit 64. **Fix:** under the override, the value of `--in` and
   `--out` is never classified or rewritten.
2. **bh-ms-C2.** Under the override, `--hashlock-phrase=<value>` admits the value with surrounding whitespace
   trimmed, while `--hashlock-phrase <value>` admits it exactly. `SPEC_ms_hashlock.md` §4.3 (L389-394) says the
   phrase is used "exactly as typed: no trimming". `"--hashlock-phrase=trail "` hashes the five bytes `trail`
   (exit 0) and the holder's recorded phrase `trail ` does not open the hashlock. **Fix:** the `=` spelling admits
   the bytes after the first `=` exactly, for every secret flag, as the space spelling already does.

No new flag, type, command, refusal, printed text or exit code. Every text a holder sees after the fix is an
existing text (quoted in section 2).

## 2. Spec delta (holder-visible behaviour, decided)

The rules apply only when the override is in effect: `--allow-argv-secret` appears anywhere in argv and `argv[1]`
is one of the nine material verbs (`override_applies`, `argv_guard.rs:299-313`, unchanged). Without the override
nothing in this document changes (rule R7).

**R1. `--in FILE` / `--out FILE` (space spelling).** A token at index >= 1 that is byte-for-byte `--in` or `--out`
(no trimming, no case folding: the spelling clap binds) passes through unchanged, and so does the token after it,
whatever it is, with one exception: if the next token is exactly `--allow-argv-secret` it is dropped as it is
everywhere else, and `--in`/`--out` reaches clap without a value. Nothing is added to the admitted side channel for
either token. Consequences, all measured:
- the named file is read or written; `./-` is neither read nor created;
- a named input that does not exist fails with the existing read error, exit 1, nothing on stdout:
  `error: failed to read --in <FILE>: No such file or directory (os error 2)` (`parse.rs:105`; the text the
  same command prints today for `--in ./<FILE>`), on all nine verbs;
- `--in`/`--out` as the last token, or followed by `--allow-argv-secret`: exit 64,
  `error: a value is required for '--in <FILE>' but none was supplied` (clap; unchanged from today);
- `--in -` keeps today's meaning on every verb (a file named `-`; `-` is not material).

**R2. `--in=FILE` / `--out=FILE` (equals spelling).** A token at index >= 1 that begins byte-for-byte with `--in=`
or `--out=` passes through unchanged and is not classified. Same consequences as R1.

**R3. End of options.** Once a token at index >= 1 is exactly `--` (and was not consumed as R1's value), R1 and R2
no longer apply to any later token: clap reads everything after `--` as a positional, so a later `--in` is not a
flag and its neighbour is classified by shape exactly as today. Measured (unchanged):
`ms decode --allow-argv-secret -- --in <32 hex>` exits 64 with `error: unexpected argument '-' found` and the
material is not on stderr.

**R4. No other flag is exempt.** The value-taking non-secret flags of the nine verbs, enumerated from each verb's
`--help` at `9c0f5f3`: `--in`, `--out`, `--kind`, `--method`, `--group-size`, `--separator`, `--template`,
`--account`, `--network`, `--language`, `-k/--threshold`, `-n/--shares`, `--to`. `--in` and `--out` are the only
`PathBuf` arguments (`grep PathBuf crates/ms-cli/src/cmd/*.rs`; `gen-man --out` is a directory on a verb the
override does not apply to). Every other value is an enum, an integer or a `char`, so a material-shaped value is
never a legal value: keeping today's substitution yields clap's exit 64 naming `-`, not the material. Measured
(unchanged): `ms derive --allow-argv-secret --in card.ms1 --account <32 digits>` exits 64 with
`error: invalid value '-' for '--account <ACCOUNT>': invalid digit found in string`. Exempting those flags would put
the value into clap's error text instead.

**R5. Exact flag match.** R1/R2 match the exact bytes only. `--IN <32 hex>`, ` --in <32 hex>` and the like keep
today's shape classification; clap then refuses the unknown token (exit 64, `error: unexpected argument '--IN'
found`), and the material is not on stderr (measured, unchanged).

**R6. `=` spelling of a secret flag (C2).** For `--phrase=`, `--hex=`, `--ms1=`, `--passphrase=` and
`--hashlock-phrase=`, the admitted value is every byte after the first `=` of the token as received, untrimmed:
the same bytes the space spelling admits (`argv_guard.rs:375` pushes `value.clone()`). Measured consequences:
`"--hashlock-phrase=trail "` hashes the six bytes `trail ` (`preimage_hex` `59413e90...9b8bbe70`,
`"phrase_chars":6`), identical to `--hashlock-phrase "trail "`; a leading space was already kept and stays kept;
`--phrase=`, `--hex=` and `--ms1=` with surrounding spaces give byte-identical stdout to their space spellings
before and after (their readers trim); `--passphrase=` already admitted the exact value. The channel test
(`is_channel`) and the refusal path (`find_argv_material`) are unchanged.

**R7. Without the override, nothing changes.** `find_argv_material` and the refusal are untouched. In particular a
32/40/48/56/64-hex file name after `--in`/`--out` is still refused as raw hex entropy (exit 1, `ms: argument 3 on
ARGV ... is raw hex entropy, 32 characters long.`, nothing read or written), and the refusal's last line still
names `--allow-argv-secret`, which after this fix does what it says. (Decided here as out of scope: relaxing the
refusal for file names would make a real hex secret typed after `--in` by mistake pass unrefused.)

**R8. PROPOSED (architect confirms): a material-shaped `--in` value that is not a file.** Under the override the
value of `--in` is a path, whatever it looks like. If a holder types material there by mistake
(`ms encode --allow-argv-secret --in "<12 words>"`), the existing read error names it:
`error: failed to read --in <the 12 words>: No such file or directory (os error 2)`, exit 1 (measured on the
scratch build). Today the same invocation silently reads `./-` (or, with no `./-`, fails naming `-`). This
document keeps the existing error text: the holder has declared argv safe where they are, and the text appears on
the same terminal that already shows the command line. The alternative (redact the path in the read error when it is
material-shaped) would be a new printed text; it is not proposed.

### Spec text to add (`design/SPEC_ms_hashlock.md` §6, after item 6 (ends L548), before the `---` at L550)

The builder adds this paragraph verbatim:

> 7. **Path values are never rewritten (bh-ms-C1).** Under `--allow-argv-secret` the substitution pass leaves the
> value of `--in` and `--out` exactly as typed, in both spellings (`--in FILE`, `--in=FILE`), matched on the exact
> flag bytes and only before a literal `--`. A file name shaped like hex entropy or an ms1 string is a file name.
> Every other flag value keeps the shape test: none of them is a path, and a material-shaped value is never a
> legal value for any of them.
> 8. **The `=` spelling admits what the space spelling admits (bh-ms-C2).** For every secret flag the admitted value
> is the bytes after the first `=`, untrimmed; §4.3's "exactly as typed" holds for `--hashlock-phrase=` too.

## 3. Code delta (production; prose, no code)

All in `crates/ms-cli/src/argv_guard.rs`, function `substitute` (L320-431):

1. Track end-of-options: a local flag, set when a token at index >= 1 is exactly `--` and is reached by the loop
   (not consumed as R1's value).
2. Before the `SECRET_FLAGS` test at L330-331, while the flag is unset and `i > 0`: if the token is exactly `--in`
   or `--out`, push it; if a next token exists and is not exactly `ALLOW_FLAG`, push that token unchanged and
   advance two; otherwise advance one (the loop then drops `ALLOW_FLAG` as it already does at L326-329). If the
   token starts with exactly `--in=` or `--out=`, push it unchanged and advance one. Nothing is inserted into the
   admitted map on either path.
3. In the `=` branch (L389-411), admit `exact` (L395) for every flag; delete `raw` (L392) and the
   `lhs == "--passphrase"` conditional (L397).
4. Doc comments: `substitute`'s header (L315-316, "replace each material token with `-`") gains "except the value
   of `--in`/`--out`, which is a path (bh-ms-C1)"; the `=`-branch comment at L393-394 says the exact value is
   admitted for every flag (bh-ms-C2). No other file changes in production.
5. `CHANGELOG.md` under `## ms-cli [Unreleased]`, PROPOSED (architect confirms) text:

> ### Fixed
> - **`--allow-argv-secret` no longer rewrites `--in` and `--out` file names.** A file name of 32 to 64 hex
>   characters, or one shaped like an ms1 string, was taken for a secret on the command line and replaced by `-`:
>   `--in` then read a file named `-` in the current directory (a backup of a different seed, or `OK` from
>   `ms verify` for a file that does not exist), and `--out` wrote the card to `./-` instead of the named file,
>   exit 0. `--in=FILE` and `--out=FILE` exited 64. The named file is now read or written; a missing one fails
>   with `failed to read --in FILE` (exit 1). Without `--allow-argv-secret` such a name is still refused.
> - **`--hashlock-phrase=TEXT` keeps leading and trailing spaces**, as `--hashlock-phrase TEXT` does
>   (`SPEC_ms_hashlock.md` §4.3, "no trimming"). The `=` spelling used to drop trailing spaces, so the committed
>   hash was for a different phrase than the one written down.

Release and version bump are the coordinator's (not decided here). No flag or schema change, so no mnemonic-gui
mirror change.

## 4. RED rows

The rows are run by `scripts/ms-argv-substitute-fix-gate.sh` (committed with this document): each in a fresh
directory, test vectors only (the all-zero 12- and 15-word BIP-39 phrases, their ms1 cards, a random preimage plate
and a 2-of-3 share pair of the 12-word card). `N` is 32 `a`; `N64` is 64 `b`. Printed commands as run:

    scripts/ms-argv-substitute-fix-gate.sh red   <ms built at 9c0f5f3> /scratch/cache/ms-argv-red/gate-red
    scripts/ms-argv-substitute-fix-gate.sh green <ms built with section 3> /scratch/cache/ms-argv-red/gate-green

Results: red on `9c0f5f3`: `rows failed: 0`, rc 0 (20 RED rows showed their defect, 8 control rows held); green on
the scratch build: `rows failed: 0`, rc 0 (28 rows held). Self-test: green mode on `9c0f5f3` gives `rows failed:
20`, rc 1; red mode on the scratch build gives `rows failed: 20`, rc 1.

| row (test function) | command (after `ms`) | observed at `9c0f5f3` (RED) | expected after the fix |
|---|---|---|---|
| argv_sub_in_space_reads_named_file | `encode --allow-argv-secret --in N --json` (N = 15 words, `./-` = 12 words) | rc 0, `"word_count":12` | rc 0, `"word_count":15` |
| argv_sub_in_equals_reads_named_file | `encode --allow-argv-secret --in=N --json` (same files) | rc 64, `error: unexpected argument '-' found` | rc 0, `"word_count":15` |
| argv_sub_in_64_hex_name_reads_named_file | `inspect --allow-argv-secret --in N64 --json` (N64 = 15-word card, `./-` = 12-word card) | rc 0, 16-byte `payload_bytes_hex` | rc 0, `"payload_bytes_hex":"0000000000000000000000000000000000000000"` |
| argv_sub_in_ms1_shaped_name_reads_named_file | `verify --allow-argv-secret --in <12-word card string>` (that file = 15-word card, `./-` = 12-word card) | rc 0, `OK: valid v0.1 entr (12 words, 50 chars)` | rc 0, `OK: valid v0.1 entr (15 words, 56 chars)` |
| argv_sub_in_missing_named_file_fails (one case per verb: encode, decode, inspect, verify, derive `--network testnet`, hashlock `--kind sha256`, repair, split `-k 2 -n 3`, combine) | `<verb> --allow-argv-secret --in N` (N absent, `./-` = valid input for the verb) | rc 0 on all nine, output derived from `./-` | rc 1, stderr exactly `error: failed to read --in aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: No such file or directory (os error 2)`, stdout empty |
| argv_sub_out_space_creates_named_file (cases: encode, hashlock, repair, split) | `encode --allow-argv-secret --in seed.txt --out N`; `hashlock --allow-argv-secret --random --kind sha256 --no-engraving-card --out N`; `repair --allow-argv-secret --in card.ms1 --out N`; `split --allow-argv-secret -k 2 -n 3 --in seed.txt --out N` | encode, repair, split: rc 0, N absent, `./-` created; hashlock: rc 64, `error: an ms1 string (argument, `-`, or --in FILE) and --random were both given; ...` | rc 0, N exists with mode 0600, no `./-` |
| argv_sub_out_space_card_bytes | content of N from the encode case above | N absent | `ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f` |
| argv_sub_out_equals_creates_named_file | `encode --allow-argv-secret --in seed.txt --out=N` | rc 64, `error: unexpected argument '-' found` | rc 0, N holds the card above, no `./-` |
| argv_sub_hashlock_phrase_equals_keeps_trailing_space | `hashlock --allow-argv-secret "--hashlock-phrase=trail " --method sha256 --json` | rc 0, `preimage_hex` `897abbb1...8a747d987f` (= SHA-256 of `trail`), `"phrase_chars":5` | rc 0, `preimage_hex` `59413e908f3b72e4ce859bf153fffc8054615bc3e445f7ac5e7b2adb9b8bbe70`, `"phrase_chars":6` |

Control rows (hold before and after; they pin R3, R4, R5, R6 and R7):

| row (test function) | command (after `ms`) | expected, both builds |
|---|---|---|
| argv_sub_after_dashdash_still_substituted | `decode --allow-argv-secret -- --in N` | rc 64, `error: unexpected argument '-' found`, N not on stderr |
| argv_sub_flag_match_is_exact | `decode --allow-argv-secret --IN N` | rc 64, `error: unexpected argument '--IN' found`, N not on stderr |
| argv_sub_non_path_value_still_substituted | `derive --allow-argv-secret --in card.ms1 --account 12345678901234567890123456789012` | rc 64, `error: invalid value '-' for '--account <ACCOUNT>'`, the digits not on stderr |
| argv_sub_without_override_still_refuses | `encode --in N --json` (N = 15 words) | rc 1, stderr contains `is raw hex entropy, 32 characters long`, stdout empty |
| argv_sub_hashlock_phrase_equals_keeps_leading_space | `hashlock --allow-argv-secret "--hashlock-phrase= trail" --method sha256 --json` vs `--hashlock-phrase " trail"` | same `preimage_hex` |
| argv_sub_equals_matches_space (cases: phrase, hex, ms1) | `encode ... "--phrase= <12 words> "` vs `--phrase " <12 words> "`; `encode ... "--hex= <32 zeros> "` vs `--hex " <32 zeros> "`; `repair ... "--ms1= <card> "` vs `--ms1 " <card> "`, each with `--allow-argv-secret --json` | rc 0 twice, byte-identical stdout |

## 5. Tests that change

- **Existing tests: none.** The full workspace suite passed unchanged on the scratch build (725 run, 725 passed,
  23 skipped). In particular `hashlock_sources.rs:516-534` (`"--hashlock-phrase=--json"` admitted, digest
  `329367945b...`) still passes, and `allow_argv_secret.rs:193` (`--phrase=<PHRASE>`) still passes.
- **New integration test file** `crates/ms-cli/tests/bh1119_argv_substitute.rs`: one `#[test]` per row of section 4
  (both tables), named exactly as the row; a row with cases is one function that loops over its cases and names the
  failing case in its assertion message. Each runs the built `ms` (`assert_cmd`, as `allow_argv_secret.rs` does) in
  its own `tempfile::tempdir()`, with stdin closed, and asserts the "expected after the fix" column (RED rows) or the
  "expected, both builds" column (controls). The builder runs the file on the base first and records that exactly the 9 RED functions fail and the 6
  control functions pass (15 functions) before writing the code.
- **New unit rows** in `argv_guard.rs` `mod tests`, on the returned argv of `substitute` (the `ADMITTED` once-lock
  makes the side channel untestable per call; the returned argv is not):
  `substitute_keeps_path_values` (input `ms encode --allow-argv-secret --in N --out N64` returns
  `ms encode --in N --out N64`), `substitute_keeps_path_values_equals` (`--in=N --out=N64` unchanged),
  `substitute_path_rule_stops_at_dashdash` (`ms decode --allow-argv-secret -- --in N` returns
  `ms decode -- --in -`), `substitute_in_before_allow_flag_takes_no_value` (`ms encode --in --allow-argv-secret`
  returns `ms encode --in`).

## 6. Toolkit manual lockstep

None. No flag, printed text or exit code that the manual quotes changes. `mnemonic-toolkit` `origin/master`
`cdedd8a2` `docs/manual/src/40-cli-reference/43-ms.md` L36-46 already describes `--in FILE` and `--out FILE` as
file names and `--allow-argv-secret` as "proceeds anyway"; the fix makes the tool match that text. No mnemonic-gui
mirror change (no flag change).

## 7. Coverage map (rule -> rows)

| rule | rows |
|---|---|
| R1 | argv_sub_in_space_reads_named_file, argv_sub_in_64_hex_name_reads_named_file, argv_sub_in_ms1_shaped_name_reads_named_file, argv_sub_in_missing_named_file_fails (9 verbs), argv_sub_out_space_creates_named_file (4 verbs), argv_sub_out_space_card_bytes; unit substitute_keeps_path_values, substitute_in_before_allow_flag_takes_no_value |
| R2 | argv_sub_in_equals_reads_named_file, argv_sub_out_equals_creates_named_file; unit substitute_keeps_path_values_equals |
| R3 | argv_sub_after_dashdash_still_substituted; unit substitute_path_rule_stops_at_dashdash |
| R4 | argv_sub_non_path_value_still_substituted |
| R5 | argv_sub_flag_match_is_exact |
| R6 | argv_sub_hashlock_phrase_equals_keeps_trailing_space, argv_sub_hashlock_phrase_equals_keeps_leading_space, argv_sub_equals_matches_space (3 flags); `--passphrase=` exactness stays pinned by the existing `f687_passphrase_channels.rs` |
| R7 | argv_sub_without_override_still_refuses |
| R8 | none (PROPOSED; a row is added if the architect confirms: `encode --allow-argv-secret --in "<12 words>"` rc 1, stderr `error: failed to read --in <12 words>: No such file or directory (os error 2)`) |

## 8. Gate (builder runs once to a file and greps it)

Rust pin `cargo 1.85.0`, `$HOME/.cargo/bin` first in PATH, `--locked`, no `--release`, worktree-own `target/`:

    cargo nextest run --locked --workspace --build-jobs 6 -j 6 --no-fail-fast
    cargo test --locked --doc --workspace
    cargo clippy --locked --all-targets --workspace -- -D warnings
    cargo +1.95.0 fmt --all -- --check
    bash ci/code-docs-check.sh && bash ci/docs-only.test.sh
    scripts/ms-argv-substitute-fix-gate.sh green target/debug/ms /scratch/cache/ms-argv-gate-green

Expected: the suite count is the base's 725 plus the new rows, all passing; the gate script prints
`rows failed: 0`. Not covered by this gate: the macOS, musl, release, miri, FreeBSD and fuzz CI legs (CI on the PR
head), and review.

## 9. FOLLOWUPS

None appended in the PR. The report lists the lines to direct-push after merge (the two closing lines for C1/C2
and two new Nit/Minor lines found while measuring).
