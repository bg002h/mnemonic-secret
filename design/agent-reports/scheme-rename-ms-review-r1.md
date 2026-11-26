# Review r1: `ms derive --template` -> `--origin-path` (commit 7d1e92b)

Reviewed: `git diff origin/master..origin/claude/project-thread-0jg5n7`. The merge base is origin/master `1f8de48`. Checked against `/mnt/project-files/cli-review/scheme-origin-path-plan-2026-11-26.md`.

**Verdict: 0 Critical, 0 Important, 3 Minor.**

## Critical
None.

## Important
None.

## Minor
1. **The hidden alias shows up in clap's typo tip.** Clap's "similar argument" tip still offers the old spelling. It comes from clap, from `crates/ms-cli/src/cmd/derive.rs:60` (`alias = "template"`).
   - Scenario: `ms derive --templat bip84` prints `tip: a similar argument exists: '--template'`.
   - This breaks the plan's rule that printed text names the new spelling. No test covers it.
   - Fix: either accept and record it as a clap limitation, or check whether clap can leave aliases out of its suggestions.
2. **`a_bad_value_exits_the_same_under_either_spelling` asserts weakly** (`tests/derive_origin_path_alias.rs:65-72`). It compares only exit code and stdout. It never checks that stderr names `--origin-path <NAME>`, or that it matches between the two spellings. In practice the stderr is identical (checked by hand).
3. **The alias test only covers part of the flag combinations.** The byte-identity test covers `--hex` with and without `--json`. It does not cover `--template=VALUE`, `--account`, `--network`, `--passphrase-stdin`, `--in` or a positional ms1. I ran all of these by hand (below) and found no difference.

The toolkit manual chapter 43 and the mnemonic-gui `ms.rs` mirror still name `--template`. The plan schedules both in other repos' PRs, so this is a tracking note, not a defect.

## Checked and correct
- `OutputClass::Template` (`advisory.rs:56`) is untouched.
- No `--template` is left in src, README or docs. Only `design/` and the CHANGELOG still mention it.
- The man page from `gen-man` has no "template" left.
- `gui-schema`: the only change is `--template` -> `--origin-path`. The hidden alias is not listed.
- `--help` / `-h`: the alias is hidden.
- JSON keys are unchanged.
- The test diffs outside the new file only change the spelling and rename functions. No assertion was dropped.

## Commands and counts
- `CARGO_BUILD_JOBS=2 cargo test --locked`, log at `/home/user/scratch-ms-review/test.log`: 850 passed, 0 failed, 23 ignored. All 6 tests in `derive_origin_path_alias` pass.
- `cargo clippy --locked --all-targets -- -D warnings`: exit 0.
- `cargo fmt --all --check` (1.85 toolchain): exit 0. The CI fmt job uses 1.95.0, which is not installed here, so I could not run that check.
- I built origin/master's `ms` and compared it with the branch build. I ran the old binary with `--template`, the new one with `--origin-path`, and the new one with `--template`. This covered 30 argument combinations:
  - space and `=` forms
  - `--json`, `--account` (including 2^32-1 and 2^32), `--network testnet`
  - `--passphrase-stdin`, `--passphrase`, `--phrase-stdin`, `--phrase -`, `--hex -`, `--in`, positional ms1, `--`
  - every bip48 variant
  - bad, empty, uppercase, missing and repeated values

**Results:**
- Exit codes match in every case, including 64 for clap errors.
- stdout matches in every case.
- The alias and the new spelling give identical output in every case.
- stderr changes only where it names the flag: the bip48 note, the `--phrase-stdin` recipe, clap's `--origin-path <NAME>` errors, the typo tip and help.

## Fold (controller)

- Minor 2: `a_bad_value_exits_the_same_under_either_spelling` now also asserts stderr.
- Minor 1 (clap's typo tip names the hidden alias): accepted as a clap behaviour; not worth a custom suggester for a deprecated spelling. Applies to every repo in this rename.
- Minor 3: covered by the reviewer's 30-combination old/new binary comparison; no new tests.
