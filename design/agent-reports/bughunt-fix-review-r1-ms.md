r1: 0C/3I/7M/2N

# FIX review r1: `design/FIX_bughunt_ms.md` at `9e0fda8` (ms bug-hunt fixes)

Reviewer: Opus xhigh, independent, read-only on `bughunt/triage`. Scope: the
whole document (sections 1-10), the reference patch
(`fix-bughunt-ms-reference-r2.patch`, sha256 `b1f7d1be…cd0e`, verified), the
manual script (`toolkit-manual-edits.py`, sha256 `08306322…aa09`, verified),
toolkit `c2b09861`. Brief: `coordinator/briefs/bughunt-ms-fix-review-r1.md`.

## Verdict

The ms code delta (sections 2-4, 6-9) is correct and Sonnet-proof. Every
after-fix printed line of section 2 matches the patched build byte for byte
(27 stdout lines checked mechanically, plus every stderr and exit code). The
four spec edits and the ten manual edits apply once each. `make lint` gives the
same 10 FAILs (all `44-mk-cli.md`) with and without the edits, and
`make anchor-check` passes (9 danglers, the baseline). The script's ten texts
are identical to section 5's.

The three Importants are all in the new primer subsection **5.6.6
"Unreadable characters: what to type"** (r2-amendment). Followed literally, its
printed commands fail (all fail closed, exit 2) for card kinds and cases the
text says it covers:

- a hashlock preimage plate,
- a K-of-N share,
- an mk1 or md1 card of several strings,
- an md1 card that is one unchunked string,
- a left-out character plus one typo.

No path I walked (about 1,750 trials) printed a wrong secret or gave a wrong
verdict. One replacement text below fixes all three Importants.

## Important

### I-1 [r2-amendment] 5.6.6 tells ms1 holders "Two things work". For a hashlock preimage plate and for a K-of-N share, the printed commands fail

**Where:** FIX 5.6.6, New block: "Two things work, for up to 4 unreadable
characters in one string", the command
`mnemonic repair --ms1 - --max-indel 1 < card.txt`, and
"`ms repair --in card.txt` (or `mnemonic repair`)".

**Failure scenarios** (measured on the installed `mnemonic 0.105.1` and
`ms 0.20.1`, and on the patched `ms`):

- **Hashlock preimage plate** (`ms10hash…`, 75 characters; `ms hashlock --out`
  writes it, and 5.1 itself names it):
  - With 1-4 characters left out, `mnemonic repair --ms1 - --max-indel k < card.txt`
    exits 2 every time (12/12, 12/12, 4/4, 4/4), with
    `error: repair: chunk could not be recovered within --max-indel 1 (HRP 'ms'); the string may have more than 1 inserted/dropped characters, or a different error class`.
  - The "(or `mnemonic repair`)" route exits 2 with
    `post-correction decode failed: reserved-prefix byte was 0x03, expected 0x00`.
    It exits 2 even on a clean plate.
  - Cause: the toolkit pins ms-codec 0.7.0 (`Cargo.lock`), which predates the
    `hash` kind.
  - Only `ms repair --in card.txt` works here: 48/48 recovered, exit 4.
- **K-of-N share** (`ms12…`, from `ms split`): every route exits 2.
  - `ms repair` (0.20.1 and patched):
    `error: this is one share of a K-of-N set (threshold '2', index 'q'); use `ms combine` to recombine K shares`.
  - `mnemonic repair --ms1 -`: the same, through "post-correction decode failed".
  - `--max-indel`: unrecoverable.
  - `ms combine` with the damaged share and a clean one:
    `BCH checksum invalid (short code)`, exit 1.
  - No tool corrects a share today, yet the text says two things work.

All of these fail closed. Each is still a printed command that fails as
printed for a holder inside the stated radius, so the likelihood test keeps it
at I.

**Fix:** the replacement text below. It scopes the leave-out search away from
the hash plate, sends the hash plate to `ms repair`, and states that a share
has no repair path but K other shares.

### I-2 [r2-amendment] 5.6.6's mk1/md1 instructions fail as printed: it says to put only "the string" in the file, a one-string unchunked md1 has no indel search, and route 2 has no runnable mk1/md1 command

**Where:** FIX 5.6.6: "Put the string, with those characters left out, in
`card.txt`", "`--mk1` and `--md1` take the same search", and route 2's
"`ms repair --in card.txt` (or `mnemonic repair`)" with "a corrected string is
a candidate, exit 4".

