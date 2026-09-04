# R0 round 0 — FIDELITY lens — `IMPLEMENTATION_PLAN_ms_hashlock_H1.md`

**Artifact:** `design/IMPLEMENTATION_PLAN_ms_hashlock_H1.md` at mnemonic-secret
`36d314daa98cb1a6d9212b47d1f44cfc04be47b8` (verified unchanged at session HEAD
`33c9b35`: the only commits since add briefs and a continuity file).
**Spec:** `design/SPEC_ms_hashlock.md`, same tree.
**Question:** does an implementer who has read only the plan and the spec produce
exactly what the spec specifies, and can each step be executed as written?

**Method.** Read spec and plan whole; resolved every cited path and line range
against the tree; diffed the plan's embedded hand-wire python against the
committed `scripts/plan-handwire-ms-hashlock.py`; applied that script to a
private `cp -r` copy (`/scratch/code/shibboleth/.tmp/fidelity-lens-ms`, no build)
and read the resulting wired source for the two Criticals below; recomputed every
derivation and length constant in `python3`. Ran no build and no gate.

**What the gate already proved, and what it could not see.** The gate wires ALL
fragments at once and runs `-E 'binary(/hashlock/) | test(/hashlock/)'`. The plan
defines exactly 64 `#[test]` items (`grep -c` on the plan = 64) and the gate
reports 64 run / 64 passed, so every test the plan writes passes against a fully
wired tree. What that leaves outside the gate, and where every finding below
lives: (a) code paths no plan test exercises, (b) the plan's TASK ORDER (the gate
never applies fragments one task at a time), (c) EXISTING tests outside the
`/hashlock/` filter, (d) sites the plan never edits, (e) the CI, corpus and H0
edits the gate's own NOT-covered line names.

Machine-verified true, so not re-litigated below: all anchor derivation values
(hardened X/H, sha256 X/H, `sha256(0xab*32)` = the corpus digest,
`sha256(0^32)`); both corpus ms1 strings are 75 characters; `me` spells `pbkdf2`
and `sha2` exactly as §2 requires (`me-cli/Cargo.toml:45-46`); `openssl kdf
--help` exits 0 on OpenSSL 3.6.3; `tempfile`/`serde_json`/`zeroize` are already
available to both crates' test targets; no test asserts
`RESERVED_ID_BLOCKLIST.len()`; `Tag::try_new` is alphabet-only so `"hash"`
constructs; `encode_shares` does not route through `encode()`, so the new
emit-side tag/kind check cannot break share generation; `ms decode` already emits
the `PrivateKeyMaterial` advisory (`decode.rs:140`), so `emit_preimage`'s is
consistent rather than invented.

---

## Table 1 — Traceability: spec → plan (§1–§12)

