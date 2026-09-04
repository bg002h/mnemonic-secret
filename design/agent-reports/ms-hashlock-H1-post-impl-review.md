# H1 post-implementation review — `ms hashlock`, whole diff `4dbff0b..a150ba7`

**Reviewer:** independent adversarial execution reviewer (opus), no sub-agents.
**Brief:** `design/agent-briefs/ms-hashlock-H1-post-impl-brief.md`.
**Under review:** branch `hashlock-h1`, tip `a150ba7`, base `4dbff0b`
(46 files, +3668 / −43).
**Method:** every finding below was produced by RUNNING the shipped binary or by
mutating the shipped code in a throwaway detached worktree
(`/scratch/code/shibboleth/ms-worktrees/h1-review` at `a150ba7`, since removed).
Every mutation was reverted and the worktree left clean; nothing was committed
anywhere; no file in `mnemonic-secret` or in `ms-worktrees/hashlock-h1` was
modified. All cargo commands ran with
`PATH=$HOME/.cargo/bin:$PATH TMPDIR=/scratch/code/shibboleth/.tmp
CARGO_TARGET_DIR=/scratch/code/shibboleth/mnemonic-secret/target`.

**Baseline reproduced before any mutation:** `cargo nextest run --workspace
--locked --no-fail-fast` → `554 tests run: 554 passed, 11 skipped`, exit 0.

**Verdict: NOT GREEN — 2 Critical, 3 Important, 6 Minor, 2 Nit.**

The codec and the verb are, on everything I could construct, correct: I could not
make `ms` give a wrong preimage, print a preimage on a channel the spec calls
public, accept an input the spec refuses, or misread one kind as another. The two
Criticals are (1) a spec-normative guarantee whose only test cannot fail, and (2)
a false measurement in the implementation report. Both are cheap to close.

---

## Critical

### C-1 — the `--json` `PrivateKeyMaterial` advisory has no test that can fail; deleting the advisory leaves 554/554 green

**Spec line violated.** §4.4, `--json`: *"It carries the secret, so the
`PrivateKeyMaterial` advisory fires, as `encode --json` does. (R0 r0 correctness
I-3, I-4.)"* and §11 Outputs: *"`--json`'s schema in **both** variants … **and
its advisory**"*.

The only assertion in the tree that names the advisory is
`crates/ms-cli/tests/hashlock_outputs.rs:173-178`, inside `json_both_variants`:

```rust
assert!(
    String::from_utf8_lossy(&out.stderr).contains("private key material")
        || String::from_utf8_lossy(&out.stderr)
            .to_ascii_lowercase()
            .contains("secret")
);
```

That invocation does **not** pass `--no-engraving-card`, so stderr always carries
the card, whose first line is

```
THIS CARD CARRIES THE PREIMAGE -- the secret. stdout carries only the public digest.
```

Lowercased, that line contains `secret`. The second disjunct is therefore
satisfied by the card alone, independent of the advisory, and the assertion
cannot fail while the card's shipped wording stands.

**Constructed counterexample (mutation).** Delete the advisory from
`crates/ms-cli/src/cmd/hashlock.rs:368-370`:

```rust
    if args.json {
        emit_output_class_advisory(OutputClass::PrivateKeyMaterial, &mut stderr);
    }
```
→
```rust
    let _ = &mut stderr;
```

(and drop the now-unused `use crate::advisory::{…}`), then:

```
$ cargo nextest run --workspace --locked --no-fail-fast
     Summary [   0.325s] 554 tests run: 554 passed, 11 skipped
```

**554/554 pass with the guarantee gone.** The mutant's behaviour, verbatim:

```
$ ms hashlock --hashlock-phrase-stdin --json --no-engraving-card < anchor.txt
{"digest":"3cf5d421…","hash_record":"hash:3cf5d421…","method":{…},"phrase_chars":28,
 "preimage_hex":"c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016",
 "preimage_ms1":"ms10hashsq0p7jaf9gsjjpkjvll2l274w8a388xgqzlewp73scptwxgtjugspvs8tklufg89hqj",
 "sha256_operand":"sha256=3cf5d421…","source":"phrase (stdin)"}
                     ← stderr: nothing at all
```

