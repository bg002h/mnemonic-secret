build ms-remedies (direct fix, items E-J; source: the Opus grade `grade-ms-remedies.md`)

Base: origin/master aa25f960b7fc9251ea38ec2596e9812658622cd7 (as required). Branch fix/ms-remedies.
Head: see the final line of the returned summary (this report is the last commit; the code head is 5222cb1235d04bc8b06dab992e0c01eb2db5d21a).
PR: draft, number in the returned summary.

## RED (commit "test: RED rows ...")
Rows installed in `crates/ms-cli/tests/argv_remedy_runs.rs` (the grade's named home; the grade's helper consts
renamed only where they collided: `PHRASE` -> `PHRASE12`; duplicate `HEX` and `ms()` dropped; names unchanged).
`cargo nextest run --locked -p ms-cli --test argv_remedy_runs --no-fail-fast` on the unmodified base:
`29 tests run: 11 passed, 18 failed` = the 25 new rows (18 fail, 7 `k_*` controls pass) + the 4 existing rows pass.
Each failure's reason matches the grade (log /scratch/cache/ms-remedies-logs/red.txt): `ms encode -`, `ms split -`, `ms repair -` rc 64
unexpected argument; `ms split --in FILE`/`--hex -` rc 64 missing --threshold/--shares; `ms verify --in FILE` rc 1 string length;
`ms hashlock <ms1>` rc 1 argument 2 on ARGV; `ms hashlock --random` rc 64 needs --out; `ms encode --in seed.txt` No such file;
`--hex -` on verify/inspect/repair rc 64 unexpected argument '--hex'; decode/combine/verify-hashlock-phrase/encode rc 1.

## GREEN
Grade's `fix.diff` applied unchanged (the report's exact texts equal the diff's; no difference found), plus README lines
18/19/22/25/28/31/41/42/55 and SPEC_ms_hashlock.md :286 (also the stale `:61` -> `:46`) and :503, as §6 gives them.
`cargo nextest run --locked -p ms-cli --test argv_remedy_runs --no-fail-fast`: `29 tests run: 29 passed, 0 skipped`.

## Tests that changed
None existing. The file argv_remedy_runs.rs gained the 25 rows; its 4 old rows are untouched and pass.

## FOLLOWUPS
One `deferred-minor` entry, `ms-derive-phrase-arm-no-one-liner` (the grade's only Minor left unfixed: derive's "no one-liner").
C-20 and D are Minors the grade fixes in the diff, so no entry. The split `--out` overwrite question is item K (architect), not here.

## Gate (logs in /scratch/cache/ms-remedies-logs/, script gate.sh; `--locked` added to each cargo call)
- fmt `cargo +1.95.0 fmt --all -- --check`: rc 0
- ms-codec: preflight rc 0; nextest `210 tests run: 210 passed, 5 skipped`; doc test rc 0; repro by name `test result: ok. 1 passed`; clippy rc 0
- ms-cli: build --tests rc 0; nextest `496 tests run: 496 passed, 18 skipped`; clippy `-D warnings` rc 0
- mlock legs: g2_1, g2_3, g2_4 with `--include-ignored`: each `1 passed`
- miri `cargo +nightly miri test -p ms-cli --bin ms mlock::`: `8 passed; 0 failed; 3 ignored`
- clippy-job extras: docs-only self-test rc 0; shared-ci sha256 + code-docs-check rc 0; vectors checksum pin rc 0
- history-purge `7 passed`; the_decline `6 passed` (both `--include-ignored`; zsh/fish present)
- g6 invariant `2 passed` with SIBLING_REPO_PATH=/scratch/code/shibboleth/mnemonic-toolkit (the local checkout, not CI's pinned sibling ref)
- freebsd `cargo check --target x86_64-unknown-freebsd -p ms-cli`: rc 0
- Extra (not a workflow step): `design/` diff is docs/README/text only.

## NOT RUN
- musl native nextest: `x86_64-linux-musl-gcc` is not installed (cc-rs failure, musl.log); the workflow installs musl-tools via sudo apt.
- musl aarch64 (native arm64 runner, `--release`) and test-release-mlock-einval (`cargo test --release`): not run; `--release` is barred by the rules and there is no arm64 host.
- Toolkit manual job / doc-flag-lint: no flag or manual change (grade §8).
- The workflow's `docs-only.sh` gating step needs CI event context; every changed file set includes code, so none of the skipped-on-docs-only steps is skipped.

## STOPs
None. Observations (not decisions): the grade says 706 workspace tests; this run counted per job, not the whole workspace
(ms-codec 210 + ms-cli 496). The README's `ms verify` example comment (L24, "round-trips against the original phrase") was not in the grade's list and is left as is.

Counts: Critical 0, Important 0 (found by me), Minor 0, STOPs 0.
