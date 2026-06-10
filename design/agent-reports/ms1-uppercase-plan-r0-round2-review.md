# R0 round-2 architect review — PLAN_ms1_envelope_uppercase (2026-06-10)

Reviewer: Fable 5 architect agent (R0 round 2, post-fold verification). master @ 952bebd. Verdict: RED (0 Critical / 1 Important I1-r2 / 4 Minor; all round-1 folds verified, C1(a) mechanics re-proven empirically). Review verbatim below.

---

## Critical

None.

## Important

**I1-r2 — U3-guard's "RED TODAY AS Ok(secret)" is only true for a UNIFORM-UPPERCASE, SAME-ID fixture; the plan's cell as written does not reproduce the leak, and no existing fixture does.** `interpolate_at`'s validation loop (codex32 lib.rs:236-256 — length/hrp/threshold/id compares over ALL shares) runs BEFORE the index-match short-circuit (lib.rs:258-262) — an uppercase secret-at-S paired with a LOWERCASE companion dies in `MismatchedHrp("MS","ms")` today (an Err, not the leak). **Empirically verified both ways at 952bebd:** `combine_shares([upper secret-at-S (id "tst7"), upper share (id "tst7", Fe::A, same payload length)])` → **`Ok((Tag::ENTR, Entr(secret)))` today — the leak is real**; with a lowercase companion → Err. Gates because (1) the SECURITY cell carries a false factual claim an implementer red-verifying would "disprove" and weaken; (2) the leak configuration is the realistic one (user uppercases the whole card set for QR incl. the secret-at-S card). Fix: hand-build the pair — `from_seed(HRP, 2, "tst7", Fe::S, wire_bytes)` + `from_seed(HRP, 2, "tst7", Fe::A, filler_same_len)`, `.to_uppercase()` BOTH (same id/threshold/length so all four pre-checks pass and the short-circuit is reached); RED = `Ok` carrying the exact secret payload; post-fix = `Err(SecretShareSuppliedToCombine)`.

## Minor

**M1-r2 — C1(a)'s second `from_string` returns a Result the plan doesn't route** (infallible in practice — probe-proven byte-identical re-parse — but `?` through `Error::Codex32` or `expect` w/ soundness comment; one clause).
**M2-r2 — decode.rs cite:** the residue==0 pass-through is :232-237 (plan said :233-235). Cosmetic.
**M3-r2 — (positive) canonicalization upgrades cross-case duplicate detection** ('A' vs 'a' same index now hits the eager RepeatedIndex pre-check). Optional cell, not required.
**M4-r2 — U5 fixture note:** `from_seed("XS",…)` cannot mint the string (internal set_check_case rejects the mixed result) — build `from_seed("xs",…).to_string().to_uppercase()`.

## Fold-verification

- **C1(a) — folded correctly, mechanics RE-VERIFIED against codex32 0.1.0 + empirically:** checksum case-folds (checksum.rs:113/117; CHARS_INV field.rs:58-66; set_check_case :147-165 rejects only mixed → lowered valid-uppercase re-parses byte-identical, probe-proven); never-lowercase-before-first-parse preserves U4; raw cross-share compares confirmed (MismatchedHrp lib.rs:242, MismatchedId :251); canonical-vector ordering preserves shares.rs downstream (guard :207 sees b's'; k from fields[0] :215; distinct-index :225-232; output lowercase).
- **I1 — folded but inherited round-1's incomplete mechanics → I1-r2.**
- **I2 — folded correctly** (parse_ms1_symbols lowercases :146; residue==0 hands ORIGINAL to decode :232-237; U6 red/green annotations correct).
- **I3 — folded correctly** (rust.yml matrix ubuntu+macos; every test job -p ms-cli; trigger paths include Cargo.toml/Cargo.lock → pin bump + new cell fire CI). **Load-bearing affirmation: ms-codec is a PATH dep (crates/ms-cli/Cargo.toml:20) — the ms-cli cell exercises the LOCAL 0.4.2 code, not the registry.**
- **M1-M8 — all folded and re-verified** (3-site census exact; Parts::data() folds; uniform-uppercase combine ALREADY GREEN today confirmed; two CHANGELOGs; bare-String parity note; pub(crate) helper; no raw-case got pin (error.rs:285 pins lowercase "mq"); exact-pin site + FOLLOWUPS cites live).

## Verdict

**NOT GREEN — 0 Critical / 1 Important / 4 Minor.** Fold I1-r2's fixture spec (+ the minor one-liners), round 3.
