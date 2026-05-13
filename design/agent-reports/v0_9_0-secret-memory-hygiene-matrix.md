# v0.9.0 Cycle A — secret-memory-hygiene matrix (mnemonic-secret)

**Cycle:** OWNED-buffer secret-memory hygiene v0.9.0 Cycle A.
**Cycle authoritative reference:** the toolkit matrix file at
`/scratch/code/shibboleth/mnemonic-toolkit/design/agent-reports/v0_9_0-secret-memory-hygiene-matrix.md`
is the cross-repo authority — §0 cross-repo coverage table,
§0.5 "what is NOT closed" prose, §3 FOLLOWUPS visibility list,
and §4 Cycle B carry-overs all live there. This sibling matrix is
scoped to ms-codec + ms-cli rows.

**Cycle reports (ms-secret):**
  - Phase 0 companion: `mnemonic-toolkit/.../v0_9_0-phase-0-spec-plan-r1.md`
  - Phase 2: `v0_9_0-phase-2-zeroize-r{1,2}.md`

## §0 Cross-repo cite

See `mnemonic-toolkit/design/agent-reports/v0_9_0-secret-memory-hygiene-matrix.md`
§0 for the full cross-repo coverage table. Summary:

- mnemonic-toolkit branch `v0_9_0-phase-2-zeroize` @ `863f18a` —
  closes ~27 OWNED toolkit rows + 9 argv flag-rows + 32 SAFETY
  anchors.
- mnemonic-secret (this repo) branch `v0_9_0-phase-2-zeroize` @
  `123dea3` — closes 4 ms-codec OWNED rows + 10 ms-cli OWNED rows
  (incl. 3 clap-field rows via consume + `mem::take` + wrap
  pattern).
- descriptor-mnemonic + mnemonic-key: NO Cycle A participation
  (xpub-only material; SPEC §3 OOS-md-mk).

## §0.5 What this cycle does NOT close (ms-secret side)

Inherits all 6 classes from the toolkit matrix §0.5. ms-secret-
specific residual gaps:

- **`Payload::Entr(Vec<u8>)` public-API shape stays unwrapped.**
  Widening the public type to `Zeroizing<Vec<u8>>` is a breaking
  change deferred indefinitely per SPEC §3 OOS-2. ms-codec
  internal-only Zeroizing wraps minimize the un-scrubbed lifetime
  inside encode/decode; the public boundary is the
  caller-wrap-contract documented in `crates/ms-codec/src/payload.rs`.
  FOLLOWUP: `ms-codec-payload-zeroize-public-api` (tier
  `v1+`).

