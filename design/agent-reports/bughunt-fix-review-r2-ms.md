r2: 0C/1I/2M/2N

# FIX review r2: `design/FIX_bughunt_ms.md` at `4cf1645` (after fold r1, `05b4c64`)

Reviewer: independent, read-only on `bughunt/triage`. Brief:
`coordinator/briefs/bughunt-ms-fix-review-r2.md`. Class A+B slice: executed
advice lens, create-input table, transcription fidelity. Tools under test:
`mnemonic 0.105.1` built from toolkit `c2b09861` (own `CARGO_TARGET_DIR`), `ms`
built from `4cf1645` plus reference patch r3 (applies cleanly), cargo 1.85.0
(the pin; `~/.cargo/bin` first). Scratch worktrees and targets are removed.
Harness and raw outputs: `/scratch/cache/bughunt/r2run/`.

## Verdict

The fold is a faithful transcription and its warning line is true. One
Important remains, in the advice text of 5.6.6: the leave-out command fails
for a card typed in capitals, and the fork's engraved `ms1` seed plate IS in
capitals. It fails closed (exit 2), and its hint then points the holder at the
wrong remedy. It is carried unchanged from `9e0fda8` and from review r1's
replacement text, so it is [pre-existing]; the fold added nothing wrong to it.
No route I ran (about 1,000 invocations) printed a wrong secret or a wrong
verdict.

## Transcription fidelity (machine-compared)

| item | result |
|---|---|
| 5.6.6 New block vs review r1 "Fix text for I-1, I-2 and I-3" | byte-identical, occurs once in the FIX and once in the script's ninth edit |
| architect edits 1-10 | all applied, texts equal the ruling (checked line by line against `git diff 7400dfd 05b4c64`); edit 6's bullet placement (between `emit_ms1` and Constraint) is fine |
| architect fragment in 5.1 and in script edit 1 | present once in each, identical |
| 10 script edits vs FIX section 5 | every old and new text found in the FIX; script sha256 `f1bd63ae…` matches the header |
| r3 patch | sha256 `26c20263…` matches; the lines r3 adds over r2 are exactly the 33 lines of the delta; delta sha `66a4316e…` equals the architect's diff block byte for byte; `git apply` of r3 on `4cf1645` succeeds |
| warning line W in FIX 2.2 | identical to the line the patched `ms` prints (diff clean) |
| nine new 9.2 entries vs review M-1..M-7, N-1, N-2 | fix texts and measurements match; owning phases as the fold report says |
| patched tree | `cargo nextest run --locked --workspace -E 'binary(bughunt)'`: 21 run, 21 passed, 9 skipped (matches FIX section 6) |

## Check 1: create-input table

| command (where printed) | real interface | result |
|---|---|---|
| `ms combine --to entropy -` / `--json` / `--to ms1` on a preimage share set (2.2, W) | `--to`, `--json`, `--in`/`-`; shares from the library (the CLI has no preimage split) | text: stdout 64 hex, stderr W then advisory, exit 0; `--json`: advisory alone; `--to ms1`: `ms10hash…`, no W |
| W's own commands: `ms encode --hex -`, `ms split --hex -`, `ms derive --hex -` | all three take `--hex`; fed the preimage hex they mint an `entr` card, shares, fingerprint `b26a00b0` (W's premise is true) | run |
| W's remedy `--to ms1`; `ms hashlock --kind sha256 --in plate` and `--hex -` piped from `--to entropy` | | prints the plate; `hash:02d449a3…` both ways |
| `mnemonic repair --ms1 - --max-indel N [--max-subst S] < card.txt` | `--max-indel` 0..=4, `--max-subst` accepted with it; `-` = stdin | runs; `--max-indel 5` is exit 64 |
| `--mk1 -`, `--md1 -` | repeating, one chunk per line | needs every string (as the text says) |
| `ms repair --in card.txt` | `--in FILE`; strips display separators | runs, grouped input too |
| `mnemonic repair --mk1 - < card.txt`, `--md1 -` | | runs |
| `ms combine` (named, no args) | | runs on two clean shares, exit 0 |
| `ms hashlock --kind sha256 --hex - --out hash.ms1` (2.6) | | writes the 75-character plate |

## Check 2: case table for 5.6.6 (route 1 = leave out; route 2 = type a guess)

Trials: 5 to 8 per cell, random positions after `ms1`, 1 to 4 characters.
"Recovered" means the original string was in stdout.

