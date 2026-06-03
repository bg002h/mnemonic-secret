# `ms1` v0.1 Design Spec — Mnemonic Secret card

**Status:** v0.1 wire format locked (entr-only after r6 amendment). Reference implementation: `crates/ms-codec/`.
**Companion documents:**

- BRAINSTORM (rationale chain): [`BRAINSTORM_ms_v0_1.md`](./BRAINSTORM_ms_v0_1.md) — including §"Wire-format spike findings (2026-05-03, r6 amendment)" which records the seed/xprv removal.
- Migration contract: [`../MIGRATION.md`](../MIGRATION.md) — v0.1 → v0.2 invariants.
- Plan-mode meta-plan (out-of-tree): `/home/bcg/.claude/plans/c-ultimately-what-we-quirky-avalanche.md` — converged at r6.
- FOLLOWUPS: [`FOLLOWUPS.md`](./FOLLOWUPS.md) — open items + cross-repo coordination.
- Sibling SPECs (precedent for structure): `bg002h/mnemonic-key/design/SPEC_mk_v0_1.md`, `bg002h/descriptor-mnemonic/design/SPEC_v0_*.md`.

This document specifies the wire format for `ms1`-prefixed strings. Unlike its sibling formats `md1` and `mk1` — which fork BIP-93 BCH plumbing locally with HRP-mixed per-format target residues — `ms1` is **BIP-93 codex32 used directly** via Andrew Poelstra's `rust-codex32 = "=0.1.0"` crate (CC0). The "spec" here is therefore mostly: which BIP-93 wire fields carry which payload semantics, what the v0.1 → v0.2 migration contract is, and what an implementer must do to avoid drifting from BIP-93.

---

## §1. Scope

`ms1` encodes **secret material** for Bitcoin self-custody backups, intended to engrave alongside `mk1` (xpub) and `md1` (descriptor / wallet template) cards as a coherent restoration bundle.

In scope for v0.1:

- **BIP-39 entropy** (16/20/24/28/32 B = 128/160/192/224/256 bits, corresponding to 12/15/18/21/24 BIP-39 words). Tag: `entr`. The byte length is the unambiguous discriminator since BIP-39 word counts are bijective with byte counts.

Out of scope for v0.1 (deferred to v0.2+ with own framing):

- **Direct BIP-32 master seed** (64 B). Reserved tag `seed`, decoder rejects in v0.1. The pre-SPEC spike against `rust-codex32 v0.1.0` confirmed that 64 B + the v0.2-migration-required `0x00` prefix byte produces a 128-character string, one past BIP-93's long-code maximum of 127. The master-seed backup use case is preserved at the application layer via the BIP-39 routing in §1.2.
- **Serialized BIP-32 xpriv** (78 B). Reserved tag `xprv`, decoder rejects in v0.1. Outside both BIP-93 brackets at any length, with or without the prefix byte. Will require a sub-format outside BIP-93 codex32 (separate HRP, or forked BCH parameters) in v0.2+.
- **Other future payload kinds** (`mnem` reserved for "entropy + wordlist hint" payloads, `prvk` reserved for raw secp256k1 32-B private keys). Decoders reject.
- **K-of-N share encoding** for any payload kind. v0.1 always emits BIP-93 threshold = 0 (single-string secret); v0.2 will introduce share encoding for `entr` first.

### §1.1 Engraving as the load-bearing physical use case

Every wire-format and tooling decision in `ms1` is judged against "does this make a steel-plate backup more correct, or less?" BIP-39's 4-bit checksum is too weak to localize errors on engraved media; codex32's BCH is approximately 8 character substitutions detectable, 4 correctable (per BIP-93 §"Error Correction"). Encoding the entropy of a BIP-39 mnemonic as `ms1 entr` therefore produces a strictly more error-tolerant engravable backup than the mnemonic itself.

### §1.2 Recovery routing for BIP-32 master seeds (v0.1)

A user who possesses a BIP-39 seed phrase recovers their BIP-32 master seed without needing direct `seed` payload support:

```
BIP-39 seed phrase  →  entropy bytes (16/20/24/28/32 B; BIP-39 wordlist lookup, deterministic)
                    →  ms1 encode entr  →  ms1 string  →  engrave

ms1 string  →  ms1 decode entr  →  entropy bytes
            →  BIP-39 mnemonic (wordlist lookup + 4-bit checksum recompute)
            →  PBKDF2-HMAC-SHA512(mnemonic + passphrase, "mnemonic" + passphrase, 2048)
            →  64-B BIP-32 master seed
            →  BIP-32 derivation
```

The recovery chain re-derives the BIP-32 master seed from the entropy + (optional) passphrase. The `mnemonic-toolkit` crate (separate repo, deferred) wraps this routing as a transparent CLI.

### §1.3 Relationship to BIP-93 §"Not BIP-0039 Entropy"

BIP-93 itself contains a section (`bip-0093.mediawiki` §"Not BIP-0039 Entropy") arguing against encoding BIP-39 entropy in codex32. The authors' three concerns and `ms1` v0.1's response:

