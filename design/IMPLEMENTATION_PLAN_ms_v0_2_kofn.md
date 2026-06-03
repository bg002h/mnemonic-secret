# ms v0.2 K-of-N codex32 shares — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development (recommended) or executing-plans. Steps use checkbox (`- [ ]`) syntax. **Mandatory opus R0 on this plan + each phase + end-of-cycle, 0C/0I before code; re-dispatch after every fold** (CLAUDE.md). Per-phase TDD: failing test before impl.

**Goal:** Add BIP-93 codex32 K-of-N Shamir share encoding to `ms1` — split an `entr` or `mnem` secret into N shares, any K recombine — using codex32's native threshold(k)+index mechanism. v0.1/mnem single-strings stay byte-identical.

**Architecture:** ms-codec gains `Threshold`, `encode_shares`, `combine_shares` (a new `shares.rs` module) keyed on the codex32 threshold field; `decode` routes threshold∈2..9 to a "this is a share" error; the prefix byte (`0x00`/`0x02`) stays the payload-kind discriminator, recovered post-interpolation. ms-cli adds `ms split`/`ms combine`; the toolkit adds `mnemonic ms-shares split|combine`.

**Tech Stack:** Rust edition 2024, `codex32 = "=0.1.0"` (`from_seed`/`interpolate_at`/`Parts`), `getrandom`, `zeroize`. **ms-codec has NO CI** → local-only gate at every phase commit: `cargo test -p ms-codec && cargo test -p ms-cli && cargo clippy --all-targets -- -D warnings`. **Do NOT run `cargo fmt`** (repo not `+stable`-clean; write fmt-clean by hand). **SemVer (all MINOR):** ms-codec 0.3.0→0.4.0, ms-cli 0.6.0→0.7.0, toolkit v0.39.0→v0.40.0.

**SPEC:** `design/SPEC_ms_v0_2_kofn.md` (opus R0 GREEN; `design/agent-reports/ms-kofn-spec-R0-review.md` + `ms-kofn-design-review.md`). **Base SHAs:** mnemonic-secret `9d789b2`, toolkit `f95ddc4`, codex32 `0.1.0`. Re-grep citations at impl time.

**Branch:** `ms-v0.2-kofn` (mnemonic-secret) + `ms-v0.2-kofn-toolkit` (toolkit, P3+). Commit design artifacts (SPEC + reviews + recon + this plan) on the branch.

---

## File structure

- `crates/ms-codec/src/shares.rs` — **NEW**: `Threshold`, `encode_shares`, `combine_shares`, the share construction/recombination (the only new module).
- `crates/ms-codec/src/envelope.rs` — extract `payload_wire_bytes()` from `package()`; `discriminate`/`decode` threshold-routing.
- `crates/ms-codec/src/{consts.rs,error.rs,payload.rs,lib.rs}` — `RESERVED_ID_BLOCKLIST`, new error variants, `payload_wire_bytes` re-use, re-exports.
- `crates/ms-cli/src/cmd/{split.rs,combine.rs}` — **NEW**; `main.rs` enum/dispatch; `cmd/inspect.rs` share-branch.
- `mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/ms_shares.rs` — **NEW** (mirror `slip39.rs`); `main.rs`, `friendly.rs`, `Cargo.toml`, `scripts/install.sh`, `.github/workflows/manual.yml`.
- `MIGRATION.md` + `design/SPEC_ms_v0_1.md` §5/§4/§8 — amendment. Manual `docs/manual/src/40-cli-reference/43-ms.md` + toolkit chapter. GUI `mnemonic-gui/src/schema/{ms.rs,mnemonic.rs}`.

---

## Phase 0 — spike (the hard gate; throwaway-or-kept)

> Prove the three load-bearing claims against the pinned codex32 BEFORE any real code. If ANY fails, STOP and report — the design is wrong.

**Files:** `crates/ms-codec/tests/spike_kofn.rs` (keep as a permanent guard if it passes).

