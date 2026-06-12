# BRAINSTORM — ms-codec error rendering must not echo secret input (0.4.4)

Status: R2 **GREEN (0C/0I)** — cleared for implementation. 2026-06-12.
Reviews (persisted verbatim): error-no-echo-r0-round1-review.md (YELLOW
0C/2I — folded; `[I1]`/`[I2]` + M1–M7) and error-no-echo-r0-round2-review.md
(GREEN 0C/0I; 3 doc/discretion minors). Repo: mnemonic-secret @ master.

**Round-2 doc minors folded:** [m1] "both CHANGELOGs" = root `CHANGELOG.md`
(`## ms-codec [0.4.4]`) + `crates/ms-codec/CHANGELOG.md` (there is NO
`crates/ms-cli/CHANGELOG.md`; ms-cli is pin-only). [m2] ms-cli
`codex32_friendly.rs:44,48` independently `{:?}`-prints MismatchedHrp/Id —
provenance-bounded SAFE (HRP="ms"/id=4 chars, never the data-part); note in
the CHANGELOG that ms-cli keeps HRP/ID by design (out of this cycle's
`ms_codec::Error` scope). [m3] custom Debug may be structural (variant name
+ safe fields) or Display-delegating — implementer discretion.
FOLLOWUP: `ms-codec-error-display-echoes-input` (filed at stress-Cycle-C R0).
ms1 is SECRET-BEARING (BIP-39 entropy / BIP-32 seed / xpriv); an error that
embeds the raw input leaks secret material into any log/panic/print.

## Problem (grep-verified at HEAD)

`crates/ms-codec/src/error.rs`:
- `#[derive(Debug)]` on `Error` (error.rs:7) — so `format!("{e:?}")` shows
  every field, including the leaky ones below.
- `Error::Codex32(codex32::Error)` Display renders `{:?}` of the inner
  codex32 error (error.rs:118). codex32-0.1.0's `Error` carries the raw
  input in 3 variants: `InvalidChecksum { checksum, string: String }`
  (the FULL input), `MismatchedHrp(String, String)`, `MismatchedId(String,
  String)`. A single bit-flip of a valid ms1 share → `InvalidChecksum` →
  the whole secret rides in the error string.
- `Error::WrongHrp { got: String }` Display renders `{:?}` of `got`
  (error.rs:122). `got` is the observed HRP = everything before the last
  `'1'` (decode.rs / envelope.rs). A data-char→`'1'` mutation makes the
  "HRP" a long secret prefix → leak.

The stress-Cycle-C `ms1_no_secret_leak` fuzz target scans BOTH `Display`
and `Debug` for any ≥8-char window of the secret data-part, and currently
EXCLUDES `Codex32(_) | WrongHrp{..}` precisely because they leak. This
cycle closes the leak so that exclusion can be deleted (the oracle then
becomes the permanent regression gate).

## Goal

Neither `Display` NOR `Debug` of `ms_codec::Error` may contain any ≥8-char
contiguous window of input-derived (secret) material — for ALL reachable
inputs, across all 4 public entries (`decode`, `decode_with_correction`,
`inspect`, `combine_shares`). Structural diagnostics (error KIND, lengths,
tags, the short observed HRP) are fine and desirable.

## Constraints

