# FIX: `ms hashlock --emit-record --no-engraving-card` prints the requested record nowhere (bh-ms-I4)

Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-22. FIX table row #47 of
`coordinator/agent-reports/architect-bughunt-1119-questions.md` (L159). **Small track, grade I**, ruled by
architect **T26** (same report, L66): *refuse `--emit-record --no-engraving-card` in text mode (exit 64), naming
`--json` as the route that carries the record.* Base: `origin/master` `423f1734` (ms-cli 0.21.0). Sources:
`design/agent-reports/bug-hunt-2026-11-19.md` L110-121 (consultant, I4), `coordinator/agent-reports/bughunt-1119/`
`triage-ms-t2.md` L10 (RED/GREEN), `grade-ms.md` L35 (graded I). No product ruling (B1-B10) bears on it.

Every statement of current behaviour below was measured on a debug build of `423f1734` (cargo 1.85.0). The fixed
behaviour was measured on a scratch build with the refusal of section 3 applied and then reverted (not committed).

## 1. Summary

In text mode the `phrase:` record that `--emit-record` asks for is printed only on the stderr card
(`hashlock.rs:601-607`); stdout is the public `hash:` line alone (`:509-511`). `--no-engraving-card` suppresses the
card (`:514`), so the pair exits 0 with the record on neither stream, and the surviving notice at `:636-646` still
says `THIS RECORD CARRIES THE PHRASE.` about a record that was never printed. The same holds with `--kind` and with
`--out FILE` (the file holds the preimage ms1, not the record). Under `--json` the record is in `phrase_record`
(`:505-507`) with or without the card, so that pair is unaffected. The fix refuses the text-mode pair before
anything is read or written.

## 2. Spec delta (holder-visible behaviour)

**R1. Refusal.** `ms hashlock` with `--emit-record` and `--no-engraving-card` and without `--json` exits **64**,
prints nothing on stdout, and prints exactly one stderr line, `error: <MSG>`, where `<MSG>` is
(PROPOSED, architect confirms):

    --emit-record has nowhere to print the `phrase:` record under --no-engraving-card: in text mode the record goes on the card, and stdout carries only the public digest. Add --json to get it in the JSON object (phrase_record), or drop --no-engraving-card to get it on the card.

It holds for every phrase source (`--hashlock-phrase-stdin`, and `--hashlock-phrase` once the argv guard admits it),
every `--kind`, `--method`, and with or without `--out`.

**R2. Before any I/O.** The refusal comes before the phrase is read (stdin is not consumed; an empty or invalid
phrase is not diagnosed) and before `--out` is opened (no file is created or truncated).

**R3. Order** (PROPOSED, architect confirms). The checks that already run first keep running first: clap parsing,
the argv guard, `pick_source`'s zero/two-source refusals, and the existing `--emit-record needs a phrase` refusal
(`hashlock.rs:394-401`). So `--random --out F --emit-record --no-engraving-card` keeps today's needs-a-phrase message
(exit 64, F not created). Reason: that message is true in every mode, while R1's message tells the holder to add
`--json`, which would only lead to the needs-a-phrase refusal.

**R4. Unchanged.** `--emit-record` with the card (text), `--emit-record --json` with or without the card, and
`--no-engraving-card` without `--emit-record` behave exactly as today, bytes and exit code included. Text-mode stdout
stays the `hash:` line alone (T26: the record is a secret and stdout is the public channel).

**Help text** (`--emit-record` doc comment, `hashlock.rs:84-90`): keep the existing text and append one sentence
(PROPOSED, architect confirms):

    Text mode without the card has nowhere to put it: --emit-record --no-engraving-card without --json is refused (exit 64).

**Spec text to add** (`design/SPEC_ms_hashlock.md` §4.4, a new last bullet after the `--json` bullet that ends at
L497 "(R0 r0 correctness I-3, I-4.)", before the blank line and `---`):

    - **`--emit-record`** (F-495; phrase sources only, otherwise exit 64 "needs a phrase"): the `phrase:` record
      goes on the card (`record (phrase):`) in text mode and into `phrase_record` under `--json`, never on the
      text-mode stdout, which stays the public digest alone. **Text mode with `--no-engraving-card` is refused,
      exit 64** (bh-ms-I4, architect T26): there is no card to carry the record, and stdout is the channel this
      verb keeps public. The refusal follows the §4.1 source checks and the needs-a-phrase refusal and precedes
      reading the phrase and writing `--out`. `--json --no-engraving-card` is admitted: the object carries it.

