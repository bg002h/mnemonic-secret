# Migration guide

This file documents wire-format and API migrations across `ms-codec` minor versions. SemVer is followed with the pre-1.0 convention that the second component (`0.X`) is the breaking-change axis.

## v0.1 → v0.2 (planned)

v0.2 will add K-of-N share encoding. The migration is designed to be **non-breaking for v0.1 strings**: a v0.2 decoder reads v0.1 strings correctly. Two invariants lock the contract:

1. **Reserved-prefix byte.** v0.1 emits a `0x00` byte at the start of the BIP-93 codex32 payload. v0.2 promotes this byte to a type discriminator (`0x01 = seed`, `0x02 = entr`, `0x03 = xprv`, …). v0.1 strings (prefix `0x00`) remain forward-readable: a v0.2 decoder seeing prefix `0x00` falls back to v0.1's "type tag is in BIP-93 `id` field" interpretation.

2. **v0.2 grouping invariant.** v0.2 readers assembling K-of-N shares MUST gate on the prefix byte *before* treating BIP-93 `id` as a share-set group key. Strings with prefix `0x00` are dispatched to the v0.1 single-string decode path immediately and never participate in share grouping. Without this gate, two unrelated v0.1 strings sharing `id = "seed"` (or `entr` / `xprv`) would be misgrouped as shares of one secret.

3. **v0.2 encoder anti-collision invariant.** v0.2 encoders MUST refuse to emit any `id` value that is a member of v0.1's `RESERVED_TAG_TABLE` (`seed`, `entr`, `xprv`, `mnem`, `prvk`, plus any tags added in v0.1.x patches). For random `id` generation: re-roll on collision (rate ≈ 5 / 32⁴ ≈ 1 in 209 715, negligible). For deterministic `id` derivation (e.g., hash-of-secret): hard error, caller must change derivation nonce or use the random-generation path.

4. **API back-compat.** v0.1's `pub fn encode(tag: Tag, payload: &Payload) -> Result<String>` signature is preserved unchanged. v0.2 adds a *new* `pub fn encode_shares(tag: Tag, threshold: Threshold, payload_set: &[Payload]) -> Result<Vec<String>>` overload. `encode_shares(tag, Threshold::ZERO, &[p])` MUST produce a wire-bit-identical string to `encode(tag, &p)` for the same inputs (both go down the same envelope path: prefix `0x00`, `id` = tag, threshold = 0, share index = `s`). The v0.1 `encode` becomes a thin wrapper around `encode_shares` in v0.2's implementation. SHA-pinned regressions on v0.1 outputs continue to pass after callers swap to `encode_shares`.

These invariants are also captured in `design/SPEC_ms_v0_1.md` and the source comments of `crates/ms-codec/src/envelope.rs`.
