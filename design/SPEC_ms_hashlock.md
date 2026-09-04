# SPEC — `ms hashlock`: the hashlock preimage as an ms1 kind (`0x03`)

**Goal.** Give a miniscript `sha256(H)` hashlock a *backup*. Today the operator
invents a phrase, hashes it by hand, and the 32-byte preimage the script will
one day demand exists only in their head or in a shell history. This spec adds
(1) a derivation with a name — phrase → preimage, by one of two methods the tool
states so it can be written down outside the tool; (2) an ms1 string that
carries the preimage on a metal plate, with its own kind byte and its own
four-character id so the eye can tell it from a seed; and (3) `ms hashlock`, the
verb that produces both plus the `hash:` record the composer consumes.

**STATUS: R0 GREEN under lens-closure (2026-09-04).** Round 0 = three opus
lenses (correctness 1C/7I/6M/2N, adversarial 4C/4I/5M/1N, tests-vector
1C/11I/11M/4N), all persisted verbatim and folded in one edit (`1a14a4d`); r1
sonnet fold verification GREEN (28/28 C+I fixed, spot-checks reproduced, no
new contradiction). Lenses run: correctness, adversarial (with the operator
journeys inside it), tests/vectors, fold-verification. The journey lens is
assigned to the composer spec fold by brainstorm 4.5. **Three controller
defaults await the operator and are labelled in place**: §1 rule 2
(`TagKindMismatch`), §4.1 (`--random` requires `--out`; narrows L21), §9 (H0
precedes the 0.18.0 release; reorders 4.5). A veto folds the section back and
re-verifies. Next: the H1 plan.

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

**This spec is H1, plus the prerequisite it turned out to need.** H1b (`me`'s
classifier), H2 (the fork's Go port and the device's phrase screen), H3
(records) and H4 (the device walk) are named in the brainstorm's §4.1 and get
their own plans; the composer spec fold is a separate artifact under its own R0.
**§9 adds H0**: the two existing readers that would take a preimage plate for a
seed — the fork's `isStrictMs1` and `me`'s `validate_record` — must be guarded
and shipped BEFORE ms-cli 0.18.0 is released, because the brainstorm's
"older readers refuse" premise is measured false for both (R0 r0 adversarial
C-3). That reorders 4.5's sequence and is presented to the operator in the fold
record rather than decided here.

---

## §1. Wire format — kind `0x03`, id `hash`

The payload is `[0x03][X:32]` — 33 bytes, so the string is **75 characters**,
the same length as an entr-32 single.

`consts.rs` gains `PREIMAGE_PREFIX: u8 = 0x03`, beside the existing
`RESERVED_PREFIX = 0x00` (entr — the name is historical; `consts.rs:17`) and
`MNEM_PREFIX = 0x02` (`consts.rs:39`). `0x01` stays UNALLOCATED, as
MIGRATION.md already records. Both copies of `discriminate`'s doc comment
(`envelope.rs:114-115` and `:186-188`) gain the `0x03` line, or they are stale
the day the arm lands.

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
characters, where a person actually looks.

**The id is accepted, checked, and never dispatched on.** Three rules, each
with a site:

1. **`hash` joins the single-string accept set.** `ms_codec::decode` admits a
   tag only if it is in the v0.2 accept set, which today is `{entr}` alone
   (`decode.rs:85`, `x if x == TAG_ENTR`), with a `_ =>` catch-all that
   returns `Error::UnknownTag` (`decode.rs:101-103`). Without this rule every
   preimage single is refused before any payload dispatch runs — measured by
   the correctness lens: `unknown tag "hash"; not a member of
   RESERVED_TAG_TABLE` — and `ms hashlock --out` would emit a plate that `ms`
   itself cannot read, while `combine_shares` (which never consults the
   accept set) would read the same bytes as shares. `consts.rs` gains
   `TAG_HASH = *b"hash"` and `tag.rs` gains `Tag::HASH`. (R0 r0 correctness
   C-1.)
2. **A single's tag and prefix must agree, or the string is refused.** Readers
   DISPATCH on the prefix byte; the tag of a single (threshold `0`) is then
   CHECKED against it: `hash` over `0x03`, `entr` over `0x00` or `0x02`,
   anything else `Error::TagKindMismatch { tag, prefix }`. So a hand-made or
   corrupted `ms10hash…` string carrying a seed payload, or an `ms10entr…`
   string carrying a preimage, is refused rather than read as the other kind
   — the fail-closed direction, and the only rule under which "no misread
   converts one kind into the other" is a property a test can pin. The check
   applies to singles only: a share-set's id is random by construction and
   names no kind. (Controller default, listed for the operator's veto in the
   fold record; the alternative is pure prefix dispatch, under which an
   `ms10hash…` plate could decode as a seed.)
3. **`RESERVED_ID_BLOCKLIST` gains `hash`** (five entries today: `entr`,
   `seed`, `xprv`, `mnem`, `prvk`; `consts.rs:71`), so a share set can never
   draw an id that impersonates a preimage single. This list is consulted only
   at share-set generation (`shares.rs:50`) and never on decode — rule 2 is
   the decode-side check, and the two are different lists with different jobs.

**A `0x03` payload whose length is not 33 bytes is refused before the variant is
built**, with a new `Error::PreimageLengthMismatch { got: usize }`, where `got`
is the number of bytes AFTER the prefix byte — the would-be X — so the expected
value is always 32 (review I-2). Two reasons, and the second is the sharp one:
the entr length error would name a legal entr length as illegal, and the obvious
`data[1..33]` indexing **panics** on a short payload. The check precedes
construction; the variant is built with `<[u8; 32]>::try_from(&data[1..])`,
never slice indexing.

