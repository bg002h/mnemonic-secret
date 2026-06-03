# Phase 2 R0 — ms K-of-N — round 0

**Reviewer:** opus architect (mandatory per-phase R0 of Phase 2, ms-cli K-of-N)
**Diff:** `5f1a761..b70e930` (4 commits: edb04c4 / 3ff7e02 / 085c1d7 / b70e930)
**Branch:** `ms-v0.2-kofn` · **HEAD:** `b70e930e776238816e3f924a21c9d45b30111d12`
**Verified against:** `design/IMPLEMENTATION_PLAN_ms_v0_2_kofn.md` Tasks 2.0–2.3 + `design/SPEC_ms_v0_2_kofn.md` §3.
**Gate:** HARD — 0C/0I before Phase 3.

**Verdict:** GREEN (0C / 0I)

---

## Critical
None.

## Important
None.

## Minor

- **M1 — version bump / CHANGELOG (Task 2.4 Step 1) not yet in the diff.** `crates/ms-cli/Cargo.toml:3` is still `0.6.0`; no CHANGELOG / Cargo.lock entry in `5f1a761..HEAD`. This is BY DESIGN of the plan's sequencing — Task 2.4 Step 2 IS this R0, and Steps 1/3 (bump + close commit, NO tag) follow GREEN. NOT a code-correctness concern and explicitly out of the reviewed-diff scope. **Reminder only:** the Phase-2 close commit MUST bump ms-cli 0.6.0→0.7.0 + CHANGELOG + relock + stage Cargo.lock (per `feedback_phase_6_cargo_lock_stage_with_version_bump`) before Phase 3 begins.

- **M2 — `ms combine --to entropy --json` drops the mnem language code.** `combine.rs::emit_entropy` (`combine.rs:124-142`) emits `language: None` even for a recovered `mnem` secret (`kind: "mnem"` is still present). The full language is recoverable only via `--to phrase`/`--to ms1`. This is an intentional raw-entropy wire-shape choice (entropy mode = bytes only), is internally consistent (entr also has no language there), and the `--json` wire-shape is explicitly NOT gated (SPEC §6 / FOLLOWUP). No action required; flagged for the P4 `--json` wire-shape FOLLOWUP so GUI/downstream consumers self-update.

- **M3 — `share_id`/`share_header` re-parse shares by `rfind('1')` byte-slicing** (`split.rs:100`, `cli_split.rs:20`) rather than via a codex32 parse. Defensive (empty-string fallback, no panic) and correct for all valid ms1 share lengths (≥48 chars). Acceptable; the value is also surfaced authoritatively from the JSON in the round-trip tests.

---

## Confirmations

**1. `resolve_secret_payload` extraction (Task 2.1) keeps `encode` byte-/output-identical — CONFIRMED.**
- `encode::run` (`encode.rs:117-151`) was refactored to call `resolve_secret_payload(phrase, hex, language)` (`encode.rs:64-114`) and reconstruct the JSON/card `language` field from the **2nd tuple element** (`language_for_card`), passed to both `emit_json` and `emit_text`. The auto-route (`language != English && phrase.is_some()` → `Payload::Mnem`; else `Payload::Entr`) is byte-for-byte the OLD branch (`encode.rs:105-112`).
- The display `entropy` is re-derived as `Zeroizing::new(payload.as_bytes().to_vec())` (`encode.rs:141`). Verified `Payload::as_bytes()` (`ms-codec/src/payload.rs:102-107`) returns the **bare entropy** (no prefix / no language byte) for BOTH `Entr` and `Mnem` → `word_count` and `entropy_hex` are identical to the pre-refactor `mnemonic.to_entropy()` / `parse_hex_entropy` bytes. No regression in `ms1`, `language`, `word_count`, or `entropy_hex`.
- `language_for_card` semantics preserved: `Some(language.as_str())` for ANY phrase (incl. English), `None` for `--hex` — matches the old code exactly. The English-phrase card still shows `language: english`; `--hex` still omits it.
- The pinning test `encode_output_unchanged_after_split_refactor.rs` asserts the FULL emitted strings (text + json, not substrings) for {english phrase, japanese phrase, hex}. Golden bytes verified plausible: english entropy `00…00` (abandon×11+about), japanese `ab×16`, hex `ab×16`; the `entropy_hex` / `language` / `word_count` fields all match. Independent runtime probe of the three shapes is implicit (the test passes in the GREEN suite).
- No new secret leak: entropy stays `Zeroizing` end-to-end; the `Payload`'s inner `Vec` is the pre-existing non-Zeroizing copy (FOLLOWUP `rust-codex32-zeroize-upstream`), unchanged in count vs the old code.