- [ ] **Step 1: write the spike.** Three asserts:
```rust
use codex32::{Codex32String, Fe};
// (a) byte-identity: encode_shares(ZERO,1) == encode  → here, prove the ZERO construction matches package()'s from_seed
#[test]
fn zero_share_is_byte_identical_to_single() {
    for n in [16usize,20,24,28,32] {
        let mut data = vec![0x00u8]; data.extend(std::iter::repeat(0xABu8).take(n));
        let single = Codex32String::from_seed("ms", 0, "entr", Fe::S, &data).unwrap().to_string();
        // the share path's ZERO case must reproduce exactly this
        assert_eq!(single, Codex32String::from_seed("ms", 0, "entr", Fe::S, &data).unwrap().to_string());
    }
}
// (b) K-of-N round-trip, entr AND mnem, all 5 lengths, k∈2..9
#[test]
fn kofn_round_trip_entr_and_mnem() {
    for prefix in [vec![0x00u8], vec![0x02u8, 0x01u8]] { // entr [0x00]; mnem [0x02][lang=ja]
        for n_ent in [16usize,20,24,28,32] {
            for k in 2u8..=9 {
                let mut secret_bytes = prefix.clone(); secret_bytes.extend(std::iter::repeat(0xCDu8).take(n_ent));
                let id = "tst7";
                let secret_s = Codex32String::from_seed("ms", k, id, Fe::S, &secret_bytes).unwrap();
                // k-1 random defining shares at distinct non-s indices
                let idxs = [Fe::A, Fe::C, Fe::D, Fe::E, Fe::F, Fe::G, Fe::H, Fe::J]; // ≠ s
                let mut defining = vec![secret_s.clone()];
                for j in 0..(k as usize - 1) {
                    let mut r = prefix.clone(); r.extend(std::iter::repeat((0x10 + j as u8)).take(n_ent));
                    defining.push(Codex32String::from_seed("ms", k, id, idxs[j], &r).unwrap());
                }
                // recover the secret-at-S from any k of the DISTRIBUTED shares
                // distributed = defining[1..] (k-1) + interpolate the rest up to k total distinct non-s
                let recovered = Codex32String::interpolate_at(&defining[1..].to_vec(), /*+ derived to reach k*/ Fe::S);
                // NOTE: spike intent — assert interpolate_at(defining_set, Fe::S) returns secret_s; refine indices so a k-subset recovers.
                // The real test: collect k distributed shares, interpolate_at(_, Fe::S) == secret_s, then strip prefix → secret_bytes.
            }
        }
    }
}
// (c) C1: combine must reject an index-s input — interpolate_at short-circuits on it
#[test]
fn interpolate_short_circuits_on_index_s() {
    // build secret_s + one defining; interpolate_at([secret_s, anything], Fe::S) returns secret_s WITHOUT validating `anything`
    // → proves combine_shares must pre-reject index==s before calling interpolate_at.
}
```
- [ ] **Step 2: refine + run** `cargo test -p ms-codec --test spike_kofn`. The (b) round-trip is the load-bearing one: construct secret-at-S + (k−1) random defining shares at distinct non-`s` indices, derive the remaining distributed shares via `interpolate_at(&[secret_s, defining…], Fe::<idx>)`, then take ANY k of the n distributed shares and `interpolate_at(those_k, Fe::S)` → MUST equal `secret_s` (compare `.to_string()`), and `Parts::data()` of the result MUST start with the prefix byte + recover `secret_bytes`. Cross-check the construction against codex32 `bip_vector_3` (`lib.rs:489`). Run for entr AND mnem, all 5 lengths, k∈2..=9. **If any length/kind fails to round-trip, STOP.**
- [ ] **Step 3: prove C1** — `interpolate_at(&[secret_s, distributed_share], Fe::S)` returns `secret_s` directly (short-circuit at `lib.rs:259`) without validating `distributed_share`; and `&[secret_s, secret_s]` returns `Ok` with no `RepeatedIndex`. This empirically justifies `combine_shares`'s pre-reject of index-`s`. **If the short-circuit does NOT fire, the C1 reasoning is wrong — STOP.**
- [ ] **Step 4: commit** (keep the spike as a guard): `git add crates/ms-codec/tests/spike_kofn.rs && git commit -m "test(ms-codec): Phase-0 K-of-N spike — byte-identity + entr/mnem round-trip + C1 short-circuit (gate)"`

---

## Phase 1 — ms-codec (Threshold, encode_shares, combine_shares, decode-routing)

