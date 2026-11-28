# ms-cli

Companion CLI to the [`ms-codec`](https://crates.io/crates/ms-codec) library — encode BIP-39 entropy as `ms1` strings for steel-plate engraving, decode/inspect/verify the engraved strings, split a secret into K-of-N shares and combine them, back up a hashlock preimage, dump the SHA-pinned test-vector corpus, and BCH error-correct a damaged ms1 string.

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

A card encoded from `--hex` or from an English phrase (kind `entr`) does NOT carry the BIP-39 wordlist language. A card encoded from a non-English phrase (`ms encode --phrase - --language japanese`, say) is kind `mnem` and carries the language on the wire; `ms decode` reads it from the card, and a `--language` that disagrees is ignored with a note. For an `entr` card, `ms decode` defaults to `--language english` and says so on stdout (`language: english (12 words, default — verify against your records)`) and stderr. Record your wordlist language alongside the card either way. The BIP-39 passphrase is never on the card. The hazard is discussed in [`design/SPEC_ms_v0_1.md` §6.3](https://github.com/bg002h/mnemonic-secret/blob/master/design/SPEC_ms_v0_1.md) and [`design/SPEC_ms_mnem_wordlist_language.md`](https://github.com/bg002h/mnemonic-secret/blob/master/design/SPEC_ms_mnem_wordlist_language.md).

## K-of-N shares and hashlock preimages

```bash
# Split hex entropy into 3 shares, any 2 of which recover it. Each run draws a
# new share set; --out must not exist yet.
ms split --threshold 2 --shares 3 --hex - --out shares.txt < hex.txt

# Recombine: a file holding any 2 of the 3 share strings, one per line.
ms combine --in two-shares.txt

# A random hashlock preimage: the 32-byte secret goes to plate.ms1 (an
# ms10hash... plate string); stdout gets only the public `hash:` digest record.
ms hashlock --random --out plate.ms1

# Re-derive the digest from the plate.
ms hashlock --in plate.ms1
```

`ms split --phrase -` takes a phrase on stdin instead; a non-English phrase splits as a `mnem` share set, so the language survives. The whole share set is as secret as the seed. A hashlock preimage is not a seed: `ms decode` prints it as kind, hex and digest, never as words.

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
# exit status: 4

# Stdin via `-`:
ms repair --ms1 - < broken.txt

# JSON envelope (the D27 `RepairJson` shape of `mnemonic repair --json` /
# `mk repair --json` / `md repair --json`, plus a "verdict": "candidate"
# field that md's envelope does not carry):
ms repair --in broken.txt --json
```

| Exit | Meaning |
|---|---|
| `0` | input already valid; no correction applied; echoed unchanged. |
| `4` | **`VERIFY-ME` Candidate** (Cycle F, `ms1-repair-demote-to-candidate` — a funds fix) — at least one substitution corrected; stdout = repair report + corrected string; stderr carries an UNVERIFIED advisory. **Never exit 5** — see below. |
| `2` | unrepairable (BCH-uncorrectable: > 4 substitution errors, or structural ms1 error before correction could run). |
| `1` | I/O error or other generic failure. |

**Confirm every exit-4 card before use.** Up to 4 substitutions are corrected back to the original string; beyond that a correction can still succeed and land on a different, valid card, and an ms1 string carries nothing else to catch it. Confirm the corrected card against an independent record: for a seed card, the derived address or xpub against a known-good copy; for a hashlock preimage card (`ms10hash…`), its hash against the hashlock it was made for. `ms repair` never exits 5.

`ms repair` wraps `ms_codec::decode_with_correction` (ms-codec v0.2.0+)
and shares the `RepairJson` envelope schema byte-exact with the other
three CLIs per plan D27 (cross-CLI parser reuse). Single-chunk by
codex32-spec design (HRP `ms` is always single-string `BCH(93,80,8)`);
no `--hrp` flag.

The D9 secret-on-stdout advisory ("`warning: stdout carries private
key material (can spend) — redirect or encrypt (…)`") fires when the
corrected output carries the ms1 (which encodes the secret). Pipe to a file or to
an encryption tool to avoid scrollback exposure.

`ms repair` is the per-codec sibling of toolkit's `mnemonic repair`
(see `mnemonic-toolkit/docs/manual/src/40-cli-reference/41-mnemonic.md`
`## mnemonic repair`).

## Documentation

- [SPEC](https://github.com/bg002h/mnemonic-secret/blob/master/design/SPEC_ms_cli_v0_1.md) — full CLI surface specification.
- [`ms-codec`](https://crates.io/crates/ms-codec) — the underlying library.

## License

Dual-licensed, at your option, under the MIT License or the Unlicense (SPDX `MIT OR Unlicense`).