| card (line quoted) | route 1 | route 2 | text says | agree |
|---|---|---|---|---|
| `ms1` entr 16/32 B ("its one `ms1` string") | exit 5, recovered n=1..4 | `ms repair --in`: exit 4, n=1..4 | "unique recovery ... exits 5"; "A corrected `ms1` is a candidate, exit 4" | yes |
| `ms1` mnem 16/32 B | exit 5; exit 4 candidate when typo + `--max-subst 1` | exit 4 | same | yes |
| hashlock plate "(`ms10hash…`)" | exit 2 for n=1..4, also with `--max-subst 1` | `ms repair --in`: exit 4, n=1..4; `mnemonic repair`: exit 2 (`reserved-prefix byte was 0x03`) | "does not work for a hashlock preimage plate ... use the second way"; names `ms repair` | yes |
| K-of-N share ("starts `ms12` to `ms19`") | exit 2 | `ms repair` and `mnemonic repair`: exit 2 ("one share of a K-of-N set") | "Neither way works yet ... exit 2"; `ms combine` of K clean shares exit 0 | yes |
| `mk1` (2 strings, every string in the file) | exit 5, n=1..4 (8/8 each) | `mnemonic repair --mk1 -`: exit 5 | "exit 5 means the correction checked itself" | yes |
| `md1` chunked (3 strings) | exit 5, n=1..4 (8/8 each) | exit 5 | same | yes |
| `md1` one string | exit 2 for every n | `mnemonic repair --md1 -`: exit 4, n=1..4 | "nor for ... `md1` card of a single string"; "exit 4 means it could not" | yes |
| left out + one typo ("run the command again with `--max-subst 1`") | no `--max-subst`: exit 2 in 71 of 72 trials (ms1 entr/mnem, `mk1`, chunked `md1`; the other was indel-equivalent, exit 5); with it: 72 of 72 recovered, exit 4 | n/a | "usually exits 4, a candidate" | yes |
| budget "at most 4" (left out d, wrong e, `--max-indel d --max-subst e`) | d+e=4: (1,3) (2,2) (3,1) recovered 12/12 each on entr16, entr32, mnem16; d+e=5: 16 of 48 recovered, the rest exit 2, never a wrong string | n/a | "may be at most 4" | yes (no wrong output at 5 either) |
| engraved style: capitals, groups of ten | **exit 2 for ms1, mk1, md1; also with `--max-subst 1`**; the same card retyped in lowercase: exit 5 | works (capitals, mixed case) | no caveat | **no: I-1** |
| characters left out of two strings of one `mk1`/`md1` card | one char out of each string: exit 2 at `--max-indel 1` and `2` | one guess wrong in each string: exit 5 | scope "in one string" | M-2 |
| left out at the ends or inside `ms1`: positions 0, 1, 2, 3, last | exit 5 each | | | yes |

No trial printed a string other than the original (stdout lacking the original
with exit 0, 4 or 5): zero in the final runs. (A first harness pass flagged
mk1/md1 route 1 as wrong because that route prints only the repaired string;
a harness error, fixed and rerun.)

## Check 3: advice lens, executed

W and 5.6.6 followed literally on signet test material (`000102…0f` entropy,
preimage `0x11 x 32`, the `abandon … about` bundle). Every printed command
runs; none silently yields a wrong secret or verdict. One case fails as printed
(I-1). W's advice is true in every case it can reach: it fires only on text-mode
`--to entropy` of a preimage set; the remedy `--to ms1` prints a plate that
`ms hashlock --in` reads; piping the hex to `ms hashlock --hex -` still works;
the entr control (same bytes, kind entr) prints no W.

## Check 4: FIX 2.6 re-run (every `$` command replayed from the doc's own text)

32 command replays (both `ms` builds where relevant). 27 match exactly. The
"current" versus "after the fix" `ms repair` pair differs by design (an
installed or an unpatched `ms` against the patched one; matches on the right
build). The only real mismatches are the three `mk1` blocks (M-1 below). The
five `md1` blocks, the `ms1` blocks, the hash plate, the share, the one-string
`md1` and the 5-left-out block all reproduce exit codes, stdout and stderr.

## Findings

### I-1 [pre-existing; carried by the fold] 5.6.6's leave-out command fails on a card typed in capitals, and the fork's engraved `ms1` seed plate is in capitals

**Where:** FIX 5.6.6 New block (and the ninth edit of
`toolkit-manual-edits.py`): the first bullet, "`mnemonic repair --ms1 - --max-indel 1 < card.txt`",
and its sentence "Exit 2 means nothing was found: if the string may also hold a
wrong character, run the command again with `--max-subst 1` added". The second
bullet's "in the same case as the rest" shows the text knows capitals exist.

