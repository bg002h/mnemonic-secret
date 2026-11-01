# Bug-hunt grading pass, mnemonic-secret (ms)

Opus grading pass under RULES.md "Triage of reported bugs" item 2. Inputs:
hunt report `bughunt-2026-10-31-ms.md`, triage report `bughunt-triage-ms.md`,
RED rows at `1b7dd78`, scratch fixes and transcripts in `/scratch/cache/bughunt/ms/`.
No new hunting and no re-triage. Production code and tests untouched.

**Outcome: 0 C, 4 I (#1, #4, #5, #7), 5 M (#2, #3, #6, #8, #9), 1 N (X1). #10 FALSE POSITIVE accepted.**
#3 is M on its own merits, but the #7 fix as ruled below closes it in the same PR (see #3 row).

What I checked myself (beyond the brief's Settled facts):
- Every failing row in `controller-gate-1b7dd78.txt` fails at its own assertion with the reported symptom (lines 44-45, 64-65, 83-84, 104-105, 125-126, 156-158, 711-715, 734-737, 755-765, 782-783, 801-802).
- Upstream BIP-93 fetched from `raw.githubusercontent.com/bitcoin/bips/master/bip-0093.mediawiki` is byte-identical to the cached copy (`diff -q`: identical, 714 lines).
- On the unmodified `target/debug/ms` (0.20.1): `ms decode` names an invalid uppercase character as typed (`invalid character 'B'`), while `ms repair --ms1 -` on the same string and on a string holding `?` both print `error: string length 50 not in v0.1 set [50, 56, 62, 69, 75]`, exit 1. `ms decode` names the mixed-case character `S` for `ms10entrSqz4…` and `s` for `MS10ENTRsQZ4…`.
- `ms hashlock --out` builds its string with `ms_codec::encode(Tag::HASH, &Payload::Preimage(..))` (`crates/ms-cli/src/cmd/hashlock.rs:413`), the same call #4's fix uses.
- No caller in the constellation passes `Threshold::ZERO` to `encode_shares` (grep of mnemonic-toolkit, mnemonic-refugium and ms itself: the toolkit uses `Threshold::new`, refugium uses `ms_codec::encode`).
- The toolkit's ms1 repair wrapper (`mnemonic-toolkit/crates/mnemonic-toolkit/src/repair.rs:1312-1347`) and its own TooManyErrors text ("exceeds singleton bound = {bound}") read the `bound` field; the toolkit pins `ms-codec = "0.7"` (ms is at 0.10.0).

## Grades

| # | final grade | triage proposal | agree? | row tests the defect? | likelihood conditions / why |
|---|---|---|---|---|---|
| 1 | **I** | I | yes | Yes: three tag/payload cases, `encode()` refusal as precondition, and `decode()` of the minted string shown false. Tighten in the PR to the exact variant per case (below). | The public API returns `Ok` with a string the same crate's `decode()` refuses: an unreadable backup and an unmet documented guarantee ("byte-identical to `encode(tag, secret)`"). The settled rule keeps that C or I however rare. Not C: no shipped caller reaches it (no `Threshold::ZERO` caller anywhere), and the input is itself a caller contract error. |
| 2 | **M** | M | yes | Yes: compiles the real `src/mlock.rs` by `#[path]`, two pins on one fresh `mmap` page, `Locked:` from `/proc/self/smaps` goes 4 kB -> 0 kB while pin B is live. | Secret-handling class (operator ruling 2026-08-27): never C or I. Also latent: no two live pins share a page today (triage's gdb trace of `ms derive`). |
| 3 | **M** (closed in the PR by #7's fix) | M | grade yes; disposition differs | Yes: U+0133/U+0130 alias to `'3'`/`'0'` and are "repaired". Asserts only `is_err`; tighten to `InvalidChar(alias)`. | Not a wrong card: the output is the codeword an ASCII typo at that position would give, exit 4 UNVERIFIED; only the report's `was` character is wrong. BIP-93 line 159 lets correction treat non-bech32 characters as erasures, so accepting them is not itself non-compliant. Conditions: (a) a non-ASCII character in the typed string (a keyboard layout or paste artifact, e.g. Cyrillic `а`/`е` U+0430/U+0435, read as `0`/`5`); (b) its low byte is one of the 32 alphabet bytes (any other non-ASCII character takes #7's path instead). Disposition: #7's ruled fix refuses every character outside the alphabet; done correctly that includes non-ASCII and closes #3. Shipping `fix-07.patch` as is would leave a new "invalid character" refusal that does not refuse `ĳ`, which a whole-diff review should block as a refusal that does not refuse. |
| 4 | **I** | I | yes (fix wider than `fix-04.patch`) | Yes for RED (fails because stdout equals the default record). Permissive: any non-zero exit passes. Under the ruling below, tighten to exit 0 plus exact stdout. | A silently ignored flag (settled rule; conditions do not demote it). Conditions noted, not demoting: preimage shares exist only through the library (`ms split` and the toolkit's `ms-shares split` have no preimage source, F-468). The same early return also ignores `--to entropy`; one root cause, one fix. |
| 5 | **I** | I | yes | Yes: an index-`s` k=2 string, with `ms combine`'s refusal as precondition; JSON and text verdicts both checked. Tighten to exact values. | A wrong OK verdict does not fail closed. Why it matters: in BIP-93 the index-`s` string at threshold k IS the secret. A holder auditing a card set made by another BIP-93 tool is told the secret card is "a K-of-N share (would combine: needs 2 shares)", the opposite of what they need to know to store it safely. Not C: ms never emits one, and `ms combine` refuses it cleanly. |
| 6 | **M** | M | yes | Yes: `catch_unwind` around `Codex32String::from_string` and the public `inspect()`. | Fails closed in substance (abort, exit 101, nothing printed, no wrong data), though the panic text does not name the problem. Conditions: (a) an HRP of 29 or more characters (every ms card's HRP is `ms`); (b) a valid codex32 checksum over that HRP with a data part of 18 or fewer characters, which needs a crafted string (a typo of a real card cannot produce one: the checksum must validate); (c) the string is given to `ms inspect` (or to `ms decode` at a length in its length set; by reading, not run). No printed command fails. |
| 7 | **I** | I | yes (fix amended) | Yes for RED (the error lists the string's own length as disallowed). Its GREEN is vacuous: any error kind other than `UnexpectedStringLength` skips the `if`. Tighten. | Fails closed but names the wrong problem, telling the holder that a correct length is wrong. This is on `ms repair`, the tool for damaged cards. It needs one ordinary condition: a character outside the alphabet (`b`, `i`, `o`, `1`, or `?`). `?` is BIP-93's own mark for an unreadable character (line 158), and the toolkit manual's BCH primer (`65-bch-codex-primer.md:108`) lists "erasures (known positions, e.g., `?`)", so a holder who follows the manual hits this message. Likelihood tests (1) and (3) both fail. The codec's own doc comment on `parse_ms1_symbols` already promises `Error::Codex32` via `InvalidChar`. |
| 8 | **M** | M | grade yes; reasoning differs | Yes (codec and CLI rows): assert only that "more than 8" is absent. The follow-up should pin the ruled text. | Fails closed (exit 2), "uncorrectable" is correct, and no command is printed. The triage reads the likelihood test as making this I "by the letter". It does not: the test is a sufficient condition for M, not a necessary one, and failing (3) only means the usual grades apply. Under those, a wrong count inside a correct refusal is M when it changes no action open to the holder (recount, re-read the plate, retype are the same for 5 errors and for 9). Condition: (a) the typed string has 5 or more substitutions, or an inserted or dropped character. |
| 9 | **M** | M | yes | Yes: asserts the catch-all text is absent and exit 2. | Fails closed: exit 1, and the text names "unknown mnem wordlist-language code: 10", though inside a catch-all wrapper. Conditions: (a) a string from another or newer encoder that uses a reserved code (10-15, or a non-zero high nibble); this encoder's `validate()` never emits one, and a typo cannot produce one with a valid checksum; (b) a caller that branches on exit 2 vs 1. |
| X1 | **N** | N | yes | Yes: clean mixed case is refused by both entry points, but mixed case with one substitution is repaired. | Not a wrong card: the output is the right codeword, exit 4 UNVERIFIED, and BIP-93 line 159 permits treating wrong-case characters as erasures during correction. The only defect is inconsistency: the same string without the typo is refused (U4 in `tests/uppercase_envelope.rs`; `PLAN_ms1_envelope_uppercase.md` says "mixed-case never reaches here"). The case change is also not listed as a correction. |

BIP-93 text (upstream, identical to the cached copy):

- line 82: "As with bech32 strings, a codex32 string MUST be entirely uppercase or entirely lowercase."
- line 152: "The checksum is designed to be an error correcting code that can correct up to 4 character substitutions, up to 8 unreadable characters (called erasures), or up to 13 consecutive erasures."
- line 154: "However, implementations SHOULD NOT automatically proceed with a corrected codex32 string without user confirmation of the corrected string, either by prompting the user, or returning a corrected string in an error message and allowing the user to repeat their action."
- line 158: "* Implementations interpret "?" as an erasure."
- line 159: "* Implementations optionally interpret other non-bech32 characters, or characters with incorrect case, as erasures."
- line 161: "* If a string consisting of valid bech32 characters in the proper case can be made valid by substituting 4 or fewer characters, then the implementation suggests such a string as a correction."

The triage's quotes (lines 82, 154, 159, 161) are exact. Its conclusion holds: rejecting mixed case during correction is a choice, not a BIP-93 requirement. It omitted line 158, which bears on #7 (see "Not graded").

## #10 verdict

**FALSE POSITIVE, accepted.** The plant (`plant-10.patch`: `repeating` computed from `None::<clap::builder::ValueRange>`) reproduces exactly the reported mechanism: `get_num_args()` is `None` on an unbuilt `Command`, so `repeating` is false. The row then fails at its own assertion with `combine's shares positional ... must be repeating: {"name":"shares","repeating":false,...}`. That is the right failure for the right reason.

The real code differs from the prover's copy. The `Ty::Vec` positional arm of `vendor/clap_derive/src/derives/args.rs` (about lines 300-305) emits `.num_args(1..)`, and `Arg::get_num_args` returns `self.num_vals` (`vendor/clap_builder/src/builder/arg.rs`, about lines 4347-4350). I read both. So `get_num_args()` is already `Some(1..)` before build, and unmodified `ms gui-schema` emits `"repeating": true`. Keep the row as a passing regression pin.

## Product rulings

**#4: `ms combine --to` on a preimage share set.** Honour the flag; do not refuse. The spec row "a recovered preimage share set prints as `decode` does" (`design/SPEC_ms_hashlock.md:502`, `:285`) stays true for the default.

| `--to` | text stdout | `--json` stdout | exit |
|---|---|---|---|
| `phrase` (the default, given or not) | unchanged: the preimage record exactly as `ms decode` prints it | unchanged: `ms decode`'s preimage JSON object | 0 |
| `entropy` | one line: the 32-byte preimage as 64 lowercase hex characters | `{"schema_version":"1","kind":"preimage","entropy_hex":"<64 hex>"}` (the existing `CombineJson`, `kind` = `"preimage"`) | 0 |
| `ms1` | one line: the 75-character `hash` single, `ms_codec::encode(Tag::HASH, &payload)` (`ms10hash…`), byte-identical to what `ms hashlock --out` writes for that preimage | `{"schema_version":"1","kind":"preimage","ms1":"<that string>","entropy_hex":"<64 hex>"}` | 0 |

stderr for `entropy` and `ms1` carries only the existing PrivateKeyMaterial advisory line, as on every other `combine` path. No language advisory and no engraving card. An explicit `--to phrase` cannot be told apart from the default under `default_value = "phrase"`. It keeps printing the record, which opens with `kind: preimage`, so the holder is told what they got. That case is documentation only (manual text below). `--help` text is unchanged.

**#5: `ms inspect` on a share-threshold string at index `s`.**
- Verdict: `would_decode: false`, `would_combine: false`, `failure_reasons: ["share-index-secret"]`. Every other share stays as today (`true`, `true`, `[]`).
- Exit 0, as for every inspect verdict, including a FAIL on a single.
- The check reads `InspectReport.share_index`, which is already lowercased (`envelope::wire_string`), so an all-uppercase `S` string is caught too.
- Text output (`--json` absent), exactly, for the triage's k=2 `zzzz` string:

```
FAIL: would NOT combine
    reason: share-index-secret (share-index 's' is the secret itself, not a distributed share; ms combine refuses it)

hrp: ms
threshold: 2
id: zzzz
index: s
checksum_valid: true
kind: share
```

  The field lines are the existing ones, unchanged. `fix-05.patch` drops them; keep them. Add `"share-index-secret"` to `reason_text()` with the parenthesised text above.
- JSON, exactly (keys sorted as today): `{"failure_reasons":["share-index-secret"],"report":{"checksum_valid":true,"hrp":"ms","kind":"share","share_index":"s","tag":"zzzz","threshold":2},"schema_version":"1","would_combine":false,"would_decode":false}`.
- HRP sub-question: no change in this PR. `would_combine` mirrors what `ms combine` does, and today `combine` accepts a share set whose HRP is not `ms`, so `inspect` lists no HRP reason. Whether `combine` should require `ms` is a separate, untested question (see "Not graded"). If `combine` is changed later, `inspect` gains the existing reason name `wrong-hrp` for shares in the same change.

**#7: a character outside the codex32 alphabet in `ms repair`.** Use the triage's `InvalidChar` direction, amended. Do not use the report's error-mapping fix in this PR: it does not cure the reachable case (`fix-07a` still fails the row). After the `InvalidChar` fix, the only reachable `UnexpectedStringLength` is `decode`'s pre-dispatch gate, where the CLI's hard-coded set equals the set the codec passes. That gate's list omits the mnem lengths `[51, 58, 64, 70, 77]`; it is a separate Minor, deferred below.
- Codec: in `parse_ms1_symbols`, the first data-part character that is not in the alphabet returns `Err(Error::Codex32(crate::codex32::Error::InvalidChar(c)))`.
- `c` is the character as typed, taken from the input `s` rather than the lowercased copy, so `ms repair` and `ms decode` name the same character.
- Membership is "ASCII, and its ASCII lowercase is in the alphabet". There is no `as u8` truncation, so a non-ASCII character is never a member.
- CLI text, existing `friendly_codex32` wording: `error: invalid character 'b' (not in codex32 alphabet)`. Likewise `'B'` for an uppercase string, `'?'`, and `'ĳ'`. Exit 1 (unchanged). JSON `kind` `"Codex32"`, no `details`.
- Erasure decoding (`?` or other unreadable marks filled by BCH) is a feature, not this fix (see "Not graded"). No special `?` text in this PR.

**#8: the "more than 8 errors" text.** Ruled for the deferred follow-up.
- Codec `Display` for `TooManyErrors` and the CLI `FormatViolation` message both become exactly: `more than 4 errors (or a missing or extra character); uncorrectable`.
- The parenthesis is accurate whenever the error fires: four or fewer substitutions at the right length always decode, and a dropped or added character usually lands here (`tests/bch_all_lengths.rs` T3-ms-1 comment). The manual already tells holders to count characters for length damage (`65-bch-codex-primer.md`).
- The `bound: 8` field and JSON `details: {"bound": 8}` stay. They are pinned by `bch_decode::five_error_too_many`, `SPEC_ms_codec_test_hardening.md` T3-ms-2 and `error.rs` `too_many_errors_maps_to_format_violation_exit_2`. The toolkit also renders the field as the singleton bound ("exceeds singleton bound = 8"), so changing it to 4 would make the toolkit's text wrong.
- Exit 2 (unchanged). The manual's repair exit table already says "> 4 substitution errors", so no manual change.

**#3 / X1: non-ASCII and mixed case during correction.** Refuse both; do not treat them as erasures (BIP-93 line 159 makes that optional).
- Non-ASCII is refused in this PR, through #7's fix.
- Mixed case is refused in the deferred X1 follow-up. `decode_with_correction` returns for a mixed-case string the same `Codex32(InvalidCase(..))` that `decode` returns, naming the same character. Message: `error: mixed case at character 'S' (codex32 strings must be all-lower or all-upper)`, exit 1.
- `fix-x1.patch` names the first uppercase character instead, which is `'M'` for `MS10ENTRsQZ…` where `decode` says `'s'`. So the follow-up must not take it verbatim.

**#1: grade only.** I, with the report's fix (`fix-01.patch`).

**#9: the mapping.** `FormatViolation`, exit 2, JSON kind `"MnemUnknownLanguage"`, `details: {"got": <code>}`. Message, exactly: `unknown mnem wordlist-language code 10 (known codes are 0-9)`. `fix-09.patch`'s text uses Rust range syntax `(0..=9)`; do not use it. Not `FutureFormat`/exit 3: that variant carries a 4-byte tag and its text is about tags. The precedent for an unrecognised reserved value in a wire field is `ReservedPrefixViolation` -> exit 2.

## Fixes for the PR (C/I only)

The toolkit-manual lockstep is required for #4 and #5. Old texts were grepped in `mnemonic-toolkit/docs/manual/src`. Neither the #7 texts (`string length`, `v0.1 set`) nor any #1 text appears anywhere in the manual.

### #1: `fix-01.patch` as is
- `encode_shares` with `Threshold::ZERO` returns `encode(tag, secret)`'s result, error included.
- No holder-visible text; library only; no manual change.
- Update the rustdoc bullet to: "returns exactly what `encode(tag, secret)` returns, its error included".
- Row `bughunt_ms_01_zero_threshold_skips_tag_checks`: tighten to the exact variant per case:
  - `seed` + Entr -> `ReservedTagNotEmittedInV01 { got: *b"seed" }`
  - `hash` + Entr -> `TagKindMismatch { tag: *b"hash", prefix: 0x00 }`
  - `entr` + Preimage -> `TagKindMismatch { tag: *b"entr", prefix: 0x03 }`
- The existing byte-identity tests must stay green.

### #4: wider than `fix-04.patch`
- In `cmd/combine.rs`, the Preimage arm returns `emit_preimage(x, args.json)` only for `CombineTo::Phrase`.
- For `Entropy` and `Ms1` it proceeds with the 32 bytes and `kind` `"preimage"` through the existing emitters. `emit_ms1` encodes with `payload.kind().single_tag()` (public on `PayloadKind`) instead of the hard-coded `Tag::ENTR`.
- Outputs are as in the #4 ruling. Exit 0.
- Manual `43-ms.md:293`, the `--to <T>` row, becomes exactly: `| \`--to <T>\` | output form for the recovered secret (default \`phrase\`): \`phrase\` (BIP-39 mnemonic — \`mnem\` → wire language, \`entr\` → English), \`entropy\` (raw entropy hex), or \`ms1\` (a single unshared ms1 string). For a hashlock **preimage** share set, \`phrase\` prints the preimage record exactly as \`ms decode\` does (never words), \`entropy\` prints the 32-byte preimage as 64 hex characters, and \`ms1\` prints the 75-character \`hash\` single (\`ms10hash…\`) that \`ms decode\` and \`ms hashlock --in\` read |`
- Rows:
  - `bughunt_ms_04_combine_to_ms1_ignored_for_preimage`: require exit 0, and stdout == `encode(Tag::HASH, &Payload::Preimage(x))` + `"\n"`.
  - Add `bughunt_ms_04b_combine_to_entropy_preimage`: stdout == `"11"×32` + `"\n"`, exit 0.
  - Add `bughunt_ms_04c_combine_to_ms1_json_preimage`: stdout == the exact JSON in the ruling.
  - Add `bughunt_ms_04d_combine_to_entropy_json_preimage`: likewise.
  - The existing default-path preimage test stays as is.

### #5: `fix-05.patch` amended
- `share_index == 's'` drives `would_decode`/`would_combine` false and `failure_reasons` `["share-index-secret"]`.
- The text form prints the two FAIL lines above, then the unchanged field block.
- Exit 0.
- Manual `43-ms.md:337` (the `kind: share` bullet): insert after "...and exits 0 — it does NOT print a `threshold-not-zero` length/decode FAIL.", exactly: `One exception: a share-threshold string at index \`s\` is the secret itself, not a distributed share, and \`ms combine\` refuses it; for it \`ms inspect\` prints \`FAIL: would NOT combine\` with the reason \`share-index-secret\` (under \`--json\`: \`would_decode\` and \`would_combine\` false, \`failure_reasons: ["share-index-secret"]\`), and still exits 0.`
- Rows:
  - `bughunt_ms_05_inspect_would_combine_unchecked`: assert the exact JSON and the exact first two text lines.
  - Add `bughunt_ms_05b_inspect_uppercase_secret_share`: the same string `.to_uppercase()` gives the same verdict.
  - `inspect_share.rs` stays green as the negative control (an index-`p` share still prints `OK: K-of-N share (would combine: needs 2 shares)`).

### #7 (closes #3): `fix-07.patch` amended
- Membership and character-as-typed exactly as ruled.
- Replace the stale comment in `parse_ms1_symbols`, which says an `InvalidChar` cannot be constructed; it can, since codex32 is vendored as `crate::codex32`.
- Texts: `error: invalid character '<c>' (not in codex32 alphabet)`, exit 1, JSON `kind` `"Codex32"`, `exit_code` 1, no `details`.
- No manual change: the old text is absent and the exit code is unchanged at 1.
- Rows:
  - `bughunt_ms_07_string_length_error_contradicts_itself`: assert `kind == "Codex32"`, `exit_code == 1`, and message == `invalid character 'b' (not in codex32 alphabet)`.
  - Add `bughunt_ms_07b_repair_names_char_as_typed`, three cases on the 50-character card:
    - all-uppercase with `B` -> `invalid character 'B' (not in codex32 alphabet)`
    - `?` at index 12 -> `invalid character '?' (not in codex32 alphabet)`
    - `ĳ` at index 12 -> `invalid character 'ĳ' (not in codex32 alphabet)`
  - Codec: tighten `bughunt_ms_03_non_ascii_read_as_symbol` to `Err(Error::Codex32(InvalidChar(alias)))`. It is then a GREEN row, not `#[ignore]`d.
- Toolkit companion, no lockstep now: at its next ms-codec bump (pinned `"0.7"`), `mnemonic repair`'s ms1 path reports these as `UnparseableInput` ("parse failed before correction could run") instead of `PostCorrectionDecodeFailed` with the contradictory length text.

Note on scope: I ruled that the #7 fix closes #3 because a correct "not in the alphabet" test cannot leave non-ASCII in. If the controller keeps strict scope instead, #3's row is `#[ignore = "deferred-minor: ms-repair-non-ascii-aliases-symbol"]`. The #7 refusal then does not refuse `ĳ`, and I expect the whole-diff review to block on that.

## Deferred (`deferred-minor`; RED rows stay `#[ignore = "deferred-minor: <slug>"]`)

- `ms-mlock-unpin-not-refcounted` (#2): `PinnedPageRange::drop` munlocks pages another live pin still covers. Fix with a per-page pin count (`fix-02.patch` shape), mirrored byte-for-byte in the toolkit's `mlock.rs` (G6 invariant). Latent: no overlapping pins today.
- `ms-codex32-parts-short-data-panic` (#6): `from_string`/`inspect` panic on a checksum-valid string with a 29+ character HRP and a data part of 18 or fewer characters. Guard `parts_inner`, but not with `fix-06.patch`'s `InvalidLength(54)`: the CLI renders that as "string length 54 not a valid codex32 length (need 48-93 short or 125-127 long)", which contradicts itself. Pick an error whose text names the short data part.
- `ms-too-many-errors-says-more-than-8` (#8): reword codec `Display` and the CLI message to `more than 4 errors (or a missing or extra character); uncorrectable`. Keep `bound: 8` and its pins.
- `ms-cli-mnem-unknown-language-catch-all` (#9): add the `MnemUnknownLanguage` arm (`FormatViolation`, exit 2, `unknown mnem wordlist-language code <code> (known codes are 0-9)`, `details: {"got": <code>}`).
- `ms-repair-accepts-mixed-case-with-typo` (X1): `decode_with_correction` must refuse mixed case with the same `InvalidCase` and character `decode` gives (`fix-x1.patch` names a different character).
- `ms-length-error-lists-entr-set-only` (from #7's open question; untested, RED row owed): `decode`'s pre-dispatch length error lists only the entr lengths, so a 52-character string from a 51-character mnem card is told the valid set is `[50, 56, 62, 69, 75]`. List the union and carry `allowed` through the CLI (the report's #7 mapping fix). M by the likelihood test: fails closed and names the problem; needs a non-English (mnem) card plus a dropped or added character.

## Not graded (each needs a triage row first; noticed while answering the brief)

- `?` erasures: BIP-93 line 158 says "Implementations interpret "?" as an erasure." `ms repair` has no erasure decoding, and the toolkit manual's primer (`65-bch-codex-primer.md:108`, `:282`) lists 8-erasure correction as a property holders can fall back on. This is a feature or compliance question, not part of #7's fix.
- `ms inspect` prints `OK: would decode v0.2 … language: unknown` for a mnem language code of 10 or more, which `ms decode` refuses (triage §4). Same family as #5; untested.
- `ms combine` accepts a share set whose HRP is not `ms` (triage #5 open question). The product question is whether it should refuse with `WrongHrp`, exit 2, as `decode` does for singles.
