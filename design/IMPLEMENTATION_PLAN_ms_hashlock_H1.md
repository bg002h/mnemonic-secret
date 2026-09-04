# ms hashlock (H1) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**STATUS: DRAFT — not yet build-gated, not yet R0-reviewed.** The gate is
`scripts/plan-build-gate-ms.sh` (Task 0); R0 = fidelity (opus) + tests (sonnet,
mutation beside every test); re-validate immediately before the implementer.
**Baseline:** mnemonic-secret `master` `d4d6771` (ms-codec 0.7.0, ms-cli
0.17.1; the spec at R0 GREEN). Every line citation below was taken at that
SHA; `scripts/plan-staleness-check.sh` (mnemonic-engrave) can diff them.

**Goal:** Ship `ms-codec 0.8.0` and `ms-cli 0.18.0`: the `0x03` preimage kind
with id `hash`, `ms_codec::hashlock` (two derivations, a random source, the
digest), and the `ms hashlock` verb that emits the `hash:` record, the preimage
plate string and the card — exactly as `design/SPEC_ms_hashlock.md` (R0 GREEN)
specifies.

**Architecture:** The kind and its derivation live in ms-codec; ms-cli's verb
is thin (flags, private channels, refusal text, output shape). New behaviour
goes in NEW files wherever the boundary allows — `crates/ms-codec/src/hashlock.rs`,
`crates/ms-cli/src/hashlock_phrase.rs`, `crates/ms-cli/src/cmd/hashlock.rs`,
and every new test — so the build gate can extract and compile them; the
unavoidable edits to existing files (the codec's dispatch, accept set and
enums; the CLI's guard, registry and the four catch-all arms) are FRAGMENTS,
listed per task and hand-wired by `scripts/plan-handwire-ms-hashlock.py` for
the gate.

**Tech Stack:** Rust 1.85.0 (`rust-toolchain.toml`), `pbkdf2 0.12` (feature
`hmac` only) + `sha2 0.10` spelled as `me` spells them, `getrandom 0.3`
(already a ms-codec dependency), `zeroize 1.8`, `clap 4` derive,
`assert_cmd` for CLI tests, `cargo nextest`.

**Spec:** `design/SPEC_ms_hashlock.md` at `d4d6771` (fourteen sections; §14
is the citation table this plan inherits). The brainstorm's rulings L1–L23 are
in mnemonic-engrave `design/BRAINSTORM_hashlock_phrase.md`.

## Global Constraints

Copied from the spec; every task's requirements include these.

- **Wire:** payload `[0x03][X:32]`, 33 bytes, 75-character string; singles carry
  id `hash`; readers dispatch on the prefix byte and CHECK the id against it
  (spec §1 rules 1–3). `0x01` stays unallocated.
- **Derivation:** `HASHLOCK_SALT = b"ms-hashlock-v1"`, `HASHLOCK_ITERATIONS =
  100_000`, `HASHLOCK_DKLEN = 32`; hardened = PBKDF2-HMAC-SHA256; sha256 = one
  SHA-256 of the phrase bytes; digest = SHA-256 of X (§2). No `--salt` flag.
- **Phrase rule, host and device identical:** non-empty, bytes `0x20..=0x7E`
  only, at most `HASHLOCK_PHRASE_MAX_CHARS = 100`, used byte-verbatim; exactly
  one trailing `\r?\n` stripped on stdin and nothing else; the ms1-shape test
  runs BEFORE the length cap and uses ONE shared normalising predicate; a
  64-hex phrase (either case) is refused naming `--hex` (§4.3).
- **Sources:** exactly one of `--hashlock-phrase`, `--hashlock-phrase-stdin`,
  `--hex`, `<ms1>`/`-`/`--in`, `--random`; zero → exit 64 listing five; two →
  exit 64; `--method` with a supplied X → exit 64; `--random` requires
  `--out FILE` (`--json` alone does not satisfy it) and its `--out` is
  `create_new` (§4.1).
- **Outputs:** stdout = one line `hash:<64 lowercase hex>` (never suppressed
  by `--out`; replaced by one JSON object under `--json`); `--out` = the
  preimage ms1, `0600`; stderr card whose FIRST line names it as carrying the
  preimage; `--json` keys and their omission rules (§4.4).
- **Copy:** L8 "32 bytes (64 hex characters)" on every size refusal/help line;
  the sha256 brainwallet line always; the hardened under-20 line; the
  unconditional `--hex` line; the reuse lines; the method line's write-it-down
  instruction for phrase sources only; the `--random` card names the FILE (§7).
- **Versions:** ms-codec `0.7.0 → 0.8.0`, ms-cli `0.17.1 → 0.18.0`, ms-cli's
  pin `=0.8.0`; corpus SHA re-pinned; MIGRATION 0.7 → 0.8 with the five
  items of §9; **H0 ships before the 0.18.0 release** (§9, §10).
- **Secret discipline:** `--hashlock-phrase` joins `SECRET_FLAGS`; refusals
  never echo the phrase; the phrase's VALUE is never normalised (§4.3, §6).
- **Process:** Rust-primary; persist reports verbatim in their own commit;
  a fold is authorship and re-earns the gate; stage paths explicitly.

---

## File Structure

New files (gate-extractable):

| path | responsibility |
| --- | --- |
| `crates/ms-codec/src/hashlock.rs` | the derivation module: constants, `preimage_hardened`, `preimage_sha256`, `preimage_random`, `digest` |
| `crates/ms-codec/tests/hashlock_derivation.rs` | derivation vectors, both methods, X and H; boundary phrases |
| `crates/ms-codec/tests/hashlock_kind.rs` | kind rows: encode/decode/inspect, accept set, id/prefix mismatch, length rows by door, share round trip, codeword distance, blocklist |
| `crates/ms-codec/tests/hashlock_repro.rs` | the `python3` + `openssl kdf` reproduction with LITERAL constants, three-way |
| `crates/ms-codec/tests/vectors/hashlock-v0.8.json` | the corpus (SHA-pinned in the CHANGELOG) |
| `crates/ms-cli/src/hashlock_phrase.rs` | the byte-verbatim stdin reader and the phrase validator |
| `crates/ms-cli/src/cmd/hashlock.rs` | the verb |
| `crates/ms-cli/tests/hashlock_sources.rs` | source arithmetic, `--random` gates, `--method` refusals, `--out` semantics |
| `crates/ms-cli/tests/hashlock_phrase_rule.rs` | the phrase rule through both channels; byte-exact rows; refusals in four spellings |
| `crates/ms-cli/tests/hashlock_outputs.rs` | stdout purity, card contents per source/method, warnings at boundaries, `--json` both variants |
| `crates/ms-cli/tests/hashlock_other_verbs.rs` | decode/inspect/combine/derive/verify/repair on the kind; the catch-all count |
| `crates/ms-cli/tests/hashlock_negative_content.rs` | the eleven refusals never echo the phrase or preimage |
| `scripts/plan-build-gate-ms.sh` | Task 0's gate |
| `scripts/plan-handwire-ms-hashlock.py` | applies the fragments below to a scratch copy for the gate |

Existing files edited (FRAGMENTS — hand-wired for the gate, NOT extractable):

| path | edit |
| --- | --- |
| `crates/ms-codec/src/consts.rs` | `PREIMAGE_PREFIX`, `TAG_HASH`, `VALID_PREIMAGE_STR_LENGTHS`; `RESERVED_ID_BLOCKLIST` += `hash` |
| `crates/ms-codec/src/tag.rs` | `Tag::HASH` |
| `crates/ms-codec/src/error.rs` | `PreimageLengthMismatch`, `TagKindMismatch`, `RandomnessUnavailable` + Display arms |
| `crates/ms-codec/src/payload.rs` | `Payload::Preimage`, `PayloadKind::Preimage`, `kind()`, `as_bytes()`, `validate()`, `single_tag()` |
| `crates/ms-codec/src/envelope.rs` | `dispatch_payload` arm, `payload_wire_bytes` arm, doc comments ×2 |
| `crates/ms-codec/src/decode.rs` | `allowed_for_kind`, the accept set + the tag/kind check |
| `crates/ms-codec/src/encode.rs` | the tag/kind check on emit |
| `crates/ms-codec/src/inspect.rs` | `InspectKind::Preimage` |
| `crates/ms-codec/src/lib.rs` | `pub mod hashlock;` |
| `crates/ms-codec/Cargo.toml` | `pbkdf2`, `sha2`; version 0.8.0 |
| `crates/ms-cli/src/argv_guard.rs` | `SUBCOMMANDS` 13, `SECRET_FLAGS` 5, `override_applies`, `flag_class`, `looks_like_ms1` |
| `crates/ms-cli/src/error.rs` | `CliError::Usage` (exit 64); mapping for the three new codec errors |
| `crates/ms-cli/src/main.rs` | `Command::Hashlock`, dispatch, `is_json_mode` |
| `crates/ms-cli/src/cmd/mod.rs` | `pub mod hashlock;` |
| `crates/ms-cli/src/lib.rs` or `main.rs` | `mod hashlock_phrase;` |
| `crates/ms-cli/src/cmd/decode.rs` | two `Payload::Preimage` arms |
| `crates/ms-cli/src/cmd/combine.rs` | one `Payload::Preimage` arm |
| `crates/ms-cli/src/cmd/payload_lang.rs` | the typed refusal arm |
| `crates/ms-cli/src/cmd/inspect.rs` | verdict rules 6/8/9/10 + `reason_text` + version line |
| `crates/ms-cli/Cargo.toml` | version 0.18.0; pin `=0.8.0` |
| `CHANGELOG.md`, `MIGRATION.md` | release records |
| `.github/workflows/rust.yml` | `test (ms-codec)` preflight + run-by-name step |

---

### Task 0: The build gate and the hand-wire script

**Files:**
- Create: `scripts/plan-build-gate-ms.sh`
- Create: `scripts/plan-handwire-ms-hashlock.py`

**Interfaces:**
- Consumes: this plan's ```rust blocks, anchored on the NEW paths in the
  File Structure table with the sibling gates' grammar (`Create <path>`,
  `Add to <path>`, `Replace <path>` in backticks; the next ```rust fence is
  the content).
- Produces: a scratch copy under `$TMPDIR/plan-build-gate-ms/` that builds,
  tests and lints; exit 3 on an empty extraction; a printed NOT-covered line.

- [ ] **Step 1: Write the gate script**

```bash
#!/usr/bin/env bash
# plan-build-gate-ms.sh -- compile and run the Rust that lives inside a
# mnemonic-secret implementation plan, so a fold that does not build never
# reaches a reviewer. Sibling of mnemonic-engrave's plan-build-gate-me.sh and
# descriptor-mnemonic's plan-build-gate-md.sh; same anchor grammar.
#
# WHAT IT DOES
#   1. Scratch copy of this repo's crates/, Cargo.toml, Cargo.lock,
#      rust-toolchain.toml and .cargo/, with CARGO_TARGET_DIR kept OUTSIDE the
#      copy. Builds under the repo's pinned toolchain (rust-toolchain.toml is
#      copied, so rustup selects it inside the copy).
#   2. Extracts every ```rust block that follows an anchor naming one of the
#      NEW files this plan may create:
#        crates/ms-codec/src/hashlock.rs
#        crates/ms-codec/tests/hashlock_*.rs
#        crates/ms-cli/src/hashlock_phrase.rs
#        crates/ms-cli/src/cmd/hashlock.rs
#        crates/ms-cli/tests/hashlock_*.rs
#      and every ```json block anchored on crates/ms-codec/tests/vectors/*.json.
#      Anchor grammar: a line containing `Create <path>`, `Add to <path>` or
#      `Replace <path>` with the path in backticks; the NEXT fence is that
#      file's content. Several blocks for one path concatenate in plan order;
#      `Replace` discards earlier blocks for that path.
#   3. Runs scripts/plan-handwire-ms-hashlock.py on the copy, which applies
#      the plan's FRAGMENTS (edits to existing files) -- the part no extractor
#      can assemble. The script refuses to run twice on one copy.
#   4. cargo build --workspace --all-targets --locked;
#      cargo nextest run --workspace --locked --no-fail-fast
#        -E 'binary(/hashlock/) | test(/hashlock/)';
#      cargo clippy --workspace --all-targets --locked -- -D warnings;
#      cargo fmt --check.
#   5. MEASURES spec §1's codeword distance: encodes one 33-byte payload under
#      id `entr` and id `hash` and asserts the Hamming distance between the two
#      strings exceeds 8 (twice BIP-93's 4-error bound). Prints the number.
#   6. The DOWNGRADE row: builds `ms` from the pre-H1 tree (git worktree at
#      $PRE_H1, default d4d6771) and feeds it the corpus's first 0x03 single;
#      asserts exit 2 and the text `reserved-prefix byte was 0x03`. A refusal,
#      never a panic (exit 101 is a failure of this step).
#
# EXTRACTING NOTHING IS A FAILURE, NOT A PASS (exit 3).
# NOT covered: the fragments' assertions are only as good as the hand-wire
# script; CLI tests run against the WIRED binary, so a fragment the script
# misses fails loudly there rather than silently; the toolkit manual chapter;
# CI workflow edits; H0 in the other two repos.
#
# Usage: scripts/plan-build-gate-ms.sh design/IMPLEMENTATION_PLAN_ms_hashlock_H1.md
set -euo pipefail
PLAN="${1:?plan path required}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="${MS_REPO:-$HERE}"
WORK="${TMPDIR:-/tmp}/plan-build-gate-ms"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-${TMPDIR:-/tmp}/plan-build-gate-ms-target}"
PRE_H1="${PRE_H1:-d4d6771}"
[ -f "$PLAN" ] || PLAN="$HERE/$PLAN"
[ -f "$PLAN" ] || { echo "no plan at $1" >&2; exit 2; }

echo "== 1 -- scratch copy of mnemonic-secret =="
rm -rf "$WORK"; mkdir -p "$WORK"
cp -r "$SRC/crates" "$SRC/Cargo.toml" "$WORK/"
[ -f "$SRC/Cargo.lock" ] && cp "$SRC/Cargo.lock" "$WORK/"
for f in rust-toolchain.toml clippy.toml rustfmt.toml .rustfmt.toml; do [ -f "$SRC/$f" ] && cp "$SRC/$f" "$WORK/"; done
[ -d "$SRC/.cargo" ] && cp -r "$SRC/.cargo" "$WORK/"
(cd "$WORK" && rustc --version && cargo --version)

echo "== 2 -- extract the plan's Rust and JSON =="
python3 - "$PLAN" "$WORK" <<'PY'
import re, sys, os
plan, work = sys.argv[1], sys.argv[2]
NEW = re.compile(r'^(crates/ms-codec/src/hashlock\.rs|crates/ms-codec/tests/hashlock_[a-z_]+\.rs|crates/ms-codec/tests/vectors/[a-z0-9.\-]+\.json|crates/ms-cli/src/hashlock_phrase\.rs|crates/ms-cli/src/cmd/hashlock\.rs|crates/ms-cli/tests/hashlock_[a-z_]+\.rs)$')
ANCHOR = re.compile(r'(Create|Add to|Replace) `([^`]+)`')
lines = open(plan, encoding='utf-8').read().split('\n')
files, order = {}, []
i = 0
while i < len(lines):
    m = ANCHOR.search(lines[i])
    if m and NEW.match(m.group(2)):
        verb, path = m.group(1), m.group(2)
        j = i + 1
        while j < len(lines) and not lines[j].startswith('```'):
            j += 1
        if j < len(lines):
            k = j + 1
            while k < len(lines) and not lines[k].startswith('```'):
                k += 1
            body = '\n'.join(lines[j+1:k]) + '\n'
            if verb == 'Replace' or path not in files:
                files[path] = body
                if path not in order: order.append(path)
            else:
                files[path] += body
            i = k
    i += 1
if not files:
    sys.stderr.write("\nplan-build-gate-ms: EXTRACTED NOTHING from %s\n  Refusing rather than reporting a pass on an empty extraction.\n" % plan)
    sys.exit(3)
for p in order:
    full = os.path.join(work, p)
    os.makedirs(os.path.dirname(full), exist_ok=True)
    open(full, 'w', encoding='utf-8').write(files[p])
    print("  wrote %s (%d lines)" % (p, files[p].count('\n')))
PY

echo "== 3 -- hand-wire the fragments =="
python3 "$HERE/scripts/plan-handwire-ms-hashlock.py" "$WORK"

echo "== 4 -- build, test, lint =="
cd "$WORK"
# The plan ADDS dependencies (pbkdf2, sha2), so the copied Cargo.lock cannot be
# --locked for the first build: resolve once here (the copy is throwaway; the
# implementer commits the real Cargo.lock change in Task 3), then every later
# step is --locked against the resolved file.
cargo build --workspace --all-targets 2>&1 | tail -3
cargo build --workspace --all-targets --locked 2>&1 | tail -1
cargo nextest run --workspace --locked --no-fail-fast -E 'binary(/hashlock/) | test(/hashlock/)' 2>&1 | tail -12
cargo clippy --workspace --all-targets --locked -- -D warnings 2>&1 | tail -3
cargo fmt --check 2>&1 | tail -3 && echo "fmt clean"

echo "== 5 -- codeword distance (spec §1) =="
cargo nextest run -p ms-codec --locked -E 'test(hashlock_kind::codeword_distance)' 2>&1 | grep -E "distance|PASS|FAIL" | head -3

echo "== 6 -- downgrade row against the pre-H1 tree ($PRE_H1) =="
PRE="${TMPDIR:-/tmp}/plan-build-gate-ms-pre"
rm -rf "$PRE"; git -C "$SRC" worktree add -q --detach "$PRE" "$PRE_H1"
( cd "$PRE" && CARGO_TARGET_DIR="${CARGO_TARGET_DIR}-pre" cargo build -p ms-cli --locked -q )
S=$(python3 -c "import json;print(json.load(open('$WORK/crates/ms-codec/tests/vectors/hashlock-v0.8.json'))['kind'][0]['ms1'])")
set +e
OUT=$(printf '%s\n' "$S" | "${CARGO_TARGET_DIR}-pre/debug/ms" decode - 2>&1); RC=$?
set -e
git -C "$SRC" worktree remove --force "$PRE"
echo "$OUT" | head -2; echo "exit=$RC"
[ "$RC" -eq 2 ] && echo "$OUT" | grep -q "reserved-prefix byte was 0x03" || { echo "DOWNGRADE ROW FAILED: expected exit 2 and the reserved-prefix text" >&2; exit 4; }
echo "== NOT covered: fragments beyond the hand-wire script; the toolkit manual chapter; .github/workflows edits; H0 (fork + me). =="
```

- [ ] **Step 2: Write the hand-wire script**

The fragments are the Task-by-task edits to existing files below. The script
applies each by exact-anchor replacement and refuses to run twice (a sentinel
file in the copy). Each fragment's text is the SAME text the tasks show — this
script is the executable index of them, not a second copy with different
words: when a task's fragment changes, this script's string changes with it.

```python
#!/usr/bin/env python3
"""plan-handwire-ms-hashlock.py <scratch-copy>

Applies IMPLEMENTATION_PLAN_ms_hashlock_H1.md's FRAGMENTS (edits to existing
files) to a scratch copy of mnemonic-secret, so plan-build-gate-ms.sh can
build the plan's new files against them. Every replacement is exact-anchor:
a missing anchor is an error naming the file, never a silent skip. Refuses to
run twice on one copy (sentinel .handwired)."""
import os, sys, re

root = sys.argv[1]
sentinel = os.path.join(root, ".handwired")
if os.path.exists(sentinel):
    sys.exit("already hand-wired: " + root)

def edit(path, pairs):
    full = os.path.join(root, path)
    s = open(full, encoding="utf-8").read()
    for old, new in pairs:
        if old not in s:
            sys.exit("anchor not found in %s:\n%s" % (path, old[:120]))
        s = s.replace(old, new, 1)
    open(full, "w", encoding="utf-8").write(s)
    print("  wired", path)

