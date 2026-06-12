# Changelog — ms-codec

All notable changes to the `ms-codec` crate are documented here. The format
follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.4] — 2026-06-12

**PATCH (SECURITY) — `ms_codec::Error` no longer echoes secret input in `Display` or `Debug`.**

ms1 strings are secret-bearing (BIP-39 entropy / BIP-32 seed / xpriv). Three error-rendering paths embedded the raw input — leaking it into any log / panic / `{:?}` — and are now closed:

- `Error::Codex32(_)`: the `Display`/`Debug` arm is a manual variant match. codex32-0.1.0's three input-bearing variants — `InvalidChecksum { string }` (the FULL input), `MismatchedHrp(String, String)`, `MismatchedId(String, String)` — are intercepted explicitly and rendered structurally only (`"invalid {short|long} checksum (input withheld)"` / `"mismatched HRP across shares"` / `"mismatched ID across shares"`); a bit-flip of a valid share that produced `InvalidChecksum` previously echoed the whole secret data-part. The other 13 codex32 variants carry only safe fields (`&'static str` / `usize` / `char` / `Case` / `Fe` / `field::Error`) and are rendered via `{:?}` on the inner error AFTER the 3 leaky arms are peeled off (so a future codex32 bump can't silently route a new leaky variant through a fallback).
- `Error::WrongHrp { got }`: `got` is now capped to the first 4 chars **at construction** (char-counted, multibyte-safe) at all three build sites (`decode.rs`, `envelope.rs` ×2). A data-char→`'1'` mutation could otherwise stretch the observed "HRP" into a long secret prefix. Bounding at construction (not rendering) means downstream re-echoers (ms-cli `details.got`, the toolkit friendly-mapper) inherit the bound with no code change. **JSON note:** the ms-cli error-envelope `details.got` now shows at most 4 chars.
- `#[derive(Debug)]` on `Error` is replaced by a hand-rolled `Debug` delegating to the sanitized `Display` (a derived `Debug` would dump the leaky fields). Load-bearing for downstream `#[derive(Debug)]` wrappers whose `{:?}` transitively renders this type. Not a SemVer break (the impl is preserved; output isn't contractual).

The `ms1_no_secret_leak` fuzz oracle's `Codex32(_) | WrongHrp{..}` exclusion is deleted — it now scans every variant and is the permanent regression gate (90s bring-up clean). 5 red-first leak cells in `error.rs`; the `decode.rs` enshrining test updated to the 4-char cap. ms-cli's `friendly_codex32` keeps HRP/ID by design (provenance-bounded, never the data-part). No API/wire/signature change. ms-cli exact pin → `=0.4.4` (ms-cli version unchanged). Resolves `ms-codec-error-display-echoes-input`. Brainstorm + 2 R0 rounds: `design/BRAINSTORM_error_display_no_echo.md`, `design/agent-reports/error-no-echo-r0-round{1,2}-review.md`. **crates.io publish + toolkit pin bump pending user authorization.**

## [0.4.3] — 2026-06-12

**PATCH — `decode_with_correction` no longer panics on a non-`ms1` input with no `'1'` separator (char-boundary fix).**

- `parse_ms1_symbols` sliced `lower[..len-1]` when reporting the observed HRP for a non-`ms1` string. With no `'1'` separator the index `len-1` can land inside a multi-byte char, panicking ("byte index N is not a char boundary"). Reproducer: a single `0xaa` byte (→ the 3-byte U+FFFD via `String::from_utf8_lossy`). The slice is now `lower[..rfind('1')]` (`'1'` is ASCII, always a char boundary), with the whole string as the observed HRP when there is no separator. `decode` and `inspect` were never affected (length-gated / codex32-validated first); only `decode_with_correction` reached the raw slice — so `ms repair` / the indel oracle inherited the panic.
- Leak-neutral: the `WrongHrp.got` echo vector is the unchanged WITH-`'1'` path (byte-identical); bounding that echo is the separate `ms-codec-error-display-echoes-input` FOLLOWUP. No API/wire change.
- Found by stress-Cycle-C fuzzing (`fuzz/fuzz_targets/ms1_decode.rs`), which re-finds it instantly and is now the regression gate. Resolves `decode-with-correction-panics-on-non-char-boundary-hrp-slice`. 2 regression cells in `decode.rs`. Mini-R0 GREEN (`design/agent-reports/decode-char-boundary-fix-mini-r0-round1-review.md`). ms-cli exact pin → `=0.4.3` (ms-cli version unchanged).

