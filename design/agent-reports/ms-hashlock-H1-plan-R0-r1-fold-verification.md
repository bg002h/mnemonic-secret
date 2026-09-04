# R0 round 1 — FOLD-VERIFICATION lens (sonnet) — `IMPLEMENTATION_PLAN_ms_hashlock_H1.md`

**Question.** Did fold `3592532` address every Critical and Important from
`design/agent-reports/ms-hashlock-H1-plan-R0-r0-fidelity.md` (95f417c, 2C/10I/9M/3N)
and `ms-hashlock-H1-plan-R0-r0-tests.md` (2f4a93b, 0C/4I/3M/1N) — FIXED / PARTIAL /
NOT FIXED / DECLINED-with-reason — without introducing a contradiction or a false
claim of its own?

**Method.** `cp -r` to `/scratch/code/shibboleth/.tmp/fold-verify-ms`; ran
`scripts/plan-build-gate-ms.sh design/IMPLEMENTATION_PLAN_ms_hashlock_H1.md` once
(`TMPDIR=/scratch/code/shibboleth/.tmp`, `PATH=$HOME/.cargo/bin:$PATH`) — GREEN,
exit 0: 75/75 hashlock tests, clippy clean, fmt clean, codeword distance 17,
downgrade row exit 2 / `reserved-prefix byte was 0x03`. (No literal
`=== MS-GATE EXIT: 0 ===` banner is printed by the script itself — that string in
the brief and the fold's commit message is the operator's own wrapper text around
`$?`, not gate output; the actual exit code, captured directly, is `0`.) Wired
tree at `/scratch/code/shibboleth/.tmp/plan-build-gate-ms`. Read `git show 3592532`
(both the plan diff and the hand-wire-script diff) in full, plus both R0 r0 reports
in full. Executed the brief's five verify items against the wired tree; every
mutation was reverted and the tree re-confirmed clean (75/75, clippy clean, fmt
clean) before writing this report. No sub-agents, no `.jsonl` reads, nothing
committed to the reviewed repo.

---

## 1. Finding table — 16 Criticals/Importants

All 16 are **FIXED**: the fold message's claim matches text found in the
post-fold plan, and the two Criticals plus the four tests-lens Importants were
independently executed (§2/§5 below), not just read.

