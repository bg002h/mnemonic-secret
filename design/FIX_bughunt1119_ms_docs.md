# FIX: ms printed docs: stale install line, wrong repair exit and guarantee, reproducibility recipes that fail (bh-ms-I7, I8, I10, I11)

Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-22. FIX table row #49 of
`coordinator/agent-reports/architect-bughunt-1119-questions.md` (L161). **Small track, grade I, docs plus doc
tests plus two CI steps** (`.github/` edits are in this PR; the PR description must say so). Ruled by architect
**T25** (I7: README names `cargo install --git ... --tag ms-cli-v<ver>` plus a doc-lint row; publishing waits for
mnemonic-io-lib 0.1.2), **T27** (I10: rewrite the recipe from the pinned toolkit workflow and run it in CI; the
doc-lint row runs it, not just resolves paths), **T28** (I8: exit 4 stays; the README is the defect). Base:
`origin/master` `0f24ee79` (ms-cli 0.21.0; tag `ms-cli-v0.21.0` exists on origin). Sources: consultant
`design/agent-reports/bug-hunt-2026-11-19.md` L151-209; `coordinator/agent-reports/bughunt-1119/triage-ms-t2.md`
(I7, I8), `triage-ms-t3.md` (I10, I11), `grade-ms.md` L38-42. No product ruling (B1-B10) bears on these rows.

Every statement of current behaviour was measured on `0f24ee79` (cargo 1.85.0, debug build). The fixed behaviour
was measured on a scratch worktree with sections 2 and 3 applied (not committed), except the container half of
the D8 recipe, which this host cannot run (section 4).

## 1. Summary

- **I7.** `crates/ms-cli/README.md:10` prints `cargo install ms-cli`. crates.io holds ms-cli 0.14.0 (max_version,
  2026-11-22), so the printed line installs a tool missing every fix since; the head manifest cannot be published
  (`crates/ms-cli/Cargo.toml:31` git-only `mnemonic-io-lib`).
- **I8.** `crates/ms-cli/README.md:63` documents exit `5` `REPAIR_APPLIED`; the README's own damaged card exits
  **4** (`verdict:"candidate"`), and its stderr example (`:49-50`) shows one of the two lines the command prints and
  a truncated form of that one. `crates/ms-codec/README.md:70-73` says beyond t=4 "the call returns
  `Error::TooManyErrors`"; a card one substitution from another valid card corrects to that other card.
- **I10.** `docs/verify-reproducibility.md:284-286` tells the reader to run `ci/repro/{double-build,cc-validate,
  gzip-residue}.sh`, which are not in this repo (exit 127 as printed); the toolkit copies default to
  `CRATE=mnemonic-toolkit` / `BIN=mnemonic`, so finding them still builds the wrong crate. `:265-278` and
  `:289-291` still call F-324 KNOWN-RED, fixed 2026-09-02 (`man-release.yml:105-123`).
- **I11.** `docs/verify-reproducibility.md:335-350` (section 9.3) sets `SOURCE_DATE_EPOCH` only for `cross`; `tar`
  then expands an unset variable and packages a wrong-mtime archive at exit 0. Without `pipefail`, a missing
  binary (tar exit 2, gzip 0) also exits 0 with an archive holding no binary.

## 2. Spec delta (holder-visible texts)

Texts marked PROPOSED are new holder-visible text (architect confirms). Texts marked COPIED repeat an existing
reviewed text byte for byte.

### 2.1 I7: `crates/ms-cli/README.md` Installation (L7-14)

**D1.** Replace the fenced line `cargo install ms-cli` (L10) with (T25):

    cargo install --locked --git https://github.com/bg002h/mnemonic-secret --tag ms-cli-v0.21.0 ms-cli

**D2.** Between the closing fence (L11) and `The installed binary is named` (L13), add one paragraph (PROPOSED):

    This builds the tagged release from source. The crates.io copy (`cargo install ms-cli`) is ms-cli 0.14.0, which lacks every fix since; do not use it.

