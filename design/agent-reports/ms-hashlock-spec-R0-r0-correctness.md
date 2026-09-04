# R0 round 0 — `SPEC_ms_hashlock.md`, correctness + internal-consistency lens

**Artifact.** `mnemonic-secret/design/SPEC_ms_hashlock.md` at master `5ba61ca763804f89d27e4a551ba2117d5f2979db`.
**Tree reviewed against.** `7fc1e58` (`git diff --stat 7fc1e58 5ba61ca` = the spec file alone, 510 insertions — the two trees are otherwise identical, so every citation below is equally true of master).
**Input.** `mnemonic-engrave/design/BRAINSTORM_hashlock_phrase.md` (L1–L23, sections 4.1–4.6).
**Lens.** Is every normative statement true of the code it cites and consistent with every other statement, and does the spec carry L1–L23 and the agreed 4.x sections without loss or drift?
**Not reviewed here.** Cryptographic design (adversarial lens); test sufficiency (tests-vector lens); the brainstorm's KDF-rate figures.

**Method note.** Three claims were settled by execution rather than reading, in a `cp -r` copy of the repo at `/scratch/r0-correctness-probe` (two throwaway integration tests, `crates/ms-codec/tests/r0_probe_hash_tag.rs` and `r0_probe_lengths.rs`; nothing committed, the original tree untouched). Their raw output is quoted verbatim in C-1 and I-1.

---

## Table 1 — Rulings L1–L23 → spec

