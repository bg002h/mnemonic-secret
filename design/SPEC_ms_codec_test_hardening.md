# SPEC — ms-codec test-hardening (themes 1/2/3)

**Status:** draft → mandatory opus R0 gate (0C/0I before any implementation).
**Repo:** `mnemonic-secret` (default branch **`master`**, NOT main). **Branch:** `ms-codec-test-hardening`. **Source SHA:** `c919f4b`.
**Scope:** TEST-ONLY. `proptest` is already a `[dev-dependencies]` (Cargo.toml:20). No production change unless a guard test surfaces a bug (§6). No version bump if clean.
**Cycle:** 3rd of three per-codec test-hardening cycles (mk → md → **ms**); mirrors the shipped mk/md cycles where the single-string + self-correcting nature of ms changes the shape.

---

## §1 — What makes ms different (and why this isn't a copy of md)

Grounded against source @ `c919f4b` (survey + architect review persisted alongside this SPEC):

1. **Single-string, never chunked.** ms1 is one codex32 string per entropy (lengths 50/56/62/69/75 chars for entropy 16/20/24/28/32 bytes; `consts.rs:28` `VALID_ENTR_LENGTHS`, `lib.rs` scope note; `error.rs` `TooManyErrors` has NO `chunk_index`; no `chunk`/`split`/`reassemble` module exists). **⇒ Theme 2 has NO cross-chunk branches and NO `restamp_chunk_header` helper** (unlike md's T2e/f/g). Theme 2 is purely single-codeword.
2. **Decode is self-correcting.** The toolkit indel oracle `Ms1IndelOracle` (`mnemonic-toolkit/.../src/repair.rs:885-908`) delegates to the SELF-correcting `ms_codec::decode_with_correction` — there is NO md-style hard-verify `reassemble`. ⇒ Theme 3 is reframed (see §4): it pins the codec-contract guarantees the oracle *relies on*, not a "hard-verify fails closed on indel" contract.
3. **proptest already present + a bijection property suite exists.** `tests/round_trip.rs` proptests `decode(encode(e)) == e` across all 5 lengths (`entropy_strategy(len)` at `round_trip.rs:7`). ⇒ Theme 1 is **EXTEND** (add a corrupt→correct→decode property), not bootstrap.
4. **Secret material.** ms carries BIP-39 entropy; `tests/lint_zeroize_discipline.rs` enforces a 4-row zeroize anchor list over `src/`. New TEST code is not bound by that lint (it gates `src/` anchors), but should match existing test conventions — hold entropy in local `Vec<u8>` (as `round_trip.rs` does) and not log/persist raw bytes. No new `Zeroizing` obligation on test code; do NOT add `Zeroizing` uses that would perturb the lint's `src/` anchor count.

### §1.1 — Verified API surface (cite at plan-write time; re-grep against `master`)
- `encode(tag: Tag, payload: &Payload) -> Result<String>` (`encode.rs:16`)
- `decode(s: &str) -> Result<(Tag, Payload)>` (`decode.rs:27`) — NON-correcting; rule-9 length gate is its FIRST check (`decode.rs:29`).
- `decode_with_correction(s: &str) -> Result<(Tag, Payload, Vec<CorrectionDetail>)>` (`decode.rs:188`) — correcting; computes the BCH residue **before** the length gate (`decode.rs:196-207`); defensive re-verify guard at `decode.rs:231-239` returns `Error::TooManyErrors { bound: 8 }`.
- `CorrectionDetail { position: usize, was: char, now: char }` (`decode.rs:86`) — `position` is the 0-indexed data-part position (post-`ms1`).
- `Payload::Entr(Vec<u8>)`, `Tag::ENTR`; `Payload`/`Tag` derive `PartialEq + Debug`.
- `Error::{ TooManyErrors { bound: u8 }, UnexpectedStringLength { got, allowed }, Codex32(..), WrongHrp{..}, … }` (`error.rs`). `TooManyErrors{bound}` is the only theme-relevant struct variant; `bound` is the display constant `8`.
- BCH(93,80,8) regular code, **t = 4**, **NON-perfect** (d=8/9 family, same as md/mk → 5–8 errors can MISCORRECT to a different valid codeword). 13-symbol checksum tail. codex32 alphabet `qpzry9x8gf2tvdw0s3jn54khce6mua7l`, HRP `"ms"`.
- Reusable fixture: `VALID_MS1_12W = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f"` (50 chars, data-part 47; `bch_decode.rs:35`). Existing symbol-flip helper pattern: `corrupt_at(s, pos, xor_mask)` (`bch_decode.rs:40`).

---

## §2 — Theme 1: EXTEND the property suite with corrupt→correct→decode

**New file:** `crates/ms-codec/tests/common/mod.rs` (`#![allow(dead_code)]`) — a shared `corrupt_at(&str, pos, xor_mask) -> String` (mirrors `bch_decode.rs:40`: flip the codex32 symbol at data-part position `pos`) reused by Theme 1 + Theme 2 + Theme 3. (`bch_decode.rs` keeps its own private copy untouched.)

**New file:** `crates/ms-codec/tests/proptest_correction.rs` — keeps `round_trip.rs` (the bijection) focused; adds the correction property.

**Property P-corr (the EXTEND):** for entropy of a valid length, inject `k ∈ 1..=4` symbol errors at `k` DISTINCT random data-part positions with nonzero xor masks, then `decode_with_correction` must:
- return `Ok`, `tag == Tag::ENTR`, `payload == Payload::Entr(original)` (recovery within t), AND
- report a `Vec<CorrectionDetail>` whose `position` SET equals the injected position set, and `len() == k` (position-accuracy — the soundness fact the toolkit ⊆-gate depends on; this is where it is pinned, per §4).

**Strategy shape (plan fills exact code):** generate `entropy: Vec<u8>` of a length drawn from `{16,20,24,28,32}`; derive the data-part length `dp = encode(...).len() - 3`; draw `k ∈ 1..=4`; draw `k` distinct positions in `0..dp`; draw `k` nonzero 5-bit xor masks. Inject via `common::corrupt_at`. (Distinct positions + nonzero mask ⇒ exactly `k` real symbol errors ⇒ always within t=4 ⇒ recovery guaranteed; a failure is a genuine bug.) Entropy lives in a local `Vec<u8>` per §1.4.

**Interpretation:** a recovery failure or position-set mismatch ⇒ genuine ms-codec bug ⇒ STOP (§6). A `corrupt_at` panic ⇒ a position ran past the data-part ⇒ fix the strategy bound, not the assert.

---

## §3 — Theme 2: BCH adversarial (single-codeword)

**New file:** `crates/ms-codec/tests/bch_adversarial.rs`. Drives `decode_with_correction` on `VALID_MS1_12W` (and optionally a longer fixture). NO chunking ⇒ NO restamp/cross-chunk cells.

Existing `bch_decode.rs` covers error-counts {0, 1, 4, 5}. This file fills the gaps + adds the miscorrection safety sweep:

- **T2a — gap-fill within t:** 2-error and 3-error corruptions → recover original + report the injected positions (the `{2,3}` coverage gap).
- **T2b — checksum-region, multi-error:** 2 errors inside the trailing 13-symbol checksum tail → corrected (extends `bch_decode.rs`'s single-checksum-error cell to t-relevant multiplicity).
- **T2c — 5–8-error miscorrection sweep (headline safety property):** seeded xorshift, ~300 trials × n_err ∈ 5..=8, each injecting `n_err` DISTINCT data-part symbol errors with nonzero masks. Assert **`!= Ok(original)`** — NOT `is_err()`. Rationale (cite `decode.rs:188-239`, `bch_decode.rs:416`): the BCH code is non-perfect, so a 5–8-error pattern can legitimately MISCORRECT to a *different* valid codeword (`Ok(different)`), which is acceptable BCH behavior; what must NEVER happen is a false claim of recovering the ORIGINAL from beyond-t errors. `is_err()` would be flaky (it forbids the legitimate `Ok(different)`). **Honesty note for R0:** this sweep pins the *safety* invariant ("never silently returns the original"); it does NOT deterministically prove the `decode.rs:231-239` re-verify guard load-bearing — that guard catches the BM-bogus-locator sub-case (~2⁻²⁶ to hit randomly), so we do not claim to exercise it deterministically. The observable beyond-t behavior (Err or Ok-different, never Ok-original) is what we pin.
- **T2d — deterministic beyond-t rejection:** one hand-picked 6-error pattern (or 7/8) → `Err(Error::TooManyErrors { bound: 8 })` (complements `bch_decode.rs::five_error_too_many`; build-time verify the chosen pattern errs, swap if it miscorrects, per the md `UNCORRECTABLE` convention).

**Interpretation:** a T2c `!= Ok(original)` failure (5–8 errors returned the original) ⇒ a real silent-acceptance bug ⇒ STOP (§6). T2a/T2b correction failures ⇒ genuine bug.

---

## §4 — Theme 3: indel reject-contract (refined (A′), per architect)

**New file:** `crates/ms-codec/tests/indel_reject_contract.rs` (own file for cross-codec parallelism with `mk-codec`/`md-codec`; NOT vacuous once T3-ms-2 is present). Header states: ms1 is single-string + self-correcting (no hard-verify `reassemble`); the toolkit `Ms1IndelOracle` (`mnemonic-toolkit/.../src/repair.rs:885-908`, delegating to `ms_codec::decode_with_correction`) is sound iff (i) length-restored-but-uncorrectable candidates fail closed (`Err`), and (ii) reported `CorrectionDetail.position` values are truthful — **(ii) is pinned by `bch_decode.rs` Cells 2/3/4/6 + the Theme-1 property `P-corr`, not re-pinned here.**

- **T3-ms-1 — `raw_wrong_length_fails_closed` (codec-contract regression pin):** from `VALID_MS1_12W`, (a) insert one data char → 51-char string; (b) delete one data char → 49-char string. Each: assert `decode_with_correction(&s).is_err()`. **Do NOT pin a specific variant** — `decode_with_correction` computes the BCH residue over the wrong-length symbol vector BEFORE the rule-9 length gate (`decode.rs:188-207`), so a raw ±1 string yields `Error::TooManyErrors{bound:8}` in the overwhelming majority and only occasionally `UnexpectedStringLength`; if a variant assertion is wanted use `matches!(err, Error::TooManyErrors{..} | Error::UnexpectedStringLength{..})`. **Comment must state:** the oracle never feeds `decode_with_correction` a wrong-length string (`indel.rs` length-restores every candidate via `data_variants`/`prefix_restorations`); this cell pins the CODEC contract (rule-9 gate at `decode.rs:29` + BCH-over-wrong-length both fail closed) so a future weakening is caught. Note the deterministic-`UnexpectedStringLength` path is the NON-correcting `decode()`, already pinned by `negative.rs::rule_9`.
- **T3-ms-2 — `length_correct_uncorrectable_indel_never_self_corrects` (the distinct oracle-soundness pin):** construct a **net-zero indel pair** — delete the data char at index `a` AND insert a different symbol at index `b` (length unchanged, ≥5 symbol positions differ) — mimicking a length-restored-but-wrong candidate the indel engine's wrong delete-guess produces. Assert `decode_with_correction(&bad)` is `Err(Error::TooManyErrors { bound: 8 })`: it must NOT silently self-correct (≤4 subst) to a *different* valid `(Tag, Payload)`. This is the ms analogue of md `t3d` / mk `t3a` "never self-correct to a different valid card," and the ONE Theme-3 guarantee not covered by Themes 1/2. (Build-time: verify the net-zero pair exceeds t; if the chosen `a`/`b` happen to land within t, widen the perturbation.)

**Dropped:** a position-accuracy cell (redundant — see (ii) above).

**Interpretation:** T3-ms-2 returning `Ok` with a different valid payload ⇒ the oracle could be fed a wrong recovery ⇒ HIGH-severity, STOP (§6).

---

## §5 — Cross-cutting

- **SemVer:** test-only (`proptest` is a dev-dep) ⇒ no version bump; commit to **`master`**. **BUT** per the convergence-suite precedent, Theme 2/Theme 3 are the most likely to surface a real codec bug; if a guard test goes red, ms-codec gets a PATCH fix-bump (its own R0) and the toolkit's git-dep pin to ms-codec may need a refresh.
- **Lockstep:** no clap/CLI surface change ⇒ no GUI schema-mirror, no manual flag-coverage, no sibling-codec companion FOLLOWUP. The only conditional lockstep is a codec fix-bump → toolkit re-pin (§5 bug case).
- **CI parity (mk/md lesson):** CI uses `dtolnay/rust-toolchain@stable`; format with `cargo +stable fmt` (verify the crate edition) and gate with `cargo +stable clippy --workspace --all-targets -- -D warnings`. If `cargo +stable fmt --all --check` / clippy flag PRE-EXISTING files (a toolchain-advance drift like md/mk hit), surface to the user (chore-fix vs leave + FOLLOWUP); do NOT fold into test commits. New test files must be stable-fmt + clippy clean.
- **Zeroize:** §1.4 — match existing test conventions; no new `Zeroizing` obligation on test code; don't perturb the `lint_zeroize_discipline.rs` `src/` anchor count.

## §6 — Bug-handling

A guard/property test that goes red = a real ms-codec defect (NOT a test to weaken). STOP, report DONE_WITH_CONCERNS with the minimal failing case, and (if confirmed) fix in production with its own per-fix R0 + a PATCH bump + toolkit re-pin. The two highest-bug-likelihood cells are T2c (silent miscorrection to the original) and T3-ms-2 (silent self-correct to a different valid entropy).

## §7 — Files
- Create `crates/ms-codec/tests/common/mod.rs` (`corrupt_at` + `#![allow(dead_code)]`).
- Create `crates/ms-codec/tests/proptest_correction.rs` (Theme 1 `P-corr`).
- Create `crates/ms-codec/tests/bch_adversarial.rs` (Theme 2 T2a–T2d).
- Create `crates/ms-codec/tests/indel_reject_contract.rs` (Theme 3 T3-ms-1, T3-ms-2).
- No production file changes (unless §6 fires).

## §8 — Phasing (for the plan-doc)
- **Phase 0:** `common/mod.rs` + `proptest_correction.rs` (Theme 1). Run + interpret.
- **Phase 1:** `bch_adversarial.rs` (Theme 2).
- **Phase 2:** `indel_reject_contract.rs` (Theme 3).
- **Phase 3:** full verify + end-of-cycle R0 + ship to `master` (no bump).
