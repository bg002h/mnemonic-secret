＃ SPEC — Wave-2 secret-hygiene (ms lane): ms-cli `derive` Xpriv in-repo scrub

**Repo:** `mnemonic-secret` (the ms repo)
**Source SHA pin:** `origin/master` @ `3edf64ae62a406e5a46505637ad3e0139fa03bf3` (HEAD == origin/master, working tree clean)
**Authoring date:** 2026-06-22
**Bundled SemVer:** ms-cli `0.10.0` → **`0.11.0` (MINOR)** — recommended; PATCH `0.10.1` is the R0 fallback (see §6 + Open-questions).
**Ship mechanism:** crates.io publish of **ms-cli only** (ms-codec **NOT** bumped). Tag `ms-cli-v0.11.0`.

> **All cited line numbers below were re-grepped against the pinned SHA at write time** (citations decay every merge). The FOLLOWUPS entries' own `:220`/`:232-233` citations are stale (they predate cycle-15's derive edits); the live lines are recorded inline.

---

## 0. Scope statement

This cycle closes the **in-repo leg** of FOLLOWUP `ms-cli-derive-xpriv-master-not-zeroized` (recon item M1): a best-effort byte-scrub of the two derived `Xpriv` values in `ms derive`. The recon's framing that this was "lifetime-min ONLY / cannot scrub in-repo" is **outdated** — the toolkit's already-shipped `ScrubbedXpriv` (v0.70.0) proves both secret byte-components (`private_key` + `chain_code`) are reachable on the pinned `bitcoin = 0.32.100`.

**Out of scope / cleanly excluded (see §7):**
- The clean upstream `Zeroize`/`ZeroizeOnDrop` + non-`Copy` `Xpriv` (`rust-bitcoin-xpriv-zeroize-upstream`) — upstream-blocked.
- The entire codex32 share-spine (`ms-codec-share-strings-not-zeroized-encode-and-combine`, recon item B1) — blocked on the codex32 vendor/fork decision; foreign private-field `String`, not wrappable in-repo. **No ms-codec change this cycle.**
- **DO NOT** touch the frozen g6 mlock anchor `ms-cli-v0.7.0` — that is a separate codex32-paired cycle.

---

## 1. Verified facts (re-grepped @ `3edf64ae`)

### 1.1 The two derived-Xpriv sites — `crates/ms-cli/src/cmd/derive.rs`

| Binding | Line | Holds | Last use | Current state |
|---|---|---|---|---|
| `let master = Xpriv::new_master(args.network.kind(), &seed[..])` | **226-227** | ROOT private key (`private_key` + `chain_code`) | `:228` `master.fingerprint(&secp)`; `:239` `master.derive_priv(...)` | bare `Xpriv`, no Drop/Zeroize, sits un-scrubbed to scope end |
| `let acct_xpriv = master.derive_priv(&secp, &path)` | **238-240** | ACCOUNT private key | `:241` `Xpub::from_priv(&secp, &acct_xpriv)` | bare `Xpriv`, dropped at end of the `if let Some(t)` block |

Already-shipped (cycle-15 Lane M, commit `108e1ea`) — **DO NOT touch / redo:**
- `let seed: Zeroizing<[u8; 64]> = ...` (`:217`) + `crate::mlock::pin_pages_for(&seed[..])` (`:218`) — seed source IS scrubbed-on-drop + mlock-pinned. Correct.
- The explanatory comment block (`:220-225`) naming the slug + the upstream block. This is the **entire** "PARTIAL" that shipped — it added **no actual scrub**. This spec **reworks** that comment (§3.3).
- All clap-arg/intake `Zeroizing` wraps in derive.rs are already shipped (separate sweep legs).

`Xpriv::new_master` + `.derive_priv(` are the **only** Xpriv-materializing calls in the entire ms repo non-test source (grep `crates/` @ `3edf64ae`: exactly `derive.rs:226` + `derive.rs:239`). No caller fan-out anywhere.

