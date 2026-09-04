You are the INDEPENDENT fold-verification reviewer (sonnet tier, mechanical) for round 1 of `design/SPEC_ms_hashlock.md`'s R0 in mnemonic-secret (`/scratch/code/shibboleth/mnemonic-secret`). Round 0 = three opus lenses, persisted verbatim at `d02185e` (tests, 1C/11I/11M/4N), `e6ef0a0` (correctness, 1C/7I/6M/2N) and `4c59d8e` (adversarial, 4C/4I/5M/1N) in `design/agent-reports/ms-hashlock-spec-R0-r0-{tests,correctness,adversarial}.md`. The fold is ONE commit, `1a14a4dc5f234d09840e33c68cdebc47078f4aee`, over the draft `5ba61ca`. Its message lists what changed and why; read it first.

ONE QUESTION: did the fold address every Critical and Important from all three reports -- FIXED / PARTIAL / NOT FIXED / DECLINED-with-reason, one line each -- and did it introduce a contradiction or a false claim of its own?

Read-only; commit nothing; no sub-agents; read no `.jsonl`. Copy the repo with `cp -r` if you run anything against a modified tree. Toolchain: the repo's pinned Rust; `~/.cargo/bin/ms` is 0.16.0 (older than the tree).

## Already settled -- do not re-derive
- The controller re-measured every code claim in all three reports before folding (the fold commit message names the sites). Do not re-audit the codebase; check the FOLD against the FINDINGS.
- Three items are CONTROLLER DEFAULTS awaiting the operator, not defects: §1 rule 2 (TagKindMismatch), §4.1's L21 narrowing (`--json` no longer satisfies `--random`'s gate), §9's H0 reordering. Note whether the spec labels each as such; do not argue them.
- Adversarial M-4 is non-gating by the 2026-08-27 secret-handling ruling.

## Verify
1. Build the finding table: for every C and I in the three reports (6 + 22), the spec section that now carries the fix, quoted, and a verdict. A finding the fold says it addressed but whose text you cannot find is NOT FIXED. A finding addressed in one section and contradicted in another is PARTIAL -- name both lines.
2. Minors and Nits: one line each, FIXED / RECORDED / NOT; do not pad.
3. New contradictions the fold introduced. Read §1, §4.1, §8, §9, §11, §12 and §14 as a hostile implementer -- these are the rewritten sections. In particular: does §8's length-row paragraph agree with §1's reachable set; does §11's "`--random` without `--out` exits 64 including with `--json`" agree with §4.1 and §12.5; does §6's six-part edit agree with §11's three `/dev/null` gates; does §9's H0 agree with the front matter and §12.7; do §14's citations agree with the sections that use them.
4. Spot-check three of the fold's NEW measurable claims by running them: the wrong-length set (`22+ceil(8N/5)` against the union set); `me sysw pack` reading stdin with no `--in` (the binary is `/scratch/code/shibboleth/mnemonic-engrave/target/debug/me`, 0.8.0); and `is_ms1_shaped`'s case behaviour (transcribe `argv_guard.rs:134-145` + `format.rs`'s separator stripper and call it on an uppercase plate string).

## Severity
A C/I marked FIXED in the message but not fixed in the text = Critical (a fold that reports what it did not do). A new contradiction between two normative sentences = Important. Wording = Minor/Nit.

## Report (your final action)
Write `/scratch/code/shibboleth/mnemonic-secret/design/agent-reports/ms-hashlock-spec-R0-r1-fold-verification.md` (create; must not exist): the finding table (28 rows for C/I, then M/N), the contradiction hunt with quotes, the three spot-checks with output, closing counts and a plain GREEN / NOT GREEN. Return a two-line summary plus the path.
