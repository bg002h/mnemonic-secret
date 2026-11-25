# Final review (Fable): mnemonic-secret draft PR #48 (`claude/project-thread-utm1aa`)

Scope: `git diff $(git merge-base HEAD origin/master) HEAD` at local HEAD 527223a (code commits d6c20a2, 9b5dd21, ad63c8f). The remote head d2c4604 adds only the r2 report and FOLLOWUPS edits, which I ignored as instructed. Read-only: nothing committed, pushed or posted.

## Verdict

**0 Critical, 0 Important, 2 Minor, 3 Nit.** The three fixes do what they claim, and I found nothing that changes a card, the wire format, a passphrase value or a derived wallet. The PR can be marked ready once the PR body is corrected (M2).

## What I ran

- `CARGO_INCREMENTAL=0 cargo test --locked -j 2 -p ms-codec -p ms-cli`: exit 0, 867 passed, 0 failed, 23 ignored, 131 binaries. Log: `/tmp/claude-0/-home-user/efb9fedd-ab0a-5019-871b-00d31aeaaeb6/scratchpad/ms-fable-tests.log`.
- Probes with `target/debug/ms` (0.21.0) on a real 2-of-3 split, in `/tmp/claude-0/-home-user/efb9fedd-ab0a-5019-871b-00d31aeaaeb6/scratchpad/fb/`.

## Is each finding fixed?

- **ms#2: fixed.**
  - `ms encode --hex - --language japanese` and the `ms split` equivalent exit 64 (`Usage`), also under `--json`, and no `--out` file is created.
  - `--hex` alone and `--hex --language english` both mint `ms10entrsqqqqzqsrqszsvpcgpy9qkrqdpc8sh9lf92n5fr33s`, the card from the bug report.
  - `ms combine` on an Entr set prints `language: english (12 words, default — verify against your records)` plus the stderr note.
  - `ms derive --hex --language` is untouched (`33a65c3b`).
- **ms#1: fixed.**
  - A share with one wrong character: `ms combine` names it by position ("share 1 of 2"), and `ms repair` returns the original share at exit 4 with the share-specific UNVERIFIED line. The repaired share recombines to the original entropy.
  - Grouped (spaces), hyphenated and all-uppercase damaged shares are repaired too.
  - Damage in the header is routed correctly: threshold `2`→`0`, index `q`→`s`, index `q`→`p` and a wrong id character each come back as the original share, exit 4.
  - A valid share is still refused (exit 2). With both shares damaged, both are named.
  - The note fires only on a combine error, and only for shares that fail their checksum: a repeated-index set of valid shares prints no note.
- **ms#3: fixed.** `printf 'TREZOR\n\n'` on `--passphrase-stdin` warns on stderr and still derives `86d72e39`, so the value is unchanged. `--json` stdout is unchanged. The `@env:` channel names the variable, never the value.

## The other review questions

- **Wire format and existing cards:** unchanged. `correct_checksum` is the old body of `decode_with_correction` moved; the clean path still calls `decode` on the caller's original string and the corrected path on the same re-encoded string as before. ms#2 adds only an early `Err`. No encode path is touched.
- **Secret leakage:** none new.
  - `ms repair` on a share prints what it already prints for a card: the corrected string on stdout or `--out`, `original_chunk` in `--json` (existing schema), and the PrivateKeyMaterial advisory.
  - The combine notes and the passphrase warning carry positions and channel names only.
- **Regressions:** none found. The one intended behaviour break is the exit-64 refusal (see M1 for the documents it makes false).
- **Tests:** they cover the behaviour and r1 showed them RED on master. The gaps are N2 only.
- **Code clarity:** fine. The comments say why, and the share path in `repair.rs:161-175` is small and guarded (`index != 's'`, then a re-parse with `Codex32String::from_string`).

## Findings

### Minor

**M1. Three shipped manual lines become false, and the follow-up does not name them or have companions.**
- Where:
  - `mnemonic-toolkit/docs/manual/src/40-cli-reference/43-ms.md:87` and `:255`: "`--language` … ignored under `--hex`".
  - `mnemonic-toolkit/docs/manual-gui/src/60-ms/67-split.md:35`: "ignored under `--hex`".
  - `mnemonic-gui/src/schema/ms.rs:612`: the same help text (already logged by r1 M1).
- Scenario: an operator reads the manual, passes `--hex - --language japanese` expecting it to be ignored, and gets a usage error. It fails closed, so no funds risk.
- Why it is a finding: this repo's CLAUDE.md asks for the `43-ms.md` chapter to move in lockstep. The hunt's toolkit branch (`6e304aa`) touches `41-mnemonic.md` but not `43-ms.md`.
- The follow-up as filed: `ms-bh1125-toolkit-parity` (`design/FOLLOWUPS.md`) says the chapter "should mention" the refusal. It does not cite the lines that are now wrong, it omits `manual-gui/67-split.md`, and neither toolkit nor mnemonic-gui `FOLLOWUPS.md` has a companion entry (r2 N2 noted the GUI half).
- Fix: either pair a toolkit PR that corrects the three lines, or extend the follow-up with the exact lines and add the companion entries, and say in the PR body that the manual is knowingly behind until the next ms pin bump.

**M2. The PR body is out of date in three places.**
- GUI bullet: it asks only whether `ms_encode` emits a hidden `--language`. r1 established the real gap is `ms split`, which has no conditional (`mnemonic-gui/src/schema/ms.rs:990-996`), so a non-English selection plus `--hex` in the GUI now exits 64. The body does not say so.
- Follow-ups list: it omits three items now logged in FOLLOWUPS: `ms-bh1125-spec-hex-language` (`design/SPEC_ms_cli_v0_1.md:57` still says "ignored under --hex"), `ms-bh1125-correct-checksum-length` and `ms-bh1125-gui-split-hex-language`.
- Reviews: it does not mention that r1 and r2 ran (0 Critical, 0 Important) or where their reports are.
- The rest of the body is accurate: the test count (867 / 0 / 23) matches my run, and the per-finding descriptions match the code.
- Fix: update those three parts before marking the PR ready.

### Nit

- **N1. `crates/ms-cli/src/cmd/combine.rs:238`.** A share with one character outside the alphabet (`b`, `i`, `o`, `1`: a common misreading) gets `note: share 1 of 2 is not an ms1 share string`. It is a share with one unreadable character, and the error line right after already says `invalid character 'b'`. Suggest "holds a character that is not in the codex32 alphabet; `ms repair` cannot correct that" for `Error::Codex32(InvalidChar)`, keeping the current text for `WrongHrp`.
- **N2. No regression row for header damage.** The threshold `2`→`0` and index→`s` cases go through `decode_with_correction`'s corrected string and the `index != 's'` guard (`repair.rs:162`). They work by probe, but no test in `crates/ms-cli/tests/bh1125_share_repair.rs` pins them (r1 N5 asked for only the true index-`s` case). Add two rows.
- **N3. The logged nit about the ms#2 remedy is partly wrong.** `ms-bh1125-review-nits` item (2) says the printed remedy drops `--out`, `--json` and `--group-size`. `--out` is kept: I observed `ms encode --out='e.out' --phrase - --language japanese` and `ms split -k 2 -n 3 --out='s.out' --phrase - …`. Only `--json` and `--group-size` are dropped. Correct the line when it is next touched.
