# FIX_bughunt1119_ms_codex32_ctors: the raw codex32 constructors (bh-ms-I2; bh-ms-I3 as M)

Model: Claude Opus 5.5 (claude-opus-5-5), 2026-11-22. Author of this FIX document (not the coordinator).
Base: `origin/master` 423f173 (ms-cli 0.21.0, ms-codec 0.10.1). Branch `fix/ms-codex32-ctors`. Track: small.
Source: `design/agent-reports/bug-hunt-2026-11-19.md` I2 (L78-92) and I3 (L94-108); grading
`coordinator/agent-reports/bughunt-1119/grade-ms.md` L33-34; triage `triage-ms-t1.md` L14-18.

## 0. Rulings and scope

- **T24 (architect, decided):** `Codex32String::from_seed` is an unchecked builder (the repo's own tests build
  invalid strings through it: `crates/ms-codec/tests/hashlock_kind.rs:168-185`, `crates/ms-codec/tests/negative.rs:22`
  in `build_with`, used by `rule_4_share_index_not_secret_rejected` at L77-90). Document that; bh-ms-I3 grades **M**;
  this FIX fixes I2 and documents I3. `from_seed` keeps its behaviour byte for byte.
- **P7:** library-only reach grades I unless the function is documented as unchecked. `from_unchecksummed_string`
  is not documented as unchecked and returns `Ok` with a card its own parser refuses, so **I2 stays I** and is fixed.
- No CLI path reaches `from_unchecksummed_string`: it has no caller anywhere in this repo, the toolkit,
  mnemonic-engrave or mnemonic-refugium at their origin heads (`git grep`, 2026-11-22). `from_seed` is reached by
  `ms encode`/`ms split` through `envelope.rs:293` and `shares.rs` only with inputs ms-codec has already checked
  (bug report I3: "Ordinary `ms encode` validates its higher-level input first"), and its behaviour does not
  change. Nothing a holder sees on a terminal changes, so there is no toolkit-manual or mnemonic-gui lockstep.
- **Out of scope:** R2 (`ms-codex32-parts-short-data-panic`, FOLLOWUPS L897; the consultant's I2 fix paragraph says
  "apply R2's parser guard alongside"). R2 is not a key of this row, and its error text is open in its own FOLLOWUPS
  entry ("Pick an error whose text names the short data part"). This FIX neither widens nor narrows R2: a body that
  reaches `parts_inner` today still reaches it after the fix, through the same `sanity_check`.

## 1. Spec delta: `Codex32String::from_unchecksummed_string` (crates/ms-codec/src/codex32/mod.rs:170-195)

Current (vendored verbatim from codex32 0.1.0 `lib.rs:120-145`; the upstream bug is identical): the residue of
HRP + body is appended directly (no `input_own_target()`), in lower case, and the result is returned after
`sanity_check` only.

After the fix, for an input `s`:

1. Engine choice is unchanged: short code (13 characters) when `s.len() < 81`, else long code (15 characters).
2. HRP and body are fed to the engine as today; the engine's case and character errors (`InvalidCase`,
   `InvalidChar`, `Field`) are returned as today, before anything else.
3. **NEW:** `checksum.input_own_target()` is called before `into_residue()`, so the appended symbols are the
   checksum (the same construction `from_seed` uses at mod.rs:418-421).
4. **NEW:** the appended symbols take the case of `s`: upper case if `s` contains an ASCII upper-case letter,
   otherwise lower case. (The engine has already refused mixed case, so this is the case of the whole input. An
   input with no letter at all gets lower case, the `Fe::to_char` default.)
5. The completed `String` is wrapped in `Codex32String` (which zeroizes on drop) **before** any further check, so
   every error after completion drops a scrubbed buffer (secret hygiene; see section 7).
6. **NEW:** if the completed length is outside both brackets `from_string` accepts (short `48..94`, long
   `125..128`, mod.rs:199-204), return `Err(Error::InvalidLength(<completed length>))`, the same variant and
   value `from_string` returns for that string.
7. `sanity_check()?` runs as today; then `Ok`.