**Which lengths are reachable, and by which door — measured, because the
brainstorm's "16..46" was wrong on both halves.** An ms1 string is `22 +
ceil(8N/5)` characters for an N-byte payload, and `Codex32String::from_string`
admits 48..93 characters for the short checksum (`codex32/mod.rs:198-201`), so
the codeword bracket is **16..44 payload bytes** — a 46-byte payload is a
96-character string no entry point can construct (`InvalidLength(96)`;
measured: 45 → 94 and 46 → 96, both refused). `decode` then runs a
STRING-LENGTH gate before it reads the prefix byte (`decode.rs:46`, the union
of the v0.1 and v0.2 length sets `{50,56,62,69,75} ∪ {51,58,64,70,77}`), so
through `decode` the only `0x03` payload lengths that reach prefix dispatch at
all are `{17, 18, 21, 22, 25, 26, 29, 30, 33, 34}`, and **the wrong-length set
that can reach `PreimageLengthMismatch` through `decode` is exactly
`{17, 18, 21, 22, 25, 26, 29, 30, 34}`** — nine values, of which 34 (77
characters, a mnem length) is the off-by-one an implementer would hit. 16, 32
and 44 are refused by `UnexpectedStringLength` on `decode` and reach
`PreimageLengthMismatch` only through `combine_shares`, which has no
string-length gate. The vector rows (§8) name their entry point and the exact
error each asserts; a row that says "refused" without naming which is a gate
that passes on the wrong error. (R0 r0 correctness I-1, adversarial I-4.)

**The share axis is untouched.** Threshold and index live in the codex32 header,
orthogonal to the prefix byte; a K-of-N set of a preimage recovers to a `0x03`
payload and the codec supports it. (The CLI has no ms1 source for `ms split`
today — pre-existing, filed F-468, out of scope here.)

**Misreads cannot convert one kind into another** — and rule 2 above is what
makes that true by refusal rather than by distance. The brainstorm also records
that the two full ms1 codewords that differ only in the id field are at least
nine characters apart while BIP-93 corrects at most four; that is a property of
the whole codeword (the checksum changes with the id), not of the two
four-character ids. **MEASURE THIS AT PLAN TIME**: the plan's build gate encodes
one payload under both ids and asserts the Hamming distance between the two
strings exceeds twice the correction bound. A claim about error correction that
no command checks is exactly the class this constellation keeps finding late.
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
pub fn preimage_random() -> Result<Zeroizing<[u8; 32]>>;          // §4.1 --random
pub fn digest(preimage: &[u8; 32]) -> [u8; 32];                  // sha256(X)
```

`preimage_random` lives HERE, not in the CLI: ms-cli has no `getrandom` and no
`rand` (measured, `Cargo.toml` and `grep`), while ms-codec already depends on
`getrandom 0.3` for share ids (`shares.rs:37-43`). Putting the source of
randomness beside the derivation keeps the whole preimage surface in one crate,
keeps "failing closed" a codec contract, and gives the Go port one thing to pin
(R0 r0 correctness I-2). It uses the same `getrandom::fill` the share path uses
and returns its error rather than a zeroed buffer.

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
pub enum InspectKind { Entr, Mnem, Preimage, Unknown }
```

**The variant wraps `Zeroizing<[u8; 32]>`, not a bare array** (review M-1). A
bare `[u8; 32]` has no `Drop`, so it is memcpy'd on every move and the crate's
documented caller-wrap recipe — the one `Payload::Entr` relies on — cannot reach
it. The wrapper scrubs on drop and keeps the length rule structural: a value of
this variant cannot be the wrong length. Because no listed test can fail on the
wrapper's TYPE, the plan pins it with a compile-time assertion (a
`static_assertions` or `trybuild` check that `Payload::Preimage`'s field is
`Zeroizing<[u8; 32]>`).

**Edit sites, split by how they fail.** LOUD (exhaustive matches inside the
crate; `cargo build` reports each): `decode.rs:27` `allowed_for_kind`,
`payload.rs:93` `kind()`, `payload.rs:102` `as_bytes()`, `validate`, the
`InspectKind` projection, `dispatch_payload` (`envelope.rs:192`) and
`payload_wire_bytes` (`envelope.rs:231`). SILENT (catch-alls the compiler
never flags): the single-string accept set at `decode.rs:85-103` (§1 rule 1)
and every `_ =>` arm over `Payload`, `PayloadKind` or `InspectKind` in
downstream code — see below. `Error::ReservedPrefixViolation` stops firing for
`0x03`; **any test that pinned `0x03` as reserved flips**, and the plan
enumerates those tests mechanically rather than by reading. `InspectKind` is
NOT `#[non_exhaustive]` (`inspect.rs:12-20`), so adding `Preimage` to it is
source-breaking for any downstream exhaustive match — loud, and therefore safe,
and named in §10 rather than called "additive".

### `#[non_exhaustive]` is a hazard here, not a help

`Payload` and `PayloadKind` are `#[non_exhaustive]` (`payload.rs:29`, `:9`),
so downstream `match` statements carry a catch-all and **the compiler will not
tell anyone a variant was added.** The most visible spelling is
`_ => unreachable!` — ms-cli has exactly four, measured at `7fc1e58`
(`grep -rn '_ => unreachable' crates/ms-cli/src` = 4) — but a `_ => <value>`
arm fails **silently** where `unreachable!` at least panics, and ms-cli has
**18** of those (`grep -rn '_ =>' crates/ms-cli/src --include=*.rs | grep -v
unreachable`). Three sit inside `ms inspect`'s would-decode verdict
(`inspect.rs:182`, `:203`, `:223`) beside the CLI's own copies of the accept-set
and prefix rules (`inspect.rs:170-177`: `unknown-tag`, `non-zero-prefix`) and
`reason_text`'s wording "prefix byte is not a recognised kind (0x00=entr,
0x02=mnem)" (`inspect.rs:219`). Without those edits a valid preimage single
reaches `ms inspect` and prints `FAIL: would NOT decode v0.1` with two false
reasons. (R0 r0 correctness I-7.)

The four loud sites, by what the verb is FOR:

| site | reached from | disposition |
| --- | --- | --- |
| `cmd/decode.rs:107` | `ms decode` | **functional**: print kind, preimage hex, digest |
| `cmd/decode.rs:112` | `ms decode` | **functional**: same |
| `cmd/combine.rs:166` | `ms combine` | **functional**: a recovered preimage share set prints as `decode` does (review N-1) |
| `cmd/payload_lang.rs:61` | `ms verify`, `ms derive` | **typed refusal** with the remedy `ms hashlock <ms1>` |

The split is by *what the verb is for*, not by convenience (r1 verification,
finding 1): a verb that READS a secret prints a preimage as a preimage; a verb
that needs a SEED refuses it.

**What the arms become, and how the next kind re-triggers the sweep.** Every
site gains an explicit `Payload::Preimage(..)` arm; the `_ =>` catch-all is
KEPT and stays `unreachable!` (a future `0x04` must panic loudly, never render
as a preimage). A committed test asserts the count of `_ => unreachable` arms
in `crates/ms-cli/src` equals the number this cycle leaves (the pattern of
`consts.rs`'s bijection lock), so the next kind fails CI until its sweep is
done rather than relying on §9's discipline. (R0 r0 tests I-11.)

**`payload_lang.rs:61` is reached before any refusal a verb might add**, so the
refusal must sit there and not in `verify`/`derive` (review I-3) — AND, for
singles, only after §1 rule 1 lands: today `decode()` returns `UnknownTag`
first and neither `verify.rs:99`'s `Ok((_tag, payload))` arm nor the remedy is
ever reached. §5 states it again from the verb's side because it is the easy
one to get wrong.
---

## §4. `ms hashlock` (ms-cli 0.18.0)

### §4.1 Sources — exactly one per invocation

| source | meaning |
| --- | --- |
| `--hashlock-phrase TEXT` | the phrase, on argv; joins `SECRET_FLAGS`, so refused without `--allow-argv-secret` (§6) |
| `--hashlock-phrase-stdin` | the phrase, on stdin; a phrase file is redirected into it |
| `--hex HEX`, or `--hex -` | an existing X, exactly 32 bytes (64 hex characters); anything else is refused naming the composer spec's §8i, in both spellings |
| `<ms1>`, `-`, or `--in FILE` | a preimage-kind ms1, to re-derive H from a plate; an entr or mnem string is refused with *"that is a seed backup, not a hashlock preimage"* |
| `--random` | 32 bytes from the OS CSPRNG via `ms_codec::hashlock::preimage_random` (§2; `getrandom`, failing closed) |

Two sources is exit 64 — for every one of the ten pairs, including the two
that contend for the same stream (`--hashlock-phrase-stdin` with `--hex -` or
with `-`). **The phrase has exactly these two channels** (L20); there is no
third, and an ms1-shaped value handed to either is refused naming `--in`/`-`.

**L8, binding here and everywhere below:** every refusal and every help line
that names the preimage's size says **"32 bytes (64 hex characters)"** — both
spellings, always, because the operator asked which was meant and the answer
must never depend on which line they happened to read.

`--in FILE` means **the ms1**, as it does on the six reading verbs (L20). This
is not a free choice: the argv guard's own remedy for a refused plate string
prints `ms hashlock --in FILE`, so that channel must accept the plate or the
guard would advertise a route that does not exist (review C-1).

**`--random` refuses (exit 64, naming `--out`) unless `--out FILE` is given.**
L21 named `--json` as an alternative persistent channel, and its own rationale
— *"a preimage that reaches no persistent channel is data loss"* — is what rules
it out: `--json` is stdout, the same volatile stream the card was suppressed
on, and the operator who chose it did so to FILTER it. The constructed loss is
one natural line, `ms hashlock --random --json --no-engraving-card | jq -r
.hash_record | me sysw pack`: exit 0, the policy is funded, and `preimage_hex`
went down a pipe nobody read (R0 r0 adversarial C-1). So `--json` no longer
satisfies the gate; `--random --out FILE --json` is fine. **This narrows L21 and
is a controller default pending the operator's word**, recorded in the fold.

**Under `--random`, `--out` refuses to overwrite** (`create_new`; exit 64 naming
the existing file). The 2026-08-26 overwrite ruling was made for artifacts that
are a function of the operator's input, where a clobbered file is one re-run
away; a random preimage exists nowhere else, and the spec's own acceptance
filename (`--out X.txt`) is the shape a second policy would reuse on Tuesday
and destroy Monday's (R0 r0 adversarial C-2). The other four sources keep the
2026-08-26 semantics unchanged, and `--out` still creates owner-only.

**Zero sources is exit 64**, listing the five sources — the same treatment two
sources gets. It must not default to stdin: a bare `ms hashlock` at a terminal
would then block with no prompt, which is the `mt` finding §4.3 fixes for one
channel, and an operator who pasted their phrase at that invisible prompt
would get an ms1 parse error and a phrase in scrollback (R0 r0 adversarial
I-3).

**`--hex` gets an unconditional warning of its own**, because every other
warning in §7 is method-keyed and `--method` is refused on this source, so
under `--hex` nothing would fire — and the constellation formats a wallet's
master entropy in exactly the shape `--hex` wants (`ms decode <seed plate>`
prints it as 64 hex). The card and stderr say: *"The first spend of this hash
path publishes these 32 bytes in the clear, forever. If this value is also
anything else's secret — a seed's entropy, a key — every use of that secret is
public with it."* The tool cannot tell one 32-byte value from another, so a
refusal is not available and the copy is the whole defence (R0 r0 adversarial
C-4).

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
  deriving from it produces a valid-looking record for a different X. On the
  argv channel this refusal is never reached: the guard's layer 1 matches
  `--hashlock-phrase` in `SECRET_FLAGS` before clap and refuses with the
  argv-secret remedy instead (`argv_guard.rs:342-350`), so the `--hex` remedy is
  a property of the stdin channel and the admitted side channel only (R0 r0
  adversarial M-2).
- **An ms1-shaped phrase is refused** on both channels, naming `--in`/`-`
  (r2 review C-1). The shape test is ONE function that both the argv guard and
  the phrase channels call — `pub(crate) fn looks_like_ms1(raw: &str)`, which
  trims, lowercases, strips display separators, and only then tests the HRP
  and charset — so the two cannot drift. **The normalisation is part of the
  predicate, not of its callers**: today `is_ms1_shaped` (`argv_guard.rs:134`)
  tests `starts_with("ms1")` against the lowercase charset and the case-fold
  lives in `argv_candidates` (`argv_guard.rs:104-111`), so exporting the
  predicate alone would pass the all-uppercase BIP-173/QR spelling that
  `ms decode` accepts as first-class (`tests/decode_uppercase.rs`) — and derive
  a preimage from the text of a plate string. `argv_candidates` stops
  pre-folding and calls the same function. **The phrase's VALUE is never
  normalised**; only the shape test sees the folded copy. (R0 r0 tests C-1.)
  **The shape test runs BEFORE the length cap**: a 75-character ms1 at group
  size 2 is 112 characters, and cap-first would refuse it as "too long" without
  ever pointing at `--in` (R0 r0 adversarial N-1).
- **The 64-hex guard uses the same hex predicate as `--hex`'s parser**
  (`hex::FromHex`, `encode.rs:283`, which accepts both cases), so an uppercase
  preimage pasted as a phrase is redirected to `--hex` exactly as a lowercase
  one is. (R0 r0 tests I-6.)
- **Printable ASCII means bytes `0x20..=0x7E`, inclusive, and nothing else.**
  TAB, DEL and every C0 control byte are refused by the same rule as a
  non-ASCII byte — an `is_ascii()` check would admit a tab, and a phrase whose
  whitespace is invisible on paper is a preimage the operator cannot
  reproduce. (R0 r0 tests I-5.)

**The phrase channels use a new byte-verbatim reader**: bytes as given, exactly
one trailing `\r?\n` stripped, nothing else. They must NOT use
`parse::read_input` (strips all whitespace plus `-` and `,`) or
`parse::read_phrase_input` (trims and collapses runs) — either silently changes
X while every codec vector still passes (r2 review I-3), and `-` and `,` are
the normal joiners of the diceware output §7 tells the operator to use.
`read_stdin_passphrase` (`parse.rs:139-148`) has the right STRIPPING shape and
is the model for that half only: it is built on `read_to_string`, which fails a
non-UTF-8 byte with an io error rather than the phrase rule's named refusal.
The new reader is `Vec<u8>` via `read_to_end`, so a raw `0xFF` is refused by
the printable-ASCII rule, by name, like every other bad byte. (R0 r0
correctness M-6.)

With stdin at a terminal, `--hashlock-phrase-stdin` prints one prompt line to
stderr — `Type the hashlock phrase, then Enter.` — rather than blocking
silently (r2 review M-7; the constellation's recorded `mt` finding, where a
tool's first interaction looked like a hang).

Refusals name the rule and **never echo the phrase**.

### §4.4 Outputs

- **stdout**: one line, `hash:<64 lowercase hex>` — the record `me sysw pack`
  reads from stdin when given neither `--in` nor argv records (measured on
  `me` 0.8.0: `… | me sysw pack --out FILE` exits 0 and writes the container;
  `--in -` is NOT a stdin sentinel there and exits 2 — the spec's first draft
  named it three times, R0 r0 adversarial I-2). Lowercase because
  `sysw/composer_records.rs` refuses an uppercase `hash:` body by name
  (adversarial M-5). Public, so no stdout advisory. **`--out` never suppresses
  it** (r2 review I-5). `ms encode` suppresses its stdout artifact under
  `--out` because both channels carry the same secret; here they carry
  *different* artifacts, and copying that shape would hand `me sysw pack` an
  empty stream.
- **`--out FILE`**: the preimage as an ms1 string, mode `0600`, overwriting.
  `--out` is the preimage's channel; stdout is the digest's.
- **stderr card** (suppressed by `--no-engraving-card`), whose **first line
  names it as carrying the preimage** (r2 review M-1). This verb inverts `ms`'s
  usual polarity — stdout public, stderr secret — so `2>>log` or `2>&1 | tee`
  lands a preimage in a `0644` file and nothing else on the stream would say so.
  Then: the digest; the `sha256=` operand for `md compose --path`; the preimage
  as grouped ms1 (`--group-size`/`--separator` apply) and as hex; the **method
  line**, verbatim and copyable — for the two phrase sources; for `--hex`, `--random`
  and `<ms1>` the line reads `preimage supplied` and carries no write-it-down
  instruction, there being no phrase to write it beside (correctness N-2) —
  e.g.
  `preimage = PBKDF2-HMAC-SHA256(password = phrase, salt = "ms-hashlock-v1", iterations = 100000, dkLen = 32)`
  or `preimage = SHA-256(phrase)`; the phrase's **character count** beside it
  (review M-2 — the one signal that makes a stray space visible); the composer spec's §8i and
  F-132 lines; the reuse lines of §7; the method's warning; and the source kind
  **without its value**.
- **`--json`**: one JSON object on stdout **in place of** the record line —
  the shipped shape (`encode.rs:218-230`, `decode.rs:123`: `if args.json {
  emit_json } else { emit_text }`), stated here because L21 admits `--json` as
  `--random`'s persistent channel and a consumer must know whether stdout is a
  record or an object. It carries `hash_record` (the exact line text mode
  prints), `digest`, `sha256_operand`, `preimage_hex`, `preimage_ms1`,
  `source`, and — **for the two phrase sources only, omitted otherwise, with
  the same rule as `method`** — `method` (`{kdf, hash, salt, iterations,
  dklen}` or `{hash}`) and `phrase_chars`. Every hex value is lowercase, and a
  row pins it. It carries the secret, so the `PrivateKeyMaterial` advisory
  fires, as `encode --json` does. (R0 r0 correctness I-3, I-4.)

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
| `repair` | unchanged and benign — `cmd/repair.rs:141` binds `(_tag, _payload, corrections)` and discards the payload — but named here because it is the verb an operator reaches for with a scratched preimage plate, and §1's "misreads cannot convert one kind into another" is a claim about its engine, `decode_with_correction` (adversarial M-3) |

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

Joining it is a **six-part edit**, and each part has a symptom if skipped
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
5. **The same binding for `--hex`**, which is ALREADY in `SECRET_FLAGS`: under
   `--allow-argv-secret` the guard moves its value into `ADMITTED["--hex"]` and
   rewrites the argv to `--hex -` (`argv_guard.rs:283-297`), so a verb that
   reads `-` from stdin would read the wrong stream. Same `/dev/null` gate.
6. **And for the positional `<ms1>`**, which layer 2's shape test moves into
   `ADMITTED[CH_POSITIONAL]` and replaces with `-` (`argv_guard.rs:318-327`).
   Same gate. Three material channels, three bindings, three gates (R0 r0
   adversarial I-1).

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

Under `--random` the card says **both halves** (review M-5), and names the
artifact that actually exists when it prints: *"No phrase exists, so nothing can
be guessed, and nothing can be remembered. The file you just wrote is the only
copy until you cut the plate."* (adversarial M-1: at that moment there is no
plate, and the safest-sounding word was the wrong one.)

Under `--hex`, the unconditional line of §4.1 — the one warning that can reach
an operator who supplied X rather than a phrase.

---

## §8. Vectors and the corpus

The corpus SHA is re-pinned, which is what forces the minor bump. **Every
derivation row below is reproduced externally, not only the anchor**: a row the
implementation generated is a regression pin, not a correctness pin (tests
M-6), and PBKDF2 at 100,000 iterations over a dozen rows is cheap.

**Kind rows.** Encode/decode/inspect for `0x03`; the share round trip through
the codec API; id `hash` on singles and its blocklist entry; **`hash` in the
single-string accept set** (a `hash` single round-trips `encode` → `decode`;
correctness C-1); **id/prefix disagreement, both directions, both refused**
with `TagKindMismatch` — id `hash` over a `0x00` payload, id `entr` over a
`0x03` payload — which is the only row that can fail a reader dispatching on
the id (tests I-1); the **entr-32 / preimage pair as adjacent rows**,
`ms10entrsq…` beside `ms10hashsq…`, both 75 characters, so a
length-dispatching reader has a row to fail (tests I-2); and the **hex case**
of `hash:`, `preimage_hex` and `sha256_operand` pinned lowercase (tests M-8).

**Length rows, each naming its door and its error.** Through `decode`:
`{17, 18, 21, 22, 25, 26, 29, 30, 34}` payload bytes → `PreimageLengthMismatch
{ got }` with `got` = N−1, and 34 first among them because 77 characters is
the off-by-one. Through `combine_shares`: 16, 32 and 44 → the same error.
Through `decode`: 16, 32 and 44 → `UnexpectedStringLength` (48, 74, 93), so a
row written "refused" without the door asserts nothing. 46 is unconstructible
and appears only as a comment saying so. **The downgrade row**: a `0x03` single
fed to ms-codec 0.7 is refused with `ReservedPrefixViolation` and never
panics — the row that proves §9's Rust half (correctness I-6.3).

**Derivation rows, both methods**, each pinning **X and H** (r2 review N-1):
the anchor phrase of §2; one character (a non-hex one, so the 64-hex guard's
over-refusal is not confused with it); 20 characters; 64 and 65 (the HMAC
block boundary); 100 and 101; a phrase with leading, trailing and doubled
spaces; **a phrase with a hyphen and a comma** — `correct-horse,battery staple`
— because `-` and `,` are what `read_input` strips and what diceware emits
(tests I-4); **byte-exact rows through BOTH phrase channels**, `"  a  b "` and
`"a-b,c"`, via `--hashlock-phrase-stdin` and via `--hashlock-phrase` under
`--allow-argv-secret` through the admitted side channel, equal to the codec
vector (correctness I-6.1: the mutation is swapping in `read_phrase_input` or
`read_input` on either channel, and no codec vector can see it).

**Refusal rows.** Empty; non-ASCII (`é`) AND a raw `0xFF` byte (correctness
M-6); **TAB and DEL refused, `0x20` and `0x7E` accepted**, so the printable
boundary is pinned on both sides (tests I-5); the 64-hex refusal in lowercase
AND uppercase (tests I-6), and a short all-hex phrase (`beef`) ACCEPTED so the
guard's over-refusal direction is pinned (tests M-9); **the ms1-shaped refusal
in four spellings — lowercase, UPPERCASE, grouped, space-padded — on both
phrase channels** (tests C-1, I-3); an ms1 at group size 2 (112 characters)
refused as ms1-shaped, not as too long (adversarial N-1).

**The reproduction test, written so it cannot lie.** A test in ms-codec runs
`python3 hashlib.pbkdf2_hmac` and `openssl kdf` and compares **captured
stdout** of both against the expected hex **three ways** (Rust = python,
Rust = openssl, python = openssl). **The salt string, the iteration count, the
dkLen and every expected hex appear in the test as LITERALS, independent of the
crate's constants**, with one separate assertion that the constants equal the
literals — otherwise mutating a constant moves both sides of the comparison
together and the cross-tool check is blind to exactly the mutations it exists
for (tests I-9). It FAILS if either tool is absent — never `#[ignore]`, never a
`cfg` gate — and CI asserts the test RAN by name (`cargo nextest run -p
ms-codec -E 'test(hashlock_repro)'` reporting one test executed), because a
missing-tool skip is only one of the ways a test prints ok while proving
nothing (tests FP-4). The `test (ms-codec)` job (Ubuntu-only) gains a preflight
step that exercises the actual capability — `python3 -c 'import hashlib;
hashlib.pbkdf2_hmac("sha256", b"x", b"y", 1)'` and `openssl kdf --help` — and
logs `openssl version` and `python3 -VV` (tests M-5, FP-5); the ms-cli matrix
includes macOS, whose stock `openssl` is LibreSSL with no `kdf` (r2 review
I-6).

**Lockstep rows for H2**, the ones a device text-entry widget is most likely
to drift on (tests I-10): a 100-character phrase derives byte-identically on
host and device and a 101-character one is refused on both; the 64-hex
refusal on both; **the spaces row** (a widget that trims a trailing space
changes X silently); **the empty-phrase refusal**; **the printable-ASCII
boundary**; and the id/prefix-mismatch pair, because the device RENDERS the id.
The fork's pin test drives the vendored rows in **both directions** — an
encode-side drift, a Go encoder writing id `entr` for a preimage, is invisible
to a decode-only pin.
---

## §9. MIGRATION.md — a 0.7 → 0.8 section, and the H0 prerequisite

1. Readers that dispatch on the prefix byte **MUST** treat `0x03` as a 32-byte
   preimage and never as entropy.
2. **Length no longer implies kind** (§1), and singles of the kind carry the id
   `hash`; a single whose id and prefix disagree is refused (§1 rule 2).
3. Every downstream crate **MUST** sweep **every catch-all** over `Payload`,
   `PayloadKind` and `InspectKind` — `_ => <value>` arms as much as
   `_ => unreachable!` — because `#[non_exhaustive]` means the compiler will
   not, and a value-returning catch-all fails silently (correctness I-7).
   `InspectKind` is not `#[non_exhaustive]`, so its change is loud.
4. **The pre-tool recipe this project documented everywhere** — the composer
   spec's §8i, the W-5 walk, F-465: *"hash the passphrase to 32 bytes, then
   hash again"* — is `--method sha256`, **NOT the default**. A digest made by
   hand before 0.18.0 reproduces only with that flag (review M-3). The same
   note goes in the manual chapter and in F-465's `Which hash?` hint.
5. **A third reader shape, and the one the two Rust readers in this
   constellation actually have: "decode succeeded, therefore this is a
   seed."** `me`'s `validate_record` maps ANY `ms_codec::decode` success to
   `RecordKind::Ms`, whose `is_secret()` is true, and never looks at the
   prefix byte (`seal/record.rs:177`, `:42-43`); the fork's `isStrictMs1` is
   `len ≤ 90 && HasPrefix "ms1" && codex32.New == nil` with no prefix test at
   all (`sysw/classify.go:116-125`). Neither matches item 1's shape nor item
   3's, so a MIGRATION note written for prefix-dispatching readers would not
   have reached either.

**The brainstorm's "older readers refuse" premise is measured false, and §9
no longer claims it.** The first draft said every SH2 flashed before H2
rejects a `0x03` string as a bad prefix, so "the failure mode is a refusal and
never a seed". Measured on fork `839fa5a` by the adversarial lens and
re-verified by the controller: `isStrictMs1` accepts the string,
`sysw.Classify` returns `ClassCodex32Secret`, the unlock session labels it
`ms1`, and `unlockEngraveCodex32` calls `codex32.New` then
`backup.EngraveSeedString` — `DecodeMS1` appears zero times in that file — so
**a flashed device today cuts a preimage plate as a seed plate**, no refusal
anywhere on the path. On the Rust side `me` refuses today only because it pins
ms-codec `0.7` (`Cargo.toml:53`); on the bump, item 5's shape turns a preimage
record into a secret seed record.

| reader | on a `0x03` single | measured |
| --- | --- | --- |
| `ms` / ms-codec 0.7 | `reserved-prefix byte was 0x03`, exit 2 | refusal |
| `me` 0.7.0 / 0.8.0 (ms-codec 0.7 pinned) | refuses: not a constellation `ms1` record | refusal, until the bump |
| mnemonic-toolkit (ms-codec 0.7) | `ms_codec::decode` refuses; the catch-all is an `Err` | refusal |
| **fork `839fa5a`, the flashed SH2** | **`ClassCodex32Secret`; engraved as a seed plate** | **NOT a refusal** |

**H0 — the prerequisite.** Before ms-cli 0.18.0 is released, so that no
preimage plate can exist while a reader that would cut it as a seed is in the
field: (a) the fork's `isStrictMs1` / `seal.Classify` gains the prefix test
that makes a `0x03` string INERT (never `ClassCodex32Secret`, no new class;
L22's rule, moved forward), with the record-class vector row, merged and
**flashed to the operator's device**; (b) `me`'s `validate_record` treats kind
`0x03` as inert in the same release window as `me`'s ms-codec 0.8 bump (H1b as
already planned, now with a "before, not after" constraint). This reorders
4.5's sequence — H0 precedes the 0.18.0 release rather than following it as
H2 — and is a controller default pending the operator's word, recorded in the
fold.
---

## §10. Lockstep, SemVer and provenance

- ms-codec **0.8.0** (new kind, `hashlock` module, corpus SHA re-pinned,
  MIGRATION section); ms-cli **0.18.0** (the verb, the new arms, the guard
  edit). ms-cli's dependency pin on ms-codec moves to `=0.8.0`. The 0.x minor
  is the breaking bump in cargo semantics and this one IS source-breaking for
  any downstream exhaustive match on `InspectKind` (which is not
  `#[non_exhaustive]`) — loud, therefore safe, and stated rather than called
  "additive" (correctness M-2).
- **Release order: H0 first** (§9), then ms-codec 0.8.0 and ms-cli 0.18.0
  together per `design/RELEASE_PROCESS.md` — corpus SHA pin, CHANGELOG,
  MIGRATION, **publish dry run, both tags** — with the manual chapter in
  lockstep (correctness M-5). The H1 plan gets a `plan-build-gate-ms.sh`
  sibling of the me and md gates on the pinned toolchain, and that gate is
  what measures §1's codeword distance.
- The manual chapter
  `mnemonic-toolkit/docs/manual/src/40-cli-reference/43-ms.md` moves in
  lockstep, and the toolkit's flag-coverage lint must pass.
- **Rust-primary.** The fork's `0x03` arm and its derivation (H2) carry a
  provenance pin to the ms-codec 0.8.0 commit, and the hashlock vector corpus is
  vendored into the fork with a pin test, exactly as the compose vectors are.
  No normative behaviour is decided in Go.

---

## §11. Tests

Beyond §8's corpus, each of these names a mutation it fails under (the plan
carries the mutation beside the test, and the whole-diff review's mutation pass
pastes the failing output):