**Failure scenario (executed).** A holder reads an engraved `ms1` seed plate:
the SeedHammer fork engraves it upper-cased and grouped in tens
(`mnemonic-engrave/third_party/seedhammer/backup/hashlock_test.go:147`, "The
fork's ms1 SEED plate upper-cases and groups in tens"; `backup.go:159`
`strings.ToUpper(plate.Seed)`). One character is unreadable; they leave it out
and type the card as read, exactly as the text says ("Put the card in `card.txt`
as you read it"):

~~~text
$ cat card.txt                      (the entr16 card in capitals, grouped in tens, character 15 left out)
MS10ENTRSQ QQQZSRQSZS VPCGPY9QKR QDPC8SH9LF 92N5FR33S
$ mnemonic repair --ms1 - --max-indel 1 < card.txt
stderr: error: repair: chunk could not be recovered within --max-indel 1 (HRP 'ms'); the string may have more than 1 inserted/dropped characters, or a different error class
exit=2
~~~

`--max-indel 1 --max-subst 1` also exits 2. The same card retyped in lowercase
(`ms10entrsq qqqzsrqszs vpcgpy9qkr qdpc8sh9lf 92n5fr33s`) exits 5 and prints
`ms10entrsqqqqzqsrqszsvpcgpy9qkrqdpc8sh9lf92n5fr33s`. Same for `md1` (three
strings in capitals, one character out) and `mk1` (two strings): exit 2 in
capitals, exit 5 in lowercase. Route 2 is unaffected (capitals, and mixed case,
repair fine with `ms repair --in` and `mnemonic repair`).

Why it is Important and not Minor: the printed command fails as printed on a
card form the text covers, and the likelihood test's second condition (no
printed command fails as printed) does not hold. It fails closed (exit 2, no
wrong string), but the text's own hint then sends the holder to `--max-subst 1`,
which cannot help, so the holder concludes the plate is unrecoverable when route 2
would have worked. The same class as r1's I-1 (hash plate), one input form
further. The tool behaviour (the indel search does not fold case) predates this
branch; the text here does not tell the holder about it.

**Fix (exact).** In FIX 5.6.6 New block, and in the ninth edit of
`toolkit-manual-edits.py` (same text; re-record the script's sha256 in the FIX
header), replace the last sentence of the first bullet:

Old:
~~~text
  candidate you confirm before use. This search does not work for a
  hashlock preimage plate (`ms10hash…`), nor for every `md1` card of
  a single string; use the second way for those.
~~~

New:
~~~text
  candidate you confirm before use. This search reads lowercase only:
  if your plate is engraved in capitals, type the card in lowercase
  for this command (the second way takes either case). It does not
  work for a hashlock preimage plate (`ms10hash…`), nor for an `md1`
  card of a single string; use the second way for those.
~~~