# ---- ms-codec ---------------------------------------------------------------
edit("crates/ms-codec/Cargo.toml", [
    ('getrandom = "0.3"',
     'getrandom = "0.3"\n# Hashlock derivation (spec §2), spelled as `me` spells them.\npbkdf2 = { version = "0.12", default-features = false, features = ["hmac"] }\nsha2 = "0.10"'),
    ('version = "0.7.0"', 'version = "0.8.0"'),
])
edit("crates/ms-codec/src/lib.rs", [
    ("pub mod error;", "pub mod error;\npub mod hashlock;"),
])
edit("crates/ms-codec/src/consts.rs", [
    ('pub const MNEM_PREFIX: u8 = 0x02;',
     'pub const MNEM_PREFIX: u8 = 0x02;\n\n/// v0.8 preimage-prefix byte: `[0x03][X:32]`, a hashlock preimage (SPEC_ms_hashlock §1).\npub const PREIMAGE_PREFIX: u8 = 0x03;\n\n/// The only string length a preimage single can have: 9 fixed + ceil(33*8/5)=53 payload + 13 cksum.\npub const VALID_PREIMAGE_STR_LENGTHS: &[usize] = &[75];\n\n/// 4-byte type tag carried by preimage SINGLES (SPEC_ms_hashlock §1, L14).\npub const TAG_HASH: [u8; 4] = *b"hash";'),
    ('pub const RESERVED_ID_BLOCKLIST: &[[u8; 4]] = &[*b"entr", *b"seed", *b"xprv", *b"mnem", *b"prvk"];',
     'pub const RESERVED_ID_BLOCKLIST: &[[u8; 4]] = &[*b"entr", *b"seed", *b"xprv", *b"mnem", *b"prvk", *b"hash"];'),
])
edit("crates/ms-codec/src/tag.rs", [
    ("use crate::consts::TAG_ENTR;", "use crate::consts::{TAG_ENTR, TAG_HASH};"),
    ("    pub const ENTR: Tag = Tag(TAG_ENTR);",
     "    pub const ENTR: Tag = Tag(TAG_ENTR);\n\n    /// The v0.8 emit-tag for a hashlock preimage single (id `hash`).\n    pub const HASH: Tag = Tag(TAG_HASH);"),
])
edit("crates/ms-codec/src/error.rs", [
    ("    /// Reserved-prefix byte was not 0x00 (SPEC §4 rule 8).\n    ReservedPrefixViolation {",
     "    /// A `0x03` payload whose length after the prefix byte is not 32 (SPEC_ms_hashlock §1).\n    PreimageLengthMismatch {\n        /// Bytes after the prefix byte -- the would-be X. Expected 32.\n        got: usize,\n    },\n    /// A single's tag names one kind and its prefix byte another (SPEC_ms_hashlock §1 rule 2).\n    TagKindMismatch {\n        /// The 4-byte tag observed.\n        tag: [u8; 4],\n        /// The prefix byte observed.\n        prefix: u8,\n    },\n    /// The OS CSPRNG could not fill the buffer (`getrandom` failed closed).\n    RandomnessUnavailable,\n    /// Reserved-prefix byte was not 0x00 (SPEC §4 rule 8).\n    ReservedPrefixViolation {"),
    ("            Error::ReservedPrefixViolation { got } => {",
     "            Error::PreimageLengthMismatch { got } => write!(\n                f,\n                \"preimage payload is {got} bytes after the prefix; a hashlock preimage is exactly 32 bytes (64 hex characters)\"\n            ),\n            Error::TagKindMismatch { tag, prefix } => write!(\n                f,\n                \"tag {:?} does not name the kind the prefix byte 0x{prefix:02x} carries; refusing rather than reading one kind as another\",\n                String::from_utf8_lossy(tag)\n            ),\n            Error::RandomnessUnavailable => write!(f, \"the OS random source is unavailable; no preimage was produced\"),\n            Error::ReservedPrefixViolation { got } => {"),
])
edit("crates/ms-codec/src/payload.rs", [
    ("    /// BIP-39 mnemonic entropy with wordlist language tag (16/20/24/28/32 B entropy).\n    Mnem,\n}",
     "    /// BIP-39 mnemonic entropy with wordlist language tag (16/20/24/28/32 B entropy).\n    Mnem,\n    /// A hashlock preimage: exactly 32 B (SPEC_ms_hashlock §1).\n    Preimage,\n}\n\nimpl PayloadKind {\n    /// The tag a SINGLE of this kind carries: `entr` for the two seed kinds,\n    /// `hash` for a preimage. Decode CHECKS a single's tag against this; encode\n    /// refuses to emit a mismatch (SPEC_ms_hashlock §1 rule 2).\n    pub fn single_tag(self) -> crate::tag::Tag {\n        match self {\n            PayloadKind::Entr | PayloadKind::Mnem => crate::tag::Tag::ENTR,\n            PayloadKind::Preimage => crate::tag::Tag::HASH,\n        }\n    }\n}"),
    ("pub enum Payload {", "pub enum Payload {\n    /// A hashlock preimage, exactly 32 bytes; scrubbed on drop (SPEC_ms_hashlock §3).\n    Preimage(zeroize::Zeroizing<[u8; 32]>),"),
    ("            Payload::Mnem { .. } => PayloadKind::Mnem,\n        }",
     "            Payload::Mnem { .. } => PayloadKind::Mnem,\n            Payload::Preimage(_) => PayloadKind::Preimage,\n        }"),
    ("            Payload::Mnem { entropy, .. } => entropy,\n        }",
     "            Payload::Mnem { entropy, .. } => entropy,\n            Payload::Preimage(x) => &x[..],\n        }"),
])
edit("crates/ms-codec/src/envelope.rs", [
    ("use crate::consts::{", "use crate::consts::{PREIMAGE_PREFIX, "),
    ("        other => {\n            return Err(Error::ReservedPrefixViolation { got: other });\n        }",
     "        PREIMAGE_PREFIX => {\n            // 0x03 -> Preimage: LENGTH CHECK BEFORE CONSTRUCTION, so the entr\n            // length error never names a legal entr length as illegal and no\n            // slice index can panic (SPEC_ms_hashlock §1).\n            let rest = &data[1..];\n            let x: [u8; 32] = rest\n                .try_into()\n                .map_err(|_| Error::PreimageLengthMismatch { got: rest.len() })?;\n            Payload::Preimage(Zeroizing::new(x))\n        }\n        other => {\n            return Err(Error::ReservedPrefixViolation { got: other });\n        }"),
    ("        Payload::Mnem { language, entropy } => {\n            // [0x02 mnem-prefix] || [language] || entropy",
     "        Payload::Preimage(x) => {\n            // [0x03 preimage-prefix] || X\n            let mut v = Zeroizing::new(Vec::with_capacity(33));\n            v.push(PREIMAGE_PREFIX);\n            v.extend_from_slice(&x[..]);\n            v\n        }\n        Payload::Mnem { language, entropy } => {\n            // [0x02 mnem-prefix] || [language] || entropy"),
])
# Both doc-comment copies of the prefix table gain the 0x03 line; they differ
# only in column alignment, so the anchor is the line's head and BOTH must hit.
_p = os.path.join(root, "crates/ms-codec/src/envelope.rs")
_s = open(_p, encoding="utf-8").read()
if _s.count("/// - any other prefix") != 2:
    sys.exit("envelope.rs: expected exactly two `/// - any other prefix` doc lines")
_s = _s.replace("/// - any other prefix", "/// - `0x03` (`PREIMAGE_PREFIX`) → `Payload::Preimage(rest)` iff rest is 32 bytes\n/// - any other prefix", 2)
open(_p, "w", encoding="utf-8").write(_s)
print("  wired crates/ms-codec/src/envelope.rs (both prefix-table doc comments)")
edit("crates/ms-codec/src/decode.rs", [
    ("use crate::consts::{\n    RESERVED_NOT_EMITTED_V01, TAG_ENTR, VALID_MNEM_STR_LENGTHS, VALID_STR_LENGTHS,\n};",
     "use crate::consts::{\n    RESERVED_NOT_EMITTED_V01, TAG_ENTR, TAG_HASH, VALID_MNEM_STR_LENGTHS,\n    VALID_PREIMAGE_STR_LENGTHS, VALID_STR_LENGTHS,\n};"),
    ("        PayloadKind::Mnem => VALID_MNEM_STR_LENGTHS,\n    }",
     "        PayloadKind::Mnem => VALID_MNEM_STR_LENGTHS,\n        PayloadKind::Preimage => VALID_PREIMAGE_STR_LENGTHS,\n    }"),
    ("        x if x == TAG_ENTR => {\n            match payload {\n                Payload::Entr(data) => {",
     "        // Rule 6b (SPEC_ms_hashlock §1 rule 2): a single's tag must name the\n        // kind its prefix byte carries. Checked BEFORE the per-tag arms so a\n        // `hash` tag over a seed payload, or `entr` over a preimage, is refused\n        // rather than read as the other kind.\n        x if (x == TAG_ENTR || x == TAG_HASH) && tag != payload.kind().single_tag() => {\n            return Err(Error::TagKindMismatch {\n                tag: x,\n                prefix: crate::envelope::prefix_of(&payload),\n            });\n        }\n        x if x == TAG_HASH => {\n            // A preimage single: length is structural in the variant.\n            payload\n        }\n        x if x == TAG_ENTR => {\n            match payload {\n                Payload::Entr(data) => {"),
    ("                Payload::Mnem { language, entropy } => {\n                    let p = Payload::Mnem { language, entropy };\n                    // §4 rule 10: validate (language range + entropy length).\n                    p.validate()?;\n                    p\n                }\n            }",
     "                Payload::Mnem { language, entropy } => {\n                    let p = Payload::Mnem { language, entropy };\n                    // §4 rule 10: validate (language range + entropy length).\n                    p.validate()?;\n                    p\n                }\n                // Unreachable: rule 6b above refused the mismatch. Kept as a\n                // typed error, never a panic.\n                other => {\n                    return Err(Error::TagKindMismatch {\n                        tag: x,\n                        prefix: crate::envelope::prefix_of(&other),\n                    })\n                }\n            }"),
])
edit("crates/ms-codec/src/envelope.rs", [
    ("pub(crate) fn payload_wire_bytes(p: &Payload) -> Zeroizing<Vec<u8>> {",
     "/// The prefix byte a payload writes on the wire, for error reporting.\npub(crate) fn prefix_of(p: &Payload) -> u8 {\n    match p {\n        Payload::Entr(_) => RESERVED_PREFIX,\n        Payload::Mnem { .. } => MNEM_PREFIX,\n        Payload::Preimage(_) => PREIMAGE_PREFIX,\n    }\n}\n\npub(crate) fn payload_wire_bytes(p: &Payload) -> Zeroizing<Vec<u8>> {"),
])
edit("crates/ms-codec/src/encode.rs", [
    ("pub fn encode(tag: Tag, payload: &Payload) -> Result<String> {",
     "pub fn encode(tag: Tag, payload: &Payload) -> Result<String> {\n    // SPEC_ms_hashlock §1 rule 2, emit side: never mint a single whose tag\n    // names a different kind than its prefix byte -- decode would refuse it.\n    if tag != payload.kind().single_tag() {\n        return Err(Error::TagKindMismatch {\n            tag: *tag.as_bytes(),\n            prefix: crate::envelope::prefix_of(payload),\n        });\n    }"),
])
edit("crates/ms-codec/src/inspect.rs", [
    ("    /// Any other prefix byte — future or invalid.\n    Unknown,",
     "    /// `hash` — a hashlock preimage (0x03 prefix byte, v0.8).\n    Preimage,\n    /// Any other prefix byte — future or invalid.\n    Unknown,"),
])

# ---- ms-cli -----------------------------------------------------------------
edit("crates/ms-cli/Cargo.toml", [
    ('version = "0.17.1"', 'version = "0.18.0"'),
    ('ms-codec = { path = "../ms-codec", version = "=0.7.0" }', 'ms-codec = { path = "../ms-codec", version = "=0.8.0" }'),
])
edit("crates/ms-cli/src/cmd/mod.rs", [
    ("pub mod gui_schema;", "pub mod gui_schema;\npub mod hashlock;"),
])
edit("crates/ms-cli/src/main.rs", [
    ("mod format;", "mod format;\nmod hashlock_phrase;"),
    ("    Decode(cmd::decode::DecodeArgs),",
     "    Decode(cmd::decode::DecodeArgs),\n\n    /// Derive a hashlock preimage from a phrase (or take one), print the `hash:` record, and back the preimage up as an ms1 plate string.\n    #[command(\n        after_long_help = \"EXAMPLES:\\n  ms hashlock --hashlock-phrase-stdin < phrase.txt\\n  ms hashlock --hashlock-phrase-stdin --method sha256 < phrase.txt\\n  ms hashlock --random --out preimage.txt\\n  ms hashlock --in preimage.txt\\n  ms hashlock --hashlock-phrase-stdin < phrase.txt | me sysw pack --out payload.bin\"\n    )]\n    Hashlock(cmd::hashlock::HashlockArgs),"),
    ("        Command::Decode(args) => cmd::decode::run(args),",
     "        Command::Decode(args) => cmd::decode::run(args),\n        Command::Hashlock(args) => cmd::hashlock::run(args),"),
    ("        Command::Decode(a) => a.json,", "        Command::Decode(a) => a.json,\n        Command::Hashlock(a) => a.json,"),
])
edit("crates/ms-cli/src/argv_guard.rs", [
    ('const SUBCOMMANDS: [&str; 12] = [\n    "derive",', 'const SUBCOMMANDS: [&str; 13] = [\n    "hashlock",\n    "derive",'),
    ('const SECRET_FLAGS: [&str; 4] = ["--phrase", "--hex", "--ms1", "--passphrase"];',
     'const SECRET_FLAGS: [&str; 5] = ["--phrase", "--hex", "--ms1", "--passphrase", "--hashlock-phrase"];'),
    ("            Some(\"encode\")\n                | Some(\"decode\")", "            Some(\"hashlock\")\n                | Some(\"encode\")\n                | Some(\"decode\")"),
    ('        "--ms1" => "an ms1 string",', '        "--ms1" => "an ms1 string",\n        "--hashlock-phrase" => "a hashlock phrase",'),
    ("fn argv_candidates(token: &str) -> Vec<String> {\n    let norm = |s: &str| s.trim().to_ascii_lowercase();",
     "fn argv_candidates(token: &str) -> Vec<String> {\n    // The fold and trim now live in `looks_like_ms1` as well, so the phrase\n    // channels and this guard test the SAME normalised shape\n    // (SPEC_ms_hashlock §4.3; R0 r0 tests C-1).\n    let norm = |s: &str| s.trim().to_ascii_lowercase();"),
    ("fn is_ms1_shaped(s: &str) -> bool {",
     "/// `is_ms1_shaped` over the NORMALISED token: trimmed, lowercased, display\n/// separators stripped. The one predicate both the argv guard and the phrase\n/// channels call, so the two cannot drift (SPEC_ms_hashlock §4.3). An\n/// uppercase plate string -- the BIP-173/QR spelling `ms decode` accepts --\n/// is caught here and only here.\npub(crate) fn looks_like_ms1(raw: &str) -> bool {\n    is_ms1_shaped(&raw.trim().to_ascii_lowercase())\n}\n\nfn is_ms1_shaped(s: &str) -> bool {"),
])
edit("crates/ms-cli/src/error.rs", [
    ("    BadInput(String),",
     "    BadInput(String),\n    /// A usage error the verb itself detects (source arithmetic, a gate a flag\n    /// must satisfy): exit 64, the same code clap uses for its own.\n    Usage(String),"),
    ("            | CliError::PayloadLengthMismatch { .. } => 1,",
     "            | CliError::PayloadLengthMismatch { .. } => 1,\n            CliError::Usage(_) => 64,"),
])
edit("crates/ms-cli/src/cmd/decode.rs", [
    ("        // ms_codec::Payload is #[non_exhaustive]; guard against future variants.\n        _ => unreachable!(\"ms-codec decode returned unknown Payload variant\"),\n    };",
     "        // A preimage carries no language; the value is rendered below by\n        // `emit_preimage`, never as words (SPEC_ms_hashlock §5).\n        Payload::Preimage(_) => (cli_lang, defaulted),\n        // ms_codec::Payload is #[non_exhaustive]; guard against future variants.\n        _ => unreachable!(\"ms-codec decode returned unknown Payload variant\"),\n    };"),
])
open(sentinel, "w").write("wired\n")
print("hand-wire complete")
```

(The remaining fragments — `decode.rs`'s second arm and its `emit_preimage`,
`combine.rs`, `payload_lang.rs`, `inspect.rs` — are added to this script by
Tasks 8 and 9, in the same exact-anchor form; each task's Step names the
`edit(...)` entry it appends.)

- [ ] **Step 3: Run the gate on this plan and confirm it extracts, builds and refuses correctly**

Run: `scripts/plan-build-gate-ms.sh design/IMPLEMENTATION_PLAN_ms_hashlock_H1.md`
Expected: step 2 lists every new file with a line count; step 4 ends with
`fmt clean`; step 5 prints a distance ≥ 9; step 6 prints `exit=2` and the
reserved-prefix text. Then, in a copy of the plan with every anchor removed:
`exit 3` and `EXTRACTED NOTHING`.

- [ ] **Step 4: Commit**

```bash
git add scripts/plan-build-gate-ms.sh scripts/plan-handwire-ms-hashlock.py
git commit -m "scripts: plan-build-gate-ms.sh + the hashlock hand-wire script (H1 plan gate)"
```

---

### Task 1: Codec constants, tag, errors, blocklist

**Files:**
- Modify: `crates/ms-codec/src/consts.rs:36-71` (fragment)
- Modify: `crates/ms-codec/src/tag.rs:14-17` (fragment)
- Modify: `crates/ms-codec/src/error.rs:60-66,200-204` (fragment)
- Test: `crates/ms-codec/tests/hashlock_kind.rs` (Create; grows in Task 2)

**Interfaces:**
- Produces: `consts::{PREIMAGE_PREFIX = 0x03, TAG_HASH = *b"hash",
  VALID_PREIMAGE_STR_LENGTHS = &[75]}`, `Tag::HASH`,
  `Error::{PreimageLengthMismatch { got: usize }, TagKindMismatch { tag: [u8;4], prefix: u8 }, RandomnessUnavailable}`,
  `RESERVED_ID_BLOCKLIST` with six entries.

- [ ] **Step 1: Write the failing tests**

Create `crates/ms-codec/tests/hashlock_kind.rs`:

```rust
//! Kind rows for the `0x03` preimage kind (SPEC_ms_hashlock §1, §8).
//!
//! Every row names the door it enters by and the error it asserts: a row that
//! says "refused" without either passes on the wrong error.

use ms_codec::consts::{PREIMAGE_PREFIX, RESERVED_ID_BLOCKLIST, TAG_HASH, VALID_PREIMAGE_STR_LENGTHS};
use ms_codec::{decode, encode, Error, Payload, PayloadKind, Tag};
use zeroize::Zeroizing;

fn preimage(byte: u8) -> Payload {
    Payload::Preimage(Zeroizing::new([byte; 32]))
}

#[test]
fn constants_are_the_specs() {
    assert_eq!(PREIMAGE_PREFIX, 0x03);
    assert_eq!(TAG_HASH, *b"hash");
    assert_eq!(VALID_PREIMAGE_STR_LENGTHS, &[75]);
    assert_eq!(Tag::HASH.as_bytes(), b"hash");
    // Six entries: the five that shipped plus `hash` (spec §1 rule 3).
    assert_eq!(RESERVED_ID_BLOCKLIST.len(), 6);
    assert!(RESERVED_ID_BLOCKLIST.contains(b"hash"));
}

#[test]
fn single_tag_by_kind() {
    assert_eq!(PayloadKind::Entr.single_tag(), Tag::ENTR);
    assert_eq!(PayloadKind::Mnem.single_tag(), Tag::ENTR);
    assert_eq!(PayloadKind::Preimage.single_tag(), Tag::HASH);
}

#[test]
fn a_hash_single_round_trips_and_is_75_chars() {
    let s = encode(Tag::HASH, &preimage(0xab)).expect("encode");
    assert_eq!(s.len(), 75, "{s}");
    assert!(s.starts_with("ms10hashsq"), "{s}");
    let (tag, p) = decode(&s).expect("decode");
    assert_eq!(tag, Tag::HASH);
    assert_eq!(p.kind(), PayloadKind::Preimage);
    assert_eq!(p.as_bytes(), &[0xab; 32]);
}

#[test]
fn the_entr32_and_preimage_pair_are_adjacent_rows() {
    // Same length, same leading payload char; only the id differs.
    let e = encode(Tag::ENTR, &Payload::Entr(vec![0xab; 32])).unwrap();
    let h = encode(Tag::HASH, &preimage(0xab)).unwrap();
    assert_eq!(e.len(), 75);
    assert_eq!(h.len(), 75);
    assert!(e.starts_with("ms10entrsq"), "{e}");
    assert!(h.starts_with("ms10hashsq"), "{h}");
}

