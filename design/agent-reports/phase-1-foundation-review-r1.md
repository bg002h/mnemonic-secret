# Phase 1 (Foundation) — Opus Review r1

**Date:** 2026-05-03
**Reviewer:** feature-dev:code-reviewer (opus, async)
**Commit reviewed:** `834ff78` ("feat(ms-codec): Phase 1 foundation — consts, error, Tag, Payload")
**Files:** `crates/ms-codec/src/{lib,consts,error,tag,payload}.rs` + `Cargo.toml`
**Tests:** 11 unit tests pass; clippy clean.

## Verdict

**Ship Phase 1 and proceed to Phase 2.** Zero critical, zero important findings. Type surface, error taxonomy, `#[non_exhaustive]` discipline, doc-comments, and the bijection lock all match SPEC §3, §4, §10.3 exactly. The CSPRNG note on `Payload::Entr` is verbatim what SPEC §3.6 mandates. `Tag.0` is correctly private with the `from_raw_bytes` escape hatch reserved for `inspect()`. `Tag::ENTR = Tag(TAG_ENTR)` works because `Tag::ENTR` is defined in the same module as the private field — confirmed.

## Critical findings

None.

## Important findings

None.

## Low / Nit (informational; do not block)

1. `tag.rs:33-38` — `try_new` wrong-length branch slices `bytes.first().copied().unwrap_or(0)` etc. but the second branch (alphabet check, line 44) indexes `bytes[0..3]` directly. Both work, but the asymmetry is mildly noisy; could just `return Err(... { got: [0;4] })` on length mismatch since the bytes carry no useful diagnostic when len != 4.
2. `error.rs:69` — `Error::Codex32(e)` Display uses `{:?}` on the inner. SPEC doesn't mandate a format, but if `codex32::Error` ever gains a Display impl, switching to `{}` would improve user-facing messages.
3. `consts.rs:52` — `(data_bits + 4) / 5` is the standard ceil-div idiom; idiomatic Rust 1.85 has `usize::div_ceil` (stable since 1.73). Pure cosmetic.
4. `payload.rs:73` — test uses `vec![0u8; len]`; a property test with non-zero bytes would catch nothing additional in Phase 1 (no encoder yet) but Phase 6 should sweep this (proptest tasks already planned).
5. `error.rs:108-111` — `source()` returning `None` is correct given `codex32::Error` lacks `std::error::Error`. If a future `codex32 = "0.1.x"` patch adds the impl, revisit. SPEC §10.1 already flags the upstream surface as in flux.
6. `Cargo.toml` — no `bip39` dev-dep yet; SPEC §10.2 calls for a BIP-39 round-trip integration test. Phase 6 territory; plan adds it then.

## Affirmations

- `#[non_exhaustive]` is on `Payload`, `PayloadKind`, `Error` per SPEC §10.3.
- `Payload::Entr` doc-comment names CSPRNG responsibility verbatim per SPEC §3.6.
- `Tag.0` is private; `Tag::ENTR = Tag(TAG_ENTR)` const init works (same-module access). The r2 plan-fix has been applied correctly.
- `from_raw_bytes` is correctly named, doc-commented as tooling-only, and bypasses alphabet validation as intended.
- `Payload::validate()` length set `{16, 20, 24, 28, 32}` matches SPEC §3.5 exactly.
- Bijection test in `consts.rs::tests` matches SPEC §2.4 formula. Spot-checked: 16 B → 9+28+13=50 ✓; 32 B → 9+53+13=75 ✓; 28 B → 9+47+13=69 ✓.
- Error enum covers SPEC §4 rules 1-10 + §3.5/§3.5.1 encoder-side: 10-for-10 with the encoder-symmetry §3.5.1 reusing `ReservedTagNotEmittedInV01`.
- `From<codex32::Error>` defined; canonical `?`-based propagation path for Phase 2.
- `deny(missing_docs)` outside tests is set; every `pub` item has a doc-comment.
- `&[usize]` used for `VALID_*_LENGTHS` not `&Vec<usize>`; `&str` used for `HRP` not `String`; idiomatic.

## Nit handling

Nits 1, 2, 3, 5 deferred to FOLLOWUPS at tier `v0.1-nice-to-have`. Nits 4, 6 already planned (Phase 6 proptest sweep + bip39 dev-dep).
