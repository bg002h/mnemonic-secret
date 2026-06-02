# ms-mnem PLAN R2 review

**Plan (folded):** `design/IMPLEMENTATION_PLAN_ms_mnem_wordlist_language.md`
**R1 review:** `design/agent-reports/ms-mnem-plan-R1-review.md` (RED 0C/2I + 1 minor)
**R0 review:** `design/agent-reports/ms-mnem-plan-R0-review.md` (RED 4C/4I)
**SPEC:** `design/SPEC_ms_mnem_wordlist_language.md` (R0 GREEN)
**Base SHA:** `master` `4e5266a` (per plan).
**Reviewer:** opus architect, R2 of the mandatory R0 gate (no code until 0C/0I).
**Scope of R2:** verify the three R1 folds (I-NEW-1, I-NEW-2, N1) against LIVE source + scan for fold-introduced drift. R0's 4 Criticals + R1's seam-coherence (package/discriminate signatures) were verified RESOLVED in R1 and are NOT re-litigated here.
**Sources re-verified live:** ms-cli `src/language.rs` (CliLanguage order + as_str), `src/cmd/inspect.rs:46-106` (full `analyze()` + reason_text), `src/Cargo.toml`; ms-codec `src/inspect.rs:1-60` (InspectReport + inspect()), `src/consts.rs:39` (RESERVED_NOT_EMITTED_V01), `src/lib.rs:40-57` (pub re-exports / `pub mod consts`), `Cargo.toml`; ms `design/SPEC_ms_v0_1.md` §6.3 (:248) + the new SPEC §2 (:14-44); toolkit `crates/mnemonic-toolkit/src/language.rs` (CliLanguage order + `From<CliLanguage> for bip39::Language`), `Cargo.toml:20`, `scripts/install.sh:38`, `.github/workflows/manual.yml:88`, full `ms_codec::decode`/`Payload`/`decode_with_correction` consume-site grep, `seed_intake.rs:155-180`.

---

## Verdict: GREEN (0C/0I)

All three R1 folds are RESOLVED against live source. No fold-introduced drift; no remaining valid-mnem-rejecting site is left unnamed; spec coverage complete; SemVer intact; no K-of-N leak. The plan is implementable. Per the CLAUDE.md hard gate, this clears the R0 plan gate — implementation (Phase 0 spike) may begin.

---

## R1 fold resolution (I-NEW-1, I-NEW-2, N1)

### I-NEW-1 (footgun-class: toolkit wire-code→language mapping) — RESOLVED ✓

**(a) CliLanguage order divergence — PROVEN against live source.** The toolkit's `CliLanguage` order differs from the wire table. The plan's mandate to map via the canonical `ms_codec::MNEM_LANGUAGE_NAMES` table (code→name→`bip39::Language`) instead of `toolkit::CliLanguage as u8` is therefore load-bearing and correct. Quoted proof is in the next section.

**(b) Does the mandated mapping prevent the footgun? YES.** Plan Phase 3 Step 3 (`:135`) now states: "the wire code MUST map through the **canonical** ms-codec table: `code → ms_codec::MNEM_LANGUAGE_NAMES[code] → bip39::Language` (an explicit match keyed on the canonical code/name, NOT any local enum discriminant)" and mandates "Add a test asserting the toolkit's code→language mapping equals the canonical table for all 10 codes." This is mechanically sound:
- The wire code is the index into `MNEM_LANGUAGE_NAMES` (SPEC §3 `:27`: "low-nibble code → BIP-39 language, in `CliLanguage` declaration order (`ms-cli/src/language.rs:13-22`), English = 0"; new-SPEC table `:27`: `0 en · 1 ja · 2 ko · 3 es · 4 zh-Hans · 5 zh-Hant · 6 fr · 7 it · 8 cs · 9 pt`).
- `MNEM_LANGUAGE_NAMES` is being ADDED to `consts.rs` by Task 1.1 Step 1 (`:58`) and `ms-codec/src/lib.rs:42` already has `pub mod consts`, so the toolkit can reach `ms_codec::consts::MNEM_LANGUAGE_NAMES` with no extra re-export — confirmed live.
- An explicit `code → name → bip39::Language` match cannot accidentally route through the divergent local discriminant. The all-10-codes test pins it against drift.