| spec requirement | plan site | verdict |
| --- | --- | --- |
| §1 `PREIMAGE_PREFIX = 0x03`, `[0x03][X:32]`, 75 chars | T1 (consts fragment); T1 S1 `constants_are_the_specs` | OK |
| §1 both `discriminate` doc-comment copies gain the `0x03` line | T2 fragment (script asserts exactly two `/// - any other prefix` lines; both are real, `envelope.rs:117`, `:188`) | OK |
| §1 length-collision table (`ms10entrsq…` / `ms10hashsq…`, both 75) | T2 S1 `the_entr32_and_preimage_pair_are_adjacent_rows` | OK |
| §1 rule 1 — `hash` joins the single-string accept set; `TAG_HASH`, `Tag::HASH` | T1 + T2 `decode.rs` fragment (`x if x == TAG_HASH`); `a_hash_single_round_trips_and_is_75_chars` | OK |
| §1 rule 2 — tag/prefix must agree, refuse `TagKindMismatch`, on decode | T2 `decode.rs` fragment rule 6b; `id_and_prefix_must_agree_both_directions` | OK at the codec; **C-1** at the CLI |
| §1 rule 2 — same refusal on ENCODE | T2 `encode.rs` fragment | OK |
| §1 rule 2 — singles only; the share axis untouched | `encode()` is single-only (verified); `combine_shares` untouched | OK |
| §1 rule 3 — `RESERVED_ID_BLOCKLIST` gains `hash` (six entries) | T1 fragment; count asserted in `constants_are_the_specs` | OK |
| §1 `PreimageLengthMismatch { got }`, `got` = bytes AFTER the prefix | T1 error fragment; T2 `dispatch_payload` arm | OK |
| §1 check precedes construction; no slice-index panic | `rest.try_into()` on `&data[1..]` (`data[0]` already read) | OK |
| §1 nine wrong lengths reachable through `decode` `{17,18,21,22,25,26,29,30,34}` | T2 S1 `preimage_length_rows_through_decode_name_their_error` | OK |
| §1 `{16,32,44}` → `UnexpectedStringLength` through `decode`, `PreimageLengthMismatch` through `combine_shares` | T2 S1 two tests | OK |
| §1 46 bytes unconstructible | T2 S1 `a_46_byte_payload_is_unconstructible` | OK |
| §1 codeword distance measured at plan time, > 2× the correction bound | gate step 5 + `codeword_distance_…` (measured 17) | OK |
| §2 `HASHLOCK_SALT`/`ITERATIONS`/`DKLEN` | T3 module + `constants_are_the_specs` | OK |
| §2 four functions with the spec's signatures | T3 module | OK |
| §2 `preimage_random` in ms-codec, `getrandom::fill`, fails closed | T3 (`map_err(|_| Error::RandomnessUnavailable)`) | OK (see N-3 note) |
| §2 `digest` NOT zeroized | T3 `-> [u8; 32]` | OK |
| §2 no `--salt` flag | `HashlockArgs` has none | OK |
| §2 dependencies spelled exactly as `me` spells them | T3 Cargo fragment — byte-identical to `me-cli/Cargo.toml:45-46` | OK |
| §2 anchor row measured, both methods | T3 `ROWS[0]` — all four values reproduced | OK |
| §3 `PayloadKind::Preimage`, `Payload::Preimage(Zeroizing<[u8;32]>)`, `InspectKind::Preimage` | T2 fragments | OK |
| §3 compile-time pin that the field is `Zeroizing<[u8;32]>` | T2 S1 `preimage_field_is_zeroizing` (type ascription; compile-enforced) | OK (N-1) |
| §3 LOUD sites: `allowed_for_kind`, `kind()`, `as_bytes()`, `validate`, InspectKind projection, `dispatch_payload`, `payload_wire_bytes` | T2 fragments — all seven | OK |
| §3 SILENT: the accept set | T2 `decode.rs` fragment | OK |
| §3 SILENT: **every** `_ =>` arm over `Payload`/`PayloadKind`/`InspectKind` | T8 covers `inspect.rs:182`, `:203`, `:223`, `:231`; **`cmd/split.rs:131` is over `PayloadKind` and is never touched** | **I-3 DROPPED** |
| §3 tests that pinned `0x03` as reserved flip, enumerated mechanically | T2 S5 grep | **I-2 — remedy does not fit the one test it must fix** |
| §3 the four loud CLI sites by disposition (decode ×2 functional, combine functional, payload_lang typed refusal) | T8: ONE early-return arm in decode's first match (the second becomes unreachable), combine arm, payload_lang refusal | OK behaviourally; **M-1** (the plan's own index says "two arms") |
| §3 `_ =>` catch-alls KEPT as `unreachable!`; a committed count test | T8 `unreachable_catch_all_count_is_pinned` = 4 (measured 4 today) | OK |
| §3 the refusal sits at `payload_lang.rs:61`, not in the verb bodies | T8 fragment (anchored on the helper's signature) | OK |
| §4.1 five sources, exactly one | T7 `pick_source` | OK |
| §4.1 ten two-source pairs → 64 | T5 S1 `every_two_source_pair_exits_64` | OK |
| §4.1 zero sources → 64 listing five; never defaults to stdin | T7 `FIVE_SOURCES`; T5 `zero_sources_exits_64_listing_five` | OK |
| §4.1 `--hex` refuses anything but 32 bytes, naming §8i, in both spellings | T7 length branch names both; **odd-length / non-hex fall into `parse_hex_entropy` and name neither** | **I-9** |
| §4.1 `<ms1>` of the wrong kind → "that is a seed backup, not a hashlock preimage" | T7 `SourceKind::Ms1` arm | OK (untested — I-7) |
| §4.1 `--in FILE` means the ms1 | T7 `Source::new(args.ms1, args.in_path)`; T9 S4 extends `in_flag_six_verbs.rs` | OK |
| §4.1 `--random` requires `--out FILE`; `--json` does not satisfy it | T7 gate; T5 `random_requires_out_file_and_json_alone_does_not_satisfy_it` | OK |
| §4.1 under `--random`, `--out` refuses to overwrite (`create_new`) | T7 `write_artifact_create_new` — implemented as `path.exists()` + truncating write | **I-4 altered mechanism** |
| §4.1 the other four sources keep overwrite semantics; `--out` owner-only | T7; T5 two tests | OK |
| §4.1 `--hex`'s unconditional warning, verbatim | T7 card block | OK (M-10 note on `--no-engraving-card`) |
| §4.2 `--method` phrase-sources only, else exit 64; card reads `preimage supplied`; `--json` omits `method` | T7 `refuse_method`, `method_line`, json block; T5 test | OK |
| §4.2 the default is announced whether or not the flag was given | T7 `unwrap_or(Method::Hardened)`, always emitted | OK |
| §4.3 non-empty, `0x20..=0x7E`, ≤100, byte-verbatim | T6 `validate_phrase` | OK |
| §4.3 `HASHLOCK_PHRASE_MAX_CHARS = 100`, its own constant | T6 | OK |
| §4.3 64-hex refusal naming `--hex`, either case | T6 + T9 | OK |
| §4.3 the 64-hex guard uses the same hex predicate as `--hex`'s parser (`hex::FromHex`) | T6 uses `is_ascii_hexdigit` | **M-6** (equal today, not structural) |
| §4.3 ms1-shape refusal, ONE predicate BOTH the guard and the phrase channels call; `argv_candidates` stops pre-folding | T5 adds `looks_like_ms1`; `argv_candidates` keeps its own `norm` and never calls it | **I-5 altered** |
| §4.3 shape test BEFORE the cap | T6 order + `ms1_shape_in_four_spellings_and_before_the_cap` | OK |
| §4.3 the phrase's VALUE is never normalised | T6 (folded copy only inside the predicate) | OK |
| §4.3 byte-verbatim reader, one `\r?\n`, never `read_input`/`read_phrase_input` | T6 `read_phrase_stdin` | OK |
| §4.3 terminal prompt line "Type the hashlock phrase, then Enter." | T6 implementation | implemented; **I-8 no test (§11 requires one)** |
| §4.4 stdout = one `hash:` line, lowercase, never suppressed by `--out` | T7; T9 `stdout_is_exactly_the_record_under_out_and_under_sha256` | OK |
| §4.4 `--out` = the preimage ms1, `0600` | T7; T5 `out_is_owner_only` | OK |
| §4.4 card first line names it as carrying the preimage | T7; T9 test | OK |
| §4.4 card carries digest, `sha256=` operand, grouped ms1, hex, method line, char count, §8i + F-132, §7 reuse lines, the method warning, source kind without its value | T7 card block — all present | OK |
| §4.4 `--json` replaces the record line; keys and omission rules; lowercase; advisory | T7; T9 `json_both_variants` | OK |
| §4.4 the `me sysw pack` spelling (stdin, no `--in`) | T9 `record_line_shape_is_what_me_sysw_pack_reads` (shape only); acceptance at T11 S3 | OK (M-7 on its doc comment) |
| §5 `decode` prints kind/hex/digest, never words | T8 `emit_preimage` + test | OK |
| §5 `inspect` reports the kind | T8 fragment + test | OK for a valid single; **C-2 for a mismatched one** |
| §5 `combine` prints as `decode` does | T8 fragment + test | OK |
| §5 `derive`/`verify` refuse with `ms hashlock <ms1>` | T8 `payload_lang.rs` fragment + test | OK |
| §5 `encode --hex` unchanged (only door) | untouched | OK |
| §5 `split` — codec supports it, a test pins it | T2 `preimage_share_round_trip` | OK |
| §5 `repair` unchanged and benign | T8 `repair_on_an_undamaged_preimage_plate_is_a_no_op` | OK |
| §6 part 1 `SUBCOMMANDS` 12→13 | T5 fragment | OK |
| §6 part 2 `override_applies` | T5 fragment | OK |
| §6 part 3 `flag_class` = "a hashlock phrase" | T5 fragment + test | OK |
| §6 part 4 the phrase's `Source` bound `.on("--hashlock-phrase")` | T7 reads `admitted("--hashlock-phrase")` directly instead of via `Source` | OK behaviourally; see **I-10** for the `-` case the `Source` path would have handled |
| §6 part 5 `--hex` binding | T7 `Source::new(..).on("--hex")` | OK |
| §6 part 6 positional binding | T7 `.on(CH_POSITIONAL)` | OK |
| §6 three `/dev/null` gates, ONE TEST EACH | T5 — two tests (hex + positional share one), empty-pipe stdin | **M-3** |
| §7 sha256 brainwallet line, always | T7 + T9 `sha256_warns_at_every_length` | OK (N-2 prefix) |
| §7 hardened under-20 line | T7 + T9 `hardened_warns_under_20_only` | OK |
| §7 reuse lines verbatim | T7 — character-for-character except em dash → `--` | OK |
| §7 method line's instruction, "each method that shipped with the version named on this card" | T7 + T9 assertion | OK |
| §7 `--random` card says BOTH halves and names the artifact that exists | T7 + T9 `random_card_names_the_file_not_a_plate` | OK |
| §7 `--hex` line | T7 + T9 | OK |
| §8 kind rows (accept-set round trip, mismatch both directions, the entr-32 pair, lowercase hex) | T2, T9 | OK |
| §8 length rows by door, each naming its error | T2 tests + corpus `lengths_by_door` | OK |
| §8 the downgrade row | gate step 6 only — **no corpus row and no shipped test** | **M-4** |
| §8 derivation rows, both methods, X and H, ALL reproduced externally | T4 corpus — anchor complete, nine rows are `"…"` for the implementer with a `provenance` field | see "Corpus" below |
| §8 refusal rows | T4 corpus `refusals` + T6/T9 tests | OK |
| §8 reproduction test: literal constants, three-way captured stdout, fails-if-absent, run-by-name | T4 `hashlock_repro.rs` + CI step | OK; **M-5** on the CI spelling |
| §8 CI preflight exercising the capability; `openssl version`, `python3 -VV` | T4 S4 | OK |
| §8 lockstep rows for H2, driven in both directions | T4 corpus `lockstep` | OK |
| §9 MIGRATION items 1–5 | T10 S1 — all five present, item 5 with the measured third reader shape | OK |
| §9 H0 before the 0.18.0 release | T10 item 5 text + T11 S1 | see "H0" below |
| §10 versions 0.8.0 / 0.18.0, pin `=0.8.0` | fragments exist; **no task applies the ms-cli one** | **I-1** |
| §10 release order H0 → both crates together, corpus SHA, CHANGELOG, MIGRATION, dry run, both tags | T11 S2 | OK |
| §10 manual chapter in lockstep + flag-coverage lint | T10 S3 | OK; **M-8** (the lint is discovered, not named) |
| §10 Rust-primary / provenance pin (H2) | out of H1 scope, stated in §10 | OK |
| §11 every listed test | see I-6, I-7, I-8, M-3; all others present | partial |
| §12 acceptance items 1–8 | T11 S3 (1–6), T11 S1 (7), T11 S2 (8) | OK |

## Table 2 — Traceability: plan → spec (built but not asked for)

| plan builds | spec basis | verdict |
| --- | --- | --- |
| `consts::VALID_PREIMAGE_STR_LENGTHS = &[75]` | not named in §1/§3, but `allowed_for_kind` needs a set per kind | supporting, not operator-visible — OK |
| `envelope::prefix_of(&Payload) -> u8` (`pub(crate)`) | needed to fill `TagKindMismatch { prefix }` (§1 rule 2) | OK |
| `PayloadKind::single_tag(self) -> Tag` | §1 rule 2's check; named in the brief's interface list | OK |
| `Error::RandomnessUnavailable` | §2 "failing closed"; the variant is the mechanism | OK (getrandom's inner error is discarded — see N-3) |
| `CliError::Usage(String)` → 64 | §4.1's exit-64 refusals need a variant; CHANGELOG declares it | OK |
| `PhraseRefusal` enum + `message()` | §4.3's five refusals | OK |
| `SourceKind`/`Derived`/`refuse_method`/`method_line` | internal shape of §4.1/§4.2 | OK |
| `write_artifact_create_new` | §4.1's no-overwrite rule | mechanism differs — **I-4** |
| `emit_preimage` fires the `PrivateKeyMaterial` advisory on `decode` | matches the existing `decode.rs:140` behaviour | OK, not invented |
| `--group-size` / `--separator` on `hashlock` | §4.4 says they apply | OK |
| `inspect` reason `"tag-kind-mismatch"` + its `reason_text` | new operator-visible string, not in the spec; it is the visible half of §1 rule 2 in `inspect` | acceptable, but it fires in only ONE of the two directions — **C-2** |
| `inspect` version line `InspectKind::Preimage => "v0.8"` | new operator-visible string; §1 calls the kind a v0.8 addition | OK |
| deriving X from the literal one-character phrase `-` | spec leaves `--hashlock-phrase -` undefined | **I-10** |

---

## Findings

### C-1 (Critical) — the three new codec errors have no CLI mapping, so §1 rule 2's refusal surfaces as `unhandled ms_codec::Error variant` at exit 1

The plan's own index promises it (plan:112):

> `crates/ms-cli/src/error.rs` | `CliError::Usage` (exit 64); **mapping for the three new codec errors**

No task, no step and no fragment delivers the mapping. Verified on the wired
copy: `grep -n "PreimageLengthMismatch\|TagKindMismatch\|RandomnessUnavailable\|Usage"
crates/ms-cli/src/error.rs` returns only the four `Usage` lines. So all three fall
through the existing wildcard (`ms-cli/src/error.rs:276`):

    other => CliError::BadInput(format!("unhandled ms_codec::Error variant: {:?}", other)),

Consequences, all operator-visible:

1. `ms decode` on a forged/corrupted `ms10hash…` string carrying a seed payload —
   the fail-closed refusal §1 rule 2 exists for — prints
   `unhandled ms_codec::Error variant: Error("tag "hash" does not name the kind the
   prefix byte 0x00 carries; …")` and exits **1**, not 2.
2. The exit class is wrong. Every other ms1 format violation is
   `CliError::FormatViolation` → exit 2, and the comment immediately above
   `InconsistentShareSet` in the same file says why this matters:
   *"Routed explicitly so it does NOT fall through to the `other =>` BadInput
   (exit 1) wildcard below."* The plan reproduces exactly the defect that comment
   warns against, on the new kind's central refusal.
3. `--json` reports `"kind": "BadInput"` rather than `TagKindMismatch` /
   `PreimageLengthMismatch`.
4. `RandomnessUnavailable` from `--random` reaches the operator the same way.
5. The message opens with "unhandled … variant", which tells the operator the
   tool is broken for a refusal the spec designed.

Why nothing caught it: the wildcard compiles, and every `TagKindMismatch` /
`PreimageLengthMismatch` test in the plan is codec-level (`hashlock_kind.rs`) —
no plan test drives a mismatched or wrong-length string through the CLI.

Fix: three `From` arms (two `FormatViolation`, one `BadInput` or `FormatViolation`
for randomness) plus one CLI test per arm asserting the exit code and the absence
of the string "unhandled".

### C-2 (Critical) — `ms inspect` reports "OK: would decode" for a string `ms decode` refuses

Verified by reading the wired `crates/ms-cli/src/cmd/inspect.rs`. The Task 8
fragment relaxes rule 6:

    if tag_bytes != TAG_ENTR && tag_bytes != TAG_HASH {

and adds the compensating check only inside the `InspectKind::Preimage` arm of
rule 10:

    InspectKind::Preimage => {
        if report.payload_bytes.len() != 32 { reasons.push("payload-length-mismatch"); }
        if tag_bytes != TAG_HASH { reasons.push("tag-kind-mismatch"); }
    }

Trace a string with prefix byte `0x00` and id `hash` (the exact string the plan's
own `id_and_prefix_must_agree_both_directions` forges as
`forged_hash_over_seed`), 75 characters:

- rule 6: `tag_bytes` **is** `TAG_HASH` → no reason.
- rule 8: `report.kind` is `Entr`, not `Unknown` → no reason.
- rule 9: 75 ∈ `VALID_STR_LENGTHS` → no reason.
- rule 10: the arm `InspectKind::Entr if tag_bytes == TAG_ENTR` fails its guard;
  `Preimage` and `Mnem` do not match; `_ => {}` → no reason.

`reasons` is empty → `emit_text` prints **"OK: would decode v0.1"**, while
`ms decode` on the same bytes returns `TagKindMismatch`. The same holds for id
`hash` over a `0x02` mnem payload (77 chars → "OK: would decode v0.2").

The mirror direction (id `entr` over a `0x03` payload) IS caught, by the Preimage
arm — the asymmetry is the defect. `inspect` is the verb an operator points at a
suspicious plate, and §1 rule 2 is the rule that makes "no misread converts one
kind into the other" a property a test can pin; here the tool claims the forged
plate would decode. No plan test covers it: `inspect_reports_the_kind_with_no_false_reason`
inspects only a *valid* preimage single.

Fix: keep rule 6 as a kind-aware check (e.g. push `tag-kind-mismatch` whenever
`tag_bytes != kind_expected_tag(report.kind)`, outside the rule-10 match), and add
the two forged rows to `hashlock_other_verbs.rs`.

### I-1 (Important) — no task applies ms-cli's `=0.8.0` pin, so the workspace does not resolve from Task 3 through Task 10

The hand-wire script carries

    edit("crates/ms-cli/Cargo.toml", [
        ('version = "0.17.1"', 'version = "0.18.0"'),
        ('ms-codec = { path = "../ms-codec", version = "=0.7.0" }', '… version = "=0.8.0" }'),
    ])

but no task's Step names it. Task 3 S4 applies the **ms-codec** Cargo fragment
(which bumps `0.7.0 → 0.8.0`), and Task 11 S1 says *"Task 2/5 fragments already
bumped them; confirm"* — Task 2 is payload/envelope/decode/encode/inspect and
Task 5 is argv_guard + error.rs; neither touches a Cargo.toml. Because ms-cli
pins `version = "=0.7.0"` on a path dependency, cargo refuses to resolve the
workspace the moment ms-codec becomes 0.8.0, so Task 3 S4's `cargo test -p
ms-codec` (Expected: "PASS, six tests") fails at resolution, and every task
through Task 10 is blocked until the implementer works it out. The gate could not
see this: it applies all fragments before building.

Fix: name the ms-cli Cargo.toml fragment in Task 3's Step 4 (both crates bump in
the same step) and correct Task 11 S1's attribution.

### I-2 (Important) — `all_undefined_prefix_bytes_rejected` breaks at `0x03`, and Task 2 Step 5's prescribed remedy does not fit it

`crates/ms-codec/tests/forward_compat.rs:44-63`:

    for prefix in 1u8..=255 {
        if prefix == 0x02 { continue; }
        let mut data = vec![prefix];
        data.extend_from_slice(&entropy);            // 16 bytes → 17-byte payload
        …
        matches!(err, Error::ReservedPrefixViolation { got } if got == prefix),

At `prefix = 0x03` the wired `dispatch_payload` returns
`PreimageLengthMismatch { got: 16 }`, so the assertion fails and
`cargo test -p ms-codec` is red. Task 2 Step 5 says:

> For each, change the asserted byte to `0x01` (still unallocated) … `cargo test
> -p ms-codec` then passes whole.

There is no "asserted byte" to change in a `1u8..=255` loop — the fix is a second
`continue` (or an explicit `PreimageLengthMismatch` expectation for `0x03`), and
the plan never names this test. Note also that the step's grep can only find it
via `ReservedPrefixViolation`: measured, the repository contains **zero** literal
`0x03` occurrences (`grep -rn "0x03" crates/` = empty), so half the grep pattern
matches nothing anywhere.

### I-3 (Important) — `cmd/split.rs:131`'s `_ =>` over `PayloadKind` is not swept

Spec §3 (SILENT) and §9 item 3 are categorical: *"every `_ =>` arm over `Payload`,
`PayloadKind` or `InspectKind`"*, *"`_ => <value>` arms as much as `_ =>
unreachable!`"*. Measured, ms-cli has 18 value-returning `_ =>` arms; four are in
`inspect`'s verdict path and the plan edits all four. The fifth over one of the
three types is `crates/ms-cli/src/cmd/split.rs:127-132`, still present after
wiring:

    let (kind, language): (&'static str, Option<&'static str>) = match payload.kind() {
        PayloadKind::Entr => ("entr", None),
        PayloadKind::Mnem => ("mnem", Some(…)),
        // PayloadKind is #[non_exhaustive]; guard against future kinds.
        _ => ("unknown", None),
    };

Neither the File Structure table nor any task mentions `cmd/split.rs`, and the
plan's committed census test counts only `_ => unreachable!`, so nothing fails
when this arm is left behind. It is unreachable through today's CLI (§5: split has
no ms1 source, F-468), which is presumably why it was missed — but that is exactly
the argument §3 rejects, and the plan's self-review claims the sweep is done
("§3 → Tasks 2, 8 (the catch-all sweep…)", plan:3523).

### I-4 (Important) — `--random --out` is `path.exists()` + a truncating write, not `create_new`

Spec §4.1: *"**Under `--random`, `--out` refuses to overwrite** (`create_new`;
exit 64 naming the existing file)."* The plan (plan:2383-2391):

    pub(crate) fn write_artifact_create_new(path: &std::path::Path, body: &str) -> Result<()> {
        if path.exists() { return Err(CliError::Usage(…)); }
        write_artifact(path, body)
    }

`write_artifact` is `mnemonic_io_lib::write::write_private`, a truncating create.
The function's NAME and its doc ("REFUSES an existing path … instead of
truncating") claim `O_CREAT|O_EXCL` semantics the body does not implement: between
the `exists()` and the open, a file created at that path is truncated — the exact
loss adversarial C-2 was folded to prevent. `path.exists()` also returns false for
a dangling symlink, whose target is then created/followed. The fix is one line
(`OpenOptions::new().write(true).create_new(true).mode(0o600)`).

### I-5 (Important) — `argv_candidates` does not call `looks_like_ms1`; the anti-drift mechanism §4.3 specifies is half-built

Spec §4.3: *"The shape test is ONE function that both the argv guard and the phrase
channels call — `pub(crate) fn looks_like_ms1(raw: &str)` … so the two cannot
drift. **The normalisation is part of the predicate, not of its callers** … 
`argv_candidates` stops pre-folding and calls the same function."* (R0 r0 tests C-1.)

The plan's Task 5 fragment adds `looks_like_ms1` as a wrapper over
`is_ms1_shaped`, and touches `argv_candidates` **only to add a comment**:

    ("fn argv_candidates(token: &str) -> Vec<String> {\n    let norm = |s: &str| s.trim().to_ascii_lowercase();",
     "fn argv_candidates(token: &str) -> Vec<String> {\n    // The fold and trim now live in `looks_like_ms1` as well, …\n    let norm = |s: &str| s.trim().to_ascii_lowercase();")

Verified on the wired copy: `argv_candidates` keeps its own `norm` closure, the
guard still reaches `is_ms1_shaped` through `material_class`, and `looks_like_ms1`
has exactly one caller — `hashlock_phrase::validate_phrase`. Behaviour is identical
today (both paths trim, lowercase and strip separators), so no test can see it;
the property the spec bought with C-1 — one function, one normalisation — is not
delivered, and the fragment's own wording ("as well") records the divergence.

### I-6 (Important) — §11's `--hex` at 63 and 65 characters, upper and lower case, is dropped

§11 Sources: *"`--hex` at 63, 64 and 65 characters, upper and lower case"*. The
plan has no such test: `grep` for 63/65 in the plan finds only the corpus's
65-character *phrase* row and the phrase rule's `[..63]` slice. The only `--hex`
length row anywhere is `hashlock_negative_content.rs`'s `"abcd"`, which asserts
non-echo and nothing else. This is the coverage that would have caught I-9.

### I-7 (Important) — §11's entr-32 source refusal and the "seed backup" wording are dropped

§11 Sources: *"entr and mnem strings refused with the seed-backup wording and
**entr-32 specifically** (the colliding length; tests I-2), kind 3 accepted"*.
The plan's only wrong-kind CLI row is in `hashlock_negative_content.rs`:

    &["hashlock", "-"], b"ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f", …, "wrong ms1 kind",

which is **50 characters** (measured) — an entr-16 string, not the 75-character
entr-32 the finding is about — and the test asserts only that the input is not
echoed. Nothing asserts the spec's refusal text *"that is a seed backup, not a
hashlock preimage"*, and no mnem row exists. The colliding-length case, which is
the whole reason §1 gives preimage singles their own id, is untested at the CLI.

### I-8 (Important) — §11's terminal-prompt test is dropped, with no note

§11: *"`--hashlock-phrase-stdin` at a terminal prints the prompt line."* The
behaviour is implemented (T6 `read_phrase_stdin`, `IsTerminal`), but no test
exists and the plan does not say the row was considered. It needs a pty, so the
honest options are a small pty harness or an explicit "not testable here, covered
by acceptance" note — the plan does neither, so the row is silently absent.

### I-9 (Important) — `--hex` refusals that fail inside `parse_hex_entropy` name neither §8i nor "32 bytes (64 hex characters)"

Spec §4.1: `--hex` *"exactly 32 bytes (64 hex characters); anything else is
refused naming the composer spec's §8i, in both spellings"*, and L8 binds the dual
spelling to *every* refusal that names the preimage's size. The plan's own
message does both — but only on the length branch:

    let bytes = crate::cmd::encode::parse_hex_entropy(raw.trim())?;
    if bytes.len() != 32 { … "exactly 32 bytes (64 hex characters) -- see the composer spec's §8i" … }

An odd-length or non-hex value never reaches that branch: `parse_hex_entropy`
(`encode.rs:271-283`) returns first, with `ms encode`'s wording — *"expected
even-length hex (one byte = 2 chars); got 63 chars"*, or *"expected hex of length
32/40/48/56/64 chars (got empty input)"*, which names a set of entropy lengths
that is wrong for this verb. A 63-character paste is the likeliest real
mistyping, and it is the one that gets the wrong copy.

### I-10 (Important) — `--hashlock-phrase -` silently derives from the literal one-character phrase `-`

Every other secret channel in `ms` treats `-` as the stdin sentinel, and the argv
guard is built around that (`if value.trim() != "-"` at `argv_guard.rs:285` and
`:345`, so `--hashlock-phrase -` is neither refused by layer 1 nor moved into
`ADMITTED`). The plan's `derive` then falls through:

    match crate::argv_guard::admitted("--hashlock-phrase") {
        Some([first, ..]) => …,
        _ => Zeroizing::new(args.hashlock_phrase.as_deref().unwrap_or("").as_bytes().to_vec()),
    }

`admitted` is `None`, so the phrase becomes the byte `-`: exit 0, a valid record,
a plate, and an X that is not the operator's. The spec leaves `--hashlock-phrase -`
undefined (§4.1 gives the phrase exactly two channels), so this is a behaviour the
plan invents on an input the spec is silent about. It reads as an operator ruling —
refuse `-` on this flag naming `--hashlock-phrase-stdin`, or accept it as a literal
phrase — but the plan should not settle it by omission.

### M-1 (Minor) — the plan's index says decode.rs gets "two `Payload::Preimage` arms"; Task 8 and the script deliver one

File Structure (plan:116): "`crates/ms-cli/src/cmd/decode.rs` | two `Payload::Preimage`
arms", and Task 0's closing parenthetical (plan:449): "The remaining fragments —
`decode.rs`'s **second arm** and its `emit_preimage` …". Task 8's Files line and
the committed script both do the opposite and explain why: "ONE early-return
`Payload::Preimage` arm in the first match -- the second match is then unreachable
for the kind and keeps its catch-all". Verified: `decode.rs`'s first match is over
`&payload` and the second over `payload` by value, so the early return makes
`:112` unreachable. An implementer following the index would add a dead arm that
must invent a `Zeroizing<Vec<u8>>` for a preimage — and 32 bytes is a legal
entropy length, so that arm would render a preimage as 24 WORDS if it ever ran.

### M-2 (Minor) — `payload_wire_bytes`'s doc comment is falsified and not updated

`envelope.rs:229-230` still reads: *"`Payload` is a closed 2-variant enum within
this crate … so the match is exhaustive."* After Task 2 it is three variants. The
plan updates both `discriminate`/`dispatch_payload` prefix tables but not this one.

### M-3 (Minor) — the three `/dev/null` gates become two tests, on an empty pipe

§6 and §11 both say one test each for `--hashlock-phrase`, `--hex` and the
positional. The plan merges hex and positional into
`admitted_hex_and_positional_do_not_read_stdin`, and all of them use
`.write_stdin("")` rather than `/dev/null`. The coverage is real; the shape is not
what §11 asks for, and a merged test reports one failure for two channels.

### M-4 (Minor) — the downgrade row lives only in the plan's gate

§8 lists it among the corpus rows (*"the row that proves §9's Rust half"*), and
MIGRATION's text points at it. The plan implements it as `plan-build-gate-ms.sh`
step 6 — stronger at plan time, since it builds the pre-H1 binary — but the corpus
JSON has no downgrade entry and no shipped test re-runs it, so after H1 nothing
re-proves that a 0.7 reader refuses a 0.8 plate.

### M-5 (Minor) — the CI run-by-name step's primary spelling needs nextest, which that job does not have

Task 4 S4 writes `cargo nextest run -p ms-codec --locked -E
'test(hashlock_repro_three_ways)'` and hedges: *"(`cargo-nextest` is installed in
that job already if the job uses it; otherwise …)"*. Measured: `.github/workflows/rust.yml`
line 118-119, the whole `test-ms-codec` job is `run: cargo test -p ms-codec`. The
condition resolves to the fallback; the plan leaves the implementer to discover a
fact one grep answers, in a step whose entire job is to prove a test ran.

### M-6 (Minor) — the 64-hex guard does not use `hex::FromHex`

§4.3: *"The 64-hex guard uses the same hex predicate as `--hex`'s parser
(`hex::FromHex`, `encode.rs:283`, which accepts both cases)"* (R0 r0 tests I-6).
T6 uses `s.bytes().all(|b| b.is_ascii_hexdigit())`. Equal today; the "same
predicate" property is not structural, which is the same class as I-5.

### M-7 (Minor) — a test doc comment describes behaviour the test does not have

`record_line_shape_is_what_me_sysw_pack_reads` (plan:3192-3194): *"Skipped only if
`me` is not installed -- and then it SAYS so on stderr and still passes the shape
check."* The body never invokes `me`, never skips and never writes to stderr. It
is a pure shape check; the comment should say so.

### M-8 (Minor) — the toolkit's flag-coverage lint is discovered, not named

Task 10 S3 ends with a grep to find the lint's name rather than the command to run
it. §10 makes passing it part of the lockstep, so it is a gate whose invocation the
plan does not carry.

### M-9 (Minor) — `reason_text("unexpected-string-length")` still enumerates only entr and mnem

`inspect.rs:210-212` renders *"([50,56,62,69,75] entr / [51,58,64,70,77] mnem)"*.
The plan updates `unknown-tag` and `non-zero-prefix` in the same match but leaves
this one, so an inspect failure on a preimage-length string names two sets that no
longer describe the kinds.

### N-1 (Nit) — the `Zeroizing` pin is a type ascription, not `static_assertions`/`trybuild`

§3 names those two tools; `preimage_field_is_zeroizing` uses
`let _: &Zeroizing<[u8; 32]> = z;` inside a `#[test]`, which the compiler enforces
just as well. Property delivered, mechanism different.

### N-2 (Nit) — the sha256 warning is re-cased

§7: *"This is the brainwallet construction…"*; the card prints
"WARNING: this is the brainwallet construction…". Deliberate and harmless; noted
because §7 is a verbatim-copy section.

### N-3 (Nit) — the plan's STATUS says "ten runs to green"; the record says eleven

Plan:6 — *"ten runs to green"*. The commit message for `36d314d` enumerates
r1–r11, the gate log is `.tmp/ms-gate-r11.log`, and r11 (the `Zeroizing` type-pin
test the self-review had claimed but not written) is a substantive round. Also
`preimage_random` maps getrandom's error away (`map_err(|_| RandomnessUnavailable)`)
where §2 says it *"returns its error"*; the fail-closed intent is met, so this is
recorded rather than filed.

---

## The corpus's `…` cells, and the placeholder scan (brief item 5)

**Judgement: sufficient for provenance, NOT sufficient for byte-identical host/device
rows — two of the ten rows are not yet inputs.**

What is adequate: the instruction names both external tools, the anchor row is
complete and its command lines are pasted verbatim into `provenance`, and every
row's four values (`hardened_x/h`, `sha256_x/h`) are specified. `hashlock_repro.rs`
independently re-derives the anchor in CI. An implementer can produce and prove
the numeric cells.

What is not: two rows specify their PHRASE as a description rather than a value —

    { "phrase": "<64 printable ASCII characters, none of them all-hex>", "phrase_chars": 64, … }
    { "phrase": "<65 printable ASCII characters>", "phrase_chars": 65, … }
    { "phrase": "<100 printable ASCII characters>", … }
    { "phrase": "<101 printable ASCII characters …>", … }

The derivation is a function of the exact bytes, and §8 requires these rows to be
LOCKSTEP rows the fork's pin test drives in both directions (100 and 101 by name).
An implementer choosing their own 100-character string produces a corpus the Go
port must then copy rather than verify against, and two implementers produce two
different corpora. The four bracketed phrases must be literal in the plan. (The
same holds, less sharply, for the `refusals` rows written as
`"<the kind[0].ms1 string, grouped by 5 with spaces>"` — those are derivable from
`kind[0].ms1` mechanically, so they are fine.)

Placeholder grep across the plan: no `TBD`, no `TODO`, no "handle edge cases".
Two soft spots beyond the corpus: `"similar"`/`"appropriate"` do not appear, but
Task 10 S3's flag-coverage lint (M-8) and Task 10's `gen_man.rs` step ("only if
the man page is not generated from clap (check: …)") are steps whose command is a
discovery rather than an action. The plan's own placeholder scan (plan:3539-3542)
is stale in one respect: it flags `derive_share` in `forge_shares` "to be
confirmed", but gate run r5 already replaced it with `interpolate_at` and the plan
text now reads `Codex32String::interpolate_at` — the self-review paragraph was not
updated with the fold.

## H0 (brief item 6) — the gate is concrete on the fork half, prose on the `me` half

Task 11 Step 1 reads:

> 1. `git -C /scratch/code/shibboleth/seedhammer log --oneline -1 origin/main` and
>    the H0 fork merge SHA: the flashed device's version line reads `bg<that sha>`
>    and `sysw.Classify` on the corpus's `kind[0].ms1` is NOT `ClassCodex32Secret`
>    (the fork's H0 test names it).
> 2. `me`'s H0 commit: `me sysw pack` fed the same string refuses it as inert (not
>    `RecordKind::Ms`), on a `me` built at the bump.
> If either is missing, STOP: the release waits (spec §9, §12.7).

Half 1 can stop a release: it names a command, a comparison an operator can make
by eye against the device's version line, and a specific string with a specific
expected classification. It leans on a test that does not exist yet ("the fork's
H0 test names it"), which is fine — H0 is another plan — but the step should say
that the fork test's NAME goes here once H0 lands, or it becomes unfalsifiable.

Half 2 is prose. There is no command: "on a `me` built at the bump" names no
repository path, no build invocation, no `me` SHA to record, and "refuses it as
inert (not `RecordKind::Ms`)" names an internal enum rather than an observable —
`me sysw pack`'s exit code and message are what an operator can check. Compare the
downgrade row in the gate (step 6), which builds a specific tree, feeds a specific
string, and asserts an exit code and a text: that is the shape half 2 needs. As
written, an implementer can satisfy it by believing H1b shipped.

Both halves also lack the artefact the rest of the plan is careful about: §12.7 is
an acceptance item, and Task 11 S3 sends items 1–6 to
`design/agent-reports/ms-hashlock-H1-acceptance.md` while item 7's evidence is
only "paste the outputs into the release commit message".

## Interface consistency across tasks (brief item 4)

Checked every type, function and constant a later task consumes against its
definition: `Payload::Preimage(Zeroizing<[u8; 32]>)` (T2 → T7, T8, tests),
`PayloadKind::single_tag(self) -> Tag` (T2 → decode/encode fragments),
`hashlock::preimage_random() -> Result<Zeroizing<[u8; 32]>>` (T3 → T7),
`CliError::Usage(String)` → 64 (T5 → T7's `pick_source`, `refuse_method`,
`write_artifact_create_new`), `emit_preimage(&[u8; 32], bool) -> Result<u8>` (T8
→ decode's first match and combine's arm; both matches are over `&payload`, so
`&Zeroizing<[u8;32]>` deref-coerces to `&[u8;32]` correctly),
`write_artifact_create_new(&Path, &str) -> Result<()>` (T7; `CliError`/`Result`
are already imported at `out.rs:17`), `looks_like_ms1(&str) -> bool` `pub(crate)`
(T5 → T6), and `payload_entropy_and_language` returning `Result<(…)>` with `?` at
`verify.rs` and `derive.rs` and `.unwrap()` at its five unit-test call sites (T8).
**No mismatch found.** The one interface the plan names but never wires is the
`From<ms_codec::Error>` mapping — C-1.

## Executability spot-checks (brief item 3)

Five citations resolved at this SHA, all real: `argv_guard.rs:67` `SUBCOMMANDS:
[&str; 12]`, `:86` `SECRET_FLAGS: [&str; 4]`, `:104` `argv_candidates`, `:134`
`is_ms1_shaped`, `:256` `override_applies`, `:378` `flag_class` (six of six);
`ms-codec/src/lib.rs:47` `pub mod error;` (single occurrence, matching the
anchor); `error.rs:61-65` + Display at `:202`; `cmd/decode.rs:107` and `:112`, the
two `unreachable!` arms; `.github/workflows/rust.yml:109` = `test-ms-codec:`.

RED steps: T1 S2, T2 S2, T3 S2 and T5 S2 all describe failures that would really
occur (missing items / unknown subcommand). T8 S2's Expected is right on all four
verbs. **Two Expected lines are wrong** — T2 S5's "`cargo test -p ms-codec` then
passes whole" (I-2) and T3 S4's "PASS, six tests" (I-1, the workspace will not
resolve).

Prose vs script: the plan's Task 0 python block is a proper prefix of the
committed script (diff: the script additionally carries the `out.rs`, `combine.rs`,
`payload_lang.rs`, `inspect.rs` ×2, `ms-codec/inspect.rs`, `verify.rs`, `derive.rs`
and `emit_preimage` entries, exactly as Tasks 7 and 8 say they append them). Every
fragment shown in Tasks 7 and 8 is byte-identical to the script's string. The only
prose/script divergence is M-1's "second arm".

---

## Counts

**2 Critical · 10 Important · 9 Minor · 3 Nit**

Critical: C-1 (no CLI mapping for the three new codec errors — §1 rule 2's refusal
becomes `unhandled ms_codec::Error variant` at exit 1), C-2 (`ms inspect` reports
"OK: would decode" for a string `ms decode` refuses).

Not GREEN. Both Criticals are small fixes (three `From` arms; a kind-aware rule 6)
and both need a test that does not exist yet, which is what let them through.
