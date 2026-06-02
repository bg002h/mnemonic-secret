# End-of-cycle R0 review — ms mnem — round 1

**Verdict:** RED (1C / 0I)

## Critical / Important / Minor

### Critical

**C2 (NEW, introduced by the C1 fold `80f78fc`) — `bundle --import-json <envelope-carrying-an-English-entr-card> --language <non-English>` silently RE-EMITS the English `entr` card as a non-English `mnem` card, corrupting the wire language with NO advisory.** Same hazard class as the round-0 C1 (silent wrong-language card / advisory wrongly suppressed), re-opened in the opposite direction on a FOURTH emit path the fold perturbed.

Root cause: the C1 fold changed `synthesize_descriptor`'s emit from `match c.language { Some(non-en) => Mnem; _ => Entr }` to `c.language.unwrap_or(run_language)` (synthesize.rs:298). Correct for the descriptor-@N path, but `bundle_run_from_import_json` (bundle.rs:1770) now passes `run_language_import = args.language.unwrap_or_default()`. In that function a slot's `language: None` overloads TWO distinct meanings:
  1. Wire-decoded **entr** card from the envelope (bundle.rs:1660-1661 → `(entropy, None)`). Faithful, language-agnostic passthrough — MUST stay `Entr` regardless of `--language`.
  2. Overlay-derived from a user `--slot @N.phrase=` on a watch-only slot (bundle.rs:1739, `None`). SHOULD inherit `--language` and emit mnem.
After the fold, BOTH `None` cases inherit `run_language`. Case 1 is corrupted.

Reproduced against the built binary (override active): a watch-only single-sig concrete-descriptor bundle, inject the English entr card `ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f` (50, agnostic) into `bundle.ms1[0]`, wrap as a `bsms` envelope.
- `bundle --import-json - --network mainnet --json` → ms1[0] = `ms10entrsqqq…cj9sxraq34v7f` (50, entr). CORRECT control.
- `bundle --import-json - --network mainnet --language japanese --json` → ms1[0] = `ms10entrsqgqsqqqqqqqqqqqqqqqqqqqqqqqqqj9tawneveyd9j` (51-char **mnem, tagged Japanese**). **WRONG — English entr corrupted to Japanese mnem; NO §6.3 advisory.**

Pre-fold proof (regression, not pre-existing): at `80f78fc~1` (dcf9782) the `_ => Entr` arm emitted `Entr` for any `None`-language slot irrespective of run language → faithful. The fold introduced the corruption.

Advisory stays silent (same shape as C1): emit_unified (bundle.rs:752-756) computes `s.language.unwrap_or(run_lang)`; for the wire entr slot that is `None.unwrap_or(japanese)=japanese≠English` → suppressed — and synthesize_descriptor genuinely emits mnem-ja, so model + emit "agree" on the WRONG behavior.

Impact: a wallet imported from a 3rd-party blob whose ms1 is an English/agnostic entr card, re-bundled with `--language <non-English>` for any reason, gets a card whose embedded language tag is a lie → language-faithful recovery reconstructs the wrong wordlist's phrase. Critical.

Why missed: the only import-json mixed-language test (`mixed_language_import_json_re_emits_mnem_ja_and_entr`, cli_mnem_emit_preserve.rs:248) re-bundles withOUT `--language` (test line ~304 passes only `--network mainnet`), so the entr slot's `None.unwrap_or(English)=English` path is the only one exercised. The `--import-json` + `--language <non-English>` combination is untested → full toolkit suite (2597 passed) green over a corrupting path.

Fix prescription (confined to bundle_run_from_import_json — make slot.language airtight so NO entropy slot is `None`): (a) wire-decode loop (bundle.rs:1660-1677) `Payload::Entr` arm → `language = Some(bip39::Language::English)` (carry the "entr = agnostic/English-default" fact); `Payload::Mnem` arm → already `Some(wire)`; overlay loop (bundle.rs:1739) → `Some(args.language.unwrap_or_default().into())`. Then run_language passed is moot (overridden by the now-`Some` slots). Audit EVERY entropy-bearing slot construction in this caller to confirm none leaves `language = None`. Add regression tests: `bundle --import-json <English-entr-envelope> --language japanese` MUST emit the byte-identical 50-char entr card (assert length 50 AND `Payload::Entr`); contrast: overlay `--slot @N.phrase=<ja>` + `--language japanese` → 51-char mnem-ja. Re-dispatch for round 2.