- **`bip39::Mnemonic` interior residue (upstream-blocked).** Same
  as toolkit matrix; affects ms-cli at `cmd/encode.rs::run`,
  `cmd/decode.rs::run`, `cmd/verify.rs::run`. SAFETY anchors at
  each site cite `rust-bip39-mnemonic-zeroize-upstream` (companion
  entry in mnemonic-toolkit's tracker).

- **`codex32::Codex32String` internal residue (upstream-blocked).**
  The upstream `rust-codex32` v0.1 holds payload bytes internally
  in a `Codex32String` type that does not zeroize on drop.
  `envelope::package`'s `Zeroizing<Vec<u8>>` local scrubs the
  `data` buffer when the function exits, but the bytes that
  `Codex32String::from_seed` copied into its private buffer
  during construction are NOT scrubbed (those live for the
  `Codex32String`'s lifetime, which extends until the caller's
  binding drops). Mitigation is lifetime minimization at the
  ms-codec layer + caller-wrap discipline. FOLLOWUP:
  `rust-codex32-zeroize-upstream` (tier `external`).

## §1 Survey §1 OWNED-buffer row coverage

### ms-codec (4 rows)

| Row | Site | Status | Evidence |
|-----|------|--------|----------|
| `envelope::discriminate()` payload buffer | `envelope.rs:115-117` (post-fold line) | CLEAR | `payload_with_prefix: Zeroizing<Vec<u8>>` typed local. |
| `envelope::package()` data buffer | `envelope.rs:141-149` | CLEAR | `let mut data: Zeroizing<Vec<u8>>` typed local. |
| `decode()` Payload::Entr intermediate | `decode.rs:46-50` | CLEAR | `scrubbed: Zeroizing<Vec<u8>>` typed local before public Payload::Entr return. |
| `payload.rs` caller-wrap contract | `payload.rs:16-30` | CLEAR (docs) | doc-comment block locks the "Caller MUST wrap" invariant. |

### ms-cli (10 rows, post-R1 C-2 + R1 N-1 folds)

| Row | Site | Status | Evidence |
|-----|------|--------|----------|
| `EncodeArgs::phrase` clap-field | `cmd/encode.rs:57-60` | CLEAR | `std::mem::take(&mut args.phrase).map(Zeroizing::new)` (consume + wrap). |
| `EncodeArgs::hex` clap-field | `cmd/encode.rs:61-62` | CLEAR | same pattern: `std::mem::take(&mut args.hex).map(Zeroizing::new)`. |
| `VerifyArgs::phrase` clap-field | `cmd/verify.rs:46-48` | CLEAR | same pattern: `std::mem::take(&mut args.phrase).map(Zeroizing::new)`. |
| `parse::read_phrase_input` returned String | `parse.rs:36-43` | CLEAR | return type `Result<Zeroizing<String>>`. |
| `parse::read_stdin` raw buffer | `parse.rs:49-58` | CLEAR | `let mut buf: Zeroizing<String> = Zeroizing::new(String::new())`. |
| `cmd/encode::run` locals (phrase + entropy) | `cmd/encode.rs:63-86` | CLEAR | `let (entropy, language_for_card): (Zeroizing<Vec<u8>>, _) = ...`. SAFETY anchor at L68. |
| `cmd/encode` entropy buffer into Payload | `cmd/encode.rs:88-92` | CLEAR | `Payload::Entr((*entropy).clone())` direct (R1 N-1 fold — entropy_for_codec intermediate removed). |
| `cmd/decode::run` locals | `cmd/decode.rs:39-66` | CLEAR | `let entropy: Zeroizing<Vec<u8>>` + `let phrase: Zeroizing<String>` typed locals. |
| `cmd/verify::run` entropy + supplied + derived | `cmd/verify.rs:60-80` | CLEAR | `let entropy: Zeroizing<Vec<u8>>` + `let supplied_str: Zeroizing<String>` + `let derived_str: Zeroizing<String>`. |
| `cmd/verify` success-log derived_mnemonic | `cmd/verify.rs:80` | CLEAR | `derived_str: Zeroizing<String>` covers the to_string() output. |

### ms-cli third-party residue (PARTIAL-3RD-PARTY)

3 production `bip39::Mnemonic` call sites in ms-cli carry SAFETY
anchors:

| Site | SAFETY cite |
|------|-------------|
| `cmd/encode.rs:69` (`Mnemonic::parse_in`) | `rust-bip39-mnemonic-zeroize-upstream` |
| `cmd/decode.rs:60` (`Mnemonic::from_entropy_in`) | `rust-bip39-mnemonic-zeroize-upstream` |
| `cmd/verify.rs:73-77` (`Mnemonic::parse_in` + `Mnemonic::from_entropy_in`) | `rust-bip39-mnemonic-zeroize-upstream` |

ms-cli does NOT construct `Xpriv` or `SecretKey` — those are
toolkit-only (no curve operations in this crate).

## §2 Survey §5 argv-leakage flag-row coverage (ms-cli)

Per survey §5 ms-cli subtable, all 5 ms-cli flag-rows had stdin
routes BEFORE Cycle A (Phase 1 R1 I-2 fold confirmed: no Phase 1
argv work needed in ms-secret). Status table preserved here for
completeness:

| Flag-row | Stdin route | Status |
|----------|-------------|--------|
| `ms encode --phrase <PHRASE>` | YES (`--phrase -`) | CLEAR — pre-cycle |
| `ms encode --hex <HEX>` | YES (`--hex -`) | CLEAR — pre-cycle |
| `ms verify --phrase <PHRASE>` | YES (`--phrase -`) | CLEAR — pre-cycle |
| `ms decode <MS1>` positional | YES (`-` or omit) | CLEAR — pre-cycle |
| `ms verify <MS1>` positional | YES (`-` or omit) | CLEAR — pre-cycle |

**No ms-cli argv-in-cycle advisory was added.** The Phase 1
secret-in-argv advisory is toolkit-only per cycle scope. A
companion advisory for ms-cli is a Cycle B candidate.

## §3 FOLLOWUPS visibility cite

See toolkit matrix §3 for the full 14-SPEC-OOS + 4-cycle-surfaced
list. Entries that have ms-secret-side FOLLOWUPS:

- `ms-codec-payload-zeroize-public-api` (v1+ — open) — SPEC §3
  OOS-public-payload deferral. Entry in
  `mnemonic-secret/design/FOLLOWUPS.md`.
- `ms-codec-doc-example-zeroize-consistency` (v1+ — open) — SPEC
  §3 OOS-7 deferral. Entry in `mnemonic-secret/design/FOLLOWUPS.md`.
- `ms-cli-decode-emit-zeroize-intermediate` (v1+ — open) — SPEC
  §3 OOS-decode-stdout deferral. Entry in
  `mnemonic-secret/design/FOLLOWUPS.md`.
- `rust-codex32-zeroize-upstream` (external — open) — surfaced
  during Phase 2 ms-codec envelope.rs work. Entry in
  `mnemonic-secret/design/FOLLOWUPS.md`.
- `md-mk-private-key-surface-watch` (cross-repo — open monitoring) —
  cross-repo companion entry in `mnemonic-secret/design/FOLLOWUPS.md`
  + toolkit + md + mk.
- `secret-memory-hygiene-v0_9-cycle-a` (cross-repo — open) — cycle
  meta-entry; closes when Phase E ships. Entry in
  `mnemonic-secret/design/FOLLOWUPS.md`.

## §4 Cycle B carry-overs (ms-secret participation)

Cycle B (mlock) is **toolkit-only** per Phase 0 R3 SPLIT-CYCLE
scope. ms-secret participates in Cycle B only if a future SPEC
adds mlock to the ms-cli read paths (currently OOS).

## §5 Cycle-close gates (ms-secret-side)

- ✓ ms-codec 4 OWNED rows wrapped (Phase 2).
- ✓ ms-cli 10 OWNED rows wrapped, incl. 3 clap-field rows via
  consume + `mem::take` + wrap pattern (Phase 2).
- ✓ `crates/ms-codec/tests/lint_zeroize_discipline.rs` green.
- ✓ `crates/ms-cli/tests/lint_zeroize_discipline.rs` green.
- ✓ This matrix file in place (Phase 3 deliverable).
- ✓ Cross-repo cite to the toolkit canonical matrix in §0.

Phase E (release rollup) is the cycle-close step. Tag plan:
`ms-codec-v0.1.3` + `ms-cli-v0.1.X+1` in lockstep with
`mnemonic-toolkit-v0.9.2`.
