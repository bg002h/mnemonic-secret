# ms-mnem PLAN R0 review

**Plan:** `design/IMPLEMENTATION_PLAN_ms_mnem_wordlist_language.md`
**Spec:** `design/SPEC_ms_mnem_wordlist_language.md` (R0 GREEN)
**Base SHA verified:** `master` `4e5266a` (HEAD = `4e5266ab86b7149712a601f613e2435f28baa98c`) — matches plan/spec.
**Reviewer:** opus architect, mandatory R0 gate (no code until 0C/0I).
**Sources verified live against:** ms-codec `src/{consts,payload,envelope,decode,encode,error,inspect,lib}.rs` + `tests/{vectors.rs,vectors/v0.1.json}`; ms-cli `src/{language,format}.rs` + `src/cmd/{encode,decode,inspect}.rs` + `tests/`; codex32 0.1.0 `/tmp/cx32_inspect/codex32-0.1.0/src/lib.rs`; toolkit `Cargo.toml` + `scripts/install.sh` + `.github/workflows/{manual.yml,rust.yml}` + `crates/mnemonic-toolkit/src/{error.rs,repair.rs,cmd/*,wallet_import/overlay.rs}`. Spike math re-derived independently (PASS, all 5).

---

## Verdict: RED (4C / 4I)

The wire format, length set, AUTO routing, byte-identity gate, spike, SemVer, no-CI premise, and K-of-N exclusion are all **sound and verified**. RED is driven by **seam-signature under-specification** (the plan's encode/decode steps cannot be implemented as written without an unstated type change) and **two real footgun-class gaps** (a wrong-language toolkit derivation site + the ms-cli inspect gate logic never named) — exactly the test-invocation / source-anchor fidelity defect-class the project lesson warns about. All four Criticals are *specification completeness* defects, not design errors; folding is mechanical.

---

## Critical

### C1 — `package()` cannot reach the language byte; seam signature change unspecified (Task 1.3 Step 3)
**Evidence:** `envelope.rs:147` — `pub(crate) fn package(tag: Tag, payload_bytes: &[u8]) -> Result<Codex32String>`; called once at `encode.rs:26` as `envelope::package(tag, payload.as_bytes())`. `Payload::as_bytes()` (`payload.rs:73`) returns ONLY the inner `Vec<u8>` (for the planned `Mnem` arm it returns `entropy` per Task 1.2 Step 3 — **no language byte, no prefix**).
**Problem:** Task 1.3 Step 3 says "`package` … generalize to prepend the kind's prefix byte: for `Payload::Mnem{language,entropy}` build `[MNEM_PREFIX, language] ++ entropy`." But `package` receives `&[u8]` (= `payload.as_bytes()` = bare entropy) and **has no way to know the kind or read `language`.** As written the language byte is lost at the `encode.rs:26` boundary.
**Fix:** Pick ONE and state it in the plan: **(a)** change `package`'s signature to `package(payload: &Payload) -> Result<Codex32String>` (it then matches on the kind, prepends `0x00` for Entr / `[0x02, language]` for Mnem, derives `tag.as_str()`="entr" internally); update the single call site `encode.rs:26` to `envelope::package(payload)` and the 4 in-module `package(Tag::ENTR, &entropy)` tests (`envelope.rs:252,264`); OR **(b)** keep `package(tag, &[u8])` and assemble the full `[prefix,(lang,)entropy]` byte vector inside `encode()` (which holds the `&Payload`), passing the assembled bytes — but then `package`'s `data.push(RESERVED_PREFIX)` at `envelope.rs:156` must be removed so it doesn't double-prepend. Either is fine; the plan MUST commit to one, because the current Step 3 text is uncompilable (it asks `&[u8]` to yield a language byte).

### C2 — `discriminate()` return-type change unspecified; Step 4 and Step 5 describe two different designs (Task 1.3 Steps 4-5)
**Evidence:** `envelope.rs:91` — `pub(crate) fn discriminate(c: &Codex32String) -> Result<(Tag, Vec<u8>)>`; returns `(tag, payload_with_prefix[1..].to_vec())` at `:137` (prefix STRIPPED). Single call site `decode.rs:41` `let (tag, payload_bytes) = envelope::discriminate(&c)?;`, then `decode.rs:56-69` constructs `Payload::Entr` itself and validates. Four in-module tests destructure `(tag, recovered)`: `envelope.rs:210,219,253` + the reject tests.
**Problem:** Plan Step 4 says discriminate should "→ `Payload::Mnem{language: rest[0], entropy: rest[1..]}` (then `.validate()`)" — i.e. discriminate now **returns a `Payload`** (and runs validate). But Step 5 + the live `decode.rs:56-69` construct the `Payload` in `decode()` and need the **prefix byte / kind** to "bind length↔kind." The two steps imply incompatible seam contracts: (Step4) discriminate yields `Payload`, vs (Step5 + existing decode) decode yields the kind from a returned prefix byte. The plan never states discriminate's new signature, never mentions the 4 tests that break on a signature change, and never resolves where `.validate()` runs (discriminate vs decode — today it's decode at `:61`).
**Fix:** State discriminate's new signature explicitly. Cleanest given the live code: `discriminate(c) -> Result<(Tag, Payload)>` — it reads `data[0]`, builds `Payload::Entr(rest)` (0x00) or `Payload::Mnem{language:data[1], entropy:data[2..]}` (0x02), returns `(tag, payload)`; `decode()` then runs `payload.validate()?` (rule 10) + the kind↔length bind (rule 9, see C3-adjacent). Update the 4 in-module discriminate/package tests that currently destructure `(tag, Vec<u8>)`. (Alternatively return `(Tag, u8 /*prefix*/, Vec<u8> /*rest*/)` and keep Payload-construction in decode — but pick one and write it.)