### 1.2 Upstream secret-hygiene facts (verified against vendored source, not slug text)

- `bitcoin-0.32.100/src/bip32.rs:72-86`: `Xpriv` is `#[derive(Copy, Clone, PartialEq, Eq)]` with `pub private_key: secp256k1::SecretKey` (`:84`) + `pub chain_code: ChainCode` (`:86`). **grep `Zeroize|impl Drop|ZeroizeOnDrop` over bip32.rs → 0 matches.**
- `secp256k1-0.29.1/src/key.rs:58`: `pub struct SecretKey([u8; 32])`; `:60` `impl_non_secure_erase!(...)`; `:972` `pub fn non_secure_erase(&mut self)` exists (best-effort upstream erase; named "non_secure" precisely because `Copy` may have spilled bit-copies).
- `bitcoin-0.32.100/src/bip32.rs:50-51`: `pub struct ChainCode([u8; 32]); impl_array_newtype!(ChainCode, u8, 32);` → the macro (`bitcoin-internals-0.3.0/src/macros.rs:20`) yields `pub fn as_mut_ptr(&mut self) -> *mut u8`. **Volatile chain_code zero-write IS reachable in-repo.**
- `derive.rs` already imports `bitcoin::bip32::{DerivationPath, Xpriv, Xpub}` (`:12`) + `bitcoin::secp256k1::Secp256k1` (`:13`). No new top-level import is required for the scrub primitives (`non_secure_erase` is an inherent method on the `pub` field; `as_mut_ptr` is inherent on `ChainCode`; `core::ptr::write_volatile` is std).

### 1.3 The lint