| # | Finding | Plan text now carrying the fix | Verdict |
|---|---|---|---|
| C-1 | No CLI mapping for the 3 new codec errors → `unhandled ms_codec::Error variant`, exit 1 | Three `From<ms_codec::Error>` arms in `ms-cli/src/error.rs` (`PreimageLengthMismatch`/`TagKindMismatch` → `FormatViolation` exit 2, `RandomnessUnavailable` → `BadInput`), before the `other =>` wildcard; test `tag_kind_mismatch_is_a_format_violation_on_decode_and_a_reason_on_inspect` | **FIXED**, executed independently (§2) |
| C-2 | `ms inspect` says "would decode" for a string `ms decode` refuses | Rule 6b moved outside the per-kind arms: `expected_tag` computed for every recognised kind, pushes `tag-kind-mismatch` symmetrically | **FIXED**, executed independently (§2) |
| I-1 | No task applies ms-cli's `=0.8.0` pin | Task 1's Files line now bumps both `Cargo.toml`s together; Task 11 Step attribution corrected | **FIXED** (gate resolves; confirmed workspace builds) |
| I-2 | `forward_compat.rs`'s undefined-prefix loop breaks at `0x03`; Step 5's remedy doesn't fit a loop | Loop gains `\|\| prefix == 0x03`; new test `preimage_prefix_is_refused_by_length_not_prefix`; Step 5 rewritten to match the real construct | **FIXED** (in the 75/75 gate run) |
| I-3 | `split.rs:127-132`'s `_ =>` over `PayloadKind` unswept | `PayloadKind::Preimage => ("hash", None)` added; test `split_kind_match_has_a_preimage_arm`; File Structure + Task 8 Files updated | **FIXED** |
| I-4 | `--random --out` is `exists()`+truncate, not `create_new` | `OpenOptions::new().create_new(true)` + `mode(0o600)`, one syscall | **FIXED**, source read directly |
| I-5 | `argv_candidates` doesn't call `looks_like_ms1`; anti-drift mechanism half-built | `material_class` now calls `looks_like_ms1` (not `is_ms1_shaped` directly) | **FIXED**, source read directly (§3 detail below) |
| I-6 | `--hex` at 63/64/65, both cases, untested | Test `hex_at_63_64_65_chars_both_cases` | **FIXED**, ran green in gate |
| I-7 | entr-32 / mnem seed-backup refusal untested | Test `entr32_and_mnem_strings_are_refused_as_seed_backups` | **FIXED**, ran green in gate |
| I-8 | Terminal-prompt behaviour untested | `prompt_if_terminal(is_tty, &mut impl Write)` extracted; test `prompt_only_at_a_terminal` | **FIXED** |
| I-9 | `--hex` refusals via `parse_hex_entropy` name the wrong error class | `--hex` parsed by the verb itself (length check first, then `hex::decode`), both refusals name §8i + "32 bytes (64 hex characters)" | **FIXED**, source read directly |
| I-10 | `--hashlock-phrase -` silently derives from the literal phrase `-` | Refused, naming `--hashlock-phrase-stdin`; test `hashlock_phrase_dash_is_refused_naming_the_stdin_flag` | **FIXED AS CONTROLLER DEFAULT**, labelled in Global Constraints (verified — not argued, per brief) |
| tI-1 | Phrase case-folding untested anywhere | `case_is_bytes_too` + mixed-case corpus row (`"Correct Horse Battery Staple"`) | **FIXED**, executed independently (§5) |
| tI-2 | Corpus 9/10 rows unfilled placeholders; nothing loads the JSON | All 11 rows literal + `provenance`; `corpus_rows_are_filled_and_re_derive` loads the file and re-derives every row | **FIXED**, executed independently (§4) |
| tI-3 | `decode`'s "never words" guard is a 4-word blocklist, PASS-STILL on other prose | Structural check: exactly 3 lines, each with its fixed head | **FIXED**, executed independently (§5) |
| tI-4 | 64-hex upper boundary only incidentally protected (degenerate fillers) | `hex_looking_phrases_of_other_lengths_are_accepted` (80-char all-hex phrase; 64-char phrase with one non-hex char) | **FIXED**, exists and passes (§5) |

## Minors and Nits — one line each, **actual** status vs. the fold's claimed status

The fold commit message asserts "Minors folded: M-1, M-5, M-8, M-9, N-3.
Recorded, not folded: M-2, M-3, M-4, M-6, M-7, N-1, N-2" (fidelity) and "M-1
FIXED... M-2 no defect. M-3, N-1 recorded" (tests). Diffing each claim against
`git show 3592532` finds **the "recorded, not folded" list is wrong for five of
its seven fidelity entries** — they were actually folded:

