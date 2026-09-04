# SPEC_ms_hashlock — R0 round 0, ADVERSARIAL lens (construct the loss)

Artifact: `design/SPEC_ms_hashlock.md` at mnemonic-secret `5ba61ca`.
Tree measured against: `7fc1e58` (= the spec's own base), fork `839fa5a`,
mnemonic-engrave `crates/me-cli` 0.8.0, mnemonic-toolkit `d8f06483`.
Lens: adversarial only. Citations (§14), test coverage, and the KDF choice
(L4/L5/L13) are other lenses' and are not touched here.

Everything below was **executed**, not read. The scratch crate used to build
`0x03` strings against the shipped ms-codec 0.7.0 is
`/scratch/code/shibboleth/_experiment/hashlock-adv-r0` (public
`Codex32String::from_seed`, no modification to any repo).

Two strings are used throughout, built from the spec's own anchor phrase
`correct horse battery staple` under the hardened method:

```
preimage (0x03, id hash) ms10hashsq0p7jaf9gsjjpkjvll2l274w8a388xgqzlewp73scptwxgtjugspvs8tklufg89hqj   75 chars
entr-32  (0x00, id entr) ms10entrsqrp7jaf9gsjjpkjvll2l274w8a388xgqzlewp73scptwxgtjugspvl8sagkqdva662   75 chars
```

The spec's §2 values reproduce exactly (`pbkdf2` + `sha2` crates, third
independent tool after the controller's `python3`/`openssl`): hardened X
`c3e97525…2016`, H `3cf5d421…4c12`; sha256 X `c4bbcb1f…9a8a`, H `b867db87…96cb`.

---

## Findings

### C-1 — `--random --json` satisfies L21's gate while the preimage reaches nothing durable

**The loss:** (b) a preimage that exists nowhere → (a) funds that cannot be spent.

L21's own worked example is *"`--random --no-engraving-card` without either
exits 64 naming `--out`"*. Add `--json` to that exact invocation and it exits 0,
because §4.1 accepts `--json` as a persistent channel. But `--json` is **stdout**
— the same volatile channel the card was suppressed on — and §4.4 confirms it
carries the secret to stdout (the `PrivateKeyMaterial` advisory fires there).
Verified: `crates/ms-cli/src/advisory.rs:60-72`, `emit_output_class_advisory`
**warns and returns**; nothing refuses a terminal or a pipe.

Sequence, all of it a single natural line:

```sh
ms hashlock --random --json --no-engraving-card | jq -r '.hash_record' | me sysw pack -
```

1. `--random` draws X from `getrandom`. Nothing else on the machine has ever held it.
2. `--json` satisfies §4.1's gate. Exit 0. No warning fires about durability —
   the card that would have said *"This plate is the only copy"* was suppressed,
   and that suppression is exactly what L21 was written to catch.
3. `jq -r '.hash_record'` selects one key. `preimage_hex` and `preimage_ms1` are
   consumed by the pipe and discarded.
4. `me sysw pack` builds the container, the operator engraves the policy and
   funds the hashlock path.
5. X exists nowhere. The hash path is permanently unspendable.

The spec closed *"the preimage reaches no channel"* and left *"the channel it
reached does not persist"* open. `--out FILE` genuinely persists; `--json` does
not, and the operator who chose `--json` did so precisely because they wanted to
**filter** the output.

**Classification: refusal.** `--random` should require `--out FILE`; `--json`
alone should not satisfy the gate (or should satisfy it only when stdout is a
regular file). **Operator note:** the remedy narrows L21, which names `--json`
explicitly, so this needs the operator's word rather than a silent spec edit.
The counterexample is offered against L21's own stated rationale (*"a preimage
that reaches no persistent channel is data loss"*), not against the ruling.

---

### C-2 — `--out` silently truncates, and under `--random` the clobbered preimage cannot be re-derived

**The loss:** (b) a preimage that exists nowhere → (a) funds that cannot be spent.

Measured: `crates/ms-cli/src/out.rs:24-28` → `mnemonic_io_lib::write::write_private`,
a truncating write. §4.1 reaffirms this deliberately: *"`--out`'s overwrite
semantics are unchanged from the 2026-08-26 ruling."*

That ruling was made for `ms encode --out`, where the artifact is a **function of
the operator's input** — clobber it and re-run the same command to get it back.
`--random` breaks that precondition: the artifact is a function of entropy that
exists nowhere else. The spec imports the ruling without noticing the precondition
changed.

Sequence:

```sh
# policy A, Monday
ms hashlock --random --out preimage.txt > policyA.hash        # X_A lands in preimage.txt
me sysw pack --in policyA.hash --out payloadA.blob            # engrave the policy plate
#   (the preimage plate itself is queued — a plate is ~21 minutes on the machine)

# policy B, Tuesday
ms hashlock --random --out preimage.txt > policyB.hash        # exit 0, no prompt, no warning
```

`preimage.txt` now holds X_B. X_A is gone — Monday's terminal is closed and the
plate was never cut. Funds under policy A's hash path are unspendable.

`--out X.txt` is also the spec's own acceptance filename (§12.3), so the reused
path is the shape the spec teaches.

Note the asymmetry that makes this cheap to fix: for `--hashlock-phrase*`,
`--hex` and `<ms1>`, overwriting is harmless — the same input regenerates the
same file. Only `--random` is irreproducible, and `--random` already has its own
gate.

**Classification: refusal.** Under `--random`, `--out` opens `create_new` (or
requires an explicit `--force`). The other four sources keep the 2026-08-26
semantics unchanged.

---

### C-3 — §9's *"the failure is a refusal and never a seed"* is FALSE for a pre-H2 SH2, measured

**The loss:** (d) a plate a machine takes for something it is not.

§9 asserts: *"Older readers, including every SH2 flashed before H2, reject a
`0x03` string as a bad prefix (`ReservedPrefixViolation` in Rust,
`errMSBadPrefix` in Go — both traced by the review), so **the failure mode is a
refusal and never a seed.**"*

Measured on fork `839fa5a` — the tree a pre-H2 SH2 runs — by linking the fork's
own packages (`/tmp/gocheck`, `replace seedhammer.com => …/seedhammer`):

```
string: ms10hashsq0p7jaf9gsjjpkjvll2l274w8a388xgqzlewp73scptwxgtjugspvs8tklufg89hqj (75 chars)
  sysw.Classify = 2                    (ClassCodex32Secret)
  seal.Classify = codex32 secret
  codex32.New err = <nil>
  DecodeMS1 = prefix 0 lang 0 entlen 0 err codex32: not an m-format secret payload
  Split id="hash" thr=1 idx=115  Seed[0]=0x03 len=33
```

`DecodeMS1` does refuse. **It is never called on the path that matters.**
`sysw/classify.go:116-125` `isStrictMs1` is `len ≤ 90 && HasPrefix "ms1" &&
codex32.New(record) == nil` — no prefix test, no `DecodeMS1`. So on a flashed
SH2 today a preimage string in a payload:

1. classifies as `ClassCodex32Secret` — indistinguishable from the entr-32
   string above, which classifies identically;
2. is offered in the unlock secret session labelled **`"ms1"`**
   (`gui/unlock_session.go:65-77`, `unlockSecretLabel`);
3. on "Cut" reaches `unlockEngraveCodex32` (`gui/unlock_session.go:164-200`),
   which calls `codex32.New` and `backup.EngraveSeedString` — **and never calls
   `DecodeMS1`**. The device cuts a seed plate titled `hash`.

No refusal occurs anywhere on that path. This also contradicts L7's consequence
as recorded in the brainstorm — *"It never stores, shows, engraves or sources a
preimage"* — which §13 declares out of scope without noticing the device already
has a path that does exactly that.

Two further measurements bound the claim honestly:

- **Round 0's Q4 was half right and the spec dropped the half that mattered.**
  Q4 said *"A `0x03` string reaching `me`'s classifier … would be offered at seed
  entry, then refused at decode — a dead end."* §9 compressed that to "never a
  seed". On the fork the offer is not followed by a decode at all.
- **The Rust host is a refusal today, and stops being one on the version bump.**
  `me` pins `ms-codec = "0.7"` (`crates/me-cli/Cargo.toml:53`), and
  `seal::record::validate_record` (`seal/record.rs:151-179`) maps **any**
  `ms_codec::decode` success to `RecordKind::Ms`, whose `is_secret()` is `true`.
  It does not dispatch on the prefix byte at all. So MIGRATION item 1 (*"Readers
  that **dispatch on the prefix byte** MUST…"*) does not describe `me`, and item 3
  (*"sweep your `_ => unreachable!` arms"*) does not either — `me` has no
  `Payload` match anywhere (`grep -rn 'Payload::' crates/me-cli/src` returns only
  its own unrelated `sysw::Payload`). The MIGRATION section names two hazard
  shapes, and the one downstream Rust reader that exists has neither. Its actual
  shape is **"decode succeeded, therefore this is a seed."**

Reader table, all measured, none inferred:

| reader | on `ms10hashsq0p7…` | §9's claim |
| --- | --- | --- |
| `ms` 0.16.0 / ms-codec 0.7 | `error: reserved-prefix byte was 0x03, expected 0x00`, exit 2 | holds |
| `me sysw pack` (0.7.0 binary) | refuses: *"VALID BIP-93 … not a constellation `ms1` record … the 4-character id must be `entr`"* | holds |
| mnemonic-toolkit (ms-codec 0.7) | `ms_codec::decode` refuses; `slot_ms1.rs:82` catch-all is an `Err`, not a panic | holds |
| **fork `839fa5a` (pre-H2 SH2)** | **`ClassCodex32Secret`; engraved as an "ms1" secret plate; `DecodeMS1` never reached** | **FALSE** |

**Classification: refusal + documentation.** §9's guarantee sentence must be
corrected to the measurement; MIGRATION must name the *"decode succeeded ⇒ seed"*
shape alongside the prefix-dispatch one; and the fork's `isStrictMs1` /
`seal.Classify` prefix test must be stated as a prerequisite that lands **before
any preimage plate exists in the wild**, not as part of H2 after it.

---

### C-4 — `--hex` admits a wallet seed's entropy as a preimage, and every warning §7 calls "the defence" is phrase-keyed

**The loss:** (c) a secret exposed to everyone → (a) every key under that seed.

`--hex` takes *"an existing X, exactly 32 bytes (64 hex characters)"* (§4.1) with
no statement about where X may come from. §7's two warnings are **method-keyed**
(sha256 always; hardened under 20 characters) and §4.2 says `--method` is refused
outright with `--hex`, so under this source **neither warning can fire**. The
reuse lines do print (§4.4), but their text is about a phrase:

> *"One phrase per policy. … Never use **this phrase** as a passphrase or a
> password anywhere else — a spend publishes the preimage, and anyone can then
> test guesses at **the phrase** itself."*

Under `--hex` there is no phrase, the card's method line reads *"preimage
supplied"*, and the operator reads that paragraph as inapplicable. The one
sentence that would bind — *a spend publishes this 32-byte value in the clear,
forever* — is stated nowhere as a property of X itself; §3.7 has it, and §7's
copy converts it into a premise about guessing the phrase.

The material is one command away and the constellation formats it in exactly the
shape `--hex` wants. Measured with the shipped `ms`:

```sh
$ ms decode ms10entrsqrp7jaf9gsjjpkjvll2l274w8a388xgqzlewp73scptwxgtjugspvl8sagkqdva662
entropy: c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016
phrase: sentence entry enable marriage faith honey crop wide voice step more shaft …
```

Sequence:

1. Operator needs a preimage. A plate is ~21 minutes on the machine and they
   already hold a 24-word seed on steel. L8 has trained them that a preimage is
   *"32 bytes (64 hex characters)"* — which is exactly the width of that seed's
   entropy.
2. `ms decode <their seed plate>` → `entropy: c3e97525…2016`.
3. `ms hashlock --hex c3e97525…2016 --out preimage.ms1` — accepted. The card
   prints the digest, `preimage supplied`, and phrase-shaped reuse copy.
4. They fund the policy. One spend of the hash path publishes X in the witness.
5. X **is** the wallet's master entropy. Every key ever derived from that seed —
   in every wallet, including ones with no hashlock at all — is now public.

The direction the folded r2 review recorded (C-2 there) is the opposite one: a
preimage becoming *seed entropy* on the device, in H2. This is a preimage
*sourced from* seed entropy, on the host, in H1, and the r2 review's own Q5 lists
the misuse sequences its copy does not prevent as C-1/C-3/C-4 — none of them
this. Not previously recorded.

**Classification: warning, unconditional on `--hex`.** A refusal is not available
(the tool cannot tell one 32-byte value from another), which is precisely why
§7's own logic — *"the copy is the defence"* — applies here and is missing. The
wrong outcome is catastrophically worse than saying nothing.

---

### I-1 — the §6 guard edit binds one of the new verb's THREE material channels

§6.4 requires the verb's `Source` be built `.on("--hashlock-phrase")`, *"so an
admitted value arrives through the side channel rather than from whatever stdin
holds"*, with the gate *"the same invocation with stdin at `/dev/null` still
derives from the flag's value."* Correct — and incomplete. `ms hashlock` has
three material channels, and `substitute()` rewrites **all** of them:

- `--hex` is **already** in `SECRET_FLAGS` (`argv_guard.rs:86`,
  `["--phrase", "--hex", "--ms1", "--passphrase"]`), so `--allow-argv-secret`
  moves its value into `ADMITTED["--hex"]` and the argv becomes `--hex -`
  (`argv_guard.rs:283-297`).
- the positional `<ms1>` is caught by layer 2's shape test (`is_ms1_shaped`) and
  moved into `ADMITTED[CH_POSITIONAL]`, the token replaced by `-`
  (`argv_guard.rs:318-327`).

Every existing verb binds each channel it can receive:
`decode.rs:64`, `inspect.rs:58`, `verify.rs:72`, `derive.rs:387`
(`.on(CH_POSITIONAL)`); `encode.rs:151`, `derive.rs:418` (`.on("--hex")`).
Without the two extra bindings, the spec's own gate fails:

```sh
ms hashlock --allow-argv-secret ms10hashsq0p7… < /dev/null
```

reads the substituted `-` from stdin (empty) instead of the admitted plate
string. The route the guard's refusal advertises is exactly the one §4.1 says
must exist (review C-1), and it breaks under the override.

**Classification: refusal / spec completion.** §6 becomes a six-part edit:
`.on("--hashlock-phrase")`, `.on("--hex")`, `.on(CH_POSITIONAL)`, each with its
own `stdin < /dev/null` gate.

---

### I-2 — `me sysw pack --in -` does not exist; §12.6 is an acceptance gate that cannot pass

The spec names this command three times: §4.4's stdout bullet (*"the record
`me sysw pack --in -` consumes"*), §4.4's `--out` rationale, and **§12.6, an
acceptance criterion**. Measured against the installed `me` and confirmed in the
tree (`crates/me-cli` 0.8.0):

```sh
$ printf 'hash:a8a8…a8a8\n' | me sysw pack --in - --out /tmp/p1.blob
me: -: No such file or directory (os error 2)
exit=2
```

`crates/me-cli/src/main.rs:2486-2496`: `--in` goes straight to
`std::fs::read_to_string(p)`. `-` is the stdin sentinel **only as a positional
record** (`main.rs:2614-2645`), and the code says so in its own comment:
*"`-` stays a clap or ENOENT error on all four"* other surfaces. The working
spellings are `… | me sysw pack -` or `… | me sysw pack` (no positional, no
`--in`).

An operator following §4.4 literally gets exit 2 for a reason unrelated to what
they were doing; §12.6 run literally fails before it tests anything. That is the
"a gate that has never been executed is a hypothesis" shape.

(`hash:` records themselves are fine: `sysw/composer_records.rs:29-30` and its
vector rows accept `hash:` + 64 hex, so the chain works once the command is
spelled correctly.)

**Classification: documentation only** for §4.4, **gate correction** for §12.6.

---

### I-3 — zero sources is undefined, and it is the `mt`-hang class the spec fixes for exactly one channel

§4.1 specifies "exactly one source per invocation" and "two sources is exit 64".
It never says what `ms hashlock` alone does. The two plausible implementations
diverge, and the existing verbs do not agree with each other:

```sh
$ ms decode < /dev/null ; echo $?      # defaults to stdin
error: string length 0 not in v0.1 set [50, 56, 62, 69, 75]
1
$ ms encode < /dev/null                # required ArgGroup, usage error
error: the following required arguments were not provided:
  <--phrase <PHRASE>|--hex <HEX>>
```

`ms hashlock` has a positional `<ms1>`, so the `encode` shape is awkward and the
`decode` shape is the likelier implementation — in which case bare `ms hashlock`
at a terminal **blocks with no prompt**. §4.3 fixes precisely this for
`--hashlock-phrase-stdin` (*"prints one prompt line to stderr … rather than
blocking silently (r2 review M-7; the constellation's recorded `mt` finding)"*)
and the fix does not extend to the invocation a first-time operator actually
types. Worse if stdin defaults to the ms1 channel: the operator pastes their
phrase at the invisible prompt, gets an ms1 parse error naming a channel they did
not think they used, and the phrase is now in scrollback.

**Classification: refusal.** Zero sources exits 64 listing the five sources —
the same treatment two sources already gets.

---

### I-4 — §1's reachable-length claim is wrong; three of the four named lengths never reach prefix dispatch

§1: *"BIP-93's bracket admits 16..46 payload bytes, so 16, 32, 34 and 46 are all
reachable and each is a vector row (§8)."* Measured, building each `0x03` payload
with `Codex32String::from_seed` and feeding it to ms-codec 0.7's `decode`:

```
0x03 payload 16 bytes ->  48 chars : string length 48 outside v0.1 set [50, 56, 62, 69, 75]
0x03 payload 32 bytes ->  74 chars : string length 74 outside v0.1 set [50, 56, 62, 69, 75]
0x03 payload 33 bytes ->  75 chars : reserved-prefix byte was 0x03, expected 0x00
0x03 payload 34 bytes ->  77 chars : reserved-prefix byte was 0x03, expected 0x00
0x03 payload 46 bytes ->  96 chars : string length 96 outside v0.1 set [50, 56, 62, 69, 75]
```

ms-codec gates the **string length** before it ever reads the prefix byte, so
16, 32 and 46 are refused by `UnexpectedStringLength`. Only 34 reaches the point
where `PreimageLengthMismatch` could fire. Exhaustive sweep of payload byte
counts 2..47, the ones that reach prefix dispatch:

```
17 (50 ch)  18 (51 ch)  21 (56 ch)  22 (58 ch)  25 (62 ch)
26 (64 ch)  29 (69 ch)  30 (70 ch)  33 (75 ch)  34 (77 ch)
```

So the true reachable set of *wrong* lengths is
**{17, 18, 21, 22, 25, 26, 29, 30, 34}** — nine values, of which the spec names
one. The consequence is not a panic (the spec's `<[u8; 32]>::try_from(&data[1..])`
is correct and I could not construct one): it is that a length row written from
§1's list either cannot be satisfied at all, or asserts "refused" and passes on
the **wrong** error — a gate that reports green without ever exercising the
refusal it names. The nearest-miss row that actually matters, 34 bytes at 77
characters, is reachable only through the *mnem* length set and is the one an
off-by-one would hit.

**Classification: correction to §1's normative sentence** (the vector rows
themselves belong to the tests lens; the false reachability claim is what
generates them).

---

### M-1 — the `--random` card asserts a plate that cannot exist when it prints

§7: under `--random` the card says *"No phrase exists, so nothing can be guessed,
and nothing can be remembered. **This plate is the only copy.**"* At the moment
that line prints, there is no plate — there is a `0600` file (or a JSON object on
a pipe). The one warning standing between the operator and C-1/C-2 names the
wrong artifact, and names the safest one. Suggested shape: *"the file you just
wrote is the only copy until you cut the plate."*

### M-2 — the same 64-hex input gets two different remedies depending on the channel

§4.3 refuses a 64-character all-hex phrase *"naming `--hex`"*. On
`--hashlock-phrase` that refusal never runs: `find_argv_material`'s layer 1
(`argv_guard.rs:342-350`) matches `SECRET_FLAGS` before clap and returns
`flag_class("--hashlock-phrase")` with a `sed` history-purge block. Correct and
safe, but the operator never sees the `--hex` remedy §4.3 promises, and two
normative statements describe the same input differently. Reproduction:
`ms hashlock --hashlock-phrase c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016`.
Documentation only — §4.3 should say the refusal is on the stdin channel.

### M-3 — §5's verb table omits `repair`, the only verb that runs BCH correction on a plate

§5 enumerates decode / inspect / combine / derive / verify / encode / split.
`ms repair` is missing, and it is the verb an operator reaches for with a
scratched preimage plate. Verified benign — `cmd/repair.rs:141` binds
`(_tag, _payload, corrections)` and discards the payload, so no new arm is needed
and nothing panics — but §1's *"Misreads cannot convert one kind into another"*
is a claim about `decode_with_correction`, which is repair's engine, and repair
is never named in the section that would carry it.

### M-4 — the inverted polarity survives a `0644` log (secret-handling, non-gating by the 2026-08-27 ruling)

`ms hashlock --hashlock-phrase-stdin < phrase.txt 2>> ~/engrave.log` lands the
preimage hex and the preimage ms1 in a `0644` file. §4.4's first-line label is
the right mitigation and it is present. The louder shapes fail loudly and are
fine: `2>&1 | me sysw pack -` makes `pack` refuse the card lines as
unclassifiable, exit 4. Recorded with its reproduction per the ruling; not
blocking.

### M-5 — `hash:` requires 64 **lowercase** hex and §4.4 says only "64 hex"

`sysw/composer_records.rs` vector rows: `hash-uppercase` classifies `Unknown`
with *"record 0: hash: must be exactly 64 hex characters"*. §4.4 and §12 never
state the case, so an implementation using `{:X}` produces a record `me sysw
pack` silently refuses. One word.

### N-1 — a grouped preimage pasted as a phrase gets the length refusal, not the `--in` one

§4.3 refuses an ms1-shaped phrase *"naming `--in`/`-`"* via `is_ms1_shaped`,
which strips display separators first. But the 100-character cap is also in play:
a 75-character ms1 at `--group-size 2` is 112 characters, so the cap refuses it
first with *"at most 100 characters"* and the operator is never pointed at
`--in`. Group sizes 3, 4 and 5 (89–99 characters) reach the right refusal. Order
the two checks ms1-shape-first, or note it.

---

## The journeys, step by step

### 1. Phrase → plate → spend

- `ms hashlock --hashlock-phrase-stdin < phrase.txt` — the phrase is read
  byte-verbatim, X and H derive, `hash:H` on stdout, the card on stderr. **No
  divergence** at this step; the byte-verbatim reader and the character count on
  the card are both right, and I could not construct a phrase that survives the
  cap, the ASCII rule and the two refusals and still derives a surprising X.
- **`--out X.txt` → DIVERGENCE, C-2** (silent truncation, irreproducible under
  `--random`).
- The `hash:` record into `me sysw pack` — **DIVERGENCE, I-2**: the command the
  spec names three times does not exist.
- The composer's `Which hash?` — rows are `hash <i> <first 8>..<last 8>`
  (`gui/composer_hash.go`), fed from `sysw.ParseHashRecord`'s 64-lowercase-hex
  parse. The record shape matches; see M-5 for the case.
- **A year later, from the plate alone:** `ms hashlock --in X.txt` returns
  `hash:H` on stdout and X on the card; `ms decode X.txt` returns X on stdout
  independently, so the chain closes even with the card suppressed. **No
  divergence.**
- **From the phrase alone, method line lost:** §7's *"try each method that
  shipped with the version named on this card"* is executable — two derivations,
  compare each digest with the policy's. It is an eyeball comparison of 64 hex
  characters, which is the W-5 complaint that started this cycle, but
  `[ "$(ms hashlock --in plate)" = "hash:$H" ]` closes it in one line and the
  spec's copy does name the procedure. **No finding** — I tried to construct a
  case where the comparison is unavailable and could not: H is always in the
  policy the operator is trying to spend.

### 2. The wrong slot

| pasted | caught by | remedy executable? |
| --- | --- | --- |
| a 64-hex preimage as a phrase (stdin) | §4.3's hex refusal | yes — `--hex` |
| a 64-hex preimage as a phrase (argv) | the argv guard, layer 1 | **different remedy — M-2** |
| a plate string as a phrase | `is_ms1_shaped`, separators stripped | yes — `--in`/`-`, except **N-1** |
| a grouped plate at `--group-size 2` | the 100-char cap fires first | **no — N-1** |
| a phrase where a plate goes (`--in phrase.txt`) | an ms1 parse error | **no remedy stated** — the spec gives the phrase→ms1 direction a named remedy and not the reverse; documentation only |
| the W-5 by-hand digest under the default method | nothing at the tool | MIGRATION §9.4 + the manual + F-465's hint; **documentation only, and correctly identified as such** |

### 3. Polarity

Constructed four shell lines a careful operator writes. Three fail loudly
(`2>&1 | me sysw pack -` → exit 4; `> rec.txt 2>&1` → `pack` refuses the card
lines; `--json > file` → intended). One lands the preimage in a `0644` file:
**M-4**, non-gating by the 2026-08-27 ruling, recorded with its reproduction.
§4.4's first-line label is the correct mitigation and is present. The
*non*-secret-handling consequence of the same inversion is C-1, filed above.

### 4. The two methods

- Can the operator ever not tell which method made their digest? Only if the
  method line is lost, and §7 names the recovery. The plate carries no method and
  the spec says so (*"it is on no plate"*). The `--in` card reading
  *"preimage supplied"* is right, not a gap. **No divergence.**
- `--random`'s *"nothing can be remembered"*: what the operator holds if the
  plate is lost is **nothing**, and the spec does say so before the fact — but it
  says it about a plate that does not yet exist (**M-1**), and the two ways the
  file itself disappears are **C-1** and **C-2**.

### 5. The reader

Four readers, all executed; the table is in **C-3**. Three hold, one is false,
and it is the one §9 names explicitly (*"every SH2 flashed before H2"*). The
Rust host's refusal is real today and is a version bump away from inverting,
through a code shape MIGRATION does not describe.

### 6. The KDF choice as a design

**No new finding.** Within L4/L5/L13 I looked for an attack the two folded
reviews did not record — they have the shared-table cost (r0 I-1 → L13) and the
brainwallet rate (r0 C-1 → L12). Checked and found nothing new: the fixed salt
`ms-hashlock-v1` is domain-separated from BIP-39's `"mnemonic"` and cannot
collide with `me`'s 16-byte seal salt under `S || INT(i)` (length differs);
`digest`'s non-zeroization is correct (H is public by construction);
`preimage_hardened`/`preimage_sha256` returning `Zeroizing` and the `Payload`
variant wrapping `Zeroizing<[u8; 32]>` close the r0 M-1 shape; `--random` draws
from `getrandom` with a failing-closed `.expect` (`shares.rs:43`). The one
exploitable thing I did find in this area is about **where X comes from**, not
about the KDF: **C-4**.

---

## Counts

**4 Critical — C-1, C-2, C-3, C-4.
4 Important — I-1, I-2, I-3, I-4.
5 Minor — M-1 … M-5. 1 Nit — N-1.**

Divergence classification tally: **refusal** 4 (C-1, C-2, I-1, I-3);
**warning** 1 (C-4); **refusal + documentation** 1 (C-3);
**documentation only** 4 (I-2, M-2, M-3, M-5); **copy correction** 2 (M-1, N-1);
**correction to a normative claim** 1 (I-4); **not our concern** 0.
M-4 is recorded, non-gating by the 2026-08-27 operator ruling.

Two items need the operator rather than an editor: **C-1**'s remedy narrows
**L21** (which names `--json` as a sufficient channel), and **C-3** moves the
fork's classifier prefix test from H2 to a prerequisite of H1.
