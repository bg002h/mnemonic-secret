# Bug-hunt triage round 2, mnemonic-secret (ms): three suspicions

Agent: Sonnet triage, brief `coordinator/briefs/bughunt-ms-triage2.md`. Worktree
`/scratch/code/shibboleth/ms-worktrees/bughunt-triage2`, branch `bughunt/triage2`
(local), base `4755eb9`. Rows committed as `7a0edce` (tests only; this report is
uncommitted). Toolchain: `cargo 1.85.0` (`PATH=$HOME/.cargo/bin:$PATH`). Evidence
and scratch patches: `/scratch/cache/bughunt/ms2/`. No STOP.

**Outcome: 3 CONFIRMED (T1 one `?`, T2, T3), 1 UNRESOLVED (T1b: eight `?`,
"fix direction not runnable: feature-sized"), 0 FALSE POSITIVE.** Scratch fixes
`fix-t1.patch`, `fix-t2.patch`, `fix-t3.patch` were all reverted (`git diff` is
empty at the tip).

## 1. Gate

| run | result |
|---|---|
| `cargo fmt --check` | exit 0 (after `cargo fmt` on the two new files only) |
| `cargo clippy --locked --all-targets -- -D warnings` | exit 0 (`clippy.txt`) |
| `cargo nextest run --locked --workspace --no-fail-fast` on `7a0edce` (`final-gate.txt`, run once) | 670 run, 655 passed, **15 failed**, 14 skipped |