The preimage goes to stdout and no advisory warns. Restored, the shipped code
does fire the advisory — the defect is entirely in the gate, which is exactly
the class the brief rates Critical (*"a test that cannot fail on a normative
guarantee"*).

**Note on scope.** `crates/ms-cli/tests/cli_output_class.rs` (12 tests) covers
the advisory for other verbs, but none of its rows is `hashlock` — I ran the
whole workspace under the mutation, not just the hashlock binaries, and nothing
anywhere caught it.

---

### C-2 — the implementation report's final-gate nextest count is false (535 claimed, 554 measured), and the sentence that explains the gap is also false

**Where.** `design/agent-reports/ms-hashlock-H1-implementation-report.md`,
"Final gate — CI's own commands, verbatim tails":

> `cargo nextest run --workspace --locked --no-fail-fast` (the parallel runner)
> — `535 tests run: 535 passed, 0 failed, 11 skipped`. (nextest counts test
> binaries' cases only; `cargo test`'s 555 additionally counts doc-tests and the
> `--bin ms` unit tests it reports separately.)

**Measured at the branch tip `a150ba7`:**

```
$ cargo nextest run --workspace --locked --no-fail-fast
     Summary [   0.307s] 554 tests run: 554 passed, 11 skipped
```

**Where 535 comes from.** It is Task 8's number, correctly reported earlier in
the same document, carried into the final-gate section unchanged. Checked out
and re-run:

```
$ git checkout -q 178d0bd && cargo nextest run --workspace --locked --no-fail-fast
     Summary [   0.238s] 535 tests run: 535 passed, 11 skipped
```

Task 9 added 19 cases (`hashlock_phrase_rule` 7, `hashlock_outputs` 9,
`hashlock_negative_content` 1, `exit_codes_table` +1, `in_flag_six_verbs` +1);
535 + 19 = 554.

**The explanation is false too, and it is the load-bearing half.** nextest DOES
run the `--bin ms` unit tests — 72 of them in the captured run:

```
$ grep -c "ms-cli::bin/ms" suite.log
72
```

so the real `cargo test` 555 vs nextest 554 gap is exactly the one doc-test
(`crates/ms-codec/src/lib.rs - (line 16)`), not "doc-tests and the `--bin ms`
unit tests". As written, the parenthetical makes a 20-case discrepancy look
routine — the precise shape that would have absorbed a real regression of 19
dropped tests without anyone noticing.

The rest of the report's measurable claims that I checked all hold (see the
deviation table and "report claims verified" below); this is an isolated stale
copy, and the remedy is a two-line correction to the report. The code is
unaffected. Rated Critical because the brief names *"a false claim in the
implementation report"* as Critical.

---

## Important

### I-1 — the terminal prompt tells the operator to press Enter, and Enter does not end the read; the tool still hangs