**Failure scenarios** (signet `abandon … about` bip84 bundle: mk1 is 2 strings,
md1 is 3 strings; plus `md encode 'wpkh(@0/<0;1>/*)'` = `md1yqpqqxqq8xtwhw4xwn4qh`):

1. **Only the damaged string in `card.txt`.** Read literally, "Put the string"
   means one string. Then `mnemonic repair --mk1 - --max-indel 1 < card.txt` and
   the `--md1` form both exit 2 (`could not be recovered within --max-indel 1`).
   The search needs every string of the card: with all of them it exits 5 (12/12
   for each). Route 2 with only the damaged md1 string exits 2
   (`chunk set incomplete: got 1 chunks, expected 3`).
2. **A one-string unchunked md1** (`md encode`'s short form) has no indel
   recovery. `--md1 - --max-indel k` exits 2 for every k (12/12, 12/12, 4/4,
   4/4). This contradicts "`--md1` take the same search"; `41-mnemonic.md`
   already documents it ("a non-chunked md1 indel … is not recoverable at all
   (exit 2)").
3. **Route 2 has no command for mk1/md1.** `ms repair --in card.txt` refuses
   mk1/md1 (wrong HRP). Bare `mnemonic repair` is not a command:
   - `mnemonic repair < card.txt` exits 64 (`required arguments were not provided`);
   - `mnemonic repair --in card.txt` exits 64 (`unexpected argument '--in'`).

   The working forms are `mnemonic repair --mk1 - < card.txt` and
   `--md1 - < card.txt`.
4. **"exit 4" is wrong for mk1 and chunked md1.** A corrected mk1 or chunked
   md1 card with all its strings exits **5** (12/12 at each k = 1-4), not 4.
   Only ms1 and a one-string unchunked md1 give exit 4. This is harmless on its
   own, but the text is wrong when followed literally.

**Fix:** the replacement text below.

### I-3 [r2-amendment] "(fewer if the string has other mistakes)" promises that the leave-out route tolerates other mistakes. The printed command tolerates none

**Where:** FIX 5.6.6: "Two things work, for up to 4 unreadable characters in
one string (fewer if the string has other mistakes)", followed by the command
`mnemonic repair --ms1 - --max-indel 1 < card.txt`.

**Failure scenario.** A holder leaves out 1 or 2 unreadable characters and
also has one ordinary typo in the string (2-3 errors, within the
placeholders + substitutions ≤ 4 budget). The printed command, which has no
`--max-subst`, exits 2 in 119/120 trials across ms1 entr16, entr32, mnem51,
mk1 and chunked md1 (`… or a different error class`). The one other trial was
a mnem k=2 run where the typo happened to be indel-equivalent; it recovered
the original, exit 5. With `--max-subst 1` added, the same kind of inputs
recover in 120/120 (exit 4). The text never names `--max-subst` except as a
reference pointer.

**Fix:** the replacement text below: "Exit 2 means nothing was found: … run
the command again with `--max-subst 1` added …".

### Fix text for I-1, I-2 and I-3 (one replacement)

In FIX 5.6.6, replace the whole New block (from `### Unreadable characters: what to type`
up to, not including, `## Hand-decodability of ms1 via the bundled codex32 paper-computer`)
with the text below. Make the same change to the ninth edit of
`toolkit-manual-edits.py` and re-record its sha256 in the FIX header.
Also re-run 2.6 with these commands. The matrix under "Advice lens" below is
my run of every command in the text. With this text in place, `make lint`
gives the same 10 FAILs, `make anchor-check` passes, and pandoc renders the
heading, both list items and the code block.

~~~text
### Unreadable characters: what to type

No repair tool decodes erasures yet, so do not type `?` for a
character you cannot read. Two things work, for up to 4 unreadable
characters in one string; any other wrong character in that string
counts toward the 4. Put the card in `card.txt` as you read it: its
one `ms1` string, or every string of an `mk1` or `md1` card, one per
line (not only the damaged one).

- **Leave each unreadable character out** and let `mnemonic repair`
  search for the gaps. Give `--max-indel` the number you left out (1
  to 4; this example left out 1):

  ```sh
  mnemonic repair --ms1 - --max-indel 1 < card.txt
  ```

  For an `mk1` or `md1` card write `--mk1 -` or `--md1 -` in place of
  `--ms1 -`. A unique recovery prints the full string and exits 5;
  several equally valid candidates are all printed, exit 4, and you
  keep the one that matches every character you can read. Exit 2
  means nothing was found: if the string may also hold a wrong
  character, run the command again with `--max-subst 1` added (raise
  it for more wrong characters; the characters left out plus the
  wrong ones may be at most 4); such a recovery usually exits 4, a
  candidate you confirm before use. This search does not work for a
  hashlock preimage plate (`ms10hash…`), nor for every `md1` card of
  a single string; use the second way for those.
- **Type your best guess** for each unreadable character: any
  character of the card's alphabet, in the same case as the rest.
  Then run `ms repair --in card.txt` for an `ms1` card, or
  `mnemonic repair --mk1 - < card.txt` (`--md1 -` for an `md1` card).
  Up to 4 wrong characters in total are corrected, and exit 0 means
  every guess was right. A corrected `ms1` is a candidate, exit 4,
  which you confirm before use. For an `mk1` or `md1` card, exit 5
  means the correction checked itself across the card's strings;
  exit 4 means it could not, so confirm it before use.

Neither way works yet for one share of a K-of-N `ms1` set (a string
that starts `ms12` to `ms19`): `ms repair` and `mnemonic repair` both
refuse a share, exit 2. If you hold K other shares that read cleanly,
recover the secret from them with `ms combine`.

Either way, compare the result with the plate before you rely on it.
~~~

(Line wrapping is chosen so that no continuation line starts with `N.`, which
pandoc could read as an ordered list.)

## Minor (to `design/FOLLOWUPS.md`, `deferred-minor`)

- **M-1 [r1-doc] #1's "returns `encode(tag, secret)` unchanged, error
  included" is false when the payload is also invalid.** `encode_shares` runs
  `secret.validate()?` before the ZERO branch, and `encode` checks the tag
  first. Measured with a scratch test on the patched tree:
  - `seed` + `Entr(15 B)`: `encode` gives `ReservedTagNotEmittedInV01`;
    `encode_shares(ZERO)` gives `PayloadLengthMismatch`.
  - `hash` + `Entr(15 B)`: `TagKindMismatch` vs `PayloadLengthMismatch`.
  - `seed` + `Mnem{10}`: `ReservedTag…` vs `MnemUnknownLanguage(10)`.

  Both sides refuse, so nothing is minted. The risk: a builder honouring the
  behaviour text could move `validate()` and diverge from section 3 item 1.
  Conditions: library-only ZERO caller (none exists), a bad tag AND an invalid
  payload. Fix: 2.1 Behaviour, after "error included", add ", for any `secret`
  that passes `Payload::validate()`; `encode_shares` keeps running
  `secret.validate()?` first, so an invalid payload is refused with its own
  error whatever the tag". In the 2.1 spec New text, after "`TagKindMismatch`
  for a tag that names another kind", add "; a payload that fails `validate()`
  is refused with that error first, as before".
- **M-2 [r2-amendment] The new subsection contradicts the untouched one just
  before it.** Primer lines 177-181 say "**Correction of length-changing damage
  is not supported.**" (and line 186: "only physical inspection can recover"),
  while 5.6.6 tells the holder to create a deletion and let
  `mnemonic repair --max-indel` find it. Fix: replace the lines 177-181 bullet
  with:

  ```text
  - **The decoder does not correct length-changing damage.** It
    reports "string length out of range" or "checksum invalid"
    without identifying the missing or extra character's position.
    The polynomial syndrome encodes substitution-error positions, not
    insertion / deletion positions. `mnemonic repair --max-indel N`
    searches for up to 4 missing or extra characters instead; see
    *Unreadable characters: what to type* below.
  ```
- **M-3 [r2-amendment] 5.6.4 keeps "same as regular code: 4 / 8 / 13" for the
  long code.** BIP-93 line 398 says the long checksum corrects "up to 15
  consecutive erasures". This is text the FIX touches, and no tool decodes
  erasures, so the error is harmless today. Fix: New cell
  `4 / 8 / 15 by the code (BIP-93's long checksum: up to 15 consecutive erasures); the repair tools implement the 4 substitutions only`.
- **M-4 [r2-amendment] 5.6.6's insertion instruction, followed literally, makes
  the heading vanish.** "Replace the heading line (with its preceding blank
  line) by the subsection followed by the heading" leaves no blank line between
  `  auditable on the plate.` and `### Unreadable…`. Pandoc then folds the
  heading into the previous bullet (rendered and confirmed:
  `…auditable on the plate. ### Unreadable characters: what to type</li>`), so
  every *Unreadable characters…* cross-reference points at nothing.
  `make lint` and `anchor-check` do not catch it. The script does it right
  (`'\n## Hand…\n'` → `'\n### Unreadable…'`). Fix: reword the instruction to
  "Insert the subsection, then one blank line, between the blank line that
  precedes the heading and the heading itself (the script's ninth edit does
  exactly this)."
- **M-5 [r2-amendment] The 9.4 toolkit FOLLOWUPS text is incomplete after
  round 2.**
  - Its "ms-cli half" lists `ms combine --to` and `ms inspect` of the secret
    share, but not T2 (`ms inspect` of an unknown mnem language), which 5.5
    also describes.
  - It does not record that `mnemonic repair` cannot read a hash plate at
    ms-codec 0.7.0 (I-1).

  Fix: say "`ms inspect` of the secret share and of an unknown mnem language",
  and append: "(c) At 0.7.0, `mnemonic repair --ms1` cannot read a hashlock
  preimage plate (`ms10hash…`): a clean, corrected or `--max-indel` input exits
  2 (`reserved-prefix byte was 0x03, expected 0x00`). The BCH primer's
  *Unreadable characters: what to type* says so; revisit that sentence at the
  bump."
- **M-6 [r1-doc] 5.3 keeps the `ms repair` exit table as is, and the table
  misleads for #7's case.** Row `2` says "structural ms1 error before
  correction could run". A character outside the alphabet (including a typed
  `?`, which the primer now discusses) is exactly that, yet it exits 1. Fails
  closed. Fix: the `43-ms.md` exit-table row `| `1` | I/O error or other generic failure. |`
  becomes `| `1` | I/O error or other generic failure, including a character outside the codex32 alphabet (`invalid character '<c>' (not in codex32 alphabet)`, e.g. a typed `?`). |`,
  and 5.3 says so.
