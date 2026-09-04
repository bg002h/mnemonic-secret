# H1 implementation report — `ms hashlock`, ms-codec 0.8.0 / ms-cli 0.18.0

**Author:** the single H1 implementer (one agent, worktree
`/scratch/code/shibboleth/ms-worktrees/hashlock-h1`, branch `hashlock-h1`).
**Plan:** `design/IMPLEMENTATION_PLAN_ms_hashlock_H1.md` at ms `4dbff0b`
(STATUS R0 GREEN). **Spec:** `design/SPEC_ms_hashlock.md`.
**Base:** `4dbff0b` (master's tip when the worktree was cut; master has since
gained one unrelated commit, `c985f02`, a push report — no conflict, the merge
base is still `4dbff0b`).
**Branch tip:** `78ab6a216193e234c4071d46f9b51046f7f8d7fd`.
**Scope executed:** Tasks 1–10. Task 0 was already committed at the base;
Task 11 (the release) was NOT executed — see *Not done*.

Nothing was pushed. No repository other than `mnemonic-secret` was touched, and
no commit was made on `master`.

---

## Per-task record

Every command below ran with
`PATH=$HOME/.cargo/bin:$PATH TMPDIR=/scratch/code/shibboleth/.tmp
CARGO_TARGET_DIR=/scratch/code/shibboleth/mnemonic-secret/target`.

### Task 0 — the gate and the hand-wire script

**Not re-done: already committed at the base.** `scripts/plan-build-gate-ms.sh`
(142 lines) and `scripts/plan-handwire-ms-hashlock.py` (235 lines at the base)
exist at `4dbff0b`; the plan's STATUS records eleven gate runs to green. Per the
brief, `plan-build-gate-ms.sh` was **not run** by the implementer. The hand-wire
script's `edit(path, [(anchor, replacement), …])` entries were used as the
byte-exact source for every fragment, applied to the working tree one task at a
time by a selective applier that `exec`s the chosen top-level statements of the
committed script (so no fragment was retyped).

### Task 1 — `bd76cec` — constants, tag, errors, blocklist, both version bumps

- **RED:** `cargo test -p ms-codec --test hashlock_kind` →
  `error: could not compile ms-codec (test "hashlock_kind") due to 16 previous errors`,
  first being
  `error[E0432]: unresolved import` on `PREIMAGE_PREFIX`/`TAG_HASH`/
  `VALID_PREIMAGE_STR_LENGTHS`, plus
  `error[E0599]: no variant named TagKindMismatch found for enum ms_codec::Error`.
- **After the fragments (still red, one step further, as the plan's Step 4
  predicts):** `error[E0599]: no variant or associated item named 'Preimage'
  found for enum 'Payload'` and `no method named 'single_tag' found for enum
  'PayloadKind'` — 5 errors.
- **PASS count:** none yet by design (Task 2 turns the file green).
- **Deviation (staging):** the plan's Step 5 `git add` list omits the two
  `Cargo.toml` files and `Cargo.lock`, although its own Step 3 and Files list
  say both version lines and the `=0.8.0` pin move in this task. All three were
  staged here, or the tree would have carried them into an unrelated commit.

### Task 2 — `ff5da74` — `Payload::Preimage`, dispatch, accept set, tag/kind checks

- **RED:** after appending the plan's second `hashlock_kind.rs` block —
  `error[E0599]: no variant or associated item named 'Preimage' found for enum
  'Payload'` … `for enum 'InspectKind'`; 7 errors.
- **PASS:** `cargo test -p ms-codec --test hashlock_kind` →
  `test result: ok. 14 passed; 0 failed; 0 ignored`. `codeword distance
  entr/hash = 17` printed (spec §1 needs > 8).
- **Step 5 RED, verbatim:** `forward_compat.rs:57` —
  `prefix 0x03: expected ReservedPrefixViolation, got Error("preimage payload
  is 16 bytes after the prefix; a hashlock preimage is exactly 32 bytes (64 hex
  characters)")`. Fixed by the script's `forward_compat.rs` entry (the loop
  skips `0x03`).
- **Whole-crate gate:** `cargo nextest run -p ms-codec --locked --no-fail-fast`
  → `181 tests run: 181 passed, 0 skipped`.
