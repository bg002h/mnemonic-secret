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
  - **[obs] ✓ RESOLVED (Cycle-B, ms-codec 0.7.0, 2026-06-23)** `recovered-secret-string-not-zeroized` — closed by the codex32 vendor/fork; `Codex32String` now `ZeroizeOnDrop`-scrubs the recovered `secret` (and the whole share spine) on drop. See `ms-codec-share-strings-not-zeroized-encode-and-combine`. — combine_shares binds `let secret = Codex32String::interpolate_at(...)` (shares.rs:236); Codex32String is a newtype over String (codex32-0.1.0 lib.rs:102 `pub struct Codex32String(String)`) with no Dro (`crates/ms-codec/src/shares.rs:236-242; codex32-0.1.0 lib.rs:102; crates/ms-codec/tests/lint_zeroize_discipline.rs:62-69`). **BROADENED (2026-06-21 keymat sweep):** the recovered `secret` is only ONE of ~7 secret-equivalent bare `Codex32String`/`Vec<Codex32String>`/`Vec<String>` bindings across the share spine (encode + combine); the full per-binding surface — incl. the `secret_s` full-secret-at-S in `encode_shares` and the input-share `parsed` vectors in `combine_shares` — is now enumerated in its own first-class entry `ms-codec-share-strings-not-zeroized-encode-and-combine` (below).
- **Status:** open (backlog index; individual items dispositioned in the report). 2 of 3 resolved (`combine-no-length-validation-panic`, ms-codec v0.4.1; `recovered-secret-string-not-zeroized`, Cycle-B ms-codec 0.7.0); the `pr2-exposure-claim-verified-sound` `[obs]` remains (verified-sound, upstream-tracking).
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

### `sibling-gui-schema-v5-default-value-emission` — `ms gui-schema` emits version-1 JSON (no `default_value`), so mnemonic-gui cannot two-side its `ms` defaults drift gate (companion)