**Spec line.** §4.3: *"With stdin at a terminal, `--hashlock-phrase-stdin`
prints one prompt line to stderr — `Type the hashlock phrase, then Enter.` —
rather than blocking silently (r2 review M-7; the constellation's recorded `mt`
finding, where a tool's first interaction looked like a hang)."*

`read_phrase_stdin` (`crates/ms-cli/src/hashlock_phrase.rs:80-91`) is built on
`read_to_end`, which returns only at **EOF**. On a terminal in canonical mode,
Enter delivers a line; it does not deliver EOF. So the prompt instructs the one
action that cannot finish the read.

**Constructed counterexample** (real pty via `pty.fork()`, phrase written after
the prompt, output captured verbatim):

```
########## MODE: enter only ##########
AFTER-SPAWN(1s): b'Type the hashlock phrase, then Enter.\r\n'
AFTER-ENTER(2s): b'correct horse battery staple\r\n'          ← terminal echo only; ms produced nothing
                                                                and was still blocked when killed

########## MODE: enter then Ctrl-D ##########
AFTER-SPAWN(1s): b'Type the hashlock phrase, then Enter.\r\n'
AFTER-ENTER(2s): b'correct horse battery staple\r\n'
AFTER-CTRL-D(3s): b'hash:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12\r\n'
```

Only Ctrl-D completes it. The derived value is correct once it does, so this is
not a wrong result — it is the anti-hang mitigation failing at the exact moment
it exists for. The operator's next reasonable action after Enter is Ctrl-C.

**Control, to show what is new.** The sibling stdin channel blocks *silently*
(pre-existing):

```
$ ms derive --in seed32.txt --passphrase-stdin      # at a real pty
PROMPT?: b''                                        # nothing printed, blocked
```

and `grep -rn "then Enter|Type the" crates/ms-cli/src` returns only
`hashlock_phrase.rs:76` and its unit test — this is the CLI's only prompt. The
prompt's wording is prescribed verbatim by the spec, so the defect is inherited;
the whole-diff review is where it can still be caught before the operator meets
it. `prompt_only_at_a_terminal` (the one test) asserts the string is written and
cannot see that the read never ends.

---

### I-2 — `ms hashlock --separator` has no value parser; `ms encode` and `ms split` both refuse the same input, and the card can carry an unengravable "plate string"

**Site.** `crates/ms-cli/src/cmd/hashlock.rs:78-80`:

```rust
    /// Group separator on the card.
    #[arg(long, default_value_t = ' ')]
    pub separator: char,
```

against `crates/ms-cli/src/cmd/encode.rs:89` and `crates/ms-cli/src/cmd/split.rs:89`,
which are both:

```rust
    #[arg(long, default_value = "space", value_parser = crate::format::parse_separator)]
```

`parse_separator`'s own doc says *"One `parse_separator` serves both `ms encode`
and `ms split`, so it cannot bind to one of them."* `ms hashlock` is now a third
emitter of a grouped ms1 that is not bound to it.

**Constructed counterexample.**

```
$ ms hashlock --in X.txt --separator q 2>&1 >/dev/null | grep 'preimage (ms1)'
preimage (ms1):  ms10hqashsqq0p7jaqf9gsjqjpkjvqll2l2q74w8aq388xgqqzlewqp73scqptwxgqtjugsqpvs8tqklufgq89hqj
```

`q` is in the codex32 charset, so `strip_display_separators` will not remove it
on read-back: a plate cut from that card is a 90-character string `ms` refuses.
The controls, same binary, same flag value:

```
$ ms encode --hex 000102…0f --separator q --allow-argv-secret
error: invalid value 'q' for '--separator <SEPARATOR>': invalid separator "q"; expected `space` (or the literal " ")

$ ms encode --hex 000102…0f --separator - --allow-argv-secret
error: invalid value '-' for '--separator <SEPARATOR>': separator "-" is no longer offered: `ms` emits whitespace grouping only. …
```

while `ms hashlock --in X.txt --separator -` happily emits a hyphen-grouped card.

**Provenance.** This is faithful to the plan — `IMPLEMENTATION_PLAN_ms_hashlock_H1.md:2271-2273`
carries the same `#[arg(long, default_value_t = ' ')]` — so it is a plan
omission the implementer reproduced, not an implementer deviation. Spec §4.4
says only *"(`--group-size`/`--separator` apply)"*. No test covers it.
`--group-size 0` and `--group-size 255` are both safe (`render_grouped` returns
the input unchanged at 0; measured).

---

### I-3 — under `--allow-argv-secret`, `--hashlock-phrase` with a missing value swallows the next flag and derives a preimage from it at exit 0

`ms hashlock` is the first verb where the argv guard's flag-swallow produces a
**success**, because §4.3's phrase rule admits every printable-ASCII string ≤ 100
characters, so nothing downstream can reject a swallowed flag name.

**Constructed counterexample.**

```
$ ms hashlock --allow-argv-secret --hashlock-phrase --json --no-engraving-card < /dev/null
hash:329367945b164ccb91c6b124ab903227e34f468e9f82c5806b1ca4a194d4c613
$ echo "exit=$?"
exit=0

$ python3 -c "import hashlib; x=hashlib.pbkdf2_hmac('sha256',b'--json',b'ms-hashlock-v1',100000,32); print(hashlib.sha256(x).hexdigest())"
329367945b164ccb91c6b124ab903227e34f468e9f82c5806b1ca4a194d4c613
```

The operator asked for JSON, got no JSON, and got a preimage derived from the
six-byte string `--json`, at exit 0. `substitute`
(`crates/ms-cli/src/argv_guard.rs:290-310`) consumed `--json` as the flag's
value and rewrote argv to `--hashlock-phrase -`.

**Controls — the mechanism is pre-existing, the successful outcome is not.**
Same shape on the two flags that already lived in `SECRET_FLAGS`:

```
$ ms encode --allow-argv-secret --hex --json --no-engraving-card
error: invalid character 'j' at position 0                       exit=1

$ ms encode --allow-argv-secret --phrase --json --no-engraving-card
error: BIP-39 word count 1 invalid (must be 12, 15, 18, 21, or 24)  exit=1
```

and without the guard in the way, clap refuses outright:

```
$ ms hashlock --method --json
error: a value is required for '--method <METHOD>' but none was supplied   exit=64
```

**Mitigation that exists.** With the card (the default), stderr shows
`phrase:          6 characters …` — §4.4's character count, added by review M-2
precisely as *"the one signal that makes a stray space visible"*. The failure
therefore needs `--allow-argv-secret` **and** an omitted value **and**
`--no-engraving-card` to be silent; with the card it is visible but not refused.
No test covers a swallowed flag on this verb.

---

## Minor

### M-1 — `ms inspect` prints the preimage on stdout with no output-class advisory, while `ms decode` fires one

```
$ ms inspect --in X.txt
OK: would decode v0.8
…
payload_bytes: c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016
kind: preimage
                       ← stderr: nothing

$ ms decode --in X.txt
kind:      preimage (hashlock, 32 bytes / 64 hex characters)
preimage:  c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016
digest:    3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
warning: stdout carries private key material (can spend) — redirect or encrypt …
```

Pre-existing shape extended to the new kind (`ms inspect` on an entr single is
silent the same way — checked as a control), and the implementer already
recorded it. Secret-handling class, never gates (operator ruling 2026-08-27).

### M-2 — the terminal echoes the phrase during `--hashlock-phrase-stdin`

The pty transcript in I-1 shows `correct horse battery staple` echoed back by
the terminal; the reader does not disable ECHO. Secret-handling class, never
gates. Recorded with its reproduction as the ruling requires.

### M-3 — three test names/doc comments over-promise what their bodies do

- `crates/ms-cli/tests/gui_schema_emits_spec_v7_json.rs:280`
  `fn the_schema_names_every_flag_p2_added_and_the_total_is_55()` now asserts
  `total == 67`. The name is the grep handle; it is now wrong.
- `crates/ms-cli/tests/hashlock_phrase_rule.rs:211` `fn lockstep_100_and_101()`
  exercises only the 100-character row; the 101 refusal is covered, but by
  `printable_ascii_boundary_and_cap`.
- `crates/ms-cli/tests/hashlock_sources.rs:444` doc comment *"`--out` is 0600
  (owner-only) on every source"* — the body exercises the phrase source only, so
  `write_artifact_create_new`'s mode is untested. (I measured it separately:
  `--random --out rnd.txt` → `600`, and `OpenOptions::mode(0o600)` can only be
  narrowed by umask, never widened, so the behaviour is right.)

