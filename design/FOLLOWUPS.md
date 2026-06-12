# Follow-up tracker

Single source of truth for items that surfaced during a review or implementation pass but were not fixed in the same commit. Mirrors the conventions of the sibling `descriptor-mnemonic` and `mnemonic-key` repos.

## How to use this file

**Format for each entry:**

```markdown
### `audit-2026-06-10-backlog` — verified findings from the first independent Fable constellation audit

- **Surfaced:** 2026-06-10, the 23-agent read-only architecture audit (find → adversarial-verify → synthesize). 48 verified findings constellation-wide (0 critical); this repo's share below. **Full report + per-finding detail (claim/evidence/fix/disposition):** `../../mnemonic-toolkit/design/agent-reports/constellation-architecture-audit-2026-06-10.md` (committed in the toolkit repo). Promote any line to its own `### <id>` entry when worked; resolve here as fixed.
- **This repo's verified findings (3):**
  - **[IMPORTANT] ✓ RESOLVED (ms-codec v0.4.1, 2026-06-10)** `combine-no-length-validation-panic` — promoted to its own entry below; the Entr arm of `dispatch_payload` now `validate()`s (parity with the Mnem arm).
  - **[obs]** `pr2-exposure-claim-verified-sound` — PR#2's padding bug requires reconstructing a share via Codex32String::from_seed from decomposed data+metadata. combine_shares never does this — it parses shares via from_string (shares.rs:184), recove (`crates/ms-codec/src/shares.rs:180-243; crates/ms-codec/tests/codex32_upstream_recovery_regression.rs; crates/ms-codec/tests/spike_kofn.rs:187; crates/ms-codec/src/shares.rs:418`)
  - **[obs]** `recovered-secret-string-not-zeroized` — combine_shares binds `let secret = Codex32String::interpolate_at(...)` (shares.rs:236); Codex32String is a newtype over String (codex32-0.1.0 lib.rs:102 `pub struct Codex32String(String)`) with no Dro (`crates/ms-codec/src/shares.rs:236-242; codex32-0.1.0 lib.rs:102; crates/ms-codec/tests/lint_zeroize_discipline.rs:62-69`)
- **Status:** open (backlog index; individual items dispositioned in the report). 1 of 3 resolved (`combine-no-length-validation-panic`, ms-codec v0.4.1); the two `[obs]` items remain.
- **Tier:** audit-backlog.

### `ms1-envelope-uppercase-bip173` — envelope layer rejects valid all-uppercase ms1 (case-sensitive HRP/share-index compare past codex32)

