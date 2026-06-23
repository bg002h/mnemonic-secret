# SPEC — Cycle-B: vendor (inline) codex32 into ms-codec + scrub the share spine

**Persisted to:** `/scratch/code/shibboleth/mnemonic-secret/design/SPEC_codex32_vendor_fork_cluster.md`
**Status:** DRAFT — pending mandatory R0 architect gate (0C/0I before any code).
**Cross-repo:** `mnemonic-secret` (ms-codec + ms-cli) + `mnemonic-toolkit` (paired, non-optional).
**Source SHAs verified LIVE (citations re-grepped, recon @6e3ee8e was grounding):** ms-codec `origin/master` @ **6e3ee8e**; toolkit `origin/master` @ **684e510**; vendored `codex32-0.1.0` checksum `d230935f…918e9`, CC0-1.0.

Slugs closed: `rust-codex32-zeroize-upstream`, `codex32-upstream-dormant-vendor-vs-accept-decision`, `ms-codec-share-strings-not-zeroized-encode-and-combine`; companions `rust-codex32-upstream-pr2-recovery-bug-not-exposed` (anchor re-point) + `[obs] recovered-secret-string-not-zeroized` (FOLLOWUPS.md:16) + a toolkit companion note.

---

## 0. DECISION: INLINE (shape A) — `pub mod codex32` inside ms-codec

**RESOLVED (A) inline.** Vendor codex32's 3 runtime modules under `crates/ms-codec/src/codex32/`, drop the external `codex32 = "=0.1.0"` workspace dep, re-export `ms_codec::codex32::{Codex32String, Fe, Error, Parts, ChecksumEngine}`. Consumers migrate by a mechanical path rewrite `codex32::` → `ms_codec::codex32::`.