### M-4 — stale line citation in the new CI step

`.github/workflows/rust.yml:128`: `# The job runs cargo test, not nextest
(rust.yml:118-119, measured).` After the preflight step the step it cites is at
`rust.yml:124-125`. The claim itself is true.

### M-5 — `write_artifact`'s doc comment was orphaned onto the new function

In `crates/ms-cli/src/out.rs`, the new `write_artifact_create_new` was inserted
between `write_artifact`'s doc comment and `write_artifact` itself, so the
paragraph *"The refusal names the PATH and never the artifact…"* now documents
`create_new` and `write_artifact` (line 61) has no doc comment at all.

### M-6 — two tests write to fixed paths in the shared `/tmp`

`hashlock_sources.rs:167` (`/tmp/ms-hashlock-pair-test.txt`) and `:204`
(`/tmp/ms-hashlock-method-test.txt`), while every other test in the file uses
`tempfile::tempdir()`. Neither file is actually created (both invocations exit
64 before the write), so this is latent rather than flaky today.

---

## Nit

### N-1 — `--group-size` is `u8` on `hashlock`, `u16` on `encode`/`split`

`hashlock.rs:76-77` vs `encode.rs:86`. `--group-size 256` is a clap parse error
on `hashlock` and accepted on `encode`. No behavioural consequence.

### N-2 — no 76-character `0x03` string exists (brief item 1)