- **DEVIATION (code — the one real plan defect found):** the plan's `encode.rs`
  fragment inserts the tag/kind check at the **top** of `pub fn encode`, before
  the `RESERVED_NOT_EMITTED_V01` check. As written it fires first for `seed`
  and `xprv`, replacing the shipped v0.1 SPEC §4 rule 7 error
  (`ReservedTagNotEmittedInV01`) with `TagKindMismatch`, and two shipped unit
  tests went red:
  `ms-codec encode::tests::encode_rejects_seed_tag` and
  `encode_rejects_xprv_tag` (`181 tests run: 179 passed, 2 failed`).
  **What I did:** moved the check to sit **after** the reserved-not-emitted
  check, with a comment naming why, and moved the hand-wire script's entry with
  it so the script and the tree cannot drift.
  **Why this and not something else:** it is the smallest edit that keeps a
  shipped error contract intact, and it leaves every refusal the hashlock spec
  names in force — `id_and_prefix_must_agree_both_directions` still passes both
  directions (`hash` over a `0x00` payload, `entr` over a `0x03` payload). The
  alternative, narrowing the predicate to `tag ∈ {entr, hash}` to mirror
  decode's rule 6b, would also work but weakens the emit-side refusal for
  arbitrary ids; nothing in production encodes one (`grep -rn "Tag::try_new"
  crates/` → 8 hits, all in tests or in the decode path).
- **Plan count note:** the plan's Step 4 Expected says "all eleven tests";
  `hashlock_kind.rs` carries **fourteen**.

### Task 3 — `571d88c` — `ms_codec::hashlock`

- **RED:** `cargo test -p ms-codec --test hashlock_derivation` →
  `error[E0432]: unresolved import 'ms_codec::hashlock'` (plus
  `error: couldn't read crates/ms-codec/tests/vectors/hashlock-v0.8.json`).
