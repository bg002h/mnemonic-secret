# Implementation review — ms-codec 0.4.2 uppercase acceptance (2026-06-10)

Reviewer: Fable 5 implementation-review agent (post-impl, pre-commit). Plan @ design/PLAN_ms1_envelope_uppercase.md (R0 GREEN r3). Verdict: GREEN (0 Critical / 0 Important / 3 Minor — all folded: dates corrected to the tag day, toolkit companion updated at ship time, this review persisted). Review verbatim below.

---

## Critical

None.

## Important

None.

## Minor

1. **Forward-dated release entries (2026-06-11; today is 2026-06-10)** across CHANGELOG.md:7, crates/ms-codec/CHANGELOG.md:7, design/FOLLOWUPS.md:25. Correct to the tag day.
2. **Toolkit-side companion update is the one ritual leg not yet done** (other repo, correctly absent from this diff): fix the misleading "flips the uppercase-ms1 leg green" phrasing at toolkit FOLLOWUPS:81 (staged cells pin ERRORS — must be INVERTED) + consider annotating `toolkit-ms-codec-pin-bump-0-4-1-combine-fix` to bundle 0.4.2.
3. **CHANGELOG cites the not-yet-persisted impl review** — persist before commit.

## Verdict

**GREEN — 0 Critical / 0 Important / 3 Minor (non-gating).** Evidence:

- **wire_string helper** (envelope.rs:85-103): doc carries soundness + Zeroizing-parity; used at exactly the 3 production sites (discriminate :116, inspect.rs:69, shares.rs:227 defense-in-depth annotated). No remaining raw to_string→extract path.
- **C1(a) ordering verified** (shares.rs:186-263): first parse :188-191 preserves InvalidCase; canonical re-parse :203-209 with `?` routing; canonical vector feeds extraction :224 AND interpolate_at :263. decode.rs + mlock.rs untouched (g6 safe).
- **Tests verbatim**: 9 ms-codec + 1 ms-cli cells; U3-guard fixture exactly the R0-r2 spec.
- **TDD integrity proven live**: scratch-revert → u3_mixed RED (MismatchedHrp) and u3_guard RED showing the LEAK verbatim — `Ok((Tag(entr), Entr([171×16])))` = the exact 0xAB secret returned. Restored sha256-identical, suite re-green.
- **Ritual**: 0.4.2 + =0.4.2 pin + Cargo.lock; ms-cli 0.7.0 unchanged; both CHANGELOGs accurate (incl. the security record + WrongHrp canonicalization + CI caveat); FOLLOWUPS resolution carries the security record + INVERT note + publish blocker.
- **Gate**: workspace zero failures; clippy clean.