String length is `22 + ceil(8N/5)`, so `N=33 → 75` and `N=34 → 77`; 76 is
unreachable for any payload. The 74-character case (`N=32`) does exist and is
refused by the string-length gate before prefix dispatch, as the spec says:

```
$ ms decode --in f.txt      # ms10hashsq…w4swv59qxvwra44g, 74 chars
error: string length 74 not in v0.1 set [50, 56, 62, 69, 75]     exit=1
```

---

## The eight deviations — verdict each

| # | claim | verdict | evidence |
| --- | --- | --- | --- |
| D1 | Task 1 also staged both `Cargo.toml`s and `Cargo.lock` | **TRUE, correct** | `git show --stat bd76cec` lists `Cargo.lock`, `crates/ms-cli/Cargo.toml`, `crates/ms-codec/Cargo.toml` alongside the three source files and the test |
| D2 | the tag/kind check moved BELOW `RESERVED_NOT_EMITTED_V01`; the hand-wire script moved with it | **TRUE, correct, and the script matches** | `encode.rs:23-31` sits after the reserved check; `scripts/plan-handwire-ms-hashlock.py:95-100` now anchors on `"    // §3.5: payload length validation."` and its replacement text is byte-identical to the shipped block. Both directions of §1 rule 2 still refuse — mutation M4b (delete the check) kills `id_and_prefix_must_agree_both_directions`. `seed`/`xprv` keep `ReservedTagNotEmittedInV01` (measured: a forged `seed`-over-`0x03` single exits 3 on `decode`, matching the pre-existing `seed`-over-`0x00` control) |
| D3 | the corpus moved to Task 3 because `include_str!` is a compile-time dependency | **TRUE — and I measured both halves** | file removed → `error: couldn't read … include_str!` → *build* fails; one `hardened_x` replaced by `<sha>` → *test* fails: `"  a  b ": hardened_x is not 64 lowercase hex (a placeholder left in the corpus?): "<sha>"`. So a placeholder is caught by a test and a missing file by the build, exactly as claimed |
| D4 | RED was 15/15, not "eight fail, three do not" | **PLAUSIBLE, not re-derived** | the mechanism is right (`SUBCOMMANDS` has no `"hashlock"` before Task 5, so clap answers first, also with 64). Verified today that clap's unknown-flag path is 64: `ms hashlock --hashlock-phras X` → `error: unexpected argument '--hashlock-phras' found`, exit 64. Re-running Task 5's RED would require reverting a committed task; the claim is internally consistent and its consequence (assert the MESSAGE, not just the code) is honoured in `exit_codes_table.rs` |
| D5 | GUI schema pin 55 → 67; hashlock contributes twelve flags | **TRUE, measured from the binary** | `ms gui-schema \| python3` → per-subcommand `derive 11, encode 10, decode 4, hashlock 12, inspect 3, verify 5, vectors 1, gen-man 1, repair 5, split 11, combine 4` = **67**; hashlock's twelve are exactly the twelve named, `<MS1>` is a positional. (The test's *name* still says 55 — M-3.) |
| D6 | corpus SHA filled, dates left `unreleased` | **TRUE** | `sha256sum crates/ms-codec/tests/vectors/hashlock-v0.8.json` = `a46c197a3640fe8af4ca4370b46a9637466649227163ce6761bb032354811d30`, byte-identical to `CHANGELOG.md:36`; both headings read `— unreleased` |
| D7 | toolkit manual filed as a follow-up instead of edited | **TRUE, with an owning phase** | `design/FOLLOWUPS.md` gains `toolkit-manual-ms-hashlock-chapter`, *owning phase: **H1 Task 10 / the 0.18.0 release, Task 11***, tier `cross-repo`, carrying the twelve flags and `make -C docs/manual lint`. Its man-page claim also holds: `ms gen-man --out DIR` emits 13 pages including `ms-hashlock.1` |
| D8 | Task 0's two scripts already exist at the base | **TRUE** | at `4dbff0b`: `scripts/plan-build-gate-ms.sh` (142 lines) and `scripts/plan-handwire-ms-hashlock.py` (235 lines) |

**Plan count discrepancies** (reported as records, not defects) — all four
confirmed by running the suite: `hashlock_kind` 14, `hashlock_derivation` 8,
`hashlock_phrase.rs` 8 unit tests, `hashlock_sources` 15.

