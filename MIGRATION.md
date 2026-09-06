# Migration guide

This file documents wire-format and API migrations across `ms-codec` minor versions. SemVer is followed with the pre-1.0 convention that the second component (`0.X`) is the breaking-change axis.

## v0.1 scope (after r6 amendment, 2026-05-03)

v0.1 ships **`entr` only** (BIP-39 entropy, 16/20/24/28/32 B). The pre-SPEC spike against `rust-codex32 = "=0.1.0"` found that `seed` (64 B) + the locked `0x00` reserved-prefix byte overflows BIP-93 codex32's long-code length bracket (128-char string, one past the bracket max of 127), and `xprv` (78 B) is outside both BIP-93 brackets at any length. See `design/BRAINSTORM_ms_v0_1.md` §"Wire-format spike findings (2026-05-03, r6 amendment)" for the empirical evidence and the FOLLOWUPS handle `ms1-v01-payload-bracket-overflow-prefix-byte-incompatibility` for the cross-repo record.

The B-use-case from the brainstorm Q2 (BIP-32 master seed backup) is preserved via the routing `BIP-39 seed phrase → entropy bytes → ms1 entr → engrave → recover entropy → BIP-39 mnemonic → (with passphrase) PBKDF2 → 64-B BIP-32 master seed`. Direct `seed` and `xprv` payloads are reserved for v0.2+, which will require BCH framing outside BIP-93's existing length brackets (separate sub-format or new HRP — design TBD).

## v0.1 → v0.2 (K-of-N shares — SHIPPED)

v0.2 adds BIP-93 codex32 **K-of-N Shamir share encoding** for **both `entr` and `mnem`** secrets: a secret splits into N shares, any K of which recombine to the original, using codex32's *native* threshold(`k`)+index Shamir mechanism. The migration is **non-breaking for v0.1/mnem single-strings**: a v0.2 decoder reads them unchanged, and a v0.1 decoder correctly rejects a v0.2 share (it sees a non-zero threshold). Four invariants lock the contract:

