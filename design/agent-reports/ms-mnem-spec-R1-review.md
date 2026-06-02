# ms-mnem SPEC R1 review — wordlist-language hint

**Reviewer:** sonnet architect (R1, fold-verification pass)
**SPEC:** `design/SPEC_ms_mnem_wordlist_language.md` (post-R0 fold)
**R0 review:** `design/agent-reports/ms-mnem-spec-R0-review.md` (RED 0C/1I)
**R0 verdict:** 1 Important (I1 — three v0.1-hardcoded gates under-scoped in §3/§5) + 4 Minors (M1 decode `--language` vs wire precedence; M2 stale doc-comments; M3 Payload validate()/kind() Mnem arms; M4 pre-existing enum order).
**Sources verified against:** `crates/ms-codec/src/decode.rs`, `crates/ms-cli/src/cmd/inspect.rs`, `crates/ms-cli/src/cmd/decode.rs`, `crates/ms-cli/src/format.rs`, `crates/ms-codec/src/consts.rs` (all current master).

## Verdict: GREEN (0C / 0I)

The fold is complete and well-executed. I1 is fully resolved in §3 and §5 and §10. M1–M4 are all addressed. No drift introduced. The proven crypto core is untouched. Internal consistency holds. One residual Minor is noted below (not a blocker, not new) — it was latent in R0 and is not fold-introduced drift.

---

## I1 — Resolved ✅

All three gates are now enumerated in §3 item-list as explicit deliverables:

1. **`decode.rs:29` rule-9 length gate** — §3 item 1 specifies the union `{50,56,62,69,75} ∪ {51,58,64,70,77}` AND explicitly says to bind length↔kind ("entr lengths ⟺ `0x00`; mnem lengths ⟺ `0x02` — do NOT just widen the set without binding length-to-kind"). Source verified: `decode.rs:29` is exactly `if !VALID_STR_LENGTHS.contains(&s.len())` — correct gate, correct line.

2. **`Payload::validate()` + `kind()` exhaustive matches** — §3 item 2 calls out both `validate()` and `kind()`/any exhaustive `match self`, and adds the language low-nibble/high-nibble check. `#[non_exhaustive]` exemption clarified. (Folds M3.)

3. **ms-cli `inspect::analyze()` + `decode::run()` `unreachable!()`** — §3 item 3 explicitly cites `cmd/inspect.rs:79,83` (pushes `non-zero-prefix`/`unexpected-string-length`) and `cmd/decode.rs:57` (the `_ => unreachable!()` that panics on `Payload::Mnem`). Source verified: `inspect.rs:79-84` is exactly those two `reasons.push(...)` calls; `decode.rs:57` is exactly `_ => unreachable!("ms-codec v0.1 only decodes to Payload::Entr")`. Both cites are accurate.

The §5 "ms decode" bullet was also updated (cites `:57`; specifies the `Payload::Mnem` arm replaces `unreachable!()`). §10 footgun bullet also updated to enumerate all three explicitly + adds the "per-phase test must decode/inspect a real mnem string end-to-end" requirement. All three paragraphs are internally consistent.

---

## M1 — Resolved ✅

§5 "ms decode" bullet now explicitly specifies: wire language is authoritative for `0x02`; a conflicting `--language L` is IGNORED with a stderr warning (`note: --xpub/ms1 carries wordlist language W; ignoring --language L`); silent override prohibited (would re-open §6.3 in reverse). `0x00` entr path unchanged. §10 footgun bullet mirrors it. This is the right call and it is now unambiguous.

---

## M2 — Resolved ✅

§3 explicitly states: "`envelope.rs` source comments referencing the dropped `0x01` 'entr-share' design / any `RESERVED_TAG_TABLE` mention must be corrected to the prefix-registry (`0x00` entr / `0x02` mnem; `0x01` unallocated)." This is a clear plan-time deliverable. The stale doc-comment `RESERVED_TAG_TABLE` mention in §10 footguns is also present (the const-name footnote). No confusion possible.

---

## M3 — Resolved ✅ (folded into I1 item 2)