---

## Mutation table

Each mutation was applied in the review worktree, the named test binary run, and
the mutation reverted. "KILLED" = the guard failed as its doc comment promises.

| # | mutation | target guard | result |
| --- | --- | --- | --- |
| M1a | delete `Payload::Preimage(x) => return emit_preimage(…)` from `cmd/decode.rs:106` | never-words | **KILLED** — `decode_prints_kind_hex_and_digest_and_never_words` and `decode_json_carries_kind_and_digest` both FAIL (`10 tests run: 8 passed, 2 failed`) |
| M1b | `emit_preimage` prints a 4th line (`phrase: abandon abandon abandon`) | never-words | **KILLED** — `decode's text output for a preimage is exactly three lines` |
| M2 | add a 5th `_ => unreachable!` in `cmd/split.rs` | catch-all census | **KILLED** — `assertion left == right failed … left: 5, right: 4` |
| M3a | one corpus `hardened_x` → `"<sha>"` | corpus loader | **KILLED** — `corpus_rows_are_filled_and_re_derive`: `hardened_x is not 64 lowercase hex (a placeholder left in the corpus?)` |
| M3b | delete `tests/vectors/hashlock-v0.8.json` | corpus loader | **KILLED at BUILD** — `error: couldn't read …; could not compile ms-codec (test "hashlock_derivation")` |
| M4a | delete decode's rule 6b arm (`decode.rs:91-96`) | tag/kind mismatch | **KILLED** — and the mutant does exactly the misread §1 rule 2 forbids: `unwrap_err() on an Ok value: (Tag([104,97,115,104]), Entr([171 × 32]))` — a `hash` single read as entropy |
| M4b | delete encode's emit-side rule 2 (`encode.rs:26-31`) | tag/kind mismatch | **KILLED** — `id_and_prefix_must_agree_both_directions` |
| M4c | delete the `TagKindMismatch` arm from `From<ms_codec::Error>` | CLI error mapping | **KILLED** — `error: unhandled ms_codec::Error variant …` exit 1, expected 2 |
| M5 | `write_artifact_create_new` → `write_artifact` for `--random` | create_new refusal | **KILLED** — `random_out_refuses_to_overwrite_but_other_sources_overwrite`, `left: Some(0), right: Some(64)` |
| M6 | `looks_like_ms1` drops the trim+lowercase normalisation | ms1-shape predicate | **KILLED** — `refusals_in_four_spellings_on_both_channels_name_the_ms1_route` (`left: Some(0), right: Some(1)`) and the unit test `ms1_shape_in_four_spellings_and_before_the_cap` |
| **M7** | **delete the `PrivateKeyMaterial` advisory from `hashlock --json`** | **`json_both_variants`** | **SURVIVED — 554/554 workspace tests pass. See C-1.** |

---

## What I ran and could NOT break (brief items 1–8)

**1. The wire and the readers (§1, §5).** A preimage single round-trips
`hashlock --out` → `decode` / `inspect` / `combine` / `hashlock --in`, at 75
characters, mode `600`:
`ms10hashsq0p7jaf9gsjjpkjvll2l274w8a388xgqzlewp73scptwxgtjugspvs8tklufg89hqj`.
Forged strings built through `Codex32String::from_seed` and fed to
`decode`/`inspect`/`hashlock`/`verify` by `--in`:

| forged string | decode | inspect reasons |
| --- | --- | --- |
| id `hash` over a `0x00` payload | exit 2, `the id "hash" names a different kind than the prefix byte 0x00 carries; refusing rather than reading one kind as another` | `["tag-kind-mismatch"]` |
| id `entr` over a `0x03` payload | exit 2, same wording with `0x03` | `["tag-kind-mismatch"]` |
| 77 chars / 34-byte payload | exit 2, `preimage payload is 33 bytes after the prefix; a hashlock preimage is exactly 32 bytes (64 hex characters)` | `["unexpected-string-length","payload-length-mismatch"]` |
| 70 chars / 30-byte payload | exit 2, `… is 29 bytes after the prefix …` | same |
| 74 chars / 32-byte payload | exit 1, `string length 74 not in v0.1 set [50, 56, 62, 69, 75]` | same |
| id `seed` over `0x03` | exit 3 (`ReservedTagNotEmittedInV01`; control with `0x00` behaves identically — pre-existing) | `["reserved-tag-not-emitted"]` |
| id `mnem` / `zzzz` over `0x03` | exit 2, `unknown tag "mnem"` / `"zzzz"` | `["unknown-tag"]` |