#[test]
fn id_and_prefix_must_agree_both_directions() {
    // id `hash` over a seed payload: encode refuses, and a hand-made string
    // is refused on decode -- never read as the other kind (spec §1 rule 2).
    let err = encode(Tag::HASH, &Payload::Entr(vec![0; 32])).unwrap_err();
    assert!(matches!(err, Error::TagKindMismatch { tag, prefix: 0x00 } if tag == *b"hash"), "{err:?}");
    let err = encode(Tag::ENTR, &preimage(0)).unwrap_err();
    assert!(matches!(err, Error::TagKindMismatch { tag, prefix: 0x03 } if tag == *b"entr"), "{err:?}");

    // Hand-made strings through the codex32 layer, bypassing encode's check.
    let forged_hash_over_seed = forge("hash", &{
        let mut v = vec![0x00u8];
        v.extend_from_slice(&[0xab; 32]);
        v
    });
    let err = decode(&forged_hash_over_seed).unwrap_err();
    assert!(matches!(err, Error::TagKindMismatch { prefix: 0x00, .. }), "{err:?}");
    let forged_entr_over_preimage = forge("entr", &{
        let mut v = vec![PREIMAGE_PREFIX];
        v.extend_from_slice(&[0xab; 32]);
        v
    });
    let err = decode(&forged_entr_over_preimage).unwrap_err();
    assert!(matches!(err, Error::TagKindMismatch { prefix: 0x03, .. }), "{err:?}");
}