**D3. Version rule.** The tag in D1 is `ms-cli-v<V>` where `<V>` is the `version` of `crates/ms-cli/Cargo.toml`.
A release bump changes both in one commit (lint row 1 of section 3 fails otherwise). `--locked` builds
against the tag's committed `Cargo.lock` (the consultant asks for the tested `--locked` route); the toolkit's CI
uses the same `--git ... --tag ms-cli-v0.21.0 ms-cli` form without it. Other `cargo install ms-cli` mentions stay:
`design/SPEC_ms_cli_v0_1.md:770`, `design/IMPLEMENTATION_PLAN_ms_cli_v0_1.md:3456` and
`design/SPEC_wave2_secret_hygiene_ms.md:147` are design records, not install instructions.

### 2.2 I8: `crates/ms-cli/README.md` `ms repair` section (L39-67)

**D4.** In the example block, replace the two stderr lines (L49-50) with these four lines (the two stderr lines
are COPIED from the measured output of `ms repair --in broken.txt` at base; the exit line is PROPOSED):

    # stderr:
    #   warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
    #   repair: correction UNVERIFIED — a corrected seed card cannot be self-verified; confirm the derived address/xpub against a known-good copy before use; BIP-93 recommends confirming a corrected codex32 string
    # exit status: 4

**D5.** In the exit table, replace the `5` row (L63) with the `4` row, and the `2` row (L64) with the `2` row,
both COPIED from the toolkit manual `docs/manual/src/40-cli-reference/43-ms.md` L789-790 (toolkit `origin/master`
`b57187b3`):

    | `4` | **`VERIFY-ME` Candidate** (Cycle F, `ms1-repair-demote-to-candidate` — a funds fix) — at least one substitution corrected; stdout = repair report + corrected string; stderr carries an UNVERIFIED advisory. **Never exit 5** — see below. |
    | `2` | unrepairable (BCH-uncorrectable: > 4 substitution errors, or structural ms1 error before correction could run). |