Consequence (the contract the tests pin): every `Ok` from `from_unchecksummed_string` is a string that
`Codex32String::from_string` accepts, and completing the first `len - 13` (short) or `len - 15` (long) characters
of a valid codex32 string reproduces that string exactly, in its case.

Implementation note (code judgement, not a product point): do not route the completed string through
`from_string(String)` by value: on its `InvalidLength` path it drops the moved `String` unscrubbed, which would
regress the current `sanity_check` error path (that one drops a `Codex32String`). Steps 5-7 give the same accept
set and the same error values. The scratch variant that passed (section 4) is
`/scratch/cache/ms-codex32-ctors-logs/scratch-fix2.diff`.

## 2. Spec delta: doc comments (library documentation; PROPOSED texts, architect confirms)

2a. `from_unchecksummed_string` (mod.rs:170, replaces the one-line comment):

    /// Construct a codex32 string from a not-yet-checksummed string
    ///
    /// Appends the checksum: the 13-character short code when `s` is shorter
    /// than 81 bytes, otherwise the 15-character long code, in the case of `s`.
    /// Every `Ok` result is accepted by [`Codex32String::from_string`]. A
    /// completed string outside both codex32 lengths (48 to 93 characters
    /// short, 125 to 127 long) is refused with [`Error::InvalidLength`]
    /// carrying the completed length.