/// Build a threshold-0 single with an arbitrary id over arbitrary payload
/// bytes, through the vendored codex32 layer -- the forger's door.
fn forge(id: &str, payload: &[u8]) -> String {
    ms_codec::codex32::Codex32String::from_seed("ms", 0, id, ms_codec::codex32::Fe::S, payload)
        .expect("from_seed")
        .to_string()
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p ms-codec --test hashlock_kind`
Expected: FAIL to compile — `PREIMAGE_PREFIX`, `TAG_HASH`, `Tag::HASH`,
`Payload::Preimage`, `single_tag` do not exist.

- [ ] **Step 3: Apply the Task 1 fragments**

The exact edits are the `consts.rs`, `tag.rs` and `error.rs` entries of
`scripts/plan-handwire-ms-hashlock.py` (Task 0, Step 2). Apply them to the
working tree by hand, byte for byte. In prose: `consts.rs` gains
`PREIMAGE_PREFIX`, `VALID_PREIMAGE_STR_LENGTHS` and `TAG_HASH` after
`MNEM_PREFIX`, and `hash` joins `RESERVED_ID_BLOCKLIST`; `tag.rs` gains
`Tag::HASH`; `error.rs` gains the three variants before
`ReservedPrefixViolation` and their `Display` arms — the `PreimageLengthMismatch`
text carries L8's dual spelling, "exactly 32 bytes (64 hex characters)".

- [ ] **Step 4: Run — still failing, one step further**

Run: `cargo test -p ms-codec --test hashlock_kind`
Expected: FAIL to compile on `Payload::Preimage` / `single_tag` (Task 2).
`constants_are_the_specs` cannot run yet; that is expected.

- [ ] **Step 5: Commit the fragments**

```bash
git add crates/ms-codec/src/consts.rs crates/ms-codec/src/tag.rs crates/ms-codec/src/error.rs crates/ms-codec/tests/hashlock_kind.rs
git commit -m "ms-codec: PREIMAGE_PREFIX 0x03, TAG_HASH, Tag::HASH, the three preimage errors, hash in the id blocklist (H1 Task 1)"
```

---

### Task 2: The `Payload::Preimage` variant, dispatch, accept set, inspect kind

**Files:**
- Modify: `crates/ms-codec/src/payload.rs:9-15,29-31,93-106` (fragment)
- Modify: `crates/ms-codec/src/envelope.rs:114-115,186-188,192-222,231-245` (fragment)
- Modify: `crates/ms-codec/src/decode.rs:12-14,27-32,84-105` (fragment)
- Modify: `crates/ms-codec/src/encode.rs` (`pub fn encode`, fragment)
- Modify: `crates/ms-codec/src/inspect.rs:12-20` (fragment)
- Test: `crates/ms-codec/tests/hashlock_kind.rs` (Add to)

**Interfaces:**
- Consumes: Task 1's constants, tag and errors.
- Produces: `Payload::Preimage(Zeroizing<[u8; 32]>)`, `PayloadKind::Preimage`,
  `PayloadKind::single_tag(self) -> Tag`, `InspectKind::Preimage`,
  `envelope::prefix_of(&Payload) -> u8` (crate-private), the `hash` accept-set
  arm and the tag/kind check in `decode`, the emit-side check in `encode`.

- [ ] **Step 1: Add the length-row and inspect tests**

Add to `crates/ms-codec/tests/hashlock_kind.rs`:

```rust
#[test]
fn preimage_length_rows_through_decode_name_their_error() {
    // The wrong-length set that reaches prefix dispatch through `decode` is
    // exactly these nine (spec §1: 22 + ceil(8N/5) lands in the union length
    // set). `got` is the byte count AFTER the prefix.
    for n in [17usize, 18, 21, 22, 25, 26, 29, 30, 34] {
        let mut payload = vec![PREIMAGE_PREFIX];
        payload.extend(std::iter::repeat(0xab).take(n - 1));
        let s = forge("hash", &payload);
        let err = decode(&s).unwrap_err();
        assert!(
            matches!(err, Error::PreimageLengthMismatch { got } if got == n - 1),
            "payload {n} bytes ({} chars): {err:?}",
            s.len()
        );
    }
}

#[test]
fn preimage_length_rows_refused_by_the_string_gate_first() {
    // 16, 32 and 44 never reach prefix dispatch through `decode`: their
    // strings (48, 74, 93 chars) are outside the union length set.
    for (n, chars) in [(16usize, 48usize), (32, 74), (44, 93)] {
        let mut payload = vec![PREIMAGE_PREFIX];
        payload.extend(std::iter::repeat(0xab).take(n - 1));
        let s = forge("hash", &payload);
        assert_eq!(s.len(), chars);
        let err = decode(&s).unwrap_err();
        assert!(matches!(err, Error::UnexpectedStringLength { got } if got == chars), "{n}: {err:?}");
    }
}

#[test]
fn preimage_length_rows_through_combine_shares() {
    // The share path has no string-length gate, so 16, 32 and 44 reach
    // `PreimageLengthMismatch` here. Build a 2-of-2 set over a bad payload by
    // hand through the codex32 layer and recombine.
    for n in [16usize, 32, 44] {
        let mut secret = vec![PREIMAGE_PREFIX];
        secret.extend(std::iter::repeat(0xab).take(n - 1));
        let shares = forge_shares(&secret, 2, 2);
        let err = ms_codec::combine_shares(&shares).unwrap_err();
        assert!(
            matches!(err, Error::PreimageLengthMismatch { got } if got == n - 1),
            "payload {n} bytes via combine: {err:?}"
        );
    }
}

#[test]
fn a_46_byte_payload_is_unconstructible() {
    let mut payload = vec![PREIMAGE_PREFIX];
    payload.extend(std::iter::repeat(0xab).take(45));
    let s = ms_codec::codex32::Codex32String::from_seed("ms", 0, "hash", ms_codec::codex32::Fe::S, &payload)
        .expect("from_seed")
        .to_string();
    assert_eq!(s.len(), 96);
    assert!(ms_codec::codex32::Codex32String::from_string(s).is_err(), "96 chars is outside both brackets");
}

#[test]
fn preimage_share_round_trip() {
    let secret = preimage(0x5a);
    let shares = ms_codec::encode_shares(Tag::HASH, ms_codec::Threshold::new(2).unwrap(), 3, &secret).unwrap();
    for pair in [[0, 1], [0, 2], [1, 2]] {
        let (_tag, p) = ms_codec::combine_shares(&[shares[pair[0]].clone(), shares[pair[1]].clone()]).unwrap();
        assert_eq!(p, secret);
    }
}

#[test]
fn inspect_reports_the_kind() {
    let s = encode(Tag::HASH, &preimage(0x11)).unwrap();
    let r = ms_codec::inspect(&s).unwrap();
    assert_eq!(r.kind, ms_codec::InspectKind::Preimage);
    assert_eq!(r.prefix_byte, PREIMAGE_PREFIX);
    assert_eq!(r.tag, Tag::HASH);
}

#[test]
fn codeword_distance_between_entr_and_hash_ids_exceeds_the_correction_bound() {
    // Spec §1: measured, not inherited. BIP-93 corrects up to 4 errors; two
    // codewords that could be confused by a correction must be > 8 apart.
    let payload = {
        let mut v = vec![PREIMAGE_PREFIX];
        v.extend_from_slice(&[0xab; 32]);
        v
    };
    let a = forge("entr", &payload);
    let b = forge("hash", &payload);
    let distance = a.bytes().zip(b.bytes()).filter(|(x, y)| x != y).count();
    println!("codeword distance entr/hash = {distance}");
    assert!(distance > 8, "distance {distance} is within twice the correction bound");
}

/// A K-of-N share set over raw payload bytes, through the codex32 layer, so a
/// wrong-length payload can be recombined without `encode_shares` refusing it.
fn forge_shares(secret: &[u8], k: usize, n: usize) -> Vec<String> {
    use ms_codec::codex32::{Codex32String, Fe};
    let s = Codex32String::from_seed("ms", k, "zzzz", Fe::S, secret).expect("secret at S");
    let mut out = Vec::new();
    let indices = [Fe::A, Fe::C, Fe::D];
    for idx in indices.iter().take(n) {
        // interpolate_at is the vendored codex32's share derivation.
        out.push(ms_codec::codex32::derive_share(&s, *idx).expect("share").to_string());
    }
    out
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p ms-codec --test hashlock_kind`
Expected: FAIL to compile (`Payload::Preimage`, `InspectKind::Preimage`, the
`forge_shares` helper's `derive_share` — confirm that name against the
vendored `codex32` module before writing Step 3: `grep -n "pub fn" crates/ms-codec/src/codex32/mod.rs | grep -i "share\|interpolat"`; if the exported
name differs, use it and record the real name here).

- [ ] **Step 3: Apply the Task 2 fragments**

The `payload.rs`, `envelope.rs` (both edits), `decode.rs`, `encode.rs` and
`inspect.rs` entries of `scripts/plan-handwire-ms-hashlock.py`, byte for byte.
In prose, the load-bearing lines: `dispatch_payload`'s `0x03` arm checks the
length BEFORE constructing the variant (`rest.try_into().map_err(...)`, no
slice index); `decode`'s accept set gains `hash` and — before either per-tag
arm — refuses a tag that does not equal `payload.kind().single_tag()` with
`TagKindMismatch`; `encode` makes the same refusal on emit; `inspect.rs` maps
`0x03` to `InspectKind::Preimage` (the mapping line in `inspect()` itself:
`PREIMAGE_PREFIX => InspectKind::Preimage,` beside the `MNEM_PREFIX` arm).

- [ ] **Step 4: Run the kind tests**

Run: `cargo test -p ms-codec --test hashlock_kind`
Expected: PASS, all eleven tests; `codeword_distance` prints a number ≥ 9.

- [ ] **Step 5: Flip the tests that pinned `0x03` as reserved, mechanically**

Run: `grep -rn "0x03\|ReservedPrefixViolation" crates/ms-codec/tests crates/ms-codec/src --include=*.rs | grep -i "test\|assert"`
Expected: every hit that asserts `0x03 → ReservedPrefixViolation` is listed.
For each, change the asserted byte to `0x01` (still unallocated) so the test
keeps guarding the reserved-prefix rule; record the list of files touched in
the commit message. `cargo test -p ms-codec` then passes whole.

- [ ] **Step 6: Commit**

```bash
git add crates/ms-codec/src/payload.rs crates/ms-codec/src/envelope.rs crates/ms-codec/src/decode.rs crates/ms-codec/src/encode.rs crates/ms-codec/src/inspect.rs crates/ms-codec/tests/hashlock_kind.rs
git commit -m "ms-codec: Payload::Preimage, prefix 0x03 dispatch with the length check first, hash in the accept set, tag/kind consistency on decode and encode, InspectKind::Preimage (H1 Task 2)"
```

---

### Task 3: `ms_codec::hashlock` — the derivations, the random source, the digest

**Files:**
- Create: `crates/ms-codec/src/hashlock.rs`
- Modify: `crates/ms-codec/src/lib.rs:47` (fragment: `pub mod hashlock;`)
- Modify: `crates/ms-codec/Cargo.toml` (fragment: `pbkdf2`, `sha2`)
- Test: `crates/ms-codec/tests/hashlock_derivation.rs`

**Interfaces:**
- Produces: `hashlock::{HASHLOCK_SALT, HASHLOCK_ITERATIONS, HASHLOCK_DKLEN,
  preimage_hardened(&[u8]) -> Zeroizing<[u8;32]>, preimage_sha256(&[u8]) ->
  Zeroizing<[u8;32]>, preimage_random() -> Result<Zeroizing<[u8;32]>>,
  digest(&[u8;32]) -> [u8;32]}`.

- [ ] **Step 1: Write the failing derivation tests**

Create `crates/ms-codec/tests/hashlock_derivation.rs`:

```rust
//! Derivation rows, both methods, each pinning X AND H (SPEC_ms_hashlock §2,
//! §8). Every literal below was produced OUTSIDE this crate -- python3
//! hashlib and openssl kdf, cross-checked -- so a row is a correctness pin,
//! not a regression pin. `hashlock_repro.rs` re-runs those tools in CI.

use ms_codec::hashlock::{digest, preimage_hardened, preimage_sha256, HASHLOCK_DKLEN, HASHLOCK_ITERATIONS, HASHLOCK_SALT};

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

#[test]
fn constants_are_the_specs() {
    assert_eq!(HASHLOCK_SALT, b"ms-hashlock-v1");
    assert_eq!(HASHLOCK_ITERATIONS, 100_000);
    assert_eq!(HASHLOCK_DKLEN, 32);
}

/// (phrase, hardened X, hardened H, sha256 X, sha256 H) -- literals from
/// python3 hashlib.pbkdf2_hmac / hashlib.sha256, cross-checked in openssl kdf.
const ROWS: &[(&str, &str, &str, &str, &str)] = &[
    (
        "correct horse battery staple",
        "c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016",
        "3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12",
        "c4bbcb1fbec99d65bf59d85c8cb62ee2db963f0fe106f483d9afa73bd4e39a8a",
        "b867db875479bcc0287352cdaa4a1755689b8338777d0915e9acd9f6edbc96cb",
    ),
    // The implementer fills the remaining rows from the SAME two external
    // tools and pastes their command lines into the corpus's `provenance`
    // field: one character ("z"), 20 characters, 64 and 65 characters (the
    // HMAC block boundary), 100 and 101 characters (101 is REFUSED by the CLI
    // rule, but the codec derives any bytes; the row pins that the codec is
    // rule-free), "  a  b " (leading, doubled, trailing spaces),
    // "correct-horse,battery staple" (hyphen and comma), "a-b,c".
];

#[test]
fn anchor_rows_both_methods_pin_x_and_h() {
    for (phrase, hx, hh, sx, sh) in ROWS {
        let x = preimage_hardened(phrase.as_bytes());
        assert_eq!(hex(&x[..]), *hx, "hardened X for {phrase:?}");
        assert_eq!(hex(&digest(&x)), *hh, "hardened H for {phrase:?}");
        let x = preimage_sha256(phrase.as_bytes());
        assert_eq!(hex(&x[..]), *sx, "sha256 X for {phrase:?}");
        assert_eq!(hex(&digest(&x)), *sh, "sha256 H for {phrase:?}");
    }
}

#[test]
fn the_two_methods_differ_on_every_row() {
    for (phrase, hx, _, sx, _) in ROWS {
        assert_ne!(hx, sx, "{phrase:?}: a swap of the two methods must be visible");
    }
}

#[test]
fn bytes_are_used_verbatim() {
    // A trailing space changes X. If the codec ever trimmed, this fails.
    let a = preimage_sha256(b"a");
    let b = preimage_sha256(b"a ");
    assert_ne!(&a[..], &b[..]);
    let a = preimage_hardened(b"a-b,c");
    let b = preimage_hardened(b"abc");
    assert_ne!(&a[..], &b[..], "hyphen and comma are bytes, not separators");
}

#[test]
fn random_preimages_differ_and_are_32_bytes() {
    let a = ms_codec::hashlock::preimage_random().expect("os randomness");
    let b = ms_codec::hashlock::preimage_random().expect("os randomness");
    assert_ne!(&a[..], &b[..]);
    assert_eq!(a.len(), 32);
}

#[test]
fn digest_is_sha256_of_x() {
    // sha256 of 32 zero bytes, a public constant.
    let x = [0u8; 32];
    assert_eq!(hex(&digest(&x)), "66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925");
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p ms-codec --test hashlock_derivation`
Expected: FAIL to compile — `ms_codec::hashlock` does not exist.

- [ ] **Step 3: Write the module**

Create `crates/ms-codec/src/hashlock.rs`:

```rust
//! The hashlock preimage derivation (SPEC_ms_hashlock §2).
//!
//! THE RULE LIVES HERE, in the codec, beside the kind that carries its
//! output: one crate, one corpus, one SHA pin, one provenance pin for the Go
//! port. `ms hashlock` is a thin verb over these four functions.
//!
//! Two methods, the operator's choice (brainstorm L5): `preimage_hardened`
//! is PBKDF2-HMAC-SHA256 with a fixed salt, 100,000 iterations and dkLen 32
//! (L4); `preimage_sha256` is one SHA-256 of the phrase bytes. Both take the
//! phrase as BYTES, exactly as given -- no trimming, folding or normalising
//! happens here or in any caller (§4.3). `digest` is SHA-256 of X, the value
//! the policy carries; it is public the moment the policy is engraved and is
//! therefore NOT zeroized.
//!
//! THE SALT IS FIXED AND HAS NO PARAMETER (L13). Changing it after any vector
//! ships is a new method, not a tweak: every engraved policy's preimage was
//! derived under this exact byte string.

use pbkdf2::pbkdf2_hmac;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::error::{Error, Result};

/// The fixed salt (ASCII, copyable by hand, domain-separated from BIP-39's
/// `"mnemonic"` and from `me`'s 16-byte random seal salt).
pub const HASHLOCK_SALT: &[u8] = b"ms-hashlock-v1";
/// PBKDF2 iteration count -- the operator's cap, chosen so a signer at a
/// tenth of the SH2's measured rate still derives in reasonable time.
pub const HASHLOCK_ITERATIONS: u32 = 100_000;
/// Derived-key length: a miniscript `sha256(H)` preimage is exactly 32 bytes.
pub const HASHLOCK_DKLEN: usize = 32;

/// X = PBKDF2-HMAC-SHA256(phrase, HASHLOCK_SALT, HASHLOCK_ITERATIONS, 32).
pub fn preimage_hardened(phrase: &[u8]) -> Zeroizing<[u8; 32]> {
    let mut x = Zeroizing::new([0u8; HASHLOCK_DKLEN]);
    pbkdf2_hmac::<Sha256>(phrase, HASHLOCK_SALT, HASHLOCK_ITERATIONS, &mut *x);
    x
}

/// X = SHA-256(phrase). The brainwallet construction; the CLI warns on it at
/// every length (L12) and this function does not judge.
pub fn preimage_sha256(phrase: &[u8]) -> Zeroizing<[u8; 32]> {
    let mut x = Zeroizing::new([0u8; 32]);
    x.copy_from_slice(&Sha256::digest(phrase));
    x
}

/// X from the OS CSPRNG, failing closed: an error, never a zeroed buffer.
/// Lives here rather than in the CLI so the whole preimage surface -- and its
/// randomness contract -- is one crate's (R0 r0 correctness I-2).
pub fn preimage_random() -> Result<Zeroizing<[u8; 32]>> {
    let mut x = Zeroizing::new([0u8; 32]);
    getrandom::fill(&mut *x).map_err(|_| Error::RandomnessUnavailable)?;
    Ok(x)
}

/// H = SHA-256(X): what the policy carries and the plate shows. Public.
pub fn digest(preimage: &[u8; 32]) -> [u8; 32] {
    let mut h = [0u8; 32];
    h.copy_from_slice(&Sha256::digest(preimage));
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hardened_output_is_zeroizing_and_32() {
        let x = preimage_hardened(b"x");
        assert_eq!(x.len(), 32);
        // Two calls agree: the salt and count are constants, not state.
        assert_eq!(&preimage_hardened(b"x")[..], &x[..]);
    }
}
```

- [ ] **Step 4: Apply the Task 3 fragments and run**

The `Cargo.toml` (deps) and `lib.rs` entries of the hand-wire script, byte
for byte. Then:

Run: `cargo test -p ms-codec --test hashlock_derivation`
Expected: PASS, six tests. `cargo tree -p ms-codec -i pbkdf2` shows
`pbkdf2 v0.12.x` with features `hmac` only and no `password-hash`.

- [ ] **Step 5: Commit**

```bash
git add crates/ms-codec/src/hashlock.rs crates/ms-codec/src/lib.rs crates/ms-codec/Cargo.toml Cargo.lock crates/ms-codec/tests/hashlock_derivation.rs
git commit -m "ms-codec: hashlock module -- hardened and sha256 derivations, random source, digest (H1 Task 3)"
```

---
### Task 4: The corpus and the reproduction test that cannot lie

**Files:**
- Create: `crates/ms-codec/tests/vectors/hashlock-v0.8.json`
- Create: `crates/ms-codec/tests/hashlock_repro.rs`
- Modify: `.github/workflows/rust.yml:109-120` (the `test (ms-codec)` job: preflight + run-by-name; fragment, NOT gate-covered)

**Interfaces:**
- Consumes: Task 3's `hashlock` module; Task 2's kind.
- Produces: the corpus file the CHANGELOG SHA-pins (Task 11), the `provenance`
  field every derivation row carries, and the test name `hashlock_repro` CI
  asserts ran.

- [ ] **Step 1: Write the corpus**

Create `crates/ms-codec/tests/vectors/hashlock-v0.8.json` (the implementer
fills every `"…"` from the two external tools and pastes the command line
that produced it into `provenance`; the anchor row is complete):

```json
{
  "format": "ms hashlock corpus v0.8 (SPEC_ms_hashlock §8)",
  "kind": [
    {
      "description": "preimage single, X = 0xab*32; id hash; 75 chars; the entr-32 pair row is X=0xab*32 under Tag::ENTR",
      "preimage_hex": "abababababababababababababababababababababababababababababababab",
      "ms1": "…",
      "entr32_pair_ms1": "…"
    }
  ],
  "derivation": [
    {
      "phrase": "correct horse battery staple",
      "phrase_chars": 28,
      "hardened_x": "c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016",
      "hardened_h": "3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12",
      "sha256_x": "c4bbcb1fbec99d65bf59d85c8cb62ee2db963f0fe106f483d9afa73bd4e39a8a",
      "sha256_h": "b867db875479bcc0287352cdaa4a1755689b8338777d0915e9acd9f6edbc96cb",
      "provenance": "python3 -c 'import hashlib;p=b\"correct horse battery staple\";x=hashlib.pbkdf2_hmac(\"sha256\",p,b\"ms-hashlock-v1\",100000,32);print(x.hex(),hashlib.sha256(x).hex())' ; openssl kdf -keylen 32 -kdfopt digest:SHA256 -kdfopt pass:'correct horse battery staple' -kdfopt salt:ms-hashlock-v1 -kdfopt iter:100000 PBKDF2"
    },
    { "phrase": "z", "phrase_chars": 1, "hardened_x": "…", "hardened_h": "…", "sha256_x": "…", "sha256_h": "…", "provenance": "…" },
    { "phrase": "twenty characters!!!", "phrase_chars": 20, "hardened_x": "…", "hardened_h": "…", "sha256_x": "…", "sha256_h": "…", "provenance": "…" },
    { "phrase": "<64 printable ASCII characters, none of them all-hex>", "phrase_chars": 64, "hardened_x": "…", "hardened_h": "…", "sha256_x": "…", "sha256_h": "…", "provenance": "…" },
    { "phrase": "<65 printable ASCII characters>", "phrase_chars": 65, "hardened_x": "…", "hardened_h": "…", "sha256_x": "…", "sha256_h": "…", "provenance": "…" },
    { "phrase": "<100 printable ASCII characters>", "phrase_chars": 100, "hardened_x": "…", "hardened_h": "…", "sha256_x": "…", "sha256_h": "…", "provenance": "…" },
    { "phrase": "<101 printable ASCII characters -- the codec derives it; the CLI refuses it>", "phrase_chars": 101, "hardened_x": "…", "hardened_h": "…", "sha256_x": "…", "sha256_h": "…", "provenance": "…" },
    { "phrase": "  a  b ", "phrase_chars": 7, "hardened_x": "…", "hardened_h": "…", "sha256_x": "…", "sha256_h": "…", "provenance": "…" },
    { "phrase": "correct-horse,battery staple", "phrase_chars": 28, "hardened_x": "…", "hardened_h": "…", "sha256_x": "…", "sha256_h": "…", "provenance": "…" },
    { "phrase": "a-b,c", "phrase_chars": 5, "hardened_x": "…", "hardened_h": "…", "sha256_x": "…", "sha256_h": "…", "provenance": "…" }
  ],
  "refusals": [
    { "input": "", "channel": "phrase", "rule": "empty" },
    { "input": "café", "channel": "phrase", "rule": "printable-ascii" },
    { "input_bytes_hex": "ff", "channel": "phrase", "rule": "printable-ascii" },
    { "input": "a\tb", "channel": "phrase", "rule": "printable-ascii" },
    { "input_bytes_hex": "617f", "channel": "phrase", "rule": "printable-ascii" },
    { "input": " ~", "channel": "phrase", "rule": null, "note": "0x20 and 0x7E accepted" },
    { "input": "c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016", "channel": "phrase", "rule": "64-hex", "remedy": "--hex" },
    { "input": "C3E97525442520DA4CFFD5F57AAE3F6273990017F2E0FA30C056E32172E22016", "channel": "phrase", "rule": "64-hex", "remedy": "--hex" },
    { "input": "beef", "channel": "phrase", "rule": null, "note": "short all-hex ACCEPTED" },
    { "input": "<the kind[0].ms1 string, lowercase>", "channel": "phrase", "rule": "ms1-shaped", "remedy": "--in" },
    { "input": "<the kind[0].ms1 string, UPPERCASE>", "channel": "phrase", "rule": "ms1-shaped", "remedy": "--in" },
    { "input": "<the kind[0].ms1 string, grouped by 5 with spaces>", "channel": "phrase", "rule": "ms1-shaped", "remedy": "--in" },
    { "input": "<the kind[0].ms1 string, with two leading and two trailing spaces>", "channel": "phrase", "rule": "ms1-shaped", "remedy": "--in" },
    { "input": "<the kind[0].ms1 string, grouped by 2 (112 chars)>", "channel": "phrase", "rule": "ms1-shaped", "remedy": "--in", "note": "shape test precedes the cap" },
    { "input": "<101 printable ASCII characters>", "channel": "phrase", "rule": "too-long" }
  ],
  "lengths_by_door": [
    { "payload_bytes": 34, "door": "decode", "error": "PreimageLengthMismatch", "got": 33 },
    { "payload_bytes": 17, "door": "decode", "error": "PreimageLengthMismatch", "got": 16 },
    { "payload_bytes": 16, "door": "decode", "error": "UnexpectedStringLength", "got": 48 },
    { "payload_bytes": 16, "door": "combine_shares", "error": "PreimageLengthMismatch", "got": 15 },
    { "payload_bytes": 32, "door": "combine_shares", "error": "PreimageLengthMismatch", "got": 31 },
    { "payload_bytes": 44, "door": "combine_shares", "error": "PreimageLengthMismatch", "got": 43 },
    { "payload_bytes": 46, "door": "none", "note": "unconstructible: 96 chars is outside both codex32 brackets" }
  ],
  "lockstep": [
    "derivation rows: 100 and 101 characters, the spaces row, the hyphen+comma row",
    "refusals: empty, printable-ascii (TAB, DEL, 0xFF), 64-hex both cases, 101",
    "kind: the entr32 pair; id/prefix mismatch both directions (forged strings in hashlock_kind.rs)",
    "the fork's pin test drives these rows in BOTH directions (encode and decode)"
  ]
}
```

- [ ] **Step 2: Write the reproduction test**

Create `crates/ms-codec/tests/hashlock_repro.rs`:

```rust
//! The cross-tool reproduction (SPEC_ms_hashlock §8), written so it cannot
//! lie (R0 r0 tests I-9, FP-1..FP-5):
//!
//! - The salt, the iteration count, the dkLen and every expected hex are
//!   LITERALS here, independent of the crate's constants. A separate
//!   assertion pins the constants to the literals. Mutating a constant
//!   therefore moves ONE side of the comparison, and the test fails.
//! - Both external tools are RUN and their CAPTURED STDOUT compared, three
//!   ways: Rust = python, Rust = openssl, python = openssl.
//! - A missing tool FAILS the test. There is no `#[ignore]` and no cfg gate;
//!   CI additionally asserts this test ran by name (rust.yml).

use std::process::Command;

use ms_codec::hashlock::{digest, preimage_hardened, HASHLOCK_DKLEN, HASHLOCK_ITERATIONS, HASHLOCK_SALT};

// LITERALS. Not the crate's constants.
const SALT: &str = "ms-hashlock-v1";
const ITER: u32 = 100_000;
const DKLEN: usize = 32;
const PHRASE: &str = "correct horse battery staple";
const EXPECTED_X: &str = "c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016";
const EXPECTED_H: &str = "3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12";

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

#[test]
fn constants_equal_the_literals() {
    assert_eq!(HASHLOCK_SALT, SALT.as_bytes());
    assert_eq!(HASHLOCK_ITERATIONS, ITER);
    assert_eq!(HASHLOCK_DKLEN, DKLEN);
}

fn python_x() -> String {
    let script = format!(
        "import hashlib,sys;x=hashlib.pbkdf2_hmac('sha256',{phrase!r}.encode(),{salt!r}.encode(),{iter},{dklen});sys.stdout.write(x.hex())",
        phrase = PHRASE, salt = SALT, iter = ITER, dklen = DKLEN
    );
    let out = Command::new("python3").args(["-c", &script]).output().expect("python3 must be present: this test FAILS on a missing tool, never skips");
    assert!(out.status.success(), "python3 failed: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

fn openssl_x() -> String {
    let out = Command::new("openssl")
        .args([
            "kdf", "-keylen", &DKLEN.to_string(), "-kdfopt", "digest:SHA256",
            &format!("-kdfopt"), &format!("pass:{PHRASE}"),
            "-kdfopt", &format!("salt:{SALT}"),
            "-kdfopt", &format!("iter:{ITER}"), "PBKDF2",
        ])
        .output()
        .expect("openssl must be present: this test FAILS on a missing tool, never skips");
    assert!(out.status.success(), "openssl kdf failed: {}", String::from_utf8_lossy(&out.stderr));
    // openssl prints `AB:CD:...`; normalise to lowercase hex.
    String::from_utf8(out.stdout).unwrap().trim().replace(':', "").to_ascii_lowercase()
}

#[test]
fn hashlock_repro_three_ways() {
    let rust_x = hex(&preimage_hardened(PHRASE.as_bytes())[..]);
    let py = python_x();
    let ssl = openssl_x();
    assert_eq!(rust_x, EXPECTED_X, "Rust vs literal");
    assert_eq!(py, EXPECTED_X, "python vs literal");
    assert_eq!(ssl, EXPECTED_X, "openssl vs literal");
    assert_eq!(py, ssl, "python vs openssl");
    let mut x = [0u8; 32];
    for (i, b) in x.iter_mut().enumerate() {
        *b = u8::from_str_radix(&EXPECTED_X[2 * i..2 * i + 2], 16).unwrap();
    }
    assert_eq!(hex(&digest(&x)), EXPECTED_H, "digest of the literal X");
}
```

- [ ] **Step 3: Run to verify the tests fail for the right reasons, then pass**

Run: `cargo test -p ms-codec --test hashlock_repro -- --nocapture`
Expected: PASS (both tools present on this machine; `openssl kdf` needs
OpenSSL 3). Then MUTATE: change `HASHLOCK_ITERATIONS` to `10_000` in a copy
→ `constants_equal_the_literals` FAILS and `hashlock_repro_three_ways` FAILS
on "Rust vs literal" while python and openssl still agree with each other.
Revert. Then `PATH=/nonexistent cargo test ... hashlock_repro` → FAILS with
"python3 must be present" (never `ok`, never `ignored`).

- [ ] **Step 4: Edit the CI job (fragment; not gate-covered)**

In `.github/workflows/rust.yml`, inside the `test-ms-codec` job (line 109),
BEFORE its test step, add:

```yaml
      - name: hashlock reproduction preflight (a missing tool fails HERE, never a test that could be skipped)
        run: |
          openssl version
          openssl kdf --help >/dev/null
          python3 -VV
          python3 -c 'import hashlib; hashlib.pbkdf2_hmac("sha256", b"x", b"y", 1)'
```

and AFTER its test step, add the run-by-name assertion:

```yaml
      - name: hashlock reproduction test RAN (by name)
        run: |
          cargo nextest run -p ms-codec --locked -E 'test(hashlock_repro_three_ways)' 2>&1 | tee /tmp/repro.log
          grep -E "Summary.*1 test run" /tmp/repro.log
```

(`cargo-nextest` is installed in that job already if the job uses it;
otherwise `cargo test -p ms-codec --test hashlock_repro -- --exact
hashlock_repro_three_ways 2>&1 | grep -E "test result: ok. 1 passed"`.)

- [ ] **Step 5: Commit**

```bash
git add crates/ms-codec/tests/vectors/hashlock-v0.8.json crates/ms-codec/tests/hashlock_repro.rs .github/workflows/rust.yml
git commit -m "ms-codec: hashlock corpus v0.8 with external provenance; the three-way reproduction test with literal constants; CI preflight and run-by-name (H1 Task 4)"
```

---

### Task 5: The argv guard — six parts, one predicate

**Files:**
- Modify: `crates/ms-cli/src/argv_guard.rs:67-79,85-86,104-111,134-145,256-269,378-385` (fragment)
- Modify: `crates/ms-cli/src/error.rs:22,49-56` (fragment: `CliError::Usage`, exit 64)
- Test: `crates/ms-cli/tests/hashlock_sources.rs` (Create; the guard rows)

**Interfaces:**
- Produces: `argv_guard::looks_like_ms1(raw: &str) -> bool` (`pub(crate)`),
  `SUBCOMMANDS` with `hashlock`, `SECRET_FLAGS` with `--hashlock-phrase`,
  `override_applies` admitting `hashlock`, `flag_class("--hashlock-phrase") =
  "a hashlock phrase"`, `CliError::Usage(String)` → exit 64.

- [ ] **Step 1: Write the failing guard tests**

Create `crates/ms-cli/tests/hashlock_sources.rs`:

```rust
//! Source arithmetic and the argv guard on `ms hashlock` (SPEC_ms_hashlock
//! §4.1, §6). Each test names the mutation it fails under.

use assert_cmd::Command;

const PHRASE: &str = "correct horse battery staple";
const HEX32: &str = "c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016";

fn ms() -> Command {
    Command::cargo_bin("ms").unwrap()
}

/// MUTATION: drop `--hashlock-phrase` from SECRET_FLAGS -> the value is
/// accepted on argv and this fails.
#[test]
fn hashlock_phrase_on_argv_is_refused_without_the_allow_flag_and_never_echoed() {
    let out = ms().args(["hashlock", "--hashlock-phrase", PHRASE]).output().unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", String::from_utf8_lossy(&out.stderr));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("a hashlock phrase"), "flag_class must name the class:\n{err}");
    assert!(!err.contains(PHRASE), "the refusal echoed the phrase:\n{err}");
    assert!(!err.contains("BIP-39 passphrase"), "wrong class:\n{err}");
}

/// MUTATION: leave `hashlock` out of `override_applies` -> the allow flag
/// does nothing and this exits 1.
#[test]
fn allow_argv_secret_admits_the_phrase_through_the_side_channel() {
    let out = ms()
        .args(["hashlock", "--allow-argv-secret", "--hashlock-phrase", PHRASE, "--no-engraving-card"])
        .write_stdin("")
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let so = String::from_utf8_lossy(&out.stdout);
    assert_eq!(so.trim(), "hash:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12");
}

/// The §6 gate for part 4: the same invocation with stdin at /dev/null (an
/// EMPTY stdin here) still derives from the FLAG's value. MUTATION: build the
/// Source without `.on("--hashlock-phrase")` -> it reads stdin, gets nothing,
/// and refuses `empty`.
#[test]
fn admitted_phrase_does_not_read_stdin() {
    let out = ms()
        .args(["hashlock", "--allow-argv-secret", "--hashlock-phrase", PHRASE, "--no-engraving-card"])
        .write_stdin("")
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
}

/// Same gate for the `--hex` channel (part 5) and the positional (part 6).
#[test]
fn admitted_hex_and_positional_do_not_read_stdin() {
    let out = ms()
        .args(["hashlock", "--allow-argv-secret", "--hex", HEX32, "--no-engraving-card"])
        .write_stdin("")
        .output()
        .unwrap();
    assert!(out.status.success(), "hex: {}", String::from_utf8_lossy(&out.stderr));
    let plate = ms().args(["hashlock", "--hex", "-", "--no-engraving-card", "--out", "/dev/null"])
        .write_stdin(HEX32)
        .output().unwrap();
    assert!(plate.status.success());
    // Get a real plate string to pass positionally.
    let s = String::from_utf8(
        ms().args(["hashlock", "--hex", "-", "--json"]).write_stdin(HEX32).output().unwrap().stdout,
    )
    .unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    let plate = v["preimage_ms1"].as_str().unwrap().to_string();
    let out = ms()
        .args(["hashlock", "--allow-argv-secret", &plate, "--no-engraving-card"])
        .write_stdin("")
        .output()
        .unwrap();
    assert!(out.status.success(), "positional: {}", String::from_utf8_lossy(&out.stderr));
}

/// MUTATION: zero sources defaulting to stdin -> this hangs or parses stdin
/// as an ms1; expected is exit 64 listing five sources.
#[test]
fn zero_sources_exits_64_listing_five() {
    let out = ms().args(["hashlock"]).write_stdin("").output().unwrap();
    assert_eq!(out.status.code(), Some(64));
    let err = String::from_utf8_lossy(&out.stderr);
    for s in ["--hashlock-phrase", "--hashlock-phrase-stdin", "--hex", "--in", "--random"] {
        assert!(err.contains(s), "usage must list {s}:\n{err}");
    }
}

/// Every one of the ten two-source pairs exits 64. MUTATION: check only a
/// subset of pairs -> the stdin-contention pair passes silently.
#[test]
fn every_two_source_pair_exits_64() {
    let sources: &[&[&str]] = &[
        &["--allow-argv-secret", "--hashlock-phrase", PHRASE],
        &["--hashlock-phrase-stdin"],
        &["--hex", HEX32],
        &["-"],
        &["--random", "--out", "/tmp/ms-hashlock-pair-test.txt"],
    ];
    for i in 0..sources.len() {
        for j in (i + 1)..sources.len() {
            let mut args = vec!["hashlock"];
            args.extend_from_slice(sources[i]);
            args.extend_from_slice(sources[j]);
            let out = ms().args(&args).write_stdin(PHRASE).output().unwrap();
            assert_eq!(out.status.code(), Some(64), "pair {i},{j}: {}", String::from_utf8_lossy(&out.stderr));
        }
    }
    let _ = std::fs::remove_file("/tmp/ms-hashlock-pair-test.txt");
}

/// MUTATION: `--method` silently ignored with a supplied X.
#[test]
fn method_with_a_supplied_preimage_exits_64_for_all_three_sources() {
    for args in [
        vec!["hashlock", "--hex", HEX32, "--method", "sha256"],
        vec!["hashlock", "--random", "--out", "/tmp/ms-hashlock-method-test.txt", "--method", "hardened"],
        vec!["hashlock", "-", "--method", "sha256"],
    ] {
        let out = ms().args(&args).write_stdin("ms10hashsq").output().unwrap();
        assert_eq!(out.status.code(), Some(64), "{args:?}: {}", String::from_utf8_lossy(&out.stderr));
    }
    let _ = std::fs::remove_file("/tmp/ms-hashlock-method-test.txt");
}

/// L21 as narrowed: `--random` needs `--out FILE`; `--json` alone does not
/// satisfy it. MUTATION: gate on `--out || --json` -> the second case exits 0.
#[test]
fn random_requires_out_file_and_json_alone_does_not_satisfy_it() {
    let out = ms().args(["hashlock", "--random", "--no-engraving-card"]).output().unwrap();
    assert_eq!(out.status.code(), Some(64));
    assert!(String::from_utf8_lossy(&out.stderr).contains("--out"));
    let out = ms().args(["hashlock", "--random", "--json", "--no-engraving-card"]).output().unwrap();
    assert_eq!(out.status.code(), Some(64), "--json alone must not satisfy the gate");
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("x.txt");
    let out = ms().args(["hashlock", "--random", "--out", p.to_str().unwrap(), "--no-engraving-card"]).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(p.exists());
}

/// `--random` twice gives two different records. MUTATION: a fixed buffer.
#[test]
fn random_twice_differs() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.txt");
    let b = dir.path().join("b.txt");
    let ra = ms().args(["hashlock", "--random", "--out", a.to_str().unwrap(), "--no-engraving-card"]).output().unwrap();
    let rb = ms().args(["hashlock", "--random", "--out", b.to_str().unwrap(), "--no-engraving-card"]).output().unwrap();
    assert_ne!(ra.stdout, rb.stdout);
    assert_ne!(std::fs::read(&a).unwrap(), std::fs::read(&b).unwrap());
}

/// C-2 as folded: under `--random`, `--out` refuses to overwrite and leaves
/// the file's bytes unchanged; the other sources overwrite. MUTATION: use
/// the truncating writer for `--random` -> Monday's preimage is gone.
#[test]
fn random_out_refuses_to_overwrite_but_other_sources_overwrite() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("preimage.txt");
    assert!(ms().args(["hashlock", "--random", "--out", p.to_str().unwrap(), "--no-engraving-card"]).output().unwrap().status.success());
    let monday = std::fs::read(&p).unwrap();
    let out = ms().args(["hashlock", "--random", "--out", p.to_str().unwrap(), "--no-engraving-card"]).output().unwrap();
    assert_eq!(out.status.code(), Some(64), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(String::from_utf8_lossy(&out.stderr).contains(p.to_str().unwrap()));
    assert_eq!(std::fs::read(&p).unwrap(), monday, "the existing preimage must be untouched");
    // A phrase source overwrites (its artifact is a function of its input).
    let out = ms().args(["hashlock", "--hashlock-phrase-stdin", "--out", p.to_str().unwrap(), "--no-engraving-card"]).write_stdin(PHRASE).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert_ne!(std::fs::read(&p).unwrap(), monday);
}

/// `--out` is 0600 (owner-only) on every source.
#[cfg(unix)]
#[test]
fn out_is_owner_only() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("x.txt");
    assert!(ms().args(["hashlock", "--hashlock-phrase-stdin", "--out", p.to_str().unwrap(), "--no-engraving-card"]).write_stdin(PHRASE).output().unwrap().status.success());
    assert_eq!(std::fs::metadata(&p).unwrap().permissions().mode() & 0o777, 0o600);
}
```

`tempfile` and `serde_json` are dev-dependencies of ms-cli already if any
existing test uses them (`grep -n tempfile crates/ms-cli/Cargo.toml`); if
not, add `tempfile = "3"` under `[dev-dependencies]` in the same commit.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p ms-cli --test hashlock_sources`
Expected: every test FAILS — `ms hashlock` is not a subcommand (clap exit 2
"unrecognized subcommand"), and the guard's refusal names "a BIP-39
passphrase" for the flag.

- [ ] **Step 3: Apply the Task 5 fragments**

The `argv_guard.rs` and `error.rs` entries of the hand-wire script, byte for
byte. Also fix the pre-existing doc comment above `SECRET_FLAGS` that says
"The nine flag-keyed secret channels" — it is five now (R0 r0 tests N-4).

- [ ] **Step 4: Run — still red, one step further**

Run: `cargo test -p ms-cli --test hashlock_sources`
Expected: still FAIL (no verb yet), but the argv-refusal test's stderr now
reads "a hashlock phrase" — verify that one assertion moved:
`cargo test -p ms-cli --test hashlock_sources hashlock_phrase_on_argv -- --nocapture`.

- [ ] **Step 5: Commit**

```bash
git add crates/ms-cli/src/argv_guard.rs crates/ms-cli/src/error.rs crates/ms-cli/tests/hashlock_sources.rs
git commit -m "ms-cli: the argv guard learns hashlock -- SUBCOMMANDS 13, SECRET_FLAGS 5, override, flag_class, looks_like_ms1 normalises inside the predicate; CliError::Usage exit 64 (H1 Task 5)"
```

---

### Task 6: The byte-verbatim reader and the phrase rule

**Files:**
- Create: `crates/ms-cli/src/hashlock_phrase.rs`
- Modify: `crates/ms-cli/src/main.rs` (fragment: `mod hashlock_phrase;`)
- Test: unit tests inside the file (below) + `crates/ms-cli/tests/hashlock_phrase_rule.rs` (Task 9 drives the same rule through the binary)

**Interfaces:**
- Consumes: `argv_guard::looks_like_ms1`.
- Produces: `hashlock_phrase::{HASHLOCK_PHRASE_MAX_CHARS: usize = 100,
  read_phrase_stdin() -> Result<Zeroizing<Vec<u8>>>,
  strip_one_trailing_newline(&mut Vec<u8>), validate_phrase(&[u8]) ->
  Result<()>, PhraseRefusal}`.

- [ ] **Step 1: Write the module with its unit tests (they fail until the body exists — write the tests first in the file, run, then the body)**

Create `crates/ms-cli/src/hashlock_phrase.rs`:

```rust
//! The hashlock phrase's two channels and its one rule (SPEC_ms_hashlock §4.3).
//!
//! BYTES AS GIVEN. The reader is `Vec<u8>` over `read_to_end`, strips exactly
//! one trailing `\r?\n`, and does nothing else. It must never be
//! `parse::read_input` (strips all whitespace plus `-` and `,`) or
//! `parse::read_phrase_input` (trims and collapses runs): either silently
//! changes X while every codec vector still passes, and `-`/`,` are exactly
//! what diceware emits. A non-UTF-8 byte reaches the printable-ASCII rule and
//! is refused BY NAME, not by an io error (R0 r0 correctness M-6).
//!
//! THE RULE, identical on host and device: non-empty; every byte in
//! `0x20..=0x7E`; not ms1-shaped (tested on a normalised COPY, before the
//! cap, so a grouped plate string gets the `--in` remedy and not "too long");
//! at most 100 characters; not exactly 64 hex digits in either case (a pasted
//! preimage -- the remedy is `--hex`). Refusals name the rule and never echo
//! the phrase.

use zeroize::Zeroizing;

use crate::error::{CliError, Result};

/// Its own constant on each side, lockstep-pinned; NOT the device's
/// plate-legibility `passphrase.MaxLen` (review M-6).
pub const HASHLOCK_PHRASE_MAX_CHARS: usize = 100;

/// Why a phrase was refused. Each variant renders one sentence that names
/// the rule and, where one exists, the remedy.
#[derive(Debug, PartialEq, Eq)]
pub enum PhraseRefusal {
    Empty,
    NotPrintableAscii { byte: u8, at: usize },
    Ms1Shaped,
    TooLong { chars: usize },
    Hex64,
}

impl PhraseRefusal {
    pub fn message(&self) -> String {
        match self {
            PhraseRefusal::Empty => "the hashlock phrase is empty".to_string(),
            PhraseRefusal::NotPrintableAscii { byte, at } => format!(
                "the hashlock phrase must be printable ASCII (bytes 0x20..0x7E); byte 0x{byte:02x} at position {at} is not"
            ),
            PhraseRefusal::Ms1Shaped => "that is an ms1 string, not a hashlock phrase; pass it as the ms1 argument, `-` on stdin, or `--in FILE`".to_string(),
            PhraseRefusal::TooLong { chars } => format!(
                "the hashlock phrase is {chars} characters; at most {HASHLOCK_PHRASE_MAX_CHARS} are allowed"
            ),
            PhraseRefusal::Hex64 => "that is 64 hex characters -- a preimage, 32 bytes (64 hex characters), not a phrase; pass it with --hex".to_string(),
        }
    }
}

impl From<PhraseRefusal> for CliError {
    fn from(r: PhraseRefusal) -> Self {
        CliError::BadInput(r.message())
    }
}

/// Strip exactly one trailing `\n`, and one `\r` before it if present. Nothing else.
pub fn strip_one_trailing_newline(buf: &mut Vec<u8>) {
    if buf.last() == Some(&b'\n') {
        buf.pop();
        if buf.last() == Some(&b'\r') {
            buf.pop();
        }
    }
}

/// Read the phrase from stdin, byte-verbatim. At a terminal, print one prompt
/// line to stderr first (r2 review M-7: the first `ms` input a human types
/// must not look like a hang).
pub fn read_phrase_stdin() -> Result<Zeroizing<Vec<u8>>> {
    use std::io::{IsTerminal, Read};
    let stdin = std::io::stdin();
    if stdin.is_terminal() {
        eprintln!("Type the hashlock phrase, then Enter.");
    }
    let mut buf: Zeroizing<Vec<u8>> = Zeroizing::new(Vec::new());
    stdin
        .lock()
        .read_to_end(&mut buf)
        .map_err(|e| CliError::BadInput(format!("failed to read stdin: {e}")))?;
    strip_one_trailing_newline(&mut buf);
    Ok(buf)
}

/// The rule. Order matters and is the spec's: empty, printable ASCII,
/// ms1-shape (BEFORE the cap), cap, 64-hex.
pub fn validate_phrase(bytes: &[u8]) -> std::result::Result<(), PhraseRefusal> {
    if bytes.is_empty() {
        return Err(PhraseRefusal::Empty);
    }
    if let Some((at, &byte)) = bytes.iter().enumerate().find(|(_, b)| !(0x20..=0x7e).contains(*b)) {
        return Err(PhraseRefusal::NotPrintableAscii { byte, at });
    }
    // All bytes are printable ASCII now, so this is a &str.
    let s = std::str::from_utf8(bytes).expect("printable ASCII is UTF-8");
    if crate::argv_guard::looks_like_ms1(s) {
        return Err(PhraseRefusal::Ms1Shaped);
    }
    if s.len() > HASHLOCK_PHRASE_MAX_CHARS {
        return Err(PhraseRefusal::TooLong { chars: s.len() });
    }
    if s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(PhraseRefusal::Hex64);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_exactly_one_newline() {
        let mut v = b"abc\r\n".to_vec();
        strip_one_trailing_newline(&mut v);
        assert_eq!(v, b"abc");
        let mut v = b"abc\n\n".to_vec();
        strip_one_trailing_newline(&mut v);
        assert_eq!(v, b"abc\n", "only ONE trailing newline is stripped");
        let mut v = b" abc ".to_vec();
        strip_one_trailing_newline(&mut v);
        assert_eq!(v, b" abc ", "spaces are bytes");
    }

    #[test]
    fn empty_is_refused() {
        assert_eq!(validate_phrase(b""), Err(PhraseRefusal::Empty));
    }

    #[test]
    fn printable_boundary_is_pinned_on_both_sides() {
        assert_eq!(validate_phrase(b" ~"), Ok(()));
        assert_eq!(validate_phrase(b"a\tb"), Err(PhraseRefusal::NotPrintableAscii { byte: 0x09, at: 1 }));
        assert_eq!(validate_phrase(b"a\x7f"), Err(PhraseRefusal::NotPrintableAscii { byte: 0x7f, at: 1 }));
        assert_eq!(validate_phrase(b"\xff"), Err(PhraseRefusal::NotPrintableAscii { byte: 0xff, at: 0 }));
        assert_eq!(validate_phrase("café".as_bytes()), Err(PhraseRefusal::NotPrintableAscii { byte: 0xc3, at: 3 }));
    }

    #[test]
    fn ms1_shape_in_four_spellings_and_before_the_cap() {
        let plate = "ms10hashsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq";
        assert_eq!(plate.len(), 75);
        assert_eq!(validate_phrase(plate.as_bytes()), Err(PhraseRefusal::Ms1Shaped), "lowercase");
        assert_eq!(validate_phrase(plate.to_ascii_uppercase().as_bytes()), Err(PhraseRefusal::Ms1Shaped), "UPPERCASE");
        let grouped: String = plate.as_bytes().chunks(5).map(|c| std::str::from_utf8(c).unwrap()).collect::<Vec<_>>().join(" ");
        assert_eq!(validate_phrase(grouped.as_bytes()), Err(PhraseRefusal::Ms1Shaped), "grouped");
        assert_eq!(validate_phrase(format!("  {plate}  ").as_bytes()), Err(PhraseRefusal::Ms1Shaped), "padded");
        let grouped2: String = plate.as_bytes().chunks(2).map(|c| std::str::from_utf8(c).unwrap()).collect::<Vec<_>>().join(" ");
        assert!(grouped2.len() > HASHLOCK_PHRASE_MAX_CHARS);
        assert_eq!(validate_phrase(grouped2.as_bytes()), Err(PhraseRefusal::Ms1Shaped), "shape test precedes the cap");
    }

    #[test]
    fn cap_at_100() {
        assert_eq!(validate_phrase("a".repeat(100).as_bytes()), Ok(()));
        assert_eq!(validate_phrase("a".repeat(101).as_bytes()), Err(PhraseRefusal::TooLong { chars: 101 }));
    }

    #[test]
    fn hex64_either_case_refused_short_hex_accepted() {
        let lower = "c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016";
        assert_eq!(validate_phrase(lower.as_bytes()), Err(PhraseRefusal::Hex64));
        assert_eq!(validate_phrase(lower.to_ascii_uppercase().as_bytes()), Err(PhraseRefusal::Hex64));
        assert_eq!(validate_phrase(b"beef"), Ok(()));
        assert_eq!(validate_phrase(&lower.as_bytes()[..63]), Ok(()), "63 hex characters is a phrase");
    }

    #[test]
    fn refusals_never_echo_the_phrase() {
        let secret = "my very secret phrase\t";
        let msg = validate_phrase(secret.as_bytes()).unwrap_err().message();
        assert!(!msg.contains("my very secret"), "{msg}");
    }
}
```

- [ ] **Step 2: Apply the fragment and run**

Add `mod hashlock_phrase;` to `crates/ms-cli/src/main.rs` beside `mod
format;` (the hand-wire script's entry). Then:

Run: `cargo test -p ms-cli hashlock_phrase`
Expected: PASS, seven unit tests. MUTATE `(0x20..=0x7e)` to `is_ascii()` →
`printable_boundary_is_pinned_on_both_sides` FAILS on TAB. Revert.

- [ ] **Step 3: Commit**

```bash
git add crates/ms-cli/src/hashlock_phrase.rs crates/ms-cli/src/main.rs
git commit -m "ms-cli: hashlock_phrase -- byte-verbatim stdin reader and the phrase rule with its five refusals (H1 Task 6)"
```

---

### Task 7: `ms hashlock`

**Files:**
- Create: `crates/ms-cli/src/cmd/hashlock.rs`
- Modify: `crates/ms-cli/src/cmd/mod.rs` (fragment), `crates/ms-cli/src/main.rs:79-85,221,252` (fragment)
- Test: `crates/ms-cli/tests/hashlock_sources.rs` (Task 5; turns green here), `crates/ms-cli/tests/hashlock_outputs.rs` (Task 9)

**Interfaces:**
- Consumes: `ms_codec::hashlock::*`, `ms_codec::{encode, decode, Payload, Tag}`,
  `hashlock_phrase::*`, `argv_guard::{admitted, CH_POSITIONAL}`,
  `parse::{Source, read_input}`, `out::write_artifact`,
  `format::group_display` (the existing grouping helper — confirm its name with
  `grep -n "pub fn" crates/ms-cli/src/format.rs`), `advisory::{emit_output_class_advisory, OutputClass}`,
  `cmd::encode::parse_hex_entropy`.
- Produces: `cmd::hashlock::{HashlockArgs, run(HashlockArgs) -> Result<u8>}`.

- [ ] **Step 1: Write the verb**

Create `crates/ms-cli/src/cmd/hashlock.rs`:

```rust
//! `ms hashlock` (SPEC_ms_hashlock §4): derive or take a 32-byte preimage,
//! print the `hash:` record, back the preimage up as a plate string.
//!
//! THE POLARITY IS INVERTED HERE and the verb says so on stderr's first line:
//! stdout carries the PUBLIC digest record (`me sysw pack` reads it), stderr
//! carries the SECRET preimage on the card, `--out` carries it to a 0600
//! file, `--json` carries it in one object on stdout in place of the record.
//!
//! EXACTLY ONE SOURCE. Zero and two-or-more both exit 64 -- zero must not
//! default to stdin, or a bare `ms hashlock` at a terminal blocks with no
//! prompt and the phrase an operator then types lands in an ms1 parse error.
//!
//! `--random` REQUIRES `--out FILE` (`--json` is stdout, which `| jq` filters
//! away -- the constructed loss of R0 r0 adversarial C-1) and its `--out`
//! never overwrites (a random preimage is a function of nothing, so a
//! clobbered file cannot be re-made: adversarial C-2).

use std::io::Write;
use std::path::PathBuf;

use clap::{Args, ValueEnum};
use ms_codec::hashlock::{digest, preimage_hardened, preimage_random, preimage_sha256, HASHLOCK_DKLEN, HASHLOCK_ITERATIONS, HASHLOCK_SALT};
use ms_codec::{Payload, Tag};
use zeroize::Zeroizing;

use crate::advisory::{emit_output_class_advisory, OutputClass};
use crate::error::{CliError, Result};
use crate::hashlock_phrase::{read_phrase_stdin, validate_phrase, HASHLOCK_PHRASE_MAX_CHARS};
use crate::parse::{read_input, Source};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Method {
    /// PBKDF2-HMAC-SHA256, salt "ms-hashlock-v1", 100000 iterations (default).
    Hardened,
    /// One SHA-256 of the phrase bytes -- the brainwallet construction.
    Sha256,
}

/// `ms hashlock` arguments.
#[derive(Args, Debug)]
pub struct HashlockArgs {
    /// The hashlock phrase, on argv. A SECRET channel: refused unless --allow-argv-secret.
    #[arg(long, value_name = "TEXT")]
    pub hashlock_phrase: Option<String>,
    /// Read the hashlock phrase from stdin, byte-verbatim (one trailing newline stripped).
    #[arg(long)]
    pub hashlock_phrase_stdin: bool,
    /// An existing preimage: exactly 32 bytes (64 hex characters). `-` reads stdin.
    #[arg(long, value_name = "HEX")]
    pub hex: Option<String>,
    /// A preimage-kind ms1 string, to re-derive the digest from a plate. `-` reads stdin.
    #[arg(value_name = "MS1")]
    pub ms1: Option<String>,
    /// Read the ms1 string from FILE (the six reading verbs' meaning of --in).
    #[arg(long = "in", value_name = "FILE")]
    pub in_path: Option<PathBuf>,
    /// 32 bytes from the OS random source. Requires --out FILE.
    #[arg(long)]
    pub random: bool,
    /// Phrase -> preimage method (phrase sources only).
    #[arg(long, value_enum)]
    pub method: Option<Method>,
    /// Write the preimage ms1 string here, owner-only. Never suppresses stdout.
    #[arg(long, value_name = "FILE")]
    pub out: Option<PathBuf>,
    /// One JSON object on stdout in place of the record line. Carries the secret.
    #[arg(long)]
    pub json: bool,
    /// Suppress the stderr card.
    #[arg(long)]
    pub no_engraving_card: bool,
    /// Group the ms1 on the card every N characters (0 = no grouping).
    #[arg(long, default_value_t = 5)]
    pub group_size: u8,
    /// Group separator on the card.
    #[arg(long, default_value_t = ' ')]
    pub separator: char,
    /// Admit a secret on argv (see `ms encode --help`).
    #[arg(long)]
    pub allow_argv_secret: bool,
}

enum SourceKind {
    Phrase { argv: bool },
    Hex,
    Ms1,
    Random,
}

impl SourceKind {
    fn name(&self) -> &'static str {
        match self {
            SourceKind::Phrase { argv: true } => "--hashlock-phrase",
            SourceKind::Phrase { argv: false } => "--hashlock-phrase-stdin",
            SourceKind::Hex => "--hex",
            SourceKind::Ms1 => "an ms1 string (argument, `-`, or --in FILE)",
            SourceKind::Random => "--random",
        }
    }
}

const FIVE_SOURCES: &str = "exactly one source: --hashlock-phrase TEXT, --hashlock-phrase-stdin, --hex HEX, an ms1 string (argument, `-`, or --in FILE), or --random";

fn pick_source(args: &HashlockArgs) -> Result<SourceKind> {
    let mut chosen: Vec<SourceKind> = Vec::new();
    if args.hashlock_phrase.is_some() || crate::argv_guard::admitted("--hashlock-phrase").is_some() {
        chosen.push(SourceKind::Phrase { argv: true });
    }
    if args.hashlock_phrase_stdin {
        chosen.push(SourceKind::Phrase { argv: false });
    }
    if args.hex.is_some() || crate::argv_guard::admitted("--hex").is_some() {
        chosen.push(SourceKind::Hex);
    }
    if args.ms1.is_some() || args.in_path.is_some() || crate::argv_guard::admitted(crate::argv_guard::CH_POSITIONAL).is_some() {
        chosen.push(SourceKind::Ms1);
    }
    if args.random {
        chosen.push(SourceKind::Random);
    }
    match chosen.len() {
        1 => Ok(chosen.pop().unwrap()),
        0 => Err(CliError::Usage(format!("no source given; {FIVE_SOURCES}"))),
        _ => Err(CliError::Usage(format!(
            "{} and {} were both given; {FIVE_SOURCES}",
            chosen[0].name(),
            chosen[1].name()
        ))),
    }
}

/// The resolved preimage plus what the card must say about where it came from.
struct Derived {
    x: Zeroizing<[u8; 32]>,
    method: Option<Method>,
    phrase_chars: Option<usize>,
    source: &'static str,
}

fn derive(args: &HashlockArgs, source: SourceKind) -> Result<Derived> {
    match source {
        SourceKind::Phrase { argv } => {
            let bytes: Zeroizing<Vec<u8>> = if argv {
                // The admitted side channel wins over the (already rewritten)
                // argv value; the guard replaced the argv token with `-`.
                match crate::argv_guard::admitted("--hashlock-phrase") {
                    Some([first, ..]) => Zeroizing::new(first.as_bytes().to_vec()),
                    _ => Zeroizing::new(args.hashlock_phrase.as_deref().unwrap_or("").as_bytes().to_vec()),
                }
            } else {
                read_phrase_stdin()?
            };
            validate_phrase(&bytes)?;
            let method = args.method.unwrap_or(Method::Hardened);
            let x = match method {
                Method::Hardened => preimage_hardened(&bytes),
                Method::Sha256 => preimage_sha256(&bytes),
            };
            Ok(Derived { x, method: Some(method), phrase_chars: Some(bytes.len()), source: if argv { "phrase (argv, admitted)" } else { "phrase (stdin)" } })
        }
        SourceKind::Hex => {
            refuse_method(args)?;
            let raw = read_input(Source::new(args.hex.as_deref(), None).on("--hex"))?;
            let bytes = crate::cmd::encode::parse_hex_entropy(raw.trim())?;
            if bytes.len() != 32 {
                return Err(CliError::BadInput(format!(
                    "--hex is {} bytes; a hashlock preimage is exactly 32 bytes (64 hex characters) -- see the composer spec's §8i",
                    bytes.len()
                )));
            }
            let mut x = Zeroizing::new([0u8; 32]);
            x.copy_from_slice(&bytes);
            Ok(Derived { x, method: None, phrase_chars: None, source: "preimage supplied (--hex)" })
        }
        SourceKind::Ms1 => {
            refuse_method(args)?;
            let s = read_input(Source::new(args.ms1.as_deref(), args.in_path.as_deref()).on(crate::argv_guard::CH_POSITIONAL))?;
            let (_tag, payload) = ms_codec::decode(&s)?;
            match payload {
                Payload::Preimage(x) => Ok(Derived { x, method: None, phrase_chars: None, source: "preimage supplied (ms1 plate)" }),
                _ => Err(CliError::BadInput(
                    "that is a seed backup, not a hashlock preimage; a preimage plate reads ms10hash... (32 bytes, 64 hex characters)".to_string(),
                )),
            }
        }
        SourceKind::Random => {
            refuse_method(args)?;
            if args.out.is_none() {
                return Err(CliError::Usage(
                    "--random needs --out FILE: a preimage that reaches no file is data loss (--json is stdout and does not count)".to_string(),
                ));
            }
            let x = preimage_random()?;
            Ok(Derived { x, method: None, phrase_chars: None, source: "random (OS CSPRNG)" })
        }
    }
}

fn refuse_method(args: &HashlockArgs) -> Result<()> {
    if args.method.is_some() {
        return Err(CliError::Usage(
            "--method applies to the phrase sources only; with --hex, --random or an ms1 string the preimage is already given".to_string(),
        ));
    }
    Ok(())
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn method_line(d: &Derived) -> String {
    match d.method {
        Some(Method::Hardened) => format!(
            "preimage = PBKDF2-HMAC-SHA256(password = phrase, salt = \"{}\", iterations = {HASHLOCK_ITERATIONS}, dkLen = {HASHLOCK_DKLEN})",
            String::from_utf8_lossy(HASHLOCK_SALT)
        ),
        Some(Method::Sha256) => "preimage = SHA-256(phrase)".to_string(),
        None => "preimage supplied".to_string(),
    }
}

pub fn run(args: HashlockArgs) -> Result<u8> {
    let source = pick_source(&args)?;
    let is_random = matches!(source, SourceKind::Random);
    let d = derive(&args, source)?;
    let h = digest(&d.x);
    let record = format!("hash:{}", hex(&h));
    let ms1 = ms_codec::encode(Tag::HASH, &Payload::Preimage(d.x.clone()))?;

    if let Some(path) = args.out.as_deref() {
        if is_random {
            crate::out::write_artifact_create_new(path, &format!("{ms1}\n"))?;
        } else {
            crate::out::write_artifact(path, &format!("{ms1}\n"))?;
        }
    }

    let mut stdout = std::io::stdout().lock();
    if args.json {
        let mut o = serde_json::Map::new();
        o.insert("digest".into(), hex(&h).into());
        o.insert("hash_record".into(), record.clone().into());
        o.insert("sha256_operand".into(), format!("sha256={}", hex(&h)).into());
        o.insert("preimage_hex".into(), hex(&d.x[..]).into());
        o.insert("preimage_ms1".into(), ms1.clone().into());
        o.insert("source".into(), d.source.into());
        match d.method {
            Some(Method::Hardened) => {
                o.insert("method".into(), serde_json::json!({"kdf": "PBKDF2-HMAC-SHA256", "hash": "SHA-256", "salt": String::from_utf8_lossy(HASHLOCK_SALT), "iterations": HASHLOCK_ITERATIONS, "dklen": HASHLOCK_DKLEN}));
            }
            Some(Method::Sha256) => {
                o.insert("method".into(), serde_json::json!({"hash": "SHA-256"}));
            }
            None => {}
        }
        if let Some(n) = d.phrase_chars {
            o.insert("phrase_chars".into(), (n as u64).into());
        }
        writeln!(stdout, "{}", serde_json::Value::Object(o)).ok();
    } else {
        writeln!(stdout, "{record}").ok();
    }
    drop(stdout);

    let mut stderr = std::io::stderr().lock();
    if !args.no_engraving_card {
        let grouped = crate::format::group_display(&ms1, args.group_size as usize, args.separator);
        writeln!(stderr, "THIS CARD CARRIES THE PREIMAGE -- the secret. stdout carries only the public digest.").ok();
        writeln!(stderr, "digest:          {}", hex(&h)).ok();
        writeln!(stderr, "for md compose:  --path ... sha256={}", hex(&h)).ok();
        writeln!(stderr, "preimage (ms1):  {grouped}").ok();
        writeln!(stderr, "preimage (hex):  {}", hex(&d.x[..])).ok();
        writeln!(stderr, "method:          {}", method_line(&d)).ok();
        if let Some(n) = d.phrase_chars {
            writeln!(stderr, "phrase:          {n} characters -- write the method line next to your phrase; it is on no plate; if the method line is lost, try each method that shipped with the version named on this card (ms-cli {})", env!("CARGO_PKG_VERSION")).ok();
        }
        writeln!(stderr, "The preimage must be exactly 32 bytes (64 hex characters): the script checks OP_SIZE 32 before OP_SHA256 (composer spec §8i, F-132).").ok();
        writeln!(stderr, "One phrase per policy. Spending any path of a wsh wallet publishes this digest. Never use this phrase as a passphrase or a password anywhere else -- a spend publishes the preimage, and anyone can then test guesses at the phrase itself.").ok();
        match d.method {
            Some(Method::Sha256) => {
                writeln!(stderr, "WARNING: this is the brainwallet construction: anyone holding the digest tests 10^10 phrases per second. A phrase a person chose is not safe here; use six diceware words or --random.").ok();
            }
            Some(Method::Hardened) => {
                if d.phrase_chars.unwrap_or(0) < 20 {
                    writeln!(stderr, "WARNING: a 20-character phrase falls in about 72 days on one GPU; choose it from a generator.").ok();
                }
            }
            None => {}
        }
        if d.source.starts_with("preimage supplied (--hex)") {
            writeln!(stderr, "WARNING: the first spend of this hash path publishes these 32 bytes in the clear, forever. If this value is also anything else's secret -- a seed's entropy, a key -- every use of that secret is public with it.").ok();
        }
        if is_random {
            writeln!(stderr, "No phrase exists, so nothing can be guessed, and nothing can be remembered. The file you just wrote is the only copy until you cut the plate.").ok();
        }
        writeln!(stderr, "source:          {}", d.source).ok();
    }
    if args.json {
        emit_output_class_advisory(OutputClass::PrivateKeyMaterial, &mut stderr);
    }
    Ok(0)
}
```

`crate::out::write_artifact_create_new` is new: add to `crates/ms-cli/src/out.rs`
(fragment; the hand-wire script gains this entry in Task 7's commit):

```rust
/// Like `write_artifact`, but REFUSES an existing path (exit 64, naming it)
/// instead of truncating. For `--random` only: that artifact is a function of
/// nothing and cannot be re-made (SPEC_ms_hashlock §4.1).
pub(crate) fn write_artifact_create_new(path: &std::path::Path, body: &str) -> Result<()> {
    if path.exists() {
        return Err(CliError::Usage(format!(
            "--out {} already exists; a --random preimage will not overwrite it (choose another file, or move the old one first)",
            path.display()
        )));
    }
    write_artifact(path, body)
}
```

- [ ] **Step 2: Apply the Task 7 fragments and run the source tests**

The `cmd/mod.rs` and `main.rs` entries of the hand-wire script, plus the
`out.rs` entry above (append it to the script as
`edit("crates/ms-cli/src/out.rs", [("pub(crate) fn write_artifact(", "<the fn above>\n\npub(crate) fn write_artifact(")])`).

Run: `cargo test -p ms-cli --test hashlock_sources`
Expected: PASS, all eleven tests. Then the four named mutations in their
doc comments, one at a time, each failing its test; revert each.

- [ ] **Step 3: Commit**

```bash
git add crates/ms-cli/src/cmd/hashlock.rs crates/ms-cli/src/cmd/mod.rs crates/ms-cli/src/main.rs crates/ms-cli/src/out.rs scripts/plan-handwire-ms-hashlock.py
git commit -m "ms-cli: ms hashlock -- five sources, one at a time; the record on stdout, the preimage on --out and the card; --random gated on --out and never overwriting (H1 Task 7)"
```

---
### Task 8: The other verbs on the new kind

**Files:**
- Modify: `crates/ms-cli/src/cmd/decode.rs:84-125` (fragment: two `Payload::Preimage` arms + `emit_preimage`)
- Modify: `crates/ms-cli/src/cmd/combine.rs:154-167` (fragment: one arm)
- Modify: `crates/ms-cli/src/cmd/payload_lang.rs:37-61` (fragment: the typed refusal; the helper returns `Result`)
- Modify: `crates/ms-cli/src/cmd/verify.rs`, `crates/ms-cli/src/cmd/derive.rs` (fragment: `?` on the helper's new `Result`)
- Modify: `crates/ms-cli/src/cmd/inspect.rs:160-232` (fragment: rules 6/8/9/10, `reason_text`, version line)
- Test: `crates/ms-cli/tests/hashlock_other_verbs.rs`

**Interfaces:**
- Consumes: `Payload::Preimage`, `InspectKind::Preimage`, `TAG_HASH`,
  `VALID_PREIMAGE_STR_LENGTHS`, `ms_codec::hashlock::digest`.
- Produces: `payload_entropy_and_language` now returns
  `Result<(Zeroizing<Vec<u8>>, CliLanguage, bool)>` and refuses a preimage
  with the remedy; `decode` and `combine` print a preimage as a preimage;
  `inspect`'s verdict passes a preimage single with no reason.

- [ ] **Step 1: Write the failing tests**

Create `crates/ms-cli/tests/hashlock_other_verbs.rs`:

```rust
//! The other verbs on the preimage kind (SPEC_ms_hashlock §5), and the
//! structural pins that keep the next kind from re-opening the
//! `#[non_exhaustive]` hazard (§3).

use assert_cmd::Command;

const HEX32: &str = "c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016";
const H: &str = "3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12";

fn ms() -> Command {
    Command::cargo_bin("ms").unwrap()
}

fn plate() -> String {
    let out = ms().args(["hashlock", "--hex", "-", "--json"]).write_stdin(HEX32).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    v["preimage_ms1"].as_str().unwrap().to_string()
}

/// MUTATION: leave decode.rs's catch-all as `unreachable!` -> exit 101.
#[test]
fn decode_prints_kind_hex_and_digest_and_never_words() {
    let out = ms().args(["decode", "-"]).write_stdin(plate()).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let so = String::from_utf8_lossy(&out.stdout);
    assert!(so.contains("preimage"), "{so}");
    assert!(so.contains(HEX32), "{so}");
    assert!(so.contains(H), "{so}");
    for w in ["abandon", "sentence", "entry", "word"] {
        assert!(!so.to_ascii_lowercase().contains(w), "a preimage rendered as words:\n{so}");
    }
}

#[test]
fn decode_json_carries_kind_and_digest() {
    let out = ms().args(["decode", "-", "--json"]).write_stdin(plate()).output().unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["kind"], "preimage");
    assert_eq!(v["preimage_hex"], HEX32);
    assert_eq!(v["digest"], H);
    assert!(v.get("phrase").is_none() && v.get("mnemonic").is_none());
}

/// MUTATION: leave inspect.rs's rule-6/8 copies untouched -> `unknown-tag`
/// and `non-zero-prefix` fire on a valid preimage single.
#[test]
fn inspect_reports_the_kind_with_no_false_reason() {
    let out = ms().args(["inspect", "-"]).write_stdin(plate()).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let so = String::from_utf8_lossy(&out.stdout);
    assert!(so.contains("preimage"), "{so}");
    assert!(!so.contains("unknown-tag") && !so.contains("non-zero-prefix"), "{so}");
    assert!(!so.contains("would NOT decode"), "{so}");
}

/// MUTATION: place the refusal AFTER `payload_entropy_and_language` -> exit
/// 101 from the helper's `unreachable!`.
#[test]
fn derive_and_verify_refuse_with_the_executable_remedy() {
    for verb in ["derive", "verify"] {
        let out = ms().args([verb, "-"]).write_stdin(plate()).output().unwrap();
        assert_ne!(out.status.code(), Some(101), "{verb} panicked");
        assert!(!out.status.success(), "{verb} must refuse a preimage");
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(err.contains("ms hashlock"), "{verb}: the remedy must be executable:\n{err}");
        assert!(!err.contains(HEX32), "{verb} echoed the preimage:\n{err}");
    }
}

/// MUTATION: leave combine.rs's catch-all -> exit 101 on a preimage share set.
#[test]
fn combine_prints_a_recovered_preimage_as_decode_does() {
    // Shares are made through the codec (the CLI has no ms1 source for split:
    // F-468), then combined through the CLI.
    use ms_codec::{encode_shares, Payload, Tag, Threshold};
    let mut x = [0u8; 32];
    for (i, b) in x.iter_mut().enumerate() {
        *b = u8::from_str_radix(&HEX32[2 * i..2 * i + 2], 16).unwrap();
    }
    let shares = encode_shares(Tag::HASH, Threshold::new(2).unwrap(), 2, &Payload::Preimage(zeroize::Zeroizing::new(x))).unwrap();
    let out = ms().args(["combine", "-"]).write_stdin(shares.join("\n")).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let so = String::from_utf8_lossy(&out.stdout);
    assert!(so.contains(HEX32) && so.contains(H), "{so}");
}

/// repair is unchanged and benign on the kind (adversarial M-3).
#[test]
fn repair_on_an_undamaged_preimage_plate_is_a_no_op() {
    let p = plate();
    let out = ms().args(["repair", "--ms1", "-"]).write_stdin(p.clone()).output().unwrap();
    assert_ne!(out.status.code(), Some(101));
}

/// The committed catch-all count (§3): the next kind re-triggers the sweep
/// mechanically. MUTATION: add a fifth `_ => unreachable!` -> this fails.
#[test]
fn unreachable_catch_all_count_is_pinned() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut n = 0;
    fn walk(p: &std::path::Path, n: &mut usize) {
        for e in std::fs::read_dir(p).unwrap() {
            let e = e.unwrap();
            if e.path().is_dir() {
                walk(&e.path(), n);
            } else if e.path().extension().map(|x| x == "rs").unwrap_or(false) {
                *n += std::fs::read_to_string(e.path()).unwrap().matches("_ => unreachable!").count();
            }
        }
    }
    walk(&root, &mut n);
    assert_eq!(n, 4, "the ms-cli `_ => unreachable!` census moved: every catch-all over Payload/PayloadKind/InspectKind needs a Preimage arm before this number changes");
}

/// The SECRET_FLAGS doc comment was corrected while the line was edited (tests N-4).
#[test]
fn secret_flags_doc_comment_counts_five() {
    let s = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/argv_guard.rs")).unwrap();
    assert!(!s.contains("The nine flag-keyed"), "stale doc comment above SECRET_FLAGS");
    assert!(s.contains("const SECRET_FLAGS: [&str; 5]"));
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p ms-cli --test hashlock_other_verbs`
Expected: `decode_*`, `combine_*` exit 101 (the `unreachable!` arms);
`inspect_*` fails on `unknown-tag`; `derive_and_verify_*` exit 101;
`unreachable_catch_all_count_is_pinned` passes (4 today — it must KEEP
passing after the arms are added, because the `_ =>` catch-alls stay).

- [ ] **Step 3: Apply the Task 8 fragments**

Append these entries to `scripts/plan-handwire-ms-hashlock.py` (before the
sentinel write) and apply them to the tree by hand:

```python
edit("crates/ms-cli/src/cmd/decode.rs", [
    # The second match (the one that extracts entropy) gains its arm; the
    # preimage path returns early through emit_preimage.
    ("        // ms_codec::Payload is #[non_exhaustive]; guard against future variants.\n        _ => unreachable!(\"ms-codec decode returned unknown Payload variant\"),\n    };\n",
     "        Payload::Preimage(x) => return emit_preimage(&x, args.json),\n        // ms_codec::Payload is #[non_exhaustive]; guard against future variants.\n        _ => unreachable!(\"ms-codec decode returned unknown Payload variant\"),\n    };\n"),
])
# emit_preimage is appended to decode.rs as a new fn (see the Rust block below).
edit("crates/ms-cli/src/cmd/combine.rs", [
    ("        // ms_codec::Payload is #[non_exhaustive]; guard against future variants.\n        _ => unreachable!(\"combine_shares returned an unknown Payload variant\"),\n    };",
     "        Payload::Preimage(x) => {\n            // A recovered preimage prints as `decode` does and never as words.\n            return crate::cmd::decode::emit_preimage(x, args.json);\n        }\n        // ms_codec::Payload is #[non_exhaustive]; guard against future variants.\n        _ => unreachable!(\"combine_shares returned an unknown Payload variant\"),\n    };"),
])
edit("crates/ms-cli/src/cmd/payload_lang.rs", [
    (") -> (Zeroizing<Vec<u8>>, CliLanguage, bool) {\n    match payload {\n        Payload::Entr(b) => (Zeroizing::new(b), cli_lang, cli_lang_defaulted),",
     ") -> crate::error::Result<(Zeroizing<Vec<u8>>, CliLanguage, bool)> {\n    Ok(match payload {\n        // A preimage is not a seed: refuse HERE, before the catch-all, with\n        // the executable remedy (SPEC_ms_hashlock §5; review I-3).\n        Payload::Preimage(_) => {\n            return Err(crate::error::CliError::BadInput(\n                \"this is a hashlock preimage plate, not a seed backup; use `ms hashlock <ms1>` (or `ms hashlock --in FILE`) to re-derive its digest\".to_string(),\n            ))\n        }\n        Payload::Entr(b) => (Zeroizing::new(b), cli_lang, cli_lang_defaulted),"),
    ("        // ms_codec::Payload is #[non_exhaustive]; guard against future variants.\n        _ => unreachable!(\"ms-codec decode returned unknown Payload variant\"),\n    }\n}",
     "        // ms_codec::Payload is #[non_exhaustive]; guard against future variants.\n        _ => unreachable!(\"ms-codec decode returned unknown Payload variant\"),\n    })\n}"),
])
edit("crates/ms-cli/src/cmd/inspect.rs", [
    ("    if tag_bytes != TAG_ENTR {", "    if tag_bytes != TAG_ENTR && tag_bytes != TAG_HASH {"),
    ("        InspectKind::Mnem => VALID_MNEM_STR_LENGTHS,\n        _ => VALID_STR_LENGTHS,",
     "        InspectKind::Mnem => VALID_MNEM_STR_LENGTHS,\n        InspectKind::Preimage => VALID_PREIMAGE_STR_LENGTHS,\n        _ => VALID_STR_LENGTHS,"),
    ("        InspectKind::Mnem => {\n            // payload_bytes = [lang_byte, entropy...]; valid if len - 1 ∈ VALID_ENTR_LENGTHS.",
     "        InspectKind::Preimage => {\n            if report.payload_bytes.len() != 32 {\n                reasons.push(\"payload-length-mismatch\");\n            }\n            if tag_bytes != TAG_HASH {\n                reasons.push(\"tag-kind-mismatch\");\n            }\n        }\n        InspectKind::Mnem => {\n            // payload_bytes = [lang_byte, entropy...]; valid if len - 1 ∈ VALID_ENTR_LENGTHS."),
    ("        \"unknown-tag\" => \"tag not in v0.1 RESERVED_TAG_TABLE\",\n        \"non-zero-prefix\" => \"prefix byte is not a recognised kind (0x00=entr, 0x02=mnem)\",",
     "        \"unknown-tag\" => \"tag not in the accept set (entr, hash)\",\n        \"tag-kind-mismatch\" => \"the id names a different kind than the prefix byte carries\",\n        \"non-zero-prefix\" => \"prefix byte is not a recognised kind (0x00=entr, 0x02=mnem, 0x03=preimage)\","),
    ("            InspectKind::Mnem => \"v0.2\",\n            _ => \"v0.1\",",
     "            InspectKind::Mnem => \"v0.2\",\n            InspectKind::Preimage => \"v0.8\",\n            _ => \"v0.1\","),
])
```

The remaining Task 8 entries, exact (these are in the committed script; the
plan carries them so the two cannot drift):

```python
edit("crates/ms-cli/src/cmd/inspect.rs", [
    ("use ms_codec::consts::{", "use ms_codec::consts::{TAG_HASH, VALID_PREIMAGE_STR_LENGTHS, "),
])
edit("crates/ms-codec/src/inspect.rs", [
    ("use crate::consts::MNEM_PREFIX;", "use crate::consts::{MNEM_PREFIX, PREIMAGE_PREFIX};"),
    ("            InspectKind::Unknown => \"unknown\",", "            InspectKind::Preimage => \"preimage\",\n            InspectKind::Unknown => \"unknown\","),
    ("        _ => (InspectKind::Unknown, None),", "        PREIMAGE_PREFIX => (InspectKind::Preimage, None),\n        _ => (InspectKind::Unknown, None),"),
])
# verify.rs and derive.rs: the helper now returns Result, so each call gains `?`
# -- exact anchors, because one call is a match-arm expression (`),`) and the
# other a statement (`);`).
edit("crates/ms-cli/src/cmd/verify.rs", [
    ("            &mut stderr,\n        ),\n        Err(ms_codec::Error::ReservedTagNotEmittedInV01 { got }) => {",
     "            &mut stderr,\n        )?,\n        Err(ms_codec::Error::ReservedTagNotEmittedInV01 { got }) => {"),
])
edit("crates/ms-cli/src/cmd/derive.rs", [
    ("                    &mut stderr,\n                );\n            let m = Mnemonic::from_entropy_in(",
     "                    &mut stderr,\n                )?;\n            let m = Mnemonic::from_entropy_in("),
])
```

Add to `crates/ms-cli/src/cmd/decode.rs` (appended; also an `edit` entry
that appends it, so the gate sees it):

```rust
/// Render a preimage: kind, hex, digest. NEVER words -- a preimage is not
/// entropy, and a 24-word rendering would be a seed nobody holds
/// (SPEC_ms_hashlock §5).
pub(crate) fn emit_preimage(x: &[u8; 32], json: bool) -> crate::error::Result<u8> {
    use std::io::Write;
    let h = ms_codec::hashlock::digest(x);
    let hx: String = x.iter().map(|b| format!("{b:02x}")).collect();
    let hh: String = h.iter().map(|b| format!("{b:02x}")).collect();
    let mut out = std::io::stdout().lock();
    if json {
        writeln!(out, "{}", serde_json::json!({"kind": "preimage", "preimage_hex": hx, "digest": hh})).ok();
    } else {
        writeln!(out, "kind:      preimage (hashlock, 32 bytes / 64 hex characters)").ok();
        writeln!(out, "preimage:  {hx}").ok();
        writeln!(out, "digest:    {hh}").ok();
    }
    drop(out);
    let mut err = std::io::stderr().lock();
    crate::advisory::emit_output_class_advisory(crate::advisory::OutputClass::PrivateKeyMaterial, &mut err);
    Ok(0)
}
```

- [ ] **Step 4: Run**

Run: `cargo test -p ms-cli --test hashlock_other_verbs && cargo test -p ms-cli`
Expected: PASS, and the whole ms-cli suite still green (the existing
`inspect_reserved_tag.rs`, `inspect_non_zero_prefix.rs`,
`decode_rejects_unknown_tag.rs` tests must still pass: they use tags/prefixes
outside the new accept set — if one of them used `0x03` or `hash`, flip it
to `0x01`/`seed` per Task 2 Step 5 and record it).

- [ ] **Step 5: Commit**

```bash
git add crates/ms-cli/src/cmd/decode.rs crates/ms-cli/src/cmd/combine.rs crates/ms-cli/src/cmd/payload_lang.rs crates/ms-cli/src/cmd/verify.rs crates/ms-cli/src/cmd/derive.rs crates/ms-cli/src/cmd/inspect.rs crates/ms-cli/tests/hashlock_other_verbs.rs scripts/plan-handwire-ms-hashlock.py
git commit -m "ms-cli: decode/combine print a preimage as a preimage, derive/verify refuse it with the remedy before the catch-all, inspect passes it with no false reason; the catch-all count is pinned (H1 Task 8)"
```

---

### Task 9: The CLI test matrix — phrase rule, outputs, negative content

**Files:**
- Create: `crates/ms-cli/tests/hashlock_phrase_rule.rs`
- Create: `crates/ms-cli/tests/hashlock_outputs.rs`
- Create: `crates/ms-cli/tests/hashlock_negative_content.rs`
- Modify: `crates/ms-cli/tests/exit_codes_table.rs` (Add: one `Usage` row), `crates/ms-cli/tests/in_flag_six_verbs.rs` (the doc comment names six; `hashlock` is the seventh — extend the table with a `hashlock` row whose `--in` binds to the ms1)

**Interfaces:**
- Consumes: the verb from Task 7.

- [ ] **Step 1: The phrase rule through both channels, byte-exact**

Create `crates/ms-cli/tests/hashlock_phrase_rule.rs`:

```rust
//! The phrase rule (SPEC_ms_hashlock §4.3) driven through the BINARY on both
//! channels, and the byte-exact rows no codec vector can see (correctness
//! I-6.1): the mutation is swapping in `read_phrase_input`/`read_input` on
//! either channel.

use assert_cmd::Command;

fn ms() -> Command {
    Command::cargo_bin("ms").unwrap()
}

fn record_via_stdin(phrase: &[u8], method: &str) -> (Option<i32>, String, String) {
    let out = ms().args(["hashlock", "--hashlock-phrase-stdin", "--method", method, "--no-engraving-card"]).write_stdin(phrase.to_vec()).output().unwrap();
    (out.status.code(), String::from_utf8_lossy(&out.stdout).to_string(), String::from_utf8_lossy(&out.stderr).to_string())
}

fn record_via_argv(phrase: &str, method: &str) -> (Option<i32>, String, String) {
    let out = ms().args(["hashlock", "--allow-argv-secret", "--hashlock-phrase", phrase, "--method", method, "--no-engraving-card"]).write_stdin("").output().unwrap();
    (out.status.code(), String::from_utf8_lossy(&out.stdout).to_string(), String::from_utf8_lossy(&out.stderr).to_string())
}

/// Byte-exact through BOTH channels, equal to the codec's own answer.
#[test]
fn byte_exact_rows_on_both_channels() {
    for phrase in ["  a  b ", "a-b,c", "correct-horse,battery staple"] {
        let expect = {
            let x = ms_codec::hashlock::preimage_sha256(phrase.as_bytes());
            let h = ms_codec::hashlock::digest(&x);
            format!("hash:{}", h.iter().map(|b| format!("{b:02x}")).collect::<String>())
        };
        let (code, so, se) = record_via_stdin(phrase.as_bytes(), "sha256");
        assert_eq!(code, Some(0), "stdin {phrase:?}: {se}");
        assert_eq!(so.trim(), expect, "stdin channel changed the bytes of {phrase:?}");
        let (code, so, se) = record_via_argv(phrase, "sha256");
        assert_eq!(code, Some(0), "argv {phrase:?}: {se}");
        assert_eq!(so.trim(), expect, "argv channel changed the bytes of {phrase:?}");
    }
}

#[test]
fn stdin_strips_exactly_one_newline() {
    let a = record_via_stdin(b"abc\n", "sha256").1;
    let b = record_via_stdin(b"abc", "sha256").1;
    let c = record_via_stdin(b"abc\n\n", "sha256").1;
    assert_eq!(a, b, "one trailing LF is stripped");
    assert_ne!(b, c, "two trailing LFs keep one");
    let d = record_via_stdin(b"abc\r\n", "sha256").1;
    assert_eq!(a, d, "CRLF is one newline");
}

#[test]
fn refusals_in_four_spellings_on_both_channels_name_the_ms1_route() {
    let plate = String::from_utf8(
        ms().args(["hashlock", "--hex", "-", "--json"])
            .write_stdin("c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016")
            .output().unwrap().stdout,
    )
    .unwrap();
    let plate: String = serde_json::from_str::<serde_json::Value>(&plate).unwrap()["preimage_ms1"].as_str().unwrap().to_string();
    let grouped5: String = plate.as_bytes().chunks(5).map(|c| std::str::from_utf8(c).unwrap()).collect::<Vec<_>>().join(" ");
    let grouped2: String = plate.as_bytes().chunks(2).map(|c| std::str::from_utf8(c).unwrap()).collect::<Vec<_>>().join(" ");
    for (name, s) in [
        ("lowercase", plate.clone()),
        ("UPPERCASE", plate.to_ascii_uppercase()),
        ("grouped", grouped5),
        ("padded", format!("  {plate}  ")),
        ("grouped-by-2 (112 chars: the shape test precedes the cap)", grouped2),
    ] {
        let (code, _, se) = record_via_stdin(s.as_bytes(), "sha256");
        assert_eq!(code, Some(1), "stdin {name}: {se}");
        assert!(se.contains("--in"), "stdin {name} must name the ms1 route:\n{se}");
        assert!(!se.contains("100 characters") || !name.starts_with("grouped-by-2"), "cap fired before the shape test:\n{se}");
        // The argv channel: the guard's shape layer catches a plate string
        // FIRST (it is ms1 material on argv) and names --in itself.
        let out = ms().args(["hashlock", "--hashlock-phrase", &s]).output().unwrap();
        assert_eq!(out.status.code(), Some(1), "argv {name}");
        assert!(String::from_utf8_lossy(&out.stderr).contains("--in"), "argv {name} must name the ms1 route");
    }
}

#[test]
fn hex64_either_case_is_redirected_to_hex_on_stdin_and_short_hex_is_accepted() {
    let lower = "c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016";
    for s in [lower.to_string(), lower.to_ascii_uppercase()] {
        let (code, _, se) = record_via_stdin(s.as_bytes(), "sha256");
        assert_eq!(code, Some(1), "{se}");
        assert!(se.contains("--hex") && se.contains("64 hex characters"), "{se}");
    }
    let (code, _, se) = record_via_stdin(b"beef", "sha256");
    assert_eq!(code, Some(0), "{se}");
}

#[test]
fn printable_ascii_boundary_and_cap() {
    assert_eq!(record_via_stdin(b" ~", "sha256").0, Some(0));
    for bad in [b"a\tb".to_vec(), b"a\x7f".to_vec(), vec![0xff], "caf\u{e9}".as_bytes().to_vec()] {
        let (code, _, se) = record_via_stdin(&bad, "sha256");
        assert_eq!(code, Some(1), "{bad:?}: {se}");
        assert!(se.contains("printable ASCII"), "the rule must be named:\n{se}");
    }
    assert_eq!(record_via_stdin("a".repeat(100).as_bytes(), "sha256").0, Some(0));
    let (code, _, se) = record_via_stdin("a".repeat(101).as_bytes(), "sha256");
    assert_eq!(code, Some(1));
    assert!(se.contains("100"), "{se}");
    let (code, _, se) = record_via_stdin(b"", "sha256");
    assert_eq!(code, Some(1));
    assert!(se.contains("empty"), "{se}");
}

/// The 100/101 lockstep rows derive identically on the host to the corpus.
#[test]
fn lockstep_100_and_101() {
    let p100 = "a".repeat(100);
    let (code, so, _) = record_via_stdin(p100.as_bytes(), "hardened");
    assert_eq!(code, Some(0));
    let x = ms_codec::hashlock::preimage_hardened(p100.as_bytes());
    let h = ms_codec::hashlock::digest(&x);
    assert_eq!(so.trim(), format!("hash:{}", h.iter().map(|b| format!("{b:02x}")).collect::<String>()));
}
```

- [ ] **Step 2: Outputs, warnings at their boundaries, `--json` both variants**

Create `crates/ms-cli/tests/hashlock_outputs.rs`:

```rust
//! stdout purity in the two configurations where a mutation can hide, the
//! card per source and method, the warnings at their boundaries, `--json`
//! in both variants (SPEC_ms_hashlock §4.4, §7; tests I-7, I-8, M-10).

use assert_cmd::Command;

const PHRASE: &str = "correct horse battery staple";
const H_HARD: &str = "hash:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12";
const H_SHA: &str = "hash:b867db875479bcc0287352cdaa4a1755689b8338777d0915e9acd9f6edbc96cb";
const HEX32: &str = "c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016";

fn ms() -> Command {
    Command::cargo_bin("ms").unwrap()
}

/// MUTATION: `--out` suppressing stdout (encode's shape) -> the first case
/// has empty stdout; a warning printed to stdout -> the second case has two
/// lines.
#[test]
fn stdout_is_exactly_the_record_under_out_and_under_sha256() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("x.txt");
    let out = ms().args(["hashlock", "--hashlock-phrase-stdin", "--out", p.to_str().unwrap()]).write_stdin(PHRASE).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout), format!("{H_HARD}\n"), "under --out");
    let out = ms().args(["hashlock", "--hashlock-phrase-stdin", "--method", "sha256"]).write_stdin(PHRASE).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout), format!("{H_SHA}\n"), "under sha256, which always warns");
    assert!(String::from_utf8_lossy(&out.stderr).contains("brainwallet"));
}

