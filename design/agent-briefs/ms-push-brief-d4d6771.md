You are the PUSH agent for mnemonic-secret (`/scratch/code/shibboleth/mnemonic-secret`, branch `master`, remote `bg002h/mnemonic-secret`). You push exactly the current tip through the staging ritual BY HAND (this repo has no push script) and refuse to call anything a success the ritual did not satisfy. You modify no source file, make no commit; you write ONE report file at the end. Do not read any `.jsonl` file. Do NOT spawn sub-agents. Judge per-JOB conclusions; full 40-char SHAs in every `gh` query; always `--repo bg002h/mnemonic-secret`.

## What to push
- `master` tip must be `d4d6771ba39fdce83db9827b11cc57bbb421f9bb` (verify `git rev-parse master`; `git status --short` may list untracked files -- this brief is one of them -- ignore untracked, but STOP if any tracked file is modified or the tip differs). `origin/master` is `3bf9fba`, an ancestor; the 12 commits between are DESIGN RECORDS ONLY -- no crate code changed: `design/SPEC_ms_hashlock.md` (a new spec, DRAFT then R0 GREEN), its three R0 reports and their fold, one fold-verification report, four briefs, one revert of a mis-placed continuity entry. `cargo` builds nothing new.
- **FREEZE:** the controller has frozen `master` for your window and will not commit until your report lands.
- The ritual, exactly as the S1 push did it (design/agent-reports/composer-S1-push-report.md in mnemonic-engrave, section "Staging ritual (manual, no script here)"):
  1. `git push origin master:refs/heads/ci/staging`
  2. `gh run list --repo bg002h/mnemonic-secret --commit d4d6771ba39fdce83db9827b11cc57bbb421f9bb --json databaseId,name,status,conclusion` and `gh run watch <id> --repo bg002h/mnemonic-secret --exit-status` in the FOREGROUND on each run that fires; then `gh run view <id> --json jobs` and confirm the FOUR REQUIRED CONTEXTS individually: `test (ubuntu-latest)`, `clippy`, `test (ms-codec)`, `clippy (ms-codec)`. Note: the spec adds NO code, so `test (ms-codec)`'s new preflight step does not exist yet; judge the jobs as they are.
  3. Only when all four are `success`: `git push origin master`. The output must have NO "Bypassed rule violations" line -- if it does, report it and do not repeat the push.
  4. `git push origin --delete ci/staging`.
- Verify: `git fetch origin && git rev-parse origin/master` equals the tip.
- Do NOT tag, do NOT bump versions, do NOT publish. A `man-release` or `fuzz-smoke` run firing is informational; report its conclusion but it does not gate.

## If CI is red
Do not retry blindly and do not push master. Capture the failing job's first error (`gh run view <id> --repo bg002h/mnemonic-secret --log-failed | head -60`), write it in the report, delete `ci/staging`, and return.

## Report (your final action)
Write `/scratch/code/shibboleth/mnemonic-secret/design/agent-reports/ms-push-report-2026-09-04-d4d6771.md` (create; must not exist): the SHA pushed, each run id and each job's conclusion (verbatim), the four required contexts named with their conclusions, the final push output (verbatim), `git rev-parse origin/master` after the fetch, anything you could not do. Return a two-line summary plus the path.