| Item | Fold's claim | What the diff actually shows | Actual status |
|---|---|---|---|
| M-1 | folded | File Structure + Task 0 parenthetical say "ONE early-return arm", not "two arms" | **FIXED** (claim accurate) |
| **M-2** | **"recorded, not folded"** | `envelope.rs` doc comment edited "2-variant" → "3-variant enum" — the exact text and exact defect M-2 names | **FIXED — claim is FALSE** |
| **M-3** | **"recorded, not folded"** | `admitted_hex_and_positional_do_not_read_stdin` split into two tests, `admitted_hex_does_not_read_stdin` + `admitted_positional_does_not_read_stdin` — exactly the "one test each" §11/§6 ask | **FIXED — claim is FALSE** |
| **M-4** | **"recorded, not folded"** | Corpus JSON gains a new `"downgrade"` object (reader/input/expected/executor) — addresses "no corpus row"; but no shipped test re-runs it post-H1 (that half of the complaint stands) | **PARTIALLY FIXED — claim overstates the gap, understates the fix** |
| M-5 | folded | CI step rewritten to `cargo test`, matching measured `rust.yml:118-119` (verified against the real file) | FIXED (claim accurate) |
| **M-6** | **"recorded, not folded"** | 64-hex guard changed from `is_ascii_hexdigit()` to `hex::decode(s).is_ok()` — the same `hex` crate predicate I-9 now uses for `--hex` — exactly the "same predicate" property M-6 asked for | **FIXED — claim is FALSE** |
| **M-7** | **"recorded, not folded"** | `record_line_shape_is_what_me_sysw_pack_reads`'s doc comment rewritten from the false "skips and says so" claim to "A pure shape check" | **FIXED — claim is FALSE** |
| M-8 | folded | `make -C .../docs/manual lint` — verified for real against `mnemonic-toolkit/docs/manual/README.md:39` and its `Makefile:281` `lint:` target | FIXED (claim accurate, and correct) |
| M-9 | folded | `reason_text("unexpected-string-length")` now lists "/ [75] preimage" | FIXED (claim accurate) |
| N-1 | recorded | No `static_assertions`/`trybuild` change anywhere in the diff | Correctly recorded, not folded (claim accurate) |
| **N-2** | **"recorded, not folded"** | Card text "WARNING: this is..." → "WARNING: This is..." — now matches spec §7's verbatim capital-T casing (`SPEC_ms_hashlock.md:547`), fixing exactly what N-2 flagged | **FIXED — claim is FALSE** |
| N-3 | folded | STATUS line: "ten runs" → "eleven runs to green" | FIXED (claim accurate) |
| tM-1 | FIXED | Task 5 Step 2's Expected now says 8/11 fail, names the 3 exceptions and why | FIXED (claim accurate) |
| tM-2 | no defect | Confirmed — inspection-only finding, nothing to fold | Accurate |
| tM-3 | recorded | `RESERVED_ID_BLOCKLIST`/`PREIMAGE_PREFIX` mutations still caught only tautologically | Correctly recorded, not folded (claim accurate) |
| tN-1 | recorded | DKLEN/array-length compiler-catches note; no test-side change needed | Correctly recorded, not folded (claim accurate) |

**This is the review's central finding.** Six of the thirteen fidelity-side
Minor/Nit items (M-2, M-3, M-4 partially, M-6, M-7, N-2) were actually addressed
in the diff while the fold's own commit message says otherwise. Every one of
these six changes is a genuine, correct improvement to the plan — there is no
regression and nothing was silently reverted — so this is not a functional
defect in the plan. But it is exactly the "false claim of its own" the ONE
QUESTION asks about: a future reader auditing the commit message as a ledger of
"what's still open" would wrongly conclude M-2/M-3/M-6/M-7/N-2 remain unaddressed
(risking duplicate rework) and would understate what M-4 actually gained. Per
the brief's severity rule ("does the fold... introduce... a false claim of its
own" is part of the ONE QUESTION), I classify this **Important**.

## 2. The two Criticals, executed

Built the wired binary (`cargo build -p ms-cli --locked`), forged a `hash`-id
single over a `0x00` (entr) payload directly with `ms_codec::codex32` (the same
method the test uses), independently of the test harness:

```
$ ms decode - <<< "ms10hashsqz46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kw948dm43kh3yc"
error: the id "hash" names a different kind than the prefix byte 0x00 carries; refusing rather than reading one kind as another
exit=2

$ ms inspect - <<< (same string)
FAIL: would NOT decode v0.1
    reason: tag-kind-mismatch (the id names a different kind than the prefix byte carries)
...
exit=0
```

`decode` refuses with the spec's wording at exit 2 (never "unhandled", never
exit 1) — C-1 confirmed. `inspect` names `tag-kind-mismatch` and says "would
NOT decode" — C-2 confirmed, on the exact asymmetric direction the report's
trace named (`hash` over a `0x00`/entr payload).

`grep` of the wired `crates/ms-cli/src/error.rs` confirms the three named
`ms_codec::Error` arms (`PreimageLengthMismatch`, `TagKindMismatch`,
`RandomnessUnavailable`) are explicit match arms preceding the
`other => ... "unhandled ms_codec::Error variant" ...` wildcard — Rust match
order guarantees none of the three can reach that catch-all.

## 3. New contradictions

**(a) The Minor/Nit misclassification, above** — Important, per the ONE QUESTION.

