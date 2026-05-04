# ms-cli

Companion CLI to the [`ms-codec`](https://crates.io/crates/ms-codec) library — encode BIP-39 entropy as `ms1` strings for steel-plate engraving, decode/inspect/verify the engraved strings, and dump the SHA-pinned test-vector corpus.

5 commands: `encode`, `decode`, `inspect`, `verify`, `vectors`.

## Installation

```bash
cargo install ms-cli
```

The installed binary is named `ms`.

## Quickstart

```bash
# Encode a 12-word BIP-39 mnemonic.
ms encode --phrase "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"

# Recover from an engraved string.
ms decode ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f

# Verify an engraved string round-trips against the original phrase.
ms verify ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f --phrase "abandon abandon ... about"

# Inspect a candidate string for structural validity.
ms inspect ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f
```

All commands support stdin input (`-` or omitted positional) and `--json` for tooling.

## Engraving caveat

`ms1` v0.1 does NOT carry the BIP-39 wordlist language on the wire. Users with non-English wallets MUST record their wordlist language alongside the engraved card. Decode-time `--language english` is the default; the CLI emits a non-suppressible stderr warning when defaulting. See the [SPEC §6.3](https://github.com/bg002h/mnemonic-secret/blob/master/design/SPEC_ms_cli_v0_1.md) for the full hazard discussion.

## Documentation

- [SPEC](https://github.com/bg002h/mnemonic-secret/blob/master/design/SPEC_ms_cli_v0_1.md) — full CLI surface specification.
- [`ms-codec`](https://crates.io/crates/ms-codec) — the underlying library.

## License

CC0 1.0 Universal.
