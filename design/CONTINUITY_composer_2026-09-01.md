
## ms-cli 0.17.1 released; F-324 CLOSED (2026-09-02)

- ms master 3bf9fba457b0245c41b35d3aaab0f18fcd4cd2c3 (via staging), tag
  `ms-cli-v0.17.1`, man-release run green (repro + musl legs), seven assets;
  0.17.0 release notes annotated. Reports: `f324-close-report.md`,
  `ms-cli-v0.17.1-release-report.md`. F-324 closed in FOLLOWUPS (9d31862).
- Still owed on the host: me 0.8.1 (the `+`-sign path tightening on master
  c05074f1, CHANGELOG [Unreleased]) -- cut it with the next host change or
  when S3 needs it; not urgent (a `+`-signed origin never came from `md`).
- **R0 round 0 LANDED and FOLDED (2026-09-04).** Three opus lenses on
  `SPEC_ms_hashlock.md`: tests 1C/11I/11M/4N (`d02185e`), correctness
  1C/7I/6M/2N (`e6ef0a0`), adversarial 4C/4I/5M/1N (`4c59d8e`) -- all in
  mnemonic-secret. Every measurable claim re-derived by the controller before
  the fold (the fold commit lists the sites). The six Criticals, in one line
  each: the single-string accept set never admitted `hash` so no preimage
  plate was readable; `is_ms1_shaped` does not case-fold, so an UPPERCASE
  plate string passed the phrase channel; `--random --json | jq` lost the only
  copy of X at exit 0; `--out` truncates, so a second `--random` clobbered an
  irreproducible preimage; the flashed SH2 CUTS a preimage plate as a seed
  (isStrictMs1 has no prefix test; unlockEngraveCodex32 never calls DecodeMS1)
  and `me` will classify one as a secret seed record on the ms-codec bump --
  the brainstorm's "older readers refuse" premise is measured false; and
  `--hex` accepts a seed's entropy as X with no warning able to fire.
  ONE fold at ms `1a14a4dc5f234d09840e33c68cdebc47078f4aee` (message = the machine-check record); r1 sonnet
  fold verification DISPATCHED with
  `design/agent-briefs/ms-hashlock-spec-R0-r1-fold-verification-brief.md` ->
  `design/agent-reports/ms-hashlock-spec-R0-r1-fold-verification.md`.
- **THREE CONTROLLER DEFAULTS AWAIT THE OPERATOR** (rulings are theirs; each
  is labelled in the spec): (1) §1 rule 2 -- a single whose id and prefix
  disagree is REFUSED (`TagKindMismatch`) rather than dispatched on the
  prefix; (2) §4.1 -- `--random` requires `--out FILE`; `--json` alone no
  longer satisfies the gate (narrows L21); (3) §9 -- **H0**: the fork's
  `isStrictMs1` prefix test (flashed) and `me`'s classifier guard ship BEFORE
  ms-cli 0.18.0 is released (reorders 4.5). If the operator vetoes any, fold
  the spec back and re-verify.
- Also measured this session, unrelated to a ruling: `me sysw pack --in -` is
  NOT a stdin sentinel (exit 2); the no-argument form reads stdin (exit 0) --
  the spec had it wrong three times, now `… | me sysw pack`. A stray empty
  directory named `-` (Aug 26) was removed from the engrave checkout.