## [0.4.2] — 2026-06-10

**PATCH — accept all-uppercase ms1 per BIP-173; combine secret-leak guard restored.**

- Wire extraction canonicalizes (lowercases) the owned string copy after codex32 validation — uppercase cards decode/inspect/repair/combine; mixed-case within one string still rejects (`InvalidCase`).
- SECURITY: uniform-uppercase secret-at-`S` no longer bypasses `SecretShareSuppliedToCombine` (raw `b's'` compare missed `b'S'`; in a same-id uppercase set the interpolation short-circuit returned the secret).
- Mixed-case SETS combine via per-share re-canonicalization before interpolation; recovered output lowercase. `WrongHrp.got` reports the canonicalized form.

## [0.4.1] — 2026-06-10

**SemVer-PATCH — `combine_shares` rejects (no longer aborts on) a non-standard-length Entr share set.**

### Fixed

- `dispatch_payload`'s `Entr` (`0x00`) arm now calls `validate()` — parity with the `Mnem` arm and the function's own doc contract. Previously a **valid-checksum but non-standard-length** Entr share set (entropy length ∉ {16,20,24,28,32}) recovered via `combine_shares` returned an *unvalidated* `Payload::Entr`; the downstream `ms combine --to phrase` / `ms decode` then hit `bip39::Mnemonic::from_entropy_in(...).expect(...)` and **panicked** (abort, exit 101). The Entr arm now returns `Error::PayloadLengthMismatch` for all `dispatch_payload` callers (single-string `discriminate` + `combine_shares`), so the CLI surfaces a clean error and the `.expect` invariants in `ms-cli combine.rs` / `decode.rs` become true. The encode path was never affected (it validates up front). No new error variant; no API/wire change. Resolves audit-2026-06-10 finding `combine-no-length-validation-panic` (I9).

## [0.4.0] — 2026-06-03

**SemVer-MINOR — K-of-N codex32 Shamir shares.** Split an `entr` or `mnem` secret
into N shares, any K of which recombine. v0.1/mnem single-strings stay
byte-identical and forward-readable.

### Added

- **`Threshold`** (`ZERO` const + `new(2..=9)`), **`encode_shares(tag, threshold, n, &Payload) -> Vec<String>`** (derives all N shares internally via `getrandom`; `ZERO`/`n=1` is byte-identical to `encode`), **`combine_shares(&[String]) -> (Tag, Payload)`** (recovers via `interpolate_at(Fe::S)`; works for entr AND mnem — language survives the split).
- Shares key on the codex32 **threshold field** (`k`) + per-share **index** + group by `id` (BIP-93 native); the secret-at-S is never distributed. `0x01` stays unallocated (the prefix byte remains the payload-kind discriminator).
- `RESERVED_ID_BLOCKLIST` (anti-collision for random share-set ids; retains `mnem`). New errors `InvalidShareCount`, `InvalidThreshold`, `IsShareNotSingleString`, `SecretShareSuppliedToCombine`; codex32 share errors surface via `Error::Codex32`.

### Changed

- `decode` of a threshold∈2..9 string returns `IsShareNotSingleString` (was the v0.1 `ThresholdNotZero` hard-reject) — routes the user to recombination. The internal `[prefix]||payload` assembly is factored into `payload_wire_bytes()` (shared by `package`/`encode_shares`); `package` byte-identical. §5/MIGRATION.md migration contract amended (threshold-field dispatch).

## [0.3.0] — 2026-06-01

**SemVer-MINOR — new `mnem` payload kind: BIP-39 wordlist language on the wire.**
Resolves the §6.3 non-English-seed footgun (a non-English mnemonic could only be
backed up as raw `entr` entropy, losing which wordlist regenerates it).

### Added

- **`Payload::Mnem { language: u8, entropy: Vec<u8> }`** — a second payload kind
  behind a new `0x02` prefix byte. Byte-aligned layout `[0x02][language][entropy]`
  (the language byte joins the existing reserved-prefix slot; no bit-packing).
  `language` indexes the new `MNEM_LANGUAGE_NAMES` table (10 BIP-39 wordlists,
  English = 0).
- New consts `MNEM_PREFIX = 0x02`, `VALID_MNEM_STR_LENGTHS = [51, 58, 64, 70, 77]`,
  `MNEM_LANGUAGE_NAMES`. New error variant `MnemUnknownLanguage(u8)`.