**CHANGELOG** (`CHANGELOG.md`, under `## ms-cli [Unreleased]`, new `### Fixed` heading):

    - **`ms hashlock --emit-record --no-engraving-card` is refused in text mode** (exit 64, bh-ms-I4). It exited 0
      with the `phrase:` record on neither stream while the stderr notice said THIS RECORD CARRIES THE PHRASE. The
      refusal names `--json` (the record in `phrase_record`) and the card; nothing is read or written before it.

## 3. Code delta (production; prose, no code)

`crates/ms-cli/src/cmd/hashlock.rs` only:

1. In `run`, directly after the existing needs-a-phrase refusal (ends L401) and before `let is_random` (L402):
   when `args.emit_record && args.no_engraving_card && !args.json`, return `CliError::Usage(<MSG>)` (exit 64 via
   `error.rs:59`; `main` prints `error: <MSG>`). This is before `derive` (L403, where the phrase is read) and before
   the `--out` write (L418-428), which gives R2.
2. Append the help sentence of section 2 to the `emit_record` doc comment (L84-90).

Nothing else changes. The `THIS RECORD CARRIES THE PHRASE` notice (L636-646) and the F-535 note (L686-699) keep their
guards; in text mode they now only follow a card that printed the record. No flag is added or removed, so the
`gui_schema_emits_spec_v7_json.rs:283` flag total (70) is unchanged.

## 4. RED rows

New file `crates/ms-cli/tests/bh1119_hashlock_emit_record.rs`, one `#[test]` per row, named exactly as the row,
each running the built `ms` (`assert_cmd`, as `hashlock_emit_record.rs` does) in its own `tempfile::tempdir()` as cwd.
`<MSG>` is the R1 text; `REC` is `phrase:68617264656e65642c747261696c` (hex of `hardened,trail`). Shell spelling
below is what was run on the base (`M` = the built `ms`).

| row (= test fn) | command | measured on base `423f1734` | expected after the fix |
|---|---|---|---|
| `bh1119_ms_i4_emit_record_no_card_text_refused` | `printf %s trail \| $M hashlock --hashlock-phrase-stdin --emit-record --no-engraving-card` | exit 0; stdout `hash:8d429098f143fb65ac17e10380ad2e53ccb1677bc514cd10335ba38cc1eea23c`; stderr (8 lines) starts `THIS RECORD CARRIES THE PHRASE.`; `REC` on neither stream | exit 64; stdout empty; stderr exactly `error: <MSG>\n` |
| `bh1119_ms_i4_emit_record_no_card_kind_refused` | as above plus `--kind ripemd160` | exit 0; stdout `hash:ripemd160:17492aea148ed4d95834293cf2ad875b7def4f45`; stderr includes the F-535 `NOTE: the phrase: record above carries the METHOD` about a record not printed; `REC` on neither stream | exit 64; stdout empty; stderr exactly `error: <MSG>\n` |
| `bh1119_ms_i4_emit_record_no_card_out_refused_before_write` | as the first row plus `--out p.ms1` | exit 0; `p.ms1` created (mode 0600, the preimage ms1); `REC` in neither stream nor the file | exit 64; stdout empty; stderr exactly `error: <MSG>\n`; `p.ms1` does not exist |
| `bh1119_ms_i4_emit_record_no_card_refused_before_reading_phrase` | `printf '' \| $M hashlock --hashlock-phrase-stdin --emit-record --no-engraving-card` | exit 1; stderr `error: the hashlock phrase is empty` | exit 64; stdout empty; stderr exactly `error: <MSG>\n` |

Controls (pass on both builds; they pin R3 and R4):