**Guard and channels.** The argv guard for `--hashlock-phrase` — refused
without the allow flag, the value never echoed — and the `flag_class` wording
("a hashlock phrase"); **the three `/dev/null` gates of §6** (`--hashlock-phrase`,
`--hex`, the positional), one test each; stdin stripping of exactly one LF or
CRLF, and a phrase file with two trailing newlines keeping one; `--in FILE`;
`--hashlock-phrase-stdin` at a terminal prints the prompt line.

**Sources.** `--hex` at 63, 64 and 65 characters, upper and lower case; entr
and mnem strings refused with the seed-backup wording and **entr-32
specifically** (the colliding length; tests I-2), kind 3 accepted; `--random`
twice gives two different records; **zero sources exits 64 listing five**;
**every one of the ten two-source pairs exits 64**, the stdin-contention pair
(`--hashlock-phrase-stdin` with `-`) by name (tests M-4); `--random` without
`--out` exits 64 naming `--out` — **including with `--json`** — and `--random
--out FILE` succeeds; `--random --out` onto an existing file exits 64 and
leaves the file's bytes unchanged, while the other four sources overwrite;
`--method` with each of `--hex`, `--random` and `<ms1>` exits 64 (tests M-3).

**Outputs.** stdout is exactly the record line, lowercase hex, **under `--out`
and under `--method sha256`** — the two configurations where a stdout-purity
mutation can hide, since the anchor phrase is 28 characters and warns under
neither default (tests I-7); `--out` is `0600`; `--json`'s schema in **both**
variants — phrase source with `method` and `phrase_chars`, supplied source
without (tests M-10) — and its advisory; the card per source and per method,
including both `--random` halves naming the FILE, the `--hex` line, §7's reuse
lines, and **the warnings at their boundaries**: hardened at 19 (warns) and 20
(does not), sha256 at 100 characters (still warns) (tests I-8); the method
line's instruction text pinned, including "each method that shipped with the
version named on this card" (tests N-3); the character count present for
phrase sources and absent otherwise.