- `InspectReport` gains `kind: InspectKind` (`Entr`/`Mnem`/`Unknown`) +
  `language: Option<u8>`, classified from the prefix byte (both `#[non_exhaustive]`).

### Changed

- `package`/`discriminate` now carry the typed `Payload` across the envelope seam
  (was a raw byte vector), so the language byte survives encode→decode.
- The decode length-gate binds string-length ↔ payload-kind: `entr` ⟺
  `{50,56,62,69,75}`, `mnem` ⟺ `{51,58,64,70,77}` — a length carrying the wrong
  kind is rejected (`UnexpectedStringLength` / `PayloadLengthMismatch`).
- `mnem` removed from `RESERVED_NOT_EMITTED_V01` (it is now an emitted kind).

The v0.1 `entr` (`0x00`) path is **byte-identical** — the SHA-pinned v0.1 vector
corpus passes unchanged. `decode_with_correction` (BCH) works for all five `mnem`
string lengths (guarded against the documented length-divergence bug class).

## [0.2.1] — 2026-05-29

### Fixed

- **`decode_with_correction` now error-corrects all entropy lengths, not just
  16-byte (12-word) seeds.** The hand-rolled BCH path used a wrong
  `POLYMOD_INIT` (`0x23181b3`) and an empirically-lifted `MS_REGULAR_CONST`
  (`0x962958058f2c192a`) calibrated to a single 12-word vector, so `polymod_run`
  was length-variant for valid codewords and `decode_with_correction` returned
  `TooManyErrors` on CLEAN 20/24/28/32-byte ms1 strings. Corrected to the
  standard codex32 short-code start state (`POLYMOD_INIT = 1`) and the true
  "SECRETSHARE32" target (`MS_REGULAR_CONST = 0x10ce0795c2fd1e62a`). The
  generator and Berlekamp-Massey/Chien/Forney decoder were already correct and
  are unchanged. Downstream impact: the toolkit's `ms repair`, `repair
  --max-indel`, and `Ms1IndelOracle` now work for 15/18/21/24-word seeds.
  Root cause + evidence: `design/BUG_decode_with_correction_length_divergence.md`.

### Added

- `tests/bch_all_lengths.rs` — all-five-length BCH regression suite (the
  constant-derivation + single-target-residue gate that would have caught the
  bug; clean-passthrough; 1–4-error correction with position checks; the 5–8-
  error miscorrection sweep; and the indel reject-contract). Replaces the prior
  12-word-only test monoculture that hid the defect.

## [0.1.1] — 2026-05-07

BIP test vector audit close-out (Phase 10 of the v0.7.1 audit cycle). No
wire-format changes; pure test-coverage extension.

### Added

- 4 new entries in `tests/vectors/v0.1.json` (custom corpus grows 2 → 6):
  - 15-word all-zero entropy (BIP-39 `[0; 20]`).
  - 18-word all-zero entropy (BIP-39 `[0; 24]`).
  - 21-word all-zero entropy (BIP-39 `[0; 28]`).
  - 15-word non-zero entropy (`0123456789abcdef0123456789abcdef01234567`)
    — catches entropy-bit-ordering regressions zero-entropy vectors miss.
- `tests/bip93_cross_format.rs` — 2 new tests pinning BIP-93 §Test Vector
  93.4 (256-bit `leet`) cross-format conformance:
  - Payload extraction via upstream `rust-codex32` is byte-stable.
  - Re-encoding 93.4's 32-byte payload as ms-codec entr round-trips and
    the resulting ms1 string is parseable by upstream codex32 (proves
    ms-codec is a proper sub-format of BIP-93 codex32 at the byte level
    for the `entr` length bucket).
  - BIP-93 spec: <https://github.com/bitcoin/bips/blob/master/bip-0093.mediawiki>.

### Internal

- Audit matrix `design/agent-reports/v0_7_1-bip-test-vector-audit-matrix.md`
  updated: BIP-93 §93.4 row flipped MISSING → COVERED; custom-corpus row
  flipped 2 → 6 entries; AMBIGUOUS Discovery #2 (non-zero entropy missing)
  marked RESOLVED.
- Workspace-internal: `ms-cli` ms-codec dep pin bumped `=0.1.0` → `=0.1.1`,
  ms-cli's mirrored `vectors/v0.1.json` extended to match (parity test
  `vectors_corpus_parity_with_ms_codec` still passes).

## [0.1.0] — 2026-05-03

Initial public release of `ms-codec`. See `design/SPEC_ms_v0_1.md` for the
full wire-format specification.
