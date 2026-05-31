# `ms derive` Implementation Plan

> Per-phase TDD (tests before impl); per-phase opus review to 0C/0I persisted to `design/agent-reports/`. Checkbox steps.

**Goal:** Add `ms derive` — from an ms1 card / `--hex` / `--phrase`, emit the master fingerprint (always) + an account xpub (with `--template`). Read-only public derivation, no seed/xprv/signing.

**Architecture:** New `cmd/derive.rs` (DeriveArgs + run). New `bitcoin = "0.32"` dep + a small `advisory::secret_in_argv_warning` helper (ms-cli has none). New `DeriveJson` in format.rs. Wired into main.rs (alphabetical `Derive`).

**Tech stack:** Rust, bip39 2, bitcoin 0.32, clap, serde_json, zeroize. Spec: `design/SPEC_ms_derive.md` (R1 GREEN @ `e3d5665`). SemVer ms-cli 0.4.2→0.5.0.

**Verified APIs (@ e3d5665 + toolkit derive_slot.rs):** `run(args) -> Result<u8>` (by value); `Command`/dispatch/`is_json_mode` exhaustive (main.rs:68/150/176, clap-err→64); `read_input`/`is_stdin_arg`/`read_stdin`(mlock) (parse.rs:21/97/65); `parse_hex_entropy`/`read_phrase_input` (encode.rs); `Mnemonic::{parse_in,from_entropy_in,to_seed,to_entropy}`; `From<CliLanguage> for bip39::Language` + `as_str` (language.rs:43/27); `CliError::BadInput(String)→1` (no `From<bitcoin>`); `Secp256k1::new()` (signing); `Xpriv::{new_master,fingerprint,derive_priv}`/`Xpub::from_priv`; decode's `Option<CliLanguage>`+`(English,defaulted)`+"DEFAULT" annotation (decode.rs).

---

## File structure
- **Create** `crates/ms-cli/src/cmd/derive.rs` — DeriveArgs, Template/Net enums, run.
- **Create** `crates/ms-cli/src/advisory.rs` — `secret_in_argv_warning`.
- **Modify** `crates/ms-cli/src/cmd/mod.rs` — `pub mod derive;`; `src/main.rs` — `mod advisory;` + `Derive` arm (alphabetical) + dispatch + is_json_mode.
- **Modify** `crates/ms-cli/src/format.rs` — `DeriveJson`.
- **Modify** `crates/ms-cli/Cargo.toml` — `bitcoin = "0.32"`, version 0.5.0.
- **Create** `crates/ms-cli/tests/cli_derive.rs`.
- **Lockstep:** toolkit `43-ms.md` + `cli-subcommands.list`; GUI `schema/ms.rs` (+repair backfill) + pins; toolkit ms pins.

---

## Phase 0 — deps + advisory helper

### Task 0.1: add bitcoin dep + advisory helper
- [ ] `Cargo.toml`: add `bitcoin = "0.32"` to `[dependencies]`. Build (downloads).
- [ ] Create `src/advisory.rs` (port toolkit byte-shape):
```rust
use std::io::Write;
/// Stderr advisory when a secret arrives inline on argv (visible via /proc/$PID/cmdline).
pub fn secret_in_argv_warning<W: Write>(stderr: &mut W, flag: &str, alternative: &str) {
    let _ = writeln!(
        stderr,
        "warning: secret material on argv ({flag}) — pipe via {alternative} to avoid /proc/$PID/cmdline exposure"
    );
}
```
  Add `mod advisory;` to main.rs. Build. Commit.

---

## Phase 1 — DeriveArgs + fingerprint core

