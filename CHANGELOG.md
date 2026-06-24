# Changelog

All notable changes to `ms-codec` and `ms-cli` are documented in this file. Each release entry is prefixed with the crate name (`## ms-codec [0.1.0]`, `## ms-cli [0.1.0]`).

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project follows [SemVer](https://semver.org/spec/v2.0.0.html) with the pre-1.0 convention that the second component (`0.X`) is the breaking-change axis.

## ms-cli [0.13.1] — 2026-06-23

**SemVer-PATCH — BSD secret-hygiene parity + FreeBSD compile-gate. `set_non_dumpable()` (in `crates/ms-cli/src/process_hardening.rs`) was fenced `#[cfg(target_os = "linux")]` and a silent no-op on the BSDs, so an `ms` process on FreeBSD/OpenBSD/NetBSD could be ptrace/ktrace-introspected and could drop a core file the BIP-39 entropy / mnemonic spills into. A second cfg arm restores parity. No new CLI flag / subcommand / output-shape. Linux behavior unchanged (the new arm is cfg-gated off everywhere but the BSDs). `ms-codec` UNTOUCHED. Shipped in lockstep with `mnemonic-toolkit` 0.73.1 / `md-cli` 0.11.1 / `mk-cli` 0.11.1 (byte-identical executable arm in all four CLI crates).**

### Changed

- **`set_non_dumpable()` gains a BSD parity arm** (`crates/ms-cli/src/process_hardening.rs`). Keeps the Linux `prctl(PR_SET_DUMPABLE, 0)` arm; adds a `#[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]` arm that does (i) on FreeBSD only, `procctl(P_PID, 0, PROC_TRACE_CTL, PROC_TRACE_CTL_DISABLE)` (disables ptrace/ktrace introspection AND core dumping) and (ii) on all three BSDs, `setrlimit(RLIMIT_CORE, {0, 0})` (hard-zeros the core-dump size). Best-effort (return values ignored). macOS/Windows remain a documented no-op. No `libc` version bump. Compile-gated BSD unit tests added (compile-checked but never executed by the chosen CI).
- **FreeBSD compile-gate added to CI** (`.github/workflows/rust.yml`, new `freebsd-compile-gate` job). Runs a WHOLE-CRATE `cargo check --target x86_64-unknown-freebsd -p ms-cli` (NOT `--lib` — ms-cli is bin-only [`[[bin]] name = "ms"`] and its `process_hardening` lives in the bin target; a `--lib` check would be silent false-green). Compile-covers the BSD hardening arm.

## ms-cli [0.13.0] — 2026-06-23

**SemVer-MINOR — `ms gen-man`: self-emit roff man pages from the compiled clap command tree.**

### Added

- **New `ms gen-man --out <DIR>` subcommand.** Writes one roff man page per (sub)command into `<DIR>` (`ms.1` plus `ms-<sub>.1` for each subcommand), via `clap_mangen::generate_to(Cli::command(), &dir)` — clap-generated, hence **binary-faithful by construction** (no hand-authored content, no content-fidelity gate). The directory is created if absent. Part of the constellation-wide man-page rollout (`mnemonic`/`md`/`ms`/`mk` all gain `gen-man`); `scripts/install.sh` drops the pages into the user manpath post-`cargo install`, and the man set ships as the `ms-man.tar.gz` release asset on each `ms-cli-v*` tag.
- New `clap_mangen = "0.3"` dependency (needs clap `^4.0`; ms-cli is on clap 4.6.1 — no clap bump).

### Notes

- The generator uses the **naive** `generate_to` call with **no pre-`.build()`**: a pre-build would materialize clap's `help` pseudo-subcommand shadow tree into spurious `*-help*.1` pages. A negative-canary test asserts zero `*-help*.1` pages are emitted.
- `--no-auto-repair`-style global flags do not apply here (ms has no `global=true` flag); clap_mangen 0.3 renders global args in zero pages regardless.
- **No `ms-codec` change** (man generation is a CLI-only concern) — codec is NOT bumped.

## ms-cli [0.11.0] — 2026-06-22

**SemVer-MINOR — secret-memory-hygiene: `ms derive` best-effort byte-scrubs the derived master/account `Xpriv` (Wave-2 ms lane).**

### Changed

- **`ms derive` now confines the two derived `Xpriv` values (root + account private keys) in a binary-private, move-only `ScrubbedXpriv` newtype** that BEST-EFFORT byte-scrubs on drop: `SecretKey::non_secure_erase()` over the spending key + a `write_volatile` zero-write over the 32 `chain_code` bytes. Mirrors the R0-blessed `ScrubbedXpriv` shipped in the toolkit (v0.70.0). `master_fingerprint` and `account_xpub` are materialized before either wrapper drops, so `ms derive` stdout and `--json` output are **byte-identical** before/after. The introduction of a named secret-confinement type makes this MINOR (consistent with the constellation's secret-type-migration precedent), even though the observable output is unchanged.
- The newtype derives no `Debug` and keeps the inner `Xpriv` private, removing the latent `{:?}` Debug-leak surface a bare `Xpriv` carries under bitcoin's `std` feature (RULE Z-DEBUG).

### Notes

- This closes the **in-repo leg** of `ms-cli-derive-xpriv-master-not-zeroized`. The scrub is **best-effort**: `bitcoin::bip32::Xpriv` (and its `SecretKey`) are upstream `#[derive(Copy)]`, so the compiler may have spilled transient bit-copies the scrub cannot reach — which is why secp256k1 names its erase `non_secure_erase`. The CLEAN fix (a `Zeroize`/non-`Copy` `Xpriv`) is upstream-blocked and stays tracked as `rust-bitcoin-xpriv-zeroize-upstream`. The source seed remains `Zeroizing<[u8; 64]>` + mlock-pinned.
- **No public API / CLI flag / output-shape change.** `ms-codec` is **not** bumped (the change is entirely in `crates/ms-cli`).

## ms-codec [0.6.0] — 2026-06-21

**SemVer-MINOR — secret-memory-hygiene: `InspectReport` redacts + scrubs the entropy; `decode()` theater-clone removed (cycle-15 Lane M).**

### Changed (breaking, public API)

- **`InspectReport.payload_bytes` is now `Zeroizing<Vec<u8>>`** (was bare `Vec<u8>`) — scrub-on-drop. Read-only consumers (`.len()`, `hex::encode(&field)`) are unaffected (`Deref<Target=Vec<u8>>`); code that bound the field by value or relied on the derived `Debug` is a breaking change ⇒ MINOR.
- **`InspectReport`'s `Debug` is hand-rolled (no longer derived)** to redact `payload_bytes` as `[REDACTED; N bytes]` (RULE Z-DEBUG: `Zeroizing`'s own derived `Debug` is non-redacting). All structural fields stay visible.

### Fixed

- **`decode()` no longer allocates an extra un-scrubbed entropy copy.** The old `Zeroizing::new(data)` + deref-clone into `Payload` only scrubbed the moved-from buffer while the live `Payload` got a fresh bare copy — net theater. The bytes now move straight into `Payload` (strictly fewer copies).

### Notes

- **Wire format UNCHANGED.** `ms1` encode/decode bytes + the `Payload` / `decode()` `(Tag, Payload)` shape are byte-identical (guarded by a new wire-invariant test) — downstream `mnemonic-toolkit` is a recompile-only pin bump.
- The codex32 share-string leg (`Codex32String` / `String`-backed, dormant upstream, no `Drop`) is **enumerated and deferred** (FOLLOWUP `ms-codec-share-strings-not-zeroized-encode-and-combine` stays `open`, bound to the codex32 vendor/fork decision). The reachable `Vec<u8>` share buffers stay `Zeroizing`.

## ms-cli [0.10.0] — 2026-06-21

**SemVer-MINOR — secret-memory-hygiene: intake/report/output zeroize + `RepairDetail` Debug-drop; re-pins `ms-codec =0.6.0` (cycle-15 Lane M).**

### Fixed

- **`ms inspect` / `ms repair` wrap their ms1 intake in `Zeroizing<String>`** (inspect was the lone unwrapped intake command).
- **`RepairDetail`'s chunk fields are `Zeroizing<String>`, and its `#[derive(Debug)]` is DROPPED** (no `{:?}` consumer; a derived `Debug` over the wrapped secret chunks would leak — RULE Z-DEBUG). `Clone` kept.
- **`ms verify`'s `emit_round_trip_ok` counts words off a `Zeroizing<String>` temp** instead of a bare `_mnemonic.to_string()`.
- **`--json` emit buffers carrying secret material (entropy hex / phrase / shares / ms1) are scrubbed on drop** (encode / decode / combine / split / inspect / repair) — defense-in-depth.

### Notes

- **`--help` / clap surface UNCHANGED** (zeroize is user-invisible) — no manual / GUI schema-mirror update.
- **`--json` wire shapes UNCHANGED** (the `Zeroizing` wraps are in-memory only).
- **`ms derive`'s `Xpriv` master/account remain bare** — `bitcoin::bip32::Xpriv` has no `Zeroize` (rust-bitcoin upstream); PARTIAL: lifetime-min + comment, tracked by the new `rust-bitcoin-xpriv-zeroize-upstream` FOLLOWUP. The source seed is already `Zeroizing` + mlock-pinned.

## ms-cli [0.9.0] — 2026-06-21

**SemVer-MINOR — non-English BIP-39 wordlist correctness + advisory/hygiene (constellation bug-hunt cycle-8).**

### Fixed

- **`ms derive` / `ms verify` no longer PANIC (`unreachable!`) on a valid non-English (mnem) ms1** (H4/H5). Both commands previously rebuilt the BIP-39 mnemonic from the `--language` flag (English by default), so a non-English seed either panicked or produced a WRONG fingerprint. They now decode using the **wire wordlist-language byte** carried by the ms1 payload (shared `payload_entropy_and_language` helper), so the derived master fingerprint / xpub matches the seed's actual language.
- **`CliError`'s `Debug` is hand-rolled** so a secret ms1 can no longer ride out via the inner `codex32::Error` `{:?}` rendering (L5). Mirrors the ms-codec [0.4.4] secret-withholding discipline at the CLI error-envelope layer.

### Changed

- **`--language` is now advisory for a mnem ms1.** When the flag disagrees with the wire wordlist-language byte, the commands emit a `note:` (stderr) and proceed with the **wire** language rather than honoring the flag. `verify --language` is `Option`-ized so the default no longer fabricates a spurious disagreement `note:`.
- **`ms combine --to entropy` emits a non-English-wordlist advisory** (stderr) — the recovered entropy is correct; the language byte is re-encode metadata, so the advisory flags that a downstream re-encode must restore the language (L26).

### Notes

`ms-codec` is **UNCHANGED**; the `ms-codec` exact pin stays `=0.5.0`. The wire-language fixes are ms-cli-local (decode-and-derive routing + error `Debug`). Part of the constellation bug-hunt program; companions tracked per-finding (H4/H5/L5/L26).

## ms-cli [0.8.1] — 2026-06-21

**SemVer-PATCH — dependency bump to `ms-codec =0.5.0`; inherits the cross-share polynomial-consistency check in `combine_shares`. Constellation bug-hunt cycle-4 (M6), ms-cli leg.**

### Fixed

- An inconsistent (same-id, mixed-polynomial) share set now surfaces an explicit `InconsistentShareSet` → **exit-2 `FormatViolation`** arm with an accurate message, rather than falling through the `other =>` wildcard to `BadInput` / exit 1. Inherits the actual reject from `ms-codec [0.5.0]` via the exact-pin bump. Companion: `ms-codec [0.5.0]`.

## ms-codec [0.5.0] — 2026-06-21

**SemVer-MINOR — FUNDS-SAFETY: `combine_shares` rejects a same-id mixed-polynomial share set that previously returned a SILENT WRONG secret. Beyond-BIP-93 defense-in-depth. Constellation bug-hunt cycle-4 (M6).**

### Added

- New `Error::InconsistentShareSet` (additive → MINOR).

### Fixed

- codex32 K-of-N Shamir recovery carries no digest share, and `combine_shares` previously interpolated the secret over ALL supplied shares with no truncate-to-`k` and no cross-share consistency check — so a same-id (same hrp / id / threshold / length) but DIFFERENT-polynomial share set combined to a SILENT WRONG secret with no error. `combine_shares` now recovers the secret from EXACTLY the first `k` shares (which define the polynomial), then verifies every EXTRA supplied share lies on that same polynomial (re-derived `interpolate_at(k_set, idx)` must equal the supplied share) → `Error::InconsistentShareSet` on any mismatch.

### Notes

A valid exactly-`k` combine is **bit-identical** to the prior all-shares interpolation (`k == n` → empty membership loop), and a valid `n > k` all-consistent combine recovers the same secret (every extra lies on the curve). The irreducible limit — an exactly-`k` mixed pair is undetectable, since any `k` points define a polynomial — is noted in-test. Companion: `ms-cli [0.8.1]` (inherits via the exact-pin bump).

## ms-cli [0.8.0] — 2026-06-15

**SemVer-MINOR — standardized mstring display-grouping on `ms encode` + `ms split`; default text output is now space/5 print-once (was print-twice + wrap@10). Part of the cross-constellation `display-grouping-render-strip-v1` cycle (P2).**

### Added

- **`ms encode --group-size <u16>`** (default `5`, `0` = unbroken) + **`--separator <space|hyphen|comma>`** (keyword or literal `" "|-|,`, default `space`) — insert a separator every N characters in the emitted ms1 text. SPEC §3/§5. Same two flags on **`ms split`** (applied to each share). The default `ms encode` text output is now **single line, space/5, print-once** (previously `<ms1>\n\n<chunked-form>` with a wrap@10 second block) — a default-output change, hence MINOR. `--json` ALWAYS carries the canonical **unbroken** string.
- **`ms split` print-once:** stdout carries the N share strings one per line in the flag-controlled grouped form (pipe-friendly into `ms combine -`); the human labels ("share i of n") move to **stderr**.
- **`ms combine -`→stdin:** `ms combine` gains multiline share intake from stdin (one share per line; parallel to `mk`'s `read_mk1_strings`), so `ms split | ms combine -` round-trips.
- **Separator-stripping intake everywhere:** `ms decode`/`inspect`/`repair`/`encode --hex` (via `read_input`) and `ms combine` (positional + stdin) now strip ALL whitespace + `-` + `,` (SPEC §3.2) so a grouped or unbroken card both re-ingest. (ms-codec's decode does not tolerate separators; the legacy `strip_whitespace` handled whitespace only — the net-new coverage is `-`/`,`.)
- Conformance vectors `design/display-grouping-vectors.tsv` (byte-identical copy of the toolkit canonical) + `.sha256`, CI-pinned (`sha256sum -c` in the clippy job) + a bin-crate driver test over every row.

### Removed / Changed

- **The doubling-dedup heuristic in `parse.rs::strip_whitespace` is decommissioned** — it existed to absorb the print-twice `<ms1>\n\n<chunked>` stdout piped into a decoder; print-once everywhere makes the doubling unreachable. `strip_whitespace` now plain-strips display separators (no dedup).
- `format.rs::chunked` (5/space/wrap@10) is deleted; replaced by `render_grouped` (single line, configurable separator).

### Notes

stdout text was never a declared-stable interface and `--json` is unaffected. **`ms-codec` is UNCHANGED** (the pure fns are ms-cli-local — ms-cli is bin-only; conformance test is a bin-crate `#[cfg(test)]`). The `ms-codec` exact pin stays `=0.4.4`. Cross-repo lockstep (toolkit collapse + manuals; `mnemonic-gui` schema-mirror flags + separator dropdown) lands in later phases; FOLLOWUP `display-grouping-render-strip-v1`.

## ms-codec [0.4.4] — 2026-06-12

**SemVer-PATCH (SECURITY) — `ms_codec::Error` no longer echoes secret input in `Display` or `Debug`.**

### Fixed

- ms1 strings are SECRET-BEARING (BIP-39 entropy / BIP-32 seed / xpriv). Three error-rendering paths previously embedded the raw input, leaking secret material into any log / panic / `{:?}`:
  - `Error::Codex32(_)` rendered `{:?}` of the inner `codex32::Error`, whose `InvalidChecksum { string }` carries the FULL input and `MismatchedHrp(..)` / `MismatchedId(..)` carry the share strings. A single bit-flip of a valid ms1 share → `InvalidChecksum` → the whole secret rode in the message. The `Display`/`Debug` Codex32 arm is now a manual variant match: the three input-bearing variants are intercepted explicitly and rendered structurally only (`"invalid {short|long} checksum (input withheld)"`, `"mismatched HRP across shares"`, `"mismatched ID across shares"`); the other 13 codex32 variants carry only `&'static str` / `usize` / `char` / `Case` / `Fe` / `field::Error` (≤1 echoed char) and are rendered with their safe fields.
  - `Error::WrongHrp { got }` echoed the observed HRP; a data-char→`'1'` mutation could stretch that "HRP" into a long secret prefix. `got` is now CAPPED to the first 4 chars AT CONSTRUCTION (char-counted, multibyte-safe), at all three build sites (`decode.rs`, `envelope.rs` ×2). 4 < the 8-char leak window and still carries the "you typed mk1/lnbc not ms1" diagnostic. **JSON-consumer note:** because the cap is on the stored field, the ms-cli error-envelope `details.got` value now shows at most 4 chars (it previously echoed the full observed HRP); any downstream that pinned the full `got` should expect the shortened value.
  - `#[derive(Debug)]` on `Error` (which would dump every field, including the leaky ones) is replaced by a hand-rolled `Debug` that delegates to the sanitized `Display`. This is load-bearing for downstream `#[derive(Debug)]` wrappers (toolkit `ToolkitError` / `CliError`) whose `{:?}` transitively renders this type. Not a SemVer break — the `Debug` impl is preserved; its exact output is not contractual.
- The stress-Cycle-C `ms1_no_secret_leak` fuzz oracle previously EXCLUDED `Codex32(_)` and `WrongHrp{..}` because they leaked; the exclusion is now DELETED and the oracle scans every variant — it is the permanent regression gate for this fix (90s bring-up run clean).
- **ms-cli note:** `friendly_codex32` keeps the HRP / ID strings of `MismatchedHrp` / `MismatchedId` by design — those are provenance-bounded (HRP = `"ms"`, id = 4 chars, drawn from `interpolate_at` on already-valid `Codex32String`, never the data-part), out of this cycle's `ms_codec::Error` scope.

5 red-first leak cells in `ms-codec/src/error.rs` (each asserts neither `Display` nor `Debug` carries an 8-char window of the secret); the leak-enshrining test in `decode.rs` is updated to assert the 4-char cap. No API / wire / signature change. ms-cli exact pin → `=0.4.4` (ms-cli version unchanged). Resolves `ms-codec-error-display-echoes-input` (+ its toolkit companion). Brainstorm + 2 R0 rounds: `design/BRAINSTORM_error_display_no_echo.md`, `design/agent-reports/error-no-echo-r0-round{1,2}-review.md`. **crates.io publish + toolkit pin bump pending user authorization.**

## ms-codec [0.4.3] — 2026-06-12

**SemVer-PATCH — `decode_with_correction` no longer panics on a non-`ms1` input with no `'1'` separator (char-boundary fix).**

### Fixed

- `parse_ms1_symbols` reported the observed HRP for a non-`ms1` string by slicing `lower[..len-1]`. With no `'1'` separator, `len-1` can land inside a multi-byte char → panic ("byte index N is not a char boundary"). Minimized reproducer: a single `0xaa` byte (→ U+FFFD via `String::from_utf8_lossy`). Now slices at `rfind('1')` (`'1'` is ASCII, so always a char boundary; no Unicode char's UTF-8 bytes contain `0x31`), and uses the whole string as the observed HRP when there is no separator. Only `decode_with_correction` reached the raw slice — `decode`/`inspect` were length-gated / codex32-validated first — so `ms repair` and the indel oracle inherited the panic.
- Leak-neutral: the `WrongHrp.got` echo vector is the byte-identical WITH-`'1'` path; bounding that echo is the separate `ms-codec-error-display-echoes-input` FOLLOWUP.

Found by stress-Cycle-C fuzzing (`fuzz/fuzz_targets/ms1_decode.rs`), now the regression gate (re-enabled in the `fuzz-smoke.yml` smoke matrix). 2 regression cells in `decode.rs`. No API/wire change. ms-cli exact pin → `=0.4.3` (ms-cli version unchanged). Resolves `decode-with-correction-panics-on-non-char-boundary-hrp-slice`. Plan + mini-R0: `design/PLAN_decode_with_correction_char_boundary_fix.md`, `design/agent-reports/decode-char-boundary-fix-mini-r0-round1-review.md`. **crates.io publish + toolkit pin bump pending user authorization.**

## ms-codec [0.4.2] — 2026-06-10

**SemVer-PATCH — accept all-uppercase ms1 per BIP-173 (the QR alphanumeric form); fixes a combine-side secret-leak guard bypass.**

### Fixed

- The wire layer (envelope discrimination, inspect, combine grouping) read RAW string bytes past codex32's case-folded checksum validation, so a valid all-uppercase MS1 card failed `WrongHrp { got: "MS" }`. Wire extraction now canonicalizes the owned copy (lowercase) — codex32 has already enforced consistent case + a valid checksum upstream, so this is canonical-form normalization, not case-laundering (mixed-case still dies as `InvalidCase` before any of this). Pristine uppercase cards now decode, inspect, repair (`decode_with_correction`'s clean-codeword pass-through), and combine.
- **SECURITY (combine guard restored):** an all-uppercase secret-at-`S` card bypassed the `SecretShareSuppliedToCombine` guard (raw `b's'` compare missed `b'S'`) and — in a uniform-uppercase same-id set — codex32's index-match short-circuit RETURNED THE SECRET PAYLOAD from `combine_shares`. The canonicalized fields restore the guard; pinned by a red-first test that demonstrated the leak.
- Mixed-case SETS now combine (one consistently-uppercase share among lowercase): `combine_shares` re-canonicalizes each share after its first parse and hands the canonical vector to interpolation (codex32's cross-share hrp/id compares are raw); recovered output is lowercase.
- A true wrong-HRP error now reports the canonicalized form (`got: "xs"` for `XS1…`).

10 new cells (ms-codec `uppercase_envelope.rs` ×9 + an ms-cli CI-visible decode cell — CI runs only `-p ms-cli`). Wire emission unchanged (lowercase). ms-cli exact pin → `=0.4.2` (ms-cli version unchanged). Resolves `ms1-envelope-uppercase-bip173` (companion of toolkit v0.53.3 / audit M11). Plan + 3 R0 rounds + impl review: `design/PLAN_ms1_envelope_uppercase.md`, `design/agent-reports/ms1-uppercase-*.md`.

## ms-codec [0.4.1] — 2026-06-10

**SemVer-PATCH — `combine_shares` rejects (no longer aborts on) a non-standard-length Entr share set.**

### Fixed

- `dispatch_payload`'s `Entr` (`0x00`) arm now calls `validate()`. A **valid-checksum but non-standard-length** Entr share set (entropy length ∉ {16,20,24,28,32}) recovered via `combine_shares` previously returned an *unvalidated* payload, and `ms combine --to phrase` / `ms decode` then **panicked** inside `bip39::from_entropy_in(...).expect(...)` (exit 101). The Entr arm now returns `Error::PayloadLengthMismatch`, so the CLI surfaces a clean error instead of aborting. Encode path unaffected (validates up front); no new error variant, no API/wire change. `ms-cli` is unchanged (inherits the fix via the `ms-codec` bump). Resolves audit-2026-06-10 finding `combine-no-length-validation-panic` (I9). See `crates/ms-codec/CHANGELOG.md`.

## ms-cli [0.7.0] — 2026-06-03

**SemVer-MINOR — `ms split` / `ms combine` (K-of-N codex32 shares).**

### Added

- **`ms split [--phrase|--hex] -k <K> -n <N> [--language] [--json]`** — split a secret into N codex32 shares (any K recombine); a non-English `--phrase` produces `mnem` shares so the wordlist language survives the split. Output is private-key-material (the share *set* is secret-equivalent).
- **`ms combine <share>... [--to phrase|entropy|ms1] [--json]`** — recombine ≥K shares into the recovered secret (phrase in the on-wire language for mnem). Surfaces the codex32/share error taxonomy (below-threshold, mismatched id/threshold/length, repeated index, secret-share-supplied).
- **`ms inspect` of a lone share** is now first-class (reports `kind: share` + threshold/id/index, exit 0) rather than a decode failure.

### Changed

- `ms decode` of a threshold∈2..9 string now exits 2 `IsShareNotSingleString` (directing to `ms combine`), replacing the v0.1 `ThresholdNotZero`. Re-pins `ms-codec` to `=0.4.0`.

## ms-codec [0.4.0] — 2026-06-03

See `crates/ms-codec/CHANGELOG.md` [0.4.0]: `Threshold` + `encode_shares`/`combine_shares` (K-of-N for entr AND mnem), threshold-field share dispatch, `0x01` unallocated, byte-identical v0.1/mnem single-strings.

## ms-cli [0.6.0] — 2026-06-01

**SemVer-MINOR — `ms` records the BIP-39 wordlist language on the wire for
non-English seeds.** Fixes the long-standing §6.3 non-English-seed footgun: a
non-English mnemonic backed up as `ms1` previously lost which wordlist
regenerates it (entropy alone is language-agnostic; the seed is PBKDF2 over the
language-specific string).

### Added

- **`ms encode` auto-routes a non-English `--phrase` to the new `mnem` payload**
  (records `--language` on the wire). English phrases and `--hex` input stay
  byte-identical `entr` — no wire change for the existing common case.
- **`ms decode` of a `mnem` string emits the phrase in the on-wire language.**
  The wire language is authoritative: if `--language` is supplied and disagrees,
  the wire wins and a stderr `note:` reports the override (exit 0). `entr` decode
  is unchanged (English default + the existing DEFAULT annotation).
- **`ms inspect` reports `kind: mnem` + `language: <name>`** (text and `--json`)
  and recognizes the `0x02` prefix + `mnem` lengths as valid.

Re-pins `ms-codec` to `=0.3.0` (the `mnem` payload kind).

## ms-codec [0.3.0] — 2026-06-01

See `crates/ms-codec/CHANGELOG.md` [0.3.0]: new `Payload::Mnem` (`0x02` prefix,
byte-aligned `[0x02][language][entropy]`), `package`/`discriminate` carry the
typed `Payload`, length-gate bound to kind, `InspectReport.kind`/`language`. The
v0.1 `entr` path is byte-identical.

## ms-cli [0.5.1] — 2026-05-31

### Added
- **Output-type stderr advisory (constellation cycle B, Phase 1).** `ms encode`/`ms decode` (BIP-39 entropy = private key material) and `ms repair` now emit `warning: stdout carries private key material (can spend) …`; `ms derive` (public derivation) emits `note: stdout is watch-only …`. Byte-identical wording to `mnemonic`'s advisory (cross-repo parity test). Replaces the prior `ms repair` "secret material on stdout" line. stderr-only.

## ms-cli [0.5.0] — 2026-05-31

**SemVer-MINOR — new `ms derive` subcommand: read-only public derivation (master fingerprint + account xpub).** Theme B piece #3 of the m-format constellation (after `mk derive`/`mk address` and `mnemonic addresses`). `ms` could recover the BIP-39 entropy (`ms decode`) but not produce the **master fingerprint** — the cheapest "did I recover the RIGHT seed?" verification oracle. `ms derive` fills that.

- **`ms derive [<ms1>] [--hex|--phrase] [--template] [--account] [--network] [--passphrase|--passphrase-stdin] [--language] [--json]`** — always emits the master fingerprint; with `--template` (bip44/49/84/86) also an account xpub at `m/<purpose>'/<coin>'/<account>'`. **PUBLIC outputs ONLY** — no master seed, root xprv, or private keys on stdout, no signing (a user wanting the xprv uses the toolkit's `mnemonic convert`).
- **The wordlist language is load-bearing here:** the BIP-39 seed = PBKDF2 over the language-specific mnemonic string, so the master fingerprint/xpub depend on `--language` — `ms derive` carries `ms decode`'s "DEFAULT" annotation (stdout + stderr) when `--language` is omitted.
- Adds `bitcoin = "0.32"` to ms-cli (the derivation spine: seed → master xpriv → fingerprint / account xpub). `--passphrase`/`--passphrase-stdin` is ms-cli's first passphrase channel (single-stdin guard); inline secrets get a new argv-leak advisory. ms-codec unchanged at 0.2.1.
- Lockstep: manual `43-ms.md` + GUI `mnemonic-gui/src/schema/ms.rs` (+ backfilled the never-mirrored `repair`) + toolkit ms-cli pin.

## ms-cli [0.4.1] — 2026-05-23

**SemVer-PATCH — process argv-hardening (`PR_SET_DUMPABLE`).** `ms` now calls `prctl(PR_SET_DUMPABLE, 0)` at the top of `main()` (Linux; no-op elsewhere), making `/proc/$PID/` unreadable to OTHER non-root UIDs and disabling core dumps — so a secret passed inline on argv can no longer be harvested by another user via `/proc/$PID/cmdline` or a core file. Residual same-UID window documented + accepted. New `process_hardening` module (`libc` already a dep). Part of the m-format constellation argv-hardening rollout (mnemonic-toolkit v0.34.7 + md-cli v0.6.1 + mk-cli v0.4.2). Tracked via the toolkit's `argv-overwrite-after-parse` FOLLOWUP closure.

## ms-cli [0.3.0] — 2026-05-13

v0.9.0 cross-repo Cycle B (`mlock(2)` page-pinning infrastructure), Phase
3b + Phase E rollup for ms-cli. Companion to `mnemonic-toolkit-v0.10.0`.
Cycle SPEC at `mnemonic-toolkit/design/SPEC_secret_memory_hygiene_v0_9_B.md`;
cross-repo audit matrix at
`mnemonic-toolkit/design/agent-reports/v0_9_B-secret-memory-hygiene-matrix.md`.

No user-facing CLI surface change: no flag additions or removals; exit
codes unchanged; JSON schemas unchanged. mlock soft-failures (if any)
emit a 2-line stderr summary at end-of-process per Cycle B SPEC §6 G2.5.

### Added (Phase 3b — mlock inline-copy + Site 5 + main wire)

- New `src/mlock.rs` (538 LOC): inline copy of toolkit's `mlock` module
  surface per SPEC §5 + §6 G6 ("fork-and-document-pattern over
  shared-crate-extraction"; constellation stays at 4 crates). Surface:
  `pin_pages_for(buf: &[u8]) -> PinnedPageRange` slice-fn primitive
  (Fix-B-only; no wrapper type), `PinnedPageRange { start, page_count }`
  + munlock-on-Drop, `MlockState` process-static singleton with
  `failure_count` + `total_bytes_unlocked` + `first_errno` aggregation,
  `report_at_exit()` end-of-process 2-line stderr emitter, `#[cfg(test)]`
  `FailMode` injection harness (`MNEMONIC_TEST_MLOCK_FAIL_MODE` env var
  with `eperm` / `enomem` / `einval` / `off` modes).
- Site 5 pin: `parse::read_stdin()` adds
  `let _entropy_pin = crate::mlock::pin_pages_for(buf.as_bytes());`
  immediately after the read returns (`parse.rs:65`); pin scope-bound to
  the buffer's lifetime.
- `main.rs`: `mlock::report_at_exit()` call before exit (mirrors
  toolkit's main-wire).
- New `libc = "0.2"` dep.

### Added (PE — release rollup)

- `.github/workflows/rust.yml` (NEW): first Rust CI workflow for ms-cli
  (ms-codec has its own separate workflow). Jobs: `test` (Ubuntu + macOS
  matrix with `ulimit -l 65536` on Linux; cargo test + 3 fault-injection
  steps for G2.1 eperm + G2.3-debug einval + G2.4 off control),
  `test-release-mlock-einval` (Linux release build; SPEC §6 G2.3 release
  branch), `miri` (Ubuntu nightly; SPEC §6 G4.b),
  `clippy --all-targets -- -D warnings`, `g6-invariant` (SPEC §6 G6
  cross-repo inline-copy invariant; checks out toolkit at master and
  asserts normalized `mlock.rs` byte-equal).
- `tests/mlock_g6_invariant.rs` (NEW): SPEC §6 G6 enforcement. Reads
  ms-cli's `mlock.rs` and toolkit's `mlock.rs`, normalizes both
  (strip `//`, `///`, `//!` comment lines at start-of-trimmed-line;
  preserve `use` statements + `#[cfg]` attributes), and asserts
  byte-equal + name-export parity against a static MANIFEST (14
  top-level items). Sibling-repo path discovery via `SIBLING_REPO_PATH`
  env var with adjacent-dir relative fallback for local-dev.

### Cycle review history (ms-cli participation)

- Phase 3b: R1 Opus 0C/0I cross-repo CLEAR
  (`mnemonic-toolkit/design/agent-reports/v0_9_B-phase-3b-r1.md`).

### Tests

- 2 new G6 invariant tests in `tests/mlock_g6_invariant.rs`.
- mlock module's `g2_*` `#[ignore]`-gated subprocess tests reachable via
  `--include-ignored` per workflow steps.
- All pre-existing ms-cli tests green.

### What didn't change

- ms1 wire format unchanged (Cycle B is functionally transparent —
  SPEC §6 G7).
- ms-codec dep exact-pin: `=0.1.3` (no Cycle B work in ms-codec).
- v0.2.x → v0.3.x bump tracks the cycle-major axis (per Cycle B SPEC
  §4 PE), not a breaking change in the SemVer sense — there is no
  public-API surface in a binary-only crate.

## ms-cli [0.2.2] — 2026-05-13

v0.9.0 cross-repo Cycle A (OWNED-buffer secret-memory hygiene), Phase E
patch bump for ms-cli. No user-facing API change (no flag additions /
removals; exit codes unchanged; JSON schemas unchanged).

### Added (zeroize discipline; internal-only)

- New `zeroize = "1.8"` dep.
- `EncodeArgs::phrase`, `EncodeArgs::hex`, `VerifyArgs::phrase` clap-field
  rows now consume + immediately wrap: `Zeroizing::new(std::mem::take(...))`
  at `run()` entry, so the clap-resident `String` buffer is scrubbed on
  drop.
- `parse::read_phrase_input` returns `Result<Zeroizing<String>>`;
  `parse::read_stdin` uses `Zeroizing<String>` for its raw read buffer.
- `cmd/encode::run`, `cmd/decode::run`, `cmd/verify::run` use
  `Zeroizing<Vec<u8>>` / `Zeroizing<String>` typed locals for entropy
  and phrase transits. `Payload::Entr` consumer side wraps per the
  ms-codec caller-wrap contract.
- New lint `tests/lint_zeroize_discipline.rs` enumerates 10 ms-cli
  OWNED-buffer rows + per-row evidence anchors.

### Internal (workspace-internal dep bump)

- `ms-codec` exact-pin: `=0.1.2` → `=0.1.3` (companion lockstep release).

### Known third-party residue

- `bip39::Mnemonic` interior buffer is not zeroize-aware
  (FOLLOWUP `rust-bip39-mnemonic-zeroize-upstream`, tier `external`).
  SAFETY-anchor doc-comments at every Mnemonic call site in
  `cmd/encode.rs`, `cmd/decode.rs`, `cmd/verify.rs`.

### Tests

- 10 ms-cli OWNED-buffer rows enumerated in `lint_zeroize_discipline.rs`.
- All pre-existing ms-cli tests green on the rebased Phase 2 work.

## ms-codec [0.1.3] — 2026-05-13

v0.9.0 cross-repo Cycle A (OWNED-buffer secret-memory hygiene), Phase E
patch bump for ms-codec. Cycle SPEC at
`mnemonic-toolkit/design/SPEC_secret_memory_hygiene_v0_9_0.md`; cross-repo
audit matrix at `design/agent-reports/v0_9_0-secret-memory-hygiene-matrix.md`
(sibling) and the toolkit canonical matrix.

### Added (zeroize discipline; no library API change)

- New `zeroize = "1.8"` dev-equivalent dep (in workspace toolchain via
  `ms-cli`).
- Internal `Zeroizing<Vec<u8>>` local-wrap discipline in `envelope::package`,
  `envelope::discriminate`, and `decode::decode`. Drop-time scrub on
  every intermediate `Vec<u8>` that carries `Payload::Entr` bytes.
- `payload.rs` doc-comment block locks the public-API caller-wrap
  contract: callers of `decode()` MUST wrap the returned
  `Payload::Entr(Vec<u8>)` in `Zeroizing::new(...)` to inherit
  drop-time scrub.
- New lint `tests/lint_zeroize_discipline.rs` enumerates 4 ms-codec
  OWNED-buffer rows + their per-row evidence anchors.

### What didn't change

- ms1 wire format unchanged.
- Public API surface unchanged (`Payload::Entr(Vec<u8>)` shape preserved;
  widening to `Zeroizing<Vec<u8>>` is a breaking change deferred per
  SPEC §3 OOS-public-payload — FOLLOWUP `ms-codec-payload-zeroize-public-api`).
- v0.1 → v0.2 migration contract unchanged.

### Known third-party residue

- `codex32::Codex32String` internal buffer is not zeroize-aware
  (FOLLOWUP `rust-codex32-zeroize-upstream`, tier `external`).

### Tests

- 4 OWNED-buffer rows + parametric evidence cells in
  `lint_zeroize_discipline.rs`.
- Existing 59 cells (52 pre-Cycle-A + 7 from v0.8.0 cycle) all green
  on the rebased Phase 2 work.

## ms-cli [0.2.1] — 2026-05-12

### Fixed

- `ms --version` and `ms --help` now exit `0` instead of `64`. The
  v0.2.0 `fn main()` mapped every `Cli::try_parse()` `Err` to
  `ExitCode::from(64)`, but clap returns `Err` for two non-error
  terminations as well — `ErrorKind::DisplayVersion` (`--version`)
  and `ErrorKind::DisplayHelp` (`--help`). The output already
  prints to stdout in those cases; the canonical Unix convention
  is exit 0. The fix branches on `e.kind()` and returns
  `ExitCode::SUCCESS` for the two display variants, preserving the
  SPEC §6 carve-out (exit 64 instead of clap's default 2, so 2
  stays reserved for ms1 format violations) for real parse errors.
  Discovered during `bg002h/mnemonic-gui` v0.2.0 release prep
  (companion: `bg002h/mnemonic-gui`).
- Two new regression cells in `tests/exit_codes_table.rs`:
  `version_flag_exits_zero_and_prints_version` and
  `help_flag_exits_zero_and_prints_help`.
- `cargo fmt` applied to `src/main.rs` — the rustfmt-preferred
  shape for the new `match e.kind()` arm uses a block body when
  the `|` pattern needs to wrap.

## ms-cli [0.2.0] — 2026-05-12

### What's new

- New `ms gui-schema` subcommand emits SPEC §7 JSON describing the CLI's flag surface (subcommand list, flag names, `required`, `kind`, dropdown `choices`, positionals). Consumed by the [`bg002h/mnemonic-gui`](https://github.com/bg002h/mnemonic-gui) schema-mirror CI gate (v0.2 Phase C). Companion: `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `mnemonic-gui-schema-mirror`.
- Implementation walks `clap::CommandFactory::command()` reflection — JSON stays in lockstep with `Cli` automatically; the GUI's mirror gate catches drift.
- Intentionally lossy: complex GUI `FlagKind` variants map to `"text"` upstream and are hand-overridden in the GUI schema file after JSON-bootstrap import. `"boolean"` is produced for `SetTrue` / `SetFalse` / `Count` actions; `"dropdown"` is produced when `Arg::get_possible_values()` is non-empty.

### What didn't change

- All 5 v0.1 subcommands (`encode`, `decode`, `inspect`, `verify`, `vectors`) keep their flag surface, exit codes (0/1/2/3/4/64), and `--json` schemas verbatim.
- Wire format (ms1) is unchanged — `ms-codec` is unaffected at `=0.1.1`.

### Tests

11 new integration tests in `tests/gui_schema_emits_spec_v7_json.rs` covering: exit-0, JSON-parseable, `version == 1`, `cli == "ms"`, `encode`/`decode`/`verify` subcommands present, `encode --phrase` / `--hex` flags, `--language` dropdown with hyphenated `chinese-simplified` / `chinese-traditional` (not `simplifiedchinese`), `--json` boolean kind across subcommands, `vectors --pretty` boolean, `inspect` surface. The v0.1 test surface (77 tests) is preserved.

## ms-cli [0.1.0] — 2026-05-04

### What's new

- Initial release. Companion CLI to ms-codec v0.1.0.
- 5 subcommands: encode, decode, inspect, verify, vectors.
- Phrase-first encode (`--phrase` headline; `--hex` escape hatch); structured `--json` output mode across all commands.
- Strip-whitespace stdin uniform across commands — handles pipe round-trip, engraver-typed-back chunked form, and copy-paste artifacts with one mechanism.
- BIP-39 wordlist enforcement: 10 wordlists supported via `--language` (default `english` with non-suppressible stderr warning surfacing the SPEC §6.3 hazard).
- Exit codes per SPEC §6: 0/1/2/3/4 (verify round-trip mismatch is its own exit code) plus 64 for clap usage errors (overrides clap's default 2 to keep ms1 format violations distinct).
- Engraving-friendly stdout: encode emits `<ms1>\n\n<chunked-form>` (5-char groups, 10/line max, never mid-chunk).
- `verify --phrase` round-trip check: useful for engraver-typed-back proofreading. Phrases never echoed to output (secrets discipline).

### Tests

77 tests across the surface: 29 unit (Phase 1 modules) + 48 integration (`assert_cmd`). cargo build / clippy --all-targets -D warnings / fmt --check all clean.

## ms-codec [0.1.2] — 2026-05-13

v0.8.0 cross-repo BIP-vector adoption cycle, Phase 2. Cycle SPEC at
`mnemonic-toolkit/design/SPEC_test_vector_audit_v0_8_0.md`; per-phase
review at `design/agent-reports/v0_8_0-phase-2-bip93-corpus-r1.md`.

### Added (tests-only; no library API change)

- `tests/bip93_inline_vectors.rs` — full BIP-93 §Test Vectors inline
  corpus pin. 5 valid cells (§93.1–.5: 16-byte / 16-byte / 16-byte /
  32-byte / 64-byte master seeds across k=0 / k=2 / k=3 + long-codex32
  shapes); 1 parametric cell asserting all 64 BIP-93 §Invalid entries
  are rejected by `rust-codex32 =0.1.0`; 1 invariant cell guarding the
  invalid-corpus count.
- `design/agent-reports/v0_8_0-bip-test-vector-audit-matrix.md` — v0.8.0
  successor to the v0.7.1 matrix. Cross-cites the toolkit hub matrix +
  sibling-repo matrices.
- `design/FOLLOWUPS.md` — `bip-vector-adoption-v0_8` (cycle companion)
  and `bip93-invalid-corpus-granular-error-pin` (deferred future
  tightening).

### Corrected

- v0.7.1 audit matrix footnote claimed BIP-93 §Invalid has "42
  strings"; live count via `gh api repos/bitcoin/bips/contents/bip-0093.mediawiki`
  is 64. Source-of-truth corrected at v0.8.0; v0.7.1 matrix carries a
  SUPERSEDED header with forward-pointer.

### What didn't change

- ms1 wire format unchanged.
- Public API surface unchanged.
- v0.1 → v0.2 migration contract unchanged.
- All pre-existing ms-codec tests still pass; +7 cells from this
  cycle → 59 ms-codec total at v0.1.2.

## ms-codec [0.1.0] — 2026-05-03

### What's new

- Initial release. Reference implementation of the **ms1** backup format (HRP `ms`) for BIP-39 entropy.
- Wire format: BIP-93 codex32 used directly via Andrew Poelstra's `rust-codex32 = "=0.1.0"` (CC0). No fork.
- v0.1 payload kind: `entr` (BIP-39 entropy, 16/20/24/28/32 B = BIP-39 word counts {12, 15, 18, 21, 24}).
- v0.1 emitted strings: 50/56/62/69/75 chars (short codex32 checksum only).
- Public API: `encode(Tag, &Payload) -> Result<String>`, `decode(&str) -> Result<(Tag, Payload)>`, `inspect(&str) -> Result<InspectReport>`.
- `Tag::ENTR` const; `Payload::Entr(Vec<u8>)`; `InspectReport` for debugging.
- Decoder applies the full SPEC §4 validity rule set (10 rules); encoder mirrors the reserved-not-emitted-tag rejection (SPEC §3.5.1).
- v0.2 K-of-N share-encoding migration designed up-front via the `0x00` reserved-prefix byte; v0.1 strings remain forward-readable by v0.2 decoders. See [`MIGRATION.md`](MIGRATION.md).
- `Payload`, `PayloadKind`, `Error`, `InspectReport` are `#[non_exhaustive]` from day 1 to allow semver-minor variant additions.
- `Tag` field is private; construction via `try_new` (alphabet-validated) or `from_raw_bytes` (tooling-only).

### What didn't change

(N/A — initial release.)

### Migration notes

(N/A — initial release. See [`MIGRATION.md`](MIGRATION.md) for the planned v0.1 → v0.2 contract.)

### Tests

- 50 tests across all targets: 28 unit + 1 doc-test (Quickstart) + 10 negative + 5 round-trip proptests + 2 forward-compat + 3 BIP-39 integration + 1 vector-corpus replay.
- `cargo build`, `cargo clippy --all-targets -D warnings`, `cargo fmt --check` all clean.

### Wire-format SHA pin

The canonical test vectors at `crates/ms-codec/tests/vectors/v0.1.json` are SHA-256-pinned at this release. Subsequent corpus changes that alter the SHA require a SemVer minor bump per the pre-1.0 breaking-change-axis convention.

```text
sha256(crates/ms-codec/tests/vectors/v0.1.json) = f8d671f543101a4b90fd028126aef66958ff4050e38a32baa48ff298cdf2901a
```

## Unreleased

(none)