### Task 1.1 — `Threshold` type + `RESERVED_ID_BLOCKLIST` + new errors
**Files:** Create `crates/ms-codec/src/shares.rs`; modify `consts.rs`, `error.rs`, `lib.rs`.
- [ ] **Step 1: failing unit tests** (in `shares.rs` `#[cfg(test)]`): `Threshold::new(2)`/`new(9)` Ok; `new(0)`→`Err(InvalidThreshold(0))`; `new(1)`→`Err(InvalidThreshold(1))`; `new(10)`→`Err(InvalidThreshold(10))`; `Threshold::ZERO.get()==0`; `Threshold::new(5)?.get()==5`.
- [ ] **Step 2: run → FAIL** (`shares` module doesn't exist). `cargo test -p ms-codec --lib shares`.
- [ ] **Step 3: implement.** In `shares.rs`: `pub struct Threshold(u8);` with `pub const ZERO: Threshold = Threshold(0);`, `pub fn new(k: u8) -> Result<Threshold>` (Ok for 2..=9 else `Error::InvalidThreshold(k)`), `pub fn get(self) -> u8`. In `consts.rs` (after `:62`): `pub const RESERVED_ID_BLOCKLIST: &[[u8;4]] = &[*b"entr", *b"seed", *b"xprv", *b"mnem", *b"prvk"];` (distinct from `RESERVED_NOT_EMITTED_V01` `:62` which stays `[seed,xprv,prvk]`). In `error.rs` (enum `:9`, NON-alphabetical pre-existing — insert NEW variants alphabetically among themselves, do NOT retro-sort): add `InvalidShareCount { k: u8, n: usize }`, `InvalidThreshold(u8)`, `IsShareNotSingleString { threshold: char, index: char }`, `SecretShareSuppliedToCombine` with **`Display` arms ONLY (R0-I2: `ms_codec::Error` has NO `exit_code`/`kind` methods — those live in ms-cli's `CliError`; do NOT add them here).** The exit-code/message mapping is ms-cli's job (Task 2.0). Add `pub mod shares;` + `pub use shares::{Threshold, encode_shares, combine_shares};` to `lib.rs` (`:52-57` re-export block).
- [ ] **Step 4: run → PASS.** Commit: `git add crates/ms-codec/src/{shares.rs,consts.rs,error.rs,lib.rs} && git commit -m "feat(ms-codec): Threshold type + RESERVED_ID_BLOCKLIST + K-of-N error variants (P1)"`

### Task 1.2 — `payload_wire_bytes` helper (R0-m1: extract from `package()`)
**Files:** `envelope.rs`, `payload.rs`.
- [ ] **Step 1: failing test:** `payload_wire_bytes(&Payload::Entr(vec![0xAB;16]))` == `[0x00, 0xAB×16]`; `payload_wire_bytes(&Payload::Mnem{language:1, entropy:vec![0xAB;16]})` == `[0x02, 0x01, 0xAB×16]`. (`Zeroizing<Vec<u8>>` return.)
- [ ] **Step 2: run → FAIL.**
- [ ] **Step 3: implement.** Add `pub(crate) fn payload_wire_bytes(p: &Payload) -> Zeroizing<Vec<u8>>` (in `envelope.rs`): `match p { Payload::Entr(e) => [RESERVED_PREFIX]++e, Payload::Mnem{language,entropy} => [MNEM_PREFIX,*language]++entropy, _ => unreachable!() }`. **Refactor `package()` (`envelope.rs` ~:171 `v.push(RESERVED_PREFIX)`) to call `payload_wire_bytes()`** — `package` MUST stay byte-identical (the v0.1 vector suite is the gate). Confirm `Tag::as_str()` (`tag.rs:56`), NOT a `tag.id()`.
- [ ] **Step 4: run → PASS** + `cargo test -p ms-codec --test vectors` (the v0.1 byte-identity corpus must pass UNCHANGED). Commit.

### Task 1.3 — `encode_shares` (R0 I3 signature; M5 derive-all-N; getrandom)
**Files:** `shares.rs`.
- [ ] **Step 1: failing tests:** (a) `encode_shares(Tag::ENTR, Threshold::ZERO, 1, &p)` == `vec![encode(Tag::ENTR, &p)?]` byte-identical (entr + mnem). (b) `encode_shares(Tag::ENTR, Threshold::new(2)?, 3, &entr_p)?.len()==3`; each parses, has threshold char `2`, distinct non-`s` indices, same `id`. (c) `encode_shares(_, new(2)?, 1, _)` → `Err(InvalidShareCount{k:2,n:1})`; `n=32` → `Err(InvalidShareCount)`. (d) round-trip: split → `combine_shares(any 2 of 3)` == the secret (deferred to 1.4's combine, or inline a manual interpolate check).
- [ ] **Step 2: run → FAIL.**
- [ ] **Step 3: implement** `pub fn encode_shares(tag: Tag, threshold: Threshold, n: usize, secret: &Payload) -> Result<Vec<String>>`:
  - `secret.validate()?`; `let bytes = payload_wire_bytes(secret);`
  - `if threshold == ZERO`: require `n==1` (else `InvalidShareCount`); return `vec![Codex32String::from_seed(HRP, 0, tag.as_str(), Fe::S, &bytes)?.to_string()]` — **id = tag (NOT random)**, byte-identical to `encode`.
  - else `k = threshold.get()`: require `k as usize <= n && n <= 31` (else `InvalidShareCount{k,n}`).
    1. random `id`: 4 codex32-charset chars via `getrandom`, re-roll while `id.as_bytes() ∈ RESERVED_ID_BLOCKLIST`.
    2. `secret_s = from_seed(HRP, k, &id, Fe::S, &bytes)?`.
    3. `k-1` defining shares: pick distinct non-`s` indices `idx_1..idx_{k-1}` (from the 31-index pool); each `from_seed(HRP, k, &id, idx_j, &csprng_bytes_of(bytes.len()))?` (CSPRNG payload, same length).
    4. defining set = `[secret_s, def_1..def_{k-1}]` (k points); for the remaining `n-(k-1)` distinct non-`s`/non-defining indices: `Codex32String::interpolate_at(&defining_set, idx)?`.
    5. return the `n` **distributed** strings (`def_1..def_{k-1}` + interpolated); NOT `secret_s`.
  - **RNG (R0-minor):** the workspace tree has `getrandom` **0.3.x** → the API is `getrandom::fill(&mut buf)` (NOT the 0.2 `getrandom::getrandom`); verify the pinned version + add to `crates/ms-codec/Cargo.toml` if absent (`mk_codec` precedent). Zeroize the random/secret intermediates.
  - **Index selection:** choose the `n` distinct non-`s` indices in a FIXED canonical order from the codex32 alphabet pool (e.g. iterate `qpzry9x8gf2tvdw03jn54khce6mua7l` skipping `s`, taking the first `n`); the `k-1` defining indices are the first `k-1` of that order, the interpolated ones the rest. Deterministic index assignment (only the defining-share PAYLOADS are random).
- [ ] **Step 4: run → PASS.** Commit: `git add crates/ms-codec/src/shares.rs crates/ms-codec/Cargo.toml && git commit -m "feat(ms-codec): encode_shares — K-of-N split (entr+mnem), ZERO byte-identical (P1)"`

### Task 1.4 — `combine_shares` (R0 C1 + I2 + I3 full pre-validation)
**Files:** `shares.rs`.
- [ ] **Step 1: failing tests:** round-trip (split via 1.3 → combine any k → `(Tag, Payload)` == original, entr + mnem, language preserved); `<k` shares → `Error::Codex32(ThresholdNotPassed{..})`; duplicate index → `RepeatedIndex`; **index-`s` input → `Error::SecretShareSuppliedToCombine` (C1)**; mismatched id/threshold/length/hrp → respective `Mismatched*`.
- [ ] **Step 2: run → FAIL.**
- [ ] **Step 3: implement** `pub fn combine_shares(shares: &[String]) -> Result<(Tag, Payload)>`:
  - parse each via `Codex32String::from_string` (map errors via `Error::Codex32`).
  - pre-validation BEFORE `interpolate_at`: (i) parse the first share's `Parts` for threshold `k`; **(ii) reject any share whose `parts.share_index == Fe::S` → `Error::SecretShareSuppliedToCombine` (C1 — the short-circuit at `lib.rs:259` would otherwise bypass validation);** (iii) `shares.len() >= k` else `Error::Codex32(ThresholdNotPassed{..})` (or surface directly); (iv) distinct `share_index` across inputs else `Error::RepeatedIndex` (don't rely on `interpolate_at`'s lazy check).
  - `let secret = Codex32String::interpolate_at(&parsed, Fe::S)?;` (surfaces `Mismatched{Hrp,Id,Threshold,Length}` via `Error::Codex32`).
  - **R0-I1 — extract a header-gate-free dispatch helper** `pub(crate) fn dispatch_payload(data: &[u8]) -> Result<Payload>` in `envelope.rs`: the prefix→Payload logic ONLY (read `data[0]`: `0x00`→`Payload::Entr(rest)`, `0x02`→`Payload::Mnem{language:rest[0], entropy:rest[1..]}`, else `ReservedPrefixViolation`; then `validate()`) — with NO threshold/share-index header check. Refactor `discriminate`'s TAIL (after its header gate `:105-112`) to call `dispatch_payload(&c.parts().data())`; `combine_shares` calls `dispatch_payload(&secret.parts().data())`. **(R0-m-r1: it's `Codex32String::parts().data()` — `Parts::data()`, per envelope.rs:131 — NOT `c.data()`.)** **No flag** (a flag would conflate two meanings). This is the cleanest factoring — the threshold gate stays only in `discriminate`'s header path.
  - **R0-C1-Tag — `combine_shares` returns `(Tag::ENTR, dispatch_payload(&secret.parts().data())?)`.** The recovered secret-at-S carries the share-set's RANDOM `id` (NOT a type tag); routing through anything that builds `Tag` from the id field would return `Tag(<random>)` — silently wrong. The payload KIND is the prefix byte (via `dispatch_payload`); the Tag is always `Tag::ENTR`. **Discard the random id.** (Do NOT call `discriminate` here — it would rebuild Tag from the id.)
- [ ] **Step 4: run → PASS** (full round-trip green). Commit: `git add crates/ms-codec/src/shares.rs && git commit -m "feat(ms-codec): combine_shares — recover via interpolate_at(S) + C1 index-s reject + I2/I3 validation (P1)"`

### Task 1.5 — `decode` threshold-routing (R0 I1) + the §5/MIGRATION amendment
**Files:** `envelope.rs` (`discriminate`), `decode.rs`, `MIGRATION.md`, `design/SPEC_ms_v0_1.md`.
- [ ] **Step 1: failing tests:** `decode(<a threshold=2 share string>)` → `Err(IsShareNotSingleString{threshold:'2', index:..})` (NOT `ThresholdNotZero`); `decode(<v0.1 entr single>)` + `decode(<mnem single>)` still Ok (unchanged).
- [ ] **Step 2: run → FAIL.**
- [ ] **Step 3: implement.** In `discriminate` (`envelope.rs:95`), the current threshold check (`:105` `if threshold_byte != THRESHOLD_V01 → ThresholdNotZero`): **replace** with: `match threshold_byte { b'0' => /* proceed */, b'2'..=b'9' => return Err(Error::IsShareNotSingleString { threshold: threshold_byte as char, index: share_index_byte as char }), other => return Err(Error::ThresholdNotZero { got: other }) }`. Keep the share-index check (`:110`) only on the `b'0'` path. **Amend `MIGRATION.md` (repo root) + `SPEC_ms_v0_1.md §5` invariants 1+2+4, the §5 registry table, §4 rules 3/4 (reject→route), §8 table** per SPEC §5 (this plan's SPEC). Commit the docs in this task.
- [ ] **Step 4: run → PASS.** Commit: `git add crates/ms-codec/src/{envelope.rs,decode.rs} MIGRATION.md design/SPEC_ms_v0_1.md && git commit -m "feat(ms-codec): decode routes threshold 2..9 to IsShareNotSingleString + §5/MIGRATION amendment (P1)"`

### Task 1.6 — Phase-1 gate + per-phase opus R0
- [ ] **Step 1:** `cargo test -p ms-codec && cargo test -p ms-cli && cargo clippy --all-targets -- -D warnings` (NO fmt) → all green. Bump `crates/ms-codec/Cargo.toml` 0.3.0→0.4.0 + CHANGELOG; `cargo build` to relock; stage `Cargo.lock`.
- [ ] **Step 2: per-phase opus R0** of the P1 diff → persist to `design/agent-reports/ms-kofn-phase-1-R0-review.md`; loop to 0C/0I. Scrutinize: the `combine_shares` discriminate/threshold-gate interaction (1.4 flag), the byte-identity (package refactor), the C1 reject, error-variant placement.
- [ ] **Step 3: commit (no tag).**

---

## Phase 2 — ms-cli (`ms split` / `ms combine` / inspect-of-share)

### Task 2.0 — ms-cli error surfacing (R0-C2 — the missed 2nd instance)
**Files:** `crates/ms-cli/src/error.rs` (the `From<ms_codec::Error> for CliError` at `:201`, which has a wildcard `_ => BadInput`/"unhandled" → exit 1).
- [ ] **Step 1: failing test:** an `ms_codec::Error::IsShareNotSingleString{..}` (and `SecretShareSuppliedToCombine`, `InvalidThreshold`, `InvalidShareCount`) mapped through `CliError` → a clean message (NOT "unhandled ms_codec::Error variant") + the intended exit code; `Error::Codex32(ThresholdNotPassed/Mismatched*/RepeatedIndex)` → friendly via `codex32_friendly`.
- [ ] **Step 2: run → FAIL** (wildcard `_ => BadInput` at `error.rs:201`).
- [ ] **Step 3: implement.** Add explicit `From`/dispatch arms in `ms-cli/src/error.rs:201` for the 4 new `ms_codec::Error` variants (map to the appropriate `CliError` kind + exit code — `IsShareNotSingleString`/`SecretShareSuppliedToCombine`/`InvalidThreshold`/`InvalidShareCount` → a usage-class code, not the generic exit-1 `BadInput`); ensure `Error::Codex32(<share variant>)` routes through `codex32_friendly` (`codex32_friendly.rs:11-57`). **This is the CLASS fix — the toolkit's analogous wildcard is Task 3.3; both must be done.**
- [ ] **Step 4: run → PASS.** Commit.

### Task 2.1 — `ms split`
**Files:** Create `crates/ms-cli/src/cmd/split.rs`; `main.rs` (enum `:69`, dispatch `:155`).
- [ ] **Step 1: failing integration test** (`crates/ms-cli/tests/`): `ms split --phrase "<en 12-word>" -k 2 -n 3` → 3 lines, each an ms1 of entr-share length, threshold char `2`, same id, distinct indices; `ms combine` of any 2 → the entr phrase. `ms split --language japanese --phrase "<ja>" -k 2 -n 3` → 3 mnem-shares; combine → ja phrase. `-k 1` / `-n 1` / `-n 32` → clean error (exit 64/usage).
- [ ] **Step 2: run → FAIL.**
- [ ] **Step 3: implement** `cmd/split.rs`. **R0-I3: there is NO reusable source-parse fn — `encode::run()` (`:59-100`) does phrase/hex→`Payload` inline.** First EXTRACT it: add **`pub(crate) fn resolve_secret_payload(phrase: Option<&str>, hex: Option<&str>, language: CliLanguage) -> Result<(Payload, Option<&'static str>)>`** (R0-I-r1 — the tuple's 2nd element is `language_for_card`: `Some(language.as_str())` for a phrase, `None` for `--hex`; `CliLanguage::as_str` is `&'static str`, language.rs:51) — the phrase/hex→entropy + the non-English-phrase→`Payload::Mnem` / English|hex→`Payload::Entr` AUTO-route — into `cmd/encode.rs`. Refactor `encode::run` to call it and **reconstruct `language_for_card` from the tuple's 2nd element** (encode.rs:65 currently binds both; a bare `Payload` can't tell English-phrase from `--hex` since both → `Payload::Entr` → the card/json `language` field would regress). **Add a Task-2.1 sub-assertion: `encode`'s text + `--json` output is UNCHANGED for {english phrase, non-english phrase, hex}** (the byte-identity claim, now tested at the encode-output level). Then `cmd/split.rs` calls the SAME helper (ignoring the 2nd tuple element — it re-derives language into the share-set via the `Payload`) so a non-English phrase splits as mnem (language survives). Then `ms_codec::encode_shares(Tag::ENTR, Threshold::new(k)?, n, &payload)`, print N shares (text: one per line + engraving grouping; `--json`: `{shares, k, n, id, kind, language?}`). Add `Split(cmd::split::SplitArgs)` to the enum + dispatch. **Output-class:** `emit_output_class_advisory(OutputClass::PrivateKeyMaterial, …)` (advisory.rs:37) — the share SET is secret-equivalent. Wrap entropy in `Zeroizing`.
- [ ] **Step 4: run → PASS.** Commit.

### Task 2.2 — `ms combine`
**Files:** Create `crates/ms-cli/src/cmd/combine.rs`; `main.rs`.
- [ ] **Step 1: failing tests:** `ms combine <s1> <s2> [--to phrase]` → the recovered phrase (wire language for mnem); `--to entropy`/`--to ms1`; `<k` → `ThresholdNotPassed` friendly message; index-`s` input → `SecretShareSuppliedToCombine` friendly message; duplicate → `RepeatedIndex`.
- [ ] **Step 2: run → FAIL.**
- [ ] **Step 3: implement** `cmd/combine.rs`: positional `<share>...`, `ms_codec::combine_shares(&shares)` → `(Tag, Payload)`; emit per `--to` (default `phrase` — render via the Payload's language for mnem, English for entr; reuse `cmd/decode.rs`'s phrase rendering). Errors via `codex32_friendly.rs` (already maps the codex32 share variants, `:11-57`) + the new ms-codec variants. Output-class `PrivateKeyMaterial`; `Zeroizing`. Add enum + dispatch.
- [ ] **Step 4: run → PASS.** Commit.

### Task 2.3 — `ms inspect` of a share (R0-m2: first-class at `analyze()`)
**Files:** `crates/ms-cli/src/cmd/inspect.rs` (`analyze()` `:49`).
- [ ] **Step 1: failing test:** `ms inspect <a threshold=2 share>` → exit 0, reports `kind: share`, `threshold: 2`, `id`, `index`; does NOT print `threshold-not-zero`/`FAIL`.
- [ ] **Step 2: run → FAIL** (`analyze()` rule 3 `:58` pushes `threshold-not-zero`).
- [ ] **Step 3: implement.** In `analyze()` (`:49`): BEFORE rule 3, add a share branch — if `report.threshold ∈ 2..=9` → it's a share: `would_decode` semantics become "would combine (needs k)"; do NOT push `threshold-not-zero`/length failures. Surface `kind: share` + threshold/id/index in `emit_text`/`emit_json` (`InspectReport` already carries `threshold` + `share_index` — R0-confirmed; reuse them). **R0-minor: SUPPRESS the garbage `prefix_byte`/`kind`/`payload_bytes` fields for a share** — a distributed share's `data()[0]` is an interpolated value, NOT a meaningful prefix; inspect must not display/interpret it as a payload kind.
- [ ] **Step 4: run → PASS.** Commit.

### Task 2.4 — Phase-2 gate + version bump + per-phase R0
- [ ] **Step 1:** full local gate (test ms-codec + ms-cli + clippy). Bump `crates/ms-cli/Cargo.toml` 0.6.0→0.7.0 + CHANGELOG; relock; stage Cargo.lock.
- [ ] **Step 2: per-phase opus R0** → `design/agent-reports/ms-kofn-phase-2-R0-review.md`; 0C/0I.
- [ ] **Step 3: commit (no tag).**

---

## Phase 3 — toolkit `mnemonic ms-shares` (SEVERABLE — ship P1+P2 standalone if this stalls)

> Toolkit re-pin requires ms-codec 0.4.0 + ms-cli v0.7.0 PUBLISHED. Develop against a TEMP `[patch.crates-io] ms-codec = { path = "../mnemonic-secret/crates/ms-codec" }` (uncommitted/removed at ship — the mnem-cycle pattern). Branch `ms-v0.2-kofn-toolkit`.

### Task 3.1 — re-pin + build harness
- [ ] **Step 1:** TEMP path-override in toolkit workspace `Cargo.toml [patch.crates-io]`; bump `crates/mnemonic-toolkit/Cargo.toml:20` `ms-codec = "0.3.0"`→`"0.4.0"`; bump ms-cli tag pins `ms-cli-v0.6.0`→`v0.7.0` at `scripts/install.sh:38` + `.github/workflows/manual.yml:88`; `cargo build -p mnemonic-toolkit` (resolves local 0.4.0).

### Task 3.2 — `mnemonic ms-shares split|combine` (mirror `slip39.rs`)
**Files:** Create `crates/mnemonic-toolkit/src/cmd/ms_shares.rs` (mirror `cmd/slip39.rs`); `main.rs` (enum `:89`, `Slip39` `:119` / dispatch `:173` as the template).
- [ ] **Step 1: failing tests** (R0-minor: **match `cmd/slip39.rs`'s ACTUAL input grammar** — verify at impl time whether it's `--from phrase=…`/`--threshold`/`--shares` vs `--phrase`/`-k`/`-n`; mirror it exactly, don't invent flags): `mnemonic ms-shares split <ja source> -k/threshold 2 -n/shares 3` → 3 mnem-shares; `mnemonic ms-shares combine <s1> <s2> --to ms1` → recovered ms1; `--to phrase` → ja phrase; `combine --to ms1` piped into `bundle --slot @0.ms1=<recovered>` → a valid bundle (composition). entr + mnem.
- [ ] **Step 2: run → FAIL.**
- [ ] **Step 3: implement** `cmd/ms_shares.rs` with `split`/`combine` modes (clap subcommand, mirroring `Slip39Args`'s split-or-combine shape): `split` → `ms_codec::encode_shares`; `combine` → `ms_codec::combine_shares` → emit per `--to`. Add `MsShares(cmd::ms_shares::MsSharesArgs)` to the toolkit enum + dispatch (`run(args, stdin, stdout, stderr)` signature like slip39). Output-class `PrivateKeyMaterial`; `Zeroizing`.
- [ ] **Step 4: run → PASS.** Commit.

### Task 3.3 — consume-path threshold-dispatch + friendly arms (R0-m3)
**Files:** `friendly.rs` (`friendly_ms_codec` `:42`, wildcard `:80`), `ms_codec_exit_code`, the consume sites (`cmd/inspect.rs:171` `decode_card`).
- [ ] **Step 1: failing test:** `mnemonic inspect <a share>` / `mnemonic convert --from ms1=<share>` → a friendly "this is a K-of-N share; use `mnemonic ms-shares combine`" message (NOT "unhandled ms_codec::Error variant"), exit code mapped.
- [ ] **Step 2: run → FAIL** (wildcard `:80` → "unhandled").
- [ ] **Step 3: implement.** Add explicit arms in `friendly_ms_codec` (`:42-80`) for `IsShareNotSingleString`, `SecretShareSuppliedToCombine`, and the surfaced `Codex32(Mismatched*/RepeatedIndex/ThresholdNotPassed)`; add `ms_codec_exit_code` arms (map `IsShareNotSingleString` to a usage-ish code). The consume paths propagate via `?` (no logic change beyond the friendly map).
- [ ] **Step 4: run → PASS.** Commit.

### Task 3.4 — Phase-3 gate + version bump + per-phase R0
- [ ] **Step 1:** `cargo test -p mnemonic-toolkit --no-fail-fast && cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` (toolkit CI gates clippy). Bump `Cargo.toml` v0.39.0→v0.40.0 + both README markers + CHANGELOG + `install.sh` toolkit self-pin. **Run `cargo test --no-fail-fast` WORKSPACE-wide** (the mnem-cycle lesson — a phase-1 regression escaped a crate-scoped run).
- [ ] **Step 2: per-phase opus R0** → `design/agent-reports/ms-kofn-phase-3-R0-review.md`; 0C/0I.
- [ ] **Step 3: commit (no tag).** Override stays (removed at ship).

---

## Phase 4 — docs / GUI lockstep + contract-test re-spec

### Task 4.1 — re-spec the broken contract test (R0 I5; shared-gate sweep)
- [ ] **Step 1:** `crates/ms-cli/tests/decode_rejects_threshold_not_zero.rs` currently asserts a threshold=2 string → `ThresholdNotZero` (text `:25` "threshold"; json `:46` kind `"ThresholdNotZero"`, exit 2). The new behavior: → `IsShareNotSingleString`. **Re-spec** the test (rename to `decode_routes_share_to_is_share_not_single_string` or update assertions) to assert kind `"IsShareNotSingleString"` + **the exit code Task 2.0 establishes** (NOT the old exit 2 — R0-C2 warned this test would otherwise bake the exit-1 bug in as "expected"; the re-spec MUST match the Task-2.0 mapping, proven correct, per the recapture-golden-only-when-correct lesson). **Grep ALL tests** (`grep -rn 'ThresholdNotZero\|threshold' crates/*/tests/`) for other assertions on the old reject; update each.
- [ ] **Step 2:** full suite green.
- [ ] **Step 3: commit.**

### Task 4.2 — manual + GUI schema_mirror
- [ ] **Step 1:** Manual `docs/manual/src/40-cli-reference/43-ms.md` (`ms split`/`combine`) + the toolkit chapter (`mnemonic ms-shares`). Build all 4 binaries + `make -C docs/manual audit` (exit 0; new non-English share transcripts only if added).
- [ ] **Step 2:** GUI `mnemonic-gui/src/schema/ms.rs` (`split`/`combine` flag-name entries) + `mnemonic.rs` (`ms-shares` + modes) — the `schema_mirror` test consumes `gui-schema`; the new subcommands auto-appear (reflective), the hand-mirror must catch up. **File a FOLLOWUP** for the `--json` wire-shape (ungated; `ms split`/`combine`/`inspect`-share + toolkit) → paired-PR self-update. Commit.

### Task 4.3 — end-of-cycle opus R0 + design audit trail
- [ ] **Step 1: end-of-cycle opus R0** (ms diff + toolkit diff) → `design/agent-reports/ms-kofn-end-of-cycle-R0-review.md`; loop to 0C/0I.
- [ ] **Step 2: commit the design audit trail** (SPEC + this plan + all `ms-kofn-*` reviews + the recon) on the branch.
- [ ] **Step 3: ship sequence (authorization-gated):** publish ms-codec 0.4.0 + ms-cli 0.7.0 → push ms tags `ms-codec-v0.4.0`/`ms-cli-v0.7.0` → REMOVE the toolkit `[patch.crates-io] ms-codec` override → `cargo build` → assert `Cargo.lock` ms-codec is `registry+`-sourced + `cargo metadata --locked` passes → commit toolkit re-pin + v0.40.0 → tag `mnemonic-toolkit-v0.40.0` → merge all default branches → flip the SPEC §8 / FOLLOWUP statuses.

---

## Self-review (spec coverage)

- SPEC §1 wire/threshold-dispatch → P1 1.5 + P0. §2 API (Threshold, encode_shares, combine_shares, decode, errors, consts) → 1.1–1.5. §3 ms-cli (split/combine/inspect, output-class) → 2.1–2.3. §4 toolkit (ms-shares, threshold-dispatch, friendly) → 3.2–3.3. §5 migration → 1.5. §6 lockstep (GUI/manual/re-pin/SemVer) → 3.1, 4.2, version-bump steps. §7 phasing → matches P0–P4. §8 tests → embedded per task + 4.1. §9 footguns (C1, signature, threshold-dispatch, id-blocklist, contract-test, n≤31, ZERO-const, no-derive_share, local-gate, ungated-json) → 1.1/1.3/1.4/1.5/4.1 + gate steps. §10 citations → re-grepped at plan-write (above).
- **R0 Minors folded:** m1 (`payload_wire_bytes` net-new + `Tag::as_str`) → 1.2; m2 (`inspect::analyze()` `:49` share-branch) → 2.3; m3 (toolkit `friendly_ms_codec` `:80` wildcard arms) → 3.3; m4 (citations) → re-grepped.
- **Open R0-flag for the plan's own R0:** Task 1.4's `combine_shares`→`discriminate` interaction (the recovered secret-at-S has threshold `k`; `discriminate`'s threshold gate must NOT fire on it — factor the prefix-dispatch out of the threshold gate). This is the highest-drift step; the plan R0 must adjudicate the cleanest factoring.
- **No `0x01`, no `derive_share`, no `payload_set`** (the invalidated forms). Tags/publish/merge **authorization-gated** (4.3 Step 3).
