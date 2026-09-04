# SPEC_ms_hashlock — R0 round 0, TESTS-AND-VECTORS lens

**Artifact:** `design/SPEC_ms_hashlock.md` at mnemonic-secret `5ba61ca` (tree cited: `7fc1e58`).
**Lens:** for each guarantee, name the mutation that breaks it, then say whether §8's rows and
§11's tests **as written** would catch it. A guarantee whose named mutation passes every listed
test is a finding.
**Method note:** the mutation table below was built from §1–§7 alone and written to scratch
**before** §8/§11 were read, so it is not anchored by the spec's own list. 63 mutations were
enumerated pre-read; the table adds the ones the spec's own text later suggested.

**Counts: 1 Critical / 11 Important / 11 Minor / 4 Nit.**

---

## 1. Mutation table

Row key: **§8** rows are named R*n*, **§11** tests T*n*, **§12** acceptance A*n*.
"NOT CAUGHT" means no listed row or test, read literally, can fail on that mutation.

### 1.1 Wire format / codec

| # | mutation | CAUGHT BY / NOT CAUGHT |
| --- | --- | --- |
| K1 | `PREIMAGE_PREFIX = 0x01` | **CAUGHT** — §8 "Encode/decode/inspect for `0x03`" pins the literal ms1 string. *But* A3's wording alone ("a 75-character `ms10hashsq…` string") does **not** catch it: `0x00`–`0x03` all encode a leading `q`, which §1 itself states. See N-1. |
| K2 | length check `!= 32` instead of `!= 33` | **CAUGHT** — §8's 32-byte length row is "refused by name"; under the mutation the legal 33-byte payload is refused and R1 fails too. |
| K3 | length check **after** `try_from`, or `data[1..33]` slice indexing | **CAUGHT** — §8's 16-byte row panics on the indexing form and yields a different error on the `try_from` form. This is exactly what the 16/32/34/46 rows are for; well specified. |
| K4 | `PreimageLengthMismatch { got }` reports the **payload** length (33/17/47) instead of the X length (32/16/46) | **NOT CAUGHT** — rows are "each refused by name", which pins the variant, not the field. The spec never defines what `got` counts. → **M-1** |
| K5 | reader dispatches on the 4-char **id** instead of the prefix byte | **NOT CAUGHT** — every listed row has id and prefix **agreeing**. No row exists where they disagree. → **I-1** |
| K6 | `--in` dispatches on **length** (75 chars) → an entr-32 single read as a preimage | **NOT CAUGHT as written** — T5 says "entr and mnem strings refused"; it does not say **entr-32**. An entr-16 (50 chars) test passes under the mutation. → **I-2** |
| K7 | `hash` added to the blocklist but singles still emitted with `entr` | **CAUGHT** — §8 "id `hash` on singles **and** its blocklist entry" names both halves. |
| K8 | singles emit `hash` but the blocklist is not updated | **CAUGHT** — same row. |
| K9 | split/combine loses the `0x03` prefix (recovers as entr) | **CAUGHT** — §8 "the share round trip through the codec API"; T14. |
| K10 | `ReservedPrefixViolation` still fires for `0x03` | **CAUGHT** — R1. |

### 1.2 Derivation

| # | mutation | CAUGHT BY / NOT CAUGHT |
| --- | --- | --- |
| D1 | salt one byte wrong (`ms-hashlock-v2`, trailing NUL, trailing `\n`) | **CAUGHT** by the anchor row's X — **unless** the reproduction test is constants-parameterised, in which case the cross-tool check moves with the mutation. → **I-9** |
| D2/D3/D4 | iterations 10,000 / 1,000,000 / 99,999 | **CAUGHT** by the anchor row's X (same caveat as D1). |
| D5 | the two methods swapped **inside ms-codec** | **CAUGHT** — §8 pins X and H for **both** methods; A1/A2 pin both digests. |
| D5b | the two methods swapped **in the CLI wiring** (flag → function) | **NOT CAUGHT by §8 or §11** — §8's rows are codec-level; §11 pins shapes, not values; the card's method line is rendered from the same flag so it stays self-consistent. Only A1/A2 (manual acceptance) hold it. → **I-… see I-7 note / M-6** and the remedy in §5 below. |
| D6 | `dkLen = 16` | **CAUGHT** — anchor row. |
| D7 | `dkLen = 64`, truncated to 32 | **NOT CAUGHT — and correctly so.** PBKDF2's first 32 bytes at dkLen 64 are byte-identical to dkLen 32, so the mutation is unobservable and benign. Listed to show the table is not padded. |
| D8 | `digest` computes `sha256(phrase)` instead of `sha256(X)` | **CAUGHT** — under `--method sha256` the mutation makes H == X (`c4bbcb1f…`), which the anchor row's H column contradicts. The signature `digest(&[u8; 32])` also blocks it at the type level for the 28-byte anchor phrase. Good design. |
| D9 | H emitted as X | **CAUGHT** — anchor row pins both columns (this is r2 N-1, correctly folded). |
| D10 | `preimage_sha256` = `sha256(phrase ‖ salt)` | **CAUGHT** — anchor row. |
| D11 | `Zeroizing` dropped from `Payload::Preimage` | **NOT CAUGHT** — a type-level property no listed test can fail. Secret-handling → **M-7** (Minor by the 2026-08-27 ruling). |

