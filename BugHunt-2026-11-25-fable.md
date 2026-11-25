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