### C3 — ms-cli inspect: the actual gate (`analyze()`) is never named; the `would_decode` rewalk + data-flow for `kind`/`language` is unspecified (Task 2.4)
**Evidence:** The function that produces the FAIL verdict is `fn analyze(report: &InspectReport, str_len: usize) -> (bool, Vec<&'static str>)` at `cmd/inspect.rs:46-92`. It pushes `"non-zero-prefix"` at `:80` (guard `:79 report.prefix_byte != 0x00`) and `"unexpected-string-length"` at `:84` (guard `:83 !VALID_STR_LENGTHS.contains(&str_len)`). `would_decode = reasons.is_empty()` (`:91`). The lib struct `ms_codec::InspectReport` (`ms-codec/src/inspect.rs:13-28`) has **no `kind`/`language` field** and `ms_codec::inspect()` does **not** dispatch on the prefix byte — it surfaces `prefix_byte=0x02` raw but classifies nothing. The CLI emit structs are `format.rs:73 InspectReportJson` + `format.rs:85 InspectJson`.
**Problem:** Task 2.4 cites `inspect.rs:80,84,96,102` (correct line nums) but never names `analyze()` or its `(would_decode, reasons)` contract — yet `analyze` IS the gate that must learn the `0x02` arm (a `0x02`/mnem-length string must yield `reasons.is_empty()==true`, kind=mnem). It also says "Add `kind`+`language` fields to the inspect report struct(s)" without specifying WHERE the language is decoded: ms-codec's `InspectReport` has no language, and `ms_codec::inspect()` doesn't compute it. So the data must come from either (a) extending `ms_codec::inspect()` to return kind+language (a lib change — then bump rationale + a lib test), or (b) ms-cli decoding `report.prefix_byte==0x02 → report.payload_bytes[0]` into a language locally. The plan picks neither.
**Fix:** Specify: (1) `analyze()` gains prefix dispatch — `0x02` + `VALID_MNEM_STR_LENGTHS` length ⇒ no `non-zero-prefix`/`unexpected-string-length` push (and the rule-10 entr check at `:87` must not fire for mnem); (2) decide whether `kind`/`language` are computed in `ms_codec::InspectReport` (lib) or derived in ms-cli from `prefix_byte`+`payload_bytes`; (3) add the fields to `InspectReportJson` (`format.rs:73`) + populate in `emit_json` (`inspect.rs:131-143`) + `emit_text` (`inspect.rs:118-127`); (4) fix the reason-text literal `[50, 56, 62, 69, 75]` at `inspect.rs:96` so it doesn't claim mnem lengths are invalid. Note: the plan's "ms-codec format.rs or ms-cli" hedge is wrong on the path — there is **no `ms-codec/src/format.rs`**; `InspectReport` lives in `ms-codec/src/inspect.rs`.

