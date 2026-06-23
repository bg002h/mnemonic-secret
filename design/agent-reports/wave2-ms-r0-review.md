# R0 Review — Wave-2 secret-hygiene (ms lane): ms-cli `derive` Xpriv in-repo scrub

**Reviewer:** opus architect (adversarial R0)
**Spec SHA pin:** `mnemonic-secret` @ `3edf64ae62a406e5a46505637ad3e0139fa03bf3`
**Verification:** HEAD == origin/master == `3edf64ae`, working tree clean (confirmed). All cited paths/lines re-grepped against this SHA; upstream secret-hygiene facts verified against the vendored crate source, NOT the slug text.

## VERDICT: GREEN — 0 Critical / 0 Important / 4 Minor

The gate is satisfied. Implementation may proceed once the user confirms the three Open-questions (design choices, not gaps). The Minor findings are drafting nits and require no fold to start coding; address them in-flight.

---

## Critical
None.

## Important
None.

## Minor

### M1 — §3.2 accessor method-name inconsistency (`derive_priv_scrubbed` vs `derive_priv`)
The rewire bullet writes `let acct_xpriv = ScrubbedXpriv::new(master.derive_priv_scrubbed(&secp, &path)...)` but the very next clause says "Add a `fn derive_priv` to `ScrubbedXpriv` delegating to `self.0.derive_priv(...)`". The two names contradict. The accessor itself does NOT scrub (the child is wrapped by the caller), so the name `derive_priv_scrubbed` is misleading. Recommend the plain `fn derive_priv(&self, secp, path) -> Result<Xpriv, bip32::Error>` delegating to `self.0.derive_priv` — the inner parent never escapes; the returned child is wrapped by `ScrubbedXpriv::new` at the call site. Open-question #3 already surfaces this; just make the body self-consistent.

### M2 — Cargo.lock float-cmp line off-by-one
§5 cites float-cmp's unrelated `0.10.0` at "line 297"; live `name = "float-cmp"` is at **line 296** (the ms-cli block citation "444-445" IS correct: name@444, version@445). Both sites are "regenerate via `cargo update -p ms-cli`, do NOT hand-edit" guidance, so the off-by-one is harmless. Re-grep at impl start per the spec's own citation-decay guard.

### M3 — §6 stray `ms-codec-` slug lead-in
§6's first bullet leads with `ms-codec-derive-xpriv-master-not-zeroized` before self-correcting to `ms-cli-derive-xpriv-master-not-zeroized`. No `ms-codec-`-prefixed slug exists; the live entry header is at FOLLOWUPS.md:470, status at :478 (verified). Drop the stray `ms-codec-` lead to avoid a wrong grep target.

### M4 — Unstated Debug-leak IMPROVEMENT (favors OPTION 2)
ms-cli enables bitcoin's `std` feature (via `default`; verified `cargo tree -p ms-cli -e features -i bitcoin`), so `Xpriv` derives `Debug` (`#[cfg_attr(feature = "std", derive(Debug))]`, bip32.rs:71). The bare `master`/`acct_xpriv` bindings today carry a live `{:?}`-leaking Debug (never formatted → no current leak, but latent). OPTION 2's `ScrubbedXpriv` does NOT derive Debug and makes the inner `Xpriv` private, REMOVING that latent leak surface — RULE Z-DEBUG aligned, and the repo already enforces that exact discipline via `repair_detail_does_not_derive_debug` (lint_zeroize_discipline.rs). OPTION 1's `mut Xpriv` RETAINS the Debug-leak surface. Worth a one-line note as a secondary reason to prefer OPTION 2; not a blocker.

---

## Verified-correct (the load-bearing claims, re-grepped @ `3edf64ae`)