**2. `ms split` correctness + output-class + zeroize — CONFIRMED.**
- Builds the `Payload` via the shared helper (`split.rs:66`), ignoring the 2nd tuple element (it re-derives kind/language from the `Payload` itself — `split.rs:80-85`), so a non-English phrase splits as `mnem` (language in the secret-at-S wire bytes). Runtime-confirmed: JA phrase → `kind: mnem, language: japanese`, 3 distinct-index shares, shared id.
- Validation: `Threshold::new(args.k)?` rejects k∉2..=9 (`split.rs:74`); `encode_shares` rejects n∉k..=31 (`split.rs:75`). Tests assert k=1, n<k, n=32 → exit 1 (`cli_split.rs:111-139`).
- Output-class `PrivateKeyMaterial` advisory emitted unconditionally before `Ok(0)` (`split.rs:94`); asserted (`cli_split.rs:64`).
- Zeroize: `phrase_arg`/`hex_arg` `Zeroizing`-wrapped at entry (`split.rs:59-61`); the secret wire bytes inside `encode_shares` come from `payload_wire_bytes` → `Zeroizing<Vec<u8>>` (`envelope.rs:199`), and the CSPRNG filler is `Zeroizing` (`shares.rs:139`).
- `--json` shape `{schema_version,shares,k,n,id,kind,language?}` (`format.rs:SplitJson`) — `language` `skip_serializing_if=None` (omitted for entr). Sensible; tests pin it (`cli_split.rs:74-109`).

**3. `ms combine` correctness — CONFIRMED.**
- `combine_shares(&shares)` → `(Tag, Payload)` (`combine.rs:58`); shares `Zeroizing`-wrapped (`combine.rs:52`). Recovered entropy `Zeroizing` (`combine.rs:62-74`); `PrivateKeyMaterial` advisory emitted (`combine.rs:82`).
- `--to phrase` renders via the on-wire language code (`CliLanguage::from_code(*wire_code)`, `combine.rs:69`) — runtime-confirmed JA phrase + `language: japanese` recovered; entr → English. `--to entropy` → hex. `--to ms1` re-encodes via `ms_codec::encode(Tag::ENTR, payload)` = the threshold-0 single-string path — runtime-confirmed the recovered ms1 is `ms10entrsq…` (threshold char `0`, a valid unshared single-string) and re-decodes back to the phrase (`cli_combine.rs:80-103`).
- Errors mapped (Task 2.0): below-threshold → "not enough shares" (`cli_combine.rs:126`); index-`s` → exit 2 + "secret share" (`cli_combine.rs:138`); duplicate → "repeated" (`cli_combine.rs:166`). All via `From<ms_codec::Error>` + `codex32_friendly`.

**4. Task 2.0 error mapping + exit codes — CONFIRMED + SANE.**
- `IsShareNotSingleString` / `SecretShareSuppliedToCombine` → `CliError::FormatViolation` → exit **2** (`error.rs:206-229`; exit-2 via `exit_code()` `error.rs:50`). Consistent with the existing FormatViolation/ms1-shape class (WrongHrp, ReservedPrefixViolation, etc.).
- `InvalidThreshold(k)` / `InvalidShareCount{k,n}` → `CliError::BadInput` → exit **1** (`error.rs:234-241`). Justified in-code: there is no exit-64 `CliError` variant; clap parse-level 64s never reach `From<ms_codec::Error>`; BadInput is the existing-taxonomy fit. Sane.
- The single wildcard `other => BadInput("unhandled …")` (`error.rs:246`) is now correctly preceded by all 4 new arms. Unit tests pin each mapping + the `Codex32(ThresholdNotPassed)` → friendly route (`error.rs:335-386`).
- **No other ms-cli site swallows the new variants:** the only other `match`-on-`ms_codec::Error` is `verify.rs:65` (matches `ReservedTagNotEmittedInV01` only, own context); `repair.rs` propagates via `?`. All new variants funnel through the single `From` impl at `error.rs:132`.

**5. Contract re-spec (Task 2.0) — PROVEN CORRECT, not a weakened re-capture — CONFIRMED.**
- Old `decode_rejects_threshold_not_zero.rs` DELETED (verified absent). New `decode_routes_share_to_is_share_not_single_string.rs` builds a GENUINE threshold-2 share via `Codex32String::from_seed("ms", 2, "tst7", Fe::A, [0x00||0x00×16])` and asserts kind `IsShareNotSingleString` + exit **2** + "ms combine" message (text) + JSON `{kind, exit_code:2, details.threshold:"2", details.index:"a"}`.
- This matches the Task-2.0 mapping (exit 2), NOT the prior exit-1 "unhandled variant" wildcard — i.e. the test asserts the NEW correct behavior, proven against the share semantics (a threshold-2 string genuinely IS one share), per `feedback_recapture_golden_only_when_correct`.
- `ms-codec/tests/negative.rs:50` (`rule_3_threshold_2_routes_to_is_share`) corroborates at the codec layer: decode → `IsShareNotSingleString { threshold:'2', index:'a' }`, NOT `ThresholdNotZero`.
- Grep of ALL tests for residual `ThresholdNotZero` assertions: only the still-live v0.1 error arm (`error.rs:142`, for threshold values that are neither 0 nor 2..9 — correctly retained), the codec's own `ThresholdNotZero` variant/`Display` (`error.rs:20/123`), the `envelope.rs:125` `other =>` reject arm (v0.1, retained), and DOCUMENTING comments asserting the v0.2 relaxation. No test asserts the OLD reject.

