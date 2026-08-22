# cycle-prep recon — 2026-06-23 — codex32 vendor/fork cluster

**Origin/master SHA at recon time:** `6e3ee8e`
**Local branch:** `master`
**Sync state:** `up-to-date` (0 ahead / 0 behind)
**Untracked:** none in this repo (the 3 `cycle-prep-recon-*.md` in the system status are in the *toolkit* checkout, not here)

Slug(s) verified: `codex32-upstream-dormant-vendor-vs-accept-decision`, `rust-codex32-zeroize-upstream`, `ms-codec-share-strings-not-zeroized-encode-and-combine`. Citations are **clean/near-clean** — one DRIFTED-by-10 line on the recovered `secret` binding; everything else ACCURATE. The big finding is a **scope/SemVer constraint the slugs do not mention** (crates.io publish model forces the vendored crate to be published OR inlined — see Cross-cutting #1).

---

## Per-slug verification

### `rust-codex32-zeroize-upstream` — root-cause slug
- **WHAT:** Upstream `codex32::Codex32String`'s internal payload buffer has no `Zeroize`/`Drop`; ms-codec can only minimize lifetimes, not scrub. Closes only on an upstream Zeroize release OR an internally-controlled codex32 impl.
- **Citations:**
  - `codex32-0.1.0 lib.rs:102 — pub struct Codex32String(String)` — **ACCURATE.** Verified: `lib.rs:102 pub struct Codex32String(String);` with `#[derive(Clone, PartialEq, Eq, Hash, Debug)]` (lib.rs:101). Private `String` field, **derived `Debug` (leaks the secret string into `{:?}`/panic)**, NO `Drop`, NO `Zeroize`. This is exactly the L22-class footgun the user flagged: a raw `String`-backed secret with a non-redacting derived `Debug`.
  - `envelope::package` / `Codex32String::from_seed` "copies payload bytes into its private buffer" — **ACCURATE (with precision).** `from_seed` (lib.rs:312-380) base32-encodes `data: &[u8]` into a fresh `String ret` (lib.rs:323-361) and returns `Codex32String(ret)` (lib.rs:379). The secret therefore lives inside that owned `String` for the value's lifetime, un-scrubbed on drop. The *input* `&[u8]` is the caller's (already `Zeroizing` at ms-codec), but the encoded copy inside `Codex32String` is not reachable for scrubbing in-repo.
- **Crate facts (verified):** codex32-0.1.0 at `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/codex32-0.1.0`. **Zero `[dependencies]`** (empty section in the normalized Cargo.toml). No `build.rs`. Source = 3 modules: `lib.rs` 704, `field.rs` 319 (GF32 field arithmetic), `checksum.rs` 191 (BCH/bech32 checksum engine) = **~1214 LOC runtime** + `src/bin/correction-table.rs` 151 (dev-only error-correction-table generator, **not referenced by ms-codec/ms-cli — droppable**). License CC0-1.0 (public domain — clean to vendor/relicense).
- **Status verdict:** **`open` / blocked** (genuinely upstream-blocked; the realistic close path is this cluster's vendor/fork decision, NOT an upstream release). Accurate as written.
- **Action for brainstorm spec:** Cite SHA `6e3ee8e`. The fix is `impl Zeroize for Codex32String` (scrub the inner `String`) + `ZeroizeOnDrop` (or a manual redacting `Drop`) + a **redacting `Debug`** (replace the derived `Debug` — the derive currently prints the secret string). Note: `Codex32String` also derives `PartialEq/Eq/Hash` used by ms-codec's M6 consistency check (`derived != parsed[j]`, shares.rs:304) and `RepeatedIndex` logic — keep those derives; only `Debug` and the drop/scrub change.

### `codex32-upstream-dormant-vendor-vs-accept-decision` — the strategic decision slug
- **WHAT:** The pinned codex32 is abandoned (frozen at 0.1.0 since 2023-03-10; the promised rewrite never shipped). The two upstream items can never close via an upstream release. Decide **(a) accept** the lifetime-min mitigation vs **(b) vendor/fork** and own the fixes. The USER HAS DECIDED (b) VENDOR/FORK.
- **Citations:**
  - `dep codex32 = "=0.1.0"` (exact-pin) — **ACCURATE.** Verified `Cargo.toml:13 codex32 = "=0.1.0"`; workspace dep consumed by `ms-codec/Cargo.toml:16` and `ms-cli/Cargo.toml:24` (`{ workspace = true }`). `Cargo.lock` pins `codex32 0.1.0` checksum `d230935f…918e9`.
  - "Frozen at 0.1.0 since 2023-03-10", "0 open issues, 1 open PR (the recovery bug)", "no repository link on crates.io" — **dormancy claims; not independently re-verified against crates.io this recon** (network not exercised). Internally consistent with the companion `rust-codex32-upstream-pr2-recovery-bug-not-exposed` (which is `resolved` — our path proven unexposed + guarded by `tests/codex32_upstream_recovery_regression.rs`). For the spec: re-confirm the crates.io/upstream-repo state at write time, but dormancy is not load-bearing for the vendoring mechanics.
- **Status verdict:** **`open` → resolving** (the decision is MADE = vendor/fork; this cycle EXECUTES it). The slug body still reads "no urgency / strategic decision" — that framing is now superseded by the user's decision.
- **Action for brainstorm spec:** Cite SHA `6e3ee8e`. This slug is the umbrella; flipping it to `resolved` is the cycle's closing act. The exact-pin `=0.1.0` + the PR#2 regression anchor stay until the vendored crate replaces the dep.

### `ms-codec-share-strings-not-zeroized-encode-and-combine` — the consumer tail
- **WHAT:** ~7 secret-equivalent bare `Codex32String`/`Vec<Codex32String>`/`Vec<String>` bindings across the share spine (`encode_shares` + `combine_shares`) are held un-scrubbed because `Codex32String` has no `Drop`. The reachable `Vec<u8>` intermediates were already wrapped (cycle-15 Lane M); the `String`-backed legs are blocked on the codex32 decision.
- **Citations (all vs current `crates/ms-codec/src/shares.rs`):**
  - `secret_s: Codex32String` at `shares.rs:130` (the FULL secret at index S) — **DRIFTED but ~ACCURATE.** The binding is now at **`shares.rs:141`** (`let secret_s = Codex32String::from_seed(HRP, k_usize, &id, Fe::S, &bytes[..])?;`); `:130` now lands inside the cycle-15 lifetime-min comment block that *describes* `secret_s`. Off by ~11 lines (comment expansion). Content correct.
  - `defining: Vec<Codex32String>` at `:136` — **DRIFTED-by-11.** Now `shares.rs:147` (`let mut defining: Vec<Codex32String> = Vec::with_capacity(k_usize);`).
  - `distributed: Vec<String>` at `:148` — **DRIFTED-by-11.** Now `shares.rs:159`.
  - `single` at `:115` — **ACCURATE.** `shares.rs:115 let single = Codex32String::from_seed(...)` — exact.
  - `parsed: Vec<Codex32String>` x2 at `:195,210` (every INPUT share) — **DRIFTED-by-~11.** Now `shares.rs:206` (first parse) and `shares.rs:221` (the canonical re-parse). Two bindings, as cited; content correct.
  - `.clone()` copy `from_string(s.clone())` at `:197` — **DRIFTED-by-11.** Now `shares.rs:208`.
  - `c.to_string().to_ascii_lowercase()` at `:213` — **DRIFTED-by-11.** Now `shares.rs:224`.
  - `secret: Codex32String` (recovered full secret) at `:281` — **DRIFTED-by-10 / STRUCTURALLY mis-pointed.** `:281` is now a *comment* line ("[same hrp/id/threshold/length]"). The actual binding is **`shares.rs:291`** (`let secret = Codex32String::interpolate_at(k_set, Fe::S)...`). Largest drift in the slug; correct it.
  - `[obs] recovered-secret-string-not-zeroized` at FOLLOWUPS.md:16 cites `shares.rs:236-242` and `lint_zeroize_discipline.rs:62-69` — the `shares.rs:236` line is also stale (now `:291`); the lint citation `:62-69` is **ACCURATE** (the `ZeroizeRow{ label: "shares::{encode_shares,combine_shares} wrap OWNED secret Vecs" ... }` block is at lint_zeroize_discipline.rs:64-71, covering the `Zeroizing<Vec<u8>> filler` and `secret.parts().data()` wraps).
  - "reachable `Vec<u8>` intermediates wrapped (`filler`, recovered-secret wire bytes)" — **ACCURATE.** `filler` is `Zeroizing<Vec<u8>>` at `shares.rs:150`; the recovered wire bytes are `let data: Zeroizing<Vec<u8>> = Zeroizing::new(secret.parts().data());` at `shares.rs:317`. So the un-scrubbed residue is precisely the **`String`-backed `Codex32String`/`String` legs**, exactly as the slug claims.
- **Status verdict:** **`open` / PARTIAL — blocked-on-cluster.** The `Vec<u8>` legs are done; the `String` legs are the thing THIS cycle unblocks (once `Codex32String: Zeroize`, the spine bindings can be held in `Zeroizing<Codex32String>` / a `SecretString`-style wrapper). Status text is honest (explicitly "no false GREEN").
- **Action for brainstorm spec:** Cite SHA `6e3ee8e`. Refresh ALL line numbers to the +11/+10 drifted values above. This slug is the **acceptance criterion** for the vendoring cycle: after the vendored `Codex32String` gains `Zeroize`+`ZeroizeOnDrop`, re-audit each of the 7+ bindings and either wrap in `Zeroizing` or document that drop-scrub now covers them; then flip the slug `resolved`. Also extend the `lint_zeroize_discipline.rs` canonical list (currently floor-pinned at **exactly 4 rows**, `canonical_list_has_expected_row_count` asserts `n == 4`) to anchor the new `Codex32String` scrub — that floor MUST be bumped in lockstep or the lint goes RED.

---

## Cross-cutting observations

1. **★ LOAD-BEARING SCOPE CONSTRAINT the slugs omit — the crates.io publish model.** The toolkit consumes **`ms-codec = "0.6"` from crates.io** (registry source, `Cargo.lock` checksum `835040e2…2a9c`) and **`codex32 = "=0.1.0"` directly from crates.io**, NOT git tags. A crate published to crates.io may not depend on a `path=`-only or `git=`-only dependency. Therefore the vendored codex32 **cannot** be a bare in-repo `crates/codex32-vendored/` path-dep under a published ms-codec. Two viable shapes:
   - **(A) Inline codex32 as a private module inside ms-codec** (`crates/ms-codec/src/codex32/{mod,field,checksum}.rs`, ~1214 LOC absorbed). No new published crate. BUT — see #2 — the toolkit imports `codex32::Error` *directly* as a separate crate, so ms-codec must re-export a compatible error type, and the toolkit's own `codex32 = "=0.1.0"` dep + `friendly.rs` matches must be migrated. Highest blast radius on the public API but no new crate to publish.
   - **(B) Publish a renamed fork crate to crates.io** (e.g. `codex32-ms` / `ms-codex32`), ms-codec depends on it by version. Clean dependency story, but adds a 4th crates.io publish to the constellation and a new crate to maintain. The toolkit must repoint its direct `codex32` dep to the renamed crate (rename in `friendly.rs` imports).
   The brainstorm MUST pick (A) vs (B) up front — it drives crate count, publish list, and the toolkit's breaking-change surface. (B) is the cleaner long-term posture for "own the BCH/Shamir primitives"; (A) avoids a new publish but couples the fork's lifecycle to ms-codec releases.

2. **★ DOWNSTREAM TOOLKIT COUPLING — bigger than the slugs imply.** `ms_codec::Error::Codex32(codex32::Error)` (ms-codec error.rs:21) wraps the upstream error type by value, and the **toolkit's `friendly.rs` pattern-matches `codex32::Error::*` variants by name in 15 places** (`ThresholdNotPassed`, `RepeatedIndex`, `MismatchedLength/Hrp/Threshold/Id`, `InvalidChecksum{checksum,string}`). ms-codec's own error.rs touches `codex32::Error` 13 times. So:
   - The codex32 `Error` enum's **public variant names + field shapes are load-bearing across two repos.** The vendored fork MUST preserve `Error`'s public shape verbatim (or both ms-codec error.rs AND toolkit friendly.rs change in lockstep).
   - Under shape (B) the inner type renames (`codex32::Error` → `ms_codex32::Error`) → `ms_codec::Error::Codex32(_)`'s payload type changes → **toolkit `friendly.rs` breaks** (import + match-path rename) = a coordinated paired toolkit PR. Under shape (A), ms-codec must `pub use` the inlined error type as `ms_codec::codex32::Error` (or similar) to keep the toolkit compiling, or the toolkit migrates off the direct `codex32` dep entirely. **Either shape requires a paired toolkit change** — this is NOT an ms-codec-internal-only cycle.

3. **WIRE-FORMAT BYTE-IDENTITY (the user's 3b caution) — confirmed load-bearing + structurally safe to preserve.** The encoding lives entirely in `from_seed` (base32 packing, lib.rs:343-361) + the BCH `checksum.rs` engine (BIP-93 codex32 long/short generator polynomials) + `field.rs` GF(32) arithmetic. The Zeroize additions touch ONLY: the `Drop`/`Zeroize`/`Debug` impls on `Codex32String`. They do NOT touch `from_seed`, `from_string`, `interpolate_at`, `parts`, `checksum.rs`, or `field.rs`. **Vendoring must be a byte-for-byte copy of those three modules with zero encoding edits.** Mandatory guard: the existing `tests/bip93_inline_vectors.rs` (BIP-93 corpus: 5 valid cells + the 64-invalid parametric cell) + `tests/codex32_upstream_recovery_regression.rs` (PR#2 secret) + `tests/spike_kofn.rs` must all stay GREEN against the vendored crate, proving identical encoding/checksum behavior. Recommend adding a one-shot KAT that asserts a fixed seed → identical `ms1`/codex32 string pre- vs post-vendor.

4. **g6 mlock anchor is NOT moved by this cycle.** The "frozen g6 mlock anchor (ms-cli-v0.7.0)" is a **source-byte invariant** (`mnemonic-toolkit/tests/mlock_g6_invariant.rs` byte-compares the toolkit's `mlock.rs` against `crates/ms-cli/src/mlock.rs`), not a version pin on ms-codec. This cycle touches `shares.rs`/`envelope.rs`/`error.rs` + a new vendored codex32 module — it does **not** edit `mlock.rs` in either repo, so the g6 anchor is untouched. No action, no risk, as long as `mlock.rs` stays out of the diff (and is never `cargo fmt`'d — standing MEMORY rule).

5. **lint_zeroize_discipline.rs floor is a hard tripwire.** `canonical_list_has_expected_row_count` asserts `ZEROIZE_ROWS.len() == 4` exactly (not `>=`). Adding the `Codex32String` scrub row bumps this to 5 — the assert message must update in the same edit or the test goes RED (this is the exact "un-union → RED" tripwire class from the Group A R0 ruling in MEMORY).

6. **Two companion FOLLOWUPS update in lockstep.** `rust-codex32-upstream-pr2-recovery-bug-not-exposed` (currently `resolved`, exposure NONE) references the same `=0.1.0` pin + regression anchor — when the vendored crate replaces the dep, that anchor's "future codex32 bump" framing should be re-pointed at the vendored crate. And the `[obs] recovered-secret-string-not-zeroized` at FOLLOWUPS.md:16 is subsumed by the first-class slug — close it together.

---

## Recommended brainstorm-session scope

**Grouping — ONE cycle, internally PHASED (do NOT split into separate cycles).** All three slugs are the same root cause + its consumer; splitting would ship a half-state (vendored crate with no consumer rewire = dead code). But the cycle is large and MUST be phased behind the mandatory R0 gate:

- **Phase 0 (decision, in the brainstorm/SPEC):** lock shape **(A) inline private module** vs **(B) renamed published fork crate**. This is the single highest-leverage architectural call (drives crate count, publish list, toolkit breakage). My read: **(B)** best matches the user's stated "own the BCH/Shamir primitives / de-risk the dormant dep" intent and keeps a clean dependency boundary, at the cost of one new crates.io publish + the toolkit import rename. **(A)** is lower ceremony (no new publish) but pollutes ms-codec and still forces a toolkit migration. Either way the toolkit is touched — flag to the user that this is **not ms-codec-only**.
- **Phase 1:** vendor the 3 modules **byte-identical** (lib/field/checksum, drop the dev-only `correction-table` bin), wire up the build, prove BIP-93 corpus + PR#2 regression + spike_kofn GREEN against the vendored copy (encoding parity gate — TDD: parity test first).
- **Phase 2:** add `impl Zeroize for Codex32String` (scrub inner `String`) + `ZeroizeOnDrop` + a **redacting `Debug`** (the derive leaks today). Preserve `Clone/PartialEq/Eq/Hash` (ms-codec's M6 + RepeatedIndex depend on them). Re-run encoding parity (must stay byte-identical).
- **Phase 3:** rewire ms-codec — `Error::Codex32` inner type, the `shares.rs` spine bindings (7+) now held in `Zeroizing`/secret wrappers, bump the `lint_zeroize_discipline.rs` floor 4→5 + add the scrub row, re-audit the share-string slug to GREEN.
- **Phase 4:** the **paired toolkit change** — repoint the toolkit's direct `codex32` dep + migrate `friendly.rs`'s 15 `codex32::Error::*` matches to the vendored error path; re-pin `ms-codec` to the new version; full toolkit suite + fuzz GREEN.
- **Phase 5:** publish + tag (see SemVer below); flip all three slugs + the two companions.

**Rough LOC:** vendored codex32 ~1214 LOC copied (+ ~40 LOC of Zeroize/Drop/Debug impls). ms-codec rewire ~80-150 LOC (error type, ~7 spine bindings, lint floor). Toolkit ~30-60 LOC (`friendly.rs` import/match rename + dep repoint). Largest of the 4 maturity-program items, as the prompt anticipated — primarily from the byte-identical crate copy + the cross-repo blast radius, not algorithmic complexity.

**SemVer:**
- **codex32 vendored crate:** new crate (shape B) → `0.1.0` of `ms-codex32`/chosen name (or a private module, shape A — no version).
- **ms-codec:** `0.6.0 → 0.7.0` (**MINOR**). The change is additive-scrubbing + a transitive-dep swap; the `ms1` wire format is byte-identical (no breaking *output*). BUT `Error::Codex32`'s inner type changes under shape (B) — that's a breaking change to a `pub` enum's payload type → strictly this is a **MINOR pre-1.0 / would-be-MAJOR post-1.0** breaking-API bump. Pre-1.0, MINOR is correct and signals the break.
- **ms-cli:** `0.11.0 → 0.12.0` (**MINOR**, rides the ms-codec bump; no CLI flag change → no GUI `schema_mirror` impact, no manual mirror impact for ms-cli's own surface, but confirm no `--help` text references codex32 by crate name).
- **toolkit:** **MINOR** (consumes the breaking ms-codec `Error::Codex32` inner type via `friendly.rs`; even though end-user behavior is unchanged, the dep + a `pub`-reachable match path move → bump and re-pin). Lands as the next toolkit MINOR after the current head.

**Mandatory locksteps:**
- **Paired toolkit PR** (Cross-cutting #2) — NON-OPTIONAL; `friendly.rs` will not compile otherwise.
- **NO GUI `schema_mirror` impact** — this cycle adds/changes no clap flags/subcommands/dropdowns on any CLI. Confirm during Phase 4 but expected clean.
- **NO manual mirror impact** under `docs/manual/src/40-cli-reference/` — no CLI surface change. Confirm clean.
- **FOLLOWUPS lockstep:** flip `codex32-upstream-dormant-vendor-vs-accept-decision` (→ resolved, decision executed), `rust-codex32-zeroize-upstream` (→ resolved, fork owns Zeroize), `ms-codec-share-strings-not-zeroized-encode-and-combine` (→ resolved, spine scrubbed) + the two companions (`rust-codex32-upstream-pr2-recovery-bug-not-exposed` anchor re-point; `[obs] recovered-secret-string-not-zeroized` close) — all in the shipping commit.
- **lint floor 4→5** + new scrub row (Cross-cutting #5) — same edit.

**Inter-slug ordering:** root → consumer is forced. `rust-codex32-zeroize-upstream` (Zeroize on the vendored `Codex32String`) MUST land before `ms-codec-share-strings-...` (which consumes it); `codex32-upstream-dormant-vendor-vs-accept-decision` is the umbrella that closes last. One cycle, the phase order above.

**One cycle or phased?** **ONE cycle, 6 phases, cross-repo (ms-codec + ms-cli + toolkit).** Honestly the largest of the maturity-program's 4 items: the algorithmic risk is low (byte-identical copy of a frozen, dependency-free, CC0 crate), but the blast radius is wide — a new crate, a breaking `pub` error-type move, and a forced paired toolkit migration. Budget accordingly.