Row order stays `0`, `4`, `2`, `1` (the `4` row takes the `5` row's place).

**D6.** After the table (before `` `ms repair` wraps ``, L67), add one paragraph (PROPOSED; it carries the
consultant's kind-appropriate confirmation guidance and is the "see below" of D5):

    **Confirm every exit-4 card before use.** Up to 4 substitutions are corrected back to the original string; beyond that a correction can still succeed and land on a different, valid card, and an ms1 string carries nothing else to catch it. Confirm the corrected card against an independent record: for a seed card, the derived address or xpub against a known-good copy; for a hashlock preimage card (`ms10hash…`), its hash against the hashlock it was made for. `ms repair` never exits 5.

### 2.3 I8: `crates/ms-codec/README.md` L70-73

**D7.** Replace the paragraph `When the input exceeds the `t=4` correction capacity, ... does not localize > 4
errors).` with (PROPOSED):

    Up to 4 substitutions are corrected back to the original string. Beyond
    `t=4` the call either returns `Error::TooManyErrors { bound: 8 }` or
    corrects to a *different* valid string, the nearest codeword: the
    `BCH(93,80,8)` code cannot tell which original was meant. Treat every
    corrected result (a non-empty `CorrectionDetail` vector) as an unverified
    candidate and confirm it independently before use, as `ms repair` does
    (exit 4).

### 2.4 I10: `docs/verify-reproducibility.md` section 8 (L265-291)

**D8.** Replace L265-291 (from `**F-324 — that gate is KNOWN-RED` through the paragraph ending `that knob cannot
express one.)`) with the following, a paragraph, an intro, one fenced `bash` block, and a closing paragraph
(PROPOSED; the block is the toolkit workflow `reproducible-musl-build.yml@4120af85` job `repro-x86_64-musl`
L473-590, steps "Materialize source" through "binary runs", with the `with:` values of `man-release.yml:147-164`):

    **F-324 — fixed 2026-09-02.** Since P2, `ms` pins `mnemonic-io-lib` by git
    rev, so the gate needs a git-source `[source]` stanza beside the crates-io one.
    The `repro` caller job passes the reusable workflow's `git_source_url` /
    `git_source_rev` inputs (the rev derived from `Cargo.lock` by the `pins` job) and
    an empty `miniscript_rev`, at toolkit revision
    `4120af853ec68dc515cb4ac0eb7245a018a50c78` (the `uses:` pin and `toolkit_ref`,
    which must match). Before the toolkit gained those inputs the gate could not
    resolve `mnemonic-io-lib` offline and failed at the `ms-cli-v0.17.0` tag (run
    33621228397).

    To reproduce the two-distinct-path proof yourself, run this block in bash from
    the root of a clean checkout of the commit you are verifying (§2), with `git`
    and `docker` installed. It is the toolkit workflow's `repro-x86_64-musl` job,
    step for step: it checks out the toolkit at the pinned revision, builds its
    `Dockerfile.repro`, copies the committed source to two distinct paths and runs
    the toolkit's gate scripts with the inputs this repo's `repro` job passes. CI
    runs this block as printed (`man-release.yml` job `repro-doc-recipe`).

    ```bash
    set -euo pipefail
    umask 022
    # The toolkit revision man-release.yml's `repro` job pins (`uses:` and `toolkit_ref`):
    TOOLKIT_REF=4120af853ec68dc515cb4ac0eb7245a018a50c78
    IO_LIB_REV="$(grep -oE 'mnemonic-engrave\?rev=[0-9a-f]{40}' Cargo.lock | head -1 | grep -oE '[0-9a-f]{40}')"
    EPOCH="$(git show -s --format=%ct HEAD)"
    WORK="$(mktemp -d)"
    git clone --quiet https://github.com/bg002h/mnemonic-toolkit "$WORK/toolkit"
    git -C "$WORK/toolkit" checkout --quiet --detach "$TOOLKIT_REF"
    mkdir "$WORK/src"
    git archive HEAD | tar -x -C "$WORK/src"
    docker build --quiet -t ms-repro-gate -f "$WORK/toolkit/Dockerfile.repro" "$WORK/toolkit"
    docker run --rm --network=none \
      -v "$WORK/src":/src:ro -v "$WORK/toolkit":/toolkit:ro \
      -e CRATE=ms-cli -e BIN=ms -e TARGET=x86_64-unknown-linux-musl -e ARCH=x86_64 \
      -e MINISCRIPT_REV= \
      -e GIT_SOURCE_URL=https://github.com/bg002h/mnemonic-engrave -e GIT_SOURCE_REV="$IO_LIB_REV" \
      -e SOURCE_DATE_EPOCH="$EPOCH" -e REPRO_OUT_DIR=/build/canonical \
      -e CARGO_HOME=/cargo -e LC_ALL=C -e TZ=UTC \
      ms-repro-gate bash -euo pipefail -c '
        mkdir -p /build-a/src /build-b/src
        cp -a /src/. /build-a/src
        cp -a /src/. /build-b/src
        bash /toolkit/ci/repro/double-build.sh /build-a/src /build-b/src
        bash /toolkit/ci/repro/cc-validate.sh /build-a/src
        bash /toolkit/ci/repro/remap-off-negative.sh /build-a/src /build-b/src
        bash /toolkit/ci/repro/gzip-residue.sh /build/canonical/ms-0.0.0-repro-x86_64-linux-musl.tar.gz 03
        /build/canonical/ms --version'
    ```

    The gate scripts default to the toolkit's own crate (`CRATE=mnemonic-toolkit`,
    `BIN=mnemonic`) and its miniscript fork; the `-e` lines select `ms-cli` / `ms`,
    clear `MINISCRIPT_REV` and add the `mnemonic-io-lib` source, as the `repro`
    job's `with:` block does. Each script exits non-zero on a failed check, and
    `set -e` stops the block there. Building the image from `Dockerfile.repro` is
    enough for this proof, which compares two builds made in one image; to match a
    published hash, use the pulled image of §3 and the command of §4.

(The fenced block above is shown indented here; in the doc the fence lines and the block start at column 0 with
the block's own indentation as shown.) Paragraphs L251-263 (two-distinct-path shape, `workflow_dispatch`) are
unchanged.

Design notes (not doc text): the block copies `git archive HEAD` rather than the working tree so a local
`target/` or untracked file never enters the build; `--network=none` holds the doc's "fully offline" claim;
`remap-off-negative.sh` and `ms --version` are in because the workflow job runs them (the old block omitted them);
the canonical tarball name `ms-0.0.0-repro-x86_64-linux-musl.tar.gz` is `double-build.sh`'s `VER` default
`0.0.0-repro` (toolkit `ci/repro/double-build.sh:69`), as the workflow's gzip-residue step uses.

### 2.5 I11: `docs/verify-reproducibility.md` section 9.3 (L332-351)

**D9.** Replace L332-351 (`Run from the repo root ...` through the block's closing fence) with an intro and one
fenced `bash` block (PROPOSED; the block is `man-release.yml:258-290`, the `musl-binaries` cross step, with the
epoch set once and the final name given only on success, as the consultant's fix asks). L353-356 stay.

    Run this block in bash from the root of a checkout of the tagged commit (§2),
    with `cross` installed (`cargo install --locked cross`). It is the aarch64
    step of `man-release.yml` `musl-binaries`: the epoch is set once, from the
    commit, and reaches both the build and `tar`; the archive takes its final name
    only after `tar` and `gzip` both succeed, and the last line prints its SHA-256
    for comparison with the published `SHA256SUMS.aarch64`.

    ```bash
    set -euo pipefail
    umask 022
    EPOCH="$(git show -s --format=%ct HEAD)"
    VER="$(sed -n 's/^version = "\(.*\)"$/\1/p' crates/ms-cli/Cargo.toml | head -1)"
    IO_LIB_REV="$(grep -oE 'mnemonic-engrave\?rev=[0-9a-f]{40}' Cargo.lock | head -1 | grep -oE '[0-9a-f]{40}')"
    IO_LIB_SRC="git+https://github.com/bg002h/mnemonic-engrave?rev=${IO_LIB_REV}"
    REMAP="--remap-path-prefix=/project=/build --remap-path-prefix=/cargo=/cargo"
    CFMAP="-ffile-prefix-map=/project=/build -ffile-prefix-map=/cargo=/cargo"
    env \
      SOURCE_DATE_EPOCH="$EPOCH" \
      LC_ALL=C TZ=UTC \
      CARGO_BUILD_RUSTFLAGS="$REMAP" \
      CFLAGS="$CFMAP" \
      CFLAGS_aarch64_unknown_linux_musl="$CFMAP" \
      cross build --locked --offline --release \
        --target aarch64-unknown-linux-musl -p ms-cli --bin ms \
        --config 'source.crates-io.replace-with="vendored-sources"' \
        --config "source.\"${IO_LIB_SRC}\".git=\"https://github.com/bg002h/mnemonic-engrave\"" \
        --config "source.\"${IO_LIB_SRC}\".rev=\"${IO_LIB_REV}\"" \
        --config "source.\"${IO_LIB_SRC}\".replace-with=\"vendored-sources\"" \
        --config 'source.vendored-sources.directory="vendor"'
    tar --sort=name --owner=0 --group=0 --numeric-owner --mtime="@$EPOCH" \
      -cf - -C target/aarch64-unknown-linux-musl/release ms \
      | gzip -n -9 > "ms-${VER}-aarch64-linux-musl.tar.gz.partial"
    mv "ms-${VER}-aarch64-linux-musl.tar.gz.partial" "ms-${VER}-aarch64-linux-musl.tar.gz"
    sha256sum "ms-${VER}-aarch64-linux-musl.tar.gz"
    ```

`VER` comes from `crates/ms-cli/Cargo.toml` (the workflow takes it from the tag name; at a release tag they are
equal). On failure the block stops with a non-zero exit and may leave the `.partial` file; the final name is never
written.

### 2.6 CHANGELOG (`CHANGELOG.md`)

**D10.** Under `## ms-cli [Unreleased]` `### Fixed`, append (PROPOSED):

    - **Printed docs that installed a stale `ms` or failed as printed** (bh-ms-I7, I8, I10, I11). The README
      install line now names the release tag (`cargo install --locked --git ... --tag ms-cli-v<ver> ms-cli`; the
      crates.io copy is 0.14.0). The `ms repair` README gives exit 4 (VERIFY-ME candidate), not 5, and its full
      stderr. `docs/verify-reproducibility.md` section 8 is now a runnable two-path recipe from the pinned toolkit
      workflow (CI runs it) and section 9.3 sets the epoch once and fails on any packaging error.

**D11.** Under `## ms-codec [Unreleased]` `### Fixed`, append (PROPOSED):

    - **The README no longer promises `TooManyErrors` beyond t=4** (bh-ms-I8): a correction there can land on a
      different valid string, so every corrected result is an unverified candidate.

## 3. Test and CI delta (no production code changes)

**C1. New `ci/bh1119-doc-lint.test.sh`** (bash, run from anywhere in the repo; `cd` to the toplevel). One row per
check; each prints `ok   <row>` or `FAIL <row>: <detail>`; exit 1 if any row fails, else 0. Rows, in this order:

1. `bh1119_ms_i7_readme_install_names_release_tag`: `<V>` = the `^version = "..."` value of
   `crates/ms-cli/Cargo.toml`. FAIL unless `crates/ms-cli/README.md` has a line exactly equal to D1 with `<V>`, and
   FAIL if it has a line exactly `cargo install ms-cli`.
2. `bh1119_ms_i10_printed_repro_paths_exist`: every line inside a ```` ```sh ```` or ```` ```bash ```` fence of
   `docs/verify-reproducibility.md`, not a `#` comment, that starts (after spaces) with a relative
   `ci/repro/<name>.sh` path: FAIL listing each such path that does not exist in the checkout.
3. `bh1119_ms_i10_toolkit_ref_matches_workflow`: FAIL unless the doc has a line `TOOLKIT_REF=<40 hex>` equal to
   both the `reproducible-musl-build.yml@<sha>` pin and the `toolkit_ref:` value of
   `.github/workflows/man-release.yml`.
4. `bh1119_ms_i10_no_stale_f324_known_red`: FAIL if the doc contains `KNOWN-RED`.
5. `bh1119_ms_i11_aarch64_packaging_uses_commit_epoch`: take the fenced block(s) under `### 9.3` (up to the next
   `##`/`###` heading); substitute `<tagged-commit-SHA>` -> `HEAD`, `<VER>` -> `<V>`, `<IO_LIB_REV>` -> the
   `mnemonic-engrave?rev=` value of `Cargo.lock` (what a reader fills in; the D9 block has no placeholders).
   In a fresh `git clone --quiet --no-hardlinks <toplevel>` under `mktemp -d`, run it with `bash -c` under
   `env -u SOURCE_DATE_EPOCH`, with a stub `cross` first in `PATH` that writes `synthetic\n` to
   `target/aarch64-unknown-linux-musl/release/ms`. FAIL unless it exits 0, writes
   `ms-<V>-aarch64-linux-musl.tar.gz`, and the archive member's mtime (`TZ=UTC tar --full-time -tvzf`) equals
   `git show -s --format=%ct HEAD` rendered `%Y-%m-%d %H:%M:%S` in UTC.
6. `bh1119_ms_i11_aarch64_packaging_missing_binary_fails`: as row 5 with a stub `cross` that writes nothing. FAIL
   if the block exits 0, or if it fails but `ms-<V>-aarch64-linux-musl.tar.gz` exists.

With `--run-recipe` as its only argument, the script runs one row instead and exits with its result:
`bh1119_ms_i10_section8_recipe_runs`: extract the first ```` ```bash ```` (or `sh`) fenced block under `## 8.`
(up to the next `## ` heading) and run it with `bash -c` from the toplevel; `ok` on exit 0, else
`FAIL bh1119_ms_i10_section8_recipe_runs: section 8 block exited non-zero`. A reference implementation that
measured every RED and GREEN result below is Appendix A.

**C2. `ci/code-docs-exempt.txt`**: append (the lint reads two docs files by name; it runs before the clippy job's
docs-only guard and in man-release, so the docs-only skip never skips it):

    # Runs in the rust.yml clippy job before its docs-only guard step (and, with
    # --run-recipe, in man-release.yml), so no step the skip can skip runs it. It
    # reads the docs it lints.
    ^ci/bh1119-doc-lint\.test\.sh$

**C3. `.github/workflows/rust.yml` `clippy` job**: a new step directly after the step named `Shared CI files, and
no code names a docs file` (L384-388) and before `Docs-only change?`, so a docs-only change still runs it:

    - name: Printed commands in the docs run as printed (bh1119 doc lint)
      shell: bash
      run: bash ci/bh1119-doc-lint.test.sh

**C4. `.github/workflows/man-release.yml`**: a new job (T27 "run it in CI"; placement is the author's engineering choice, see
section 10) after the `repro` job, on the workflow's existing triggers (`ms-cli-v*` tag push,
`workflow_dispatch`). Its `name:` must not contain the doc's file name (code-docs-check scans `.yml`):

    repro-doc-recipe:
      name: reproducibility doc, section 8 recipe run as printed
      runs-on: ubuntu-latest
      timeout-minutes: 120
      permissions:
        contents: read
      steps:
        - uses: actions/checkout@v4
        - name: Run the section 8 two-distinct-path recipe as printed
          shell: bash
          run: bash ci/bh1119-doc-lint.test.sh --run-recipe

It is not a `needs:` of `musl-binaries` (a doc recipe failure does not block a release; it fails the run).

**C5. New `crates/ms-cli/tests/bh1119_ms_docs.rs`** (assert_cmd, tempfile, serde_json; each test in its own
`tempfile::tempdir()` holding `broken.txt` = the README card plus `\n`). The README is read as
`Path::new(env!("CARGO_MANIFEST_DIR")).join(env!("CARGO_PKG_README"))` (`readme = "README.md"` in the manifest), not
a `"README.md"` literal, which code-docs-check would match against the root docs file `README.md`.

- `bh1119_ms_i8_readme_exit_table_documents_candidate_exit_4`: some README line starts with
  ``| `4` | **`VERIFY-ME` Candidate**``; no README line starts with ``| `5` |``.
- `bh1119_ms_i8_readme_repair_example_matches_cli`: README contains the broken card; `ms repair --in broken.txt`
  exits 4; its stdout's last line is the corrected card; for every stderr line `L`, the README has a line exactly
  `#   L` (checked before the next assertion); the README has a line exactly `# exit status: 4`.
- `bh1119_ms_i8_control_readme_json_example_is_candidate`: `ms repair --in broken.txt --json` exits 4, `verdict`
  is `candidate`, `corrected_chunks[0]` is `ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f`.

**C6. New `crates/ms-codec/tests/bh1119_ms_docs.rs`** (README read through `CARGO_PKG_README` as C5, whitespace
runs collapsed to one space):

- `bh1119_ms_i8_codec_readme_no_too_many_errors_guarantee`: the README does not contain ``When the input exceeds the
  `t=4` correction capacity, the call returns``, and does contain ``corrects to a *different* valid string``.
- `bh1119_ms_i8_library_two_card_alias_characterisation`: triage-ms-t2's test, unchanged (encode Entr `[0;16]` and
  `[0x11;16]`; flip the second's last char `p`<->`q`; `decode_with_correction` returns one correction and the
  `[0x11;16]` payload). The consultant's "retain the two-card case as a regression".

## 4. RED rows (run on base `0f24ee79`)

Lint (reference implementation, `TMPDIR` under `/scratch/cache`), `bash ci/bh1119-doc-lint.test.sh`, rc=1:

| row | measured on base | expected after the fix |
|---|---|---|
| `bh1119_ms_i7_readme_install_names_release_tag` | `FAIL ...: README has no line 'cargo install --locked --git https://github.com/bg002h/mnemonic-secret --tag ms-cli-v0.21.0 ms-cli'` | `ok` |
| `bh1119_ms_i10_printed_repro_paths_exist` | `FAIL ...: printed command(s) not in this checkout (exit 127 as printed): ci/repro/cc-validate.sh ci/repro/double-build.sh ci/repro/gzip-residue.sh` | `ok` |
| `bh1119_ms_i10_toolkit_ref_matches_workflow` | `FAIL ...: doc has no 'TOOLKIT_REF=<40-hex>' line` | `ok` |
| `bh1119_ms_i10_no_stale_f324_known_red` | `FAIL ...: doc still says F-324 is KNOWN-RED (fixed 2026-09-02, man-release.yml)` | `ok` |
| `bh1119_ms_i11_aarch64_packaging_uses_commit_epoch` | `FAIL ...: member mtime '-9223372036854775808 ms' != commit epoch '2026-11-22 23:34:16'; stderr: tar: Substituting -9223372036854775807 for unknown date format ‘@’` | `ok` |
| `bh1119_ms_i11_aarch64_packaging_missing_binary_fails` | `FAIL ...: block exited 0 with no binary to package (archive: 1)` | `ok` |

(The epoch string is HEAD's committer time and changes with the base; the builder's RED shows the base it runs on.)

Rust, `cargo nextest run --locked --build-jobs 2 -j 2 --no-fail-fast -p ms-cli -p ms-codec --test bh1119_ms_docs`:
measured `5 tests run: 2 passed, 3 failed`.

| row (= test fn) | measured on base | expected after the fix |
|---|---|---|
| `bh1119_ms_i8_readme_exit_table_documents_candidate_exit_4` | FAIL ``README exit table has no `4` VERIFY-ME Candidate row (the code a correction returns)`` | PASS |
| `bh1119_ms_i8_readme_repair_example_matches_cli` | FAIL `README example stderr lacks: warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '\| age -e ...')` | PASS |
| `bh1119_ms_i8_codec_readme_no_too_many_errors_guarantee` | FAIL `README still promises TooManyErrors beyond t=4` | PASS |
| `bh1119_ms_i8_control_readme_json_example_is_candidate` | PASS (control) | PASS |
| `bh1119_ms_i8_library_two_card_alias_characterisation` | PASS (characterisation) | PASS |

`--run-recipe` (`bh1119_ms_i10_section8_recipe_runs`), run on base: rc=1, stderr
`bash: line 2: ci/repro/double-build.sh: No such file or directory`, the same for `cc-validate.sh`, then
`bash: line 4: VER: No such file or directory` (the `<VER>` placeholder parsed as a redirection), and
`FAIL bh1119_ms_i10_section8_recipe_runs: section 8 block exited non-zero`. On the scratch GREEN this host refused docker
(`permission denied while trying to connect to the docker API`) after the toolkit clone, checkout and `git
archive` succeeded. Its GREEN is the C4 job on the PR head (section 8).

## 5. Tests that change

None. No existing test, fixture or help text reads the changed README rows or doc sections (grep of `crates/` and
`ci/` at base: no `REPAIR_APPLIED`, `cargo install ms-cli`, `KNOWN-RED` or `verify-reproducibility` reader). The
GREEN scratch workspace run is in section 8.

## 6. Toolkit manual lockstep

None. No `ms` flag or printed text changes. The manual already documents exit 4 for `ms repair`
(`43-ms.md` L786-791, toolkit `b57187b3`), and D5 copies its rows. The toolkit gui manual's install chapters
(`docs/manual-gui/src/20-install/2[123]-*.md`) already warn against `cargo install ms-cli`.

## 7. Coverage map (item -> rows)

| item / text | rows |
|---|---|
| I7 (D1-D3) | bh1119_ms_i7_readme_install_names_release_tag |
| I8 CLI (D4-D6) | bh1119_ms_i8_readme_exit_table_documents_candidate_exit_4, bh1119_ms_i8_readme_repair_example_matches_cli, bh1119_ms_i8_control_readme_json_example_is_candidate |
| I8 library (D7) | bh1119_ms_i8_codec_readme_no_too_many_errors_guarantee, bh1119_ms_i8_library_two_card_alias_characterisation |
| I10 (D8) | bh1119_ms_i10_printed_repro_paths_exist, bh1119_ms_i10_toolkit_ref_matches_workflow, bh1119_ms_i10_no_stale_f324_known_red, bh1119_ms_i10_section8_recipe_runs (C4) |
| I11 (D9) | bh1119_ms_i11_aarch64_packaging_uses_commit_epoch, bh1119_ms_i11_aarch64_packaging_missing_binary_fails |
| D10, D11 | none (CHANGELOG prose) |

## 8. Gate (builder runs once to a file and greps it)

Rust pin `cargo 1.85.0`, `$HOME/.cargo/bin` first in PATH, `--locked`, no `--release`, worktree-own `target/`:

    cargo nextest run --locked --workspace --build-jobs 2 -j 2 --no-fail-fast
    cargo test --locked --doc --workspace
    cargo clippy --locked --all-targets --workspace -- -D warnings
    cargo +1.95.0 fmt --all -- --check
    bash ci/code-docs-check.sh && bash ci/code-docs-check.test.sh && bash ci/docs-only.test.sh
    bash ci/bh1119-doc-lint.test.sh

Expected: `786 tests run: 786 passed, 23 skipped` (base 781 plus the 5 Rust rows), every other command rc=0, the
lint printing six `ok` lines. Measured on the scratch GREEN worktree (all of sections 2 and 3 applied except D10,
D11; own target dir): exactly that. The C3/C4 workflow edits, applied there separately, parse
(`yaml.safe_load`), pass `actionlint` with no output, and keep `code-docs-check.sh` at `ok`.

Then, on the pushed PR branch, dispatch `man-release.yml` (`workflow_dispatch`, ref = the PR branch) and require
the `repro-doc-recipe` job green: that is the first run of the D8 block's container half (not run by the author,
see section 4). Not covered: the macOS, musl, release, test-release-mlock-einval, miri, FreeBSD, fuzz,
history-purge, g6-invariant and vendor-freshness CI jobs (CI on the PR head); the D9 block with a real `cross`
build (its packaging lines are run by rows 5-6 with a stub; the build half is `man-release.yml`'s own step);
running D1 on a clean machine (see section 10 for what was run); review.

## 9. Files this FIX changes (for build ordering)

`crates/ms-cli/README.md`, `crates/ms-codec/README.md`, `docs/verify-reproducibility.md`, `CHANGELOG.md`,
`ci/bh1119-doc-lint.test.sh` (new), `ci/code-docs-exempt.txt`, `crates/ms-cli/tests/bh1119_ms_docs.rs` (new),
`crates/ms-codec/tests/bh1119_ms_docs.rs` (new), `.github/workflows/rust.yml`, `.github/workflows/man-release.yml`.
Overlap with sibling ms FIX branches (origin, 2026-11-22): `CHANGELOG.md` only (every ms row appends under
`[Unreleased]`); `fix/ms-printed-commands` names `crates/ms-cli/src/{main,cmd/encode,cmd/split}.rs` and
`scripts/`; `fix/ms-{hashlock-emit-record,inspect-share-length,codex32-ctors,argv-substitute}` show no diff against
`origin/master` (`git diff --stat origin/master...<branch>`).

## 10. Notes for review

- T25's second step (registry pin after mnemonic-io-lib 0.1.2, then publish) is not in this FIX. When it lands, D1,
  D2 and lint row 1 change together.
- C4's placement (man-release.yml, tag and dispatch triggers, not a `needs:` of the publish job) is the author's
  engineering choice for T27's "run it in CI": the block needs docker and network and runs several release builds
  (double-build, cc-validate, remap-off-negative), too slow for every PR. A reviewer may move it.
- D1 was run on this host as printed plus `--root <scratch> --debug -j 2` (profile and parallelism only, to
  spare the shared host), `CARGO_TARGET_DIR` under `/scratch/cache`: rc=0, `Installed package `ms-cli v0.21.0
  (https://github.com/bg002h/mnemonic-secret?tag=ms-cli-v0.21.0#423f1734)` (executable `ms`)`, and the installed
  `ms --version` printed `ms 0.21.0`. A clean machine (empty `~/.cargo`) was not tried.

## Appendix A: reference implementation of C1 (measured RED on base, GREEN on the scratch worktree)

The builder may commit this as `ci/bh1119-doc-lint.test.sh` (only the two header comment lines differ from the
measured copy) or write its own to the C1 rows; either way the RED and GREEN of section 4 are re-run.

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
    iorev="$(grep -oE 'mnemonic-engrave\?rev=[0-9a-f]{40}' Cargo.lock | head -1 | grep -oE '[0-9a-f]{40}')"
    # A reader fills the placeholders the old block printed; the fixed block has none.
    blk="$(printf '%s\n' "$blk" | sed -e 's/<tagged-commit-SHA>/HEAD/g' -e "s/<VER>/$ver/g" -e "s/<IO_LIB_REV>/$iorev/g")"
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
