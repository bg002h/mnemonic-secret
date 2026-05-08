# Follow-up tracker

Single source of truth for items that surfaced during a review or implementation pass but were not fixed in the same commit. Mirrors the conventions of the sibling `descriptor-mnemonic` and `mnemonic-key` repos.

## How to use this file

**Format for each entry:**

```markdown
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

---

## Resolved items

(none yet)
