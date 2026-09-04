# SPEC — `ms hashlock`: the hashlock preimage as an ms1 kind (`0x03`)

**Goal.** Give a miniscript `sha256(H)` hashlock a *backup*. Today the operator
invents a phrase, hashes it by hand, and the 32-byte preimage the script will
one day demand exists only in their head or in a shell history. This spec adds
(1) a derivation with a name — phrase → preimage, by one of two methods the tool
states so it can be written down outside the tool; (2) an ms1 string that
carries the preimage on a metal plate, with its own kind byte and its own
four-character id so the eye can tell it from a seed; and (3) `ms hashlock`, the
verb that produces both plus the `hash:` record the composer consumes.

**Base SHA.** mnemonic-secret `master` `7fc1e58` (ms-codec 0.7.0, ms-cli
0.17.1). Every line citation in §14 was re-grepped at that SHA while this spec
was written; re-grep at implementation time (CLAUDE.md citation-decay).

**SemVer.** ms-codec 0.7.0 → **0.8.0**, ms-cli 0.17.1 → **0.18.0**. Both MINOR
and additive: no existing string changes bytes, and a 0.7 reader rejects a
`0x03` string as a bad prefix rather than misreading it (§9).

**Scope, locked by the brainstorm's rulings.** `design/BRAINSTORM_hashlock_phrase.md`
in mnemonic-engrave carries L1–L23 verbatim; this spec is their consequence and
does not re-open them. The ones that shape every section: **L4** the hardened
method is PBKDF2-HMAC-SHA256, 100,000 iterations, fixed salt, dkLen 32; **L5**
the operator chooses between it and plain sha256; **L6** the preimage gets its
own prefix byte; **L12** sha256 warns always and never refuses; **L13** no
`--salt` this cycle; **L14** preimage singles carry the id `hash`; **L20** `--in`
means the ms1, and the phrase has exactly two channels; **L21** `--random`
refuses unless the preimage reaches a persistent channel; **L22** the classifier
lands Rust-first and no new record class ships this cycle.

**Design substrate.** Two independent review rounds over the brainstorm, both
folded and each fold sonnet-verified: `hashlock-brainstorm-R0-r0-crypto-bitcoin-expert.md`
(cryptography + Bitcoin lens, 1C/6I/6M/2N) and
`hashlock-brainstorm-R0-r2-security-software-expert.md` (security software
engineering lens, 4C/6I/7M/3N), both in mnemonic-engrave's
`design/agent-reports/`. Findings that changed the design are cited inline by
their number.