**Why A over B (decisive, live-verified — overturns the recon's lean toward B):**
1. codex32 is consumed ONLY through ms-codec's ms1 domain — no independent-reuse story exists; B would mint a perpetual crates.io crate with one logical consumer.
2. **The forced consumer migration is IDENTICAL under A and B** — the bare extern `codex32::` path cannot survive either shape (B→`ms_codex32::`, A→`ms_codec::codex32::`). Same blast radius; B's "clean boundary" buys nothing on the migration axis.
3. A *removes* the dormant external dep (stated user intent); B *replaces* a dormant external dep with a self-owned external dep = strictly more surface (a 4th publish, a new security-owned crate, a new lockstep version site).
4. The registry-publish constraint (a published crate can't path/git-dep) is satisfied trivially by A; B is the shape that *introduces* a publish to satisfy it.
5. Wire-byte-identity is equally strong under A (byte-for-byte copy regardless of submodule vs sibling-crate).

**The one property B has that A lacks** — independent reuse of codex32 by a non-ms-codec consumer — **does not exist in this constellation and is not a goal.** Proceed with A.

R0 scrutiny hooks: (a) `pub mod codex32` keeps the SAME public surface as upstream (no widening — `checksum`/`field` stay private submodules, curated re-exports only); (b) CC0 LICENSE vendored verbatim + attribution header; (c) the toolkit's *direct* `codex32 = "=0.1.0"` dep is DROPPED entirely (post-A it names codex32 types via `ms_codec::codex32::`).

---

## 6 Phases (each = its own per-phase R0 loop to 0C/0I + TDD)

**P1 — vendor byte-identical + attribution + re-export.** ADD `src/codex32/{mod,field,checksum}.rs` (byte-for-byte copies of upstream lib.rs runtime body 1–429 / field.rs 1–263 / checksum.rs 1–191; upstream `#[cfg(test)]` modules NOT copied; dev-only `correction-table.rs` bin NOT vendored) + `src/codex32/LICENSE` + attribution header. `pub mod codex32` in lib.rs. DROP `codex32` from workspace + ms-codec + ms-cli Cargo.toml. **RED-first `tests/codex32_vendor_parity.rs`** (hard-codes BIP-93 `bip_vector_2/3/4/5` output strings + a golden `from_seed` set captured from the pre-vendor build). BIP-93 corpus / PR#2 regression / spike_kofn stay GREEN.

**P2 — Zeroize/Drop/Debug on `Codex32String` (ONLY behavioral change).** Derive `Clone, PartialEq, Eq, Hash, zeroize::ZeroizeOnDrop` (RETAIN Clone — `interpolate_at` self-return clone at lib.rs:262 + `combine_shares` clone at shares.rs:208; RETAIN PartialEq/Eq — the M6 `derived != parsed[j]` at shares.rs:304; Hash retained for source-compat). **REMOVE `Debug` from derive** (it leaked the full secret — L22-class) + hand-roll a length-only redacting `Debug`. Add `zeroize` `features = ["derive"]` to ms-codec Cargo.toml (currently absent; toolkit already has it). RED-first 8-char-window no-echo Debug test; re-run parity gate (proves zero output-byte perturbation). Encoding paths (`from_seed`/`from_string`/`interpolate_at`/`checksum.rs`/`field.rs`) NEVER touched.

**P3 — ms-codec rewire (paths + error surface).** `ms_codec::Error::Codex32`'s inner type moves `codex32::Error` → `crate::codex32::Error` (variant NAME + field SHAPE preserved = source-compatible name, but inner type-path move = pre-1.0 breaking). Rewrite `codex32::` → `crate::codex32::` across the 5 src files (`error.rs` 21/151-164/260-264, `shares.rs` 18+inline, `inspect.rs`, `envelope.rs`, `decode.rs`) + `ms_codec::codex32::` across the 7 codex32-naming test files. No new behavioral tests; whole `cargo test -p ms-codec` passes unchanged.

**P4 — share-spine scrub + lint floor.** `secret_s`/`defining`/`parsed`/recovered-`secret` (all `Codex32String`) now auto-drop-scrub (no wrapper needed); update the cycle-15 "Q2 HOLD / no Drop" comments (shares.rs 129-139, 312-316) to RESOLVED. **`distributed: Vec<String>` is the irreducible residue** (it's the RETURN value — can't wrap without changing the public type) → document under the caller-wrap contract, HONESTLY (no false GREEN). Bump `lint_zeroize_discipline.rs` floor `assert_eq!(n, 4)` → `5` + add the codex32-scrub `ZeroizeRow` (HARD tripwire — un-bumped → RED). RED-first on the count+row edit.

**P5 — ms-cli + FORCED paired toolkit change.** ms-cli: drop codex32 dep, `ms-codec = "=0.7.0"`, version 0.11.0→0.12.0, rewrite `codex32_friendly.rs`/`error.rs` + 11 test files. **Toolkit (NON-OPTIONAL): DROP `codex32 = "=0.1.0"`, `ms-codec "0.6"`→`"0.7"`, version 0.71.0→0.72.0, rewrite friendly.rs's 15 `codex32::Error::*`/`codex32::Fe::*` sites → `ms_codec::codex32::`, regenerate Cargo.lock + fuzz/Cargo.lock, add CHANGELOG 0.72.0.** Confirm GUI schema_mirror / manual / mlock-g6 all CLEAN.

**P6 — publish + tag + flips.** ms-codec 0.7.0 → crates.io+tag; ms-cli 0.12.0 → crates.io+tag; toolkit direct-FF+tag `mnemonic-toolkit-v0.72.0`. Flip the 3 slugs `resolved` + re-point the 2 companions + toolkit companion note, all in the shipping commits. Mandatory whole-diff post-impl adversarial review (folds RE-ENTER a scoped convergence review before tag).

---

## SemVer

| Crate | From → To | Bump |
|---|---|---|
| ms-codec | 0.6.0 → **0.7.0** | MINOR (pre-1.0 breaking: `Error::Codex32` inner type-path move + new `pub mod codex32` + dep drop; wire BYTE-IDENTICAL) |
| ms-cli | 0.11.0 → **0.12.0** | MINOR (rides ms-codec `=0.7.0`; no CLI flag change) |
| mnemonic-toolkit | 0.71.0 → **0.72.0** | MINOR (consumes breaking inner-type via friendly.rs; drops codex32 dep; re-pins ms-codec) |
| vendored codex32 | — | NONE (inlined private module; no crate/version) |

---

## Wire-byte-identity (single most load-bearing invariant)
Byte-for-byte copy + edits confined to the `Codex32String` derive/Debug/Drop. NEW `codex32_vendor_parity.rs` pins BIP-93-published strings + a captured golden `from_seed` set (pins to BIP + golden, NOT to itself); the existing BIP-93/PR#2/spike_kofn corpus stays GREEN; parity re-run AFTER P2. Any failure → STOP, do not patch.

## Toolkit friendly.rs coordination
The forced break: `ms_codec::Error::Codex32(_)` inner type moves out of the extern `codex32` crate. friendly.rs matches it 15× (`ThresholdNotPassed`/`RepeatedIndex`/`MismatchedLength`/`MismatchedHrp`/`MismatchedThreshold`/`MismatchedId`/`InvalidChecksum` + test-block `codex32::Fe::Q` constructions). All rewrite `codex32::` → `ms_codec::codex32::`; the toolkit's direct `codex32 = "=0.1.0"` dep is dropped. friendly messages stay byte-identical (tests assert on substrings).