| row (= test fn) | command | expected, both builds (measured on base) |
|---|---|---|
| `bh1119_ms_i4_control_card_carries_record` | `printf %s trail \| $M hashlock --hashlock-phrase-stdin --emit-record` | exit 0; stdout exactly `hash:8d429098f143fb65ac17e10380ad2e53ccb1677bc514cd10335ba38cc1eea23c\n`; stderr has the line `record (phrase): REC` exactly once |
| `bh1119_ms_i4_control_json_no_card_carries_record` | `... --emit-record --json --no-engraving-card` | exit 0; stdout JSON `phrase_record` == `REC`; stderr is exactly the one private-key-material advisory line (`warning: stdout carries private key material (can spend) — ...`) |
| `bh1119_ms_i4_control_json_with_card_carries_record` | `... --emit-record --json` | exit 0; stdout JSON `phrase_record` == `REC`; stderr has the line `record (phrase): in the JSON (phrase_record)` exactly once |
| `bh1119_ms_i4_control_no_phrase_source_refusal_first` | `$M hashlock --random --out r.ms1 --emit-record --no-engraving-card` | exit 64; stderr one line starting `error: --emit-record needs a phrase:`; `r.ms1` does not exist |
| `bh1119_ms_i4_control_no_card_without_flag_unchanged` | `printf %s trail \| $M hashlock --hashlock-phrase-stdin --no-engraving-card` | exit 0; stdout exactly the `hash:8d42...a23c` line; `phrase:` nowhere on stderr |

The builder runs the new file on the base first and records `9 tests run: 5 passed, 4 failed` (the four RED rows
fail) before writing the code.

## 5. Tests that change

- **`crates/ms-cli/tests/hashlock_emit_record.rs` `emit_record_says_the_phrase_record_does_not_carry_the_kind`
  (L185-229)**: its base argv (L205) is `["hashlock", "--hashlock-phrase-stdin", "--no-engraving-card"]`, so its first
  two cases use the refused pair. Drop `"--no-engraving-card"` from L205 (the three cases then run with the card) and
  add one doc-comment sentence: the cases run with the card because text mode without it refuses `--emit-record`
  (bh-ms-I4). Measured on the base with the card on: the NOTE is present for `--kind ripemd160 --emit-record`,
  absent for `--kind sha256 --emit-record` and for `--kind ripemd160`, so the test's expectations hold unchanged.
  This is the consultant's "move the notice test to an invocation that actually emits its record".
- **No other existing test changes.** Scratch build (refusal applied, reverted): workspace
  `758 tests run: 757 passed, 1 failed, 23 skipped`, the one failure being the test above.

## 6. Toolkit manual lockstep

`mnemonic-toolkit` `origin/master` `04c56bfa`, `docs/manual/src/40-cli-reference/43-ms.md` L723-728 (the
"`ms hashlock --emit-record` writes the `phrase:` record" paragraph): append after its last sentence (PROPOSED,
architect confirms):

    With `--no-engraving-card` and no `--json` there is no card to carry it and stdout stays the digest alone, so `ms hashlock` refuses the pair (exit 64) and names `--json` and the card as the two routes.

No flag-table change (no flag added or removed). mnemonic-gui: no schema change (flag set unchanged); see the
proposed gui FOLLOWUPS line in the report for greying the pair in the form.

## 7. Coverage map (rule -> rows)

| rule | rows |
|---|---|
| R1 | bh1119_ms_i4_emit_record_no_card_text_refused, bh1119_ms_i4_emit_record_no_card_kind_refused |
| R2 | bh1119_ms_i4_emit_record_no_card_out_refused_before_write, bh1119_ms_i4_emit_record_no_card_refused_before_reading_phrase |
| R3 | bh1119_ms_i4_control_no_phrase_source_refusal_first |
| R4 | bh1119_ms_i4_control_card_carries_record, bh1119_ms_i4_control_json_no_card_carries_record, bh1119_ms_i4_control_json_with_card_carries_record, bh1119_ms_i4_control_no_card_without_flag_unchanged; the existing seven tests of hashlock_emit_record.rs (one changed per section 5) |

## 8. Gate (builder runs once to a file and greps it)

Rust pin `cargo 1.85.0`, `$HOME/.cargo/bin` first in PATH, `--locked`, no `--release`, worktree-own `target/`:

    cargo nextest run --locked --workspace --build-jobs 2 -j 2 --no-fail-fast
    cargo test --locked --doc --workspace
    cargo clippy --locked --all-targets --workspace -- -D warnings
    cargo +1.95.0 fmt --all -- --check
    bash ci/code-docs-check.sh && bash ci/code-docs-check.test.sh && bash ci/docs-only.test.sh

Expected: `767 tests run: 767 passed, 23 skipped` (base 758 plus the 9 rows). Not covered: the macOS, musl,
release, miri, FreeBSD and fuzz CI legs (CI on the PR head), the toolkit manual's own CI, review.

## 9. FOLLOWUPS

None appended in the PR. The report lists the lines to direct-push after merge (one closing line for I4 in ms, one
Minor line for the mnemonic-gui form).
