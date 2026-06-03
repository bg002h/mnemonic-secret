# Plan R0 — ms K-of-N — round 1

**Reviewer:** opus architect (mandatory pre-impl plan gate; re-dispatch after every fold). Round 0 was RED (2C/3I + 6 minor); this round verifies the folds in `design/IMPLEMENTATION_PLAN_ms_v0_2_kofn.md` @ branch `ms-v0.2-kofn` (`47e7942`) against ground truth re-read this session: ms-cli `error.rs` (wildcard confirmed `:201`), `cmd/encode.rs` (source-parse inline `:59-101`, `language_for_card` `:65/73/105/107`), `cmd/inspect.rs` (analyze `:49-114`, garbage fields `prefix_byte`/`payload_bytes`/`kind` emitted `:155-158/185-188`), envelope.rs (discriminate `:95-155`, threshold gate `:105-109`, tail `:131-152`, package `:166-194`), decode.rs (`from_string`→`discriminate` order `:52-55`), consts.rs (THRESHOLD_V01 `:20`), codex32 0.1.0 `interpolate_at` short-circuit (line 259 `if indices[i]==target { return Ok(shares[i].clone()) }`, BEFORE the RepeatedIndex inner-loop check) + `parts_inner` threshold map (rejects bytes ∉{0,2..9} with InvalidThreshold), toolkit `friendly.rs:80` + `error.rs:364` wildcards (Task 3.3's instance), toolkit `cmd/slip39.rs` grammar (`--from phrase=`/`--group-threshold`/`--group`).

**Verdict:** RED (0C / 1I)

The four round-0 folds that matter most (C1, C2, I1, I2) are correctly and completely applied. One Important remains: the I3 fold pinned a `resolve_secret_payload` signature that DROPS the data `encode::run` needs to preserve its own observable output — directly contradicting the "keep `encode` byte-identical" instruction stated on the same plan line. This is a new drift introduced BY the I3 fold (round-0 I3's own recommendation carried the dropped element). Small surgical edit; one more fold round.

## Critical
(none)

## Important

### I-r1 — the I3 fold's pinned `resolve_secret_payload(...) -> Result<Payload>` signature is insufficient to keep `encode` byte-/output-identical; it drops `encode::run`'s `language_for_card`.
Plan:157 pins `pub(crate) fn resolve_secret_payload(phrase, hex, language: CliLanguage) -> Result<Payload>` and instructs "refactor `encode::run` to call it (keep `encode` byte-identical)." But `encode::run` (encode.rs:65) currently binds `let (entropy, language_for_card): (Zeroizing<Vec<u8>>, Option<&str>)` and uses `language_for_card` for BOTH the stderr engraving card (emit_text, encode.rs:107/166-167) AND the `--json` `language` field (emit_json, encode.rs:105/138). The distinction it carries is NOT recoverable from a bare `Payload`:
- English `--phrase` → `Payload::Entr(entropy)` + `language_for_card = Some("english")` (encode.rs:73).
- `--hex` → `Payload::Entr(entropy)` + `language_for_card = None` (encode.rs:77).
Both collapse to the SAME `Payload::Entr(entropy)`. A helper returning only `Payload` cannot tell `encode::run` which case occurred, so the refactored `encode` would either drop or wrongly populate its card/json `language` field for the English-phrase path — an observable-output regression, violating the same line's "keep `encode` byte-identical." (round-0 I3 actually recommended `-> Result<(Payload, Option<&str>)>`, line 83; the fold dropped the `Option<&str>` tuple element.)

Also, `word_count` (encode.rs:102) and `--json` `entropy_hex` (encode.rs:144) are computed from `entropy.len()` / the entropy bytes; for the **mnem** path those live inside `Payload::Mnem{entropy}` (recoverable), but for the entr path they're inside `Payload::Entr` (recoverable) — so entropy itself survives a `Payload` return; only `language_for_card`'s phrase-vs-hex bit is lost. The fix is therefore narrow.

**Fix (pin in Task 2.1):** restore round-0 I3's tuple return — `pub(crate) fn resolve_secret_payload(phrase, hex, language) -> Result<(Payload, Option<&'static str>)>` (the `Option<&'static str>` = the card-language: `Some(language.as_str())` for a phrase, `None` for hex) — and have `encode::run` reconstruct `language_for_card` from it. Add a Task-2.1 sub-assertion that `encode`'s text+json output is unchanged for {english phrase, non-english phrase, hex}. `split` ignores the second tuple element (it re-derives language into the share-set via the Payload). Note `CliLanguage::as_str` returns `&'static str` (language.rs:51), so the lifetime is clean.

## Minor

### m-r1 (carry-over, compile-caught) — `dispatch_payload(&c.data())` / `dispatch_payload(&secret.data())` — `Codex32String` has no `.data()`; it's `Parts::data()`.
Plan:126-127 write `c.data()` and `secret.data()`. The existing correct form is `c.parts().data()` (envelope.rs:131; codex32 `Parts::data()`). Two lines from the model code, compile-caught — but flag so the implementer copies `.parts().data()` and doesn't burn an iteration.

### m-r2 (carry-over) — `dispatch_payload` mnem-branch length guard still unstated.
Round-0 I1 recommended a `data.len()` guard on the mnem branch. Plan:126 specifies the indexing with no guard. This PRESERVES the existing `discriminate` behavior (envelope.rs:142-143 indexes unguarded, relying on the upstream ≥16-B payload floor), so NOT a regression; belt-and-suspenders, optional.

## Confirmations (folds verified)

- **C1 (combine→Tag::ENTR) — FOLDED CORRECTLY + COMPLETE.** Plan:127 returns `(Tag::ENTR, dispatch_payload(&secret.data())?)`, explicitly discards the random id, and states "Do NOT call `discriminate` here." The round-trip test (plan:120) asserts `(Tag, Payload) == original`; `encode_shares` always tags `Tag::ENTR` ⟹ pins `Tag::ENTR`. codex32 `interpolate_at` short-circuit re-verified at line 259 BEFORE the RepeatedIndex check — index-`s` pre-reject (plan:124 (ii)) closes it.
- **C2 (ms-cli error map) — FOLDED CORRECTLY.** Wildcard confirmed `ms-cli/src/error.rs:201`. Task 2.0 adds 4 arms + routes `Codex32(<share>)` via `codex32_friendly`. Task 2.0 + Task 4.1 cross-bound to the same exit code (recommend exit 2/FormatViolation to preserve the contract test with least churn).
- **I1 (dispatch_payload) — FOLDED CORRECTLY + SOUND.** Header-gate-free helper in envelope.rs; discriminate tail refactored to call it; combine_shares calls it; no flag. b'0' byte-behavior preserved (the factored tail is the exact prefix→Payload dispatch). decode's only discriminate call (decode.rs:55) is post-`from_string`, so the relaxed threshold-match `other =>` arm is unreachable (parts_inner already rejects ∉{0,2..9}) — harmless defensive arm.
- **I2 (Display-only) — FOLDED CORRECTLY.** ms-codec error.rs has only Display + From<codex32::Error>; no exit_code/kind. Correct.
- **m1 (getrandom 0.3 `fill`) — FOLDED.** Correct API/version + add to Cargo.toml.
- **deterministic index selection — FOLDED.** Fixed canonical-order non-`s` indices; only payloads random.
- **m2 (share-inspect garbage-field suppression) — FOLDED CORRECTLY.** Verified the garbage fields at inspect.rs:155-158/185-188; share branch surfaces only threshold/id/index.
- **m3 (P3 grammar-match) — FOLDED CORRECTLY.** slip39 uses `--from phrase=`/`--group-threshold`/`--group`; plan defers to impl-time verification, no longer hard-codes `-k`/`-n`.

## New-drift scan
- Symbol-creation closure clean: `dispatch_payload` (Task 1.4, used same task), `payload_wire_bytes` (1.2→1.3), `resolve_secret_payload` (2.1), `Threshold`/`encode_shares`/`combine_shares` (1.1/1.3/1.4 → 2.x/3.x). No task references a symbol no task creates. Task 2.0 after P1 variants — fine.
- The ONLY drift the folds introduced is I-r1.

**Disposition:** RED — 0C / 1I. Fold I-r1 (restore the `resolve_secret_payload` tuple return + add the encode-output-unchanged assertion). Fix m-r1 (`.parts().data()`) inline; m-r2 optional. Re-dispatch round 2 — expect GREEN. Implementation BLOCKED until 0C/0I.