- `Error::Codex32(codex32::Error)` is a PUBLIC variant; its payload type is
  API. Changing the stored type is a breaking change (the `0.X` axis is the
  breaking axis pre-1.0, so a 0.5.0 would be needed). PREFER keeping the
  type and sanitizing only the RENDERING (0.4.4 patch) if that fully closes
  the oracle. (A caller who deliberately extracts the inner error and
  `{:?}`-prints it can still leak — defense-in-depth via type change is a
  separate, breaking, later option; note it but don't gate on it.)
- Other variants' Display already echo at most a `[u8;4]` tag / single char
  / numbers — below the 8-char window. Verify; leave as-is.
- `WrongHrp.got` is genuinely useful for short legit cases ("you typed
  mk1/lnbc1 not ms1"). Preserve that diagnostic while bounding the echo.

## Proposed approach (round-1 folded — decisions locked)

1. **Custom `Debug` for `ms_codec::Error`** — remove `#[derive(Debug)]`,
   hand-impl `Debug` (structural but NON-echoing; may delegate to the
   sanitized Display). LOAD-BEARING for the toolkit [M3]: `ToolkitError`
   and `CliError` both `#[derive(Debug)]` and wrap ms-codec/codex32, so
   `format!("{toolkit_err:?}")` (panics / `expect` / `log`) transitively
   relies on ms-codec's hand-Debug being non-leaking. Custom-Debug is NOT
   a SemVer break (Debug output isn't contractual; the impl is preserved).
2. **`Codex32(e)` arm (both Display + Debug) — MANUAL variant match,
   mandatory** [M1: codex32 has NO `Display` impl, so `{:?}`→`{}` does not
   compile]. Render only the codex32 error KIND, never its String fields.
   codex32-0.1.0 has 16 variants; exactly **3 are leaky** and MUST be
   handled explicitly [M2]:
   - `InvalidChecksum { checksum, string }` → "invalid {checksum} checksum"
     (the `checksum: &'static str` "short"/"long" is safe; DROP `string`).
   - `MismatchedHrp(_, _)` → "mismatched HRP across shares" (DROP both).
   - `MismatchedId(_, _)` → "mismatched ID across shares" (DROP both).
   The other 13 carry only `&'static str` / `usize` / `char` / `Case` /
   `Fe` / `field::Error` (all SAFE, ≤1 echoed char < the 8-window) — render
   them with their numeric/char fields [OQ4: include them; matches ms-cli's
   shipped `friendly_codex32`]. Enumerate the 3 leaky explicitly (no
   generic Debug fallback) so a future codex32 bump can't silently route a
   new leaky variant through. (`MismatchedHrp/Id` are provenance-bounded
   SAFE for ms1 anyway — from `interpolate_at` on valid Codex32String,
   hrp="ms"(2)/id=4 chars — but dropped for robustness [M4].)
3. **`WrongHrp.got` — bound at CONSTRUCTION, char-counted [I1][I2] (HARD
   REQUIREMENT, not an option).** Rendering-only redaction is REJECTED:
   ms-cli (`error.rs:137-141`, message + JSON `details.got`) and the
   toolkit (`friendly.rs:95-97`) read the raw `got` field BY VALUE and
   re-echo it, bypassing any ms-codec render sanitization. Only a
   construction-time bound on the stored field fixes all three surfaces
   (ms-codec Display+Debug, ms-cli, toolkit) with zero downstream code
   change. **Store `got.chars().take(4).collect::<String>()`** — first 4
   CHARS (not bytes: `"ñ"/"é"/"😀"` are the v0.4.3 multibyte-panic cases; a
   byte slice re-panics). 4 < 8 (oracle-safe) and preserves the
   "you typed mk1/lnbc not ms1" diagnostic; the 4 echoed chars (~20 bits)
   are unrecoverable. Construction sites (re-grep exact lines at write
   time): decode.rs:~152 (the 0.4.3 char-safe site), envelope.rs:~65,
   envelope.rs:~121. (Fallback if a reviewer wants strictly-zero echoed
   secret chars: option (c) — render "non-ms HRP (N chars)" with NO echoed
   chars when the prefix exceeds a plausible HRP length; loses the hint.)

## Anti-vacuity / tests

- Unit cells in error.rs (or a new test): for a constructed
  `Codex32(InvalidChecksum{string: <50-char secret>})`,
  `MismatchedHrp(<secret>,<secret>)`, and a `WrongHrp{got: <50-char
  secret>}`, assert NEITHER `format!("{e}")` NOR `format!("{e:?}")`
  contains any 8-char window of the secret. (Red-first against HEAD: they
  currently leak.)
- Delete the `Codex32(_) | WrongHrp{..}` exclusion from the Cycle-C
  `ms1_no_secret_leak` fuzz target (mnemonic-secret `fuzz/`) so the oracle
  now guards these paths; bring-up: run it post-fix and confirm clean over
  a real budget. (The exclusion-deletion is the FOLLOWUP's "set shrinks to
  zero" close.)
- **UPDATE the leak-enshrining test [I2]:** `decode.rs:388-401`
  (`decode_with_correction_no_separator_multibyte_does_not_panic`)
  currently asserts `got == s.to_ascii_lowercase()` for `"é".repeat(25)` —
  it CODIFIES the full echo and WILL break under the cap. Change it to
  assert `got == s.chars().take(4).collect::<String>().to_ascii_lowercase()`
  (or equivalent) — i.e. the bounded value. The short-HRP tests survive
  unchanged: decode.rs:409 ("xy"), decode.rs:415 ("ñ"),
  uppercase_envelope.rs:184 ("xs"), toolkit friendly.rs:373 ("mq") — all
  ≤4-char HRPs. No ms-cli test pins the `got` value.

## Release mechanics

- ms-codec 0.4.3 → **0.4.4** (PATCH — internal rendering change, no wire/
  type change IF the API-preserving approach holds). ms-cli pin →`=0.4.4`
  (pin-only). Both CHANGELOGs. Tag `ms-codec-v0.4.4`. Publish (user-
  authorized). Toolkit pin bump 0.4.3→0.4.4 + a toolkit patch release
  (the toolkit's `friendly_ms_codec` renders these errors — confirm it
  doesn't independently re-leak, and that the bound improves its output).
- Resolve FOLLOWUP `ms-codec-error-display-echoes-input` + its toolkit
  companion.

## Resolved decisions (round-1 R0 answers, adopted)

1. **OQ1:** codex32-0.1.0 has NO `Display` impl on `Error` → `{:?}`→`{}`
   does not compile. Manual variant match is MANDATORY [M1].
2. **OQ2:** API-preserving 0.4.4 PATCH (keep `Codex32(codex32::Error)`,
   sanitize Display + custom Debug). Not a SemVer break. Residual
   "caller extracts inner codex32::Error and `{:?}`-prints it" vector =
   accepted/deferred (a sanitized-kind type-change would be a breaking
   0.5.0; not gated).
3. **OQ3:** `got` = first 4 CHARS, bounded at CONSTRUCTION (HARD req per
   [I1]; char-counted per [I2]).
4. **OQ4:** include the safe numeric/char fields of the 13 non-String
   codex32 variants; structural-only for the 3 String variants.
5. **OQ5 (downstream sweep):** with construction-bound `got` + custom
   Debug, ms-cli and the toolkit need NO code change — only pin bumps
   (ms-cli `=0.4.4`, toolkit `0.4.3`→`0.4.4` + a toolkit patch release).
   InvalidChecksum is already withheld in both user-visible paths
   (ms-cli `friendly_codex32:27`, toolkit `friendly.rs` v0.53.4).

## Release mechanics (folded)

- ms-codec 0.4.3 → **0.4.4** PATCH (internal rendering + a stored-field
  char-bound; no type/signature/wire change). ms-cli pin →`=0.4.4`
  (pin-only). Both CHANGELOGs (note the JSON `details.got` shortening
  [M5]). Tag `ms-codec-v0.4.4`. Publish (user-authorized).
- Delete the `Codex32(_) | WrongHrp{..}` exclusion from
  `fuzz/fuzz_targets/ms1_no_secret_leak.rs` (`is_known_echo` lines 70-81,
  call site 109) [M6]; bring-up the oracle GREEN over a real budget.
- Toolkit: pin 0.4.3→0.4.4 + a patch release (pin-only, no toolkit code
  change once `got` is construction-bounded); verify the toolkit suite
  green at the bumped dep.
- Resolve FOLLOWUP `ms-codec-error-display-echoes-input` + its toolkit
  companion.