**Negative-content matrix** (Minor class by the 2026-08-27 ruling, recorded
here as the brainstorm agreed it): one row per refusal — empty, non-ASCII,
control byte, over 100, 64-hex, ms1-shaped, `--hex` wrong length, wrong ms1
kind, zero sources, two sources, `--method` with a supplied X — asserting the
phrase and the preimage appear in neither stdout, stderr nor the `--json`
error envelope (correctness M-3; mutation: a refusal built with
`format!("... {phrase}")`).

**The other verbs.** `decode` prints kind, hex and digest and **never words**
(tests N-2); `inspect` reports the kind and its would-decode verdict passes
with no false reasons; `combine` on a preimage share set; `derive` and
`verify` refuse with the executable remedy — and for a single, that the
refusal is reached at all (it is not until §1 rule 1 lands); **one test per
`unreachable!` site that panics on 0.17.x**; the committed catch-all count
(§3); the compile-time `Zeroizing` pin on `Payload::Preimage` (tests M-7);
`SECRET_FLAGS`'s doc comment corrected from "nine" while the line is edited
(tests N-4).

**Codec.** `TagKindMismatch` both directions; the `hash` accept-set
round-trip; the length rows by door; the downgrade row against 0.7.

**Toolchain.** MSRV, clippy, fmt; the man page carries the verb; the toolkit
manual's flag-coverage lint passes.