### 1.3 Phrase reader and phrase rule

| # | mutation | CAUGHT BY / NOT CAUGHT |
| --- | --- | --- |
| P1 | reader strips **two** trailing newlines | **CAUGHT** by T2 ("stdin stripping of **exactly one** LF or CRLF") *provided the test feeds a doubled newline*. Worth noting the shape: with printable-ASCII-only, `"abc\n\n"` correctly becomes `"abc\n"` → **refused**; the mutation makes it derive. The test's assertion is a refusal, not a different X. |
| P2 | reader trims a leading/trailing space | **CAUGHT** — §8's "phrase with leading, trailing and doubled spaces" row. |
| P3 | reader uses `parse::read_input` | **CAUGHT** — same row (read_input strips all whitespace). |
| P4 | reader uses `parse::read_phrase_input` | **CAUGHT** — same row (trims + collapses runs). |
| P4b | reader strips **`-` and `,` only** (e.g. reuses `format::strip_display_separators`, which the adjacent `is_ms1_shaped` calls) | **NOT CAUGHT** — no listed row carries a `-` or a `,` in the phrase. Measured: `format.rs:12` `is_display_separator` = whitespace ‖ `-` ‖ `,`. §4.3 names all three characters as the hazard; §8 covers one. → **I-4** |
| P5 | CRLF mishandled | **CAUGHT** — T2. |
| P6/P7 | `HASHLOCK_PHRASE_MAX_CHARS` = 99 / 101 | **CAUGHT** — §8's "100 and 101" rows. Well chosen. |
| P8/P10 | printable-ASCII predicate is `is_ascii()` → admits TAB (0x09), DEL (0x7F), control bytes | **NOT CAUGHT** — §8's only boundary row is "a non-ASCII refusal", which exercises the *non-ASCII* side. Consequence: an invisible-whitespace phrase derives, is unreproducible later → preimage loss. → **I-5** |
| P9 | predicate is `is_ascii_graphic()` → rejects SPACE | **CAUGHT** — the anchor phrase and the spaces row both contain spaces. The boundary is pinned in one direction only. |
| P11 | empty phrase accepted | **CAUGHT** — §8's "empty" row. |
| P12 | 64-hex refusal tests 63 (or 65) | **CAUGHT** — §8's "the 64-hex refusal" row fails. |
| P12b | 64-hex refusal fires on **any** all-hex phrase (over-refusal, e.g. `beef`) | **NOT CAUGHT** — no row asserts a short all-hex phrase is *accepted*; §8's "one character" row has unspecified content (if that character is a hex digit it would catch it, by luck). Fails closed. → **M-9** |
| P13 | 64-hex refusal is lowercase-only → 64 **uppercase** hex chars derive as a phrase | **NOT CAUGHT** — one refusal row, presumably lowercase. Measured: `--hex` uses `hex::FromHex` (`encode.rs:283`), which accepts uppercase, so the two predicates disagree — the exact drift §4.3 unified `is_ms1_shaped` to prevent. → **I-6** |
| P14 | `is_ms1_shaped` not called on the **stdin** channel | **NOT CAUGHT** — §8 lists no ms1-shaped refusal row; §11's "entr and mnem strings refused" reads as the `--in`/`<ms1>` source, not the phrase channels. → **I-3** |
| P15 | `is_ms1_shaped` called, but **without** `argv_candidates`' case folding — i.e. exactly what §4.3 prescribes | **NOT CAUGHT, and the defect is already live.** Measured below. → **C-1** |
| P16 | `HASHLOCK_PHRASE_MAX_CHARS` bound to the device's `passphrase.MaxLen` | **NOT CAUGHT today** (both are 100); breaks silently the day the device number moves. Census item → §5. |

### 1.4 CLI surface

