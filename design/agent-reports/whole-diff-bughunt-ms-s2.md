s2: 0C/0I/5M/2N

# Whole-diff review s2 (Sonnet xhigh): ms bug-hunt fixes, tests that can fail

Question: can every test this build adds or changes fail when the behaviour it pins is broken, and does every FIX section 6 row assert the FIX's exact output?

Answer: yes to both. No Critical, no Important. Five Minors and two Nits are hand-built mutants that survive because the FIX prose states a rule no row pins. None needs a fix in this PR (file as `deferred-minor`).

## Scope and method

- ms `7b6169b` (report commit `ca0a2e1`), toolkit companion `451ca5f8`. Read-only; the only files written are this report (uncommitted) and scratch under `/scratch/cache/bughunt/`.
- `~/.cargo/bin` first on PATH: cargo 1.85.0 (the pin). Scratch worktree `wd-s2-ms` at `7b6169b` (removed after use), `CARGO_TARGET_DIR=/scratch/cache/bughunt/wd-s2-ms-target` (left in place). Fault harness: `/scratch/cache/bughunt/wd-s2-faults/{faults.py,run.sh}`; one full `cargo nextest run --locked --workspace --no-fail-fast` per fault, logs in `/scratch/cache/bughunt/wd-s2-faults/out/` (`summary.txt` has one line per fault). Baseline there: 668 run, 668 passed, 23 skipped.
- Build vs reference: I applied `/scratch/cache/bughunt/ms-grade/fix-bughunt-ms-reference-r3.patch` (sha256 `26c20263...`, matches the FIX) to `f78c403`'s `crates/`. All 4 production files and all 4 test files are byte-identical to `7b6169b`. So the build is the FIX's reference patch, and any row/FIX mismatch below is a FIX property, not a transcription slip.

## 1. Mutation-gate survivors

`outcomes.json` of `/scratch/cache/mutation-gate/bughunt-triage-ca0a2e1b/`: 30 mutants, 30 CaughtMutant, 0 missed, 0 timeout, 0 unviable (cargo-mutants 27.1.0; baseline Success). Mutated files are exactly the 4 production files of the diff. Nothing to grade. The tool's mutants are function-level plus a few operators (`parse_ms1_symbols` got 3 FnValue and one `==` to `!=`; `run` got `!=`, `!`, `||`); it does not mutate `is_ascii()`, `to_ascii_lowercase()`, `&s[..]` vs `&lower[..]`, `payload.kind().single_tag()`, constant strings, `is_some_and` bodies or statement order. Section 2 covers those by hand.

## 2. Each section 6 row: plant the fault, confirm it fails for the stated reason

File-level revert of each production file to `f78c403` (the literal "revert that hunk"):

| revert | rows that fail (all else 668 minus these pass) |
|---|---|
| `shares.rs` | codec `bughunt_ms_01_zero_threshold_skips_tag_checks`: `seed tag + Entr: want ReservedTagNotEmittedInV01, got Ok(["ms10seedsqqqq...pscrjek0cdvwc"])` (the FIX's "RED today" text) |
| `decode.rs` | codec `03` (`U+0133 (reads as '3') accepted ... repaired: [(9, '3', '2')]`, and U+0130), codec `07` (`lowercase b: want InvalidChar('b'), got Err(Error("string length 50 outside v0.1 set [50, 56, 62, 69, 75]"))`), CLI `07` (`left: "UnexpectedStringLength" right: "Codex32"`), CLI `07b` (`left: (Some(1), Some("UnexpectedStringLength"), Some("string length 50 not in v0.1 set ...")) right: (Some(1), Some("Codex32"), Some("invalid character 'B' ..."))`) |
| `combine.rs` | CLI `04`, `04b`, `04c`, `04d`: stdout is the 8-line record / `ms decode`'s preimage JSON object (the FIX's "RED today") |
| `inspect.rs` | CLI `05`, `05b` (`"failure_reasons":[] ... "would_combine":true,"would_decode":true`), CLI `t2` (`"would_decode":true,"failure_reasons":[]`) |
| all four | exactly 12 rows fail: codec 01, 03, 07; CLI 04, 04b, 04c, 04d, 05, 05b, 07, 07b; t2. This is the FIX's list of 12 RED rows |