**(c) Is `ms_codec::MNEM_LANGUAGE_NAMES` the right single source of truth, and is ms-cli's `CliLanguage::from_code` safe? YES on both.**
- It is the canonical wire invariant (SPEC §3 `:27`: "the wire invariant lives in the lib"), so both consumers (ms-cli and the toolkit) keying off it is correct.
- ms-cli's `CliLanguage::from_code` (Task 2.1, `:95`) is safe **by construction**: ms-cli's `CliLanguage` declaration order IS the wire order (proven below — `English, Japanese, Korean, Spanish, ChineseSimplified, ChineseTraditional, French, Italian, Czech, Portuguese` = `MNEM_LANGUAGE_NAMES`), so `code()` = discriminant and `from_code` round-trips. Task 2.1 ALSO asserts "`code()` order matches `ms_codec::MNEM_LANGUAGE_NAMES`" (`:95`) — a regression pin against future reordering of ms-cli's enum. The asymmetry is correctly handled: ms-cli relies on construction-order-equals-wire-order (asserted), the toolkit must NOT (its order diverges) and is forced through the canonical table. No contradiction between Task 2.1's discriminant-cast approach and Phase 3 Step 3's table approach — they apply to two enums with two different orders, each handled correctly.

### I-NEW-2 (inspect `analyze()` rule-10 `:87`) — RESOLVED ✓

Live `cmd/inspect.rs:87` IS the `payload-length-mismatch` rule: `if tag_bytes == TAG_ENTR && !VALID_ENTR_LENGTHS.contains(&report.payload_bytes.len()) { reasons.push("payload-length-mismatch"); }`. For a valid mnem string this WOULD fire: the wire tag is `entr` (SPEC §3 `:29`: id stays `"entr"`; only the prefix byte distinguishes mnem), so `tag_bytes == TAG_ENTR` is TRUE; and `report.payload_bytes` is the bytes-after-prefix (ms-codec `inspect.rs`: `payload_bytes = payload_with_prefix[1..]`), i.e. `[lang]++entropy` = 17/21/25/29/33 bytes, none of which are in `VALID_ENTR_LENGTHS = {16,20,24,28,32}` → pushes `payload-length-mismatch` → `would_decode=false`. Confirmed: without folding this, Task 2.4 Step 1's failing test ("does NOT print FAIL") could not go green.

The plan now folds it: Task 2.4 Step 3 (`:114`) explicitly names "`+ :87` (`payload-length-mismatch`, R1 I-NEW-2) — the rule-10 site ALSO fires for mnem (the wire tag is `entr` but the payload is `[lang][entropy]` = 17/21/25/29/33 bytes ∉ `VALID_ENTR_LENGTHS`); make the payload-length check **kind-aware** (mnem payload length = entropy+1 for the lang byte)." The "entropy+1" arithmetic is correct (1 lang byte). The same sentence also directs the reason-text (`:103`) to stop claiming the mnem payload is invalid. Resolved.

### N1 (toolkit ms-codec pin is caret, not exact) — RESOLVED ✓

Live `crates/mnemonic-toolkit/Cargo.toml:20` = `ms-codec = "0.2.1"` — caret (no `=`). Live `crates/ms-cli/Cargo.toml:20` = `ms-codec = { path = "../ms-codec", version = "=0.2.1" }` — exact, a DIFFERENT file. The plan now states both correctly:
- Phase 3 Step 2 (`:133`): "bump the toolkit's ms-codec **lib** pin — caret `version = "0.2.1"` (NOT exact; R1 N1) → `"0.3.0"` ... (The exact `=0.2.1` pin is ms-cli's own ms-codec dep, bumped in Phase 2 Step 2 — a different file.)" ✓
- Task 2.5 Step 2 (`:120`): "if ms-cli pins ms-codec by version, bump that dep too" — and the R1 review already verified ms-cli's `=0.2.1` → `=0.3.0` is REQUIRED; the imperative covers it. ✓

The stale "exact pin" parenthetical R1 flagged is gone. Both characterizations now match live source.

---

## CliLanguage order divergence (toolkit vs wire — quoted proof)

**Wire table = ms-cli `CliLanguage` declaration order = `MNEM_LANGUAGE_NAMES`** (live `ms-cli/src/language.rs:12-23`):
```
English, Japanese, Korean, Spanish, ChineseSimplified, ChineseTraditional, French, Italian, Czech, Portuguese
  0        1        2       3          4                  5                   6       7       8      9
```
Cross-checked against new-SPEC §3 `:27` table: `0 en · 1 ja · 2 ko · 3 es · 4 zh-Hans · 5 zh-Hant · 6 fr · 7 it · 8 cs · 9 pt` — IDENTICAL. And the plan's `MNEM_LANGUAGE_NAMES` literal (Task 1.1 `:58`): `["english","japanese","korean","spanish","chinese-simplified","chinese-traditional","french","italian","czech","portuguese"]` — IDENTICAL.