| # | mutation | CAUGHT BY / NOT CAUGHT |
| --- | --- | --- |
| C1 | `--random` gates on `--out` **only** (rejects `--random --json`) | **NOT CAUGHT** — T8 and A5 test only the *refusal* when both are absent; nothing asserts `--random --json` **succeeds**. Over-refusal, fails closed. → **M-2** |
| C2/C3 | `--random` gate weakened or removed | **CAUGHT** — T8, A5. |
| C4 | `--out` suppresses stdout (copying `ms encode`) | **NOT CAUGHT as written** — T10 ("stdout is exactly the record line") does not say it runs **under `--out`**; T11 checks only mode and overwrite; A3 checks the file, not stdout. §4.4 states "**`--out` never suppresses it**" normatively (r2 I-5). → **I-7** |
| C5 | `--method` silently ignored with `--hex` / `--random` / `<ms1>` | **PARTIALLY CAUGHT** — T9 says "`--method` with **a** supplied preimage" (singular); three supplied sources exist. → **M-3** |
| C6 | method line announced only when `--method` was explicitly given | **CAUGHT in practice** — clap defaults the value, so the mutation needs an `Option<Method>` and extra code; T12 ("card's contents per method") plus A1 (which omits the flag) cover the realistic shape. Not a finding. |
| C7 | two-source detection covers only one pair | **PARTIALLY CAUGHT** — T7 is singular; 10 pairs exist. The sharp one is stdin contention: `--hashlock-phrase-stdin` + `--hex -`. → **M-4** |
| C9 | `hash:` record spelled `sha256:`, `hash: `, or uppercase hex | **CAUGHT for the prefix** by T10 + A1; **NOT CAUGHT for hex case** by any row. → **M-8** |
| C10 | the card or a warning is written to **stdout** | **CAUGHT only if T10 runs in a warning-emitting configuration.** Measured: the anchor phrase is 28 characters, and §7 warns for hardened only **under 20**, so A1's invocation emits no warning at all. → **I-7** (second half) |
| C12 | `--out` writes **grouped** ms1 | **CAUGHT** — A3's "75-character" wording rules out separators. §4.4's "`--group-size`/`--separator` apply" is scoped to the card; A3 disambiguates. Good. |
| C13 | `--out` mode `0644` | **CAUGHT** — T11. |
| C14 | `--json` emits `method` for a supplied preimage, or omits `phrase_chars` | **PARTIALLY CAUGHT** — T13 is one "schema" test; §4.4's conditional-key clause needs both variants. → **M-10** |

### 1.5 Verb dispatch and argv guard

| # | mutation | CAUGHT BY / NOT CAUGHT |
| --- | --- | --- |
| V1 | any of the four `_ => unreachable!` arms left in place | **CAUGHT — the strongest test in the spec.** T11's "**one test per `unreachable!` site that panics on 0.17.x**" is a mutation proof by construction: the test is required to demonstrate its own failure against the pre-change binary. A4 repeats it. This formulation should be the model for the rest of §11. |
| V1b | the arms become `_ => <functional>` catch-alls rather than explicit `Payload::Preimage(..)` arms | **NOT CAUGHT, and not specified.** A future kind `0x04` would then be silently rendered as a preimage. §9.3 makes the sweep a *discipline*, not a gate. → **I-11**, census §5. |
| V2 | `derive`/`verify` refusal placed **after** `payload_entropy_and_language` | **CAUGHT in effect** — post-change the helper's arm is itself a typed refusal carrying the same remedy, so the user-visible result is identical; the real hazard (the 0.17.x panic) is covered by T11. Not a finding. |
| V3 | `decode` prints words for a preimage | **CAUGHT** — A4 ("**never** words"); §11's T14 line should say so too. → **N-2** |
| V4 | `encode --hex` starts emitting `0x03` | **CAUGHT** — by the **pre-existing** entr-32 corpus rows, which would all flip. Worth stating in the plan as intended coverage. |
| A1g | `--hashlock-phrase` not in `SECRET_FLAGS` | **CAUGHT** — T1. |
| A2g | `SUBCOMMANDS` left at 12 | **CAUGHT twice** — T1 (the guard would not fire), and measured: `SUBCOMMANDS: [&str; 12]` is a **sized array** (`argv_guard.rs:67`), so 12→13 is a compile error if missed. |
| A3g | `override_applies` not updated (`--allow-argv-secret` dead) | **CAUGHT by §6's own named gate** ("the same invocation with stdin at `/dev/null` still derives from the flag's value"), which §11 does not repeat. → **M-11** |
| A4g | `flag_class` not updated → refusal says "a BIP-39 passphrase" | **NOT CAUGHT** — T1 pins the refusal, not its class string. §6 says each of the four parts "has a symptom"; §11 lists one test. → **M-11** |
| A5g | `Source` not built `.on("--hashlock-phrase")` | **CAUGHT** — §6's `/dev/null` gate. |

### 1.6 Copy (§7)

| # | mutation | CAUGHT BY / NOT CAUGHT |
| --- | --- | --- |
| W1 | sha256 warning gated on length instead of firing **always** | **NOT CAUGHT** — A2 demonstrates "always" at exactly one length (the 28-char anchor). §8's other sha256 rows are *derivation* rows (X/H), not card rows. → **I-8** |
| W2 | hardened warning threshold moved to 19 or 21 | **NOT CAUGHT** — T12 has no length axis; §8's 20-character row pins X, not the warning. → **I-8** |
| W3 | reuse lines dropped | **CAUGHT** — T12 names them explicitly. |
| W4 | method-line instruction reverts to "try both" | **NOT CAUGHT** unless T12 pins the sentence verbatim. → **N-3** |
| W5 | `--random` card prints only one half | **CAUGHT** — T12 names "both `--random` halves". |

---

## 2. Findings

### C-1 (Critical) — the ms1-shaped-phrase refusal does not refuse the uppercase form, and no row would show it

§4.3 states the guarantee unconditionally and names its mechanism:

