# SPEC — ms-codec: `mnem` wordlist-language hint on the wire (Cycle 1 of the v0.2 split)

**Repo:** `mnemonic-secret` (ms-codec lib + ms-cli). **Branch:** `ms-v0.2-kofn-mnem`. **Base SHA:** `master` `4e5266a` (re-grep citations at impl time).
**Supersedes** the both-features `SPEC_ms_v0_2.md` draft, which R0 returned RED (3C/4I) and which decoupled into two cycles. **This is Cycle 1 (mnem only).** K-of-N shares → a separate later cycle (threshold-keyed shares + the SPEC_ms_v0_1 §5 / MIGRATION.md amendment) — **explicitly out of scope here.**
**Extends:** `SPEC_ms_v0_1.md` §6.3 (the non-English footgun) + §8 (deferred `mnem`). R0 review of the prior draft: `design/agent-reports/ms-v0-2-spec-R0-review.md`.
**Status:** **R0 GREEN** — R0 RED (0C/1I; decode-path gates under-scoped) → fold → R1 **GREEN (0C/0I)**. The central de-risk (byte-aligned mnem constructs for all 5 entropy lengths via `from_seed`, `data()` round-trips, set {51,58,64,70,77} disjoint from entr) is **empirically proven** by the reviewer's codex32 spike. Reviews: `design/agent-reports/ms-mnem-spec-R{0,1}-review.md` (+ the both-features `ms-v0-2-spec-R0-review.md` that drove the split). Cleared to plan-doc (its own R0 gate).

## 1. Goal
Put the BIP-39 wordlist language **on the `ms1` wire** so a recovered card is self-describing, fixing the §6.3 footgun (a non-English seed recovered via English-defaulted software silently derives a different master seed → empty wallet — today only mitigated by a stderr advisory, toolkit v0.37.11). A new `mnem` payload kind = entropy + a language byte, dispatched by the leading payload **prefix byte** the v0.1 SPEC reserved for new kinds. SemVer: **ms-codec 0.2.1 → 0.3.0**, **ms-cli 0.5.1 → 0.6.0** (both MINOR; v0.1 `entr` wire preserved byte-identically).

## 2. Wire format
The leading payload byte (before the entropy) is the type discriminator; the decoder dispatches on it first.

| prefix | kind | payload (pre-codex32) | string-length set |
|---|---|---|---|
| `0x00` | v0.1 `entr` (unchanged) | `[0x00][entropy:8N]` | {50,56,62,69,75} |
| `0x02` | **`mnem`** (this cycle) | `[0x02][lang_byte:8][entropy:8N]` — **byte-aligned** | **{51,58,64,70,77}** (verify in Phase 0) |
| `0x01`, `0x03..0xFF` | unallocated | — | — |

- **Byte-aligned, not bit-aligned (R0 C1).** A 4-bit `[0x02][lang:4][entropy]` layout is UNCONSTRUCTIBLE for 15/18/24-word seeds: codex32 0.1.0's `sanity_check` (lib.rs:113-116) rejects payloads whose trailing 5-bit group exceeds 4 bits, which `12+8N` bits trips for N=20/24/32. So the language takes a **full byte** (low nibble = code, high nibble reserved-0). This makes `mnem` encodable through the existing byte-oriented `from_seed(&[u8])` + decodable through `Parts::data()` — **no symbol-slicing / bit-path** (which the bit-aligned design would have forced). The "4-bit hint" is information-theoretic (10 languages fit in 4 bits) but physically a 1-byte field; correct SPEC_ms_v0_1 §6.3's "4-bit-encoded" wording to "1-byte language field (low nibble used)."
- **`0x01` is NOT used** by this cycle. The prior draft's "`0x01` = entr-share prefix" was crypto-unsound (R0 C2: distributed Shamir shares can't carry a stable payload byte — they key on the threshold field instead); K-of-N's later cycle will NOT claim `0x01`. It stays unallocated.
- **Prefix-byte registry** (normative, SPEC_ms_v0_1 §5): `0x00=v0.1-entr`, `0x02=v0.2-mnem`, all others unallocated/claim-via-PR.
- **Forward-readability:** v0.1 `0x00` strings round-trip unchanged (the decoder's `0x00` arm = today's path). A v0.1-only decoder rejects a `mnem` string (its length ∉ {50,56,62,69,75}; `Error::UnexpectedStringLength`) — acceptable (mnem is a v0.2 capability).