- **Surfaced:** 2026-07-11, mnemonic-gui FOLLOWUP-burndown batch (S2 / constellation-eval #6 extension). mnemonic-gui's `tests/schema_mirror_defaults_drift.rs` gates the toolkit (`mnemonic`, whose `gui-schema` is version 5 — per-flag `default_value` populated) two-sidedly (`default_value` + `choices`), but `ms gui-schema` is still **version 1**: it OMITS the `default_value` key on every flag (R0-verified live at pinned `ms-cli-v0.13.0` — 0 flags carry it). The GUI batch could therefore only extend the gate to `ms` **CHOICES-only** (8 `ms` dropdown flags) plus a SELF-ARMING one-sided guard ("IF the JSON ever carries a non-null `default_value` it must equal the hand mirror"), vacuously green until `ms` emits v5. A true two-sided `ms` defaults gate (catching a silent mirror omission the way `mnemonic`'s does) is infeasible until then.
- **Fix (if pursued):** bump `ms-cli`'s `gui-schema` emitter to v5 parity with the toolkit (populate each flag's `default_value` from its clap-derive default — e.g. `ms encode --language`'s `[default: english]`), release, then mnemonic-gui bumps its `pinned-upstream.toml` `ms` pin + re-points the S2 one-sided guard to a full two-sided gate. Needs an `ms-cli` release + a GUI pin bump — a future cross-repo cycle. The one `ms` mirror-default that would populate: `ms encode --language`.
- **Status:** OPEN. **Tier:** `cross-repo` (producer side: `ms gui-schema` emitter; consumer side: mnemonic-gui re-points the gate once the pin bumps). **Companion:** `mnemonic-gui/FOLLOWUPS.md` (primary) + `descriptor-mnemonic` + `mnemonic-key` `design/FOLLOWUPS.md` `sibling-gui-schema-v5-default-value-emission`.

### `bsd-process-hardening-parity-procctl-rlimit-core` — `ms`'s `set_non_dumpable()` was a silent no-op on the BSDs (companion)

- **Surfaced:** 2026-06-23, the constellation-wide musl/BSD secret-hygiene recon (toolkit `design/SPEC_bsd_hygiene_and_freebsd_gate.md`, Cycle A). `ms`'s `set_non_dumpable()` in `crates/ms-cli/src/process_hardening.rs` was fenced `#[cfg(target_os = "linux")]` and a silent no-op on FreeBSD/OpenBSD/NetBSD — the anti-core-dump + anti-ptrace-introspection protection did not run, so an `ms` process on a BSD could be ptrace/ktrace-introspected and could drop a core file the BIP-39 entropy / mnemonic spills into.
- **Status:** ✓ **RESOLVED (`ms-cli` 0.13.1, 2026-06-23).** Added a BYTE-IDENTICAL (across all four CLI crates) BSD cfg arm: `#[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]` doing (i) FreeBSD-only `procctl(P_PID, 0, PROC_TRACE_CTL, PROC_TRACE_CTL_DISABLE)` and (ii) all-three-BSD `setrlimit(RLIMIT_CORE, {0, 0})`. Best-effort. macOS/Windows remain a documented no-op. No `libc` bump. `ms-codec` NO-BUMP. No CLI flag / subcommand / output-shape change. Linux behavior unchanged.
- **Tier:** `cross-repo`. **Companion:** `mnemonic-toolkit` (primary spec author) + `descriptor-mnemonic` + `mnemonic-key` `design/FOLLOWUPS.md` `bsd-process-hardening-parity-procctl-rlimit-core`.

### `freebsd-compile-gate-ci` — no CI leg compile-checked `ms`'s FreeBSD build / BSD hardening arm (companion)

- **Surfaced:** 2026-06-23, the BSD recon (Cycle C). Nothing in `ms`'s CI caught a Linux-only syscall/cfg/crate breaking the `cargo install`-on-FreeBSD path or the new BSD hardening arm.
- **Status:** ✓ **RESOLVED (NO-BUMP CI infra, 2026-06-23).** Added a `freebsd-compile-gate` job to `.github/workflows/rust.yml` running WHOLE-CRATE `cargo check --target x86_64-unknown-freebsd -p ms-cli` (NEVER `--lib` — `ms-cli` is bin-only [`[[bin]] name = "ms"`, no `src/lib.rs`]; `process_hardening` lives in the bin target, so `--lib` would be silent false-green). `x86_64-unknown-freebsd` is Tier 2 with Host Tools; bare `rustup target add` validated locally (the cross-rs fallback was not needed).
- **Tier:** `cross-repo` / `infra`. **Companion:** `mnemonic-toolkit` (toolkit-primary, `--lib`-correct) + `descriptor-mnemonic` + `mnemonic-key` `design/FOLLOWUPS.md` `freebsd-compile-gate-ci`.

### `ms-cli-ungated-mod-mlock-windows-asymmetry` — `ms-cli` declares `mod mlock;` UNGATED vs the toolkit's `#[cfg(unix)]` gate (note-only; Windows-only-relevant)

- **Surfaced:** 2026-06-23, the BSD recon (toolkit `design/SPEC_bsd_hygiene_and_freebsd_gate.md`, Non-Goal #6). `crates/ms-cli/src/main.rs:21` declares `mod mlock;` UNGATED, whereas the toolkit gates its `mlock` mount with `#[cfg(unix)]`. `mlock.rs` uses POSIX `libc::mlock` / `libc::sysconf` / `_SC_PAGESIZE`, none of which exist on Windows — so an ungated mount would fail to compile a Windows `ms` build.
- **Status:** `open` — **NOTE-ONLY, deliberately NOT fixed.** Windows-only-relevant: both musl and all three BSDs are unix, so the asymmetry never bites the BSD/musl work; `ms-cli` ships/CIs no Windows target, so it is latent, not live. Filed per the recon's explicit "file, do not fix" disposition. Pick up if/when a Windows `ms` build is added (mirror the toolkit's `#[cfg(unix)] mod mlock;` gate, keeping g6 byte-equality in mind — the gate attribute would need matching treatment in both repos).
- **Tier:** `infra` / `next-cycle`. **Companion:** `mnemonic-toolkit` `design/FOLLOWUPS.md` `ms-cli-ungated-mod-mlock-windows-asymmetry` (the cross-cite from the spec author).

### `display-grouping-render-strip-v1` — ✓ RESOLVED (full cycle shipped; reconciled 2026-06-22) — standardized mstring display-grouping (`ms` CLI flags + intake strip; companion)

- **Surfaced:** 2026-06-15, the cross-constellation **mstring display-grouping** cycle (P2 = mnemonic-secret). User-requested standardization of `ms1`/`mk1`/`md1` display output across all four CLIs (`mnemonic`/`md`/`ms`/`mk`).
- **Where:** `crates/ms-cli/src/format.rs` (`render_grouped`, `strip_display_separators`, `is_display_separator`, `parse_separator` — kept LOCAL to ms-cli, bin-only; `chunked` deleted); `cmd/encode.rs` + `cmd/split.rs` (`--group-size`/`--separator`); `cmd/combine.rs` (`-`→stdin `read_shares`); `parse.rs::strip_whitespace` (now strips `-`/`,` too; doubling-dedup heuristic removed); canonical vectors `design/display-grouping-vectors.tsv` (+ `.sha256`, CI-pinned in the clippy job).
- **What (SHIPPED this cycle, ms-cli 0.8.0):** `ms encode` + `ms split` gain `--group-size <u16>` (default 5, `0`=unbroken) + `--separator <space|hyphen|comma>` (default space); text output is now **space/5 print-once** (the old `<ms1>\n\n<chunked>` print-twice + wrap@10 are gone). `ms split` emits shares one-per-line on stdout with labels→stderr. `--json` stays UNBROKEN. Every ms1-intake surface (decode/inspect/repair/encode-`--hex` via `read_input`; `ms combine` positional + `-`→stdin) strips display separators (whitespace + `-` + `,`). The doubling-dedup heuristic is decommissioned (emit is print-once). **ms-codec UNCHANGED** (fns are ms-cli-local). Drift control = copy-with-checksum conformance vectors (canonical TSV authored in the toolkit; byte-identical copy + `.sha256` here; CI `sha256sum -c` + a bin-crate driver test).
- **Note:** ms-codec's decode does NOT tolerate display separators (no md-style "D11"); the legacy `strip_whitespace` handled whitespace only → the net-new strip coverage is `-`/`,` + the structural uniformity.
- **Why deferred / residual:** P4 (toolkit) pin-bumps + collapses `format.rs` + regenerates goldens + updates both manuals; P5 (`mnemonic-gui`) `schema_mirror` flags + separator keyword dropdown. The canonical-vector checksum is a lagging drift gate; the leading control is the paired-PR discipline.
- **Status:** ✓ RESOLVED (reconciled 2026-06-22) — full cross-repo cycle shipped: P2 ms-cli 0.8.0 (this repo), P1 md-cli 0.7.0, P3 mk-cli 0.9.0, P4 toolkit v0.56.0, P5 mnemonic-gui v0.41.0. Verified at reconcile: `ms encode`/`ms split --group-size/--separator` live; vectors + `.sha256` present. Canonical record: `../../mnemonic-toolkit/design/FOLLOWUPS.md` (`display-grouping-render-strip-v1`).
- **Tier:** `cross-repo`.
- **Companion:** mnemonic-toolkit `design/SPEC_mstring_display_grouping.md` (canonical spec) + `design/FOLLOWUPS.md` (`display-grouping-render-strip-v1`, filed in P4) + descriptor-mnemonic `design/FOLLOWUPS.md` (`display-grouping-render-strip-v1`, P1).

### `ms-codec-no-ci-workflow` — add CI (test + clippy + fmt) for both crates + a one-time fmt normalization

- **Surfaced:** 2026-06-01, ms `mnem` v0.2 cycle (Phase 0 spike + every phase gate).
- **Where:** `.github/workflows/` — two workflows now (`rust.yml` + `fuzz-smoke.yml`). `rust.yml` was scoped to `crates/ms-cli/**` with **no `fmt` step**, and `ms-codec` had **no CI at all**.
- **What:** (a) Add a workflow that runs `cargo test --no-fail-fast`, `cargo clippy --all-targets -- -D warnings`, and `cargo +stable fmt --check --all` across **both** crates on push/PR. The `mnem` cycle's only gate was local verification because of this gap. (b) Before the `fmt --check` step can pass, the repo needs a one-time repo-wide normalization: `cargo +stable fmt --all` currently rewrites ~16 pre-existing files (across `ms-codec` and `ms-cli`, drift accumulated from the prior advisory cycle). Land that as a standalone `chore(fmt)` commit FIRST — do **not** bundle it into a feature cycle (the `mnem` cycle deliberately wrote fmt-clean-by-hand and skipped the fmt gate to avoid pulling that churn in).
- **Why deferred:** out of scope for the `mnem` feature; the local gate (full suite + clippy at every phase) was sufficient for this cycle. CI hardening is its own small cycle.
- **Status:** ✓ RESOLVED (CI added + fmt-normalized; NO-BUMP; `5ba05c6` chore-fmt + this CI commit) — Wave-3 lane W3-7 per `design/SPEC_wave3_ms_codec_ci.md` (R0 GREEN 0C/0I). Resolution:
  - (a) **chore(fmt)** normalized **43 non-mlock files** (ms-codec 8 src + 7 tests; ms-cli 10 src + 18 tests) via `cargo +1.95.0 fmt --all` + `git checkout -- crates/ms-cli/src/mlock.rs` (mlock.rs kept g6-synced/unformatted). Landed standalone FIRST.
  - (b) **ci** extended `.github/workflows/rust.yml` with three NEW jobs: `fmt` (pinned 1.95.0, mlock.rs carve-out — `grep -v '/mlock\.rs$'` over `Diff in` headers, copied verbatim from the toolkit's gate), `test-ms-codec` (`cargo test -p ms-codec`), `clippy-ms-codec` (`cargo clippy -p ms-codec --all-targets -- -D warnings`). The `--all` fmt job covers ms-cli with the SAME carve-out (no separate ms-cli fmt step needed). Path filter widened to `crates/ms-codec/**` on both push + pull_request so the new ms-codec jobs actually fire.
  - (c) ms-codec's 19 test binaries now run in CI for the FIRST time.
  - (d) The `~16 files` estimate above was stale — the actual 1.95.0/rustfmt-1.9.0-stable scope is **43 non-mlock files**, matching the toolkit's Wave-1 `toolkit-rustfmt-1-95-0-rebaseline-divergence` chore (this lane is its sibling).
  - NO-BUMP (ms-codec 0.6.0 + ms-cli 0.11.0; pure whitespace + CI, no public API/CLI/wire change). `mlock-rs-fmt-exempt` / `mlock-g4-a-page-count-assert-flake` remain deferred to the next ms-cli g6-pin tag (mlock.rs intentionally NOT reformatted here).
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

### `mlock-g4-a-page-count-assert-flake` — `g4_a_pin_and_zeroize_compose_without_panic` over-asserts `page_count == 1` on a non-page-aligned 64-byte Vec (byte-shared via g6)

- **Surfaced:** 2026-06-17, observed flaking during the toolkit's D1-B (actions-major) master CI re-run. The test does `let mut v: Vec<u8> = vec![0xAAu8; 64];` then `assert_eq!(pin.page_count, 1, "64-byte buf pins exactly one page")` — the plain (non-page-aligned) 64-byte `Vec` can straddle a page boundary under the parallel runner → `page_count == 2` → rare failure. **Production is correct** (pinning 2 pages is fine; `round_to_pages` in `mlock.rs` is right); only the test's `== 1` assertion is over-strict and incidental to the test's actual purpose (pin + `zeroize()` + drop compose *without panic*).
- **Where:** `crates/ms-cli/src/mlock.rs:429-434` (assert at `:434`) — byte-identical to the toolkit's `crates/mnemonic-toolkit/src/mlock.rs:424-433` (synced via the SPEC §6 G6 invariant). `mlock.rs` is the same git blob across `ms-cli-v0.7.0`/`v0.8.0`/master, so there is zero file-drift between those refs.
- **What (the minimal fix, when it IS done):** **delete the single `assert_eq!(pin.page_count, 1, …)` line** in BOTH repos identically (−1 synced line). Coverage is not lost: the toolkit's resolved `mlock-g1-1-test-page-alignment-luck` (page-aligned `alloc_zeroed`, in the unsynced `tests/mlock_unit.rs`) already pins the deterministic `page_count == 1` contract; g4_a's unique value is the zeroize-compose, preserved by the deletion. NOT relax-to-`1..=2` (trivially-true low-value assertion) and NOT page-align-the-buffer (largest synced delta, duplicates g1_1).
- **Why deferred (architect consult, opus, 2026-06-17 — VERDICT: leave tracked):** g4_a lives INSIDE the synced `mlock.rs`, which the g6 invariant byte-compares against the FROZEN pinned tag (`ms-cli-v0.7.0`, from the toolkit's `scripts/install.sh`). Any non-comment byte change forces editing both repos + **publishing a new public ms-cli tag** (v0.7.0 is immutable) + re-pinning the toolkit — an outward-facing, irreversible action that moves a deliberately-frozen anchor, disproportionate to a rare cosmetic flake. A standalone v0.7.1 backport would also NOT discharge the toolkit's `mlock-rs-fmt-exempt` (it wouldn't be 1.95.0-formatted) — worst of both worlds.
- **Trigger to fix:** fold the one-line deletion into both repos the **next time the ms-cli g6 pin is bumped for an independent reason** (ideally the 1.95.0-formatted tag that also discharges `mlock-rs-fmt-exempt`), so marginal cost is ~zero. Until then leave it tracked; the rare flake costs only an occasional CI re-run.
- **Status:** open
- **Tier:** `cross-repo`
- **Companion:** `mnemonic-toolkit/design/FOLLOWUPS.md::mlock-g4-a-page-count-assert-flake` (primary, with the full g6/cost analysis + architect verdict).

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
- **BROADENED (2026-06-21 keymat sweep):** the same class extends beyond decode's intermediate `String` to ALL `--json` emit structs and their secret-bearing owned fields across encode/combine/split/inspect — enumerated in its own first-class entry `ms-cli-json-output-structs-bare-secret-strings` (below). This entry remains the decode-intermediate-`String` leg; the new slug carries the per-struct/per-field surface.
- **Status:** `resolved (ms-cli 0.10.0)` — cycle-15 Lane M: `decode.rs`'s `emit_json` serialized output `String` is now wrapped in `Zeroizing` (the decode-intermediate leg this entry tracked is closed as part of the broader `ms-cli-json-output-structs-bare-secret-strings` fix). `emit_text` prints the already-`Zeroizing`-wrapped `phrase` directly (no new bare intermediate).
- **Tier:** `v1+`

### `rust-codex32-zeroize-upstream` — `codex32::Codex32String` internal payload buffer has no `Zeroize`

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 2 ms-codec envelope work — surfaced while landing the Zeroizing<Vec<u8>> local in `envelope::package`.
- **Where:** Upstream crate `codex32 = "0.1"` (the `rust-codex32` repo). Affects `crates/ms-codec/src/envelope.rs::package` — `Codex32String::from_seed` copies payload bytes into its private buffer during construction; those bytes live for the `Codex32String`'s lifetime (extends until the caller's binding drops).
- **What:** `envelope::package`'s `Zeroizing<Vec<u8>>` local scrubs the `data` buffer when the function exits, but the bytes that `Codex32String::from_seed` copied into its private buffer during construction are NOT scrubbed. Mitigation is lifetime minimization at the ms-codec layer + caller-wrap discipline. Closes when upstream `rust-codex32` adds `impl Drop` + Zeroize on `Codex32String` (or when ms-codec migrates to an internally-controlled codex32 implementation).
- **Status:** `resolved` (Cycle-B, 2026-06-23, ms-codec 0.7.0) — closed via the **vendor/fork** path (`codex32-upstream-dormant-vendor-vs-accept-decision` shape A): codex32 is now vendored inline as `ms_codec::codex32`, and `Codex32String` derives `zeroize::ZeroizeOnDrop` (+ a hand-rolled length-only redacting `Debug`), so its internal payload buffer is scrubbed on drop. The dormant-upstream blocker no longer applies; a compile-time `Codex32String: ZeroizeOnDrop` bound test gates the guarantee.
- **Tier:** `external`

### `rust-codex32-upstream-pr2-recovery-bug-not-exposed` — upstream codex32 PR #2 (shamir recover padding bug); our path verified unexposed + guarded

- **Surfaced:** 2026-06-09, constellation backup/safety gap audit — upstream-repo check on the pinned `codex32 = "=0.1.0"` dep.
- **Where:** Upstream `apoelstra/rust-codex32` **PR #2** (scgbckbone, opened 2026-... [Dec 5 2025], updated Apr 16 2026, **unmerged**): "Serialization to seed & subsequent re-serialization to shares breaks shamir recover result." Root cause: padding — reconstructing a share via `Codex32String::from_seed` from decomposed `data`+metadata recovers a WRONG secret (last-nibble flip on a 16-byte/128-bit secret, `…4979 9` vs `…4979 f`).
- **What:** **Our pipeline is NOT exposed.** `ms_codec::shares::combine_shares` recovers via `Codex32String::interpolate_at` over the parsed share STRINGS (`shares.rs:236`), never the decompose-to-`data` → `from_seed` reload the bug requires — we carry the full codex32 share string end-to-end. Verified two ways: (1) structural (combine never calls `from_seed`); (2) empirical — `ms split`→`ms combine` of PR#2's exact 16-byte secret recovers it correctly across all 2-of-3 pairs; broad cross-length coverage in `tests/spike_kofn.rs` (claim b) + `shares.rs::combine_round_trip_entr_and_mnem_all_lengths` is GREEN. Added a NAMED regression anchor: `crates/ms-codec/tests/codex32_upstream_recovery_regression.rs` (pins PR#2's exact secret round-tripping correctly — fails loudly with a pointer if a future `codex32` bump reintroduces the bug on our path).
- **Status:** `resolved` (our exposure: NONE, verified + guarded). The upstream PR remains open on a dormant crate. **Cycle-B update (2026-06-23):** codex32 is now **vendored inline** in ms-codec (`ms_codec::codex32`, ms-codec 0.7.0); the named anchor `tests/codex32_upstream_recovery_regression.rs` now guards the VENDORED `interpolate_at` recovery path (still GREEN — a future vendored-code change reintroducing the bug would trip it).
- **Tier:** `external` (upstream-tracking; no action required on our side).

### `codex32-upstream-dormant-vendor-vs-accept-decision` — the pinned codex32 crate is abandoned; decide vendor/fork vs accept

- **Surfaced:** 2026-06-09, same audit.
- **Where:** dep `codex32 = "=0.1.0"` (crates.io; source `apoelstra/rust-codex32`). Frozen at **0.1.0 since 2023-03-10**; maintainer note: "as of July 2023 the library is slated to be largely rewritten… may not be worthwhile to improve it until that rewrite arrives" — that rewrite never shipped (3 years on). crates.io carries no repository link. 0 open issues, 1 open PR (the recovery bug above).
- **What:** Both codex32-upstream items (`rust-codex32-zeroize-upstream` + PR#2 above) can never close via an upstream RELEASE — the upstream is dormant. For a steel-backup-of-funds tool sitting on a dormant secret-sharing dep, the dependency posture is a deliberate decision worth making rather than drifting: **(a) accept** (we're unexposed to the recovery bug; the zeroize gap has a working lifetime-minimization mitigation), or **(b) vendor/fork** the crate (own the fixes — adds Zeroize/Drop, de-risks the dormant dep, but takes on maintenance of the BCH/Shamir primitives). Pre-decision: keep the `=0.1.0` exact-pin (no surprise bumps) + the spike/named guards (catch any future bump that breaks invariants).
- **Status:** `resolved` (Cycle-B, 2026-06-23, ms-codec 0.7.0) — **decision MADE: (b) vendor/fork, shape A (inline).** codex32-0.1.0's 3 runtime modules are vendored byte-identical as a private `pub mod codex32` inside ms-codec; the external `codex32 = "=0.1.0"` dep is dropped from the workspace (+ ms-cli + toolkit v0.72.0). Owns the Zeroize/Drop fixes and de-risks the dormant dep; wire encoding is byte-identical (the BIP-93 + captured-golden `codex32_vendor_parity` gate proves it). CC0 LICENSE + attribution headers carried.
- **Tier:** `external`

### `codex32-error-enum-redacting-debug-defense-in-depth` — vendored `codex32::Error` keeps the upstream derived `Debug`; `InvalidChecksum` carries the full ms1 string (2 more carry provenance-bounded fields; NOT reachable today; redacting Debug would be defense-in-depth at the cost of the byte-identical-vendor invariant)

- **Surfaced:** 2026-06-23, Cycle-B post-impl review (vendored-codex32 secret-hygiene sweep, the `Codex32String` drop-scrub fold).
- **Where:** `crates/ms-codec/src/codex32/mod.rs` — `#[derive(Debug)]` on `pub enum Error` (mod.rs:69). Three variants embed the full ms1 string: `InvalidChecksum { checksum, string }` (mod.rs:86), `MismatchedHrp(String, String)` (mod.rs:102), `MismatchedId(String, String)` (mod.rs:106).
- **What:** Unlike the sibling `Codex32String` (which Cycle-B gave a hand-rolled length-only `Debug`), the vendored `codex32::Error` retains upstream's *derived* `Debug`. Strictly, only `InvalidChecksum { string }` (mod.rs:86) embeds the FULL input ms1 string; `MismatchedHrp` (mod.rs:102) and `MismatchedId` (mod.rs:106) carry provenance-bounded fields (hrp=`"ms"` / a 4-char id) that `error.rs:147-153` classifies as safe-but-dropped-for-robustness. A bare `{:?}` of the `InvalidChecksum` variant would echo the secret-bearing string. **This leak is NOT reachable today**, verified four ways: (1) the wrapper `ms_codec::Error::Codex32(_)`'s `Display` peels all 3 leaky arms before its `{safe:?}` fallback (`crates/ms-codec/src/error.rs:157-169`) and the `Debug` impl routes through the same redaction; (2) the toolkit's `friendly_ms_codec` + ms-cli surfaces redact (no bare `{:?}` of a leaky inner variant escapes); (3) no bare `{:?}` of the inner `codex32::Error` exists in `src/`; (4) the `no_echo` / Debug-redaction tests (`codex32_zeroize_debug.rs`, `inspect_report_debug_redaction.rs`, the error-Display redaction cells) prove no ≥8-char secret window reaches any surface.
- **Why deferred (by choice, NOT a defect):** Hand-rolling a redacting `Debug` on `codex32::Error` would be belt-and-suspenders defense-in-depth (it would harden against a *future* code path that `{:?}`-prints a raw inner variant before the wrapper peels it). But the Cycle-B vendor invariant is **byte-identical to upstream codex32-0.1.0 except for the two documented `Codex32String` edits** (the `ZeroizeOnDrop` derive + the redacting `Codex32String` Debug); adding a third hand-rolled impl on `Error` diverges further from that audited-against-upstream baseline. The redaction is therefore enforced at the wrapper boundary (where it is load-bearing and tested) rather than on the vendored type itself. A future cycle could revisit this trade if the vendor-divergence budget is reopened.
- **Status:** `open` (catalog — defense-in-depth, deferred-by-choice; no reachable defect today).
- **Tier:** `external` (vendored-upstream surface).

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

### `ms-codec-inspect-report-payload-bytes-bare-and-debug` — public `inspect()` returns raw entropy in a `#[derive(Debug)]` struct with a bare `Vec<u8>` field

- **Surfaced:** 2026-06-21, secret-key-material hygiene sweep (`mnemonic-toolkit/design/agent-reports/sweep-keymat-mnemonic-secret.md`, finding #1, headline). Audited against `origin/master` @ `e80ea3b`.
- **Secret type / gap class:** raw BIP-39 entropy `Vec<u8>` / class 1 (bare buffer) + class 2 (Debug leak).
- **Where:** `crates/ms-codec/src/inspect.rs:34` (`#[derive(Debug, Clone)]`), `:36-56` (`pub struct InspectReport`), `:48` (`pub payload_bytes: Vec<u8>`), `:80-103` (populated from the decoded payload). Verified vs current `origin/master`.
- **What:** `InspectReport` is the one PUBLIC codec API surface that hands back un-wrapped raw entropy: it derives `Debug` over a bare `pub payload_bytes: Vec<u8>`, so any `{:?}` / `expect` / log of the report (or a wrapper deriving Debug over it) dumps the full seed, and the `Vec<u8>` lives un-scrubbed until drop.
- **Fix direction:** stop deriving `Debug` over the raw bytes (hand-roll `Debug` to redact `payload_bytes`, mirroring the `ms-codec-error-display-echoes-input` precedent) and/or hold the field as `Zeroizing<Vec<u8>>` — both at the public-API boundary, so callers inherit redaction + scrub.
- **Severity:** **High** (codec-library leak-to-Debug of root entropy + bare buffer; both escalation triggers fire — the single public codec entry point returning raw secret bytes in a Debug-printable, non-scrubbing container).
- **Status:** `resolved (ms-codec 0.6.0)` — cycle-15 Lane M, Design A: `payload_bytes: Zeroizing<Vec<u8>>` + hand-rolled redacting `Debug` (`[REDACTED; N bytes]`); marquee Debug-redaction RED test. `PartialEq` left underived; `Deref` keeps readers green.
- **Tier:** secret-hygiene.
- **Companion:** part of the constellation-wide "derived-output + codec-library-internal secret-`String`/`Vec` not zeroized" pattern surfaced by the 2026-06-21 secret-keymat sweep — siblings in `mnemonic-toolkit` (bip85 / electrum / seedqr derived-output) and `mnemonic-gui` (run-holders); toolkit cycle-14 / L22 closed the clap-arg/handler-field leg, this is the codec/output leg.

### `ms-codec-decode-scrub-defeated-by-clone-into-bare-vec` — `decode()` "scrub" `.clone()`s the entropy into a fresh bare `Vec`, adding an un-scrubbed copy

- **Surfaced:** 2026-06-21, secret-key-material hygiene sweep (`mnemonic-toolkit/design/agent-reports/sweep-keymat-mnemonic-secret.md`, finding #2). Audited against `origin/master` @ `e80ea3b`.
- **Secret type / gap class:** entropy `Vec<u8>` / class 1 (bare buffer) + class 6 (intermediate not effectively scrubbed).
- **Where:** `crates/ms-codec/src/decode.rs:82-83` (Entr arm) and `:89-90` (Mnem arm). Verified vs current `origin/master`.
- **What:** The scrub pattern is `let scrubbed: Zeroizing<Vec<u8>> = Zeroizing::new(data); let p = Payload::Entr((*scrubbed).clone());` — the `.clone()` allocates a FRESH bare `Vec<u8>` that becomes the live public payload (never scrubbed); the `Zeroizing` only scrubs the already-moved-from `data`. Net effect is an EXTRA un-scrubbed heap copy, not a removed one, and the lint anchors on `let scrubbed: Zeroizing<Vec<u8>>` so it reads GREEN while the clone defeats the intent.
- **Fix direction:** drop the redundant `.clone()` — move the (already de-Zeroized) bytes straight into the public `Payload` (the public boundary is bare-by-design per `ms-codec-payload-zeroize-public-api`, so the honest move is strictly fewer copies than the clone), and tighten the lint so it cannot read GREEN on a clone-into-bare-`Vec`.
- **Severity:** **Med** (entropy-in-bare-buffer in the codec library; a zeroize that visibly does the opposite of its stated purpose is worse than honest caller-wrap, and the lint gives false assurance).
- **Status:** `resolved (ms-codec 0.6.0)` — cycle-15 Lane M: the deref-clone is removed (bytes move straight into `Payload`); the theater lint row is dropped and replaced by a negative-anchor test (`decode_has_no_clone_into_bare_vec`).
- **Tier:** secret-hygiene.
- **Companion:** part of the constellation-wide "derived-output + codec-library-internal secret-`String`/`Vec` not zeroized" pattern surfaced by the 2026-06-21 secret-keymat sweep — siblings in `mnemonic-toolkit` (bip85 / electrum / seedqr derived-output) and `mnemonic-gui` (run-holders); toolkit cycle-14 / L22 closed the clap-arg/handler-field leg, this is the codec/output leg.

### `ms-codec-share-strings-not-zeroized-encode-and-combine` — codex32 share strings + the secret-at-S held in bare `String`-backed types across the whole share spine

- **Surfaced:** 2026-06-21, secret-key-material hygiene sweep (`mnemonic-toolkit/design/agent-reports/sweep-keymat-mnemonic-secret.md`, finding #3). Audited against `origin/master` @ `e80ea3b`. Broadens the tracked `[obs] recovered-secret-string-not-zeroized` (audit-2026-06-10-backlog) from the single recovered-`secret` binding to the full per-binding surface.
- **Secret type / gap class:** codex32 share strings + the full secret-at-S / class 1 (bare buffer) + class 4 (`String`-backed copies escaping).
- **Where:** `encode_shares`: `secret_s: Codex32String` (`shares.rs:130`, the FULL secret at index S), `defining: Vec<Codex32String>` (`:136`), `distributed: Vec<String>` (`:148`), `single` (`:115`). `combine_shares`: `parsed: Vec<Codex32String>` x2 (`:195,210`, every INPUT share), `secret: Codex32String` (`:281`, the recovered full secret), plus the `.clone()` copies `from_string(s.clone())` (`:197`) and `c.to_string().to_ascii_lowercase()` (`:213`). Verified vs current `origin/master`.
- **What:** `Codex32String` is a newtype over `String` (codex32-0.1.0) with NO Drop/Zeroize, so every share string and the secret-at-S string is held bare and dropped un-scrubbed. Each is secret-equivalent (any share leaks partial secret; `secret_s` / the recovered `secret` leak everything). Only the recovered-`secret` binding is currently tracked (`[obs]`); the `parsed` input vectors, `secret_s`, and the clone copies are the same class but un-enumerated.
- **Fix direction:** root cause is the dormant-upstream `rust-codex32-zeroize-upstream` (a `Drop`/`Zeroize` on `Codex32String`) — so the realistic close path is the vendor/fork decision in `codex32-upstream-dormant-vendor-vs-accept-decision`; meanwhile minimize lifetimes and (where cheap) hold the `Vec<u8>`-shaped intermediates in `Zeroizing`. This entry's job is to give that vendor/fork decision the full secret surface (all ~7 bindings, not just one).
- **Severity:** **Med** (codec-library secret material in bare `String`-backed buffers; **arguably High** for `secret_s` and the recovered `secret`, which are total-leak-equivalent).
- **Status:** `resolved` (Cycle-B, 2026-06-23, ms-codec 0.7.0) — the codex32 vendor/fork (shape A inline) closed the root cause: `Codex32String` now derives `ZeroizeOnDrop`, so every share-spine `Codex32String` binding (`secret_s`, `defining`, the `parsed` input vectors x2, the recovered `secret`, `derived`) auto-scrubs on drop. The sole residue is the **public return value** `distributed: Vec<String>` (`shares.rs:148`) + its `.to_string()` copies — irreducibly a caller-responsibility leg (it IS the function's output), honestly documented as such (no false GREEN). The cycle-15 `Vec<u8>` wraps remain. Supersedes the `[obs] recovered-secret-string-not-zeroized` backlog item.
- **Tier:** secret-hygiene (root cause `external`, upstream-blocked via `rust-codex32-zeroize-upstream`).
- **Companion:** part of the constellation-wide "derived-output + codec-library-internal secret-`String`/`Vec` not zeroized" pattern surfaced by the 2026-06-21 secret-keymat sweep — siblings in `mnemonic-toolkit` (bip85 / electrum / seedqr derived-output) and `mnemonic-gui` (run-holders); toolkit cycle-14 / L22 closed the clap-arg/handler-field leg, this is the codec/output leg. Roots in `rust-codex32-zeroize-upstream` + `codex32-upstream-dormant-vendor-vs-accept-decision`.

### `ms-cli-inspect-intake-and-entropy-not-zeroized` — `ms inspect` is the lone ms1-intake command that does NOT wrap its input in `Zeroizing`

- **Surfaced:** 2026-06-21, secret-key-material hygiene sweep (`mnemonic-toolkit/design/agent-reports/sweep-keymat-mnemonic-secret.md`, finding #5). Audited against `origin/master` @ `e80ea3b`.
- **Secret type / gap class:** ms1 string (seed-secret-equivalent) + raw entropy / class 1 (bare buffer).
- **Where:** `crates/ms-cli/src/cmd/inspect.rs:33` (`let ms1 = read_input(...)` — NOT `Zeroizing`-wrapped), `:34` (`ms_codec::inspect(&ms1)` → bare `InspectReport.payload_bytes` per #1), `:217,247` (`hex::encode(&report.payload_bytes)`). Verified vs current `origin/master`.
- **What:** `ms inspect` is the ONLY ms1-intake command that holds its input in a bare `String` (contrast decode.rs / verify.rs / derive.rs / repair.rs, all `Zeroizing`-wrapped); it also carries the bare `payload_bytes` from finding #1, and prints the full entropy hex on stdout while holding it un-scrubbed.
- **Fix direction:** wrap the `read_input` result in `Zeroizing<String>` to match the sibling intake commands (and let finding #1's `InspectReport` redaction/scrub cover the report bytes).
- **Severity:** **Med** (CLI intake of seed-secret-equivalent material in a bare buffer; the lone asymmetry vs every sibling intake — not enumerated in the zeroize lint's rows).
- **Status:** `resolved (ms-cli 0.10.0)` — cycle-15 Lane M: `inspect.rs` ms1 intake → `Zeroizing<String>`; lint row added; `InspectReport.payload_bytes` redaction/scrub covered by #1.
- **Tier:** secret-hygiene.
- **Companion:** part of the constellation-wide "derived-output + codec-library-internal secret-`String`/`Vec` not zeroized" pattern surfaced by the 2026-06-21 secret-keymat sweep — toolkit cycle-14 / L22 closed the clap-arg/handler-field leg; this is the CLI-intake/output leg.

### `ms-cli-repair-intake-and-report-strings-not-zeroized` — `ms repair` holds the ms1 input + corrected ms1 + report chunks in bare `String`s

- **Surfaced:** 2026-06-21, secret-key-material hygiene sweep (`mnemonic-toolkit/design/agent-reports/sweep-keymat-mnemonic-secret.md`, finding #6). Audited against `origin/master` @ `e80ea3b`.
- **Secret type / gap class:** ms1 string (seed-secret-equivalent) / class 1 (bare buffer) + class 4 (copies escaping).
- **Where:** `crates/ms-cli/src/cmd/repair.rs:75` (`let original = read_input(...)` — bare `String`), `:63-70` (`struct RepairDetail { original_chunk: String, corrected_chunk: String }`, fields `:65-66`), `:89` (`original.clone()`), `:90,94` (`corrected_chunk.clone()` + `vec![corrected_chunk]`). Verified vs current `origin/master`.
- **What:** `ms repair`'s ms1 input is held bare, cloned into `RepairDetail.original_chunk`/`corrected_chunk` (both bare `String`), and the corrected ms1 — itself a valid, decodable secret string — is collected into `corrected_chunks: Vec<String>`; the whole path carries seed-secret-equivalent material in multiple un-scrubbed buffers.
- **Fix direction:** wrap the `read_input` result + the corrected-chunk vector in `Zeroizing`, and hold `RepairDetail`'s chunk fields as `Zeroizing<String>` (or scrub on the build→emit→drop boundary).
- **Severity:** **Med** (CLI carries seed-secret-equivalent ms1 strings across multiple bare copies; repair re-emits a fully-valid recoverable ms1 — not in the zeroize lint's rows).
- **Status:** `resolved (ms-cli 0.10.0)` — cycle-15 Lane M: ms1 intake + corrected-chunks vector → `Zeroizing`; `RepairDetail` chunk fields → `Zeroizing<String>` and its `#[derive(Debug)]` DROPPED (RULE Z-DEBUG; no `{:?}` consumer); negative-anchor test `repair_detail_does_not_derive_debug` + intake/chunk-field lint rows.
- **Tier:** secret-hygiene.
- **Companion:** part of the constellation-wide "derived-output + codec-library-internal secret-`String`/`Vec` not zeroized" pattern surfaced by the 2026-06-21 secret-keymat sweep — toolkit cycle-14 / L22 closed the clap-arg/handler-field leg; this is the CLI-intake/output leg.

### `ms-cli-derive-xpriv-master-not-zeroized` — derived master/account `Xpriv` (root private key) held in a bare rust-bitcoin type

- **Surfaced:** 2026-06-21, secret-key-material hygiene sweep (`mnemonic-toolkit/design/agent-reports/sweep-keymat-mnemonic-secret.md`, finding #7). Audited against `origin/master` @ `e80ea3b`.
- **Secret type / gap class:** master/account `bitcoin::bip32::Xpriv` (private key derived from seed) / class 1 (bare third-party type, upstream-blocked).
- **Where:** `crates/ms-cli/src/cmd/derive.rs:226` (`Xpriv::new_master(...)` → `master`), `:238-239` (`master.derive_priv(...)` → `acct_xpriv`); the source seed at `:217-218` IS `Zeroizing<[u8; 64]>` + mlock-pinned (good). (Pre-Wave-2 lines were `:220` / `:232-233`; the Wave-2 `ScrubbedXpriv` rewire shifted the derivation sites to `:226` / `:238-239`.) Verified vs current `origin/master`.
- **What:** The seed is scrubbed+pinned, but the derived `master` and `acct_xpriv` `Xpriv` values hold the root/account PRIVATE keys and have no Drop/Zeroize (rust-bitcoin), so they sat bare until scope-end — same third-party-blocked class as the tracked `rust-bip39-mnemonic-zeroize-upstream`, but for `Xpriv`. `ms derive` is the only place an actual xpriv is materialized.
- **Fix direction:** DONE (in-repo leg) — the two derived `Xpriv` values are now confined in a binary-private move-only `ScrubbedXpriv` newtype whose `Drop` does a best-effort byte-scrub (`SecretKey::non_secure_erase()` + a `write_volatile` chain_code zero-write); `master_fingerprint`/`account_xpub` are materialized before either wrapper drops, so output is byte-identical. The residual CLEAN fix (a `Zeroize`/non-`Copy` `Xpriv`) is upstream-blocked, tracked as `rust-bitcoin-xpriv-zeroize-upstream`.
- **Severity:** **Med** (live root private key in a bare third-party type; defense-in-depth).
- **Status:** `resolved (in-repo leg, ms-cli 0.11.0)` — Wave-2 ms lane: best-effort byte-scrub of the derived master/account `Xpriv` via the `ScrubbedXpriv` newtype (mirrors the toolkit's R0-blessed v0.70.0 pattern); lint row + count (13→14) + byte-identical-output regression + compile-time move-only guard + runtime drop-witness. The seed stays `Zeroizing` + mlock-pinned. RESIDUAL: only the clean `Zeroize`/non-`Copy` `Xpriv` stays upstream-blocked → `rust-bitcoin-xpriv-zeroize-upstream` (stays open).
- **Tier:** secret-hygiene (residual root cause `external`, rust-bitcoin upstream).
- **Companion:** part of the constellation-wide "derived-output + codec-library-internal secret-`String`/`Vec` not zeroized" pattern surfaced by the 2026-06-21 secret-keymat sweep — siblings in `mnemonic-toolkit` (bip85 / electrum / seedqr derived-output) and `mnemonic-gui` (run-holders); toolkit cycle-14 / L22 closed the clap-arg/handler-field leg, this is the derived-output leg.

### `rust-bitcoin-xpriv-zeroize-upstream` — `bitcoin::bip32::Xpriv` has no `Zeroize`/`Drop` (root/account private key held bare)

- **Surfaced:** 2026-06-21, cycle-15 Lane M (filed while landing the `ms-cli-derive-xpriv-master-not-zeroized` PARTIAL). The `Xpriv` analogue of `rust-bip39-mnemonic-zeroize-upstream`. Audited against `mnemonic-secret origin/master` @ `6f9f60b`.
- **Secret type / gap class:** master/account `bitcoin::bip32::Xpriv` (private key derived from seed) / class 1 (bare third-party type, upstream-blocked).
- **Where:** Upstream crate `bitcoin = "0.32"` (rust-bitcoin). Affects `crates/ms-cli/src/cmd/derive.rs:226` (`Xpriv::new_master` → `master`), `:238-239` (`master.derive_priv` → `acct_xpriv`). `Xpriv` is `#[derive(Copy)]` and implements no `Zeroize`/`Drop`, so a CLEAN (spill-proof) scrub of the derived private keys is not authorable in-repo; the Wave-2 in-repo `ScrubbedXpriv` is best-effort only.
- **What:** `ms derive` is the only place an actual xpriv is materialized. The source seed IS `Zeroizing<[u8;64]>` + mlock-pinned, and the named-binding residue is now best-effort byte-scrubbed (Wave-2 `ScrubbedXpriv`), but the `Copy`-spilled transient bit-copies the compiler may have made remain unreachable until rust-bitcoin drops `Copy` + adds `Zeroize`.
- **Fix direction:** upstream — add a `Zeroize`/`ZeroizeOnDrop` impl AND drop the `Copy` derive on rust-bitcoin's `Xpriv` (a breaking change). Until then, the in-repo best-effort byte-scrub ships (`ms-cli 0.11.0`, `ScrubbedXpriv`); this entry tracks only the clean upstream close.
- **Severity:** **Med** (live root private key in a bare third-party type; defense-in-depth, upstream-blocked).
- **Status:** `open` (upstream-blocked — the in-repo best-effort scrub now ships in `ms-cli 0.11.0`; only the clean `Zeroize`/non-`Copy` `Xpriv` remains, which cannot be authored in-repo).
- **Tier:** secret-hygiene (root cause `external`, rust-bitcoin upstream).
- **Companion:** the `Xpriv` analogue of `rust-bip39-mnemonic-zeroize-upstream`; the in-repo leg `ms-cli-derive-xpriv-master-not-zeroized` is now `resolved (in-repo leg, ms-cli 0.11.0)` via the best-effort `ScrubbedXpriv` byte-scrub.

### `ms-cli-json-output-structs-bare-secret-strings` — all `--json` emit structs carry secret hex/phrase/shares/ms1 in bare owned `String`s

- **Surfaced:** 2026-06-21, secret-key-material hygiene sweep (`mnemonic-toolkit/design/agent-reports/sweep-keymat-mnemonic-secret.md`, finding #8). Audited against `origin/master` @ `e80ea3b`. Broadens the tracked `ms-cli-decode-emit-zeroize-intermediate` (decode-only) to the per-struct / per-field surface across encode/combine/split/inspect.
- **Secret type / gap class:** entropy hex / phrase / shares / ms1 in bare `String` / class 1 (bare buffer) + class 4 (copies escaping).
- **Where:** `crates/ms-cli/src/format.rs` — `EncodeJson.entropy_hex` (`:61`), `DecodeJson.entropy_hex`+`.phrase` (`:100-101`), `CombineJson.entropy_hex`+`.phrase`+`.ms1` (`:86-89`), `SplitJson.shares: Vec<String>` (`:69`), `InspectReportJson.payload_bytes_hex` (`:129`); plus each `emit_json`'s `let s = to_string(&json)` serialized buffer. Verified vs current `origin/master`.
- **What:** Every `--json` emit struct holds secret material (hex entropy, full phrase, full share set, ms1) as bare owned `String`/`Vec<String>` fields, plus the serialized output `String`; none are `Zeroizing`. Short-lived (build → serialize → drop) but plaintext-secret in un-scrubbed heap until drop.
- **Fix direction:** hold the secret-bearing fields (and the serialized output `String`) in `Zeroizing` (or scrub on the build→serialize→drop boundary) — consistent with the broader decode-emit treatment in `ms-cli-decode-emit-zeroize-intermediate`.
- **Severity:** **Low** (STDOUT-LEAK-adjacent — the data goes to stdout by design one syscall later; defense-in-depth only).
- **Status:** `resolved (ms-cli 0.10.0)` — cycle-15 Lane M: the serialized secret-bearing output `String` is wrapped in `Zeroizing` before `println!` across encode/decode/combine/split/inspect/repair. (verify/share/derive JSON carry no private material — word counts / xpub / fingerprint — left bare.) The parent decode-intermediate leg `ms-cli-decode-emit-zeroize-intermediate` is now covered.
- **Tier:** secret-hygiene (defense-in-depth; broadens `ms-cli-decode-emit-zeroize-intermediate`).
- **Companion:** part of the constellation-wide "derived-output + codec-library-internal secret-`String`/`Vec` not zeroized" pattern surfaced by the 2026-06-21 secret-keymat sweep — toolkit cycle-14 / L22 closed the clap-arg/handler-field leg; this is the CLI-output leg.

### `ms-cli-verify-derived-to-string-temp-not-wrapped` — `emit_round_trip_ok` materializes the full phrase in an un-wrapped `to_string()` temp

- **Surfaced:** 2026-06-21, secret-key-material hygiene sweep (`mnemonic-toolkit/design/agent-reports/sweep-keymat-mnemonic-secret.md`, finding #9, low-confidence). Audited against `origin/master` @ `e80ea3b`.
- **Secret type / gap class:** BIP-39 phrase `String` / class 1 (bare buffer).
- **Where:** `crates/ms-cli/src/cmd/verify.rs:170` (`_mnemonic.to_string()` inside `emit_round_trip_ok`, fn at `:169`). (Report cited `:146`; verified-current line is `:170` — the cycle-8 verify changes shifted it.) The main compare path at `:116-118` IS wrapped via `derived_str`/`supplied_str`. Verified vs current `origin/master`.
- **What:** `emit_round_trip_ok` calls `_mnemonic.to_string()` to count words — a bare temporary `String` holding the FULL phrase, not `Zeroizing`-wrapped, dropped un-scrubbed. One un-wrapped full-phrase temp slips past the otherwise-thorough verify zeroize discipline.
- **Fix direction:** wrap the `to_string()` temp in `Zeroizing` (or count words off the already-wrapped `derived_str` rather than re-materializing).
- **Severity:** **Low** (a single short-lived phrase temp on the success path; defense-in-depth).
- **Status:** `resolved (ms-cli 0.10.0)` — cycle-15 Lane M: `emit_round_trip_ok` counts words off a `Zeroizing<String>` temp (`wc_src`); the existing FALSE-GREEN verify lint row (anchored `derived_str` at `:117`) is RE-POINTED to the `emit_round_trip_ok` site so it actually guards the former `:170` leak.
- **Tier:** secret-hygiene (defense-in-depth).
- **Companion:** part of the constellation-wide "derived-output + codec-library-internal secret-`String`/`Vec` not zeroized" pattern surfaced by the 2026-06-21 secret-keymat sweep — toolkit cycle-14 / L22 closed the clap-arg/handler-field leg; this is the CLI-output leg.

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
- **Why deferred:** found during a DIFFERENT cycle's R0 (fuzzing infra, test-only NO-BUMP); the fix is library behavior change deserving its own R0. The Cycle-C `ms1_no_secret_leak` fuzz target shipped WITH a documented variant-matched exclusion set (`Codex32(_) | WrongHrp{..}` skip the window-scan) that SHRINKS when this lands — i.e. the oracle becomes the regression gate the moment the echo is withheld.
- **Status:** **resolved** 2026-06-12 (ms-codec **0.4.4**, in-repo) — (1) the `Codex32(e)` Display/Debug arm is a manual variant match intercepting the 3 input-bearing codex32 variants (`InvalidChecksum.string`, `MismatchedHrp`, `MismatchedId`) BEFORE any `{:?}`, rendering them structurally only; the other 13 carry ≤1 echoed char. (2) `WrongHrp.got` is capped to the first 4 chars **at construction** (char-counted, multibyte-safe) at all 3 build sites — so ms-cli `details.got` + the toolkit friendly-mapper inherit the bound for free. (3) `#[derive(Debug)]` → hand-rolled `Debug` delegating to sanitized Display (the derive would dump the leaky fields). The `ms1_no_secret_leak` fuzz exclusion is DELETED — the oracle now scans Codex32(_) and WrongHrp{..} and is the permanent regression gate (90s bring-up clean). 5 red-first leak cells + revert-an-arm non-vacuity proof; R0 ×2 GREEN + impl-review GREEN. ms-cli `friendly_codex32` keeps HRP/ID by design (provenance-bounded). ms-cli pin →`=0.4.4`. **SHIPPED 2026-06-12:** ms-codec 0.4.4 PUBLISHED to crates.io (tag `ms-codec-v0.4.4`); toolkit pin-bumped 0.4.3→0.4.4 at **v0.54.4** (tagged) — the `mnemonic` friendly-mapper + any `{:?}` of `ToolkitError` now inherit the protection. Fully propagated; nothing pending.
- **Tier:** secret-hygiene.
- **Companion:** `mnemonic-toolkit` `design/FOLLOWUPS.md::ms-codec-error-display-echoes-input` (toolkit-side companion linking the v0.53.4 friendly-mapper withholding precedent).

### `decode-with-correction-panics-on-non-char-boundary-hrp-slice` — `decode_with_correction` aborts (panic) on non-UTF-8 / no-`1` input

- **Surfaced:** 2026-06-11, mnemonic-toolkit stress Cycle C **ms-phase fuzzing** (the `ms1_decode` fuzz target, first local run, found within seconds). Minimal reproducer: a single `0xaa` byte.
- **Where:** `crates/ms-codec/src/decode.rs:150-151` in `parse_ms1_symbols` (the WrongHrp-construction path), reached from the public `decode_with_correction` (decode.rs:221 → :225). At ms-codec 0.4.2.
- **What:** `parse_ms1_symbols` lowercases the input, and when it does not start with `ms1` it tries to report the observed HRP: `let hrp_end = lower.rfind('1').map(|i| i + 1).unwrap_or(lower.len()); let got = lower[..hrp_end.saturating_sub(1)].to_string();`. When the string has **no `'1'`**, `hrp_end = lower.len()`, so the slice is `lower[..len-1]`. If the byte at `len-1` is **inside a multi-byte UTF-8 char** (e.g. a U+FFFD `�` produced by `String::from_utf8_lossy` on stray bytes like `0xaa`/`0xff`), the slice falls on a non-char-boundary and **`str` indexing panics** → libFuzzer abort (exit 77). A single `0xaa` byte (→ `from_utf8_lossy` → `"�"`, 3 bytes, no `'1'`) panics: `"end byte index 2 is not a char boundary; it is inside '�' (bytes 0..3)"`.
- **Scope:** ONLY `decode_with_correction` panics. The non-correcting `decode("�")` returns a clean `Err(UnexpectedStringLength { got: 3, ... })` (its length gate runs first); `inspect("�")` returns a clean `Err(Codex32(InvalidLength(3)))`. So the bug is isolated to `parse_ms1_symbols`' HRP-report slice. Toolkit callers that route arbitrary/untrusted input through `decode_with_correction` (e.g. `ms repair` and the indel-repair oracle) inherit the abort.
- **Severity:** panic-not-corruption (no funds/wrong-card/secret-leak — it aborts before any payload is produced; it is a DoS/robustness bug, the never-panic charter class). Toolkit-side the friendly-mapper / process boundary may already trap or never feed non-UTF-8, but the library contract is "clean error on arbitrary input", which this violates.
- **Fix sketch (its own mini-R0, NOT done this cycle per the fuzz charter's no-src-change rule):** slice on a char boundary — e.g. compute the HRP region with `char_indices()` / `.get(..end)` and fall back to `""`/the whole string, or simply bound `got` to the chars up to (not including) the last `'1'` using a char-safe operation. Add a `decode_with_correction` characterization test pinning the clean-Err behavior on `"�"`, all-non-alphabet bytes, and an empty-after-lossy input.
- **Reproducer (for the fix cycle):** `String::from_utf8_lossy(&[0xaa])` → `decode_with_correction(&s)` panics; equivalently any byte sequence whose `from_utf8_lossy` form contains no `'1'` and ends inside a multi-byte char. The Cycle-C `ms1_decode` fuzz target re-finds it instantly, so it is the regression gate once fixed.
- **Status:** **resolved** 2026-06-12 (ms-codec **0.4.3**, in-repo) — `parse_ms1_symbols` now slices at `rfind('1')` (`'1'` is ASCII, always a char boundary; no Unicode char's UTF-8 bytes contain `0x31`, empirically verified) and uses the whole string as the observed HRP when there is no separator. WITH-`'1'` path byte-identical (leak-neutral; the `WrongHrp.got` echo bounding stays with `ms-codec-error-display-echoes-input`). 2 regression cells in `decode.rs` (red-first: panicked at decode.rs:151 pre-fix). Mini-R0 GREEN (`design/agent-reports/decode-char-boundary-fix-mini-r0-round1-review.md`; the adversarial sweep confirmed `parse_ms1_symbols` is the SOLE char-boundary panic site across all 4 public entries). `ms1_decode` fuzz target re-enabled in the `fuzz-smoke.yml` smoke matrix as the regression gate (3.9M execs clean post-fix). ms-cli exact pin → `=0.4.3` (pin-only, ms-cli version unchanged, per the 0.4.1/0.4.2 precedent). **SHIPPED 2026-06-12:** ms-codec 0.4.3 PUBLISHED to crates.io (tag `ms-codec-v0.4.3`); toolkit pin-bumped to 0.4.3 at **v0.54.3** (tagged) — `mnemonic` / `ms repair` now get the char-boundary fix. (0.4.3 is an ancestor of the same-day 0.4.4 publish; the toolkit is now on 0.4.4.) Fully propagated; nothing pending.
- **Tier:** robustness (never-panic).

### `reproducible-builds` — bit-for-bit reproducible musl release binaries (path-remap + hermetic container + committed vendor/)

- **Surfaced:** 2026-06-24, the constellation **`reproducible-builds-musl`** cycle (toolkit-led — `mnemonic-toolkit`), P3 recon (`mnemonic-toolkit/design/P3_RECON_codec_repos.md`). `ms` had no prior `reproducible-builds` slug; this is the entry of record for its leg.
- **Where:** `.github/workflows/man-release.yml` `musl-binaries` (the published x86_64 + aarch64 `ms` musl binaries) + the new `repro` caller job; the committed `vendor/` tree; `Cross.toml`; `docs/verify-reproducibility.md`.
- **What:** before this work, the `ms` musl release binaries were NOT bit-for-bit reproducible — a default release build bakes the absolute build path into `.rodata` (panic-`Location` / `file!()` literals; also a `$HOME` PRIVACY leak) and lets `cc` stamp `__DATE__`/`__TIME__` into the libsecp256k1 objects, so two builds from byte-identical source at different paths diverged. The published `SHA256SUMS` was therefore an integrity statement only, not a provenance statement.
- **Status:** ✓ **RESOLVED 2026-06-24 — the toolkit-led `reproducible-builds-musl` cycle, P3b (ms leg).** All inputs that made the build non-reproducible are now pinned, and the published `ms` musl binaries are bit-for-bit reproducible:
  - **The `--remap-path-prefix` remap is ADDED**, in the **re-homed** release build (`man-release.yml` `musl-binaries`), not in a committed `.cargo/config.toml` value (a committed config value passes to rustc verbatim with no `$PWD` expansion → no-op + false assurance). The x86_64 leg runs `cargo build … --remap-path-prefix=/build/src=/build` inside the hermetic container at the fixed `/build/src`; the aarch64 (`cross`) leg uses `--remap-path-prefix=/project=/build` for cross's fixed internal mount. CFLAGS `-ffile-prefix-map` + `SOURCE_DATE_EPOCH` close the secp256k1-sys `cc`-under-musl leaks.
  - **Hermetic build env** via the **digest-pinned container** (`rust:1.85.0@sha256:0ff31c…` + musl-tools, homed in the toolkit's `Dockerfile.repro`, consumed BY BUILT-DIGEST) for x86_64 and the **digest-pinned `cross` image** (`Cross.toml`, `ghcr.io/cross-rs/aarch64-unknown-linux-musl@sha256:702154f5…`) for aarch64.
  - **Committed `vendor/`** (the full crates.io dep graph, INERT — no committed `[source]` block) makes the build `--locked --offline`. `ms` is **fork-free** (no miniscript dep) → the **TWO-block** `--config` `[source]` activation (crates-io + vendored-sources; no git-fork stanza). Offline two-block resolution from committed `vendor/` empirically verified (EXIT 0; dropping `vendored-sources.directory` REDs EXIT 101 — the directory block is load-bearing). `cargo vendor` emitted only the two `[source]` blocks (no git source); Cargo.lock byte-unchanged.
  - **Deterministic packaging:** every `tar czf` replaced with `tar --sort=name --owner=0 --group=0 --numeric-owner --mtime=@$EPOCH … | gzip -n -9`; per-arch `SHA256SUMS.<arch>` + `PROVENANCE.<arch>.txt` (commit SHA + epoch + container/cross digest) uploaded `--clobber`.
  - **CI proves it** via the toolkit's reusable `reproducible-musl-build.yml` (the `repro` caller job; two-distinct-path double-build + cc-validate + gzip-residue), runnable WITHOUT a release tag via `man-release.yml`'s `workflow_dispatch` (the man-build + release-upload steps and the whole `musl-binaries` job are `if:`-guarded off for a manual dispatch). Per-binary verify recipe authored at `docs/verify-reproducibility.md`. All NO-BUMP (CI-infra + docs).
- **Tier:** ✓ RESOLVED — path-remap + hermetic env + committed vendor/ all delivered by the toolkit-led `reproducible-builds-musl` cycle, P3b.
- **Companion:** the toolkit `reproducible-builds-musl` cycle (`mnemonic-toolkit` — `cycle-prep-recon-reproducible-builds-musl.md` + the P3 recon `design/P3_RECON_codec_repos.md`; the centralized recipe `reproducible-musl-build.yml` + `Dockerfile.repro` + `ci/repro/*.sh` + `Cross.toml`, pinned at toolkit `6e37b18e`). `md` (`descriptor-mnemonic`) was the FIRST codec re-home (proved the cross-repo pattern at `e8474f48`); `ms` follows it, `mk` follows.

### `vendor-freshness-pr-gate` — no PR-time guard that `vendor/` satisfies `Cargo.lock` (companion to mnemonic-toolkit)

This repo commits a `vendor/` tree consumed by the `--offline --locked` reproducible build, but has NO leading PR-time check that it stays in sync with `Cargo.lock` — the same latent bug that broke `mnemonic-toolkit` **v0.74.0**'s reproducible release (a codec dep bump without `cargo vendor` → the tag-triggered repro build could not resolve, caught only at the release tag).

- **Status:** ✓ **RESOLVED (2026-06-28)** — ported `ci/repro/vendor-freshness.sh` + `.github/workflows/vendor-freshness.yml` (TWO-block fork-free form; defensive git-source tripwire added so a future git dep fails closed rather than silently mis-resolving). Empirically verified FRESH→exit 0, STALE→exit 1 (vendor restored byte-clean); workflow runs on PR + push to the default branch, path-filtered. **Tier:** `ci`. **Companion:** `mnemonic-toolkit` `design/FOLLOWUPS.md::vendor-freshness-pr-gate` (RESOLVED there 2026-06-26) + `docs/verify-reproducibility.md`.
