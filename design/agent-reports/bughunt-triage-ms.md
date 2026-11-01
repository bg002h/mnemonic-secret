# Bug-hunt triage, mnemonic-secret (ms)

Agent: Sonnet triage. Worktree `/scratch/code/shibboleth/ms-worktrees/bughunt-triage`,
branch `bughunt/triage`, tests committed as `1b7dd78` (tests only; the report is
uncommitted). Hunt report: `design/agent-reports/bughunt-2026-10-31-ms.md`.

**Outcome: 10 CONFIRMED (#1-#9 and X1), 1 FALSE POSITIVE (#10), 0 UNRESOLVED. No STOP.**
Every confirmed row was checked in both directions on the pinned toolchain
(rustc 1.85.0): the row fails on unmodified code, and passes with a scratch fix
(`/scratch/cache/bughunt/ms/fix-NN.patch`, never committed, all reverted; `git diff` is empty).
Two things differ from the report: #7's example is not constructible and the
report's own fix does not cure the reachable case; #10 is false.

## 1. Baseline and final gate

Environment note: `cargo` resolved to `/usr/bin/cargo` 1.98.1, which shadows the rustup
shim, so `rust-toolchain.toml` (1.85.0) was ignored at first. The baseline was run on
both; I did everything else (fmt, clippy, fixes, final gate) with
`PATH=$HOME/.cargo/bin:$PATH` (`cargo 1.85.0`). A papercuts row is appended.

| run | result |
|---|---|
| Baseline, unmodified, `cargo nextest run --locked --workspace`, rustc 1.98.1 (`baseline-nextest.txt`) | 646 run, 646 passed, 0 failed, 11 skipped |
| Same set on pinned 1.85.0 (`-E 'not binary(bughunt)'`, `baseline-1.85-nextest.txt`) | 646 run, 646 passed, 11 skipped |
| `cargo test --locked --doc --workspace` (1.85.0) | 1 passed (ms_codec doc-test) |
| `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings` (1.85.0, with the new tests) | both exit 0 |
| **Final gate on committed tip `1b7dd78`** (`final-gate.txt`) | 666 run, 655 passed, **11 failed**, 14 skipped |

The final gate's 666 = 646 baseline + 20 new. Of the 20, 12 are my rows (5 codec, 7 CLI)
and 8 are re-runs of `mlock.rs`'s own unit tests, which come along because the #2 row
compiles the real `src/mlock.rs` into the test crate by `#[path]` (all 8 pass; the 3
extra skips are that module's `#[ignore]`d tests).

Failing set (11 tests) vs CONFIRMED set (10 bugs, #8 has a codec row and a CLI row):

- Failing: `ms-codec::bughunt` `bughunt_ms_01_zero_threshold_skips_tag_checks`,
  `_03_non_ascii_read_as_symbol`, `_06_long_hrp_short_data_panics`,
  `_08_too_many_errors_bound_is_8`, `_x1_mixed_case_correction`;
  `ms-cli::bughunt` `bughunt_ms_02_mlock_unpin_not_refcounted`,
  `_04_combine_to_ms1_ignored_for_preimage`, `_05_inspect_would_combine_unchecked`,
  `_07_string_length_error_contradicts_itself`, `_08_cli_repair_message_claims_more_than_8`,
  `_09_mnem_unknown_language_unhandled`.
- CONFIRMED: #1, #2, #3, #4, #5, #6, #7, #8, #9, X1. Equal. The only other new row,
  `bughunt_ms_10_gui_schema_combine_shares_not_repeating`, passes (false positive).
  No pre-existing test fails.

## 2. Verdicts

Fix/plant evidence was re-run on 1.85.0 (`verify-fix-NN-1.85.txt`); each fix's full-suite
regression run was done on 1.98.1 (`fixNN-suite.txt`): only the other bughunt rows failed,
except where noted for #8. Quotes in the "unmodified" column are the panic line from `final-gate.txt`.

| # | verdict | test | unmodified-run evidence (quoted) | fix or plant evidence | CLI reach | proposed grade | likelihood conditions / why |
|---|---|---|---|---|---|---|---|
| 1 | CONFIRMED | `ms-codec/tests/bughunt.rs::bughunt_ms_01_zero_threshold_skips_tag_checks` | `seed tag + Entr: encode_shares(ZERO) returned Ok(1 string(s) of len 50); decode() of it is_ok=false` (same for `hash tag + Entr`, and `entr tag + Preimage` len 75) | `fix-01.patch` = the report's fix (`return Ok(vec![crate::encode::encode(tag, secret)?]);`): `PASS bughunt_ms_01...`; codec suite 208/212 pass, the 4 fails are other bughunt rows | Library API only: `ms_codec::encode_shares(tag, Threshold::ZERO, 1, &payload)`. `ms split` cannot reach it: `Threshold::new(0)` refuses (`ms split -k 0` prints `error: invalid threshold 0; K-of-N shares require k in 2..=9`, exit 1) and the tag is fixed `Tag::ENTR` | **I** | Rule: a returned `Ok` string that `decode()` refuses is an unreadable backup, "stays C or I however rare". It needs a library caller that passes the zero threshold and a reserved or mismatched tag (caller error), so Opus may reasonably drop it to M |
| 2 | CONFIRMED | `ms-cli/tests/bughunt.rs::bughunt_ms_02_mlock_unpin_not_refcounted` (compiles the real `src/mlock.rs` by `#[path]`; a private bin-crate module has no other door) | `dropping pin A unlocked the page while pin B (same page) is still live: Locked went 4 kB -> 0 kB` (read from `/proc/self/smaps`, VMA of an `mmap`ed page) | `fix-02.patch` (per-page count map, `munlock` only at count 0): `PASS bughunt_ms_02...`, no other test regressed | **None today.** `mlock` is a private module of the bin crate. `gdb` on `ms derive --phrase - --template bip84` shows only sequential pins, never two live on one page: `mlock addr=0x555555a74000 len=4096`, `munlock addr=0x555555a74000 len=4096`, `mlock addr=0x7fffffffa000 len=4096`, `munlock addr=0x7fffffffa000 len=4096`. Pin sites are `parse.rs` (2, function-local) and `derive.rs:518` (stack seed, nothing pinned after it) | **M** | Secret-handling class: never C or I (operator, 2026-08-27); log as a follow-up. Also latent (no overlapping pin exists yet). A real fix must be mirrored in the toolkit's `mlock.rs` (G6 byte-equality invariant) |
| 3 | CONFIRMED | `ms-codec/tests/bughunt.rs::bughunt_ms_03_non_ascii_read_as_symbol` | `U+0133 (reads as '3') accepted by decode_with_correction and repaired: [(9, '3', '2')]` and `U+0130 (reads as '0') ... [(9, '0', '2')]` | `fix-03.patch` (reject non-ASCII before the `as u8` cast with `Codex32(InvalidChar(c))`): `PASS bughunt_ms_03...` | `printf 'ms10entrsqz4\xc4\xb342424242424242424242424q8lurdrzjnyn08' \| ms repair --ms1 -` -> `# Repair report` / `#   ms1 chunk 0: 1 correction at position 9: '3' -> '2'` / `ms10entrsqz4242424242424242424242424q8lurdrzjnyn08`, exit 4 (UNVERIFIED warning). `ms decode` and `ms inspect` refuse the same non-alphabet input | **M** | The repaired card is a valid codeword and exit 4 says "confirm before use", so no wrong card; the defect is that `was` reports `'3'` (the holder typed U+0133). Likelihood test (1) is not literally met (it succeeds, flagged UNVERIFIED). Unusual conditions: a holder enters a non-ASCII lookalike glyph; it falls in the data part; at most 4 errors in all. Opus to rule M vs I |
| 4 | CONFIRMED | `ms-cli/tests/bughunt.rs::bughunt_ms_04_combine_to_ms1_ignored_for_preimage` | `` `--to ms1` exited 0 and printed exactly what the default prints (flag silently ignored): kind: preimage (hashlock, 32 bytes / 64 hex characters) / preimage: 1111...1111 `` | `fix-04.patch` (Preimage arm: `if args.to == CombineTo::Ms1 { println!("{}", ms_codec::encode(Tag::HASH, &payload)?); return Ok(0); }`): `PASS bughunt_ms_04...`. The report's "honour `CombineTo::Ms1`" cannot just reuse `emit_ms1`: that calls `encode(Tag::ENTR, payload)`, which refuses a `Preimage` with `TagKindMismatch` (by reading, not run) | `ms combine --to ms1 --in preimage-shares.txt` -> exit 0, prints the 8-line preimage record (`kind:      preimage ...`, four digests), no ms1 string. The shares must come from the library (`ms split` has no preimage source, F-468) | **I** | Rule: a silently ignored flag stays C or I however rare. Conditions (do not demote it): preimage shares exist only via the library; the holder asks for `--to ms1` |
| 5 | CONFIRMED (index-`s` case only) | `ms-cli/tests/bughunt.rs::bughunt_ms_05_inspect_would_combine_unchecked` | `inspect --json says would_combine=true, failure_reasons=[] (combine refuses this string)` and `inspect text verdict is "OK: K-of-N share (would combine: needs 2 shares)" (combine refuses this string)` | `fix-05.patch` (index `s` -> `would_combine:false`, reason `share-index-secret`, text `FAIL:`): `PASS bughunt_ms_05...`; `inspect_share.rs` still passes | `ms inspect --json --in s-index-k2.txt` -> `{"failure_reasons":[],"report":{...,"share_index":"s",...},"would_combine":true,"would_decode":true}` exit 0; `ms combine --in s-index-k2.txt` -> `error: the secret share (index 's') must not be combined; ...` exit 2 | **I** | A wrong "OK" verdict is a wrong result, so not Minor (it does not fail closed); `combine` itself refuses safely. Conditions: the holder needs a secret-at-S string (the CLI never emits one) and runs `inspect` on it. The report's other examples are NOT bugs: shares carry a random id, and `ms combine` accepts a share set with HRP `xx` (checked), so "would combine" is accurate there |
| 6 | CONFIRMED | `ms-codec/tests/bughunt.rs::bughunt_ms_06_long_hrp_short_data_panics` | `Codex32String::from_string PANICKED` and `ms_codec::inspect PANICKED`; panic at `crates/ms-codec/src/codex32/mod.rs:249:24: byte range starts at 6 but ends at 0` | `fix-06.patch` (`parts_inner`: `if s.len() < 6 + checksum_len { return Err(Error::InvalidLength(self.0.len())); }`): `PASS bughunt_ms_06...` | `echo laaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa12e0ds56ptqt4x \| ms inspect -` -> `thread 'main' panicked at crates/ms-codec/src/codex32/mod.rs:249:24: byte range starts at 6 but ends at 0`, exit 101. The same string through `ms decode -` -> `error: string length 54 not in v0.1 set [50, 56, 62, 69, 75]`, exit 1 (the length gate shields `decode`) | **M** | Fails closed in effect (abort, no output, nothing wrong produced), but a panic is not a clean refusal and a library consumer could be crashed. Conditions (2+): a checksum-valid string with a 40-char HRP; its data part exactly the checksum length; 48..=93 chars; typed into `inspect`. No printed command fails |
| 7 | CONFIRMED (different input, report's fix insufficient) | `ms-cli/tests/bughunt.rs::bughunt_ms_07_string_length_error_contradicts_itself` | `self-contradictory error: "string length 50 not in v0.1 set [50, 56, 62, 69, 75]"`. The report's example (a 56-char mnem string) is not constructible: a mnem payload that long fails earlier with `tag "entr" payload length 19 not in expected set [16, 20, 24, 28, 32]`. The reachable case is the stand-in at `decode.rs:~213`, where `parse_ms1_symbols` returns `UnexpectedStringLength` for a non-alphabet character | **Report's fix** (carry `allowed` through the CLI mapping, `fix-07a-report-fix.patch`): row still FAILS: `self-contradictory error: "string length 50 not in set [50, 56, 62, 69, 75]"`. **Minimal fix** (`fix-07.patch`: `parse_ms1_symbols` returns `Codex32(InvalidChar(c))`): `PASS bughunt_ms_07...`, no regression | `echo ms10entrsqz4b42424242424242424242424q8lurdrzjnyn08 \| ms repair --ms1 -` -> `error: string length 50 not in v0.1 set [50, 56, 62, 69, 75]`, exit 1. Control, same string `ms decode -` -> `error: invalid character 'b' (not in codex32 alphabet)`, exit 1 | **I** | Fails closed, but the message names the wrong problem (tells the holder the right length is wrong) on `ms repair`, the tool meant for a typo'd card, with a plausible typo (`b`, `i`, `o`). One ordinary condition, so the likelihood test's (3) is not met. Opus may demote to M since `decode` gives the right text |
| 8 | CONFIRMED | `ms-codec/tests/bughunt.rs::bughunt_ms_08_too_many_errors_bound_is_8`; `ms-cli/tests/bughunt.rs::bughunt_ms_08_cli_repair_message_claims_more_than_8` | `a string with 5 substitutions is reported as: "more than 8 errors; uncorrectable" (the decoder corrects at most 4; 5 errors is not 'more than 8')`; CLI: `` `ms repair` on a string with 5 substitutions says: "more than 8 errors; uncorrectable" `` | `fix-08.patch` (reword codec `Display` and CLI message to `more than 4 errors; uncorrectable`, `bound: 8` field untouched): both rows `PASS`. Changing the field to `bound: 4` instead (tried, not kept) breaks the existing `bch_decode::five_error_too_many`, which asserts `bound == 8`, and `design/SPEC_ms_codec_test_hardening.md` T3-ms-2 pins `{ bound: 8 }` | `echo ms106ntrspz424t4242k2424t42424242424q8lurdrzjnyn08 \| ms repair --ms1 -` (the valid card with 5 substitutions) -> `error: more than 8 errors; uncorrectable`, exit 2 | **M** | Text only; fails closed (exit 2, no data); no printed command fails. Only one unusual condition (a card with at least 5 substitutions), so by the letter of the likelihood test this is I; I propose M because the operative word "uncorrectable" is right and nothing acts on the number. Opus to rule. BIP-93 says it corrects up to 4 substitutions and detects up to 8 |
| 9 | CONFIRMED | `ms-cli/tests/bughunt.rs::bughunt_ms_09_mnem_unknown_language_unhandled` | `falls through to the catch-all: error: unhandled ms_codec::Error variant: Error("unknown mnem wordlist-language code: 10")` | `fix-09.patch` (the arm from `prover-9/fix.diff`: `FormatViolation`, kind `MnemUnknownLanguage`): `PASS bughunt_ms_09...` | `ms decode --in mnem-lang10.txt` (51-char checksum-valid mnem string, language byte 10) -> `error: unhandled ms_codec::Error variant: Error("unknown mnem wordlist-language code: 10")`, exit 1 (JSON kind `BadInput`) | **M** | Fails closed (non-zero, names the problem, though in debug wording, with exit 1 instead of 2). Conditions (2+): a string from a newer or other encoder that uses language >= 10 (this encoder's `validate()` never emits one); decoded by this older `ms`. No printed command fails |
| 10 | **FALSE POSITIVE** | `ms-cli/tests/bughunt.rs::bughunt_ms_10_gui_schema_combine_shares_not_repeating` | Passes on unmodified code: `ms gui-schema` gives `[{'name': 'shares', 'required': False, 'repeating': True}]` | **Plant** (`plant-10.patch`: `repeating: None::<clap::builder::ValueRange>.map(\|r\| r.max_values() > 1).unwrap_or(false)`, i.e. the report's "`get_num_args()` is None"): row FAILS: `assertion left == right failed: combine's shares positional (Vec<String>, K or more) must be repeating: {"name":"shares","repeating":false,"required":false}`; reverted. Why the real code differs from the prover's copy: clap's derive already sets `.num_args(1..)` for a `Vec<T>` positional (`vendor/clap_derive/src/derives/args.rs:288` and `:305`), so `Arg::get_num_args()` (`vendor/clap_builder/src/builder/arg.rs:4349`, returns `self.num_vals`) is `Some` on the unbuilt `Cli::command()`. The prover's `combine_repeating_test.rs` never ran (offline build failed) | n/a | n/a | n/a |
| X1 | CONFIRMED as behaviour (BIP-93 does not require the rejection) | `ms-codec/tests/bughunt.rs::bughunt_ms_x1_mixed_case_correction` | `a mixed-case string with one substitution was accepted and repaired: Ok([(27, 't', '2')])`. Preconditions pass: a CLEAN mixed-case string is rejected by `decode` and by `decode_with_correction` | `fix-x1.patch` (reject a string holding both ASCII cases in `parse_ms1_symbols`, `Codex32(InvalidCase)`): `PASS bughunt_ms_x1...` | `echo ms10entrSqz4242424242424242424t42424q8lurdrzjnyn08 \| ms repair --ms1 -` (the valid card with `s` upper-cased and one substitution) -> `# Repair report` / `#   ms1 chunk 0: 1 correction at position 27: 't' -> '2'` / `ms10entrsqz4242424242424242424242424q8lurdrzjnyn08`, exit 4. Clean mixed-case through `ms repair` or `ms decode` -> `error: mixed case at character 'N' (codex32 strings must be all-lower or all-upper)`, exit 1 | **N** | See the BIP-93 quotation below. Not a wrong card: the result is a valid codeword flagged UNVERIFIED (exit 4), and the case change is not reported as a correction. The inconsistency with `decode`/`combine` (project invariant U4, `tests/uppercase_envelope.rs`) is the only defect. Opus to rule N vs M |

### X1: what BIP-93 says (fetched `bitcoin/bips` `bip-0093.mediawiki`, lines 82, 154, 159, 161)

- Validity: "As with bech32 strings, a codex32 string MUST be entirely uppercase or entirely lowercase."
- Correction: "Implementations optionally interpret other non-bech32 characters, or characters with incorrect case, as erasures."
- Correction: "If a string consisting of valid bech32 characters in the proper case can be made valid by substituting 4 or fewer characters, then the implementation suggests such a string as a correction."
- Correction: "implementations SHOULD NOT automatically proceed with a corrected codex32 string without user confirmation of the corrected string"

So the correction path may accept wrong-case characters (as erasures) and must only suggest, which `ms repair` does (exit 4, "UNVERIFIED"). The hunter's "BIP-93 says to reject" is true of a mixed-case string as a valid string, and not mandated for correction. The same sentence ("characters with incorrect case, as erasures") also covers #3's non-ASCII characters.

### Reach transcripts

Verbatim in `/scratch/cache/bughunt/ms/reach.txt` (unmodified binary `ms 0.20.1`, built from the clean worktree, exit codes recorded); condensed above. Generated inputs: `preimage-shares.txt`, `s-index-k2.txt`, `mnem-lang10.txt` in the same directory.

## 3. Open product questions

- **#4**: print the recovered `hash` ms1 for `--to ms1` on preimage shares, or refuse? And `--to entropy` / `--to phrase` are also silently ignored for a preimage (phrase is "never words" by design).
- **#5**: which reasons should `inspect` list for a share-kind string? Only index `s` is wrong today (combine refuses it). `ms combine` accepts shares whose HRP is not `ms`; should it require `ms`, and should `inspect` then say so?
- **#7**: for a non-alphabet character in `ms repair`, say `invalid character` (as `decode` does) or treat it as an erasure per BIP-93? Also the pre-dispatch length error lists only the entr set, though mnem and preimage lengths are valid.
- **#8**: reword only (the `bound: 8` field is pinned by `bch_decode::five_error_too_many`, SPEC_ms_codec_test_hardening T3-ms-2 and possibly the toolkit), or change the field and its pins?
- **#3 / X1**: reject non-ASCII and mixed case in the correction path (project invariant U4), or treat them as erasures (BIP-93 optional)?
- **#1**: the report's fix (delegate to `encode()`) is clear; the question is only whether a library-only caller error is I or M.
- **#9**: `FormatViolation`/exit 2 as the report says (kind name `MnemUnknownLanguage` from the prover's patch)?

## 4. Noticed, not tested

- `ms inspect` on a mnem string with language byte 10 prints `OK: would decode v0.2 ... language: unknown` although `ms decode` refuses it (same family as #5).
- `decode.rs` post-dispatch kind-length gate (lines ~62-70) looks unreachable: each kind and entropy length maps to exactly one string length, so earlier checks fire first; that is why #7's example cannot be built.
- Adding any `#[cfg(test)]` module to `crates/ms-cli/src/mlock.rs` would trip `mlock_g6_invariant.rs` (byte-equality with the toolkit's copy), which is why the #2 row uses `#[path]` from `tests/bughunt.rs`; the real #2 fix must be mirrored in the toolkit.
- Toolchain trap: `/usr/bin/cargo` 1.98.1 shadows the rustup shim, so the 1.85.0 pin is ignored unless `~/.cargo/bin` comes first (papercuts row added).