(The "every" to "an" change is bundled: "does not work for every md1 card of a
single string" can be read as "works for some", N-2.) Add to 2.6, after the
5-left-out block and before "The limits stated in 5.6 were measured":

~~~text
The leave-out search with a card typed in capitals, as an engraved `ms1` plate reads (grouped in tens, character 15 left out), then the same card in lowercase:

$ cat card.txt
MS10ENTRSQ QQQZSRQSZS VPCGPY9QKR QDPC8SH9LF 92N5FR33S
$ mnemonic repair --ms1 - --max-indel 1 < card.txt
stderr: error: repair: chunk could not be recovered within --max-indel 1 (HRP 'ms'); the string may have more than 1 inserted/dropped characters, or a different error class
exit=2

$ cat card.txt
ms10entrsq qqqzsrqszs vpcgpy9qkr qdpc8sh9lf 92n5fr33s
$ mnemonic repair --ms1 - --max-indel 1 < card.txt
stdout: ms10entrsqqqqzqsrqszsvpcgpy9qkrqdpc8sh9lf92n5fr33s
stderr: warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
exit=5
~~~

(Wrap that block in the section's usual `~~~text` fence.) Add one 9.2 entry,
`toolkit-repair-indel-search-rejects-capitals` (repo: **mnemonic-toolkit**;
owning phase: the toolkit's next `mnemonic repair` change; status
`deferred-minor`, tier text/behaviour, non-gating): "`mnemonic repair
--max-indel` exits 2 on an all-capitals `ms1`, `mk1` or `md1` card that the same
tool repairs without `--max-indel`; the primer tells the holder to retype in
lowercase (FIX 5.6.6)." No code change in this PR.

## Minor (to `design/FOLLOWUPS.md`, `deferred-minor`)

- **M-1 [fold] 2.6's three `mk1` card displays are truncated at 80 columns, so
  the transcripts do not reproduce as printed.** FIX lines 527, 541 and 552
  print the first `mk1` string as 80 characters; the real string is 111. Run
  with the cards as printed at 527 and 541, the three commands that read them
  exit 2 (`could not be recovered` and, for the guess, `too many errors ...
  singleton bound = 8`), not the recorded 5, 4 and 5. Line 552 is truncated
  too, but its recorded exit 2 is the same either way. With the full strings every recorded result reproduces
  (exit 5, exit 4 with `--max-subst 1`, exit 5 with the `'2' -> '9'` correction),
  so the 5.6.6 text is right and only the evidence is mis-typed. Conditions:
  evidence section only, no holder text, and the next reader who re-runs 2.6
  sees a false failure. Fix: in those three lines append
  `zpn3glvvquvgpms6mzamgctgratycf` to lines 527 and 552 (they end `…rfg8rw7dk`)
  and `kzpn3glvvquvgpms6mzamgctgratycf` to line 541 (it ends `…rfg8rw7d`). The
  full lines are
  `mk1qpg397pqqsq5gjlzeeutks2zvzrtp70pm6trteu3ssr0mjvzcsa5rf53k63zcmsnf3z6rfg8rw7dkzpn3glvvquvgpms6mzamgctgratycf`
  (527, 552) and
  `mk1qpg397pqqsq5gjlze2eutks2zvzrtp70pm6trteu3ssr0mjvzcsa5rf53k63zcmsnf3z6rfg8rw7dkzpn3glvvquvgpms6mzamgctgratycf`
  (541).
- **M-2 [pre-existing] Characters left out of two strings of one `mk1`/`md1`
  card: the leave-out command exits 2, and the text is silent.** One character
  out of each of the two `mk1` strings: `--max-indel 1` and `--max-indel 2` both
  exit 2. One string left out and a (wrong) guess typed in the other works
  (exit 5; exit 4 with `--max-subst 1`), and route 2 with one wrong guess in
  each string works (exit 5). Fails closed; the text's scope is "in one
  string". Conditions: an `mk1`/`md1` card, two of its strings each with an
  unreadable character, and the leave-out route chosen. Fix, after "(not only
  the damaged one)." in 5.6.6: "If unreadable characters are in more than one
  string of an `mk1` or `md1` card, leave characters out of one string only and
  type a best guess for the others."

## Nit

- **N-1 [fold] 2.6's `ms split` transcript is not what `ms split` prints.**
  The doc shows `share 1: ms12h997qq65…`; the tool prints all N shares on
  stdout (grouped in fives) and `share i of N:` labels on stderr. The shares
  used afterwards are valid (the `ms combine` block reproduces). Fix: show
  `$ ms split …` as three stdout lines of share strings and note that the
  `share i of N:` labels are on stderr, or mark the line "(abridged)".
- **N-2 [pre-existing wording]** "nor for every `md1` card of a single string":
  "every" reads as "not all". Bundled into the I-1 fix text ("an").

## Check 5: deferred minors in touched text

The fold changed no existing 9.2 entry (it appended nine). Re-read against the
changed text: `ms-repair-accepts-mixed-case-with-typo` (the tool repairs a
lowercase guess inside a capitals card, measured, exit 4; 5.6.6's "same case as
the rest" is only stricter), `ms-decode-calls-punctuation-mixed-case` (5.6.6
says do not type `?`; both repair tools refuse it, exit 1 and 2) and the new
`fix-primer-insert-instruction-eats-heading` (the script is right and the build
applies the script). All stay Minor. None is C or I.

## Fold accounting

From fold r1 (`05b4c64`, Sonnet xhigh): 0C/0I/1M/1N (M-1, N-1; both in the 2.6
transcripts). Carried through it unchanged from the earlier text or from review
r1's replacement: 1I (I-1), 1M (M-2), 1N (N-2).

## Machine-checked by this review (so the controller need not)

Programmatic comparison of the 5.6.6 block, the ten script edits, the delta,
the warning line and the shas; 32 transcript replays; about 1,000 repair
invocations across 9 card kinds (random positions, n=1..4, both routes, typo
plus `--max-subst`, budget bound, ends of the string, capitals, grouped input);
`binary(bughunt)` 21/21 on the patched tree. Not re-run: the full workspace
nextest, clippy, fmt and `make lint` (the fold gate has them; the controller
re-checked the starred items).

## Design concern (ungraded)

`mnemonic repair --max-indel` folding case in the tool would remove the I-1
caveat from the primer.
