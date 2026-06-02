# End-of-cycle R0 review — ms mnem — round 0

**Verdict:** RED (1C / 0I)

## Critical / Important / Minor

### Critical

**C1 — `bundle --descriptor <@N> --slot @N.phrase=<non-English>` (and `@N.entropy=`) silently emits a language-stripped `entr` card AND suppresses the §6.3 advisory — the exact footgun this cycle exists to fix, re-opened on the descriptor-placeholder emit path.**

Root cause: there are **three** ms1-emit paths in `bundle`, but the Phase-3 design only modeled two.

- `resolve_slots` (phrase/entropy via `--template`) → `synthesize_unified` — emit resolves `s.language.unwrap_or(run_language)` (`synthesize.rs:828`). Non-English ⇒ `Mnem`. **Correct.**
- `bundle --import-json` → `synthesize_descriptor` — `slot.language` populated from the wire (`bundle.rs:1658`). **Correct.**
- **`bundle --descriptor <…@N…>` + `--slot @N.phrase=`/`@N.entropy=` → `bundle_run_unified_descriptor` → `synthesize_descriptor`.** Here the slots are built with `language: None` (`bundle.rs:1426`, re-cloned `:1482`), and `synthesize_descriptor` (`synthesize.rs:295`) maps `None ⇒ Payload::Entr` with **no `run_language` fallback**. The phrase is parsed/derived correctly under `--language` (`bundle.rs:1316-1321`), but the emitted ms1 is `entr` — the wordlist language is **lost on the wire**.

Compounding: the advisory at `emit_unified` (`bundle.rs:750-765`) models the emit as `slot.language.unwrap_or(run_lang)`, so for this path it computes `None.unwrap_or(japanese) = japanese ≠ English ⇒ advisory SUPPRESSED` — while the actual card is `entr`. So the language is lost **with no warning at all**.

Reproduced against the built binary (override active):
```
$ mnemonic bundle --descriptor "wpkh(@0)" --slot "@0.phrase=<ja 12-word>" \
      --language japanese --network mainnet --no-engraving-card
# ms1 (entropy, BCH-checksummed)
ms10entrsqqqszqgpqyqszqgpqyqszqgpqyqsvs7sppkfn54wj      ← 50-char ENTR, not 51-char mnem
# stderr: (only the argv-leak warning + "can spend" line — NO non-English advisory)
```
Contrast (same seed, template mode — correct): `mnemonic bundle --slot @0.phrase=<ja> --language japanese --template bip84` → `ms10entrsqgqszqgpqyqszqgpqyqszqgpqyqsz8ecw5r9pj3pk3` (51-char **mnem**).

Why missed: the Phase-3 design R2/R3 reviews (`ms-mnem-phase-3-design-R2-review.md` lines 13-16; R3 line 23 sibling-sweep) explicitly classified `bundle.rs:1369` (`@N.phrase` derive) as "raw hex → `language: None` always … correctly OUT of scope," concluding `synthesize_descriptor` was the import-json emitter only. That dataflow claim is **factually wrong** — `bundle_run_unified_descriptor:1464` routes this path's emit through `synthesize_descriptor` too. The single in-repo non-English `synthesize_descriptor` test (`synthesize.rs:1445`) uses only `bip39::Language::English`, and the new `cli_mnem_emit_preserve.rs` covers template-mode + import-json but never descriptor-`@N` + non-English phrase, so nothing catches it.

Severity = Critical per the gate's own definition ("an entr card still loses a non-English language while the advisory is wrongly suppressed" / "silently wrong-wallet"): a Japanese seed bundled this way yields a card that English-defaulted recovery silently resolves to a different seed → different wallet → funds appear lost, with no on-screen warning.

Fix (straightforward, `run_language` is in hand at both call sites): thread the run language into `synthesize_descriptor` (e.g. `synthesize_descriptor(descriptor, cosigners, privacy, run_language)`) and resolve `c.language.unwrap_or(run_language)` exactly as `synthesize_unified` does at `:828` — OR, at the `bundle_run_unified_descriptor` slot-construction sites (`bundle.rs:1420-1428` / `1476-1484`), set `language: Some(args.language().into())` for secret-bearing phrase/entropy slots (leaving watch-only/import-json untouched). Then add a non-English `bundle --descriptor wpkh(@0) --slot @0.phrase=<ja>` test asserting a 51-char mnem ms1 + round-trip. Import-json must continue to take its language from the wire (don't override slots that already carry `Some`). After the fix, re-key/verify the `emit_unified` advisory stays consistent (with the fix it'll correctly suppress because the card is now genuinely mnem).