### Task 1.1: failing test (fingerprint from an ms1 card)
- [ ] **Test** `tests/cli_derive.rs` (build an ms1 of the all-zeros 16-byte entropy via `ms encode --hex 00..00`, then `ms derive <ms1>` → master_fingerprint `73c5da0a` (the abandon-seed master fp; oracle: `mnemonic convert --from phrase="abandon … about" --to fingerprint --template bip84`)):
```rust
use assert_cmd::Command;
fn ms(args: &[&str]) -> std::process::Output { Command::cargo_bin("ms").unwrap().args(args).output().unwrap() }
#[test] fn derive_fingerprint_from_ms1() {
    let enc = ms(&["encode","--hex","00000000000000000000000000000000"]);
    let ms1 = String::from_utf8(enc.stdout).unwrap();
    let ms1 = ms1.lines().next().unwrap().trim();
    let o = ms(&["derive", ms1]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(String::from_utf8(o.stdout).unwrap().contains("73c5da0a"));
}
```
- [ ] **Run → FAIL** (no subcommand).
- [ ] **Implement** `DeriveArgs` + the fingerprint path:
```rust
#[derive(Args, Debug)]
#[command(group = clap::ArgGroup::new("entropy_src").args(["ms1","phrase","hex"]))]
pub struct DeriveArgs {
    pub ms1: Option<String>,                 // positional; stdin if -/omitted
    #[arg(long)] pub hex: Option<String>,
    #[arg(long)] pub phrase: Option<String>,
    #[arg(long, value_enum)] pub template: Option<Template>,
    #[arg(long, default_value_t = 0)] pub account: u32,
    #[arg(long, value_enum, default_value_t = Net::Mainnet)] pub network: Net,
    #[arg(long)] pub passphrase: Option<String>,
    #[arg(long, conflicts_with = "passphrase")] pub passphrase_stdin: bool,
    #[arg(long)] pub language: Option<CliLanguage>,   // NO default_value (decode shape)
    #[arg(long)] pub json: bool,
}
#[derive(Copy,Clone,Debug,PartialEq,Eq,clap::ValueEnum)] #[clap(rename_all="lower")]
pub enum Template { Bip44, Bip49, Bip84, Bip86 }
impl Template { fn purpose(self)->u32 { match self {Self::Bip44=>44,Self::Bip49=>49,Self::Bip84=>84,Self::Bip86=>86} } }
#[derive(Copy,Clone,Debug,PartialEq,Eq,clap::ValueEnum)] #[clap(rename_all="lower")]
pub enum Net { Mainnet, Testnet }
impl Net {
    fn kind(self)->bitcoin::NetworkKind { match self {Self::Mainnet=>bitcoin::NetworkKind::Main,Self::Testnet=>bitcoin::NetworkKind::Test} }
    fn coin(self)->u32 { match self {Self::Mainnet=>0,Self::Testnet=>1} }
    fn as_str(self)->&'static str { match self {Self::Mainnet=>"mainnet",Self::Testnet=>"testnet"} }
}
```
  `pub fn run(mut args: DeriveArgs) -> Result<u8>` (M1 — `mut` required for `mem::take`, per encode.rs:50): `mem::take` secrets→Zeroizing; resolve `(lang, defaulted)`; resolve entropy (ms1→decode→Entr; --hex→`parse_hex_entropy` — **M2: promote it `pub(crate)` in encode.rs + `use crate::cmd::encode::parse_hex_entropy`**, single source of truth; --phrase→parse_in.to_entropy); `mnemonic = from_entropy_in(lang,&entropy)` (or the parsed phrase); passphrase (stdin/inline); `seed = mnemonic.to_seed(&pp)`; `let secp = Secp256k1::new(); let master = Xpriv::new_master(net.kind(), &seed[..]).map_err(|e| CliError::BadInput(format!("master derive: {e}")))?; let fp = master.fingerprint(&secp);` emit text (fingerprint only, lowercase hex via `fp.to_string()`). Wire `Derive` (alphabetical) into Command + dispatch + is_json_mode.
- [ ] **Run → PASS. Commit.**

### Task 1.2: --hex/--phrase parity + input exclusivity + Phase-1 review
- [ ] Tests: `--hex 00..00` + `--phrase "abandon…about"` → same fp as ms1; ms1+--hex → clap conflict (exit 64); bad hex → BadInput (1); bad phrase → Bip39. Implement. Dispatch Phase-1 opus review → persist → 0C/0I.

---

## Phase 2 — account xpub (--template / --network)

### Task 2.1: --template account xpub
- [ ] Tests: `--template bip84 --account 0` → account_xpub == `mnemonic convert --from phrase="abandon…about" --to xpub --template bip84 --network mainnet`; bip44/49/86 parity; `--account 1` differs; no --template → no account_xpub line; `--network testnet` → `tpub`; fingerprint identical mainnet vs testnet. **Implement:**
```rust
// (M4: `use std::str::FromStr;` at top of derive.rs for `DerivationPath::from_str`.)
if let Some(t) = args.template {
    let path = bitcoin::bip32::DerivationPath::from_str(
        &format!("m/{}'/{}'/{}'", t.purpose(), args.network.coin(), args.account))
        .map_err(|e| CliError::BadInput(format!("account path: {e}")))?;  // M5: map_err, not unwrap
    let acct = master.derive_priv(&secp, &path).map_err(|e| CliError::BadInput(format!("account derive: {e}")))?;
    let account_xpub = bitcoin::bip32::Xpub::from_priv(&secp, &acct);
    // emit account_path (m/..'/..'/..') + account_xpub
}
```
- [ ] Run → PASS → commit. Phase-2 review → 0C/0I.

---

## Phase 3 — language DEFAULT annotation / passphrase / guards / advisory