**This spec is H1 only.** H1b (`me`'s classifier), H2 (the fork's Go port and
the device's phrase screen), H3 (records) and H4 (the device walk) are named in
the brainstorm's §4.1 and get their own plans; the composer spec fold is a
separate artifact under its own R0.

---

## §1. Wire format — kind `0x03`, id `hash`

The payload is `[0x03][X:32]` — 33 bytes, so the string is **75 characters**,
the same length as an entr-32 single.

`consts.rs` gains `PREIMAGE_PREFIX: u8 = 0x03`, beside the existing
`RESERVED_PREFIX = 0x00` (entr — the name is historical) and `MNEM_PREFIX =
0x02`. `0x01` stays UNALLOCATED, as MIGRATION.md already records.

**Length no longer implies kind, and that is new.** entr and mnem never share a
string length (50/56/62/69/75 against 51/58/64/70/77), so until now the length
alone identified the kind. The preimage is the first kind that collides: 75
characters, exactly entr-32. It also shares entr's first payload character —
`0x00`, `0x02` and `0x03` agree in their top five bits, so all three encode a
leading `q` (measured: an entr-32 single is `ms10entrsqqqqq…`). Two kinds that
differ only in a bit no human can see is a reading hazard, and it is why L14
gives preimage singles their own id:

| kind | single string begins | length |
| --- | --- | --- |
| entr-32 | `ms10entrsq…` | 75 |
| preimage | `ms10hashsq…` | 75 |

So the plate says which instrument it is, at the fourth through seventh
characters, where a person actually looks. **Readers still dispatch on the
prefix byte** — the id is a human affordance, never a parse input.
`RESERVED_ID_BLOCKLIST` gains `hash` (five entries today: `entr`, `seed`,
`xprv`, `mnem`, `prvk`), so a share set can never draw an id that impersonates a
preimage single.

**A `0x03` payload whose length is not 33 bytes is refused before the variant is
built**, with a new `Error::PreimageLengthMismatch { got: usize }` (review I-2).
Two reasons, and the second is the sharp one: the entr length error would name a
legal entr length as illegal, and the obvious `data[1..33]` indexing **panics**
on a short payload. The check precedes construction; the variant is built with
`<[u8; 32]>::try_from(&data[1..])`, never slice indexing. BIP-93's bracket
admits 16..46 payload bytes, so 16, 32, 34 and 46 are all reachable and each is
a vector row (§8).

**The share axis is untouched.** Threshold and index live in the codex32 header,
orthogonal to the prefix byte; a K-of-N set of a preimage recovers to a `0x03`
payload and the codec supports it. (The CLI has no ms1 source for `ms split`
today — pre-existing, filed F-468, out of scope here.)

**Misreads cannot convert one kind into another.** The brainstorm records that
the `entr` and `hash` codewords are at least nine characters apart while BIP-93
corrects at most four. **MEASURE THIS AT PLAN TIME** rather than inheriting it:
the plan's build gate computes the distance between the two id codewords in the
bech32 alphabet and asserts it exceeds twice the correction bound. A claim about
error correction that no command checks is exactly the class this constellation
keeps finding late.

---

## §2. Derivation — `ms_codec::hashlock`

A new module in **ms-codec**, not in the CLI, so the kind and the rule that
fills it share one crate, one corpus and one SHA pin, and the Go port pins its
provenance against a single version (L10).

```
pub const HASHLOCK_SALT: &[u8] = b"ms-hashlock-v1";
pub const HASHLOCK_ITERATIONS: u32 = 100_000;
pub const HASHLOCK_DKLEN: usize = 32;

pub fn preimage_hardened(phrase: &[u8]) -> Zeroizing<[u8; 32]>;  // L4
pub fn preimage_sha256(phrase: &[u8]) -> Zeroizing<[u8; 32]>;    // L5
pub fn digest(preimage: &[u8; 32]) -> [u8; 32];                  // sha256(X)
```

`digest`'s output is **not** zeroized: H is public the moment the policy is
engraved, and wrapping it would imply a secrecy the value does not have.

**The salt is fixed and has no flag** (L13). `"ms-hashlock-v1"` is ASCII, short
enough to copy by hand, and domain-separated from BIP-39's `"mnemonic"`
(different PRF role, iteration count and dkLen) and from `me`'s 16-byte random
seal salt (different length, so `S || INT(i)` can never coincide). The cost is
recorded rather than hidden: one precomputation table over common phrases breaks
every ms1 hashlock ever made, which is why the sha256 warning names a generator
and `--random` exists. `--salt` is filed as F-469 for a later cycle; **changing
the salt after any vector ships is a new method, not a parameter change.**

**Dependencies, spelled exactly as `me` spells them** so the constellation has
one lockfile shape: `pbkdf2 = { version = "0.12", default-features = false,
features = ["hmac"] }` and `sha2 = "0.10"`. No direct `hmac`, no
`password-hash`. Both are pure Rust with no build script and are already trusted
in this constellation by `me`'s sealed payload.

**Measured, reproduced in two independent tools while this spec was written**
(phrase `correct horse battery staple`, 28 bytes):

| method | X (preimage) | H (digest) |
| --- | --- | --- |
| hardened | `c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016` | `3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12` |
| sha256 | `c4bbcb1fbec99d65bf59d85c8cb62ee2db963f0fe106f483d9afa73bd4e39a8a` | `b867db875479bcc0287352cdaa4a1755689b8338777d0915e9acd9f6edbc96cb` |

`python3 hashlib.pbkdf2_hmac` and `openssl kdf -keylen 32 -kdfopt digest:SHA256
-kdfopt pass:… -kdfopt salt:ms-hashlock-v1 -kdfopt iter:100000 PBKDF2` agree
byte-for-byte on the hardened X. The sha256 H is the value the S4 device walk
(W-5) recorded, which is what makes this pair the corpus's anchor row.

---

## §3. ms-codec API (0.8.0)

```
pub enum PayloadKind { Entr, Mnem, Preimage }
pub enum Payload { Entr(..), Mnem { .. }, Preimage(Zeroizing<[u8; 32]>) }
```

**The variant wraps `Zeroizing<[u8; 32]>`, not a bare array** (review M-1). A
bare `[u8; 32]` has no `Drop`, so it is memcpy'd on every move and the crate's
documented caller-wrap recipe — the one `Payload::Entr` relies on — cannot reach
it. The wrapper scrubs on drop and keeps the length rule structural: a value of
this variant cannot be the wrong length.

Arms are added in `dispatch_payload` (`envelope.rs:192`), `payload_wire_bytes`
(`envelope.rs:231`), `validate`, and the `InspectKind` projection.
`Error::ReservedPrefixViolation` stops firing for `0x03`; **any test that pinned
`0x03` as reserved flips**, and the plan enumerates those tests mechanically
rather than by reading.

### `#[non_exhaustive]` is a hazard here, not a help

`Payload` is `#[non_exhaustive]` (`payload.rs:29`), so downstream `match`
statements carry a catch-all and **the compiler will not tell anyone a variant
was added.** ms-cli has exactly four such arms — measured at `7fc1e58`,
`grep -rn '_ => unreachable' crates/ms-cli/src` returns 4:

| site | reached from | disposition |
| --- | --- | --- |
| `cmd/decode.rs:107` | `ms decode` | **functional**: print kind, preimage hex, digest |
| `cmd/decode.rs:112` | `ms decode` | **functional**: same |
| `cmd/combine.rs:166` | `ms combine` | **functional**: a recovered preimage share set prints as `decode` does (review N-1) |
| `cmd/payload_lang.rs:61` | `ms verify`, `ms derive` | **typed refusal** with the remedy `ms hashlock <ms1>` |

The split is by *what the verb is for*, not by convenience (r1 verification,
finding 1): a verb that READS a secret prints a preimage as a preimage; a verb
that needs a SEED refuses it. Without the arms, all four panic at runtime on the
first `0x03` string — `unreachable!` is a promise the new variant breaks.

**`payload_lang.rs:61` is reached before any refusal a verb might add**, so the
refusal must sit there and not in `verify`/`derive` (review I-3). §5 states it
again from the verb's side because it is the easy one to get wrong.

---

## §4. `ms hashlock` (ms-cli 0.18.0)

### §4.1 Sources — exactly one per invocation

| source | meaning |
| --- | --- |
| `--hashlock-phrase TEXT` | the phrase, on argv; joins `SECRET_FLAGS`, so refused without `--allow-argv-secret` (§6) |
| `--hashlock-phrase-stdin` | the phrase, on stdin; a phrase file is redirected into it |
| `--hex HEX`, or `--hex -` | an existing X, exactly 32 bytes (64 hex characters) |
| `<ms1>`, `-`, or `--in FILE` | a preimage-kind ms1, to re-derive H from a plate |
| `--random` | 32 bytes from the OS CSPRNG (`getrandom`, failing closed) |

Two sources is exit 64. **The phrase has exactly these two channels** (L20);
there is no third, and an ms1-shaped value handed to either is refused naming
`--in`/`-`.

`--in FILE` means **the ms1**, as it does on the six reading verbs (L20). This
is not a free choice: the argv guard's own remedy for a refused plate string
prints `ms hashlock --in FILE`, so that channel must accept the plate or the
guard would advertise a route that does not exist (review C-1).

**`--random` refuses (exit 64, naming `--out`) unless `--out FILE` or `--json`
is given** (L21, r2 review C-3). With the card suppressed or redirected, the
digest would reach a payload while its preimage existed nowhere — data loss,
which gates. `--out`'s overwrite semantics are unchanged from the 2026-08-26
ruling.

### §4.2 Method (L5)

`--method hardened` (default) or `--method sha256`, **for the phrase sources
only**. `--hex`, `--random` and `<ms1>` supply X directly, so `--method` with
any of them is **refused at exit 64** (r2 review M-6: a flag the operator set
that does nothing is a defect); the card's method line reads `preimage
supplied`, and `--json` omits the `method` key.

The default is announced, never assumed: it appears on the card and in `--json`
whether or not the flag was given.

### §4.3 The phrase rule — identical on host and device

Non-empty, **printable ASCII only**, at most **100 characters**, and the bytes
are used **exactly as typed**: no trimming, no case folding, no Unicode
normalisation. The cap is a usability bound, not a security one — no entropy is
gained past the HMAC block boundary.

`HASHLOCK_PHRASE_MAX_CHARS = 100` is **its own constant on each side**, pinned
in lockstep (review M-6). It must not be bound to the device's
`passphrase.MaxLen`, which is a plate-legibility number that can move for its
own reasons.

Two refusals that exist because the alternative is a silently different X:

- **A phrase of exactly 64 characters, all hex digits, is refused**, naming
  `--hex` (review I-6). It is a preimage pasted into the wrong slot, and
  deriving from it produces a valid-looking record for a different X.
- **An ms1-shaped phrase is refused** on both channels, naming `--in`/`-`
  (r2 review C-1), reusing `argv_guard::is_ms1_shaped` — which is a private
  `fn` today (`argv_guard.rs:134`) and becomes `pub(crate)` — so the two
  predicates cannot drift. That function already strips display separators
  before testing, which is why a grouped plate string is caught too.

**The phrase channels use a new byte-verbatim reader**: bytes as given, exactly
one trailing `\r?\n` stripped, nothing else. They must NOT use
`parse::read_input` (strips all whitespace plus `-` and `,`) or
`parse::read_phrase_input` (trims and collapses runs) — either silently changes
X while every codec vector still passes (r2 review I-3). `read_stdin_passphrase`
(`parse.rs:139`) already has the right stripping shape and is the model.

With stdin at a terminal, `--hashlock-phrase-stdin` prints one prompt line to
stderr — `Type the hashlock phrase, then Enter.` — rather than blocking
silently (r2 review M-7; the constellation's recorded `mt` finding, where a
tool's first interaction looked like a hang).

Refusals name the rule and **never echo the phrase**.

### §4.4 Outputs

- **stdout**: one line, `hash:<64 hex>` — the record `me sysw pack --in -`
  consumes. Public, so no stdout advisory. **`--out` never suppresses it**
  (r2 review I-5). `ms encode` suppresses its stdout artifact under `--out`
  because both channels carry the same secret; here they carry *different*
  artifacts, and copying that shape would hand `me sysw pack` an empty stream.
- **`--out FILE`**: the preimage as an ms1 string, mode `0600`, overwriting.
  `--out` is the preimage's channel; stdout is the digest's.
- **stderr card** (suppressed by `--no-engraving-card`), whose **first line
  names it as carrying the preimage** (r2 review M-1). This verb inverts `ms`'s
  usual polarity — stdout public, stderr secret — so `2>>log` or `2>&1 | tee`
  lands a preimage in a `0644` file and nothing else on the stream would say so.
  Then: the digest; the `sha256=` operand for `md compose --path`; the preimage
  as grouped ms1 (`--group-size`/`--separator` apply) and as hex; the **method
  line**, verbatim and copyable, e.g.
  `preimage = PBKDF2-HMAC-SHA256(password = phrase, salt = "ms-hashlock-v1", iterations = 100000, dkLen = 32)`
  or `preimage = SHA-256(phrase)`; the phrase's **character count** beside it
  (review M-2 — the one signal that makes a stray space visible); the composer spec's §8i and
  F-132 lines; the reuse lines of §7; the method's warning; and the source kind
  **without its value**.
- **`--json`**: `digest`, `hash_record`, `sha256_operand`, `preimage_hex`,
  `preimage_ms1`, `source`, `method` (`{kdf, hash, salt, iterations, dklen}` or
  `{hash}`, omitted for supplied preimages), `phrase_chars`. It carries the
  secret, so the `PrivateKeyMaterial` advisory fires, as `encode --json` does.

---

## §5. The other verbs on the new kind

| verb | behaviour |
| --- | --- |
| `decode` | prints kind, preimage hex and digest. **Never words** — a preimage is not entropy |
| `inspect` | reports the kind |
| `combine` | a recovered preimage share set prints as `decode` does |
| `derive`, `verify` | **refuse**, with the executable remedy `ms hashlock <ms1>` |
| `encode --hex` | unchanged: stays `entr`, so **`ms hashlock` is the only door that creates the kind** |
| `split` | the codec supports shares of the kind and a test pins it; the CLI has no ms1 source (F-468, out of scope) |

The `derive`/`verify` refusal **sits on the `Ok((tag, payload))` arm before the
shared `payload_entropy_and_language` helper** (review I-3). Today that helper's
`_ => unreachable!` (`payload_lang.rs:61`) would panic first, so a refusal
written into the verb bodies would be dead code that never runs.

---

## §6. The argv guard

`--hashlock-phrase` joins `SECRET_FLAGS` (`argv_guard.rs:86`, four entries
today: `--phrase`, `--hex`, `--ms1`, `--passphrase`), which is matched on raw
argv **before clap** — the placement that matters, because a guard downstream of
the parser has already lost (a lesson this constellation recorded when clap
echoed a secret in a usage error before the refusal ran).

Joining it is a **four-part edit**, and each part has a symptom if skipped
(reviews I-1, I-2, M-3):

1. `SUBCOMMANDS` (`argv_guard.rs:67`) grows from `[&str; 12]` to `[&str; 13]`,
   so the refusal and the purge pattern name `hashlock`.
2. `override_applies`'s verb match, so `--allow-argv-secret` actually works on
   this verb.
3. `flag_class`, so the refusal says "a hashlock phrase" and not "a BIP-39
   passphrase".
4. The verb's `Source` is built `.on("--hashlock-phrase")`, so an admitted value
   arrives through the side channel rather than from whatever stdin holds.
   **The gate for this one:** the same invocation with stdin at `/dev/null`
   still derives from the flag's value.

---

## §7. Copy that carries weight

Two warnings, and neither refuses (L12):

- **`--method sha256`, always, at every length**: *"This is the brainwallet
  construction: anyone holding the digest tests 10^10 phrases per second. A
  phrase a person chose is not safe here; use six diceware words or --random."*
- **`--method hardened`, under 20 characters**: *"a 20-character phrase falls in
  about 72 days on one GPU; choose it from a generator"*, and the tool proceeds.

Neither floor can see a dictionary phrase, so **the copy is the defence**, and
it names the remedy rather than gesturing at one.

The reuse lines, on the card and (H2) in the device's confirm modal — the tool
never sees other policies or other passwords, so it cannot detect reuse and the
copy is the whole defence (review I-5):

> One phrase per policy. Spending any path of a wsh wallet publishes this
> digest. Never use this phrase as a passphrase or a password anywhere else — a
> spend publishes the preimage, and anyone can then test guesses at the phrase
> itself.

The method line carries its own instruction: *write this next to your phrase; it
is on no plate; if the method line is lost, try each method that shipped with
the version named on this card.* Not "try both" — that phrasing outlives its own
precondition the day a third method ships (review M-4).

Under `--random` the card says **both halves** (review M-5): *"No phrase exists,
so nothing can be guessed, and nothing can be remembered. This plate is the only
copy."*

---

## §8. Vectors and the corpus

The corpus SHA is re-pinned, which is what forces the minor bump.

**Kind rows.** Encode/decode/inspect for `0x03`; the share round trip through
the codec API; id `hash` on singles and its blocklist entry; length rows for
`0x03` payloads of **16, 32, 34 and 46** bytes, each refused by name (BIP-93's
bracket reaches 16..46).

**Derivation rows, both methods**, each pinning **X and H** (r2 review N-1 — a
row that pins only H cannot tell a wrong X from a wrong digest): the anchor
phrase of §2; one character; 20 characters; 64 and 65 (the HMAC block
boundary); 100 and 101; a phrase with leading, trailing and doubled spaces; the
64-hex refusal; a non-ASCII refusal; empty.

**A test executes the `python3` and `openssl kdf` reproductions and FAILS if
either tool is absent** — a skip that prints ok is the false-PASS shape this
project refuses. That test lives in ms-codec and runs in the `test (ms-codec)`
job, which is Ubuntu-only; **that job gains a preflight step**
(`openssl kdf --help`, `python3 -c 'import hashlib'`) so a missing tool fails
the step rather than a test someone can `#[ignore]` (r2 review I-6: the ms-cli
matrix includes macOS, whose stock `openssl` is LibreSSL and has no `kdf`).