## 3. Codec changes (ms-codec)
- **`Payload` enum** gains `Mnem { language: u8, entropy: Vec<u8> }` (enum is `#[non_exhaustive]`, `payload.rs:28` — additive, no break). `entropy` validated ∈ {16,20,24,28,32} bytes exactly as `Entr`.
- **Language table** (wire-canonical, NEW in ms-codec — e.g. `consts.rs` or a small `mnem.rs`): low-nibble code → BIP-39 language, in `CliLanguage` declaration order (`ms-cli/src/language.rs:13-22`), **English = 0** (all-zero byte ⇒ English). `0 en · 1 ja · 2 ko · 3 es · 4 zh-Hans · 5 zh-Hant · 6 fr · 7 it · 8 cs · 9 pt · 10–15 reserved`. ms-cli's `CliLanguage` maps onto this (the wire invariant lives in the lib).
- **Dispatch becomes load-bearing.** `envelope.rs::discriminate` currently hardcodes the `0x00` prefix; extend to read `data[0]`: `0x00` → existing entr path (`id`-tag interpretation); `0x02` → mnem (`data[1]`=language byte → validate low-nibble∈0..9 + high-nibble==0, else `Error::MnemUnknownLanguage(u8)`; `data[2..]`=entropy). Any other prefix on an `id="entr"` string → error. (No share-grouping logic this cycle — that's K-of-N.)
- **Encode.** `mnem` goes through the same byte path as entr: `from_seed(HRP, 0, "entr", Fe::S, &[0x02, lang_byte, ..entropy])`. (id stays `"entr"`; the `0x02` prefix byte, not the id, distinguishes mnem — unambiguous because dispatch is on the prefix byte. R0: confirm.)
- **Decode** via `Parts::data()` (byte-aligned — `data[0]`=prefix, `data[1]`=lang, `data[2..]`=entropy). No bit manipulation.
- **`mnem` leaves the reject set:** remove `*b"mnem"` from `RESERVED_NOT_EMITTED_V01` (`consts.rs:39`) — it is now an emitted kind. (There is NO `RESERVED_TAG_TABLE`; the prior draft's cite was wrong — R0 I1. No random-`id` anti-collision is needed this cycle: mnem uses `id="entr"`, no sharing.)
- **`VALID_MNEM_STR_LENGTHS`** const + a bijection test (mirror `consts.rs:50-62`): `len = 9(header) + ⌈8(N+2)/5⌉ + 13(checksum)` for N∈{16,20,24,28,32}. **Phase 0 confirms the exact set empirically** (do not hardcode {51,58,64,70,77} until the spike verifies all 5 construct + pass `sanity_check`).
- New `Error` variants alphabetical (project convention): `MnemUnknownLanguage(u8)` (+ Display/`kind`/`validate` arms). New variants sorted alphabetically; pre-existing variants NOT retro-sorted (consistent with prior cycles).
- **v0.1-hardcoded gates that MUST learn the `0x02` arm (R0 I1 — enumerate exhaustively; each currently rejects/panics on a valid mnem string):**
  1. **`decode.rs:29` rule-9 length check** rejects any string length ∉ {50,56,62,69,75} *before* prefix dispatch → a mnem string (51/58/64/70/77) fails. **Fix:** the length gate must accept the **union** {50,56,62,69,75} ∪ {51,58,64,70,77}, then dispatch on the prefix byte and per-kind-validate the length (entr lengths ⟺ `0x00`; mnem lengths ⟺ `0x02`) — do NOT just widen the set without binding length-to-kind.
  2. **`Payload` impls** — add `Mnem` arms to `validate()` (entropy ∈ {16,20,24,28,32}; language low-nibble ∈ 0..9, high-nibble 0) and `kind()`/any exhaustive `match self` on `Payload` (M3). `#[non_exhaustive]` does not exempt in-crate matches.
  3. **(CLI, §5)** ms-cli `inspect::analyze()` (`cmd/inspect.rs:79,83`) pushes `non-zero-prefix`/`unexpected-string-length` for a `0x02`/non-v0.1-length string → prints `FAIL: would NOT decode`; and `decode::run()` (`cmd/decode.rs:57`) `_ => unreachable!()` **PANICS** on `Payload::Mnem`. Both must gain a `0x02`/`Mnem` arm.
- **Stale doc-comments (R0 M2):** `envelope.rs` source comments referencing the dropped `0x01` "entr-share" design / any `RESERVED_TAG_TABLE` mention must be corrected to the prefix-registry (`0x00` entr / `0x02` mnem; `0x01` unallocated).

## 4. AUTO default (`ms encode`)
- English (or `--language` unspecified, defaulting English) or `--hex` input → **`entr`** (`0x00`, byte-identical to v0.1 — zero ripple, zero risk).
- Non-English `--language` (with `--phrase`) → **`mnem`** (`0x02`) carrying that language.
- `--hex` carries no language → always `entr`.
Decode-unambiguous (prefix-byte dispatch). `ms encode` gains **no new flag** (reuses the existing `--language`, `encode.rs:38-39`).

## 5. CLI surface (ms-cli)
- **`ms encode`** — AUTO-routes per §4 (no new flag).
- **`ms decode`** — dispatches `0x02` (replace the `cmd/decode.rs:57` `_ => unreachable!()` with a `Payload::Mnem` arm — R0 I1): emits the BIP-39 phrase **in the on-wire language**; the card is self-describing, so no `--language` guess / no DEFAULT-language warning. **`--language` precedence (R0 M1):** for a `mnem` string the **wire language is authoritative** — if `--language L` is *also* passed and conflicts with the wire's language W, IGNORE L and emit a stderr warning (`note: --xpub/ms1 carries wordlist language W; ignoring --language L`). A user must not be able to silently override a self-describing card to the wrong language (that would re-open the §6.3 footgun in reverse). For `0x00` entr (no wire language): `--language` behaves exactly as today (defaulted to English + DEFAULT annotation).
- **`ms inspect`** — shows `kind: mnem` + the decoded language for `0x02`. **Explicit deliverable (R1 minor):** the `InspectReport`/`InspectReportJson`/`InspectJson` structs (`format.rs`) gain `kind` + `language` fields (currently absent) — add them, not just the text output.
- **No new subcommand, no new flag** ⇒ **NO GUI `schema_mirror` change** (verified: the gate is flag-NAME/subcommand set-equality; mnem adds neither). The `--json` `decode`/`inspect` shape gains a `language` field — ungated by schema_mirror (wire-shape isn't gated), so the GUI/toolkit JSON consumers self-update via the paired-PR rule if they read it.
- **Output-class advisory** (Phase-1 `advisory.rs`): `ms decode`/`inspect` of a `mnem` (or entr) string emit entropy → `private key material (can spend)` (unchanged class; mnem is still entropy).

## 6. CI / verification (R0 C3 — ms-codec has NO CI today)
The only workflow (`rust.yml`) is scoped to `crates/ms-cli/**`; there is **no ms-codec CI and no fmt step anywhere**. Do NOT write "CI gates it." This cycle: codify **local verification as a hard per-phase gate** — `cargo test -p ms-codec && cargo test -p ms-cli && cargo clippy --all-targets -- -D warnings && cargo +stable fmt --check --all` must pass at every phase commit. **File a FOLLOWUP** `ms-codec-no-ci-workflow` (add an ms-codec CI job) — fixing the gate gap is out of this small cycle's scope but must be recorded.

## 7. Test plan
- **Phase 0 spike (R0 C1 — the monoculture gate):** construct `mnem` for ALL FIVE entropy lengths {16,20,24,28,32} via `from_seed(HRP,0,"entr",Fe::S,&[0x02,lang,..entropy])`; assert each succeeds (passes `sanity_check`) + record the exact string lengths → pin `VALID_MNEM_STR_LENGTHS`. (A 12-or-21-word-only fixture would have hidden C1 — test all five.)
- **entr byte-identity:** every SHA-pinned v0.1 entr vector still produces byte-identical output (the dispatch change must not perturb the `0x00` path).
- **mnem round-trip:** for each language × each entropy length: `encode --language L --phrase …` → `0x02` string → `decode` recovers the entropy AND the language AND the correct phrase; `inspect` shows kind=mnem + language.
- **AUTO routing:** English/`--hex` → `0x00` (byte-identical to entr); non-English → `0x02`. 
- **language table:** each code 0–9 → correct language; codes 10–15 + high-nibble≠0 → `MnemUnknownLanguage`.
- **wire-correctness:** a hand-computed BIP-93/codex32 reference vector for ≥1 (language, entropy) pair confirms the on-wire bytes match the format exactly (not just self-round-trip).
- **prefix dispatch:** `0x00`→entr, `0x02`→mnem; a v0.1-only decode of a mnem string fails cleanly (`UnexpectedStringLength`).
- Full `cargo test` (ms-codec + ms-cli) + clippy `-D warnings` + `cargo +stable fmt --check --all`.

## 8. Lockstep / SemVer
- **ms-codec 0.2.1 → 0.3.0** (new `Payload::Mnem`, language table, prefix dispatch — wire-additive) + **ms-cli 0.5.1 → 0.6.0** (encode AUTO-route + decode/inspect mnem). Both MINOR; **crates.io publish** (ms is on crates.io). Family-stable token (SPEC §7) rolls to `ms-codec 0.3`.
- **manual** `mnemonic-toolkit/docs/manual/src/40-cli-reference/43-ms.md` — document the AUTO behavior + mnem (manual lives in the toolkit repo). Check for any CI-gated transcript that encodes a non-English seed (likely none → no re-capture).
- **toolkit** — re-pin ms-codec (lib, crates.io `0.2.1→0.3.0`) + ms-cli (tag `v0.5.1→v0.6.0`) in `Cargo.toml`/`install.sh`/`manual.yml`; toolkit `inspect`/decode paths learn the `0x02` arm (surface language). Toolkit's own ms1 EMIT path is UNCHANGED (it passes English/no-language → `0x00` entr).
- **GUI** — **no `schema_mirror` change** (no new flag/subcommand). If the GUI reads the `decode --json` `language` field, that's a self-update (wire-shape ungated) — paired-PR if pursued; not required for the gate.

## 9. Phasing (for the plan)
- **Phase 0 — spike (gate before any other work):** all 5 mnem lengths construct via `from_seed`; pin `VALID_MNEM_STR_LENGTHS`. Throwaway, but its result is load-bearing.
- **Phase 1 — codec:** `Payload::Mnem` + language table + `0x02` dispatch in `discriminate()` + mnem encode/decode via from_seed/data(); remove `mnem` from `RESERVED_NOT_EMITTED_V01`; `MnemUnknownLanguage`. **Gate: entr byte-identity (SHA vectors) + the hand-computed reference vector.**
- **Phase 2 — CLI:** `ms encode` AUTO-routing; `ms decode`/`inspect` `0x02` dispatch + language surfacing.
- **Phase 3 — lockstep + release:** manual `43-ms.md`; SPEC_ms_v0_1 §6.3 reword; toolkit re-pin + toolkit inspect `0x02` arm; file FOLLOWUP `ms-codec-no-ci-workflow`; version bumps + crates.io publish (gated on authorization).
- Mandatory opus R0 on this SPEC + the plan-doc + each phase + end-of-cycle.

## 10. Footguns (carry to plan-doc)
- **Phase-0 spike is non-negotiable** — verify all 5 lengths construct (R0 C1; the byte-aligned layout *should* pass `sanity_check` for all, but PROVE it, all five, before building on it).
- **entr byte-identity:** the dispatch change must not shift the `0x00` path by a byte (SHA vectors gate).
- **wire-correctness:** validate against a hand-computed BIP-93 reference, not just internal round-trip (a self-consistent-but-wrong packing round-trips green).
- **ms-codec has NO CI** (R0 C3) — local verification is the only gate; run the full suite + fmt at every phase commit; file the CI FOLLOWUP.
- **const names** — `TAG_ENTR` + `RESERVED_NOT_EMITTED_V01` (no `RESERVED_TAG_TABLE`); R0 I1.
- **Three v0.1-hardcoded gates reject/PANIC on a valid mnem string (R0 I1) — handle ALL:** `decode.rs:29` rule-9 length check (bind length↔kind, don't just widen), `Payload::validate()`/`kind()` exhaustive matches, and ms-cli `inspect::analyze()` (`cmd/inspect.rs:79,83`) + `decode::run()` (`cmd/decode.rs:57` `unreachable!()` PANICS). A per-phase test must decode/inspect a real mnem string end-to-end (not just the codec round-trip) to catch the CLI-layer gates.
- **decode `--language` vs wire (R0 M1):** for a mnem string the wire language WINS; a conflicting `--language` is warned-and-ignored (never silently honored — that re-opens §6.3 in reverse).
- **alphabetical Error variant + match-arm ordering** (project convention).
- **K-of-N is OUT of scope** — no `Threshold`, no `encode_shares`, no `0x01`, no SPEC §5 amendment this cycle.
- mnemonic-secret IS fmt-checked locally (edition 2024 → `cargo +stable fmt`), even though not CI-gated.