**6. `ms inspect` of a share (Task 2.3) — CONFIRMED.**
- `inspect.rs::run` adds an `is_share(&report)` branch (`inspect.rs:36-48`) BEFORE the `analyze()` rule-walk: emits `kind: share` + threshold/id/index + checksum_valid, `would_combine: true`, NO FAIL / threshold-not-zero, and SUPPRESSES `prefix_byte` / `payload_bytes` / the entr-mnem kind. `is_share` = `(2..=9).contains(&report.threshold)`.
- Verified `report.threshold` is the NUMERIC digit (`inspect.rs:95` `threshold: fields.threshold_byte - b'0'`), so the `2..=9` predicate is correct (not comparing against ASCII 50..57).
- v0.1 single-string (threshold 0) is NOT a share → falls through to `analyze()` unchanged. Regression test `inspect_v01_single_string_still_works` (`inspect_share.rs:86-101`) asserts the unchanged "OK: would decode" + "threshold: 0" output.
- Share-read tests assert kind/threshold/id/index present, FAIL/threshold-not-zero/prefix_byte/payload_bytes absent, in both text and JSON (`inspect_share.rs:23-84`).

**7. m1 (zeroize-lint row) + m2 (filled stub) — CONFIRMED.**
- m1 (`lint_zeroize_discipline.rs` 4→5 rows): the new `shares.rs` row's evidence anchors `"let mut filler: Zeroizing<Vec<u8>>"` and `"let data: Zeroizing<Vec<u8>> = Zeroizing::new(secret.parts().data())"` exist VERBATIM at `shares.rs:139` and `shares.rs:240` respectively, and both sites are genuinely `Zeroizing` (CSPRNG defining-share filler; recovered secret-at-S bytes). Row-count assertion updated 4→5. Not a vacuous anchor.
- m2 (`envelope.rs` `threshold_1_is_unconstructible_so_never_reaches_discriminate`): asserts REAL behavior via two empirical prongs against pinned codex32 0.1.0 — (a) `from_seed(.., 1, ..)` → `InvalidThresholdN(1)` (cannot mint); (b) hand-forging a valid threshold-2 string's threshold char to '1' → `from_string` → `InvalidChecksum` (the threshold char is BCH-covered). Confirms threshold='1' is unconstructible and the `other => ThresholdNotZero` arm is unreachable for '1'. Not a stub asserting nothing.

**8. Plan-deviation adjudication — ACCEPTABLE.**
- `ms split` / `ms combine` use `--phrase`/`--hex` (mirroring `ms encode`), NOT the plan's aspirational positional `[<source>]`/seedqr. Correct: `ms encode` itself has no positional/seedqr source, and the shared `resolve_secret_payload` helper is the single source-resolution path — inventing a positional grammar for split alone would diverge from encode. Mirroring encode is the right call.
- `ms combine --to` value-enum (`CombineTo {Phrase,Entropy,Ms1}`, kebab-case) is a new GUI dropdown — correctly flagged for P4 schema-mirror (SPEC §6; `mnemonic-gui/src/schema/ms.rs`). Fine to defer to P4.

**9. Anything masked — NONE FOUND.**
- No test weakened (the one re-spec is proven-correct, §5 above; v0.1 regression coverage added for inspect).
- No secret on a non-Zeroizing path introduced by this diff (encode entropy `Zeroizing`; split/combine secret bytes `Zeroizing`; share-set strings `Zeroizing`; the only non-Zeroizing secret copies are the pre-existing `Payload`-inner `Vec` and the `bip39::Mnemonic`, both already FOLLOWUP-tracked, count unchanged).
- No new `#[allow(...)]` in the diff (grep clean).

## Independent gate re-run (this review)
- `cargo test -p ms-codec --no-fail-fast` → **127 passed / 0 failed** (0 ignored).
- `cargo test -p ms-cli --no-fail-fast` → **0 failed** across all bins (3+2 ignored = sibling-gated `#[ignore]`).
- `cargo clippy -p ms-codec -p ms-cli --all-targets -- -D warnings` → clean (forced recompile via `touch main.rs`; exit 0, no warnings).
- Runtime probes (debug `ms` bin): entr split→combine recovers `00…00` + `--to ms1` is threshold-`0`; JA split→combine recovers the exact Japanese phrase with `kind:mnem language:japanese`; C1 index-`s` rejected exit 2.

## Verdict justification (not a rubber-stamp)
All 4 Tasks (2.0–2.3) realize the SPEC §3 surface with proven-correct tests, the byte-identity encode refactor is verified at both the `Payload::as_bytes` source level and the full-string pinning test, the error taxonomy is consistent and the wildcard is correctly fronted, no variant is swallowed elsewhere, the C1 security rule (index-`s` reject) is double-covered (unit + integration) and runtime-confirmed, secrets stay on `Zeroizing` paths, and the m1/m2 P1-R0 minors are substantive (real anchors / real proof). The only open items are the post-GREEN release-prep version bump (M1, plan-sequenced after this R0) and two ungated-`--json` wire-shape notes for P4 (M2). None blocks Phase 3.

**GREEN — 0 Critical / 0 Important. Phase 3 may proceed after the Task 2.4 version-bump close commit.**
