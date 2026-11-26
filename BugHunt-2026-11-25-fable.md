# Bug hunt 2026-11-25 (Fable): mnemonic-secret

Report-only review of `mnemonic-secret` at `1f8de48`, part of a constellation-wide hunt across all 11 bg002h repositories. Nothing in this repository was changed by the hunt; this file is the only addition.

**How it was done.** Eight Fable reviewers each audited an area (one of them the boundaries between repositories), reproducing what they could with locally built binaries or scratch tests that were not committed. Three further Fable verifiers then tried to disprove every High and Medium finding (and several Low ones) and re-graded them. Grades below are the verifiers' final grades where they ran.

**Grades.** Critical: can lose funds or leak a seed in ordinary use. High: wrong wallet, addresses or keys, or secret exposure, plausible in normal use, or recovery broken. Medium: needs an unusual but realistic input, or fails in a way a careful user might not notice. Low: fails closed, hardening, or a small exposure. Info: context, not a bug.

**Categories.** security (secret or key exposure, injection, integrity of what is trusted), funds (wrong address, amount, descriptor or recovery that could lose or strand coins), other.

**Status.** "Confirmed (reproduced)" means a command or test showed it. "Confirmed by reading" means the code path was traced but not executed. "Reviewer-confirmed, not re-verified" means the first reviewer reproduced or read it and no verifier re-checked it.

**Totals:** 1 Medium, 4 Low. No Critical findings.

## Summary

| ID | Grade | Category | Status | Finding |
|---|---|---|---|---|
| ms#2 | Medium | funds | Confirmed (reproduced) | `--language` with `--hex` silently dropped by `ms encode`/`ms split` (honoured by `ms derive`): a different wallet |
| ms#1 | Low | funds | Confirmed (reproduced) | A K-of-N share with one wrong character has no repair path in ms-cli (toolkit can repair it) |
| ms#3 | Low | funds | Confirmed (reproduced) | A passphrase from stdin/`@env:` keeps embedded newlines without warning |
| X6 | Low | other | Reviewer-confirmed, not re-verified | Non-ASCII characters read as codex32 symbols by ms-codec 0.7.0/0.9.0 used by consumers |
| X8 | Low | other | Reviewer-confirmed, not re-verified | Pin and label drift (incl. ms-codec 0.7.0 can mint an undecodable `mnem`-tagged card) |

IDs: `md#`, `mk#`, `ms#`, `toolkit#`, `G`, `refugium#`, `wallet#`, `E`, `T`, `SH#`, `M` are per-repository findings; `X` findings are at a boundary between repositories and appear in every report they touch.

## Findings

## ms#2: `--language` with `--hex` silently dropped by `ms encode`/`ms split` (honoured by `ms derive`): a different wallet

**Grade:** Medium · **Category:** funds · **Status:** Confirmed (reproduced)

**Reviewer write-up** (source: `ms`)

### 2. `--language` given with `--hex` is dropped silently by `ms encode` / `ms split`, but honoured by `ms derive` — Low/Medium — CONFIRMED

**Evidence**
- `crates/ms-cli/src/cmd/encode.rs:187`: `if language != CliLanguage::English && phrase_source.is_some() { Mnem } else { Entr }` — with `--hex` the explicit language is discarded and a language-less Entr card (or Entr share set, `split.rs:133`) is minted, exit 0, with no note. The flag help does say "Ignored under --hex" (`encode.rs:77`, `split.rs:71`), and SPEC_ms_mnem_wordlist_language §"--hex carries no language".
- `crates/ms-cli/src/cmd/derive.rs:481`: the same pair `--hex - --language japanese` is honoured (`Mnemonic::from_entropy_in(lang, …)`), so the two verbs disagree about one command line.
- `crates/ms-cli/src/cmd/combine.rs:161,253`: an Entr share set is reported as `language: english (12 words)` with no "default — verify" annotation and no note (unlike `ms decode`, which prints both), and `combine` has no `--language`.