- **M-7 [r2-amendment] Run what you print: the primer's printed commands have
  no executed pin.** No RED row, no `verify-examples` transcript. 2.6 ran them
  once, on one entr16 card. Every section 6 row that pins a printed command
  does run it (see below), so the gap is only the manual's commands. Fix: the
  companion PR adds a `verify-examples` transcript (or a toolkit test) per
  route, on a damaged test card.

## Nit

- **N-1 [r1-doc]** The #7 message names the character as typed, so an
  invisible character prints invisibly. A pasted U+200B gives
  `error: invalid character '​' (not in codex32 alphabet)`, exit 1 (measured;
  `ms decode` says `string length 53 …` for the same input). An escaped form
  (`'\u{200b}'`) for non-printing characters would name it.
- **N-2 [r2-amendment]** 9.5 should cite the existing toolkit item
  `m-format-indel-erasure-decode-extend-to-8` (`design/FOLLOWUPS.md:4175`,
  open). Its "Companion" line expects an ms entry "to be filed in lockstep
  when scheduled", and Brian's `?`-in-indel direction is the same work.

## Mandatory checks

### 1. Create-input table

| command (where printed) | real interface (patched `ms` / `mnemonic 0.105.1`) | result |
|---|---|---|
| `ms combine --to {phrase,entropy,ms1} [--json] --in FILE` / `-` (2.2, rows 04-04d) | group `shares` or `--in` required; `--to` value enum; `--json` | runs as printed |
| `ms inspect [--json] --in FILE` / `-` (2.3, 2.5, rows 05, T2) | `[MS1]`, `--in` (conflicts with `MS1`), `--json` | runs |
| `ms repair [--json] --ms1 -` / `--in FILE` (2.4, 2.6, primer) | `<--ms1|--in>` required; strips display separators (grouped input repaired) | runs; `--ms1 - --in x` is exit 64, and nothing prints that |
| `ms decode --in FILE` / `-` | ok | runs |
| `ms hashlock --kind sha256 --hex - --out F` / `--in F` (2.2) | ok | runs; `cmp` identical to the `combine --to ms1` output |
| `mnemonic repair --ms1 - --max-indel N` (primer) | `--ms1` takes one chunk, `-` = stdin; `--max-indel` 0..=4 (5 → `invalid value '5' … 0..=4`); `--max-subst` 0..=4, no effect without `--max-indel` | runs for entr/mnem; fails for hash and share (I-1) |
| `--mk1 -` / `--md1 -` (primer) | repeating; `-` reads chunks one per line | needs every string of the card (I-2) |
| `mnemonic repair` bare, route 2 (primer) | needs one of `--ms1`/`--mk1`/`--md1`/positional; no `--in` | exit 64 (I-2) |
| `make lint`, `make anchor-check` (FIX header) | — | lint: same 10 FAILs before/after; anchor-check OK |

