# Adversarial whole-diff review r1: mnemonic-secret draft PR #48 (`claude/project-thread-utm1aa`)

Scope: `git diff origin/master...claude/project-thread-utm1aa` (4 commits, d6c20a2..93dbc84): ms#2, ms#3, ms#1, FOLLOWUPS.
Reviewer: independent adversarial review, read-only (no tracked file edited; a temporary worktree at origin/master was used for the RED check and removed).

## Verdict

**0 Critical, 0 Important, 4 Minor, 6 Nit.** All three fixes do what they claim. No path changes what a card encodes or decodes to, or the passphrase/seed that reaches derivation. The tests fail without the fixes. In my view the PR can go forward; the Minors are doc, spec, GUI and release follow-ups, not defects in the code.

## Evidence gathered

- Scoped tests on the branch (`CARGO_INCREMENTAL=0 cargo test --locked -j 2`): bh1125_hex_language 8/8, bh1125_share_repair 6/6, bh1125_passphrase_control_char 6/6, cli_repair, cli_combine, combine_entropy_language_advisory, encode_hex_input, bh1119_printed_commands (30), bh1119_ms_docs, argv_remedy_runs, cli_help_pointer, cli_derive, cli_split, decode_routes_share_to_is_share_not_single_string: all passed. `cargo test -p ms-codec`: all passed. `cargo fmt --check`: clean. `cargo clippy -p ms-codec -p ms-cli --all-targets -D warnings`: clean. `ci/code-docs-check.sh`: clean.
- **RED check.** I ran the three new test files against **origin/master** source in a worktree. 13 tests that assert a fix FAIL there: hex_language 5, passphrase 4, share_repair 4. The 7 regression guards pass on both master and the branch: `encode_hex_card_is_unchanged`, `encode_phrase_with_non_english_language_still_works`, `combine_mnem_names_its_language_plainly`, `values_are_unchanged`, `ordinary_passphrases_do_not_warn`, `repair_of_a_valid_share_is_unchanged` and `repair_of_a_share_beyond_correction_is_refused`. That a guard passes on both is the evidence that the value or card it pins is unchanged.
- **Correction cross-checked independently against the BIP-93 text.** I wrote a Python harness with BIP-93's own `ms32_polymod` (GEN constants, init 0x23181b3, target 0x10ce0795c2fd1e62a). It was not derived from ms-codec. It split random 16- and 32-byte entropy (k 2..4, n k..5), damaged one random share with r random substitutions and ran `ms repair --json`.
  - r = 1..4, 200 trials: 200/200 exit 4, the output passed the BIP-93 reference checksum, the output equalled the original share, and stderr carried "UNVERIFIED … K-of-N share".
  - r = 5, 6, 8, 12, 400 trials: 400/400 exit 2 with nothing on stdout. No mis-correction was observed; the expected rate is about 1e-8 per trial.
  - All shares from `ms split` (rust-codex32) pass the BIP-93 reference, which confirms the reference constants. ms-codec's `MS_REGULAR_CONST` (`bch.rs:46`) equals BIP-93's short-code target.
- **Index probes** (vectors from `scripts/ms-printed-commands-fix-gate.sh`):
  - A valid index-`a` share is refused as before (exit 2, "use `ms combine`").
  - A damaged index-`a` share is repaired (exit 4). Uppercase input works too, and the output is lowercase.
  - A valid or damaged index-`s` threshold-2 string is still refused with the bh-ms-R11 "secret share" message (exit 2). The `index != 's'` guard holds.
- **The combine note's advice runs as printed.** `ms combine --to ms1 | ms decode - --language japanese` gives the Japanese phrase.

## Question 1: card and derivation invariance; can the ms#2 refusal fire on a legitimate call?

- `decode_with_correction` (`crates/ms-codec/src/decode.rs:263-275`) behaves exactly as before. It calls `correct_checksum`, which is the old body moved verbatim, with the same parse, residue, `decode_regular_errors`, bounds check and defensive re-verify.
  - With no correction needed, it still calls `decode(s)` on the **original** string, so case rules are unchanged.
  - With corrections, it calls `decode(corrected_str)` on the same `encode_ms1_string` output as before.
  - The errors and their order are unchanged.
