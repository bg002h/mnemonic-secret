#!/usr/bin/env bash
# Gate for design/FIX_bughunt1119_ms_out_writes.md (bh-ms-I13, bh-ms-I1), section 7.
#
# What it does (run from the repo root, once, to a file; then grep it):
#   0. `cargo search mnemonic-io-lib --limit 1` lists 0.1.3 (the FIX says 0.1.2; per the
#      batch overrides 0.1.2 is never published and 0.1.3 is read in its place).
#   1. In a detached worktree at the 08ea22c base with only the new test file added,
#      `bh1119_ms_out_writes` shows `27 tests run: 5 passed, 22 failed` and the 22
#      failing names are exactly the RED set of section 5.
#   2. On the head: the pin lines (Cargo.toml, Cargo.lock, vendor-git-sources.txt,
#      man-release.yml, doc and lint), `actionlint`, the same test binary 27 passed, then
#      the CI set (fmt 1.95.0, nextest workspace, doc tests, clippy, ignored mlock and
#      purge tests, the doc scripts, vendor checks, freebsd check) and red.sh with the
#      fixed cells grepped.
#
# Not covered: the toolkit manual edit (M1, M2; a separate toolkit change), the
# man-release and release workflows (CI only), the macOS, musl, nightly, miri and fuzz
# legs, the WORDING_REVIEW rows, a workflow_dispatch run of man-release.yml.
#
# Every cargo call runs with CARGO_BUILD_JOBS=6 after a host-load gate. Never --release.
# Usage: bash scripts/ms-out-writes-fix-gate.sh
set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_BUILD_JOBS=6
REPO=$(git rev-parse --show-toplevel)
cd "$REPO"
BASE=${BASE:-08ea22cd4000843974112eaf10d71eb373e50c55}
W=${W:-/scratch/cache/ms-out-writes-gate-base-wt}
BASE_TARGET=${BASE_TARGET:-/scratch/cache/ms-out-writes-gate-base-target}
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-/scratch/cache/ms-out-writes-gate-target}
OUT=${OUT:-/scratch/cache/ms-out-writes-gate-out}
case "$W$BASE_TARGET$CARGO_TARGET_DIR$OUT" in /tmp*) echo "refusing /tmp" >&2; exit 2;; esac
mkdir -p "$OUT"
fail() { echo "GATE FAIL: $*" >&2; exit 1; }
load() { until [ "$(cut -d' ' -f1 /proc/loadavg | cut -d. -f1)" -lt 30 ]; do sleep 30; done; }
cargo --version | grep -q '^cargo 1\.85\.' || fail "cargo is not the 1.85 pin: $(cargo --version)"
TEST=crates/ms-cli/tests/bh1119_ms_out_writes.rs
NEXT=(--build-jobs 6 -j 6)

echo "== step 0: io-lib 0.1.3 is on crates.io"
cargo search mnemonic-io-lib --limit 1 | grep -q '^mnemonic-io-lib = "0.1.3"' || fail "0.1.3 is not listed"

echo "== step 1: RED on the base $BASE"
if [ ! -d "$W" ]; then git worktree add --detach "$W" "$BASE" >/dev/null; fi
git -C "$W" checkout -q --detach "$BASE"
cp "$TEST" "$W/$TEST"
load
(cd "$W" && CARGO_TARGET_DIR="$BASE_TARGET" cargo nextest run --locked -p ms-cli --test bh1119_ms_out_writes --no-fail-fast "${NEXT[@]}") >"$OUT/red.log" 2>&1 || true
grep -q '27 tests run: 5 passed, 22 failed' "$OUT/red.log" || fail "base is not 5 passed / 22 failed (see $OUT/red.log)"
RED="o1_encode_failed_overwrite_keeps_previous_card o2_hashlock_failed_overwrite_keeps_previous_plate o3_repair_failed_overwrite_keeps_previous_card o4_absent_target_failed_write_leaves_nothing o5_missing_directory_text_and_exit o6_read_only_directory_refuses_and_keeps_file o8_stdio_alias_onto_a_file_names_no_state o9_create_new_failed_write_text o9b_create_new_retry_after_failed_write o9c_random_preimage_failed_write_leaves_nothing o10_encode_json_stdout_is_out_refused o11_encode_json_stdout_hard_link_refused o12_encode_json_stdout_symlink_refused o13_hashlock_stdout_is_out_refused o14_hashlock_json_stdout_is_out_refused o15_repair_stdout_is_out_refused o17_stderr_on_the_out_file_keeps_the_card o19_hashlock_random_out_with_stdout_on_it_refused o22_encode_json_stdout_appended_to_out_keeps_card o23_hashlock_random_json_stdout_appended_keeps_preimage o24_split_json_stdout_appended_keeps_shares o25_json_failed_overwrite_line_on_stderr_no_envelope"
got=$(grep -E '^\s+FAIL \[' "$OUT/red.log" | grep -o 'bh1119_ms_out_writes [a-z0-9_]*' | awk '{print $2}' | sort -u | tr '\n' ' ')
want=$(printf '%s\n' $RED | sort -u | tr '\n' ' ')
[ "$got" = "$want" ] || fail "the failing names are not the RED set: got [$got]"
echo "RED ok: 5 passed, 22 failed, the names match"

