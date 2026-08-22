# SPEC — Cycle-B: vendor (inline) codex32 into ms-codec + scrub the share spine

**Status:** DRAFT — pending mandatory R0 architect gate (0C/0I before any code).
**Author (single):** Cycle-B SPEC author.
**Cross-repo:** `mnemonic-secret` (ms-codec + ms-cli) + `mnemonic-toolkit` (paired, non-optional).
**Grounding recon:** `mnemonic-secret/cycle-prep-recon-codex32-vendor-fork-cluster.md` (recon SHA `6e3ee8e`).
**Source SHAs verified live for this SPEC:**
- `mnemonic-secret` `origin/master` @ **`6e3ee8e`** (HEAD, up-to-date).
- `mnemonic-toolkit` `origin/master` @ **`684e510`** (HEAD).
- Vendored crate: `codex32-0.1.0` (`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/codex32-0.1.0`), checksum `d230935faa4d0521349d228f39aba4ff489cf2a8bcab4d84e31f4cbd6fe918e9`, license CC0-1.0.

Slugs closed by this cycle (all in `mnemonic-secret/design/FOLLOWUPS.md`):
`rust-codex32-zeroize-upstream`, `codex32-upstream-dormant-vendor-vs-accept-decision`,
`ms-codec-share-strings-not-zeroized-encode-and-combine`; companions
`rust-codex32-upstream-pr2-recovery-bug-not-exposed` (anchor re-point) and the
`[obs] recovered-secret-string-not-zeroized` line (FOLLOWUPS.md:16).

---

## 0. Decision: INLINE (shape A) — private `pub mod codex32` inside ms-codec

**RESOLVED: (A) inline.** Vendor the three runtime modules of `codex32-0.1.0` as a
crate-owned module tree under `crates/ms-codec/src/codex32/` and **drop the external
`codex32 = "=0.1.0"` workspace dependency**. ms-codec re-exports the inlined surface
as `ms_codec::codex32::{Codex32String, Fe, Error, Parts, ChecksumEngine}` so consumers
migrate by a single mechanical path rewrite `codex32::` → `ms_codec::codex32::`.

### Why A, not B (a new published `ms-codex32` crate)

The recon framed this as genuinely open and leaned (b). Live verification overturns
that lean for THIS constellation:

1. **codex32 is consumed only through ms-codec's domain.** Every direct `codex32::`
   consumer (ms-cli, toolkit) reaches it *because of* ms1 — there is no independent
   reuse story. A separate crates.io crate would be a perpetual publish/own/version
   obligation with exactly one logical consumer (ms-codec) and two transitive ones
   that already depend on ms-codec.
2. **The forced consumer migration is IDENTICAL under A and B.** The bare `codex32::`
   *extern-crate path* cannot survive either shape — under B it becomes
   `ms_codex32::`, under A it becomes `ms_codec::codex32::`. Every `use codex32::…`
   and `codex32::Error::…` / `codex32::Fe::…` / `codex32::Codex32String::…` site
   rewrites either way (enumerated in §5). So B's "clean dependency boundary" buys
   **nothing** on the migration axis — the blast radius is the same.