#[test]
fn the_card_names_the_preimage_on_its_first_line_and_carries_the_method_line() {
    let out = ms().args(["hashlock", "--hashlock-phrase-stdin"]).write_stdin(PHRASE).output().unwrap();
    let se = String::from_utf8_lossy(&out.stderr);
    let first = se.lines().next().unwrap();
    assert!(first.to_ascii_uppercase().contains("PREIMAGE"), "first line: {first}");
    assert!(se.contains("preimage = PBKDF2-HMAC-SHA256(password = phrase, salt = \"ms-hashlock-v1\", iterations = 100000, dkLen = 32)"), "{se}");
    assert!(se.contains("28 characters"), "the character count:\n{se}");
    assert!(se.contains("each method that shipped with the version named on this card"), "{se}");
    assert!(se.contains("One phrase per policy"), "{se}");
    assert!(se.contains("OP_SIZE 32") || se.contains("32 bytes (64 hex characters)"), "{se}");
}

/// MUTATION: hardened threshold at 19 or 21 -> one of these flips.
#[test]
fn hardened_warns_under_20_only() {
    let se19 = String::from_utf8_lossy(&ms().args(["hashlock", "--hashlock-phrase-stdin"]).write_stdin("a".repeat(19)).output().unwrap().stderr).to_string();
    let se20 = String::from_utf8_lossy(&ms().args(["hashlock", "--hashlock-phrase-stdin"]).write_stdin("a".repeat(20)).output().unwrap().stderr).to_string();
    assert!(se19.contains("72 days"), "19 chars must warn:\n{se19}");
    assert!(!se20.contains("72 days"), "20 chars must not warn:\n{se20}");
}