> **An ms1-shaped phrase is refused** on both channels, naming `--in`/`-`
> (r2 review C-1), reusing `argv_guard::is_ms1_shaped` — which is a private
> `fn` today (`argv_guard.rs:134`) and becomes `pub(crate)` — **so the two
> predicates cannot drift.** That function already strips display separators
> before testing, which is why a grouped plate string is caught too.

The last sentence is true; the guarantee is not. The case folding that makes the **argv** guard
catch `MS1…` does not live in `is_ms1_shaped` — it lives in its caller, `argv_candidates`
(`argv_guard.rs:104-111`), whose doc comment says so:

> **Neither trimming nor case-folding is optional.** ` ms1…`, `ms1…` and an
> uppercase `MS1…` are the same material; a classifier that saw only the
> literal token would let two of the four spellings through…

`is_ms1_shaped` itself tests `t.starts_with("ms1")` against a lowercase `BECH32_CHARSET`
(`argv_guard.rs:143-145`). Exporting the predicate *without* its normalisation exports the
half that "let[s] two of the four spellings through" — the drift §4.3 says cannot happen.

**Measured**, by transcribing `argv_guard.rs:134-146` + `format.rs:12-14` verbatim into a
standalone `rustc` binary:

```
PREDICATE ALONE (what SPEC §4.3 says the phrase path reuses):
  lowercase   -> true
  UPPERCASE   -> false   <-- QR/BIP-173 form, accepted by `ms decode`
  grouped     -> true
  leading sp  -> true
VIA argv_candidates() normalisation (what the ARGV guard actually does):
  UPPERCASE   -> true
```

Cross-checked against the shipped `target/debug/ms`: `ms decode MS10ENTRSQ…34V7F` on argv **is**
refused as material (the argv path normalises), and `crates/ms-cli/tests/decode_uppercase.rs`
shows `ms decode` **accepts** the all-uppercase form as a first-class QR spelling, with
`crates/ms-codec/tests/uppercase_envelope.rs` and `design/PLAN_ms1_envelope_uppercase.md`
behind it. So the uppercase spelling is not exotic: it is a shipped, CI-executed acceptance of
the very crate this spec extends.

**Consequence.** An operator who pastes the uppercase transcription of a preimage plate into
`--hashlock-phrase-stdin` is not refused. The tool derives a preimage from the ASCII text of
their ms1 string and emits a `hash:` record and a plate that have nothing to do with the plate
they pasted — the r2 C-1 hazard the refusal exists to close, arriving through the one spelling
the predicate cannot see. The trimming half is fine (`strip_display_separators` removes leading
whitespace); **case is the only hole**.

**Why this is my finding and not the correctness lens's:** no listed row or test uses an
uppercase ms1 as a phrase — §8 has no ms1-shaped refusal row at all (see I-3), so the corpus
could not reveal it at any point in the cycle.

**Remedy (one clause + rows).** Have §4.3 name the normalisation, not just the predicate:
export a `pub(crate) fn is_ms1_shaped_normalized(s) = is_ms1_shaped(&s.trim().to_ascii_lowercase())`
(or make `is_ms1_shaped` fold internally and have `argv_candidates` stop pre-folding), and add
§8 rows for the ms1-shaped refusal in **four** spellings — lowercase, uppercase, grouped,
space-padded — on **both** channels.

---

### I-1 — no row where the id and the prefix byte disagree, so "dispatch on the prefix" is unfalsifiable

§1 makes it normative: "**Readers still dispatch on the prefix** — the id is a human affordance,
never a parse input." Every §8 kind row has id and prefix agreeing, so a reader that dispatched
on the id would pass all of them.

The mutation is reachable: measured, `RESERVED_ID_BLOCKLIST` is consulted **only** in
`shares.rs:50` (re-roll during share-set generation) and a test at `shares.rs:469`. It is not
consulted on decode. Nothing prevents a hand-made or adversarial string carrying id `hash` with
prefix `0x00`, or id `entr` with prefix `0x03`.

**Remedy:** two rows — (a) id `hash`, payload prefix `0x00` → decodes as **entr**; (b) id
`entr`, payload prefix `0x03` → decodes as **preimage**. These are also the two rows the Go
port needs most (§4), because the device renders the id to a human.

---

### I-2 — §11's kind-confusion test does not pin entr-**32**, the colliding length

§1's headline hazard is that the preimage is "the first kind that collides: 75 characters,
exactly entr-32". §11's test is "entr and mnem strings refused and kind 3 accepted" — no length
named. An entr-16 (50 characters) satisfies it while a length-dispatching `--in` passes.

**Remedy:** say **entr-32** in T5, and pin the pair in §8 as adjacent rows (`ms10entrsq…` vs
`ms10hashsq…`, both 75).

---

### I-3 — the ms1-shaped-phrase refusal has no vector row and no test, on either channel

Independent of C-1's case gap. §8's refusal rows are "the 64-hex refusal; a non-ASCII refusal;
empty" — the ms1-shaped refusal is absent. §11's only candidate line reads as the `--in` source.
So the folded r2 **Critical** C-1 ships with nothing that can fail on it.

