# R0 Review — ms-codec error no-echo fix (round 1)

Reviewer: Fable 5 architect agent (aa4e557bdc294b652), 2026-06-12.
Target: design/BRAINSTORM_error_display_no_echo.md @ mnemonic-secret master (post-0.4.3).
Persisted verbatim per CLAUDE.md convention.

## Verdict: YELLOW

The leak is real, the API-preserving 0.4.4 approach is sound, and the core direction is correct. Two Important defects: (1) the WrongHrp fix MUST bound `got` at CONSTRUCTION (char-counted), not rendering-only, because ms-cli and the toolkit read the raw `got`/inner-codex32 *by value* and re-echo it; (2) an existing ms-codec test enshrines the full no-separator echo and will break — and the cap must be char-boundary-safe (else it re-introduces the v0.4.3 panic). Plus OQ1's premise is empirically false (codex32 has no Display). Fold and re-dispatch.

## Critical
- none

## Important
- **[I1] WrongHrp fix MUST bound `got` at CONSTRUCTION, not rendering-only — otherwise the downstream re-renders still leak (the ms-codec fix is necessary-but-insufficient).** Two downstream sites read raw `got` by value and re-echo, bypassing any ms-codec Display/Debug sanitization: ms-cli `crates/ms-cli/src/error.rs:137-141` (`format!("wrong HRP: got {:?}...", got)` + JSON `details.got`); toolkit `crates/mnemonic-toolkit/src/friendly.rs:95-97` (`format!("ms1 wrong HRP: got {:?}...", got)`). Only a construction-time bound on the stored `got` field fixes all three surfaces at once without coordinated downstream edits. Make construction-bounding a HARD requirement; reject rendering-only.
- **[I2] An existing ms-codec test ENSHRINES the full no-separator `got` echo and WILL break — "tests survive the cap" is wrong.** `crates/ms-codec/src/decode.rs:388-401` (`decode_with_correction_no_separator_multibyte_does_not_panic`) asserts `assert_eq!(got, s.to_ascii_lowercase())` for `"é".repeat(25)` (50-byte run). Must update to assert the bounded value. The cap must be char-counted (`.chars().take(N)`), NOT a byte slice — `"ñ"/"é"/"😀"` are the v0.4.3 multibyte-panic regression cases; a byte slice re-introduces the panic.

