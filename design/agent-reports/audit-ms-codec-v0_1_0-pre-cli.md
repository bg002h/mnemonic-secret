# Audit: `ms-codec v0.1.0` pre-`ms-cli`

**Date:** 2026-05-04
**Auditor:** feature-dev:code-architect (opus)
**Trigger:** `audit_before_extending` workflow convention — ms-cli is v0.x atop shipped ms-codec v0.1.0.

## Verdict

v0.1 surface is largely sound — the `entr`-only narrowing simplified the API enough that ms-cli can build atop it without major rework. **Three Critical-for-CLI hazards** (all CLI-side specification gaps the brainstorm MUST cover, not library bugs) and **five Important-for-CLI** items deserve explicit treatment in the SPEC. Two genuine library-side defects are worth a v0.1.1 patch but don't block ms-cli.

## Critical-for-CLI

1. **`Error::Codex32` Display leaks `{:?}` to end users.** `error.rs:69` formats `codex32::Error` with `{:?}` because upstream lacks `Display` (FOLLOWUPS `phase-1-low-2`, external). End-user-facing output for the most common failure (bad checksum/character) will look like `codex32 parse error: InvalidChecksum { checksum: "short", string: "ms10..." }`. **CLI implication:** ms-cli MUST own a `codex32::Error → human message` mapper (not lean on `Display`). The mapping table is small (~15 variants in `/tmp/codex32-extract/codex32-0.1.0/src/lib.rs:42-83`) and stable since the dep is exact-pinned `=0.1.0`. Spec a `friendly_codex32(&codex32::Error) -> String` helper in ms-cli.

2. **No emit-time language selection on the BIP-39 surface.** `tests/bip39_integration.rs` always uses `Language::English`. The README warning at `crates/ms-codec/README.md:40-42` is informational — there's nothing in ms-codec's API enforcing a language record. **CLI implication:** SPEC §6.3 makes this load-bearing for engraving correctness. ms-cli `encode --phrase` MUST require `--language` to be either (a) explicit, or (b) defaulted to `english` *with a stderr warning*. Silent English defaulting is a foot-gun the CLI must own; ms-codec deliberately punted it.

3. **`decode()` discards the input string and prefix byte.** `decode.rs:19` returns only `(Tag, Payload)`. The reserved-prefix byte (a v0.2-migration discriminator), the original string, and length aren't surfaced. **CLI implication:** for `ms decode`, the CLI will want to emit the entropy hex **plus** verifiable provenance fields (string length, BIP-39 word count derived from byte length, prefix byte for the v0.2 forward-compat story). The CLI must call `inspect()` *in addition to* `decode()`, OR build a thin wrapper. Spec which.

## Important-for-CLI

4. **Exit-code categorization is feasible but non-obvious.** `Error` variants split cleanly: `Codex32 | UnexpectedStringLength | PayloadLengthMismatch` = user-input (exit 1); `WrongHrp | ThresholdNotZero | ShareIndexNotSecret | TagInvalidAlphabet | UnknownTag | ReservedPrefixViolation` = format-violation (exit 2); `ReservedTagNotEmittedInV01` = "valid future format, refusing in v0.1" — semantically distinct, deserves its own exit code (3?) so scripts can detect "this is a v0.2 string". **CLI MUST spec the mapping table.**

5. **`inspect()` doesn't surface *why* a string would fail decode.** `inspect.rs:34` returns raw fields; the caller infers rule violations. ms-cli `inspect` will want `would_decode: bool` plus a `failure_reasons: Vec<&str>` (or a `validate_against_decode_rules() -> Vec<Error>` library helper). **Library-side ergonomic hole; CLI can paper over with a local re-validator that reuses `consts::*`.**

6. **`VALID_ENTR_LENGTHS` is `pub` in `consts` — usable for `generate`.** `consts.rs:29`. CLI's `ms generate --words 12|15|18|21|24` can map word-count → byte-length via the bijective `[16,20,24,28,32]`. **Inherits cleanly** as long as CLI imports `ms_codec::consts::VALID_ENTR_LENGTHS` rather than hardcoding.

7. **Engraving chunking has no library helper.** No grouping helper in `consts` or elsewhere. `Codex32String::parts_inner` uses `rsplit_once('1')` so the last `'1'` is the separator (BIP-93 alphabet excludes `b/i/o/1` from data, so `'1'` won't appear after the separator in v0.1) — **5-char-group chunking is safe to do post-hoc in the CLI**. Spec the chunk size (md-cli precedent: 5).

8. **Vector corpus path is dev-only.** `tests/vectors/v0.1.json` is reachable only from the test harness. ms-cli's `vectors` command will need either a `build.rs`-baked `include_str!` of the JSON, or its own copy. Spec which (md-cli's pattern is to `include_str!` from a `crates/md-codec/vectors/v0_1.json` exposed via a `pub mod vectors` — ms-codec has no such module today).

## Library-side defects (v0.1.1 patch candidates)

- **`Tag::from_raw_bytes` is `pub` but doc says "tooling-only"** (`tag.rs:18-24`). Naming doesn't enforce the convention. Either `pub(crate)` with a separate `inspect`-internal accessor, or rename to `from_raw_bytes_unchecked`. Patch in v0.1.1.
- **`Tag::as_str` returns `"<non-utf8>"` for raw-byte tags** (`tag.rs:56`), which round-trips into `error.rs:91` and `error.rs:107`'s `unwrap_or("<non-utf8>")`. Defensive but the codex32 alphabet is ASCII so this branch is unreachable for `try_new`-validated tags. Acceptable; just confirm the inspect-path doesn't accidentally emit `"<non-utf8>"` to a user.

## Inherits cleanly

- `Payload::Entr` length validation symmetric on encode and decode (`encode.rs:24`, `decode.rs:47`). No "encoder accepts what decoder rejects" footguns.
- Encoder reserved-tag symmetry (SPEC §3.5.1, `encode.rs:18-22`) — passing `Tag::try_new("seed")` to `encode()` fails at the boundary, not after producing a string.
- `#[non_exhaustive]` consistently applied to `Payload`, `PayloadKind`, `InspectReport`, `Error` — CLI's `match` arms will compile-warn on v0.2 additions.
- `consts::VALID_STR_LENGTHS` is `pub` and tested-bijective with `VALID_ENTR_LENGTHS` (`consts.rs:50-62`). CLI's argument validation can use it directly.
- `discriminate()` defense-in-depth checks (HRP, threshold, share-index) are redundant with `rust-codex32` but produce domain-typed errors — exactly what a CLI wants.
- BIP-39 round-trip is bit-exact at all 5 lengths (`bip39_integration.rs:9-92`). No alignment/padding hazards for the CLI's phrase ↔ entropy routing.

## Bottom line

Proceed with the brainstorm. The three Critical items are CLI-spec items (error-message UX, language defaulting, decode-vs-inspect routing) — they are CLI design decisions, not library blockers. The two library-side defects are v0.1.1 patches, not v0.1 ship-stoppers.
