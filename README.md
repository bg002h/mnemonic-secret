# mnemonic-secret (`ms1`)

Reference implementation of the **ms1** backup format — BIP-93 codex32 directly applied to **secret material** (BIP-39 entropy, BIP-32 master seed, xpriv) for steel-engravable backups.

ms1 is the third sibling in the m-format family:

- [`md1`](https://github.com/bg002h/descriptor-mnemonic) — wallet descriptors / templates (HRP `md`)
- [`mk1`](https://github.com/bg002h/mnemonic-key) — single xpubs (HRP `mk`)
- **`ms1`** — secret material (HRP `ms`) — *this repo*

The three formats engrave together as a coherent backup bundle: md1 = template, mk1 = xpubs, ms1 = secret. v0.1 is single-string (BIP-93 threshold = 0); K-of-N share encoding is planned in v0.2.

## Status

Pre-v0.1. Wire format and public API specified in [`design/SPEC_ms_v0_1.md`](design/SPEC_ms_v0_1.md). Phase-by-phase implementation tracked in [`design/IMPLEMENTATION_PLAN_ms_v0_1.md`](design/IMPLEMENTATION_PLAN_ms_v0_1.md). Brainstorm rationale in [`design/BRAINSTORM_ms_v0_1.md`](design/BRAINSTORM_ms_v0_1.md).

## License

CC0-1.0 (matches the sibling repos).
