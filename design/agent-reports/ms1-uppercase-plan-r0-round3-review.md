# R0 round-3 architect review — PLAN_ms1_envelope_uppercase (2026-06-10)

Reviewer: Fable 5 architect agent (R0 round 3, post-fold verification). master @ 952bebd. Verdict: GREEN (0 Critical / 0 Important / 2 non-gating Minors). Review verbatim below.

---

## Critical

None.

## Important

None.

## Minor

**M1-r3 — with C1(a)'s canonical vector, the `wire_string` call at the shares.rs:200 extraction site becomes a no-op.** Harmless defense-in-depth, but clarify which mechanism is load-bearing for combine (the canonical vector: extraction alone cannot fix interpolate_at's raw compares). One clarifying clause; does not gate.

**M2-r3 — cosmetic, fold-trail only:** round 2's M7 cite "error.rs:285" is crates/ms-cli's error.rs. The sweep conclusion holds (re-swept: no test in either crate pins an uppercase WrongHrp.got). No plan edit required.

## Fold-verification

- **I1-r2 — folded correctly and EMPIRICALLY RE-PROVEN (independent probe this round):** the uniform-uppercase same-id pair returns **`Ok((ENTR, Entr([0xAB;16])))` today — the exact secret payload, leak confirmed**; the lowercase-companion variant errs in MismatchedHrp (does not reproduce) — exactly as folded. Mechanics re-confirmed in codex32 0.1.0 (validation loop :236-256 precedes the short-circuit :258-262; Fe::from_char folds 'S'). from_seed signature matches the call shape. Probe deleted; tree clean.
- **M1-r2 / M2-r2 / M4-r2 — all folded correctly** (Result routed via `?`; decode.rs:232-237; U5 from_seed("xs")-then-uppercase).

Whole-plan re-scan: 3-site census exact (all other extract_wire_fields callers are tests); U1/U6 red/green empirically re-proven (pristine-uppercase fails, 1-error uppercase repairs today); CI/ritual claims hold (matrix ubuntu+macos, -p ms-cli only, path dep at crates/ms-cli/Cargo.toml:20 =0.4.1); two-CHANGELOG shapes match; FOLLOWUPS cites live both repos; ms-cli analyze probes at cmd/inspect.rs:114-124; no raw-case got pin.

## Verdict

**GREEN — 0 Critical / 0 Important / 2 Minor (non-gating).** The plan is implementation-ready; the U3-guard leak claim independently re-proven. No round 4 required.
