# R0 ARCHITECT REVIEW — SPEC_ms_codec_test_hardening.md

Opus code-architect, mandatory pre-impl gate. SPEC verified against live source @ branch `master` (SHA `c919f4b`). Persisted by controller (review agent had no Write tool).

## Headline confirmations (grep-verified)
- `encode(Tag,&Payload)->Result<String>` (encode.rs:16); `decode(&str)->Result<(Tag,Payload)>` (decode.rs:27) with rule-9 length gate FIRST (decode.rs:29-34); `decode_with_correction(&str)->Result<(Tag,Payload,Vec<CorrectionDetail>)>` (decode.rs:188); residue computed BEFORE length gate (parse_ms1_symbols decode.rs:192 has no length check → residue decode.rs:195-197); re-verify guard decode.rs:233-239 → `TooManyErrors{bound:8}`.
- `CorrectionDetail{position,was,now}` (decode.rs:86-95, derives Debug/Clone/PartialEq/Eq); `Payload::Entr(Vec<u8>)` (payload.rs:27 PartialEq/Eq/Debug); `Tag::ENTR` (tag.rs:16, PartialEq/Eq/Debug); `Error::{TooManyErrors{bound:u8} error.rs:76-79, UnexpectedStringLength{got,allowed} error.rs:49-54}`.
- Single-string: `TooManyErrors` has only `bound` (no chunk_index); no chunk/split/reassemble module. BCH(93,80,8) non-perfect, t=4 (`deg>4→None` bch_decode.rs:416).
- `VALID_MS1_12W` (bch_decode.rs:35, 50 chars/data-part 47), `corrupt_at` (bch_decode.rs:40), alphabet+HRP "ms" (decode.rs:100, consts.rs:11).
- `Ms1IndelOracle` delegates to decode_with_correction (repair.rs:892), ⊆-filters on CorrectionDetail.position (repair.rs:894-897); `indel.rs` length-restores all candidates (data_variants indel.rs:213-260, prefix_restorations indel.rs:139-169). proptest dev-dep (Cargo.toml:20). lint gates src/ anchors only (lint_zeroize_discipline.rs:36-57); round_trip.rs holds raw Vec<u8>.

## THE CRUX (T3-ms-1 mechanism) — VERIFIED CORRECT
`decode_with_correction("…51 chars…")`: parse_ms1_symbols (no length check) → residue over wrong-length vector → `decode_regular_errors` overwhelmingly `None` → `Err(TooManyErrors{bound:8})`. The rule-9 `UnexpectedStringLength` gate (decode.rs:29) is only reached on `decode()` re-entry, NOT by a raw wrong-length string. So the SPEC is RIGHT: assert `is_err()`, do NOT pin `UnexpectedStringLength` (a naive variant pin would be FALSE-FAILING). The SPEC explicitly warns against this trap.

## CRITICAL — None.
## IMPORTANT — None.
Theme 1 sound (k distinct positions + nonzero masks ⇒ exactly k≤4 errors ⇒ recovery guaranteed; position-set + len()==k meaningful). T2c `!= Ok(original)` correct (non-perfect code → Ok(different) legitimate; is_err would be flaky; honesty note about guard load-bearing ~2⁻²⁶ accurate). Theme-3 position-accuracy redundancy genuine (bch_decode.rs Cells 2/3/4/6: assert_eq! position at :85/:113/:140/:189); T3-ms-2 distinct from five_error_too_many (bch_decode.rs:151 = 5 substitutions vs T3-ms-2 = net-zero indel). Oracle ⊆-gate + length-restored-candidate claims hold. Zeroize "no new test obligation" correct.

## MINOR
- M1 — `VALID_ENTR_LENGTHS` cite: §1 says consts.rs:28 (doc-comment); `pub const` is consts.rs:29. Content correct.
- M2 — `Ms1IndelOracle` cite repair.rs:885-908; struct decl is repair.rs:884 (impl 885-908). Harmless.
- M3 — Theme-1 strategy must bound positions to `0..dp` (dp=encode().len()-3, incl. checksum tail), NOT `dp-13`. SPEC already says `0..dp`; plan-vigilance note.
- M4 — T2d/T3-ms-2 build-time-verify instruction present + correct (md UNCORRECTABLE convention). No action.

## VERDICT: GREEN (0C / 0I / 4M)
SPEC faithfully transcribes verified source. The load-bearing crux + the `!= Ok(original)` choice both check out. No described test is vacuous/false-passing/false-failing. The 4 MINORs are cosmetic line-cite drift + plan-vigilance. 0C/0I met — SPEC may advance to the plan-doc (own R0). Fold M1/M2 line-cites; carry M3 as a plan strategy-bound note.