| BIP-93 author concern | `ms1` v0.1 response |
|---|---|
| BIP-39's 4-bit checksum overhead doesn't fit in the short bracket alongside 128-bit entropy. | `ms1` does **not** put the BIP-39 checksum on the wire. The BIP-39 mnemonic is re-derived at decode time from entropy + the standard wordlist + recomputed checksum. Wire payload is raw entropy (16-32 B) only. |
| BIP-39's language/wordlist multiplicity means a non-English user could silently recover the wrong mnemonic from English-defaulted wallet software. | **Real concern, documented in §6.3.** v0.1 does not encode the wordlist language on the wire. Users with non-English wallets MUST record their wordlist language alongside the engraved card. The reserved-not-emitted tag `mnem` is allocated for a future v0.2+ "entropy + wordlist hint" payload that addresses this. |
| BIP-39's PBKDF2 overhead (2048 iterations) for re-deriving the master seed. | `ms1` accepts this overhead as the cost of avoiding direct master-seed payloads in v0.1 (which can't fit in BIP-93's brackets with the v0.2-migration prefix byte anyway). The PBKDF2 step happens once at recovery. |

The BIP-93 authors' final recommendation — "encode the entropy of a BIP-0032 master seed" — uses "entropy" to mean the 16-64-byte master seed itself, not BIP-39 entropy. v0.1 of `ms1` cannot encode that directly (see §1 out-of-scope notes); v0.2+ will.

---

## §2. String Layer

### 2.1 HRP

`ms` (lowercase, 2 characters). Separator is `1` per BIP 173, giving the prefix `ms1`.

This HRP is BIP-93 codex32's HRP. `ms1` does not introduce a new HRP — by design, an `ms1` string IS a BIP-93 codex32 string with additional payload semantics that BIP-93 itself leaves implementation-defined (the type-tag interpretation of the `id` field; see §3).

### 2.2 BCH plumbing

Delegated entirely to `rust-codex32 = "=0.1.0"`. `ms-codec` does not implement, fork, or vendor any BCH polynomial. The exact-pin is per the BRAINSTORM Q3 closure and the upstream README's note that the crate is "pretty rough" and slated for a rewrite around `rust-bech32`. The contact surface with `rust-codex32` is concentrated in `crates/ms-codec/src/envelope.rs` so a future `codex32 = "0.2"` rewrite is absorbable in one module.

`ms1` v0.1 emits and accepts only the **short codex32 checksum** (13 codex32 characters). The long checksum (15 characters) is unused in v0.1 because the v0.1 emit-tag (`entr`, 16-32 B + 1-B prefix) always fits in the short bracket. v0.1 decoders MUST also reject long-checksum strings (see §4).

### 2.3 BIP-93 conformance — threshold and share-index

`ms1` v0.1 always emits BIP-93 threshold = `0` and share-index = `s`. This is non-negotiable per BIP-93's specification. From `bip-0093.mediawiki` §"Specification" (lines 73-76 of the canonical mediawiki source at `https://github.com/bitcoin/bips/blob/master/bip-0093.mediawiki`):

> A threshold parameter, which MUST be a single digit between "2" and "9", or the digit "0".
> *** If the threshold parameter is "0" then the share index, defined below, MUST have a value of "s" (or "S").
>
> A share index, which is any bech32 character. Note that a share index value of "s" (or "S") is special and denotes the unshared secret (see section "Unshared Secret").

And from §"Unshared Secret" (lines 146-158):

> When the share index of a valid codex32 string (converted to lowercase) is the letter "s", we call the string a codex32 secret.
> [...]
> For an unshared secret, the threshold parameter (the first character of the data part) is ignored (beyond the fact it must be a digit for the codex32 string to be valid).
> We recommend using the digit "0" for the threshold parameter in this case.

`ms1` v0.1 emit-side: threshold = `0` always. `ms1` v0.1 decode-side: rejects threshold ≠ `0` (`Error::ThresholdNotZero`) and rejects share-index ≠ `s` (`Error::ShareIndexNotSecret`). The latter is also enforced upstream by `rust-codex32 v0.1.0`'s parse (see `Codex32String::parts_inner` line 202-204), but `ms-codec` re-checks for defense-in-depth and to surface a domain-typed error to its callers.

### 2.4 Length envelope

v0.1 ms1 strings ride only the BIP-93 **short code** bracket. With the wire-format below (HRP=2 + sep=1 + threshold=1 + id=4 + share-index=1 + payload + cksum=13 = 22 fixed chars + payload), the v0.1 string lengths are:

| BIP-39 word count | entropy bytes | `data` (incl. 0x00 prefix) | payload symbols | total str.len |
|---|---|---|---|---|
| 12 | 16 | 17 | 28 | 50 |
| 15 | 20 | 21 | 34 | 56 |
| 18 | 24 | 25 | 40 | 62 |
| 21 | 28 | 29 | 47 | 69 |
| 24 | 32 | 33 | 53 | 75 |

All five lengths sit well inside the BIP-93 short bracket (codex32 reference accepts data part 45-93 chars, total 48-96; `rust-codex32 v0.1.0` accepts total 48-93). v0.1 decoders MUST reject any total-length outside this set (§4 rule 9).

**Padding bits.** When the payload's total bit-length is not a multiple of 5, the final 5-bit codex32 symbol carries 1-4 padding bits at the low end. Per BIP-93 §"Unshared Secret" line 155 ("we do NOT require that the incomplete group be all zeros"), encoders MAY emit any pad-bit value and decoders MUST ignore them. Concretely, the entr lengths above produce final-symbol pad bits of {4, 2, 0, 3, 1} for 16/20/24/28/32-B entropy respectively.

### 2.5 Wire field assignments

For a v0.1 ms1 string parsed into BIP-93 fields:

| BIP-93 field | v0.1 ms1 value |
|---|---|
| HRP | `ms` (lowercase) |
| separator | `1` |
| threshold | `0` (digit zero) |
| id | the **type tag** — one of the codex32-alphabet 4-char values from `RESERVED_TAG_TABLE` (§3.3); v0.1 emits `entr` only |
| share-index | `s` |
| payload | `0x00` reserved-prefix byte ‖ entropy bytes (16/20/24/28/32 B) |
| checksum | BCH(short) — 13 codex32 chars |

The `id` field carries the type-tag in v0.1 because BIP-93 itself leaves the field's *content* implementation-defined (BIP-93 says only "an identifier consisting of 4 bech32 characters" and "We do not define how to choose the identifier, beyond noting that it SHOULD be distinct for every master seed and share set the user may need to disambiguate"). v0.2 reverts `id` to BIP-93's recommended random-per-secret-set semantics (so K-of-N shares can group correctly) and migrates the type tag to the first payload byte; see §5.

---

## §3. Payload Semantics

### 3.1 The 0x00 reserved-prefix byte

Every v0.1 ms1 payload begins with a single byte of value `0x00`. In v0.1 this byte is reserved (decoder MUST reject any non-zero value with `Error::ReservedPrefixViolation`); in v0.2 it is promoted to a type discriminator. This makes the v0.2 share-encoding migration **non-breaking for v0.1 strings** — a v0.2 decoder seeing a payload prefix of `0x00` falls back to v0.1's "type tag is in BIP-93 `id` field" interpretation. See §5 for the full v0.1 → v0.2 contract.

### 3.2 `Tag` type

A `Tag` is a 4-byte value where each byte is a codex32-alphabet character (`qpzry9x8gf2tvdw0s3jn54khce6mua7l`). Construction MUST validate the alphabet at construction; out-of-alphabet bytes return `Error::TagInvalidAlphabet`. A `Tag` value that is structurally valid but not a member of `RESERVED_TAG_TABLE` returns `Error::UnknownTag` at decode time.

The `Tag` constants exposed in v0.1's public API:

```rust
pub const ENTR: Tag = Tag(*b"entr");
```

v0.1 deliberately does NOT expose `pub const SEED` or `pub const XPRV` constants. The 4-byte values `seed` and `xprv` are members of `RESERVED_TAG_TABLE` (decoder rejects on receipt with `Error::ReservedTagNotEmittedInV01`), but exposing them as public Tag constants would invite encoder misuse — the only callable `Tag` values in v0.1 should be those an encoder may legitimately emit.

`Tag` values reserved as compile-time constants are SemVer-stable from v0.1.0 onward. New tag constants added in v0.1.x or v0.2 minor releases are additive (semver-minor). Removing or renaming a tag constant is a semver-major change.

### 3.3 `RESERVED_TAG_TABLE`

Curated to ms1's actual purpose (secret material, not metadata or certificates). The table grows by SemVer-minor only.

| tag | meaning | v0.1 emit | v0.1 accept |
|---|---|---|---|
| `entr` | BIP-39 entropy (128/160/192/224/256 b = 16/20/24/28/32 B) | yes | yes |
| `seed` | BIP-32 master seed (64 B) | no (overflows BIP-93 long bracket with prefix byte) | reject (`Error::ReservedTagNotEmittedInV01`) |
| `xprv` | serialized BIP-32 xpriv (78 B stripped of Base58Check) | no (outside BIP-93 brackets at any length) | reject (`Error::ReservedTagNotEmittedInV01`) |
| `mnem` | reserved for future "entropy + wordlist-language hint" payload (length TBD in v0.2+; see §6.3) | no | reject (`Error::ReservedTagNotEmittedInV01`) |
| `prvk` | reserved for future raw secp256k1 32-B private key (length 32 B if used directly; final framing TBD in v0.2+) | no | reject (`Error::ReservedTagNotEmittedInV01`) |

Tags structurally valid (alphabet-conforming) but not in the table cause `Error::UnknownTag`.

### 3.4 `Payload` type (v0.1)

```rust
#[non_exhaustive]
pub enum Payload {
    Entr(Vec<u8>),    // 16/20/24/28/32 B (BIP-39 entropy)
}
```

The `#[non_exhaustive]` attribute is permanent from v0.1.0 onward; future variants (e.g., `Mnem`, `Seed`, `Xprv` once their framing is settled) are semver-minor additions. Removing `#[non_exhaustive]` would be semver-major and is not contemplated.

### 3.5 `Payload::Entr` byte-length validation

Encoder MUST reject `Payload::Entr(data)` with `data.len() ∉ {16, 20, 24, 28, 32}` (`Error::PayloadLengthMismatch { tag: Tag::ENTR, expected: <set>, got }`). Decoder applies the same check after extracting the payload bytes following the prefix byte.

The set `{16, 20, 24, 28, 32}` corresponds bijectively to BIP-39 word counts `{12, 15, 18, 21, 24}`. ms-codec does not separately carry the word count on the wire; the byte length is the unambiguous discriminator.

### 3.5.1 Encoder symmetry on reserved-not-emitted tags

Encoder MUST reject any `Tag` that is reserved-not-emitted in v0.1 with `Error::ReservedTagNotEmittedInV01` (the same variant decoder rule 7 raises). v0.1's only emit-tag is `Tag::ENTR`; passing `Tag::try_new("seed")`, `Tag::try_new("xprv")`, `Tag::try_new("mnem")`, or `Tag::try_new("prvk")` to `encode()` MUST fail at the encoder boundary, not produce a string that the decoder will then reject. The encoder check mirrors decoder rule 7 in §4 to prevent an asymmetry where a v0.1 ms-codec could emit a string the v0.1 ms-codec itself cannot decode.