A valid preimage single gives `OK: would decode v0.8`, `tag: hash`,
`prefix_byte: 0x03`, `kind: preimage`, **`failure_reasons: []`** — no false
reason. UPPERCASE, space-grouped and hyphen-grouped forms of the plate all
re-derive the same digest; mixed case is refused. `derive`/`verify` refuse with
the executable remedy on both the text and `--json` paths; nothing exited 101
anywhere. `combine` on a real 2-of-3 preimage share set prints kind/hex/digest
and the digest matches `python3 hashlib.sha256(b'\x5a'*32)`. One share alone is
refused (`this is one share of a K-of-N set …`). `ms split --in <plate>` reads
`--in` as a PHRASE (pre-existing F-468) and refuses. `ms encode --hex` still
emits `entr`, so `ms hashlock` remains the only CLI door to the kind. The share
id blocklist mechanism (`shares.rs:40-55`) rejects a drawn `hash`.

**2. Derivation (§2, §8).** Three corpus rows re-derived from the SHIPPED binary
and cross-checked outside the crate:

```
$ openssl kdf -keylen 32 -kdfopt digest:SHA256 -kdfopt pass:'correct horse battery staple' \
      -kdfopt salt:ms-hashlock-v1 -kdfopt iter:100000 PBKDF2
C3:E9:75:25:44:25:20:DA:4C:FF:D5:F5:7A:AE:3F:62:73:99:00:17:F2:E0:FA:30:C0:56:E3:21:72:E2:20:16
$ openssl kdf … -kdfopt pass:'  a  b ' …
CA:E9:F5:66:33:50:A8:64:62:A1:94:01:55:16:65:58:46:BC:68:80:F1:34:E1:56:22:7E:58:23:23:E0:14:6B
```

matching `hashlock_derivation.rs` rows 1 and 8, and the binary's own
`hash:07ca621d…` / `hash:8680bbf9…` / `hash:36d5ad9d…` for `"  a  b "`,
`"a-b,c"` and `"Correct Horse Battery Staple"` equal python's. The reader is
byte-verbatim: one LF stripped, CRLF stripped as one, **two** LFs keep one and
are then refused as non-printable (`byte 0x0a at position 28`); leading and
trailing spaces, `-` and `,` and case all change X. 64-hex in both cases refused
naming `--hex`; 63 hex and `beef` accepted; 100 accepted, 101 refused; empty,
TAB, DEL, `é` and a raw `0xFF` each refused by name; `" ~"` accepted. ms1-shaped
refused in all five spellings (lower, UPPER, grouped-5, space-padded,
grouped-by-2 at 112 characters) on **both** channels, and the 112-character one
names `--in` rather than the cap — the shape test does precede it.

**3. The argv guard (§6).** `--hashlock-phrase VALUE` and
`--hashlock-phrase=VALUE` are both refused before clap, exit 1, with
`is a hashlock phrase, N characters long` (not "BIP-39 passphrase"), and the
value never appears on stdout or stderr. Abbreviations do not reach the guard
and are not admitted either: `--hashlock-phras`, `--hashlock-phr=`,
`--hashlock` → `error: unexpected argument … found`, exit 64, value not echoed.
`--hex` on argv and the positional ms1 are both refused. `--hashlock-phrase -`
is refused naming `--hashlock-phrase-stdin`, with and without the allow flag.
**All three `/dev/null` gates hold** — with `--allow-argv-secret` and stdin at
`/dev/null`, `--hashlock-phrase`, `--hashlock-phrase=`, `--hex`, `--hex=` and
the positional each derive `hash:3cf5d421…`; with a *different* phrase on stdin
the argv value still wins. Through the admitted side channel every phrase-rule
refusal is reached (64-hex, ms1-shaped lower and UPPER, 101, empty, TAB).
`ms hashlock --help` prints all twelve flags, says `Requires --out FILE` on
`--random`, and spells the size as *"exactly 32 bytes (64 hex characters)"* (L8).