**Toolkit `CliLanguage` declaration order** (live `mnemonic-toolkit/src/language.rs:10-22`):
```
English, SimplifiedChinese, TraditionalChinese, Czech, French, Italian, Japanese, Korean, Portuguese, Spanish
  0          1                 2                  3      4       5        6         7       8           9
```

**Divergence is real and dangerous.** Same code, different language under the two orderings:
- code `1`: wire = **Japanese**, toolkit-discriminant = SimplifiedChinese
- code `2`: wire = **Korean**, toolkit-discriminant = TraditionalChinese
- code `3`: wire = **Spanish**, toolkit-discriminant = Czech
- code `6`: wire = **French**, toolkit-discriminant = Japanese
- … only code `0` (English) coincides.

So `toolkit::CliLanguage as u8` (or `from_repr`/discriminant indexing) applied to a wire code would silently mis-derive — re-opening the exact §6.3 footgun this cycle exists to close (a Japanese card decoded under SimplifiedChinese → different PBKDF2 string → different master seed → empty wallet). The plan's canonical-table mandate (I-NEW-1 fold) is the correct and only-safe bridge. Note the toolkit DOES already have `impl From<CliLanguage> for bip39::Language` (`language.rs:42-55`) — but that is keyed on the toolkit's own VARIANTS, so it is only safe when reached via a `CliLanguage` value, never via a raw wire `u8`. The plan correctly routes wire `u8 → name → bip39::Language`, bypassing the toolkit enum entirely.

---

## analyze() full rule scan (any other mnem-rejecting rule?)

Re-read all of `analyze()` (`cmd/inspect.rs:46-92`) rule-by-rule, tracing a VALID mnem string (wire tag `entr`, prefix `0x02`, str_len 51, payload `[lang][16B entropy]` = 17 bytes, threshold 0, share-index 's', hrp "ms"):

| Rule | Site | Condition | Fires on valid mnem? | Named in plan? |
|---|---|---|---|---|
| 2 (hrp) | `:51` | `hrp != "ms"` | NO (hrp="ms") | n/a |
| 3 (threshold) | `:55` | `threshold != 0` | NO (=0) | n/a |
| 4 (share-index) | `:59` | `share_index != 's'` | NO (='s') | n/a |
| 6/7 (tag) | `:66-77` | `tag_bytes != TAG_ENTR` | NO (tag IS `entr`) | n/a |
| 8 (prefix) | `:79` | `prefix_byte != 0x00` | **YES** (0x02) | ✓ `:80` |
| 9 (str-len) | `:83` | `str_len ∉ VALID_STR_LENGTHS` | **YES** (51 ∉ {50,56,62,69,75}) | ✓ `:84` |
| 10 (payload-len) | `:87` | `tag==entr && payload_len ∉ VALID_ENTR_LENGTHS` | **YES** (17 ∉ {16,20,24,28,32}) | ✓ `:87` (I-NEW-2 fold) |

**Exactly three rules (8, 9, 10) fire for a valid mnem, and all three are now named in the plan.** No fourth rule rejects it. (Note the code numbers rules 2–10; there is no "rule 1" / "rule 5" branch in `analyze()` — rule 1 = the BIP-93 parse handled earlier by `ms_codec::inspect()` at `run():31`, rule 5 is the prefix-existence covered by rule 8. A clean mnem parses fine at the BIP-93 layer because it is a real codex32 string — only the v0.1 semantic rules reject it, and those are the three above.) The plan's data-source change is also coherent: ms-codec `inspect()` will classify `kind` + `language` from `prefix_byte` (Task 2.4 `:113`), surfaced via new `InspectReport.kind`/`language: Option<u8>` fields — `InspectReport` is `#[non_exhaustive]` (live `inspect.rs:12`) so adding fields is non-breaking. The reason_text at `:96` (`[50,56,62,69,75]`) and `:103` (`entr payload length ...`) are correctly flagged for kind-aware update so the diagnostic stops lying about a valid mnem.

---

## New-drift scan