| # | ruling (short) | lands in | verdict |
| --- | --- | --- | --- |
| L1 | Home and order: `ms` owns the host command, Rust first, the device is a behaviour-faithful Go port | front-matter "This spec is H1 only"; §10 "**Rust-primary.** … No normative behaviour is decided in Go" | CARRIED |
| L2 | Terminology: hashlock phrase / preimage X / digest H; flags `--hashlock-phrase*` | Goal ¶; §4.1 flag table; used consistently throughout — the only occurrences of "passphrase" are code identifiers (`read_stdin_passphrase`, `passphrase.MaxLen`) or the deliberate contrast in §7's reuse line | CARRIED |
| L3 | Iteration cap 100,000; the tool must STATE the method for an external backup | §2 `HASHLOCK_ITERATIONS = 100_000`; §4.4 method line; §7 "write this next to your phrase; it is on no plate" | CARRIED (the 1/10th-signer margin is rationale, not restated — correct, per the brief's no-re-derive instruction) |
| L4 | PBKDF2-HMAC-SHA256, 100,000, fixed salt, dkLen 32 | §2 constants block | CARRIED |
| L5 | Two methods, the operator's choice; card and `--json` always name the method | §4.2; §7 | CARRIED |
| L6 | The preimage gets its own kind byte; 32 bytes only; `decode` never words; the device never offers it as a seed | §1 (`PREIMAGE_PREFIX = 0x03`, `[0x03][X:32]`); §5 `decode` row "**Never words**"; §13 + §9 for the device half | CARRIED |
| L7 | Device scope this cycle = digest only | §13 "a preimage plate or a preimage as a source **on the device** (L7 — the device is digest-only this cycle)" | CARRIED |
| L8 | **"Every refusal and help line says '32 bytes (64 hex characters)'"** | §4.1 table row only | **NOT CARRIED as a rule** — see I-5 |
| L9 | 4.2's `ms hashlock` surface agreed as a form | §4 in full | CARRIED (sentence-level in Table 2) |
| L10 | Kind + derivation live in ms-codec; ms-cli's pin moves to `=0.8.0` | §2 ¶1; §10 bullet 1 (verified: `crates/ms-cli/Cargo.toml:20` is `version = "=0.7.0"` today) | CARRIED |
| L11 | Review before anything else (r0 crypto lens) | "Design substrate" ¶ cites the report by filename and counts | CARRIED |
| L12 | sha256 warns always, never refuses; hardened keeps the 20-char warning | §7 "Two warnings, and neither refuses (L12)" | CARRIED |
| L13 | No `--salt` this cycle; F-469 | §2 "The salt is fixed and has no flag (L13) … `--salt` is filed as F-469" | CARRIED |
| L14 | Preimage singles carry id `hash`; readers dispatch on the prefix; blocklist gains `hash` | §1 table + "**Readers still dispatch on the prefix byte**" + `RESERVED_ID_BLOCKLIST` ¶ | CARRIED in text — **but unimplementable as specified**, see C-1 |
| L15 | No scrub discipline for the phrase or X **on the device** | — | N/A to H1 (device leg, H2). §3's host-side `Zeroizing<[u8; 32]>` is a separate, non-conflicting r0 M-1 fold |
| L16 | 4.4 (the device leg) agreed | referenced only where H1 must lockstep: §4.3 `HASHLOCK_PHRASE_MAX_CHARS`, §7 "(H2) in the device's confirm modal", §8 lockstep rows | N/A to H1, correctly scoped |
| L17 | 4.5 process and homes agreed | front-matter "H1b …, H2 …, H3 … and H4 … get their own plans; the composer spec fold is a separate artifact under its own R0" | CARRIED in part — see M-5 |
| L18 | A second lens (r2 security-software) before ruling on 4.6 | "Design substrate" ¶ cites the r2 report and its counts | CARRIED |
| L19 | Pause before spec | — | N/A (lifted by the operator; the spec exists) |
| L20 | `--in` means the ms1; the phrase has exactly two channels; an ms1-shaped phrase is refused on both | §4.1 "**The phrase has exactly these two channels** (L20)"; §4.3 ms1-shape refusal (verified: exactly six shipped verbs give `--in` ms1 meaning — `decode.rs:42`, `inspect.rs:43`, `verify.rs:40`, `derive.rs:47`, `repair.rs:101`, `combine.rs:56`; `encode.rs:72` and `split.rs:68` use `--in` for a phrase/hex source, so "the six reading verbs" is exact) | CARRIED |
| L21 | `--random` refuses unless `--out FILE` or `--json`; `--out` overwrite semantics unchanged | §4.1 ¶3 | CARRIED (see the operator note at the end) |
| L22 | The classifier lands Rust-first as H1b; the fork mirrors it; no new class | front-matter names H1b; §9 ¶5; §10 bullet 3 | CARRIED by reference (H1b/H2 detail correctly out of this spec) |
| L23 | 4.6 (testing) stands with the r2 additions | §8 + §11 | CARRIED in part — see I-6 and M-3 |

**Not carried: 1 of 23 (L8).** Two (L15, L19) are N/A to H1; two (L17, L23) are partial and are itemised in Table 2.

---

## Table 2 — Brainstorm 4.2 / 4.3 / 4.5 / 4.6 normative sentences → spec

### 4.2 `ms hashlock` (L9)

| brainstorm sentence | lands in |
| --- | --- |
| `--hashlock-phrase TEXT` joins `SECRET_FLAGS`, refused without `--allow-argv-secret` | §4.1 row 1; §6 ¶1 |
| `--hashlock-phrase-stdin`; one trailing LF/CRLF stripped; a phrase file is redirected into it | §4.1 row 2; §4.3 ¶5 |
| These are the ONLY phrase channels (L20) | §4.1 ¶2 |
| `SUBCOMMANDS` `[&str; 12]` → 13 so the refusal and purge pattern name `hashlock` | §6 item 1 |
| `override_applies`'s verb match | §6 item 2 |
| `flag_class` says "a hashlock phrase" | §6 item 3 |
| `Source` built `.on("--hashlock-phrase")`; the stdin-at-`/dev/null` gate | §6 item 4 |
| tty stdin prints one stderr prompt, "Type the hashlock phrase, then Enter." | §4.3 ¶6 |
| `--hex HEX` / `--hex -`: exactly 32 bytes (64 hex characters) | §4.1 row 3 |
| "…anything else is **refused naming §8i**" | **DROPPED** (M-4) |
| `<ms1>`, `-`, `--in FILE`: a preimage-kind ms1; `--in` means the ms1 (review C-1 rationale) | §4.1 row 4 + ¶3 |
| "An entr or mnem string is refused: *that is a seed backup, not a hashlock preimage*" | behaviour in §11; **the quoted copy DROPPED** (M-4) |
| `--random`: 32 bytes from the OS CSPRNG (`getrandom`, failing closed) | §4.1 row 5 — but see I-2 |
| `--random` card says both halves | §7 ¶5 |
| `--random` REFUSES unless `--out FILE` or `--json` (L21) | §4.1 ¶3 |
| Method for the phrase sources only; `--method` with `--hex`/`--random`/`<ms1>` refused at exit 64; card reads "preimage supplied"; `--json` omits `method` | §4.2 ¶1 |
| Default `hardened`, announced on the card and in `--json` | §4.2 ¶2 |
| sha256 → the brainwallet line always; hardened → the 72-days line under 20 characters; neither refuses | §7 ¶¶1–3 |
| Phrase rule: non-empty, printable ASCII, ≤100, dedicated `HASHLOCK_PHRASE_MAX_CHARS` on each side, bytes exactly as typed | §4.3 ¶¶1–2 |
| Refusals name the rule and never echo the phrase | §4.3 ¶7 |
| A 64-character all-hex phrase is refused naming `--hex`, host and device | §4.3 bullet 1 (+ "identical on host and device" in the heading) |
| An ms1-shaped phrase is refused on both channels naming `--in`/`-`, reusing `argv_guard::is_ms1_shaped` | §4.3 bullet 2 |
| A NEW byte-verbatim reader; never `read_input` / `read_phrase_input` | §4.3 ¶4 |
| No entropy past 64 characters; the cap is a usability bound | §4.3 ¶1 |
| stdout `hash:<64 hex>`; public, no stdout advisory; `--out` never suppresses it | §4.4 bullet 1 — but see I-3 |
| `--out FILE`: the preimage ms1, 0600, overwriting | §4.4 bullet 2 |
| stderr card: first line names the preimage; digest; `sha256=` operand; grouped ms1 + hex; the method line + "write this next to your phrase…"; the character count; §8i and F-132; the 3.7 lines; the method's warning; the source kind without its value | §4.4 bullet 3 + §7 (all twelve items present) |
| `--json`: digest, hash_record, sha256_operand, preimage_hex, preimage_ms1, source, method, phrase_chars; PrivateKeyMaterial advisory | §4.4 bullet 4 — but see I-4 |
| Other verbs: decode / inspect / derive / verify / combine / `encode --hex` / split | §5 table (six rows) |
| Versions 0.8.0 / 0.18.0 | §10 bullet 1 |

### 4.3 The preimage kind in ms1 (L10)

| brainstorm sentence | lands in |
| --- | --- |
| Wire `[0x03][X:32]`, 33 bytes, 75 characters | §1 ¶1 (75 confirmed by probe) |
| Length no longer implies kind; 50/56/62/69/75 vs 51/58/64/70/77 | §1 ¶3 (matches `consts.rs:33` and `:43` exactly) |
| First payload character is `q` because 0x00/0x02/0x03 share their top five bits | §1 ¶3 (confirmed by probe: `ms10hashsqz…` / `…sq2…` / `…sqw…` for 0x00/0x02/0x03) |
| Preimage singles carry id `hash`; the plate reads `ms10hash…`; readers dispatch on the prefix; `RESERVED_ID_BLOCKLIST` gains `hash` | §1 ¶¶3–4 — see C-1 |
| No misread converts one into the other (codewords ≥9 apart, BIP-93 corrects ≤4) | §1 ¶6, **converted into a plan-time measurement** — a strengthening, not a drift |
| A `0x03` payload of any length but 33 is refused BEFORE construction; `PreimageLengthMismatch`; `try_from` | §1 ¶5 |
| The share axis is untouched; a K-of-N set recovers to a `0x03` payload | §1 ¶4 (verified: `combine_shares` calls `dispatch_payload(&data)` at `shares.rs:321` with no tag or string-length gate and returns `Tag::ENTR` — the claim is sound) |
| `Payload::Preimage(Zeroizing<[u8; 32]>)`; matching `PayloadKind`/`InspectKind`; arms in `dispatch_payload`, `payload_wire_bytes`, `validate` | §3 ¶¶1–2 — the enumeration is incomplete in both documents, see C-1 and I-7 |
| `ReservedPrefixViolation` stops firing for `0x03`; pinned tests flip; machine-checked at plan time | §3 ¶2 |
| `non_exhaustive` is a hazard; four `unreachable!` arms; `verify.rs:99` / `derive.rs:434` reach the last one first; split by what the verb is FOR | §3 ¶3 + table + ¶5 (verified: `grep -rn 'payload_entropy_and_language' crates/ --include=*.rs` outside `payload_lang.rs` returns exactly `derive.rs:434` and `verify.rs:99`, both on the `decode()` `Ok` arm — the "reached only from `verify` and `derive`" claim is TRUE) |
| Derivation in the codec: `ms_codec::hashlock` with named constants | §2 |
| `pbkdf2` / `sha2` spelled exactly as `me` spells them | §2 ¶4 (verified byte-for-byte against `me-cli/Cargo.toml:45-46`) |
| ms-cli pin `=0.8.0`; `ms hashlock` a thin verb | §10 bullet 1; §2 ¶1 |
| Vectors: round trips, share round trip, inspect kind, both methods pinning BOTH X and H, the python3 + openssl reproductions RUN, length rows 16/32/34/46, lockstep rows, the CI preflight step, corpus SHA re-pinned | §8 in full — but the 16..46 bracket is wrong, see I-1 |
| MIGRATION.md 0.7 → 0.8 section, four items, plus "older readers refuse, never a seed" | §9 items 1–4 + ¶5 (`MIGRATION.md:23` confirms `0x01`/`0x03` are claimable-unallocated) |

### 4.5 Process and homes (L17)

| brainstorm sentence | lands in |
| --- | --- |
| Two specs: `SPEC_ms_hashlock.md` under its own R0 with correctness, adversarial and tests-vector lenses; the composer spec fold under its own R0 with a journey lens | front-matter ¶5 (this spec's half); the lens list is carried by the dispatch brief, not the spec |
| Plans, one per stage, each build-gated and re-validated immediately before its implementer; **H1 gets a `plan-build-gate-ms.sh`** | **DROPPED** (M-5) |
| The order: ms spec R0 → H1 plan R0 → one implementer → whole-diff review → fold → sonnet verification → merge through the staging ritual → release per `RELEASE_PROCESS.md` (**corpus SHA pin, CHANGELOG, MIGRATION, publish dry run, both tags**), manual chapter in lockstep → H1b → … | §12 item 7 carries the corpus SHA pin, MIGRATION, CHANGELOG, manual chapter and both version bumps; §10 carries the manual lockstep + the toolkit flag-coverage lint. **The publish dry run and both tags are DROPPED** (M-5) |
| Tiers and reports (opus / sonnet / not fable; every agent writes its own report) | correctly out of a spec — process rule, lives in CLAUDE.md |
| Rust-primary pins: the fork's arm and derivation pin to the ms-codec 0.8.0 commit; the corpus is vendored into the fork with a pin test | §10 bullet 3 |

### 4.6 Testing (L23) — H1 rows only (H1b/H2/H3/H4 are out of this spec by design)

| brainstorm row | lands in |
| --- | --- |
| Kind `0x03` encode/decode/inspect | §8 "Kind rows" |
| The share round trip through the codec API | §8 |
| Length rows 16/32/34/46 each refused by name | §8 — see I-1 |
| id `hash` on singles and its blocklist entry | §8 |
| Derivation rows both methods: W-5 phrase, 1 char, 20, 64, 65, 100, 101, leading/trailing/doubled spaces, the 64-hex refusal, a non-ASCII refusal, empty | §8 "Derivation rows" (all eleven present) |
| The reproduction test FAILS if `python3` or `openssl kdf` is absent | §8 ¶3 |
| Argv guard (refused without the allow flag; the value never echoed) | §11 |
| stdin stripping of exactly one LF or CRLF | §11 |
| `--in FILE`; `--hex` at 63/64/65; entr and mnem refused, kind 3 accepted | §11 |
| `--random` twice gives two records; two sources exit 64; stdout is exactly the record line; `--out` is 0600 and overwrites | §11 |
| Card contents per method incl. both `--random` halves and the 3.7 lines; `--json` schema and advisory | §11 |
| `decode`, `inspect`, `combine` on the kind; `derive`/`verify` refuse with the remedy | §11 |
| One test per `unreachable!` site that panics on 0.17.x; MSRV, clippy, fmt; the man page carries the verb | §11 |
| The toolkit manual's flag-coverage lint passes | §10 (not §11 — carried, relocated) |
| Review gates: the plan's tests lens mutates them; the whole-diff mutation pass pastes the output | §11 "Review gates" |
| r2: `--random --no-engraving-card` / `--random 2>/dev/null` without `--out`/`--json` exits 64 naming `--out` | §11 (compressed to one row — both variants subsumed) |
| r2: `--allow-argv-secret` derives from the flag value; the same with stdin at `/dev/null`; `< other.txt` never derives from the file | §6 item 4 ("**The gate for this one**") |
| r2: the guard's refusal text says "hashlock" and "a hashlock phrase" | §6 items 1 and 3 |
| r2: the `--json` `method` shape per source | §11 (subsumed by "the `--json` schema") |
| r2: the CI preflight step in `test (ms-codec)` | §8 ¶3 |
| r2: **byte-exact rows `"  a  b "` and `"a-b,c"` through BOTH phrase channels** | **DROPPED from §11** (I-6) |
| r2: **an ms1-shaped phrase on both phrase channels refused naming the ms1 route** | **DROPPED from §11** (I-6) |
| r2: **the downgrade row (a `0x03` string on the 0.17.x-equivalent codec refuses, never panics)** | **DROPPED from §11** (I-6) |
| r2: **NEGATIVE-CONTENT rows, one per refusal (nine listed): the phrase and preimage appear in neither stdout, stderr, nor the `--json` error envelope** | **DROPPED from §11** (M-3) |
| r2: "stdout is exactly the record line" runs WITH `--out` as well | §4.4 states the behaviour; the qualifier is dropped from §11's row (folded into M-3's note) |

---

## Findings

### C-1 (Critical) — `decode()`'s tag accept-set is never widened, so no preimage single can ever be read back

**Spec, §1 (line 62-68):**

> it is why L14 gives preimage singles their own id:
> | preimage | `ms10hashsq…` | 75 |

**Spec, §12 acceptance items 3 and 4:**

> 3. `--out X.txt` writes a 75-character `ms10hashsq…` string at mode `0600`, and `ms hashlock --in X.txt` re-derives the same digest.
> 4. `ms decode` on that string prints the kind, the preimage hex and the digest, and **never** words; `ms derive` and `ms verify` on it refuse with the remedy

**Spec, §3 (line 162-163)** — the complete list of edit sites the spec names:

> Arms are added in `dispatch_payload` (`envelope.rs:192`), `payload_wire_bytes` (`envelope.rs:231`), `validate`, and the `InspectKind` projection.

**The code, `crates/ms-codec/src/decode.rs:75-106`** (SPEC §4 rule 6):

```
    // §4 rule 6: tag must be in the v0.2 accept set (currently {entr}).
    let payload = match *tag.as_bytes() {
        x if x == TAG_ENTR => { … }
        _ => {
            return Err(Error::UnknownTag {
                got: *tag.as_bytes(),
            });
        }
    };
```

`decode()` runs the tag accept-set test *after* `envelope::discriminate`, so a well-formed 75-character `ms10hashsq…` string that carries a `Payload::Preimage` is refused at `decode.rs:101` with `UnknownTag`. Nothing in the spec adds `hash` to that set, and the arm is a `_ =>` catch-all on `[u8; 4]` — **the compiler emits no error and no warning**, which is exactly the silent-absorption failure mode §3 and §9 item 3 warn about, one level outside where they look.

**Measured** (`/scratch/r0-correctness-probe`, `crates/ms-codec/tests/r0_probe_hash_tag.rs`, `cargo test -p ms-codec`), encoding under the *existing* `0x00` prefix so only the id is under test:

```
ENCODED = ms10hashsqz46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kw948dm43kh3yc  len=75
DECODE  = Err(Error("unknown tag "hash"; not a member of RESERVED_TAG_TABLE"))
```

The encoder has no such gate (`encode.rs:16-28` refuses only `RESERVED_NOT_EMITTED_V01` = `seed`/`xprv`/`prvk`, and `hash` is in the codex32 alphabet), so `ms hashlock --out` emits a plate that `ms` itself cannot read — breaking the very invariant `encode.rs`'s own doc comment exists to hold ("preventing a v0.1 ms-codec from emitting a string that v0.1 ms-codec itself cannot decode").

**Blast radius, all silent:**
- §12 items 3 and 4 both fail; a preimage engraved on metal is unreadable by the tool that made it.
- §4.1's `<ms1>` / `-` / `--in FILE` source cannot work, so L20's ruling (`--in` means the ms1) is unimplementable and the argv guard's own remedy — `ms hashlock --in FILE`, the thing review C-1 of the r2 round was raised to fix — advertises a route that still does not exist.
- §5's `decode` and `combine` rows: `decode` fails; `combine` is unaffected (`combine_shares` returns `Tag::ENTR` and never consults the accept-set), so the two disagree.
- §3's and §5's central design claim — that the `derive`/`verify` refusal must sit at `payload_lang.rs:61` "before any refusal a verb might add" — is **defeated for singles**: `decode()` returns `Err` first, `verify.rs:99`'s `Ok((_tag, payload))` arm is never taken, and the operator gets `unknown tag "hash"` instead of the executable `ms hashlock <ms1>` remedy. (`verify.rs:105` has a bespoke arm for `ReservedTagNotEmittedInV01` and none for `UnknownTag`.)

**Also missing from §3's list, but LOUD** (exhaustive matches inside the crate, so `cargo build` reports them — named here only so the plan does not treat §3 as complete): `decode.rs:27` `allowed_for_kind`, `payload.rs:93` `kind()`, `payload.rs:102` `as_bytes()`. The pre-dispatch gate `decode.rs:22` `is_known_length` needs no change (75 is already in `VALID_STR_LENGTHS`).

**What the spec must state:** that `hash` joins the rule-6 accept set at `decode.rs:84-105` (or that the rule becomes kind-keyed rather than tag-keyed), with a test that a `hash` single round-trips through `encode` → `decode`.

---

### I-1 (Important) — "BIP-93's bracket admits 16..46 payload bytes, so 16, 32, 34 and 46 are all reachable" is false on both halves

**Spec, §1 (line 81-83):**

> BIP-93's bracket admits 16..46 payload bytes, so 16, 32, 34 and 46 are all reachable and each is a vector row (§8).

**Spec, §8:**

> length rows for `0x03` payloads of **16, 32, 34 and 46** bytes, each refused by name (BIP-93's bracket reaches 16..46).

**The code, `crates/ms-codec/src/codex32/mod.rs:198-205`:**

```
    pub fn from_string(s: String) -> Result<Self, Error> {
        let (name, mut checksum) = if s.len() >= 48 && s.len() < 94 {
            ("short", checksum::Engine::new_codex32_short())
        } else if s.len() >= 125 && s.len() < 128 {
```

An ms1 string is `22 + ceil(8N/5)` characters for an N-byte payload (9 fixed + 13 checksum, per the bijection test at `consts.rs:82-93`), so the short-checksum bracket 48..93 admits **N = 16..44**, not 16..46. **Measured** (`r0_probe_lengths.rs`, `0x03` prefix, id `hash`):

```
payload 16 B -> str len  48  parse OK                          decode: string length 48 outside v0.1 set [50, 56, 62, 69, 75]
payload 32 B -> str len  74  parse OK                          decode: string length 74 outside v0.1 set [50, 56, 62, 69, 75]
payload 33 B -> str len  75  parse OK                          decode: reserved-prefix byte was 0x03, expected 0x00
payload 34 B -> str len  77  parse OK                          decode: reserved-prefix byte was 0x03, expected 0x00
payload 44 B -> str len  93  parse OK                          decode: string length 93 outside v0.1 set [50, 56, 62, 69, 75]
payload 45 B -> str len  94  from_string REJECTS: InvalidLength(94)
payload 46 B -> str len  96  from_string REJECTS: InvalidLength(96)
```

Two consequences:

1. **A 46-byte `0x03` payload cannot exist as an ms1 string at all.** It is a 96-character string, which `Codex32String::from_string` refuses with `InvalidLength(96)` — before any ms-codec rule runs, and on every public entry point (`decode`, `decode_with_correction`, `combine_shares`, `inspect`). The §8 row "46 bytes, refused by name" is unconstructible; the largest reachable payload is 44.
2. **Of the four rows, only 34 reaches `PreimageLengthMismatch` through `decode()`.** 16 and 32 are refused earlier by the pre-dispatch gate at `decode.rs:46` with `UnexpectedStringLength` (their 48- and 74-character strings are outside the union set). They *are* reachable with `PreimageLengthMismatch` through `combine_shares`, which has no string-length gate — but the spec does not say which path each row takes, so "each refused **by name**" names two different errors depending on a route the spec never picks.

Fix: state the real bracket (16..44 for the short checksum), replace the 46 row with 44, and say for each row which entry point it enters by and therefore which error it asserts.

---

### I-2 (Important) — `--random` has no source of randomness in the crate that implements it

**Spec, §4.1 (line 203):**

> | `--random` | 32 bytes from the OS CSPRNG (`getrandom`, failing closed) |

`crates/ms-cli/Cargo.toml` lists `ms-codec`, `mnemonic-io-lib`, `bip39`, `bitcoin`, `clap`, `clap_mangen`, `hex`, `libc`, `serde`, `serde_json`, `zeroize` — **no `getrandom`, no `rand`**, and `grep -rn 'getrandom\|rand::\|OsRng' crates/ms-cli/src/` returns nothing. `getrandom` is a dependency of **ms-codec** (`ms-codec/Cargo.toml`), used only by `shares.rs:43` `random_id()`; ms-codec exports no random-preimage helper, and §2's API block lists only `preimage_hardened`, `preimage_sha256` and `digest`.

So the spec specifies a flag with no stated implementation route. Both plausible resolutions are unstated and are not interchangeable: adding `getrandom` to ms-cli (a new CLI dependency, and the "failing closed" contract then lives in ms-cli), or adding a `ms_codec::hashlock::preimage_random()` to §2's API (which keeps the Rust-primary surface in one crate and makes the Go port's story consistent, and is what §2's own "the kind and the rule that fills it share one crate" argument points to). §10's SemVer table also changes depending on which.

---

### I-3 (Important) — the interaction of `--json` with the stdout record line is unstated, and §11's own test contradicts it

**Spec, §4.4 bullet 1:**

> - **stdout**: one line, `hash:<64 hex>` — the record `me sysw pack --in -` consumes. Public, so no stdout advisory. **`--out` never suppresses it**

**Spec, §11:**

> stdout is exactly the record line

**Spec, §4.4 bullet 4:**

> - **`--json`**: `digest`, `hash_record`, `sha256_operand`, `preimage_hex`, `preimage_ms1`, `source`, `method` …, `phrase_chars`.

The spec says stdout is the record line, says only `--out` cannot suppress it, and separately says `--json` emits eight keys — without ever saying where the JSON goes or what happens to the record line. On the shipped precedent it replaces it: `cmd/encode.rs:218-230` is `if args.json { emit_json(…) } else { emit_text(…) }`, both on stdout, and `cmd/decode.rs:123` has the same shape. Under that reading `--json` *does* suppress the record line, which the spec never licenses and which makes §11's "stdout is exactly the record line" a test that cannot be written for the `--json` case.

This is load-bearing rather than cosmetic because L21 admits `--json` **instead of** `--out` as `--random`'s persistent channel: under `--random --json` the preimage exists only in `preimage_hex`/`preimage_ms1` on stdout, so whether `hash:` is also there decides whether stdout is parseable as a record at all.

---

### I-4 (Important) — `phrase_chars` and the card's character count have no defined value for the three preimage-supplied sources

**Spec, §4.2:**

> `--hex`, `--random` and `<ms1>` supply X directly … the card's method line reads `preimage supplied`, and `--json` omits the `method` key.

**Spec, §4.4:**

> the phrase's **character count** beside it (review M-2 …)
> - **`--json`**: … `method` (`{kdf, hash, salt, iterations, dklen}` or `{hash}`, **omitted for supplied preimages**), `phrase_chars`.

`method` carries an explicit omission rule; `phrase_chars` carries none, and neither does the card's character count. For `ms hashlock --random --json` there is no phrase, so `phrase_chars` is undefined — two implementers will emit `0`, `null`, or omit it, and `0` reads as "an empty phrase was accepted". §11 requires a test of "the `--json` schema"; for three of the five sources there is no schema to test. Same omission rule as `method` is the obvious fix, but the spec must say it.

---

### I-5 (Important) — L8 is not carried

**Brainstorm L8, consequence column (verbatim):**

> Every refusal and help line says "32 bytes (64 hex characters)".

The spec states the dual spelling exactly once, in a source table (§4.1: "an existing X, exactly 32 bytes (64 hex characters)"), and nowhere as a rule binding refusals or help text. §11's `--hex` row pins lengths (63/64/65) but no wording, and §4.3's 64-hex-phrase refusal is specified only as "naming `--hex`". The ruling's universal quantifier — the whole point of L8, which came from the operator asking "Do we mean 64 hex chars or 32?" — has no carrier in the spec and no gate.

---

### I-6 (Important) — three agreed 4.6 H1 test rows are dropped from §11

§11's CLI list is complete as written; the three rows below, all added by the r2 security-software review and all ruled to stand by L23, do not appear in it. (Whether §11 is *sufficient* belongs to the tests lens; that they were agreed and are absent is a traceability defect.)

1. **Byte-exact phrase rows.** Brainstorm 4.6: *"byte-exact rows `"  a  b "` and `"a-b,c"` through BOTH phrase channels — `--hashlock-phrase-stdin`, and `--hashlock-phrase` under `--allow-argv-secret` via the admitted side channel — equal the codec vector (mutation: swap in `read_phrase_input` or `read_input` on either channel — no codec vector can catch it; r3 verification finding 3)."* §11 has only "stdin stripping of exactly one LF or CRLF", which does not exercise interior whitespace, `-` or `,` and does not cover the argv channel — i.e. it is exactly the test that would still pass under the named mutation. This is the row the r3 fold verification singled out, and §4.3 ¶4 is the rule it protects.
2. **The ms1-shaped-phrase rows.** Brainstorm 4.6: *"an ms1-shaped phrase on both phrase channels is refused naming the ms1 route (mutation: delete the shape check on one channel)."* §11 has "entr and mnem strings refused and kind 3 accepted", which is the `<ms1>` **source**, not the phrase channels. §4.3 bullet 2 specifies the behaviour; nothing in §11 tests it, and the mutation the brainstorm names ("delete the shape check on one channel") would survive §11 intact.
3. **The downgrade row.** Brainstorm 4.6: *"the downgrade row (a `0x03` string on the 0.17.x-equivalent codec refuses, never panics)."* This is the row that proves §9's closing claim ("Older readers … reject a `0x03` string as a bad prefix … so **the failure mode is a refusal and never a seed**"). §11's nearest row — "one test per `unreachable!` site that panics on 0.17.x" — is the *opposite* test (it pins the pre-fix panic in ms-cli, not the old codec's refusal), so §9's claim currently has no gate.

---

### I-7 (Important) — the `#[non_exhaustive]` sweep is scoped to `unreachable!` arms, but the silent arms are `_ => <value>` — three of them are in `ms inspect`, and §5 understates what that verb does

**Spec, §9 item 3:**

> Every downstream crate **MUST** sweep its `_ => unreachable!` arms over `Payload`, because `#[non_exhaustive]` means the compiler will not.

**Spec, §3:**

> ms-cli has exactly four such arms — measured at `7fc1e58`, `grep -rn '_ => unreachable' crates/ms-cli/src` returns 4

The measurement is correct, but it defines the hazard by its most visible spelling. `#[non_exhaustive]` hides *every* catch-all, and a `_ =>` arm that returns a value fails **silently** where `unreachable!` at least panics loudly. `grep -rn '_ =>' crates/ms-cli/src --include=*.rs | grep -v unreachable` returns **18**; three of them are matches on `InspectKind` inside `ms inspect`'s would-decode verdict:

```
crates/ms-cli/src/cmd/inspect.rs:180-183   let valid_lengths: &[usize] = match report.kind {
                                               InspectKind::Mnem => VALID_MNEM_STR_LENGTHS,
                                               _ => VALID_STR_LENGTHS,
crates/ms-cli/src/cmd/inspect.rs:190-204   match report.kind { InspectKind::Entr if tag_bytes == TAG_ENTR => …
                                                               InspectKind::Mnem => …
                                                               _ => {}
crates/ms-cli/src/cmd/inspect.rs:229-232   let version = match report.kind { InspectKind::Mnem => "v0.2", _ => "v0.1" };
```

plus, at `inspect.rs:170-178`, the CLI's own copies of rules 6 and 8:

```
            // Rule 6: tag not in accept set.
            reasons.push("unknown-tag");
    if report.kind == InspectKind::Unknown {
        reasons.push("non-zero-prefix");
```

So a valid preimage single reaches `ms inspect` and prints `FAIL: would NOT decode v0.1` with reasons `unknown-tag` and `non-zero-prefix`, and `reason_text` at `inspect.rs:219` renders "prefix byte is not a recognised kind (0x00=entr, 0x02=mnem)". **§5's row `| inspect | reports the kind |` is the only thing the spec says about the verb**, and §3's edit list (`dispatch_payload`, `payload_wire_bytes`, `validate`, the `InspectKind` projection) is entirely codec-side — `ms-cli/src/cmd/inspect.rs` appears nowhere in the spec.

Fix: widen §9 item 3 to "every catch-all over `Payload`, `PayloadKind` or `InspectKind`, not only `_ => unreachable!`", give the plan the measured 18-arm sweep rather than the 4-arm one, and state in §5 or §3 that `ms inspect`'s verdict machinery (`inspect.rs:160-232`, including `reason_text`) gains the kind.

---

### M-1 (Minor) — §14 cites the doc comment, not the definition, for the two prefix constants

**Spec, §14:**

> | `RESERVED_PREFIX` = `0x00`, `MNEM_PREFIX` = `0x02` | `crates/ms-codec/src/envelope.rs:114-115` |

`envelope.rs:114-115` are lines of the `discriminate` doc comment (`/// - `0x00` (`RESERVED_PREFIX`) → `Payload::Entr(rest)``). The definitions are `crates/ms-codec/src/consts.rs:17` and `:39` — which is where §1 correctly says `PREIMAGE_PREFIX` goes ("`consts.rs` gains `PREIMAGE_PREFIX: u8 = 0x03`, beside the existing…"). The claim is true at the cited lines; the citation just points an implementer at the wrong file. Also worth a second row: the same doc-comment text is duplicated at `envelope.rs:186-188` and both copies need the `0x03` line, or they go stale the day the arm lands.

### M-2 (Minor) — "MINOR and additive" understates a source-breaking enum change

**Spec, front matter:**

> **SemVer.** ms-codec 0.7.0 → **0.8.0**, ms-cli 0.17.1 → **0.18.0**. Both MINOR and additive

The version numbers are right (0.x minor *is* the breaking bump in cargo semantics), but `Payload` and `PayloadKind` are `#[non_exhaustive]` (`payload.rs:29`, `:9`) while **`InspectKind` is not** (`inspect.rs:12-20`). Adding `Preimage` to it is source-breaking for any downstream exhaustive match — loud, and therefore safe, but not "additive", and §9's downstream note mentions only `Payload`.

### M-3 (Minor) — the nine-row negative-content matrix is dropped from §11

Brainstorm 4.6 (r2 additions): *"NEGATIVE-CONTENT rows, one per refusal (empty, non-ASCII, over 100, 64-hex, ms1-shaped, `--hex` wrong length, wrong ms1 kind, two sources, `--method` with a given X): the phrase and the preimage appear in neither stdout, stderr, nor the `--json` error envelope on stdout (mutation: a refusal built with `format!("... {phrase}")`)."* §11 carries only "the value never echoed", and only for the argv guard. §4.3 ¶7 states the rule ("Refusals name the rule and **never echo the phrase**") with no gate for the other eight refusals. **Minor, not Important, per the 2026-08-27 operator ruling** — the defect class is material reaching a stream. Reproduction for the follow-up: build any of the nine refusals with the phrase interpolated into its message and observe that no §11 row fails. §11's "stdout is exactly the record line" also loses 4.6's "runs WITH `--out` as well" qualifier, though §4.4 states the behaviour.

### M-4 (Minor) — two agreed 4.2 refusal texts are dropped

(a) *"`--hex HEX` or `--hex -`: an existing X, exactly 32 bytes (64 hex characters); **anything else is refused naming §8i**."* §4.1's row states the accepted shape only; the refusal's content is gone. (b) *"An entr or mnem string is refused: **'that is a seed backup, not a hashlock preimage'**."* The behaviour survives in §11 ("entr and mnem strings refused"); the quoted copy does not. Both are exactly the class §7 argues is load-bearing ("the copy is the defence").

### M-5 (Minor) — 4.5's H1 build gate and two release steps are dropped

*"H1 gets a `plan-build-gate-ms.sh` sibling of the me and md gates on the pinned toolchain"* appears nowhere in the spec, and no other H1 artifact exists yet to carry it — so under the project's own "a fold is authorship and re-earns the gate" rule, the H1 plan currently has no named gate script. §12 item 7 lists the corpus SHA pin, MIGRATION, CHANGELOG, manual chapter and both bumps, but drops 4.5's **publish dry run** and **both tags**.

### M-6 (Minor) — the byte-verbatim reader's model is UTF-8-bounded, so one refusal will not name its rule

**Spec, §4.3:** *"The phrase channels use a new byte-verbatim reader: bytes as given, exactly one trailing `\r?\n` stripped, nothing else. … `read_stdin_passphrase` (`parse.rs:139`) already has the right stripping shape and is the model."*

The stripping shape is exactly right (`parse.rs:139-148` pops one `\n` then one `\r`, and nothing else — verified). But it is built on `read_stdin()` (`parse.rs:150-157`), which is `io::stdin().read_to_string(&mut buf)` — a non-UTF-8 byte fails there with `failed to read stdin: stream did not contain valid UTF-8`, not with the phrase rule's named refusal. §4.3 promises "Refusals name the rule" and §8 wants "a non-ASCII refusal" row; a UTF-8 non-ASCII phrase (`"é"`) reaches the ASCII check and refuses correctly, a raw `0xFF` does not. Since the rule is printable ASCII the outcome is still a refusal, so this is wording plus one vector row, not a behaviour change — but the spec should say whether "byte-verbatim" means `Vec<u8>` or inherits `String`.

### N-1 (Nit) — "the distance between the two id codewords"

**Spec, §1:** *"the plan's build gate computes the distance between the two id codewords in the bech32 alphabet and asserts it exceeds twice the correction bound."* Two 4-character ids can differ in at most 4 positions, so a distance of 9 is not a property of the ids; it is the minimum distance between the two full ms1 **codewords** that differ in the id field (changing the id changes the checksum). Worth spelling out, because the plan has to compute the right thing for the assertion to mean anything — which is the spec's own stated reason for deferring it.

### N-2 (Nit) — the method line's instruction is phrase-only but printed on every source

**Spec, §7:** *"The method line carries its own instruction: *write this next to your phrase; it is on no plate…*"* Under `--hex`, `--random` and `<ms1>` §4.2 says the method line reads `preimage supplied`, and there is no phrase to write anything next to. §7 gives `--random` its own two-half copy but does not say the write-it-down instruction is suppressed for the three supplied-preimage sources.

---

## Note for the operator (not a spec defect — L21 is a ruling)

L21 admits `--json` as an alternative to `--out FILE` for `--random`, on the ground that a preimage reaching no persistent channel is data loss. `--json` writes to **stdout**, which is not persistent: `ms hashlock --random --json > /dev/null` and `ms hashlock --random --json | grep digest` both lose the preimage in exactly the way `--random --no-engraving-card` does, which is the case r2 review C-3 raised. Only `--out FILE` is actually a persistent channel. If the operator wants the rule to bind, `--random` would require `--out`; `--json` alone would remain refused. The spec carries the ruling faithfully as written, so this is recorded for the operator rather than filed as a finding.

**Second note.** Brainstorm 4.5 assigns this spec three lenses ("correctness, adversarial and tests-vector") and the journey lens to the composer spec fold, while the brainstorm's section 7 closing says *"the journey lens belongs to the spec, which will carry the walks."* This spec carries no operator journey — §12 is a runnable acceptance list, not a walk. The two brainstorm sentences disagree; the operator may want to say which binds before the spec closes.

---

## Counts

| severity | count |
| --- | --- |
| Critical | **1** (C-1) |
| Important | **7** (I-1 … I-7) |
| Minor | **6** (M-1 … M-6) |
| Nit | **2** (N-1, N-2) |

**Traceability:** 1 of 23 rulings not carried (L8 → I-5); 2 partial (L17 → M-5, L23 → I-6 + M-3). Of the brainstorm's 4.2/4.3/4.5/4.6 normative sentences, 7 are DROPPED (2 in 4.2 → M-4; 2 in 4.5 → M-5; 3+1 in 4.6 → I-6, M-3) and 0 are stated differently from the brainstorm.

**What execution added that reading did not:** C-1 and I-1 were both invisible to a read of the spec against the spec — C-1 because the blocking arm is a `_ =>` catch-all in a file the spec never names, I-1 because the bracket arithmetic looks plausible until a 46-byte payload is actually built. Both were found in one `cargo test` run.
