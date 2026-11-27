# Independent review: ms exit-code implementation (2026-11-26)

Reviewed: `git diff 844888b^..HEAD` on `fx/mnemonic-secret` (844888b closed-stdout
fix; ff7cb4e, d54e140 exit-code spec). Against `SPEC_constellation_exit_codes.md`
rev 5 §2-§5, §7c. Reviewer did not write the code; no source edited.

**Verdict: NOT GREEN — 0 Critical, 1 Important, 4 Minor, 5 Nit.**

What I ran (debug profile, `-p ms-cli` only): built `ms`; ran
`--test exit_codes_constellation --test output_errors` (15 + 13 pass); ran the
ignored `bughunt::bughunt_ms_09_mnem_unknown_language_unhandled` (fails, below);
hand probes with a Python BIP-93 encoder for crafted strings (unknown language
byte, unknown tag, corrected non-card), closed / full stdout, empty stdin per
verb, R2 orderings.

Checked and found correct: the §3 numbers and `Bits`; the R11 legend (word for
word the spec's text, whitespace-normalised, and it ends root `--help`); R8
(`>/dev/full` and `>&-` give 32, repair 52, failing `--json` 34, 64/101 kept,
EPIPE keeps the verb's code); R2 argv guard before clap (secret + bad flag = 8);
decision 3 (empty stdin is 2 on decode, verify, inspect, derive, repair,
hashlock, encode, combine); decision 8 (`inspect` prints the full report and
exits 2); R5 (`verify` no longer prints "OK: valid future format"); verify
mismatch 1; two stdin readers 64; split K/N 64 and checked before the secret is
read; `--out` refusals 8; digest-shaped phrase 8; unset `@env:` 32; R14 (JSON
`exit_code` = `CliError::exit_code()`); decision 6's 22 path (corrected
non-card printed with warning, `--out` not written, clean non-card 2 with
nothing printed). No secret newly echoed; no `--out` written where it was not
before.

---

## Important

### I1. A Mnem card with an unknown language byte exits 101 ("internal error, report it"); §3 test 6 gives 2. The R13 codec table omits the variant, so it passes.

- **Where:** `crates/ms-cli/src/error.rs` `From<ms_codec::Error>`: no arm for
  `ms_codec::Error::MnemUnknownLanguage(u8)` (ms-codec `error.rs:29`, raised by
  `payload.rs:96`), so it falls to the `other =>` wildcard, now
  `CliError::Internal` (101). The table `from_ms_codec_every_variant_has_its_class`
  (`error.rs` ~426) lists 19 variants and omits this one, so its name
  ("every variant") is a false PASS.
- **Measured** (string `ms10entrsqg9qqq…` = prefix 0x02, language 10, 16 zero
  bytes, valid checksum):
  `ms decode -` → `error: unhandled ms_codec::Error variant: Error("unknown mnem wordlist-language code: 10")`, **exit 101**;
  same for `verify -` and `repair --ms1 -` (101); `decode --json` → envelope
  `kind: "Internal", exit_code: 101`. Before this diff it was 1 (BadInput), and
  FOLLOWUP `ms-cli-mnem-unknown-language-catch-all` already prescribes the arm
  (FormatViolation, exit 2). `inspect` of the same string correctly exits 2
  (it has its own analysis), so `inspect` and `decode` now disagree by class.
- **Failure scenario:** a card from a newer or foreign encoder tells the user
  and any GUI (R12 keys on 101) that `ms` has a bug, instead of "this input is
  invalid, fix or replace it". The CHANGELOG bullet "101 (was 1): … an
  undispatched `ms_codec::Error` variant" ships this as intended behaviour.
  `combine` and `derive` reach the same arm.
- **Fix:** add the arm per the FOLLOWUP (`FormatViolation { underlying_kind:
  "MnemUnknownLanguage", … }` → 2, details `{"got": code}`), add the row to
  `from_ms_codec_every_variant_has_its_class`, un-ignore
  `bughunt_ms_09_mnem_unknown_language_unhandled` (its assertion is already
  exit 2), close the FOLLOWUP. To keep the table honest for the next variant:
  ms-codec is in this workspace, so a test can compare the table's variant
  names against the `pub enum Error` variants parsed from
  `crates/ms-codec/src/error.rs` (or ms-codec can expose a `#[cfg(test)]`/doc-hidden
  list), so a new codec variant fails a test instead of silently becoming 101.

## Minor

### M1. The R13 `CliError` table cannot catch a new variant without a class.

- **Where:** `error.rs` `exit_code_table_per_variant`. The exhaustive `match e`
  inside the loop forces a new variant into the match arm, but not into `rows`;
  `assert_eq!(rows.len(), 14)` only fails if someone adds a row. A new variant
  added to the match with no row passes.
- **Fix:** make the exhaustive match produce the expected class per variant
  (`let want = match e { CliError::BadInput(_) => INVALID, … }`) over one
  constructed instance per variant, or build `rows` from a function whose body
  is an exhaustive match returning `(instance, class)`.

### M2. `repair --json` in the 22 case says `"verdict":"candidate"` and nothing about "not a card".

- **Where:** `cmd/repair.rs` (`emit_json` path when `not_a_card.is_some()`).
  Measured: exit 22, stdout
  `{"…","verdict":"candidate","corrected_chunks":[…],…}`; the "NOT an ms1 card"
  line is stderr only.
- **Failure scenario:** a JSON consumer reads one stream (SPEC §6.3) and sees an
  ordinary candidate; R5 ("a message must agree with its code") is met only by
  the exit status. R12 says a GUI decodes the bits, so not data loss, but the
  envelope disagrees with its own status.
- **Fix:** either a distinct verdict (e.g. `"not-a-card"`) or an added field
  (`"decodes": false` plus the reason kind) — a JSON shape change, so coordinate
  with the cross-CLI repair envelope parity and the GUI.

### M3. R2 order: three usage-class checks run after input is read, or after a refusal.

- `passphrase_input.rs:514` invalid `@env:` NAME (64, implementer's Q4) is
  evaluated only after the ms1 is read and decoded: `printf garbage | ms derive
  - --passphrase @env:bad-name` → 2; with a valid ms1 → 64. R2: "A usage error
  exits 64 before any input is read." Fix: validate the `@env:` name form at
  argument resolution, before `read_input`.
- `cmd/hashlock.rs` `run`: `refuse_stdout_alias` (8) runs before `pick_source`
  (64), so `ms hashlock --hex X --random --out F > F` is 8, not 64. Fix: move
  `pick_source` and the `--emit-record` gates before the alias check (both read
  nothing).
- `cmd/derive.rs:546` `--account 2147483648` (u32 accepted by clap, invalid
  hardened index) → `BadInput`, **2**, after the secret is read. An account
  index is a setting (decision 2: "counts, ranges"), so 64, checked up front.
  Fix: `value_parser = clap::value_parser!(u32).range(..0x8000_0000)` (clap's
  error is then 64 with nothing read).

### M4. R8 on Windows: a closed stdout still exits 0 with the output lost.

- **Where:** `sink.rs` `read_only` (`#[cfg(not(unix))] false`) and
  `closed_fds` (unix-only list). The comment itself notes std turns an invalid
  handle into a success, which is exactly the closed-stdout case. Also any unix
  not in the list (illumos, solaris, aix, haiku) keeps the pre-fix behaviour.
- **Fix:** on Windows, at the first stdout write check
  `GetStdHandle(STD_OUTPUT_HANDLE)` for null / `INVALID_HANDLE_VALUE` and record
  EBADF-equivalent; or state the platform gap in the CHANGELOG and SPEC §6 so
  R8's claim is not made for Windows.

## Nit

- N1. Stale codes: `src/argv_guard.rs:265` ("Refuse, with this message. Exit 1."
  — it is 8); `crates/ms-codec/README.md:76` ("as `ms repair` does (exit 4)" —
  20); `design/FOLLOWUPS.md:917-921, :1249, :1372` (exit 1 / "Ok return becomes
  1" — now 101→2 after I1, and +32); `design/SPEC_ms_derive.md:24` still says
  `BadInput → exit 1` and the bip32 wraps are BadInput (now Internal 101).
- N2. `correct_only` (`cmd/repair.rs`) re-implements the codec's BCH steps
  (alphabet, `hrp_expand`, `MS_REGULAR_CONST`, re-verify). It matches
  `decode_with_correction` today (I compared it line by line with
  `ms-codec/src/decode.rs:263-330`), but it will drift silently if the codec's
  framing changes. Add a test that, for a set of damaged valid cards,
  `correct_only(s)` equals `decode_with_correction(s).2`, and file a FOLLOWUP
  for an ms-codec API that returns the corrections on post-correction decode
  failure (next ms-codec minor).
- N3. In the 22 path the stderr reason uses `e.message()`; for I1's variant it
  reads "unhandled ms_codec::Error variant…" (goes away with I1's fix).
- N4. The `--out … was not written` line in the 22 case prints before the
  report and the UNVERIFIED line; printing it last (after "NOT an ms1 card")
  would read in cause-then-consequence order.
- N5. CHANGELOG "Fixed" entry for the closed stdout says it "exits 1 … (32 since
  the exit-code change above)": fine, but the earlier bh-ms-I1 entry's "(exit
  64)" for `--out` = stdout is now 8; add "(8 since …)" for symmetry.

---

## Closed-stdout probe (844888b)

Correct as far as can be tested here (Linux):
- Ordering: `.init_array` constructors run before `lang_start`, so the probe
  sees fd 1 closed before std's `sanitize_standard_fds` plugs it. Measured
  `ms vectors >&-`, `decode >&-`, `--help >&-`, `--version >&-` → 32 with the
  `Bad file descriptor` line; `encode --out F >&-` → 0 and F holds only the card
  (`output_errors::closed_stdout_with_out_file_succeeds_and_nothing_goes_astray`).
- Plugging: `open` returns the lowest free fd, and iterating 0,1,2 in order
  guarantees each closed fd gets its own number; a stray `n != fd` is closed.
- Unsafe: only `fcntl`, `open` of a C-string literal, `close`, an errno read and
  a relaxed atomic store; no allocation or std runtime use before init. Sound.
  The `extern "C" fn()` signature is fine for glibc's `(argc, argv, envp)` call.
- Not verified here: macOS (`__DATA,__mod_init_func`) and the BSDs; the pattern
  matches the `ctor` crate's. Windows: M4.

## Implementer's spec questions

1. **22 via CLI-side BCH; `--out` not written.** Agree. Decision 6 asks that the
   corrected string be shown to "a person repairing by hand"; filing a non-card
   for an engraver serves no one, and R1 lets the verb report only what it
   printed. Re-running BCH avoids a crates.io bump; see N2 for the drift guard.
   The JSON gap is M2.
2. **`inspect` of a share that would not combine → 2.** Agree: the report's
   "would NOT combine" is the share's form of "would not decode" (R5), and §3
   test 6 ("wrong for this verb, or insufficient for it") classifies it 2. A
   combinable share at 0 is right.
3. **`--in /dev/stdin` + `--passphrase -` → 64 though detected from fds.** Agree:
   comparing fd identities reads no input data; it is the command line naming
   one stream twice, the spec's "`ms` reading two inputs from stdin | 64" row.
4. **`@env:` name 64 / unset 32 / non-UTF-8 2.** Classes agree with §3 (and the
   worked-table row for unset). The order does not (M3).
5. **New JSON `kind`s; existing-`--out` kind `Usage`→`Refused`.** Acceptable:
   R14 governs only `exit_code`. It is a JSON contract change: flag it to the
   mnemonic-gui and toolkit-manual agents (and the GUI manual) in the same
   lockstep as the codes.
6. **Text-mode reserved tag on decode/derive/repair already printed the error.**
   Agree; only `verify` changed, as measured
   (`future_format_names_the_problem` covers all five verbs).