**Lockstep rows for H2**: a 100-character phrase derives byte-identically on
host and device; a 101-character one is refused on both; the 64-hex refusal on
both.

---

## §9. MIGRATION.md — a 0.7 → 0.8 section

1. Readers that dispatch on the prefix byte **MUST** treat `0x03` as a 32-byte
   preimage and never as entropy.
2. **Length no longer implies kind** (§1), and singles of the kind carry the id
   `hash`.
3. Every downstream crate **MUST** sweep its `_ => unreachable!` arms over
   `Payload`, because `#[non_exhaustive]` means the compiler will not.
4. **The pre-tool recipe this project documented everywhere** — the composer
   spec's §8i, the W-5 walk, F-465: *"hash the passphrase to 32 bytes, then hash
   again"* — is `--method sha256`, **NOT the default**. A digest made by hand
   before 0.18.0 reproduces only with that flag (review M-3). The same note goes
   in the manual chapter and in F-465's `Which hash?` hint.

Older readers, including every SH2 flashed before H2, reject a `0x03` string as
a bad prefix (`ReservedPrefixViolation` in Rust, `errMSBadPrefix` in Go — both
traced by the review), so **the failure mode is a refusal and never a seed.**

---

## §10. Lockstep, SemVer and provenance

- ms-codec **0.8.0** (new kind, `hashlock` module, corpus SHA re-pinned,
  MIGRATION section); ms-cli **0.18.0** (the verb, the new arms, the guard
  edit). ms-cli's dependency pin on ms-codec moves to `=0.8.0`.