### C4 — Phase-3 toolkit consume-sites un-enumerated; one re-opens the §6.3 footgun the cycle is fixing (Phase 3 Step 3)
**Evidence:** The toolkit consumes `ms_codec::decode` at 6+ sites that match `Payload`; all already carry wildcard arms (so `#[non_exhaustive]` + new `Mnem` will **compile**, not break — but silently mis-handle):
- **`cmd/xpub_search/seed_intake.rs:166-173`** — `Ok((_tag, payload))` → `payload.as_bytes()` → `Mnemonic::from_entropy_in(args.language().into(), …)`. **Uses the CLI `--language`, NOT the wire language.** A Japanese `mnem` ms1 fed to `xpub-search` would re-derive under English → wrong master seed → empty wallet. This is *exactly* the §6.3 footgun, re-opened inside the toolkit. (Note `as_bytes()` returns entropy for Mnem, so it won't error — it silently derives wrong.)
- **`cmd/bundle.rs:1613-1625`** (`_ =>` → "payload is not entropy" error), **`cmd/convert.rs:1446-1458`** (`_ =>` error), **`cmd/silent_payment.rs:135-143`** (`_ =>` error), **`wallet_import/overlay.rs:127-139`** (`Ok(_) =>` error) — all REJECT a valid mnem string post-cycle. **`cmd/verify_bundle.rs:2400-2405`** (`Ok(_) | Err(_) => continue`) silently SKIPS mnem verification. **`cmd/inspect.rs:171`** returns `InspectPayload::Ms1{tag, payload}` then formats — need a Mnem arm in `emit_inspect_text`.
- SAFE: `repair.rs:818 repair_via_ms_codec` discards `_tag`/`_payload`, operates on the string-correction layer → Mnem-agnostic; it will even (correctly) BCH-repair corroded mnem cards.
**Problem:** SPEC §8 + plan Phase 3 Step 3 say only "toolkit inspect/decode paths learn the `0x02` arm (surface language)" — far too vague. It names neither `seed_intake.rs` (the footgun) nor the 5 reject/skip sites, and gives no decision on whether rejecting a mnem in bundle/convert/silent-payment/overlay is acceptable-out-of-scope or a regression.
**Fix:** Enumerate every `ms_codec::decode` consume-site in Phase 3 (grep: `ms_codec::decode` in toolkit src = the 6 above). At minimum, `seed_intake.rs` MUST source the wire language from `Payload::Mnem.language` (overriding `args.language()`) — non-negotiable, it's the cycle's own bug class. For the reject sites, make an explicit per-site call: either accept Mnem (extract entropy, use wire language) or document the deliberate reject. State the toolkit PATCH bump only after the consume-side is decided. (Compile is safe; correctness is not.)

---

## Important

### I1 — Second/third length-gate sites + the `allowed` reporting set not addressed (Task 1.3 Step 5)
**Evidence:** Beyond `decode.rs:29` (the gate the plan names), `VALID_STR_LENGTHS` is also referenced at `decode.rs:32` (the `allowed:` field of the rule-9 error), `decode.rs:135-138` (`parse_ms1_symbols` stand-in error inside `decode_with_correction`), and `envelope.rs:70` (`extract_wire_fields` too-short defensive error). After widening rule-9 to the union, a genuinely-out-of-set length will still report `allowed: VALID_STR_LENGTHS` (entr-only) — misleading.
**Fix:** Task 1.3 Step 5 should (a) widen the rule-9 *accept* set to the union AND bind length↔kind after prefix dispatch (the substance — correct as planned), and (b) update the `allowed:` reported set (a union const, or per-kind) so the error message isn't a lie. `decode.rs:135` / `envelope.rs:70` are defensive stand-ins (only hit on invalid-char / too-short, never on a real mnem) → Minor, but mention them so they aren't "fixed" wrongly.

### I2 — `decode_with_correction` BCH path interaction with the union gate not called out (Task 1.3 / Phase 3)
**Evidence:** `decode_with_correction` (`decode.rs:188-246`) calls `decode(s)` at `:201` (clean) and `:244` (post-correction). Once rule-9 accepts the union, a clean OR BCH-corrected mnem string flows through and returns `Payload::Mnem`. The toolkit's `repair_via_ms_codec` (`repair.rs:820`) consumes this — it's Mnem-safe (discards payload), but the codec-level `decode_with_correction` will now return a `Mnem` tuple to any caller.
**Fix:** Add a Task-1.3 note that `decode_with_correction`'s two `decode()` calls inherit the union gate automatically (no separate change needed) and a one-line test that a (clean) mnem string round-trips through `decode_with_correction` to `Payload::Mnem` with empty corrections. Low effort, closes the "BCH path silently wrong" risk.

### I3 — Toolkit ms-cli tag re-pin FROM-version is wrong at both sites (Phase 3 Step 2)
**Evidence:** Plan Step 2 says "bump … ms-cli tag pin `v0.5.1→v0.6.0` (`install.sh` + `manual.yml`)." Live: `scripts/install.sh:38` = `ms-cli-v0.5.0`; `.github/workflows/manual.yml:88` = `cargo install … --tag ms-cli-v0.5.0 ms-cli`. **The current pin is `ms-cli-v0.5.0`, not v0.5.1** (ms-cli v0.5.1 published to crates.io but the toolkit install pins were never bumped — a pre-existing lag). Tag format is the prefixed `ms-cli-vX.Y.Z`.
**Fix:** Correct Step 2 to bump `ms-cli-v0.5.0 → ms-cli-v0.6.0` at both sites (note the `ms-cli-` prefix). The ms-codec **lib** pin is correct: `crates/mnemonic-toolkit/Cargo.toml:20` = `ms-codec = "0.2.1"` → `0.3.0`. Also run the `sibling-pin-check.yml` logic (it cross-checks install.sh vs manual.yml tags) — the plan correctly calls this, just with the wrong FROM literal.

### I4 — No real Japanese (or any non-English) BIP-39 test phrase exists; Tasks 2.2/2.3 use a placeholder (Tasks 2.2, 2.3)
**Evidence:** Grep across `crates/ms-cli/tests` + `src` for japanese/non-Latin: the only hit is `encode_rejects_bad_language.rs` which feeds an *English* phrase under `--language japanese` (a NEGATIVE test) — there is **no checksum-valid Japanese phrase fixture anywhere.** Tasks 2.2/2.3 write `"<12-word ja phrase>"` / `"<english>"` as placeholders.
**Fix:** Provide a concrete, checksum-valid non-English fixture (e.g. generate via `bip39::Mnemonic::from_entropy_in(Language::Japanese, &[0u8;16])` and pin its `.to_string()`, or lift a known BIP-39 Japanese test vector). State it in the plan so the implementer doesn't invent an invalid phrase (which would make Task 2.2's "fails→passes" TDD spuriously pass/fail). Also pin the expected mnem string length (51 for 12-word) explicitly, not `∈{51,…}`.