2b. `from_seed` (mod.rs:358, replaces `/// Creates a S share from bare seed data`; T24's "document that"):

    /// Creates a S share from bare seed data
    ///
    /// This is an unchecked builder. It refuses a threshold other than 0 or 2
    /// to 9, an identifier that is not 4 characters, an identifier character
    /// outside the bech32 alphabet, and an upper-case HRP or identifier (it
    /// builds lower case only). It does NOT check that the share index is `s`
    /// when the threshold is 0, nor that the finished string has a codex32
    /// length (with HRP `ms`: 16 to 44 or 63 to 64 bytes of `data`). Such a
    /// result is `Ok`, but [`Codex32String::from_string`] refuses its string;
    /// check the result with `from_string` unless the inputs are known to be
    /// valid. The crate's negative tests rely on this to build unreadable
    /// fixtures.

Every claim in 2b was measured on 423f173 (scratch test, not committed; log
`/scratch/cache/ms-codex32-ctors-logs/scan.log`): readable `data` lengths with HRP `ms`, threshold 0, index `s`:
`[16..=44, 63, 64]`; id `tes1` -> `InvalidChar('1')`; id `tesb` -> `InvalidChar('b')`; id `TEST` ->
`InvalidCase(Lower, 'T')`; HRP `MS` -> `InvalidCase(Upper, 's')`; threshold 1 -> `InvalidThresholdN(1)`.
Both comments carry no code block (CI runs `cargo test -p ms-codec --doc`); the two intra-doc links resolve inside
the module (`Codex32String`, `Error` are defined in mod.rs).

## 3. Spec delta: vendor header and CHANGELOG (PROPOSED texts, architect confirms)

3a. The vendor header (mod.rs:7-18) says "The ONLY substantive edits are: (1) ... (2) ... (3) ...". Two edits:

- mod.rs:8 becomes `// The ONLY substantive edits are (4) below and: (1) Zeroize/ZeroizeOnDrop + a redacting Debug`.
- After mod.rs:18 (`// alongside as src/codex32/LICENSE.`) insert:

      //
      // (4) `from_unchecksummed_string` appends a correct checksum in the input's
      // case and refuses a completed length `from_string` would refuse (bh-ms-I2,
      // design/FIX_bughunt1119_ms_codex32_ctors.md; upstream appended the raw
      // residue in lower case). It is not an encoding path: no ms-codec code calls it.

The ENCODING list on mod.rs:15-16 and `tests/codex32_vendor_parity.rs:1-17` stay as they are (neither names
`from_unchecksummed_string`; `from_seed` is unchanged).

3b. Root `CHANGELOG.md`: a new `## ms-codec [Unreleased]` heading inserted directly above `## ms-cli [Unreleased]`
(L7), with:

    ### Fixed
    - **`codex32::Codex32String::from_unchecksummed_string` returns a readable card.** It used to append the raw
      checksum residue instead of the checksum, always in lower case, so the string it returned was refused by
      `from_string` (bh-ms-I2). It now appends the checksum in the input's case, and refuses with
      `InvalidLength` a completed string of a length `from_string` would refuse. No ms-cli command used it.
    - **`codex32::Codex32String::from_seed` is documented as an unchecked builder** (bh-ms-I3): it does not
      check the share index against a zero threshold, nor the finished length. Its behaviour is unchanged.

No version bump in this FIX's PR; the entry waits under `[Unreleased]` for the next ms-codec release. The consumers
vendor ms-codec and none calls `from_unchecksummed_string`, so no pin bump is owed.

## 4. RED rows (run on 423f173, signet-only data: the all-zero 12-word test card and BIP-93 vector 5)

New test file `crates/ms-codec/tests/bh1119_ms_codex32_ctors.rs` (the builder writes it; the version that ran is
`/scratch/cache/ms-codex32-ctors-logs/bh1119_ms_codex32_ctors.rs`). Constants: `CARD =
"ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f"`, `LONG` = BIP-93 vector 5 (`MS100C8VSM32...UNDVARHRAK`, 127
characters, already at `tests/bip93_inline_vectors.rs:138`).

Command: `cargo nextest run --locked -p ms-codec --test bh1119_ms_codex32_ctors --no-fail-fast`

| Row (test fn) | Asserts | On 423f173 (observed) | After fix |
|---|---|---|---|
| `i2_lowercase_body_completes_to_card` | completing `CARD[..len-13]` == `CARD` | FAIL `left: "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqedy78kc80nx7c"` `right: "...cj9sxraq34v7f"` | PASS |
| `i2_uppercase_body_completes_to_uppercase_card` | completing upper `CARD[..len-13]` == upper `CARD` | FAIL `left: "MS10ENTRSQQQQQQQQQQQQQQQQQQQQQQQQQQQQedy78kc80nx7c"` `right: "...CJ9SXRAQ34V7F"` | PASS |
| `i2_long_body_completes_to_bip93_vector_5` | completing `LONG[..len-15]` == `LONG`, and the same in lower case | FAIL `left: "...XY06F3zchhnkx9k83ax0"` `right: "...XY06FHPV80UNDVARHRAK"` | PASS |
| `i2_completed_length_below_short_bracket_is_refused` | 30-char body (`ms10entrs` + 21 `q`) -> `Err(InvalidLength(43))` | FAIL `30-char body: Ok(Codex32String([REDACTED; 43 chars]))` | PASS |
| `i2_completed_length_between_brackets_is_refused` | 91-char body (`ms10entrs` + 82 `q`) -> `Err(InvalidLength(106))` | FAIL `91-char body: Ok(Codex32String([REDACTED; 106 chars]))` | PASS |
| `i2_controls_unchanged_refusals` (control) | mixed case -> `InvalidCase`; `b` -> `Err`; threshold 0 + index `a` -> `InvalidShareIndex(Fe::A)` | PASS | PASS |
| `i3_from_seed_unchecked_builder_contract` (control; pins T24) | `from_seed("ms",0,"test",Fe::A,16 B)` Ok and `from_string` -> `InvalidShareIndex(Fe::A)`; 45 B Ok and `from_string` -> `InvalidLength(94)`; HRP `MS`/id `TEST` -> `InvalidCase`; 16/20/24/28/32/64 B reparse | PASS | PASS |

Observed: `Summary 7 tests run: 2 passed, 5 failed` on 423f173 (log `/scratch/cache/ms-codex32-ctors-logs/red4.log`).
With the scratch fix (`scratch-fix2.diff`) the whole ms-codec suite: `Summary 220 tests run: 220 passed, 5 skipped`
(log `green2.log`), so `hashlock_kind::a_46_byte_payload_is_unconstructible` and
`negative::rule_4_share_index_not_secret_rejected` stay green (they need `from_seed` unchecked, which it stays).

I3 has no RED row: T24 makes it a documentation item, and its control row is green before and after. The
documentation itself is checked by the reviewer against section 2b; a doc-comment grep row would pin wording, not
behaviour, so none is proposed.

## 5. Tests that change

- New: `crates/ms-codec/tests/bh1119_ms_codex32_ctors.rs` (the 7 rows above, names exactly as in the table).
- Unchanged and still green: `hashlock_kind.rs`, `negative.rs`, `codex32_vendor_parity.rs`,
  `bip93_inline_vectors.rs`, every ms-cli test (none calls `from_unchecksummed_string`).
- No existing test asserts the old (wrong) suffix.

## 6. Coverage map (rule -> row)

| Rule (section) | Row |
|---|---|
| 1.3 target applied (short) | `i2_lowercase_body_completes_to_card` |
| 1.3 target applied (long) | `i2_long_body_completes_to_bip93_vector_5` |
| 1.4 suffix in input case (upper) | `i2_uppercase_body_completes_to_uppercase_card`, `i2_long_body_completes_to_bip93_vector_5` |
| 1.4 suffix in input case (lower) | `i2_lowercase_body_completes_to_card`, `i2_long_body_completes_to_bip93_vector_5` |
| 1.6 length below short bracket | `i2_completed_length_below_short_bracket_is_refused` |
| 1.6 length between brackets | `i2_completed_length_between_brackets_is_refused` |
| 1.2, 1.7 existing refusals kept | `i2_controls_unchanged_refusals` |
| 1.5 scrubbed error path | none (not observable from a test; reviewer checks the diff against 1.5) |
| 2b `from_seed` contract | `i3_from_seed_unchecked_builder_contract` |
| 2a, 3a, 3b texts | reviewer, against the PROPOSED texts |

## 7. Secret hygiene

The body passed to `from_unchecksummed_string` is secret material when it carries a seed. Today the
post-completion error (`sanity_check`) drops a zeroizing `Codex32String`; rule 1.5 keeps that true for the new
length refusal. Two pre-existing leaks in the same function are NOT changed by this FIX and are filed (section 9):
`s.reserve_exact(len)` (mod.rs:178) may reallocate and free the caller's buffer unscrubbed, and the early `?`
returns at mod.rs:186-187 drop `s` as a plain `String`. No caller exists in any repo today.

## 8. Gate

Source files this FIX changes (for build ordering against #3, #46, #47, #48): `crates/ms-codec/src/codex32/mod.rs`,
`CHANGELOG.md` (root), new `crates/ms-codec/tests/bh1119_ms_codex32_ctors.rs`. No ms-cli file.

Gate line (every CI job touching ms-codec in `.github/workflows/rust.yml`: `fmt`, `test-ms-codec`,
`clippy-ms-codec`, `test`, `clippy`, plus `docs-only`):

    PATH="$HOME/.cargo/bin:$PATH"; cargo +1.95.0 fmt --all -- --check && cargo nextest run --locked -p ms-codec --no-fail-fast && cargo test --locked -p ms-codec --doc && cargo clippy --locked -p ms-codec --all-targets -- -D warnings && cargo nextest run --locked --workspace --no-fail-fast && cargo clippy --locked --all-targets -p ms-cli -- -D warnings

Run on 423f173 without the fix (with `--build-jobs 2 -j 2` for the loaded host), exit 0: ms-codec `213 tests run:
213 passed, 5 skipped`, workspace `758 tests run: 758 passed, 23 skipped` (log
`/scratch/cache/ms-codex32-ctors-logs/gate-base.log`). After the fix ms-codec counts 220 (7 new rows).

Not covered by that line: the macOS, musl, FreeBSD, miri, fuzz-smoke, release and mlock CI legs (no ms-cli or
platform code changes; green CI on the PR head covers them).

## 9. FOLLOWUPS

Closes no existing FOLLOWUPS line (bh-ms-I2 and bh-ms-I3 were never filed there). Proposed new line (in the
report, not appended in this branch):

    - [ms-codex32-unchecksummed-input-unscrubbed] status=open grade=M kind=secret owner=none filed=2026-11-22 src=design/FIX_bughunt1119_ms_codex32_ctors.md#7 :: Codex32String::from_unchecksummed_string: reserve_exact may reallocate and free the caller's body unscrubbed, and the early ? returns drop it as a plain String; no caller in any repo today.
