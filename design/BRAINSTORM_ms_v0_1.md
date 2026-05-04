# Brainstorm: ms1 v0.1

**Date:** 2026-05-03
**Status:** Re-opened and amended 2026-05-03 (r6 amendment) following pre-SPEC encode/decode spike against `rust-codex32 = "=0.1.0"`. Outputs feed `SPEC_ms_v0_1.md`.
**Plan-mode meta-plan:** `/home/bcg/.claude/plans/c-ultimately-what-we-quirky-avalanche.md` (out-of-tree; converged at r5 after 5 reviewer rounds; r6 amendment captured below in §"Wire-format spike findings (2026-05-03)")
**FOLLOWUPS handle:** `ms1-v01-payload-bracket-overflow-prefix-byte-incompatibility` (primary entry in `design/FOLLOWUPS.md`; cross-repo mirrors in `mnemonic-key/design/FOLLOWUPS.md` and `descriptor-mnemonic/design/FOLLOWUPS.md`).

This document captures the brainstorm rationale chain that led to ms1's v0.1 design. The SPEC cites this doc rather than re-litigating each decision. The companion long-term roadmap is in the user's auto-memory at `~/.claude/projects/-scratch-code-shibboleth-descriptor-mnemonic/memory/ms1_toolkit_long_term_roadmap.md`.

## Why this format exists (the load-bearing observation)

Most Bitcoin users back up only a BIP-39 seed phrase. That seed alone is **not sufficient for self-recovery** — spending depends on the implicit assumption that a future wallet will guess the same template (BIP-44 / 49 / 84 / 86 / 48-multisig / …) the user is silently relying on. Today the seed is also the worst part of a backup to engrave: BIP-39's checksum is too weak to localize errors on a steel plate.

The m-format triad (md1 + mk1 + ms1) plus a future `mnemonic-toolkit` removes both fragilities by making the **secret + template + key bundle** explicitly engravable as a coherent set, so the user no longer depends on industry conventions outliving them.

## The 5-question rationale chain

Each question was asked once with multiple-choice options. The user's answer, plus the reasoning that anchored it, is recorded below.

### Q1 — what is "ms1"?

**Options:**
- A: New third sibling format with HRP `ms` (parallel to md1/mk1).
- B: Typo for md1 — extend md-codec with private-bearing TLV tags.
- C: Typo for mk1 — extend mk-codec to encode an xpriv.
- D: Something else.

**Answer:** D, refined to "something like A, but instead of K-of-N shares from codex32 immediately, just the encoding/checksum layer. We will eventually deploy K-of-N shares for m{d,k,s}1 strings."

**Implication:** new sibling format, new sibling repo, deferred share-encoding across the *whole family* — not just ms1.

### Q2 — what's the engraving use case?

**Options:**
- A: Multisig cosigner secrets engraved alongside descriptors and xpubs.
- B: Generic xpriv/seed backup, format-unified with md1/mk1.
- C: BIP-39 entropy / mnemonic re-encoding with real BCH error correction.
- D: Future Shamir share carrier, plain-encoded as a stepping stone.
- E: Other / a mix.

**Answer:** B + C. Generic xpriv/seed backup *and* BIP-39 entropy with real BCH error correction.

**Implication (original):** payload set is at minimum {seed, entr, xprv}; multiple payload kinds means a discriminator is required.

**r6 amendment (2026-05-03):** the pre-SPEC spike (see §"Wire-format spike findings" below) discovered that `seed` (64 B) and `xprv` (78 B) cannot ride on BIP-93 codex32 with the locked `0x00` reserved-prefix byte (and `xprv` cannot ride on BIP-93 codex32 at *any* length). v0.1 ships **`entr` only**. The B-use-case (engraving a backup that re-derives a BIP-32 master seed) is **not lost** — it is satisfied by the routing `BIP-39 seed phrase → entropy bytes → ms1 entr → engraved → recover entropy → BIP-39 mnemonic → (with passphrase) PBKDF2 → 64-B BIP-32 master seed`. Direct `seed` and `xprv` payloads are reserved for v0.2+, which will need to design BCH framing outside BIP-93's existing length brackets.

### Q3 — what does codex32 have for share splitting? (knowledge question + critical discovery)

ms1 was anchored on "codex32" the whole time. Q3 surfaced two important facts:

1. **BIP-93 codex32 (Andrew Poelstra) is the share-splitting spec we'd want** — Shamir over GF(32), each share is a complete codex32 string with threshold/id/share-index header fields, K shares with distinct indices reconstruct via Lagrange interpolation. Threshold = 0 = unsplit single-string secret. This is exactly the v0.2 future and the v0.1 present.