**4. `--random` (§4.1).** No `--out` → exit 64 naming `--out`; `--json` alone
does not satisfy it (exit 64, `{"kind":"Usage","exit_code":64}` on stdout);
`--out FILE` → exit 0, mode `600`; a second run onto the same path → exit 64
naming the file, **bytes unchanged** (`diff` clean); `--out /dev/null` and
`--out <dir>` → exit 64 (`already exists`); `--out <read-only dir>/p` → exit 1
`Permission denied`; `--out <missing dir>/p` → exit 1. In every failing case
stdout is empty and the exit code is non-zero — **no path prints exit 0 after
losing the only copy**. Two runs give different records and different files.

**5. Outputs (§4.4, §7).** stdout is exactly `hash:<64 lowercase hex>` under
`--out` and under `--method sha256`; the card's first line is
`THIS CARD CARRIES THE PREIMAGE -- the secret. …`; the method line is verbatim
and copyable; `phrase: 28 characters` appears for phrase sources and is absent
for `--hex`/`--random`/ms1, where the method line reads `preimage supplied`.
Warnings are method-keyed, not phrase-keyed: hardened warns at 19 and not at 20
or 100; sha256 warns at 19, 20 and 100. `--hex` gets its unconditional line;
`--random` gets both halves and names the FILE. `--json` carries exactly
`digest, hash_record, sha256_operand, preimage_hex, preimage_ms1, source`
(+ `method`, `phrase_chars` for phrase sources only), all hex lowercase.
Source arithmetic: zero sources → 64 listing five; **all ten two-source pairs →
64** (three of them need `--allow-argv-secret` to get past the guard first, which
is the guard doing its job); both stdin-contention pairs → 64.

**6. The deviations.** See the table above — all eight verified.

**8. Rust-primary.** `git diff --name-only 4dbff0b..a150ba7` touches nothing
outside `mnemonic-secret`. `ms-codec 0.8.0`, `ms-cli 0.18.0`,
`ms-codec = { path = "../ms-codec", version = "=0.8.0" }`. Deps are spelled as
the spec dictates: `pbkdf2 = { version = "0.12", default-features = false,
features = ["hmac"] }` and `sha2 = "0.10"`; `Cargo.lock`'s `ms-codec` block has
no direct `hmac` and there is no `password-hash` entry at all. Constants:
`HASHLOCK_SALT = b"ms-hashlock-v1"`, `HASHLOCK_ITERATIONS = 100_000`,
`HASHLOCK_DKLEN = 32`, pinned to independent literals by
`hashlock_repro.rs:23-25` + `constants_equal_the_literals`. CI's
`test (ms-codec)` job is Ubuntu-only and the macOS matrix job runs
`cargo test -p ms-cli`, never `--workspace`, so `hashlock_repro` cannot meet
LibreSSL. The run-by-name gate fails correctly on a rename (a filtered-out test
prints `0 passed` and the `grep -E "test result: ok. 1 passed"` fails).

**Report claims independently re-measured and TRUE:** the acceptance items 1–5
(digests, plate string, `600`, `decode`/`inspect`/`derive`/`verify` behaviour,
`--random` gates), `codeword distance entr/hash = 17`, `ms gen-man` emitting
`ms-hashlock.1` among 13 pages, the corpus SHA, `grep -rn "Tag::try_new"
crates/` = 8 hits, and the four plan-count discrepancies. The one that is false
is C-2.

---

## Counts

| severity | count |
| --- | --- |
| Critical | **2** (C-1 advisory gate cannot fail; C-2 false final-gate count in the report) |
| Important | **3** (I-1 prompt vs `read_to_end`; I-2 unvalidated `--separator`; I-3 flag swallow succeeds) |
| Minor | **6** (M-1 … M-6) |
| Nit | **2** (N-1, N-2) |

**NOT GREEN.**

Nothing here is a wrong preimage, a leaked preimage on a channel the spec calls
public, an accepted input the spec refuses, or a misread between kinds — I tried
to construct all four and could not. C-1 is a gate that cannot fail on a
guarantee the spec states twice; C-2 is a record that would have absorbed a
19-test regression; the three Importants are journey-shaped defects that only
running the binary can reach.
