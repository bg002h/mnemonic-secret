# Changelog

All notable changes to `ms-codec` and `ms-cli` are documented in this file. Each release entry is prefixed with the crate name (`## ms-codec [0.1.0]`, `## ms-cli [0.1.0]`).

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project follows [SemVer](https://semver.org/spec/v2.0.0.html) with the pre-1.0 convention that the second component (`0.X`) is the breaking-change axis.

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