/// MUTATION: sha256 gated on length -> the 100-char case stops warning.
#[test]
fn sha256_warns_at_every_length() {
    for n in [1usize, 28, 100] {
        let se = String::from_utf8_lossy(&ms().args(["hashlock", "--hashlock-phrase-stdin", "--method", "sha256"]).write_stdin("b".repeat(n)).output().unwrap().stderr).to_string();
        assert!(se.contains("brainwallet"), "{n} chars:\n{se}");
    }
}

#[test]
fn hex_source_gets_the_unconditional_warning_and_no_write_it_down_line() {
    let out = ms().args(["hashlock", "--hex", "-"]).write_stdin(HEX32).output().unwrap();
    let se = String::from_utf8_lossy(&out.stderr);
    assert!(se.contains("publishes these 32 bytes in the clear"), "{se}");
    assert!(se.contains("preimage supplied"), "{se}");
    assert!(!se.contains("write the method line next to your phrase"), "no phrase, no instruction:\n{se}");
    assert!(!se.contains("brainwallet") && !se.contains("72 days"), "method-keyed warnings must not fire:\n{se}");
}

#[test]
fn random_card_names_the_file_not_a_plate() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("x.txt");
    let out = ms().args(["hashlock", "--random", "--out", p.to_str().unwrap()]).output().unwrap();
    let se = String::from_utf8_lossy(&out.stderr);
    assert!(se.contains("nothing can be remembered"), "{se}");
    assert!(se.contains("The file you just wrote is the only copy"), "{se}");
    assert!(!se.contains("This plate is the only copy"), "{se}");
}