### Important
None.

### Minor
- M1/M2 from round 0 stand (pre-existing convert over-warning; one-directional language round-trip test). Out of scope.

## Confirmations (incl. the COMPLETE emit-site sweep list + suite results)

- **C1 (round-0) fold is CORRECT and symmetric.** synthesize_descriptor:298 `c.language.unwrap_or(run_language)` is logic-identical to synthesize_unified:831. The three new descriptor-@N tests pass and would fail pre-fix (assert `len==51` + `Payload::Mnem{WIRE_JAPANESE}`). Round-trip + advisory-suppression tests strong. English golden + v0.1 entr corpus + `mixed_language_import_json_re_emits_mnem_ja_and_entr` all pass.

- **COMPLETE ms1-emit-site sweep** (independent grep `ms_codec::encode`/`Payload::Entr|Mnem` across crates/mnemonic-toolkit/src/): 5 construction sites, 2 CLI-reachable.
  1. synthesize.rs:150 `synthesize_full` — `#[allow(dead_code)]`, test-only callers. Not a footgun.
  2. synthesize.rs:308 `synthesize_descriptor` — `emit_lang = c.language.unwrap_or(run_language)`. Per-card correct; the DEFECT is the import-json CALLER passing the wrong run_language (C2).
  3. synthesize.rs:469 `synthesize_multisig_full` — `#[allow(dead_code)]`, test-only. Uses `seed_mnemonic.language()` (always carries language). Correct.
  4. synthesize.rs:841 `synthesize_unified` (CLI hot path, template mode) — `s.language.unwrap_or(run_language)`. Correct.
  5. convert.rs:1235 `convert --to ms1` — `if language == English { Entr } else { Mnem }`, per-source. Correct.
  All decode-side matches (verify_bundle.rs:2414-2415, silent_payment.rs:143-144, inspect.rs:193/300, overlay.rs:132-135, convert.rs:1473-1477, bundle.rs:1661-1662) read the wire language, out of scope for emit-strip.

- **synthesize_descriptor caller audit (6 non-test):** descriptor-@N (bundle.rs:1465, `args.language`) ✓; concrete-descriptor (bundle.rs:1543, English) ✓ MOOT (`descriptor_concrete_to_resolved_slots` always sets `entropy: None`); verify_emit_from_expected (verify_bundle.rs:923, `args.language`) ✓; import_wallet (import_wallet.rs:1398, English) ✓; **import-json (bundle.rs:1771, `args.language`) is WRONG — C2.** Test sites pass English ✓.

- **Derive-path re-sweep (`from_entropy_in`/`Language::English`):** no ms1-reachable derive hardcodes English. All use per-card/per-slot wire language (derive_slot.rs:77/138, seed_intake.rs:175, silent_payment.rs:114, overlay.rs:178, verify_bundle.rs:2449, seed_xor/slip39 per-share). English literals in seedqr.rs (English-only format), test-cfg sites, and the language.rs tables are out of scope. The C1 fix did NOT break or add a derive-site English hardcode.

- **Carrier-semantics:** descriptor-@N resolved_slots reconstruction (bundle.rs:1484-1488) sets `Some(c.language.unwrap_or(run_language))` for entropy slots → emit_unified advisory agrees for THAT path (correct). The model IS abused in bundle_run_from_import_json (None overloaded for wire-entr-passthrough + overlay-derive) — the proximate cause of C2.

- **Suites + clippy (against the override):** mnemonic-toolkit `cargo test --no-fail-fast` = 2597 passed / 0 failed / 12 ignored; clippy `--all-targets -D warnings` clean. mnemonic-secret `cargo test -p ms-codec -p ms-cli` = 245 passed / 0 failed. ALL GREEN — exactly why C2 is dangerous: no test covers `--import-json` + `--language`.

**Gate: RED — do not ship.** The C1 fold is correct in isolation but introduced C2 (a fourth corrupting emit path) on `bundle --import-json` + `--language`. Fix per the prescription (make import-json slot.language airtight — no `None` entropy slot; pin both the corruption-repro and the overlay-contrast tests), then re-dispatch for round 2.
