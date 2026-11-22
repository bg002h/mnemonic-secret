# FIX: `ms inspect` says a share of an impossible length would combine (bh-ms-I12)

Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-22. FIX table row #48 of
`coordinator/agent-reports/architect-bughunt-1119-questions.md` (L160). **Small-fix track**, grade I
(`coordinator/agent-reports/bughunt-1119/grade-ms.md` L43, L67-68). Base: `origin/master` `423f1734`
(ms-cli 0.21.0). Sources: `design/agent-reports/bug-hunt-2026-11-19.md` L212-226 (consultant),
`coordinator/agent-reports/bughunt-1119/triage-ms-t3.md` L17, L28 (RED/GREEN), `xref-ms.md` L36 (prior art:
`FIX_bughunt_ms.md` #5 fixed the index-`s` case only). No T-ruling names this row. The architect ruled the reason
string (grade Q5) a wording point, not a Brian question (architect report L29): it is written below as PROPOSED and
goes through a `design/WORDING_REVIEW.md` row.

Every statement of current behaviour below was measured on a build of `423f1734`; every statement of fixed behaviour
was measured on a scratch build with the change of section 3 applied and then reverted (not committed). Scratch
files, kept outside the repo as references only (the builder writes from this document, not from them):
`/scratch/cache/ms-inspect-share-length-red/bh1119_inspect_share_length.rs` (the RED rows of section 4, sha256
`4d5ba22f613e0016f5b305a7886b223f7e087322e7e60ec088a5678ff8d27a96`) and `.scratch-green.patch` in the same
directory (sha256 `1b4b22768fe47bda65a736f57e7570d3b8b156f7d58ea31c1ae594c9de462b6b`). On the scratch build
`cargo nextest run --locked -p ms-cli` gave `550 tests run: 550 passed, 18 skipped` (545 existing plus the 5 new
rows): no existing test changes.

## 1. Summary

`ms inspect` on a lone K-of-N share (threshold 2..9) reports `OK: K-of-N share (would combine: needs k shares)`
(`--json`: `would_decode` and `would_combine` true) whenever the share is not at index `s`. It never looks at the
string length. A share's data length equals the secret's (prefix byte plus payload), so a share whose string length
is not one an ms1 secret can have cannot be recombined: `ms combine` recovers the secret and then refuses its
payload length. A holder who checks a set with `inspect` before storing it is told it would combine.

**Fix:** the share branch also checks the string length against the union of every ms1 secret length
(`VALID_STR_LENGTHS` entr, `VALID_MNEM_STR_LENGTHS` mnem, `VALID_PREIMAGE_STR_LENGTHS` preimage:
`[50, 51, 56, 58, 62, 64, 69, 70, 75, 77]`). Outside it, `inspect` prints the existing share FAIL verdict with the
existing reason `unexpected-string-length`, exit 0, in both the text and the JSON emitter.

## 2. Current behaviour (measured on `423f1734`)

Fixture: `Codex32String::from_seed("ms", 2, "test", <index>, &[0u8; n])` (the consultant's; public zero data,
no wallet). `n = 40` gives the 86-character share
`ms12testaqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqthh5axw2fra4t` (index `a`).

- `ms inspect --json -` on it: exit 0, stdout
  `{"failure_reasons":[],"report":{"checksum_valid":true,"hrp":"ms","kind":"share","share_index":"a","tag":"test","threshold":2},"schema_version":"1","would_combine":true,"would_decode":true}`
- `ms inspect -` on it: exit 0, first line `OK: K-of-N share (would combine: needs 2 shares)`.
- `ms combine --to entropy -` on the index-`a` and index-`c` pair: exit 1, stdout empty, stderr
  `error: tag "entr" payload length 39 not in expected set [16, 20, 24, 28, 32]`.
- Every codex32-valid share length outside the union gets the positive verdict: for `n` in 16..=44 and 63..=64
  (string lengths 48-93 and 125-127, the lengths codex32 parses), the wrong ones are
  `(16, 48), (19, 53), (20, 54), (23, 59), (24, 61), (27, 66), (28, 67), (31, 72), (32, 74), (35, 78), (36, 80),
  (37, 82), (38, 83), (39, 85), (40, 86), (41, 88), (42, 90), (43, 91), (44, 93), (63, 125), (64, 127)`
  (data bytes, string length). The ten union lengths already get the positive verdict.
- Out of scope, unchanged: a share string of a length codex32 does not parse (outside 48-93 and 125-127) is
  refused before any verdict: exit 1, `error: string length <L> not a valid codex32 length (need 48-93 short or
  125-127 long)`.

Site: `crates/ms-cli/src/cmd/inspect.rs` L67-74 (share branch: no length passed), L104-127 (`emit_share_text`),
L133-154 (`emit_share_json`); both decide only on `is_secret_share` (L96-98). Combine's check:
`crates/ms-codec/src/envelope.rs` L194-231 (`dispatch_payload`, called from `shares.rs` L321).

## 3. Change

### 3.1 Code (`crates/ms-cli/src/cmd/inspect.rs`)

1. The share branch passes the input's string length (`ms1.len()`, the same length `analyze` already receives at
   L76) to both share emitters.