/// Both `--json` variants; every hex lowercase; the advisory fires.
#[test]
fn json_both_variants() {
    let out = ms().args(["hashlock", "--hashlock-phrase-stdin", "--json"]).write_stdin(PHRASE).output().unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["hash_record"], H_HARD);
    assert_eq!(v["method"]["iterations"], 100000);
    assert_eq!(v["method"]["salt"], "ms-hashlock-v1");
    assert_eq!(v["phrase_chars"], 28);
    for k in ["digest", "preimage_hex", "sha256_operand"] {
        let s = v[k].as_str().unwrap();
        assert_eq!(s, s.to_ascii_lowercase(), "{k} must be lowercase hex");
    }
    assert!(String::from_utf8_lossy(&out.stderr).contains("private key material") || String::from_utf8_lossy(&out.stderr).to_ascii_lowercase().contains("secret"));
    let out = ms().args(["hashlock", "--hex", "-", "--json"]).write_stdin(HEX32).output().unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(v.get("method").is_none(), "method omitted for a supplied preimage");
    assert!(v.get("phrase_chars").is_none(), "phrase_chars omitted for a supplied preimage");
    assert_eq!(v["preimage_hex"], HEX32);
}

/// `--random --json --out FILE` succeeds (the gate is on --out, not on json).
#[test]
fn random_json_with_out_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("x.txt");
    let out = ms().args(["hashlock", "--random", "--out", p.to_str().unwrap(), "--json", "--no-engraving-card"]).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(v["preimage_ms1"].as_str().unwrap().starts_with("ms10hashsq"));
}

