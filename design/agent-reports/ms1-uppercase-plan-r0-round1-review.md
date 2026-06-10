# R0 round-1 architect review — PLAN_ms1_envelope_uppercase (2026-06-10)

Reviewer: Fable 5 architect agent (R0, pre-implementation). Plan @ design/PLAN_ms1_envelope_uppercase.md, master @ 952bebd. Verdict: RED (1 Critical / 3 Important / 8 Minor). Review verbatim below.

---

## Critical

**C1 — U3's mixed-case-SET claim is wrong as designed; the planned fix cannot make it pass.** The plan asserts that a mixed set (one consistently-uppercase share among lowercase ones) "must WORK," but the `wire_string` design only canonicalizes the *extracted-field* copies — the parsed `Codex32String`s handed to interpolation at `shares.rs:236` keep their original case, and pinned codex32 0.1.0 `interpolate_at` does **raw case-sensitive cross-share compares**: `lib.rs:241-243` (`MismatchedHrp("MS","ms")`) and `lib.rs:250-252` (raw-case `id`). A mixed-case set fails inside codex32 no matter what ms-codec's field extraction does. Fix, pick one: **(a) — recommended:** in `combine_shares`, after each share passes `Codex32String::from_string(s.clone())` (preserving the mixed-case-within-one-string `InvalidCase` rejection — do NOT lowercase before the first parse, that would launder mixed case and break U4), re-canonicalize via `Codex32String::from_string(c.to_string().to_ascii_lowercase())` (the checksum engine case-folds, so a lowercased valid string re-parses); pass the canonical vector to both field extraction and `interpolate_at` (this also makes the interpolated output lowercase — codex32 re-emits uppercase when hrp is uppercase, lib.rs:298-305). **(b)** re-scope U3 to uniform-case sets + pin MismatchedHrp for mixed (weaker, contradicts the plan's motivation).

## Important

**I1 — uppercase secret-at-S bypasses the `SecretShareSuppliedToCombine` guard TODAY; the plan fixes it silently with no test.** `shares.rs:207` compares `idx == b's'` raw — an all-uppercase secret-at-S has `b'S'`, sails past the guard, reaches `interpolate_at`'s index-match short-circuit (lib.rs:259-262), and `combine_shares` RETURNS THE SECRET PAYLOAD, defeating the policy guard (comment shares.rs:205-208). The fix restores the guard — a security-adjacent restoration must be pinned. Add to U3: `combine_shares([uppercase secret-at-S, share])` → `Err(SecretShareSuppliedToCombine)`, RED today (currently `Ok`).

**I2 — census misses `decode_with_correction`'s clean-codeword pass-through.** `decode.rs:145-154` (parse_ms1_symbols) already lowercases, but the residue==0 path at `decode.rs:233-235` calls `decode(s)` with the ORIGINAL string — today a PRISTINE uppercase card fails WrongHrp while a CORRUPTED one repairs fine (corrections re-emit lowercase, :276-277). Covered only transitively; the prescribed sweep (grep to_string-then-slice) would never surface it. Add **U6**: `decode_with_correction(&upper_clean)` == lowercase twin with empty corrections (RED today) + `decode_with_correction(&upper_with_1_error)` repairs (green today — pins the asymmetry's resolution). This is the `ms repair` public surface (ms-cli repair.rs:122-147 already lowercases → CLI fixed transitively once U6 holds).

**I3 — Ritual verification surface mischaracterized: NO Windows lanes, and CI never runs ms-codec's tests.** rust.yml:38 matrix = ubuntu+macos only; every test job runs `cargo test -p ms-cli` (rust.yml:51,58,73,86,101,123,181) — none of U1-U6 (ms-codec) execute in CI (open FOLLOWUP `ms-codec-no-ci-workflow`; rust.yml:6's "ms-codec has its own separate workflow" comment is itself false). Fixes: (1) gate = local full suite + clippy per the I9 precedent — say so; (2) add ONE ms-cli integration cell (`ms decode <UPPERCASE_MS1>` succeeds, crates/ms-cli/tests/) for CI-executed coverage (the pin bump touches Cargo.toml/Cargo.lock, in rust.yml's path filters). g6 mlock: only compares crates/ms-cli/src/mlock.rs — untouched, no trip (confirmed both sides).

## Minor

**M1 — shares.rs:302 is a TEST helper** (mod tests, reads always-lowercase encode output) — not a production site. Complete src census = exactly three: envelope.rs:96, inspect.rs:65, shares.rs:200 (envelope.rs:65's separator-less WrongHrp arm covered once input is the lowered copy).
**M2 — parts().data() question CLOSED with verified fact:** codex32 `Parts::data()` case-folds (lib.rs:399-405 via Fe::from_char; CHARS_INV maps both cases) — inspect.rs:75 + shares.rs:240 need NO rerouting. Inspect adjudication AFFIRMATIVE: the "raw observation" intent is about non-table tag VALUES, not case; lowercasing loses nothing; transitively fixes ms-cli analyze() (cmd/inspect.rs:115 `report.hrp != "ms"`, :123 `!= 's'`) which today emits spurious wrong-hrp/share-index reasons for uppercase cards.
**M3 — per-cell red/green:** uniform-uppercase U3-set likely ALREADY GREEN today (digit threshold; Fe folds; uniform set passes raw compares; data() folds). Genuinely red today: U1, U2, I1's secret-at-S, U3-mixed (under (a)), U6-clean.
**M4 — CHANGELOG convention = TWO files:** root CHANGELOG.md (crate-prefixed entry) + crates/ms-codec/CHANGELOG.md; no ms-cli changelog (unchanged).
**M5 — wire_string returns a bare String carrying the secret:** pre-existing parity with the current c.to_string() copies; either Zeroizing<String> or an explicit parity note.
**M6 — helper is pub(crate)** (used from inspect.rs + shares.rs); envelope.rs placement right.
**M7 — U5 pin-sweep clean:** no test pins raw-case `got`; codex32-direct case tests unaffected.
**M8 — ms-cli confirmed no case-sensitive probes** before ms-codec; repair.rs already lowercases; no lowercase-only docs claims; only the exact-pin bump (crates/ms-cli/Cargo.toml:20). Tag namespace ms-codec-v0.4.2 ✓; SemVer PATCH sound.

## Verdict

**RED — 1 Critical, 3 Important.** Fold C1 option (a), add the I1/I2 cells + I3's ms-cli CI-visible cell, correct the ritual text, fold the minors, re-dispatch.
