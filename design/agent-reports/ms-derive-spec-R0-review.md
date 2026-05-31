# R0 Architect Review — SPEC_ms_derive.md (ms-cli 0.5.0)

Reviewer: feature-dev:code-reviewer (opus). Reviewed against real source in
`mnemonic-secret/crates/ms-cli/`, the toolkit derive_slot/template/network/secret_advisory/convert,
the GUI `schema/ms.rs` + pinned-upstream.toml, the manual 43-ms.md + cli-subcommands.list, and bitcoin
0.32 / bip39 2.2 APIs (docs.rs + bip32.rs source).

Core cryptographic design is SOUND: every load-bearing API claim (`to_seed`, `from_entropy_in`,
`parse_in`, `Xpriv::new_master`, `Xpriv::fingerprint`, `Xpriv::derive_priv`, `Xpub::from_priv`)
correct; no-secret-on-stdout boundary cleanly enforceable. But 3 Criticals + 5 Importants.

## Critical

**C1 — §4 test #1 oracle is broken: `convert --to fingerprint` REQUIRES `--template`.** convert.rs:1169
puts `Fingerprint` in `needs_derive`; `:1173` hard-errors "--template is required for derivation
targets" when absent. The oracle `mnemonic convert --from phrase=<x> --to fingerprint` exits 1.
**Fix:** add `--template bip84` to the oracle (fingerprint is template-INVARIANT → a useful
cross-check); explicitly state §3.1 that `ms derive` DELIBERATELY emits the master fingerprint with NO
`--template` required (template only gates the optional account xpub) — don't let an implementer "fix"
it to match toolkit.

**C2 — argv-leak advisory does NOT exist in ms-cli (open Q4 resolved).** Grep of `ms-cli/src` finds no
`secret_in_argv_warning`/argv/cmdline advisory (only `process_hardening` PR_SET_DUMPABLE). encode's
note is a *storage* note, not argv-leak. The helper lives in the TOOLKIT (`secret_advisory.rs:34`),
which ms-cli doesn't dep. Test #4 asserts the advisory → can't pass. **Fix:** specify adding a small
inline-advisory emitter to ms-cli (port the toolkit one-liner: `"warning: secret material on argv
(<flag>) — pipe via <alt> to avoid /proc/$PID/cmdline exposure"`); state which flags trigger
(`--phrase`/`--hex`/`--passphrase` + inline positional `ms1`) + the `<alt>` for each.

**C3 — GUI schema-mirror incomplete: `repair` is ALREADY missing from `mnemonic-gui/src/schema/ms.rs`.**
ms.rs lists only inspect/encode/decode/verify/vectors (lines 210-242), pinned "ms 0.2.1" /
`ms-cli-v0.4.1`. `repair` (live in main.rs, surfaced via clap-reflective gui-schema) was never
mirrored. On the pin bump the `schema_mirror` flag-NAME gate diffs the FULL reflected set
(repair + derive) → fires on BOTH. (The CLAUDE.md `gui-schema-mirror-lockstep-discipline` v0.27.2
case.) **Fix:** §3.2 must backfill `repair` (its `--ms1` + `--json`) ALONGSIDE adding `derive`, same
lockstep PR. (Same pattern the mk-cli v0.6.0 cycle hit.)

## Important
- **I1 — `Derive` insertion + alphabetical ordering.** `Command` (main.rs:68), the dispatch match
  (:150), and `is_json_mode` (:176) are exhaustive — each gains a `Derive` arm. Per CLAUDE.md
  alphabetical-variant convention, insert `Derive` BEFORE `Encode` in all three; `is_json_mode` arm
  returns `a.json`.
