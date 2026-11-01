# FIX: the ms bug-hunt Importants — `encode_shares` ZERO tag checks (#1), `combine --to` on preimage shares (#4), `inspect` of the secret share (#5), the non-alphabet refusal during correction (#7, closes #3), `inspect` of an unknown mnem language (T2), and the BCH primer's erasure overclaim (toolkit manual)

Small-fix track (RULES.md "Token diet and risk-matched review", item 4). Branch `bughunt/triage` from `4755eb9` (it already carries the RED rows, `1b7dd78`). Toolkit manual lockstep: a companion PR in `mnemonic-toolkit` on branch `docs/bughunt-ms-manual`, from `origin/master` `c2b09861`.

Sources, all in `design/agent-reports/`:

- `bughunt-2026-10-31-ms.md` (hunt report, `3b897f3`)
- `bughunt-triage-ms.md` (triage, `0867c4b`)
- `bughunt-grade-ms.md` (grades and product rulings, `4755eb9`)
- `bughunt-triage2-ms.md` (triage round 2, `5399625`; its RED rows are `14b67b2`)
- `bughunt-grade2-ms.md` (round-2 grades and rulings, written with this amendment)

The controller's scope decision is settled: #1, #4, #5 and #7 (Important), plus #3, which the #7 fix closes with no extra code. Round 2 adds T2 and the primer overclaim (both Important, `bughunt-grade2-ms.md`). T1 and T3 are Minor and stay out of this PR. T1b (eight `?` recovered) is feature work, sequenced after this PR by Brian's ruling of 2026-11-01, "Fix bugs before erasure feature work" (9.5).