2. **HRP collision: `ms` is BIP-93's HRP.** mk-codec already negative-tests `InvalidHrp("ms")` — `ms` was already taken when mk1 picked `mk`. So ms1 is not an unclaimed namespace; it *is* BIP-93 codex32.

**Follow-up choice (X / Y / Z):**
- X: Use BIP-93 codex32 directly via `rust-codex32`. v0.1 = always emit threshold = 0. ms1 inherits the upstream wire format; we add only payload semantics.
- Y: Build ms1 as md/mk-shaped sibling with a different HRP; collide-by-HRP would be a footgun.
- Z: Hybrid — adopt BIP-93 wire envelope but layer our own TLV inside.

**Answer:** X — "I want to reuse Andrew's work, not replace it."

**Implication:** ms1 is not a new wire format we invent; it is BIP-93 codex32 used directly. The "spec" we author is mostly: which crate we depend on, what payload semantics live in our envelope, how an ms1 string co-engraves with md1+mk1.

### Q4 — how does the payload distinguish seed vs entropy vs xpriv?

**Options:**
- A: Length-based, no metadata bytes (16/20/24/28/32 = entropy; 64 = seed; 78 = xpriv; non-overlapping).
- B: Discriminator byte prefixed to the secret.
- C: Repurpose BIP-93's 4-char `id` field as a type tag (e.g., `seed`, `entr`, `xprv`).
- D: Hybrid.

**Answer:** C. Repurpose `id` as a type tag.

**Implication (original):** v0.1 payload semantics use BIP-93's `id` field for typing. Locked tag set: `seed`, `entr`, `xprv`, plus reserved-not-emitted `mnem`, `prvk`. **But:** in v0.2 when K-of-N shares ship, `id` MUST revert to BIP-93's "random per secret-set" semantics so shares group correctly. The v0.1 → v0.2 migration must move the type discriminator off `id`. To make that migration non-breaking for v0.1 strings, v0.1 also reserves a `0x00` payload-prefix byte that v0.2 promotes to a type discriminator.

**r6 amendment (2026-05-03):** locked tag set is **`entr` emit-and-accept; `seed` / `xprv` / `mnem` / `prvk` reserved-not-emitted (decoder rejects with `Error::ReservedTagNotEmittedInV01` or analog).** The id-as-discriminator + 0x00-prefix mechanism is **preserved unchanged for `entr`**: a v0.1 entr string with prefix `0x00` and id `entr` remains forward-readable by v0.2 decoders. v0.2 promotion of `0x01..0xFF` as type discriminators still applies for any future payload kind that fits the BIP-93 short bracket. Out-of-bracket payloads (`seed`, `xprv`) need a separate v0.2+ design that doesn't ride on BIP-93's existing brackets.

### Q5 — broader product scope (the toolkit framing)

The user clarified ms1 is part of a larger toolkit: "We are going for a toolkit that uses ms1, mk1, and md1 strings for backup/share splitting as well as allows users to input xpubs, wallet descriptor templates, policies, miniscripts, xpriv, seed phrase, etc. The challenge users face is that most only have a seed phrase that is hard to engrave correctly; ms1 strings are optimized for engraving; mk-codec and md-codec are extending this idea to keys and wallets… we want to be permissive on input and as expressive as input permits for output."

**Implication:** the four-format architecture (`md1` + `mk1` + `ms1` + `mnemonic-toolkit`). v0.1 of this repo is just the foundation; the toolkit lives in a separate repo and depends on all three codecs as published artifacts. v0.1 of ms-codec must not paint the toolkit into corners.

## Decisions locked from this brainstorm

