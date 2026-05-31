# End-of-cycle R0 review — ms-cli 0.5.0 `ms derive`

Reviewer: feature-dev:code-reviewer (opus). Reviewed the full diff master..HEAD vs the GREEN spec/plan.
Derivation spine correct (Secp256k1::new signing; new_master→fingerprint; derive_priv→from_priv;
coin 0/1; path string agrees both build sites); no-secret-on-stdout holds; fp 73c5da0a + bip84
xpub6CatW… are the attested abandon-seed oracles; main.rs alphabetical wiring; clap/JSON/DEFAULT all
match spec; Cargo.lock synced.

## CRITICAL
**C1 — `--passphrase-stdin` corrupts the passphrase via `read_input`→`strip_whitespace`.** derive.rs:192
reads the stdin passphrase with `read_input(Some("-"))`, which (parse.rs:21/81) strips ALL Unicode
whitespace AND dedups a doubled even-length string (the ms1 encode-pipe heuristic). A multi-word
passphrase `correct horse battery staple` → `correcthorsebatterystaple` (wrong fp, no error); doubled
even strings halved. Worse, it DISAGREES with inline `--passphrase "a b"` (raw argv, verbatim) — a
metamorphic inconsistency, behind the very verification oracle ms derive exists to provide. Suite missed
it (only pipes `TREZOR`, no whitespace). **Fix:** byte-preserving stdin passphrase reader (strip only a
single trailing `\r?\n`, mirror toolkit read_stdin_passphrase) + a multi-word-passphrase regression test
asserting stdin==inline fp.

## IMPORTANT
- **I1 — manual lockstep not done.** `43-ms.md` has no `ms derive` section; `cli-subcommands.list` lacks
  `ms derive`; intro "Five subcommands" stale. CI-gated (manual.yml flag-coverage) — MUST land before tag.
- **I2 — GUI schema-mirror not done.** `mnemonic-gui/src/schema/ms.rs` lacks `derive` AND the pre-existing
  un-mirrored `repair`; pin un-bumped. Land the paired PR (add derive + backfill repair + bump pin)
  before/with the tag (gui-schema-mirror-lockstep-discipline).

## MINOR (non-gating)
- M1 account-path format! built twice (agree; cosmetic). M2 parse_hex_entropy length not BIP-39-set
  (bad len → Bip39 err exit 1, graceful). M3 unreachable! on Payload non_exhaustive (decode.rs parity).

VERDICT: RED (1C/2I)

---
## Fold: C1 = read_stdin_passphrase (byte-preserving) + test; I1 = manual; I2 = GUI (paired); all before tag.
