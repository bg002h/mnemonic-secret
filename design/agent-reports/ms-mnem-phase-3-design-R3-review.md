# Phase 3 (faithful) R0 design review — ms mnem — round 3

**Verdict:** GREEN (0C/0I)

Base SHA verified: toolkit `e572888` (current HEAD). Every Phase-3 citation re-grepped live against this SHA. I6 + I7 folds confirmed against source; the M5/M8 notes are present; the final sibling-drift sweep found NO new silent-mis-derive / language-loss / unhandled-Mnem site. Cleared for toolkit implementation.

## Critical
- *(none)*

## Important
- *(none)*

## Minor
- **[m1 — citation drift, ±1-2 lines, self-correcting]** Several `Payload::Entr(...)` destructuring arms are cited one line off (the plan cites the `_ => Err`/`continue` line; source has `Payload::Entr(...)` one line above). verify_bundle: `Ok((_tag, Payload::Entr(bytes))) => bytes` at `:2401`, `Ok(_) | Err(_) => continue` at `:2404`, derive `:2411` (plan's `:2400-2404` range correctly brackets it). bundle import-json: `Payload::Entr(bytes) =>` `:1619`, `_ => return Err` `:1620` (plan cites `:1620`). convert: `Payload::Entr(bytes) =>` `:1453`, `_ => return Err` `:1454` (plan cites `:1454`). Exact-or-±1; compile-forced exhaustive matches, cannot mislead. Minor.
- **[m2 — repair match is ALREADY Mnem-tolerant]** `repair.rs:820`/`:892` already bind `Ok((_tag, _payload, corrections))` — `_payload` is a wildcard, so a `Mnem` payload flows through with ZERO match change (repair is round-trip-only: rebuilds the corrected string from `corrections`, discards the payload). The load-bearing ask (the Japanese-mnem ≤4-symbol repair test) stands; "relax the match" phrasing is a harmless no-op. Minor.
- **[m3 — emit-site line shorthand]** Step 5 cites per-slot emits as `synthesize.rs:291/778`; the actual `Payload::Entr` lines are `:293` (`synthesize_descriptor`, fn @ `:229`) and `:784` (`synthesize_unified`, fn @ `:697`). Step 5 ALSO lists the precise set `152/293/443/784`, so the implementer has the exact lines. Minor.

## Confirmations
- **[I6] ✓ FIXED.** The rewritten carrier paragraph + the bundle import-json bullet correctly separate the two flows: **import-json is EMIT-ONLY** — `bundle_run_from_import_json` decodes at `bundle.rs:1613`, stores entropy at `:1626`, does NOT re-derive (xpub from the mk1 chunk, `json_envelope.rs:364`); the ONLY language reader is the emit `synthesize_descriptor` (`synthesize.rs:293`, fn @ `:229`). The **`resolve_slots`/hex flow is separate** — `resolve_slots` (`bundle.rs:448-709`) has NO `ms_codec::decode`; derive at `:619`/`:626` reads `lang = language.unwrap_or_default()` from raw hex, emits via `synthesize_unified` (`:784`); slots are `language: None`. The `slot.language.unwrap_or_else(|| args.language().into())` resolution is stated identically in the carrier paragraph, Step 4 bundle bullet, and Step 5. No remaining place implies a re-derive on import-json.
- **[I7] ✓ FIXED.** Step 4 bullet 3 now quotes the source match, calls out the silent `continue` skip (verified at `verify_bundle.rs:2401`/`:2404`), and mandates widening to BIND both `Entr(bytes)` and `Mnem { language, entropy }` keeping `Err(_) => continue`, then per-cosigner language resolution (the `:2411` derive). The mixed-language multisig verify-bundle test is in the Step 4 + Step 5 test lists.
- **[M5] ✓ present.** Step 3 carries the all-10-label pin (test c) + bip39 external anchors (test d: codes 1/4/5) + bip39-identity (a) + round-trip (b). The order-divergence is documented and the string-equality bridge forbidden.
- **[M8] ✓ present.** The carrier paragraph names the ~28-constructor cascade and states "all `None` EXCEPT `bundle.rs:1626`." Spot-checked bundle constructor sites exist; compile-forced.
- **[sibling-sweep] ✓ EXHAUSTIVE — nothing new.** All 7 `Payload::Entr(`-binding sites classified/accounted (emit `convert.rs:1217` + test-only `synthesize.rs:152`; decode/extract `convert.rs:1453`, `bundle.rs:1619`, `silent_payment.rs:137`, `overlay.rs:128`; I7 silent-skip `verify_bundle.rs:2401`). No OTHER silent-skip: verify_bundle `:1227`/`:1620` bind `Ok(_)` and route through byte-string-identity comparison (no derive, `Mnem` passes cleanly); `inspect.rs:171` binds `(tag, payload)` whole (Step 6). All `from_entropy_in` ms1-reachable sites in Steps 4/5; non-ms1 sites correctly OUT of scope (`bip85.rs:90`, `seedqr.rs:200`, `slip39.rs:671`, `seed_xor.rs:208/350`, `bundle.rs:1369`). Step 6 KEEPS the §6.3 advisory at slip39 + convert-raw-entropy. `decode_with_correction` sites wildcard-bind (round-trip-only). No omitted emit site.

---
**Gate decision: GREEN (0C / 0I).** The I6 dataflow-prose fold and the I7 verify_bundle bind-both-arms fold are both correct against source `e572888`. M5 + M8 notes present. The exhaustive sibling-drift sweep over every `Payload::Entr`-binding arm, every `from_entropy_in` derive site, every `ms_codec::decode`/`decode_with_correction` site, and every emit site found NO new silent mis-derive, language-loss, or unhandled-`Mnem` match. The three Minors are ±1-2-line citation polish / a no-op match-relax phrasing — all compile-forced and non-misleading. **Toolkit Phase-3 implementation is CLEARED to begin.**