### 2. Case tables (an undecided cell would be a Sonnet-proof finding)

- **#1** (2.1):
  - reserved tag + valid payload → `ReservedTagNotEmittedInV01`;
  - `hash` + Entr/Mnem → `TagKindMismatch{prefix 0x00/0x02}`;
  - `entr` + Preimage → `TagKindMismatch{0x03}`;
  - valid → `vec![encode(..)]`, byte-identical;
  - `n != 1` → `InvalidShareCount` first;
  - bad tag + invalid payload → `validate()` error first. This cell is
    misdescribed (M-1); the code instruction decides it.

  All other cells are decided.
- **#4** (2.2): preimage × {default, explicit `phrase`, `entropy`, `ms1`} ×
  {text, `--json`}. All 8 cells are decided and all 8 measured equal to 2.2.
  stderr is exactly one advisory line, with no language advisory
  (`non_english_seed_advisory` gets English). Entr and mnem are unchanged.
- **#5** (2.3):
  - k ∈ 2..9 with index `s`, lower or upper case → FAIL `share-index-secret`,
    exit 0 (measured, both cases);
  - any other index → OK;
  - a single → the unchanged rule walk;
  - a non-`ms` share at index `s` → this reason only (T3 deferred).

  All decided.
- **#7** (2.4):
  - `b`, `B` (all-uppercase string), `?`, `ĳ`, `İ`, `1`, `o` in the data part →
    `InvalidChar(as typed)`, exit 1, JSON kind `Codex32`, `details` null
    (measured `b`, `B`, `?`, `ĳ`, `1`, `o`, U+200B);
  - wrong HRP → `WrongHrp`, unchanged;
  - mixed case → X1, unchanged;
  - separators and tabs → stripped by the CLI, unchanged.

  All decided.