**Source sites (derive.rs):**
- `Xpriv::new_master(args.network.kind(), &seed[..])` → `master` at **derive.rs:226-227** ✓ (spec §1.1)
- `master.fingerprint(&secp)` → `master_fp` at **:228** ✓
- `master.derive_priv(&secp, &path)` → `acct_xpriv` at **:238-239** ✓
- `Xpub::from_priv(&secp, &acct_xpriv)` → `acct_xpub` at **:241** ✓
- `master` has NO use after :239 (grep: :228 + :238 only); `master_fp`/`acct_xpub` are materialized (`.to_string()`) into owned values BEFORE any scrub → **byte-identical output guarantee holds** ✓
- seed `Zeroizing<[u8;64]>` at :217 + `mlock::pin_pages_for` at :218 — already shipped (cycle-15 Lane M @108e1ea), correctly out-of-scope ✓
- `Xpriv::new_master` + `.derive_priv` are the ONLY xpriv-materializing calls in non-test source (grep `crates/`) → **zero caller fan-out, no signature changes** ✓

**Upstream facts (vendored source, verified not from slug):**
- bitcoin 0.32.100 + secp256k1 0.29.1 resolved for ms-cli (`cargo tree`) ✓
- `Xpriv` is `#[derive(Copy, Clone, PartialEq, Eq)]` with `pub private_key: secp256k1::SecretKey` + `pub chain_code: ChainCode` (bip32.rs:72-86); 0 `Zeroize|impl Drop|ZeroizeOnDrop` matches ✓
- `SecretKey([u8;32])` with `impl_non_secure_erase!` + `pub fn non_secure_erase(&mut self)` (secp256k1 key.rs:58-60, 972) — note it erases to `[1u8;32]`, not zeros, which still destroys the secret (fine) ✓
- `ChainCode([u8;32])` + `impl_array_newtype!` → `as_mut_ptr(&mut self) -> *mut u8` borrows the backing array by `&mut` (bitcoin-internals macros.rs:20-23) → the volatile zero-write hits real backing storage ✓
- `derive_priv(&self, ...)` takes `&self` (bip32.rs:608) → confirms the `ScrubbedXpriv` `&self` delegating accessor design (parent never moves/escapes) ✓
- **Best-effort caveat is correctly framed and NOT overclaimed:** `new_master` builds an un-scrubbed `Hmac<sha512>` (64-byte private+chaincode intermediate) on the stack (bip32.rs:578-590); `derive_priv`/`ckd_priv` do `let mut sk: Xpriv = *self;` (Copy spill) + per-step hmac intermediates (bip32.rs:613) — these transient copies are unreachable in-repo. The spec (§0, §2, §8) honestly scopes the scrub to the NAMED-binding residue (the locals that sit to scope-end), NOT a complete wipe. ✓

