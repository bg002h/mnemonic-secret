#!/usr/bin/env bash
# vendor/ freshness guard — the LEADING (PR-time) gate. CODEC (fork-free) form.
#
# REDs iff the committed `vendor/` tree cannot satisfy the current `Cargo.lock`
# under the reproducible build's `--offline --locked` source-replacement config.
# This is the v0.74.0 failure class that hit the toolkit: a dep bump that updates
# Cargo.lock but forgets `cargo vendor vendor/`, so the release `--offline`
# reproducible build can't resolve the bumped dep and publishes NO musl binary.
# That gate is LAGGING (fires only at the release tag); this makes the same
# failure surface on the PR.
#
# Cheap by design: `cargo metadata` does FULL-workspace, all-target resolution
# with NO compile / NO musl toolchain / NO Docker. With vendored-sources
# replacement active, resolution validates EVERY Cargo.lock entry against vendor/
# regardless of target cfg (proven in the toolkit R0 — no musl-only false
# negative). Ported verbatim from mnemonic-toolkit:ci/repro/vendor-freshness.sh.
#
# THREE-BLOCK FORM AS OF P2 (2026-08-27). This crate WAS fork-free and used the
# two-block form (crates-io + vendored-sources). P2 pins `mnemonic-io-lib` — the
# shared IO mechanism six m-format binaries adopt — by GitHub rev, which puts the
# FIRST `source = "git+…"` line into Cargo.lock. The two-block config cannot
# redirect that source key, so `--offline` reaches the live host and this gate
# REDs. That is exactly what the fail-closed check below reported when the pin
# landed, and the fix is the one it names: a per-source git-fork stanza.
#
# The rev is DERIVED from Cargo.lock rather than written here, mirroring the
# toolkit's MINISCRIPT_REV handling, so moving the pin forward does not silently
# leave this script pointing at the old rev. Fail CLOSED on an empty match.
#
# `path =` is not the alternative: `freebsd-compile-gate` and both `musl-check`
# targets build from a clean checkout on foreign targets and a path dep out of
# the workspace fails there first.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

# Derive the mnemonic-engrave rev from Cargo.lock (authoritative, comment-free)
# so the 3-block source config auto-tracks the pin in crates/ms-cli/Cargo.toml.
IO_LIB_REV="$(grep -oE 'mnemonic-engrave\?rev=[0-9a-f]{40}' Cargo.lock | head -1 | grep -oE '[0-9a-f]{40}' || true)"
if [ -z "$IO_LIB_REV" ]; then
  echo "::error::vendor-freshness: could not derive the mnemonic-io-lib rev from Cargo.lock" \
       "(expected a 'mnemonic-engrave?rev=<40-hex>' source line). Failing closed." >&2
  exit 1
fi

# Fail CLOSED if any OTHER git source appears in Cargo.lock: the config below
# redirects exactly one, so a second would not be served and `--offline` would
# silently reach the live host (or mis-resolve) instead of REDing.
UNKNOWN_GIT="$(grep -E '^source = "git\+' Cargo.lock | grep -vc 'mnemonic-engrave' || true)"
if [ "${UNKNOWN_GIT:-0}" -ne 0 ]; then
  echo "::error::vendor-freshness: Cargo.lock has a git source this config does not" \
       "redirect. Add a per-source git-fork [source] stanza for it (see the block below)." >&2
  grep -E '^source = "git\+' Cargo.lock | grep -v 'mnemonic-engrave' >&2
  exit 1
fi

# Three-block source-replacement: crates-io + the mnemonic-engrave git source +
# vendored-sources -> the committed vendor/ tree. Emitted verbatim by
# `cargo vendor vendor/`, which is also what produced vendor/mnemonic-io-lib/.
IO_LIB_SRC="git+https://github.com/bg002h/mnemonic-engrave?rev=${IO_LIB_REV}"
SRC_CONFIG=(
  --config 'source.crates-io.replace-with="vendored-sources"'
  --config "source.\"${IO_LIB_SRC}\".git=\"https://github.com/bg002h/mnemonic-engrave\""
  --config "source.\"${IO_LIB_SRC}\".rev=\"${IO_LIB_REV}\""
  --config "source.\"${IO_LIB_SRC}\".replace-with=\"vendored-sources\""
  --config 'source.vendored-sources.directory="vendor"'
)

echo "vendor-freshness: resolving Cargo.lock against committed vendor/ (offline, locked; mnemonic-io-lib rev ${IO_LIB_REV}) ..."
if cargo metadata --format-version 1 --locked --offline "${SRC_CONFIG[@]}" >/dev/null; then
  echo "vendor-freshness: OK — vendor/ satisfies Cargo.lock."
else
  echo "::error::vendor/ is out of sync with Cargo.lock — the --offline --locked reproducible build" \
       "cannot resolve a dependency from the committed vendor/ tree. Run 'cargo vendor vendor/' and" \
       "commit the result (see docs/verify-reproducibility.md). This is the toolkit v0.74.0 release-CI" \
       "failure class, now caught at PR time." >&2
  exit 1
fi