- The manual chapter
  `mnemonic-toolkit/docs/manual/src/40-cli-reference/43-ms.md` moves in
  lockstep, and the toolkit's flag-coverage lint must pass.
- **Rust-primary.** The fork's `0x03` arm and its derivation (H2) carry a
  provenance pin to the ms-codec 0.8.0 commit, and the hashlock vector corpus is
  vendored into the fork with a pin test, exactly as the compose vectors are.
  No normative behaviour is decided in Go.

---

## §11. Tests

Beyond §8's corpus:

**CLI.** The argv guard for `--hashlock-phrase` (refused without the allow flag;
the value never echoed); stdin stripping of exactly one LF or CRLF; `--in FILE`;
`--hex` at 63, 64 and 65 characters; entr and mnem strings refused and kind 3
accepted; `--random` twice gives two different records; two sources exit 64;
`--random` without `--out`/`--json` exits 64 naming `--out`; `--method` with a
supplied preimage exits 64; stdout is exactly the record line; `--out` is `0600`
and overwrites; the card's contents per method, including both `--random` halves
and §7's reuse lines; the `--json` schema and its advisory; `decode`, `inspect`
and `combine` on the kind; `derive` and `verify` refusing with the remedy;
**one test per `unreachable!` site that panics on 0.17.x**; MSRV, clippy, fmt;
the man page carries the verb.