---

## Minor

- **M1 — SPEC_ms_v0_1 §6.3 reword target.** Live text to reword is `SPEC_ms_v0_1.md:248` ("entropy + 4-bit-encoded wordlist-language hint … fit in 4 bits"). Plan Step 1's reword target string matches. Also `SPEC_ms_v0_1.md:161` (the `mnem` table row "length TBD in v0.2+") and `:59` ("future v0.2+ … addresses this") become partially stale — consider touching `:161` ("length TBD" → the now-known set) in the same reword.
- **M2 — bijection-test cite off by 3 lines.** Plan Task 1.1 Step 2 says "mirroring `consts.rs:45-59`"; the live entr bijection test spans `consts.rs:45-62` (`mod tests` body `:49-62`). The plan's proposed formula `9 + ceil(8*(entr+2)/5) + 13` is **mathematically correct for all 5** (verified: 51/58/64/70/77), but note the live test uses `(entropy_bytes + 1) * 8` then `.div_ceil(5)` — the mnem mirror should use `(entropy_bytes + 2) * 8` (prefix + lang) `.div_ceil(5)`. Plan's `ceil(8*(N+2)/5)` is equivalent. Just fix the line range.
- **M3 — stale rust.yml comment (not this plan's defect, but adjacent).** `.github/workflows/rust.yml:6` claims "ms-codec has its own separate workflow" — FALSE (only `rust.yml` exists, path-scoped to `crates/ms-cli/**`). The SPEC/plan "ms-codec has NO CI" premise is therefore CORRECT and the FOLLOWUP `ms-codec-no-ci-workflow` is valid (verified not already filed in `design/FOLLOWUPS.md`). Optionally have Phase 3 Step 1 also correct that stale comment.
- **M4 — `payload.rs:74` cite is `:73`.** The `as_bytes()` accessor `match self` is at `payload.rs:73-77` (plan says `:74`). `validate()` is `:50` ✓, `kind()` is `:66` ✓. Off-by-one only.
- **M5 — decode JSON shape for mnem.** `format.rs:48 DecodeJson` already has `language` + `language_defaulted`. For a mnem string, `language_defaulted` should be `false` (wire-authoritative). Plan Task 2.3 implies this but doesn't state the JSON field semantics — note that `language_defaulted=false` + wire language for mnem.

---

## Spec-coverage matrix (§ → task → status)

| SPEC § | Requirement | Plan task | Status |
|---|---|---|---|
| §2 wire (byte-aligned, prefix table, {51..77}) | Phase 0 + Task 1.1 | ✓ (spike math re-verified PASS) |
| §2 0x01 stays unallocated | (absence) | ✓ no 0x01 anywhere |
| §3 `Payload::Mnem` variant | Task 1.2 Step 3 | ✓ |
| §3 language table (en=0..pt=9) | Task 1.1 Step 1 | ✓ (order matches `CliLanguage` live) |
| §3 dispatch in `discriminate` | Task 1.3 Step 4 | ✗ **C2** (signature/contract unspecified) |
| §3 encode via from_seed | Task 1.3 Step 3 | ✗ **C1** (package can't reach lang byte) |
| §3 decode via `data()` | Task 1.3 Step 4-5 | ✗ **C2** |
| §3 remove `mnem` from RESERVED_NOT_EMITTED_V01 | Task 1.1 Step 1 | ✓ (`consts.rs:39` confirmed) |
| §3 length-gate union + bind-to-kind | Task 1.3 Step 5 | ◑ substance ✓, **I1** (2nd/3rd sites + `allowed`) |
| §3 validate()/kind()/as_bytes() arms | Task 1.2 Step 3 | ✓ (lines 50/66/73) |
| §3 `VALID_MNEM_STR_LENGTHS` + bijection | Task 1.1 Step 1-2 | ✓ (formula correct; **M2** line range) |
| §3 `MnemUnknownLanguage` (alphabetical) | Task 1.1 Step 3 | ✓ |
| §3 gate 1: decode.rs rule-9 | Task 1.3 Step 5 | ◑ **I1** |
| §3 gate 2: Payload exhaustive matches | Task 1.2 Step 3 | ✓ |
| §3 gate 3a: ms-cli inspect analyze | Task 2.4 | ✗ **C3** (analyze() never named) |
| §3 gate 3b: ms-cli decode unreachable!() | Task 2.3 | ✓ (`cmd/decode.rs:57` confirmed) |
| §3 stale 0x01/RESERVED_TAG_TABLE doc-comments | Task 1.3 Step 4 | ✓ (`envelope.rs:86-90` target) |
| §4 AUTO default (en/hex→entr, non-en→mnem) | Task 2.2 | ✓ |
| §5 decode wire-wins precedence + warn | Task 2.3 | ✓ (logic sound; **M5** JSON note) |
| §5 inspect kind+language report fields | Task 2.4 | ✗ **C3** (data-flow + struct location) |
| §5 no GUI schema_mirror change | Phase 3 (absence) | ✓ (no new flag/subcommand — verified) |
| §6 no-CI local gate + FOLLOWUP | every phase gate + P3.1 | ✓ (premise verified true; **M3**) |
| §7 spike (all 5) | Phase 0 | ✓ PROVEN |
| §7 entr byte-identity | Task 1.4 Step 1 | ✓ (`tests/vectors.rs` asserts `s==v.ms1`) |
| §7 wire-correctness hand vector | Task 1.4 Step 2 | ✓ |
| §7 round-trip / AUTO / table / dispatch | Tasks 1.2-1.4, 2.2-2.4 | ◑ (**I4** no real ja phrase) |
| §8 lockstep (toolkit/manual/SemVer) | Phase 3 | ◑ **I3** (wrong FROM tag) + **C4** (consume-sites) |
| §8 toolkit inspect/decode 0x02 arm | Phase 3 Step 3 | ✗ **C4** |
| §9 phasing | matches | ✓ |
| §10 footguns (8) | embedded | ◑ (precedence ✓, but C4 = toolkit-side §6.3) |
| §K-of-N OUT | (absence) | ✓ (no Threshold/encode_shares/0x01/§5-amend) |

---

## Source-anchor & TDD verification

**Anchors (cite → live → status):**
- `consts.rs` `RESERVED_PREFIX=0x00` :17 ✓; `VALID_ENTR_LENGTHS` :29 ✓; `VALID_STR_LENGTHS={50,56,62,69,75}` :33 ✓; `TAG_ENTR=*b"entr"` :36 ✓; `RESERVED_NOT_EMITTED_V01` incl `*b"mnem"` :39 ✓ (removal target correct); bijection test :45-62 (plan said :45-59 → **M2**).
- `payload.rs` `PayloadKind #[non_exhaustive]` :10-11 ✓; `Payload #[non_exhaustive]` :28-29 ✓ (plan cited :29 ✓); `validate()` :50 ✓; `kind()` :66 ✓; `as_bytes()` :73 (plan said :74 → **M4**).
- `envelope.rs` `discriminate` :91 ✓; prefix check `data[0]!=RESERVED_PREFIX` :131 ✓; `package` :147 ✓; stale 0x01-share doc :86-90 ✓.
- **`package` signature = `(tag: Tag, payload_bytes: &[u8])`** — confirmed `:147`; takes `&[u8]` NOT `&Payload`. This is the crux of **C1**. Kind-routing is better placed by either changing the signature to `&Payload` OR assembling bytes in `encode()`; the plan must choose. `encode(tag,&Payload)` lives in `encode.rs:16`; it calls `package(tag, payload.as_bytes())` at `:26`.
- `decode.rs` rule-9 gate :29 + `Error::UnexpectedStringLength` :30 ✓ (BEFORE `from_string` :38 and `discriminate` :41 — the union-fix IS needed, correct); plan's "`decode.rs:57 _ => unreachable!()`" is **NOT** in ms-codec decode.rs (`:57` there is `x if x == TAG_ENTR =>` the entr arm). The `unreachable!()` is in **ms-cli `cmd/decode.rs:57`** — plan Task 2.3 correctly scopes it there ✓. (SPEC §3/§10 also cite `cmd/decode.rs:57` ✓.)
- `encode.rs:90` "emits Payload::Entr" — that's **ms-cli `cmd/encode.rs:90`** (`ms_codec::encode(Tag::ENTR, &Payload::Entr(...))`) ✓; ms-codec `encode.rs` has no `:90` Entr-emit (it's generic). Plan Task 2.2 correctly targets `cmd/encode.rs:90` ✓.
- `cmd/inspect.rs:80,84,96,102` — `:80` push `non-zero-prefix` ✓ (guard `:79`); `:84` push `unexpected-string-length` ✓ (guard `:83`); `:96` reason-text `unexpected-string-length` w/ literal `[50,56,62,69,75]` ✓; `:102` reason-text `non-zero-prefix` ✓. BUT the gate FUNCTION `analyze()` (`:46-92`) is never named → **C3**.
- `language.rs` `CliLanguage` 10 variants English-first :12-23 ✓ (declaration order = `MNEM_LANGUAGE_NAMES`); has `as_str()` :27 but **NO `code()`/`from_code()`** → plan Task 2.1 correctly adds them ✓.
- inspect report structs: `InspectReportJson` `format.rs:73` ✓; `InspectJson` `format.rs:85` ✓; **`InspectReport` is in ms-codec `inspect.rs:13`, NOT `format.rs`** — plan's "ms-codec format.rs or ms-cli" hedge is path-wrong (no ms-codec/format.rs) → folded into **C3**.

**Spike validity:** ✓ PROVEN. Independently re-derived: data bytes B=N+2 ∈{18,22,26,30,34}; symbols=⌈8B/5⌉∈{29,36,42,48,55}; `sanity_check` incomplete_group=`(symbols·5)%8`∈{1,4,2,0,3} — all ≤4 → **all 5 construct**; total len=9+symbols+13={51,58,64,70,77}, disjoint from entr {50,56,62,69,75}. `Parts::data()` (`codex32 lib.rs:399`) byte-granular, returns exactly B bytes for whole-byte payloads → `back[0]=0x02, back[1]=lang, back[2..2+n]=entropy` valid. Matches SPEC R0 reviewer's empirical spike (review §51-68). The spike test as written will PASS. (Spike correctly tests all 5, not a monoculture — closes the C1 gate.)

**Gate ordering (Task 1.3 Step 5):** Sound in principle — rule-9 widen-to-union happens at `decode.rs:29` (before parse), kind-bind after prefix dispatch. Because length sets are **disjoint** AND dispatch is on the prefix byte, no mnem length can be honored as entr or vice-versa (defense-in-depth). The ordering is correct *once C2 resolves how the kind/prefix reaches the bind point* — the bind needs the prefix byte, which today's `discriminate` strips and discards. So I1+C2 are coupled: fix discriminate to surface the kind, then bind.

**TDD per task:**
- Task 1.2 (Mnem validate/kind): would fail-then-pass ✓ (`Mnem` doesn't exist → compile-fail → exists → pass). Note: the `language:0x10` "high nibble set" case is subsumed by `language>=10` per the plan's own note — fine.
- Task 1.3 (round-trip): would fail-then-pass ONLY after C1+C2 resolved (as written, the encode side drops the language byte → the round-trip test would fail to even compile or would lose the language). ✗ blocked on C1/C2.
- Task 1.4 (entr byte-identity): ✓ `cargo test -p ms-codec --test vectors` is the right invocation; `vectors.rs:39` asserts `s == v.ms1` byte-for-byte; `vectors/v0.1.json` exists (2.2KB). Pin holds.
- Task 1.4 Step 2 (wire vector): ✓ sound (hand/spike-golden).
- Task 2.2 (encode AUTO): would fail-then-pass ✓ in shape, but **I4** — no real `<12-word ja phrase>` fixture exists; placeholder must be replaced with a checksum-valid phrase or the test can't run.
- Task 2.3 (decode Mnem + precedence): ✓ in shape (`cmd/decode.rs:57 unreachable!()` PANICS → arm added). `args.language` (raw `Option`, `:29`) is still available at `:44` to detect "explicitly passed" → precedence logic implementable. **I4** fixture needed.
- Task 2.4 (inspect 0x02): ✗ **C3** — the failing-then-passing target is `analyze()`'s `(would_decode,reasons)`, not the push-lines in isolation; plan must target the function + the report-struct data-flow.

---

## Notes

- **Compile-safety of Phase 3 (good news):** every toolkit `match` on `ms_codec::Payload` already has a wildcard arm (`#[non_exhaustive]` forced it), and `ms_codec_exit_code` (`toolkit error.rs:352`, `_ => 1`) + `From<ms_codec::Error>` (`:815`, `other => MsCodec(other)`) both catch-all. So `Payload::Mnem` + `MnemUnknownLanguage` will **compile** in the toolkit with zero changes. C4 is therefore a *silent-correctness* gate, not a build gate — which is exactly why it's dangerous (no compiler nudge) and must be enumerated, not left to "learn the arm."
- **The genuinely-proven core is large:** wire format, length set, byte-path, AUTO routing, no-GUI-lockstep, no-CI premise, K-of-N exclusion, SemVer (0.2.1→0.3.0 / 0.5.1→0.6.0 MINOR, `#[non_exhaustive]` absorbs the additive variant), entr byte-identity. None of these regressed; all four Criticals are completeness/specification defects in the encode/decode seam wiring + the two CLI/toolkit gate sites.
- **ms-cli pins ms-codec by `version = "=0.2.1"`** (`crates/ms-cli/Cargo.toml:20`, exact-version path dep) — plan Task 2.5 Step 2 "if ms-cli pins ms-codec by version, bump that dep too" is **REQUIRED** (not conditional): the `=0.2.1` must become `=0.3.0` or the workspace won't build after the ms-codec bump. Make it imperative.
- **FOLLOWUP `ms-codec-no-ci-workflow`** — verified NOT already in `design/FOLLOWUPS.md`; valid to file. (The mlock Cycle-B entry at FOLLOWUPS :83 added an ms-*cli* workflow only.)
- **Re-dispatch after fold** (per CLAUDE.md "reviewer-loop continues after every fold"): the C1/C2 seam decision is the highest-drift fold — verify the chosen discriminate/package signatures compile against the 4 in-module envelope tests + the single decode/encode call sites before claiming GREEN.
