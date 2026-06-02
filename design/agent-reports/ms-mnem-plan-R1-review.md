# ms-mnem PLAN R1 review

**Plan (folded):** `design/IMPLEMENTATION_PLAN_ms_mnem_wordlist_language.md`
**R0 review:** `design/agent-reports/ms-mnem-plan-R0-review.md` (RED 4C/4I)
**SPEC:** `design/SPEC_ms_mnem_wordlist_language.md` (R0 GREEN)
**Base SHA verified:** `master` `4e5266ab86b7149712a601f613e2435f28baa98c` (HEAD) — matches plan/spec.
**Reviewer:** opus architect, R1 of the mandatory R0 gate (no code until 0C/0I).
**Sources re-verified live:** ms-codec `src/{envelope,decode,encode,payload,consts,error,inspect,lib}.rs`; ms-cli `src/{language,format}.rs` + `src/cmd/{decode,inspect}.rs` + `Cargo.toml`; ms-codec `Cargo.toml`; `design/SPEC_ms_v0_1.md` §2.1/§6.3; `design/FOLLOWUPS.md`; `Cargo.lock` (bip39 2.2.2); toolkit `crates/mnemonic-toolkit/{Cargo.toml,src/language.rs}` + every `ms_codec::decode`/`decode_with_correction`/`Payload` consume-site; `scripts/install.sh`; `.github/workflows/manual.yml`.

---

## Verdict: RED (0C / 2I)

The four R0 Criticals are **all RESOLVED** against live source — the package/discriminate seam signatures, the `analyze()` gate naming, the length-gate enumeration, and the toolkit consume-site enumeration are now coherent and implementable. **However the fold introduced / left open two Important-class gaps**, both at the highest-drift seam the gate was told to scrutinize:

- **I-NEW-1 (toolkit wire-language→`bip39::Language` mapping unspecified, AND the toolkit's own `CliLanguage` discriminant order DIVERGES from the wire `MNEM_LANGUAGE_NAMES` order).** The C4 footgun-fix at `seed_intake.rs:166` says "use the wire language" but gives the executor no mechanism to turn `Payload::Mnem.language: u8` into a `bip39::Language` — and the obvious-looking `CliLanguage as u8` is *wrong* because the toolkit enum is ordered differently from the wire code. A naive implementation silently re-derives the wrong language — re-opening the very §6.3 footgun this cycle exists to close.
- **I-NEW-2 (`analyze()` rule-10 site `:87` left out of the C3 fold).** For a mnem string the wire tag is `entr`, so `analyze()`'s rule-10 check at `cmd/inspect.rs:87` fires `payload-length-mismatch` (payload_bytes carries the lang byte → len 17/21/25/29/33 ∉ VALID_ENTR_LENGTHS) → `would_decode=false`. Task 2.4 names `:80`/`:84` but not `:87`, so the Step-1 failing test ("does NOT print FAIL") would not go green after the named edits.

Neither is a design error; both are the same *seam-completeness* defect-class R0 was chartered to catch, surfacing one layer deeper after the fold. Per the CLAUDE.md gate, RED until folded + re-dispatched.

---

## R0 fold resolution

### C1 — `package(&Payload)` seam — RESOLVED ✓
Live `envelope.rs:147` = `pub(crate) fn package(tag: Tag, payload_bytes: &[u8]) -> Result<Codex32String>`; sole caller `encode.rs:26` = `envelope::package(tag, payload.as_bytes())` (✓ exactly as cited). `package` pushes `RESERVED_PREFIX` (0x00) at `:156` then the bare payload bytes. Plan Task 1.3 Step 3 now commits to **option (a)**: change signature to `package(tag: Tag, payload: &Payload)`, match the kind to assemble `[RESERVED_PREFIX]++e` (Entr — **byte-identical**, the `0x00` push moves from a literal into the Entr arm) / `[MNEM_PREFIX,*language]++entropy` (Mnem), then `from_seed("ms",0,"entr",Fe::S,&data)`; update caller `encode.rs:26` → `package(tag, payload)`. **Entr arm stays byte-identical** ✓ (still `[0x00]++entropy` → same `from_seed` args). In-module `package` tests correctly identified: there are exactly **2** (`envelope.rs:252`, `:264`, both `package(Tag::ENTR, &entropy)`); plan says "Update the in-module envelope tests that call `package`" (count-agnostic, correct — R0's "4" was a slip; only 2 exist). Coherent + compilable.

### C2 — `discriminate -> (Tag, Payload)` — RESOLVED ✓
Live `envelope.rs:91` = `discriminate(c) -> Result<(Tag, Vec<u8>)>`; strips prefix, returns `payload_with_prefix[1..].to_vec()` at `:137`; reserved-prefix reject `data[0] != RESERVED_PREFIX` at `:131`. Sole caller `decode.rs:41` `let (tag, payload_bytes) = discriminate(&c)?;`; `decode()` then builds `Payload::Entr` + validates at `:56-69`. Plan Step 4 now states the new signature explicitly: `discriminate(c) -> Result<(Tag, Payload)>`, dispatch on `data[0]` (0x00→`Payload::Entr(rest)`, 0x02→`Payload::Mnem{language:rest[0], entropy:rest[1..]}`.validate(), else reserved-prefix error). Step 5 makes `decode()` consume the typed Payload (no longer self-constructs). The split `0x02 → Mnem{language:rest[0], entropy:rest[1..]}` is correct against the byte-aligned layout (`data[0]`=prefix already consumed by discriminate's read, so `rest`=`data[1..]`; `rest[0]`=lang, `rest[1..]`=entropy) — confirmed against SPEC `:30` (`data[0]`=prefix, `data[1]`=lang, `data[2..]`=entropy) ✓. The 4 in-module `tests_discriminate` tests (`:206/:215/:224/:235`) are named for update; of these, the 3 tuple-destructuring ones (`:210/:219` + `tests_package` `:253`) need `recovered:Vec<u8>` → a `Payload` comparison, while the 2 `matches!(…, Err(…))` reject tests (`:230/:240`) are unaffected by the Ok-type change — the plan's "4 … that destructure" slightly over-scopes but is harmless (it updates the module). `.validate()` placement resolved: runs in `discriminate` for Mnem (plan Step 4) — note the entr `validate()` still also runs in `decode` at the rule-10 bind (see C2/I1 below). Coherent.

### C2/I1 — length-gate sites — RESOLVED ✓
Plan Step 5 now enumerates **all four** live sites, each verified:
- `decode.rs:29` accept-gate `!VALID_STR_LENGTHS.contains(&s.len())` ✓
- `decode.rs:32` `allowed: VALID_STR_LENGTHS` (the reporting set) ✓
- `decode.rs:135-137` `parse_ms1_symbols` stand-in (`allowed: VALID_STR_LENGTHS`) ✓ — correctly flagged as a defensive invalid-CHAR stand-in (it never length-gates a real mnem; a 51-char mnem flows through char-by-char), "do NOT just fix it wrongly"
- `envelope.rs:68-70` `extract_wire_fields` too-short defensive error (`allowed: VALID_STR_LENGTHS`) ✓
Step 5 mandates: accept the **union** `VALID_STR_LENGTHS ∪ VALID_MNEM_STR_LENGTHS` at the accept-gate, then bind length↔kind from the discriminated `Payload` (Entr⟺{50,56,62,69,75}, Mnem⟺{51,58,64,70,77}), and **fix the `allowed:` reporting set** so the error message isn't a lie (R0 I1's second half). Disjoint length sets + prefix-byte dispatch = no cross-honoring (defense-in-depth) ✓. The gate-vs-bind ordering is coherent now that C2 surfaces the kind to the bind point.

### I2 — `decode_with_correction` inherits the union gate — RESOLVED ✓
Live `decode_with_correction` (`decode.rs:188-246`) calls `decode(s)` at `:201` (clean codeword) and `decode(&corrected_str)` at `:244` (post-correction) — both hit the `:29` union gate, so the gate is inherited with **no separate change**. Plan Step 5b states this and adds the mnem round-trip-through-correction test (≤4-symbol corruption → corrects → `Payload::Mnem`). Coherent — and `parse_ms1_symbols` at `:192` runs before the gate but is char-based, so a clean 51-char mnem reaches `decode(s)` at `:201` fine.

### C3 — ms-cli inspect gate — RESOLVED (with residual I-NEW-2) ◑
Live: `analyze(report, str_len) -> (bool, Vec<&'static str>)` at `cmd/inspect.rs:46-92`; `would_decode = reasons.is_empty()` at `:91`; pushes `non-zero-prefix` `:80` (guard `:79`), `unexpected-string-length` `:84` (guard `:83`); reason-text literal `[50,56,62,69,75]` at `:96`, `non-zero-prefix` text at `:102`. `ms_codec::InspectReport` is in **`ms-codec/src/inspect.rs:13-28`** (NOT format.rs — there is no `ms-codec/src/format.rs`); it has `prefix_byte` + `payload_bytes` but **no `kind`/`language`**; `ms_codec::inspect()` surfaces `prefix_byte` raw, classifies nothing. CLI emit structs `InspectReportJson` `format.rs:73` + `InspectJson` `format.rs:85`.
Plan Task 2.4 Step 3 now **names `analyze()`** + its `(would_decode, reasons)` contract, locates `InspectReport` correctly in ms-codec `inspect.rs` (not format.rs), decides the data source (extend ms-codec `inspect()` to classify kind + language, surfaced via new `kind`/`language: Option<u8>` fields), and fixes the `:80`/`:84` pushes + `:96` reason-text. **Resolved for the prefix/length pushes.** ✗ **but it omits the rule-10 site `:87`** — see I-NEW-2.

### C4 — toolkit consume-sites — RESOLVED (with residual I-NEW-1) ◑
Re-grep `grep -rn 'ms_codec::decode\|Payload::Entr\|decode_with_correction\|ms_codec::Payload' crates/mnemonic-toolkit/src/` confirms the consume-sites. **Decode sites that destructure a Payload (the ones that matter):**
- `cmd/xpub_search/seed_intake.rs:166` — `Ok((_tag, payload))` → `payload.as_bytes().to_vec()` → `Mnemonic::from_entropy_in(args.language().into(), …)` at `:172`. **THE FOOTGUN — confirmed exactly:** derives under the CLI `--language`, not the wire language. ✓
- `cmd/bundle.rs:1613-1625` — `Payload::Entr(bytes)` / `_ =>` BadInput "payload is not entropy" (REJECT) ✓
- `cmd/convert.rs:1446-1458` — `Payload::Entr(bytes)` / `_ =>` BadInput "non-Entr payload" (REJECT) ✓
- `cmd/silent_payment.rs:135-143` — `Payload::Entr(b)` / `_ =>` SilentPayment error (REJECT) ✓
- `wallet_import/overlay.rs:127-139` — `Ok((_tag, Payload::Entr(bytes)))` / `Ok(_) =>` BadInput (REJECT) ✓
- `cmd/inspect.rs:160-176` — `InspectPayload::Ms1{tag, payload}` (`:171` decode) → `emit_inspect_text` `:179` needs a Mnem arm ✓
**Decode sites that DISCARD the payload (Mnem-agnostic, SAFE):** `verify_bundle.rs:1227` (`Ok(_)`), `:1620` (`Ok(_)`), `:2400` (`Ok((_tag, Payload::Entr(bytes)))` — actually destructures, but only for the entropy-match path; a mnem falls to its `Err`/`continue` skip), `repair.rs:820` + `:892` (`decode_with_correction`, discard `_p`). The plan Step 3 mandates the grep + per-site `Payload::Mnem` arm + the wire-language fix at seed_intake + a toolkit integration test (mnem card → derive under WIRE language). **Resolved as enumeration**, and the plan defers per-site triage to execution-time via the grep (acceptable — it requires the grep + per-site decision). ✗ **but the wire-language→bip39::Language mapping is unspecified and the toolkit enum order diverges** — see I-NEW-1.
(Note: R0's C4 named ~6-7 sites; the live grep surfaces 2 extra `verify_bundle` decode sites `:1227`/`:1620` + a 2nd `decode_with_correction` `:892` R0 didn't list — all SAFE (payload discarded), so the omission doesn't change the outcome, but the plan's reliance on the grep (not a frozen count) is what makes it robust.)

### I3 — toolkit pin versions — RESOLVED ✓ (one stray citation error → see new-drift N1)
Live pins verified: `scripts/install.sh:38` = `ms-cli|…|ms-cli-v0.5.0|yes|`; `.github/workflows/manual.yml:88` = `cargo install … --tag ms-cli-v0.5.0 ms-cli`. **Current pin is `ms-cli-v0.5.0`, NOT v0.5.1** ✓. Plan Step 2 now says `ms-cli-v0.5.0 → ms-cli-v0.6.0` at both sites (with the `ms-cli-` prefix) ✓. ms-codec version `0.2.1`→`0.3.0` ✓; ms-cli version `0.5.1`→`0.6.0` ✓; ms-cli's `=0.2.1` exact dep on ms-codec (`crates/ms-cli/Cargo.toml:20` `ms-codec = { path="../ms-codec", version="=0.2.1" }`)→`=0.3.0` REQUIRED (plan Task 2.5 Step 2 — imperative). **Caveat (N1):** plan Step 2 mislabels the *toolkit* ms-codec lib pin as `version = "=0.2.1"` (exact) — it is actually `ms-codec = "0.2.1"` (caret, `crates/mnemonic-toolkit/Cargo.toml:20`). The `=0.2.1` exact pin is ms-cli's, a different file. Cosmetic (the bump target `→ 0.3.0` is correct either way), but the citation is wrong.

### I4 — non-English test fixture — RESOLVED ✓ (residual Minor on exact-length pin)
No checksum-valid non-English fixture exists in `crates/ms-cli/tests` or `src`. Plan Task 2.2 Step 1 now generates it in-test: `bip39::Mnemonic::from_entropy_in(bip39::Language::Japanese, &[0xABu8;16]).unwrap().to_string()`. `from_entropy_in` exists in the pinned `bip39 = 2.2.2` (already used live at `cmd/decode.rs:62`); ms-cli pins `bip39 = { version="2", features=["all-languages"] }` (`Cargo.toml:21`) so the Japanese wordlist is available in tests ✓ (I7 satisfied). **Residual Minor:** R0 I4 also asked to pin the exact mnem length 51 for the 16-byte fixture; the plan still asserts `length ∈ {51,58,64,70,77}` (line 98). The test runs correctly either way — non-blocking, but tightening to `== 51` is the stronger gate R0 requested.

---

## Seam-contract coherence (package / discriminate vs callers + in-module tests)

| Element | Live | Plan's change | Coherent? |
|---|---|---|---|
| `package` sig | `(tag: Tag, payload_bytes: &[u8])` `:147` | `(tag: Tag, payload: &Payload)` | ✓ |
| `package` caller | `encode.rs:26` `package(tag, payload.as_bytes())` | `package(tag, payload)` | ✓ |
| `package` Entr byte-identity | `[0x00]++entropy` via `:156` push | `[RESERVED_PREFIX]++e` in Entr arm | ✓ identical |
| `package` Mnem | n/a | `[MNEM_PREFIX,*language]++entropy` | ✓ |
| `package` in-module tests | 2 (`:252`,`:264`) | "the tests that call package" | ✓ (R0's "4" was a slip) |
| `discriminate` sig | `(c) -> Result<(Tag, Vec<u8>)>` `:91` | `(c) -> Result<(Tag, Payload)>` | ✓ |
| `discriminate` caller | `decode.rs:41`; decode builds Payload `:56-69` | decode consumes typed Payload | ✓ |
| `discriminate` `0x02` split | n/a | `Mnem{language:rest[0], entropy:rest[1..]}`.validate() | ✓ (matches SPEC byte layout) |
| `discriminate` in-module tests | `tests_discriminate` 4 (`:206/:215/:224/:235`) | "the 4 in-module tests" | ✓ (3 destructure, 2 matches!-only — all in module) |
| `decode` rule-9 + bind | `:29`/`:32` + tag-dispatch `:56-69` | union accept + bind-to-kind + fix `allowed:` | ✓ |

All seam signatures compile against the named callers + tests. The C1/C2 highest-drift fold is internally consistent.

---

## New-drift scan

- **N1 (Minor, folds into I3):** plan Phase 3 Step 2 (line 133) says the toolkit ms-codec lib pin is "`version = "=0.2.1"` — exact pin". Live `crates/mnemonic-toolkit/Cargo.toml:20` = `ms-codec = "0.2.1"` (caret, **not** `=`). The `=0.2.1` exact pin belongs to **ms-cli**'s Cargo.toml (`:20`). Bump target `→ 0.3.0` is correct; only the "exact pin" characterization of the toolkit file is wrong. Fix the parenthetical.
- **N2 (note, not a defect):** the prefix-byte design keeps the wire `id` = `"entr"` for BOTH kinds (SPEC `:29`), so removing `*b"mnem"` from `RESERVED_NOT_EMITTED_V01` (plan Task 1.1 Step 1, SPEC-mandated at `:31`/`:74`) is **benign**: a literal `mnem`-tagged string is never emitted, and if one were decoded it would fall through `decode.rs:44` (rule-7) to the `decode.rs:64` `_ =>` `UnknownTag` arm (rule 6). No correctness impact; flagged only so the implementer doesn't expect the removal to change dispatch. **Also:** removing `mnem` from `RESERVED_NOT_EMITTED_V01` changes ms-cli `analyze()`'s rule-6/7 classification at `cmd/inspect.rs:66-77` for a literal `mnem`-tagged inspect input (would now report `unknown-tag` instead of `reserved-tag-not-emitted`) — purely a diagnostic-text change on an input no one emits; harmless, but a test asserting `reserved-tag-not-emitted` for `mnem` (none found in `tests/`) would need updating if one is added.
- **N3 (note):** SPEC `:67`/`:69` still say "ms-cli 0.5.1 → 0.6.0" / "tag v0.5.1→v0.6.0" — that's the SPEC's pre-fold claim; the plan I3 fold correctly supersedes it with the live `v0.5.0` tag pin. No action (SPEC is GREEN; the plan is the live authority for impl).
- **No K-of-N leak:** no `0x01`, `Threshold`, `encode_shares`, or §5-amendment anywhere in the plan ✓.
- **SemVer:** ms-codec 0.2.1→0.3.0 (additive `Payload::Mnem` + new const + new error variant, all behind `#[non_exhaustive]`) MINOR ✓; ms-cli 0.5.1→0.6.0 MINOR ✓. No GUI `schema_mirror` change (no new flag/subcommand/dropdown — `ms encode` reuses `--language`; verified against the no-new-flag SPEC `:44`) ✓.
- **Spec coverage:** §2/§3/§4/§5/§6/§7/§8/§9/§10 all mapped; the two open Importants sit inside §5 (inspect gate) and §8 (toolkit lockstep).

---

## Required folds to reach GREEN

1. **I-NEW-1 (toolkit wire-language mapping).** In Phase 3 Step 3, specify how the toolkit turns `Payload::Mnem.language: u8` into a `bip39::Language` at `seed_intake.rs` (and any other site that surfaces language). The toolkit's own `CliLanguage` (`crates/mnemonic-toolkit/src/language.rs:10-22`) is ordered `English, SimplifiedChinese, TraditionalChinese, Czech, French, Italian, Japanese, Korean, Portuguese, Spanish` — a DIFFERENT order from the wire code (`MNEM_LANGUAGE_NAMES` = `english, japanese, korean, spanish, chinese-simplified, chinese-traditional, french, italian, czech, portuguese`). So `CliLanguage as u8` is WRONG and would silently mis-derive. State the bridge: either (a) ms-codec exposes a `pub fn language_name_from_code(u8) -> Option<&'static str>` (or expose `MNEM_LANGUAGE_NAMES` publicly — the plan already adds it to `consts.rs`) and the toolkit maps name→`bip39::Language`, or (b) ms-codec exposes a `code → bip39::Language` directly. Pick one and write it; this is the cycle's own footgun and must not be left to a `as u8` guess.
2. **I-NEW-2 (`analyze()` rule-10 site).** Add `cmd/inspect.rs:87` to Task 2.4: for a mnem string the wire tag is `entr`, so the rule-10 guard `tag_bytes == TAG_ENTR && !VALID_ENTR_LENGTHS.contains(&report.payload_bytes.len())` fires `payload-length-mismatch` (payload_bytes = `[lang]++entropy`, len 17/21/25/29/33). The mnem arm must skip rule-10 for `prefix_byte==0x02` (or validate the mnem entropy length against `VALID_ENTR_LENGTHS` after stripping the lang byte). Without this, Task 2.4 Step 1's failing test does not go green.

Recommended (non-blocking): fix N1's "exact pin" parenthetical; tighten I4's `∈{51,…}` → `== 51` for the 16-byte Japanese fixture.

---

## Notes
- All four R0 Criticals RESOLVED with live-source evidence; the seam-signature fold (C1/C2 — the flagged highest-drift item) is coherent against callers + the 2 package / 4 discriminate in-module tests.
- Both new Importants are the same seam-completeness defect-class one layer down (a missed gate-site inside `analyze()`; a missing cross-repo type bridge) — mechanical to fold, no design change.
- Per CLAUDE.md "reviewer-loop continues after every fold": fold I-NEW-1 + I-NEW-2, persist, re-dispatch R2. Do not start Phase 0 until GREEN.
- `from_entropy_in` / `bip39 2.2.2` / `features=["all-languages"]` all confirmed present — the in-test Japanese fixture is constructible.