Note the asymmetry: §4.3 introduces two refusals in the same breath ("Two refusals that exist
because the alternative is a silently different X"), and only one of them got a row.

**Remedy:** add the refusal to §8's row list and to §11 naming **both** channels.

---

### I-4 — the whitespace-sensitivity row covers spaces; the spec names `-` and `,` as equally hazardous

§4.3 rules out `parse::read_input` precisely because it "strips all whitespace **plus `-` and
`,`**". §8's guarding row is "a phrase with leading, trailing and doubled **spaces**". Measured
(`format.rs:12`): `is_display_separator(c) = c.is_whitespace() || c == '-' || c == ','`, and
`strip_display_separators` is called by `is_ms1_shaped` — the function §4.3 is making
`pub(crate)` and wiring in next to the phrase path. Reusing it (or `read_input`) on the phrase
itself silently rewrites `correct-horse-battery-staple` to `correcthorsebatterystaple`.

Hyphen-joined and comma-separated phrases are the normal output shape of diceware generators —
the generators §7's copy tells the operator to use.

**Remedy:** one row with a hyphen and a comma in the phrase, pinning X and H.

---

### I-5 — the printable-ASCII boundary is pinned on the non-ASCII side only

§4.3: "Non-empty, **printable ASCII only**". §8's boundary row is "a non-ASCII refusal", which
exercises bytes ≥ 0x80. A predicate written as `is_ascii()` — the obvious one-call mistake —
admits TAB (0x09), DEL (0x7F) and every C0 control byte while passing every listed row.

**Consequence:** a phrase containing a tab derives successfully, and the operator writes down a
phrase whose whitespace is invisible on paper and unreproducible on a plate. That is preimage
loss, and it is the failure the §4.4 character-count line exists to make visible — so the two
mitigations have a shared blind spot.

**Remedy:** rows for TAB and DEL (refused), alongside the existing non-ASCII row; and a row for
`0x20`/`0x7E` accepted, so the boundary is pinned on both sides.

---

### I-6 — the 64-hex guard's case predicate is not unified with `--hex`'s parser

§4.3 refuses a 64-character all-hex phrase because "deriving from it produces a valid-looking
record for a different X". §8 carries one row for it. Measured: `--hex` resolves through
`parse_hex_entropy` → `Vec::<u8>::from_hex` (`encode.rs:283`, `use hex::FromHex`), and the `hex`
crate accepts **upper and lower** case. If the phrase guard is written lowercase-only, an
operator pasting uppercase hex — the form many tools emit — is not redirected to `--hex`; they
get a silently different X.

This is the same drift class §4.3 unified `is_ms1_shaped` to prevent; the hex predicate got no
such treatment.

**Remedy:** state that the phrase guard uses the same hex predicate as `--hex`, and add an
uppercase 64-hex refusal row.

---

### I-7 — stdout purity has one test, unspecified in both configurations where it can break

Two normative guarantees ride on stdout being exactly one line: §4.4's "**`--out` never
suppresses it**" (r2 I-5, folded), and the polarity inversion — stdout public, stderr secret —
that the whole verb is built around. §11's guard is "stdout is exactly the record line", with no
configuration named. Two mutations pass it:

- **`--out` suppresses stdout.** T10 does not say it runs under `--out`; T11 checks only mode and
  overwrite; A3 checks the file. `me sysw pack --in -` then receives an empty stream — the exact
  outcome §4.4 says copying `ms encode`'s shape would produce.
- **Warnings or the card go to stdout.** Measured: the anchor phrase is **28** characters, and
  §7 warns for hardened only **under 20**, so A1's invocation emits no warning at all. If T10 is
  run on the anchor under the default method, the most likely stdout-pollution mutation never
  meets a warning.

*Rubric note:* neither mutation changes X or H, so the brief's item-2 rule would say Minor. I am
filing Important because both are normative, review-derived guarantees whose only test cannot
fail on them, and the fix is a few words.

**Remedy:** "stdout is exactly the record line — **under `--out`, and under `--method sha256`
(which always warns)**".

---

### I-8 — the warnings are never tested at the point where they fire

§7 emphasises "**`--method sha256`, always, at every length**" and sets the hardened threshold at
"under 20 characters". §11 has T12 ("the card's contents per method") with no length axis, and A2
demonstrates "always" at one length (28). Both threshold mutations — sha256 gated on length,
hardened threshold at 19 or 21 — pass every listed test.

§7 says "**the copy is the defence**"; L12 rules the sha256 warning always fires. A defence with
no boundary test is the thing this project keeps finding late.

**Remedy:** card assertions at 19/20 characters (hardened, warn / no warn) and at 100 characters
(sha256, still warns).

---

### I-9 — the reproduction test can be written so it proves nothing, and §8's wording does not forbid it

§8 requires the test to execute `python3` and `openssl kdf` and to fail if either is absent, and
adds a CI preflight. Both are good. Neither closes the false-PASS that matters: **if the external
command line is built from `HASHLOCK_SALT` / `HASHLOCK_ITERATIONS` / `HASHLOCK_DKLEN`, then
mutating those constants moves both sides of the comparison together and the test still passes** —
blinding the cross-tool check to exactly the mutations (D1–D4) it exists to catch.

Full design set in §3 below.

**Remedy:** §8 states that the salt string, the iteration count, the dkLen and the expected hex
appear in the test as **literals**, independent of the crate's constants, with a separate
assertion that the constants equal those literals; and that the test compares **captured stdout**
of both tools, three ways.

---

### I-10 — the lockstep set omits the three drifts a Go port is most likely to have

§8's lockstep rows are exactly three: a 100-character phrase identical on both sides, a
101-character refusal, the 64-hex refusal. All three test the **cap** and a **refusal**. The host
half of the phrase rule that a device text-entry widget is most likely to get wrong is not there:

- **the leading/trailing/doubled-spaces row** — a TinyGo entry widget that trims a trailing space
  is the single most plausible port drift, and it changes X silently;
- **the empty-phrase refusal** — a device that admits it derives a preimage anyone can guess;
- **the printable-ASCII boundary** (per I-5), doubled on a device whose keyboard may offer
  characters the host rule refuses.

§10 vendors the whole corpus into the fork with a pin test, which mitigates this if the pin test
drives every row — but §8's lockstep list is what H2's plan will read, and it names three rows.

**Remedy:** move the spaces row, the empty refusal and the ASCII-boundary rows into the lockstep
set, and state that the fork's pin test drives the vendored rows in **both** directions (an
encode-side drift — a Go encoder writing id `entr` for a preimage — is invisible to a decode-only
pin test).

