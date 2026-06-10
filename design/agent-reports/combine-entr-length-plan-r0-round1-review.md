# R0 Architecture Review — PLAN_combine_entr_length_validation.md — Round 1

**Reviewer:** Fable 5 (feature-dev:code-architect)
**Date:** 2026-06-10
**Plan doc:** `design/PLAN_combine_entr_length_validation.md`

---

## VERDICT: YELLOW — 1 Critical, 0 Important

The fix itself (the one-arm change to `dispatch_payload`) is correct, sound, and complete. The gap description, call-chain analysis, error-propagation chain, SemVer reasoning, and caller-impact analysis are all accurate. The plan fails at one specific point: the test helper as written will not compile.

---

## Critical

**C1 — `RESERVED_PREFIX` is not in scope in `shares.rs::mod tests`; the test fixture will fail to compile.**

`shares.rs` imports only `use crate::consts::{HRP, RESERVED_ID_BLOCKLIST, SHARE_INDEX_V01};` at module level (`shares.rs:13`). The `mod tests` block inherits only what is in scope in the enclosing module; `RESERVED_PREFIX` is NOT among those imports and is therefore not pulled in by `use super::*`.

The `nonstandard_entr_distributed` helper (plan §2) uses `let mut bytes = vec![RESERVED_PREFIX];` → unresolved name → `error[E0425]: cannot find value 'RESERVED_PREFIX' in this scope`.

Fix (one line inside `mod tests`): `use crate::consts::RESERVED_PREFIX;` — or replace the bare name with literal `0x00u8` (const's value at `consts.rs:17`). The import is cleaner.

The existing test at `shares.rs:471-472` (`combine_rejects_secret_share_index_s`) avoids this by going through `crate::envelope::payload_wire_bytes(&entr_p())` rather than constructing the prefix directly — which is why the gap was not noticed. The plan's claim that "`RESERVED_PREFIX` is in scope" is incorrect. All OTHER listed names (`HRP`, `non_s_index_pool`, `random_id`, `Codex32String`, `Fe`, `combine_shares`, `dispatch_payload`, `Error`) ARE genuinely in scope via `use super::*`.

---

## Important

None.

---

## Minor

**m1 — `decode.rs:92-93` carries an identical `.expect`, transitively fixed (informational).** `crates/ms-cli/src/cmd/decode.rs:92-93`: `Mnemonic::from_entropy_in(lang, &entropy[..]).expect("ms-codec validates entropy length; from_entropy_in cannot fail")`. The decode path calls `dispatch_payload` via `discriminate` (envelope.rs:150), so after the fix the Entr arm validates and this `.expect` invariant also becomes true. No extra code change — the fix covers it transitively; worth noting in the plan/CHANGELOG.

**m2 — Two CHANGELOG files.** Workspace root `CHANGELOG.md` documents both ms-codec + ms-cli; `crates/ms-codec/CHANGELOG.md` also exists. Add a `### Fixed` entry to both. Fix touches ms-codec only.

**m3 — Pre-fix TDD claim caveat.** The plan correctly notes `combine_shares` returns `Ok(Entr([17 bytes]))` pre-fix (panic is in `emit_phrase`, not the codec), so Test A's `matches!(Err)` fails (not panics) pre-fix under standard `panic=unwind` test mode. Sound; could be slightly clearer.

**m4 — SemVer granularity.** ms-cli binary is unchanged (no flag/output change; the `.expect` comment tweak is optional prose). Bump ms-codec only (0.4.0 → 0.4.1). Don't auto-bump ms-cli for a codec patch with no CLI delta — confirm by convention.

---

## Confirmed-accurate claims

1. **Gap.** `envelope.rs:167-188`: Entr arm (:169-172) returns `Payload::Entr(data[1..].to_vec())` NO validate; Mnem arm (:173-182) calls `p.validate()?`. Doc-comment :165 "then `validate()`" is the violated contract. Line numbers match live source.
2. **`Payload::validate()`.** `payload.rs:66-73`: Entr branch checks `VALID_ENTR_LENGTHS.contains(&data.len())` → `Error::PayloadLengthMismatch { tag, expected, got }`. `VALID_ENTR_LENGTHS = &[16,20,24,28,32]` (`consts.rs:29`). `PayloadLengthMismatch` pre-exists (`error.rs:58-65`). No new variant.
3. **CLI propagation.** `combine.rs:58` `combine_shares(&shares)?`; `From<ms_codec::Error> for CliError` complete (`ms-cli/src/error.rs:132-248`, `PayloadLengthMismatch` arm :184-188). Err → `emit_phrase` (:77) not called → `.expect` (:97) unreachable.
4. **`combine_shares` propagation.** `shares.rs:241` `dispatch_payload(&data)?`. Validate in Entr arm → `combine_shares` returns `Err(PayloadLengthMismatch)`.
5. **Exactly two `dispatch_payload` callers.** `envelope.rs:150` (discriminate) + `shares.rs:241` (combine_shares). No third caller to break.
6. **`from_seed` signature.** `codex32-0.1.0/src/lib.rs:312-318`: `from_seed(hrp:&str, threshold:usize, id:&str, share_idx:Fe, data:&[u8])`. Plan's call form matches (k:usize, id:String→&id).
7. **18-byte payload accepted by codex32.** 18 data bytes → 29 symbols → total 51 chars ∈ [48,94). `from_seed` succeeds.
8. **`nonstandard_entr_distributed(2,2,17)` reaches `dispatch_payload`.** Two shares, threshold '2', distinct non-s indices, same id/length → all combine_shares gates pass → interpolate_at recovers `[0x00, 0xCD×17]` → dispatch_payload reached.
9. **Pattern correct.** `PayloadLengthMismatch { got: 17, .. }` — three named fields (tag/expected/got), `got: usize`, `..` ignores rest.
10. **Pre-fix returns Ok not panic** (verified in #3/#8). Test A assertion fails cleanly pre-fix.
11. **Toolkit inherits the fix.** `mnemonic-toolkit/src/cmd/ms_shares.rs:385` `ms_codec::combine_shares(...).map_err(ToolkitError::from)?`; Phrase arm :432-436 uses `.map_err(ToolkitError::Bip39)?` (not `.expect`) → no panic site. Same library fn.
12. **SemVer PATCH.** ms-codec 0.4.0 → 0.4.1. No API/flag/variant change; schema_mirror unaffected.

---

## Required fold before implementation

Fold C1: add `use crate::consts::RESERVED_PREFIX;` inside `mod tests` in `shares.rs` (or use literal `0x00u8`). No other changes required; the fix code itself is correct as-written.
