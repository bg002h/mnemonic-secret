
## ms-cli 0.17.1 released; F-324 CLOSED (2026-09-02)

- ms master 3bf9fba457b0245c41b35d3aaab0f18fcd4cd2c3 (via staging), tag
  `ms-cli-v0.17.1`, man-release run green (repro + musl legs), seven assets;
  0.17.0 release notes annotated. Reports: `f324-close-report.md`,
  `ms-cli-v0.17.1-release-report.md`. F-324 closed in FOLLOWUPS (9d31862).
- Still owed on the host: me 0.8.1 (the `+`-sign path tightening on master
  c05074f1, CHANGELOG [Unreleased]) -- cut it with the next host change or
  when S3 needs it; not urgent (a `+`-signed origin never came from `md`).
- **H1 PLAN BUILD GATE GREEN at ms `36d314d`** (eleven gate runs from the
  draft; the commit message lists what each found -- anchors, a real
  `validate()` gap, seven compile defects, five test defects, clippy, fmt,
  the downgrade row's source, the type-pin test). Gate on that SHA: 64/64
  hashlock tests, clippy and fmt clean, codeword distance 17, downgrade row
  exit 2 with the reserved-prefix text. The corpus anchor row is filled with
  measured values. **R0 round 0 DISPATCHED**: fidelity (opus) ->
  `ms-hashlock-H1-plan-R0-r0-fidelity.md`, tests (sonnet, runs the gate and
  mutates the wired scratch) -> `ms-hashlock-H1-plan-R0-r0-tests.md`, briefs
  in `mnemonic-secret/design/agent-briefs/`. If a report exists on resume,
  persist it (own commit, ms); if not, re-dispatch. Then: fold (one edit),
  re-gate (a fold re-earns it), sonnet fold verification, STATUS R0 GREEN,
  re-validate immediately before ONE opus implementer in an ms worktree.
  ms master has the plan commits unpushed; push via the hand ritual when no
  commit is imminent.
