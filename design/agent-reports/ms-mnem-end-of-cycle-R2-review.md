# End-of-cycle R0 review — ms mnem — round 2

**Verdict:** GREEN (0C / 0I)

History: R0 found C1 (descriptor-@N emit stripped language); the C1 fold (`80f78fc`) introduced C2 (import-json wire-entr card corrupted to mnem under `--language`); C2 was folded (`326fe08`). This round verifies C2 + proves no C3 + re-runs the exhaustive emit-site sweep.

## Critical
None. C2 is fixed; the fix introduced no C3.

## Important
None.

## Minor
- **M3 (NEW, design-clarity, non-blocking — pre-existing, NOT fold-introduced):** plan line 147 "English/entropy/entr-source → entr (no regression)" could be read to mean a raw-entropy CLI source always emits `entr`. The governing dataflow note at plan line 144 (R2-I6) clarifies: the `resolve_slots` flow ("source = phrase/**entropy**/seedqr/`--slot @N.entropy=<hex>`") has `language: None` and "derive + emit **resolve through `--language`**." Empirically: `bundle --slot @0.entropy=<16B-hex> --template bip84 --language japanese` emits 51-char `mnem`-ja (English run → 50-char entr). By design — a fresh CLI raw-entropy input under explicit `--language` honors it, distinct from a *wire* `entr` card (C2-pinned to agnostic/English passthrough). Suggest a one-line plan/SPEC clarification. Out of scope for this gate.
- M1/M2 from round 0 stand (pre-existing convert over-warning; one-directional round-trip test). Out of scope.

## Confirmations (airtight-invariant proof + COMPLETE emit-site sweep + suite results)

**1. C2 fold (`326fe08`) correct + complete.** Re-read all of `bundle_run_from_import_json` (bundle.rs:1587–1813). Exactly TWO sites set `.entropy = Some(...)`, EACH immediately followed by `.language = Some(...)`: wire-decode loop (bundle.rs:1667–1683): `Payload::Entr → English`, `Payload::Mnem → wire_code_to_bip39(wire_lang)`, then `entropy=Some`(1682)+`language=Some`(1683); overlay `--slot @N.phrase=` loop (1689–1750): `entropy=Some`(1745)+`language=Some(language.into())` where `language=args.language.unwrap_or_default()`(1749). No `ResolvedSlot {}` literal, no `--slot @N.entropy=` overlay, no seedqr overlay, no second merge loop in this function. Initial slots from `envelope_to_resolved_slots`→`mk1_card_to_resolved_slot` (json_envelope.rs:364–372) always `entropy:None, language:None` (watch-only). **No entropy-bearing slot left `None`. No C3.**

**2. No C3 downstream.** Advisory (`emit_unified`, bundle.rs:750–765): fires iff `any_secret_bearing() && run_lang != English && ∃ entropy slot with language.unwrap_or(run_lang)==English`. A wire-entr slot now `Some(English)` → under `--language japanese` predicate true → advisory FIRES (correct: an entr card under a non-English run is language-losing; warning appropriate). Over-warning at worst, never under-warning; model + emit now AGREE on the CORRECT behavior (entr stays entr, advisory fires) vs the pre-fix agreement on WRONG (mnem-ja, suppressed). Import-json doesn't re-derive (xpub from envelope mk1), so `Some(English)` feeds only the emit.

