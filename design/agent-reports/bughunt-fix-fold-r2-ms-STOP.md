fold r2: STOP at brief step 3, no edit, no commit; worktree clean at c22a867; gate not run; 3 undecided points

# Fold r2 STOP: `design/FIX_bughunt_ms.md` (ms bug-hunt FIX, fold of `bughunt-fix-review-r2-ms.md`)

Stopped before any edit. `bughunt/triage` is at `c22a867` with a clean tree. Nothing was committed or pushed. The script was not copied and the gate was not run. The brief's STOP rule applies: any point its texts do not decide is a STOP.

## Undecided points

1. **Slugs for M-1, M-2, N-1, N-2 (brief step 3).** Each 9.2 entry needs a `### \`slug\`` heading (format: FIX lines 965-968). The review report gives no slug for M-1 (`bughunt-fix-review-r2-ms.md` line 188), M-2 (line 206), N-1 (line 220) or N-2 (line 226).
2. **Titles and `Tier:` for those four entries (brief step 3).** The 9.2 format needs a `— title` after the slug and a `Tier:` line (example entries: FIX lines 968-977). The brief decides only status `deferred-minor` and owning phase "none — ownerless residue". The report gives no title or tier for the four.
3. **Title of the 9.4 entry (brief step 2).** The brief gives the slug `toolkit-repair-indel-search-rejects-capitals`, the body, status, tier and owning phase. It does not give the title. The 9.4 heading format is "### \`slug\` — title (companion)" (FIX line 1147).

## Checked and consistent (no action needed)

- The 5.6.6 Old text occurs once in FIX (lines 820-822). It also occurs in `/scratch/cache/bughunt/ms-grade/toolkit-manual-edits.py` (lines 77-78). The script sha256 is `f1bd63ae…`, matching the FIX header.
- The 2.6 insertion point is after the 5-left-out block's closing fence and before "The limits stated in 5.6 were measured" (FIX line 625).
- The report's 2.6 block puts its intro sentence inside the `~~~text` fence. The brief's "exact" rule means I followed that literally; this is not a STOP.

## Requested

Supply the slugs, titles and tiers for the four 9.2 entries and the title for the 9.4 entry. Then re-dispatch from brief step 1. Step 1 is ready as written.
