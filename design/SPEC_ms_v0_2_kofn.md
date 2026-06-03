# SPEC — ms v0.2 K-of-N codex32 shares

**Goal:** Add BIP-93 codex32 K-of-N Shamir share encoding to `ms1` (the format's defining capability; v0.1 hardcodes threshold=0 = unshared). A secret (`entr` **or** `mnem`) splits into N shares, any K of which recombine to the original — using codex32's *native* threshold+index mechanism, NOT a payload byte. v0.1/mnem single-strings stay byte-identical and forward-readable.

**Base SHA:** mnemonic-secret `master` `9d789b2` (ms-codec 0.3.0 / ms-cli 0.6.0, post mnem-v0.2). Toolkit `mnemonic-toolkit` `f95ddc4` (v0.39.0). Re-grep all line citations at impl time (CLAUDE.md citation-decay).
**SemVer:** ms-codec 0.3.0→**0.4.0**, ms-cli 0.6.0→**0.7.0**, mnemonic-toolkit v0.39.0→**v0.40.0** — all **MINOR** (additive; v0.1 wire preserved; a v0.1 decoder correctly rejects shares).
**Scope (user-locked 2026-06-02):** (1) **entr + mnem** both shareable (language survives the split). (2) ms-cli `ms split` / `ms combine`. (3) Toolkit `mnemonic ms-shares` with `split` + `combine` modes (mirrors `slip39`/`seed-xor`).
**Design substrate:** crypto verified vs codex32 0.1.0 source + runtime spikes in `design/agent-reports/ms-kofn-design-review.md` (the pre-SPEC architect review) + `ms-v0-2-spec-R0-review.md` (C2/I2/I3/M5). This SPEC folds design-review C1, C2, I1–I7, M1–M6.

---

## §1. Wire format — threshold-field dispatch (amends SPEC_ms_v0_1 §5 + MIGRATION.md)

K-of-N rides codex32's native Shamir. The **prefix byte stays the payload-KIND discriminator** (`0x00`=entr, `0x02`=mnem); **`0x01` stays UNALLOCATED** (the old §5 "0x01=entr-share" idea is crypto-invalid — dropped, design-review C2-of-prior-R0). Share-ness lives in the **codex32 header**, an orthogonal axis:

- A **share-set** shares one random 4-char `id` (BIP-93 recommended random-per-set semantics; NOT a type tag in v0.2-shares).
- Each share's codex32 header carries a **threshold char `k`** (`'2'..'9'`) and a **distinct share-index char** (bech32 charset minus `s`).
- The **secret-at-S** (index `s`, threshold `k`) holds the real payload (`[0x00][entropy]` entr / `[0x02][lang][entropy]` mnem). **It is NEVER distributed** — it is the recovery target only.
- A distributed share's payload is a Lagrange-interpolation output (opaque bytes; its `data()[0]` is NOT a stable prefix — verified 0xcf/0x68/0x5e/… in the design review). **Therefore dispatch MUST be on the threshold field, never on a share's payload byte.**

**Decode dispatch (the load-bearing rule):** read the threshold char first. `threshold == '0'` → existing single-string path (then `0x00`=entr / `0x02`=mnem via the prefix byte, unchanged). `threshold ∈ '2'..'9'` → this is one share of a K-of-N set; a single-string `decode` MUST NOT route it into `discriminate()` (its `data()[0]` is garbage → would yield a misleading `ReservedPrefixViolation`). See §2 `decode` + §3 `inspect`.

**Bounds (verified, design-review I4):** `2 ≤ k ≤ n ≤ 31`. There are exactly **31** valid non-`s` share indices (the 32-char bech32 alphabet `qpzry9x8gf2tvdw0s3jn54khce6mua7l` minus `s`); `n = 32` exhausts. `k = 1` and `k > 9` are invalid (codex32 `from_seed` accepts threshold 0 or 2..9 only).

**No length collision:** an entr-16 single-string (50 chars) and a 50-char share are disambiguated by the **threshold char on the wire**, not by length. Dispatch is unambiguous.

**Registry table (replaces SPEC_ms_v0_1 §5 closing-para table):** prefix-byte axis (payload kind) is **orthogonal** to the threshold axis (share vs single):
- prefix `0x00` = entr · `0x01` = **unallocated** · `0x02` = mnem · `0x03..0xFF` = unallocated (claim-via-PR).
- threshold `'0'` = single (unshared) · `'2'..'9'` = K-of-N share-set.
- A share of an entr secret recovers to a `0x00` payload; a share of a mnem secret recovers to a `0x02` payload. The prefix byte is meaningful only on the recovered secret-at-S.

---

## §2. ms-codec API (0.4.0)

### `Threshold` (new type; design-review M1)
```rust
pub struct Threshold(u8);
impl Threshold {
    pub const ZERO: Threshold;                 // unshared single-string; a const, NOT new(0)
    pub fn new(k: u8) -> Result<Threshold>;    // k ∈ 2..=9, else Error::InvalidThreshold(k)
    pub fn get(self) -> u8;
}
```

### `encode_shares` (design-review I3 — `(tag, threshold, n, &secret)`, NOT `payload_set`)
```rust
pub fn encode_shares(tag: Tag, threshold: Threshold, n: usize, secret: &Payload)
    -> Result<Vec<String>>;
```
- `threshold == ZERO`: `n` MUST be 1; returns `vec![single_string]` **byte-identical** to `encode(tag, secret)` (both reduce to `from_seed(HRP, 0, tag.as_str(), Fe::S, [prefix]||payload)`, deterministic — the Phase-0 gate). **The ZERO path keeps `id = tag` (NOT a random id)** — random `id` is ONLY for `k∈2..9` share-sets; this is load-bearing for byte-identity (R0-confirmed).
- `threshold == k ∈ 2..=9`: validate `k ≤ n ≤ 31` (else `Error::InvalidShareCount { k, n }`). Construction:
  1. secret-at-S = `from_seed(HRP, k, id, Fe::S, secret_wire_bytes(secret))` where `id` = a random 4-char codex32 id NOT in `RESERVED_ID_BLOCKLIST` (re-roll on collision; rate ≈ 5/32⁴).
  2. `k-1` random **defining shares** at distinct non-`s` indices, each `from_seed(HRP, k, id, Fe::<idx>, csprng_bytes)` with CSPRNG payload of the **same byte length** as the secret. **RNG = `getrandom` internally** (no injected-RNG param; `mk_codec::encode` precedent).
  3. for the remaining `n-(k-1)` distinct non-`s` indices: `interpolate_at(&[secret_at_S, defining…], Fe::<idx>)`.
  4. return the `n` **distributed** shares (`k-1` defining + `n-(k-1)` interpolated); the secret-at-S is NOT returned.
- Works for **entr and mnem** payloads identically (byte-agnostic; verified all 5 lengths). The wire bytes are `[0x00]||entropy` (entr) or `[0x02][lang]||entropy` (mnem). **(R0-m1 — NET-NEW:** `Payload` has no `wire_bytes()` method today; the `[prefix]||payload` assembly lives inline in `envelope::package()`. The plan MUST factor it into a reusable helper — a `pub(crate) fn payload_wire_bytes(&Payload) -> Zeroizing<Vec<u8>>` or a `Payload` method — and call it from BOTH `package()` and `encode_shares()`. The codec uses `Tag::as_str()` (tag.rs:56), NOT a `tag.id()`.)

### `combine_shares` (design-review C1 + I2 + I3)
```rust
pub fn combine_shares(shares: &[String]) -> Result<(Tag, Payload)>;
```
Pre-validation **before** `interpolate_at` (because `interpolate_at`'s `target==input-index` short-circuit at codex32 `lib.rs:259` bypasses its own checks):
1. parse each; all share the same hrp/id/threshold/length (else surface codex32 `Mismatched{Hrp,Id,Threshold,Length}`).
2. **count ≥ k** (the parsed threshold), else `Error::ThresholdNotPassed`.
3. **distinct indices** (design-review I3 — `interpolate_at`'s `RepeatedIndex` is lazy), else `Error::RepeatedIndex`.
4. **C1: reject any input share whose index is `s`** → `Error::SecretShareSuppliedToCombine` (the secret-at-S is never a combine input; codex32 does NOT reject a threshold∈2..9 / index-`s` string, and the short-circuit would silently "succeed").
Then `interpolate_at(shares, Fe::S)` → secret bytes → route through the existing `discriminate()` + `Payload::validate()` (NO re-implemented prefix/length checks) → `(Tag, Payload)` (entr or mnem).

### `decode` dispatch (design-review I1, I5)
`decode(&str)`: read the threshold char first. `'0'` → existing path (unchanged). `'2'..'9'` → `Error::IsShareNotSingleString { threshold, index }` whose `Display` directs the user to `ms combine` (do NOT route into `discriminate()`). This **relaxes** the v0.1 `Error::ThresholdNotZero` hard-reject into a route → re-spec the contract test (§8). `decode_with_correction` (BCH) on a share: a share IS a valid codeword (residue 0) → passes through to `decode` → `IsShareNotSingleString` (acceptable; spec'd).

### Errors (design-review M4)
New variants **alphabetical-inserted** (Display arms reordered to match): `InvalidShareCount`, `InvalidThreshold`, `IsShareNotSingleString`, `SecretShareSuppliedToCombine`. codex32 share errors (`MismatchedHrp/Id/Length/Threshold`, `RepeatedIndex`, `ThresholdNotPassed`) surface via the existing `Error::Codex32(inner)` wrap. Pre-existing v0.1 non-alphabetical variants are NOT retro-sorted (mirror toolkit's `error-rs-retroactive-alphabetical-sort` deferral).

### Consts
- `RESERVED_ID_BLOCKLIST = [b"entr", b"seed", b"xprv", b"mnem", b"prvk"]` — anti-collision for random `id` generation (design-review I4). **DISTINCT** from the decoder-reject `RESERVED_NOT_EMITTED_V01 = [seed, xprv, prvk]` (`consts.rs:62`; `mnem` was removed in Cycle 1 — but it MUST stay in the id-blocklist).
- `THRESHOLD_V01 = b'0'` / `SHARE_INDEX_V01 = b's'` (`consts.rs:20/23`) unchanged.

---

## §3. ms-cli (0.7.0)

- **`ms split [<source>] -k <K> -n <N> [--language <L>] [--json]`** — source = the same forms `ms encode` accepts (`--phrase`/`--hex`/positional/seedqr); a non-English `--phrase` produces a **mnem** secret (language preserved in every share's recovered secret). Emits N share strings (text: one per line + engraving grouping; `--json`: `{shares:[…], k, n, id, kind, language?}`). Validates `2≤k≤n≤31`.
- **`ms combine <share>... [--to phrase|entropy|ms1] [--json]`** — combines (default `--to phrase`), emitting the recovered secret in its on-wire language (mnem) or English-default (entr); surfaces the §2 error taxonomy via `codex32_friendly.rs` (already maps the codex32 share errors — `codex32_friendly.rs:31-77`).
- **`ms inspect <share>`** (design-review I1 + R0-m2) — a lone share is a **first-class read**: report `kind: share`, `threshold: k`, `id`, `index`, `would_combine: needs k shares` — NOT a `FAIL`/`would-not-decode`. **The site is the rule-walker `cmd/inspect.rs::analyze()` (~:49)** — currently a threshold∈2..9 string trips its prefix/length FAIL reasons; add a share branch BEFORE those rules (the threshold-field check). `InspectReport` already carries the needed fields.
- **Output class (design-review M2):** `ms split` emits the whole N-share **set** in one invocation → **`PrivateKeyMaterial`** (the set is secret-equivalent). `ms combine` (emits the secret) → `PrivateKeyMaterial`. Recovered/secret bytes wrapped in `Zeroizing` (design-review M3).

---

## §4. Toolkit `mnemonic ms-shares` (v0.40.0)

New subcommand mirroring `slip39`/`seed-xor` (split-OR-combine in one subcommand; user-locked C2 resolution):
- **`mnemonic ms-shares split [<source>] -k <K> -n <N> [--language] [--json]`** → N ms1 shares.
- **`mnemonic ms-shares combine <share>... [--to ms1|phrase|entropy] [--json]`** → recovered secret; composes into the existing pipeline (`combine --to ms1` → feed `bundle`/`addresses`/`convert`).
- Toolkit consume paths (`inspect`/`decode`/`repair` via `cmd/inspect.rs:171` `decode_card`) propagate `IsShareNotSingleString` via `?` → surfaced with a friendly map. Dispatch on the **threshold field** (never `0x01`). **(R0-m3 — the toolkit's `ms_codec_exit_code` (`_ => 1`) + `friendly_ms_codec` (`_ => "unhandled…"`) wildcards compile but MIS-surface the new ms-codec variants; P3 MUST add explicit arms (esp. `IsShareNotSingleString`, `SecretShareSuppliedToCombine`, the surfaced `Mismatched*`) — verify/extend `mnemonic-toolkit/src/friendly.rs`.)**
- Recovered secret → `Zeroizing` + `PrivateKeyMaterial` advisory (design-review M3).
- Re-pin: ms-codec 0.3.0→0.4.0 (crates.io lib), ms-cli v0.6.0→v0.7.0 (tag at `scripts/install.sh` + `.github/workflows/manual.yml`).

---

## §5. Migration amendment (design-review I6)

Rewrite in BOTH `MIGRATION.md` (repo root) and `SPEC_ms_v0_1.md §5`:
- **Invariant 1:** `0x01` → **unallocated** (NOT entr-share); the prefix byte is the payload-KIND discriminator; shares are keyed on the threshold field. Add the orthogonal-axes note (§1 registry table).
- **Invariant 2:** replace the prefix-byte grouping gate with the **threshold-field gate** (`'0'`→single; `'2'..'9'`→share, group by `id`); distributed shares have no stable prefix byte.
- **Invariant 3:** unchanged (anti-collision `id` blocklist; now via `RESERVED_ID_BLOCKLIST` incl `mnem`).
- **Invariant 4 (design-review I3):** signature changes to `encode_shares(tag, Threshold, n, &Payload)`; the byte-identity CLAIM survives (`encode_shares(tag, ZERO, 1, &p) ≡ encode(tag, &p)`) but the SIGNATURE does not (the recon/old-§5 `payload_set` form is wrong).
- `SPEC_ms_v0_1 §4` rules 3/4 (`threshold≠0`/`index≠s` hard-reject) → reframe reject→route. `§8` deferred-table "K-of-N for entr" → shipped this cycle, broaden to **entr+mnem**.
- `envelope.rs` source comments are ALREADY on the threshold/share-index design (`:3`, `:92`) — no comment fix owed.

---

## §6. Lockstep + SemVer

- **GUI `schema_mirror` (REAL this time):** `mnemonic-gui/src/schema/ms.rs` gains `split`/`combine` flag-name entries; `mnemonic-gui/src/schema/mnemonic.rs` gains `ms-shares` (+ its `split`/`combine` modes/flags). Gated (reflective `gui-schema` auto-emits; the hand-mirror must catch up). **Paired-PR discipline.**
- **`--json` wire-shapes (NOT gated, design-review I7):** `ms split --json` (shares set), `ms combine --json`, `ms inspect --json` (share fields), and the toolkit equivalents — self-update GUI/downstream consumers via the paired-PR rule.
- **Manual:** `docs/manual/src/40-cli-reference/43-ms.md` (`ms split`/`combine`) + the toolkit chapter (`mnemonic ms-shares`).
- **SemVer:** all MINOR (above). NOT breaking: v0.1/mnem single-strings decode unchanged; shares are a new wire shape; the decoder relaxes a reject into a route.

---

## §7. Phasing (design-review recommended; P3 severable)

- **P0 — spike (the hard gate; throwaway-or-kept):** prove (a) `encode_shares(tag, ZERO, 1, &p)` byte-identical to `encode` against the SHA-pinned v0.1 vectors; (b) a K-of-N round-trip for entr AND mnem, all 5 lengths, k∈2..9, against codex32 `bip_vector_3`; (c) **C1**: `combine_shares` rejects an index-`s` input. If any fails, STOP — the design is wrong.
- **P1 — ms-codec:** `Threshold`, `encode_shares`, `combine_shares` (full C1/I2/I3 pre-validation), `decode` threshold-routing (I1/I5), new errors (M4), `RESERVED_ID_BLOCKLIST` (I4), §5/MIGRATION amendment (§5 here). Per-phase opus R0.
- **P2 — ms-cli:** `ms split` / `ms combine` / `ms inspect`-of-share; output-class; `codex32_friendly` reuse. Version bump 0.7.0.
- **P3 — toolkit (SEVERABLE — last):** `mnemonic ms-shares` split+combine; threshold-dispatch in consume paths; re-pin; v0.40.0. If the toolkit surface stalls, P1+P2 ship standalone and P3 defers.
- **P4 — docs/GUI lockstep:** manual + `schema_mirror` (ms.rs + mnemonic.rs) + the broken-test re-spec.
- **Gate:** ms-codec has NO CI (`ms-codec-no-ci-workflow` open) → **local-only**: full `cargo test -p ms-codec && -p ms-cli`, `clippy --all-targets -D warnings`, at every phase commit. NO fmt gate (repo not `+stable`-clean; write fmt-clean by hand). Mandatory opus R0 on this SPEC + the plan + each phase + end-of-cycle (0C/0I before code; re-dispatch after every fold).

---

## §8. Tests

- **Byte-identity:** `encode_shares(ENTR, ZERO, 1, &p)` == the SHA-pinned v0.1 vector strings (every entr length); same for mnem ZERO vs the Cycle-1 mnem vectors.
- **K-of-N round-trip:** for entr AND mnem, all 5 lengths, every (k,n) with k∈2..9, n∈k..=min(k+3,31): split → take a random k-subset → combine → recovers the exact secret (entr `Payload::Entr` / mnem `Payload::Mnem{language}` with language preserved). Cross-check the secret-at-S derivation against codex32 `bip_vector_3`.
- **combine validation:** `<k` shares → `ThresholdNotPassed`; duplicate index → `RepeatedIndex`; **index-`s` input → `SecretShareSuppliedToCombine` (C1)**; mismatched id/threshold/length/hrp → the respective `Mismatched*`.
- **Bounds:** `k=1`/`k=10` → `InvalidThreshold`; `n<k`/`n=32` → `InvalidShareCount`. `id` colliding with `RESERVED_ID_BLOCKLIST` re-rolls (deterministic-seed test via a stubbed sequence if feasible, else statistical).
- **decode/inspect:** lone share `decode` → `IsShareNotSingleString` (NOT `ThresholdNotZero`/`ReservedPrefixViolation`); `inspect` of a share reports threshold/id/index. **Re-spec `crates/ms-cli/tests/decode_rejects_threshold_not_zero.rs`** (+ grep all tests for the old reject — shared-gate sweep lesson).
- **Toolkit:** `mnemonic ms-shares split` → `combine` round-trip; `combine --to ms1` → `bundle` end-to-end (entr AND mnem); threshold-dispatch in inspect/decode.
- **Output-class / zeroize:** `ms split`/`combine` emit the `PrivateKeyMaterial` advisory; recovered bytes are `Zeroizing` (extend the zeroize-discipline lint anchor).

---

## §9. Footguns / R0-anticipated

1. **C1 short-circuit** — `interpolate_at`'s `target==input-index` early return bypasses validation; `combine_shares` pre-rejects index-`s` (the most subtle, security-relevant rule).
2. **encode_shares signature** — `(tag, Threshold, n, &Payload)`, not the stale `payload_set`; ZERO byte-identity must hold; MIGRATION/§5 inv 4 amended.
3. **threshold-field dispatch, never the payload byte** — distributed shares have garbage `data()[0]`.
4. **`mnem` in the id-blocklist but NOT the decoder-reject set** — two distinct const tables.
5. **the relaxed threshold-0 reject breaks a CONTRACT test** — full shared-gate sweep.
6. **n ≤ 31** index exhaustion; **k ∈ 2..=9** (no 1).
7. **`Threshold::ZERO` is a const**, not `new(0)`.
8. **no public `derive_share`** (M5) — `encode_shares` derives all N internally (avoids re-stamp + secret-S-derivation footguns).
9. **local-only verification** (no CI); **fmt by hand** (repo not clean).
10. **GUI/--json wire-shape ungated** — paired-PR discipline.

---

## §10. Citations (re-grep at impl time; base `9d789b2` / codex32 0.1.0)

- codex32 0.1.0 (`~/.cargo/registry/src/*/codex32-0.1.0/src/lib.rs`): `from_seed` threshold map + bit-packing 312-380; `interpolate_at` 217-308 (count 230, eager mismatch checks 238-252, **target short-circuit 259-263**, lazy `RepeatedIndex` 283-285); `parts_inner` 177-206 (threshold-0⇒S guard 202-204); `Parts::data()` 399-428; `bip_vector_3` 489-516. `field.rs`: `Fe::S`=Fe(16).
- ms-codec `9d789b2`: `consts.rs` (THRESHOLD_V01 20, SHARE_INDEX_V01 23, RESERVED_NOT_EMITTED_V01 62, VALID_MNEM_STR_LENGTHS 43); `envelope.rs` (discriminate + threshold/share-index header check 105-112, `package`/`from_seed` 187, byte-aligned mnem 175-182); `decode.rs` (length-gate union, threshold reject to relax); `error.rs` (`Error::ThresholdNotZero` 20, `Codex32` wrap 11/146-150, non-alphabetical order); `payload.rs` (Payload Entr/Mnem, validate, `as_bytes`).
- ms-cli `9d789b2`: `main.rs` `enum Command` at :69 (8 subcommands through :127 — Derive/Encode/Decode/Inspect/Verify/Vectors/Repair + GuiSchema); `codex32_friendly.rs` arms ~:11-57 (codex32 share-error friendly map); `advisory.rs` (OutputClass). (R0-m4 citation correction.)
- toolkit `f95ddc4`: `main.rs:110-119` (seed-xor + slip39 split-or-combine pattern); `cmd/inspect.rs:171` (`decode_card`); `friendly.rs`; `scripts/install.sh` + `.github/workflows/manual.yml` (ms-cli tag pin); `src/schema` (GUI mirror — actually `mnemonic-gui/src/schema/{ms.rs,mnemonic.rs}`).
- Migration: `MIGRATION.md` (repo **root**, four invariants — `../MIGRATION.md` from `design/`) + `SPEC_ms_v0_1.md §5` (212-226), §4 rules, §8 table.
