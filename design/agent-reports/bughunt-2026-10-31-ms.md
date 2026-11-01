# Bug report: bg002h/mnemonic-secret @ 85efbd4

## Summary
- **Scope:** 44 Rust source files with about 13,900 lines, in the `ms-codec` and `ms-cli` crates. Tests and fuzz targets were not read.
- **Agents:** 11 slices, each read by a reader and a second reviewer. All 22 finished.
- **Result:** 11 suspicions were rated 3 or higher. **10 were confirmed**: 6 demonstrated with a failing test and 4 by reading. 1 was rejected.
- **How they were tested:** the pinned toolchain and a git dependency are unavailable offline, so the crates couldn't be built. Each test is a standalone `rustc` copy of the function in question. Tests are under `/tmp/huntms/prover-N/` in the sandbox.

The core codex32 maths checked out. Reviewers hand-checked the GF(32) tables, the generators, the target residues and the share interpolation, and found nothing wrong.

## Confirmed bugs (worst first)

### Medium

**1. `encode_shares` with threshold 0 skips encode()'s tag checks, so it can mint strings that `decode` rejects** (confirmed by reading)
- Where: `ms-codec/src/shares.rs` `encode_shares`, around lines 110-118. It calls `Codex32String::from_seed` directly instead of `encode()`.
- Input: `encode_shares(Tag::try_new("seed"), Threshold::ZERO, 1, &Payload::Entr([0;16]))`, or `Tag::HASH` with an Entr payload, or `Tag::ENTR` with a Preimage payload.
- Should: return the same error as `encode()` (ReservedTagNotEmittedInV01 or TagKindMismatch). The docstring promises output byte-identical to `encode()`.
- Does: returns Ok with an ms1 string that `decode()` refuses, i.e. a backup that can't be read back.
- Fix: `return Ok(vec![crate::encode::encode(tag, secret)?]);`.

**2. `mlock` unlocking is not reference-counted, so dropping one pin unlocks another live secret's page** (demonstrated)
- Where: `ms-cli/src/mlock.rs` `PinnedPageRange::drop`, line 95.
- Input: two pinned secret buffers on the same page; drop the first.
- Should: the second secret's page stays locked.
- Does: Locked memory drops from 4 kB to 0 while the second pin is still live, so that secret can be swapped to disk, with no warning.
- Test: `prover-5/t.rs`.
- Fix: keep a count per page and call `munlock` only when it reaches 0.

**3. Non-ASCII characters are read as codex32 symbols during error correction** (demonstrated)
- Where: `ms-codec/src/decode.rs` `parse_ms1_symbols`, line 208. `c as u8` keeps only the low byte of the character.
- Input: an ms1 string with one character replaced by `ĳ` (U+0133), passed to `decode_with_correction`.
- Should: reject it, because `ĳ` is not in the codex32 alphabet.
- Does: reads `ĳ` as `'3'` (symbol 17), and may then "repair" the string and accept it.
- Test: `prover-4/t.rs`.
- Fix: reject non-ASCII characters before the cast.

**4. `ms combine --to ms1` is silently ignored for preimage shares** (confirmed by reading)
- Where: `ms-cli/src/cmd/combine.rs`, line 169. The Preimage arm returns early, before the `match args.to`.
- Input: `ms combine --to ms1` on shares split from a preimage (hash-tag) ms1.
- Should: print the recovered ms1 string, or fail with a clear error. The documented behaviour of `--to ms1` is to re-encode the secret.
- Does: prints the preimage hex record and exits 0.
- Fix: honour `CombineTo::Ms1` in the preimage arm.

**5. `ms inspect` says any k≥2 string "would combine", with no checks** (demonstrated)
- Where: `ms-cli/src/cmd/inspect.rs` `is_share` / `emit_share_json`, line 89. `would_decode` and `would_combine` are hardcoded to true.
- Input: a checksum-valid k=2 string with share index `s`, a wrong HRP or an unknown tag.
- Should: report false and list failure reasons. For example, `combine` rejects an `s`-index share with SecretShareSuppliedToCombine.
- Does: reports OK with empty `failure_reasons`.
- Test: `prover-7/t.rs`.

### Low

**6. `Codex32String::from_string` panics on a long HRP with a short data part** (demonstrated)
- Where: `ms-codec/src/codex32/mod.rs` `parts_inner`, around line 249.
- Input: `"laaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa12e0ds56ptqt4x"`, which is in the allowed length range and has a valid checksum.
- Should: return an error.
- Does: panics with "byte range starts at 6 but ends at 0". This can be reached through the public `inspect` and decode on untrusted input.
- Test: `prover-3/src/lib.rs`.
- Fix: `if s.len() < 6 + checksum_len { return Err(InvalidLength) }`.

**7. The string-length error always lists the entr length set** (demonstrated)
- Where: `ms-cli/src/error.rs`, line 204. The mapping drops the codec's `allowed` field.
- Does: a 56-character mnem string gets "string length 56 not in v0.1 set [50, 56, 62, 69, 75]", which contradicts itself.
- Test: `prover-6/t.rs`.
- Fix: carry `allowed` through to the message and details.

**8. "more than 8 errors; uncorrectable" when the decoder's limit is 4** (confirmed by reading)
- Where: `ms-codec/src/decode.rs`, lines 285, 295 and 315: `TooManyErrors { bound: 8 }`.
- Does: a string with 5 errors is reported as having more than 8.
- Fix: use `bound: 4`, or reword the message.

**9. `MnemUnknownLanguage` falls through to the catch-all error** (confirmed by reading)
- Where: `ms-cli/src/error.rs`, line 339. There is no arm for this variant.
- Input: a checksum-valid mnem string with language byte 10 or higher.
- Should: FormatViolation, exit 2, like the other format errors.
- Does: BadInput, exit 1, with the text "unhandled ms_codec::Error variant".
- Fix: add an explicit arm.

**10. The GUI schema marks `combine`'s shares positional as non-repeating** (confirmed by reading)
- Where: `ms-cli/src/cmd/gui_schema.rs`, line 117. It reads `get_num_args()` from a clap Command that hasn't been built yet, so the value is None.
- Fix: call `cmd.build()` first, or check for `ArgAction::Append`.

## Rejected
- **`combine_shares` always returns `Tag::ENTR`, even for a Preimage** (`shares.rs:322`). Rejected because this is documented, intended behaviour: the payload type comes from the prefix byte, and no caller relies on the tag.

## Not checked
These 17 suspicions were rated 2 or lower and were not sent to provers. The notable ones:
- The correction path accepts mixed-case input, which BIP-93 says to reject.
- `envelope.rs:195` panics on a 0- or 1-byte buffer.
- `derive.rs:518`: the seed's page is unlocked before the seed is wiped, because of drop order.
- `argv_guard.rs`: inconsistent trimming and lowercasing between the guard and clap.
- A 20-character hashlock phrase gets no short-phrase warning.
- An `--account` value of 2^31 or more gives a vague error.
- Several messages that say "v0.1 entr" for mnem cards.
- An empty positional share is accepted.

## What was read
Every non-test line of all 44 files was read by both a reader and a reviewer.