2. One helper computes the share's failure reasons, in this order (the order `ms combine` checks them: the index-`s`
   refusal at `shares.rs` step 2, the payload length after recovery):
   1. `share-index-secret` when the index is `s` (unchanged rule);
   2. `unexpected-string-length` when the length is in none of `VALID_STR_LENGTHS`, `VALID_MNEM_STR_LENGTHS`,
      `VALID_PREIMAGE_STR_LENGTHS` (the three constants already imported at L9-10; no new constant, no new list).
3. `emit_share_text`: if the reasons are empty, the current OK line (unchanged); otherwise `FAIL: would NOT combine`
   followed by one `    reason: <tag> (<reason_text(tag)>)` line per reason, in the order above. The field lines
   after the blank line are unchanged.
4. `emit_share_json`: `would_decode` and `would_combine` are both `reasons.is_empty()`; `failure_reasons` is the
   reasons list. Key set and order unchanged.
5. `reason_text` is unchanged (no new tag, no new text). Doc comments of the two emitters and of the share branch
   name the length rule next to the index-`s` rule.

Exit code: 0 for every share verdict (unchanged). No flag, type or command changes.

### 3.2 Printed texts

| Case | Text stdout (exactly) | `--json` stdout (exactly) |
|---|---|---|
| bad length, index `a` (n = 40) | `FAIL: would NOT combine`<br>`    reason: unexpected-string-length (string length not in valid set for this kind ([50,56,62,69,75] entr / [51,58,64,70,77] mnem / [75] preimage))`<br>(blank line)<br>`hrp: ms` / `threshold: 2` / `id: test` / `index: a` / `checksum_valid: true` / `kind: share` | `{"failure_reasons":["unexpected-string-length"],"report":{"checksum_valid":true,"hrp":"ms","kind":"share","share_index":"a","tag":"test","threshold":2},"schema_version":"1","would_combine":false,"would_decode":false}` |
| bad length, index `s` (n = 40) | `FAIL: would NOT combine`<br>`    reason: share-index-secret (share-index 's' is the secret itself, not a distributed share; ms combine refuses it)`<br>`    reason: unexpected-string-length (...as above...)`<br>(blank line, fields with `index: s`) | `{"failure_reasons":["share-index-secret","unexpected-string-length"],...,"share_index":"s",...,"would_combine":false,"would_decode":false}` |
| union length, any index but `s` | unchanged (`OK: K-of-N share (would combine: needs k shares)`) | unchanged |
| union length, index `s` | unchanged (`bughunt_ms_05` fixtures) | unchanged |

