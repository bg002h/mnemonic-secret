# mnemonic-secret

Paths below are relative to this Git root, including when working inside a crate.

## Start here

Read [CLAUDE.md](CLAUDE.md) explicitly; AGENTS.md does not load it automatically.
Read applicable ancestor guidance too. Existing process rules remain applicable;
verify historical architecture/status statements against source.
Read the shared [guide](https://github.com/bg002h/mnemonic-toolkit/blob/docs/constellation-architecture/docs/architecture/README.md) and [workflow](https://github.com/bg002h/mnemonic-toolkit/blob/docs/constellation-architecture/docs/architecture/WORKFLOW.md).
For sibling checkouts/worktrees, prefer local `../mnemonic-toolkit/docs/architecture/`;
remote links support standalone checkouts while documentation branches await integration.
Check Git status, the current branch, Cargo.toml and Cargo.lock before editing.
Preserve unrelated work and use explicit task-owned paths when staging.

## Where behavior lives

- [ms-codec](crates/ms-codec/src/lib.rs) owns payloads, envelope/BCH, shares and hashlock primitives.
- [CLI](crates/ms-cli/src/main.rs) owns mnemonic/passphrase intake, derivation and operator reporting.
- Read [payload types](crates/ms-codec/src/payload.rs), [shares](crates/ms-codec/src/shares.rs),
  [language handling](crates/ms-cli/src/cmd/payload_lang.rs) and [repair](crates/ms-cli/src/cmd/repair.rs)
  before changing recovery. Dispatch by payload kind; a preimage is not mnemonic entropy.
- Preserve the distinction between Entr and Mnem language, separately supplied passphrases,
  and a corrected candidate versus a verified wallet. Do not infer secret-buffer
  scrubbing from the public type or Debug implementation.

## Contracts and checks

Read [base spec](design/SPEC_ms_v0_1.md), [sharing](design/SPEC_ms_v0_2_kofn.md),
[hashlocks](design/SPEC_ms_hashlock.md), and [MIGRATION.md](MIGRATION.md) for the affected contract.
Use codec vector/parity/recovery tests and CLI integration tests under `crates/*/tests/`;
CI definitions and `ci/repro/vendor-freshness.sh` determine current validation.
For API/wire changes, audit consumer locks before promising toolkit/me support.
For CLI surface changes, update the shared toolkit manual and utility GUI mirrors.