**Impact**
An operator holding the entropy of a non-English wallet checks it with `ms derive --hex - --language japanese` (correct fingerprint), then backs it up with `ms split --hex - --language japanese …` or `ms encode --hex - --language japanese`. Both succeed silently and write a backup with no language. At recovery, `ms combine` prints an English phrase labelled plainly "language: english", and `ms derive` on the card defaults to English: a different seed and a different wallet (Known hazard "the same entropy under two wordlists is two different wallets"). The explicit, load-bearing flag was accepted and ignored at the moment it mattered.

**Reproduction**
```
E=000102030405060708090a0b0c0d0e0f
echo $E | $MS derive --hex - --language japanese
master_fingerprint:  33a65c3b
language:            japanese
echo $E | $MS encode --hex - --language japanese
ms10entrsqqqqzqsrqszsvpcgpy9qkrqdpc8sh9lf92n5fr33s          # an Entr card, identical to the one minted without --language
word count: 12
passphrase: not stored in ms1 (record separately if used)   # no language line, no note; rc=0
echo $E | $MS split --hex - --language japanese -k 2 -n 2 > sj.txt      # rc=0, stderr: "share 1 of 2:" "share 2 of 2:" + output-class warning only
$MS combine --in sj.txt
entropy: 000102030405060708090a0b0c0d0e0f
phrase: abandon amount liar amount expire adjust cage candy arch gather drum buyer
language: english (12 words)
kind: entr
echo ms10entrsqqqqzqsrqszsvpcgpy9qkrqdpc8sh9lf92n5fr33s | $MS derive -
master_fingerprint:  05d027a5
language:            english (DEFAULT)
```
`33a65c3b` (Japanese) and `05d027a5` (English) were both confirmed with an independent Python PBKDF2/BIP-32 computation (`work-mkms/fp.py`).

**Fix direction**: refuse (or at minimum warn on stderr for) an explicit non-English `--language` together with `--hex` on `encode` and `split`, pointing at the phrase channel; give `ms combine`'s Entr output the same "default — verify against your records" annotation `ms decode` prints.

**Verifier re-check** (source: `verify-2`)

### ms#2: `--language` dropped silently with `--hex` on encode/split

**Verdict: CONFIRMED. Grade: Medium (write-up said Low/Medium). Category: funds.**

```
$ echo 000102…0e0f | ms derive --hex - --language japanese   -> master_fingerprint: 33a65c3b, language: japanese
$ echo 000102…0e0f | ms encode --hex - --language japanese   -> ms10entrsqqqqzqsrqszsvpcgpy9qkrqdpc8sh9lf92n5fr33s   rc=0, no note
$ echo 000102…0e0f | ms split --hex - --language japanese -k 2 -n 2 > sj.txt      rc=0, no note
$ ms combine --in sj.txt   -> phrase: abandon amount liar …   language: english (12 words)   kind: entr
```

The help does say "Ignored under --hex" (`encode.rs:77`), which is why this is not higher. But an explicit
non-English flag is accepted and discarded with nothing on stderr; `derive` honours the same pair; and
`combine` labels the result plainly "english" where `ms decode` on the same card prints "default — verify
against your records" plus a note. A non-English user backing up from entropy gets a backup that restores, by
default, to a different wallet. The entropy is intact, so funds are reachable once the language is recalled.

## ms#1: A K-of-N share with one wrong character has no repair path in ms-cli (toolkit can repair it)

**Grade:** Low · **Category:** funds · **Status:** Confirmed (reproduced)

**Reviewer write-up** (source: `ms`)

### 1. A K-of-N share with a transcription error cannot be repaired by any verb — Medium — CONFIRMED — cross-repo (mnemonic-toolkit, suspected)

**Evidence**
- `crates/ms-codec/src/decode.rs:318`: `decode_with_correction` computes and verifies the BCH correction, then calls `decode(&corrected_str)?`, which returns `IsShareNotSingleString` for any threshold 2-9 string (`envelope.rs:145`). The correction it just computed is dropped.
- `crates/ms-cli/src/cmd/repair.rs:147`: `ms repair` has no other route, so it exits 2 with "this is one share of a K-of-N set …; use `ms combine`".
- `crates/ms-codec/src/shares.rs` `combine_shares`: parses every share with `Codex32String::from_string`, so one bad character fails the whole set with a checksum error that does not say which share.
- Cross-repo: `mnemonic-toolkit/crates/mnemonic-toolkit/src/repair.rs:1467` calls the same `ms_codec::decode_with_correction` (toolkit lock resolves ms-codec 0.7.0), so `mnemonic repair --ms1` is expected to refuse a share the same way (SUSPECTED, read only; not run).