- ms#2 (`encode.rs:138-146`) only adds an early `Err`. The Payload routing (`encode.rs:212`) is untouched. `--hex` alone and `--hex --language english` mint the byte-identical Entr card; `encode_hex_card_is_unchanged` passes on master and on the branch.
- **Can the refusal fire on a legitimate call?** clap's `default_value = "english"` means an omitted `--language` is English, so a default never trips it.
  - In the GUI, `ms encode` hides `--language` when `--hex` is set (`mnemonic-gui/src/form/conditional.rs:741-745`; Hidden suppresses argv), so encode is safe.
  - `ms split` in the GUI has **no conditional** (`mnemonic-gui/src/schema/ms.rs:990-996`, `conditional: None`), so a GUI user who picks a non-English language and `--hex` now gets exit 64. That is the intended refusal of exactly the dangerous pair, and it fails closed, but see M1.
  - The toolkit implements `ms-shares` itself and does not spawn `ms`.
  - `ms derive --hex --language` is untouched.
- ms#2 (combine, `combine.rs:284-304`): only the human text output changes, by adding the annotation and a stderr note. `--json` is unchanged. No in-constellation consumer parses `ms combine` text (grep of mnemonic-gui, toolkit and refugium).
- ms#3 (`passphrase_input.rs:558-574`): each arm returns exactly the value it returned before. Stdin calls `read_stdin_passphrase()`, Env calls `resolve_env(value.unwrap_or(""))`, Argv uses `admitted.or(value)`, and Absent returns `""`. Only an `eprintln!` is added. `values_are_unchanged` reproduces the bug report's fingerprints 3dd37943, 86d72e39 and a865e667 on both master and the branch. `derive.rs:514` is the only caller.

## Question 2: ms#1

- **The math.** No BCH code is new. `correct_checksum` is the pre-existing correction body, and both the independent BIP-93 cross-check and the fuzz above confirm it.
- **Where the share path is reached.** `repair.rs:161-175` is reached only after `decode` refused the corrected string with `IsShareNotSingleString`. `decode` runs `is_known_length` (`decode.rs:48`) and rust-codex32 `from_string` before the threshold dispatch (`envelope.rs:131-150`). So only known lengths with a valid BIP-93 short checksum after correction get there. The corrected string is also re-parsed with `Codex32String::from_string`.
- **The output is not more secret than before.** The printed string is `reconstruct_corrected(original, corrections)`, which the fuzz showed equals the original share. Before this PR a share drew no stdout output. Now one corrected share, the same material the user typed, goes to stdout or `--out` with the existing `PrivateKeyMaterial` advisory. Nothing more secret than the input is printed. The combine note names shares by position only, and the test asserts the share text is not echoed.
- **"Corrected is not verified" is kept.** A corrected share exits 4 with `verdict: "candidate"`, never 0 or 5. A clean share is still refused with exit 2. TooManyErrors exits 2 with empty stdout. The UNVERIFIED text says recombining cannot verify and tells the user to confirm the address or xpub. The mis-correction risk is stated in the stderr line and in the `correct_checksum` rustdoc.
- **The API change is additive.** One new `pub fn` re-exported from `lib.rs:56`, and no signature changes. Releasing it is M3.

## Question 3: ms#3

- The value is unchanged (Question 1).
- The warning (`control_char_warning`, `passphrase_input.rs:586-592`) contains only the channel name: "stdin", "argv", or "environment variable <VAR>". The variable name is from argv and not secret. The tests assert that "TREZOR" and "ZOR" are absent from stderr.

## Question 4: tests

The tests are sound and RED on master (above). Strong points:

- The repair test asserts that the corrected share equals the original and recombines to the original entropy.
- `refusal_command_runs_as_printed` executes the printed remedy, for both verbs, through to a Japanese decode or combine.
- The combine test asserts the right share index and the absence of the other one.

The gaps are N5 and the Minor note in M4.

## Findings

### Minor

**M1. GUI `ms split` can now emit a refused pair, and the GUI help says "ignored under --hex". No GUI follow-up is filed.**
- Where: `mnemonic-gui/src/schema/ms.rs:607-616` (help "ignored under --hex"; `ms split` has `conditional: None` at `:990-996`). The repo hazard is "GUI mirror (ms-cli)".
- Scenario: a GUI user leaves Japanese selected from an earlier phrase split, switches to `--hex` and presses Split. The GUI help promised the language would be ignored, and the run now exits 64 with a usage error.
- Effect: it fails closed and is arguably correct. But the mirror text is now false, and `ms-bh1125-toolkit-parity` names only the toolkit manual.
- Fix: file a mnemonic-gui companion FOLLOWUP to update both help strings and add an `ms_split` conditional mirroring `ms_encode`, or record why not.

