#!/usr/bin/env bash
# Gate for design/FIX_ms_output_sink.md (ms output sink and E4, crash sweep F1 family).
#
# What it does:
#   1. Extracts the document's blocks (section 3 plain `diff` blocks, section 4 `diff`
#      rows, the `sh` RED commands and the `text` guard command) and checks that the
#      code blocks and the rows hash to the sha256 the document states. If the architect's
#      files exist, it also cmp's them byte for byte.
#   2. In its own detached worktree at the base commit: applies the rows, runs every
#      RED command as printed and requires each to FAIL with its `## expect:`
#      substring; the guard row must PASS on the base; whole set "11 tests run: 1 passed, 10 failed".
#   3. Applies the code blocks (FOLLOWUPS included); requires the eleven rows green, the
#      full workspace suite "679 tests run: 679 passed, 23 skipped", doc tests, fmt
#      (+1.95.0, the CI pin), clippy.
#   4. Self-test: the checker must reject a wrong expectation and a run that passed,
#      or the gate dies.
#
# What it does NOT cover: the grade's 18 plants and the architect's two (not re-run); the macOS, musl, release,
# miri, FreeBSD, history-purge, g6 and fuzz CI legs; the toolkit manual section; the
# coordinator's mutation-gate.sh; any review; the `<COMMIT>` substitution in FOLLOWUPS.
#
# Usage: scripts/ms-output-sink-fix-gate.sh [DOC]   (run from the repo root)
set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
DOC=${1:-design/FIX_ms_output_sink.md}
REPO=$(git rev-parse --show-toplevel)
DOC=$(realpath "$DOC")
BASE=${BASE:-548da55a4b709442be3391ec8d44a1b74f1aa61b}
W=${W:-/scratch/cache/ms-output-sink-gate-wt}
OUT=${OUT:-/scratch/cache/ms-output-sink-gate-out}
GRADE=${GRADE:-/scratch/cache/architect-output-sink-ro/ms}
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-/scratch/cache/ms-output-sink-gate-target}
case "$W$OUT$CARGO_TARGET_DIR" in /tmp*) echo "refusing /tmp" >&2; exit 2;; esac
mkdir -p "$OUT"
die() { echo "GATE FAIL: $*" >&2; exit 1; }
cargo --version | grep -q '^cargo 1\.85\.' || die "cargo is not the 1.85 pin: $(cargo --version)"

# 1. extract
python3 -I - "$DOC" "$OUT" <<'PY'
import sys,re,hashlib
doc,out=sys.argv[1:3]
t=open(doc).read()
def section(a,b):
    i=t.index(a); j=t.index(b,i); return t[i:j]
def blocks(s,tag):
    return re.findall(r'(?m)^```'+tag+r'\n(.*?)^```\n',s,re.S)
s3=section('\n## 3. ','\n## 4. ')
s4=section('\n## 4. ','\n## 5. ')
code=''.join(blocks(s3,'diff'))
rows=''.join(blocks(s4,'diff'))
red=''.join(blocks(s4,'sh'))
guard=''.join(blocks(s4,'text'))
want=re.search(r'sha256 `([0-9a-f]{64})`, checked',s3).group(1)
wantr=re.search(r'sha256 `([0-9a-f]{64})`\)',s4).group(1)
assert hashlib.sha256(code.encode()).hexdigest()==want,'code blocks differ from the stated sha256'
assert hashlib.sha256(rows.encode()).hexdigest()==wantr,'rows blocks differ from the stated sha256'
assert len(blocks(s3,'diff'))==8,'expected 8 code blocks'
assert len(blocks(s4,'diff'))==2,'expected 2 rows blocks'
assert len([l for l in red.splitlines() if l.startswith('cargo nextest')])==10,'expected 10 RED commands'
assert len([l for l in guard.splitlines() if l.startswith('cargo nextest')])==1,'expected 1 guard command'
for n,v in (('code.diff',code),('rows.diff',rows),('red.sh',red),('guard.sh',guard)):
    open(out+'/'+n,'w').write(v)
PY
if [ -f "$GRADE/fix.diff" ]; then
  cmp "$OUT/code.diff" "$GRADE/fix.diff" || die "code blocks differ from the architect's fix.diff"
  cmp "$OUT/rows.diff" "$GRADE/rows.diff" || die "rows differ from the architect's rows.diff"
  echo "cmp: blocks identical to the architect's files"
else echo "cmp: architect files absent, sha256 only"; fi

