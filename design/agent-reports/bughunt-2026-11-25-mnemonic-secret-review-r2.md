# Fold review r2: mnemonic-secret draft PR #48 (`claude/project-thread-utm1aa`), fold commit 527223a

Reviewer: second (fold) reviewer, read-only. No tracked file was edited, nothing was committed or pushed, and nothing was posted to GitHub. Probes ran in the scratchpad (`.../scratchpad/r2/`).

## Verdict

**0 Critical, 0 Important, 1 Minor, 3 Nit.** The fold logs all four r1 Minors and all six r1 Nits at r1's grades, and it changes nothing else. r1 M4 is **not Important**. I reproduced it and no ms-cli verb prints or uses wrong secret material. Part of r1's reasoning for M4 is wrong: an error in an aliased position is refused, not mis-corrected (details below). Minor remains the right grade for M4.

## Evidence

- `CARGO_INCREMENTAL=0 cargo test --locked -j 2 -p ms-codec -p ms-cli` on 527223a: exit 0. 867 passed, 0 failed, 23 ignored, across 131 test binaries (log: `.../scratchpad/r2-tests.log`).
- `git diff 93dbc84 527223a --stat`: 2 files changed, 111 insertions, 0 deletions.
- The saved `design/agent-reports/bughunt-2026-11-25-mnemonic-secret-review-r1.md` is byte-identical to the scratchpad r1 (`diff` is empty).
- I wrote an independent BIP-93 `ms32_polymod` in Python (`r2/bip93.py`). It accepts BIP-93 test vector 1, `ms10testsxxxxxxxxxxxxxxxxxxxxxxxxxx4nzvca9cmczlw`. With it I crafted probe strings from a real `ms split` 2-of-3 set and ran each through `ms repair --json`, `ms combine` (paired with a valid share), `ms decode` and `ms inspect`.

## Q1: does the fold log every r1 Minor and Nit faithfully?

| r1 item | Fold | Grade kept? | Faithful? |
|---|---|---|---|
| M1 (GUI `ms split` has no `--hex`/`--language` conditional; help text false) | `ms-bh1125-gui-split-hex-language`, FOLLOWUPS.md:1400 | M, yes | Yes: it cites the file and line and gives the effect. It leaves out r1's proposed fix (add an `ms_split` conditional that mirrors `ms_encode`), and there is no companion entry in mnemonic-gui's FOLLOWUPS (see N2). |
| M2 (SPEC_ms_cli_v0_1.md:57 says "ignored under --hex") | `ms-bh1125-spec-hex-language`, :1401 | M, yes | Yes |
| M3 (new public API with no release or version bump) | Mapped to the existing `ms-codec-correct-checksum-release` (:1398, filed in 93dbc84 before r1) | See N1 | The substance matches. The mapping is recorded only inside the M4 line; the release line still cites only the bug-hunt source and carries grade=L. |
| M4 (`correct_checksum` does not bound length) | `ms-bh1125-correct-checksum-length`, :1402 | M, yes | Yes. "by reading" is accurate for r1, and the claim holds (reproduced below). |
| N1–N6 | `ms-bh1125-review-nits`, :1403, grade N | N, yes | All six are present in order. N4 drops r1's side note that `char::is_control` misses U+200B, U+00A0 and U+2028, an adjacent point r1 called out of scope. |

**Line format.** Each new line follows the existing one-line form: `- [slug] status= grade= kind= owner= filed= src= repo= :: text`. The `kind` values (cross-repo, docs, input) are all already used in the file. The `src=` anchors point at the saved report.

## Q2: did the fold change anything else?

No. The fold adds the r1 report verbatim and appends 4 FOLLOWUPS lines. No existing line, code, test or other doc changed.

## Q3: is r1 M4 Important? Reproduction

**Which callers reach it.** `correct_checksum` is called only from `repair.rs:164` (the share path), `combine.rs:226` (the note) and `decode_with_correction` (`decode.rs:264`). `decode` and `inspect` never call it.

**Probe results.** Every probe used a valid share S1 of a real 2-of-3 set (data part length L = 47, total 50).

| Probe | `correct_checksum` | `ms repair` | `ms combine` + valid share | `decode` / `inspect` |
|---|---|---|---|---|
| P1: S1 with 1 substitution (control) | corrected | exit 4, candidate equals S1, UNVERIFIED | note "ms repair can propose", exit 1 checksum | checksum error |
| P2a: 110-char string with a valid short checksum | clean | exit 1, length | exit 1, length | length |
| P2b: P2a with 1 error at data position 50 (degree 56, not aliased) | **Ok, "corrected"** | exit 1, length | **note sends the holder to `ms repair`**, exit 1, length | length |
| P2c: P2a with 1 error at data position 2 (degree 104, aliased to 11) | TooManyErrors | exit 2, "more than 8 errors" | note "more errors than ms repair can correct", exit 1, length | length |
| P2d: P2a with an equal-magnitude pair 93 apart | **residue 0, "clean"** | exit 1, length | no note, exit 1, length | length |
| P2e: P2a with 2 non-aliased errors | Ok, corrected | exit 1, length | note sends the holder to `ms repair` | length |
| P3: 36-char string with a valid checksum and 1 error | Ok, corrected | exit 1, length | note sends the holder to `ms repair` | length |
| huge: 20,022 chars with 1 aliased error | TooManyErrors, 15 ms | exit 2 | note, exit 1, length | — |
| P5: S1 with 1 substitution, mixed case | Ok, corrected | **exit 4, candidate = S1** | note, then a mixed-case error | mixed-case error |

