# v0.8.0 BIP test vector audit matrix — mnemonic-secret (ms-codec)

Built 2026-05-13 per the v0.8.0 cross-repo audit cycle.
**Predecessor (still authoritative for everything v0.8.0 did not
change):**
[`v0_7_1-bip-test-vector-audit-matrix.md`](v0_7_1-bip-test-vector-audit-matrix.md)
(marked SUPERSEDED at v0.8.0 in lockstep with this file).

**Cycle SPEC:** `mnemonic-toolkit/design/SPEC_test_vector_audit_v0_8_0.md`.
**Cycle plan:** `/home/bcg/.claude/plans/v0_8_0-bip-vector-adoption.md`.
**Phase 2 R1:** [`v0_8_0-phase-2-bip93-corpus-r1.md`](v0_8_0-phase-2-bip93-corpus-r1.md).

## §0 Cycle disposition

**ms-codec Phase 2 of the v0.8.0 cycle: BIP-93 full inline corpus
pin.** Adds upstream BIP-93 §Test Vectors coverage that v0.7.1
transitively delegated to `rust-codex32 =0.1.0` (except §93.4
which was already byte-pinned at the cross-format level).

The v0.7.1 ms-codec matrix's footnote "§Invalid: 42 strings" was
an earlier-snapshot artifact; live count via
`gh api repos/bitcoin/bips/contents/bip-0093.mediawiki` returns 64
`<code>`-tagged bullets in the §Invalid section. v0.8.0 cell count
reflects the verified count.

## §1 BIP-93 — codex32 full inline corpus

Source: <https://github.com/bitcoin/bips/blob/master/bip-0093.mediawiki>
§Test Vectors.

### Valid vectors (§93.1–.5)

| # | Description | Status | Test fn |
|---|---|---|---|
| BIP-93.1 | k=0, identifier `test`, 16-byte master seed | COVERED | `vector_1_no_split_16_byte_secret` |
| BIP-93.2 | k=2, identifier `NAME`, share `S` recovers 16-byte master seed | COVERED | `vector_2_k_of_2_share_s_recovers_secret` |
| BIP-93.3 | k=3, identifier `cash`, share `s` canonical (first of 4 alternates) | COVERED | `vector_3_k_of_3_share_s_canonical` |
| BIP-93.4 | k=0, identifier `leet`, 32-byte master seed | COVERED | `vector_4_no_split_32_byte_secret` (plus pre-existing cross-format byte-pin in `bip93_cross_format.rs`) |
| BIP-93.5 | long codex32, 64-byte master seed, all-uppercase | COVERED | `vector_5_long_codex32_512_bit_secret` |

### Invalid vectors (§Invalid)

| Bucket | Count | Status | Test fn |
|---|---|---|---|
| All 64 §Invalid `<code>`-bullet entries (truncated/mixed-case HRP, bad-checksum, length-violation) | 64 | COVERED (coarse `is_err()`) | `all_invalid_vectors_rejected_by_codex32` |

Per-entry granular `codex32::Error` variant classification is
deferred — see FOLLOWUP `bip93-invalid-corpus-granular-error-pin`.
The BIP-93 §Invalid prose only says "incorrect checksums" without
per-entry categorization, so pinning the upstream variant would
amount to pinning `rust-codex32`'s internal classification rather
than a BIP-published claim.

### Invariants

- `invalid_corpus_length_is_64` — guards against silent upstream BIP
  drift adding/removing invalid entries.

**Test file:** `crates/ms-codec/tests/bip93_inline_vectors.rs`.
**Phase 2 commit:** `7101c16` (impl), `d0a76b2` (close fold).

## §2 BIP coverage unchanged from v0.7.1

- BIP-39 integration tests (English wordlist): COVERED via
  `bip39_integration.rs` (unchanged from v0.1.1).
- BIP-93 §93.4 cross-format byte-pin: COVERED via
  `bip93_cross_format.rs` (unchanged from v0.1.1). v0.8.0 adds a
  second pin path through `bip93_inline_vectors.rs` for the same
  vector; intentional overlap.

## §3 Sibling-repo cross-coverage (cycle context)

Cross-cite per-repo v0.8.0 matrices:

- `descriptor-mnemonic/design/agent-reports/v0_8_0-bip-test-vector-audit-matrix.md` —
  Phase 1: BIP-341 wallet-test-vectors (+7 cells + 2 invariants).
- `mnemonic-toolkit/design/agent-reports/v0_8_0-bip-test-vector-audit-matrix.md` —
  Phase 3: BIP-85 v85.3 (+1 cell); §0 cross-repo coverage table.
- `mnemonic-key/design/agent-reports/v0_8_0-bip-test-vector-audit-matrix.md` —
  no scope this cycle; cross-repo audit symmetry only.