# worktree at the base
[ -e "$W/.git" ] || git -C "$REPO" worktree add --detach "$W" "$BASE" >/dev/null
reset() {
  git -C "$W" reset -q --hard "$BASE"
  rm -f "$W/crates/ms-cli/tests/output_errors.rs" "$W/crates/ms-cli/tests/future_format_names_the_problem.rs" "$W/crates/ms-cli/src/sink.rs"
}
reset
git -C "$W" rev-parse HEAD | grep -q "^$BASE" || die "worktree is not at the base"

# checker: 0 only if the run FAILED and its output holds the substring
red_check() { # $1 log, $2 rc, $3 expect
  [ "$2" -ne 0 ] && grep -qF -- "$3" "$1"
}
# 2. RED on the base
git -C "$W" apply "$OUT/rows.diff"
n=0; first_log=; first_rc=
while IFS= read -r line; do
  case "$line" in cargo\ nextest*) ;; *) continue;; esac
  cmd=${line%% ## expect: *}; exp=${line#* ## expect: }
  n=$((n+1)); log="$OUT/red-$n.log"
  rc=0; (cd "$W" && eval "$cmd") >"$log" 2>&1 || rc=$?
  red_check "$log" "$rc" "$exp" || die "RED row $n did not fail for its stated reason: $cmd (rc=$rc, expect: $exp)"
  echo "RED $n ok (rc=$rc): $cmd"
  [ -n "$first_log" ] || { first_log=$log; first_rc=$rc; }
done < "$OUT/red.sh"
[ "$n" -eq 10 ] || die "expected 10 RED commands, ran $n"
# the guard passes on the base
gcmd=$(grep '^cargo nextest' "$OUT/guard.sh")
rc=0; (cd "$W" && eval "$gcmd") >"$OUT/guard.log" 2>&1 || rc=$?
[ "$rc" -eq 0 ] && grep -q '1 test run: 1 passed' "$OUT/guard.log" || die "the guard row did not pass on the base (rc=$rc)"
echo "guard ok on the base: verify_says_ok_on_stdout_and_nothing_on_stderr"
# whole-set RED
rc=0; (cd "$W" && cargo nextest run --locked -j 8 --no-fail-fast -E 'binary(output_errors) | binary(future_format_names_the_problem)') >"$OUT/red-all.log" 2>&1 || rc=$?
grep -q '11 tests run: 1 passed, 10 failed' "$OUT/red-all.log" || die "whole-set RED is not 1 passed, 10 failed"
echo "RED whole set: $(grep -o '11 tests run: 1 passed, 10 failed' "$OUT/red-all.log")"
# self-test of the checker
red_check "$first_log" "$first_rc" "NO-SUCH-TEXT-XYZ" && die "self-test: a wrong expectation was accepted"
red_check "$OUT/guard.log" 0 "1 test run" && die "self-test: a passing run was accepted as RED"
echo "self-test ok: the checker rejects a broken row"

# 3. GREEN
git -C "$W" apply "$OUT/code.diff"
grep -q 'ms-output-sink' "$W/design/FOLLOWUPS.md" || die "FOLLOWUPS entry missing"
grep -q 'ms-future-format-silent-exit-3' "$W/design/FOLLOWUPS.md" || die "E4 FOLLOWUPS entry missing"
run() { # name cmd...
  local name=$1; shift
  (cd "$W" && "$@") >"$OUT/$name.log" 2>&1 || { tail -n 30 "$OUT/$name.log" >&2; die "$name failed"; }
}
run rows cargo nextest run --locked -j 8 -E 'binary(output_errors) | binary(future_format_names_the_problem)'
grep -q '11 tests run: 11 passed' "$OUT/rows.log" || die "rows are not 11/11"
run suite cargo nextest run --locked -j 8
grep -q '679 tests run: 679 passed, 23 skipped' "$OUT/suite.log" || die "suite is not 679/679: $(grep 'tests run' "$OUT/suite.log")"
run doc cargo test --locked --workspace --doc
run fmt cargo +1.95.0 fmt --all -- --check
run clippy cargo clippy --locked --workspace --all-targets -- -D warnings
echo "GREEN rows: $(grep -o '11 tests run: 11 passed' "$OUT/rows.log")"
echo "GREEN suite: $(grep -o '679 tests run: 679 passed, 23 skipped' "$OUT/suite.log")"
echo "GREEN doc, fmt, clippy: ok"
reset
echo "GATE PASS"