---

### I-11 — nothing structural stops the next kind from re-opening the `unreachable!` hazard

§3 is right that `#[non_exhaustive]` means "the compiler will not tell anyone a variant was
added", and T11's per-site panic test is the best-specified gate in the spec. But two things are
left as prose:

1. **The spec never says what the arms become.** If the four sites become `_ => <functional>`
   catch-alls, a future kind `0x04` is silently rendered as a preimage — a wrong result, not a
   panic. If they become explicit `Payload::Preimage(..)` arms with `_ =>` retained, the hazard
   recurs identically and needs the same sweep.
2. **§9.3's remedy is a discipline** ("Every downstream crate **MUST** sweep its
   `_ => unreachable!` arms"), reproduced by no command — the shape this constellation has
   recorded as a trap.

**Remedy:** state that the arms are explicit `Payload::Preimage(..)` matches, and commit a test
asserting the ms-cli catch-all count (`grep -rc '_ => unreachable' crates/ms-cli/src`) equals the
number the cycle intends, so the next kind re-triggers the sweep mechanically. Precedents exist
in-repo: `bip93_inline_vectors.rs:240 invalid_corpus_length_is_64`, and the `consts.rs` test that
"Locks the bijection … so that a future edit to one without the other fails CI loudly".

---

### Minor

- **M-1** — `Error::PreimageLengthMismatch { got: usize }`: the spec never says whether `got` is
  the payload length or the X length, so §8's "each refused by name" rows cannot pin it and an
  off-by-one in the operator-visible number is untestable. Define it.
- **M-2** — Only the `--random` **refusal** is tested (T8, A5). Nothing asserts `--random --json`
  without `--out` **succeeds**, so a gate narrowed to `--out` alone passes. Over-refusal, fails
  closed.
- **M-3** — T9 says "`--method` with **a** supplied preimage"; §4.2 refuses it for three sources
  (`--hex`, `--random`, `<ms1>`). Name all three.
- **M-4** — T7 "two sources exit 64" covers one of ten pairs. Name at least the stdin-contention
  pair (`--hashlock-phrase-stdin` + `--hex -`), where two sources claim the same stream.
- **M-5** — The preflight `python3 -c 'import hashlib'` is close to unfalsifiable: `hashlib` is
  stdlib, so it can only fail when `python3` is absent entirely. Exercise the actual capability:
  `python3 -c 'import hashlib; hashlib.pbkdf2_hmac("sha256", b"x", b"y", 1)'`. (The other half is
  sound — measured here, `openssl kdf --help` exits **0** on OpenSSL 3.6.3.)