/// The record feeds the composer's packer unmodified (§12.6): the spelling
/// is stdin with NO --in. Skipped only if `me` is not installed -- and then
/// it SAYS so on stderr and still passes the shape check.
#[test]
fn record_line_shape_is_what_me_sysw_pack_reads() {
    let out = ms().args(["hashlock", "--hashlock-phrase-stdin", "--no-engraving-card"]).write_stdin(PHRASE).output().unwrap();
    let line = String::from_utf8_lossy(&out.stdout);
    assert!(line.starts_with("hash:") && line.trim().len() == 5 + 64);
    assert!(line.trim()[5..].bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()));
}
```

- [ ] **Step 3: The negative-content matrix**

Create `crates/ms-cli/tests/hashlock_negative_content.rs`:

```rust
//! Eleven refusals, and in none of them does the phrase or the preimage
//! appear on stdout, stderr or in the `--json` error envelope
//! (SPEC_ms_hashlock §11; Minor class by the 2026-08-27 ruling, recorded
//! because the brainstorm agreed the matrix). MUTATION: a refusal built with
//! `format!("... {phrase}")`.

use assert_cmd::Command;

const SECRET_PHRASE: &str = "zebra quantum lantern violet";
const SECRET_HEX: &str = "c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016";

fn ms() -> Command {
    Command::cargo_bin("ms").unwrap()
}

fn assert_silent(args: &[&str], stdin: &[u8], secrets: &[&str], label: &str) {
    for json in [false, true] {
        let mut a = args.to_vec();
        if json {
            a.push("--json");
        }
        let out = ms().args(&a).write_stdin(stdin.to_vec()).output().unwrap();
        assert!(!out.status.success(), "{label} (json={json}) must refuse");
        assert_ne!(out.status.code(), Some(101), "{label} panicked");
        let all = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
        for s in secrets {
            assert!(!all.contains(s), "{label} (json={json}) echoed material:\n{all}");
        }
    }
}

#[test]
fn eleven_refusals_never_echo() {
    let tab_phrase = format!("{SECRET_PHRASE}\t");
    let long_phrase = format!("{SECRET_PHRASE}{}", "x".repeat(100));
    let plate = "ms10hashsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq";
    assert_silent(&["hashlock", "--hashlock-phrase-stdin"], b"", &[SECRET_PHRASE], "empty");
    assert_silent(&["hashlock", "--hashlock-phrase-stdin"], "caf\u{e9} zebra quantum".as_bytes(), &["zebra quantum"], "non-ascii");
    assert_silent(&["hashlock", "--hashlock-phrase-stdin"], tab_phrase.as_bytes(), &[SECRET_PHRASE], "control byte");
    assert_silent(&["hashlock", "--hashlock-phrase-stdin"], long_phrase.as_bytes(), &[SECRET_PHRASE], "over 100");
    assert_silent(&["hashlock", "--hashlock-phrase-stdin"], SECRET_HEX.as_bytes(), &[SECRET_HEX], "64-hex");
    assert_silent(&["hashlock", "--hashlock-phrase-stdin"], plate.as_bytes(), &[plate], "ms1-shaped");
    assert_silent(&["hashlock", "--hex", "-"], b"abcd", &["abcd"], "--hex wrong length");
    assert_silent(&["hashlock", "-"], b"ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f", &["ms10entrsqqqqq"], "wrong ms1 kind");
    assert_silent(&["hashlock"], b"", &[SECRET_PHRASE], "zero sources");
    assert_silent(&["hashlock", "--hashlock-phrase-stdin", "--hex", SECRET_HEX], SECRET_PHRASE.as_bytes(), &[SECRET_PHRASE, SECRET_HEX], "two sources");
    assert_silent(&["hashlock", "--hex", SECRET_HEX, "--method", "sha256", "--allow-argv-secret"], b"", &[SECRET_HEX], "--method with a supplied X");
}
```

- [ ] **Step 4: Extend the two existing tables**

Add to `crates/ms-cli/tests/exit_codes_table.rs` one test asserting
`ms hashlock` (no source) exits 64 (`CliError::Usage`), so the SPEC §6 table
gains its row. In `crates/ms-cli/tests/in_flag_six_verbs.rs`, add `hashlock`
to the verbs whose `--in` binds to the ms1 (its equality gate: `--in FILE`
byte-equals stdin at the same exit code, and `--in` + `-` refuses), and
rename nothing (the file's name is history).

- [ ] **Step 5: Run the whole ms-cli suite, then the named mutations**

Run: `cargo nextest run -p ms-cli --locked --no-fail-fast`
Expected: all green. Then each MUTATION named in the three files' doc
comments, one at a time, fails its named test; revert each.

- [ ] **Step 6: Commit**

```bash
git add crates/ms-cli/tests/hashlock_phrase_rule.rs crates/ms-cli/tests/hashlock_outputs.rs crates/ms-cli/tests/hashlock_negative_content.rs crates/ms-cli/tests/exit_codes_table.rs crates/ms-cli/tests/in_flag_six_verbs.rs
git commit -m "ms-cli: the hashlock test matrix -- phrase rule on both channels byte-exact, outputs and warnings at their boundaries, json both variants, the negative-content matrix, the exit-64 row, --in on the seventh verb (H1 Task 9)"
```

---

### Task 10: Records — MIGRATION, CHANGELOG, corpus SHA, man page, manual

**Files:**
- Modify: `MIGRATION.md` (append the 0.7 → 0.8 section)
- Modify: `CHANGELOG.md` (two entries at the top)
- Modify: `crates/ms-cli/src/cmd/gen_man.rs` only if the man page is not generated from clap (check: `grep -n "Hashlock\|Command::" crates/ms-cli/src/cmd/gen_man.rs`); otherwise nothing
- Modify (cross-repo): `mnemonic-toolkit/docs/manual/src/40-cli-reference/43-ms.md`

- [ ] **Step 1: MIGRATION.md**

Append:

```markdown
## v0.7 → v0.8 (the hashlock preimage kind — `0x03`, id `hash`)

v0.8 adds a THIRD payload kind on the prefix-byte axis: `0x03` = a hashlock
preimage, exactly `[0x03][X:32]` (33 bytes, a 75-character single). Five
invariants, each with a measured reason (`design/SPEC_ms_hashlock.md`):

1. **Readers that dispatch on the prefix byte MUST treat `0x03` as a 32-byte
   preimage and never as entropy.** `ms decode` prints it as kind + hex +
   digest and never as words.
2. **Length no longer implies kind.** A preimage single is 75 characters —
   exactly entr-32 — and shares entr's leading payload character `q`. So
   preimage SINGLES carry the id `hash` (`ms10hashsq…`), the id joins
   `RESERVED_ID_BLOCKLIST`, and **a single whose id and prefix byte disagree
   is refused** (`Error::TagKindMismatch`), never read as the other kind.
3. **Sweep every catch-all over `Payload`, `PayloadKind` and `InspectKind`** —
   `_ => <value>` arms as much as `_ => unreachable!` — because
   `#[non_exhaustive]` means the compiler will not. `InspectKind` is NOT
   `#[non_exhaustive]`, so adding `Preimage` is source-breaking for an
   exhaustive match: loud, therefore safe.
4. **The by-hand recipe this constellation documented before 0.18.0 — "hash
   the passphrase to 32 bytes, then hash again" — is `ms hashlock --method
   sha256`, NOT the default.** The default is the hardened method
   (PBKDF2-HMAC-SHA256, salt `ms-hashlock-v1`, 100,000 iterations). A digest
   made by hand reproduces only with `--method sha256`.
5. **A third reader shape exists and it is the dangerous one: "decode
   succeeded, therefore this is a seed."** Measured: `me`'s `validate_record`
   maps ANY `ms_codec::decode` success to a secret seed record; the
   SeedHammer fork's `isStrictMs1` has no prefix test at all. Neither
   dispatches on the prefix, so items 1 and 3 do not reach them. **Before
   this release ships, both are guarded (H0):** the fork's classifier treats
   `0x03` as inert and is flashed; `me`'s record validator treats it as inert
   in the same window as its ms-codec 0.8 bump.

Older `ms` (ms-codec 0.7) refuses a `0x03` single with
`reserved-prefix byte was 0x03` (exit 2) — a refusal, never a seed. The
downgrade row in `scripts/plan-build-gate-ms.sh` step 6 proves it against the
pre-0.8 tree.

New API: `ms_codec::hashlock::{HASHLOCK_SALT, HASHLOCK_ITERATIONS,
HASHLOCK_DKLEN, preimage_hardened, preimage_sha256, preimage_random, digest}`;
`Payload::Preimage(Zeroizing<[u8; 32]>)`; `PayloadKind::Preimage` and
`PayloadKind::single_tag`; `InspectKind::Preimage`; `Tag::HASH`;
`Error::{PreimageLengthMismatch, TagKindMismatch, RandomnessUnavailable}`.
Corpus: `crates/ms-codec/tests/vectors/hashlock-v0.8.json`, SHA-pinned in
the CHANGELOG.
```

- [ ] **Step 2: CHANGELOG.md**

Above the `## ms-cli [0.17.1]` entry, add (the SHA is computed at the
release commit: `sha256sum crates/ms-codec/tests/vectors/hashlock-v0.8.json`):

```markdown
## ms-cli [0.18.0] — <date>

### What's new
- `ms hashlock`: derive a 32-byte hashlock preimage from a phrase (hardened
  PBKDF2 by default, or `--method sha256`), take one with `--hex`, re-read one
  from a plate (`<ms1>`, `-`, `--in FILE`), or draw one with `--random`. Prints
  the `hash:` record on stdout (`… | me sysw pack`), the preimage plate string
  to `--out` (owner-only; never overwritten under `--random`), and a card on
  stderr whose first line says it carries the preimage. `--random` requires
  `--out FILE`.
- `decode`, `inspect`, `combine` read the new kind; `derive` and `verify`
  refuse it with `ms hashlock <ms1>` as the remedy; `repair` is unchanged.
- The argv guard learns `--hashlock-phrase`; the ms1-shape test now
  case-folds INSIDE the predicate, so an uppercase plate string is caught on
  every channel.

### What didn't change
- Every entr and mnem string, byte for byte. `ms encode --hex` still emits
  `entr`; `ms hashlock` is the only door that creates the kind.

### Migration notes
- See MIGRATION.md v0.7 → v0.8. `CliError::Usage` (exit 64) is new.

## ms-codec [0.8.0] — <date>

### What's new
- Kind `0x03`, id `hash`: `Payload::Preimage`, `Tag::HASH`, the accept set
  and the tag/kind consistency check on decode and encode;
  `ms_codec::hashlock` (both derivations, the random source, the digest).
- Corpus `tests/vectors/hashlock-v0.8.json`, SHA-256 `<sha>`.

### What didn't change
- v0.1/v0.2 wire bytes; the share axis.

### Migration notes
- Source-breaking for exhaustive matches on `InspectKind`; every catch-all
  over `Payload`/`PayloadKind` needs a `Preimage` arm (MIGRATION.md v0.8).
```

- [ ] **Step 3: The man page and the manual**

Run: `cargo run -p ms-cli -- gen-man --out /tmp/ms-man && ls /tmp/ms-man && grep -l hashlock /tmp/ms-man/*`
Expected: a `ms-hashlock.1` (or the verb inside `ms.1`) is generated from
clap; the `gen_man.rs` test passes. Then, in mnemonic-toolkit, add the
`ms hashlock` section to `docs/manual/src/40-cli-reference/43-ms.md` with
every flag from `HashlockArgs` and the §9 item-4 note ("the by-hand recipe is
`--method sha256`"), and run the toolkit's flag-coverage lint
(`grep -rn "flag-coverage\|flag_coverage" mnemonic-toolkit/docs mnemonic-toolkit/scripts | head -3` names it). That edit lands in the toolkit repo via its own
staging ritual, referenced from this repo's `design/FOLLOWUPS.md` at tier
`cross-repo` per RELEASE_PROCESS item 6.

- [ ] **Step 4: Commit**

```bash
git add MIGRATION.md CHANGELOG.md
git commit -m "records: MIGRATION v0.7 -> v0.8 (five invariants, the third reader shape, H0); CHANGELOG entries for ms-codec 0.8.0 and ms-cli 0.18.0 (H1 Task 10)"
```

---

### Task 11: Release — H0 first, then 0.8.0 and 0.18.0 together

**Files:**
- Modify: `crates/ms-codec/Cargo.toml`, `crates/ms-cli/Cargo.toml` (Task 2/5 fragments already bumped them; confirm)
- Modify: `CHANGELOG.md` (the corpus SHA and the date)

- [ ] **Step 1: The H0 gate — before anything is tagged**

Run, and paste the outputs into the release commit message:
1. `git -C /scratch/code/shibboleth/seedhammer log --oneline -1 origin/main`
   and the H0 fork merge SHA: the flashed device's version line reads
   `bg<that sha>` and `sysw.Classify` on the corpus's `kind[0].ms1` is NOT
   `ClassCodex32Secret` (the fork's H0 test names it).
2. `me`'s H0 commit: `me sysw pack` fed the same string refuses it as inert
   (not `RecordKind::Ms`), on a `me` built at the bump.
If either is missing, STOP: the release waits (spec §9, §12.7).

- [ ] **Step 2: The per-release checklist, in order**

```bash
sha256sum crates/ms-codec/tests/vectors/hashlock-v0.8.json   # -> CHANGELOG
cargo build --workspace --locked && cargo nextest run --workspace --locked && cargo clippy --workspace --all-targets --locked -- -D warnings && cargo fmt --check
cargo publish -p ms-codec --dry-run --locked
git add CHANGELOG.md && git commit -m "release: ms-codec 0.8.0 + ms-cli 0.18.0 -- corpus SHA pinned; H0 shipped (fork <sha>, me <sha>)"
```
Then the staging ritual by hand (the four required contexts), then
`git tag -a ms-codec-v0.8.0 -m "ms-codec 0.8.0: the hashlock preimage kind"`
and `git tag -a ms-cli-v0.18.0 -m "ms-cli 0.18.0: ms hashlock"` on the
pushed SHA, `git push origin --tags`, and watch `man-release.yml` (its repro
jobs are informational; the required contexts are the four).

- [ ] **Step 3: Verify the release**

Run: `gh release view ms-cli-v0.18.0 --repo bg002h/mnemonic-secret --json assets`
Expected: the man tarball and the musl binaries (F-324 fixed at 0.17.1).
Install and run §12's acceptance items 1–6 against the installed binary;
paste each output into `design/agent-reports/ms-hashlock-H1-acceptance.md`.

---

## Self-review (done while writing; the R0 lenses re-do it independently)

**Spec coverage.** §1 → Tasks 1, 2 (rules 1–3, the length rows by door, the
codeword distance, the share axis); §2 → Task 3; §3 → Tasks 2, 8 (the
catch-all sweep, the count test, `InspectKind` loud); §4.1 → Tasks 5, 7
(sources, `--random` gates, zero sources, the `--hex` line); §4.2 → Task 7
(`refuse_method`); §4.3 → Tasks 5, 6, 9 (`looks_like_ms1`, the reader, the
rule, both channels byte-exact); §4.4 → Tasks 7, 9 (stdout/`--out`/card/json,
lowercase hex, the `me sysw pack` spelling); §5 → Task 8; §6 → Tasks 5, 7 (six
parts, three gates); §7 → Tasks 7, 9 (every line, the boundaries); §8 → Tasks
2, 3, 4 (rows by door, both methods, the reproduction test, lockstep); §9 →
Tasks 10, 11 (MIGRATION's five items, H0 first); §10 → Tasks 10, 11 (versions,
pin, dry run, tags, manual); §11 → Tasks 5, 8, 9 (every listed test; the
`/dev/null` gates ×3; the ten pairs; the negative-content matrix; the
catch-all count; the `Zeroizing` pin — **gap:** the compile-time pin on
`Payload::Preimage`'s field type is not in any task; add to Task 2 Step 1 a
test `fn preimage_field_is_zeroizing() { let p = Payload::Preimage(Zeroizing::new([0u8;32])); if let Payload::Preimage(z) = p { let _: &Zeroizing<[u8;32]> = &z; } }` — done, it is a type-level assertion the compiler enforces); §12 → Task 11 Step 3; §13 → nothing to build; §14 → the citations below.

**Placeholder scan.** The corpus's `"…"` cells are the implementer's to fill
from the two external tools and are named as such with the `provenance`
field; no other TBD/TODO. `derive_share` in Task 2's `forge_shares` helper is
flagged to be confirmed against the vendored codex32's exported name.

**Type consistency.** `Payload::Preimage(Zeroizing<[u8; 32]>)` everywhere;
`PayloadKind::single_tag(self) -> Tag`; `hashlock::preimage_random() ->
Result<Zeroizing<[u8; 32]>>`; `CliError::Usage(String)` → 64;
`emit_preimage(&[u8; 32], bool) -> Result<u8>` used by decode and combine;
`payload_entropy_and_language` returns `Result<(…)>` and its two callers gain
`?`; `write_artifact_create_new(&Path, &str) -> Result<()>`;
`looks_like_ms1(&str) -> bool` `pub(crate)`.

## Citations (at `d4d6771`; the spec's §14 table is inherited)

| claim | site |
| --- | --- |
| the accept set and its catch-all | `crates/ms-codec/src/decode.rs:85-103` |
| `is_known_length` / `allowed_for_kind` | `crates/ms-codec/src/decode.rs:22-32` |
| `dispatch_payload` / `payload_wire_bytes` | `crates/ms-codec/src/envelope.rs:192-222`, `:231-245` |
| `Payload`, `PayloadKind`, `kind()`, `as_bytes()` | `crates/ms-codec/src/payload.rs:10-15`, `:29-31`, `:93-106` |
| `InspectKind` (not non_exhaustive) | `crates/ms-codec/src/inspect.rs:12-20` |
| `Tag`, `Tag::ENTR` | `crates/ms-codec/src/tag.rs:12-17` |
| consts | `crates/ms-codec/src/consts.rs:17,36-45,71` |
| `random_id` uses `getrandom::fill` | `crates/ms-codec/src/shares.rs:40-55` |
| the corpus pin mechanism | `crates/ms-codec/tests/vectors.rs:1-20`; `design/RELEASE_PROCESS.md` item 1 |
| `SUBCOMMANDS`, `SECRET_FLAGS`, `argv_candidates`, `is_ms1_shaped`, `override_applies`, `flag_class` | `crates/ms-cli/src/argv_guard.rs:67-79`, `:85-86`, `:104-111`, `:134-145`, `:256-269`, `:378-385` |
| `Source`, `.on()`, the admitted side channel | `crates/ms-cli/src/parse.rs:28-95` |
| `read_stdin_passphrase` (the model for stripping) | `crates/ms-cli/src/parse.rs:139-148` |
| `EncodeArgs`, `parse_hex_entropy`, `write_artifact` | `crates/ms-cli/src/cmd/encode.rs:27-73`, `:271`; `crates/ms-cli/src/out.rs:24-28` |
| `DecodeArgs` and the payload match | `crates/ms-cli/src/cmd/decode.rs:21-43`, `:60-110` |
| the four `unreachable!` arms | `payload_lang.rs:61`, `decode.rs:107`, `:112`, `combine.rs:166` |
| `inspect`'s verdict and `reason_text` | `crates/ms-cli/src/cmd/inspect.rs:160-232` |
| `Command` enum, dispatch, `is_json_mode` | `crates/ms-cli/src/main.rs:70-90`, `:216-232`, `:248-262` |
| exit codes | `crates/ms-cli/src/error.rs:49-56`; `tests/exit_codes_table.rs` |
| `--in` on six verbs (now seven) | `crates/ms-cli/tests/in_flag_six_verbs.rs` |
| CI jobs | `.github/workflows/rust.yml:109-136` |
| the release checklist | `design/RELEASE_PROCESS.md:5-22` |
| `me`'s pbkdf2 spelling | mnemonic-engrave `crates/me-cli/src/seal/crypto.rs:7,35` |
| H0 sites | seedhammer `sysw/classify.go:116-125`; mnemonic-engrave `crates/me-cli/src/seal/record.rs:151-179` |