### Task 3.1: language annotation + passphrase + single-stdin + argv advisory
- [ ] Tests: same entropy `--language english` vs `--language french` → DIFFERENT fp; `--language` omitted → "DEFAULT" on stdout+stderr (decode parity); `--passphrase x` changes fp; `--passphrase-stdin` reads stdin; ms1-from-stdin + `--passphrase-stdin` → BadInput (exit 1); inline `--phrase`/`--hex`/`--passphrase` → argv advisory on stderr. **Implement:** the DEFAULT annotation (mirror decode.rs emit), passphrase resolution (`passphrase_stdin → read_input(Some("-"))`; else inline), the single-stdin guard (`is_stdin_arg(ms1) || phrase==Some("-") || hex==Some("-")`) + `secret_in_argv_warning` emits for each inline secret. Run → PASS → commit. Phase-3 review → 0C/0I.

---

## Phase 4 — --json + no-secret assertion

### Task 4.1: DeriveJson
- [ ] Tests: `--json` shape `{schema_version:"1", master_fingerprint, network, language, language_defaulted, account_path?, account_xpub?}` (account fields omitted without --template — skip_serializing_if); valid JSON; **stdout never contains `xprv`/`tprv` or a 64-byte seed hex** (no-secret boundary). **Implement** `DeriveJson` in format.rs (`#[serde(skip_serializing_if="Option::is_none")]` on account fields). Run → PASS → commit. Phase-4 review → 0C/0I.

---

## Phase 5 — version + CHANGELOG + clippy/fmt + end-of-cycle + ship

### Task 5.1: bump + gate
- [ ] `crates/ms-cli/Cargo.toml` 0.4.2→0.5.0; `Cargo.lock` re-resolve; **I2: add the `## ms-cli [0.5.0]` entry to the top-level `/CHANGELOG.md`** (ms-cli has NO per-crate CHANGELOG; the root one carries crate-prefixed `## ms-cli [X]` entries — latest is `[0.4.1]`). `cargo clippy -p ms-cli --all-targets -- -D warnings`; `cargo test -p ms-cli --no-fail-fast` 0 failures. **M3: mnemonic-secret has NO fmt CI gate** (rust.yml = test/clippy/miri; edition 2021) — `cargo +stable fmt` is local hygiene only, NOT an authoritative gate (mirror the toolkit "no fmt gate" lesson, NOT the library-codec "fmt authoritative" one).
### Task 5.2: end-of-cycle R0
- [ ] Dispatch opus over `master..HEAD`; persist; fold to 0C/0I (re-dispatch per fold).
### Task 5.3: ship
- [ ] Clean tree; `git fetch`; `master==origin/master`; ff-merge master; push; tag `ms-cli-v0.5.0`; push tag; `cargo publish -p ms-cli` (ms-cli IS on crates.io; ms-codec 0.2.1 + bitcoin 0.32 resolve — dry-run first).

---

## Phase 6 — lockstep
### Task 6.1: manual (toolkit repo)
- [ ] `43-ms.md`: `ms derive` section (every flag) + fix "Five subcommands" count + add `ms derive` to `cli-subcommands.list`; flag-coverage lint vs the v0.5.0 ms binary.
### Task 6.2: toolkit sibling-pin
- [ ] toolkit ms-cli pin → `ms-cli-v0.5.0` at the TWO sites that have it (I1): `scripts/install.sh:38` + `.github/workflows/manual.yml:88` (both currently `ms-cli-v0.4.1`). **`quickstart.yml` has NO ms-cli pin** — do not touch it. (sibling-pin-check gate validates install.sh vs manual.yml.)
### Task 6.3: GUI schema-mirror (mnemonic-gui)
- [ ] `schema/ms.rs`: add `derive` SubcommandSchema + **backfill `repair`** (its `--ms1`+`--json`); bump `pinned_version` "ms 0.2.1"→"ms 0.5.0" + `pinned-upstream.toml` `ms-cli-v0.4.1`→v0.5.0. `schema_mirror` (MS_BIN=v0.5.0) → 0 drift.

---

## Self-review
- Spec coverage: §4 tests 1-9 → Phases 1-4 ✓. C2 advisory = Phase 0/3 ✓. C3 GUI repair-backfill = Task 6.3 ✓. Lockstep = Phase 6 ✓.
- No placeholders for load-bearing code (DeriveArgs, derivation flow, template path, advisory) ✓.
- Type consistency: `Template`/`Net`/`DeriveArgs`/`secret_in_argv_warning`/`DeriveJson` names consistent ✓.
- `Secp256k1::new()` (signing); `--language` Option-no-default (defaulted signal); BadInput map_err on bitcoin errors ✓.
