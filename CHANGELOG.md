# Changelog

All notable changes to `ms-codec` (and future `ms-cli`) are documented in this file. Each release entry is prefixed with the crate name (`## ms-codec [0.1.0]`).

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project follows [SemVer](https://semver.org/spec/v2.0.0.html) with the pre-1.0 convention that the second component (`0.X`) is the breaking-change axis.

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
