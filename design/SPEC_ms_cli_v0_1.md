# `ms-cli` v0.1 Design Spec — companion CLI for `ms-codec v0.1.0`

**Status:** v0.1 surface locked (brainstorm converged 2026-05-04 after r2 architect review). Reference implementation: `crates/ms-cli/`.
**Companion documents:**

- ms-codec library SPEC: [`SPEC_ms_v0_1.md`](./SPEC_ms_v0_1.md) — wire format, decoder rules, BIP-93 anchoring.
- BRAINSTORM (rationale chain): this conversation's transcript (per 2026-05-03 workflow refinement, brainstorm reviews stay in transcript and are not separately persisted).
- Migration contract: [`../MIGRATION.md`](../MIGRATION.md) — v0.1 → v0.2 invariants ms-cli must respect.
- Pre-CLI audit: [`agent-reports/audit-ms-codec-v0_1_0-pre-cli.md`](./agent-reports/audit-ms-codec-v0_1_0-pre-cli.md) — three Critical-for-CLI items (friendly codex32 errors; emit-time language enforcement; decode-vs-inspect routing) addressed in this SPEC.
- Sibling precedent: `bg002h/descriptor-mnemonic` `crates/md-cli/` — md-cli's command structure, `assert_cmd` test pattern, `ExitCode` dispatch.

This document specifies the user-facing CLI surface for `ms`. It does not re-specify the wire format (see [`SPEC_ms_v0_1.md`](./SPEC_ms_v0_1.md)) or the library API (see crate-level docs).

---

## §1. Scope

`ms` is a single binary (`crates/ms-cli/src/main.rs` → `ms` executable) for engraver-facing operations on **ms1** strings: encoding a BIP-39 mnemonic into an ms1 string suitable for steel-plate engraving, decoding an engraved ms1 back to its mnemonic, inspecting a candidate string for structural validity, verifying engraving correctness end-to-end, and dumping the canonical test-vector corpus.

In scope for v0.1:

- Five subcommands: `encode`, `decode`, `inspect`, `verify`, `vectors`.
- BIP-39 phrase ↔ entropy conversion via the `bip39 = "2"` crate. Phrase-first input on `encode` (the `--phrase` headline form), with `--hex` as a codec-thin escape hatch.
- All 10 BIP-39 wordlists supported via `--language` (default `english`).
- `--json` structured output mode on every command that produces structured data.
- Stdin uniform across all commands: `-` or omitted argument reads stdin; whitespace is stripped before parsing.

Out of scope for v0.1 (deferred to ms-cli v0.2+, the future `mnemonic-toolkit` crate, or never):

- **Generate** command (fresh-entropy → ms1 + phrase one-shot). Toolkit territory.
- **BIP-39 passphrase** ("25th word"). v0.1 wire format does not encode it (SPEC §6.4 anti-pattern note); CLI surfaces a stderr reminder but does not accept a `--passphrase` flag.
- **K-of-N share encoding.** ms-codec v0.2+ feature; ms-cli will gain `share split` / `share combine` subcommands then.
- **Bundle output** (ms1 + mk1 xpub-cards + md1 descriptor). The future `mnemonic-toolkit` is the integration surface; ms-cli stays narrow on the `ms`-format side.
- **Color / formatting / interactive prompts.** Pure batch tool; no `--color` / no TUI.

### §1.1 Engraving as the load-bearing user persona

Same framing as SPEC §1.1: every CLI decision is judged against "does this make a steel-plate backup more correct, or less?" Concretely: `encode` outputs a chunked form for proofreading; `verify --phrase` round-trips the engraved string against the original mnemonic to catch transcription errors; `inspect` surfaces verdict + reason for malformed strings.

---

## §2. Command surface

### §2.1 `ms encode` — produce an ms1 string from a BIP-39 mnemonic (or hex entropy)

```text
ms encode --phrase <words> [--language <lang>] [--no-engraving-card] [--json]
ms encode --hex <hex>      [--no-engraving-card] [--json]
```

`--phrase` and `--hex` form a clap mutually-exclusive group; exactly one must be supplied. `--phrase -` reads the phrase from stdin.

Defaults:

| flag | default | notes |
|---|---|---|
| `--language` | `english` | ignored under `--hex`; warning on stderr engraving card |
| `--no-engraving-card` | not set | suppresses stderr engraving card (for tooling) |
| `--json` | not set | enables structured stdout |

**Behavior:**

1. Parse phrase (or hex) into entropy bytes. For `--phrase`, `bip39::Mnemonic::parse_in(language, phrase)` validates the BIP-39 4-bit checksum and rejects on failure. For `--hex`, the input must be even-length and decode to 16/20/24/28/32 bytes.
2. Call `ms_codec::encode(Tag::ENTR, &Payload::Entr(entropy))`. The library validates entropy byte length against `consts::VALID_ENTR_LENGTHS` and rejects reserved tags symmetrically (SPEC §3.5.1) — both unreachable here since v0.1 only emits `Tag::ENTR` with an already-validated length.
3. **Default text-mode output:**
   - **stdout** (multi-line):
     ```text
     <ms1-string>

     <chunked-form>   (5-char groups, wrapped at 10 groups/line max — see §4)
     ```
   - **stderr** (engraving card; suppressed by `--no-engraving-card`):
     ```text
     word count: <N>
     language: <lang> (BIP-39 checksum valid)
     passphrase: not stored in ms1 (record separately if used)
     ```
4. **`--json` mode:** stdout = `{"schema_version": "1", "ms1": "<string>", "language": "<lang>", "word_count": <N>, "entropy_hex": "<hex>"}`. **Do not** emit `chunks` in JSON (presentation, not data — consumers re-chunk from `ms1` if needed).

**Encoder pre-checks:**

- `--hex <bytes>`: odd-length input rejected with friendly error "expected even-length hex (one byte = 2 chars)" (exit 1, `CliError::BadInput`).
- `--language <name>`: must be one of the 10 BIP-39 wordlists (see §8). clap value-enum rejects unknowns with usage error (exit 64).
- BIP-39 checksum mismatch: `CliError::Bip39(bip39::Error::InvalidChecksum)` → exit 1 with friendly message.
- BIP-39 wordlist mismatch: `bip39::Mnemonic::parse_in` is language-strict — `--language japanese` with English words yields `CliError::Bip39(bip39::Error::InvalidWord)` (exit 1). The CLI does not silently transcode across wordlists.

**Edge-case enumeration** (locked at v0.1; tests must cover each row):