1. **Format identity:** ms1 = HRP `ms`, BIP-93 codex32 used directly via `rust-codex32` (CC0, exact-pin `=0.1.0`).
2. **v0.1 wire format:** BIP-93 codex32 with threshold = 0, share-index = `s`, `id` field as type tag (one of the locked `RESERVED_TAG_TABLE`), payload prefixed with reserved `0x00` byte followed by the secret bytes. **r6 amendment (2026-05-03):** unchanged for the v0.1 emitted tag (`entr`); see decision 3.
3. **Payload set (r6 amendment 2026-05-03):** v0.1 emit-and-accept: **`entr` only** (BIP-39 entropy, 16/20/24/28/32 B). Reserved-not-emitted (decoder rejects with `Error::ReservedTagNotEmittedInV01` or analog): `seed`, `xprv`, `mnem`, `prvk`. **Original closure said {seed, entr, xprv}; the pre-SPEC spike (§"Wire-format spike findings" below) found that `seed` (64 B) + 0x00 prefix overflows BIP-93 codex32's long-code length bracket, and `xprv` (78 B) is outside both BIP-93 brackets at any length.** The B-use-case from Q2 (BIP-32 master seed backup) is preserved via the routing `BIP-39 seed phrase → entropy → ms1 entr → engrave → recover → BIP-39 mnemonic → PBKDF2 → master seed`; direct `seed` / `xprv` payloads are deferred to v0.2+ with their own framing.
4. **v0.1 → v0.2 migration:** designed up front and locked in MIGRATION.md. Reserved-prefix byte makes the migration non-breaking for v0.1 strings; v0.2 grouping invariant gates on the prefix; v0.2 encoder anti-collision invariant prevents a v0.2 random `id` from colliding with a v0.1 reserved tag; API back-compat preserves `encode()` signature with a new additive `encode_shares()` overload. **r6 amendment:** for entr only (the only emitted v0.1 payload). v0.2's K-of-N share encoding for `entr` rides this contract directly. v0.2's `seed` / `xprv` payload framing is a *separate* design problem — not subject to this v0.1 → v0.2 migration contract because they are not v0.1-emitted.
5. **Repo and crate layout:** new sibling repo `bg002h/mnemonic-secret`, library-only at v0.1, `crates/ms-cli` reserved as a placeholder for v0.x.
6. **Cross-repo:** four-way star; mc-codex32 shared-crate extraction retired; the *pattern* (HRP-mixed BCH with per-format target residue) will be documented in a future cross-repo `PATTERNS.md`.
7. **Shipping discipline:** MSRV `1.85` (lockstep with md-codec), three-row CI matrix (stable + beta + MSRV), `Tag` constants SemVer-stable from v0.1.0, `Payload` and `InspectReport` `#[non_exhaustive]` from day 1 (one-way door, accepted).

The SPEC at `design/SPEC_ms_v0_1.md` translates these decisions into the wire-format specification. The IMPLEMENTATION_PLAN at `design/IMPLEMENTATION_PLAN_ms_v0_1.md` translates the SPEC into a phase-by-phase build sequence with TDD discipline per phase.

## Wire-format spike findings (2026-05-03, r6 amendment)

After plan-mode r5 convergence and before the SPEC was drafted, a pre-SPEC encode/decode spike against the locked dependency `rust-codex32 = "=0.1.0"` found a wire-format issue that none of the r1..r5 reviewer rounds caught (they were doing logical/architectural review without an execution spike).

**The math:** BIP-93 codex32 (per the BIP itself, and as implemented in `rust-codex32 v0.1.0`'s `Codex32String::from_string`) accepts only two specific length brackets — short (raw payload 16–44 B) and long (raw payload 63–64 B). Empirical probe over data sizes 60..82 confirmed: encoder produces strings the decoder rejects with `InvalidLength` for every byte count outside {16–44, 63–64}.

| candidate v0.1 payload | data.len (incl. 0x00 prefix) | encoded str.len | from_string |
|---|---|---|---|
| `entr-16` | 17 | 50 | ✓ short |
| `entr-32` | 33 | 75 | ✓ short |
| `seed-64` | 65 | 128 | ✗ `InvalidLength(128)` (long max = 127) |
| `xprv-78` | 79 | 151 | ✗ `InvalidLength(151)` (out-of-bracket) |
| `seed-64` (bare) | 64 | 127 | ✓ long (BIP-93 reference vector) |
| `xprv-78` (bare) | 78 | 149 | ✗ `InvalidLength(149)` (no BIP-93 bracket fits) |

**The interaction:** three locked decisions are mutually inconsistent at most two-at-a-time — payload set {seed, entr, xprv} + 0x00 reserved-prefix byte + exact-pin `rust-codex32 = "=0.1.0"` no-fork. Spike confirmed Option C (vendor/fork to widen the long-code bracket) would be re-deriving BCH parameters, far heavier than originally framed. Option B (drop the prefix byte) would lose the v0.2 non-breaking-migration property. **Option A (drop seed/xprv; v0.1 = entr only) is the cleanly-supported path** and preserves the headline engraving thesis via the BIP-39 seed-phrase → entropy routing described in Q2's r6 amendment. Option A locked by user 2026-05-03.

**Workflow lesson** (mirrored in `feedback_spike_before_locking_wire_format.md` memory entry): plan-mode reviewer loops should include an explicit "verify round-trip against the actual pinned dep before locking the plan" step for any wire-format design riding on locked external deps. The r1..r5 reviewers all passed logical sanity but missed the empirical limit. This is parallel to the existing `audit_before_extending` discipline (which covers extending shipped code).