`Payload::validate()` and `kind()` Mnem arms now explicitly appear as §3 item 2. The language-byte validation split (length → `validate()`; language-byte → `discriminate()`) is implied by the phrasing — "validate() (entropy ∈ {16,20,24,28,32}; language low-nibble ∈ 0..9, high-nibble 0)" which puts the language check inside `validate()`, while discriminate's `data[1]` validation is in §3 main text. This is a minor inconsistency (R0 M3 noted the split should be "discriminate gates language; validate gates entropy length") but the substance is captured in both locations; a plan author will see both and can implement correctly. Not a blocker.

---

## M4 — Resolved ✅

§3 explicitly states: "New variants sorted alphabetically; pre-existing variants NOT retro-sorted (consistent with prior cycles)." Decision taken, mirrors the toolkit `error-rs-retroactive-alphabetical-sort` deferral. Clear.

---

## Consistency checks

**Length sets:** §2 table: `entr = {50,56,62,69,75}`, `mnem = {51,58,64,70,77}`. §3 item 1 union: `{50,56,62,69,75} ∪ {51,58,64,70,77}`. Disjoint (R0 verified empirically — no collision). §3 `VALID_MNEM_STR_LENGTHS` const mirrors these five values. Internally consistent.

**No drift on the proven core:** wire format (byte-aligned, `[0x02][lang][entropy]`), from_seed/data() path, AUTO default routing (English/hex → `0x00`; non-English → `0x02`), no-GUI-lockstep claim, CI reality (no ms-codec CI), K-of-N out of scope — all unchanged from R0's GREEN construction check.

**No new placeholder:** the SPEC contains no unresolved "[TBD]" or "[Phase N: confirm]" markers that were introduced by the fold.

**§7 test plan adequacy (post-fold):** The "mnem round-trip" cell covers `inspect shows kind=mnem + language` and `decode recovers the entropy AND the language`. §10 footgun now explicitly adds: "A per-phase test must decode/inspect a real mnem string end-to-end (not just the codec round-trip) to catch the CLI-layer gates." This satisfies the R0 test-plan gap (verify item 9). The plan will gate Phase 2 on these cells.

---

## Residual Minor (not fold-introduced, not a blocker)

**`InspectReport`/`InspectReportJson`/`InspectJson` struct field additions not enumerated.** R0 I1 fix note (point c) called out that `InspectReport` and `InspectReportJson`/`InspectJson` need `kind` (+ decoded `language`) fields. §5 says "shows `kind: mnem` + the decoded language for `0x02`" — the *behavior* is specified, but the SPEC does not name the three structs that gain fields. Source confirmed: `format.rs` `InspectReportJson` (lines 73-81) and `InspectJson` (lines 85-90) currently have no `kind` or `language` fields; `InspectReport` in ms-codec `inspect.rs` also lacks them. A plan author reading §5 will know the output must show kind+language, and will need to add these fields — but the specific struct names are not called out as deliverables in §3 or §5. This is a scoping ambiguity, not a missing fix (the behavior is correct). It was latent in R0 (M3 noted it obliquely); the fold didn't add or remove it. Rating: **Minor — should fold into the plan-doc** as an explicit Phase-2 deliverable (add `kind`/`language` to `InspectReportJson` + `InspectJson` + ms-codec's `InspectReport` struct). Does not block GREEN at SPEC level.

---

## Summary

| Finding | R0 status | R1 status |
|---------|-----------|-----------|
| I1 — three decode-path gates under-scoped | OPEN | RESOLVED ✅ |
| M1 — `--language` vs wire precedence unspecified | OPEN | RESOLVED ✅ |
| M2 — stale `0x01`/`RESERVED_TAG_TABLE` doc-comments | OPEN | RESOLVED ✅ |
| M3 — `validate()`/`kind()` Mnem arms | OPEN | RESOLVED ✅ (folded into I1) |
| M4 — pre-existing enum order decision | OPEN | RESOLVED ✅ |
| InspectReport struct field additions (residual M) | latent in R0 | Minor — carry to plan-doc |

**0 Critical / 0 Important.** GREEN. Plan-doc may proceed. The residual Minor on `InspectReport*` struct field naming should be captured in the plan-doc as an explicit Phase-2 deliverable.
