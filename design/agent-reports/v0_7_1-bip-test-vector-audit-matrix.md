# v0.1.1 BIP test vector audit matrix — mnemonic-secret (ms-codec)

**SUPERSEDED at v0.8.0** — see
[`v0_8_0-bip-test-vector-audit-matrix.md`](v0_8_0-bip-test-vector-audit-matrix.md)
for the BIP-93 full inline corpus added at v0.8.0 plus the
"42 invalid → 64 invalid" count correction. Coverage in this file
is still authoritative for everything v0.8.0 did not change.

Built 2026-05-07 per the v0.7.1 audit cycle plan
(`/home/bcg/.claude/plans/let-s-work-on-the-soft-waterfall.md`).

Scope: ms-codec is BIP-93 codex32 used directly via Andrew Poelstra's
`rust-codex32 = "=0.1.0"` crate (CC0). ms-codec adds payload semantics
(reserved-prefix byte + tag-as-discriminator) and a v0.1 → v0.2 migration
contract; the BIP-93 wire format itself is delegated upstream.

Status legend: same as toolkit matrix — COVERED / MISSING / OUT-OF-SCOPE-PER-USER /
OUT-OF-SCOPE-PER-SPEC.

---

## BIP-93 — codex32

Source: <https://github.com/bitcoin/bips/blob/master/bip-0093.mediawiki> §Test Vectors.

The BIP-93 §Test Vectors section publishes 5 valid vectors plus 42 invalid
strings for the upstream codex32 wire format. ms-codec consumes them via
`rust-codex32`; the upstream crate's `Codex32String` test corpus is the
source of truth for wire-format conformance. ms-codec's audit-of-interest
is "do BIP-93 vectors that fall inside the **`entr` payload** envelope
contract still round-trip through ms-codec's higher-level decode/encode?"

### Valid vectors

| # | String head | Threshold | Tag | Length | Status | Notes |
|---|---|---|---|---|---|---|
| 93.1 | `ms10testsxxxxxxxxxxxxxxxxxxxxxxxxxx4nzvca9cmczlw` | 0 | `test` | 128-bit (short) | OUT-OF-SCOPE-PER-SPEC | id `test` is not in ms-codec's `RESERVED_TAG_TABLE` (`entr` only emit; `seed`/`xprv`/`mnem`/`prvk` reserved-not-emitted). BIP-93 valid → ms-codec rejects with reserved-tag error. Negative test in `tests/negative.rs::rule_6_unknown_tag_rejected` covers a structurally-similar case. |
| 93.2 | `MS12NAMEA320...870HKKQRM` (and 4 sibling shares) | 2 | `name` | 128-bit shares | OUT-OF-SCOPE-PER-USER | v0.1 emits threshold = 0 only; share-decoding is v0.2+ scope. Negative test `rule_3_threshold_not_zero_rejected` already covers the rejection. |
| 93.3 | `ms13cashs...d6nln` (5 shares) | 3 | `cash` | 128-bit shares | OUT-OF-SCOPE-PER-USER | same as 93.2 |
| 93.4 | `ms10leetsllhdmn9m42vcsamx24zrxgs3qrl7ahwvhw4fnzrhve25gvezzyqqtum9pgv99ycma` | 0 | `leet` | 256-bit (short) | COVERED | id `leet` not in `RESERVED_TAG_TABLE`; 32-byte payload IS within ms-codec's `entr` byte-length set. `tests/bip93_cross_format.rs::bip93_vector_4_payload_extracts_via_upstream` + `bip93_vector_4_payload_round_trips_as_ms_codec_entr` pin the 32-byte payload extraction + entr re-encoding round-trip + upstream-parser conformance of the ms-codec output. |
| 93.5 | `MS100C8VSM32ZX...` 64-byte master seed (long) | 0 | `0C8V` (random) | 512-bit (long) | OUT-OF-SCOPE-PER-USER | 64-byte payload + ms-codec's 0x00 prefix overflows BIP-93 long bracket (proven in `BRAINSTORM_ms_v0_1.md` r6 spike); v0.1 reserves `seed` but rejects emit/decode. |

### Invalid vectors (42 strings)

OUT-OF-SCOPE-PER-SPEC at the ms-codec level: every invalid case is rejected
inside `rust-codex32::Codex32String::from_string`, which is exact-pinned at
`=0.1.0`. ms-codec's `tests/negative.rs` exercises 14 of its own rejection
rules (rules 1–14) covering every error variant of `ms_codec::Error`.
`rule_1_invalid_checksum_rejected`, `rule_2_wrong_hrp_rejected`,
`rule_3_threshold_not_zero_rejected`, `rule_4_share_index_not_secret_rejected`
are direct delegates to BIP-93 invariants.

### ms-codec custom corpus

`tests/vectors/v0.1.json` — 6 vectors (was 2 in v0.1.0; +4 in v0.1.1).
Source: hand-computed against BIP-39 abandon canonical entropies (and one
non-zero vector); `tests/vectors.rs::v01_corpus_round_trips` asserts
decode + re-encode bit-identity.