echo "== step 2: the head"
grep -qx 'mnemonic-io-lib = "0.1.3"' crates/ms-cli/Cargo.toml || fail 'ms-cli does not pin mnemonic-io-lib = "0.1.3"'
[ "$(grep -A2 -x 'name = "mnemonic-io-lib"' Cargo.lock | tr '\n' '|')" = 'name = "mnemonic-io-lib"|version = "0.1.3"|source = "registry+https://github.com/rust-lang/crates.io-index"|' ] || fail "Cargo.lock does not resolve io-lib 0.1.3 from crates.io"
! grep -q 'git+https://github.com/bg002h/mnemonic-engrave' Cargo.lock || fail "Cargo.lock still has an io-lib git source"
! grep -q '^mnemonic-io-lib|' ci/repro/vendor-git-sources.txt || fail "vendor-git-sources.txt still has the io-lib row"
! grep -vE '^[[:space:]]*#' .github/workflows/man-release.yml | grep -qE 'IO_LIB_REV|io_lib_rev|git_source_|mnemonic-engrave\?rev' || fail "man-release.yml still derives an io-lib git source"
! grep -qE 'IO_LIB_REV|mnemonic-engrave\?rev' docs/verify-reproducibility.md ci/bh1119-doc-lint.test.sh || fail "a doc or lint still derives an io-lib git source"
actionlint .github/workflows/man-release.yml
echo "pin lines ok"

load
cargo nextest run --locked -p ms-cli --test bh1119_ms_out_writes --no-fail-fast "${NEXT[@]}" 2>&1 | tee "$OUT/green.log" | tail -3
grep -q '27 tests run: 27 passed' "$OUT/green.log" || fail "the new test binary is not 27 passed"

echo "== CI set"
cargo +1.95.0 fmt --all -- --check
load
cargo nextest run --locked --workspace --all-targets --all-features --no-fail-fast "${NEXT[@]}" 2>&1 | tee "$OUT/workspace.log" | tail -4
grep -q ' 0 failed\|passed' "$OUT/workspace.log" || fail "no workspace summary"
! grep -qE '^\s+FAIL \[' "$OUT/workspace.log" || fail "workspace has failures"
load; cargo test --locked -p ms-codec --doc
load; cargo clippy --locked -p ms-codec --all-targets -- -D warnings
load; cargo clippy --locked --all-targets -p ms-cli -- -D warnings
for g in g2_1 g2_3 g2_4; do load; cargo test --locked -p ms-cli --bin ms "mlock::tests::$g" -- --include-ignored; done
for t in history_purge the_decline mlock_g6_invariant; do load; cargo test --locked -p ms-cli --test "$t" -- --include-ignored; done
bash ci/docs-only.test.sh
bash ci/bh1119-doc-lint.test.sh
bash ci/code-docs-check.sh
(cd design && sha256sum -c display-grouping-vectors.tsv.sha256)
sha256sum --check --strict ci/repro/shared-vendor.sha256
load; bash ci/repro/vendor-freshness.sh
load; bash ci/repro/vendor-freshness.test.sh
if rustup target list --installed | grep -q '^x86_64-unknown-freebsd$'; then
  load; cargo check --locked --target x86_64-unknown-freebsd -p ms-cli
else
  echo "SKIPPED: freebsd target not installed"
fi

echo "== red.sh on the head, fixed cells grepped"
MS="$CARGO_TARGET_DIR/debug/ms"
[ -x "$MS" ] || { load; cargo build --locked -p ms-cli --bin ms; }
MS="$MS" TMPDIR_BASE="$OUT" bash design/FIX_bughunt1119_ms_out_writes/red.sh >"$OUT/red-head.out" 2>&1
[ "$(grep -c 'are the same file' "$OUT/red-head.out")" = 10 ] || fail "expected 10 SAME refusals (o10-o15, o19, o22-o24)"
for n in o1 o2 o3 o25; do grep -A6 "^== ${n}_" "$OUT/red-head.out" | grep -q 'is unchanged\.' || fail "$n: no 'is unchanged.'"; done
for n in o4 o5 o9 o9c; do grep -A6 "^== ${n}_" "$OUT/red-head.out" | grep -q 'Nothing was written\.' || fail "$n: no 'Nothing was written.'"; done
grep -q '^F == previous' "$OUT/red-head.out" || fail "no 'F == previous'"
! grep -q 'F CHANGED' "$OUT/red-head.out" || fail "a file CHANGED"
echo "GATE OK"