3. **A removes a dormant external dep outright** (the stated user intent: "de-risk the
   dormant dep / own the BCH-Shamir primitives"). B *replaces* a dormant external dep
   with a self-owned external dep — strictly more surface (a 4th crates.io publish in
   the constellation, a new crate to security-own, a new lockstep version site).
4. **The `Cargo.lock`/registry-publish constraint (recon cross-cut #1) is satisfied
   trivially by A.** A published ms-codec with an *inlined module* has no path/git
   dep; B is the shape that *introduces* a new publish to satisfy that constraint.
5. **The wire-byte-identity guarantee is equally strong under A.** Vendoring is a
   byte-for-byte copy of `lib.rs`/`field.rs`/`checksum.rs` regardless of whether the
   copy lands in a sub-module or a sibling crate.

**Decisive reason A is preferred:** A is strictly less surface than B (no new
published crate) for an identical migration cost, and it directly executes the
"remove the dormant dep" intent rather than re-creating it under a new name. The only
property B has that A lacks — independent reuse of the codex32 primitives by a
non-ms-codec consumer — does not exist in this constellation and is not a goal.

**R0 scrutiny hooks (explicitly flagged for the architect):**
- Confirm A's `pub mod codex32` re-export does not over-expose internals: the inlined
  `Parts`, `ChecksumEngine`, `Case` were `pub` upstream; we keep the *same* public
  surface (no widening, no narrowing) so behavior is identical. (See §1.3.)
- Confirm A does not create a license-attribution gap: CC0 LICENSE file is vendored
  verbatim with an attribution header (§1.2). CC0 is public-domain — no copyleft, no
  notice-retention legal requirement, but we retain attribution as courtesy + audit
  trail.
- Confirm the toolkit's *direct* `codex32 = "=0.1.0"` dep can be **dropped entirely**
  (it only existed to name `codex32::Error`/`Fe` for `friendly.rs`); post-A the
  toolkit names them via `ms_codec::codex32::` and carries no codex32 dep. (See §5.2.)

---

## 1. Phase 1 — vendor the three modules BYTE-IDENTICAL + attribution + re-export

### 1.1 Files added (ms-codec)
- `crates/ms-codec/src/codex32/mod.rs` ← byte-for-byte copy of upstream `lib.rs`
  **runtime body only** (lines 1–429: the module doc, `Error`, `Case`,
  `Codex32String` impls, `Parts`). The upstream `#[cfg(test)] mod tests` (lib.rs
  431–704) is NOT copied (its BIP vectors are already pinned by ms-codec's own
  `tests/bip93_inline_vectors.rs` + `bip93_cross_format.rs`; copying them would
  duplicate, not strengthen). The two `mod checksum; mod field;` declarations
  (lib.rs:34–35) and `pub use checksum::Engine as ChecksumEngine; pub use field::Fe;`
  (lib.rs:38–39) are preserved verbatim.
- `crates/ms-codec/src/codex32/field.rs` ← byte-for-byte copy of upstream `field.rs`
  runtime body (lines 1–263); upstream `#[cfg(test)] mod tests` (264–319) NOT copied.
- `crates/ms-codec/src/codex32/checksum.rs` ← byte-for-byte copy of upstream
  `checksum.rs` (lines 1–191, no test module upstream).
- `crates/ms-codec/src/codex32/LICENSE` ← verbatim copy of upstream CC0 LICENSE
  (7049 bytes).
- The dev-only `src/bin/correction-table.rs` (151 LOC, error-correction-table
  generator) is **NOT vendored** (recon-confirmed: not referenced by ms-codec/ms-cli).

**ENCODING-INVARIANT (load-bearing, BIP-93 / codex32):** the vendored bodies of
`from_seed` (`mod.rs` ≈ lib.rs:312–380, base32 packing 343–361), `from_string`
(148–174), `interpolate_at` (217–309), `Parts::data` (399–428), the entire
`checksum.rs` BCH engine (generator/residue/target polynomials, `input_fe`
175–190), and the entire `field.rs` GF(32) tables + arithmetic are copied with
**ZERO edits**. Phase 1 touches encoding NOWHERE.

### 1.2 Attribution header
Prepend to each of `mod.rs`, `field.rs`, `checksum.rs` (above the existing upstream
2023 Andrew Poelstra CC0 header, which is RETAINED verbatim):
```
// Vendored from `codex32` v0.1.0 (crates.io checksum d230935f…918e9), CC0-1.0,
// by Andrew Poelstra. Inlined into ms-codec at <SHA> to own the Zeroize/Drop/Debug
// secret-hygiene fixes (FOLLOWUP codex32-upstream-dormant-vendor-vs-accept-decision).
// Runtime modules copied byte-identical; ONLY Zeroize/ZeroizeOnDrop/redacting-Debug
// added (Phase 2). Encoding (from_seed/from_string/interpolate_at/checksum/field)
// is UNCHANGED. Upstream LICENSE retained alongside as src/codex32/LICENSE.
```
Cite the doc'd source SHA (this SPEC's `6e3ee8e`) and crates.io checksum in the
top-of-tree note.

### 1.3 Re-export wiring (ms-codec lib.rs)
Add to `crates/ms-codec/src/lib.rs` (currently the `pub mod`/`mod` block at 40–51):
```rust
pub mod codex32;   // vendored BIP-93 codex32 (CC0, inlined; see src/codex32/)
```
Inside `src/codex32/mod.rs`, keep upstream's public surface verbatim:
`pub enum Error`, `pub enum Case`, `pub struct Codex32String`, `pub struct Parts`,
`pub use checksum::Engine as ChecksumEngine`, `pub use field::Fe`. The submodules
`checksum`/`field` stay private to the codex32 module (upstream had them `mod`,
not `pub mod`) — public access is the curated re-export set only. Net public
surface of `ms_codec::codex32` == upstream `codex32` crate's public surface
(verified equal; no widening).

### 1.4 Drop the external dep
- `Cargo.toml` (workspace): delete `codex32 = "=0.1.0"` from `[workspace.dependencies]`.
- `crates/ms-codec/Cargo.toml`: delete `codex32 = { workspace = true }` from
  `[dependencies]`.
- `crates/ms-cli/Cargo.toml`: delete `codex32 = { workspace = true }` from
  `[dependencies]` (ms-cli reaches codex32 via `ms_codec::codex32::` post-migration).

### 1.5 Phase-1 tests (TDD: parity gate FIRST)
- **NEW `tests/codex32_vendor_parity.rs` (RED-first, written before the vendor copy
  exists):** asserts the inlined `ms_codec::codex32` produces BYTE-IDENTICAL output
  to the recorded upstream behavior for a fixed corpus:
  - `from_seed("ms", 0, "leet", Fe::S, &seed_b)` == the BIP-93 §4 long-seed string
    (the exact upstream `bip_vector_4` expected string, hard-coded as a literal — NOT
    re-derived from the inlined code, so it pins to the BIP, not to itself).
  - `from_seed` for all five entr lengths (16/20/24/28/32) → a recorded golden set of
    output strings (captured ONCE from the pre-vendor `codex32 =0.1.0` build and
    pasted as literals — the "fixed seed → identical string pre/post-vendor" KAT the
    recon mandates).
  - `from_string` round-trips BIP-93 §1/§2/§3/§5 vectors and `interpolate_at`
    reproduces §2/§3 shares verbatim (the upstream `bip_vector_2/3` expected share
    strings as literals).
- **Existing suites that MUST stay GREEN against the vendored module (no edits in P1
  except the import-path rewrite of §5.1):** `tests/bip93_inline_vectors.rs` (5 valid
  + 64 invalid), `tests/bip93_cross_format.rs`, `tests/spike_kofn.rs` (claims a/b/c),
  `tests/codex32_upstream_recovery_regression.rs` (PR#2 secret), `tests/negative.rs`,
  `tests/uppercase_envelope.rs`, `tests/mnem_byte_aligned_lengths.rs`,
  `tests/forward_compat.rs`.
- Full `cargo test -p ms-codec` (NOT targeted — per MEMORY
  `feedback_r0_review_run_full_package_suite`).

### 1.6 Per-phase R0 + TDD
Per-phase TDD: parity test written RED before the copy. Per-phase opus R0 review →
persist verbatim to `mnemonic-secret/design/agent-reports/cycleB-phase-1-<round>-review.md`
→ fold → re-dispatch until 0C/0I before advancing. R0 focus: byte-identity of the copy
(diff the vendored bodies against the registry source), no accidental encoding edit, no
public-surface drift, attribution correctness.

---

## 2. Phase 2 — Zeroize + ZeroizeOnDrop + redacting Debug on `Codex32String`

The ONLY behavioral change to the vendored code. Touches `src/codex32/mod.rs` ONLY,
and ONLY the `Codex32String` type's derive list + new trait impls — NEVER
`from_seed`/`from_string`/`interpolate_at`/`Parts`/`checksum.rs`/`field.rs`.

### 2.1 Derive change (mod.rs ≈ lib.rs:101–102)
Upstream:
```rust
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Codex32String(String);
```
After:
```rust
#[derive(Clone, PartialEq, Eq, Hash, zeroize::ZeroizeOnDrop)]
pub struct Codex32String(String);
```
- **`Debug` is REMOVED from the derive** and hand-implemented as length-only redaction
  (§2.2). Upstream's derived `Debug` printed the full secret string — the L22-class
  footgun the user flagged.
- **`Clone`, `PartialEq`, `Eq`, `Hash` are RETAINED.** Load-bearing:
  - `Clone` — `interpolate_at`'s index-`s` short-circuit returns `shares[i].clone()`
    (mod.rs ≈ lib.rs:262); `combine_shares` does `from_string(s.clone())`
    (shares.rs:208). A cloned `Codex32String` is independently `ZeroizeOnDrop`
    (each owns its `String`; drop scrubs each) — Clone + ZeroizeOnDrop compose
    correctly.
  - `PartialEq`/`Eq` — the M6 polynomial-consistency check `derived != parsed[j]`
    (shares.rs:304). Constant-time is NOT required here (both operands are
    attacker-irrelevant: the user's own shares being recombined locally), so the
    derived `String` `==` is acceptable; do not introduce a CT compare (out of scope,
    would change semantics).
  - `Hash` — not reachable on a secret path today (no `HashSet<Codex32String>`
    verified), but cheap to retain and upstream-faithful; retaining avoids any future
    consumer breakage. (R0 may down-rule to drop it; default = retain for
    source-compat.)
- **`zeroize` is already a non-dev dependency of ms-codec** (`Cargo.toml:18
  zeroize = "1.8"`) — `ZeroizeOnDrop` derive needs the `derive` feature; ms-codec
  currently depends on `zeroize = "1.8"` WITHOUT `features = ["derive"]`. **Add
  `features = ["derive"]`** (the toolkit already does this:
  `zeroize = { version = "1.8", features = ["derive"] }`). This is a Cargo.toml
  edit in P2, flagged as a version-site touch.

### 2.2 Redacting Debug (new impl in mod.rs, next to the `Display` impl ≈ lib.rs:104)
```rust
impl fmt::Debug for Codex32String {
    /// Redacting: NEVER echoes the secret string (the derived Debug leaked it).
    /// Length-only — enough to debug a length/shape bug, nothing of the payload.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Codex32String([REDACTED; {} chars])", self.0.chars().count())
    }
}
```
Mirrors ms-codec's existing `Error`/`InspectReport` redacting-Debug discipline
(RULE Z-DEBUG). The char-count is non-sensitive (ms1 lengths are a small public set).

### 2.3 Zeroize semantics note
`ZeroizeOnDrop` derive on a single-field tuple struct over `String` zeroizes the
`String`'s heap buffer on drop (`String: Zeroize` via `zeroize`'s impl). This scrubs
the encoded secret that previously lived un-scrubbed for the value's lifetime
(closing `rust-codex32-zeroize-upstream`). `String`'s `Zeroize` zeroes the bytes and
sets len 0; the capacity allocation is freed on drop as usual — the bytes are zeroed
before free. (If R0 wants belt-and-suspenders, a manual `impl Zeroize` +
`impl Drop` is equivalent; the derive is the idiomatic minimal form — default to the
derive, R0 may escalate.)

### 2.4 Phase-2 tests (TDD)
- **NEW RED-first cell in `tests/` (e.g. `codex32_zeroize_debug.rs`):**
  `format!("{:?}", Codex32String::from_string(valid).unwrap())` MUST NOT contain any
  8-char window of the secret data-part (reuse the `contains_window`/`WINDOW=8`
  oracle pattern from `error.rs` no-echo tests); MUST contain `[REDACTED`.
- **Encoding-parity REGRESSION (must stay GREEN after the derive change):** re-run the
  full Phase-1 parity gate (`codex32_vendor_parity.rs`) + the BIP-93 suites — proves
  the Zeroize/Debug change did not perturb a single output byte.
- Optional: a drop-scrub assertion is hard to make deterministic in safe Rust
  (post-drop memory is UB to read); do NOT attempt a read-after-drop test. The
  ZeroizeOnDrop derive + the no-echo Debug test are the enforceable surface.
- Full `cargo test -p ms-codec`.

### 2.5 Per-phase R0 + TDD
RED-first Debug-redaction test before the impl. R0 → persist
`cycleB-phase-2-<round>-review.md` → fold → re-dispatch to 0C/0I. R0 focus: derive
list correctness (no lost trait a consumer needs; no leaky Debug path), zeroize
feature wiring, encoding untouched.

---

## 3. Phase 3 — ms-codec rewire (import paths + error surface)

### 3.1 Error surface decision: SOURCE-COMPATIBLE move (no public-API name change)
`ms_codec::Error::Codex32(_)`'s **inner type is `ms_codec::codex32::Error`** post-A
(was the extern `codex32::Error`). Both `ms_codec::Error::Codex32` *the variant* and
its field *shape* (a single positional `codex32::Error`) are UNCHANGED in name — only
the inner type's *crate path* moves from `codex32::Error` to `ms_codec::codex32::Error`.

- **For ms-codec's own `error.rs`:** the variant declaration
  `Codex32(codex32::Error)` (error.rs:21), the `From<codex32::Error>` impl
  (error.rs:260–264), the `Display` match arms peeling `codex32::Error::InvalidChecksum`
  /`MismatchedHrp`/`MismatchedId` (error.rs:151–164), and the `no_echo_tests`
  (error.rs:342–397) all rewrite `codex32::` → `crate::codex32::` (in-crate) or
  `codex32::` stays valid IF we add a crate-level `use crate::codex32;` shim. **Chosen
  form: rewrite to `crate::codex32::` explicitly** (no shim — explicit path, no
  ambiguity). The `From` impl name `From<crate::codex32::Error> for Error` is preserved.

- **Public-API SemVer characterization (load-bearing for the architect):** Is moving
  the inner type of a `pub` enum variant from `codex32::Error` to
  `ms_codec::codex32::Error` a breaking change? **YES, technically** — a downstream
  matcher that wrote `ms_codec::Error::Codex32(codex32::Error::Foo)` (naming the OLD
  extern crate) no longer compiles, because the extern `codex32` crate is gone from
  their resolved graph (ms-codec no longer pulls it; the downstream may still have its
  OWN `codex32` dep, in which case the TYPES differ and the match is a type error).
  The toolkit IS exactly such a matcher (friendly.rs, 15 sites) → **forced paired
  toolkit edit (Phase 5).** This is why ms-codec is a **MINOR (pre-1.0 breaking)**
  bump (§7).

### 3.2 ms-codec source import-path rewrites (Phase 3 scope)
Mechanical `codex32::` → `crate::codex32::` / `use crate::codex32::…` across the 5
ms-codec src files that name codex32 directly:
- `src/inspect.rs` (`use codex32::Codex32String;` → `use crate::codex32::Codex32String;`
  + `codex32::Fe::S`, `codex32::Error` doc refs).
- `src/envelope.rs` (`use codex32::{Codex32String, Fe};` + `codex32::Error::*` test
  refs at 390/410).
- `src/decode.rs` (`use codex32::Codex32String;` + `codex32::Fe::S`, `codex32::Error`
  refs).
- `src/error.rs` (per §3.1).
- `src/shares.rs` (`use codex32::{Codex32String, Fe};` at line 18 →
  `use crate::codex32::{Codex32String, Fe};`; plus the inline `codex32::Error::*`
  constructions at shares.rs:230, 261, 273, 538, 588, 596 etc.).

### 3.3 ms-codec test import-path rewrites
The 7 ms-codec test files naming `codex32::` (`bip93_cross_format.rs`,
`forward_compat.rs`, `bip93_inline_vectors.rs`, `mnem_byte_aligned_lengths.rs`,
`uppercase_envelope.rs`, `spike_kofn.rs`, `negative.rs`) rewrite their
`use codex32::…` / `codex32::…` to `use ms_codec::codex32::…` / `ms_codec::codex32::…`
(integration tests are external to the crate → they use the *public* re-export path
`ms_codec::codex32::`, NOT `crate::`).

### 3.4 Phase-3 tests
No NEW behavioral tests — Phase 3 is a path migration. The acceptance gate is: the
ENTIRE existing `cargo test -p ms-codec` suite compiles + passes unchanged in
behavior after the rewrite (every codex32-naming test now resolves via
`ms_codec::codex32::`). The error-surface no-echo tests (error.rs) prove the
sanitized Display/Debug still intercept the (now `crate::codex32::Error`) leaky
variants.

### 3.5 Per-phase R0 + TDD
R0 → persist `cycleB-phase-3-<round>-review.md` → 0C/0I. Focus: completeness of the
path rewrite (no stray `codex32::` extern reference left that would fail to resolve),
error-surface name preserved, no `Display`/`Debug` leak regression.

---

## 4. Phase 4 — share-string scrubbing + lint floor bump

### 4.1 Wrap the still-bare `String`-backed share legs (the slug's acceptance criterion)
Now that `Codex32String: ZeroizeOnDrop` (P2), the bare share-spine bindings in
`shares.rs` are auto-scrubbed at drop. The remaining EXPLICIT work:

- **`secret_s: Codex32String`** (shares.rs:141) — now ZeroizeOnDrop; no `Zeroizing`
  wrapper needed (it owns its scrub). Update the lifetime-min comment block
  (shares.rs:129–139) to record that the `String` leg is now drop-scrubbed (the
  cycle-15 "Q2 HOLD / blocked-on-cluster" caveat is RESOLVED).
- **`defining: Vec<Codex32String>`** (shares.rs:147), **`parsed: Vec<Codex32String>`**
  (shares.rs:206, 221), **the recovered `secret: Codex32String`** (shares.rs:291) —
  each element is ZeroizeOnDrop; the `Vec` drops each element (scrubbing) at fn return.
  No wrapper needed. Update the comment at shares.rs:312–316 (the
  "String-backed, no Drop" note) to reflect drop-scrub now covers them.
- **`distributed: Vec<String>`** (shares.rs:159) and the per-share `.to_string()`
  copies (shares.rs:161, 165) — these are **plain `String`** (the wire output handed
  back to the caller), NOT `Codex32String`. They are the function's RETURN VALUE
  (`Vec<String>`) so they MUST outlive the function — cannot be `Zeroizing`-wrapped
  without changing the public return type. **Decision: leave `distributed: Vec<String>`
  as the returned secret material; document that the CALLER owns scrub** (mirrors the
  `Payload::Entr(Vec<u8>)` caller-wrap contract already documented in payload.rs and
  enforced by the lint). The INTERMEDIATE `.to_string()` of a `Codex32String` into
  `distributed` is unavoidable (the wire form is `String`); the *source*
  `Codex32String` is drop-scrubbed. This is the irreducible residue and must be
  documented as such (NOT papered over).
- **`fields: Vec<(u8, u8)>`** (shares.rs:241) — `(threshold_byte, share_index_byte)`,
  NOT secret (header metadata). No change.

**Wire-byte-identity:** Phase 4 adds NO new allocation or transform on the encoding
path — it is comment updates + (verified-unnecessary) wrapper review. `encode_shares`
/`combine_shares` outputs stay byte-identical (the spike_kofn + combine round-trip
suites prove it).

### 4.2 lint floor bump (`tests/lint_zeroize_discipline.rs`) — HARD TRIPWIRE
The lint asserts `ZEROIZE_ROWS.len() == 4` EXACTLY (lines 79–85; the
`canonical_list_has_expected_row_count` test). Adding the codex32-scrub anchor row
bumps it to **5** — the assert literal AND its message MUST update in the SAME edit
(the "un-union → RED" tripwire from the Group A R0 ruling in MEMORY).

Add a 5th `ZeroizeRow`:
```rust
ZeroizeRow {
    label: "codex32::Codex32String scrubs its inner String on drop (vendored)",
    source_file: "src/codex32/mod.rs",
    evidence: &["zeroize::ZeroizeOnDrop", "impl fmt::Debug for Codex32String"],
},
```
and change `assert_eq!(n, 4, …)` → `assert_eq!(n, 5, …)` with an updated message
(append: "+ 1 vendored-codex32 Codex32String drop-scrub row, Cycle-B").

### 4.3 Phase-4 tests
- The bumped `canonical_list_has_expected_row_count` (now `== 5`) +
  `every_canonical_zeroize_row_has_evidence_anchor` (the new row's evidence anchors
  must resolve against `src/codex32/mod.rs`) — RED-first: write the `== 5` assert and
  the new row BEFORE P2's Debug/ZeroizeOnDrop impls land in mod.rs (so the evidence
  anchor is initially absent → RED → GREEN once P2 is in). Since P2 precedes P4, in
  practice the anchors already exist; the RED-first discipline is on the row+count
  edit (count was 4 → assert was satisfied at 4 → bumping to 5 is RED until the row is
  added).
- `tests/spike_kofn.rs`, `shares.rs` unit tests
  (`combine_round_trip_entr_and_mnem_all_lengths`,
  `combine_valid_exactly_k_unchanged`, `combine_valid_n_gt_k_all_consistent`,
  `combine_inconsistent_*`) all GREEN — proves scrub changes did not perturb combine
  semantics.
- Full `cargo test -p ms-codec`.

### 4.4 Per-phase R0 + TDD
R0 → persist `cycleB-phase-4-<round>-review.md` → 0C/0I. Focus: the irreducible
`distributed: Vec<String>` residue is HONESTLY documented (no false GREEN — same
discipline the slug status text already models), lint floor + row correct, no encoding
perturbation.

---

## 5. Phase 5 — ms-cli + toolkit coordination (the forced paired change)

### 5.1 ms-cli (same repo)
- `crates/ms-cli/Cargo.toml`: drop `codex32 = { workspace = true }` (done in P1 §1.4;
  re-confirm); bump `ms-codec = { path = "../ms-codec", version = "=0.6.0" }` →
  `version = "=0.7.0"` (exact pin — load-bearing version site).
- `crates/ms-cli/Cargo.toml`: bump `version = "0.11.0"` → `"0.12.0"` (§7).
- `src/codex32_friendly.rs`: `use codex32::Error;` → `use ms_codec::codex32::Error;`
  (the whole `friendly_codex32` matcher resolves via the re-export).
- `src/error.rs`: `Codex32(codex32::Error)` field + `codex32::Error::*` test refs →
  `ms_codec::codex32::Error`.
- The 11 ms-cli test files naming `codex32::` (`cli_combine.rs`, `inspect_share.rs`,
  `decode_rejects_unknown_tag.rs`, `json_error_envelope_per_kind.rs`,
  `verify_future_format.rs`, `inspect_reserved_tag.rs`, `verify_quiet_fail.rs`,
  `inspect_non_zero_prefix.rs`, `decode_routes_share_to_is_share_not_single_string.rs`,
  `verify_mnem_non_english.rs`, `inspect_multiple_failures.rs`): rewrite
  `use codex32::…` / `codex32::…` → `ms_codec::codex32::…`.
- The stale doc-comment in `codex32_friendly.rs:4` referencing
  `/tmp/codex32-extract/codex32-0.1.0/src/lib.rs:42-83` → re-point to
  `ms-codec/src/codex32/mod.rs` (the vendored source of truth).

### 5.2 toolkit (paired repo `mnemonic-toolkit`, NON-OPTIONAL)
- `crates/mnemonic-toolkit/Cargo.toml`:
  - **DROP `codex32 = "=0.1.0"`** entirely (lines 30–34, including the explanatory
    comment) — post-A the toolkit names codex32 types via `ms_codec::codex32::`.
  - Bump `ms-codec = "0.6"` → `ms-codec = "0.7"`.
  - Bump `version = "0.71.0"` → `"0.72.0"` (§7; confirm head is still 0.71.0 at
    ship time — re-grep, version may advance).
- `src/friendly.rs`: the 15 `codex32::Error::*` / `codex32::Fe::*` match + construct
  sites (lines 58, 65, 69, 73, 76, 79, 88, 94 in `friendly_ms_codec`; plus the test
  block sites 465, 495, 512–513, 525, 529, 533, 540, 554) rewrite
  `codex32::` → `ms_codec::codex32::`. `fe.to_char()` (line 67) is a method on the
  re-exported `Fe` — unchanged. No `use codex32` import exists in friendly.rs (it
  uses fully-qualified `codex32::`), so the rewrite is purely the qualified-path
  prefix.
- `tests/cli_invalidchecksum_redaction.rs`: any `codex32::` ref →
  `ms_codec::codex32::` (it referenced the variant in a doc-comment per the grep; if
  it names the type in code, rewrite; if only in a comment, update for accuracy).
- `Cargo.lock`: regenerate (`cargo build`/`cargo update -p ms-codec --precise 0.7.0`
  after publish) — the `codex32` package entry (lines 365–368) DISAPPEARS from the
  toolkit lock (no longer in the graph); the `ms-codec` entry (766–774) bumps to
  0.7.0 with a new checksum and drops `codex32` from its `dependencies` list (771).
- `fuzz/Cargo.lock`: same regeneration (the fuzz lock also carries `codex32` +
  `ms-codec` entries) — version-site touch per the toolkit release ritual
  (`project_toolkit_release_ritual_version_sites`: fuzz/Cargo.lock is a silent-drift
  site).
- `CHANGELOG.md` (toolkit): add a `0.72.0` entry (the toolkit HAS a tag-gated
  `changelog-check.yml` — MUST be in the version-site list).

### 5.3 Mirror-invariant confirmations (expected clean — confirm in P5)
- **GUI `schema_mirror`:** NO clap flag/subcommand/dropdown change in this cycle →
  no `mnemonic-gui/src/schema/mnemonic.rs` update. Confirm `mnemonic gui-schema`
  output is unchanged. **Expected clean** (no leading-discipline action; the lagging
  gate stays GREEN).
- **Manual mirror (`docs/manual/src/40-cli-reference/`):** NO CLI surface change on
  any of the four CLIs → no manual update. Confirm clean.
- **g6 mlock anchor:** this cycle does NOT touch `crates/ms-cli/src/mlock.rs` (ms) or
  `mnemonic-toolkit/.../mlock.rs` (toolkit) — the `mlock_g6_invariant.rs` byte-compare
  stays GREEN. **NEVER `cargo fmt` mlock.rs** (standing MEMORY rule). Confirm mlock.rs
  is out of the diff in both repos.

### 5.4 Phase-5 tests
- Full `cargo test -p ms-cli` (after the ms-codec path migration) GREEN.
- Full `cargo test -p mnemonic-toolkit` GREEN — especially `friendly.rs`'s
  `ms_codec_*` prose tests (the 15-site rewrite must keep every friendly message
  byte-identical; the tests assert on message substrings).
- Toolkit fuzz BUILD (`cargo +nightly fuzz build` or the repo's fuzz-smoke) — the
  `codex32` dep drop must not break the fuzz target graph.
- The toolkit's bitcoind differential / full suite as the repo's CI runs it.

### 5.5 Per-phase R0 + TDD
R0 → persist (BOTH repos' agent-reports as appropriate; the cross-repo review lives in
the repo whose diff is under review) → 0C/0I. Focus: the toolkit `codex32` dep drop is
complete (no dangling extern ref), friendly messages unchanged, no schema/manual/mlock
drift.

---

## 6. Phase 6 — publish, tag, FOLLOWUP flips

### 6.1 Order (root → consumer; codec/toolkit = direct-FF+tag, codecs publish to crates.io)
1. **ms-codec** → publish `0.7.0` to crates.io; tag (per the ms repo's tag namespace).
   CHANGELOG `0.7.0` entry added (ms-codec has a CHANGELOG; note the ms repo has NO
   `changelog-check.yml` gate — but the toolkit DOES, so the discipline is: ms-codec
   CHANGELOG is best-practice, toolkit CHANGELOG is gate-enforced).
2. **ms-cli** → publish `0.12.0` to crates.io (it path-deps ms-codec `=0.7.0`,
   resolvable post-publish); tag.
3. **toolkit** → bump `ms-codec = "0.7"`, drop codex32, regenerate both locks, add the
   CHANGELOG `0.72.0` entry, run the FULL suite + fuzz, then direct-FF + tag
   `mnemonic-toolkit-v0.72.0`.

(Coordination gotchas from MEMORY: detached-HEAD → `git push origin HEAD:main`;
`gh pr merge` under a worktree — verify via `gh pr view --json state,mergeCommit`;
codec/toolkit = direct-FF+tag, GUI would be PR+CI-before-tag but GUI is NOT touched
here.)

### 6.2 FOLLOWUP flips (in the shipping commit of the repo that owns each slug)
In `mnemonic-secret/design/FOLLOWUPS.md`:
- `rust-codex32-zeroize-upstream` (181) → **`resolved`** (Cycle-B): the inlined
  `Codex32String` now `ZeroizeOnDrop`s its inner String + redacting Debug. Note the
  close vehicle = inline-vendor (shape A), not an upstream release.
- `codex32-upstream-dormant-vendor-vs-accept-decision` (197) → **`resolved`** (Cycle-B):
  decision MADE + EXECUTED = (b)-equivalent via inline vendoring (shape A); external
  `codex32 =0.1.0` dep DROPPED across the workspace + toolkit. This is the umbrella;
  it closes last.
- `ms-codec-share-strings-not-zeroized-encode-and-combine` (439) → **`resolved`**
  (Cycle-B): the `Codex32String`/`String` legs are now drop-scrubbed; the irreducible
  `distributed: Vec<String>` return-value residue is documented under the caller-wrap
  contract; lint floor anchors the scrub (4→5). Record the residue honestly.
- `rust-codex32-upstream-pr2-recovery-bug-not-exposed` (189, currently `resolved`) →
  RE-POINT its "future codex32 bump" framing at the vendored `src/codex32/mod.rs`
  (the regression anchor `codex32_upstream_recovery_regression.rs` now guards the
  VENDORED code; update its doc-comment pointer). Status stays `resolved`.
- The `[obs] recovered-secret-string-not-zeroized` line (FOLLOWUPS.md:16) — mark
  subsumed/closed by the first-class share-strings slug (it is already a "broadened"
  pointer; close together; refresh the stale `shares.rs:236` citation to the live
  `shares.rs:291`).

**Toolkit companion FOLLOWUP:** the toolkit's
`friendly-ms1-invalidchecksum-echoes-full-input` (FOLLOWUPS.md:333, already
`resolved`) references `codex32 0.1.0`'s `InvalidChecksum{string}` — its source-of-
truth pointer should be updated to note codex32 is now vendored in ms-codec
(`ms_codec::codex32`); no status change. File a SMALL toolkit companion note that the
direct `codex32` dep was dropped in `v0.72.0` (cross-citing the ms `codex32-upstream-
dormant-vendor-vs-accept-decision` resolution) per the CLAUDE.md cross-repo
follow-up mirroring rule.

### 6.3 Per-phase R0 (ship review)
The mandatory POST-IMPLEMENTATION whole-diff adversarial review (per CLAUDE.md
convention 4 + MEMORY `feedback_post_impl_folds_reenter_review_loop`): an independent
opus review over the ENTIRE cross-repo diff (R0 = plan correctness; this catches
implementation-introduced regressions). Persist verbatim. Any post-review fold
RE-ENTERS a scoped convergence review before tag — do NOT self-verify a "mechanical"
CHANGELOG/pin fix and ship.

---

## 7. SemVer per crate (confirmed)

| Crate | From | To | Bump | Rationale |
|---|---|---|---|---|
| `ms-codec` | 0.6.0 | **0.7.0** | MINOR | Pre-1.0 breaking: `ms_codec::Error::Codex32`'s inner type moves `codex32::Error` → `ms_codec::codex32::Error` (a `pub`-reachable type-path change); NEW `pub mod codex32`; external `codex32` dep dropped. Wire format BYTE-IDENTICAL (no `ms1` output change). MINOR signals the break pre-1.0. |
| `ms-cli` | 0.11.0 | **0.12.0** | MINOR | Rides ms-codec 0.7.0 (exact pin `=0.7.0`); codex32 dep dropped; import paths migrated. No CLI flag change (no GUI/manual mirror impact) — but a MINOR to track the ms-codec coupling + the dep-graph change. |
| `mnemonic-toolkit` | 0.71.0 | **0.72.0** | MINOR | Consumes the breaking ms-codec `Error::Codex32` inner-type move via `friendly.rs` (15 sites); drops its direct `codex32` dep; re-pins ms-codec 0.6→0.7. End-user behavior unchanged but a `pub`-reachable match-path + dep-graph change → MINOR. (Re-grep head version at ship — may have advanced past 0.71.0.) |
| (vendored codex32) | — | — | NONE | Inlined private module (shape A); no crate, no version. |

---

## 8. Wire-byte-identity parity plan (the single most load-bearing invariant)

1. **Vendoring is a byte-for-byte copy** of `from_seed`/`from_string`/`interpolate_at`
   /`Parts`/`checksum.rs`/`field.rs` (§1.1) — the only edits ever made to the vendored
   bodies are the `Codex32String` derive list + new `Debug`/`ZeroizeOnDrop`
   (§2), which touch NO encoding path.
2. **NEW `codex32_vendor_parity.rs`** (P1, RED-first): hard-codes BIP-93-published
   output strings (from upstream `bip_vector_2/3/4/5`) + a golden set of `from_seed`
   outputs captured from the PRE-vendor `=0.1.0` build, and asserts the inlined
   module reproduces them BYTE-IDENTICALLY. Pins to the BIP + the captured golden, NOT
   to the inlined code itself.
3. **Existing corpus stays GREEN unchanged:** `bip93_inline_vectors.rs` (5 valid + 64
   invalid), `bip93_cross_format.rs` (§93.4 byte-pin), `spike_kofn.rs` (a/b/c),
   `codex32_upstream_recovery_regression.rs` (PR#2 secret), `negative.rs`,
   `uppercase_envelope.rs`, `mnem_byte_aligned_lengths.rs`, `forward_compat.rs`.
4. **Re-run the parity gate AGAIN after Phase 2** (the Zeroize/Debug change) — proves
   the derive edit perturbed zero output bytes.
5. **Cross-repo end-to-end:** the toolkit's bundle→restore + bitcoind-differential
   suites exercise ms1 encode/decode through the toolkit — green confirms the full
   pipeline is byte-stable post-vendor.

If ANY parity assertion fails at any phase: STOP — the vendor copy diverged from
upstream encoding; do not patch around it (the spike_kofn "STOP, do not patch" rule).

---

## 9. Exact file inventory per repo

**`mnemonic-secret` (ms-codec + ms-cli):**
- ADD: `crates/ms-codec/src/codex32/mod.rs`, `…/field.rs`, `…/checksum.rs`,
  `…/LICENSE`; `crates/ms-codec/tests/codex32_vendor_parity.rs`;
  `crates/ms-codec/tests/codex32_zeroize_debug.rs`.
- EDIT: `Cargo.toml` (drop workspace codex32 dep); `crates/ms-codec/Cargo.toml`
  (drop codex32 dep, zeroize `features=["derive"]`, version 0.7.0);
  `crates/ms-codec/src/lib.rs` (`pub mod codex32`); `src/error.rs`, `src/shares.rs`,
  `src/inspect.rs`, `src/envelope.rs`, `src/decode.rs` (path rewrites + share-spine
  comments); `tests/lint_zeroize_discipline.rs` (floor 4→5 + row); the 7 codex32-
  naming ms-codec test files (path rewrite); `crates/ms-codec/CHANGELOG.md` (0.7.0);
  `crates/ms-cli/Cargo.toml` (drop codex32 dep, ms-codec `=0.7.0`, version 0.12.0);
  `crates/ms-cli/src/codex32_friendly.rs`, `src/error.rs` + 11 ms-cli test files
  (path rewrites); `design/FOLLOWUPS.md` (flips).
- REMOVE FROM GRAPH (not a file edit): the `codex32` external dependency.

**`mnemonic-toolkit` (paired):**
- EDIT: `crates/mnemonic-toolkit/Cargo.toml` (drop codex32 dep, ms-codec 0.7,
  version 0.72.0); `src/friendly.rs` (15 path rewrites);
  `tests/cli_invalidchecksum_redaction.rs` (path/doc); `Cargo.lock` + `fuzz/Cargo.lock`
  (regenerate); `CHANGELOG.md` (0.72.0); `design/FOLLOWUPS.md` (companion note).

**NOT touched (confirm in diff):** any `mlock.rs` (g6 anchor); any clap-derived CLI
surface (no schema/manual mirror); `mnemonic-gui` (no schema change).

---

## 10. Phase summary (6 phases, each its own per-phase R0 + TDD)

- **P1** vendor 3 modules byte-identical + LICENSE/attribution + `pub mod codex32`
  re-export + drop external dep; RED-first parity gate; BIP-93/PR#2/spike_kofn GREEN.
- **P2** `ZeroizeOnDrop` + redacting `Debug` on `Codex32String` (only behavioral
  change); zeroize `derive` feature; RED-first Debug-redaction test; re-run parity.
- **P3** ms-codec rewire — `Error::Codex32` inner-type path move (source-compatible
  name), `codex32::` → `crate::codex32::` / `ms_codec::codex32::` across src+tests.
- **P4** share-spine scrub confirmation + honest residue doc; lint floor 4→5 + new row.
- **P5** ms-cli migration (drop dep, version 0.12.0) + the FORCED paired toolkit change
  (drop codex32 dep, friendly.rs 15-site rewrite, ms-codec 0.7, locks, CHANGELOG,
  version 0.72.0); confirm GUI-schema/manual/mlock all clean.
- **P6** publish ms-codec 0.7.0 + ms-cli 0.12.0 to crates.io, tag all three, flip 3
  slugs + 2 companions + toolkit companion note; mandatory whole-diff post-impl review.