- **PASS:** `test result: ok. 8 passed; 0 failed; 0 ignored`
  (the plan's Expected says "six tests"; the file carries eight).
- **Whole-crate gate:** `cargo nextest run -p ms-codec --locked --no-fail-fast`
  → `190 tests run: 190 passed, 0 skipped`.
- **Dependency check, measured not assumed:**
  `cargo tree -p ms-codec -e features -i pbkdf2` →
  `pbkdf2 v0.12.2 └── pbkdf2 feature "hmac"`; `grep -n password-hash Cargo.lock`
  → no match.
- **DEVIATION (task boundary):** `crates/ms-codec/tests/vectors/hashlock-v0.8.json`
  is the plan's **Task 4 Step 1** file, but `hashlock_derivation.rs`'s
  `corpus_rows_are_filled_and_re_derive` reaches it through `include_str!`,
  which is a **compile-time** dependency. Task 3 cannot compile, let alone pass,
  without it. The corpus was therefore created here, verbatim from the plan's
  fenced block, and committed with Task 3.
- **Independent machine-check of the corpus (not merely "the tests pass"):** all
  11 derivation rows re-derived in `python3 hashlib` outside the crate —
  `hardened_x`, `hardened_h`, `sha256_x`, `sha256_h` and `phrase_chars` for
  every row — **0 mismatches**; and `sha256(kind[0].preimage_hex) ==
  kind[0].digest` → True.

### Task 4 — `dac6dbb` — the reproduction test and the CI job

- **PASS:** `cargo test -p ms-codec --test hashlock_repro -- --nocapture` →
  `test result: ok. 2 passed; 0 failed; 0 ignored`.
  Local tools: `OpenSSL 3.6.3 9 Jun 2026`, `Python 3.14.7`.
- **Mutation 1 (plan Step 3), `HASHLOCK_ITERATIONS` 100_000 → 10_000:**
  ```
  test constants_equal_the_literals ... FAILED
  test hashlock_repro_three_ways ... FAILED
  assertion `left == right` failed
    left: 10000
   right: 100000
  assertion `left == right` failed: Rust vs literal
    left: "b79f3014492727d897eb5f98aeb02048b98ad00a23cbf6c0239322b59923eaeb"
   right: "c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016"
  ```
  Python and openssl still agreed with each other — the literal-vs-constant
  separation works. Reverted.
- **Mutation 2 (plan Step 3), `PATH=/nonexistent`:**
  ```
  test hashlock_repro_three_ways ... FAILED
  python3 must be present: this test FAILS on a missing tool, never skips:
    Os { code: 2, kind: NotFound, message: "No such file or directory" }
  ```
  Never `ok`, never `ignored`. Reverted.
- **CI edits:** the preflight step before, and the run-by-name step after, the
  `test-ms-codec` job's test step. The YAML parses and the job's steps are, in
  order: preflight / `cargo test -p ms-codec` / run-by-name. The run-by-name
  gate was reproduced locally:
  `cargo test -p ms-codec --locked --test hashlock_repro -- --exact
  hashlock_repro_three_ways` →
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out`,
  and `grep -E "test result: ok. 1 passed"` matched.

### Task 5 — `ac9b917` — the argv guard, `CliError::Usage`, the three `From` arms

- **RED, and it DIVERGED FROM THE PLAN'S EXPECTED (in the tests' favour).** The
  plan's Step 2 says "eight tests FAIL with clap's `unrecognized subcommand
  'hashlock'`. Three do NOT", naming the argv-refusal test as one that already
  passes because the guard runs before clap. Measured: **15 of 15 FAILED.**
  Verbatim:
  ```
  ---- hashlock_phrase_on_argv_is_refused_without_the_allow_flag_and_never_echoed ----
  assertion `left == right` failed: error: unrecognized subcommand 'hashlock'
    left: Some(64)
   right: Some(1)
  ```
  The cause: `SUBCOMMANDS` has no `"hashlock"` before this task, so the guard
  does not recognise the verb and clap answers instead. The plan's underlying
  point — clap's unrecognized-subcommand error is **also exit 64**, so the
  exit-64 tests must assert the MESSAGE too — is confirmed by that same
  `left: Some(64)` and is unchanged.
- **After the fragments (plan Step 4):** the argv-refusal test PASSES; running
  the binary shows the class wording landed —
  `ms: argument 3 on ARGV … is a hashlock phrase, 28 characters long.`, exit 1,
  the phrase not echoed. 14 still red (no verb yet).
- **Regression gate:** `cargo nextest run -p ms-cli --locked --no-fail-fast -E
  'not binary(hashlock_sources)'` → `310 tests run: 310 passed, 11 skipped`.
- **Note, not a defect:** `CliError::Usage` is dead code between Tasks 5 and 7
  (`warning: variant 'Usage' is never constructed`). Task 7 constructs it and
  the final clippy `-D warnings` run is clean.

### Task 6 — `4348312` — the byte-verbatim reader and the phrase rule

- **RED, genuinely test-first:** the file was written with the three pure
  functions (`strip_one_trailing_newline`, `prompt_if_terminal`,
  `validate_phrase`) as `unimplemented!()` bodies; all 8 unit tests FAILED with
  `not implemented: body not written yet` at
  `crates/ms-cli/src/hashlock_phrase.rs:61` and `:89`. The plan's bodies were
  then pasted.
- **PASS:** `cargo test -p ms-cli hashlock_phrase` →
  `test result: ok. 8 passed; 0 failed; 0 ignored` (the plan's Expected says
  "seven unit tests"; the file carries eight).
- **Mutation (plan Step 2), `(0x20..=0x7e).contains(*b)` → `b.is_ascii()`:**
  ```
  test printable_boundary_is_pinned_on_both_sides ... FAILED
  assertion `left == right` failed
    left: Ok(())
   right: Err(NotPrintableAscii { byte: 9, at: 1 })
  ```
  TAB was admitted. `refusals_never_echo_the_phrase` failed with it. Reverted.

### Task 7 — `3ec623e` — `ms hashlock`

- **PASS:** `cargo test -p ms-cli --test hashlock_sources` →
  `test result: ok. 15 passed; 0 failed; 0 ignored` (the plan's Expected says
  "all eleven tests"; the file carries fifteen). This is the RED from Task 5
  turning green.
- **Five mutations, each failing its named test, each reverted:**
  | mutation | test | evidence |
  | --- | --- | --- |
  | `--random` gate widened to `args.out.is_none() && !args.json` | `random_requires_out_file_and_json_alone_does_not_satisfy_it` | `assertion left == right failed: --json alone must not satisfy the gate; left: Some(0), right: Some(64)` |
  | `write_artifact_create_new` → `write_artifact` for `--random` | `random_out_refuses_to_overwrite_but_other_sources_overwrite` | `left: Some(0), right: Some(64)` |
  | `refuse_method` disabled | `method_with_a_supplied_preimage_exits_64_for_all_three_sources` | `["hashlock","--hex","-","--method","sha256"]: … left: Some(0), right: Some(64)` |
  | the `--hashlock-phrase -` arm disabled | `hashlock_phrase_dash_is_refused_naming_the_stdin_flag` | `left: Some(0), right: Some(64)` |
  | `--hashlock-phrase` dropped from `SECRET_FLAGS` | `hashlock_phrase_on_argv_is_refused_without_the_allow_flag_and_never_echoed` | `left: Some(0), right: Some(1)` — the phrase reached the verb on argv |

  Two of these are the controller defaults the plan flags for the operator
  (`--random` requires `--out`; `--hashlock-phrase -` refused), and both are
  implemented exactly as written, unredesigned.

### Task 8 — `178d0bd` — the other verbs on the new kind

- **RED, matching the plan's Step 2 exactly:** `decode` and `combine` exited 101
  from `unreachable!` (`panicked at crates/ms-cli/src/cmd/decode.rs:107` and
  `crates/ms-cli/src/cmd/combine.rs:166`); `derive_and_verify_refuse_with_the_
  executable_remedy` FAILED with `assertion left != right failed: derive
  panicked`; `inspect_reports_the_kind_with_no_false_reason` FAILED;
  `unreachable_catch_all_count_is_pinned` and
  `secret_flags_doc_comment_counts_five` already PASSED (`3 passed; 7 failed`).
- **PASS:** `cargo test -p ms-cli --test hashlock_other_verbs` →
  `test result: ok. 10 passed; 0 failed; 0 ignored`. The catch-all census still
  reads 4 after the arms were added, as the plan requires.
- **DEVIATION (a test the plan never mentions).**
  `crates/ms-cli/tests/gui_schema_emits_spec_v7_json.rs`'s bijection lock
  `the_schema_names_every_flag_p2_added_and_the_total_is_55` went RED on the
  whole-workspace run:
  ```
  assertion `left == right` failed: 36 before P2, plus --in x8,
  --allow-argv-secret x8 and --out x3. …
    left: 67
   right: 55
  ```
  `cmd/gui_schema.rs` walks `clap::CommandFactory` rather than a hand table, so
  `ms hashlock` entered the schema automatically. **Measured from the binary,
  not inferred:** `ms gui-schema` lists exactly twelve hashlock flags
  (`--hashlock-phrase`, `--hashlock-phrase-stdin`, `--hex`, `--in`, `--random`,
  `--method`, `--out`, `--json`, `--no-engraving-card`, `--group-size`,
  `--separator`, `--allow-argv-secret`) and one positional (`<MS1>`);
  55 + 12 = 67. The pin was updated to 67 and its message extended to name the
  twelve. Nothing else in that test changed.
- **Whole-workspace gate:** `cargo nextest run --workspace --locked
  --no-fail-fast` → `535 tests run: 535 passed, 0 failed, 11 skipped`.

### Task 9 — `51fa1d0` — the CLI test matrix

- **PASS on creation** (these files exercise code the earlier tasks' RED already
  earned): `hashlock_phrase_rule` 7 passed, `hashlock_outputs` 9 passed,
  `hashlock_negative_content` 1 passed (eleven refusal rows inside it).
- **Step 4, the two existing tables** — extended rather than replaced:
  - `exit_codes_table.rs` gains `exit_code_table_verb_usage_is_64`
    (`ms hashlock` with no source → 64), asserting the **message** as well as
    the code, because clap's unrecognized-subcommand error is also 64;
  - `in_flag_six_verbs.rs` gains `in_on_hashlock_equals_the_stdin_run` (the
    equality gate, on the corpus's preimage plate `PREIMAGE_MS1`), `hashlock` in
    both controls (a missing `--in` names the path; `--in` with `-` refuses),
    and a header naming the seventh verb. The file keeps its name, per the plan.
    Probed before writing: the `--in` and stdin runs are byte-identical on
    stdout and stderr at exit 0; `hashlock - --in FILE` → `error: cannot read
    from both --in … and the argument channel`, exit 1; a missing `--in` →
    `error: failed to read --in /nonexistent/nope.txt: No such file or
    directory`. Result: `exit_codes_table` 6 passed, `in_flag_six_verbs` 11
    passed.
- **Five named mutations, each failing its named test, each reverted:**
  | mutation (from the files' own doc comments) | test | evidence |
  | --- | --- | --- |
  | the stdin channel normalised like `read_input` (whitespace, `-`, `,` stripped) | `byte_exact_rows_on_both_channels` | `stdin channel changed the bytes of "  a  b "; left: "hash:a1ff8f18…", right: "hash:5f74bd9f…"` |
  | `--out` suppressing stdout (encode's shape) | `stdout_is_exactly_the_record_under_out_and_under_sha256` | `under --out; left: "", right: "hash:3cf5d421…\n"` |
  | hardened warning threshold 20 → 19 | `hardened_warns_under_20_only` | FAILED at `hashlock_outputs.rs:98` (19 chars stopped warning) |
  | sha256 warning gated on length (`< 30`) | `sha256_warns_at_every_length` | FAILED at `hashlock_outputs.rs:115` (100 chars stopped warning) |
  | `validate_phrase` refusal rebuilt as `format!("{} (phrase was {})")` | `eleven_refusals_never_echo` | FAILED at `hashlock_negative_content.rs:31` |
- **Crate gate:** `cargo nextest run -p ms-cli --locked --no-fail-fast` →
  `362 tests run: 362 passed, 11 skipped`.

### Task 10 — `78ab6a2` — records

- `MIGRATION.md`: the plan's v0.7 → v0.8 section appended verbatim (five
  invariants, the third reader shape, the H0 prerequisite).
- `CHANGELOG.md`: both entries inserted above `## ms-cli [0.17.1]`, verbatim
  except two substitutions the plan leaves to the implementer:
  - the corpus SHA is **filled**, not left as `<sha>`:
    `sha256sum crates/ms-codec/tests/vectors/hashlock-v0.8.json` =
    `a46c197a3640fe8af4ca4370b46a9637466649227163ce6761bb032354811d30`;
  - both `<date>` cells read `unreleased` — Task 11 sets the date and re-checks
    the SHA at the release commit.
- **Step 3, the man page — verified, not assumed, and NO EDIT NEEDED.**
  `cmd/gen_man.rs`'s only clap import is `clap::CommandFactory`, so the pages
  are generated from the derive tree; `ms gen-man --out DIR` emits
  `ms-hashlock.1` beside the other twelve, and
  `cargo test -p ms-cli --test gen_man` → `5 passed`, including
  `exact_page_set_matches_unbuilt_tree` and `one_distinct_page_per_subcommand`.
- **Step 3's other half NOT DONE, filed instead** — see *Not done*.

---

## Deviations from the plan, collected

| # | task | plan said | I did | why |
| --- | --- | --- | --- | --- |
| D1 | 1 | `git add` the three source files and the test only | also staged both `Cargo.toml`s and `Cargo.lock` | the same Step applies both version bumps and the `=0.8.0` pin; leaving them unstaged carries them into an unrelated commit |
| D2 | 2 | `encode.rs` fragment inserts the tag/kind check at the top of `pub fn encode` | moved it **below** the `RESERVED_NOT_EMITTED_V01` check; the hand-wire script's entry moved with it | as written it turned the shipped v0.1 §4 rule 7 error into `TagKindMismatch` for `seed`/`xprv` and failed two shipped tests (`encode_rejects_seed_tag`, `encode_rejects_xprv_tag`) |
| D3 | 3/4 | the corpus is Task 4 Step 1 | created and committed in Task 3 | `include_str!` in `hashlock_derivation.rs` is a compile-time dependency; Task 3 cannot build without it |
| D4 | 5 | RED is "eight fail, three do not" | 15 of 15 failed | the guard cannot fire before `SUBCOMMANDS` learns `hashlock`; clap answers instead, also with exit 64 |
| D5 | 8 | (silent) | updated `gui_schema_emits_spec_v7_json.rs`'s flag-count pin 55 → 67 and its message | the GUI schema is clap-reflective, so the verb's twelve flags entered it automatically; measured from `ms gui-schema` |
| D6 | 10 | CHANGELOG carries `<sha>` / `<date>` | filled the SHA, wrote `unreleased` for the dates | a placeholder SHA in a committed record is worse than a real one; the date belongs to the release commit |
| D7 | 10 | edit the toolkit manual chapter | filed a `cross-repo` FOLLOWUPS entry instead | out of the implementer's scope (never touch another repo) |
| D8 | — | Task 0 creates the two scripts | not re-done | both are already committed at the base `4dbff0b` |

**Plan count discrepancies (records, not defects).** `hashlock_kind.rs` has 14
tests, not "eleven"; `hashlock_derivation.rs` 8, not "six";
`hashlock_phrase.rs` 8 unit tests, not "seven"; `hashlock_sources.rs` 15, not
"eleven".

---

## Final gate — CI's own commands, verbatim tails

`cargo test --workspace --locked` — **exit 0**. Last lines of the run:

```
   Doc-tests ms_codec

running 1 test
test crates/ms-codec/src/lib.rs - (line 16) ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
```

Summed over every `test result:` line in that one captured run (the suite was
not run twice to collect counts): **555 passed, 0 failed, 11 ignored.**

The eight hashlock binaries inside it:

```
tests/hashlock_kind.rs             test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
tests/hashlock_derivation.rs       test result: ok.  8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
tests/hashlock_repro.rs            test result: ok.  2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
tests/hashlock_sources.rs          test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
tests/hashlock_other_verbs.rs      test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
tests/hashlock_outputs.rs          test result: ok.  9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
tests/hashlock_phrase_rule.rs      test result: ok.  7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
tests/hashlock_negative_content.rs test result: ok.  1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
```

`cargo clippy --workspace --all-targets --locked -- -D warnings` — **exit 0**:

```
    Checking hmac v0.12.1
    Checking sha2 v0.10.9
    Checking pbkdf2 v0.12.2
    Checking ms-codec v0.8.0 (/scratch/code/shibboleth/ms-worktrees/hashlock-h1/crates/ms-codec)
    Checking ms-cli v0.18.0 (/scratch/code/shibboleth/ms-worktrees/hashlock-h1/crates/ms-cli)
    Finished `dev` profile [optimized + debuginfo] target(s) in 2.32s
```

`cargo fmt --all -- --check` — **exit 0, no output**, on the repo-pinned
1.85.0. CI's fmt job uses a different, pinned formatter, so that was run too:
`cargo +1.95.0 fmt --all -- --check` — **exit 0, no output**.

`cargo nextest run --workspace --locked --no-fail-fast` (the parallel runner)
— `535 tests run: 535 passed, 0 failed, 11 skipped`. (nextest counts test
binaries' cases only; `cargo test`'s 555 additionally counts doc-tests and the
`--bin ms` unit tests it reports separately.)

---

## Spec §12 acceptance, items 1–5, run against the built binary

1. `ms hashlock --hashlock-phrase-stdin < phrase.txt` →
   `hash:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12`;
   the card's first stderr line is
   `THIS CARD CARRIES THE PREIMAGE -- the secret. stdout carries only the public digest.` ✔
2. `--method sha256` →
   `hash:b867db875479bcc0287352cdaa4a1755689b8338777d0915e9acd9f6edbc96cb`,
   and the brainwallet line is on stderr. ✔
3. `--out X.txt` wrote
   `ms10hashsq0p7jaf9gsjjpkjvll2l274w8a388xgqzlewp73scptwxgtjugspvs8tklufg89hqj`
   (75 chars + LF = 76 bytes) at mode `600`; `ms hashlock --in X.txt`
   re-derived the same digest. ✔
4. `ms decode --in X.txt` → three lines, `kind: preimage (hashlock, 32 bytes /
   64 hex characters)`, `preimage: c3e97525…`, `digest: 3cf5d421…`, never words,
   exit 0; `ms inspect` → `OK: would decode v0.8`, `tag: hash`,
   `prefix_byte: 0x03`, exit 0, no false reason; `ms derive` and `ms verify`
   both →
   `error: this is a hashlock preimage plate, not a seed backup; use
   'ms hashlock <ms1>' (or 'ms hashlock --in FILE') to re-derive its digest`,
   exit 1; nothing exited 101. ✔
5. `--random` with no `--out` → exit 64 naming `--out`; with `--json` and no
   `--out` → exit 64 (the JSON error envelope carries `"kind":"Usage"`,
   `"exit_code":64`); with `--out FILE` → exit 0; a second `--random --out` onto
   the same path → exit 64 naming the file, and the file's bytes were
   **unchanged**. ✔

Items 6 (`| me sysw pack`), 7 (the flashed device / `me` inertness) and 8 (the
release) are cross-repo, device or release items and were not run — see below.

---

## Not done, and why

- **Task 11 (Release).** Not executed. Its Step 1 is the **H0 gate**: the fork's
  `isStrictMs1`/`seal.Classify` guard merged *and flashed*, and `me`'s
  `validate_record` guard in the same window as its ms-codec 0.8 bump. Neither
  has shipped, and both live in other repositories. Its Steps 2–3 are the
  staging-push ritual, `cargo publish --dry-run`, two annotated tags, a push and
  a GitHub release check — all of which the brief forbids (never push). The
  version bumps and pin the task asks to *confirm* are in place and were
  machine-checked: `ms-codec 0.8.0`, `ms-cli 0.18.0`, `ms-codec = { path =
  "../ms-codec", version = "=0.8.0" }`, and `Cargo.lock` updated to match.
- **The toolkit manual chapter** (`mnemonic-toolkit/docs/manual/src/
  40-cli-reference/43-ms.md`, plan Task 10 Step 3): cross-repo, out of scope.
  Filed as `design/FOLLOWUPS.md::toolkit-manual-ms-hashlock-chapter`, tier
  `cross-repo`, owning phase the 0.18.0 release, carrying the twelve flags
  measured from the binary and the `make -C docs/manual lint` command.
- **`scripts/plan-build-gate-ms.sh` was not run** — the brief forbids it. It was
  used only as documentation of the fragments' provenance. Note that its step 6
  (the downgrade row against the pre-H1 tree) is therefore **not** re-proven by
  this work; the corpus's `downgrade` object records what it asserts, and the
  plan's STATUS records that the gate ran green at plan time.
- **Acceptance items 6, 7 and 8** — `me sysw pack` (a different repo's binary),
  device inertness (H0, hardware) and the release — not run.

---

## Observations the plan did not ask for

- **Secret handling (never gating, per the 2026-08-27 ruling; recorded for
  future optimisation).** `ms inspect` on a preimage plate prints
  `payload_bytes: <the preimage, in the clear>` to stdout and emits **no**
  output-class advisory. This is pre-existing behaviour extended to the new kind
  — `ms inspect` on an entr single prints its entropy the same way and is
  likewise silent (checked as a control). `ms decode` on a preimage *does* fire
  the advisory (`warning: stdout carries private key material (can spend) …`),
  so the two verbs disagree. Not a regression this cycle; worth a follow-up.
- `argv_candidates` still pre-folds (trims and lowercases) before calling
  `material_class`, which now calls `looks_like_ms1` — a predicate that
  normalises internally. Spec §4.3 says "`argv_candidates` stops pre-folding and
  calls the same function". The pre-fold is now redundant but harmless (folding
  an already-folded token is idempotent) and no test can see it; the plan's
  fragment does not touch it, so neither did I.
- `cargo fmt` reflowed `SECRET_FLAGS` into a multi-line array. Task 8's
  `secret_flags_doc_comment_counts_five` asserts the substring
  `const SECRET_FLAGS: [&str; 5]`, which survives the reflow — checked, it
  passes.
- The plan's Task 5 Files note and Task 8's "Consumes" line both attribute the
  three `From<ms_codec::Error>` arms to Task 5 Step 3; they are in the same
  `edit("crates/ms-cli/src/error.rs", …)` entry as `CliError::Usage` and were
  applied whole there, so Task 8's
  `tag_kind_mismatch_is_a_format_violation_on_decode_and_a_reason_on_inspect`
  had them available as designed.

---

## Commits on `hashlock-h1` (oldest first)

| SHA | task |
| --- | --- |
| `bd76cec` | Task 1 — constants, tag, the three errors, blocklist, both version bumps |
| `ff5da74` | Task 2 — `Payload::Preimage`, dispatch, accept set, tag/kind checks |
| `571d88c` | Task 3 — `ms_codec::hashlock` (+ the corpus, D3) |
| `dac6dbb` | Task 4 — the three-way reproduction test, CI preflight and run-by-name |
| `ac9b917` | Task 5 — the argv guard, `CliError::Usage`, the three `From` arms |
| `4348312` | Task 6 — the byte-verbatim reader and the phrase rule |
| `3ec623e` | Task 7 — `ms hashlock` |
| `178d0bd` | Task 8 — the other verbs on the new kind |
| `51fa1d0` | Task 9 — the CLI test matrix |
| `78ab6a2` | Task 10 — MIGRATION, CHANGELOG, the cross-repo follow-up |
| *this commit* | the report |
