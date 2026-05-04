# Migration guide

This file documents wire-format and API migrations across `ms-codec` minor versions. SemVer is followed with the pre-1.0 convention that the second component (`0.X`) is the breaking-change axis.

## v0.1 scope (after r6 amendment, 2026-05-03)

v0.1 ships **`entr` only** (BIP-39 entropy, 16/20/24/28/32 B). The pre-SPEC spike against `rust-codex32 = "=0.1.0"` found that `seed` (64 B) + the locked `0x00` reserved-prefix byte overflows BIP-93 codex32's long-code length bracket (128-char string, one past the bracket max of 127), and `xprv` (78 B) is outside both BIP-93 brackets at any length. See `design/BRAINSTORM_ms_v0_1.md` §"Wire-format spike findings (2026-05-03, r6 amendment)" for the empirical evidence and the FOLLOWUPS handle `ms1-v01-payload-bracket-overflow-prefix-byte-incompatibility` for the cross-repo record.

The B-use-case from the brainstorm Q2 (BIP-32 master seed backup) is preserved via the routing `BIP-39 seed phrase → entropy bytes → ms1 entr → engrave → recover entropy → BIP-39 mnemonic → (with passphrase) PBKDF2 → 64-B BIP-32 master seed`. Direct `seed` and `xprv` payloads are reserved for v0.2+, which will require BCH framing outside BIP-93's existing length brackets (separate sub-format or new HRP — design TBD).

## v0.1 → v0.2 (planned)

v0.2 will add K-of-N share encoding for `entr`. The migration is designed to be **non-breaking for v0.1 entr strings**: a v0.2 decoder reads v0.1 entr strings correctly. Four invariants lock the contract:

1. **Reserved-prefix byte.** v0.1 emits a `0x00` byte at the start of the BIP-93 codex32 payload, before the entropy bytes. v0.2 promotes this byte to a type discriminator (`0x01 = entr`, future `0x02 / 0x03 / …` for any new payload kinds added in v0.2+ that fit BIP-93's brackets). v0.1 strings (prefix `0x00`) remain forward-readable: a v0.2 decoder seeing prefix `0x00` falls back to v0.1's "type tag is in BIP-93 `id` field" interpretation, which in v0.1 always means `id = "entr"`.

2. **v0.2 grouping invariant.** v0.2 readers assembling K-of-N **`entr`** shares MUST gate on the prefix byte *before* treating BIP-93 `id` as a share-set group key. Strings with prefix `0x00` are dispatched to the v0.1 single-string decode path immediately and never participate in share grouping. Strings with prefix `0x01` (the v0.2 `entr` discriminator) are share-group candidates and group by `id`. Strings with prefix ≥ `0x02` (any future v0.2+ payload kind that fits the BIP-93 short bracket) MUST be dispatched to a kind-specific path BEFORE share grouping is attempted; they MUST NOT default into the entr-grouping path. Without these gates, two unrelated v0.1 strings (each with `id = "entr"`) would be misgrouped as shares of one secret, and any non-entr v0.2+ kind would be silently miscategorized as an entr share.

3. **v0.2 encoder anti-collision invariant.** v0.2 encoders MUST refuse to emit any `id` value that is a member of v0.1's `RESERVED_TAG_TABLE` (`entr`, `seed`, `xprv`, `mnem`, `prvk`, plus any tags added in v0.1.x patches). For random `id` generation: re-roll on collision (rate ≈ 5 / 32⁴ ≈ 1 in 209 715, negligible). For deterministic `id` derivation (e.g., hash-of-secret): hard error, caller must change derivation nonce or use the random-generation path.

4. **API back-compat.** v0.1's `pub fn encode(tag: Tag, payload: &Payload) -> Result<String>` signature is preserved unchanged. v0.2 adds a *new* `pub fn encode_shares(tag: Tag, threshold: Threshold, payload_set: &[Payload]) -> Result<Vec<String>>` overload (`Threshold` is a v0.2-introduced type; v0.1 has no public `Threshold` symbol). `encode_shares(tag, Threshold::ZERO, &[p])` MUST produce a wire-bit-identical string to `encode(tag, &p)` for the same inputs (both go down the same envelope path: prefix `0x00`, `id` = tag, threshold = 0, share index = `s`). The v0.1 `encode` becomes a thin wrapper around `encode_shares` in v0.2's implementation. SHA-pinned regressions on v0.1 outputs continue to pass after callers swap to `encode_shares`.

The `seed` / `xprv` payload framing — out of scope for v0.1 because they don't fit BIP-93's brackets — is a *separate* design problem for v0.2+ and not subject to this v0.1 → v0.2 migration contract. The v0.2 plan when written must specify whether `seed`/`xprv` ride a different sub-format (e.g., a different HRP, a forked codex32 with widened brackets, or an entirely new BCH design) and how (or whether) they coexist on the wire with v0.2's entr-K-of-N strings. **If a future v0.2+ payload kind ends up sharing HRP `ms` (e.g., a widened-bracket BIP-93 variant), it MUST claim a distinct prefix-byte value (`0x02`, `0x03`, …) and invariant #2 above MUST be updated to dispatch that prefix to its own kind-specific path.** The grouping-by-id semantics in #2 are scoped to `entr` shares only and are not implicitly portable to other kinds.

These invariants are also captured in `design/SPEC_ms_v0_1.md` and the source comments of `crates/ms-codec/src/envelope.rs`.