`crates/ms-cli/tests/lint_zeroize_discipline.rs` — `ZEROIZE_ROWS` (currently **13** rows) is a **positive evidence-anchor** list (`source.contains(needle)` per row). `canonical_list_has_expected_row_count` asserts `n == 13` (the load-bearing tripwire). **This lint has NO partition-scan / staleness-tripwire pair** (unlike the toolkit's allowlist-tier convention) — so a new site unions in **exactly one** place (the `ZEROIZE_ROWS` const) **plus** the count assert. `derive.rs` has **no row today**.

### 1.4 Test home

`crates/ms-cli/tests/cli_derive.rs` already drives both paths with golden constants: `MASTER_FP_EN = "73c5da0a"` (`:14`), `BIP84_ACCT_XPUB`, `ZEROS_HEX`. Existing tests `json_shape` (`:162`), `account_xpub_bip84_matches_oracle` (`:55`), `fingerprint_from_ms1` (`:39`), `no_secret_on_stdout` (`:177`). This is the byte-identical-output regression home.

### 1.5 The reusable pattern (DO NOT reinvent)

`mnemonic-toolkit/crates/mnemonic-toolkit/src/derive_slot.rs:195-239` — the shipped, R0-blessed `ScrubbedXpriv(Xpriv)` move-only newtype:
- `pub fn new(xpriv: Xpriv) -> Self` (by value)
- `&self` accessors `xpub(&self, secp)` / `fingerprint(&self, secp)` — inner `Xpriv` never escapes
- `impl Drop`: (1) `self.0.private_key.non_secure_erase()`; (2) `for i in 0..32 { unsafe { core::ptr::write_volatile(cc_ptr.add(i), 0u8) } }` over `self.0.chain_code.as_mut_ptr()`
- `Copy` is E0184-blocked by `impl Drop` (structural); `Clone` deliberately NOT derived; `Clone`-absence pinned at compile time by an `AmbiguousIfImpl<_>` `const _: fn()` block in `scrub_tests` (derive_slot.rs:379-398); runtime drop witness at derive_slot.rs:536-549.

---

## 2. Design decision: OPTION 2 (recommended) vs OPTION 1

Two viable shapes (decide at R0; this spec recommends **OPTION 2** and writes the test surface for it, with OPTION 1's delta noted):

### OPTION 2 — binary-private `ScrubbedXpriv` newtype (RECOMMENDED)
Port the toolkit's `ScrubbedXpriv` into `derive.rs` (or a small `derive` submodule), **binary-private** (NOT `pub`). Wrap `master` and `acct_xpriv` in `ScrubbedXpriv` immediately on derivation; read `fingerprint` / build `Xpub` via `&self` accessors.

- **Pro:** structural Copy-escape impossibility (E0184); auto-scrub on drop; mirrors an already-R0-blessed pattern → cheap R0.
- **Pub-struct-Drop trap — CONTAINED:** ms-cli is a **binary** crate; the newtype is binary-private (no `pub`, no external consumers), so the "Drop breaks move-out destructure" trap does not bite anyone. Keep it private; do **NOT** derive `Clone`/`Copy`/`Into`/`Deref`.

### OPTION 1 — function-local `mut` bindings + private `fn scrub_xpriv`
Make `master`/`acct_xpriv` `mut`; add `fn scrub_xpriv(x: &mut Xpriv)` doing (a) `x.private_key.non_secure_erase()` + (b) the volatile chain_code zero-write; call it on `acct_xpriv` after `:241` and on `master` after `:239`.

- **Pro:** lowest blast radius, no new type. **Con:** no structural Copy-escape guard; manual call-placement is forgettable (the lint backstops it).

**Both** honor the same best-effort caveat: `Xpriv` is `Copy` upstream, so scrubbing a named binding does not guarantee the compiler kept no transient copy — the inherent limit, already documented in the toolkit SAFETY note and the FOLLOWUP. **Do NOT** try to defeat it. **Do NOT** wrap the `Xpriv` in `Zeroizing` (no `Zeroize` impl → will not compile). **Do NOT** use `SecretString` (that is for `String`, not `Xpriv`).

---

## 3. The change (OPTION 2 shape)

### 3.1 Add the binary-private `ScrubbedXpriv` to `derive.rs`

Port `mnemonic-toolkit/.../derive_slot.rs:195-239` verbatim in shape (newtype + `new` + `xpub` + `fingerprint` + `impl Drop`), **dropping `pub`** (binary-private). No `#[allow(dead_code)]` needed — it is consumed immediately by `run()`. Carry the SAFETY comment on the volatile loop (live, 32-byte, aligned, `u8` no-Drop / no-invalid-bitpattern). Carry the `// DO NOT add Clone/Copy/into_inner/Deref<Xpriv>` warning.

### 3.2 Rewire `run()`

- `:226`: `let master = ScrubbedXpriv::new(Xpriv::new_master(args.network.kind(), &seed[..]).map_err(...)?);`
- `:228`: `let master_fp = master.fingerprint(&secp);`
- `:238-240`: `let acct_xpriv = ScrubbedXpriv::new(master.derive_priv_scrubbed(&secp, &path).map_err(...)?);` — i.e. `master` must expose a `&self` derive accessor that returns the child `Xpriv` by value (then immediately wrapped). Add `fn derive_priv(&self, secp, path) -> Result<Xpriv, bip32::Error>` to `ScrubbedXpriv` delegating to `self.0.derive_priv(secp, path)` (the child `Xpriv` is the value we then wrap). The inner parent `Xpriv` still never escapes.
- `:241`: `let acct_xpub = acct_xpriv.xpub(&secp);`
- `master` (ScrubbedXpriv) drops at end of `run()` → scrub; `acct_xpriv` (ScrubbedXpriv) drops at end of the `if let Some(t)` block → scrub.

**Output is byte-identical:** `master_fp` and `acct_xpub` are materialized (`.to_string()`) before either `ScrubbedXpriv` drops; the scrub touches only post-last-use private memory. The `--json` `DeriveJson` (`:248`) and text paths are untouched.

> **OPTION 1 delta (if R0 chooses it):** instead of the newtype, `let mut master = ...` / `let mut acct_xpriv = ...`, a private `fn scrub_xpriv(x: &mut Xpriv)`, and explicit `scrub_xpriv(&mut acct_xpriv);` after `:241` + `scrub_xpriv(&mut master);` after the `if let` block. Same byte-identical-output guarantee.

### 3.3 Rework the `:220-225` comment block

Current text overstates the block as "we cannot scrub in-repo." Replace with: the derived `Xpriv` values are now scrubbed best-effort on drop (`ScrubbedXpriv` → `private_key.non_secure_erase()` + volatile `chain_code` zero-write); the inherent `Copy`-spill caveat remains (best-effort), and a **clean** `Zeroize`/non-`Copy` `Xpriv` stays upstream-blocked (`rust-bitcoin-xpriv-zeroize-upstream`). The seed stays `Zeroizing` + mlock-pinned.

---

## 4. Test surface (TDD — write tests first)

Per the project rule, run the **FULL** `cargo test -p ms-cli` suite (not targeted) at every gate — derive/argv/lint phases ripple into lint + output-class + vectors tests.

### 4.1 Byte-identical-output regression (write FIRST, in `cli_derive.rs`)
Add a test asserting `ms derive` stdout (text) AND `--json` output is byte-identical to the captured golden for **both** paths:
- fingerprint-only: `ms derive --hex <ZEROS_HEX>` → `master_fingerprint: 73c5da0a` (reuse `MASTER_FP_EN`).
- account-xpub: `ms derive --hex <ZEROS_HEX> --template bip84` and `--json` → `BIP84_ACCT_XPUB`.

This MUST be RED-then-GREEN-invariant across the scrub change (the scrub is output-invisible). The existing `json_shape` / `account_xpub_bip84_matches_oracle` / `fingerprint_from_ms1` / `no_secret_on_stdout` already cover the values; add an explicit "output unchanged by scrub" assertion anchored on the goldens so a regression in the rewire surfaces.

### 4.2 Lint row (union ONE place + the count)
In `crates/ms-cli/tests/lint_zeroize_discipline.rs`:
- Add **one** `ZeroizeRow` for `src/cmd/derive.rs`. Evidence anchor for OPTION 2: `&["fn drop(&mut self)", "non_secure_erase()"]` (the scrub impl) — or anchor on `"struct ScrubbedXpriv"` + `"non_secure_erase()"`. (OPTION 1: evidence `&["fn scrub_xpriv("]` / `&["master.private_key.non_secure_erase()"]`.)
- Bump `canonical_list_has_expected_row_count`: `assert_eq!(n, 13, ...)` → `14`, **and update the message string** to mention the derive-scrub row. **The count assert is load-bearing — both the `ZEROIZE_ROWS` entry and the count must move together, or it goes RED.**

### 4.3 (OPTION 2 only) Compile-time + runtime scrub witness
Port the toolkit's `scrub_tests` shape into a `#[cfg(test)] mod scrub_tests` in `derive.rs`:
- The `const _: fn() = || { ... AmbiguousIfImpl<_> ... };` compile-time `Clone`/`Copy`-absence assertion (derive_slot.rs:379-398) — the load-bearing move-only guard.
- A runtime drop-witness test (`scrubbed_xpriv_self_accessors_and_drop`, derive_slot.rs:536-549): build a known `Xpriv`, wrap, assert `xpub`/`fingerprint` match the bare upstream derivation of the same key, then drop. (Reading raw bytes post-drop is best-effort/UB-adjacent; the toolkit's runtime test asserts the **accessor surface + that drop runs**, not the post-drop byte values — mirror that, do NOT add a read-after-drop assertion.)

### 4.4 Negative
No flag/option/subcommand/output change → **no manual / CLI-reference / schema-mirror update** (lockstep mirror invariant N/A). `cli_derive.rs::no_secret_on_stdout` continues to guard that no seed/xprv reaches stdout.

---

## 5. Version & ship sites (the release ritual)

Bundled SemVer: **ms-cli `0.11.0`** (MINOR). Update every site in lockstep in the shipping commit:

| Site | Change |
|---|---|
| `crates/ms-cli/Cargo.toml:3` | `version = "0.10.0"` → `"0.11.0"` |
| `Cargo.lock` (the `name = "ms-cli"` block, line 444-445) | `version = "0.10.0"` → `"0.11.0"` (regenerate via `cargo update -p ms-cli` / a build; do NOT hand-edit float-cmp's unrelated 0.10.0 at line 297) |
| `CHANGELOG.md` | new `## ms-cli [0.11.0] — 2026-06-22` section |
| `design/FOLLOWUPS.md` | flip status (see §6) |
| Tag | `ms-cli-v0.11.0` |
| crates.io | `cargo publish -p ms-cli` |

**No README/install.sh version pins to chase:** `README.md` + `crates/ms-cli/README.md` use bare `cargo install ms-cli` (no pinned version); there is no `install.sh` in this repo. (Confirmed @ `3edf64ae`.)

**NO ms-codec bump** — the change is entirely in `crates/ms-cli`. ms-cli's `ms-codec = { path = "../ms-codec", version = "=0.6.0" }` (Cargo.toml:20) is unchanged and already satisfied by published ms-codec 0.6.0.

**NO toolkit pin bump** — `mnemonic-toolkit` consumes `ms-codec = "0.6"` (a library), **not** ms-cli. An ms-cli-only change cannot disturb the toolkit. (Confirmed: toolkit Cargo.toml depends on ms-codec, never ms-cli.)

### Publish order
Single artifact, no ordering: **publish ms-cli 0.11.0 only.** Do NOT re-publish ms-codec 0.6.0 (unchanged). No sibling-repo companion publish.

---

## 6. FOLLOWUPS status flips (in the shipping commit — verify-status-at-decision-time)

- `ms-codec-derive-xpriv-master-not-zeroized` → in `design/FOLLOWUPS.md` this is `ms-cli-derive-xpriv-master-not-zeroized` (line **470**, status at line **478**): flip `open` / **PARTIAL (cycle-15 Lane M)** → **`resolved (in-repo leg, ms-cli 0.11.0)`; residual upstream tracked as `rust-bitcoin-xpriv-zeroize-upstream`.** Note the best-effort byte-scrub (`private_key.non_secure_erase()` + volatile `chain_code` zero-write) now lands the derived-Xpriv leg; only the clean `Zeroize`/non-`Copy` `Xpriv` stays upstream-blocked. Update its `Fix direction` line (which currently says "scrub the underlying secret bytes where reachable" — now DONE).
- `rust-bitcoin-xpriv-zeroize-upstream` (line **482**): **leave `open` (upstream-blocked).** Update its `Companion`/`Fix direction` to reflect that the in-repo best-effort scrub now ships in ms-cli 0.11.0 (was "lifetime-min only … landed in ms-cli 0.10.0").

> **SemVer rationale (PATCH vs MINOR) — R0 decides.** Recommended **MINOR (0.11.0)** to stay consistent with the constellation secret-type-migration precedent (v0.10.1/v0.67.0/0.68.0/0.69.0): introducing a named secret-confinement type (`ScrubbedXpriv`) is the same architectural class even though `ms derive` stdout/`--json` (master_fingerprint + account_xpub only) is byte-identical before/after. **PATCH (0.10.1)** is defensible only if R0 picks OPTION 1 framed as a pure function-local scrub with zero signature/type change ("function-local Zeroizing/scrub = NO-BUMP/PATCH" rule). OPTION 2 introduces a (binary-private) type → leans MINOR.

---

## 7. Deferred / blocked — explicitly excluded, with why

| Item (recon key) | Status | Why excluded |
|---|---|---|
| `rust-bitcoin-xpriv-zeroize-upstream` (B2) | **BLOCKED — upstream** | `bitcoin 0.32.100` `Xpriv` has no `Zeroize`/`Drop` AND is `#[derive(Copy)]` (Copy ⊥ Drop, E0184). A clean fix requires upstream to add `Zeroize`/`ZeroizeOnDrop` **and** drop the `Copy` derive (a breaking change) — not authorable in-repo. The in-repo best-effort scrub we ship is the **mitigation, not the close**; this entry stays `open`. Actionable deliverable is upstream (file/track the rust-bitcoin issue, analogue of `rust-bip39-mnemonic-zeroize-upstream`) — out of scope for a code cycle. |
| `ms-codec-share-strings-not-zeroized-encode-and-combine` (B1) | **BLOCKED — codex32 vendor/fork decision** | `codex32::Codex32String` is `pub struct Codex32String(String)` (codex32-0.1.0 lib.rs:102) with no `Drop`/`Zeroize` and a **private** `String` field — cannot wrap-and-scrub in-repo; `Zeroizing<T>` requires `T: Zeroize` which the foreign type does not satisfy. The reusable `SecretString`/`ScrubbedXpriv` precedents compose only over values WE own. Gated on `codex32-upstream-dormant-vendor-vs-accept-decision`. **No ms-codec change, no lint row, no status flip** (adding a String-leg lint row now would be a false-GREEN — the cycle-15 anti-pattern). The already-shipped `Vec<u8>` wraps (`filler` shares.rs:150, recovered wire bytes shares.rs:318) stay as-is — do NOT double-wrap. |
| frozen g6 mlock anchor `ms-cli-v0.7.0` | **EXCLUDED** | Separate codex32-paired cycle. Do not move the pin. |

---

## 8. Risks / invariants (carry into R0)

- **Best-effort caveat is inherent, not a bug:** `Xpriv: Copy` + `SecretKey: Copy` mean the compiler may have spilled bit-copies the scrub can't reach; `non_secure_erase` is upstream-named "non_secure" for exactly this. Document it (the toolkit SAFETY note + the FOLLOWUP already do); do NOT represent the scrub as a complete/guaranteed wipe.
- **No double-Zeroizing / double-scrub:** these two bindings have no existing scrub; `non_secure_erase` + volatile-write are idempotent. The `seed` is already `Zeroizing` — do NOT re-wrap it.
- **Pub-struct-Drop trap — CONTAINED (OPTION 2):** binary-private newtype, no external consumers; keep it non-`pub`, no `Clone`/`Copy`/`Into`/`Deref`.
- **Signature fan-out — none:** both options are local to `derive.rs::run`; no public fn signature changes, no caller edits (the only two Xpriv call-sites in the repo are these). `master_fp`/`acct_xpub` are still read via `&self` accessors (OPTION 2) / before-scrub (OPTION 1).
- **Lint is single-union:** `lint_zeroize_discipline.rs` is a positive-anchor list with NO partition-scan/staleness pair — union ONE place (`ZEROIZE_ROWS`) + the count. Do NOT search for a second allowlist to update (there isn't one).
- **Citation-decay guard:** all line numbers in this spec are @ `3edf64ae`. Re-grep at implementation start if any merge has landed since.

---

## Open questions (block R0 → must resolve before code)

1. **OPTION 1 vs OPTION 2** — spec recommends OPTION 2 (mirrors the R0-blessed toolkit `ScrubbedXpriv`; structural Copy-escape impossibility). Confirm or pick OPTION 1.
2. **PATCH (0.10.1) vs MINOR (0.11.0)** — spec recommends MINOR + OPTION 2 for precedent-consistency. Confirm.
3. **`derive_priv` accessor on `ScrubbedXpriv`** (OPTION 2) — the child derivation needs `&self.0.derive_priv(...)`. Confirm the `&self` delegating accessor (parent `Xpriv` never escapes) is acceptable, vs OPTION 1's plain `master.derive_priv` on a `mut` binding.