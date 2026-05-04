# CLAUDE.md — ms1 (`mnemonic-secret`) repo notes for Claude Code sessions

This file is auto-loaded by Claude Code when starting a session in this repository.

## Project at a glance

`ms1` is the third sibling in the m-format family — a Bitcoin BIP-style backup format for **secret material**. HRP `ms`. Unlike the sibling `md1` (HRP `md`, repo `bg002h/descriptor-mnemonic`) and `mk1` (HRP `mk`, repo `bg002h/mnemonic-key`) which fork BIP-93's BCH plumbing locally with HRP-mixed per-format target residues, ms1 adopts **BIP-93 codex32 directly** via Andrew Poelstra's `rust-codex32` crate (CC0, on crates.io at `=0.1.0`). The three formats engrave together as a coherent backup bundle: md1 = template/policy, mk1 = xpubs, ms1 = secret. v0.1 is single-string (BIP-93 threshold = 0); K-of-N share encoding is planned in v0.2.

**v0.1 payload scope** (narrowed by r6 BRAINSTORM amendment after pre-SPEC spike, 2026-05-03): emits **BIP-39 entropy only** (16/20/24/28/32 B). Direct `seed` (64-B BIP-32 master seed) and `xprv` (78 B) payloads are reserved-not-emitted in v0.1 because they overflow BIP-93 codex32's length brackets when prepended with the v0.2-migration prefix byte; they are deferred to v0.2+ which will design BCH framing outside BIP-93's brackets. The BIP-32 master seed backup use case is preserved via the routing `BIP-39 seed phrase → entropy → ms1 entr → engrave → recover → BIP-39 mnemonic → PBKDF2 → master seed`. See `design/BRAINSTORM_ms_v0_1.md` §"Wire-format spike findings" and FOLLOWUPS handle `ms1-v01-payload-bracket-overflow-prefix-byte-incompatibility` for the discovery record.

## Active work

- **Plan:** [`design/IMPLEMENTATION_PLAN_ms_v0_1.md`](design/IMPLEMENTATION_PLAN_ms_v0_1.md) (to be written via the `superpowers:writing-plans` skill after the SPEC lands)
- **SPEC:** [`design/SPEC_ms_v0_1.md`](design/SPEC_ms_v0_1.md) — wire-format spec, payload semantics, v0.1 → v0.2 migration contract
- **BRAINSTORM:** [`design/BRAINSTORM_ms_v0_1.md`](design/BRAINSTORM_ms_v0_1.md) — 5-question rationale chain anchoring the SPEC's design decisions
- **MIGRATION:** [`MIGRATION.md`](MIGRATION.md) — v0.1 → v0.2 contract (reserved-prefix byte, grouping invariant, anti-collision invariant, API back-compat)
- **FOLLOWUPS:** [`design/FOLLOWUPS.md`](design/FOLLOWUPS.md) — deferred items + cross-repo coordination
- **Plan-mode meta-plan (out-of-tree):** `/home/bcg/.claude/plans/c-ultimately-what-we-quirky-avalanche.md` — converged at r5 after 5 reviewer rounds; treats this repo's SPEC + IMPLEMENTATION_PLAN as the canonical artifacts

## Workflow conventions

The user established this workflow on md1 v0.6 / v0.7 and applied it consistently to mk1 v0.1 and now ms1. The full convention is recorded in the user's auto-memory; the load-bearing parts:

1. **Per-phase opus review.** After each phase commit, dispatch a `superpowers:code-reviewer` (or equivalent) subagent with `model: opus` to verify the work.
2. **Convergence loop.** Iterate brainstorm/spec/plan/per-phase reviews until a round returns no critical or important findings — only nits and affirmations terminate the loop. Single-pass reviews miss issues.
3. **Save per-implementation-phase reports to disk** at `design/agent-reports/<filename>.md`. **Brainstorm/spec/plan/final-completion reports stay in conversation transcript** — do *not* persist them to disk per the 2026-05-03 refinement.
4. **Apply critical / important findings inline.** Should-address items get fixed before moving to the next phase, in a follow-up commit.
5. **Collect deferred items in `design/FOLLOWUPS.md`** at the appropriate tier.
6. **TDD per phase.** Tests land before impl within each task; `#[ignore]`-marked scaffolds in earlier phases get un-ignored when their code path lands.
7. **Parallel tool calls** for independent operations.
8. **Per-phase commit cadence:** feature commit + fixup commit (after review). Sometimes a third nit-cleanup commit.
9. **Terse code and doc-comments.** Short doc-comments, no narrative module prose.

## Cross-repo coordination

Four-way star: `descriptor-mnemonic` (md1) ↔ `mnemonic-key` (mk1) ↔ `mnemonic-secret` (ms1) ↔ future `mnemonic-toolkit`.

- Follow-up items affecting another sibling are mirrored: a primary entry in this repo's `design/FOLLOWUPS.md` at tier `cross-repo`, a companion entry in the affected sibling's tracker. Both entries cite each other.
- The `mc-codex32` shared-crate extraction plan (originally gated on "md+mk both at v1.0 with cross-validated vectors") was **retired 2026-05-03**: ms1 uses `rust-codex32` directly, and md1↔mk1's HRP-mixed BCH isn't upstreamable to it. md1↔mk1 BCH stays forked; the *pattern* will be documented in a future cross-repo `PATTERNS.md`. See cross-repo `FOLLOWUPS.md` entry `mc-codex32-extraction-retired-2026-05-03` in either sibling repo.

## Dependencies

- `codex32 = "=0.1.0"` (CC0, Andrew Poelstra). Exact-pin because the upstream README describes the crate as "pretty rough" and slated for a rewrite around `rust-bech32`. All payload semantics live in `ms-codec`, not in `codex32`. The contact surface is concentrated in `crates/ms-codec/src/envelope.rs` so a future `codex32 = "0.2"` rewrite is absorbable in one module.
- MSRV `1.85` — matches md-codec; do not lead, follow the family's MSRV pin.

## Practical tips

- `cargo test -p ms-codec` runs all tests.
- `cargo clippy --all-targets -D warnings` and `cargo fmt --check` are CI gates.
- Wire format SHA is pinned at v0.1.0 release per `design/RELEASE_PROCESS.md`.
