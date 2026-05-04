# Phase 2+3 (Cmd Modules + Root) — Opus Review r1

**Date:** 2026-05-04
**Reviewer:** feature-dev:code-reviewer (opus)
**Commit reviewed:** `874ea21` ("feat(ms-cli): Phase 2+3 combined — cmd modules + main.rs + vectors corpus")
**Files:** `crates/ms-cli/src/main.rs`, `crates/ms-cli/src/cmd/{mod,encode,decode,inspect,verify,vectors}.rs`, `crates/ms-cli/src/parse.rs` (+`read_phrase_input`/`normalize_phrase`), `crates/ms-cli/vectors/v0.1.json`
**Tests:** 28 unit tests pass; cargo build / clippy --all-targets -D warnings / fmt --check all clean. Smoke tests confirm `--help` + canonical `encode --phrase` flow.

## Verdict

**Proceed to Phase 4.** Zero critical, zero important findings. Phase 2+3 implementation faithfully realizes SPEC §2.1–§2.6, §5, §6, §6.1.1. The locked validation order in verify, the ascending-rule order in inspect's analyze(), the FutureFormat dual-output coordination between cmd/verify.rs and main.rs::emit_error, and the read_phrase_input/read_input split are all correct. clap derive surface matches §2.6 verbatim. Three new parse.rs tests cover the read-input distinction. 28 unit tests with all gates clean is sufficient for handoff to integration tests.

## Critical findings

None.

## Important findings

None.

## Low / Nit (do not block; defer to FOLLOWUPS at controller's discretion)

1. `cmd/encode.rs:60` — `read_input(Some(hex_arg))?` strip-whitespaces hex. SPEC §2.1 doesn't explicitly mandate whitespace tolerance on `--hex`, but it's a friendly behavior consistent with §3.2 spirit.
2. `cmd/verify.rs:106-123` — `emit_future_format` doc-comment is two paragraphs; per repo's "terse code" memo, condense to a 2-line summary.
3. `cmd/verify.rs:125` — `emit_round_trip_ok(_mnemonic: &Mnemonic)` parameter has leading underscore but IS used; drop the underscore OR pass `word_count: usize` directly from caller.
4. `cmd/decode.rs:50` — wildcard arm `_ => unreachable!(...)` is correct for `#[non_exhaustive]` Payload but slightly wider than `Ok((_, _)) =>` style used in verify.rs:56. Cosmetic.
5. `cmd/encode.rs:81-104` — `parse_hex_entropy` has explicit `len() % 2 != 0` check AND maps `OddLength` from hex crate. First branch is defensive but slightly noisy.
6. `cmd/inspect.rs:65-77` — comment says "rule 6 BEFORE rule 7" but the `if RESERVED_NOT_EMITTED_V01.contains(...)` branch fires first (pushes rule 7). Mutually exclusive in v0.1 so order never observable; prose is mildly confusing on first read.
7. `cmd/decode.rs:113-118` — `#[cfg(test)] mod tests {}` is empty. Either add a smoke test or drop the module.

## Affirmations

- `main.rs::emit_error` early-return for `FutureFormat && !json_mode` correctly avoids stderr "error:" contradicting OK stdout (verify.rs:112).
- Clap usage error overridden to `ExitCode::from(64)`, reserving 2 for ms1 format violations per §6.
- `is_json_mode` exhaustively covers all 5 Command variants; `Vectors` correctly returns false.
- `EncodeArgs` `#[group(id = "input", required = true, multiple = false)]` enforces exactly-one-of-{phrase,hex}.
- Concurrent-stdin guard in `verify.rs:42-46` fires before any read — correctly placed pre-Step 2 of §2.4.1.
- `read_phrase_input` + `normalize_phrase` use `split_whitespace().collect::<Vec<_>>().join(" ")` correctly; 3 new parse.rs tests are meaningful.
- `analyze()` in inspect.rs walks rules 2 → 3 → 4 → 6/7 → 8 → 9 → 10 ascending; multi-failure case yields ascending `failure_reasons`.
- `inspect.rs` uses the closed 8-tag set from SPEC §2.3 — kebab-case, stable.
- `vectors.rs` `include_str!` resolves correctly; `--pretty` round-trips through `serde_json::Value`; always exits 0.
- `Cli::about` matches SPEC §2.6 verbatim; all 5 subcommand `about` + `after_long_help` match §2.6 verbatim.
- `Payload::Entr(b)` extraction with `unreachable!()` defensive arm correctly handles `#[non_exhaustive]` future variants.
- `From<bip39::Error>` and `From<ms_codec::Error>` impls flow correctly via `?` in cmd handlers.
- verify.rs phrase round-trip never echoes either phrase to stdout/stderr.
- decode.rs emits stdout + stderr default-language warnings only when defaulted; explicit `--language` removes both.
- 28 unit tests pass; clippy + fmt + build clean.
- inspect's exit-3 routing constraint per §2.3.1 verified: `ms_codec::inspect()` only fails on BIP-93 parse (→ Codex32 → exit 1); reserved-tag-not-emitted appears in `failure_reasons` (would_decode=false, exit 0).
- verify-with-`--phrase`-and-bad-checksum-phrase correctly exits 1 via `Bip39` (NOT exit 4) per §2.4.1 step 3.
- All execution-time fixups (Payload non_exhaustive wildcards in decode/verify; read_phrase_input / read_input split; bip39 features = ["all-languages"] in Cargo.toml) are reviewer-verified as appropriate.

## Recommendation

Proceed to Phase 4. The 7 nits are all cosmetic / non-load-bearing; controller may defer to FOLLOWUPS or apply opportunistically.