- **T2** (2.5):
  - language 0-9 → unchanged;
  - language ≥ 10 → FAIL `mnem-unknown-language`, before
    `payload-length-mismatch`;
  - text and JSON as measured.

  The `MNEM_LANGUAGE_NAMES.get(code).is_none()` predicate equals `validate`'s
  `>= 10` (`payload.rs:95`). All decided.
- **Primer**: see the advice lens. The undecided or wrong cells are I-1..I-3.

### 3. Advice lens (executed). Every route of 5.6.6 on test material

**Inputs** (all test data):

- ms1 entr16 `ms10entrsqqqqzqsrqszsvpcgpy9qkrqdpc8sh9lf92n5fr33s`;
- ms1 entr32 (entropy `00..1f`, 75 characters);
- ms1 mnem (Italian `abaco`×11 + `abete`,
  `ms10entrsqgrsqqqqqqqqqqqqqqqqqqqqqqqqqh7r4chytf8ql8`);
- ms1 hash `ms10hashsqvg3zy…l0sajh7rfj2z`;
- ms1 share (`ms split --threshold 2 --shares 3` of `00..0f`);
- signet bip84 bundle of `abandon … about`: mk1 (2 strings), md1 (3 strings);
- `md encode 'wpkh(@0/<0;1>/*)'` (one unchunked string).

**Method.** k random data-part positions are left out (route 1) or replaced by
a wrong symbol (route 2). Route 1 runs `--max-indel k` (4 when k = 5).
"+typo" adds one more substitution. Counts are recovered/total, with exit
codes. There were n = 12 trials per cell, except route 1 at k = 3-5 (n = 4).

| card | route 1, k=1..4 | route 1 + typo, as printed | + `--max-subst 1` | route 2 `ms repair --in`, k=1..4 | route 2 `mnemonic repair --<hrp> -`, k=1..4 | k=5, any route |
|---|---|---|---|---|---|---|
| ms1 entr16 | all recovered, exit 5 | refused exit 2 (24/24) | recovered exit 4 | recovered exit 4 (48/48, installed and patched) | recovered exit 4 | refused exit 2 |
| ms1 entr32 | all, exit 5 | exit 2 | exit 4 | exit 4 | exit 4 | exit 2 |
| ms1 mnem51 | all, exit 5 | exit 2 (23/24; 1 recovered exit 5) | exit 4 | exit 4 | exit 4 | exit 2 |
| ms1 hash75 | **exit 2 (all)** | exit 2 | **exit 2** | recovered exit 4 | **exit 2** | exit 2 |
| ms1 share | **exit 2** | exit 2 | exit 2 | **exit 2** | **exit 2** | exit 2 |
| mk1, 2 strings, both in file | all, exit 5 | exit 2 | exit 4 | n/a | recovered **exit 5** | exit 2 |
| md1, 3 strings, all in file | all, exit 5 | exit 2 | exit 4 | n/a | recovered **exit 5** | exit 2 |
| md1, one unchunked string | **exit 2 (all)** | exit 2 | exit 2 | n/a | recovered exit 4 | exit 2 |

