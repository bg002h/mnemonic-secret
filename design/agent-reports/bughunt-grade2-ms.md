# Bug-hunt grading pass, round 2, mnemonic-secret (ms)

Opus grading pass (RULES.md "Triage of reported bugs" item 2), resumed from round 1 (`bughunt-grade-ms.md`, `4755eb9`) and the FIX document (`27274eb`). Inputs: triage round 2 (`bughunt-triage2-ms.md`, `5399625`; RED rows `14b67b2`), its evidence in `/scratch/cache/bughunt/ms2/`, and Brian's ruling of 2026-11-01, "Fix bugs before erasure feature work". No new hunting.

**Outcome: 2 I (T2; the toolkit manual's BCH-primer overclaim), 2 M (T1, T3), 0 C, 0 N.**
- T1b is not graded: it is feature work, planned after this PR.
- One more item is filed `needs-triage` with a proposed M, because it has no RED row yet: `ms decode` and `ms inspect` calling `?` "mixed case".
- I agree with every grade the triage proposed. It proposed no grade for the primer; I grade it I.

What I checked myself (beyond the brief's Settled facts):
- Every round-2 row fails at its own assertion on unmodified code.
- With the FIX document's edits applied on a scratch copy, T2's tightened row passes. T1, T1b and T3 still fail when run ignored; T1 and T1b now fail with `codex32 parse error: InvalidChar('?')` instead of the length message.
- The whole workspace on that copy: 667 run, 667 passed, 23 skipped.
- BIP-93 lines 70, 152-161 match upstream (fetched in round 1).
- Every command the new primer text prints was run on the installed `mnemonic 0.105.1` and `ms 0.20.1`, plus a probe of the stated limits (`/scratch/cache/bughunt/ms-grade/probe/indel_probe.txt`):
  - characters left out with `--max-indel 3` or `4`: 40/40 recovered at each count, exit 5;
  - wrong guesses through `ms repair`: 1-4 recovered (exit 4), 5 refused (60/60, exit 2).
- Toolkit manual `make lint` and `make anchor-check` on `c2b09861` with all ten edits applied: no new finding.

## Grades

| item | final grade | triage proposal | agree? | row tests the defect? | likelihood conditions / why |
|---|---|---|---|---|---|
| T1 (one `?`) | **M** | M | yes | The row tests the *feature*: recovery of one `?` at every data position (47). Today it fails at its own assertion with #7's length message; after this PR, with `InvalidChar('?')`. It cannot pass until erasure filling exists, so it is the feature's acceptance row, not a bug row. | The bug part is #7's message (graded I, fixed in this PR): after the PR, `ms repair` refuses a `?` with `error: invalid character '?' (not in codex32 alphabet)`, exit 1. What remains is a BIP-93 recommendation (lines 155-158, "we recommend that: … Implementations interpret "?" as an erasure"), which Brian has sequenced as feature work. Likelihood test after this PR: (1) fails closed, naming the character; (2) no printed command fails (no page prints a `?` command, and the corrected primer says not to type one). (3) Conditions: (a) a plate with an unreadable (not misread) character; (b) the holder types `?` instead of leaving it out or guessing, against the corrected primer. |
| T1b (eight `?`) | not graded (feature) | M (with T1) | n/a | The row tests the feature at BIP-93's stated strength. | Brian's ruling: feature work after the bug-fix PR. Row `#[ignore = "planned-feature: impl-bch-erasure-decoding-ms"]`; FOLLOWUPS entry `impl-bch-erasure-decoding-ms` (FIX §9.5). |
| T2 (inspect, language ≥ 10) | **I** | I | yes | Yes: a checksum-valid mnem string with language byte 10; `ms decode` refuses it naming the language code (precondition); `inspect` JSON and text both claim it would decode. Tightened to exact texts in FIX §6. | A wrong OK verdict does not fail closed, the same reasoning as #5 (graded I). Conditions noted but not demoting: a foreign or newer encoder's string with a code of 10 or more; `decode` itself refuses cleanly. |
| T3 (combine, HRP `xx`) | **M** | M | yes | Yes. Control: the same construction under `ms` combines. Reference: `ms decode` refuses an `xx` single. The assertion: `ms combine` of the `xx` set must exit non-zero. | (1) is not met: it succeeds rather than refusing. So the test cannot demote it, and the usual grades apply. Under them it is M: the output is exactly the secret the strings encode, so there is no wrong secret, card or data, and no printed command fails. Reach: (a) a valid-checksum share set with a non-`ms` HRP, which no ms, toolkit or BIP-93 workflow makes (BIP-93 line 70: "A human-readable part, which is the string "ms" (or "MS")"; a typo in a real share's HRP breaks its checksum; a mixed `ms`/`xx` set is refused); (b) an ms-shaped secret payload; (c) handed to `ms combine`. |
| BCH primer overclaim (toolkit `65-bch-codex-primer.md` :22-24, :108, :109, :121, :154-156, :281-282) | **I** | (left to Opus) | n/a | Read, and the alternative actions run (above). | It is recovery advice in the exact state it addresses. "Operational fallbacks" (:281-282) puts the per-card correction radius at "4 substitutions or 8 erasures in known positions". No constellation tool decodes erasures, so a holder with 5-8 unreadable characters is told they are inside the radius and is not sent to the fall-backs. The erasure row (:108) invites `?` ("known positions, e.g., `?`"), which every repair tool refuses. The actions that do work (leave the character out with `mnemonic repair --max-indel`, or type a guess) are not mentioned. Likelihood test: (1) the tools fail closed and (2) the primer prints no command, but (3) needs only one condition, a plate with unreadable characters, which is the state the section is written for. So it is not demoted, and misleading recovery advice is I. |
| `?`/`!` named "mixed case" by `ms decode`/`ms inspect` (triage open question) | proposed **M**, filed `needs-triage` | — | n/a | No RED row: `reach-t1-decode-chars.txt` is a transcript, so under the two-direction rule this is not yet CONFIRMED. | Fails closed and names the character, but calls it a case error. Needs (a) a holder who types punctuation (such as BIP-93's `?`) (b) into `decode`/`inspect`, not `repair` (c) on a lowercase string. Cause: the vendored codex32 `set_check_case` treats non-digit, non-lowercase ASCII as uppercase. |

## Item 2 — T1's bug part, and the row's disposition

**Yes: #7's ruled fix already gives `?` a correct refusal.** Measured on the fixed build:

~~~text
$ echo 'ms10entrsqz4?42424242424242424242424q8lurdrzjnyn08' | ms repair --ms1 -
stderr: error: invalid character '?' (not in codex32 alphabet)
exit=1
~~~

With `--json`: `{"schema_version":"1","error":{"kind":"Codex32","message":"invalid character '?' (not in codex32 alphabet)","exit_code":1,"details":null}}`. The library returns `Err(Error::Codex32(InvalidChar('?')))`.

**T1's row does not change to assert the refusal.** The refusal is already pinned by two of round 1's #7 rows, which the FIX document specifies: `bughunt_ms_07_codec_non_alphabet_is_invalid_char` (codec, `?` case) and `bughunt_ms_07b_repair_names_char_as_typed` (CLI, `?` in `--json` and in text mode). A second row would duplicate them.

T1's row asserts recovery, which is the feature's behaviour. It keeps its body and gets `#[ignore = "planned-feature: impl-bch-erasure-decoding-ms"]`, as T1b does. When ms-codec fills `?`, those two rows are un-ignored and the `?` cases of the #7 rows flip from refusal to recovery (recorded in the planned-feature entry).

## Rulings (exact texts)

**T2, in this PR.** In `analyze()`, in rule 10's mnem arm and before the length check (the order `Payload::validate` uses), a language code with no entry in `MNEM_LANGUAGE_NAMES` (10 or more) pushes:
- reason `mnem-unknown-language`, text `unknown mnem wordlist-language code; known codes are 0-9`.

Holder-visible result:
- Text line 1 is the existing `FAIL: would NOT decode v0.1`; line 2 is `    reason: mnem-unknown-language (unknown mnem wordlist-language code; known codes are 0-9)`; the field block is unchanged and still ends `language: unknown`.
- JSON has `"would_decode":false,"failure_reasons":["mnem-unknown-language"]`; the `report` object is unchanged.
- Exit 0.

The text avoids nested parentheses inside the reason line. `ms decode`'s own text stays the deferred `ms-cli-mnem-unknown-language-catch-all`.

The spec sentence is in `SPEC_ms_mnem_wordlist_language.md` line 49 (FIX §2.5). `SPEC_ms_cli_v0_1.md` §2.3's tag table is the v0.1 set: it already lacks `tag-kind-mismatch`, and §5.3 allows additive tags. It stays as is.

Manual: `43-ms.md:336` gains one sentence (FIX §5.5).

**T3, deferred (`ms-combine-accepts-non-ms-hrp`, FIX §9.2), wording for later.**
- `combine_shares` refuses a non-`ms` canonical HRP with `WrongHrp { got }`, so the text is `error: wrong HRP: got "xx", expected "ms"`, exit 2, as `ms decode` gives for a single.
- In the same change, `ms inspect` reports such a share as `FAIL: would NOT combine` with the existing reason `wrong-hrp` (`HRP is not "ms"`), `would_decode`/`would_combine` false, exit 0, following the #5 ruling.

**The primer, in the toolkit companion PR (FIX §5.6, seven replacements in `65-bch-codex-primer.md`).** The table rows and the opening bullet keep BIP-93's numbers as properties of the code, and add that the repair tools implement the 4 substitutions only. The erasure definition drops "(typically rendered `?`)" and says `ms repair` and `mnemonic repair` refuse a `?`. The fall-back line becomes:

~~~text
When stamping errors exceed what the repair tools can fix (4 wrong
or unreadable characters in one string; see *Unreadable characters:
what to type* above), the fall-backs are:
~~~

A new subsection, before "## Hand-decodability of ms1 via the bundled codex32 paper-computer":

~~~text
### Unreadable characters: what to type

No repair tool decodes erasures yet, so do not type `?` for a
character you cannot read. Two things work, for up to 4 unreadable
characters in one string (fewer if the string has other mistakes):

- **Leave each unreadable character out** and let `mnemonic repair`
  search for the gaps, telling it how many you left out (1 to 4).
  Put the string, with those characters left out, in `card.txt`:

  ```sh
  mnemonic repair --ms1 - --max-indel 1 < card.txt
  ```

  A unique recovery prints the full string and exits 5; several
  equally valid candidates are all printed, exit 4, and you choose.
  `--mk1` and `--md1` take the same search; see `--max-indel` and
  `--max-subst` in the `mnemonic repair` reference.
- **Type your best guess** for each unreadable character: any
  character of the card's alphabet, in the same case as the rest.
  `ms repair --in card.txt` (or `mnemonic repair`) then corrects up
  to 4 wrong characters in total; a corrected string is a candidate,
  exit 4, which you confirm before use (exit 0 means every guess was
  right).

Either way, compare the result with the plate before you rely on it.
~~~

Both printed commands run as printed on today's tools. `mnemonic repair --ms1 - --max-indel 1 < card.txt` with character 17 left out exits 5 with the original string. `ms repair --in card.txt` with character 17 guessed wrong exits 4 with the original (FIX §2.6). The text holds for the tools after this PR too: `ms repair`'s `?` refusal only changes its wording. All ten toolkit edits are a script: `/scratch/cache/bughunt/ms-grade/toolkit-manual-edits.py` (sha256 `083063227a5ef1ec8cb015fbe5e82b1e66c31b2bf1fe173af3c3eb182ce7aa09`).

**T1b, planned feature (`impl-bch-erasure-decoding-ms`, FIX §9.5), not specified here.** The entry records:
- the controller's substitution probe and the indel probes;
- the triage's estimate;
- Brian's design direction (the toolkit's `mnemonic repair` fills `?` through its indel/VERIFY-ME exits first; true BCH erasure decoding later);
- the md/mk companions `impl-bch-erasure-decoding-md-mk`, as its third leg.

Code-origin note (a correction to the controller's note): ms1's string layer is the vendored rust-codex32 0.1.0, but its correction decoder `crates/ms-codec/src/bch_decode.rs` was forked from mk-codec v0.3.1 and mirrors md-codec's (its own header says so). The entry records both, because it bears on whether the three legs share work.

## What changed in `design/FIX_bughunt_ms.md` (uncommitted, on `bughunt/triage` at `5399625`)

- **Title, sources, scope:**
  - T2 and the primer added; round-2 triage and this report cited.
  - Brian's ruling stated; T1/T3 out of the PR, T1b planned (9.5).
- **Header checks:**
  - reference patch `fix-bughunt-ms-reference-r2.patch` (sha256 `b1f7d1be83a46cddd7b4f9def6d2d346d73c0ab1191f68686ded0cb313d3cd0e`; `git apply --check` passes at `5399625`);
  - the toolkit edit script; nextest 667/667/23 skipped;
  - T2's row RED on unmodified code;
  - lint and anchor-check clean.
- **§1:** items 6 (T2) and 7 (primer), plus a "not in this PR" line.
- **§2:**
  - input `mnem-lang10.txt` added;
  - 2.4 gains the `?` (T1) paragraph and the new needs-triage slug;
  - new 2.5 (T2: behaviour, exact text and JSON, current/after transcripts, spec sentence, the SPEC_ms_cli table note);
  - new 2.6 (primer: the run commands with output, the measured limits).
- **§3:** item 5 (the `analyze()` change and the `reason_text` arm).
- **§4:** item 4 (`SPEC_ms_mnem_wordlist_language.md` line 49).
- **§5:** 5.5 (`43-ms.md:336`) and 5.6.1-5.6.7 (primer old/new texts); 5.3 reworded.
- **§6:**
  - T2 row tightened to exact stdout (and its unused `Value` import removed);
  - T1 and T1b `#[ignore = "planned-feature: impl-bch-erasure-decoding-ms"]`;
  - T3 `#[ignore = "deferred-minor: ms-combine-accepts-non-ms-hrp"]`;
  - counts updated.
- **§7:** "no other test changes" now covers `bughunt_t.rs`; two T2 greps added (no hits that change).
- **§8:** items 4-8 widened (inspect T2, four spec edits, `bughunt_t.rs`, FOLLOWUPS 9.5, the primer in the companion PR).
- **§9:**
  - 9.1 grade now Important ×6 and its build-impact lines widened;
  - 9.2 gains `ms-combine-accepts-non-ms-hrp`;
  - 9.3 now holds only `ms-decode-calls-punctuation-mixed-case`. The old needs-triage entries are resolved: `ms-repair-question-mark-erasure` became 9.5 plus that entry, `ms-inspect-ok-for-unknown-mnem-language` is fixed (T2), and `ms-combine-accepts-non-ms-hrp` moved to 9.2;
  - new 9.5 `impl-bch-erasure-decoding-ms` (status `planned-feature`, owning phase "after the bug-hunt fix PR").
- **§10:** still no STOP; observations 4 (exit 5 is the toolkit's documented indel behaviour) and 5 (the code-origin note) added.