**Review gates.** The plan's tests lens mutates these before any code is
written, and the whole-diff review's mutation pass proves each guard fails on
its named mutation with the output pasted. A guard whose named mutation does not
fail it is not a guard.

---

## §12. Acceptance

H1 is done when, on a clean checkout at the merge commit:

1. `ms hashlock --hashlock-phrase-stdin < phrase.txt` prints exactly
   `hash:3cf5d421…4c12` on stdout for the anchor phrase under the default
   method, and the card on stderr names the preimage on its first line.
2. `ms hashlock --hashlock-phrase-stdin --method sha256 < phrase.txt` prints
   `hash:b867db87…96cb` — the value the W-5 walk recorded by hand — and always
   carries the brainwallet line.
3. `--out X.txt` writes a 75-character `ms10hashsq…` string at mode `0600`, and
   `ms hashlock --in X.txt` re-derives the same digest.
4. `ms decode` on that string prints the kind, the preimage hex and the digest,
   and **never** words; `ms derive` and `ms verify` on it refuse with the
   remedy; none of the four `unreachable!` sites panics.
5. `ms hashlock --random` without `--out` or `--json` exits 64 naming `--out`.
6. The `hash:` line feeds `me sysw pack --in -` unmodified, and the composer's
   `Which hash?` payload route offers the record.