670 = 666 (round 1's tip) + 4 new rows. Passed count is unchanged at 655.

Failing set (15) = round 1's 11 + this round's 4, exactly:

- round 1 (11): `ms-codec::bughunt` `_01_zero_threshold_skips_tag_checks`, `_03_non_ascii_read_as_symbol`, `_06_long_hrp_short_data_panics`, `_08_too_many_errors_bound_is_8`, `_x1_mixed_case_correction`; `ms-cli::bughunt` `_02_mlock_unpin_not_refcounted`, `_04_combine_to_ms1_ignored_for_preimage`, `_05_inspect_would_combine_unchecked`, `_07_string_length_error_contradicts_itself`, `_08_cli_repair_message_claims_more_than_8`, `_09_mnem_unknown_language_unhandled`.
- round 2 (4): `ms-codec::bughunt_t` `bughunt_ms_t1_question_mark_erasure`, `bughunt_ms_t1b_eight_erasures_recovered`; `ms-cli::bughunt_t` `bughunt_ms_t2_inspect_unknown_language_says_ok`, `bughunt_ms_t3_combine_accepts_non_ms_hrp`.

## 2. Verdicts

Each row fails at its own assertion (not a precondition) on unmodified code
(`red-unmodified.txt`). Preconditions asserted in each row passed.

| item | verdict | row | unmodified evidence (quoted) | fix or plant evidence | CLI reach (unmodified `ms 0.20.1`) | proposed grade | conditions / why |
|---|---|---|---|---|---|---|---|
| T1 `?` erasure, one `?` | **CONFIRMED** | `ms-codec/tests/bughunt_t.rs::bughunt_ms_t1_question_mark_erasure` (every data-part index 3..49) | `one `?` (an erasure, BIP-93 line 158) was not recovered at 47 of 47 positions; first: index 3: refused: string length 50 outside v0.1 set [50, 56, 62, 69, 75]` | `fix-t1.patch`: in `decode_with_correction`, if the string holds exactly one `?`, try the 32 alphabet symbols, keep the one whose polymod residue is 0, `decode` it, report a `CorrectionDetail { was: '?', now }` (27 lines). Row: `PASS` (`verify-fix-t1.txt`). With it, `ms repair` prints `1 correction at position 9: '?' -> '2'`, exit 4 (`reach-t1-with-fix.txt`). It covers exactly one `?`; it is not the feature (see T1b) | `echo ms10entrsqz4?42424242424242424242424q8lurdrzjnyn08 \| ms repair --ms1 -` -> `error: string length 50 not in v0.1 set [50, 56, 62, 69, 75]`, exit 1. `--json`: `{"error":{"kind":"UnexpectedStringLength",...,"exit_code":1,"details":{"allowed":[50,56,62,69,75],"got":50}}}`. Same string to `ms decode -` and `ms inspect -`: `error: mixed case at character '?' (codex32 strings must be all-lower or all-upper)`, exit 1 (`reach-t1.txt`). Toolkit `mnemonic repair --ms1 -` (installed 0.105.1): `error: repair: chunk 0 parse failed before correction could run: non-bech32 char '?' at position 9`, exit 2 (`reach-t1-toolkit.txt`) | **M** (feature gap; manual overclaim ruled separately, see section 3) | BIP-93 lines 155-158 are a recommendation ("we recommend that: ... Implementations interpret "?" as an erasure"), not a MUST. (1) fails closed: yes, exit 1, but names the wrong problem (the length), which is #7, already graded I and fixed to `invalid character '?'`. (2) no printed command fails: no manual page prints a `?` command (section 3). (3) conditions: a plate with an unreadable (not misread) character; the holder types `?` for it instead of guessing a symbol; uses `ms repair`. Source: `parse_ms1_symbols` stand-in at `crates/ms-codec/src/decode.rs:207-215` (`UnexpectedStringLength` for any non-alphabet char); no erasure code anywhere (`grep -rni erasure crates` is empty) |
| T1b `?` erasure, eight `?` | **UNRESOLVED**: fix direction not runnable: feature-sized | `ms-codec/tests/bughunt_t.rs::bughunt_ms_t1b_eight_erasures_recovered` (`?` at indices 3, 9, 15, 21, 27, 33, 39, 45) | `eight `?` erasures (within BIP-93's stated 8) were refused: string length 50 outside v0.1 set [50, 56, 62, 69, 75]` | Still FAILS with `fix-t1.patch` applied (same message). Not built. Independent oracle that the behaviour is achievable: `erasure_oracle.py` (BIP-93's reference `ms32_polymod`, GF(2) linear solve, no ms code) recovers the real card uniquely for 1..8 scattered erasures (200/200 random position sets each), 8 consecutive, and 13 consecutive (`erasure_oracle.txt`) | as T1 | **M** with T1 (same item, full strength) | Feature estimate: polymod is GF(2)-linear in the symbols, so a solver builds 5e columns (polymod of unit vectors) and solves 65 equations in 5e unknowns by Gaussian elimination; the oracle is about 35 lines of Python. Needs: `?` parsing in `parse_ms1_symbols` (and uppercase), an erasure solver in `bch_decode.rs`, a policy for mixed erasures plus substitutions (BIP-93 states 4 substitutions or 8 erasures, not a mix), `CorrectionDetail.was = '?'` and the repair report, exit 4 (an erasure fill is an unverifiable candidate like a substitution), `decode`/`inspect` text for `?`, spec section and TRACE rows, mutation gate, toolkit-manual lockstep (`43-ms.md` repair chapter), then an ms-codec release and a toolkit bump (the toolkit pins `ms-codec = "0.7"` and has its own `?` pre-check). Roughly 150-250 lines of production code, 20+ tests, three repos. A smaller option is fill-and-check (the shape of `fix-t1.patch`): try the 32 symbols per `?`; it is exact for one `?`, costs 32^e polymod runs (e = 2: 1,024; e = 3: 32,768; e = 4: about 1.05 M), and is not usable at 8 (about 1.1e12), so the full 8 needs the linear solve. Controller-measured baseline (`controller-subst-probe.txt`, not re-run by me): substitution decoding gives a holder 4 unknown-position errors today, erasure decoding of `?` would give 8 known positions |
| T2 inspect, language code >= 10 | **CONFIRMED** | `ms-cli/tests/bughunt_t.rs::bughunt_ms_t2_inspect_unknown_language_says_ok` | `inspect --json says would_decode=true, failure_reasons=[] (decode refuses this string)` and `inspect text verdict is "OK: would decode v0.2" (decode refuses this string)` (preconditions passed: `decode` exits non-zero naming "unknown mnem wordlist-language code") | `fix-t2.patch`: in `analyze()`'s `InspectKind::Mnem` arm, push reason `mnem-unknown-language` when `language >= MNEM_LANGUAGE_NAMES.len()` (5 lines, plus a `reason_text` arm). Row: `PASS` (`verify-fix-t2.txt`); existing `inspect` tests still pass (only round 1's #5 row fails, `fix-t2-inspect-regress.txt`). With it: `FAIL: would NOT decode v0.1` / `reason: mnem-unknown-language (...)`; JSON `"would_decode":false,"failure_reasons":["mnem-unknown-language"]`, exit 0 (`reach-t2-with-fix.txt`) | String `ms10entrsqg9qqqqqqqqqqqqqqqqqqqqqqqqqqfqh998pfxtntx`: `ms inspect --in F` -> `OK: would decode v0.2` ... `payload_bytes: 0a00...` `kind: mnem` `language: unknown`, exit 0. `--json` -> `"would_decode":true,"failure_reasons":[]`. `ms decode --in F`, `ms verify --in F`, `ms repair --in F` -> `error: unhandled ms_codec::Error variant: Error("unknown mnem wordlist-language code: 10")`, exit 1 (`reach-t2.txt`) | **I** | Same family and same reasoning as #5 (graded I): the verdict tool prints a wrong OK, which does not fail closed (likelihood test (1) not met), and a tool's wrong claim about what it did is blocking regardless of rarity. Mitigations for Opus: needs a checksum-valid mnem string from a foreign or newer encoder with a reserved language code 10-15 or a non-zero high nibble (this encoder's `validate()` never emits one, `payload.rs:95`; a typo cannot give one with a valid checksum); a holder who audits with `inspect` before `decode`; `decode` itself refuses cleanly. Real-code gap: `analyze()` Mnem arm (`crates/ms-cli/src/cmd/inspect.rs:215-222`) checks payload length only; `emit_text`/`emit_json` print `unwrap_or("unknown")` for the name. |
| T3 combine, non-`ms` HRP | **CONFIRMED** | `ms-cli/tests/bughunt_t.rs::bughunt_ms_t3_combine_accepts_non_ms_hrp` | ``ms combine` accepted a share set with HRP `xx` (exit 0) and printed: "42424242424242424242424242424242\n"` (preconditions passed: the same construction under HRP `ms` combines to the same hex; `ms decode` refuses a single `xx` string) | `fix-t3.patch`: in `combine_shares`, after canonicalising, `WrongHrp` if any share's HRP is not `ms` (11 lines, mirroring `discriminate()`, `envelope.rs:124`). Row: `PASS` (`verify-fix-t3.txt`); all combine/share/split tests still pass (only round 1's #4 and #5 rows fail, `fix-t3-combine-regress.txt`). With it: `error: wrong HRP: got "xx", expected "ms"`, exit 2 (`reach-t3-with-fix.txt`) | Shares `xx12zzzzatfd95kj6tfd95kj6tfd95kj6tfdqz6yfuvm0ttw83` + `xx12zzzzc84lj9ljhmflj9ljhmflj9ljhmflqm0r8lx7hrahv9` (secret-at-S payload = `00` + 16 x `42`): `ms combine --in F` -> `entropy: 42424242424242424242424242424242` / `phrase: drastic bamboo mountain loyal category cancel animal embark drastic bamboo mountain lunch` / `kind: entr`, exit 0. `--json` -> `"kind":"entr","entropy_hex":"4242..."`. `ms combine --to ms1 --in F` -> `ms10entrsqppyysjzgfpyysjzgfpyysjzgfpqeekx9s9ep9dy0`, exit 0 (a valid `ms` card). Single `xx` string to `ms decode -` -> `error: wrong HRP: got "xx", expected "ms"`, exit 2. One `xx` share to `ms inspect -` -> `OK: K-of-N share (would combine: needs 2 shares)`, exit 0 (`reach-t3.txt`) | **M** (Opus may rule I) | The output IS the secret the strings encode: identical to the `ms`-HRP control (`4242...` in, `4242...` out), so no wrong card, address or data. Likelihood test (1) is not literally met (it succeeds rather than refusing), so the usual grades apply; no printed command fails. Reach: real cards cannot produce this set. BIP-93 fixes the HRP at `ms` (lines 70, 96). A typo in a real share's HRP breaks the checksum (`ms`->`mt`: `error: BCH checksum invalid (short code)`, exit 1); a mixed `ms`/`xx` set is refused (`HRP mismatch among shares: "ms" vs "xx"`, exit 1); mk1/md1 differ by target residue (manual primer: "an mk1 string can never accidentally validate as an md1 or ms1 string"). Only a crafted or non-BIP-93 set reaches it, and its payload must also be ms-shaped (prefix 0x00/0x02/0x03 plus a valid length) or `dispatch_payload` refuses. Conditions: (a) a valid-checksum share set with a non-`ms` HRP; (b) ms-shaped payload; (c) handed to `ms combine`. Why it still matters: `combine` is the one ms path that takes ms1 data and skips rule 2; `--to ms1` turns a foreign-HRP set into a valid `ms1` card. Real-code gap: `crates/ms-codec/src/shares.rs:207-252` has no HRP check (the spec lists only `Mismatched*` for combine, `design/SPEC_ms_v0_2_kofn.md:64,138`) |

## 3. T1 manual findings

Searched `/scratch/code/shibboleth/mnemonic-toolkit/docs/manual/src/` (local
checkout at `ef99021a`; the coordinator cites the same lines on `c2b09861`).
No page tells a holder to type `?` (or any other mark) into an `ms` or
`mnemonic` command. Every `?` hit outside the primer is unrelated (`$?`,
mermaid, prose). No printed `ms` command in the manual includes a `?`. The one
printed `ms repair` example, `43-ms.md:819` (`ms repair --in broken.txt`,
position 17 `q` -> `z`), runs as printed on the unmodified binary: exit 4, the
corrected string `ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f`, as the
page says (`reach-t1.txt`). The page that does promise the capability is the BCH
primer, `60-appendices/65-bch-codex-primer.md`:

- `:22-24`: "BIP-93 guarantees correction of up to 4 unknown-position substitutions, or up to 8 errors at known positions ("erasures"), or up to 13 consecutive erasures."
- `:108`: "| Correction — erasures (known positions, e.g., `?`) | up to **8** erasures corrected | BIP-93 §"Error Correction" |"
- `:109`: "| Correction — consecutive erasures (a single scratch) | up to **13** consecutive erasures corrected | BIP-93 §"Error Correction" |"
- `:154-156`: "An "erasure" in the table above means *the character at a known position is unreadable, but the position itself is preserved* (typically rendered `?`). The string length is unchanged."
- `:281-282`: "When stamping errors exceed the per-card correction radius (4 substitutions or 8 erasures in known positions), the fall-backs are:"

Reading: `:108`/`:109` state properties of the code (sourced to BIP-93), and
`:154-156` defines the word; none is a command. `:281-282` is the overclaim: it
names 8 erasures as part of the "per-card correction radius", which `ms repair`
does not have. The `ms` chapter itself is accurate, `43-ms.md:751-752`: "The
string is corrected within the `BCH(93,80,8)` `t=4` capacity (up to four
substitution errors)." and `:786` "> 4 substitution errors". A holder who reads
the primer, then types `?` into `ms repair`, gets #7's message today. Opus to
rule whether `:281-282` is an advice text that promises a recovery the tool
lacks (I) or documentation only (M).

Controller-measured, cited not re-run (`/scratch/cache/bughunt/ms2/controller-subst-probe.txt`, script beside it, master-code debug binary, 50-char entr string for entropy `000102..0f`): `ms repair` substitution correction works to spec. 1, 2, 3 and 4 random substitutions were corrected to the original in 300/300 trials each; 5 and 6 were refused (exit 2) in 300/300; zero miscorrections. One or two `?` marks gave exit 1 with the #7 message in 20/20. Rewriting each `?` as `q` and then repairing corrected every case up to 4 marks (300/300 at k = 4; the rates at k = 1-3 and above 4 are distorted because 8 of the vector's 47 data characters are already `q`: 48/300 unchanged-valid at k = 1, 118/300 refused at k = 5). So today a holder can get up to 4 unreadable characters repaired by typing any alphabet symbol for each, but no manual page says so, and the tool cannot use the known positions: the primer's 8 would need erasure decoding.

Constellation facts (coordinator-settled, not re-derived): the only erasure code
in the constellation is `wc-codec` (word cards, Reed-Solomon), not codex32. In
this repo `grep -rni erasure crates` is empty and `ms-codec`'s vendored
`src/codex32/` has none; `design/` mentions erasures only in the bug-hunt
reports. `mnemonic repair` is a thin wrapper over `decode_with_correction`
(`repair_via_ms_codec`, `repair.rs:1166`), so it shares the gap but adds its own
earlier refusal (section 2, T1 CLI reach).

## 4. Open product questions

- T1: implement `?` erasure decoding (a BIP-93 recommendation, promised by the primer), or correct the primer (`:108`, `:109`, `:281-282`) to say that `ms repair` and `mnemonic repair` correct substitutions only and do not accept `?`, optionally adding the measured workaround (type any alphabet symbol for an unreadable character; up to 4 are repaired)?
- T1: if implemented, erasures only or mixed with substitutions; the maximum count (8; 13 only when consecutive); exit 4 and `was: '?'` in the report?
- T1 / #7: `ms decode` and `ms inspect` call a `?` (or `!`) "mixed case at character '?'" (`reach-t1-decode-chars.txt`; `b` gives `invalid character 'b'`). Does the #7 ruling ("repair and decode name the same character") cover changing decode's text for `?`, and should `ms repair`, `ms decode` and `mnemonic repair` (exit 1 vs 2 today) agree on one text and code?
- T2: reason name and text for the new `inspect` refusal (`mnem-unknown-language` is the scratch name); SPEC_ms_cli §3 calls the `failure_reasons` set "locked at v0.1", so it needs a spec amendment. Should it mirror #9's wording (`unknown mnem wordlist-language code 10 (known codes are 0-9)`)? The FAIL header reads "would NOT decode v0.1" even for mnem strings (pre-existing, untouched).
- T3: refuse in the codec (`combine_shares` returns `WrongHrp`, the scratch fix; `ms combine` then prints `wrong HRP: got "xx", expected "ms"`, exit 2 as `decode` does) or in the CLI only? If `combine` changes, `inspect` gains the existing `wrong-hrp` reason for shares in the same change (grading report, #5 ruling).