| input | resulting error | exit |
|---|---|---|
| `--phrase ""` (empty) | `Bip39(BadWordCount)` "expected 12/15/18/21/24 words, got 0" | 1 |
| `--phrase " "` (whitespace only) | `Bip39(BadWordCount)` (post-trim, 0 words) | 1 |
| `--phrase "abandon"` (1 word) | `Bip39(BadWordCount)` | 1 |
| `--phrase "abandon abandon … about " + 13th word` | `Bip39(BadWordCount)` (only 12/15/18/21/24 accepted) | 1 |
| `--phrase "abandon  abandon  about"` (extra spaces) | OK if word count is valid (`bip39::Mnemonic::parse_in` uses `split_whitespace` which collapses runs); reaches checksum check | depends |
| `--hex ""` (empty) | `BadInput("expected hex of length 32/40/48/56/64 chars")` | 1 |
| `--hex "ZZ"` (non-hex chars) | `BadInput` (e.g., `"invalid character 'Z' at position 0"` — exact wording is the underlying `hex` crate's; the CLI may paraphrase) | 1 |
| `--hex "00"` (too short, even) | `BadInput("hex decodes to 1 byte; expected 16/20/24/28/32")` | 1 |
| `--hex "0".repeat(31)` (odd length) | `BadInput("expected even-length hex…")` | 1 |
| both `--phrase` and `--hex` supplied | clap usage error (mutually-exclusive group) | 64 |
| neither `--phrase` nor `--hex` supplied | clap usage error (required group) | 64 |

### §2.2 `ms decode` — recover a BIP-39 mnemonic from an ms1 string

```text
ms decode [<ms1>] [--language <lang>] [--json]
```

`<ms1>` is positional; if omitted or `-`, read from stdin (whitespace-stripped — see §3.2).

Defaults:

| flag | default | notes |
|---|---|---|
| `--language` | `english` | warning on stderr **and** in stdout language line when defaulted |
| `--json` | not set | structured stdout |

**Behavior:**

1. Read `<ms1>` from arg or stdin (after whitespace strip).
2. Call `ms_codec::decode(&ms1)?` → `(Tag, Payload)`. v0.1 always returns `Tag::ENTR` and `Payload::Entr(entropy)`. Errors mapped to `CliError` per §6.
3. Re-derive mnemonic via `bip39::Mnemonic::from_entropy_in(language, &entropy)`. (Cannot fail for valid `VALID_ENTR_LENGTHS` byte counts.)
4. **Default text-mode output (stdout):**
   ```text
   entropy: <hex>
   phrase: <mnemonic>
   language: <lang> (<N> words[, default — verify against your records])
   ```
   The "default — verify against your records" suffix appears **iff** `--language` was defaulted (not explicitly set). Surfaces the SPEC §6.3 wordlist hazard in piped output.
5. **stderr (always when `--language` defaulted):**
   ```text
   note: --language defaulted to 'english'; if your wallet was created with a different wordlist, decode with --language <lang>.
   ```
   Non-suppressible (no `--quiet` to hide it; only `--language <explicit>` removes both stderr and stdout warnings).
6. **`--json` mode:** stdout = `{"schema_version": "1", "entropy_hex": "<hex>", "phrase": "<mnemonic>", "language": "<lang>", "word_count": <N>, "language_defaulted": <bool>}`. The `language_defaulted` flag preserves the warning signal for tooling.

### §2.3 `ms inspect` — structural validity report

```text
ms inspect [<ms1>] [--json]
```

Lenient parse: returns a report even for strings that would fail decoder rules — caller examines the verdict + reasons to diagnose.

**Behavior:**

1. Read input (arg or stdin).
2. Call `ms_codec::inspect(&ms1)?` → `InspectReport` (raw structural fields).
3. Locally compute `would_decode: bool` and `failure_reasons: Vec<&'static str>` by re-walking SPEC §4 rules against the report's fields, using `ms_codec::consts::{VALID_STR_LENGTHS, VALID_ENTR_LENGTHS, RESERVED_NOT_EMITTED_V01, TAG_ENTR}` and `ms_codec::Tag`. The closed set of `failure_reasons` tags is locked at v0.1 (see §5.3 for the JSON schema):

| tag | source rule |
|---|---|
| `unexpected-string-length` | SPEC §4 rule 9 |
| `wrong-hrp` | rule 2 |
| `threshold-not-zero` | rule 3 |
| `share-index-not-secret` | rule 4 |
| `unknown-tag` | rule 6 |
| `reserved-tag-not-emitted` | rule 7 |
| `non-zero-prefix` | rule 8 |
| `payload-length-mismatch` | rule 10 |

4. **Default text-mode output (stdout):**
   ```text
   OK: would decode v0.1
   ```
   or
   ```text
   FAIL: would NOT decode v0.1
       reason: reserved-tag-not-emitted (tag "seed" is reserved-not-emitted in v0.1; deferred to v0.2+)
       reason: non-zero-prefix (got 0x01; v0.1 reserves 0x00)

   hrp: ms
   threshold: 0
   tag: seed
   share_index: s
   prefix_byte: 0x01
   payload_bytes: <hex>
   checksum_valid: true
   ```
5. **`--json` mode:** see §5.3.

Multiple `failure_reasons` may be reported in a single FAIL response (e.g., wrong-hrp AND non-zero-prefix). Order: rule number ascending.

#### §2.3.1 When `inspect()` itself errors

`ms_codec::inspect()` performs a BIP-93 parse upfront (`Codex32String::from_string`) and propagates `Error::Codex32` if that parse fails. In this case ms-cli cannot produce an `InspectReport` at all — there are no structural fields to surface.

When this happens, ms-cli `inspect` treats the error like any other CliError per §6:

- **Text mode:** stderr emits `error: <friendly_codex32(...)>`; exit 1; stdout silent.
- **`--json` mode:** stdout emits the §5.4 error envelope (`{"schema_version": "1", "error": {"kind": "Codex32", ...}}`); exit 1.

This is consistent with the audit's C3 "decode vs inspect routing" — `inspect` is more lenient than `decode` only over the *post-BIP-93-parse* surface. A string that fails BIP-93 parsing has no inspectable structure for either command.

**Note on exit-3 routing:** `inspect` cannot route exit 3 (`FutureFormat` / `ReservedTagNotEmittedInV01`). Reaching that signal requires a full `ms_codec::decode()` pass, which only `verify` does post-decode. A string that fails BIP-93 parsing because (e.g.) it uses a long-checksum framing reserved for v0.2+ surfaces here as exit 1 (`Codex32`) — the user must run `ms verify <string>` to learn whether it's a recognizable future format or genuinely malformed.

### §2.4 `ms verify` — exit-code-only validity (and optional round-trip)

```text
ms verify [<ms1>] [--phrase <words>] [--language <lang>] [--json]
```

**Behavior:**

1. Read input. Run inspect-style validation.
2. **Without `--phrase`:**
   - Valid v0.1 → exit 0; stdout `OK: valid v0.1 entr (<N> words, <M> chars)` (text mode) or success JSON object (`--json`).
   - Reserved-not-emitted (rule 7) → exit 3; stdout `OK: valid future format (v0.2+, tag <tag>)`.
   - Otherwise invalid → exit 2 (format violation) or exit 1 (user-input error per the §6 table); stdout `FAIL: <reason>`.
3. **With `--phrase`:**
   - Same validation first; if invalid, behaves as above.
   - If valid: decode → re-derive phrase via `bip39::Mnemonic::from_entropy_in(language, &entropy)` → compare `phrase.to_string()` against the supplied `--phrase`.
   - Match → exit 0; stdout `OK: round-trip valid (<N> words, language=<lang>)`.
   - Mismatch → exit 4 (`CliError::VerifyPhraseMismatch`); stdout `FAIL: phrase mismatch (decoded does not match --phrase)`. **Never echo either phrase to stdout** (both are secrets — the diff is exit-code only).
4. `--json` mode mirrors the text outcomes structurally.

#### §2.4.1 Validation order (locked)

Verify executes in a strict order so behavior is deterministic regardless of how many inputs are bad simultaneously:

1. **Read ms1 input** (arg or stdin per §3.2). On stdin read failure → exit 1, `CliError::BadInput`. **Concurrent stdin guard:** if both the ms1 positional arg AND `--phrase` resolve to stdin (i.e., user wrote `ms verify - --phrase -` or both omitted), exit 1 immediately with `CliError::BadInput("cannot read both ms1 and --phrase from stdin")` — clap can't catch this, so it's a runtime check before reading either.
2. **Decode the ms1 string** via `ms_codec::decode(&ms1)`. On failure: dispatch per §6.1.1 (so e.g. `Codex32` → exit 1; `WrongHrp` / `ThresholdNotZero` / `ShareIndexNotSecret` / `ReservedPrefixViolation` / `UnknownTag` / `TagInvalidAlphabet` → exit 2; `ReservedTagNotEmittedInV01` → exit 3 with `OK: valid future format`; `UnexpectedStringLength` / `PayloadLengthMismatch` → exit 1). **`--phrase` is NOT parsed in this branch** — verify exits before touching the phrase.
3. **Parse `--phrase` if present** via `bip39::Mnemonic::parse_in(language, phrase)`. On bip39 failure → exit 1, `CliError::Bip39` with friendly message.
4. **Compare** the decoded entropy's re-derivation against the parsed phrase. Match → exit 0; mismatch → exit 4, `CliError::VerifyPhraseMismatch`.

The order matters: an engraver who typed back a corrupt ms1 AND supplied a wrong-language phrase should see the ms1-side error first (i.e., **before** phrase parsing — "first" here means "earlier in the validation pipeline," not "higher severity"). Step 2's exit codes are still 1, 2, or 3 per §6.1.1; the dispatch is by error variant, not by some external severity ordering. The point of the "first" framing is that the phrase is never even read in step 3 if the ms1 already failed in step 2 — so a wrong-language phrase cannot mask an engraving-side error.

`verify` is the engraver round-trip command: type back the engraved ms1, supply the original phrase, exit code tells you if the engraving + your record are mutually consistent.

### §2.5 `ms vectors` — dump the SHA-pinned test-vector corpus

```text
ms vectors [--pretty]
```

**Behavior:**

1. Print the embedded v0.1 corpus JSON to stdout. Compact by default; `--pretty` indents via `serde_json::to_string_pretty`.
2. Exit 0 always. No subcommand-specific failure modes.

The corpus is `include_str!`-baked at build time from `crates/ms-cli/vectors/v0.1.json`, which is logically identical to `crates/ms-codec/tests/vectors/v0.1.json` (parsed-JSON equality — the parity test in §10.2 asserts `serde_json::Value`-equal, not byte-equal, to avoid spurious failures from whitespace or line-ending differences).

### §2.6 Per-subcommand `--help` text (locked)

clap derive emits `--help` per subcommand from the `about` and (optional) `after_long_help` attributes. The locked strings:

```rust
#[derive(Subcommand)]
enum Command {
    /// Encode a BIP-39 mnemonic (or hex entropy) as an ms1 string for engraving.
    #[command(after_long_help = "EXAMPLES:\n  ms encode --phrase \"abandon abandon … about\"\n  ms encode --phrase - < phrase.txt\n  ms encode --hex 00000000000000000000000000000000 --no-engraving-card\n  ms encode --phrase \"...\" --json | jq .ms1")]
    Encode(EncodeArgs),

    /// Decode an ms1 string back to its BIP-39 mnemonic and entropy bytes.
    #[command(after_long_help = "EXAMPLES:\n  ms decode ms10entrs…\n  ms decode - < engraved.txt\n  ms decode <ms1> --language french\n  ms decode <ms1> --json | jq .phrase")]
    Decode(DecodeArgs),

    /// Inspect an ms1 string's structural fields and decoder verdict.
    #[command(after_long_help = "EXAMPLES:\n  ms inspect <ms1>          # verdict + fields\n  ms inspect <ms1> --json   # structured output for tooling\n  printf \"ms10e ntrsq…\" | ms inspect -   # back-typed chunked form")]
    Inspect(InspectArgs),

    /// Verify an ms1 string is valid (and optionally round-trips against a phrase).
    #[command(after_long_help = "EXAMPLES:\n  ms verify <ms1>                          # exit 0 = valid v0.1\n  ms verify <ms1> --phrase \"abandon … about\"   # round-trip; exit 4 on mismatch\n  ms verify <ms1> --phrase \"...\" --json    # structured outcome")]
    Verify(VerifyArgs),

    /// Print the SHA-pinned v0.1 test-vector corpus as JSON.
    #[command(after_long_help = "EXAMPLES:\n  ms vectors                # compact JSON\n  ms vectors --pretty       # indented JSON\n  ms vectors | jq '.[0]'    # filter via jq")]
    Vectors(VectorsArgs),
}
```

Top-level `Cli::about` is `"ms — engrave-friendly BIP-39 entropy backups (the ms1 format)"`. clap auto-generates `ms --help` from the subcommand list. Examples in `after_long_help` use only documented invocations; no flag combinations not covered elsewhere in this SPEC.

---

## §3. Input/output discipline

### §3.1 Stdout vs stderr conventions

- **stdout** carries the data. Pipe-friendly: `ms encode | qrencode -t ANSI` and `ms encode | ms verify -` both work.
- **stderr** carries human-facing context — engraving card, warnings, errors. Tooling that wants stderr-clean operation can `2>/dev/null` or use `--no-engraving-card` (encode only).
- Errors always write a one-line `error: <friendly message>` to stderr in text mode; in `--json` mode the entire error structure is on stdout (per §5.4) so consumers parse one stream.

### §3.2 Stdin uniform behavior

Every command accepts its primary input (the ms1 string for decode/inspect/verify; the phrase for `encode --phrase`) from stdin via `-` or omitted argument. Stdin reader behavior:

1. Read until EOF.
2. Strip ALL Unicode whitespace (per `char::is_whitespace`) from the input.
3. Pass the resulting string to the parser.

This handles three cases with one mechanism:

- **Pipe round-trip:** `ms encode --phrase X | ms verify -` — encode emits multi-line stdout (ms1 + blank + chunked form); verify strip-whitespace concatenates the lines into the canonical ms1 string.
- **Engraver-typed-back chunked form:** `printf "ms10e ntrsq qqqqq…\n" | ms decode -` — the user typed back the chunked form from a steel plate, possibly with line wraps; whitespace strip recovers canonical ms1.
- **Terminal copy-paste artifacts:** trailing newlines, leading spaces, embedded tabs all tolerated.

A multi-ms1 file (two ms1 strings separated by blank lines) collapses to one giant invalid token and fails loudly with `Error::UnexpectedStringLength` — there is no string-splitting silent-pick-first hazard.

### §3.3 No interactive prompts

`ms` never reads from a TTY interactively. All input is either argv, file (via stdin redirection), or stdin pipe. Required arguments missing → clap usage error (exit 64).

---

## §4. Engraving card format (encode stderr)

When `encode` runs in text mode without `--no-engraving-card`:

```text
word count: <N>
language: <lang> (BIP-39 checksum valid)
passphrase: not stored in ms1 (record separately if used)
```

Additionally, the **chunked form** is part of stdout (not stderr) per Q6:

- Group size: 5 characters.
- Line wrap: 10 groups per line maximum (= 50 chars + 9 spaces = 59 chars wide).
- Wrap point: always at chunk boundary (never mid-chunk).
- Trailing partial group: allowed (e.g., a 75-char ms1 = 15 groups → line 1 has 10, line 2 has 5).

Example (75-char `ms10entrs…`):

```text
ms10e ntrsq qqqqq qqqqq qqqqq qqqqq qqqqq qqqqq qqqqq qqqqq
qqqqq qqqqq qqqqq cwugp dxtfm e2w
```

The chunked form is for proofreading by eye, not machine consumption. The canonical ms1 string is line 1 of the multi-line stdout block, suitable for `head -n 1`.

---

## §5. JSON output schemas

All `--json` outputs include `"schema_version": "1"` at the **top level**. v0.1.x adds fields additively (semver-minor); v0.2 may bump to `"2"`.

**Object key ordering** is the schema-declaration order shown in the examples below — `serde_json` preserves struct field declaration order, and ms-cli's output structs declare fields in the documented order. Tooling that diffs ms-cli outputs across runs / versions can rely on stable insertion order across v0.1.x; v0.2 may reorder freely (consumers should JSON-parse rather than diff verbatim if portability matters).

### §5.1 `encode --json`

`--phrase` invocation:

```json
{
  "schema_version": "1",
  "ms1": "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f",
  "language": "english",
  "word_count": 12,
  "entropy_hex": "00000000000000000000000000000000"
}
```

`--hex` invocation (no language was applied; `language` field is omitted):

```json
{
  "schema_version": "1",
  "ms1": "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f",
  "word_count": 12,
  "entropy_hex": "00000000000000000000000000000000"
}
```

Notes: `chunks` is intentionally absent — presentation, not data. `language` is omitted for `--hex` invocations (no language was applied — the user worked at the bytes layer). `word_count` is the BIP-39 word count for the entropy length, computable from `entropy_hex.len() / 2` regardless of input mode.

### §5.2 `decode --json`

```json
{
  "schema_version": "1",
  "entropy_hex": "00000000000000000000000000000000",
  "phrase": "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
  "language": "english",
  "word_count": 12,
  "language_defaulted": true
}
```

`language_defaulted` is `true` iff the user did not pass an explicit `--language`. Tooling can branch on this to surface the warning to its own users.

### §5.3 `inspect --json`

```json
{
  "schema_version": "1",
  "report": {
    "hrp": "ms",
    "threshold": 0,
    "tag": "seed",
    "share_index": "s",
    "prefix_byte": 1,
    "payload_bytes_hex": "aaaaaaaa…",
    "checksum_valid": true
  },
  "would_decode": false,
  "failure_reasons": ["reserved-tag-not-emitted", "non-zero-prefix"]
}
```

`failure_reasons` is a sorted, closed set of kebab-case tags drawn from §2.3's table. Stable across v0.1.x; v0.2 may add tags additively.

JSON value types: `prefix_byte` is a JSON **number** (not a hex string) representing the underlying `u8` (e.g., `1` for `0x01`, not `"0x01"`). `payload_bytes_hex` is a JSON string of lowercase hex characters. `threshold` is a JSON number (the digit value, not the ASCII byte). `tag` is a JSON string (UTF-8-decoded from the 4 codex32-alphabet bytes).

### §5.4 Error JSON schema (any command)

```json
{
  "schema_version": "1",
  "error": {
    "kind": "Bip39",
    "message": "BIP-39 checksum failure (last word does not match the entropy)",
    "exit_code": 1,
    "details": null
  }
}
```

`kind` is a closed set tied to `CliError` variants:

| kind | exit | details |
|---|---|---|
| `BadInput` | 1 | `null` (`message` carries detail) |
| `Bip39` | 1 | `null` |
| `Codex32` | 1 | `null` |
| `UnexpectedStringLength` | 1 | `{"got": <usize>, "allowed": [50, 56, 62, 69, 75]}` |
| `PayloadLengthMismatch` | 1 | `{"got": <usize>, "expected": [16, 20, 24, 28, 32]}` |
| `FormatViolation` | 2 | `{"underlying_kind": "<ms_codec_variant_name>", "fields": {...}}` |
| `FutureFormat` | 3 | `{"tag": "seed"}` (or whichever reserved tag was decoded) |
| `VerifyPhraseMismatch` | 4 | `null` (phrases are secrets — never in JSON output) |

`details` is **always present** in the JSON; the field carries `null` when no structured data exists for the variant. (Avoids the "is the field missing or is the value null?" ambiguity for strict JSON-schema consumers.) `kind` strings are stable across v0.1.x.

`exit_code` in JSON duplicates the process exit code for consumer convenience; the canonical signal is the process exit code itself.

---

## §6. Error handling and exit codes

Locked exit-code table:

| code | category | CliError variants |
|---|---|---|
| 0 | success | (no error) |
| 1 | user-input error | `BadInput`, `Bip39`, `Codex32`, `UnexpectedStringLength`, `PayloadLengthMismatch` |
| 2 | format violation | `FormatViolation` (covers `WrongHrp`, `ThresholdNotZero`, `ShareIndexNotSecret`, `TagInvalidAlphabet`, `UnknownTag`, `ReservedPrefixViolation`) |
| 3 | valid-but-future-version | `FutureFormat` (covers `ReservedTagNotEmittedInV01`) |
| 4 | verify round-trip mismatch | `VerifyPhraseMismatch` |
| 64 | usage error (clap) | usage / parse failure of argv |

**Note on exit 64.** Clap's default usage-error exit code is 2, but ms-cli reserves 2 for ms1 format-violation errors. The clap derive root in `main.rs` overrides the usage-error path to exit 64 explicitly (matching md-cli precedent at `crates/md-cli/src/main.rs:180-193`):

```rust
match Cli::try_parse() {
    Ok(cli) => /* dispatch */,
    Err(e) => {
        e.print().ok();
        return ExitCode::from(64);
    }
}
```

The 1/2 distinction matters: exit 1 = the user gave us garbage; exit 2 = the user gave us something that looks like an ms1 string but violates the wire format. Scripts can branch on this to distinguish "user typo" from "tool refused to accept malformed encoding."

Exit 3 is the script-readable signal "this is a valid future-version string." Tooling that wants to be forward-compatible can read `inspect`'s output to extract the reserved tag and route accordingly.

Exit 4 is the engraver's round-trip integrity signal: separate from generic exit 1 because it indicates a security-relevant inconsistency (engraving error, transcription bug, language mismatch, or worse) rather than a user-supplied-bad-input.

### §6.1 `CliError` enum

```rust
#[derive(Debug)]
pub enum CliError {
    /// User-input error: bad hex, missing args (exit 1).
    BadInput(String),
    /// BIP-39 phrase parse / checksum failure (exit 1).
    Bip39(bip39::Error),
    /// codex32 parse / checksum failure (exit 1).
    Codex32(codex32::Error),
    /// String length not in v0.1 set (exit 1).
    UnexpectedStringLength { got: usize },
    /// Payload byte length mismatch (exit 1).
    PayloadLengthMismatch { got: usize },
    /// Format violation — wrong HRP/threshold/share/tag/prefix (exit 2).
    /// Carries the originating ms-codec variant name + any structured fields.
    FormatViolation {
        underlying_kind: &'static str,  // e.g., "WrongHrp", "ReservedPrefixViolation"
        message: String,
        details: Option<serde_json::Value>,
    },
    /// Valid-but-future format (exit 3).
    FutureFormat { tag: [u8; 4] },
    /// Verify round-trip phrase mismatch (exit 4).
    VerifyPhraseMismatch,
}
```

`From<ms_codec::Error> for CliError` dispatches each ms-codec variant into the appropriate CliError, preserving structured fields where applicable. Implementation detail: `error.rs::from_ms_codec(e)` is the dispatch helper.

#### §6.1.1 Complete dispatch table (`ms_codec::Error` → `CliError` → JSON `details`)

For each variant of `ms_codec::Error`, the table below specifies the CliError variant, exit code, and the exact `details` JSON shape that appears in `--json` error output (per §5.4). 4-byte tag fields are encoded as JSON strings via `str::from_utf8` (defended by the codex32 alphabet being ⊂ ASCII for any tag that successfully parsed BIP-93 alphabet validation; for alphabet violators encountered defensively, the field falls back to `"got_hex": "<8-hex-chars>"`).

| `ms_codec::Error` variant | → `CliError` | exit | JSON `details` |
|---|---|---|---|
| `Codex32(e)` | `Codex32(e)` | 1 | `null` (`message` carries `friendly_codex32(&e)` text) |
| `WrongHrp { got }` | `FormatViolation { underlying_kind: "WrongHrp", … }` | 2 | `{"got": "<hrp-string>"}` |
| `ThresholdNotZero { got }` | `FormatViolation { underlying_kind: "ThresholdNotZero", … }` | 2 | `{"got": "<ASCII-digit-as-string>"}` |
| `ShareIndexNotSecret { got }` | `FormatViolation { underlying_kind: "ShareIndexNotSecret", … }` | 2 | `{"got": "<char>"}` |
| `TagInvalidAlphabet { got }` | `FormatViolation { underlying_kind: "TagInvalidAlphabet", … }` | 2 | `{"got_hex": "<8-hex-chars>"}` (bytes may not be UTF-8) |
| `UnknownTag { got }` | `FormatViolation { underlying_kind: "UnknownTag", … }` | 2 | `{"tag": "<utf8>"}` |
| `ReservedTagNotEmittedInV01 { got }` | `FutureFormat { tag: got }` | 3 | `{"tag": "<utf8>"}` |
| `ReservedPrefixViolation { got }` | `FormatViolation { underlying_kind: "ReservedPrefixViolation", … }` | 2 | `{"got": <u8-as-number>}` |
| `UnexpectedStringLength { got, allowed }` | `UnexpectedStringLength { got }` | 1 | `{"got": <usize>, "allowed": [50, 56, 62, 69, 75]}` |
| `PayloadLengthMismatch { tag, expected, got }` | `PayloadLengthMismatch { got }` | 1 | `{"tag": "<utf8>", "got": <usize>, "expected": [16, 20, 24, 28, 32]}` |

CliError-only variants (no ms-codec source):

| `CliError` variant | exit | JSON `details` |
|---|---|---|
| `BadInput(msg)` | 1 | `null` (`message` carries `msg`) |
| `Bip39(e)` | 1 | `null` (`message` carries `friendly_bip39(&e)` text) |
| `VerifyPhraseMismatch` | 4 | `null` (phrases are secrets — never in JSON output) |

The `kind` field in the §5.4 error JSON is the discriminant of the CliError variant — for `FormatViolation` it's the `underlying_kind` (e.g., `"WrongHrp"`); for the other variants it's the variant name itself (e.g., `"BadInput"`, `"Bip39"`, `"Codex32"`, `"UnexpectedStringLength"`, `"PayloadLengthMismatch"`, `"FutureFormat"`, `"VerifyPhraseMismatch"`).

### §6.2 Friendly mappers

Two modules own the variant-to-message translation:

- `codex32_friendly.rs::friendly_codex32(&codex32::Error) -> String` — covers all ~15 upstream variants of `codex32::Error` (see `/tmp/codex32-extract/codex32-0.1.0/src/lib.rs:42-83`). Stable since the upstream dep is exact-pinned `=0.1.0`.
- `bip39_friendly.rs::friendly_bip39(&bip39::Error) -> String` — covers `BadEntropyBitCount`, `BadWordCount`, `InvalidWord`, `InvalidChecksum`, `AmbiguousLanguages`, etc.

Both produce one-line user-facing messages. The unfriendly `{:?}` form is never shown to end users.

### §6.3 Display rules

Text-mode error display (stderr):

```text
error: <friendly message>
```

JSON-mode error display (stdout):

```json
{"schema_version": "1", "error": {"kind": "...", "message": "...", "exit_code": <N>, "details": ...}}
```

Errors in JSON mode go to **stdout**, not stderr — consumers parse one stream. Process exit code matches `CliError::exit_code()`.

---

## §7. BIP-39 wordlist languages

`--language` is a clap `value_enum` covering all 10 BIP-39 wordlists supported by `bip39 = "2"`:

| value | bip39 enum |
|---|---|
| `english` (default) | `Language::English` |
| `japanese` | `Language::Japanese` |
| `korean` | `Language::Korean` |
| `spanish` | `Language::Spanish` |
| `chinese-simplified` | `Language::SimplifiedChinese` |
| `chinese-traditional` | `Language::TraditionalChinese` |
| `french` | `Language::French` |
| `italian` | `Language::Italian` |
| `czech` | `Language::Czech` |
| `portuguese` | `Language::Portuguese` |

The CLI value is kebab-case; serde JSON serialization preserves kebab-case. clap rejects unknown values with usage error (exit 64).

The SPEC §6.3 wordlist hazard: ms1 v0.1 does NOT carry the language on the wire. Encode-time the user picks a language (default English); decode-time the user MUST pick the same language or get a silently-different mnemonic. ms-cli surfaces this:

- At encode time: stderr engraving card line `language: <lang>`.
- At decode time when `--language` was defaulted: stdout `language: english (default — verify against your records)` AND a non-suppressible stderr warning. Only `--language <explicit>` removes both.

The CLI cannot eliminate the hazard (ms1 v0.1 wire format is fixed); it can only make the hazard visible at every CLI surface.

---

## §8. Out-of-scope items deferred

| item | reason | future version |
|---|---|---|
| `ms generate` | toolkit territory; would need CSPRNG + secret-handling design | mnemonic-toolkit v0.1 |
| BIP-39 passphrase support | v0.1 wire does not encode it; SPEC §6.4 names the engraving anti-pattern | possibly never (toolkit may surface) |
| Bundle output (ms1 + mk1 + md1) | requires all three sibling crates published; integration is toolkit's job | mnemonic-toolkit v0.1 |
| K-of-N share commands | ms-codec v0.2 adds the wire format | ms-cli v0.2 |
| Color / TUI / interactive prompts | YAGNI for v0.1 batch tool | possibly never |
| `--phrase-file` flag | stdin via `--phrase -` covers the file-input case | n/a |
| `--no-color` flag | YAGNI for v0.1 (no color emitted anyway) | n/a |

---

## §9. Closures from brainstorm

The 9 brainstorm decisions plus mechanical defaults plus 2 architect-review iterations drive the surface above:

| # | Locked answer | SPEC section |
|---|---|---|
| Q1 | `encode` is phrase-first; `--phrase` headline, `--hex` escape hatch. | §2.1 |
| Q2 | `--language english` default with stderr engraving-card warning. | §2.1, §7 |
| Q3 | Surface = encode + decode + inspect + verify + vectors (no generate). | §2 |
| Q4 | ms-codec stays at v0.1.0; ms-cli targets it as-is. | §10 |
| Q5 | `decode` stdout = labeled block; `--json` for tooling. | §2.2, §5.2 |
| Q6 | `encode` stdout = ms1 + chunked form multi-line; stderr = engraving card. | §2.1, §4 |
| Q7 | `inspect` verdict-first + structured fields below. | §2.3, §5.3 |
| Q8 | Stdin uniform; whitespace-stripped. | §3.2 |
| Q9 | `verify` = exit-code by default; `--phrase` round-trips. | §2.4 |
| Mech-1 | Exit codes: 0/1/2/3/4/64 per §6 table. | §6 |
| Mech-2 | BIP-39 checksum validated at encode time. | §2.1 |
| Mech-3 | 10 BIP-39 languages via clap value-enum. | §7 |
| Mech-4 | Chunking 5-char groups, 10 groups/line, never mid-chunk. | §4 |
| Mech-5 | Vectors: in-tree `crates/ms-cli/vectors/v0.1.json`, parity-tested. | §2.5, §10.2 |
| Mech-6 | Friendly codex32 + bip39 mappers. | §6.2 |
| Arch-r1-C1 | Passphrase out-of-scope; stderr line on engraving card. | §2.1, §8 |
| Arch-r1-C2 | Strip-whitespace stdin readers across decode/verify/inspect. | §3.2 |
| Arch-r1-C3 | Decode default-English warning surfaces on stdout AND stderr. | §2.2, §7 |
| Arch-r1-I1 | Exit code 4 for `verify --phrase` mismatch. | §6 |
| Arch-r1-I2 | `friendly_bip39` mapper alongside `friendly_codex32`. | §6.2 |
| Arch-r1-I3 | Vectors corpus = in-tree copy, parity-tested. | §2.5, §10.2 |
| Arch-r1-I4 | `schema_version` field in every `--json` output. | §5 |
| Arch-r2-A1 | `schema_version` hoisted to top-level of error JSON. | §5.4 |
| Arch-r2-L1 | `FormatViolation` preserves structured `underlying_kind` + `details`. | §5.4, §6.1 |
| Arch-r2-L2 | `inspect` `failure_reasons` is closed set of kebab-case tags. | §2.3, §5.3 |
| Arch-r2-L3 | `encode --json` does not emit `chunks`. | §5.1 |

---

## §10. Reference implementation

`crates/ms-cli/` — the v0.1 deliverable. Layout:

```
crates/ms-cli/
├── Cargo.toml
├── vectors/v0.1.json                  — JSON-equal to ms-codec/tests/vectors/v0.1.json (parity test)
└── src/
    ├── main.rs                        — clap derive root, ExitCode dispatch
    ├── cmd/{mod,encode,decode,inspect,verify,vectors}.rs
    ├── format.rs                      — chunk(), engraving-card formatter, JSON output structs
    ├── parse.rs                       — input source resolution (arg | stdin); whitespace strip
    ├── error.rs                       — CliError enum + From<ms_codec::Error> + exit_code()
    ├── codex32_friendly.rs            — friendly_codex32(&codex32::Error) -> String
    ├── bip39_friendly.rs              — friendly_bip39(&bip39::Error) -> String
    └── language.rs                    — clap value_enum + serde + From<bip39::Language>
```

The vectors corpus is `include_str!`-baked at compile time directly from `cmd/vectors.rs`:

```rust
const VECTORS_V0_1_JSON: &str = include_str!("../../vectors/v0.1.json");
```

No `build.rs` is required — the macro reads the file at compile time relative to the source location. The corpus path is fixed at v0.1 (`v0.1.json`); future versions add new files (`v0.2.json`) rather than mutating the existing one (per RELEASE_PROCESS.md SHA-pin discipline).

### §10.0 Module dependency graph (build order)

```
        main.rs
           │
           ▼
       cmd/{encode, decode, inspect, verify, vectors}.rs
           │
           ▼
       format.rs ─── parse.rs ─── language.rs
           │           │              │
           ▼           ▼              ▼
                    error.rs
                    │      │
                    ▼      ▼
       codex32_friendly.rs   bip39_friendly.rs
```

Phase ordering for the IMPLEMENTATION_PLAN:

- **Phase 1 (leaves):** `error.rs`, `codex32_friendly.rs`, `bip39_friendly.rs`, `language.rs`, `format.rs`, `parse.rs`. No internal-crate dependencies; each can be implemented + unit-tested in isolation.
- **Phase 2 (commands):** `cmd/encode.rs`, `cmd/decode.rs`, `cmd/inspect.rs`, `cmd/verify.rs`, `cmd/vectors.rs`. Each consumes Phase 1 modules + the `ms-codec` library. Independent of each other.
- **Phase 3 (root):** `main.rs` glues the subcommand dispatch + `ExitCode` mapping. `cmd/mod.rs` re-exports.
- **Phase 4 (integration):** `crates/ms-cli/tests/*.rs` (the ~25 `assert_cmd` tests) + `vectors_parity.rs`.
- **Phase 5 (release prep):** `cargo publish --dry-run`, version bump, CHANGELOG, `--help` smoke test.

Dependencies (pinned in `crates/ms-cli/Cargo.toml`):

```toml
[dependencies]
ms-codec = { path = "../ms-codec", version = "=0.1.0" }
bip39 = "2"
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
```

No `anyhow` (we own `CliError`). No `getrandom` (no generate). No `tracing` / `log` (batch tool, not service).

### §10.1 Test strategy

**Unit tests** in each module:

- `format::chunk(&str, 5)` correctness across all 5 v0.1 lengths {50, 56, 62, 69, 75}; line-wrap at chunk boundary; max line width 59.
- `codex32_friendly` covers each upstream variant.
- `bip39_friendly` covers each bip39 variant.
- `CliError::exit_code()` returns 1/2/3/4 for each variant.
- `From<ms_codec::Error> for CliError` dispatches each ms-codec variant into the right CliError.
- `parse::strip_stdin_whitespace` handles canonical, chunked-back-typed, leading/trailing space, embedded newlines.
- `language.rs` value-enum: each of 10 languages parses; unknown rejected; serde JSON round-trips kebab-case.

**Integration tests** under `crates/ms-cli/tests/` using `assert_cmd = "2"` (~25 tests):

```text
encode_canonical_12_word.rs        — abandon-about → expected stdout (ms1 + chunked) + stderr engraving card
encode_canonical_24_word.rs        — abandon-art likewise
encode_hex_input.rs                — --hex 00…00 round-trips equivalent to --phrase
encode_rejects_bad_checksum.rs     — flipped last word → exit 1
encode_rejects_bad_language.rs     — Spanish words with --language english → exit 1
encode_rejects_odd_length_hex.rs   — friendly error on odd-length hex
encode_emits_passphrase_warning.rs — stderr contains "passphrase: not stored in ms1"
encode_no_engraving_card.rs        — flag suppresses stderr block; stdout unchanged
decode_round_trip.rs               — labeled-block format; default-language warning visible
decode_default_english_in_stdout.rs — stdout language line carries "(default — verify…)"
decode_explicit_language_no_warning.rs — explicit --language removes both warnings
inspect_valid_string.rs            — verdict OK + fields; --json schema check
inspect_non_zero_prefix.rs         — verdict FAIL with rule 8 reason tag
inspect_reserved_tag.rs            — id="seed" → rule 7 reason tag
inspect_multiple_failures.rs       — string with 2 violations reports both, sorted
verify_quiet_pass.rs               — exit 0, OK summary
verify_quiet_fail.rs               — exit 2, FAIL summary
verify_future_format.rs            — id="seed" → exit 3, OK future format
verify_phrase_round_trip_ok.rs     — exit 0
verify_phrase_round_trip_mismatch.rs — exit 4 (no phrase echoed to output)
encode_pipe_to_verify.rs           — `ms encode | ms verify -` exit 0 (validates strip-whitespace)
encode_pipe_to_decode.rs           — `ms encode | ms decode -` recovers original phrase
back_typed_chunked_form_decodes.rs — `printf "ms10e ntrsq…" | ms decode -` works
vectors_compact.rs                 — `ms vectors` parseable JSON; first vector matches spec
vectors_pretty.rs                  — `--pretty` indented, same content
inspect_codex32_parse_failure.rs   — bad-checksum input → exit 1; text-mode stderr error; --json error envelope on stdout (per §2.3.1)
json_error_envelope_per_kind.rs    — parametric: one assertion per CliError `kind` (BadInput/Bip39/Codex32/UnexpectedStringLength/PayloadLengthMismatch/FormatViolation/FutureFormat/VerifyPhraseMismatch); each verifies §5.4 schema
exit_codes_table.rs                — parametric: one row per CliError variant × text/JSON modes (~16 assertions)
```

**Vector-corpus parity test** (`tests/vectors_parity.rs`):

```rust
let cli_corpus: serde_json::Value = serde_json::from_str(include_str!("../vectors/v0.1.json")).unwrap();
let codec_corpus: serde_json::Value = serde_json::from_str(include_str!("../../ms-codec/tests/vectors/v0.1.json")).unwrap();
assert_eq!(cli_corpus, codec_corpus, "vectors corpus drifted between ms-cli and ms-codec");
```

Parsed-equality (not byte-equality) avoids spurious failures from whitespace / line-ending differences.

### §10.2 CI gates

Mirror ms-codec: `cargo build`, `cargo test`, `cargo clippy --all-targets -D warnings`, `cargo fmt --check`. Stable + beta + MSRV `1.85` three-row matrix. Per-phase opus reviews persist to `design/agent-reports/` (per established workflow); brainstorm/spec/plan reviews stay in conversation transcript.

### §10.3 Versioning + publishing

`crates/ms-cli/Cargo.toml`:

- `name = "ms-cli"` (the workspace package name).
- `version = "0.1.0-dev"` until release, then `0.1.0`.
- `[[bin]] name = "ms"` (the installed binary).
- `publish = true` from v0.1.0 (the Cargo.toml currently has `publish = false`; flip to `true` at release time).

The `ms` binary publishes to crates.io as `ms-cli`; `cargo install ms-cli` produces `~/.cargo/bin/ms`.

---

## §11. Cross-format and v0.x roadmap

`ms-cli` v0.1 stays narrow: BIP-39 entropy in/out, ms1 strings on the wire. Roadmap:

- **ms-cli v0.1.x patches:** fixups for v0.1 issues; library-side codex32 helper additions (track `ms-codec-validate-against-decode-rules-helper` FOLLOWUPS for `inspect`'s re-validator dedup).
- **ms-codec v0.2 (K-of-N shares):** ms-cli will gain `share split <ms1> --threshold K` and `share combine <share1> <share2> ... <shareK>`. The migration contract from MIGRATION.md governs the wire format; CLI is a thin user-facing wrapper.
- **mnemonic-toolkit v0.1:** separate repo / crate. Depends on `ms-codec`, `mk-codec`, `md-codec` as published artifacts. Provides `mnemonic bundle <phrase>` (emits ms1 + mk1 + md1 cards). ms-cli stays focused on the ms-format side; toolkit handles cross-format integration.

---

## Appendix A — provenance

This v0.1 SPEC was written 2026-05-04 by a multi-step brainstorm against the converged ms-codec v0.1.0 surface (just shipped at `bg002h/mnemonic-secret`). Pre-brainstorm audit: `agent-reports/audit-ms-codec-v0_1_0-pre-cli.md`. Brainstorm reviewer-loop: r1 surfaced 3 critical (passphrase, encode→verify pipeline, decode-language hazard) + 4 important (exit-code 4, bip39_friendly mapper, vectors path, schema_version) + 8 nits, all resolved inline with user-locked decisions; r2 surfaced 1 important (JSON schema hoisting) + 9 nits, all resolved inline; r3 (this SPEC's reviewer-loop) pending.

Per the 2026-05-03 workflow refinement, brainstorm/spec/plan reviewer reports stay in conversation transcript; only per-implementation-phase reports persist to `design/agent-reports/`.

## Revision history

(Tracks this SPEC's reviewer-loop convergence. Independent of brainstorm-stage architect rounds.)

- **r1** — 2026-05-04 initial draft from converged brainstorm.
- **r5** — 2026-05-04 r3 SPEC review terminator (0 critical / 0 important / 6 nits). 3 actionable r3 nits applied inline: §2.1 edge-case table gains an "extra spaces in phrase" row noting `bip39::parse_in` uses `split_whitespace` (resolves r3-N2); §2.1 hex-error row marked illustrative ("e.g.") rather than verbatim, since exact wording comes from the upstream `hex` crate (resolves r3-N3); §2.5 / §2.6 reordered in source so file order matches numerical order (resolves r3-N5). 3 r3 nits skipped as already-affirmations: r3-N1 (already correct), r3-N4 (Rust raw-string style is impl-detail), r3-N6 (em-dash matches md-cli precedent).

- **r4** — 2026-05-04 user-requested completion of all 5 deferred r2 SPEC-review nits inline (no longer FOLLOWUPS-deferred): §2.4.1 prose clarification on "first" meaning "earlier in pipeline" not severity (resolves r2-nit-1); §2.3.1 explicit acknowledgement that inspect cannot route exit 3 (resolves r2-nit-3); new §2.6 lockdown of per-subcommand clap `about` + `after_long_help` strings with concrete EXAMPLES blocks (resolves r2-nit-4); §5 preamble adds JSON key-ordering stability note (resolves r2-nit-6); §2.1 "Encoder pre-checks" gains an edge-case enumeration table covering empty/whitespace/short/non-hex/conflict/missing inputs (resolves r2-nit-7). Corresponding FOLLOWUPS entries `ms-cli-v01-spec-r2-nit-{1,3,4,6,7}` updated to status `resolved 2026-05-04`.

- **r3** — 2026-05-04 reviewer-loop terminator (r2 SPEC review returned 0 critical / 0 important / 8 nits; recommendation = ship for user review). 3 high-value nits applied inline: §5.4 locks `details` field as always-present (null when empty) to remove JSON-schema ambiguity; §2.1 adds explicit BIP-39 wordlist-mismatch behavior note (`bip39::parse_in` is language-strict; no silent transcoding); §2.4.1 step 1 adds concurrent-stdin guard (`ms verify - --phrase -` exits 1 with `BadInput`). Remaining 5 nits deferred to IMPLEMENTATION_PLAN or FOLLOWUPS at SPEC user-review time.

- **r2** — 2026-05-04 integrated 2 critical + 3 important findings from r1 SPEC review: §6.1.1 complete dispatch table mapping every `ms_codec::Error` variant → `CliError` + JSON `details` shape (resolves r1-C1 ambiguity); §2.4.1 verify validation-order subsection (resolves r1-C2 dispatch ambiguity); §2.3.1 explicit handling for `inspect()` BIP-93-parse failures (resolves r1-I3); §10.0 module dependency graph + build phase ordering (resolves r1-I2); §6 note clarifying clap exit-code override to 64 (resolves r1-nit on clap default); §5.1 second JSON example for `--hex` invocation (resolves r1-nit); §5.3 explicit JSON value-type clarifications for `prefix_byte` / `payload_bytes_hex` / `threshold` / `tag` (resolves r1-nit on JSON types); test surface gains `inspect_codex32_parse_failure.rs` + `json_error_envelope_per_kind.rs` (resolves r1-nits on missing tests). §10 module layout updated to drop `build.rs` (`include_str!` is sufficient) and clarify vectors corpus is JSON-equal not byte-equal to ms-codec (resolves nit on build.rs vs include_str! inconsistency, in lockstep with self-review fix to §2.5).