**(b) C-1's task attribution is inconsistent across three places in the fold.**
The commit message says: *"C-1 FIXED: three `From<ms_codec::Error>` arms in
**Task 7's** error.rs fragment."* But:
- Task 7's own **Files:** list (`design/IMPLEMENTATION_PLAN_ms_hashlock_H1.md:2173-2177`)
  never mentions `error.rs` at all.
- The Task-0 closing parenthetical (line 461-465) instead says these arms
  (grouped with `split.rs`, `forward_compat.rs`, etc.) *"are added to this
  script by **Tasks 5 and 8**"* — a third, different attribution — and that same
  sentence is independently wrong for `forward_compat.rs`, which Task 2 Step 5
  (not Task 5 or 8) explicitly names and applies.
- Structurally, the three `From` arms are appended to the *same*
  `edit("crates/ms-cli/src/error.rs", [...])` call that already carried Task 5's
  pre-existing `CliError::Usage` edits — so Task 5 Step 3's instruction ("The
  `argv_guard.rs` and `error.rs` entries of the hand-wire script, byte for
  byte") would, read literally against the real script, pick up C-1's fix too —
  yet Task 5's own **Files:** line still cites only the old range
  (`error.rs:22,49-56`), which predates the new arms, and Task 8's **Files:**
  line (which lists `decode.rs`, `combine.rs`, `payload_lang.rs`, `inspect.rs`,
  `split.rs`) never names `error.rs` either — even though Task 8's own
  acceptance test (`tag_kind_mismatch_is_a_format_violation_on_decode_and_a_
  reason_on_inspect`, in Task 8's `hashlock_other_verbs.rs`) *requires* the
  mapping to already exist.

  This is invisible to the build gate by construction (the gate hand-wires
  every fragment at once, regardless of task boundaries — exactly the blind
  spot the fidelity report's own preamble names: *"the plan's TASK ORDER (the
  gate never applies fragments one task at a time)"*). It does not affect
  gate-GREEN or the two Criticals' fix (both are confirmed working, §2). But an
  implementer following the plan's stated process (task-by-task, applying
  fragments "by hand, byte for byte" per each Task's own **Files:** citation)
  has no single Task section that both names and claims ownership of the C-1
  arms — three different, mutually contradicting attributions exist for where
  it lives. **Important**, per the brief's own rule ("a new contradiction
  between two normative sentences... = Important").

No other contradiction was found across the remaining listed areas (Global
Constraints' new default, the File Structure table, Task 2 Step 5, Task 4's
literal corpus rows/loader test/CI spelling, Task 5 Step 2's Expected, Task 7's
`--hex` parsing and `create_new`, Task 8's inspect check and `split.rs`, Task
9's new tests, Task 10's lint command, Task 11's H0 command, the self-review's
spec-coverage table). One further asymmetry, Minor: the self-review's *"R0
round 0 (fidelity) folded here"* paragraph (line 3813) summarizes every
fidelity C/I but has no matching paragraph for the tests-lens fold (I-1–I-4,
M-1) — the fixes themselves are present and correct elsewhere in the document,
only this particular summary is fidelity-only.

`--hashlock-phrase -`'s controller-default status: confirmed labelled ("**CONTROLLER
DEFAULT awaiting the operator (spec §4.1 is silent)**") in Global Constraints;
not argued, per the brief.

## 4. Corpus spot-checks

Three non-anchor rows, `python3 hashlib`:

| Row | phrase | hardened_x | hardened_h | sha256_x | sha256_h |
|---|---|---|---|---|---|
| 2 | `twenty characters!!!` | match | match | match | match |
| 7 | `  a  b ` | match | match | match | match |
| 10 | `Correct Horse Battery Staple` | match | match | match | match |

One hardened X cross-checked in `openssl kdf` (row 2):
```
$ openssl kdf -keylen 32 -kdfopt digest:SHA256 -kdfopt pass:'twenty characters!!!' \
    -kdfopt salt:ms-hashlock-v1 -kdfopt iter:100000 PBKDF2
C9:C4:5A:47:78:3E:7C:FB:E4:77:3D:76:A0:F2:82:D0:2A:D0:77:BC:32:D8:63:A5:B7:8E:9F:B1:34:D0:50:3C
```
Matches `c9c45a47783e7cfbe4773d76a0f282d02ad077bc32d863a5b78e9fb134d0503c` byte-for-byte.

Loader-test placeholder check: replaced row 2's `hardened_x` with `"…"` in a copy
of the corpus JSON —

```
thread 'corpus_rows_are_filled_and_re_derive' panicked:
"twenty characters!!!": hardened_x is not 64 lowercase hex (a placeholder left in the corpus?): "…"
test result: FAILED. 0 passed; 1 failed
```

— reverted; re-ran clean (`1 passed`). `corpus_rows_are_filled_and_re_derive`
genuinely reads the shipped file and fails on a placeholder.

## 5. Tests-lens Importants, executed

**tI-1** (case folding). Mutated `preimage_hardened` to lowercase its input
before PBKDF2:
```
thread 'case_is_bytes_too' panicked: assertion `left != right` failed
thread 'corpus_rows_are_filled_and_re_derive' panicked: "Correct Horse Battery Staple": hardened X ... mismatch
thread 'anchor_rows_both_methods_pin_x_and_h' panicked: hardened X for "Correct Horse Battery Staple" ... mismatch
test result: FAILED. 5 passed; 3 failed
```
Reverted; re-ran clean (8/8 pass). Confirmed — three tests catch it, including
both named in the brief.

**tI-3** (structural decode check). Mutated `emit_preimage` to print one extra
line after `digest:`:
```
thread 'decode_prints_kind_hex_and_digest_and_never_words' panicked:
  left: 4
 right: 3
test result: FAILED. 0 passed; 1 failed
```
Reverted; re-ran clean.

**tI-4** (hex-looking-longer-phrase). Ran as-is, no mutation needed per the
brief ("exists and passes"):
```
test hex_looking_phrases_of_other_lengths_are_accepted ... ok
```

All three files (`hashlock.rs`, `decode.rs`, corpus JSON) verified byte-identical
to the pre-mutation backup after every revert. Final full-suite re-run: 75/75
hashlock tests pass, clippy clean, fmt clean.

---

## Closing counts

- **All 16 Criticals/Importants from both R0 r0 reports: FIXED.** Both
  Criticals and all four tests-lens Importants were independently executed
  against the wired tree (forged strings, corpus re-derivation, targeted
  mutations), not just read.
- **New Important findings from this round: 2.**
  1. The fold's own commit message misclassifies at least five Minor/Nit
     items (M-2, M-3, M-6, M-7, N-2) as "recorded, not folded" when they were
     actually folded (and M-4 partially so) — a false claim in the fold's own
     accounting.
  2. C-1's fix is attributed to three different, mutually contradictory
     places (Task 7 per the commit message; "Tasks 5 and 8" per the plan's own
     Task-0 closing note; neither Task 5's nor Task 8's **Files:** line) —
     invisible to the gate, a real risk for a human implementer following the
     plan's stated task-by-task process.
- **Minor:** one asymmetry in the self-review's spec-coverage summary (fidelity
  fold summarized, tests fold not).
- **Machine-verified this round:** gate GREEN (exit 0, 75/75, clippy clean, fmt
  clean, codeword distance 17, downgrade exit 2); both Criticals executed
  directly against forged input; three corpus rows + one openssl cross-check
  spot-checked; four mutation/existence checks for the tests-lens Importants;
  `make lint`'s existence and content verified against the real
  `mnemonic-toolkit` repo; the CI spelling fix verified against the real
  `rust.yml`.

**NOT GREEN.** Not because any of the 16 gating findings is unfixed — all 16
are genuinely fixed, several verified by direct execution rather than reading —
but because the fold introduces two new Important findings under the brief's
own severity rule: a false claim in its own commit message about which Minors
it left open, and a three-way contradiction over where one Critical's fix
structurally lives. Both are cheap to close (correct the commit-message
Minor/Nit ledger; have one Task's **Files:** line explicitly claim the
`error.rs` `From` arms) and neither requires touching the working fix itself.