**Review gates.** The plan's tests lens mutates these before any code is
written, and the whole-diff review's mutation pass proves each guard fails on
its named mutation with the output pasted. A guard whose named mutation does
not fail it is not a guard.
---

## §12. Acceptance

H1 is done when, on a clean checkout at the merge commit, with H0 (§9) already
shipped and flashed:

1. `ms hashlock --hashlock-phrase-stdin < phrase.txt` prints exactly
   `hash:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12` on
   stdout for the anchor phrase under the default method, and the card on
   stderr names the preimage on its first line.
2. `ms hashlock --hashlock-phrase-stdin --method sha256 < phrase.txt` prints
   `hash:b867db875479bcc0287352cdaa4a1755689b8338777d0915e9acd9f6edbc96cb` —
   the value the W-5 walk recorded by hand — and always carries the brainwallet
   line.
3. `--out X.txt` writes the 75-character string **the corpus row pins**
   (`ms10hashsq…`; the leading `q` alone cannot distinguish `0x03` from `0x01`,
   tests N-1) at mode `0600`, and `ms hashlock --in X.txt` re-derives the same
   digest.
4. `ms decode` on that string prints the kind, the preimage hex and the digest,
   and **never** words; `ms inspect` reports the kind with no false reason;
   `ms derive` and `ms verify` on it refuse with the remedy; none of the four
   `unreachable!` sites panics.