### 3.6 Caller responsibility for entropy quality

`ms-codec` does NOT check the statistical quality of bytes passed to `Payload::Entr`. Callers are responsible for sourcing entropy from a vetted CSPRNG (or from a BIP-39 mnemonic the user already trusts). The doc-comment on `Payload::Entr` MUST say this explicitly to dissuade an implementer from adding FIPS-style entropy-quality checks. Such checks would slow encoding and provide false assurance — they cannot detect, e.g., an attacker-supplied "pseudo-random" seed crafted to pass standard randomness tests.

---

## §4. Decoder Validity Rules

A v0.1 ms-codec decoder MUST reject input that:

1. Fails BIP-93 codex32 parsing in `rust-codex32` (delegated; `Error::Codex32(<inner>)`). This covers: invalid checksum, invalid character, mixed case, unsupported overall length, etc.
2. Has HRP ≠ `ms` (`Error::WrongHrp`). The upstream `rust-codex32 v0.1.0` parser is HRP-agnostic; `ms-codec` re-checks.
3. Has threshold ≠ `0` (`Error::ThresholdNotZero`). v0.1 is single-string only. **(v0.2 amendment, SPEC_ms_v0_2_kofn §1: this hard-reject is RELAXED into a route — threshold ∈ `'2'..'9'` now means "one share of a K-of-N set" and surfaces `Error::IsShareNotSingleString { threshold, index }` directing the user to `ms combine`; only a threshold byte that is neither `'0'` nor `'2'..'9'` still raises `ThresholdNotZero`.)**
4. Has share-index ≠ `s` (`Error::ShareIndexNotSecret`). Required for threshold = 0 per BIP-93 §"Specification". **(v0.2 amendment: this check is now scoped to the threshold = `'0'` path only — a distributed share carries a non-`s` index by design; the share path is routed by rule 3 before this check.)**
5. Has an `id` field (type tag) that is structurally invalid (out-of-codex32-alphabet bytes — unreachable for inputs that pass BIP-93 parsing, but checked defensively): `Error::TagInvalidAlphabet`.
6. Has an `id` field that is a structurally valid 4-byte tag but is not a member of `RESERVED_TAG_TABLE` (`Error::UnknownTag`).
7. Has an `id` field that is a member of `RESERVED_TAG_TABLE` but is reserved-not-emitted in v0.1 (currently `seed`, `xprv`, `mnem`, `prvk`): `Error::ReservedTagNotEmittedInV01`.
8. Has a payload prefix byte ≠ `0x00` (`Error::ReservedPrefixViolation`). v0.2 promotes this byte to a type discriminator; v0.1 must reject any non-zero value to lock the v0.2-migration contract from day 1.
9. Has a total string length outside the v0.1-emittable set `{50, 56, 62, 69, 75}` (`Error::UnexpectedStringLength { got, allowed: [50, 56, 62, 69, 75] }`). This rule rejects, in particular, any long-checksum string (length ≥ 125).
10. After extracting the payload bytes following the `0x00` prefix, has a payload length not matching the entr length set `{16, 20, 24, 28, 32}` (`Error::PayloadLengthMismatch { tag: Tag::ENTR, expected: [16, 20, 24, 28, 32], got }`). This rule is reachable in principle if rule 9 is somehow bypassed — defensive only.

Note: BIP-93's §"Unshared Secret" explicitly says (line 155) "we do NOT require that the incomplete group be all zeros." `ms-codec` follows this — the trailing pad bits of the final 5-bit codex32 symbol may be any value; the decoder ignores them. (The upstream `rust-codex32 v0.1.0`'s `Parts::data()` discards them per BIP-93.)

---

## §5. v0.1 → v0.2 Migration Contract