| # | Description | Entropy | ms1 | Status | Notes |
|---|---|---|---|---|---|
| MSV1 | 12-word abandon | `00000000000000000000000000000000` | `ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f` | COVERED | `tests/vectors.rs::v01_corpus_round_trips` |
| MSV2 | 15-word all-zero | `0000...` (20 B) | `ms10entrsqqq...qqqqke34pjrgn5p6k` | COVERED | same (added v0.1.1) |
| MSV3 | 18-word all-zero | `0000...` (24 B) | `ms10entrsqqq...qqqqsqgw5k4rjyhy0` | COVERED | same (added v0.1.1) |
| MSV4 | 21-word all-zero | `0000...` (28 B) | `ms10entrsqqq...qqqqkt649dq594pyz` | COVERED | same (added v0.1.1) |
| MSV5 | 24-word abandon | `0000...` (32 B) | `ms10entrsqqq...qqqqcwugpdxtfme2w` | COVERED | same |
| MSV6 | 15-word non-zero | `0123...4567` (20 B) | `ms10entrsqqqjx3t83x4ummcpydzk0zdtehhszg69vucrgd4pcjx3kkj` | COVERED | same (added v0.1.1; catches entropy-bit-ordering bugs zero-entropy vectors miss) |

### BIP-93 valid-string conversions to ms-codec equivalents

**RESOLVED v0.1.1.** `tests/bip93_cross_format.rs` pins BIP-93 §93.4's 32-byte
payload extraction via upstream `rust-codex32` and re-encoding as ms-codec
entr; the ms-codec output is then re-parsed by upstream codex32 to confirm
ms-codec is a proper sub-format. This catches any drift in upstream bit-packing
across `rust-codex32` patch versions (we exact-pin at `=0.1.0`, so drift is
gated to manual bumps).

---

## BIP-39 — entropy ↔ mnemonic

Source: <https://raw.githubusercontent.com/trezor/python-mnemonic/master/vectors.json>.

ms-codec's value proposition is "ms1 entr is a stronger backup of BIP-39
entropy than the mnemonic itself" (SPEC §1.1). The `bip39_integration.rs`
test file already verifies entropy↔mnemonic via the upstream `bip39 = 2`
crate at all 5 BIP-39 lengths.

| # | Length | Mnemonic head | Status | Notes |
|---|---|---|---|---|
| 1 | 16 B / 12 words | "abandon...about" | COVERED | `bip39_integration.rs::bip39_12_word_round_trip_english` |
| 9 | 32 B / 24 words | "abandon...art" | COVERED | `bip39_integration.rs::bip39_24_word_round_trip_english` |
| (random) 16/20/24/28/32 B | det-seeded | n/a | COVERED | `bip39_integration.rs::bip39_random_entropy_round_trips_at_all_word_counts` |

Trezor corpus byte-pinning at the entropy-only layer is OUT-OF-SCOPE-PER-SPEC
for ms-codec — the toolkit owns the BIP-39 derivation surface (Phase 1
of the toolkit-side audit pins the Trezor quad). ms-codec only verifies
"`bip39 = 2` faithfully round-trips entropy↔mnemonic" via `bip39_integration.rs`
without claiming Trezor-byte conformance (delegated upstream).

---

## Cross-repo invariants

Phase 10 also adds (or audits the absence of):

1. **Reserved-prefix-byte round-trip pin** — `tests/forward_compat.rs::flipping_prefix_byte_to_v02_value_rejects_at_v01_decoder` already COVERS the v0.1→v0.2 migration contract. No new test needed unless v0.2 ships in this cycle (it doesn't).
2. **HRP `ms` collision audit** — already covered cross-repo in mk1's `design/AUDIT_hrp_mk_collision.md`. No ms-codec-side artifact.
3. **`bip39 = 2` exact-pin drift detection** — `Cargo.lock` is checked in; CI catches version drift.

---

## Summary

| Category | Total vectors | Covered | Missing (in-scope) | Out-of-scope-per-user | Out-of-scope-per-spec |
|---|---|---|---|---|---|
| BIP-93 valid (5) | 5 | 1 (93.4 via cross-format pin, v0.1.1) | 0 | 3 (shares + 64-B seed) | 1 (id `test`) |
| BIP-93 invalid (42) | 42 | rules 1–14 in `negative.rs` | 0 | 0 | 42 (delegated upstream) |
| ms-codec custom corpus | 6 | 6 | 0 | 0 | 0 |
| BIP-39 entropy↔mnemonic | n/a | 3 (12/24 + property) | 0 | 0 | Trezor byte pin → toolkit |
| **TOTAL net-new pins (Phase 10, shipped v0.1.1)** | — | **6** (4 corpus entries + 2 cross-format tests) | — | — | — |

Phase 10 shipped: 4 corpus entries (15/18/21-word all-zero + 1 non-zero
20-B) extending `tests/vectors/v0.1.json` from 2 → 6 vectors, plus
`tests/bip93_cross_format.rs` with 2 cross-format conformance tests.
Test count delta: 127 → 129 (+2 distinct test functions; the corpus loop
test absorbs the 4 new corpus entries without adding test-function
counts).

---

## Discoveries (require architect review before pinning)

1. **No bug-shaped findings.** ms-codec delegates the wire format upstream
   to `rust-codex32 = "=0.1.0"`; vector-level audits are upstream's
   responsibility. ms-codec's audit surface is the value-add layer
   (reserved-prefix byte, tag-as-discriminator, v0.1→v0.2 migration
   contract), all of which are already exercised in `tests/forward_compat.rs`,
   `tests/negative.rs`, `tests/round_trip.rs`.

2. **RESOLVED v0.1.1 — non-zero entropy added to custom corpus.** MSV6
   in `tests/vectors/v0.1.json` is a 20-byte non-zero vector
   (`0123456789abcdef0123456789abcdef01234567`). A bug that flipped
   entropy-bit ordering would now surface as an MSV6 mismatch.

3. **No cross-impl conformance corpus exists.** ms1 has no second
   independent encoder to cross-validate against (rust-codex32 is the
   only one). Any ms1 string that round-trips through ms-codec is
   trivially conformant to itself; "conformance" is `rust-codex32`-equivalence.
   This is the same posture as md1 v0.1 / mk1 v0.1.