PROPOSED (architect confirms; WORDING_REVIEW row): reuse the existing tag `unexpected-string-length` and its existing
`reason_text` for a share, as the deferred `ms-combine-accepts-non-ms-hrp` line (FOLLOWUPS L950) already plans for
`wrong-hrp`. Alternative B: a share-specific tag, e.g. `share-string-length` with text `string length is not the
length of any ms1 secret ([50,51,56,58,62,64,69,70,75,77]); ms combine refuses it`. Recommendation: A (no new
holder text; the existing text lists all three sets, which is the share rule). The two-reason order is also
PROPOSED (it follows combine's check order).

### 3.3 SPEC (`design/SPEC_ms_v0_2_kofn.md` L87)

Replace the sentence that starts `**Exception (FIX_bughunt_ms #5):**` through `exit 0.` with (PROPOSED):

> **Exceptions:** `inspect` reports a share as `FAIL: would NOT combine` (`--json`: `would_decode` and
> `would_combine` false), exit 0, when (a) it sits at index `s`: the secret-at-S, which `combine` refuses
> (`SecretShareSuppliedToCombine`), reason `share-index-secret` (FIX_bughunt_ms #5); or (b) its string length is
> not one an ms1 secret can have, the union of the entr, mnem and preimage sets `[50, 51, 56, 58, 62, 64, 69, 70,
> 75, 77]`, which `combine` refuses after recovery (`PayloadLengthMismatch` / `PreimageLengthMismatch`), reason
> `unexpected-string-length` (FIX_bughunt1119_ms_inspect_share_length). When both apply both reasons print,
> `share-index-secret` first. A positive verdict is necessary, not sufficient: a lone share cannot show the kind
> the set recovers, nor that the holder has the rest of the set.

### 3.4 Toolkit manual lockstep (`mnemonic-toolkit` `docs/manual/src/40-cli-reference/43-ms.md` L342, read at `04c56bfa`)

Replace the sentence that starts `One exception: a share-threshold string at index` through `and still exits 0.`
with (PROPOSED):

> Two exceptions print `FAIL: would NOT combine` and still exit 0 (under `--json`: `would_decode` and
> `would_combine` false): a share-threshold string at index `s` is the secret itself, not a distributed share, and
> `ms combine` refuses it (reason `share-index-secret`, `failure_reasons: ["share-index-secret"]`); and a share whose
> string length is not one an ms1 secret can have (50, 51, 56, 58, 62, 64, 69, 70, 75 or 77 characters) cannot be
> recombined (reason `unexpected-string-length`). When both apply, both reasons are listed, `share-index-secret`
> first. A share that passes can still fail `ms combine`: one share cannot show which kind of secret its set
> recovers.

No flag changes, so `tests/lint.sh flag-coverage` is unaffected.

### 3.5 CHANGELOG (`CHANGELOG.md`, under `## ms-cli [Unreleased]`)

`### Fixed` line (PROPOSED): **`ms inspect` no longer says a share of an impossible length would combine.** A share
whose string length is not one an ms1 secret can have (50, 51, 56, 58, 62, 64, 69, 70, 75 or 77 characters) now
prints `FAIL: would NOT combine` with reason `unexpected-string-length` (`--json`: `would_decode` and
`would_combine` false), exit 0 as before; `ms combine` refuses such a set (bh-ms-I12).

## 4. RED rows

New file `crates/ms-cli/tests/bh1119_inspect_share_length.rs` (own file, so the parallel ms FIX rows do not collide
in `bughunt.rs`). Material on stdin (`-`). The union is written in the test as a literal, not read from
`ms_codec::consts`. Command for all rows:
`cargo nextest run --locked -p ms-cli --test bh1119_inspect_share_length --no-fail-fast`.
On `423f1734`: `5 tests run: 1 passed, 4 failed` (the oracle row is a guard and passes on the base).

| Row | Asserts (fixed behaviour) | On base (measured) |
|---|---|---|
| `bh1119_ms_i12_inspect_share_length_json` | n = 40, index `a`: exit 0, stdout = the JSON of 3.2 row 1 plus `\n` | FAIL: `failure_reasons":[]`, `would_combine":true` |
| `bh1119_ms_i12_inspect_share_length_text` | n = 40, index `a`: exit 0, stdout = the text of 3.2 row 1 | FAIL: stdout starts `OK: K-of-N share (would combine: needs 2 shares)` |
| `bh1119_ms_i12_inspect_secret_share_bad_length_both_reasons` | n = 40, index `s`: JSON and text of 3.2 row 2, exit 0 | FAIL: `failure_reasons":["share-index-secret"]` only |
| `bh1119_ms_i12_inspect_share_verdict_tracks_length` | for n in 16..=44 and 63..=64 at index `a`: both modes exit 0; JSON `would_combine` and `would_decode` equal "length in union"; text starts with the OK line iff in union, with `FAIL: would NOT combine` iff not (covers the mnem-length and preimage-length controls: n = 18 gives 51, n = 33 gives 75) | FAIL: the 21 (n, length) pairs listed in section 2 |
| `bh1119_ms_i12_combine_oracle` (guard) | `combine --to entropy -` of the n = 40 `a`/`c` pair: exit 1, stdout empty, stderr exactly `error: tag "entr" payload length 39 not in expected set [16, 20, 24, 28, 32]\n`; entr control n = 17 pair: exit 0, stdout starts with 32 `0`s and a newline; mnem control data `[0x02, 0x00] ++ [0; 16]` (length 51) pair: exit 0, same stdout start | PASS (stays green after the fix) |

GREEN on the scratch fix: all five pass; `cargo nextest run --locked -p ms-cli`: `550 tests run: 550 passed, 18
skipped`.

## 5. Tests that change

None. Checked on the scratch build: `inspect_share.rs` (17-byte share, length 50) and `bughunt.rs`
`bughunt_ms_05_inspect_would_combine_unchecked` / `bughunt_ms_05b_inspect_uppercase_secret_share` (index `s`,
length 50) keep their exact outputs. The comment at `bughunt.rs` L192 ("no other share reason") stays true for its
fixture.

## 6. Gate

Gate line for the builder (CI jobs `fmt`, `test`, `clippy`, `test-ms-codec` unaffected; run once to a file):
`cargo +1.95.0 fmt --all -- --check && cargo nextest run --locked --workspace --no-fail-fast && cargo clippy
--all-targets -p ms-cli -- -D warnings && bash ci/code-docs-check.sh`, then the toolkit manual `tests/lint.sh` on
the manual branch. Not covered: the miri, musl, FreeBSD, release, history-purge and g6 legs (no change reaches
them), review.

## 7. Files this FIX changes

ms repo: `crates/ms-cli/src/cmd/inspect.rs`, `crates/ms-cli/tests/bh1119_inspect_share_length.rs` (new),
`design/SPEC_ms_v0_2_kofn.md`, `CHANGELOG.md`. Toolkit repo: `docs/manual/src/40-cli-reference/43-ms.md`.

## 8. FOLLOWUPS

No existing `design/FOLLOWUPS.md` line names bh-ms-I12. No new line proposed.