5. `ms hashlock --random` without `--out` exits 64 naming `--out`, with or
   without `--json`; with `--out FILE` it succeeds and refuses to overwrite.
6. `ms hashlock … | me sysw pack --out payload.bin` — stdin, no `--in` —
   builds the container, and the composer's `Which hash?` payload route offers
   the record. (Measured spelling; `--in -` exits 2.)
7. A `0x03` single fed to the flashed device is INERT — `sysw.Classify` is not
   `ClassCodex32Secret` and no engrave path offers it — and fed to `me` at the
   0.8 bump is inert likewise (H0, §9).
8. The corpus SHA pin, MIGRATION section, CHANGELOG, manual chapter, both
   version bumps, the publish dry run and both tags are in the same release.
---

## §13. Out of scope

hash160/ripemd160/hash256 on the host (the composer composes sha256 only, and
ripemd160 cannot be a preimage derivation in any case: every miniscript hash
fragment demands a 32-byte X via `OP_SIZE 32 EQUALVERIFY`, and ripemd160 yields
20); a preimage plate or a preimage as a source **on the device** (L7 — the
device is digest-only this cycle); any non-ASCII phrase; K-of-N shares of a
preimage from the CLI (F-468); an operator-chosen salt (F-469, L13); refusing
a `--hex` value that is also a seed's entropy — the tool cannot tell one
32-byte value from another, which is why §4.1's warning exists.