- **I2 — §2 CliError quote drift.** Real: `BadInput(String)`, `Bip39(bip39::Error)`,
  `Codex32(codex32::Error)`, `UnexpectedStringLength{got}`, `PayloadLengthMismatch{got,tag}`,
  `FormatViolation{…}`, `FutureFormat{tag}`, `VerifyPhraseMismatch`. `BadInput→exit 1` reuse is correct,
  but there's NO `From<bitcoin::…> for CliError` → wrap via `.map_err(|e| CliError::BadInput(format!
  (...)))` at `new_master`/`derive_priv`. State this.
- **I3 — single-stdin guard precision.** Use `is_stdin_arg` (parse.rs:97; true for None/`-`), mirror
  verify.rs:50. Spec: `entropy_reads_stdin = is_stdin_arg(ms1) || phrase==Some("-") || hex==Some("-")`
  (only the active source); if `--passphrase-stdin` && entropy_reads_stdin → BadInput.
- **I4 — mlock claim overstated.** Only `read_stdin` (parse.rs:65) pins; inline-argv path (parse.rs:22)
  does NOT — inline secrets get `mem::take`→Zeroizing scrub only. Scope the §3.1 hygiene claim
  (stdin-pinned; inline Zeroizing-scrubbed; argv-byte pinning out of scope, consistent w/ encode +
  PR_SET_DUMPABLE). Dovetails with C2 (why inline gets a warning).
- **I5 — manual count stale.** `43-ms.md:4` "Five subcommands" (already 6 documented incl. repair).
  →"Seven" after derive (or drop the brittle count). Confirm `ms derive` added to cli-subcommands.list.

## Minor
- **M1 (open):** `Xpriv::fingerprint`/`derive_priv`/`Xpub::from_priv` need `C: Signing` →
  `Secp256k1::new()` (derive_slot.rs:81/142), NOT `verification_only()` (won't compile). State it.
- **M2 (Q3 CONFIRMED):** master fingerprint network-INDEPENDENT (HASH160 of pubkey, no NetworkKind).
  Test #5 correct.
- **M3 (Q4 CONFIRMED):** `to_seed` = PBKDF2 over language-specific mnemonic string → english≠french
  fingerprint; `--hex` still needs language (entropy→mnemonic→seed). decode "DEFAULT" annotation right.
  (bip39 `unicode-normalization` is default-on; ms-cli doesn't set default-features=false — keep it.)
- **M4:** `--account: u32` `default_value_t = 0` (toolkit parity).
- **M5 (open §11):** no CliTemplate port needed — `ValueEnum {Bip44,Bip49,Bip84,Bip86}` + inline
  `format!("m/{purpose}'/{coin}'/{account}'")` (purpose 44/49/84/86; coin 0 mainnet/1 testnet) →
  `DerivationPath::from_str`. State the maps.
- **M6:** `DeriveJson` in format.rs; `account_path`/`account_xpub` `#[serde(skip_serializing_if =
  "Option::is_none")]` (omit, not null — crate convention).
- **M7:** SemVer 0.4.2→0.5.0 MINOR correct; ms-cli IS on crates.io (max 0.4.2) → publish post-tag valid.
- **M8 (open Q1):** prefer `#[command(group = ArgGroup::new("entropy_src").args(["ms1","phrase","hex"]))]`
  (NOT required — stdin-default to ms1) over per-arg conflicts_with. Pick one explicitly.
- **M9 (open Q2):** `--template` optional (fingerprint-only without it) — confirmed correct.

**VERDICT: RED (3C/5I)**

---

## Fold applied (controller, verified @ e3d5665 + toolkit)
- C1: confirmed convert.rs:1169/1173. §4 oracle → `--template bip84`; §3.1 notes ms derive omits
  --template for the fingerprint by design.
- C2: confirmed no advisory in ms-cli. §3.1/§3.2 specify porting a `secret_in_argv_warning` emitter.
- C3: confirmed ms.rs missing repair. §3.2 backfills repair + adds derive.
- I1-I5 + M1/M4/M5/M6/M8 folded; M2/M3/M7/M9 confirmed (no change).