Rows the file reverts cannot show (the 04e control; row 10 has no production hunk):

- `04e`: fails only when the preimage warning is printed for a non-preimage (`F3f`, below). It cannot fail on the revert, by design (a control).
- `10`: with `gui_schema.rs`'s `repeating` forced to `false` (`F6b`), `bughunt_ms_10_gui_schema_combine_shares_not_repeating` fails. So the "kept as a pin" row can fail. (My first plant, `> 1000`, was ineffective because `Vec<String>`'s max is `usize::MAX`; discarded.)

Single-fault plants (each in its own build; "caught by" lists only the failing tests):

| id | fault | caught by |
|---|---|---|
| F1b | ZERO branch checks the reserved tag but not kind mismatch | codec 01 |
| F2a | `InvalidChar` carries the lowercased char | CLI 07b, codec 07 |
| F2b | `c as u8` without the `is_ascii` guard (the #3 alias) | CLI 07b, codec 03 |
| F2c | no ASCII-lowercase fold | CLI 07b, codec 07, `uppercase_envelope::u6_*` x2 |
| F2d | non-alphabet char skipped | CLI 07, 07b, codec 03, 07 |
| F3a | `Payload::Preimage` always returns the record | CLI 04, 04b, 04c, 04d |
| F3b | the record is never returned (the `--to phrase` default breaks) | `hashlock_other_verbs::combine_prints_a_recovered_preimage_as_decode_does` (pre-existing) |
| F3c | `emit_ms1` back to `Tag::ENTR` | CLI 04, 04c |
| F3c2 | `emit_ms1` always `Tag::HASH` | `cli_combine`, `combine_entropy_language_advisory` x2, `f677_help_matches_behaviour` (pre-existing) |
| F3d / F3e / F3f | warning removed / also under `--json` / printed for every kind | 04b / 04d / 04e |
| F3g / F3h / F3k | warning on stdout / after the advisory / text clause dropped | 04b each |
| F3l / F3m | JSON `kind` wrong / hex truncated to 16 bytes | 04c+04d / 04b+04c+04d |
| F4a, F4a2, F4b, F4b2, F4c, F4d | text header / reason line / `would_decode` / `would_combine` / `failure_reasons` / reason text wrong | 05 and 05b each |
| F4g | `ms_codec::inspect` stops lowercasing | 05b, `uppercase_envelope::u2_*` |
| F4h | text-mode secret share exits 1 | 05 |
| F5a, F5e | T2 reason not pushed / reason text wrong | t2 |

## 3. FIX section 2 outputs vs rows

Every section 6 row asserts the FIX's exact output, with the FIX's own stream/exit coverage:

- 01: the three refusal patterns with their payload fields, plus the byte-identity control.
- 03, codec 07: `InvalidChar(c)` with `c` as typed (also `decode()` for `b`, `B`).
- 04, 04b, 04c, 04d, 04e: exit 0, stdout byte-exact; 04b stderr = warning, newline, advisory, newline; 04d and 04e stderr = advisory alone.
- 05, 05b: exit 0, the nine text lines and the JSON line, byte-exact. T2: twelve text lines and the JSON line, byte-exact, plus the `ms decode` precondition.
- CLI 07: exit 1, `Codex32`, message, `details` null. 07b: the same for `B`, `?`, U+0133; text mode for `?`: exit 1, stdout empty, stderr byte-exact.
- Every CLI row runs the built `ms` through `assert_cmd::Command::cargo_bin("ms")`; none is a text-only pin.

The FIX section 2 outputs the build changed or restructured with no row asserting them (all graded below; none is Important):

1. `--to ms1` stderr, text and `--json` (FIX 2.2: "stderr carries the advisory alone"). Rows 04 and 04c assert stdout and exit only. See M4.
2. `ms combine [--to phrase] --json` on a preimage set (FIX 2.2: unchanged, `ms decode`'s object). The build wrapped this line in the new `if`; only the text form has a test (the pre-existing `combine_prints_a_recovered_preimage_as_decode_does`). See M3.
3. Secret share at `k` = 3..9: only `k` = 2 is pinned. See N1.
4. T2: only code 10 with a valid payload length; the order against `payload-length-mismatch`; code 9 as the top "known" boundary. See M5.
5. The 75-character `hash` single that `--to ms1` prints, being readable by `ms decode` and `ms hashlock --in` (FIX 2.2 and the manual text 5.1). No row; I ran it on the built `ms`: `ms decode --in` prints the preimage record, `ms hashlock --kind sha256 --in` prints `hash:02d449a3...`. See N2.

## 4. The ignored rows

All 9 `#[ignore]` reasons name a heading that exists in `design/FOLLOWUPS.md`, with a matching status and row reference:

| reason | slug (line) | status |
|---|---|---|
| `deferred-minor: ms-mlock-unpin-not-refcounted` | 887 | `deferred-minor` |
| `deferred-minor: ms-codex32-parts-short-data-panic` | 898 | `deferred-minor` |
| `deferred-minor: ms-too-many-errors-says-more-than-8` (x2) | 908 | `deferred-minor` |
| `deferred-minor: ms-cli-mnem-unknown-language-catch-all` | 918 | `deferred-minor` |
| `deferred-minor: ms-repair-accepts-mixed-case-with-typo` | 928 | `deferred-minor` |
| `deferred-minor: ms-combine-accepts-non-ms-hrp` | 947 | `deferred-minor` |
| `planned-feature: impl-bch-erasure-decoding-ms` (x2, t1 and t1b) | 1098 | `planned-feature` |

Run with `--run-ignored ignored-only -E 'binary(bughunt) | binary(bughunt_t)'`, all 9 fail for the stated reason: 02 `Locked went 4 kB -> 0 kB`; 06 `Codex32String::from_string PANICKED` and `inspect PANICKED`; 08 codec and CLI `"more than 8 errors; uncorrectable"`; 09 `unhandled ms_codec::Error variant ... code: 10`; x1 `accepted and repaired: Ok([(27, 't', '2')])`; t1 `not recovered at 47 of 47 positions; first: index 3: refused: codex32 parse error: InvalidChar('?')`; t1b `eight '?' erasures ... were refused: codex32 parse error: InvalidChar('?')`; t3 `accepted a share set with HRP xx (exit 0)`. (The same filter also picks up the `mlock::tests::g2_*` env-gated ignored tests that the `#[path]` module brings into the `bughunt` binary: g2_1 and g2_3 fail without their env var, g2_4 passes. They are pre-existing and no CI step runs the `bughunt` binary with `--include-ignored`: `rust.yml` uses `--bin ms` or `--test <name>`.)

## 5. Findings

No Critical, no Important. Each Minor below passes the likelihood test: the tool fails closed (a refusal or FAIL verdict is printed either way), no printed command fails as printed, and the path needs two or more unusual conditions. The hand-built mutants are in `faults.py` under the id shown; all ran green (survived) over 668 tests.

**M1 (F1c). `encode_shares` ZERO: "`n != 1` is checked first" is unpinned.** FIX 2.1 says `n != 1` still returns `InvalidShareCount { k: 0, n }` first. Moving `encode(tag, secret)?` ahead of the `n` check survives: a reserved tag with `n = 2` then returns `ReservedTagNotEmittedInV01` instead of `InvalidShareCount`. Conditions: a library caller passing `Threshold::ZERO` (none does; `ms split -k 0` refuses earlier), with `n != 1`, with a reserved or mismatched tag. Both outcomes are refusals. Fix text (one more assertion in row 01): `assert!(matches!(encode_shares(seed_tag, Threshold::ZERO, 2, &entr16), Err(Error::InvalidShareCount { k: 0, n: 2 })));`

**M2 (F2e). "First" non-alphabet character is unpinned.** FIX 2.4 says the first data-part character not in the alphabet ends the parse. A variant that scans on and reports the last one survives (every row has a single bad character). Conditions: two or more non-alphabet characters in one `ms repair` input. It still refuses, naming a character that really is invalid. Fix text (codec 07): `format!("{}b?{}", &good[..12], &good[14..])` must give `InvalidChar('b')`.

**M3 (F3b2). `ms combine --json` (default `--to phrase`) on a preimage set is unpinned.** Passing `false` instead of `args.json` to `emit_preimage` survives: `--json` would print the text record. The build rewrapped this line in the new `if args.to == CombineTo::Phrase`. Conditions: a future edit to that line, on a preimage set, with `--json`. Fix text (new row next to 04e): `ms combine --json -` on the preimage shares exits 0 with stdout equal to `ms decode --json -` on `encode(Tag::HASH, &x)`.

**M4 (F3i, F3j). `--to ms1` stderr is unasserted.** Printing the "these 64 hex characters" warning on `--to ms1` text mode survives (F3i), as does dropping the PrivateKeyMaterial advisory on `--to ms1` (F3j; no test anywhere asserts that advisory for `--to ms1`, for any kind, so this half is older than the build, and it is secret-handling, which is never C or I). Conditions: a regression in a one-arm code path, on a preimage set with `--to ms1`. Fix text: add `assert_eq!(err_s(&o), format!("{PRIVATE_KEY_ADVISORY}\n"));` to rows 04 and 04c.

**M5 (F5b, F5c, F5d). T2 is pinned at one point.** Row t2 uses code 10 with a valid length. Survivors: the reason pushed after `payload-length-mismatch` (FIX 2.5 and section 3 item 5 say before it); `get(code as usize + 1)` (flags code 9, Portuguese, as unknown: a false FAIL on a valid card; no test inspects a mnem string with code 9); `code == 10` only (codes 11..255 and 0x10 unflagged). The real code is one expression, `MNEM_LANGUAGE_NAMES.get(code as usize).is_none()`, so these need a future edit. Conditions: an edit to that expression, and (for the order) a hand-made string with both an unknown code and a bad length. Fix text (extend row t2): inspect a language-9 mnem string and assert `OK: would decode v0.2`; inspect a language-0x10 string and assert `mnem-unknown-language`; inspect language 10 with a payload of 17 bytes after the code and assert `failure_reasons` starts `["unexpected-string-length", "mnem-unknown-language", "payload-length-mismatch"]` (check the exact vector against the built `ms` before pinning).

**N1 (F4e). Secret share at `k` = 3..9.** Only `k` = 2 is pinned; a predicate narrowed to `threshold == 2` survives. The predicate is `share_index == 's'` and `is_share` already gates on `2..=9`, so it is hypothetical.

**N2. The read-back claim has no row.** FIX 2.2 and manual 5.1 say the `--to ms1` string is read by `ms decode` and `ms hashlock --in`. Row 04 compares it to `ms_codec::encode(Tag::HASH, ...)` but never runs those commands. They work on the built `ms` (checked above).

Equivalent mutant noted: F4f (`share_index == 's' || == 'S'`) passes by design, because `ms_codec::inspect` canonicalises to lowercase (F4g shows 05b fails when it does not).

## 6. Not re-reported

Findings already filed in FIX 9.2 to 9.5 and the toolkit FOLLOWUPS were not re-reported, including `ms-encode-shares-zero-validate-order` (`secret.validate()?` runs before the ZERO branch).

CI-side check (no finding): the new test binaries compile on the CI targets. `libc` and `zeroize` are unconditional dependencies of ms-cli, and libc 0.2.186 defines `MAP_ANONYMOUS` for Apple (`src/unix/bsd/apple/mod.rs:2072`).

Design concern (ungraded): the `bughunt` test binary compiles the real `mlock.rs` via `#[path]`, so its 8 lock unit tests run twice (also in `--bin ms`); harmless, but it doubles the surface the `--skip mlock` musl leg must keep skipping.