## Minor
- **[M1] OQ1 premise FALSE — no `{:?}`→`{}` simplification.** `codex32-0.1.0::Error` is `#[derive(Debug)]` only; NO `impl Display for Error`, no `std::error::Error`. The Codex32 arm fix MUST be a manual `match`.
- **[M2] codex32 Error = 16 variants; the brainstorm's 3 leaky (InvalidChecksum.string, MismatchedHrp, MismatchedId) is COMPLETE.** `field::Error` (ExtraChar(char)/NotAByte/InvalidByte(u8)) is SAFE. Enumerate the leaky 3 explicitly; `_ =>` wildcard acceptable for the safe ones, but explicit-leaky prevents a future codex32 bump silently routing a new leaky variant through a Debug fallback.
- **[M3] Keep custom Debug (not just Display) — it's load-bearing for the toolkit.** `ToolkitError` `#[derive(Debug)]` wraps `MsCodec(ms_codec::Error)`; `CliError` `#[derive(Debug)]` wraps `Codex32(codex32::Error)`. `format!("{toolkit_err:?}")` (panics/expect/log) transitively relies on ms-codec's hand-Debug being non-leaking. toolkit `friendly_ms_codec` already withholds InvalidChecksum (v0.53.4, verified). The `CliError::Codex32(c)` Debug (inner codex32 directly) is NOT covered by the ms-codec fix → accepted/deferred (ms-cli `friendly_codex32` drops the string in user-visible output; only a deliberate `{:?}` of CliError leaks).
- **[M4] MismatchedHrp/MismatchedId echo HRP/ID (`{:?}`) but are provenance-bounded SAFE for ms1** — from `interpolate_at` on already-valid Codex32String → hrp="ms"(2), id=4 chars → < 8. State the provenance bound; no change.
- **[M5] JSON `details.got` shortens under the cap (no test break — `json_error_envelope_per_kind.rs:74` asserts only kind/exit_code; WrongHrp case uses 2-char "mq").** Note the shortening in the CHANGELOG for JSON consumers.
- **[M6] Fuzz exclusion deletion is exact: `fuzz/fuzz_targets/ms1_no_secret_leak.rs` `is_known_echo` lines 70-81 (`Codex32(_) | WrongHrp{..}`), call site line 109, WINDOW=8 line 47.** Delete both arms; bring-up the oracle GREEN over a real budget post-fix.
- **[M7] `From<codex32::Error>` + `source()`→None unaffected (codex32 isn't std::error::Error; chain stops at ms-codec). No source()-walk leak.**

## Answers to open questions
**OQ1 — codex32 Display echo?** Premise FALSE — codex32::Error has NO Display impl (won't compile with `{}`). Manual variant match mandatory.
**OQ2 — API-preserving 0.4.4 sufficient & non-breaking?** Yes. Keep `Codex32(codex32::Error)`, sanitize Display + hand-roll Debug. Removing `#[derive(Debug)]` + hand-impl is NOT a SemVer break (output isn't contractual; presence is, and is preserved). True PATCH. Caveat: sufficient for ms-codec's own rendering only; downstream forces construction-bound (I1); residual extract-inner-and-{:?} vector deferred (would be breaking 0.5.0 type-change).
**OQ3 — WrongHrp.got cap.** Option (a), construction-time, char-counted, bound = 4. `got.chars().take(4).collect()` at decode.rs:158-162, envelope.rs:65, envelope.rs:121. < 8 mandatory (oracle); char-counted mandatory (I2 multibyte panic); construction-time fixes ms-cli+toolkit free. Tradeoff: 4 echoed chars (~20 bits, unrecoverable) acceptable for the diagnostic value ("you typed mk1/lnbc not ms1"); fall back to option (c) "non-ms HRP, N chars" with zero echoed chars if any reviewer wants strictly-zero. Update decode.rs:388-401 (I2); decode.rs:409 "xy"/:415 "ñ"/uppercase_envelope.rs:184 "xs"/toolkit friendly.rs:373 "mq" survive.
**OQ4 — single-value codex32 variants in rendered kind?** Include numeric/char fields (≤1 char, < 8 window, useful transcription hints) — matches ms-cli's shipped `friendly_codex32`. Only the 3 String variants get structural-only.
**OQ5 / downstream sweep:** ms-cli WrongHrp (error.rs:139-140) + toolkit WrongHrp (friendly.rs:96) fixed IFF construction-bounded. InvalidChecksum already dropped in both user-visible paths. Catch-all `{:?}` of whole error safe after hand-Debug. NO toolkit-side automated leak oracle → another reason to bound at construction. Net: with construction-bound + custom-Debug, ms-cli/toolkit need NO code change — just pin bumps (ms-cli `=0.4.4`, toolkit `0.4.4` + patch release). Verify toolkit suite green at bumped dep.

**Release:** 0.4.4 PATCH. ms-cli pin `=0.4.4`. Toolkit pin 0.4.3→0.4.4 + patch release (pin-only). Both CHANGELOGs + JSON-shorten note. Tag `ms-codec-v0.4.4`, publish (user-auth). Resolves `ms-codec-error-display-echoes-input` (the WrongHrp/Codex32-Display leak; the InvalidChecksum-at-toolkit one was already closed v0.53.4).

## Evidence log
```
LEAK confirmed (scratch, deleted): bit-flip valid ms1 → decode → Codex32(InvalidChecksum{string:"ms10entrs...47-char"}); Display+Debug both 47-char window, LEAK true/true
WrongHrp via decode_with_correction: got.len()=45 (qpzry9x8gf2tvdw... no-/spurious-sep); Display+Debug 45-char window LEAK true/true
codex32-0.1.0: NO impl Display for Error (only derive Debug, lib.rs:41; no std::error::Error); only Display impls are Fe, Codex32String, and a private bin struct
codex32 Error = 16 variants; LEAKY=3 (InvalidChecksum.string, MismatchedHrp, MismatchedId); field::Error SAFE
downstream: ms-cli error.rs:137-141 WrongHrp echo (msg+json); toolkit friendly.rs:95-97 WrongHrp echo; InvalidChecksum withheld both (v0.53.4 / friendly_codex32:27); ToolkitError+CliError derive Debug → transitive (M3)
enshrining test: decode.rs:388-401 assert_eq!(got, full lowercased) for "é".repeat(25) → WILL BREAK; short-HRP tests survive
fuzz exclusion: ms1_no_secret_leak.rs is_known_echo 70-81, call 109, WINDOW=8 line 47
tree left as found
```

To GREEN: fold I1 (construction-time char-bounded got, hard requirement) + I2 (update decode.rs:388-401, char-safe cap), correct OQ1 premise (M1), re-dispatch.