**How this document was checked.** The worktree was copied, without `.git` and `target/`, to `/scratch/cache/bughunt/ms-grade/scratch`. Every code, test and spec edit below was applied there and nowhere else. The production and test edits are saved as a verified reference patch: `/scratch/cache/bughunt/ms-grade/fix-bughunt-ms-reference-r2.patch` (sha256 `b1f7d1be83a46cddd7b4f9def6d2d346d73c0ab1191f68686ded0cb313d3cd0e`; round 1's `fix-bughunt-ms-reference.patch` is its subset). `git apply --check` of that patch on this worktree (at `5399625`) passes. The toolkit manual edits of section 5 are a script, `/scratch/cache/bughunt/ms-grade/toolkit-manual-edits.py` (sha256 `083063227a5ef1ec8cb015fbe5e82b1e66c31b2bf1fe173af3c3eb182ce7aa09`), which asserts that each old text occurs once.

All runs used `PATH=$HOME/.cargo/bin:$PATH` (cargo 1.85.0, the pin), `--locked`, and no `--release`.

On the copy with the edits:

| check | result |
|---|---|
| `cargo nextest run --locked --workspace` | 667 run, 667 passed, 23 skipped |
| `cargo test --locked --doc --workspace` | 1 passed |
| `cargo clippy --locked --all-targets --workspace -- -D warnings` | exit 0 |
| `cargo fmt --all --check` | exit 0 |
| `cargo +1.95.0 fmt --all -- --check` (CI's fmt toolchain) | exit 0 |
| `bash ci/code-docs-check.sh` (after a throwaway `git init` of the copy) | ok |
| `bash ci/docs-only.test.sh` | exit 0 |

The other direction was checked on a second copy holding only the new test files over unmodified production code (`/scratch/cache/bughunt/ms-grade/scratch-red`). `cargo nextest run --locked --workspace -E 'binary(bughunt)'` there gave 20 run, 9 passed, 11 failed: exactly the 11 rows of section 6 that this build turns GREEN. The 9 passes are the 8 `mlock.rs` unit tests the `#[path]` module brings in, plus #10. Round 2, on the same copy with round 2's rows as edited here: `-E 'binary(bughunt_t)'` gave 1 run, 0 passed, 1 failed (`bughunt_ms_t2_inspect_unknown_language_says_ok`), 3 skipped (the three rows this document ignores).

The toolkit manual lint (`make lint`) was run on two copies of toolkit `origin/master` `c2b09861` `docs/manual`, one as is and one with all of section 5's edits (the script above). Both give the same 10 FAIL lines, all in `44-mk-cli.md` (stale `mk` markers against the locally installed `mk`). Section 5 adds none, and `make anchor-check` passes (9 danglers, matching its baseline). This worktree's `git status` is clean apart from this document.

## 1. Summary

1. **#1 `encode_shares` with `Threshold::ZERO` skipped `encode()`'s tag checks** (Important). It minted strings that `decode` refuses. It now returns exactly what `encode(tag, secret)` returns. This is library-only and has no CLI text.
2. **#4 `ms combine --to` was silently ignored for a preimage share set** (Important). `--to ms1` now prints the `hash` single and `--to entropy` the 64-hex preimage, in text and `--json`. The default `--to phrase` is unchanged: it prints the record exactly as `ms decode` does.
3. **#5 `ms inspect` said "would combine" for the secret share at index `s`** (Important). It now prints `FAIL: would NOT combine` with the reason `share-index-secret`, and `--json` sets `would_decode` and `would_combine` false. Exit 0 as before.
4. **#7 `ms repair` named the wrong problem for a character outside the alphabet** (Important). It said "string length 50 not in v0.1 set [50, …]". It now says `invalid character '<c>' (not in codex32 alphabet)`, naming the character as typed. Exit 1, unchanged.
5. **#3 non-ASCII characters were read as codex32 symbols during correction** (Minor, closed by 4). A non-ASCII character is never a member of the alphabet, so the #7 refusal covers it.
6. **T2 `ms inspect` said "OK: would decode" for a mnem string whose language code `ms decode` refuses** (Important). It now prints `FAIL: would NOT decode v0.1` with the reason `mnem-unknown-language`, and `--json` sets `would_decode` false. Exit 0 as before.
7. **The toolkit manual's BCH primer promised 8-erasure correction** (Important; documentation, companion PR). No tool decodes erasures. The primer now says so, tells a holder not to type `?`, and names two actions that work for up to 4 unreadable characters: leave them out and run `mnemonic repair --max-indel N`, or type a best guess and run `ms repair`.

Not in this PR: T1 (a `?` is refused rather than filled; Minor, and its bug part is #7's message, fixed by 4), T3 (deferred-minor, 9.2) and T1b (planned feature, 9.5).

Nothing else changes for a holder. `--help` texts are unchanged, as are exit codes and every other `inspect`, `combine`, `repair` and `decode` path. Versions also stay unchanged in this PR (see section 8, "Release").

## 2. Holder-visible behaviour, per fix

The commands below ran in `/scratch/cache/bughunt/ms` (the triage's inputs). "Current" is `ms` 0.20.1 built from this worktree with the pin. "After the fix" is the copy built with this document's edits. Both builds print `ms 0.20.1`.

The two input files are test data:
- `preimage-shares.txt`: the two lines `ms12vmruqztk6xc8gwv7spxztdcw0te78amupqh4nsw0cyd5ytgqdyyuktr2ac74fygee9uxswh` and `ms12vmrup8jvav82tscsxjrnmq8p4fssf3hhq99uk5lk6t0tefk5jtyh0f60rg6ukjt2k645scc`, a 2-of-2 split of the dummy preimage X = `0x11` × 32, made with `ms_codec::encode_shares` because `ms split` has no preimage source (F-468).
- `s-index-k2.txt`: `ms12zzzzsqqqqqqqqqqqqqqqqqqqqqqqqqqqqfkawxakz4l4cj`, which is `Codex32String::from_seed("ms", 2, "zzzz", Fe::S, &[0u8; 17])`.
- `mnem-lang10.txt`: `ms10entrsqg9qqqqqqqqqqqqqqqqqqqqqqqqqqfqh998pfxtntx`, which is `Codex32String::from_seed("ms", 0, "entr", Fe::S, &[0x02, 10, 0 × 16])`: a mnem payload with language byte 10.

Notation: `stdout:` and `stderr:` show each stream's lines; "(empty)" means no bytes.

### 2.1 #1 (library only)

**Behaviour.** For `threshold == Threshold::ZERO` and `n == 1`, `encode_shares(tag, ZERO, 1, secret)` returns `encode(tag, secret)` unchanged, error included:
- `ReservedTagNotEmittedInV01 { got }` for a reserved tag (`seed`, `xprv`, `prvk`);
- `TagKindMismatch { tag, prefix }` for `hash` + Entr/Mnem or `entr` + Preimage.

A valid input still returns `vec![single]`, byte-identical to `encode()` as before; `zero_share_is_byte_identical_to_encode_{entr,mnem}` stay green. `n != 1` still returns `InvalidShareCount { k: 0, n }` first. No caller in the constellation passes `Threshold::ZERO` (grep in `bughunt-grade-ms.md`). No CLI path reaches it: `ms split -k 0` refuses earlier.

**Spec sentence:** `design/SPEC_ms_v0_2_kofn.md` line 51 (§2 `encode_shares`). Replace, once:

Old:
~~~text
returns `vec![single_string]` **byte-identical** to `encode(tag, secret)` (both reduce to `from_seed(HRP, 0, tag.as_str(), Fe::S, [prefix]||payload)`, deterministic — the Phase-0 gate).
~~~

New:
~~~text
returns exactly what `encode(tag, secret)` returns: `vec![single_string]` **byte-identical** to it (both reduce to `from_seed(HRP, 0, tag.as_str(), Fe::S, [prefix]||payload)`, deterministic — the Phase-0 gate), or its refusal (`ReservedTagNotEmittedInV01` for a reserved tag, `TagKindMismatch` for a tag that names another kind), never a string `decode` refuses (FIX_bughunt_ms #1).
~~~

**Rustdoc**, `crates/ms-codec/src/shares.rs` (the `encode_shares` doc, first bullet). Replace the two lines `/// - \`threshold == ZERO\`: \`n\` MUST be 1; returns a single string **byte-identical**` and `///   to \`encode(tag, secret)\` — the v0.1 single-string construction` with:
~~~text
/// - `threshold == ZERO`: `n` MUST be 1; returns exactly what `encode(tag, secret)`
///   returns, its refusal included (a reserved tag, or a tag that names another
///   kind, is refused, never minted) — the v0.1 single-string construction
~~~

### 2.2 #4 `ms combine --to` on a preimage share set

**Behaviour:**

| `--to` | text stdout | `--json` stdout | exit |
|---|---|---|---|
| `phrase` (the default, given or not) | unchanged: the preimage record exactly as `ms decode` prints it | unchanged: `ms decode`'s preimage JSON object | 0 |
| `entropy` | one line: the 32-byte preimage as 64 lowercase hex characters | `{"schema_version":"1","kind":"preimage","entropy_hex":"<64 hex>"}` | 0 |
| `ms1` | one line: `ms_codec::encode(Tag::HASH, &payload)`, the 75-character `hash` single, byte-identical to what `ms hashlock --out` writes for that preimage | `{"schema_version":"1","kind":"preimage","ms1":"<that string>","entropy_hex":"<64 hex>"}` | 0 |

- stderr carries exactly the existing PrivateKeyMaterial advisory line, as on every `combine` path. There is no language advisory and no engraving card.
- The JSON is `combine`'s existing `CombineJson` (field order `schema_version`, `kind`, `ms1`, `entropy_hex`; absent fields skipped).
- An explicit `--to phrase` is indistinguishable from the default. It keeps printing the record, whose first line says `kind: preimage`; that case is covered by documentation only (section 5).
- Entr and mnem share sets are unchanged.

Current:
~~~text
$ ms combine --to ms1 --in preimage-shares.txt
stdout: kind:      preimage (hashlock, 32 bytes / 64 hex characters)
stdout: preimage:  1111111111111111111111111111111111111111111111111111111111111111
stdout: digests:
stdout:   sha256     02d449a31fbb267c8f352e9968a79e3e5fc95c1bbeaa502fd6454ebde5a4bedc
stdout:   hash256    59420d36b80353ed5a5822ca464cc9bffb8abe9cd63959651d3cd85a8252d83f
stdout:   ripemd160  e5a0500d3009d803632658b21646518eb42b8433
stdout:   hash160    4d5209774e22ea63e2987579562d9a4ea3dab623
stdout:            (a preimage carries no hash kind; match the one your wallet uses)
stderr: warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
exit=0
~~~

`ms combine --to entropy --in preimage-shares.txt` prints the same eight lines (exit 0). `--to ms1 --json` and `--to entropy --json` each print `ms decode`'s preimage object (exit 0):

~~~text
{"digest":"02d449a31fbb267c8f352e9968a79e3e5fc95c1bbeaa502fd6454ebde5a4bedc","digest_kind":"sha256","digests_by_kind":{"hash160":"4d5209774e22ea63e2987579562d9a4ea3dab623","hash256":"59420d36b80353ed5a5822ca464cc9bffb8abe9cd63959651d3cd85a8252d83f","ripemd160":"e5a0500d3009d803632658b21646518eb42b8433","sha256":"02d449a31fbb267c8f352e9968a79e3e5fc95c1bbeaa502fd6454ebde5a4bedc"},"kind":"preimage","preimage_hex":"1111111111111111111111111111111111111111111111111111111111111111"}
~~~

After the fix:
~~~text
$ ms combine --to ms1 --in preimage-shares.txt
stdout: ms10hashsqvg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyl0sajh7rfj2z
stderr: warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
exit=0

$ ms combine --to entropy --in preimage-shares.txt
stdout: 1111111111111111111111111111111111111111111111111111111111111111
stderr: warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
exit=0

$ ms combine --to ms1 --json --in preimage-shares.txt
stdout: {"schema_version":"1","kind":"preimage","ms1":"ms10hashsqvg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyl0sajh7rfj2z","entropy_hex":"1111111111111111111111111111111111111111111111111111111111111111"}
stderr: (the same advisory line)
exit=0

$ ms combine --to entropy --json --in preimage-shares.txt
stdout: {"schema_version":"1","kind":"preimage","entropy_hex":"1111111111111111111111111111111111111111111111111111111111111111"}
stderr: (the same advisory line)
exit=0
~~~

`ms combine --in preimage-shares.txt` and `ms combine --json --in preimage-shares.txt` print byte for byte what they print today; `cmp` against the default confirmed `--to phrase` identical on both builds.

The printed `hash` single was checked on the fixed build, after `ms combine --to ms1 --in preimage-shares.txt > combined-hash.ms1`:
- It is byte-identical to what `printf '1111111111111111111111111111111111111111111111111111111111111111\n' | ms hashlock --kind sha256 --hex - --out hashlock-out.ms1` writes (`cmp`: identical).
- `ms hashlock --kind sha256 --in combined-hash.ms1` reads it (exit 0; stdout `hash:02d449a31fbb267c8f352e9968a79e3e5fc95c1bbeaa502fd6454ebde5a4bedc`).
- `ms decode --in combined-hash.ms1` prints the preimage record.

**Spec sentence:** `design/SPEC_ms_hashlock.md` line 502 (§5 "The other verbs on the new kind"). Replace the whole row, once:

Old:
~~~text
| `combine` | a recovered preimage share set prints as `decode` does |
~~~

New:
~~~text
| `combine` | a recovered preimage share set prints as `decode` does under `--to phrase` (the default); `--to entropy` prints the 32-byte preimage as 64 hex characters and `--to ms1` prints the `hash` single, byte-identical to what `ms hashlock --out` writes; under `--json` those two print `combine`'s object with `kind` `preimage` (FIX_bughunt_ms #4) |
~~~

Line 285 (the H1 edit-site table row for `cmd/combine.rs:166`) is unchanged: it records the H1 build's sites.

### 2.3 #5 `ms inspect` of a share-threshold string at index `s`

**Behaviour.** When the report's threshold is `2..=9` and `share_index == 's'`, the following hold. The report's fields are canonical lowercase, so an all-uppercase string behaves identically.
- Text: line 1 is `FAIL: would NOT combine`, and line 2 is `    reason: share-index-secret (share-index 's' is the secret itself, not a distributed share; ms combine refuses it)` (four spaces, as on a single's reason line). Then a blank line and the unchanged field block.
- JSON: `would_decode` false, `would_combine` false, `failure_reasons` `["share-index-secret"]`. The `report` object is unchanged.
- Exit 0, as for every `inspect` verdict.

Every other share is unchanged (`OK: K-of-N share (would combine: needs k shares)`; `true`, `true`, `[]`). Every single is unchanged.

Current:
~~~text
$ ms inspect --in s-index-k2.txt
stdout: OK: K-of-N share (would combine: needs 2 shares)
stdout: (blank)
stdout: hrp: ms
stdout: threshold: 2
stdout: id: zzzz
stdout: index: s
stdout: checksum_valid: true
stdout: kind: share
exit=0

$ ms inspect --json --in s-index-k2.txt
stdout: {"failure_reasons":[],"report":{"checksum_valid":true,"hrp":"ms","kind":"share","share_index":"s","tag":"zzzz","threshold":2},"schema_version":"1","would_combine":true,"would_decode":true}
exit=0

$ tr a-z A-Z < s-index-k2.txt | ms inspect --json -
stdout: (the same line as --json above)
exit=0

$ ms combine --in s-index-k2.txt
stdout: (empty)
stderr: error: the secret share (index 's') must not be combined; supply only distributed shares (the secret is the recovery target)
exit=2
~~~

After the fix (`ms combine` unchanged):
~~~text
$ ms inspect --in s-index-k2.txt
stdout: FAIL: would NOT combine
stdout:     reason: share-index-secret (share-index 's' is the secret itself, not a distributed share; ms combine refuses it)
stdout: (blank)
stdout: hrp: ms
stdout: threshold: 2
stdout: id: zzzz
stdout: index: s
stdout: checksum_valid: true
stdout: kind: share
exit=0

$ ms inspect --json --in s-index-k2.txt
stdout: {"failure_reasons":["share-index-secret"],"report":{"checksum_valid":true,"hrp":"ms","kind":"share","share_index":"s","tag":"zzzz","threshold":2},"schema_version":"1","would_combine":false,"would_decode":false}
exit=0

$ tr a-z A-Z < s-index-k2.txt | ms inspect --json -
stdout: (the same line as --json above)
exit=0
~~~

**Spec sentence:** `design/SPEC_ms_v0_2_kofn.md` line 86 (§3 `ms inspect <share>`). Replace, once:

Old:
~~~text
`InspectReport` already carries the needed fields.
~~~

New:
~~~text
`InspectReport` already carries the needed fields. **Exception (FIX_bughunt_ms #5):** a share-threshold string at index `s` is the secret-at-S, which `combine` refuses (`SecretShareSuppliedToCombine`); `inspect` reports it as `FAIL: would NOT combine` with reason `share-index-secret` (`--json`: `would_decode` and `would_combine` false), exit 0.
~~~

**Doc comment**, `crates/ms-cli/src/cmd/inspect.rs` `emit_share_json`. Replace `` `would_decode: true` carries the "would combine" semantics (a share is a valid read). `` with `` `would_decode` carries the "would combine" semantics: true for a distributed share, false for the secret at index `s` (reason `share-index-secret`). ``

### 2.4 #7 (closes #3) — a character outside the alphabet during correction

**Behaviour.** In `decode_with_correction` (`ms repair`, and every library caller), the first data-part character that is not in the codex32 alphabet ends the parse with `Error::Codex32(codex32::Error::InvalidChar(c))`:
- `c` is the character as typed: it is taken from the input, not from a lowercased copy.
- A character is a member only if it is ASCII and its ASCII lowercase is in the alphabet. A non-ASCII character is never a member and is never truncated to a byte.

CLI rendering, using the existing `friendly_codex32` text:
- text mode: `error: invalid character '<c>' (not in codex32 alphabet)` on stderr, stdout empty, exit 1;
- `--json`: `{"schema_version":"1","error":{"kind":"Codex32","message":"invalid character '<c>' (not in codex32 alphabet)","exit_code":1,"details":null}}`.

The exit code is 1 before and after. For a letter outside the alphabet, `ms repair` now names the same character `ms decode` names. For `?` in a lowercase string, `ms decode` answers with its case check instead (filed as `ms-decode-calls-punctuation-mixed-case`, 9.3). Mixed case is unchanged; that is X1, deferred.

**`?` (T1).** A `?` gets this refusal too: `error: invalid character '?' (not in codex32 alphabet)`, exit 1 (shown below). That is the correct refusal until erasure decoding exists (9.5). T1's row asserts recovery, so it is the feature's acceptance row and is ignored (section 6); the refusal is pinned by `bughunt_ms_07_codec_non_alphabet_is_invalid_char` and `bughunt_ms_07b_repair_names_char_as_typed`.

Current:
~~~text
$ echo ms10entrsqz4b42424242424242424242424q8lurdrzjnyn08 | ms repair --ms1 -
stdout: (empty)
stderr: error: string length 50 not in v0.1 set [50, 56, 62, 69, 75]
exit=1

$ echo ms10entrsqz4b42424242424242424242424q8lurdrzjnyn08 | ms repair --json --ms1 -
stdout: {"schema_version":"1","error":{"kind":"UnexpectedStringLength","message":"string length 50 not in v0.1 set [50, 56, 62, 69, 75]","exit_code":1,"details":{"allowed":[50,56,62,69,75],"got":50}}}
exit=1

$ echo MS10ENTRSQZ4B42424242424242424242424Q8LURDRZJNYN08 | ms repair --ms1 -
stderr: error: string length 50 not in v0.1 set [50, 56, 62, 69, 75]
exit=1

$ echo 'ms10entrsqz4?42424242424242424242424q8lurdrzjnyn08' | ms repair --ms1 -
stderr: error: string length 50 not in v0.1 set [50, 56, 62, 69, 75]
exit=1

$ echo MS10ENTRSQZ4B42424242424242424242424Q8LURDRZJNYN08 | ms decode -
stderr: error: invalid character 'B' (not in codex32 alphabet)
exit=1

$ printf 'ms10entrsqz4\xc4\xb342424242424242424242424q8lurdrzjnyn08\n' | ms repair --ms1 -
stdout: # Repair report
stdout: #   ms1 chunk 0: 1 correction at position 9: '3' -> '2'
stdout: ms10entrsqz4242424242424242424242424q8lurdrzjnyn08
stderr: warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
stderr: repair: correction UNVERIFIED — a corrected seed card cannot be self-verified; confirm the derived address/xpub against a known-good copy before use; BIP-93 recommends confirming a corrected codex32 string
exit=4
~~~

After the fix:
~~~text
$ echo ms10entrsqz4b42424242424242424242424q8lurdrzjnyn08 | ms repair --ms1 -
stdout: (empty)
stderr: error: invalid character 'b' (not in codex32 alphabet)
exit=1

$ echo ms10entrsqz4b42424242424242424242424q8lurdrzjnyn08 | ms repair --json --ms1 -
stdout: {"schema_version":"1","error":{"kind":"Codex32","message":"invalid character 'b' (not in codex32 alphabet)","exit_code":1,"details":null}}
exit=1

$ echo MS10ENTRSQZ4B42424242424242424242424Q8LURDRZJNYN08 | ms repair --ms1 -
stderr: error: invalid character 'B' (not in codex32 alphabet)
exit=1

$ echo 'ms10entrsqz4?42424242424242424242424q8lurdrzjnyn08' | ms repair --ms1 -
stderr: error: invalid character '?' (not in codex32 alphabet)
exit=1

$ printf 'ms10entrsqz4\xc4\xb342424242424242424242424q8lurdrzjnyn08\n' | ms repair --ms1 -
stdout: (empty)
stderr: error: invalid character 'ĳ' (not in codex32 alphabet)
exit=1

$ printf 'ms10entrsqz4\xc4\xb342424242424242424242424q8lurdrzjnyn08\n' | ms repair --json --ms1 -
stdout: {"schema_version":"1","error":{"kind":"Codex32","message":"invalid character 'ĳ' (not in codex32 alphabet)","exit_code":1,"details":null}}
exit=1
~~~

**Spec sentence:** none. No spec in `design/` states the error for this case; the D29 error-mapping table lives in the toolkit's v0.22.x plan and is not in this repo. The function's rustdoc (`decode.rs` lines 170-172: "or [`Error::Codex32`] (via a `crate::codex32::Error::InvalidChar`) if any data-part character is not in the codex32 alphabet") already states the new behaviour and stays as is.

The implementation comment above the loop (`decode.rs` lines 200-206, "Non-alphabet characters can't appear … We can't fabricate a `crate::codex32::Error` value here …") is false since codex32 was vendored, and is replaced by:
~~~text
// Every data-part character must be in the codex32 alphabet. The first one
// that is not is refused as `InvalidChar`, named as the caller typed it
// (not lowercased), so `ms repair` names the same character `decode` does.
// A non-ASCII character is never a member: it is never truncated to a
// byte (which read U+0133 as '3'). `lower` starts with the ASCII "ms1", so
// byte 3 of `s` is a char boundary.
~~~

### 2.5 T2 `ms inspect` of a mnem string with an unknown language code

**Behaviour.** For a single (threshold `0`) of kind mnem whose language code is not one `ms` knows (10 or more), which `ms decode` refuses with `MnemUnknownLanguage`:
- `analyze()` pushes the reason `mnem-unknown-language` in rule 10's mnem arm, before `payload-length-mismatch` (the order `Payload::validate` checks them in).
- Text: line 1 is the existing `FAIL: would NOT decode v0.1`, and line 2 is `    reason: mnem-unknown-language (unknown mnem wordlist-language code; known codes are 0-9)`. The field block is unchanged and still ends `language: unknown`.
- JSON: `would_decode` false, `failure_reasons` `["mnem-unknown-language"]`. The `report` object is unchanged.
- Exit 0, as for every `inspect` verdict.

A mnem string with a known code (0-9) is unchanged. The header "would NOT decode v0.1" is the existing FAIL wording for every single and is not changed here.

Current:
~~~text
$ ms inspect --in mnem-lang10.txt
stdout: OK: would decode v0.2
stdout: (blank)
stdout: hrp: ms
stdout: threshold: 0
stdout: tag: entr
stdout: share_index: s
stdout: prefix_byte: 0x02
stdout: payload_bytes: 0a00000000000000000000000000000000
stdout: checksum_valid: true
stdout: kind: mnem
stdout: language: unknown
exit=0

$ ms inspect --json --in mnem-lang10.txt
stdout: {"schema_version":"1","report":{"hrp":"ms","threshold":0,"tag":"entr","share_index":"s","prefix_byte":2,"payload_bytes_hex":"0a00000000000000000000000000000000","checksum_valid":true,"kind":"mnem","language":"unknown"},"would_decode":true,"failure_reasons":[]}
exit=0

$ ms decode --in mnem-lang10.txt
stderr: error: unhandled ms_codec::Error variant: Error("unknown mnem wordlist-language code: 10")
exit=1
~~~

After the fix (`ms decode` unchanged: its text is the deferred `ms-cli-mnem-unknown-language-catch-all`):
~~~text
$ ms inspect --in mnem-lang10.txt
stdout: FAIL: would NOT decode v0.1
stdout:     reason: mnem-unknown-language (unknown mnem wordlist-language code; known codes are 0-9)
stdout: (blank)
stdout: hrp: ms
stdout: threshold: 0
stdout: tag: entr
stdout: share_index: s
stdout: prefix_byte: 0x02
stdout: payload_bytes: 0a00000000000000000000000000000000
stdout: checksum_valid: true
stdout: kind: mnem
stdout: language: unknown
exit=0

$ ms inspect --json --in mnem-lang10.txt
stdout: {"schema_version":"1","report":{"hrp":"ms","threshold":0,"tag":"entr","share_index":"s","prefix_byte":2,"payload_bytes_hex":"0a00000000000000000000000000000000","checksum_valid":true,"kind":"mnem","language":"unknown"},"would_decode":false,"failure_reasons":["mnem-unknown-language"]}
exit=0
~~~

**Spec sentence:** `design/SPEC_ms_mnem_wordlist_language.md` line 49 (Phase 2 CLI, `ms inspect`). Replace, once:

Old:
~~~text
- **`ms inspect`** — shows `kind: mnem` + the decoded language for `0x02`.
~~~

New:
~~~text
- **`ms inspect`** — shows `kind: mnem` + the decoded language for `0x02`. **Unknown code (FIX_bughunt_ms T2):** a language code `decode` refuses (`MnemUnknownLanguage`: 10-15, or a non-zero high nibble) prints `language: unknown` and FAILs with reason `mnem-unknown-language` (`--json`: `would_decode` false), exit 0.
~~~

`design/SPEC_ms_cli_v0_1.md` §2.3's tag table stays as is. It is the v0.1 set: it already lacks `tag-kind-mismatch` (added by SPEC_ms_hashlock), and §5.3 says "v0.2 may add tags additively". The new tags are specified where their rules live: `share-index-secret` in SPEC_ms_v0_2_kofn (2.3) and `mnem-unknown-language` here.

### 2.6 The BCH primer's erasure claims (toolkit manual, companion PR)

**Behaviour:** no tool changes. The primer stops promising erasure correction, tells a holder not to type `?`, and names two actions that work today. The exact texts are in 5.6. Every command the new text prints was run on the installed tools (`mnemonic 0.105.1`, `ms 0.20.1`), with the 50-character card `ms10entrsqqqqzqsrqszsvpcgpy9qkrqdpc8sh9lf92n5fr33s` (entropy `000102…0f`, test data):

~~~text
$ cat card.txt                       (character 17 left out)
ms10entrsqqqqzqsrszsvpcgpy9qkrqdpc8sh9lf92n5fr33s
$ mnemonic repair --ms1 - --max-indel 1 < card.txt
stdout: ms10entrsqqqqzqsrqszsvpcgpy9qkrqdpc8sh9lf92n5fr33s
stderr: warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
exit=5

$ cat card.txt                       (character 17 guessed wrong as z)
ms10entrsqqqqzqsrzszsvpcgpy9qkrqdpc8sh9lf92n5fr33s
$ ms repair --in card.txt
stdout: # Repair report
stdout: #   ms1 chunk 0: 1 correction at position 14: 'z' -> 'q'
stdout: ms10entrsqqqqzqsrqszsvpcgpy9qkrqdpc8sh9lf92n5fr33s
stderr: warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
stderr: repair: correction UNVERIFIED — a corrected seed card cannot be self-verified; confirm the derived address/xpub against a known-good copy before use; BIP-93 recommends confirming a corrected codex32 string
exit=4

$ cat card.txt                       (character 17 typed as ?)
ms10entrsqqqqzqsr?szsvpcgpy9qkrqdpc8sh9lf92n5fr33s
$ mnemonic repair --ms1 - < card.txt
stderr: error: repair: chunk 0 parse failed before correction could run: non-bech32 char '?' at position 14
exit=2
$ ms repair --in card.txt            (current)
stderr: error: string length 50 not in v0.1 set [50, 56, 62, 69, 75]
exit=1
$ ms repair --in card.txt            (after the fix)
stderr: error: invalid character '?' (not in codex32 alphabet)
exit=1
~~~

The limits stated in 5.6 were measured:
- Leaving characters out: `mnemonic repair --max-indel 3` and `--max-indel 4` recovered 40/40 random sets of 3 and of 4 left out (exit 5; `/scratch/cache/bughunt/ms-grade/probe/indel_probe.txt`). The controller's probe recovered 1 (47/47) and 2 (60/60) (`/scratch/cache/bughunt/ms2/controller-indel-probe.txt`).
- Guessing: `ms repair` recovered 1, 2, 3 and 4 wrong guesses (47/47, then 60/60 each, exit 4) and refused 5 (60/60, exit 2) (same probe file).

## 3. Code delta (production)

The reference patch is the verified implementation. The implementer may write the code differently; the texts, behaviours and constraints below are binding.

1. `crates/ms-codec/src/shares.rs` `encode_shares`, ZERO branch: after the `n != 1` check, `return Ok(vec![crate::encode::encode(tag, secret)?]);`. This is `fix-01.patch` unchanged. Add the rustdoc of 2.1.
2. `crates/ms-codec/src/decode.rs` `parse_ms1_symbols`:
   - The data part is read from the original `s` after `HRP_PREFIX`, not from `lower`.
   - Membership is "`c.is_ascii()` and `c.to_ascii_lowercase()` is in `CODEX32_ALPHABET`".
   - The error is `Error::Codex32(crate::codex32::Error::InvalidChar(c))`.
   - The HRP check and the `WrongHrp` path are unchanged.
   - Replace the comment as in 2.4.
3. `crates/ms-cli/src/cmd/combine.rs`:
   - The `Payload::Preimage(x)` arm returns `crate::cmd::decode::emit_preimage(x, args.json)` only when `args.to == CombineTo::Phrase`. Otherwise it yields `(Zeroizing::new(x.to_vec()), CliLanguage::English, "preimage")` and falls through to the existing `match args.to`.
   - `emit_ms1` encodes with `payload.kind().single_tag()` instead of the hard-coded `Tag::ENTR`. (`Payload::kind` and `PayloadKind::single_tag` are public; entr/mnem still get `Tag::ENTR`.)
   - Constraint: the ms-cli `_ => unreachable!` census stays 4 (`hashlock_other_verbs.rs::unreachable_catch_all_count_is_pinned`).
4. `crates/ms-cli/src/cmd/inspect.rs`:
   - A predicate `share_index == 's'` drives `emit_share_text` and `emit_share_json` as in 2.3.
   - `reason_text` gains the arm `"share-index-secret" => "share-index 's' is the secret itself, not a distributed share; ms combine refuses it"`.
   - Add the doc comment of 2.3.
5. `crates/ms-cli/src/cmd/inspect.rs` `analyze()`, rule 10's `InspectKind::Mnem` arm (T2): when `report.language` is a code with no entry in `MNEM_LANGUAGE_NAMES` (10 or more), push `"mnem-unknown-language"` before the payload-length check. `reason_text` gains the arm `"mnem-unknown-language" => "unknown mnem wordlist-language code; known codes are 0-9"`.

## 4. Spec delta (this repo)

1. `design/SPEC_ms_v0_2_kofn.md` line 51: the replacement in 2.1.
2. `design/SPEC_ms_v0_2_kofn.md` line 86: the replacement in 2.3.
3. `design/SPEC_ms_hashlock.md` line 502: the replacement in 2.2.
4. `design/SPEC_ms_mnem_wordlist_language.md` line 49: the replacement in 2.5.

Each old text occurs exactly once (checked by script on the copy). No other spec, `MIGRATION.md` or `CHANGELOG.md` text changes in this PR.

## 5. Manual delta (companion PR, `mnemonic-toolkit`, branch `docs/bughunt-ms-manual`)

File `docs/manual/src/40-cli-reference/43-ms.md` at `origin/master` `c2b09861`. It is identical to the toolkit's local `master` for this file.

**5.1** Line 293, the `ms combine` flag row `--to <T>`. Replace the whole line.

Old:
~~~text
| `--to <T>` | output form for the recovered secret (default `phrase`): `phrase` (BIP-39 mnemonic — `mnem` → wire language, `entr` → English), `entropy` (raw entropy hex), or `ms1` (a single unshared ms1 string) |
~~~

New:
~~~text
| `--to <T>` | output form for the recovered secret (default `phrase`): `phrase` (BIP-39 mnemonic — `mnem` → wire language, `entr` → English), `entropy` (raw entropy hex), or `ms1` (a single unshared ms1 string). For a hashlock **preimage** share set, `phrase` prints the preimage record exactly as `ms decode` does (never words), `entropy` prints the 32-byte preimage as 64 hex characters, and `ms1` prints the 75-character `hash` single (`ms10hash…`) that `ms decode` and `ms hashlock --in` read |
~~~

**5.2** Line 337, the `kind: share` bullet of `ms inspect`. Replace, once:

Old:
~~~text
and exits 0 — it does NOT print a `threshold-not-zero` length/decode FAIL.
~~~

New:
~~~text
and exits 0 — it does NOT print a `threshold-not-zero` length/decode FAIL. One exception: a share-threshold string at index `s` is the secret itself, not a distributed share, and `ms combine` refuses it; for it `ms inspect` prints `FAIL: would NOT combine` with the reason `share-index-secret` (under `--json`: `would_decode` and `would_combine` false, `failure_reasons: ["share-index-secret"]`), and still exits 0.
~~~

**5.3** No other manual text changes besides 5.5 and 5.6. Grep run at `c2b09861`: `git grep -n -E "ms combine|ms inspect" origin/master -- docs/manual/src`. Outside `43-ms.md` it finds `50-comparing/52-toolkit-vs-ms-cli.md:13` (inspect in general) and `40-cli-reference/41-mnemonic.md:2612` (a link to `ms split`/`ms combine`); both stay true.

For #1 and #7 there is no manual text. `git grep -n -E "string length|v0\.1 set|invalid character|encode_shares" origin/master -- docs/manual/src/40-cli-reference/43-ms.md` finds nothing. The `ms repair` exit-code table (line 780 onward) already gives 1 for "other generic failure" and stays as is.

**5.4** The companion PR also appends the toolkit FOLLOWUPS entry of 9.4.

**5.5** (T2) Line 336, the `kind: mnem` bullet of `ms inspect`. Replace the whole line.

Old:
~~~text
- `kind: mnem` — language-tagged payload (ms-cli v0.2+ / ms-codec 0.3.0+); the `language` field is also present, naming the stored wordlist (e.g. `japanese`).
~~~

New:
~~~text
- `kind: mnem` — language-tagged payload (ms-cli v0.2+ / ms-codec 0.3.0+); the `language` field is also present, naming the stored wordlist (e.g. `japanese`). A language code this `ms` does not know prints `language: unknown`, and the verdict is `FAIL` with the reason `mnem-unknown-language` (under `--json`: `would_decode` false), because `ms decode` refuses the string.
~~~

**5.6** (primer) `docs/manual/src/60-appendices/65-bch-codex-primer.md` at `c2b09861`: seven replacements, each of a text that occurs once.

**5.6.1 Lines 21-25, "What a BCH code does", the "few errors" bullet.**

Old:
~~~text
- If a few errors: the polynomial syndrome *locates* the error
  positions. BIP-93 guarantees correction of up to 4 unknown-position
  substitutions, or up to 8 errors at known positions ("erasures"),
  or up to 13 consecutive erasures. See *Error detection and
  correction guarantees per card* below for the precise table.
~~~

New:
~~~text
- If a few errors: the polynomial syndrome *locates* the error
  positions. BIP-93's code can correct up to 4 unknown-position
  substitutions, or up to 8 errors at known positions ("erasures"),
  or up to 13 consecutive erasures. The repair tools implement the
  first of these only: up to 4 wrong characters in a string. See
  *Error detection and correction guarantees per card* below for
  the precise table, and *Unreadable characters: what to type* for
  a character you cannot read.
~~~

**5.6.2 Line 108, the regular-code table row for erasures.**

Old:
~~~text
| Correction — erasures (known positions, e.g., `?`) | up to **8** erasures corrected | BIP-93 §"Error Correction" |
~~~

New:
~~~text
| Correction — erasures (known positions) | up to **8** erasures, by the code; no repair tool decodes erasures yet (see *Unreadable characters: what to type*) | BIP-93 §"Error Correction" |
~~~

**5.6.3 Line 109, the regular-code table row for consecutive erasures.**

Old:
~~~text
| Correction — consecutive erasures (a single scratch) | up to **13** consecutive erasures corrected | BIP-93 §"Error Correction" |
~~~

New:
~~~text
| Correction — consecutive erasures (a single scratch) | up to **13** consecutive erasures, by the code; no repair tool decodes erasures yet | BIP-93 §"Error Correction" |
~~~

**5.6.4 Line 121, the long-code table row.**

Old:
~~~text
| Correction (substitutions / erasures / consecutive erasures) | same as regular code: 4 / 8 / 13 | BIP-93 §"Error Correction" applied to the longer code |
~~~

New:
~~~text
| Correction (substitutions / erasures / consecutive erasures) | same as regular code: 4 / 8 / 13 by the code; the repair tools implement the 4 substitutions only | BIP-93 §"Error Correction" applied to the longer code |
~~~

**5.6.5 Lines 154-156, the definition of an erasure.**

Old:
~~~text
An "erasure" in the table above means *the character at a known
position is unreadable, but the position itself is preserved*
(typically rendered `?`). The string length is unchanged.
~~~

New:
~~~text
An "erasure" in the table above means *the character at a known
position is unreadable, but the position itself is preserved*.
BIP-93 suggests marking one with `?`, but `ms repair` and
`mnemonic repair` refuse a `?`; see *Unreadable characters: what to
type* below.
~~~

**5.6.6 Before line 192 (`## Hand-decodability of ms1 via the bundled codex32 paper-computer`), a new subsection. Replace the heading line (with its preceding blank line) by the subsection followed by the heading.**

Old:
~~~text
## Hand-decodability of ms1 via the bundled codex32 paper-computer
~~~

New:
~~~text
### Unreadable characters: what to type

No repair tool decodes erasures yet, so do not type `?` for a
character you cannot read. Two things work, for up to 4 unreadable
characters in one string (fewer if the string has other mistakes):

- **Leave each unreadable character out** and let `mnemonic repair`
  search for the gaps, telling it how many you left out (1 to 4).
  Put the string, with those characters left out, in `card.txt`:

  ```sh
  mnemonic repair --ms1 - --max-indel 1 < card.txt
  ```

  A unique recovery prints the full string and exits 5; several
  equally valid candidates are all printed, exit 4, and you choose.
  `--mk1` and `--md1` take the same search; see `--max-indel` and
  `--max-subst` in the `mnemonic repair` reference.
- **Type your best guess** for each unreadable character: any
  character of the card's alphabet, in the same case as the rest.
  `ms repair --in card.txt` (or `mnemonic repair`) then corrects up
  to 4 wrong characters in total; a corrected string is a candidate,
  exit 4, which you confirm before use (exit 0 means every guess was
  right).

Either way, compare the result with the plate before you rely on it.

## Hand-decodability of ms1 via the bundled codex32 paper-computer
~~~

**5.6.7 Lines 281-282, "Operational fallbacks".**

Old:
~~~text
When stamping errors exceed the per-card correction radius (4
substitutions or 8 erasures in known positions), the fall-backs are:
~~~

New:
~~~text
When stamping errors exceed what the repair tools can fix (4 wrong
or unreadable characters in one string; see *Unreadable characters:
what to type* above), the fall-backs are:
~~~

The two commands printed in 5.6.6 are the ones run in 2.6. `make lint` and `make anchor-check` on the edited copy add no finding (header of this document).

## 6. RED rows

Round 1's rows are in `crates/ms-codec/tests/bughunt.rs` and `crates/ms-cli/tests/bughunt.rs`; round 2's in the two `bughunt_t.rs` files. "RED today" quotes the failure on unmodified production code (from `/scratch/cache/bughunt/ms-grade/red-nextest.txt` and `red2-nextest.txt`). "After" is the exact assertion; each passed on the fixed copy.

| row | after the build | exact assertion | RED today |
|---|---|---|---|
| codec `bughunt_ms_01_zero_threshold_skips_tag_checks` | GREEN (tightened) | `encode_shares(Tag::try_new("seed").unwrap(), Threshold::ZERO, 1, &Payload::Entr(vec![0; 16]))` matches `Err(Error::ReservedTagNotEmittedInV01 { got })` with `got == *b"seed"`; `encode_shares(Tag::HASH, ZERO, 1, &Entr(vec![0; 16]))` matches `Err(Error::TagKindMismatch { tag, prefix: 0x00 })` with `tag == *b"hash"`; `encode_shares(Tag::ENTR, ZERO, 1, &Preimage([0; 32]))` matches `Err(Error::TagKindMismatch { tag, prefix: 0x03 })` with `tag == *b"entr"`; control: `encode_shares(Tag::HASH, ZERO, 1, &Preimage([0x11; 32])).unwrap() == vec![encode(Tag::HASH, &same).unwrap()]` | `seed tag + Entr: want ReservedTagNotEmittedInV01, got Ok(["ms10seedsqqqqqqqqqqqqqqqqqqqqqqqqqqqqpscrjek0cdvwc"])` |
| codec `bughunt_ms_03_non_ascii_read_as_symbol` | GREEN (tightened; closed by #7) | for U+0133 and U+0130 at byte 12 of the entr-16 card: `decode_with_correction(&bad)` matches `Err(Error::Codex32(codex32::Error::InvalidChar(c)))` with `c == alias` | `U+0133 (reads as '3') accepted by decode_with_correction and repaired: [(9, '3', '2')]` |
| codec `bughunt_ms_07_codec_non_alphabet_is_invalid_char` | NEW, GREEN | at byte 12 of the entr-16 card: lowercase `b`, `B` in the all-uppercase card, and `?` each give `Err(Error::Codex32(InvalidChar(c)))` from `decode_with_correction` with `c` = `'b'`, `'B'`, `'?'`; for `b` and `B`, `decode()` gives the same error with the same `c` | `lowercase b: want InvalidChar('b'), got Err(Error("string length 50 outside v0.1 set [50, 56, 62, 69, 75]"))` |
| CLI `bughunt_ms_04_combine_to_ms1_ignored_for_preimage` | GREEN (tightened) | input: `encode_shares(Tag::HASH, Threshold::new(2), 2, &Preimage([0x11; 32]))` joined by `\n` on stdin; `ms combine --to ms1 -` exits 0 and stdout == `encode(Tag::HASH, &Preimage([0x11; 32])) + "\n"` (`ms10hashsqvg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyl0sajh7rfj2z\n`), a string that starts `ms10hashs` and is 75 characters | stdout is the 8-line record (`left: "kind:      preimage …"`) |
| CLI `bughunt_ms_04b_combine_to_entropy_preimage` | NEW, GREEN | same input; `ms combine --to entropy -` exits 0, stdout == `"11"` × 32 + `"\n"` | stdout is the record |
| CLI `bughunt_ms_04c_combine_to_ms1_json_preimage` | NEW, GREEN | `ms combine --to ms1 --json -` exits 0, stdout == the `--to ms1 --json` line of 2.2 + `"\n"` | stdout is `ms decode`'s preimage object |
| CLI `bughunt_ms_04d_combine_to_entropy_json_preimage` | NEW, GREEN | `ms combine --to entropy --json -` exits 0, stdout == the `--to entropy --json` line of 2.2 + `"\n"` | stdout is `ms decode`'s preimage object |
| CLI `bughunt_ms_05_inspect_would_combine_unchecked` | GREEN (tightened) | preconditions: `ms combine -` on the `s-index-k2.txt` string exits 2 with stderr containing `secret share`; `ms inspect --json -` exits 0 with stdout == the after-fix `--json` line of 2.3 + `"\n"`; `ms inspect -` exits 0 with stdout == the nine after-fix text lines of 2.3, each ending `\n` (line 3 empty) | `left: "{\"failure_reasons\":[],…\"would_combine\":true,\"would_decode\":true}\n"` |
| CLI `bughunt_ms_05b_inspect_uppercase_secret_share` | NEW, GREEN | the same string `.to_uppercase()`: identical `--json` stdout and identical text stdout (the fields print lowercase: `hrp: ms`, `id: zzzz`, `index: s`) | as 05 |
| CLI `bughunt_ms_07_string_length_error_contradicts_itself` | GREEN (tightened) | `ms repair --json --ms1 -` on `ms10entrsqz4b42424242424242424242424q8lurdrzjnyn08`: exit 1, `error.kind == "Codex32"`, `error.message == "invalid character 'b' (not in codex32 alphabet)"`, `error.details` null | `left: String("UnexpectedStringLength") right: "Codex32"` |
| CLI `bughunt_ms_07b_repair_names_char_as_typed` | NEW, GREEN | `--json` on `MS10ENTRSQZ4B42424242424242424242424Q8LURDRZJNYN08`, `ms10entrsqz4?42424242424242424242424q8lurdrzjnyn08` and the same with U+0133 at byte 12: each `(exit, kind, message)` == `(1, "Codex32", "invalid character '<c>' (not in codex32 alphabet)")` with `<c>` = `B`, `?`, `ĳ`; text mode on the `?` string: exit 1, stdout empty, stderr == `error: invalid character '?' (not in codex32 alphabet)\n` | `left: (Some(1), Some("UnexpectedStringLength"), Some("string length 50 not in v0.1 set [50, 56, 62, 69, 75]"))` |
| CLI `bughunt_ms_10_gui_schema_combine_shares_not_repeating` | unchanged, passes | (FALSE POSITIVE, kept as a pin) | passes today |
| CLI `bughunt_ms_02_mlock_unpin_not_refcounted` | `#[ignore = "deferred-minor: ms-mlock-unpin-not-refcounted"]`, body unchanged | — | still fails when run ignored |
| codec `bughunt_ms_06_long_hrp_short_data_panics` | `#[ignore = "deferred-minor: ms-codex32-parts-short-data-panic"]`, body unchanged | — | still fails when run ignored |
| codec `bughunt_ms_08_too_many_errors_bound_is_8` | `#[ignore = "deferred-minor: ms-too-many-errors-says-more-than-8"]`, body unchanged | — | still fails when run ignored |
| CLI `bughunt_ms_08_cli_repair_message_claims_more_than_8` | `#[ignore = "deferred-minor: ms-too-many-errors-says-more-than-8"]`, body unchanged | — | still fails when run ignored |
| CLI `bughunt_ms_09_mnem_unknown_language_unhandled` | `#[ignore = "deferred-minor: ms-cli-mnem-unknown-language-catch-all"]`, body unchanged | — | still fails when run ignored |
| codec `bughunt_ms_x1_mixed_case_correction` | `#[ignore = "deferred-minor: ms-repair-accepts-mixed-case-with-typo"]`, body unchanged | — | still fails when run ignored |
| CLI `bughunt_ms_t2_inspect_unknown_language_says_ok` (round 2) | GREEN (tightened) | preconditions unchanged (`ms decode -` on the language-10 string exits non-zero naming "unknown mnem wordlist-language code"); `ms inspect --json -` exits 0 with stdout == the after-fix `--json` line of 2.5 + `"\n"`; `ms inspect -` exits 0 with stdout == the twelve after-fix text lines of 2.5, each ending `\n` (line 3 empty). The `serde_json::Value` import becomes unused and is removed | `left: "{\"schema_version\":\"1\",…\"would_decode\":true,\"failure_reasons\":[]}\n"` |
| codec `bughunt_ms_t1_question_mark_erasure` (round 2) | `#[ignore = "planned-feature: impl-bch-erasure-decoding-ms"]`, body unchanged: it is the feature's acceptance row (recovery), not a bug row | — | today: `index 3: refused: string length 50 outside v0.1 set …`; after this PR it still fails, now with `refused: codex32 parse error: InvalidChar('?')` (measured) |
| codec `bughunt_ms_t1b_eight_erasures_recovered` (round 2) | `#[ignore = "planned-feature: impl-bch-erasure-decoding-ms"]`, body unchanged | — | after this PR: `eight `?` erasures (within BIP-93's stated 8) were refused: codex32 parse error: InvalidChar('?')` |
| CLI `bughunt_ms_t3_combine_accepts_non_ms_hrp` (round 2) | `#[ignore = "deferred-minor: ms-combine-accepts-non-ms-hrp"]`, body unchanged | — | still fails when run ignored |

The header comments of rows 04, 05 and 07, codec 03 and T2 say "open product question" or describe the old behaviour today. They become the ruling, with exact text in the reference patch; the wording is not binding.

Counts on the fixed copy:
- `-E 'binary(bughunt) & test(/^bughunt_ms_/)'`: 12 run, 12 passed, 17 skipped.
- `--run-ignored ignored-only` with the same filter: 6 run, 6 failed. These are the deferred rows; they stay red as the record of their defects.
- `-E 'binary(bughunt)'`: 20 run, 20 passed, 9 skipped.
- `-E 'binary(bughunt_t)'`: 1 run, 1 passed, 3 skipped; `--run-ignored ignored-only -E 'binary(bughunt_t)'`: 3 run, 3 failed (the planned-feature and deferred rows).
- Whole workspace: 667 run, 667 passed, 23 skipped, against 670 run, 655 passed, 15 failed, 14 skipped at `7a0edce` (controller; the same test tree as `5399625`).

## 7. Tests that change

**None outside the `bughunt.rs` and `bughunt_t.rs` files.** On the fixed copy every pre-existing test passes unchanged (667/667 above). The greps that looked for pins of the old behaviour, run on this worktree:

| grep | hits outside `bughunt.rs` | why none changes |
|---|---|---|
| `grep -rn "Threshold::ZERO" crates --include='*.rs'` | `shares.rs` 110 (the branch), 358, 397, 404, 412 (unit tests: byte-identity for entr/mnem with `Tag::ENTR`, `n == 2` refusal); `error.rs` 129 (doc) | all use valid tag/payload pairs |
| `grep -rln -- '"--to"' crates/ms-cli/tests` | `cli_combine.rs`, `combine_entropy_language_advisory.rs` | entr/mnem share sets only; the preimage test `hashlock_other_verbs.rs::combine_prints_a_recovered_preimage_as_decode_does` uses the default `--to` |
| `grep -rn -E "would_combine\|would combine" crates/ms-cli/tests` | `inspect_share.rs` 4, 70, 104 | index-`p` share; stays `OK` |
| `grep -rn "UnexpectedStringLength" crates/ms-codec/tests crates/ms-cli/tests` | `bch_all_lengths.rs` 170/193 (deleted alphabet symbol), `hashlock_kind.rs` 144, `vectors/hashlock-v0.8.json` 283, `negative.rs` 154, `json_error_envelope_per_kind.rs` 59 | all are `decode()`'s length gate or a wrong-length string; none puts a character outside the alphabet into `decode_with_correction` |
| `grep -rn -E "string length [0-9]+ not in v0\.1 set\|outside v0\.1 set" crates --include='*.rs'` | `argv_guard.rs` 482-483, `argv_remedy_runs.rs` 14-15, `freed_stdin.rs` 22 (comments), `error.rs` 230 (codec Display) | wrong-length inputs; unchanged |
| `grep -rn -E "language: unknown\|\"language\":\"unknown\"\|\"unknown\"" crates/ms-cli/tests --include='*.rs'` (T2) | none | — |
| `grep -rln -E "kind: mnem\|\"mnem\"" crates/ms-cli/tests` (T2) | `cli_split.rs`, `inspect_mnem_string.rs` | known language codes (0-9); verdict stays `OK` |

## 8. Build impact

Each item is a line of 9.1 with status `owed-to-build`, closed by its GREEN commit.

1. `crates/ms-codec/src/shares.rs`: #1, section 3 item 1, and the rustdoc of 2.1.
2. `crates/ms-codec/src/decode.rs`: #7 (closes #3), section 3 item 2, and the comment of 2.4.
3. `crates/ms-cli/src/cmd/combine.rs`: #4, section 3 item 3. The unreachable census stays 4.
4. `crates/ms-cli/src/cmd/inspect.rs`: #5, section 3 item 4, and the doc comment of 2.3; T2, section 3 item 5.
5. Specs: the four replacements of section 4.
6. Tests: every row of section 6 (tightened, new and ignored), in both `bughunt.rs` and both `bughunt_t.rs`. Then `cargo fmt`.
7. `design/FOLLOWUPS.md`: append 9.1, 9.2, 9.3 and 9.5, in that order.
8. Toolkit manual companion PR (branch `docs/bughunt-ms-manual` from `c2b09861`): 5.1, 5.2, 5.5 and 5.6 (the script `toolkit-manual-edits.py` applies all ten), plus the FOLLOWUPS entry 9.4. `docs/manual` `make audit` must show no finding in `43-ms.md` or `65-bch-codex-primer.md`. It merges with or after this PR.
9. **Toolkit consumption: no toolkit code or test change, in this PR or in the companion.**
   - The toolkit's `mnemonic repair` wraps `ms_codec::decode_with_correction` (`crates/mnemonic-toolkit/src/repair.rs:1166` `repair_via_ms_codec`, and `Ms1IndelOracle::validate` at `:1389`). It depends on `ms-codec = "0.7"` from crates.io (`Cargo.lock`: 0.7.0, whose `parse_ms1_symbols` has the same `c as u8` loop).
   - So #7/#3 and #1 reach `mnemonic repair` only after an ms-codec release and a toolkit pin bump. That bump is separate toolkit work, recorded in 9.4.
   - What the bump changes, by reading at `c2b09861`:
     - `repair_card`'s ms1 path is unaffected: its `parse_chunk` pre-gate (`repair.rs:709-722`) already refuses a non-ASCII or non-bech32 character as `UnparseableInput` before ms-codec runs.
     - `Ms1IndelOracle::validate` maps every `Err` to `None`, so a candidate holding a non-ASCII character becomes a rejection. Under 0.7.0 it could be read as an alphabet symbol.
     - `encode_shares` is called only with `Threshold::new(k)` (`cmd/ms_shares.rs:297`, `slot_ms1.rs:161`, `tests/cli_ms1_slot.rs:643`).
   - No toolkit test or manual text pins a changed message. Grep at `c2b09861`: `git grep -n -E "outside v0\.1 set|not in v0\.1 set|parse failed before correction|PostCorrectionDecodeFailed|UnparseableInput|InvalidChar|ms_codec::decode_with_correction" origin/master -- crates docs`. Its only ms1-message hit is `friendly.rs:119` (`"ms1 string length {} not in v0.1 set …"`, which maps `decode()`'s length gate and is unchanged). The `parse failed before correction` hit (`tests/cli_positional_hrp_autodetect.rs:453`) comes from the toolkit's own pre-gate. `git grep -n -E "non-ASCII char|non-bech32 char|ĳ" origin/master -- crates/mnemonic-toolkit/tests` finds nothing.
   - This item closes when the companion PR's commit adds 9.4.

**Gate (not a change).** Run every job of `.github/workflows/rust.yml` locally or as green CI on the PR head. That covers both fmt toolchains, clippy, `cargo nextest run --locked --workspace` and the `-p ms-codec` jobs, the `--include-ignored` history-purge, decline and G6 jobs, the mlock fault-injection steps and miri. Also run `vendor-freshness.yml` (`bash ci/repro/vendor-freshness.sh`), `bash ci/code-docs-check.sh` and `bash ci/docs-only.test.sh`, and the counts of section 6. Then run `coordinator/scripts/mutation-gate.sh <worktree> <the commit that adds this document>` before any review. Review: the change touches holder-facing `ms-cli` text and the repair engine (`decode_with_correction`), so the s1 slice runs section 2's commands.

**Release (recommendation, not a build item).** Versions do not change in this PR, following the `15ce93a` precedent: CHANGELOG and version in a release commit after merge. Recommended:
- ms-codec 0.10.0 → 0.10.1 and ms-cli 0.20.1 → 0.20.2, both PATCH (fixes to documented contracts; no flag, API or wire change).
- ms-cli's exact pin `ms-codec = "=0.10.0"` moves with it.
- The toolkit then moves its `ms-cli-v0.20.1` tag pins (`scripts/install.sh` canonical and the workflows the sibling-pin-check gate ties to it, including `manual.yml`). 5.1 and 5.2 describe that release.

## 9. FOLLOWUPS entries (exact text)

### 9.1 Append to `design/FOLLOWUPS.md` (owed to this build)

~~~markdown
### `bughunt-ms-fix` — ms bug-hunt Importants: `encode_shares` ZERO tag checks (#1), `combine --to` on preimage shares (#4), `inspect` of the secret share (#5), the non-alphabet refusal during correction (#7, closes #3), `inspect` of an unknown mnem language (T2), the BCH primer's erasure overclaim: build obligations (repo: **mnemonic-secret** + manual companion in **mnemonic-toolkit**; owning phase: **the build of branch `bughunt/triage`**)

- **Surfaced:** 2026-10-31 Claude API bug hunt (`design/agent-reports/bughunt-2026-10-31-ms.md`), triage `bughunt-triage-ms.md`, grades `bughunt-grade-ms.md` (`4755eb9`); round 2: triage `bughunt-triage2-ms.md` (`5399625`), grades `bughunt-grade2-ms.md`. FIX: `design/FIX_bughunt_ms.md` (small-fix track).
- **Grade:** Important ×6 (#1 an unreadable backup from a public API; #4 a silently ignored flag; #5 and T2 a wrong OK verdict; #7 a refusal that names the wrong problem on `ms repair`; the toolkit manual's BCH primer promising erasure correction no tool has), plus #3 (Minor) closed by #7's fix.
- **Companion:** `mnemonic-toolkit` branch `docs/bughunt-ms-manual` (manual 43-ms.md and 65-bch-codex-primer.md, and `design/FOLLOWUPS.md::toolkit-ms-codec-bump-bughunt-ms-fixes`).
- **Build impact (1):** `shares.rs` ZERO path returns `encode()`'s result: `owed-to-build`.
- **Build impact (2):** `decode.rs` `parse_ms1_symbols` refuses a non-alphabet character as `InvalidChar` (as typed; non-ASCII never a member): `owed-to-build`.
- **Build impact (3):** `combine.rs` honours `--to entropy` / `--to ms1` for a preimage: `owed-to-build`.
- **Build impact (4):** `inspect.rs` FAIL verdicts: reason `share-index-secret` for index `s` (#5) and `mnem-unknown-language` for an unknown mnem language code (T2): `owed-to-build`.
- **Build impact (5):** the four spec replacements (SPEC_ms_v0_2_kofn.md lines 51 and 86, SPEC_ms_hashlock.md line 502, SPEC_ms_mnem_wordlist_language.md line 49): `owed-to-build`.
- **Build impact (6):** the RED rows of FIX §6 (tightened, new, ignored), in `bughunt.rs` and `bughunt_t.rs`: `owed-to-build`.
- **Build impact (7):** the deferred-minor, needs-triage and planned-feature entries of FIX §9.2, §9.3 and §9.5: `owed-to-build`.
- **Build impact (8):** toolkit manual companion PR (FIX §5: 43-ms.md and the BCH primer) and its FOLLOWUPS entry: `owed-to-build`.
- **Build impact (9):** toolkit consumption recorded, no toolkit code change (FIX §8 item 9): `owed-to-build`.
- **Status:** `open`; owed to the build.
- **Tier:** `cross-repo`
~~~

### 9.2 Append to `design/FOLLOWUPS.md` (status `deferred-minor`)

~~~markdown
### `ms-mlock-unpin-not-refcounted` — dropping one mlock pin unlocks a page another live pin still covers (repo: **mnemonic-secret**, mirrored in **mnemonic-toolkit**; owning phase: **none — ownerless residue; latent until two pins share a page**)

- **Surfaced:** 2026-10-31 bug hunt #2; triage CONFIRMED; graded Minor (`design/agent-reports/bughunt-grade-ms.md`): secret-handling class (operator ruling 2026-08-27), latent.
- **Where:** `crates/ms-cli/src/mlock.rs` `PinnedPageRange::drop` (munlock is unconditional).
- **What:** mlock is not reference-counted, so dropping pin A munlocks a page that live pin B still covers (`Locked:` 4 kB → 0 kB, measured). No caller holds two live pins on one page today.
- **Fix (for later):** a per-page pin count (the triage's `fix-02.patch` shape: count on pin, munlock a page only when its count reaches 0), mirrored byte-for-byte in the toolkit's `mlock.rs` (G6 invariant, `tests/mlock_g6_invariant.rs`).
- **RED row:** `crates/ms-cli/tests/bughunt.rs::bughunt_ms_02_mlock_unpin_not_refcounted`, `#[ignore = "deferred-minor: ms-mlock-unpin-not-refcounted"]`.
- **Why deferred:** Minor; the FIX PR carries only C and I (`design/FIX_bughunt_ms.md`).
- **Status:** `deferred-minor`
- **Tier:** `secret-handling` (non-gating), `cross-repo`

### `ms-codex32-parts-short-data-panic` — `Codex32String::from_string` / `inspect` panic on a checksum-valid string with a 29+ character HRP and a data part of 18 or fewer characters (repo: **mnemonic-secret**; owning phase: **none — ownerless residue**)

- **Surfaced:** 2026-10-31 bug hunt #6; triage CONFIRMED (`ms inspect` exits 101: `byte range starts at 6 but ends at 0`); graded Minor (abort, nothing printed; needs a crafted string).
- **Where:** `crates/ms-codec/src/codex32/mod.rs` `parts_inner` (slices `s[6..s.len() - checksum_len]`).
- **Fix (for later):** guard `parts_inner`, but not with `fix-06.patch`'s `InvalidLength(54)`: the CLI renders that as "string length 54 not a valid codex32 length (need 48-93 short or 125-127 long)", which contradicts itself. Pick an error whose text names the short data part.
- **RED row:** `crates/ms-codec/tests/bughunt.rs::bughunt_ms_06_long_hrp_short_data_panics`, `#[ignore = "deferred-minor: ms-codex32-parts-short-data-panic"]`.
- **Why deferred:** Minor; the FIX PR carries only C and I.
- **Status:** `deferred-minor`
- **Tier:** robustness (non-gating)

### `ms-too-many-errors-says-more-than-8` — `ms repair` says "more than 8 errors; uncorrectable" for 5 errors; the decoder corrects at most 4 (repo: **mnemonic-secret**; owning phase: **none — ownerless residue**)

- **Surfaced:** 2026-10-31 bug hunt #8; triage CONFIRMED; graded Minor (a wrong count inside a correct refusal; exit 2).
- **Where:** `crates/ms-codec/src/error.rs` `Display` for `TooManyErrors` and `crates/ms-cli/src/error.rs` `From<ms_codec::Error>` (`format!("more than {} errors; uncorrectable", bound)`).
- **Fix (for later):** the codec `Display` and the CLI `FormatViolation` message both become exactly `more than 4 errors (or a missing or extra character); uncorrectable`. The `bound: 8` field and JSON `details: {"bound": 8}` stay (pinned by `bch_decode::five_error_too_many`, SPEC_ms_codec_test_hardening T3-ms-2, `error.rs` `too_many_errors_maps_to_format_violation_exit_2`, and the toolkit's "exceeds singleton bound = 8"). Exit 2 unchanged; the manual's repair exit table already says "> 4 substitution errors".
- **RED rows:** `crates/ms-codec/tests/bughunt.rs::bughunt_ms_08_too_many_errors_bound_is_8` and `crates/ms-cli/tests/bughunt.rs::bughunt_ms_08_cli_repair_message_claims_more_than_8`, both `#[ignore = "deferred-minor: ms-too-many-errors-says-more-than-8"]`.
- **Why deferred:** Minor; the FIX PR carries only C and I.
- **Status:** `deferred-minor`
- **Tier:** text (non-gating)

### `ms-cli-mnem-unknown-language-catch-all` — a mnem string with language code 10 or more falls through to "unhandled ms_codec::Error variant", exit 1 (repo: **mnemonic-secret**; owning phase: **none — ownerless residue**)

- **Surfaced:** 2026-10-31 bug hunt #9; triage CONFIRMED (`ms decode` → `error: unhandled ms_codec::Error variant: Error("unknown mnem wordlist-language code: 10")`, exit 1); graded Minor (fails closed; needs a foreign or newer encoder).
- **Where:** `crates/ms-cli/src/error.rs` `From<ms_codec::Error>` (no `MnemUnknownLanguage` arm).
- **Fix (for later):** add the arm `FormatViolation`, exit 2, JSON kind `"MnemUnknownLanguage"`, `details: {"got": <code>}`, message exactly `unknown mnem wordlist-language code <code> (known codes are 0-9)`. Not `fix-09.patch`'s text (Rust range syntax `(0..=9)`); not `FutureFormat`/exit 3.
- **RED row:** `crates/ms-cli/tests/bughunt.rs::bughunt_ms_09_mnem_unknown_language_unhandled`, `#[ignore = "deferred-minor: ms-cli-mnem-unknown-language-catch-all"]`.
- **Why deferred:** Minor; the FIX PR carries only C and I.
- **Status:** `deferred-minor`
- **Tier:** text / exit code (non-gating)

### `ms-repair-accepts-mixed-case-with-typo` — `decode_with_correction` repairs a mixed-case string that also has an error, though the same string without the error is refused (repo: **mnemonic-secret**; owning phase: **none — ownerless residue**)

- **Surfaced:** 2026-10-31 bug hunt (unrated item X1); triage CONFIRMED as behaviour; graded Nit (the output is the right codeword, exit 4 UNVERIFIED; BIP-93 line 159 permits treating wrong-case characters as erasures; the defect is inconsistency with `decode`/`combine`, U4).
- **Where:** `crates/ms-codec/src/decode.rs` `parse_ms1_symbols` (case-folds every character).
- **Fix (for later):** `decode_with_correction` refuses a mixed-case string with the same `Codex32(InvalidCase(..))` that `decode` returns for it, naming the same character (`ms decode` names `S` for `ms10entrSqz4…` and `s` for `MS10ENTRsQZ4…`); message `mixed case at character '<c>' (codex32 strings must be all-lower or all-upper)`, exit 1. Not `fix-x1.patch` verbatim (it names the first uppercase character).
- **RED row:** `crates/ms-codec/tests/bughunt.rs::bughunt_ms_x1_mixed_case_correction`, `#[ignore = "deferred-minor: ms-repair-accepts-mixed-case-with-typo"]`.
- **Why deferred:** Nit; the FIX PR carries only C and I.
- **Status:** `deferred-minor`
- **Tier:** consistency (non-gating)

### `ms-length-error-lists-entr-set-only` — `decode`'s length error lists only the entr lengths, so a 52-character string from a 51-character mnem card is told the valid set is [50, 56, 62, 69, 75] (repo: **mnemonic-secret**, text mirrored in **mnemonic-toolkit** `friendly.rs:119`; owning phase: **none — ownerless residue**)

- **Surfaced:** 2026-11-01, the triage's #7 open question; graded Minor in `bughunt-grade-ms.md` (fails closed and names the problem; needs a non-English mnem card plus a dropped or added character). Untested: a RED row is owed.
- **Where:** `crates/ms-codec/src/decode.rs` `decode` pre-dispatch gate (`allowed: VALID_STR_LENGTHS`); `crates/ms-cli/src/error.rs` `CliError::UnexpectedStringLength` (hard-coded `[50, 56, 62, 69, 75]` in `message()` and `details()`).
- **Fix (for later):** the codec pre-dispatch gate reports the union `[50, 51, 56, 58, 62, 64, 69, 70, 75, 77]`; the CLI carries `allowed` through (the hunt report's #7 mapping fix) and prints `string length <got> not in the valid set [50, 51, 56, 58, 62, 64, 69, 70, 75, 77]`, JSON `details.allowed` the same list. Exit 1 unchanged.
- **Why deferred:** Minor; the FIX PR carries only C and I.
- **Status:** `deferred-minor`
- **Tier:** text (non-gating)

### `ms-combine-accepts-non-ms-hrp` — `ms combine` recovers a share set whose HRP is not `ms`, while `decode` refuses such a single with `WrongHrp` (repo: **mnemonic-secret**; owning phase: **none — ownerless residue**)

- **Surfaced:** 2026-11-01, bug-hunt triage round 2, T3 (`design/agent-reports/bughunt-triage2-ms.md`), CONFIRMED; graded Minor (`bughunt-grade2-ms.md`): the output is exactly the secret the strings encode, and no ms, toolkit or BIP-93 artifact carries a non-`ms` HRP (BIP-93 line 70); a typo in a real share's HRP breaks its checksum, and a mixed `ms`/`xx` set is refused.
- **Where:** `crates/ms-codec/src/shares.rs` `combine_shares` (no HRP check; `envelope::discriminate` has one for singles).
- **Fix (for later):** `combine_shares` refuses a set whose canonical HRP is not `ms` with `Error::WrongHrp { got }` (the first 4 characters, as `discriminate` reports), so `ms combine` prints `error: wrong HRP: got "xx", expected "ms"`, exit 2, as `ms decode` does for a single (the triage's `fix-t3.patch` shape). In the same change `ms inspect` reports such a share as `FAIL: would NOT combine` with the existing reason `wrong-hrp` (`HRP is not "ms"`), `would_decode` and `would_combine` false, exit 0 (as the #5 ruling).
- **RED row:** `crates/ms-cli/tests/bughunt_t.rs::bughunt_ms_t3_combine_accepts_non_ms_hrp`, `#[ignore = "deferred-minor: ms-combine-accepts-non-ms-hrp"]`.
- **Why deferred:** Minor; the FIX PR carries only C and I.
- **Status:** `deferred-minor`
- **Tier:** consistency (non-gating)
~~~

### 9.3 Append to `design/FOLLOWUPS.md` (status `needs-triage`)

~~~markdown
### `ms-decode-calls-punctuation-mixed-case` — `ms decode` and `ms inspect` call `?` or `!` in a lowercase string "mixed case", where `ms repair` (after FIX_bughunt_ms) says "invalid character" (repo: **mnemonic-secret**; owning phase: **the next ms triage pass**)

- **Surfaced:** 2026-11-01, while checking `design/FIX_bughunt_ms.md`, and the round-2 triage's open question (`/scratch/cache/bughunt/ms2/reach-t1-decode-chars.txt`). No RED row yet. Proposed grade Minor: it fails closed and names the character, but calls it a case error; it needs a holder who types a punctuation mark, such as BIP-93's `?`, into `decode` or `inspect` on a lowercase string.
- **What:** on ms 0.20.1, `echo 'ms10entrsqz4?42424242424242424242424q8lurdrzjnyn08' | ms decode -` prints `error: mixed case at character '?' (codex32 strings must be all-lower or all-upper)`, exit 1; `!` gives the same; `b` gives `invalid character 'b' (not in codex32 alphabet)`; the all-uppercase twin with `?` gives `invalid character '?'`. Cause: the vendored codex32 `set_check_case` (`crates/ms-codec/src/codex32/checksum.rs:166`) treats any ASCII character that is neither a digit nor lowercase as uppercase.
- **Proposed fix (for the triage to confirm):** only an ASCII letter sets or checks the case; any other character outside the alphabet is `InvalidChar(c)`, so `ms decode` and `ms inspect` print `invalid character '?' (not in codex32 alphabet)`, exit 1, the text `ms repair` prints after FIX_bughunt_ms. It edits vendored codex32 code, whose parity gate covers valid strings only. The toolkit's `mnemonic repair` already refuses with its own accurate text (`non-bech32 char '?' at position N`, exit 2).
- **Status:** `needs-triage`
- **Tier:** text (grade at triage)
~~~

### 9.4 Append to `mnemonic-toolkit` `design/FOLLOWUPS.md` (in the companion PR)

~~~markdown
### `toolkit-ms-codec-bump-bughunt-ms-fixes` — what the ms bug-hunt fixes change in the toolkit at its next ms-codec bump (companion)

- **Surfaced:** 2026-11-01, mnemonic-secret `design/FIX_bughunt_ms.md` (branch `bughunt/triage`). **Companion:** `mnemonic-secret/design/FOLLOWUPS.md::bughunt-ms-fix`.
- **Where:** `crates/mnemonic-toolkit/Cargo.toml` `ms-codec = "0.7"` (`Cargo.lock` 0.7.0).
- **What:** the ms-codec half reaches the toolkit only through an ms-codec release and this pin. (a) `decode_with_correction` refuses any data-part character outside the codex32 alphabet as `Codex32(InvalidChar(c))` (c as typed) instead of `UnexpectedStringLength`, and never reads a non-ASCII character as a symbol. `repair_card`'s ms1 path is unaffected (its `parse_chunk` pre-gate refuses such characters first, `repair.rs:709-722`); `Ms1IndelOracle::validate` (`repair.rs:1389`) maps every `Err` to `None`, so a candidate holding a non-ASCII character is rejected where 0.7.0 could read it as an alphabet symbol. (b) `encode_shares(_, Threshold::ZERO, ..)` refuses what `encode` refuses; the toolkit calls it only with `Threshold::new(k)`. No toolkit test or manual text pins either old behaviour (grep at `c2b09861`, recorded in the FIX §8 item 9). The ms-cli half (`ms combine --to` on preimage shares, `ms inspect` of the secret share) reaches the manual's pinned binary when the toolkit's `ms-cli-v0.20.1` tag pins move to the release that carries it; this PR's 43-ms.md text describes that release.
- **Why deferred:** the ms-codec release and the pin bump (0.7 → 0.10.x) are separate toolkit work.
- **Status:** `open`
- **Tier:** `cross-repo`
~~~

### 9.5 Append to `design/FOLLOWUPS.md` (status `planned-feature`)

~~~markdown
### `impl-bch-erasure-decoding-ms` — erasure decoding of `?` for ms1, the third leg of `impl-bch-erasure-decoding-md-mk` (repo: **mnemonic-secret**, first step in **mnemonic-toolkit**; owning phase: **after the bug-hunt fix PR**)

- **Surfaced:** 2026-11-01, bug-hunt triage round 2, T1 and T1b (`design/agent-reports/bughunt-triage2-ms.md`, `bughunt-grade2-ms.md`). BIP-93 line 152: "up to 8 unreadable characters (called erasures)"; lines 155 and 158: "we recommend that: … Implementations interpret "?" as an erasure." Brian 2026-11-01: "Fix bugs before erasure feature work". The bug-fix PR (`design/FIX_bughunt_ms.md`) makes a `?` a correct refusal (`invalid character '?' (not in codex32 alphabet)`, exit 1) and corrects the toolkit manual's BCH primer, which promised 8-erasure correction.
- **Companions:** `descriptor-mnemonic/design/FOLLOWUPS.md` and `mnemonic-key/design/FOLLOWUPS.md` → `impl-bch-erasure-decoding-md-mk`, filed 2026-07-10 by the BIP-alignment cycle (`mnemonic-toolkit/design/BUGLIST_bip_alignment_cycle_2026-07-10.md` DG-1), which downgraded md1/mk1's erasure MUST to SHOULD/informative. ms was outside that cycle. This entry is the ms1 leg.
- **Code:** ms1's string layer is the vendored rust-codex32 0.1.0 (`crates/ms-codec/src/codex32/`; no erasure code). Its substitution decoder is ms-codec's own `bch.rs`/`bch_decode.rs` (BIP-93's regular code, target `MS_REGULAR_CONST = 0x10ce0795c2fd1e62a`). `bch_decode.rs` was forked from mk-codec v0.3.1 and mirrors md-codec's, so the three legs share one decoder shape and differ in target residue.
- **Measured (controller, `/scratch/cache/bughunt/ms2/controller-subst-probe.txt`):** `ms repair` corrects 1, 2, 3 and 4 random substitutions to the original in 300/300 trials each and refuses 5 and 6 in 300/300, with no miscorrection. One or two `?` gave exit 1 with #7's length message in 20/20. Rewriting each `?` as `q` and then repairing corrected every case up to 4 marks (300/300 at k = 4). Also measured: leaving 1-4 characters out and running `mnemonic repair --max-indel N` recovers them (exit 5; controller `controller-indel-probe.txt`, grading `/scratch/cache/bughunt/ms-grade/probe/indel_probe.txt`).
- **Triage estimate:** polymod is GF(2)-linear in the symbols, so an erasure solve is a 65-equation system in 5e unknowns; the triage's Python oracle (`/scratch/cache/bughunt/ms2/erasure_oracle.py`) recovers 1-8 scattered and 8 or 13 consecutive erasures uniquely. The work: `?` parsing, the solver, a policy for erasures mixed with substitutions, `CorrectionDetail.was = '?'`, exit 4, spec and TRACE rows, manual lockstep, then an ms-codec release and a toolkit bump. About 150-250 lines of production code, 20+ tests, three repos. Fill-and-check (32^e candidates) is exact for small e and not usable at 8.
- **Design direction (Brian 2026-11-01, "So maybe a better fix is to add "?" feature to indel?"):** the preferred first step. The toolkit's `mnemonic repair` accepts `?`, fills each marked position with all 32 symbols (32^k candidates; practical to about k = 4-5), checksum-checks, and reports through the existing indel/VERIFY-ME exits for ms1, mk1 and md1 alike. A valid fill of 8 or fewer marked positions is unique by BIP-93's 8-erasure guarantee, and with substitutions mixed in while marks + 2 × substitutions ≤ 8 (md1/mk1 need their code distance confirmed). True BCH erasure decoding (to 8 marks, and re-upgrading the md1/mk1 BIP MUST) stays the later leg.
- **Rows:** `crates/ms-codec/tests/bughunt_t.rs::bughunt_ms_t1_question_mark_erasure` and `::bughunt_ms_t1b_eight_erasures_recovered`, both `#[ignore = "planned-feature: impl-bch-erasure-decoding-ms"]`; they test `ms_codec::decode_with_correction`, so they stay ignored until ms-codec itself fills `?`. That change also flips the `?` cases of `bughunt_ms_07_codec_non_alphabet_is_invalid_char` and `bughunt_ms_07b_repair_names_char_as_typed` from refusal to recovery, and the BCH primer's "Unreadable characters: what to type".
- **Status:** `planned-feature`
- **Tier:** `feature` / cross-repo
~~~

## 10. STOPs

None. Every holder-visible point is decided in sections 2 and 5. The #7 fix as specified turns `bughunt_ms_03` GREEN (measured on the fixed copy), so #3 is closed, not deferred.

Observations for the review (not STOPs):

1. **`?` in `ms decode`.** While checking that `repair` and `decode` name the same character, I found that `ms decode` on a lowercase string holding `?` says "mixed case at character '?'". This is pre-existing and comes from the vendored codex32 case check, so the shared-character claim is made for letters only. It is filed in 9.3 (`ms-decode-calls-punctuation-mixed-case`) and not fixed here.
2. **`entropy_hex` for a preimage.** `combine --json` uses its existing `CombineJson` (schema `"1"`) with `kind` `"preimage"`, so a preimage's 64 hex characters sit under `entropy_hex`. `ms decode`'s own preimage object calls them `preimage_hex`; the default `--to phrase --json` keeps that object. This was ruled in `bughunt-grade-ms.md`. A new key would change the `combine` schema.
3. **Manual describes the next release.** The toolkit's manual CI installs `ms-cli-v0.20.1`. Sections 5.1, 5.2 and 5.5 add prose only (no runnable example), and the lint shows no new finding. The text becomes true of the pinned binary when the release moves the pin (section 8, "Release"). 5.6's commands run as printed on today's tools.
4. **Exit 5 for a left-out character.** The primer says a unique `--max-indel` recovery exits 5; that is `mnemonic repair`'s documented behaviour ("a unique indel candidate re-validates the full checksum, so it self-verifies"), measured here, not a claim this document makes about safety. The text still tells the holder to compare the result with the plate.
5. **Code origin of ms's decoder.** The controller's note said ms uses rust-codex32 0.1.0 rather than md/mk's forked `bch.rs`. That is true of ms's string layer, but its correction decoder `bch_decode.rs` was forked from mk-codec v0.3.1 (its own header says so). 9.5 records both, since it bears on how the three erasure legs share work.
