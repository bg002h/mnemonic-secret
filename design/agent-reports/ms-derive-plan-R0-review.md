# R0 Architect Review — IMPLEMENTATION_PLAN_ms_derive.md (ms-cli 0.5.0)

Reviewer: feature-dev:code-reviewer (opus). Verified against real source + bitcoin 0.32.9 / bip39
2.2.2 / clap 4.6.

VERIFIED CORRECT (no action): `default_value_t = Net::Mainnet` on a ValueEnum (silent_payment.rs:51
precedent, no Default derive); positional in ArgGroup (valid clap); all bitcoin 0.32 APIs
(`new_master`/`fingerprint`/`derive_priv`/`from_priv` sigs + map_err on the Result-returning ones;
`Secp256k1::new()` signing); `Fingerprint` Display = LowerHex → `73c5da0a` lowercase (no .to_lowercase
needed); `to_seed → [u8;64]`; the all-zeros→`73c5da0a` MASTER-fp test oracle (no-template emits master
fp); fingerprint network-independence; 3 exhaustive Command sites; GUI repair un-mirrored (C3); manual
"Five" stale (I5); all 9 §4 cells mapped; per-phase TDD + review gates present.

## Critical — None.
## Important
- **I1** — Task 6.2 lists `quickstart.yml` as an ms-cli pin site; it has NONE. The ms pin is only at
  `scripts/install.sh:38` + `manual.yml:88` (both `ms-cli-v0.4.1`). [Folded: Task 6.2 → those 2 sites,
  drop quickstart.]
- **I2** — Task 5.1 CHANGELOG path wrong; no `crates/ms-cli/CHANGELOG.md`. The repo has a top-level
  `/CHANGELOG.md` with crate-prefixed `## ms-cli [X]` entries (latest 0.4.1). [Folded: → `## ms-cli
  [0.5.0]` in `/CHANGELOG.md`.]

## Minor (folded)
- M1 `run(mut args)` (mem::take needs mut). M2 `parse_hex_entropy` is encode-module-private → promote
  pub(crate) + import. M3 fmt: mnemonic-secret has NO fmt gate, edition 2021 (drop "AUTHORITATIVE").
  M4 `use std::str::FromStr` in derive.rs. M5 `DerivationPath::from_str` → map_err not unwrap.

VERDICT: RED (0C/2I) — both citation-accuracy in the lockstep phases; code compiles. Folded → R1.
