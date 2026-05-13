# Changelog

All notable changes to `ms-codec` and `ms-cli` are documented in this file. Each release entry is prefixed with the crate name (`## ms-codec [0.1.0]`, `## ms-cli [0.1.0]`).

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project follows [SemVer](https://semver.org/spec/v2.0.0.html) with the pre-1.0 convention that the second component (`0.X`) is the breaking-change axis.

## ms-cli [0.2.2] — 2026-05-13

v0.9.0 cross-repo Cycle A (OWNED-buffer secret-memory hygiene), Phase E
patch bump for ms-cli. No user-facing API change (no flag additions /
removals; exit codes unchanged; JSON schemas unchanged).

### Added (zeroize discipline; internal-only)

- New `zeroize = "1.8"` dep.
- `EncodeArgs::phrase`, `EncodeArgs::hex`, `VerifyArgs::phrase` clap-field
  rows now consume + immediately wrap: `Zeroizing::new(std::mem::take(...))`
  at `run()` entry, so the clap-resident `String` buffer is scrubbed on
  drop.
- `parse::read_phrase_input` returns `Result<Zeroizing<String>>`;
  `parse::read_stdin` uses `Zeroizing<String>` for its raw read buffer.
- `cmd/encode::run`, `cmd/decode::run`, `cmd/verify::run` use
  `Zeroizing<Vec<u8>>` / `Zeroizing<String>` typed locals for entropy
  and phrase transits. `Payload::Entr` consumer side wraps per the
  ms-codec caller-wrap contract.
- New lint `tests/lint_zeroize_discipline.rs` enumerates 10 ms-cli
  OWNED-buffer rows + per-row evidence anchors.

### Internal (workspace-internal dep bump)

- `ms-codec` exact-pin: `=0.1.2` → `=0.1.3` (companion lockstep release).

### Known third-party residue

- `bip39::Mnemonic` interior buffer is not zeroize-aware
  (FOLLOWUP `rust-bip39-mnemonic-zeroize-upstream`, tier `external`).
  SAFETY-anchor doc-comments at every Mnemonic call site in
  `cmd/encode.rs`, `cmd/decode.rs`, `cmd/verify.rs`.

### Tests

- 10 ms-cli OWNED-buffer rows enumerated in `lint_zeroize_discipline.rs`.
- All pre-existing ms-cli tests green on the rebased Phase 2 work.

## ms-codec [0.1.3] — 2026-05-13

v0.9.0 cross-repo Cycle A (OWNED-buffer secret-memory hygiene), Phase E
patch bump for ms-codec. Cycle SPEC at
`mnemonic-toolkit/design/SPEC_secret_memory_hygiene_v0_9_0.md`; cross-repo
audit matrix at `design/agent-reports/v0_9_0-secret-memory-hygiene-matrix.md`
(sibling) and the toolkit canonical matrix.

### Added (zeroize discipline; no library API change)

- New `zeroize = "1.8"` dev-equivalent dep (in workspace toolchain via
  `ms-cli`).
- Internal `Zeroizing<Vec<u8>>` local-wrap discipline in `envelope::package`,
  `envelope::discriminate`, and `decode::decode`. Drop-time scrub on
  every intermediate `Vec<u8>` that carries `Payload::Entr` bytes.
- `payload.rs` doc-comment block locks the public-API caller-wrap
  contract: callers of `decode()` MUST wrap the returned
  `Payload::Entr(Vec<u8>)` in `Zeroizing::new(...)` to inherit
  drop-time scrub.
- New lint `tests/lint_zeroize_discipline.rs` enumerates 4 ms-codec
  OWNED-buffer rows + their per-row evidence anchors.

### What didn't change

- ms1 wire format unchanged.
- Public API surface unchanged (`Payload::Entr(Vec<u8>)` shape preserved;
  widening to `Zeroizing<Vec<u8>>` is a breaking change deferred per
  SPEC §3 OOS-public-payload — FOLLOWUP `ms-codec-payload-zeroize-public-api`).
- v0.1 → v0.2 migration contract unchanged.

### Known third-party residue

- `codex32::Codex32String` internal buffer is not zeroize-aware
  (FOLLOWUP `rust-codex32-zeroize-upstream`, tier `external`).

### Tests

- 4 OWNED-buffer rows + parametric evidence cells in
  `lint_zeroize_discipline.rs`.
- Existing 59 cells (52 pre-Cycle-A + 7 from v0.8.0 cycle) all green
  on the rebased Phase 2 work.

## ms-cli [0.2.1] — 2026-05-12

### Fixed

- `ms --version` and `ms --help` now exit `0` instead of `64`. The
  v0.2.0 `fn main()` mapped every `Cli::try_parse()` `Err` to
  `ExitCode::from(64)`, but clap returns `Err` for two non-error
  terminations as well — `ErrorKind::DisplayVersion` (`--version`)
  and `ErrorKind::DisplayHelp` (`--help`). The output already
  prints to stdout in those cases; the canonical Unix convention
  is exit 0. The fix branches on `e.kind()` and returns
  `ExitCode::SUCCESS` for the two display variants, preserving the
  SPEC §6 carve-out (exit 64 instead of clap's default 2, so 2
  stays reserved for ms1 format violations) for real parse errors.
  Discovered during `bg002h/mnemonic-gui` v0.2.0 release prep
  (companion: `bg002h/mnemonic-gui`).
- Two new regression cells in `tests/exit_codes_table.rs`:
  `version_flag_exits_zero_and_prints_version` and
  `help_flag_exits_zero_and_prints_help`.
- `cargo fmt` applied to `src/main.rs` — the rustfmt-preferred
  shape for the new `match e.kind()` arm uses a block body when
  the `|` pattern needs to wrap.

## ms-cli [0.2.0] — 2026-05-12

### What's new

- New `ms gui-schema` subcommand emits SPEC §7 JSON describing the CLI's flag surface (subcommand list, flag names, `required`, `kind`, dropdown `choices`, positionals). Consumed by the [`bg002h/mnemonic-gui`](https://github.com/bg002h/mnemonic-gui) schema-mirror CI gate (v0.2 Phase C). Companion: `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `mnemonic-gui-schema-mirror`.
- Implementation walks `clap::CommandFactory::command()` reflection — JSON stays in lockstep with `Cli` automatically; the GUI's mirror gate catches drift.
- Intentionally lossy: complex GUI `FlagKind` variants map to `"text"` upstream and are hand-overridden in the GUI schema file after JSON-bootstrap import. `"boolean"` is produced for `SetTrue` / `SetFalse` / `Count` actions; `"dropdown"` is produced when `Arg::get_possible_values()` is non-empty.

### What didn't change

- All 5 v0.1 subcommands (`encode`, `decode`, `inspect`, `verify`, `vectors`) keep their flag surface, exit codes (0/1/2/3/4/64), and `--json` schemas verbatim.
- Wire format (ms1) is unchanged — `ms-codec` is unaffected at `=0.1.1`.

### Tests

11 new integration tests in `tests/gui_schema_emits_spec_v7_json.rs` covering: exit-0, JSON-parseable, `version == 1`, `cli == "ms"`, `encode`/`decode`/`verify` subcommands present, `encode --phrase` / `--hex` flags, `--language` dropdown with hyphenated `chinese-simplified` / `chinese-traditional` (not `simplifiedchinese`), `--json` boolean kind across subcommands, `vectors --pretty` boolean, `inspect` surface. The v0.1 test surface (77 tests) is preserved.

## ms-cli [0.1.0] — 2026-05-04

### What's new

- Initial release. Companion CLI to ms-codec v0.1.0.
- 5 subcommands: encode, decode, inspect, verify, vectors.
- Phrase-first encode (`--phrase` headline; `--hex` escape hatch); structured `--json` output mode across all commands.
- Strip-whitespace stdin uniform across commands — handles pipe round-trip, engraver-typed-back chunked form, and copy-paste artifacts with one mechanism.
- BIP-39 wordlist enforcement: 10 wordlists supported via `--language` (default `english` with non-suppressible stderr warning surfacing the SPEC §6.3 hazard).
- Exit codes per SPEC §6: 0/1/2/3/4 (verify round-trip mismatch is its own exit code) plus 64 for clap usage errors (overrides clap's default 2 to keep ms1 format violations distinct).
- Engraving-friendly stdout: encode emits `<ms1>\n\n<chunked-form>` (5-char groups, 10/line max, never mid-chunk).
- `verify --phrase` round-trip check: useful for engraver-typed-back proofreading. Phrases never echoed to output (secrets discipline).

### Tests

77 tests across the surface: 29 unit (Phase 1 modules) + 48 integration (`assert_cmd`). cargo build / clippy --all-targets -D warnings / fmt --check all clean.

## ms-codec [0.1.2] — 2026-05-13

v0.8.0 cross-repo BIP-vector adoption cycle, Phase 2. Cycle SPEC at
`mnemonic-toolkit/design/SPEC_test_vector_audit_v0_8_0.md`; per-phase
review at `design/agent-reports/v0_8_0-phase-2-bip93-corpus-r1.md`.

### Added (tests-only; no library API change)

- `tests/bip93_inline_vectors.rs` — full BIP-93 §Test Vectors inline
  corpus pin. 5 valid cells (§93.1–.5: 16-byte / 16-byte / 16-byte /
  32-byte / 64-byte master seeds across k=0 / k=2 / k=3 + long-codex32
  shapes); 1 parametric cell asserting all 64 BIP-93 §Invalid entries
  are rejected by `rust-codex32 =0.1.0`; 1 invariant cell guarding the
  invalid-corpus count.
- `design/agent-reports/v0_8_0-bip-test-vector-audit-matrix.md` — v0.8.0
  successor to the v0.7.1 matrix. Cross-cites the toolkit hub matrix +
  sibling-repo matrices.
- `design/FOLLOWUPS.md` — `bip-vector-adoption-v0_8` (cycle companion)
  and `bip93-invalid-corpus-granular-error-pin` (deferred future
  tightening).

### Corrected

- v0.7.1 audit matrix footnote claimed BIP-93 §Invalid has "42
  strings"; live count via `gh api repos/bitcoin/bips/contents/bip-0093.mediawiki`
  is 64. Source-of-truth corrected at v0.8.0; v0.7.1 matrix carries a
  SUPERSEDED header with forward-pointer.

### What didn't change

- ms1 wire format unchanged.
- Public API surface unchanged.
- v0.1 → v0.2 migration contract unchanged.
- All pre-existing ms-codec tests still pass; +7 cells from this
  cycle → 59 ms-codec total at v0.1.2.

## ms-codec [0.1.0] — 2026-05-03

### What's new

- Initial release. Reference implementation of the **ms1** backup format (HRP `ms`) for BIP-39 entropy.
- Wire format: BIP-93 codex32 used directly via Andrew Poelstra's `rust-codex32 = "=0.1.0"` (CC0). No fork.
- v0.1 payload kind: `entr` (BIP-39 entropy, 16/20/24/28/32 B = BIP-39 word counts {12, 15, 18, 21, 24}).
- v0.1 emitted strings: 50/56/62/69/75 chars (short codex32 checksum only).
- Public API: `encode(Tag, &Payload) -> Result<String>`, `decode(&str) -> Result<(Tag, Payload)>`, `inspect(&str) -> Result<InspectReport>`.
- `Tag::ENTR` const; `Payload::Entr(Vec<u8>)`; `InspectReport` for debugging.
- Decoder applies the full SPEC §4 validity rule set (10 rules); encoder mirrors the reserved-not-emitted-tag rejection (SPEC §3.5.1).
- v0.2 K-of-N share-encoding migration designed up-front via the `0x00` reserved-prefix byte; v0.1 strings remain forward-readable by v0.2 decoders. See [`MIGRATION.md`](MIGRATION.md).
- `Payload`, `PayloadKind`, `Error`, `InspectReport` are `#[non_exhaustive]` from day 1 to allow semver-minor variant additions.
- `Tag` field is private; construction via `try_new` (alphabet-validated) or `from_raw_bytes` (tooling-only).

### What didn't change

(N/A — initial release.)

### Migration notes

(N/A — initial release. See [`MIGRATION.md`](MIGRATION.md) for the planned v0.1 → v0.2 contract.)

### Tests

- 50 tests across all targets: 28 unit + 1 doc-test (Quickstart) + 10 negative + 5 round-trip proptests + 2 forward-compat + 3 BIP-39 integration + 1 vector-corpus replay.
- `cargo build`, `cargo clippy --all-targets -D warnings`, `cargo fmt --check` all clean.

### Wire-format SHA pin

The canonical test vectors at `crates/ms-codec/tests/vectors/v0.1.json` are SHA-256-pinned at this release. Subsequent corpus changes that alter the SHA require a SemVer minor bump per the pre-1.0 breaking-change-axis convention.

```text
sha256(crates/ms-codec/tests/vectors/v0.1.json) = f8d671f543101a4b90fd028126aef66958ff4050e38a32baa48ff298cdf2901a
```

## Unreleased

(none)
