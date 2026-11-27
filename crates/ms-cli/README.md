# ms-cli

Companion CLI to the [`ms-codec`](https://crates.io/crates/ms-codec) library — encode BIP-39 entropy as `ms1` strings for steel-plate engraving, decode/inspect/verify the engraved strings, dump the SHA-pinned test-vector corpus, and BCH error-correct a damaged ms1 string.

`ms` keeps `encode`, `decode` and the ms1-only verbs `hashlock`, `split` and `combine`. Verbs that several formats share belong to the toolkit (`mnemonic`). `repair` moves first: `ms repair` is planned for removal, and `mnemonic repair --ms1 -` (paste the string, press Enter, then Ctrl-D) repairs a seed ms1 string today. `mnemonic repair` does not read a hashlock preimage plate (`ms10hash…`) yet: repair one with `ms repair --ms1 -`, the same way. `inspect`, `verify` and `derive` are candidates to move later. `vectors`, `gui-schema` and `gen-man` describe `ms` itself and stay. Full table: [Which tool owns which verb](https://github.com/bg002h/mnemonic-toolkit/blob/master/docs/manual/src/50-comparing/51-format-decision.md#which-tool-owns-which-verb).

## Installation

```bash
cargo install --locked --git https://github.com/bg002h/mnemonic-secret --tag ms-cli-v0.21.0 ms-cli
```

This builds the tagged release from source. The crates.io copy (`cargo install ms-cli`) is ms-cli 0.14.0, which lacks every fix since; do not use it.

The installed binary is named `ms`.

## Quickstart

```bash
# Write your 12-word phrase into seed.txt first, with a text editor: a phrase typed on a command line is refused, and your shell would keep it in history.

# Encode the phrase held in seed.txt into the card file card.ms1.
ms encode --phrase - --out card.ms1 < seed.txt

# Recover from an engraved string.
ms decode --in card.ms1

# Verify an engraved string round-trips against the original phrase.
ms verify --in card.ms1 --phrase - < seed.txt

# Inspect a candidate string for structural validity.
ms inspect --in card.ms1
```

A secret on the command line (phrase, hex or ms1) is refused, because the shell has already written it to history; every command reads it from a file (`--in FILE`) or stdin, and takes `--json` for tooling.

## Engraving caveat

`ms1` v0.1 does NOT carry the BIP-39 wordlist language on the wire. Users with non-English wallets MUST record their wordlist language alongside the engraved card. Decode-time `--language english` is the default; the CLI emits a non-suppressible stderr warning when defaulting. See the [SPEC §6.3](https://github.com/bg002h/mnemonic-secret/blob/master/design/SPEC_ms_cli_v0_1.md) for the full hazard discussion.

## `ms repair` — BCH error correction (v0.4.0)

```bash
# Repair a corrupted ms1 string (up to 4 single-character substitutions
# via BCH(93,80,8) t=4 capacity). broken.txt holds ms10entrsqqqqqqqqqqqzqqqqqqqqqqqqqqqqcj9sxraq34v7f.
ms repair --in broken.txt
# stdout:
#   # Repair report
#   #   ms1 chunk 0: 1 correction at position 17: 'z' -> 'q'
#   ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f   # corrected on last line
# stderr:
#   warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
#   repair: correction UNVERIFIED — a corrected seed card cannot be self-verified; confirm the derived address/xpub against a known-good copy before use; BIP-93 recommends confirming a corrected codex32 string
# exit status: 20 (16 a correction was applied + 4 verify before trusting)

# Stdin via `-`:
ms repair --ms1 - < broken.txt

# JSON envelope (cross-CLI parser reuse: byte-matches
# `mnemonic repair --json` / `mk repair --json` / `md repair --json`):
ms repair --in broken.txt --json
```

The codes are the constellation's (see the legend at the end of `ms --help`):
1 to 63 are sums of 1 a check said no, 2 an input is invalid, 4 verify before
trusting, 8 refused, 16 a correction was applied, 32 a file or device failed;
64 is a bad command line and 101 an internal error.

| Exit | Meaning |
|---|---|
| `0` | input already valid; no correction applied; echoed unchanged. |
| `20` | **`VERIFY-ME` Candidate** (16 + 4; Cycle F, `ms1-repair-demote-to-candidate` — a funds fix) — at least one substitution corrected; stdout = repair report + corrected string; stderr carries an UNVERIFIED advisory. A single ms1 has no oracle, so 16 never comes without 4. |
| `22` | a correction was applied, but the corrected string is not an ms1 card this version decodes (20 + 2): it is printed with the warning, and `--out` is not written. |
| `2` | unrepairable (BCH-uncorrectable: > 4 substitution errors, a structural ms1 error before correction could run, or a clean string that is not a card), or empty input. Nothing is printed. |
| `8` | refused: secret on argv without `--allow-argv-secret`, or `--out` is the file stdout is on. |
| `32` | a file failed: `--in` unreadable, `--out` unwritable, or stdout could not be written (added to the other bits: a lost report on a candidate is 52). |
| `64` | bad command line. |

**Confirm every exit-20 card before use.** Up to 4 substitutions are corrected back to the original string; beyond that a correction can still succeed and land on a different, valid card, and an ms1 string carries nothing else to catch it. Confirm the corrected card against an independent record: for a seed card, the derived address or xpub against a known-good copy; for a hashlock preimage card (`ms10hash…`), its hash against the hashlock it was made for.

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
