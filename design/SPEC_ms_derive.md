# SPEC — `ms derive` (read-only public derivation: master fingerprint + account xpub)

**Repo:** mnemonic-secret (ms-cli). **Branch:** `ms-derive-subcommand` off `master` @ `e3d5665`.
**SemVer:** ms-cli **0.4.2 → 0.5.0** (MINOR — new subcommand). ms-codec unchanged (**0.2.1** — no codec change).
**New dep:** `bitcoin = "0.32"` added to `crates/ms-cli/Cargo.toml` (ms-cli currently has `bip39 = "2"` but no `bitcoin`).

---

## §1. Context & motivation

Theme B piece #3 of the constellation feature survey ("see it after you recover it"). `ms` recovers the BIP-39 entropy from an ms1 card (`ms decode`) but **cannot produce the master fingerprint** — the cheapest "did I recover the RIGHT seed?" verification oracle (ms-cli's own btcrecover help footer points users elsewhere for any derivation). This SPEC adds **`ms derive`**: from an ms1 card (or `--hex`/`--phrase`) emit the **master fingerprint** (always) and, with `--template`, an **account xpub** for watch-only setup.

**Firm product boundary (user-set):** read-only **PUBLIC** derivation ONLY — master fingerprint (4 bytes) + account xpub. **No master seed / root xprv / private keys on stdout, no signing.** (A user wanting the xprv uses the toolkit's `mnemonic convert`; `ms` stays public-derivation-only.) Mirrors the just-shipped `mk derive`/`mnemonic addresses` read-only surfaces.

**Language is load-bearing here.** seed = `PBKDF2(language-specific mnemonic string, "mnemonic"+passphrase)`, so the entropy → fingerprint/xpub derivation depends on the BIP-39 **wordlist language**. `ms derive` therefore carries `ms decode`'s "DEFAULT" language annotation (SPEC §6.3 precedent) when `--language` is omitted — the same non-English footgun surface ms decode already annotates. (No v0.37.11-style "language-losing emit" advisory: the fingerprint/xpub are DERIVED keys, language already applied.)

---

## §2. Source ground truth (verified @ `e3d5665`)

- **`crates/ms-cli/src/main.rs:68`** — `enum Command { Encode, Decode, Inspect, Verify, Vectors, GuiSchema, Repair }`; dispatch `match` (`:150`) `cmd::X::run(args)` (args BY VALUE, `-> Result<u8>`); `is_json_mode` (`:176`) reads `a.json`. Clap parse errors → `ExitCode::from(64)` (`:143`); help/version → SUCCESS. **I1: insert `Derive(cmd::derive::DeriveArgs)` ALPHABETICALLY (before `Encode`) in all THREE exhaustive sites** — `Command`, the dispatch match, and `is_json_mode` (its arm returns `a.json`) — per CLAUDE.md alphabetical-variant convention.
- **`crates/ms-cli/src/cmd/decode.rs:36`** — `run` pattern: `read_input(args.ms1.as_deref())?` (stdin if `None`/`-`) → `ms_codec::decode(&ms1)` → `(_tag, Payload::Entr(b))` → `Zeroizing<Vec<u8>>` entropy → `Mnemonic::from_entropy_in(lang, &entropy)`. Default-language: `(CliLanguage::English, defaulted=true)` when `--language` absent, annotated "DEFAULT" on stdout+stderr.
- **`crates/ms-cli/src/cmd/encode.rs:26`** — `#[command(group = clap::ArgGroup::new("input").required(true).args(["phrase","hex"]))]`; `mem::take` secret `Option<String>` fields into `Zeroizing<String>` at run entry; `parse_hex_entropy`, `read_phrase_input`, `Mnemonic::parse_in(lang, phrase).to_entropy()`.
- **`crates/ms-cli/src/error.rs:14`** — `pub enum CliError { BadInput(String), Bip39(bip39::Error), Codex32(codex32::Error), UnexpectedStringLength{got}, PayloadLengthMismatch{got,tag}, FormatViolation{…}, FutureFormat{tag}, VerifyPhraseMismatch }`. `BadInput(String) → exit 1` (`:43-49`, the usage-error catch-all). **Reuse `BadInput`** for new conditions (bad network, dual-stdin, bitcoin derivation error) — NO new variant. **I2: there is NO `From<bitcoin::bip32::Error> for CliError`** → wrap bitcoin/bip32 errors via `.map_err(|e| CliError::BadInput(format!("…: {e}")))` (exit 1) at each `?` site (`new_master`, `derive_priv`).
- **`crates/ms-cli/src/format.rs:48`** — `*Json` structs are serde with `schema_version: &'static str` ("1"). Add a `DeriveJson` struct.
- **`crates/ms-cli/src/language.rs:43`** — `impl From<CliLanguage> for bip39::Language` (`.into()`); `as_str()` (`:27`).
- **`crates/ms-cli/src/parse.rs:21`** — `read_input(Option<&str>) -> Result<String>` (stdin via `-`/None); `read_stdin` mlock-pins (`:65`). `crates/ms-cli/src/mlock.rs` present.
- **`bip39 = "2"`** present; **`bitcoin` absent** → add `bitcoin = "0.32"`. Derivation spine to mirror: toolkit `derive_slot.rs` (`Xpriv::new_master(network_kind, &seed)` → `.fingerprint(&secp)` → `.derive_priv(&secp, path)` → `Xpub::from_priv(&secp, &acct_xpriv)`).

---

## §3. Design

### 3.1 `ms derive`

```
ms derive [<MS1>] [--hex <HEX> | --phrase <WORDS>] [--template <bip44|bip49|bip84|bip86>]
          [--account <N>] [--network <mainnet|testnet>] [--passphrase <V> | --passphrase-stdin]
          [--language <L>] [--json]
```

- **Input (at most one entropy source; M8):** declare `#[command(group = clap::ArgGroup::new("entropy_src").args(["ms1","phrase","hex"]))]` (NOT `.required(true)` — the default is `ms1`-from-stdin). clap enforces at-most-one; with none, `read_input(None)` reads `ms1` from stdin.
  - positional `ms1: Option<String>` (stdin if `-`/omitted, like `ms decode`) → `ms_codec::decode` → `Payload::Entr` entropy. **DEFAULT input.**
  - `--hex <HEX>` → `parse_hex_entropy` (16/20/24/28/32 B).
  - `--phrase <WORDS>` → `Mnemonic::parse_in(lang, …).to_entropy()`.
  - Secret args (`--hex`/`--phrase`/`--passphrase`) `mem::take`→`Zeroizing` at run entry (encode precedent); entropy `Zeroizing<Vec<u8>>`.
- **`--language <L>`** — clap `Option<CliLanguage>` **WITHOUT** a `default_value` (mirror `decode.rs`, NOT encode's eager `default_value="english"` which would erase the defaulted signal); resolve `(English, defaulted=true)` when `None`, with the "DEFAULT" stdout+stderr annotation (§1). Load-bearing: it forms the mnemonic string → seed → fingerprint/xpub. For `--hex` input it still selects the wordlist for the entropy→mnemonic→seed step.
- **`--passphrase <V>` / `--passphrase-stdin`** — BIP-39 passphrase (ms-cli's FIRST passphrase flag). `mem::take`→`Zeroizing`.
  - **C2 — argv-leak advisory (NEW in ms-cli):** ms-cli has no inline-secret advisory today. Port the toolkit's one-liner — a new `crate::advisory::secret_in_argv_warning(stderr, flag, alt)` (or inline `writeln!`) emitting the exact byte-shape `"warning: secret material on argv (<flag>) — pipe via <alt> to avoid /proc/$PID/cmdline exposure"`. Fires for each inline secret: `--phrase` (alt `--phrase -`), `--hex` (alt `--hex -`), `--passphrase` (alt `--passphrase-stdin`), and the positional `ms1` when given inline (alt `-`). Mirror the toolkit's emit-for-every-inline-secret scope.
  - **I3 — single-stdin guard:** `entropy_reads_stdin = is_stdin_arg(args.ms1.as_deref()) || args.phrase.as_deref()==Some("-") || args.hex.as_deref()==Some("-")` (only the active source — `is_stdin_arg` at parse.rs:97 is true for `None`/`-`; mirror verify.rs:50). If `--passphrase-stdin` AND `entropy_reads_stdin` → `BadInput("cannot read both the entropy source and --passphrase from stdin")`. `--passphrase-stdin` reads via the mlock-pinned `read_stdin` path.
- **Derivation (bitcoin 0.32):** `let secp = Secp256k1::new();` — **M1: full SIGNING context required** (`Xpriv::fingerprint`/`derive_priv` + `Xpub::from_priv` are `C: Signing`; `verification_only()` won't compile; mirror derive_slot.rs:81).
  - `mnemonic = Mnemonic::from_entropy_in(lang, &entropy)` (ms1/hex) or the parsed phrase (`--phrase`).
  - `seed = mnemonic.to_seed(&passphrase)` (`Zeroizing`-pinned 64 B).
  - `master = Xpriv::new_master(network.network_kind(), &seed).map_err(BadInput)?`; `master_fingerprint = master.fingerprint(&secp)` (PUBLIC; **C1: emitted with NO `--template` required** — `ms derive` deliberately diverges from toolkit-convert here, which requires `--template` even for `--to fingerprint`; the fingerprint is template-invariant).
  - **If `--template`:** `--template` is a `ValueEnum {Bip44,Bip49,Bip84,Bip86}` (M5 — no CliTemplate port). `account_path = format!("m/{purpose}'/{coin}'/{account}'")` (purpose 44/49/84/86; coin 0 mainnet / 1 testnet; account `--account` u32 default 0) → `DerivationPath::from_str`; `account_xpub = Xpub::from_priv(&secp, &master.derive_priv(&secp, &account_path).map_err(BadInput)?)` (PUBLIC; `xpub`/`tpub` per network).
- **Network:** `--network mainnet|testnet` (default mainnet). Affects `account_xpub` serialization (`xpub`/`tpub`) + coin-type. (signet/regtest share testnet xpub bytes — out of scope for v1; ms is network-light.)
- **Output (text):** `master_fingerprint: <8 hex>` always; when `--template`: also `account_path:` + `account_xpub:` (aligned-label block, decode style). NO seed/xprv lines.
- **Output (`--json`):** `DeriveJson` (new struct in `format.rs` next to `DecodeJson`) `{ schema_version:"1", master_fingerprint, network, account_path?, account_xpub?, language, language_defaulted }`; `account_path`/`account_xpub` are `Option` with `#[serde(skip_serializing_if = "Option::is_none")]` (M6 — OMITTED, not `null`, without `--template`; crate convention).
- **Secret hygiene (I4):** entropy/seed `Zeroizing`-scrubbed; the **stdin route** is mlock-pinned via `read_stdin` (parse.rs:65), but **inline-argv secrets** (`--hex`/`--phrase`/`--passphrase`, positional `ms1`) are `mem::take`→`Zeroizing` scrubbed only — argv-byte page-pinning is out of scope (consistent with encode/decode/verify + the `PR_SET_DUMPABLE` argv-hardening in `main`; this is exactly why inline secrets get the C2 advisory). `Xpriv` has no Drop+Zeroize (SAFETY comment, bounded lifetime — mirror derive_slot). Only PUBLIC fingerprint+xpub reach stdout.

### 3.2 SemVer + lockstep

- **ms-cli 0.4.2 → 0.5.0** (MINOR). ms-codec stays 0.2.1. `Cargo.lock` re-resolve.
- **Manual** — `mnemonic-toolkit/docs/manual/src/40-cli-reference/43-ms.md` (toolkit repo): new `ms derive` section (every flag) + add `ms derive` to `docs/manual/tests/cli-subcommands.list`; **I5: fix the stale intro count `43-ms.md:4` "Five subcommands"** (already 6 documented incl. repair → "Seven" with derive, or drop the brittle count). CLAUDE.md mirror invariant; `manual.yml` flag-coverage gate.
- **GUI** — `mnemonic-gui/src/schema/ms.rs` (paired PR): **C3 — `repair` is ALREADY un-mirrored** (ms.rs lists only inspect/encode/decode/verify/vectors; `repair` shipped in ms-cli but was never added). The `ms gui-schema` is clap-reflective, so the `schema_mirror` flag-NAME gate diffs the FULL set on the pin bump → fires on `repair` AND `derive`. So this PR MUST: (a) add the `derive` `SubcommandSchema` (`--hex`/`--phrase`/`--template`/`--account`/`--network`/`--passphrase`/`--passphrase-stdin`/`--language`/`--json` + `ms1` positional); (b) **backfill `repair`** (`--ms1` + `--json`); (c) bump `pinned_version` "ms 0.2.1"→"ms 0.5.0" + `pinned-upstream.toml` `ms-cli-v0.4.1`→`ms-cli-v0.5.0`. (Same pre-existing-drift absorption the mk-cli v0.6.0 cycle hit; cite `gui-schema-mirror-lockstep-discipline`.)
- **Toolkit sibling-pin** — `mnemonic-toolkit` `install.sh` + `.github/workflows/manual.yml` + `quickstart.yml` ms-cli pin → `ms-cli-v0.5.0` (sibling-pin-check gate).
- **crates.io** — ms-cli IS on crates.io (prior cycles published); `cargo publish` after tag (ms-codec 0.2.1 + bitcoin 0.32 resolve).

---

## §4. Test plan (per-phase TDD)

1. **Fingerprint correctness** — `ms derive <ms1-card>` → master_fingerprint matches an independent oracle. **C1: the toolkit oracle requires `--template`** — `mnemonic convert --from phrase=<same> --to fingerprint --template bip84` (the fingerprint is template-INVARIANT, so any template works — itself a cross-check) — OR a known BIP-39 vector (abandon×11-about all-zeros → `73c5da0a`). `--hex`/`--phrase` parity with the ms1 of the same entropy.
2. **Account xpub** — `--template bip84 --account 0` → account_xpub matches `mnemonic convert --from phrase=<same> --to xpub --template bip84 --network mainnet`; bip44/49/86 parity; `--account 1` differs; no `--template` → fingerprint only (no xpub line).
3. **Language load-bearing** — same entropy, `--language english` vs `--language french` → DIFFERENT fingerprint (proves language affects the seed); `--language` omitted → "DEFAULT" annotation on stdout+stderr (decode parity).
4. **Passphrase** — `--passphrase x` changes the fingerprint; `--passphrase-stdin` reads stdin; single-stdin guard (ms1 from stdin + `--passphrase-stdin` → BadInput exit 1); inline `--passphrase`/`--phrase`/`--hex` → argv-leak advisory.
5. **Network** — `--network testnet` → `tpub` account xpub; default mainnet → `xpub`; fingerprint network-independent (same for both).
6. **Input exclusivity** — ms1 + `--hex`, `--hex` + `--phrase`, etc. → clap conflict (exit 64); bad hex → BadInput (exit 1); bad phrase → Bip39.
7. **`--json` shape** — `{schema_version:"1", master_fingerprint, network, language, language_defaulted, account_path?, account_xpub?}`; account fields present only with `--template`; valid JSON; NO seed/xprv field.
8. **No-secret-on-stdout** — assert stdout never contains an `xprv`/`tprv` prefix or a 64-byte seed hex (only fingerprint + xpub).
9. **Lockstep** — `ms gui-schema` includes `derive`; manual flag-coverage passes.

---

## §5. Non-goals / boundaries
- **No master seed / root xprv / private keys on stdout** (firm boundary — the toolkit's `convert` does secret outputs).
- **No signing.**
- **Single-sig templates only** (bip44/49/84/86); no multisig/`--path` power-user surface in v1 (the toolkit covers those).
- **mainnet/testnet only** for `--network` (signet/regtest share testnet xpub bytes; out of scope).

---

## §6. Open questions for R0
1. Input: positional ms1 + `--hex` + `--phrase` (3-way exclusive, stdin-default to ms1) — confirm the clap structure is sound (positional conflicts_with flags).
2. `--template` optional (fingerprint-only without it) vs a default (e.g. bip84) — SPEC picks optional (don't assume a script type). Confirm.
3. Master fingerprint network-independence — confirm `Xpriv::new_master(NetworkKind, seed).fingerprint()` is identical for Main vs Test kind (the fingerprint is HASH160 of the pubkey, version-independent). If NOT, document.
4. argv-leak advisory — does ms-cli have `secret_in_argv_warning` / an equivalent? (encode emits a passphrase-not-stored note; verify the inline-secret advisory mechanism.)