7. The corpus SHA pin, MIGRATION section, CHANGELOG, manual chapter and both
   version bumps are in the same merge.

---

## §13. Out of scope

hash160/ripemd160/hash256 on the host (the composer composes sha256 only, and
ripemd160 cannot be a preimage derivation in any case: every miniscript hash
fragment demands a 32-byte X via `OP_SIZE 32 EQUALVERIFY`, and ripemd160 yields
20); a preimage plate or a preimage as a source **on the device** (L7 — the
device is digest-only this cycle); any non-ASCII phrase; K-of-N shares of a
preimage from the CLI (F-468); an operator-chosen salt (F-469, L13).

---

## §14. Citations — measured at `7fc1e58`, re-grep at implementation time

| claim | site |
| --- | --- |
| four `_ => unreachable!` arms over `Payload` | `cmd/payload_lang.rs:61`, `cmd/decode.rs:107`, `cmd/decode.rs:112`, `cmd/combine.rs:166` (`grep -rn '_ => unreachable' crates/ms-cli/src` = 4) |
| `Payload` is `#[non_exhaustive]` | `crates/ms-codec/src/payload.rs:29` |
| `PayloadKind` has exactly `Entr`, `Mnem` | `crates/ms-codec/src/payload.rs:10-15` |
| prefix dispatch and wire projection | `crates/ms-codec/src/envelope.rs:192` (`dispatch_payload`), `:231` (`payload_wire_bytes`), `:216` (`ReservedPrefixViolation`) |
| `RESERVED_PREFIX` = `0x00`, `MNEM_PREFIX` = `0x02` | `crates/ms-codec/src/envelope.rs:114-115` |
| `RESERVED_ID_BLOCKLIST`, five entries | `crates/ms-codec/src/consts.rs:71` |
| `Error::ReservedPrefixViolation { got }` | `crates/ms-codec/src/error.rs:62`, rendered `:202` |
| `SECRET_FLAGS`, four entries | `crates/ms-cli/src/argv_guard.rs:86` |
| `SUBCOMMANDS: [&str; 12]` | `crates/ms-cli/src/argv_guard.rs:67` |
| `is_ms1_shaped`, private, strips display separators first | `crates/ms-cli/src/argv_guard.rs:134-142` |
| singles carry `Tag::ENTR` | `crates/ms-cli/src/cmd/encode.rs:200` |
| one trailing LF/CRLF stripped | `crates/ms-cli/src/parse.rs:139-148` (`read_stdin_passphrase`) |
| advisory classes | `crates/ms-cli/src/advisory.rs:53` (`OutputClass`) |
| an entr-32 single is `ms10entrsq…`, 75 characters | measured with the shipped `ms` while writing this spec |
| the anchor derivation values | measured in `python3 hashlib` and `openssl kdf`, §2 |
