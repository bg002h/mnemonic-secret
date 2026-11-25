#!/usr/bin/env bash
# bh-ms-I7, I10, I11 doc-lint rows (FIX_bughunt1119_ms_docs). Exit 1 if a row fails.
# Usage: bash ci/bh1119-doc-lint.test.sh [--run-recipe]
set -u
fail=0
root="$(git rev-parse --show-toplevel)"
cd "$root"
doc=docs/verify-reproducibility.md
readme=crates/ms-cli/README.md
row_fail() { echo "FAIL $1: $2"; fail=1; }
row_ok() { echo "ok   $1"; }

# --run-recipe: run the section 8 two-distinct-path block as printed (docker, network).
if [ "${1:-}" = --run-recipe ]; then
  r=bh1119_ms_i10_section8_recipe_runs
  blk8="$(awk '/^## 8\./{s=1;next} /^## /{s=0} s' "$doc" | awk '/^```(sh|bash)$/{f=1;next} /^```/{if(f)exit} f')"
  if [ -z "$blk8" ]; then row_fail $r "no fenced block under '## 8.'"; exit 1; fi
  if bash -c "$blk8"; then row_ok $r; exit 0; else row_fail $r "section 8 block exited non-zero"; exit 1; fi
fi

# I7
ver="$(sed -n 's/^version = "\(.*\)"$/\1/p' crates/ms-cli/Cargo.toml | head -1)"
want="cargo install --locked --git https://github.com/bg002h/mnemonic-secret --tag ms-cli-v${ver} ms-cli"
r=bh1119_ms_i7_readme_install_names_release_tag
if ! grep -qxF "$want" "$readme"; then row_fail $r "README has no line '$want'"
elif grep -qxF 'cargo install ms-cli' "$readme"; then row_fail $r "README still prints 'cargo install ms-cli' (crates.io 0.14.0)"
else row_ok $r; fi

# I10 (a): bare relative ci/repro paths that do not exist here
r=bh1119_ms_i10_printed_repro_paths_exist
bad=""
for p in $(awk '/^```(sh|bash)$/{f=1;next} /^```/{f=0} f' "$doc" | grep -v '^ *#' \
           | grep -oE '^ *ci/repro/[A-Za-z0-9_.-]+\.sh' | tr -d ' ' | sort -u); do
  [ -e "$p" ] || bad="$bad $p"
done
if [ -n "$bad" ]; then row_fail $r "printed command(s) not in this checkout (exit 127 as printed):$bad"; else row_ok $r; fi

# I10 (b): TOOLKIT_REF equals both man-release.yml pins
r=bh1119_ms_i10_toolkit_ref_matches_workflow
mr=.github/workflows/man-release.yml
uses="$(grep -oE 'reproducible-musl-build\.yml@[0-9a-f]{40}' "$mr" | head -1 | cut -d@ -f2)"
tref="$(grep -oE '^ *toolkit_ref: [0-9a-f]{40}' "$mr" | head -1 | awk '{print $2}')"
dref="$(grep -oE '^TOOLKIT_REF=[0-9a-f]{40}$' "$doc" | head -1 | cut -d= -f2)"
if [ -z "$dref" ]; then row_fail $r "doc has no 'TOOLKIT_REF=<40-hex>' line"
elif [ "$dref" != "$uses" ] || [ "$dref" != "$tref" ]; then row_fail $r "doc TOOLKIT_REF $dref != man-release.yml uses@$uses / toolkit_ref $tref"
else row_ok $r; fi

# I10 (c): the stale F-324 KNOWN-RED text is gone
r=bh1119_ms_i10_no_stale_f324_known_red
if grep -q 'KNOWN-RED' "$doc"; then row_fail $r "doc still says F-324 is KNOWN-RED (fixed 2026-09-02, man-release.yml)"; else row_ok $r; fi

# I11: run the section 9.3 block with a stub cross, in a fresh env and a clone of HEAD
blk="$(awk '/^### 9\.3/{s=1;next} /^#{2,3} /{s=0} s' "$doc" | awk '/^```(sh|bash)$/{f=1;next} /^```/{f=0} f')"
# A reader fills the placeholders the old block printed; the fixed block has none.
blk="$(printf '%s\n' "$blk" | sed -e 's/<tagged-commit-SHA>/HEAD/g' -e "s/<VER>/$ver/g")"
run93() { # $1 = stub mode (bin|none), $2 = fresh dir; prints rc; leaves clone in $2/repo
  t="$2"
  git clone --quiet --no-hardlinks "$root" "$t/repo" 2>/dev/null
  mkdir "$t/bin"
  if [ "$1" = bin ]; then
    printf '#!/bin/sh\nmkdir -p target/aarch64-unknown-linux-musl/release\nprintf "synthetic\\n" > target/aarch64-unknown-linux-musl/release/ms\n' > "$t/bin/cross"
  else
    printf '#!/bin/sh\nexit 0\n' > "$t/bin/cross"
  fi
  chmod 755 "$t/bin/cross"
  ( cd "$t/repo" && env -u SOURCE_DATE_EPOCH PATH="$t/bin:$PATH" bash -c "$blk" ) > "$t/out" 2> "$t/err"
  echo $?
}
epoch="$(git show -s --format=%ct HEAD)"
cleanup() { [ -n "${t1:-}" ] && rm -r "$t1"; [ -n "${t2:-}" ] && rm -r "$t2"; return 0; }
trap cleanup EXIT
r=bh1119_ms_i11_aarch64_packaging_uses_commit_epoch
t1="$(mktemp -d)"; rc="$(run93 bin "$t1")"
arc="$t1/repo/ms-${ver}-aarch64-linux-musl.tar.gz"
if [ "$rc" != 0 ]; then row_fail $r "block exited $rc: $(head -1 "$t1/err")"
elif [ ! -f "$arc" ]; then row_fail $r "no ms-${ver}-aarch64-linux-musl.tar.gz written"
else
  want_m="$(TZ=UTC date -d "@$epoch" '+%Y-%m-%d %H:%M:%S')"
  if [ "$(TZ=UTC tar --full-time -tvzf "$arc" | awk '{print $4" "$5}')" != "$want_m" ]; then
    row_fail $r "member mtime '$(TZ=UTC tar --full-time -tvzf "$arc" | awk '{print $4" "$5}')' != commit epoch '$want_m'; stderr: $(head -1 "$t1/err")"
  else row_ok $r; fi
fi
r=bh1119_ms_i11_aarch64_packaging_missing_binary_fails
t2="$(mktemp -d)"; rc="$(run93 none "$t2")"
if [ "$rc" = 0 ]; then row_fail $r "block exited 0 with no binary to package (archive: $(ls "$t2/repo" | grep -c 'aarch64-linux-musl.tar.gz$'))"
elif [ -e "$t2/repo/ms-${ver}-aarch64-linux-musl.tar.gz" ]; then row_fail $r "block failed (rc $rc) but left ms-${ver}-aarch64-linux-musl.tar.gz"
else row_ok $r; fi
exit $fail