---

## §14. Citations — measured at `7fc1e58` (ms), `839fa5a` (fork), `me` 0.8.0; re-grep at implementation time

| claim | site |
| --- | --- |
| the single-string accept set is `{entr}` with an `UnknownTag` catch-all | `crates/ms-codec/src/decode.rs:85-103`; `error.rs:51` |
| `decode`'s string-length gate precedes the prefix read | `crates/ms-codec/src/decode.rs:46` |
| short-checksum codeword bracket 48..93 characters | `crates/ms-codec/src/codex32/mod.rs:198-201` |
| four `_ => unreachable!` arms over `Payload` | `cmd/payload_lang.rs:61`, `cmd/decode.rs:107`, `cmd/decode.rs:112`, `cmd/combine.rs:166` (`grep -rn '_ => unreachable' crates/ms-cli/src` = 4) |
| eighteen value-returning `_ =>` arms; three in `inspect`'s verdict | `grep -rn '_ =>' crates/ms-cli/src --include=*.rs` minus the `unreachable` ones = 18; `crates/ms-cli/src/cmd/inspect.rs:170-177`, `:182`, `:203`, `:219`, `:223` |
| `Payload`, `PayloadKind` are `#[non_exhaustive]`; `InspectKind` is not | `crates/ms-codec/src/payload.rs:29`, `:9`; `crates/ms-codec/src/inspect.rs:12-20` |
| `PayloadKind` has exactly `Entr`, `Mnem` | `crates/ms-codec/src/payload.rs:10-15` |
| prefix dispatch and wire projection | `crates/ms-codec/src/envelope.rs:192` (`dispatch_payload`), `:231` (`payload_wire_bytes`), `:216` (`ReservedPrefixViolation`) |
| `RESERVED_PREFIX` = `0x00`, `MNEM_PREFIX` = `0x02` — definitions | `crates/ms-codec/src/consts.rs:17`, `:39`; doc-comment copies at `envelope.rs:114-115` and `:186-188` |
| `RESERVED_ID_BLOCKLIST`, five entries; consulted only at share generation | `crates/ms-codec/src/consts.rs:71`; `shares.rs:50` (and the test at `:469`) |
| `getrandom` is a ms-codec dependency used for share ids; ms-cli has none | `crates/ms-codec/Cargo.toml:18`; `shares.rs:37-43`; `crates/ms-cli/Cargo.toml` |
| `Error::ReservedPrefixViolation { got }` | `crates/ms-codec/src/error.rs:62`, rendered `:202` |
| `SECRET_FLAGS`, four entries (doc comment says "nine") | `crates/ms-cli/src/argv_guard.rs:85-86` |
| `SUBCOMMANDS: [&str; 12]` | `crates/ms-cli/src/argv_guard.rs:67` |
| `argv_candidates` trims and lowercases; `is_ms1_shaped` does not | `crates/ms-cli/src/argv_guard.rs:104-111`, `:134-145` |
| the guard rewrites `--hex` and the positional into admitted channels | `crates/ms-cli/src/argv_guard.rs:283-297`, `:318-327`; layer 1 at `:342-350` |
| `ms decode` accepts the all-uppercase form | `crates/ms-cli/tests/decode_uppercase.rs` |
| `--hex` parses via `hex::FromHex`, both cases | `crates/ms-cli/src/cmd/encode.rs:283` |
| `--json` replaces the text output on stdout | `crates/ms-cli/src/cmd/encode.rs:218-230`; `decode.rs:123` |
| `--out` is a truncating `write_private` | `crates/ms-cli/src/out.rs:24-28` |
| the advisory writes one line and returns | `crates/ms-cli/src/advisory.rs:60-72` |
| `repair` discards the payload | `crates/ms-cli/src/cmd/repair.rs:141` |
| singles carry `Tag::ENTR` | `crates/ms-cli/src/cmd/encode.rs:200` |
| one trailing LF/CRLF stripped, on a `read_to_string` base | `crates/ms-cli/src/parse.rs:139-157` |
| advisory classes | `crates/ms-cli/src/advisory.rs:53` (`OutputClass`) |
| `me` pins ms-codec `0.7`; any decode success is `RecordKind::Ms`, secret | mnemonic-engrave `crates/me-cli/Cargo.toml:53`; `src/seal/record.rs:177`, `:42-43` |
| `hash:` must be 64 lowercase hex | mnemonic-engrave `crates/me-cli/src/sysw/composer_records.rs:331` (`hash-uppercase` → `Unknown`) |
| `me sysw pack` reads stdin with no `--in`; `--in -` is a path | measured on `me` 0.8.0: exit 0 / exit 2 |
| the fork's `isStrictMs1` has no prefix test; the engrave path never calls `DecodeMS1` | seedhammer `sysw/classify.go:116-125`; `gui/unlock_session.go` (`unlockEngraveCodex32`: `codex32.New` then `backup.EngraveSeedString`; `DecodeMS1` ×0) |
| an entr-32 single is `ms10entrsq…`, 75 characters | measured with the shipped `ms` |
| the anchor derivation values | measured in `python3 hashlib` and `openssl kdf`, §2 |
| string length `22 + ceil(8N/5)`; the reachable sets | measured, §1 |
