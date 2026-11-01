r3: 0C/0I/1M/2N

# FIX review r3: `design/FIX_bughunt_ms.md` after fold r2 (`e67eb6a`, Haiku high)

Reviewer: independent, read-only on `bughunt/triage` at `6cae352`. Brief:
`coordinator/briefs/bughunt-ms-fix-review-r3.md`. Class A+B slice: transcription
fidelity of fold r2 and the executed advice lens on the changed 5.6.6.
Tools under test: `mnemonic 0.105.1` built in a scratch worktree at toolkit
`c2b09861` (own `CARGO_TARGET_DIR=/scratch/cache/bughunt/r3-tk-target`; the
version string carries no commit, so the build from `c2b09861` is the
provenance; worktree removed, target left); installed `ms 0.20.1`; and, for the
same matrix again, the patched `ms` the r2 review left at
`/scratch/cache/bughunt/r2-target-ms/debug/ms` (built from `4cf1645` plus
reference patch r3; I did not rebuild it). Harness and outputs:
`/scratch/cache/bughunt/r3run/` (`harness.py`, `harness.out`,
`harness-patched-ms.out`, `cmp.py`).

## Verdict

Fold r2 is a faithful transcription, and the changed 5.6.6 holds for every card
form it covers, capitals included. No printed command fails as printed once the
text is followed to the end of the bullet, and no route printed a wrong secret
or verdict in about 1,000 invocations. One Minor (the caveat's position) and two
Nits; nothing blocks.

## Check 1: transcription (machine-compared with `python3 -I`, `cmp.py`)

| item | result |
|---|---|
| I-1 New text vs review r2 "Fix (exact)" New | byte-identical; occurs once in the FIX and once in the script |
| I-1 Old text | gone from FIX and script; present in the pre-fold script (`diff` of backup vs edited script: exactly the 3-line to 5-line change, nothing else) |
| whole FIX 5.6.6 New block vs script ninth edit | the FIX block occurs verbatim once in the script |
| script sha256 | `e56d598ec4c486be260bbe5364eafbfa9ab4ffe32e57daaf9b8427cc164124ee` = the FIX header; the old `f1bd63ae` no longer appears in the FIX |
| 2.6 block vs review | prose line above the fence and the fenced block are byte-identical to the report's block, placed after the 5-left-out block and before "The limits stated in 5.6 were measured", per the coordinator's layout decision |
| `git diff 4e68fb5 e67eb6a` | one file (`design/FIX_bughunt_ms.md`), 62 insertions, 4 deletions; no stray edit |
| N-2 | closed by the I-1 text ("every" to "an"); no 9.2 entry, as decided; commit message says so |
| 9.4 entry `toolkit-repair-indel-search-rejects-capitals` | heading verbatim from the coordinator's decision; fields in 9.4's order; status and tier as decided; body is the report's quoted text plus one sentence (N-1) |
| 9.2 entries M-1, M-2, N-1 | headings verbatim; fix texts, measurements and counts equal the review's; M-1's line numbers (527, 541, 552) are the truncated lines and the full-line fix strings reproduce (below) |

M-1's measurements re-run: lines 527/528 as printed, `mnemonic repair --mk1 -
--max-indel 1` gives `chunk could not be recovered` (exit 2); lines 541/542 as
printed give `too many errors ... singleton bound = 8` (exit 2); with the full
strings the entry gives, both recover (exit 5 and the `'2' -> '9'` correction).
The entry's full 527 string matches the entry's claim exactly.

## Check 3: run what you print (the fold's added 2.6 commands)

Both new commands re-run on the `c2b09861` build with the card contents the
block shows. Stdout, stderr and exit equal the block's (capitals: exit 2 and the
`--max-indel 1` message; lowercase: `ms10entrsqqqqzqsrqszsvpcgpy9qkrqdpc8sh9lf92n5fr33s`,
the private-key warning, exit 5). The capitals card is the entr16 card with its
character 15 left out, as the prose says.

## Check 2: advice lens, executed (5.6.6 as it now reads)

Cards: `ms1` entr16, entr32 and mnem16 (minted with `ms encode`), the `mk1` card
of two strings and the chunked `md1` card of three strings (2.6's own cards, each
confirmed clean by `mnemonic repair`, exit 0), plus the single-string `md1`, a
hashlock plate and a K-of-N share set for the cases 5.6.6 names. Each typed in
capitals grouped in tens, capitals plain, lowercase grouped, lowercase plain
(mk1/md1: one string per line). "ok" = stdout holds only original strings and
the damaged card's original (a harness error I made and fixed first: mk1/md1
leave-out prints only the repaired string).

| case (all four forms unless noted) | capitals | lowercase | text says | agree |
|---|---|---|---|---|
| leave one out, `--max-indel 1` (ms1 entr16 at all 47 positions; entr32 10; mnem16 8; mk1 13; md1 chunked 19) | exit 2 in every trial | exit 5, ok in every trial | capitals: "type the card in lowercase"; lowercase: "unique recovery ... exits 5" | yes |
| leave one out, plus `--max-subst 1` and no typo (the hint's step on a capitals card) | exit 2 in every trial | exit 4 or 5, ok | capitals still fails, hence the caveat | yes |
| one left out plus one typo, `--max-indel 1 --max-subst 1` | exit 2 in every trial | exit 4 in every trial (ms1 47+10+8, mk1 13, md1 19), ok | "such a recovery usually exits 4, a candidate" | yes |
| route 2, one wrong guess in the same case (`ms repair --in` for ms1, `mnemonic repair --mk1/--md1 -`) | ms1 exit 4, mk1/md1 chunked exit 5, ok | same | "the second way takes either case"; exits 4 and 5 as the text says | yes |
| single-string `md1` | leave-out exit 2 (all forms); route 2 exit 4, ok | same | "nor for an `md1` card of a single string"; exit 4 | yes |
| hashlock plate `ms10hash…` | leave-out exit 2 (`mnemonic repair`); `ms repair --in` with a guess exit 4, ok | same | "does not work for a hashlock preimage plate ... use the second way" | yes |
| K-of-N share in capitals, one guess | `ms repair` and `mnemonic repair` exit 2 ("one share of a K-of-N set"); `ms combine -` of two clean capitals grouped shares exit 0, right secret | lowercase `ms combine` exit 0 | "Neither way works ... `ms combine`" | yes |
| mk1/md1 with only the damaged string lowercased and the other strings left in capitals | exit 5 (exit 4 with `--max-subst 1`), ok | n/a | text says retype the card in lowercase; the tool needs only the damaged string | stricter text, harmless |
| mk1/md1 with the damaged string in capitals and the others lowercase | exit 2 | n/a | consistent with the rule | yes |

The patched `ms` (r2 build) gives identical results over the same matrix, and no
trial in either run printed a string that was not an original. The 9.4 entry's
"Measured" sentence is true: the capitals entr16 card with one character out
exits 2 at `--max-indel 1`, also with `--max-subst 1`; the same card in
lowercase exits 5.

## Findings

### M-1 [fold; placement dictated by review r2's exact text] 5.6.6 names the lowercase rule after the hint that cannot help a capitals card, and the preamble still says "as you read it"

**Where:** FIX 5.6.6 New block (and the ninth edit of the script): the preamble
"Put the card in `card.txt` as you read it", the code block, and the sentence
order "Exit 2 means nothing was found: if the string may also hold a wrong
character, run the command again with `--max-subst 1` added ... This search
reads lowercase only: if your plate is engraved in capitals, type the card in
lowercase for this command".

**What (executed).** A holder with an engraved capitals plate follows the
preamble ("as you read it", so capitals), runs the printed command (exit 2), and
reads the next sentence, which sends them to `--max-subst 1`; that also exits 2
on every capitals card (measured above). The rule that fixes it is four lines
later in the same bullet. Fails closed (exit 2, no wrong string); the text does
say what to do. Conditions (the likelihood test): (a) a capitals plate, which is
the fork's default for an `ms1` seed plate, so not unusual; (b) the holder runs
the command before reading the end of the bullet; (c) on exit 2 they follow the
`--max-subst 1` hint without reading the next sentence. (b) and (c) are two
unusual conditions, and the tool fails closed, so this is Minor; no printed
command fails once the bullet is read to its end.

**Fix (for later).** Deferred-minor, owning phase none (ownerless residue),
tier docs (non-gating). In 5.6.6, move the sentence so the rule precedes the
command. Delete from the end of the first bullet: "This search reads lowercase
only: if your plate is engraved in capitals, type the card in lowercase for this
command (the second way takes either case). " and in the bullet's first
sentence replace "Give `--max-indel` the number you left out (1 to 4; this
example left out 1):" with "Type the card in lowercase for this command, even
if your plate is engraved in capitals: this search reads lowercase only (the
second way takes either case). Give `--max-indel` the number you left out (1 to
4; this example left out 1):". Same edit in the script's ninth edit; re-record
its sha256.

## Nits

- **N-1 [fold] The 9.4 entry `toolkit-repair-indel-search-rejects-capitals`
  carries one sentence the report's quoted entry text does not:** "Measured: the
  engraved capitals entr-16 card ... exits 5." It is drawn from I-1's body, it is
  true (measured above), and the brief said the body is the report's quoted text.
  Not holder-visible; no action needed beyond noting that fold r2 added a
  sentence it was not given. Also the report's "No code change in this PR" moved
  from the What to the Why-deferred line; harmless.
- **N-2 [fold] FIX 5.4 (line 710) and the build list (line 943) still say "the
  toolkit FOLLOWUPS entry 9.4" / "the FOLLOWUPS entry 9.4" in the singular; 9.4
  now has two entries.** Wording only. Fix (for later): "entries of 9.4" in both
  places.

## Check: deferred minors in touched text (re-graded)

The fold added entries and changed no existing 9.2 entry. Re-read against the
changed 5.6.6 bullet: `ms-repair-accepts-mixed-case-with-typo` (route 2 "same
case as the rest": unchanged and still only stricter), `toolkit-ms-codec-bump-followup-text-incomplete`
(its "revisit that sentence at the bump" now points at a sentence with the
capitals clause before it; still true), `toolkit-primer-length-damage-bullet-contradicts-indel`
(its replacement bullet says `--max-indel N` searches for missing or extra
characters; it is followed by the cross-reference to this subsection, which
carries the lowercase rule), `primer-printed-commands-have-no-executed-pin`
(2.6 now has one more block). All stay Minor. None is C or I.

## Fold accounting

From fold r2 (`e67eb6a`, Haiku high): 0C/0I/1M/2N (M-1 above is the review's own
sentence order, placed as instructed; N-1 and N-2 are the fold's). The review r2
Important (I-1) is closed: the lowercase rule is in the text, the 2.6 evidence
reproduces, the tool half is filed as a toolkit companion entry. No
transcription deviation in holder-visible text.

## Machine-checked by this review

Programmatic comparison of I-1's Old/New, the 2.6 block, the whole 5.6.6 block
against the script, the header sha; two 2.6 command replays; M-1's three
truncated-line reproductions; about 1,000 repair invocations (matrix above,
twice, with two `ms` builds); K-of-N, hashlock and single-string `md1` cells.
Not re-run: `make lint` and `make anchor-check` (settled by the controller's gate
for this brief).

## Design concern (ungraded)

None beyond the filed toolkit entry: `mnemonic repair --max-indel` folding case
would remove the lowercase clause (and M-1) from the primer.