**Impact**
`ms combine` sends the holder of a damaged share to nothing, and `ms repair` sends them back to `ms combine`: a loop. A single mis-engraved or mis-read character in one share — well inside the code's 4-substitution capacity, and exactly what `ms repair` fixes on a single-string card — makes that share unusable with the suite. When only K shares survive (always the case for a K-of-K split) the secret is unrecoverable with these tools; the holder must correct the codex32 string by hand or with another implementation. Funds are not lost in principle, but the backup's advertised error correction does not exist for shares, and nothing says so.

**Reproduction**
```
echo 000102030405060708090a0b0c0d0e0f | $MS split --hex - -k 2 -n 2 --group-size 0 2>/dev/null > r.txt
cat r.txt
ms12kyxnqfk4c4vfflm7whnkxfupy6kqq068sday2wzhku58kl
ms12kyxnptcgddj2cnup53hdaaftwytqg5l8sf3dvrsqv73xrj
# one character of share 1 changed (w -> q at offset 20):
printf '%s\n%s\n' ms12kyxnqfk4c4vfflm7qhnkxfupy6kqq068sday2wzhku58kl ms12kyxnptcgddj2cnup53hdaaftwytqg5l8sf3dvrsqv73xrj > r_bad.txt
$MS combine --in r_bad.txt
error: BCH checksum invalid (short code); engraving error or transcription typo
rc=1
echo ms12kyxnqfk4c4vfflm7qhnkxfupy6kqq068sday2wzhku58kl > one.txt
$MS repair --in one.txt
error: this is one share of a K-of-N set (threshold '2', index 'q'); use `ms combine` to recombine K shares
rc=2
$MS repair --in one.txt --json
{"schema_version":"1","error":{"kind":"IsShareNotSingleString","message":"this is one share of a K-of-N set (threshold '2', index 'q'); use `ms combine` to recombine K shares","exit_code":2,"details":{"index":"q","threshold":"2"}}}
rc=2
```
(The share id is random per split; any share and any single substitution reproduce it.)