**Findings from the reproduction**

1. **No verb prints or uses wrong secret material.**
   - `ms repair` reaches the share path only after `decode(corrected)` returned `IsShareNotSingleString` (`repair.rs:161-163`). That return requires `is_known_length` plus a successful rust-codex32 `from_string` and envelope parse of the corrected string. A wrong-length string never gets there.
   - `ms combine` uses `correct_checksum` only to choose the wording of a stderr note. The combine itself goes through rust-codex32, which refuses any length outside 48-93 or 125-127.
2. **r1's mechanism is partly wrong.** r1 expected aliased Chien positions to give "spurious corrections". In fact `chien_search` (`bch_decode.rs:284-303`) scans every degree in `0..L`, so a root that aliases finds two hits. Then `error_degrees.len() != deg`, and the result is `None`, which is TooManyErrors (P2c, huge). The function fails safe there.
3. **The real residue.**
   - On a string of the wrong length, `correct_checksum` can return `Ok` with genuine short-code corrections (P2b, P2e, P3).
   - Past 93 symbols, the code no longer detects an equal-magnitude error pair 93 apart, so `Ok` comes back with no corrections at all (P2d).
   - The output is then a string that passes the short checksum but is no valid codex32 or ms1 string.
   - **Effect in the CLI:** a misleading combine note that sends the holder to `ms repair`, which then refuses on length.
   - **Effect for a library caller:** `Ok` does not mean "valid share". A caller that parses the output (rust-codex32 `from_string`, `combine_shares`) still refuses it. Only a caller that treats `Ok` itself as validity, against the rustdoc at `decode.rs:288-290`, would be misled.
   - This is **Minor**, as r1 graded it and as the fold logged it.

### Minor

**M1 (r2). r1 M4 stands at Minor, but its recorded mechanism and fix need two corrections before anyone acts on the FOLLOWUP.** (`design/FOLLOWUPS.md:1402`; `crates/ms-codec/src/decode.rs:299-350`)
- **(a) Mechanism.** Aliased positions are refused, not mis-corrected. The exposure is `Ok` (corrected or clean) on strings of the wrong length, including the cancelling pair 93 apart (P2d).
- **(b) The proposed fix**, "refuse lengths outside the ms1 set inside correct_checksum", also changes `decode_with_correction`, which calls `correct_checksum` first (`decode.rs:264`).
  - Today an over-long string with an aliased error gets `TooManyErrors` (exit 2, "more than 8 errors") from `ms repair`, ahead of the length error (P2c, huge). After the fix it would get `UnexpectedStringLength`.
  - That is probably better, but it changes an error kind the toolkit's `RepairError` mapping consumes (parity with the toolkit's D29). The fix needs a toolkit parity check and a test for each of P2b, P2c and P2d.
- Not a blocker for this PR; log it on the existing line when the line is next touched.

### Nit

- **N1** (`design/FOLLOWUPS.md:1398`). r1 M3 is folded into `ms-codec-correct-checksum-release`, which carries `grade=L`, a grade outside the I/M/N scale used by the other lines (only 2 lines use L). It also does not cite r1. The cross-reference sits only in the M4 line. Consider adding the r1 src to the release line.
- **N2** (`design/FOLLOWUPS.md:1400`). The GUI item is filed only here, as `repo=mnemonic-gui`. mnemonic-gui's `FOLLOWUPS.md` has no companion line, even though it already has a `bh1125-gui-review-r1-residue` section for this bug hunt. That breaks the mirrored-companion convention in the repo CLAUDE.md files, and the line also drops r1's concrete fix (the `ms_split` conditional).
- **N3** (`crates/ms-cli/src/cmd/repair.rs:161-175`). The new share path inherits the known Nit `ms-repair-accepts-mixed-case-with-typo` (FOLLOWUPS.md:927).
  - A mixed-case damaged share (P5) is repaired to the right share (exit 4, UNVERIFIED). The same share with no damage is refused as mixed case.
  - No wrong material results; the output equals the original share.
  - That entry's fix and its ignored RED row (`bughunt.rs::bughunt_ms_x1_mixed_case_correction`) cover only `decode_with_correction`. When it is fixed, put the case check in `correct_checksum` so the share path and the combine note are covered too.

## Bottom line

The fold is faithful and minimal. r1 M4 is not Important: no ms-cli verb, and no library caller that parses what it gets back, ends up with wrong secret material. The PR can proceed under Brian's rule. The items above are logging refinements only.