### Important
None.

### Minor

- **M1 — `convert --from ms1=<entr> --to entropy --language french` fires a misleading "encoding a french BIP-39 seed as raw entropy" advisory** even though the source was a language-agnostic `entr` card and `--language french` was effectively ignored for the entropy output (`convert.rs:1009-1016`, source node not consulted). Pre-existing v0.37.11 behavior, not introduced by this cycle; out of scope but worth a FOLLOWUP. Low impact (over-warning, never under-warning).
- **M2 — `language.rs` test `bip39_to_wire_code_round_trip_all_10` proves the inverse only in one direction** (`bip39_to_wire_code(wire_code_to_bip39(c)) == c`). The other composition is implied transitively by the bijection over 10 entries, and `wire_code_round_trip_bip39_identity` cross-checks against `wire_code_to_cli`, so it's fully pinned in aggregate — noting only that no single test states the reverse identity literally.

## Confirmations

- **Language match tables (`language.rs`) are explicit, code/variant-keyed (no string bridge), and correct for all 10.** Chinese swap right: toolkit `SimplifiedChinese ↔ wire 4`, `Japanese ↔ wire 1`, `TraditionalChinese ↔ wire 5`. `bip39_to_wire_code ∘ wire_code_to_bip39 == identity` tested; codes ≥10 reject. External anchors + label-pin tests present.
- **Cross-repo wire mapping identical across all three layers.** `ms_codec::MNEM_LANGUAGE_NAMES` = ms-cli `CliLanguage` order (`code() = self as u8`, test-locked) = toolkit explicit tables. No divergence.
- **Per-card carrier `ResolvedSlot.language`** is `None` at all other constructors, `Some(wire_code_to_bip39(code))` ONLY at import-json mnem arm (`bundle.rs:1658`). Compile-forced.
- **Derive sites use the per-card wire language:** `seed_intake.rs:174`, `silent_payment.rs:138`, `convert.rs:1478`, `overlay.rs:135-142`, `verify_bundle.rs:2407-2438` (the former silent `continue` now BINDS `Mnem` + cross-checks under wire language with a wire-wins note). No ms1-reachable derive hardcodes English; non-ms1 sites correctly out of scope.
- **English byte-identity preserved.** `synthesize_unified:458` + `synthesize_descriptor:295` emit `Payload::Entr` unchanged for English; the v0.38.4 golden is gated + passes. (The C1 bug is that the *non-English* descriptor-@N path ALSO emits entr — wrong direction.)
- **Mixed-language multisig emits per-slot** for the import-json path (test passes). (Descriptor-@N is the C1 gap.)
- **TEMP `[patch.crates-io] ms-codec = { path = … }` (Cargo.toml:21)** clearly marked `REMOVE AT SHIP` + plan-Step-9 ref. Dep pin `ms-codec = "0.3.0"` (caret); ms-cli tag pins `v0.6.0` in install.sh:38 + manual.yml:88. Step 9 covers removal+relock.
- **Suites + clippy:** ms-codec + ms-cli = 245 tests, 0 failures, clippy clean. mnemonic-toolkit (`--no-fail-fast`, against the override) = all suites green, 0 failed, clippy clean. (The toolkit suite passing is precisely why C1 is dangerous — no test for the broken path.)
- **`--json` wire-shape:** decode/inspect `language` additions ungated by `schema_mirror` (flag-name parity only), self-updating per paired-PR rule; no new flag/subcommand ⇒ no GUI schema change. Exit codes unchanged. Zeroization intact (entropy stays `Zeroizing`; `language` carrier is a non-secret enum).

**Gate: RED — do not ship.** C1 must be fixed (thread `run_language` into the descriptor-`@N` emit path so non-English phrase/entropy slots emit `mnem`, and verify the `emit_unified` advisory stays consistent), with a regression test for `bundle --descriptor wpkh(@0) --slot @0.phrase=<non-English>`, then re-dispatch for round 1.