**Fix direction**: give the codec a correction entry that returns the corrected string and positions without requiring a single-string payload (md-codec's `correct_chunks` is the sibling precedent), let `ms repair` emit a corrected share as an exit-4 unverified candidate, and have `ms combine` name the failing share and point at `ms repair`.

**Verifier re-check** (source: `verify-2`)

### ms#1: a K-of-N share with one wrong character cannot be repaired

**Verdict: CONFIRMED-but-downgraded. Grade: Low (claimed Medium). Category: funds (recovery).**

True for ms-cli alone: with one character changed in share 1 of a 2-of-3 split, `ms combine` says "BCH
checksum invalid (short code)" (rc=1) without naming the share, and `ms repair` says "this is one share of a
K-of-N set …; use `ms combine`" (rc=2). That loop is real.

"By any verb", and the suspected toolkit half, are false. I ran the toolkit path:

```
$ echo ms12jqgpq9y7xa0zusnvqjvmqxq4j7hm4k8ls7esl2y3uerm6w | mnemonic repair --ms1 -
# Repair report
#   ms1 chunk 0: 1 correction at position 17: 'q' -> '8'
ms12jqgpq9y7xa0zusnv8jvmqxq4j7hm4k8ls7esl2y3uerm6w
repair: correction UNVERIFIED — a corrected K-of-N share cannot be self-verified; recombine it with the other shares (mnemonic ms-shares combine …) …
rc=4
```

The output equals the original share. So the suite does repair a damaged share, as an unverified candidate
with the right next step; recovery is not broken. What remains is ms-cli's dead-end guidance: neither message
mentions `mnemonic repair`. Fails closed, hence Low.

## ms#3: A passphrase from stdin/`@env:` keeps embedded newlines without warning

**Grade:** Low · **Category:** funds · **Status:** Confirmed (reproduced)

**Reviewer write-up** (source: `ms`)

### 3. A passphrase read from stdin or `@env:` keeps embedded newlines without a word — Low — CONFIRMED (deliberate rule, no warning)

**Evidence**
- `crates/ms-cli/src/passphrase_input.rs:536` `strip_one_newline` removes exactly one trailing `\n`/`\r\n`; the module table (lines 9-11) and the unit test at `:623` (`"TREZOR\n\n" -> "TREZOR\n"`) show this is intended. Nothing warns when the remaining passphrase contains a line break or other control character.

**Impact**
A passphrase file with one extra blank line, or two lines, derives a wallet from a passphrase that contains a newline, at exit 0 with only the usual notes. No hardware or software wallet lets a user type that passphrase, so a fingerprint or xpub recorded from this run (for example `ms derive … --passphrase-stdin < pass.txt` at backup time) describes a wallet the owner cannot open elsewhere, and the mismatch surfaces only when compared against another tool.

**Reproduction** (card `ms10entrsqqqqzqsrqszsvpcgpy9qkrqdpc8sh9lf92n5fr33s` in `card.ms1`)
```
printf 'TREZOR\n'    | $MS derive --in card.ms1 --passphrase-stdin   -> master_fingerprint: 3dd37943   (passphrase "TREZOR")
printf 'TREZOR\n\n'  | $MS derive --in card.ms1 --passphrase-stdin   -> master_fingerprint: 86d72e39   (passphrase "TREZOR\n")
printf 'TREZOR\nX\n' | $MS derive --in card.ms1 --passphrase-stdin   -> master_fingerprint: a865e667   (passphrase "TREZOR\nX")
```
All three exit 0; stderr carries only the language-default and watch-only notes. The independent Python check gives the same three fingerprints for the three passphrases in brackets.

**Fix direction**: keep the strip-one rule but print a stderr warning (or refuse without an explicit opt-in) when the resolved passphrase still contains `\n`, `\r` or another control character.

**Verifier re-check** (source: `verify-2`)

### ms#3: passphrase keeps embedded newlines

**Verdict: CONFIRMED (deliberate rule). Grade: Low. Category: funds.**

`printf 'TREZOR\n'`, `'TREZOR\n\n'` and `'TREZOR\nX\n'` into `ms derive --in card.ms1 --passphrase-stdin` give
`3dd37943`, `86d72e39`, `a865e667`, all rc=0, stderr carrying only the language and watch-only notes. The
strip-one rule is documented in the module table (`passphrase_input.rs:6-12`), shared with the toolkit and
pinned by 37 vectors, so the behaviour is intended; the gap is only that no warning fires when the resolved
passphrase still holds a line break. It needs a malformed passphrase file.

## X6: Non-ASCII characters read as codex32 symbols by ms-codec 0.7.0/0.9.0 used by consumers

**Grade:** Low · **Category:** other · **Status:** Reviewer-confirmed, not re-verified

**Reviewer write-up** (source: `cross`)

### X6. Non-ASCII characters are still read as codex32 symbols by the pinned ms-codec and mk-codec

- **Severity:** Low
- **Status:** CONFIRMED (probes)
- **Repos:** mnemonic-toolkit and mnemonic-gui (ms-codec 0.7.0, mk-codec 0.4.1); mnemonic-refugium and
  refugium-wallet (ms-codec 0.9.0, mk-codec 0.5.0); mnemonic-engrave and descriptor-mnemonic (mk-codec 0.5.0);
  refugium-wallet also md-codec 0.48.4 (the md fix is in 0.48.5; CHANGELOG only)
- **Evidence and reproduction:**
  - ms: fixed in ms-codec 0.10.1 (bughunt ms #7). `ms-codec/src/decode.rs:184` (0.7.0) and `:208` (0.9.0):
    `let lc = c as u8;`. A valid card with two characters changed, one of them to U+0133 (which truncates to `3`,
    the right symbol there):
    ```
    $ probe-tk ms-fix "ms10entrsqqqqzqsrqszsvpcgpy9qkrqdpc8sh9lf92n5fq3ĳs"
    Ok(("entr", "Entr([0, 1, ..., 15])", 1))          # 0.7.0: accepted, one correction reported
    $ probe-rf ms-fix  (same string)   -> Ok(..., 1)   # 0.9.0
    $ probe-msh ms-fix (same string)   -> Err(Error("codex32 parse error: InvalidChar('ĳ')"))   # HEAD
    ```
    The toolkit reaches this through `repair` (`crates/mnemonic-toolkit/src/repair.rs:1467,1540`). Its
    `apply_ms_corrections` (`repair.rs:1507-1527`) rebuilds the string from `chunk.to_lowercase()` (Unicode, not
    ASCII), so the "corrected" card it prints keeps the non-ASCII character (and shifts positions for a character
    whose lowercase is two characters, such as U+0130): the printed repair is not a valid ms1.
  - mk: fixed in mk-codec 0.5.1. `string_layer/bch.rs:664` (0.4.1) and `:673` (0.5.0) use `s.to_lowercase()`.
    An all-uppercase card with one `K` replaced by U+212A KELVIN SIGN:
    ```
    $ probe-tk mk-dec MK1QPN2ZNPQQSQ64W7VM4EUT<U+212A>S2Q... MK1QPN2ZNPP0F30...   -> Ok(KeyCard {...})   # 0.4.1
    $ probe-rf  (same)  -> Ok(KeyCard {...})                                         # 0.5.0
    $ probe-mkh (same)  -> Err(InvalidChar { ch: 'K', position: 21 })                # 0.6.0
    ```
- **Impact:** a card typed with a look-alike character is accepted (mk) or "repaired" (ms) instead of refused; in
  the toolkit the printed ms1 repair can be an invalid string. No wrong key or wrong seed results: the checksum
  still has to hold.
- **Fix direction:** move the pins (ms-codec >= 0.10.1, mk-codec >= 0.5.1); make `apply_ms_corrections` use
  ASCII lowercasing.

## X8: Pin and label drift (incl. ms-codec 0.7.0 can mint an undecodable `mnem`-tagged card)

**Grade:** Low · **Category:** other · **Status:** Reviewer-confirmed, not re-verified

**Reviewer write-up** (source: `cross`)

### X8. Smaller pin and label drift

- **Severity:** Low. **Status:** CONFIRMED by reading.
- `mnemonic-gui/pinned-upstream.toml:42` pins `md-cli-v0.21.0`; md-cli 0.21.1 is released (`e11b4ce`) with four
  fixes, among them `md encode` minting an undecodable card and duplicate `--key` handling. The comment block above
  it (`:32-41`) still describes v0.20.3 and v0.11.0.
- mk-codec < 0.5 draws a random `chunk_set_id`; the toolkit avoids it for bundles by pinning
  `top20(policy_id_stub) ^ slot` (`synthesize.rs:77-99`), but `word_card_adapter.rs:165` calls plain
  `mk_codec::encode`, and the toolkit's rule differs from mk-codec >= 0.5 / the SeedHammer port
  (`top20(sha256(bytecode))`). The same key card therefore has different strings from `mnemonic bundle`, `mk encode`
  and the device. Decoders accept all three (documented in mk-codec's CHANGELOG as intended); a plate compared
  character by character against another tool's output will not match.
- ms-codec 0.7.0 `encode(Tag "mnem", Payload::Mnem)` returns `Ok("ms10mnems...")`, a card no version decodes
  (`probe-tk ms-enc-mnem 3 <hex> mnem`, then `ms-dec` gives "unknown tag "mnem"" on 0.7.0, 0.9.0 and HEAD). Fixed
  in 0.8.0. The toolkit only ever passes `Tag::ENTR` (`slot_ms1.rs:100,108,162`, `synthesize.rs:395,575,1169`,
  `convert.rs:1452`, `ms_shares.rs:314`), so it is not reached.
- mk-cli resolves md-codec 0.42.0 from crates.io (documented hazard in `mnemonic-key/CLAUDE.md`): it predates wire
  version 8 (0.46.0), so `--from-md1` of a Liana-unspendable card is refused there. Not re-tested.

---

_Generated by [Claude Code](https://claude.ai/code) on 2026-11-25. Findings were not fixed; each fix needs its own review under this repository's gates._

---

## Addendum: the 2026-11-25 bug hunt compared with the 2026-11-19 bug hunt

_Written 2026-11-26 for the coordinator that holds the 2026-11-19 fix queue. Read-only comparison: nothing here was merged or pushed to a default branch._

_This is the per-repo copy. The full addendum, with every repo's 11-19 table, is appended to `BugHunt-2026-11-25-fable.md` in mnemonic-refugium (report PR #247, branch `claude/project-thread-ly86x4`)._

### Why this exists, and how to use it

Two hunts ran a week apart:

- **2026-11-19** (OpenAI Codex, gpt-6). It covered 7 repos: descriptor-mnemonic (md), mnemonic-engrave (me), mnemonic-key (mk), mnemonic-secret (ms), mnemonic-toolkit (tk), mnemonic-transaction (mt) and seedhammer (sh). The reports are `design/agent-reports/bug-hunt-2026-11-19.md` on each default branch. The keys are `bh-<repo>-<id>`.
- **2026-11-25** (Fable). It covered all 11 repos. Each repo has a `BugHunt-2026-11-25-fable.md` report in an open, add-only PR on branch `claude/project-thread-ly86x4`. The fixes are in a second set of open PRs on branch `claude/project-thread-utm1aa`, one per repo.

Before you run any queued 11-19 item:

1. Look the key up in the per-repo tables below. Each table says where the fix is (the merged PR on the default branch), whether the 11-25 hunt found that bug again or a related one, and what is still left to do.
2. If a row's last column says "nothing", the item is done. Don't redo it.
3. Every 11-25 fix branch starts from the current default branch (checked 2026-11-26: 0 commits behind in all 10 repos). So every 11-19 fix merged so far is already underneath the 11-25 fix PRs, and the 11-25 PRs do not undo any of them.

#### The 11-25 PRs

| Repo | Report PR (11-25 findings) | Fix PR (11-25 fixes) |
|---|---|---|
| descriptor-mnemonic | #75 | #76 |
| mnemonic-engrave | #67 | #68 |
| mnemonic-key | #42 | #43 |
| mnemonic-secret | #47 | #48 |
| mnemonic-toolkit | #226 | #227 |
| mnemonic-transaction | #31 | #32 |
| seedhammer | #28 | #29 |
| mnemonic-refugium | #247 | #248 |
| mnemonic-gui | #57 | #58 |
| refugium-wallet | #228 | #229 |
| rust-miniscript | #1 | none (it has only Info findings) |

All of these PRs were open and unmerged when this addendum was written.

#### How the 11-19 fix status was worked out (and its limits)

- The starting point is the closure record `coordinator/agent-reports/recon-bughunt-1119-closure.md` in mnemonic-refugium (2026-11-24). It maps 125 Critical/Important keys to 58 FIX rows and found 27 of them open and 6 unrouted.
- After that record was written, more PRs merged on 11-24 and 11-25. I matched them to keys by the key names their commit titles cite: md#70, #71, #74; mk#37, #38, #39; ms#41, #42; tk#211, #212, #214, #220; mt#26, #27, #29; me#64; sh#25. With those, **every 11-19 Critical/Important key that had a FIX row is now merged.** What is still open are keys that were graded Minor and filed as FOLLOWUPS lines.
- "Merged" means the PR that names the key is on the default branch. I did not re-run each key's reproduction against the merged code.
- The closure record lists ms "I5" as the stdin-alias bug, but that bug is ms M5 in the source report. Source I5 is "verify calls a Japanese Mnem card v0.1 entr". Both are open FOLLOWUPS lines (see the ms table).

### Summary of the overlap

The 11-25 hunt has 50 distinct findings. Compared with the 11-19 hunt:

- **6 are the same bug as an 11-19 finding.** In every case the 11-19 fix merged, but only in one place: one repo, or Rust and not the Go port. The 11-25 hunt found the same bug somewhere else.
- **5 are related, and I'm not sure whether they count as the same bug** (marked UNCERTAIN).
- **About 39 are new.** That includes all 16 findings from the four repos the 11-19 hunt did not cover: mnemonic-gui, mnemonic-refugium, refugium-wallet and rust-miniscript.
- **None of the 11-19 bugs that are still open was found again on 11-25.**

| 11-25 finding | 11-19 key | Match | 11-19 fix (merged) | Where the bug was still present on 11-25 | 11-25 fix PR (open) |
|---|---|---|---|---|---|
| X7 (md-codec pin before 0.48.6 can mint a card the decoder refuses) | bh-md-C2 | SAME | md#59, released in md-codec 0.48.6/0.48.7 | consumers' md-codec pins (engrave, refugium) | engrave#68 `fc5f7c5`, refugium#248 `f8a0d9b`: both pin descriptor-mnemonic `c8cde23` |
| E5 (engrave's mt-codec pin) | bh-mt-I18 | SAME | mt#21 | engrave's mt-codec pin `eec7910b` | engrave#68 `67c3f1c`: pins mt `9b094e0b` |
| X4 (WIF slot writes a depth-0, zero-chain-code xpub onto cards) | bh-mk-C1 | SAME | mk#30 (mk-cli; not released yet) | toolkit mints these cards; md and the Go port derive from them; GUI and installer pin mk-cli 0.17.0 | tk#227 `59057e10` (refuses a WIF slot in template mode), md#76 `59c158e` (md refuses it). Still to do: release mk-cli and move the GUI/installer pins (gui#58 records that no release exists yet) |
| SH#6 (Go: change list equals receive list) | bh-md-I6 | SAME, Go port | md#64 (Rust) | seedhammer `address/`, `gui/` | seedhammer#29 `c2326a3` |
| SH#4 (Go `mk.Encode` lacks the origin refusals) | bh-mk-I9 (same function as bh-sh-I10) | SAME, Go port | mk#29 (Rust, mk-codec 0.6.0); sh#13 fixed only the network check | seedhammer `mk/encode.go` | seedhammer#29 `19b6075` |
| md#3 (placeholder with no wildcard minted as `/*`) | bh-md-C1 | SAME root (same lexer line, `parse/template.rs:345`) | md#60 fixed only the `md decompose` route | md template lexer (encode/address/descriptor) | md#76 `59c158e` and `6604c88` (bare `@N` refused at mint) |
| toolkit#3 (Word Card parity budget below what is stated) | bh-tk-I6 | UNCERTAIN: same weakness (one wrong word loses the card), different positions (header and checkpoint words today; the stop-sign word on 11-19) | tk#212 (stop-sign recovery) | header and checkpoint words | tk#227 `e6fd50c8`, `573dc14f`: the help and manual now state the budget the decoder really has. The decoder itself is unchanged |
| md#4 (`c:` over a non-key child mints a card md cannot restore) | bh-md-I5 | UNCERTAIN: same class (the encoder accepts what restore refuses); md#62 added the mint-time self-decode, but that check does not type-check | md#62 | md-codec mint check | md#76 `f8b4a67`, `ca48a05`, `5ad5a9b`, `fcefa83` (type rule at mint; address/descriptor refuse invalid miniscript unless `--allow-invalid-miniscript`) |
| SH#3 (Go `md.Decode` accepts four shapes Rust refuses) | bh-md-C2 | UNCERTAIN: one of the four shapes (an override keyed on baseline `@0`) is the card C2 used to mint | md#59 (Rust encoder) | Go decoder | seedhammer#29 `216a8af` |
| mk#2 (`mk verify --from-md1` passes a non-cosigner) | bh-mk-I2, I15 | RELATED: encode got the membership check, verify did not | mk#31 | `mk verify` | mk#43 `015d106` |
| toolkit#6 (`--account` >= 2^31 panics in six commands) | toolkit FOLLOWUP `xpub-search-account-window-2p31-panic` (listed as "previously filed" in the 11-19 tk report) | RELATED: same panic class; the earlier fix covered xpub-search only | tk#214 | bundle, verify-bundle, export-wallet, addresses, restore, convert | tk#227 `2d0057e8` |

Not an 11-19 match, but worth knowing:

- X6 (non-ASCII characters read as codex32 symbols by older ms/mk codec pins) repeats a bug from the **October 31** hunt that was fixed in the codecs. mk#43 `34c69b8` covers mk-cli's own md1 input. The consumer pins are tracked by the 11-25 X6/X8 entries.
- toolkit#7 (legacy `sh(sortedmulti)` defaults to BIP-48 `1'`) sits next to the 11-19 "previously filed" toolkit line `xpub-search-sh-wsh-default-origin-notice`, but it is a different wrapper and a different bug. It is fixed by tk#227 `6c99456c` plus the BIP-defaults follow-ups.
- engrave E3 (`--as md1` accepts a contradictory origin) is the md1 route of the ruling in `FIX_sec_canonical_keeps_xpub_depth` (me#54, 11-23). That ruling was not an 11-19 key. engrave#68 `094979e` fixes it.

### Work that the 11-25 PRs create for later

These items follow from the 11-25 fixes. Check the coordinator's queue against them so nothing is done twice:

- **md-codec 0.49.0 re-pins.** md#76 changes minting (explicit origins on keyed cards, the bare `@N` refusal, the type rule, thresh "any k of n"). After it merges, mnemonic-refugium, mnemonic-engrave (their md-codec pins must stay equal), mnemonic-toolkit, refugium-wallet and the seedhammer port each need a re-pin or a check. engrave#68 and refugium#248 pin `c8cde23` (0.48.7), not 0.49.0.
- **mk-cli release.** The bh-mk-C1 fix (mk#30) is still unreleased. gui#58 and the toolkit installer still pin `mk-cli-v0.17.0` (11-25 X4, G5).
- **seedhammer provenance pins.** seedhammer#29 `dde21fe` re-pins md, mk and mt to what the ports converge to (SH#5). Re-check them after md#76 merges.

### mnemonic-secret (ms): 11-19 keys

Source: `design/agent-reports/bug-hunt-2026-11-19.md` on `master`. 11-25 fix PR = ms#48. No 11-25 ms finding matches an 11-19 key.

| Key | 11-19 finding | Fixed by (merged) | 11-25 overlap | Left to do |
|---|---|---|---|---|
| C1 | argv override rewrites input and output filenames | ms#32 | none | nothing |
| C2 | The equals spelling hashes a different phrase | ms#32 | none | nothing |
| I1 | Report/digest stream overwrites a named backup | ms#42 | none | nothing |
| I2 | Checksum constructor returns an unreadable card | ms#35 | none | nothing |
| I3 | Raw seed constructor accepts what its reader refuses | ms#35 | none | nothing |
| I4 | A requested hashlock record can disappear | ms#34 | none | nothing |
| I5 | Verify calls a Japanese Mnem card "v0.1 entr" (graded M) | open: FOLLOWUPS `ms-verify-mnem-labelled-entr` | none | as FOLLOWUPS says |
| I6 | File-input error prints an unusable command | ms#41 | none | nothing |
| I7 | Cargo cannot package for the advertised route | ms#37 | none | nothing |
| I8 | Repair instructions give the wrong status | ms#37 | none | nothing |
| I9 | manpath example points at the filesystem root | ms#41 | none | nothing |
| I10 | Reproducibility proof points at missing scripts | ms#37 | none | nothing |
| I11 | aarch64 recipe can package the wrong archive | ms#37 | none | nothing |
| I12 | Inspect approves impossible share lengths | ms#36 | none | nothing |
| I13 | A failed replacement destroys the previous backup | ms#42 | none | nothing |
| M5 | Verify's single-stdin check misses a file alias (the closure record calls it "I5") | open: FOLLOWUPS `ms-verify-stdin-alias-slips-guard` | none | as FOLLOWUPS says |
| M1-M4, R1-R13 | Minors and reproduced prior filings (R11 fixed in ms#41) | FOLLOWUPS lines filed 2026-11-24 (`08ea22c`) | none re-found | as FOLLOWUPS say |

11-25 findings here (ms#1-#3, X6, X8) have no 11-19 link.

