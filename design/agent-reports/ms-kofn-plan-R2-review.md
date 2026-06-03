# Plan R0 — ms K-of-N — round 2

**Verdict:** GREEN (0C/0I)

Round 1 was RED (0C/1I). This round verifies the two folds @ `eadee0f` against source (`cmd/encode.rs:51-111` language_for_card :65/73/77/105/107/166; `language.rs` as_str :51 / code :28; `envelope.rs:131` parts().data(); `:142/178-179` mnem wire byte; `payload.rs:104-105` entropy accessor).

## Critical / Important — (none)
## Minor — (none new; m-r2 optional mnem-branch guard intentionally unfolded, not a regression)

## Confirmations
- **I-r1 FOLDED CORRECTLY + COMPLETE.** Task 2.1 `resolve_secret_payload(...) -> Result<(Payload, Option<&'static str>)>`; 2nd element = encode's `language_for_card` (`Some(language.as_str())` phrase / `None` hex — matches encode.rs:73/77). encode::run reconstructs both consumers (stderr card :166-167, --json language :143). The encode-output-unchanged assertion {english phrase, non-english phrase, hex} is present + correct. split ignores the 2nd element — sound: mnem language survives via the WIRE ([0x02][language][entropy] → dispatch_payload → Payload::Mnem), independent of language_for_card. word_count (:102) + entropy_hex (:144) recoverable from the returned Payload. encode byte-/output-identity fully restored; `&'static` lifetime clean.
- **m-r1 FOLDED CORRECTLY.** Plan:126-127 use `c.parts().data()` / `secret.parts().data()` (matches envelope.rs:131). No stale `.data()`.
- **No new drift.** Symbol-creation closure clean; no task references an uncreated symbol; no ordering/contradiction. All round-0 folds (C1/C2/I1/I2/I3/m1-m3/det-index/getrandom-0.3) carried + undisturbed.
- **Completeness.** Self-review block maps every SPEC §1-§9 + every design-review C/I/M to a task. Implementable end-to-end.

**Disposition:** GREEN — 0C/0I. Implementation CLEARED — Phase 0 (spike hard-gate) first; the 3 claims (ZERO byte-identity, entr+mnem k∈2..9 all-5-lengths round-trip, C1 index-s short-circuit) MUST pass before real code. Per-phase TDD + per-phase opus R0 to 0C/0I remain mandatory.