**Reusable pattern (toolkit, NOT reinvented):**
- `ScrubbedXpriv(Xpriv)` move-only newtype at derive_slot.rs:195-239 matches the spec's §1.5/§3.1 description verbatim: `new(by value)`, `&self` `xpub`/`fingerprint`, `impl Drop` doing `private_key.non_secure_erase()` + volatile 32-byte chain_code zero-write with the live SAFETY note ✓
- Compile-time move-only guard `const _: fn() = || { ... AmbiguousIfImpl<_> ... }` at derive_slot.rs:379-398; runtime drop-witness `scrubbed_xpriv_self_accessors_and_drop` at :536-549 (asserts accessor surface + that drop runs, NO read-after-drop assertion — spec §4.3 correctly mirrors this) ✓
- The toolkit runtime test reuses `master` after `ScrubbedXpriv::new(master)` (it's `Copy`) — independent witness of the Copy-spill caveat ✓

**SemVer / pub-struct-Drop trap:**
- ms-cli is **binary-only** (no `src/lib.rs`; only main.rs + modules) → the pub-struct-Drop "breaks move-out destructure for external users" trap is structurally CONTAINED (no external library consumers). The newtype is binary-private (non-`pub`). Spec §2/§8 correct ✓
- MINOR (0.11.0) is consistent with the constellation named-secret-type-migration precedent (v0.10.1/0.67.0/0.68.0/0.69.0) even with byte-identical stdout/`--json`; PATCH (0.10.1) is the defensible OPTION-1-only fallback. The spec defers the call to R0/user — correct framing ✓

**Lint surface:**
- `lint_zeroize_discipline.rs` `ZEROIZE_ROWS` = **13 array entries** (positive evidence-anchor, `source.contains(needle).any()`); `canonical_list_has_expected_row_count` asserts `assert_eq!(n, 13, ...)` — load-bearing tripwire ✓
- **NO partition-scan / staleness-tripwire pair** (grep: no partition/allowlist/walkdir/read_dir) → single-union: new row goes in `ZEROIZE_ROWS` + the count assert (BOTH must co-move or RED). Spec §1.3/§4.2/§8 correct; "do NOT search for a second allowlist (there isn't one)" verified ✓
- Precedent for the Debug-drop discipline exists (`repair_detail_does_not_derive_debug`) ✓

**Test home:**
- `cli_derive.rs` has 16 tests; golden constants `ZEROS_HEX` (:12), `MASTER_FP_EN = "73c5da0a"` (:14), `BIP84_ACCT_XPUB` (:16); existing `json_shape`/`account_xpub_bip84_matches_oracle`/`fingerprint_from_ms1`/`no_secret_on_stdout` present — valid byte-identical regression home ✓

**Version & ship sites:**
- ms-cli Cargo.toml:3 = `version = "0.10.0"` ✓; ms-codec dep `=0.6.0` (Cargo.toml:20) unchanged & satisfied ✓
- Cargo.lock ms-cli block @444-445 ✓
- CHANGELOG.md present, per-crate-prefixed format (`## ms-cli [X] — date`); current head `## ms-cli [0.10.0]` ✓
- NO install.sh (confirmed); README uses bare `cargo install ms-cli` (no pinned version) — spec §5 correct ✓
- **NO changelog/version CI gate in this repo** (workflows = rust.yml + fuzz-smoke.yml; 0 CHANGELOG/version references) — so the task-prompt's "changelog-check fires on the tag" applies to the TOOLKIT repo, NOT ms; the spec correctly lists CHANGELOG as a ship-site WITHOUT claiming an enforcing gate ✓
- tag `ms-cli-v0.10.0` exists locally + on origin (points to f943166, origin/master's parent) → 0.10.0 is the live baseline, 0.11.0 is the correct next ✓
- toolkit consumes `ms-codec = "0.6"` (a LIBRARY), never ms-cli → "NO toolkit pin bump" correct ✓

**Blocked/excluded legs (correctly fenced, no half-implementation, no false-GREEN):**
- `rust-bitcoin-xpriv-zeroize-upstream` (FOLLOWUPS:482) — stays `open`; the in-repo scrub is mitigation not close. Correct ✓
- `ms-codec-share-strings-not-zeroized-encode-and-combine` — `codex32::Codex32String(String)` has a PRIVATE String field + `#[derive(..., Debug)]` (codex32 lib.rs:102), un-wrappable in-repo; `Zeroizing<T>` needs `T: Zeroize`. NO ms-codec change, NO lint row, NO status flip (avoids the cycle-15 false-GREEN anti-pattern). The already-`Zeroizing` `Vec<u8>` filler (shares.rs:150) correctly left as-is. Correct ✓
- frozen g6 mlock anchor `ms-cli-v0.7.0` — excluded, do not move. Correct ✓

## Conclusion
The spec is unusually rigorous: every citation re-grepped, upstream facts checked against vendored source (not slug text), the reusable R0-blessed pattern correctly identified and ported, blocked legs cleanly fenced, and the test surface genuinely catches the rewire regression (byte-identical goldens + lint row/count co-move + compile-time move-only guard). The three Open-questions are legitimate design confirmations (OPTION 1 vs 2; PATCH vs MINOR; accessor shape), not gaps. **GREEN: 0C / 0I.** Proceed to implementation after the user resolves the Open-questions; fold the 4 Minor nits in-flight.