- **M-6** — Only the **anchor** row has stated independent provenance ("reproduced in two
  independent tools"). The other derivation rows (1, 20, 64, 65, 100, 101 characters, spaces)
  have none; if generated by the implementation they are regression pins, not correctness pins.
  Either run the reproduction over **all** derivation rows (PBKDF2 at 100k × ~14 rows is cheap)
  or say plainly which rows are regression-only. The repo already knows this shape —
  `codex32_vendor_parity.rs` documents a "golden corpus captured ONCE from the PRE-vendor
  implementation". This is also the only thing standing behind D5b (a CLI method-wiring swap),
  whose sole pin today is manual acceptance A1/A2.
- **M-7** — `Zeroizing` on `Payload::Preimage` (§3) is a type-level property no listed test can
  fail; a bare `[u8; 32]` would pass everything. Secret-handling → Minor per the 2026-08-27
  ruling. A `static_assertions`/trybuild pin would close it.
- **M-8** — No row pins the **hex case** of `hash:<64 hex>`, `preimage_hex` or `sha256_operand`.
  A Go port or a refactor emitting uppercase satisfies every listed test; the only check is A6,
  which is manual and cross-repo.
- **M-9** — The 64-hex guard's over-refusal direction is untested: no row asserts that a short
  all-hex phrase (`beef`) is accepted. §8's "one character" row would catch it only if that
  character happens to be a hex digit — specify it.
- **M-10** — §4.4's conditional `--json` keys (`method` omitted for supplied preimages) sit under
  one "schema" test; pin both variants.
- **M-11** — §6 says the four-part guard edit has "a symptom if skipped" for each part, and names
  a gate for part 4 ("the same invocation with stdin at `/dev/null` still derives from the flag's
  value"). §11 carries one argv-guard test and does not repeat that gate. Part 3 (`flag_class`)
  has no test at all. List four.

### Nit

- **N-1** — A3's "writes a 75-character `ms10hashsq…` string" is satisfied by a wrong
  `PREIMAGE_PREFIX`: §1 itself notes `0x00`, `0x02` and `0x03` share their top five bits and all
  render a leading `q` — and so does `0x01`. The corpus row is what pins the prefix; the
  acceptance line should cite the full expected string or say "per the corpus row".
- **N-2** — §5 and A4 say `decode` must **never** print words; §11's "decode, inspect and combine
  on the kind" does not. Carry the word "never" into the test line.
- **N-3** — The method line's instruction sentence (§7, "try each method that shipped with the
  version named on this card" — deliberately not "try both", review M-4) has no test pinning its
  text.
- **N-4** — Pre-existing, and the spec's §6 edit touches it: `SECRET_FLAGS`'s doc comment reads
  "The nine flag-keyed secret channels" above `const SECRET_FLAGS: [&str; 4]`
  (`argv_guard.rs:85-86`). Fix the comment while the line is being edited (it becomes 5).

---

## 3. The reproduction test — false-PASS designs (brief item 3)

§8's wording, for reference: the test "executes the `python3` and `openssl kdf` reproductions and
FAILS if either tool is absent", plus a preflight (`openssl kdf --help`,
`python3 -c 'import hashlib'`) in the Ubuntu-only `test (ms-codec)` job "so a missing tool fails
the step rather than a test someone can `#[ignore]`".

| # | false-PASS design | does §8's wording forbid it? |
| --- | --- | --- |
| **FP-1** | **Constants-parameterised command.** The test builds the python/openssl invocation from `HASHLOCK_SALT`, `HASHLOCK_ITERATIONS`, `HASHLOCK_DKLEN`. Mutating a constant moves both sides together; the "independent" tool reproduces the mutation faithfully and the test prints ok. | **NO.** §8 requires execution and presence, never independence of the parameters. This is the important one — it re-opens D1–D4, the mutations the test exists for. → **I-9** |
| **FP-2** | **Self-comparison.** The test asserts `preimage_hardened(p) == <constant>` where the constant came from the vector file, which was generated by `preimage_hardened`; the external tools are run and their output discarded or only status-checked. Rust-vs-Rust wearing a cross-tool costume. | **PARTLY.** "Executes the reproductions" implies comparing output, but §8 never says the comparison is three-way against a literal. Close it explicitly. |
| **FP-3** | **Exit-status-only assertion.** `.status().success()` on the `openssl kdf` command, stdout never captured. `openssl kdf` exits 0 while printing a value nobody read. | **NO.** Say "compares captured stdout". |
| **FP-4** | **`#[ignore]` (or a feature/`cfg` gate) on the test itself.** The preflight proves the *tools* are present; nothing proves the *test ran*. `cargo test` prints `ok` with `N ignored`. A 100k-iteration PBKDF2 over ~14 rows is exactly the kind of test someone marks slow. | **PARTLY, and only for one motive.** §8 anticipates an ignore caused by a *missing tool*; an ignore added for slowness or flake is untouched. Close it with a CI step that asserts the test executed **by name** — e.g. `cargo nextest run -p ms-codec -E 'test(hashlock_repro)'` and assert the summary reports 1 test run. |
| **FP-5** | **Stub on PATH.** A shim `openssl`/`python3` earlier in `PATH` satisfying `--help` and echoing the expected hex. | **NO** — though this is the weakest of the five on a hosted runner. Cheap mitigation: record `openssl version` and `python3 -VV` in the job log. |
| **FP-6** | **Preflight and test in different jobs**, so the preflight passes somewhere the test does not run. | **YES — already forbidden.** §8 pins both to `test (ms-codec)` and explains the LibreSSL/macOS reasoning (r2 I-6). Credit where due. |

---

## 4. Vector sufficiency for the Go port (brief item 4)

Rows that would **not** see a behaviour-faithful Go port drifting, ranked:

1. **Phrase-entry normalisation** — no lockstep row makes the device's entry path byte-verbatim.
   A widget that trims a trailing space produces a different X with no error anywhere. (I-10)
2. **The empty-phrase refusal** and **the printable-ASCII boundary** — neither is in the lockstep
   set; both are the difference between a refusal and a guessable or unreproducible preimage.
   (I-10, I-5)
3. **id ↔ prefix disagreement** — the device *renders* the id, so a Go reader tempted to dispatch
   on it has no row that fails. These two rows matter more for the port than for the host. (I-1)
4. **Encode-direction drift** — §10's vendored pin test must drive rows in both directions; a
   decode-only pin misses a Go encoder emitting the wrong id or prefix.
5. **Hex case / record grammar** — the `hash:` record is a cross-repo contract with no row; case
   or separator drift is caught only by manual acceptance A6. (M-8)
6. **The uppercase ms1 spelling** — whatever the host decides for C-1, the device needs the same
   rule as a row, since the ms1 QR form is exactly what a device would ingest.

The kind rows, the share round trip, the length rows and the both-methods X/H rows are
sufficient for what they cover; the gap is entirely on the **phrase-rule** side, which is the
half with no shared code at all.

---

## 5. The census question (brief item 5)

Measured at `7fc1e58`:

| count | declaration | verdict |
| --- | --- | --- |
| twelve subcommands | `const SUBCOMMANDS: [&str; 12]` (`argv_guard.rs:67`) | **Prose is fine — already compiler-enforced.** A sized array; 12→13 cannot be silently skipped. |
| four secret flags | `const SECRET_FLAGS: [&str; 4]` (`argv_guard.rs:86`) | **Prose is fine — already compiler-enforced**, same reason. (Fix the "nine" comment, N-4.) |
| five blocklist ids | `pub const RESERVED_ID_BLOCKLIST: &[[u8; 4]]` (`consts.rs:71`) | **Should become an assertion.** It is a **slice**, not a sized array, so the count is enforced by nothing, and the entry is consulted only in `shares.rs:50`. Either change the type to `[[u8; 4]; 6]` or add `assert_eq!(RESERVED_ID_BLOCKLIST.len(), 6)`. |
| four `unreachable!` arms | four `match` sites, `#[non_exhaustive]` upstream | **Must become an assertion** — it is the one count no type can carry, which is precisely what §3 says about `#[non_exhaustive]`. A committed count test is the only mechanical form. → I-11 |
| `HASHLOCK_PHRASE_MAX_CHARS = 100` | new, both sides | **Should become an assertion**: `const _: () = assert!(HASHLOCK_PHRASE_MAX_CHARS == 100);` on each side, given §4.3's explicit warning against binding it to the device's `passphrase.MaxLen`. |
| new corpus row count | new | **Should become an assertion.** The existing replay asserts only `!corpus.is_empty()` (`vectors.rs:29-32`), which would not notice a corpus that lost most of its rows. Precedent: `invalid_corpus_length_is_64`. |
| 75 characters; the 16..46 BIP-93 bracket; three `PayloadKind` variants; `ms10hashsq…`/`ms10entrsq…` | — | **Prose is fine.** The vector rows and ms-codec's own exhaustive matches are the assertions. |

---

## 6. Closing counts

- **Mutations enumerated:** 63 pre-read from §1–§7, plus 8 added while reading §8–§14 → **71**.
- **Not caught by any listed row or test:** 23.
- **Findings: 1 Critical / 11 Important / 11 Minor / 4 Nit.**
- **Machine-checked while writing:** `is_ms1_shaped` vs `argv_candidates` case behaviour
  (standalone `rustc` reproduction of `argv_guard.rs:134-146` + `format.rs:12-14`); live
  `target/debug/ms decode` on lowercase and uppercase ms1 on argv; `is_display_separator`'s
  character set; `RESERVED_ID_BLOCKLIST`'s declared type and its two use sites; `SUBCOMMANDS`
  and `SECRET_FLAGS` array types; `parse_hex_entropy` → `hex::FromHex`; `read_stdin_passphrase`'s
  stripping shape; `openssl kdf --help` exit status (0, OpenSSL 3.6.3) and
  `python3 -c 'import hashlib'` exit status (0); the anchor phrase length (28, vs §7's 20-char
  hardened threshold).
- **Not attempted** (other lenses): citation audit, funds-loss journeys, any challenge to L1–L23.
- **Nothing committed; the tree was not modified.**

### The one-line version

§8 and §11 are strong where the spec had a *value* to pin — the length rows, the both-methods
X/H rows, the 100/101 cap rows, and T11's per-site panic proof are all genuinely falsifiable —
and thin wherever the guarantee is a **refusal**, a **channel**, or a **warning**: the ms1-shaped
refusal has no row at all (and the mechanism §4.3 names cannot see the uppercase spelling, C-1),
the printable-ASCII and hex-case boundaries are pinned on one side only, and stdout purity and
the warnings are tested in configurations where their mutations cannot fire.