Further measured cases:

- mk1/md1 with only the damaged string in the file: route 1 exits 2 for both;
  route 2 gives mk1 exit 4 and md1 exit 2.
- A typed `?`: `mnemonic repair` exits 2 (`non-bech32 char '?' at position 14`).
  `ms repair` exits 1 (installed: the length message; patched:
  `invalid character '?'`).
- `--max-indel 5` is refused by clap.

**Wrong secret:** none in about 1,750 trials (no "other candidate" outcome).
Ambiguity, where it can occur, is safe under "compare with the plate": two
codewords differ in at least 9 positions, so at most one candidate matches
every readable position.

The other texts:

- 5.1 (`combine --to`), 5.2 (index-`s` FAIL) and 5.5 (unknown-language FAIL)
  were each followed on the patched build and are correct.
- 5.6.1-5.6.5 and 5.6.7 are correct, apart from M-2 and M-3.

### 4. Run what you print

Every section 6 row that pins a printed command runs it with a concrete
fixture, and asserts an exact stdout and exit (and stderr where the row says
so):

- CLI 04, 04b, 04c and 04d: `ms combine` on `encode_shares` output;
- 05 and 05b: `ms inspect` and `ms combine` on the `from_seed` index-`s`
  share;
- 07 and 07b: `ms repair` on literal strings;
- T2: `ms inspect` and `ms decode`.

The codec rows (01, 03, 07) call the library, and the ignored rows keep their
bodies. No row pins only text. The gap is the manual's primer commands
(M-7).

### 5. Deferred minors in touched text

`design/FOLLOWUPS.md` at `9e0fda8` holds no `deferred-minor` entry (grep count
0), so only the 9.2 entries are re-graded:

- **`ms-too-many-errors-says-more-than-8`** gains reach: the new primer sends
  holders who make 5 or more guesses to `ms repair`, which says "more than 8
  errors", against the primer's "4". It still fails closed (exit 2) and stays
  **Minor**.
- **`ms-repair-accepts-mixed-case-with-typo`**: #7 rewrites its cited loop,
  but the ignored row still fails, so the behaviour is intact. It stays
  **Nit/Minor**.
- **`ms-length-error-lists-entr-set-only`**, **`ms-cli-mnem-unknown-language-catch-all`**,
  **`ms-combine-accepts-non-ms-hrp`**, **`ms-mlock-unpin-not-refcounted`** and
  **`ms-codex32-parts-short-data-panic`** are unchanged.

None becomes C or I.

### 6. External protocol facts (BIP-93, `bip-0093.upstream.mediawiki`)

- Line 152 gives 4 substitutions, 8 erasures and 13 consecutive erasures.
  5.6.1 and 5.6.2-5.6.3 are correct.
- Lines 155 and 158 say "we recommend that: … Implementations interpret "?"
  as an erasure". 5.6.5 ("BIP-93 suggests marking one with `?`") and 9.5 are
  correct.
- Line 398 says the long checksum corrects "up to 15 consecutive erasures".
  5.6.4 says 13 (M-3).

## Section 10 known points

I re-graded each; none is wrong.

1. `?` called "mixed case": correct and filed (9.3).
2. The preimage under `entropy_hex`: ruled.
3. The manual describes the next release: prose only, and no command fails.
   The I-1 fix's hash-plate sentence is tied to the toolkit's ms-codec 0.7 pin,
   so it should be revisited at the bump (M-5).
4. Exit 5 for a left-out character: correct, measured.
5. The decoder's origin: correct. The `bch_decode.rs` header says it was
   forked from mk-codec v0.3.1.

## Settled facts this review did not re-derive

- The suite, fmt and clippy results and the ignore census (controller,
  `fixcheck/`).
- The grades and rulings of `bughunt-grade-ms.md` and `bughunt-grade2-ms.md`.

## Design concern (ungraded)

In text mode, `ms combine --to entropy` on a preimage share set prints 64
bare hex characters with no kind label. That output looks exactly like 32-byte
seed entropy, and `ms encode --hex -` would mint an `entr` seed card from it,
which is the hashlock warning's "also anything else's secret" case.
