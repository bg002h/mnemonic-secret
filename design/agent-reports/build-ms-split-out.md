Build: `ms split --out` refuses an existing file (ruling: coordinator `agent-reports/architect-ms-split-out-refusal.md`). Branch fix/ms-split-out.

Base: origin/fix/ms-remedies 15d24a38f9f0932acb759835a59f4a11ed2faccb (origin/master aa25f960b7fc9251ea38ec2596e9812658622cd7, unmoved).
RED commit 181da4d (rows only). GREEN commit 97d5170ca08e99d5b99a605f8f0756cec542953e. Head = this report's commit (see PR). PR: see below.

## RED
`cargo nextest run --locked -p ms-cli --test split_out_refuses_existing` on the base: `10 tests run: 2 passed, 8 failed, 0 skipped` (log red.txt). 8 failing: existing, empty, dir, dangling symlink, symlink-to-file, /dev/null, json envelope, before-read. 2 pins pass (missing parent, hashlock text).

## GREEN
Same command: `10 tests run: 10 passed, 0 skipped`. Workspace nextest `717 tests run: 717 passed, 23 skipped`. The unit test `cmd::split::tests::write_shares_refuses_a_file_that_appeared_after_the_check` is in the ms-cli run.

## What was applied
The ruling's diff (`split-out.diff`), all hunks clean except CHANGELOG.md and design/FOLLOWUPS.md, re-applied by hand keeping both branches' entries: the CHANGELOG entry is a new `### Changed` section above `### Fixed` under `ms-cli [Unreleased]` (the branch had no Changed section); the FOLLOWUPS entry is appended after the branch's last entry. `cargo fmt --all` made no change.

## Tests that changed
- `tests/f677_help_matches_behaviour.rs`: `if verb == "split" { return; }` before the shares fixture (the ruling's change; the printed help example is unchanged). No other existing test changed.

## Gate (logs /scratch/cache/ms-split-out-logs/, script gate.sh; summary.txt), head 97d5170
- fmt `cargo +1.95.0 fmt --all -- --check`: rc 0
- ms-codec: preflight rc 0; nextest `210 tests run: 210 passed, 5 skipped`; doc `1 passed`; repro by name `1 passed`; clippy rc 0
- ms-cli: build --tests rc 0; nextest `507 tests run: 507 passed, 18 skipped`; clippy `-D warnings` rc 0
- mlock legs g2_1, g2_3, g2_4: each `1 passed`
- miri `mlock::`: `8 passed; 0 failed; 3 ignored`
- docs-only self-test rc 0; shared-ci sha256 + code-docs-check rc 0; vectors pin rc 0
- history_purge `7 passed`; the_decline `6 passed`; g6 `2 passed` (local toolkit checkout as sibling)
- freebsd `cargo check --target x86_64-unknown-freebsd -p ms-cli`: rc 0

## NOT RUN
- musl x86_64 native nextest: `x86_64-linux-musl-gcc` is not installed (rc 101, musl.log); CI installs musl-tools.
- musl aarch64 (native arm64, `--release`) and test-release-mlock-einval (`--release`): barred / no arm64 host.
- Toolkit manual job / doc-flag-lint: the toolkit manual is not touched here (rides the ms pin bump).

## Not in this build
Toolkit manual delta (`43-ms.md:42-46`, `:244`) and GUI help text/manual page; recorded in the cross-repo FOLLOWUPS entry. No version bump, no tag.

## STOPs
None. (The prompt's "8 failing + 2 pins + 1 unit row" matches: the unit row lives in the lib test run.)

Counts: Critical 0, Important 0, Minor 0, Nit 0