**3. EXHAUSTIVE emit-site sweep (third pass).** `grep -rn 'ms_codec::encode'` = exactly 5 sites:
| Site | Resolution | Verdict |
|---|---|---|
| synthesize.rs:150 `synthesize_full` | unconditional `Entr` | `#[allow(dead_code)]`, test-only callers — NOT CLI-reachable |
| synthesize.rs:308 `synthesize_descriptor` | `c.language.unwrap_or(run_language)` | correct; every caller's entropy slots now `Some` |
| synthesize.rs:469 `synthesize_multisig_full` | `seed_mnemonic.language()` | test-only |
| synthesize.rs:841 `synthesize_unified` | `s.language.unwrap_or(run_language)` | correct; seed-intake entropy `None`→honor `--language` by design (plan §144) |
| convert.rs:1235 `convert --to ms1` | `English→Entr else Mnem{cli_code}`, single-source | correct |
No `export-wallet` ms1 emit (consumes `bundle.ms1` verbatim → faithful passthrough). No independent `verify-bundle` emit (its synthesize calls re-synthesize the *expected* bundle for comparison through the same carrier).

**4. Caller audit (third pass).** `synthesize_descriptor` (5 non-test): descriptor-@N (bundle.rs:1465; slots 1477–1498 set `Some(c.language.unwrap_or(run_language))` — C1 ✓); concrete-descriptor (1543, English; watch-only `entropy:None`, moot ✓); import-json (1781; all entropy slots `Some` — C2 ✓, run_language moot); import_wallet.rs:1398 (English, no phrase ✓); verify_bundle.rs:923 (comparison-only ✓). `synthesize_unified` (bundle.rs:395 seed-intake + 3× verify_bundle comparison): resolve_slots phrase/seedqr (526–535) + raw-entropy (646–655) carry `language:None` → `unwrap_or(args.language)` → mnem for non-English typed phrase (the cycle's purpose) + mnem for raw entropy under explicit `--language` (by design). NOT touched by C1/C2 folds.

**Cycle-base fact:** at `e572888`, `ResolvedSlot` had NO `language` field and ALL emit sites unconditionally emitted `Payload::Entr` (no `Mnem` arm). The per-slot carrier is net-new this cycle; neither fold touched `resolve_slots`.

**5. No perturbation.** `cli_mnem_emit_preserve`: 12 passed (incl. `import_json_entr_card_stays_entr_under_language_japanese` = C2 repro byte-identical 50-char entr + `Payload::Entr`; `mixed_language_import_json_re_emits_mnem_ja_and_entr`; the 3 descriptor-@N tests; `english_phrase_convert_ms1_golden_byte_identity`). `cli_mnem_per_card_language`: 4 passed.

**6. Suites + clippy (against the dev `[patch.crates-io] ms-codec = path` override — expected pre-ship per plan Step 2).**
- `cargo test -p mnemonic-toolkit --no-fail-fast`: **2598 passed / 0 failed / 12 ignored** (+1 vs R1 = the C2 repro). clippy `--all-targets -D warnings`: clean.
- `cargo test -p ms-codec`: **98 passed / 0 failed**.
- `cargo test -p ms-cli`: one FLAKY failure under default parallelism — `mlock::tests::g4_a_pin_and_zeroize_compose_without_panic` (RLIMIT_MEMLOCK exhaustion when many mlocking tests run concurrently). Single-threaded: **147 passed / 0 failed** (passes in isolation). Environmental, NOT a cycle regression — the mlock surface (prior cycle) is untouched by the mnem diff; the last ms-repo cycle commit (`7820049`) is docs-only.

**Gate: GREEN — 0C / 0I.** Every entropy-bearing slot in `bundle_run_from_import_json` now carries `Some(language)` (wire-`Entr`→`Some(English)`, wire-`Mnem`→`Some(wire)`, overlay-phrase→`Some(--language)`), so `synthesize_descriptor`'s `unwrap_or(run_language)` can never fabricate or strip. No C3 (advisory over-warns at worst). The 5-site emit sweep + export-wallet/verify-bundle no-emit confirmation + caller audit show the only `--language`-resolving entropy paths are fresh CLI sources (typed phrase / raw entropy / convert) — the intended R0-reviewed behavior — distinct from wire-`entr`-card passthrough (pinned agnostic). The emit-language invariant is airtight. Cleared to ship (after the Step 9 override-removal + `cargo metadata --locked` re-lock gate).