- **Surfaced:** 2026-06-10, toolkit v0.53.3 HRP-case cycle recon (audit M11). **Companion:** `mnemonic-toolkit/design/FOLLOWUPS.md::hrp-classifier-rejects-valid-uppercase-cards` (resolved there; the toolkit's probes are now case-insensitive and pass the ORIGINAL string through — this entry is the remaining leg).
- **Where:** `crates/ms-codec/src/envelope.rs:100` (`fields.hrp != HRP` — raw compare) and `:112` (`share_index_byte != SHARE_INDEX_V01` — raw `b's'` compare), at ms-codec 0.4.0/0.4.1.
- **What:** codex32 itself accepts consistent-uppercase strings (BIP-173/93: uppercase is the QR alphanumeric-mode form, so engraved/QR'd cards legitimately come back uppercase; the checksum engine case-folds and `set_check_case` rejects only MIXED) — but ms-codec's envelope discrimination then compares the raw HRP/share-index case-sensitively, so a valid all-uppercase MS1 card fails `WrongHrp { got: "MS" }`. Fix: case-normalize the envelope comparisons (lowercase `fields.hrp` and the share-index byte before comparing), keeping mixed-case rejection where codex32 already enforces it. After shipping, the toolkit pin bump cycle must INVERT the staged toolkit characterization cells (they currently pin the WrongHrp/repair-marker ERRORS — a bare pin bump turns them RED, nothing flips green automatically; staged v0.53.3: inspect/repair/silent-payment uppercase-ms1 cells in cli_hrp_case_insensitive.rs).
- **Status:** **resolved** `ms-codec-v0.4.2` (2026-06-10) — wire extraction canonicalizes the owned copy (envelope `wire_string` + inspect + combine's C1(a) canonical vector for codex32's raw cross-share compares). BONUS SECURITY FIX found by the R0 loop: a uniform-uppercase same-id secret-at-`S` set BYPASSED the `SecretShareSuppliedToCombine` guard and `combine_shares` returned the secret payload (raw `b's'` compare missed `b'S'` + interpolation's index-match short-circuit) — guard restored, red-first pinned. 10 new cells. The toolkit pin bump (blocked on crates.io publish of 0.4.1+0.4.2) must INVERT its staged characterization cells (cli_hrp_case_insensitive.rs). Plan + 3 R0 rounds: `design/PLAN_ms1_envelope_uppercase.md`, `design/agent-reports/ms1-uppercase-*.md`.
- **Consumed (2026-06-10):** toolkit **v0.53.5** bumped its ms-codec pin 0.4.0 → 0.4.2 (both now on crates.io) — uppercase ms1 decodes end-to-end there, and `mnemonic ms-shares combine` inherits this entry's combine secret-guard fix (red-first toolkit cell pins the refusal). The staged v0.53.3 characterization cells were inverted.
- **Tier:** `bip173-conformance`

### `combine-no-length-validation-panic` — `ms combine` aborted on a non-standard-length Entr share set (RESOLVED ms-codec v0.4.1)

- **Surfaced:** 2026-06-10 audit (above). **Resolved:** 2026-06-10, ms-codec v0.4.1 (PATCH). Plan + reviews: `design/PLAN_combine_entr_length_validation.md`, `design/agent-reports/combine-entr-length-plan-r0-round{1,2}-review.md`.
- **Companion:** `mnemonic-toolkit/design/FOLLOWUPS.md::toolkit-ms-codec-pin-bump-0-4-1-combine-fix` — the toolkit's `mnemonic ms-shares combine` inherits this fix once its `ms-codec` pin bumps 0.4.0 → 0.4.1 (pending crates.io publish).
- **What:** `dispatch_payload` (`crates/ms-codec/src/envelope.rs`) validated only its `Mnem` arm; the `Entr` (`0x00`) arm returned `Payload::Entr(data[1..].to_vec())` unvalidated. A **valid-checksum but non-standard-length** Entr share set (entropy ∉ {16,20,24,28,32}; constructible directly via codex32 since `encode_shares`/`encode` validate up front) recovered through `combine_shares` then reached `bip39::Mnemonic::from_entropy_in(...).expect(...)` in `ms-cli combine.rs:97` (and the twin in `decode.rs:93`) and **panicked** (abort, exit 101) instead of returning a clean error.
- **Fix:** add `p.validate()?` to the `Entr` arm — parity with the `Mnem` arm AND the function's own doc contract (envelope.rs:155-166 already claimed "then `validate()`"). Closes the gap for ALL `dispatch_payload` callers (single-string `discriminate` + `combine_shares`); both `.expect` invariants become true. Returns the pre-existing `Error::PayloadLengthMismatch` — **no new error variant, no API/wire/flag change.** **Severity:** panic-not-corruption (no funds/wrong-card/secret-leak — the secret never renders). **SemVer PATCH** (ms-codec only; ms-cli binary unchanged, inherits via the bump). Tests: end-to-end `combine_shares` rejection (fixture mirrors `encode_shares`' codex32 construction sans the validate guard) + unit `dispatch_payload` reject/positive-control (TDD: both red pre-fix — `combine_shares` returned `Ok(Entr([..17 bytes]))`). Toolkit `mnemonic ms-shares combine` delegates to the same `combine_shares` → inherits the fix (note filed).
- **Tier:** resolved.

### `<short-id>` — <one-line title>

- **Surfaced:** Phase X.Y review of commit <SHA>, or "inline TODO at <file>:<line>"
- **Where:** `<file>:<line>` or "design — Cargo.toml `[patch]` block"
- **What:** 1–3 sentences describing the gap or improvement opportunity
- **Why deferred:** the reason it didn't ship in the original commit
- **Status:** `open` | `resolved <COMMIT>` | `wont-fix — <one-line reason>`
- **Tier:** `v0.1-blocker` | `v0.1-nice-to-have` | `v0.2` | `cross-repo` | `v1+` | `external`
```

## Tiers (definitions)

- **`v0.1-blocker`**: must fix before tagging `ms-codec-v0.1.0`. Failing to fix = ship blocked.
- **`v0.1-nice-to-have`**: should fix before v0.1 if time permits, but won't block release.
- **`v0.2`**: explicitly deferred to v0.2 (e.g., K-of-N share encoding work).
- **`cross-repo`**: depends on coordination with sibling repos (`descriptor-mnemonic`, `mnemonic-key`, future `mnemonic-toolkit`). Mirrored by a companion entry in the affected sibling's tracker.
- **`v1+`**: deferred indefinitely.
- **`external`**: depends on work outside this repo (e.g., upstream `rust-codex32` PR merging).

---

## Open items

### `ms-codec-no-ci-workflow` — add CI (test + clippy + fmt) for both crates + a one-time fmt normalization

- **Surfaced:** 2026-06-01, ms `mnem` v0.2 cycle (Phase 0 spike + every phase gate).
- **Where:** `.github/workflows/` — the only workflow is `rust.yml`, scoped to `crates/ms-cli/**` and with **no `fmt` step**. `ms-codec` has **no CI at all**.
- **What:** (a) Add a workflow that runs `cargo test --no-fail-fast`, `cargo clippy --all-targets -- -D warnings`, and `cargo +stable fmt --check --all` across **both** crates on push/PR. The `mnem` cycle's only gate was local verification because of this gap. (b) Before the `fmt --check` step can pass, the repo needs a one-time repo-wide normalization: `cargo +stable fmt --all` currently rewrites ~16 pre-existing files (across `ms-codec` and `ms-cli`, drift accumulated from the prior advisory cycle). Land that as a standalone `chore(fmt)` commit FIRST — do **not** bundle it into a feature cycle (the `mnem` cycle deliberately wrote fmt-clean-by-hand and skipped the fmt gate to avoid pulling that churn in).
- **Why deferred:** out of scope for the `mnem` feature; the local gate (full suite + clippy at every phase) was sufficient for this cycle. CI hardening is its own small cycle.
- **Status:** open
- **Tier:** `v0.1-nice-to-have`

### `ms-codec-decode-with-correction-public-api` — promote `decode_with_correction` for downstream BCH consumers

- **Surfaced:** 2026-05-17, mnemonic-toolkit v0.22.0 cycle (BCH error-correction launch).
- **Where:** `crates/ms-codec/src/decode.rs` (new public surface).
- **What:** Add `pub fn decode_with_correction(s: &str) -> Result<(Tag, Payload, Vec<RepairDetail>)>` that internally runs BCH correction within t=4 capacity before the existing decode pipeline. Lets toolkit `repair.rs` consume the sibling-codec native API instead of replicating BCH primitives (codex32-vs-mk-codec polymod-frame translation currently lives in toolkit per the empirical `MS_NUMS_TARGET = 0x962958058f2c192a` derivation).
- **Why deferred:** toolkit v0.22.0 shipped its own primitive consuming mk-codec's promoted BCH internals; adopting a native ms-codec API is a v0.23+ cleanup.
- **Status:** `resolved f3fa531` — v0.22.x follow-ups cycle Phase B.3+B.4: new `bch` module (vendored from md-codec's structure parameterized on `MS_REGULAR_CONST = 0x962958058f2c192a`, byte-exact with toolkit's vendored constant) landed at `676097d` (B.3); new `bch_decode` module (~280 LOC BM+Chien port) + `decode_with_correction(s: &str) -> Result<(Tag, Payload, Vec<CorrectionDetail>), Error>` per Q1 lock + new `Error::TooManyErrors { bound: 8 }` variant + 9 unit cells landed at `f3fa531` (B.4). ms-codec v0.2.0. Toolkit-side consumer migration tracked at `toolkit-repair-consume-native-codec-api`.
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-toolkit` FOLLOWUPS.md `ms-codec-decode-with-correction-public-api`

### `ms-cli-repair-flag` — `ms repair` subcommand mirroring toolkit's `mnemonic repair`

- **Surfaced:** 2026-05-17, mnemonic-toolkit v0.22.0 brainstorm.
- **Where:** `crates/ms-cli/src/cmd/` (NEW subcommand).
- **What:** Add `ms repair <ms1>` for ms1 BCH error-correction (up to 4 substitutions per chunk). Mirrors the toolkit's `mnemonic repair --ms1` subcommand at the per-codec CLI level. Blocked on `ms-codec-decode-with-correction-public-api` (or could vendor toolkit's per-HRP correction primitive).
- **Status:** `resolved 18f558a` — v0.22.x follow-ups cycle Phase B.5: new `ms-cli/src/cmd/repair.rs` with `--ms1 <MS1>` required option (single-chunk per codex32 spec) + `--json` flag + exit-code parity (`0` already valid / `5` REPAIR_APPLIED / `2` unrepairable) + cross-CLI `RepairJson` schema parity (D27) + D9 secret-on-stdout advisory preserved. Wraps `ms_codec::decode_with_correction` (B.4). D25 handler-signature unification cascade (all 5 pre-existing handlers refactored to `Result<u8>` with `Ok(0)` terminators; runtime no-op). 5 integration cells. ms-cli v0.4.0.
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-toolkit` FOLLOWUPS.md `ms-cli-repair-flag`

### `toolkit-repair-consume-native-codec-api` — toolkit-side consumer of native ms-codec correction API

- **Surfaced:** 2026-05-17, mnemonic-toolkit v0.22.0 R1.
- **Where:** cross-repo coordination point; informational mirror in this sibling so the dependency is visible from both sides.
- **What:** Once `ms-codec-decode-with-correction-public-api` lands, toolkit `repair.rs` will switch its ms1 path from the empirical mk-codec-frame primitive call to the native ms-codec API (cleaner layering; one BCH implementation per codec).
- **Status:** `resolved b8ca6df` — v0.22.x follow-ups cycle Phase B.7: toolkit `repair.rs` deleted `MS_NUMS_TARGET` vendored constant + `(Self::Ms1, BchCode::Regular)` arm in `target_residue()`; new `repair_via_ms_codec` private helper delegates to `ms_codec::decode_with_correction` (B.4) with `ms_codec::Error` → `RepairError` translation per plan §2.B.4 D29 table; new `RepairError::PostCorrectionDecodeFailed { chunk_index: Option<usize>, detail: String }` catch-all variant absorbs orphan §4-rule decoder errors. mk1 branch unchanged (mk-codec primitives still consumed natively). mnemonic-toolkit v0.23.0.
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-toolkit` FOLLOWUPS.md `toolkit-repair-consume-native-codec-api`

### `md-codec-decode-with-correction-supports-non-chunked-md1` — sibling-codec consistency mirror

- **Surfaced:** 2026-05-17, v0.22.x follow-ups cycle Phase B.8 (filed after B.6+B.7 surfaced the gap). Informational mirror in mnemonic-secret to keep sibling-codec consumers (and any future ms-codec API extension that takes a similar shape) aware of the cross-codec asymmetry.
- **Where:** Cross-repo coordination point; primary lives at `descriptor-mnemonic/design/FOLLOWUPS.md` `md-codec-decode-with-correction-supports-non-chunked-md1`. ms-codec's `decode_with_correction` is single-chunk by codex32-spec design (HRP `ms` is always single-string `BCH(93,80,8)`), so the constraint asymmetry is structural — md1 is the only HRP family where chunked vs non-chunked is a wire-format distinction.
- **What:** Tracking entry only — when md-codec's `decode_with_correction` gains non-chunked-form coverage, document the cross-codec parity (or explicit non-parity) here so consumers of the ms-codec wrapper understand the structural difference. No ms-codec API change required.
- **Why deferred:** ms-codec scope is unaffected; tracked for cross-codec API surface consistency only.
- **Status:** open
- **Tier:** `cross-repo`
- **Companion:** `bg002h/descriptor-mnemonic` `design/FOLLOWUPS.md` `md-codec-decode-with-correction-supports-non-chunked-md1` (primary).

### `secret-memory-hygiene-v0_9-cycle-a` — cross-repo cycle: OWNED-buffer secret-memory hygiene v0.9.0 Cycle A

- **Surfaced:** 2026-05-13. Cycle SPEC at `mnemonic-toolkit/design/SPEC_secret_memory_hygiene_v0_9_0.md`. Plan at `/home/bcg/.claude/plans/v0_9_0-secret-memory-hygiene.md`. Survey precursor at `mnemonic-toolkit/design/agent-reports/v0_9_0-secret-memory-survey.md`. R1+R2+R3+R4+R5 architect-review disposition at `mnemonic-toolkit/design/agent-reports/v0_9_0-phase-0-spec-plan-r1.md` (5 rounds: Sonnet/Sonnet/Opus/Opus/Sonnet, cleared CLEAR 0C/0I after R3 SPLIT-CYCLE pushback + user decisions).
- **Where:** mnemonic-secret Phase 2 = zeroize discipline in ms-codec + ms-cli. ms-codec scope (4 production OWNED rows): `crates/ms-codec/src/{payload,decode,envelope}.rs` — internal Zeroizing wraps in encode/decode helpers; public `Payload::Entr(Vec<u8>)` shape unchanged (SPEC §3 OOS-public-payload). ms-cli scope (10 OWNED rows incl. 3 clap-field rows): `crates/ms-cli/src/{parse,cmd/encode,cmd/decode,cmd/verify}.rs` + `EncodeArgs.phrase` / `EncodeArgs.hex` / `VerifyArgs.phrase` clap-derived fields via `Zeroizing::new(std::mem::take(...))` pattern. Phase 3 = hygiene matrix file at `design/agent-reports/v0_9_0-secret-memory-hygiene-matrix.md`.
- **What:** This repo's contribution to the v0.9.0 cross-repo OWNED-buffer hygiene cycle. ms-cli has NO Phase 1 argv work (survey §5 marks all 5 ms-cli flag-rows YES — already have stdin route). Closes when the cycle's hygiene-matrix doc lands in this repo (Phase 3) and the patch tags are cut at Phase E (`ms-codec-v0.1.3` + `ms-cli-v0.1.X+1`).
- **Status:** `resolved ab8c73f` — `ms-cli-v0.2.2` tag pushed 2026-05-13 (sibling release commit `b1694e2` for `ms-codec-v0.1.3`). Companion `mnemonic-toolkit-v0.9.2` tag at `9035656` (bg002h/mnemonic-toolkit). All 6 SPEC §6 gates satisfied; cycle B (mlock, toolkit-only) deferred.
- **Tier:** `cross-repo`
- **Companion:** `mnemonic-toolkit/design/FOLLOWUPS.md` — same `secret-memory-hygiene-v0_9-cycle-a` short-id (primary entry). md / mk repos do NOT receive a companion entry this cycle (xpub-only material).

### `secret-memory-hygiene-cycle-b` — cross-repo cycle: mlock infrastructure (Cycle B continuation)

- **Surfaced:** 2026-05-13. Cycle SPEC at `mnemonic-toolkit/design/SPEC_secret_memory_hygiene_v0_9_B.md`. Reviewer-loop CLEAR 0C/0I across R1+R2 (`mnemonic-toolkit/design/agent-reports/v0_9_B-phase-0-spec-r1.md` + `...-r2.md`). Companion FOLLOWUP `cycle-b-pre-spec-questions` in `mnemonic-toolkit/design/FOLLOWUPS.md` captures the 4 pre-SPEC questions + 5 brainstorming-session questions resolved at SPEC drafting.
- **Where:** mnemonic-secret Phase 3b at `87965b6` (~40 LOC). New module `crates/ms-cli/src/mlock.rs` carrying an inline copy of the slice fn `pin_pages_for(&[u8]) -> PinnedPageRange` + `PinnedPageRange + Drop` + `MlockState` (process-local) + `report_at_exit()`. Apply-site: `crates/ms-cli/src/parse.rs:65` — `pin_pages_for(s.as_bytes())` after `read_stdin()` returns its `String` (site #5 per Cycle B SPEC §2 row 5; post Cycle A `Zeroizing<String>` shift). `MlockedZeroizing<T>` wrapper was retired in Phase 2 R0 (Fix B); slice-fn primitive is the only API surface. PE.T2 adds the first ms-cli Rust CI workflow at `6a1dad6`; PE.T3 adds the SPEC §6 G6 invariant test mirror at `tests/mlock_g6_invariant.rs`.
- **What:** Cycle B continues v0.9.0 Cycle A's secret-memory hygiene work. Cycle A added Zeroizing-on-Drop discipline to OWNED secret buffers; Cycle B layers `mlock(2)` page-pinning on top (POSIX-only — Linux + macOS; Windows VirtualLock deferred per SPEC §3 `OOS-windows-virtuallock`). Cycle B is cross-repo: toolkit handles sites 1-4 (clap args + ResolvedSlot.entropy + DerivedAccount.entropy + bip85 [u8;64] heap-promoted in Phase 1), ms-cli handles site #5. Inline-copy invariant (Cycle B SPEC §6 G6) is CI-enforced by `tests/mlock_g6_invariant.rs` in both repos — normalized source byte-equality + 14-item MANIFEST name-export parity.
- **Why deferred from Cycle A:** R3 SPLIT-CYCLE finding from Cycle A Phase 0 — combining mlock with Zeroizing would have doubled Cycle A's review surface; splitting keeps each cycle's blast radius reviewable.
- **Status:** `resolved 2e7c275` — `ms-cli-v0.3.0` tag pushed 2026-05-13. Companion lockstep tag: `mnemonic-toolkit-v0.10.0` (mnemonic-toolkit `9f63e8e`). All 7 SPEC §6 gates satisfied (G1 functional / G2 soft-fail / G3 platform / G4.a Cycle A Drop preserved + G4.b Miri / G5 lockstep tags / G6 inline-copy invariant test / G7 wire-format unchanged). Cycle-close artifacts: cross-repo audit matrix at `mnemonic-toolkit/design/agent-reports/v0_9_B-secret-memory-hygiene-matrix.md`; PE R0 report at `mnemonic-toolkit/design/agent-reports/v0_9_B-PE-r0.md`.
- **Tier:** `cross-repo`
- **Companion:** `mnemonic-toolkit/design/FOLLOWUPS.md` — same `secret-memory-hygiene-cycle-b` short-id (primary tracker entry). md / mk repos do NOT receive a companion entry this cycle (xpub-only material per Cycle A `OOS-md-mk` class).

### `ms-codec-payload-zeroize-public-api` — widen `Payload::Entr(Vec<u8>)` to `Payload::Entr(Zeroizing<Vec<u8>>)` (breaking)

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 `OOS-public-payload` class.
- **Where:** `crates/ms-codec/src/payload.rs:16-30` — `pub enum Payload { Entr(Vec<u8>), ... }`. The public-API shape is `Vec<u8>`; Cycle A added a caller-wrap-contract doc-comment but did not change the type.
- **What:** Wrapping the public variant in `Zeroizing<Vec<u8>>` (or adding `impl Drop for Payload` to scrub on drop) would give scrub-on-drop semantics to the public-API surface but is a breaking change for external library consumers. Adding `impl Drop` blocks move-out destructuring patterns (`let Payload::Entr(v) = payload` move) per Rust E0509. Cycle A keeps `Payload::Entr(Vec<u8>)` shape AND no Drop impl on Payload; internal callers in `ms-codec` are tightened to use Zeroizing *behind* the public surface (encode/decode helpers' intermediate buffers); the public variant continues to be caller-managed (callers responsible for Zeroizing-wrapping the returned Vec).
- **Why deferred:** Breaking change for external library consumers. A future cycle can decide to break the API for a hardened `Payload`.
- **Status:** `open`
- **Tier:** `v1+`

### `ms-codec-doc-example-zeroize-consistency` — apply Zeroizing in `ms-codec` `lib.rs` doc-example for pattern consistency

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 `OOS-7` class (Phase 0 R1 I-1 fold).
- **Where:** `crates/ms-codec/src/lib.rs:18-19,29-30` — the public doc-test example carrying a literal entropy value.
- **What:** The doc-test example uses a synthetic vector chosen for documentation, not real secret material. Wrapping it in Zeroizing would add visual noise to the public API's documentation example without any security benefit (the literal is plaintext in the source anyway). Optional future cycle could apply Zeroizing for pattern-consistency reasons.
- **Why deferred:** No security benefit; consistency-only. Doc-tests are not production secret material.
- **Status:** `open`
- **Tier:** `v1+`

### `ms-cli-decode-emit-zeroize-intermediate` — Zeroize the `emit_json`/`emit_text` intermediate String in ms-cli decode

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 `OOS-decode-stdout` class (Phase 0 R1 C-2 fold — OWNED-row counting).
- **Where:** `crates/ms-cli/src/cmd/decode.rs:67-94` — the `emit_json` / `emit_text` paths.
- **What:** These paths are primarily STDOUT-LEAK: the values go to stdout by design (that is the command's purpose). Wrapping the intermediate `String` before flush is theoretically possible but adds machinery for zero practical benefit — the entropy and phrase land on stdout one syscall later. Optional future cycle could apply Zeroizing for pattern-consistency reasons.
- **Why deferred:** No practical benefit; values are emitted to stdout by design.
- **Status:** `open`
- **Tier:** `v1+`

### `rust-codex32-zeroize-upstream` — `codex32::Codex32String` internal payload buffer has no `Zeroize`

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 2 ms-codec envelope work — surfaced while landing the Zeroizing<Vec<u8>> local in `envelope::package`.
- **Where:** Upstream crate `codex32 = "0.1"` (the `rust-codex32` repo). Affects `crates/ms-codec/src/envelope.rs::package` — `Codex32String::from_seed` copies payload bytes into its private buffer during construction; those bytes live for the `Codex32String`'s lifetime (extends until the caller's binding drops).
- **What:** `envelope::package`'s `Zeroizing<Vec<u8>>` local scrubs the `data` buffer when the function exits, but the bytes that `Codex32String::from_seed` copied into its private buffer during construction are NOT scrubbed. Mitigation is lifetime minimization at the ms-codec layer + caller-wrap discipline. Closes when upstream `rust-codex32` adds `impl Drop` + Zeroize on `Codex32String` (or when ms-codec migrates to an internally-controlled codex32 implementation).
- **Status:** `open` (upstream-blocked) — **NOTE (2026-06-09): the upstream crate is effectively DORMANT** (see `rust-codex32-upstream-pr2-recovery-bug-not-exposed` + `codex32-upstream-dormant-vendor-vs-accept-decision`), so "closes when upstream adds Zeroize" is unlikely to ever fire; the realistic close path is vendor/fork or accept-the-mitigation.
- **Tier:** `external`

### `rust-codex32-upstream-pr2-recovery-bug-not-exposed` — upstream codex32 PR #2 (shamir recover padding bug); our path verified unexposed + guarded

- **Surfaced:** 2026-06-09, constellation backup/safety gap audit — upstream-repo check on the pinned `codex32 = "=0.1.0"` dep.
- **Where:** Upstream `apoelstra/rust-codex32` **PR #2** (scgbckbone, opened 2026-... [Dec 5 2025], updated Apr 16 2026, **unmerged**): "Serialization to seed & subsequent re-serialization to shares breaks shamir recover result." Root cause: padding — reconstructing a share via `Codex32String::from_seed` from decomposed `data`+metadata recovers a WRONG secret (last-nibble flip on a 16-byte/128-bit secret, `…4979 9` vs `…4979 f`).
- **What:** **Our pipeline is NOT exposed.** `ms_codec::shares::combine_shares` recovers via `Codex32String::interpolate_at` over the parsed share STRINGS (`shares.rs:236`), never the decompose-to-`data` → `from_seed` reload the bug requires — we carry the full codex32 share string end-to-end. Verified two ways: (1) structural (combine never calls `from_seed`); (2) empirical — `ms split`→`ms combine` of PR#2's exact 16-byte secret recovers it correctly across all 2-of-3 pairs; broad cross-length coverage in `tests/spike_kofn.rs` (claim b) + `shares.rs::combine_round_trip_entr_and_mnem_all_lengths` is GREEN. Added a NAMED regression anchor: `crates/ms-codec/tests/codex32_upstream_recovery_regression.rs` (pins PR#2's exact secret round-tripping correctly — fails loudly with a pointer if a future `codex32` bump reintroduces the bug on our path).
- **Status:** `resolved` (our exposure: NONE, verified + guarded). The upstream PR remains open on a dormant crate; we track the dependency decision separately.
- **Tier:** `external` (upstream-tracking; no action required on our side).

### `codex32-upstream-dormant-vendor-vs-accept-decision` — the pinned codex32 crate is abandoned; decide vendor/fork vs accept

- **Surfaced:** 2026-06-09, same audit.
- **Where:** dep `codex32 = "=0.1.0"` (crates.io; source `apoelstra/rust-codex32`). Frozen at **0.1.0 since 2023-03-10**; maintainer note: "as of July 2023 the library is slated to be largely rewritten… may not be worthwhile to improve it until that rewrite arrives" — that rewrite never shipped (3 years on). crates.io carries no repository link. 0 open issues, 1 open PR (the recovery bug above).
- **What:** Both codex32-upstream items (`rust-codex32-zeroize-upstream` + PR#2 above) can never close via an upstream RELEASE — the upstream is dormant. For a steel-backup-of-funds tool sitting on a dormant secret-sharing dep, the dependency posture is a deliberate decision worth making rather than drifting: **(a) accept** (we're unexposed to the recovery bug; the zeroize gap has a working lifetime-minimization mitigation), or **(b) vendor/fork** the crate (own the fixes — adds Zeroize/Drop, de-risks the dormant dep, but takes on maintenance of the BCH/Shamir primitives). Pre-decision: keep the `=0.1.0` exact-pin (no surprise bumps) + the spike/named guards (catch any future bump that breaks invariants).
- **Status:** `open` (strategic decision, no urgency — we are not exposed to a live bug).
- **Tier:** `external`

### `md-mk-private-key-surface-watch` — reopen md/mk Cycle A participation if either repo grows a private-key surface

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 `OOS-md-mk` class.
- **Where:** `descriptor-mnemonic` repo (md-codec + md-cli) and `mnemonic-key` repo (mk-codec + mk-cli). Currently both hold xpub-only / descriptor-only material with no private-key buffer.
- **What:** Cycle A drops the no-scope-symmetry matrix stubs originally planned for md/mk repos because they have no secret material to audit. If either repo later gains a private-key surface (e.g., a future md-codec descriptor-binding with embedded xprv, or an mk-codec xprv passthrough), this FOLLOWUP fires and Cycle A's hygiene discipline (Zeroizing + SAFETY anchors + matrix delta) reopens for the affected sibling.
- **Why deferred:** No secret material to audit today.
- **Status:** `open` (monitoring)
- **Tier:** `cross-repo`
- **Companion:** `mnemonic-toolkit/design/FOLLOWUPS.md` (primary tracker), `descriptor-mnemonic/design/FOLLOWUPS.md`, `mnemonic-key/design/FOLLOWUPS.md` — same `md-mk-private-key-surface-watch` short-id.

### `bip-vector-adoption-v0_8` — cross-repo cycle: BIP-vector adoption v0.8.0

- **Surfaced:** 2026-05-13. Cycle SPEC at `mnemonic-toolkit/design/SPEC_test_vector_audit_v0_8_0.md`. Plan at `/home/bcg/.claude/plans/v0_8_0-bip-vector-adoption.md`. R1 review at `mnemonic-toolkit/design/agent-reports/v0_8_0-phase-0-spec-plan-r1.md`.
- **Where:** mnemonic-secret Phase 2 = BIP-93 inline corpus adoption in `crates/ms-codec/tests/bip93_inline_vectors.rs` (+5 valid cells + 1 parametric cell asserting all 64 BIP-93 §Invalid entries are rejected by `rust-codex32 =0.1.0`).
- **What:** This repo's contribution to the v0.8.0 cross-repo vectors-only patch cycle. Closes when the cycle's audit-matrix successor doc lands at `design/agent-reports/v0_8_0-bip-test-vector-audit-matrix.md` (Phase 4) and the patch tag is cut at Phase E. The v0.7.1 matrix's footnote of "42 invalid strings" was corrected to 64 at Phase 0 via `gh api` count of the live BIP-93 §Invalid `<code>`-bullet list.
- **Status:** `resolved 527c9c7` — ms-codec-v0.1.2 tag pushed; cycle close PR #7 merged. Companion sibling-repo tags: md-codec-v0.32.1 (descriptor-mnemonic ef00e07), mnemonic-toolkit-v0.9.1 (f036737).
- **Tier:** `cross-repo`
- **Companion:** `mnemonic-toolkit/design/FOLLOWUPS.md`, `descriptor-mnemonic/design/FOLLOWUPS.md`, `mnemonic-key/design/FOLLOWUPS.md` — same `bip-vector-adoption-v0_8` short-id in each.

### `bip93-invalid-corpus-granular-error-pin` — BIP-93 §Invalid per-vector error-variant classification deferred

- **Surfaced:** 2026-05-13, v0.8.0 Phase 2 design. File-level doc-comment in `tests/bip93_inline_vectors.rs` records the deferral inline.
- **Where:** `crates/ms-codec/tests/bip93_inline_vectors.rs` — the parametric `all_invalid_vectors_rejected_by_codex32` test asserts only `is_err()`, not which `codex32::Error` variant.
- **What:** `rust-codex32 =0.1.0`'s `Error` enum is granular enough to distinguish bad-checksum vs invalid-char vs length-violation vs mixed-case. The BIP-93 §Invalid section, however, only says "These examples have incorrect checksums" and does not categorize each of the 64 entries. Pinning the variant per entry would amount to pinning `rust-codex32`'s internal classification rather than a BIP-published claim. Resolution path: classify each invalid vector by inspection (truncated HRP / mixed case / bad checksum / etc.) and assert the matching variant; tightens the test against a `rust-codex32` re-classification on a future bump.
- **Status:** `open` (coarse `is_err()` shipped at v0.8.0; granular pin is a future tightening).
- **Tier:** `v1+`
- **Companion:** None (single-repo concern).

### `manual-cli-surface-mirror` — ms-cli flag/API changes must mirror to the toolkit-side user manual

- **Surfaced:** 2026-05-07, m-format-star user manual v0.1 release in `bg002h/mnemonic-toolkit` (`manual-v0.1.0` tag; toolkit PR #1).
- **Where:** Cross-repo coordination only; no ms-codec / ms-cli source change required at filing time. Future ms-cli flag additions must touch `mnemonic-toolkit/docs/manual/src/40-cli-reference/43-ms.md` in lockstep.
- **What:** v0.1 of the m-format-star user manual lives in the `mnemonic-toolkit` repo and mirrors `ms-cli`'s 5 subcommands verbatim against ms-codec v0.1.1 / ms-cli v0.1.0. The manual's `tests/lint.sh flag-coverage` CI step parses `--help` output for each `<binary, subcommand>` pair and asserts each flag appears in the manual chapter. Adding or removing a flag in `ms-cli` without updating the manual will fail the manual-side CI on the next push to `docs/manual/`. **Companion:** primary entry `manual-cli-surface-mirror` in `mnemonic-toolkit/design/FOLLOWUPS.md`; sibling companions in `descriptor-mnemonic/design/FOLLOWUPS.md` and `mnemonic-key/design/FOLLOWUPS.md`.
- **Why filed:** the manual is a separate artifact (its own `manual-v*` versioning); without an explicit mirror invariant, sibling-side flag changes would silently drift the manual.
- **Status:** `open` (mirror invariant active for the lifetime of `mnemonic-toolkit/docs/manual/`)
- **Tier:** `cross-repo`

### `phase-2-3-low-1` — envelope.rs defensive empty-payload arm yields misleading error variant

- **Surfaced:** Phase 2+3 review r1 (`design/agent-reports/phase-2-3-envelope-encode-decode-review-r1.md` low-1).
- **Where:** `crates/ms-codec/src/envelope.rs:108` (the `payload_with_prefix.is_empty()` defensive arm).
- **What:** Returns `Error::ReservedPrefixViolation { got: 0 }`, but `got: 0` is what a *valid* prefix byte looks like — confusing in logs. Unreachable for valid v0.1 strings (rule 9 length check guarantees payload non-empty), but the code path exists for direct envelope-internal calls. Consider `Error::UnexpectedStringLength` or a dedicated invariant-broken variant.
- **Why deferred:** unreachable in practice; cosmetic-only diagnostic improvement.
- **Status:** `resolved 2026-05-03 — defensive empty-check removed entirely. Reasoning documented inline: any string that passed extract_wire_fields (≥sep+20 chars) and Codex32String::from_string (≥48 chars for short codex32) yields a payload of ≥26 codex32 symbols ≈ 16 raw bytes, so payload cannot be empty.`
- **Tier:** `v0.1-nice-to-have`

### `phase-2-3-low-2` — extract_wire_fields length-check arithmetic is cryptic

- **Surfaced:** Phase 2+3 review r1 (low-2).
- **Where:** `crates/ms-codec/src/envelope.rs::extract_wire_fields` length-check expression.
- **What:** `s.len() < sep + PAYLOAD_START_OFFSET + CHECKSUM_LEN_SHORT` is correct but reads cryptically. A comment "minimum sep+20 for any v0.1-shaped string" or refactor against `VALID_STR_LENGTHS.iter().min()` would aid readability.
- **Why deferred:** stylistic.
- **Status:** `resolved 2026-05-03 — added explanatory comment "fixed wire prefix after sep is 7 chars (threshold + 4-char id + share-index) + 13-char short checksum = 20" above the length check.`
- **Tier:** `v0.1-nice-to-have`

### `phase-1-low-1` — `Tag::try_new` wrong-length branch produces noisy diagnostic bytes

- **Surfaced:** Phase 1 review r1 (`design/agent-reports/phase-1-foundation-review-r1.md` low-1).
- **Where:** `crates/ms-codec/src/tag.rs:33-38`.
- **What:** The wrong-length branch reconstructs partial input bytes via `bytes.first().copied().unwrap_or(0)` etc., but those bytes carry no diagnostic value when `len != 4`. Could just return `Error::TagInvalidAlphabet { got: [0; 4] }`.
- **Why deferred:** cosmetic; tests assert variant only, not bytes.
- **Status:** `resolved 2026-05-03 — simplified to Err(Error::TagInvalidAlphabet { got: [0; 4] }) with explanatory comment.`
- **Tier:** `v0.1-nice-to-have`

### `phase-1-low-2` — `Error::Codex32` Display uses `{:?}` on inner

- **Surfaced:** Phase 1 review r1 (low-2).
- **Where:** `crates/ms-codec/src/error.rs::Display::fmt` Codex32 arm.
- **What:** `codex32::Error` doesn't impl Display in v0.1.0. If a future `codex32` patch adds Display, switch from `{:?}` to `{}` for user-facing messages.
- **Why deferred:** dependent on upstream change.
- **Status:** `open`
- **Tier:** `external`

### `phase-1-low-3` — `consts.rs` ceil-div could use `usize::div_ceil`

- **Surfaced:** Phase 1 review r1 (low-3).
- **Where:** `crates/ms-codec/src/consts.rs::tests` bijection test.
- **What:** `(data_bits + 4) / 5` is the standard ceil-div idiom; stable `usize::div_ceil` (Rust 1.73+) is more readable. MSRV 1.85 supports it.
- **Why deferred:** cosmetic.
- **Status:** `resolved 2026-05-03 — switched to data_bits.div_ceil(5).`
- **Tier:** `v0.1-nice-to-have`

### `phase-1-low-5` — `Error::source()` returns `None` always

- **Surfaced:** Phase 1 review r1 (low-5).
- **Where:** `crates/ms-codec/src/error.rs::std::error::Error::source`.
- **What:** Correct given `codex32::Error` lacks `std::error::Error` impl in v0.1.0. If a future `codex32` patch adds the impl, change `Codex32` arm to `Some(e)`. Tracked alongside the parallel `external`-tier note in SPEC §10.1.
- **Why deferred:** dependent on upstream change.
- **Status:** `open`
- **Tier:** `external`

### `plan-r2-nit-followups-slug-format` — Phase 1 Task 1.7 nit-format snippet uses `\`slug\`` heading style

- **Surfaced:** IMPLEMENTATION_PLAN review r1 (2026-05-03; finding nit #1).
- **Where:** `design/IMPLEMENTATION_PLAN_ms_v0_1.md` Phase 1 Task 1.7 Step 4 (FOLLOWUPS entry template).
- **What:** The template uses `### \`phase-1-low-N\`` heading. Other entries in this repo's FOLLOWUPS use kebab-case slugs without backticks. Cosmetic; verify against this file's existing entries' header style and adjust the template before Phase 1 review fires.
- **Why deferred:** template-only; doesn't affect implementation correctness.
- **Status:** `resolved 2026-05-03 — plan template updated to plain kebab-case slug heading (no backticks) per the actual style used by all real entries in this file.`
- **Tier:** `v0.1-nice-to-have`

### `plan-r2-nit-readme-step-granularity` — Phase 7 Task 7.5 README rewrite is one chunky step

- **Surfaced:** IMPLEMENTATION_PLAN review r1 (2026-05-03; finding nit #3).
- **Where:** `design/IMPLEMENTATION_PLAN_ms_v0_1.md` Phase 7 Task 7.5.
- **What:** writing-plans skill recommends 2-5 minutes per step; the README rewrite is a single ~80-line step. Consider splitting into "draft README content" + "verify links" sub-steps for cleaner progress tracking.
- **Why deferred:** cosmetic; doesn't affect content quality.
- **Status:** `wont-fix 2026-05-03 — the plan is now historical (used to drive the implementation, won't be re-executed). Splitting steps post-execution would be churn without value. Future plans should observe the 2-5-minute granularity guideline at draft time.`
- **Tier:** `v0.1-nice-to-have`

### `plan-r2-nit-rule2-comment-wording` — Phase 5 rule_2 test comment wording

- **Surfaced:** IMPLEMENTATION_PLAN review r1 (2026-05-03; finding nit #4).
- **Where:** `design/IMPLEMENTATION_PLAN_ms_v0_1.md` Phase 5 Task 5.1 `tests/negative.rs` rule_2 test (build_with HRP "mq").
- **What:** The "Note:" comment reads as if SPEC §4 mandates rule-9-before-rule-1 ordering. SPEC §4 numbers rules but doesn't strictly mandate check-order; the implementation chose rule 9 first as a defensive optimization. Reword to "implementation choice" not "SPEC mandate."
- **Why deferred:** cosmetic; doesn't affect test behavior.
- **Status:** `resolved 2026-05-03 — comment in tests/negative.rs rule_2 test reworded to clarify rule 9 ordering is an implementation choice / defensive optimization, not a SPEC requirement.`
- **Tier:** `v0.1-nice-to-have`

### `plan-r2-nit-consts-naming-style` — `consts.rs` mixes naming/value-style conventions

- **Surfaced:** IMPLEMENTATION_PLAN review r1 (2026-05-03; finding nit #5).
- **Where:** `design/IMPLEMENTATION_PLAN_ms_v0_1.md` Phase 1 Task 1.2 Step 3 (`crates/ms-codec/src/consts.rs`).
- **What:** Three naming conventions in one file: `THRESHOLD_V01: u8 = b'0'` (ASCII byte literal), `SHARE_INDEX_V01: u8 = b's'` (ASCII byte literal), `RESERVED_PREFIX: u8 = 0x00` (hex literal). Reviewer-flaggable but not behaviorally significant. Pick one convention or document why each chose its form.
- **Why deferred:** cosmetic; doesn't affect code behavior.
- **Status:** `resolved 2026-05-03 — added a Naming-convention paragraph to the consts.rs module-level doc-comment explaining the rule (ASCII byte literals for character semantics, hex literals for byte semantics; both produce u8).`
- **Tier:** `v0.1-nice-to-have`

### `ms-cli-v01-spec-r2-nit-1` — §2.4.1 verify validation-order prose clarity

- **Surfaced:** SPEC_ms_cli_v0_1 review r2 (in-conversation; 2026-05-04).
- **Where:** `design/SPEC_ms_cli_v0_1.md` §2.4.1 step 2 prose.
- **What:** "ms1-side error first" framing reads as severity-ordering when it actually means "before phrase parsing." Add a one-line clarification at draft time of the IMPLEMENTATION_PLAN or in a SPEC patch.
- **Why deferred:** cosmetic; impl is unambiguous from §6.1.1 dispatch table.
- **Status:** `resolved 2026-05-04 — §2.4.1 prose clarified inline at user request: "first" explicitly means "earlier in validation pipeline" not severity tier.`
- **Tier:** `v0.1-nice-to-have`

### `ms-cli-v01-spec-r2-nit-3` — §2.3.1 inspect cannot route exit 3 for future-format strings

- **Surfaced:** SPEC_ms_cli_v0_1 review r2.
- **Where:** `design/SPEC_ms_cli_v0_1.md` §2.3.1.
- **What:** Inspect on a string that fails BIP-93 parse (e.g., long-checksum framing that's actually a future v0.2+ string) returns exit 1, not exit 3. Only `verify` post-decode can route exit 3. Add a one-line acknowledgement to §2.3.1.
- **Why deferred:** correctness is unaffected; users discover this via inspect's `failure_reasons` field.
- **Status:** `resolved 2026-05-04 — §2.3.1 gains explicit "Note on exit-3 routing" paragraph at user request.`
- **Tier:** `v0.1-nice-to-have`

### `ms-cli-v01-spec-r2-nit-4` — Per-subcommand clap `about` / `after_long_help` strings unspecified

- **Surfaced:** SPEC_ms_cli_v0_1 review r2.
- **Where:** SPEC §2 (commands) + future IMPLEMENTATION_PLAN.
- **What:** SPEC doesn't pin the `--help` output text per subcommand. md-cli precedent (`crates/md-cli/src/main.rs:50, 59, 95, 144`) uses `after_long_help = "EXAMPLES:..."`. The IMPLEMENTATION_PLAN should write per-subcommand `about` + `after_long_help` strings and SPEC §2.6 should reference them.
- **Why deferred:** mechanical fill-in at IMPLEMENTATION_PLAN draft time.
- **Status:** `resolved 2026-05-04 — new §2.6 added at user request: locks `about` + `after_long_help` strings for all 5 subcommands with concrete EXAMPLES blocks.`
- **Tier:** `v0.1-nice-to-have`

### `ms-cli-v01-spec-r2-nit-6` — JSON object key ordering not pinned

- **Surfaced:** SPEC_ms_cli_v0_1 review r2.
- **Where:** SPEC §5.
- **What:** `serde_json` preserves struct-field declaration order, but the SPEC doesn't pin this as a stability guarantee. Tools that diff outputs care. Add one sentence: "JSON object key order is the schema-declaration order (struct field order); stable across v0.1.x."
- **Why deferred:** convention rather than requirement; impl observably stable.
- **Status:** `resolved 2026-05-04 — §5 preamble adds the stability note at user request.`
- **Tier:** `v0.1-nice-to-have`

### `ms-cli-v01-spec-r2-nit-7` — Encoder edge-case enumeration in §2.1

- **Surfaced:** SPEC_ms_cli_v0_1 review r2.
- **Where:** SPEC §2.1 "Encoder pre-checks".
- **What:** `--phrase ""`, `--phrase " "`, `--hex ""`, `--hex "ZZ"` produce specific errors but aren't enumerated. All hit exit 1 (Bip39 BadWordCount / Bip39 BadWordCount / PayloadLengthMismatch / BadInput). Adding the enumeration removes test-surface ambiguity.
- **Why deferred:** behaviors are unambiguous; spec can be tightened at IMPLEMENTATION_PLAN time when test fixtures are written.
- **Status:** `resolved 2026-05-04 — §2.1 "Encoder pre-checks" gains a 10-row edge-case table at user request: empty/whitespace/short/odd/non-hex/conflict/missing inputs each map to specific CliError + exit code.`
- **Tier:** `v0.1-nice-to-have`

### `ms-cli-v01-plan-r2-nit-N1` — `verify --phrase` uses single `args.language` for both supplied phrase parse + entropy re-derivation

- **Surfaced:** IMPLEMENTATION_PLAN_ms_cli_v0_1 review r2 (in-conversation; 2026-05-04).
- **Where:** `cmd/verify.rs::run` (per IMPLEMENTATION_PLAN Phase 2 Task 2.5).
- **What:** Round-trip check parses supplied phrase with `args.language` AND re-derives mnemonic from decoded entropy with `args.language` — so the comparison happens in language space rather than entropy space. If a user originally encoded with English but supplied `--language french` along with a French phrase, both `parse_in(French)` succeeds (assuming the French phrase has valid checksum) and `from_entropy_in(French)` produces a French mnemonic; the comparison agrees if the user's French phrase happens to encode the same entropy. This is semantically correct (verifies the language-and-phrase tuple round-trips) but doesn't catch "user encoded with English, recorded the French translation, supplied the French translation at verify time" — which would round-trip OK under French even though the originally-engraved card was English-derived. SPEC §6.3 hazard surfaces this orthogonally via the language warning; verify could surface a stronger warning when args.language differs from the encoder's claimed language at engrave time, but ms1 v0.1 wire format doesn't carry that.
- **Why deferred:** correctness for the documented use case; the failure mode requires a language change between encode and verify which is itself an inconsistency the user should have caught at the SPEC §6.3 hazard surface.
- **Status:** `open`
- **Tier:** `v0.1-nice-to-have`

### `ms-cli-v01-plan-r2-nit-N3` — `parse_hex_entropy` defers length-set validation to `ms_codec::encode`

- **Surfaced:** IMPLEMENTATION_PLAN_ms_cli_v0_1 review r2 (2026-05-04).
- **Where:** `cmd/encode.rs::parse_hex_entropy` (per IMPLEMENTATION_PLAN Phase 2 Task 2.2).
- **What:** Hex like `--hex 0011223344` (5 bytes) passes hex parse and is handed to `ms_codec::encode`, which rejects with `PayloadLengthMismatch` (mapped to CliError::PayloadLengthMismatch — exit 1). User sees `tag "entr" payload length 5 not in expected set [16, 20, 24, 28, 32]`. Functionally correct but the message wording comes from ms-codec rather than a hex-specific pre-check. SPEC §2.1 edge-case row would prefer "hex decodes to 5 bytes; expected 16/20/24/28/32" wording.
- **Why deferred:** Functionally identical exit code + similar message; cosmetic improvement only.
- **Status:** `open`
- **Tier:** `v0.1-nice-to-have`

### `ms1-v01-payload-bracket-overflow-prefix-byte-incompatibility` — v0.1 `0x00`-prefix-byte design overflows BIP-93 codex32's long-code length bracket for `seed` / `xprv` payloads

- **Surfaced:** 2026-05-03 pre-SPEC spike against `rust-codex32 = "=0.1.0"` (in conversation; before SPEC drafted). Companion mirrors: same-id entry in `mnemonic-key/design/FOLLOWUPS.md` and `descriptor-mnemonic/design/FOLLOWUPS.md`, both at tier `cross-repo`.
- **Where:** SPEC (not yet drafted), `BRAINSTORM_ms_v0_1.md` Q4 closure (locks `seed`/`entr`/`xprv` payload set), `MIGRATION.md` invariant 1 (locks the `0x00` reserved-prefix byte), and the meta-plan `/home/bcg/.claude/plans/c-ultimately-what-we-quirky-avalanche.md` §"ms-codec v0.1 architecture" / §"v0.2 migration seam" / §"RESERVED_TAG_TABLE".
- **What:** BIP-93 codex32 (per the BIP itself, and as implemented in `rust-codex32 = "=0.1.0"`) accepts only two specific length brackets — short (raw payload 16-44 B) and long (raw payload 63-64 B). The locked v0.1 wire format prepends a `0x00` reserved-prefix byte to the raw secret to enable the v0.2 non-breaking migration; this pushes a 64-B BIP-32 master seed to a 65-B effective payload (128-char string, one past the long-bracket max of 127). Empirical spike (encode→decode against `rust-codex32 v0.1.0` over data sizes 60..82) confirmed: encoder produces a string the decoder rejects with `InvalidLength` for every size outside {16-44, 63-64} bytes. `xprv` (78 B) was never inside any BIP-93 bracket, with or without the prefix. Three locked decisions interact (payload set {seed, entr, xprv} + `0x00` reserved-prefix byte + exact-pin `=0.1.0` no-fork), but at most two are simultaneously satisfiable.
- **Why deferred:** Surfaces SPEC-blocker *before* the SPEC is drafted; cannot be deferred. Logged here so future sessions / sibling-repo readers see the discovery provenance once a remediation lands. Active candidates (in conversation): (A) drop `seed`/`xprv`; v0.1 = `entr` only — strongest fit given the engraving thesis. (B) drop the `0x00` prefix; v0.1 uses `id` as sole discriminator and the v0.2 migration loses the non-breaking-for-v0.1-strings property. (C) vendor/fork `rust-codex32` with a wider long-code — requires re-deriving BCH parameters, much heavier than originally framed.
- **Workflow lesson:** the plan-mode r1..r5 reviewer loop did logical/architectural review without an execute-encode/decode-against-locked-deps spike. Five rounds missed the issue. Future wire-format plans riding on locked external deps should include an explicit "verify round-trip against the actual pinned dep before locking the plan" step, parallel to the existing `audit_before_extending` memory entry.
- **Status:** `resolved 2026-05-03 — Option A locked + shipped in ms-codec v0.1.0 (tag ab374ed). v0.1 narrowed to entr-only; seed/xprv reserved-not-emitted with decoder rejection (Error::ReservedTagNotEmittedInV01) and encoder symmetry (SPEC §3.5.1). 50 tests passing including the forward-compat 1..=255 prefix-byte sweep that locks the v0.2-migration contract from day 1. BIP-32 master seed backup use case preserved via the BIP-39 phrase → entropy → ms1 entr → engrave → recover → BIP-39 mnemonic → PBKDF2 routing in SPEC §1.2 / README. Cross-repo mirrors in mk1 + md1 closed in lockstep.`
- **Tier:** `v0.1-blocker`

### `ms-kofn-json-wire-shape-ungated` — `ms split`/`combine`/`inspect`-share + `mnemonic ms-shares` `--json` wire-shapes (and the `--to` value-enum) are NOT schema_mirror-gated

- **Surfaced:** 2026-06-03, ms K-of-N v0.2 cycle Phase 4 (Task 4.2c) — ms-codec 0.4.0 / ms-cli v0.7.0 / mnemonic-toolkit v0.40.0.
- **Where:** `crates/ms-cli/src/cmd/{split.rs,combine.rs,inspect.rs}` (the `--json` emit paths); toolkit `crates/mnemonic-toolkit/src/cmd/ms_shares.rs` (`split`/`combine` `--json` emit). GUI mirror `mnemonic-gui/src/schema/{ms.rs,mnemonic.rs}` (the consumer of the *flag-name* projection).
- **What:** The new K-of-N surface adds `--json` output objects that downstream GUI consumers may parse: `ms split --json` → `{ shares, k, n, id, kind, language? }`; `ms combine --json` → the recovered-secret object; `ms inspect --json` of a share → `{ kind: "share", threshold, id, index }` (with the payload-kind/`prefix_byte` fields suppressed); `mnemonic ms-shares split --json` → `{ "shares": [...] }`; `mnemonic ms-shares combine --json` → the recovered-secret object. The `schema_mirror` gate (`mnemonic-gui/tests/schema_mirror.rs` + `schema_mirror_secret_drift.rs`) enforces ONLY clap **flag-NAME** parity (plus the per-flag `secret` projection) — it does NOT gate the runtime `--json` **wire-shape** of any of these subcommands, nor the `combine --to` value-enum dropdown contents (`phrase|entropy|ms1`). A wire-shape change (renamed/added/removed JSON key, or a new `--to` value) will NOT trip any automated drift gate; it accumulates silently until a GUI consumer mis-parses at runtime.
- **Why deferred:** This is the documented standing posture for ALL toolkit/sibling `--json` wire-shapes (per `mnemonic-toolkit/CLAUDE.md` "Scope of the gate — clap flag-NAME parity, NOT JSON wire-shape"; the broader generalization is the toolkit FOLLOWUP `schema-mirror-flag-name-vs-wire-shape-conceptual-clarification` option (b), v0.30+). Downstream consumers self-update via the **paired-PR rule**: any `--json` wire-shape or `--to` value-enum change to this K-of-N surface MUST land a same-cycle (or paired sibling) PR on `mnemonic-gui` that updates the consumer. This entry records the K-of-N instances so a future wire-shape edit knows where the un-gated consumers live.
- **Companion:** `mnemonic-toolkit/design/FOLLOWUPS.md` entry `ms-kofn-json-wire-shape-ungated` (toolkit-side mirror); generalization tracked at toolkit `schema-mirror-flag-name-vs-wire-shape-conceptual-clarification`.
- **Status:** `open` (standing-posture / paired-PR tracking — fires no automated gate by design).
- **Tier:** `cross-repo`

---

## Resolved items

(none yet)

### `mnemonic-gui-schema-mirror` — companion to `bg002h/mnemonic-gui` schema gate

- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `mnemonic-gui-schema-mirror`; CI gate at `.github/workflows/schema-mirror.yml`.
- **Where:** This CLI's clap-derive `Args` blocks for every subcommand the GUI surfaces (v0.1: `ms inspect`; v0.2+: encode/decode/verify/vectors). Also `crates/ms-cli/src/cmd/gui_schema.rs` (the SPEC §7 reflection emitter shipped in `ms-cli-v0.2.0`).
- **What:** The `mnemonic-gui` GUI mirrors this CLI's clap-derive flag surface at pinned tag `ms-cli-v0.1.0` (regex path) / `ms-cli-v0.2.0`+ (JSON path via `ms gui-schema`). Any flag add / remove / rename / `conflicts_with` / `required_unless_present_any` change in this repo's CLI surface must land in lockstep with a companion `mnemonic-gui` PR that bumps the schema + the `pinned-upstream.toml` tag for this CLI. The `mnemonic-gui` CI gate runs `cargo install --locked --git <this-repo> --tag <pin>` + `cargo test --test schema_mirror`, so drift surfaces as a CI failure.
- **Phase C.2 (v0.2):** `ms gui-schema` subcommand added — emits SPEC §7 JSON via `clap::CommandFactory` reflection. Stays in lockstep with `Cli` automatically (no parallel hand-written table to maintain).
- **Status:** `open` (mirror-invariant; tracking only — every flag-surface PR carries this lockstep work).
- **Tier:** `v1 / mirror-invariant`

### `mnem-wordlist-language-hint-on-wire` — v0.2+ payload kind embedding the BIP-39 wordlist language

- **Surfaced:** 2026-05-30, constellation feature-coverage survey → Theme-C cycle-prep recon (`mnemonic-toolkit/cycle-prep-recon-theme-c-footguns.md`, item 1).
- **Where:** reserved tag `mnem` (`crates/ms-codec/src/consts.rs:39` `RESERVED_NOT_EMITTED_V01`); hazard documented at SPEC §6.3 (`design/SPEC_ms_v0_1.md:59`) + `crates/ms-codec/README.md:42`.
- **What:** ms1 v0.1 does not carry the BIP-39 wordlist language on the wire — a non-English user recovering via an English-defaulted *third-party* wallet silently derives a different BIP-32 master seed → different addresses → empty wallet. (Note: `ms decode` ITSELF is not silent — it loud-annotates "DEFAULT" on stdout AND stderr when `--language` is omitted, `crates/ms-cli/src/cmd/decode.rs:43`; the residual risk is other software.) A `mnem` payload kind (entropy + wordlist-language discriminant) makes the card self-describing so ANY decoder is unambiguous.
- **Scope note:** NOT an independent small fix — `mnem` rides the **v0.2 prefix-byte migration** (`0x00`/`0x01` discriminator, SPEC §1.3 `:24-29`), the same framing K-of-N share encoding and the `seed`/`xprv`/`prvk` kinds all require. Sequence WITH the ms-v0.2 cycle, not standalone.
- **Status:** `resolved` (2026-06-02) — **ms-codec 0.3.0 + ms-cli 0.6.0 + mnemonic-toolkit v0.39.0** (crates.io + tags `ms-codec-v0.3.0`/`ms-cli-v0.6.0`/`mnemonic-toolkit-v0.39.0`). Implemented as the `mnem` payload kind behind a **`0x02` prefix byte** (byte-aligned `[0x02][language][entropy]`; the 4-bit-packed form was unconstructible for 3/5 lengths under codex32 `sanity_check`). `ms encode` auto-routes non-English phrases → `mnem`; `ms decode`/`inspect` surface the wire language; the toolkit faithfully DERIVES with + EMITS the per-card wire language (closing the footgun for third-party AND in-toolkit recovery). English/entr byte-identical. K-of-N was de-scoped to a separate later cycle (the prefix-byte migration that `mnem` rides is now shipped, so K-of-N keys on the threshold field — see `SPEC_ms_v0_1.md` §5). SPEC `design/SPEC_ms_mnem_wordlist_language.md`; audit trail `design/agent-reports/ms-mnem-*`.
- **Tier:** `v0.2-feature`.

### `ms-codec-error-display-echoes-input` — ms-codec `Error` Display echoes raw input (the secret share) for checksum/HRP failures

- **Surfaced:** 2026-06-11, mnemonic-toolkit stress Cycle C R0 (fuzzing brainstorm, `mnemonic-toolkit/design/agent-reports/cycle-c-fuzzing-r0-round1-review.md` [C1]) — found at REVIEW time while designing the `ms1_no_secret_leak` fuzz oracle, before any fuzzing.
- **Where:** `crates/ms-codec/src/error.rs:118` Displays `Error::Codex32(e)` as `write!(f, "codex32 parse error: {:?}", e)`, and codex32-0.1.0 carries the FULL input string inside its String-bearing variants (`InvalidChecksum { string }`, `MismatchedHrp(String, String)`, `MismatchedId(String, String)`). A single bit-flip of a valid ms1 share → checksum failure → `Codex32(InvalidChecksum)` → the error string contains every substring of the secret share's data-part. Also `Error::WrongHrp { got }` (`error.rs:122`) echoes the attacker-/input-controlled observed HRP; a data-char→`1` mutation shifts the bech32 separator so the parsed "HRP" is a long prefix of the secret.
- **What:** ms1 is SECRET-BEARING; a library-layer error that embeds the raw input means any caller that logs/prints the error (or formats it into a panic) leaks secret material. The toolkit already withholds this at ITS layer (v0.53.4 friendly-mapper, `[[project_toolkit_v0_53_6_schema_gate_secret_string]]` lineage) — but ms-codec-native callers have no such guard. Fix: bound/withhold the input-derived fields at the ms-codec boundary — drop or length-cap `InvalidChecksum.string` echoes (Display a structural summary, not the raw string), and cap `WrongHrp.got` to the HRP region only (it should never contain data-part symbols). Its own cycle (error-surface change + Display-contract review).
- **Why deferred:** found during a DIFFERENT cycle's R0 (fuzzing infra, test-only NO-BUMP); the fix is library behavior change deserving its own R0. The Cycle-C `ms1_no_secret_leak` fuzz target ships WITH a documented variant-matched exclusion set (`Codex32(_) | WrongHrp{..}` skip the window-scan) that SHRINKS when this lands — i.e. the oracle becomes the regression gate the moment the echo is withheld.
- **Status:** `open`
- **Tier:** `next-cycle` / secret-hygiene.
- **Companion:** `mnemonic-toolkit` `design/FOLLOWUPS.md::ms-codec-error-display-echoes-input` (toolkit-side companion linking the v0.53.4 friendly-mapper withholding precedent).
