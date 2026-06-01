# ms-cli

Companion CLI to the [`ms-codec`](https://crates.io/crates/ms-codec) library — encode BIP-39 entropy as `ms1` strings for steel-plate engraving, decode/inspect/verify the engraved strings, dump the SHA-pinned test-vector corpus, and BCH error-correct a damaged ms1 string.

6 commands: `encode`, `decode`, `inspect`, `verify`, `vectors`, `repair` (v0.4.0+).

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

## `ms repair` — BCH error correction (v0.4.0)

```bash
# Repair a corrupted ms1 string (up to 4 single-character substitutions
# via BCH(93,80,8) t=4 capacity).
ms repair --ms1 ms10entrsqqqqqqqqqqqzqqqqqqqqqqqqqqqqcj9sxraq34v7f
# stdout:
#   # Repair report
#   #   ms1 chunk 0: 1 correction at position 17: 'z' -> 'q'
#   ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f   # corrected on last line
# stderr:
#   warning: stdout carries private key material (can spend) — redirect or encrypt

# Stdin via `-`:
ms repair --ms1 - < broken.txt

# JSON envelope (cross-CLI parser reuse: byte-matches
# `mnemonic repair --json` / `mk repair --json` / `md repair --json`):
ms repair --ms1 ms10entrsqq... --json
```

| Exit | Meaning |
|---|---|
| `0` | input already valid; no correction applied; echoed unchanged. |
| `5` | `REPAIR_APPLIED` — at least one substitution corrected; stdout = repair report + corrected string. Consistent across all four CLIs per plan D26 (`mnemonic` / `mk` / `ms` / `md`). |
| `2` | unrepairable (more than 4 substitution errors, or structural ms1 error before correction could run). |
| `1` | I/O error or other generic failure. |

`ms repair` wraps `ms_codec::decode_with_correction` (ms-codec v0.2.0+)
and shares the `RepairJson` envelope schema byte-exact with the other
three CLIs per plan D27 (cross-CLI parser reuse). Single-chunk by
codex32-spec design (HRP `ms` is always single-string `BCH(93,80,8)`);
no `--hrp` flag.

The D9 secret-on-stdout advisory ("`warning: secret material on
stdout — consider redirecting`") fires when the corrected output
carries the ms1 (which encodes BIP-39 entropy). Pipe to a file or to
an encryption tool to avoid scrollback exposure.

`ms repair` is the per-codec sibling of toolkit's `mnemonic repair`
(see `mnemonic-toolkit/docs/manual/src/40-cli-reference/41-mnemonic.md`
`## mnemonic repair`).

## Documentation

- [SPEC](https://github.com/bg002h/mnemonic-secret/blob/master/design/SPEC_ms_cli_v0_1.md) — full CLI surface specification.
- [`ms-codec`](https://crates.io/crates/ms-codec) — the underlying library.

## License

MIT License.
