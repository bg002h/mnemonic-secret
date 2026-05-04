# Agent reports

Per-implementation-phase reviewer reports persist here. **Brainstorm, spec, plan, and final-completion reviewer reports are NOT persisted here** — they stay in the conversation transcript (and the artifact's revision history). This refinement was locked 2026-05-03.

## File-naming convention

```
v<X.Y.Z>-phase-<P>-review-<commit>.md          # per-phase implementation review
v<X.Y.Z>-phase-<P>-review-<commit>-r<N>.md     # rN if multi-round
```

For batched phase work (parallel agents in one phase), distinguish by bucket id:

```
v<X.Y.Z>-phase-<P>-review-bucket-<id>-<commit>.md
```

The cross-repo `feedback_iterative_review_every_phase.md` auto-memory entry has the full convention.
