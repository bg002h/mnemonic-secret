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
# fmt is its own statement so a diff STOPS the gate (an `a | b && c` list would
# let set -e walk past it); the diff's head is printed for the fold.
if ! cargo fmt --check > "${WORK}/fmt.diff" 2>&1; then
  echo "rustfmt diff ($(grep -c '^Diff in' "${WORK}/fmt.diff") hunks):"; head -40 "${WORK}/fmt.diff"; exit 5
fi
echo "fmt clean"

echo "== 5 -- codeword distance (spec §1) =="
cargo nextest run -p ms-codec --locked --no-capture -E 'test(codeword_distance)' 2>&1 | grep -E "codeword distance|PASS|FAIL" | head -3

echo "== 6 -- downgrade row against the pre-H1 tree ($PRE_H1) =="
PRE="${TMPDIR:-/tmp}/plan-build-gate-ms-pre"
rm -rf "$PRE"; git -C "$SRC" worktree add -q --detach "$PRE" "$PRE_H1"
( cd "$PRE" && CARGO_TARGET_DIR="${CARGO_TARGET_DIR}-pre" cargo build -p ms-cli --locked -q )
# The string comes from the WIRED binary, not from a corpus cell an implementer
# may not have filled yet: the gate's job is to prove the old reader refuses
# what the new writer emits.
S=$(printf 'ab%.0s' $(seq 32) | "${CARGO_TARGET_DIR}/debug/ms" hashlock --hex - --json --no-engraving-card 2>/dev/null | python3 -c "import json,sys;print(json.load(sys.stdin)['preimage_ms1'])")
[ -n "$S" ] || { echo "could not obtain a preimage plate string from the wired ms" >&2; exit 4; }
echo "plate: $S"
set +e
OUT=$(printf '%s\n' "$S" | "${CARGO_TARGET_DIR}-pre/debug/ms" decode - 2>&1); RC=$?
set -e
git -C "$SRC" worktree remove --force "$PRE"
echo "$OUT" | head -2; echo "exit=$RC"
[ "$RC" -eq 2 ] && echo "$OUT" | grep -q "reserved-prefix byte was 0x03" || { echo "DOWNGRADE ROW FAILED: expected exit 2 and the reserved-prefix text" >&2; exit 4; }
echo "== NOT covered: fragments beyond the hand-wire script; the toolkit manual chapter; .github/workflows edits; H0 (fork + me). =="