- **No fold-introduced contradiction.** Task 2.1 (ms-cli `code()` = discriminant) and Phase 3 Step 3 (toolkit maps via canonical table, NOT discriminant) are consistent — they govern two differently-ordered enums; ms-cli's order matches the wire by construction (asserted), the toolkit's does not (forbidden from discriminant-cast). The §6.3 fix is consistent across artifacts: Phase 3 Step 1 (`:132`) rewords `SPEC_ms_v0_1.md` §6.3 "4-bit-encoded" → "1-byte language field (low nibble used)"; live target confirmed at `SPEC_ms_v0_1.md:248` ("entropy + 4-bit-encoded wordlist-language hint") — the new SPEC §2 `:20` already records this correction directive.
- **No remaining valid-mnem-rejecting site left unnamed.** Toolkit grep (`ms_codec::decode|Payload::Entr|decode_with_correction|ms_codec::Payload`) re-run live: every decode/destructure site matches the R1 enumeration — the footgun `seed_intake.rs:166`; the reject-with-`_=>` sites `bundle.rs:1619`, `convert.rs:1453`, `silent_payment.rs:137`, `overlay.rs:128`; the inspect surface `inspect.rs:171`; the payload-discarding-SAFE sites `verify_bundle.rs:1227/1620/2400` + `repair.rs:820/892` (`decode_with_correction`, `_p` discarded). The `error.rs:356`/`friendly.rs:74` hits are `PayloadLengthMismatch` error-formatting, not decode sites — unaffected. Plan Step 3 mandates the grep + per-site `Payload::Mnem` arm + the wire-language fix at `seed_intake` + the canonical-table mapping + an integration test. Codec/CLI sides: the three v0.1-hardcoded gates (decode length-gate union, `decode::run` Mnem arm, `inspect::analyze` rules 8/9/10) are all named. No unhandled site remains.
- **N2 (benign, carried from R1, re-confirmed):** removing `*b"mnem"` from `RESERVED_NOT_EMITTED_V01` (live `consts.rs:39` — present) is benign per R1; a literal `mnem`-tagged inspect input would shift from `reserved-tag-not-emitted` to `unknown-tag` diagnostic text. No test in `tests/` asserts that string; no correctness impact. The plan's removal (Task 1.1 `:58`) is SPEC-mandated and harmless.
- **No K-of-N leak:** no `0x01`, `Threshold`, `encode_shares`, share-grouping, or §5-amendment anywhere in the plan. SPEC §2 `:21` explicitly keeps `0x01` unallocated. ✓
- **SemVer intact:** ms-codec 0.2.1→0.3.0 (additive `Payload::Mnem` behind `#[non_exhaustive]` + new const + new `MnemUnknownLanguage` error variant) = MINOR ✓; ms-cli 0.5.1→0.6.0 (additive behavior, reuses `--language`, no new flag) = MINOR ✓; toolkit caret lib pin `0.2.1`→`0.3.0` + ms-cli tag `v0.5.0`→`v0.6.0` (live pins confirmed `install.sh:38`/`manual.yml:88`) + toolkit own version PATCH. No GUI `schema_mirror` change (no new flag/subcommand/dropdown) ✓.
- **Spec coverage still complete:** §2–§10 all mapped (plan Self-review `:142-144`); the two former-open Importants (inspect gate §5; toolkit lockstep §8) are now closed by the folds.

---

## Notes

- All three R1 folds (I-NEW-1 footgun-class wire-code mapping, I-NEW-2 rule-10 `:87`, N1 caret-pin) RESOLVED with live-source evidence. The CliLanguage divergence is empirically proven (two quoted orderings, 4 mismatched codes).
- `analyze()` full rule scan confirms EXACTLY rules 8/9/10 reject a valid mnem and ALL three are named — the C3/I-NEW-2 gate is now complete; no fourth rejecting rule exists.
- `ms_codec::consts` is `pub` and `MNEM_LANGUAGE_NAMES` is being added there, so the canonical table is reachable from both ms-cli and the toolkit with no extra re-export plumbing — the I-NEW-1 bridge is constructible as written.
- The R1-recommended (non-blocking) nits — tighten Task 2.2's `∈{51,…}` → `== 51` for the 16-byte fixture — remain optional polish, NOT gate-blocking; the test passes either way. Not raised as a finding.
- GREEN clears the mandatory R0 plan gate. Per CLAUDE.md "reviewer-loop continues after every fold," no further fold is required (0C/0I). Phase 0 spike may begin; persist per-phase reviews to `design/agent-reports/` and run the full local gate (test + clippy -D + `+stable fmt --check`) at every phase commit (ms-codec has no CI).