1. **Prefix byte is the payload-KIND discriminator; share-ness is the threshold field (orthogonal axes).** The byte at the start of the BIP-93 codex32 payload is the *kind* discriminator: `0x00` = entr, `0x02` = mnem; `0x01` is **UNALLOCATED** (the originally-planned `0x01 = entr-share` idea was crypto-invalid — a distributed share's first payload byte is a Lagrange-interpolation output, NOT a stable prefix — and was dropped). Share-vs-single lives on a **separate** axis: the codex32 **threshold char** in the header (`'0'` = single/unshared; `'2'..'9'` = one share of a K-of-N set). The prefix byte is meaningful only on the recovered **secret-at-S** (codex32 index `s`), never on a distributed share. A share of an `entr` secret recovers to a `0x00` payload; a share of a `mnem` secret recovers to a `0x02` payload.

2. **Threshold-field dispatch invariant.** A reader MUST dispatch on the **threshold char first**, before interpreting the payload byte or grouping by `id`. `threshold == '0'` → the v0.1 single-string decode path (then `0x00`/`0x02` via the prefix byte, unchanged). `threshold ∈ '2'..'9'` → this is one share of a K-of-N set; group by `id` and recombine via `combine_shares` — a single-string `decode` MUST NOT route it into the prefix-byte dispatch (its payload byte is garbage and would yield a misleading `ReservedPrefixViolation`). v0.2 `decode` surfaces `Error::IsShareNotSingleString { threshold, index }` for such a string, directing the user to `ms combine`. (This **relaxes** the v0.1 `ThresholdNotZero` hard-reject into a route.) Without the threshold-first gate, distributed shares would be mis-decoded and unrelated singles could be misgrouped.

3. **Encoder anti-collision invariant.** A K-of-N share-set uses a **random per-set 4-char `id`** (BIP-93 recommended random-per-set semantics; NOT a type tag). The encoder MUST refuse to emit an `id` that collides with `RESERVED_ID_BLOCKLIST` (`entr`, `seed`, `xprv`, `mnem`, `prvk`) — re-roll on collision (rate ≈ 5 / 32⁴ ≈ 1 in 209 715, negligible). Note `mnem` stays in the id-blocklist even though it was dropped from the decoder-reject set `RESERVED_NOT_EMITTED_V01` in Cycle 1 (the two const tables are distinct). The unshared (`ZERO`) path keeps `id = tag` — random `id` is ONLY for `k ∈ 2..9` share-sets (load-bearing for byte-identity).

4. **API + byte-identity.** v0.1's `pub fn encode(tag: Tag, payload: &Payload) -> Result<String>` is preserved unchanged. v0.2 adds `pub fn encode_shares(tag: Tag, threshold: Threshold, n: usize, secret: &Payload) -> Result<Vec<String>>` and `pub fn combine_shares(shares: &[String]) -> Result<(Tag, Payload)>` (`Threshold` is a v0.2-introduced type with a `ZERO` const + `new(2..=9)`). The byte-identity CLAIM holds — `encode_shares(tag, Threshold::ZERO, 1, &p)` is wire-bit-identical to `encode(tag, &p)` (both reduce to `from_seed(HRP, 0, tag, Fe::S, [prefix]||payload)`, deterministic) — but the originally-planned `payload_set: &[Payload]` SIGNATURE was wrong and is replaced by `(tag, threshold, n, &secret)`. Bounds: `2 ≤ k ≤ n ≤ 31` (31 valid non-`s` share indices; `n = 32` exhausts). SHA-pinned v0.1 regressions continue to pass.

The `seed` / `xprv` payload framing — out of scope for v0.1 because they don't fit BIP-93's brackets — remains a *separate* design problem (a different sub-format / HRP / widened-bracket BCH). If a future payload kind shares HRP `ms`, it MUST claim a distinct prefix-byte value (`0x01`, `0x03`, …) on the kind axis; share-ness stays on the orthogonal threshold axis.

These invariants are also captured in `design/SPEC_ms_v0_1.md` §5 (amended) + `design/SPEC_ms_v0_2_kofn.md`, and the source comments of `crates/ms-codec/src/envelope.rs`.

## v0.7 → v0.8 (the hashlock preimage kind — `0x03`, id `hash`)

v0.8 adds a THIRD payload kind on the prefix-byte axis: `0x03` = a hashlock
preimage, exactly `[0x03][X:32]` (33 bytes, a 75-character single). Five
invariants, each with a measured reason (`design/SPEC_ms_hashlock.md`):

1. **Readers that dispatch on the prefix byte MUST treat `0x03` as a 32-byte
   preimage and never as entropy.** `ms decode` prints it as kind + hex +
   digest and never as words.
2. **Length no longer implies kind.** A preimage single is 75 characters —
   exactly entr-32 — and shares entr's leading payload character `q`. So
   preimage SINGLES carry the id `hash` (`ms10hashsq…`), the id joins
   `RESERVED_ID_BLOCKLIST`, and **a single whose id and prefix byte disagree
   is refused** (`Error::TagKindMismatch`), never read as the other kind.
3. **Sweep every catch-all over `Payload`, `PayloadKind` and `InspectKind`** —
   `_ => <value>` arms as much as `_ => unreachable!` — because
   `#[non_exhaustive]` means the compiler will not. `InspectKind` is NOT
   `#[non_exhaustive]`, so adding `Preimage` is source-breaking for an
   exhaustive match: loud, therefore safe.
4. **The by-hand recipe this constellation documented before 0.18.0 — "hash
   the passphrase to 32 bytes, then hash again" — is `ms hashlock --method
   sha256`, NOT the default.** The default is the hardened method
   (PBKDF2-HMAC-SHA256, salt `ms-hashlock-v1`, 100,000 iterations). A digest
   made by hand reproduces only with `--method sha256`.
5. **A third reader shape exists and it is the dangerous one: "decode
   succeeded, therefore this is a seed."** Measured: `me`'s `validate_record`
   maps ANY `ms_codec::decode` success to a secret seed record; the
   SeedHammer fork's `isStrictMs1` has no prefix test at all. Neither
   dispatches on the prefix, so items 1 and 3 do not reach them. **Before
   this release ships, both are guarded (H0):** the fork's classifier treats
   `0x03` as inert and is flashed; `me`'s record validator treats it as inert
   in the same window as its ms-codec 0.8 bump.

Older `ms` (ms-codec 0.7) refuses a `0x03` single with
`reserved-prefix byte was 0x03` (exit 2) — a refusal, never a seed. The
downgrade row in `scripts/plan-build-gate-ms.sh` step 6 proves it against the
pre-0.8 tree.

New API: `ms_codec::hashlock::{HASHLOCK_SALT, HASHLOCK_ITERATIONS,
HASHLOCK_DKLEN, preimage_hardened, preimage_sha256, preimage_random, digest}`;
`Payload::Preimage(Zeroizing<[u8; 32]>)`; `PayloadKind::Preimage` and
`PayloadKind::single_tag`; `InspectKind::Preimage`; `Tag::HASH`;
`Error::{PreimageLengthMismatch, TagKindMismatch, RandomnessUnavailable}`.
Corpus: `crates/ms-codec/tests/vectors/hashlock-v0.8.json`, SHA-pinned in
the CHANGELOG.

## v0.8 → v0.9 (the hashlock PHRASE rule and its plate text move into `ms-codec`)

v0.9 is **purely additive** — no wire byte, derivation, `Payload`/`Tag`/
`InspectKind` variant, or existing-verb behaviour changes. It is forced to
`0.X+1.0` by `RELEASE_PROCESS.md` item 1, not by an API break: the corpus
`tests/vectors/hashlock-v0.8.json` gains seven `qr_text` rows, moving its
SHA-256 from `a46c197a3640fe8af4ca4370b46a9637466649227163ce6761bb032354811d30`
to `4f1819cdd0862b101afd48d0478e8f0b218f933dd3da449915fa3c5eaaba21d4`.

Five new public items, all in `ms_codec::hashlock`:

1. **`validate_phrase(bytes: &[u8]) -> Result<(), PhraseRefusal>`** and
   **`PhraseRefusal`** — the hashlock PHRASE admission rule, moved in from
   `ms-cli` (where it was a private predicate) so that every reader of a
   phrase — `ms-cli` and `me sysw pack`'s `phrase:` record
   (SPEC_hashlock_H6 §3.1) alike — applies the same rule byte for byte. `me`
   depends on this crate, not on `ms-cli`'s binary, so the rule could not stay
   private without a second implementation. `ms-cli` now delegates and keeps
   only its message rendering.
2. **`looks_like_ms1(raw: &str) -> bool`** — moved alongside `validate_phrase`
   for the same reason (a phrase that merely looks like an `ms1` string is
   refused, not treated as free text).
3. **`HASHLOCK_PHRASE_MAX_CHARS: usize = 100`** — the length ceiling
   `validate_phrase` enforces, exported so a caller can size a buffer or a
   plate layout against it rather than hard-coding `100`.
4. **`qr_text(hardened: bool, phrase: &str) -> Zeroizing<String>`** — the
   exact text a hashlock PHRASE plate carries (SPEC_hashlock_H6 §8.6): a
   `hashlock v1` line, a `method:` line rendered from `HASHLOCK_SALT`,
   `HASHLOCK_ITERATIONS` and `HASHLOCK_DKLEN` (never a literal, so a parameter
   change cannot leave an engraved plate lying about how to reproduce the
   derivation), and a `phrase:` line last, LF-separated, no trailing newline.
   The return is `Zeroizing` because the phrase is in it, and the buffer is
   `Zeroizing` from the first byte with capacity reserved up front — not a
   finished `String` wrapped at the end, which would protect only the copy.

One internal convergence ships in the same commit, not a behaviour change: the
moved rule's 64-hex check is now `b.is_ascii_hexdigit()` over the already-64-byte
window instead of `hex::decode(s).is_ok()` — the same predicate over the same
window, and it keeps the `hex` crate out of a codec that does not otherwise need
it.

**API + byte-identity.** Nothing is removed, renamed, or made
`#[non_exhaustive]`; no existing signature changed. `ms-cli`'s `ms-codec`
requirement moves from `=0.8.0` to `=0.9.0` in the same commit (the pin is
exact, so the bump is not optional).