Restated for SPEC-locality. **v0.2 (SHIPPED, `SPEC_ms_v0_2_kofn.md`) adds K-of-N share encoding for both `entr` and `mnem`** while preserving forward-readability of v0.1/mnem single-strings. The four invariants below were AMENDED for the shipped v0.2 design (the original v0.1-authored text predicted a `0x01 = entr-share` prefix that turned out crypto-invalid — a distributed share's first payload byte is a Lagrange-interpolation output, not a stable prefix — so share-ness moved to the orthogonal *threshold* axis):

1. **Prefix byte is the payload-KIND discriminator; share-ness is the threshold field (orthogonal axes).** The byte at the start of the BIP-93 codex32 payload is the *kind* discriminator: `0x00` = entr, `0x02` = mnem; **`0x01` is UNALLOCATED** (the planned `0x01 = entr-share` was dropped). Share-vs-single lives on a separate axis: the codex32 **threshold char** (`'0'` = single; `'2'..'9'` = one share of a K-of-N set). The prefix byte is meaningful only on the recovered **secret-at-S**. A share of an `entr` secret recovers to a `0x00` payload; a share of a `mnem` secret recovers to a `0x02` payload. v0.1 entr/mnem singles (threshold `'0'`) remain forward-readable unchanged.

2. **Threshold-field dispatch invariant.** A reader MUST dispatch on the **threshold char first**, before interpreting the payload byte or grouping by `id`. `threshold == '0'` → the v0.1 single-string decode path (then `0x00`/`0x02` via the prefix byte). `threshold ∈ '2'..'9'` → this is one share of a K-of-N set; group by `id` and recombine via `combine_shares` — a single-string `decode` MUST NOT route it into the prefix-byte dispatch (its payload byte is garbage → misleading `ReservedPrefixViolation`); `decode` surfaces `Error::IsShareNotSingleString { threshold, index }` instead. Distributed shares carry no stable prefix byte, so dispatch is on the threshold field, never on the payload byte.

3. **v0.2 encoder anti-collision invariant.** A K-of-N share-set uses a **random per-set 4-char `id`** (BIP-93 random-per-set semantics; NOT a type tag). The encoder MUST refuse to emit an `id` in `RESERVED_ID_BLOCKLIST` (`entr`, `seed`, `xprv`, `mnem`, `prvk`) — re-roll on collision (rate ≈ 5 / 32⁴ ≈ 1 in 209 715, negligible). `mnem` stays in the id-blocklist even though it was dropped from the decoder-reject set `RESERVED_NOT_EMITTED_V01` in Cycle 1 (distinct const tables). The unshared (`ZERO`) path keeps `id = tag` — random `id` is ONLY for `k ∈ 2..9` share-sets (load-bearing for byte-identity).

4. **API + byte-identity.** v0.1's `pub fn encode(tag: Tag, payload: &Payload) -> Result<String>` is preserved unchanged. v0.2 adds `pub fn encode_shares(tag: Tag, threshold: Threshold, n: usize, secret: &Payload) -> Result<Vec<String>>` and `pub fn combine_shares(shares: &[String]) -> Result<(Tag, Payload)>` (`Threshold` is a v0.2-introduced type with a `ZERO` const + `new(2..=9)`). The byte-identity CLAIM holds — `encode_shares(tag, Threshold::ZERO, 1, &p)` is wire-bit-identical to `encode(tag, &p)` (both reduce to `from_seed(HRP, 0, tag, Fe::S, [prefix]||payload)`) — but the originally-planned `payload_set: &[Payload]` SIGNATURE was wrong and is replaced by `(tag, threshold, n, &secret)`. Bounds: `2 ≤ k ≤ n ≤ 31` (31 valid non-`s` share indices; `n = 32` exhausts).

**Prefix-byte registry (kind axis), orthogonal to the threshold axis (share vs single):** `0x00` = entr · `0x01` = **unallocated** · `0x02` = mnem · `0x03..0xFF` = unallocated (claim-via-PR). threshold `'0'` = single · `'2'..'9'` = K-of-N share-set.

These invariants are also captured in `MIGRATION.md` + `SPEC_ms_v0_2_kofn.md`, and the source comments of `crates/ms-codec/src/envelope.rs`.

---

## §6. Engraving and Privacy

### §6.1 Why engraving entropy beats engraving the mnemonic

A 24-word BIP-39 mnemonic engraved on steel, read with errors, has a 4-bit checksum to localize the errors with — far too weak. A user who reads two ambiguous letters cannot determine which was the read error and which the engraving error. The same 32 bytes of underlying entropy encoded as `ms1 entr` on the same medium has a 13-character BCH checksum that detects up to 8 character substitutions and corrects up to 4 — so the user can reconstruct the entropy even from a partially-corroded card, then re-derive the mnemonic deterministically.

### §6.2 Cross-format engraving

An `ms1 entr` card co-engraves with `mk1` xpub cards and an `md1` descriptor card to form a complete restoration bundle for a single-sig BIP-39 wallet. For multisig: the `ms1 entr` card on each cosigner's secret + the `mk1` xpub cards on every cosigner's public key + the shared `md1` descriptor card. Recovery: decode all three formats, re-derive the master seed from each `ms1 entr` (with each cosigner's passphrase if any), regenerate xprivs at the paths declared in `md1`, sign.

### §6.3 BIP-39 wordlist language — recovery hazard

BIP-39 entropy → mnemonic conversion depends on the wordlist language (English, Japanese, Spanish, French, Italian, Korean, Chinese Simplified, Chinese Traditional, Czech, Portuguese). The wordlist is NOT carried on the v0.1 ms1 wire. A user whose original wallet used a non-English wordlist who recovers via English-defaulted wallet software will silently derive a different 64-B BIP-32 master seed → different addresses → empty wallet. This is exactly the concern raised in BIP-93 §"Not BIP-0039 Entropy" (lines 411-415).

**Normative recommendations** (informational, not enforced by the format):

- Users MUST record their BIP-39 wordlist language alongside the engraved `ms1 entr` card. The recording need not be machine-readable; "English" stamped on the back of the steel plate is sufficient. English is the de-facto default and many wallets do not expose a language picker.
- Wallet software that decodes `ms1 entr` SHOULD expose a wordlist-language selector and SHOULD warn the user that the default (English) may not match their original mnemonic.
- **Addressed in v0.2 (ms-codec 0.3.0):** the `mnem` payload kind records the wordlist language on the wire. It is carried by a new `0x02` prefix byte (not a reserved tag — the wire tag stays `entr`), with the layout `[0x02][language][entropy]`: a **1-byte language field** (only the low nibble is used — values 0–9 index the 10 BIP-39 wordlists, English = 0). The field is byte-aligned, not the 4-bit-packed form this note originally anticipated: codex32's `sanity_check` rejects a sub-byte-padded payload for the 15/18/24-word entropy lengths, so a whole language byte is required. v0.1 decoders still reject the `0x02` prefix; see `SPEC_ms_mnem_wordlist_language.md` for the v0.2 specification.

### §6.4 Engraving the post-passphrase 64-B master seed is an anti-pattern

A user with a BIP-39 passphrase (the optional "25th word") might be tempted to engrave the resulting 64-B BIP-32 master seed directly to avoid having to remember the passphrase at recovery. v0.1 cannot facilitate this (no `seed` payload), and this is **intentional**: if the post-passphrase master seed is engraved, the passphrase has no defensive value — the engraved card alone is sufficient to spend. The whole point of a passphrase is that the seed alone is insufficient. v0.2+ direct `seed` framing, when it lands, SHOULD surface this anti-pattern to callers (e.g., warn-by-default at the encoder, with an explicit opt-in flag); v0.1 cannot bind v0.2's API normatively, but the recommendation should carry forward.

### §6.5 Privacy footprint

An engraved or photographed `ms1 entr` card reveals the full BIP-39 entropy → with the wordlist language and any passphrase, the full BIP-32 derivation tree, all xprivs, all addresses, all transaction history. The card alone is therefore equivalent in spend-authority to the BIP-39 mnemonic. Storage discipline:

- Engrave on durable physical media (steel, titanium, etc.).
- Store with the same physical security as a seed-phrase backup (safety-deposit box, fire safe).
- Never photograph.
- Treat any hand-off (e.g., to a wallet creator at provisioning time) with the same operational discipline as long-term storage.

---

## §7. Family-stable generator string

For wire-format SHA-pinning in test vector files, the family-stable token is `ms-codec X.Y` where `X.Y` is the major.minor version of the reference implementation's published crate. Mirrors the convention in `mk-codec` and `md-codec`. Patch-version bumps don't roll the token; a token roll requires a wire-format change (which forces a minor bump per the pre-1.0 SemVer convention).

---

## §8. Out-of-scope items deferred

| Item | Rationale | Future version |
|---|---|---|
| K-of-N share encoding for `entr` **and `mnem`** | **SHIPPED ms-codec v0.4.0** (`SPEC_ms_v0_2_kofn.md`); via codex32-native threshold(k)+index Shamir, dispatched on the threshold field (§5 amended). Broadened from the v0.1-planned entr-only scope to entr+mnem (language survives the split). | ms-codec v0.4.0 ✓ |
| Direct `seed` (64-B BIP-32 master seed) payload | Overflows BIP-93 long bracket with the v0.2-migration prefix byte; needs sub-format outside BIP-93 brackets | ms-codec v0.2+ |
| Direct `xprv` payload | Outside BIP-93 brackets at any length; same constraint | ms-codec v0.2+ |
| `mnem` (entropy + wordlist-language hint) | v0.1 documents the wordlist hazard but doesn't encode the language; reserved tag for v0.2+ | ms-codec v0.2+ |
| `prvk` (raw secp256k1 32-B private key) | No identified use case at v0.1 design time; reserved tag for future expansion | ms-codec v0.X |
| `ms-cli` companion binary | Library-first release pattern (mirrors md-codec → md-cli sequencing); placeholder reserved in workspace at `crates/ms-cli/` per `BRAINSTORM_ms_v0_1.md` | ms-cli v0.1 |
| `mnemonic-toolkit` (BIP-39 phrase → ms1 + mk1 + md1 bundle) | Separate repo `bg002h/mnemonic-toolkit`; depends on this crate as a published artifact | toolkit v0.1 |

---

## §9. Closures from BRAINSTORM

The five closures from `BRAINSTORM_ms_v0_1.md` (with r6 amendments where applicable) drive the wire format above:

| ID | Locked answer (v0.1) | SPEC section |
|---|---|---|
| Q-1 | New sibling format with HRP `ms`; eventually K-of-N shares for the whole m-format family | §1, §2.1 |
| Q-2 | BIP-39 entropy (r6: only — direct master seed and xpriv deferred to v0.2+) | §1, §1.2 |
| Q-3 | Use BIP-93 codex32 directly via `rust-codex32 = "=0.1.0"`; no fork | §2.2 |
| Q-4 | Repurpose BIP-93 `id` as type tag (r6: only `entr` is v0.1-emit; v0.2 reverts `id` to BIP-93 random semantics, type tag moves to prefix byte) | §2.5, §3, §5 |
| Q-5 | Permissive-input / expressive-output toolkit framing — out of scope for this SPEC; depended on by future `mnemonic-toolkit` | §1 (in-scope), §8 |

---

## §10. Reference implementation

`crates/ms-codec/` — the v0.1 deliverable. Layout:

```
crates/ms-codec/src/
├── lib.rs            — re-exports; doc the v0.2 migration up front
├── consts.rs         — HRP "ms"; threshold-0 invariant; RESERVED_TAG_TABLE
├── error.rs          — Error enum (decode-domain + payload-domain variants)
├── tag.rs            — Tag type: 4-char codex32-alphabet validation + reserved list
├── payload.rs        — Payload enum (v0.1: Entr only) + serialize/parse
├── envelope.rs       — THE v0.2-MIGRATION SEAM (the only module that contacts rust-codex32)
├── encode.rs         — encode(tag, payload) -> ms1 string (delegates to envelope.rs)
├── decode.rs         — decode(ms1 string) -> (Tag, Payload); rejects per §4
└── inspect.rs        — structural dump for debugging / future ms-cli inspect
crates/ms-codec/tests/
├── round_trip.rs     — proptest per BIP-39 byte length (16/20/24/28/32)
├── vectors.rs        — versioned vector corpus replay
└── vectors/
    └── v0.1.json     — wire-format SHA pinned at v0.1.0 release per RELEASE_PROCESS.md
```

Public API surface (sketch — IMPLEMENTATION_PLAN refines):

```rust
#[non_exhaustive]
pub enum PayloadKind { Entr }

#[non_exhaustive]
pub enum Payload {
    Entr(Vec<u8>),    // 16/20/24/28/32 B
}

pub struct Tag(pub [u8; 4]);
impl Tag {
    pub const ENTR: Tag;
    pub fn try_new(s: &str) -> Result<Self>;
}

pub fn encode(tag: Tag, payload: &Payload) -> Result<String>;
pub fn decode(ms1: &str) -> Result<(Tag, Payload)>;
pub fn inspect(ms1: &str) -> Result<InspectReport>;

#[non_exhaustive]
pub struct InspectReport {
    pub hrp: String,                       // expected "ms"
    pub threshold: u8,                     // expected 0 in v0.1
    pub tag: Tag,                          // the parsed type tag
    pub share_index: char,                 // expected 's' in v0.1
    pub prefix_byte: u8,                   // 0x00 in v0.1 (reserved); becomes type discriminator in v0.2+
    pub payload_bytes: Vec<u8>,            // payload after the prefix byte
    pub checksum_valid: bool,              // BCH verification result
}
```

### §10.1 `rust-codex32 v0.1.0` API contact note

The plan locked an exact-pin to `rust-codex32 = "=0.1.0"`. Pre-SPEC source review of that crate found that its public `Parts` struct (`crates/codex32-0.1.0/src/lib.rs:383-392`) has **non-`pub` fields**: only `parts.data() -> Vec<u8>` is publicly accessible. Direct field access to `hrp`, `threshold`, `id`, `share_index`, `payload`, `checksum` from outside the crate is a compile error.

This affects `decode.rs` and `inspect.rs`: validation rules §4 #2-4 (HRP / threshold / share-index checks) cannot be performed by reading `Parts` fields. Two acceptable strategies:

- **Wire-position re-parse.** `envelope.rs` re-parses the BIP-93-validated string itself, extracting HRP / threshold / id / share-index from known wire positions (HRP ends at `rfind("1")`, threshold at +1, id at +2..+6, share-index at +6, payload at +7..-13). This is what `rust-codex32`'s own `parts_inner` does internally; we duplicate the parse on the user-facing side. Defended by `rust-codex32`'s prior validation — we are re-parsing a known-valid BIP-93 string.
- **Upstream PR.** File a one-line PR against `rust-codex32 = "=0.1.0"`'s `Parts` struct adding `pub` to the field declarations. If accepted, exact-pin to the resulting patch release. Cleaner long-term but blocks on upstream cycle.

The IMPLEMENTATION_PLAN must specify which strategy Phase 1 takes. Default recommendation: wire-position re-parse, since it has no upstream-cycle dependency and the wire format is stable. Re-parse cost is negligible — `rust-codex32 v0.1.0`'s `Parts<'s>` is `Copy` and its `parts_inner` does the same parse internally on every `parts()` call.

### §10.2 Test strategy

**Discipline (non-negotiable):** tests land *before* implementation within each phase. Mirrors the per-phase TDD pattern documented in `descriptor-mnemonic/CLAUDE.md` and `mnemonic-key/CLAUDE.md`. The IMPLEMENTATION_PLAN must specify, per phase, which test files are written first and which are unblocked by later phases (mk1's pattern: `#[ignore]`-marked scaffolds in earlier phases get un-ignored when their code path lands).

- **Round-trip** proptests for `Payload::Entr`: encode → decode → assert equal payload + tag. Generators per the BIP-39 byte-length variants `{16, 20, 24, 28, 32}`.
- **BIP-93 cross-validation:** hand-construct an ms1 string via the BIP-93 reference vector format, decode it via ms-codec, assert the parts. Confirms our HRP / threshold / id / share-index / checksum agree with BIP-93 symbol-for-symbol.
- **Negative vectors** for each §4 rule: invalid checksum, wrong HRP, threshold ≠ 0, share-index ≠ `s`, unknown tag, reserved-not-emitted tag, non-zero prefix byte, wrong total length, wrong entr byte length.
- **Versioned vector corpus** at `crates/ms-codec/tests/vectors/v0.1.json`. Wire-format SHA pinned at v0.1.0 release per `design/RELEASE_PROCESS.md`.
- **Forward-compat smoke test:** encode a v0.1 string, manually flip the prefix byte to `0x01`, confirm decoder rejects with `Error::ReservedPrefixViolation`. Locks the v0.1 ↔ v0.2 contract.
- **BIP-39 round-trip integration** (in a separate test module gated on a dev-dep `bip39 = "..."`): take a 12/15/18/21/24-word English mnemonic, extract entropy, encode as ms1 entr, decode, re-derive the mnemonic from the entropy, confirm string-exact match. Catches any entropy-bit-misalignment regression.

### §10.3 Shipping discipline

- **MSRV (Minimum Supported Rust Version):** pin to **`1.85`** (matches md-codec, verified at `descriptor-mnemonic/Cargo.toml` and `rust-toolchain.toml`). Do not silently outpace md-codec's MSRV — toolkit downstream consumers will pin to the loosest of the three sibling crates' MSRVs. If md-codec bumps MSRV, ms-codec follows in lockstep, never leads.
- **CI matrix:** stable + beta + MSRV-pin (three-row minimum, mirroring `descriptor-mnemonic/.github/workflows/ci.yml`).
- **CI gates:** `cargo build`, `cargo test`, `cargo clippy --all-targets -D warnings`, `cargo fmt --check`. Per-phase opus reviews persist to `design/agent-reports/` per the established workflow (BRAINSTORM/SPEC/PLAN reviews stay in conversation per the 2026-05-03 refinement).
- **`Tag` constant SemVer policy:** `Tag::ENTR` is SemVer-stable from v0.1.0. Adding new tag constants is semver-minor; removing or renaming is semver-major. State this policy in the doc-comment on the `Tag` type.
- **`Payload` and `InspectReport` are `#[non_exhaustive]` from day 1.** Adding variants/fields is semver-minor; removing the attribute would be semver-major. One-way door, accepted.

---

## §11. BIP-93 anchoring

This SPEC is layered atop BIP-93 codex32. Implementers verifying conformance should consult these sections of the canonical mediawiki at `https://github.com/bitcoin/bips/blob/master/bip-0093.mediawiki`:

| BIP-93 section | What ms1 v0.1 inherits / responds to |
|---|---|
| §"Specification" → "codex32" (lines 65-85) | HRP, separator, threshold, identifier, share-index field definitions. ms1 v0.1 fixes HRP=`ms`, threshold=`0`, share-index=`s`, identifier=tag (§2). |
| §"Checksum" (lines 86-131) | BCH polynomial. ms1 v0.1 delegates entirely to `rust-codex32` (§2.2). |
| §"Error Correction" (lines 132-145) | Up to 8 character substitutions detected, 4 corrected (short code). ms1 inherits these guarantees. |
| §"Unshared Secret" (lines 146-188) | Threshold=0, share-index=`s`, payload-to-bytes conversion. Quoted verbatim in §2.3. |
| §"Master seed format" (lines 190-206) | BIP-93's recommended 16-to-64-byte BIP-32 master seed encoding. ms1 v0.1 does NOT do this; ms1 encodes BIP-39 entropy (16-32 B) instead. See §1.3. |
| §"Long codex32" (lines 309-358) | The 15-character checksum for 47-64-byte payloads. **ms1 v0.1 does NOT use long codex32.** All v0.1 payloads (16-32 B entropy + 1 prefix byte = 17-33 B) fit the short bracket. v0.1 decoder rejects long-checksum strings (§4 rule 9). |
| §"Not BIP-0039 Entropy" (lines 389-421) | The BIP-93 authors' arguments against encoding BIP-39 entropy in codex32. ms1 v0.1's response is in §1.3. |
| §"Test Vectors" (lines 441-549) | BIP-93's reference vectors. ms1 v0.1 cross-validates a subset that fits the threshold-0 / HRP-`ms` shape (§10.2). |

---

## Appendix A — provenance

This v0.1 SPEC was written 2026-05-03 against the converged plan-mode meta-plan (r5) at `/home/bcg/.claude/plans/c-ultimately-what-we-quirky-avalanche.md` and the BRAINSTORM at `design/BRAINSTORM_ms_v0_1.md`. A pre-SPEC encode/decode spike against the locked dependency `rust-codex32 = "=0.1.0"` found that the originally locked payload set {seed, entr, xprv} was incompatible with the locked `0x00` reserved-prefix byte under BIP-93 codex32's length brackets — the spike produced a r6 amendment to BRAINSTORM/MIGRATION/CLAUDE.md/plan that narrowed v0.1 to `entr` only. See FOLLOWUPS handle `ms1-v01-payload-bracket-overflow-prefix-byte-incompatibility` (primary entry in `design/FOLLOWUPS.md`; cross-repo mirrors in `mnemonic-key/design/FOLLOWUPS.md` and `descriptor-mnemonic/design/FOLLOWUPS.md`) for the discovery record. SPEC reviewer-loop reports stay in conversation per the 2026-05-03 refinement; the convergence record is captured in this SPEC's revision history below.

## Revision history

(Revision numbers below track this SPEC's iterative reviewer-loop convergence. They are independent of the meta-plan's r1..r6 sequence — "r6" elsewhere in this SPEC always refers to the meta-plan / BRAINSTORM amendment that narrowed v0.1 to `entr`-only on 2026-05-03.)

- **r1** — 2026-05-03 initial draft (entr-only v0.1 per the r6 BRAINSTORM amendment).
- **r2** — 2026-05-03 integrated 2 important findings from r1 review: §5 invariant #4 explicit note that `Threshold` is v0.2-introduced (no v0.1 public symbol); §5 invariant #2 + closing paragraph clarified that grouping-by-id semantics are scoped to `entr` shares only and that any future v0.2+ same-HRP kind MUST claim a distinct prefix-byte value and dispatch to its own kind-specific path. Three r1 nits taken inline: §2.4 padding-bits footnote (with concrete pad-bit values per entr length); §10.1 `Copy`-cost reassurance; this revision-history clarification on r-numbering. r1 had 0 critical, 6 nits / 9 affirmations remaining.
- **r3** — 2026-05-03 reviewer loop terminated (r2 returned 0 critical / 0 important, 5 nits / 8 affirmations). Four r2 nits taken inline: new §3.5.1 closing the encoder/decoder asymmetry on reserved-not-emitted tags (encoder MUST reject with `Error::ReservedTagNotEmittedInV01`, mirroring decoder rule 7); §3.3 table cells for `mnem` / `prvk` annotated with byte-length placeholders; §5 closing paragraph adds a v0.2 prefix-byte registry-table requirement (`0x02..0xFF = unallocated, claim-via-PR`); §6.4 forward-normative "MUST" softened to "SHOULD" with the explicit caveat that v0.1 SPEC cannot bind v0.2 API. Fifth r2 nit (§10.1 wire-position re-parse offset comment cosmetic) deferred to the IMPLEMENTATION_PLAN per the reviewer's recommendation.