**M2. A normative spec still says `--language` is "ignored under `--hex`".**
- Where: `design/SPEC_ms_cli_v0_1.md:57`. `SPEC_ms_mnem_wordlist_language.md:41-43` still holds: hex carries no language, so the card is always `entr`.
- Scenario: a later implementer conforms to the spec table and removes the refusal as "non-conforming". Nothing in the spec records the change.
- Fix: amend the table row to "refused with `--hex` when non-English (bug hunt 2026-11-25 ms#2)", or add an erratum.

**M3. A new public ms-codec API without a version bump.**
- Where: `crates/ms-codec/src/lib.rs:56`, CHANGELOG `[Unreleased]`. ms-cli pins `ms-codec = "=0.10.1"` (path dependency).
- Effect: two different crates called "0.10.1" now exist (crates.io and this tree), and a crates.io build of ms-cli at this pin does not compile.
- Already filed as `ms-codec-correct-checksum-release`, so it is acknowledged. Listed so the release gate does not miss it, because this hazard ("Consumer pins") is named in CLAUDE.md.

**M4. `correct_checksum` is public and does not bound the length, so it will "correct" strings the short code does not cover.**
- Where: `decode.rs:299-350`. The doc says it does not check length.
- The problem (by reading, not reproduced): the short-code locator has order 93, so for a data part longer than 93 symbols, or for a long-code codex32 string, Chien positions alias. The function can return `Ok` with a "corrected" string that is not a valid codex32 string, or report spurious corrections.
- Reach in ms-cli:
  - `ms repair` cannot reach this, because `decode`'s `is_known_length` gate runs first.
  - `ms combine`'s note (`combine.rs:225-242`) can tell the holder of a wrong-length string "does not pass its checksum; `ms repair` can propose a correction", and `ms repair` then refuses it. The output is misleading text, not wrong secret material.
- Risk: the public-API foot-gun is for future callers such as the toolkit.
- Fix: refuse data parts longer than 93 (or outside the ms1 length set) with `UnexpectedStringLength` in `correct_checksum`, and add a test.

### Nit

- **N1.** `ms repair --help` (`main.rs` about text) still describes only single-string correction. Add one sentence on the K-of-N share path and its exit 4.
- **N2.** The refusal's printed remedy (`encode.rs:141-145`, via `note_verb`) keeps `-k/-n` but drops `--out`, `--json` and `--group-size`. An operator who re-runs it verbatim gets the card on stdout instead of the file they had asked for. It is the same class as other printed remedies, but worth a "(add your other flags)" phrase.
- **N3.** `repair.rs:170` copies the corrected share into a plain `String` for `Codex32String::from_string`. `combine.rs:229` drops the corrected string in `Zeroizing`, but the `CorrectionDetail` `was`/`now` chars are not scrubbed. This matches the existing posture recorded in `envelope.rs:96-103` (the pipeline holds unwrapped copies), so it is hygiene only.
- **N4.** The warning says "--passphrase from stdin" when the flag used was `--passphrase-stdin`. Also, `char::is_control` does not catch invisible non-control characters (U+200B, U+00A0, U+2028). Those are out of scope but adjacent.
- **N5.** No test covers a damaged index-`s` share. It is still refused with the "secret share" message, which does not mention that the string was also corrected or damaged; I verified this by probe. The behaviour is pre-existing, but a regression test would pin the `index != 's'` guard at `repair.rs:162`.
- **N6.** The JSON envelope (`kind: "ms1"`, `verdict: "candidate"`) does not distinguish a corrected share from a corrected single-string card. A parser that follows the verdict alone would show single-card guidance for a share. The schema is pinned to the toolkit's `RepairJson` (D27), so this is a cross-repo schema question, not a fix for here.

## Mutation and RED record

| Test file | Fails on master (the fix is needed) | Passes on both (regression guards) |
|---|---|---|
| bh1125_hex_language | 5: refused ×2 (encode, split), json_envelope, refusal_command_runs_as_printed, combine_entr_marks_english_as_default | 3 |
| bh1125_passphrase_control_char | 4: extra_blank_line, two_lines, tab_or_lone_cr, env_interior_newline | 2 (values_are_unchanged, ordinary_passphrases_do_not_warn) |
| bh1125_share_repair | 4: corrects_damaged, json_candidate, out_receives, combine_names | 2 (valid_share_unchanged, beyond_correction_refused) |
| ms-codec unit tests (3) | n/a: they use the new API, which does not compile on master | — |
