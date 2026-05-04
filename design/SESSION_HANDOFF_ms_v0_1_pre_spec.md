# Session handoff — ms1 v0.1, pre-SPEC

**Date:** 2026-05-03
**Created by:** controller in the session that scaffolded this repo.
**Use this doc to:** resume the work in a fresh Claude Code session, after a `/clear`.

---

## Where we are in the workflow

The plan-mode meta-plan at `/home/bcg/.claude/plans/c-ultimately-what-we-quirky-avalanche.md` defines the post-ExitPlanMode action sequence. As of this handoff, **post-actions #1 through #5a are complete; #6 (write SPEC) is next.**

Per-action status:

| # | Action | Status |
|---|---|---|
| 1 | Persist plan-review reports | ✅ done — *deliberately not persisted* per the 2026-05-03 refinement (only implementation-phase reports go in `design/agent-reports/`). The cumulative r1..r5 findings live in the plan file's revision-history line and in this conversation's transcript. |
| 2 | Project memory entry + MEMORY.md update | ✅ done — see `~/.claude/projects/-scratch-code-shibboleth-descriptor-mnemonic/memory/ms1_toolkit_long_term_roadmap.md` |
| 3 | Cross-repo: mk1 + md1 CLAUDE.md and FOLLOWUPS updates | ✅ done — committed in respective repos |
| 4 | Cosmetic mk-codec test fix at `error.rs:177` | ✅ done — `"ms"` → `"mq"`; test verified passing |
| 5 | New repo skeleton at `/scratch/code/shibboleth/mnemonic-secret/` | ✅ done — `cargo check` passes |
| 5a | `BRAINSTORM_ms_v0_1.md` | ✅ done |
| 6 | `SPEC_ms_v0_1.md` | 🔲 **NEXT** |
| 7 | `IMPLEMENTATION_PLAN_ms_v0_1.md` (via `superpowers:writing-plans`) | 🔲 pending |
| 8 | Phase 1 implementation begins | 🔲 pending |

## Canonical artifacts (read these first in the new session)

1. `/home/bcg/.claude/plans/c-ultimately-what-we-quirky-avalanche.md` — the converged plan, r5. **Source of truth for design decisions.** Especially the sections "ms-codec v0.1 architecture", "v0.2 migration seam", "RESERVED_TAG_TABLE", "Shipping discipline", and "Decisions locked by user".
2. `/scratch/code/shibboleth/mnemonic-secret/design/BRAINSTORM_ms_v0_1.md` — the rationale chain (5 questions). The SPEC must cite this.
3. `/scratch/code/shibboleth/mnemonic-secret/MIGRATION.md` — v0.1 → v0.2 contract (reserved-prefix byte, grouping invariant, anti-collision invariant, API back-compat, wire-bit equivalence). The SPEC must restate the invariants verbatim.
4. `/scratch/code/shibboleth/mnemonic-secret/CLAUDE.md` — auto-loaded; states the workflow conventions.
5. `~/.claude/projects/-scratch-code-shibboleth-descriptor-mnemonic/memory/ms1_toolkit_long_term_roadmap.md` — auto-memory entry capturing the 4-format star + the four strategic recommendations + the mc-codex32 retirement.

## Workflow conventions to remember (the ones that bite)

- **Iterative reviewer loop until convergence.** Per-phase + spec + plan + brainstorm reviews iterate until a round returns no critical/important findings — only nits and affirmations terminate the loop. Do *not* call something done after one round.
- **Plan/spec/brainstorm reviewer reports are NOT persisted to `design/agent-reports/`.** That directory is for per-implementation-phase reports only. Capture spec/plan review convergence in the artifact's own revision-history line. (This was a 2026-05-03 refinement; older v0.15.x had spec reviews on disk under the previous policy.)
- **Terse code and doc-comments.** Short doc-comments, no narrative module prose. PR #935 was criticized for wordiness.
- **TDD per phase.** Tests land before implementation. `#[ignore]`-marked scaffolds in earlier phases get un-ignored when their code path lands.
- **Avoid `git add -A` in the descriptor-mnemonic and mnemonic-key repos** — root has untracked local helpers (e.g. `resume_may1`). Stage paths explicitly.
- **Verify HEAD content post-commit** with `git show HEAD:path` spot-checks; cargo reads the working tree, masking unstaged edits left out of a commit.
- **Don't drop "looks unused" deps without user confirmation** — `bip39` was wrongly dropped from md-cli once on this premise. Same caution applies to the `crates/ms-cli/` placeholder: do not remove it from the workspace; it has `publish = false` and a `# placeholder` comment for exactly this reason.

## Open loops

- **Commit cadence going forward.** This handoff is being saved alongside an initial commit pass. Subsequent commits (SPEC commit, IMPL_PLAN commit, per-phase commits) follow per-phase cadence: feature commit + fixup commit (after review). Sometimes a third nit-cleanup commit. **Do not commit anything else until the user explicitly approves**, per the user's general "ask before committing" preference.
- **`PATTERNS.md` cross-repo doc.** The `mc-codex32` retirement (see `mc-codex32-extraction-retired-2026-05-03` in both sibling FOLLOWUPS files) calls for documenting the *pattern* (HRP-mixed BCH + per-format target residue) in a cross-repo `PATTERNS.md`. Non-blocking; defer until the next BCH-plumbing concern surfaces.

## Suggested kickoff prompt for the fresh session

> Continue executing the post-actions in `/home/bcg/.claude/plans/c-ultimately-what-we-quirky-avalanche.md`. Skeleton (#5) and BRAINSTORM (#5a) are already on disk in `/scratch/code/shibboleth/mnemonic-secret/` — read `design/SESSION_HANDOFF_ms_v0_1_pre_spec.md` in that repo for full handoff context. Cross-repo updates (#3) and the cosmetic mk-codec test fix (#4) are already done and committed. Next is post-action #6: write `mnemonic-secret/design/SPEC_ms_v0_1.md` based on the plan + BRAINSTORM + MIGRATION.md, then run the iterative reviewer loop until convergence. Then post-action #7: IMPLEMENTATION_PLAN via the `superpowers:writing-plans` skill, same review discipline. **Do not commit anything until the user explicitly approves**, and **do not persist the SPEC reviewer reports** to `design/agent-reports/` (that directory is for implementation-phase reports only, per the 2026-05-03 refinement in `feedback_iterative_review_every_phase.md`).
