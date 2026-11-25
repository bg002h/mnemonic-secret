# Verifying reproducibility of the `ms` musl release binaries

> **Scope.** This document covers the **x86_64-unknown-linux-musl** and
> **aarch64-unknown-linux-musl** `ms` CLI binaries published on each
> `ms-cli-v*` release. It is the per-binary verify recipe for mnemonic-secret's
> leg of the constellation **`reproducible-builds-musl`** cycle (toolkit-led —
> `mnemonic-toolkit`). The hermetic build environment, the gate scripts, and the
> digest-pinned container are **homed in the toolkit**
> (`bg002h/mnemonic-toolkit`); this repo commits only its own `vendor/` tree, a
> ~15-line caller stub (`.github/workflows/man-release.yml` `repro` job), the
> re-homed `musl-binaries` build, and `Cross.toml`.
>
> **`ms` carries no git source.** `ms` does NOT depend on the miniscript
> `[patch.crates-io]` fork (it has no miniscript dependency at all), and
> `mnemonic-io-lib`, the IO mechanism shared across the m-format
> constellation, comes from crates.io: P2 (2026-08-27) pinned it by GitHub rev,
> and FIX_bughunt1119_ms_out_writes (2026-11) moved it to the registry. So the
> `cargo vendor` graph is a clean crates.io graph and the `[source]` activation
> is the **TWO-block** form: crates-io and vendored-sources.

## 1. What reproducibility buys you — provenance, not just integrity

The published `SHA256SUMS.<arch>` next to each release tarball lets you confirm a
download matches **what the maintainer uploaded** (integrity). It does NOT, on
its own, tell you the maintainer built it **from the claimed source**. A
*reproducible* build closes that gap: if you rebuild from the exact source at the
exact tag in the exact pinned environment and get the **bit-for-bit identical**
tarball, the published hash becomes a **provenance** statement — "provably built
from this source@commit" — verifiable by anyone, with no trust in the
maintainer's machine.

## 2. Source — keyed off the COMMIT SHA (not the tag name)

```sh
git clone https://github.com/bg002h/mnemonic-secret
cd mnemonic-secret
git checkout <ms-cli-vX.Y.Z>                        # the release tag
git rev-parse HEAD                                  # MUST equal published source_commit
```

**Invariant.** The published hash is valid **only** for the tuple
`{ source COMMIT SHA + SOURCE_DATE_EPOCH + container digest }`. A tag is mutable;
if it is moved / re-cut / force-pushed, the tagged-commit timestamp changes, the
epoch changes, the binary changes, and the previously-published hash is
**invalidated and re-published**. Always verify against the **commit SHA**
published in `PROVENANCE.<arch>.txt`, not the tag name.

`SOURCE_DATE_EPOCH` is derived identically by the maintainer's CI and by you:

```sh
SOURCE_DATE_EPOCH=$(git show -s --format=%ct <tagged-commit-SHA>)
```

`%ct` is the **committer date of the exact tagged commit** — NOT the tag name,
NOT an annotated tag's own `%(taggerdate)`.

## 3. The pinned environment — `docker pull` the container BY DIGEST

The x86_64 build runs inside a **digest-pinned** container homed in the toolkit
(`Dockerfile.repro` in `bg002h/mnemonic-toolkit`). The **built, layered image is
the source of truth** and is published by digest to GHCR — you `docker pull` it,
you do **not** rebuild `Dockerfile.repro` from apt (the transitive `musl-tools`
apt deps are NOT pinned by the base image digest and would resolve to whatever
the Debian mirror serves that day).

```sh
# From PROVENANCE.x86_64.txt:
CONTAINER_IMAGE=ghcr.io/bg002h/repro-musl-mnemonic-secret@sha256:<BUILT-DIGEST>
docker pull "$CONTAINER_IMAGE"      # no auth needed — the package is PUBLIC
```

> **Maintainer one-time setup — the `repro-musl-mnemonic-secret` GHCR package MUST be Public.**
> GHCR container packages are **private by default**, and an external rebuilder
> pulling by digest does so **without a token** — so the package has to be
> public for this provenance model to work. The toolkit's
> `reproducible-musl-build.yml` `build-container` job attempts to self-promote it
> to public (it is `|| true`, never hard-failing). If the self-promotion does not
> take, an admin sets it **once** by hand on the toolkit repo: GitHub →
> Packages → `repro-musl-mnemonic-secret` → Package settings → Danger Zone → Change visibility →
> Public.

- **Base image** (recorded in the toolkit's `Dockerfile.repro`): the official
  `rust:1.85.0` Debian image pinned by index digest
  `sha256:0ff31c9ffa641a62e48d543fb00b4960955ea375f40776f40f585b89e654cc5e`.
- **Built image** (`<BUILT-DIGEST>`): resolved + pushed by CI
  (`reproducible-musl-build.yml` → `build-container`) and recorded in each
  release's `PROVENANCE.x86_64.txt`.
- **Fallback only — building the container from `Dockerfile.repro`:** pin apt via
  `snapshot.debian.org` at a fixed timestamp + `apt-get install
  musl-tools=<exact-version>`. The canonical channel is
  `docker pull <built-digest>`, not a from-source rebuild.

### The fixed in-container layout

The container fixes `WORKDIR /build/src` and `CARGO_HOME=/cargo`. These fixed
literals are what make the `--remap-path-prefix` from-side a known constant
(`/build/src=/build`, `/cargo=/cargo`) — see §4.

## 4. The exact x86_64 build command + full env

Run **inside the pinned container**, at the fixed `/build/src`, with
`--network=none` (the build is fully offline against the committed `vendor/`
tree):

```sh
docker run --rm --network=none \
  -v "$PWD":/build/src -w /build/src \
  -e CARGO_HOME=/cargo \
  -e RUSTUP_TOOLCHAIN=1.85.0 \
  -e SOURCE_DATE_EPOCH="$SOURCE_DATE_EPOCH" \
  -e LC_ALL=C -e TZ=UTC \
  -e CARGO_BUILD_RUSTFLAGS="--remap-path-prefix=/build/src=/build --remap-path-prefix=/cargo=/cargo" \
  -e CFLAGS="-ffile-prefix-map=/build/src=/build -ffile-prefix-map=/cargo=/cargo" \
  -e CFLAGS_x86_64_unknown_linux_musl="-ffile-prefix-map=/build/src=/build -ffile-prefix-map=/cargo=/cargo" \
  "$CONTAINER_IMAGE" \
  bash -euxo pipefail -c '
    umask 022
    cargo build --locked --offline --release \
      --target x86_64-unknown-linux-musl -p ms-cli --bin ms \
      --config '"'"'source.crates-io.replace-with="vendored-sources"'"'"' \
      --config '"'"'source.vendored-sources.directory="vendor"'"'"'
    tar --sort=name --owner=0 --group=0 --numeric-owner --mtime="@$SOURCE_DATE_EPOCH" \
      -cf - -C target/x86_64-unknown-linux-musl/release ms \
      | gzip -n -9 > ms-<VER>-x86_64-linux-musl.tar.gz
  '
```

Notes on each load-bearing flag:

- **`--remap-path-prefix` is the *top-level* flag** (NOT `-Cremap-path-prefix`,
  which errors `unknown codegen option` on 1.85.0). It is delivered via the
  `CARGO_BUILD_RUSTFLAGS` **env** at the fixed `/build/src` — NOT a committed
  `.cargo/config.toml` value (a committed config value is passed to rustc
  verbatim with no `$PWD` expansion → it would no-op and give false assurance).
  This remap is the single biggest lever; it removes the absolute build-path
  leak in `.rodata` (a `file!()`/panic-`Location` literal) that makes default
  builds non-reproducible — and closes the `$HOME` privacy leak.
- **`CFLAGS` / `CFLAGS_<triple>` `-ffile-prefix-map`** strips absolute paths from
  the libsecp256k1 objects compiled by `cc-rs` under `musl-gcc`.
- **`SOURCE_DATE_EPOCH`** neutralizes `cc`'s `__DATE__` / `__TIME__`.
- **The TWO-block job-scoped `[source]` activation is mandatory.** The `vendor/`
  directory is committed but **inert** — there is **no committed `.cargo/config.toml
  [source]` block** (a repo-global `[source]` block would bleed into every other
  cargo job via cargo's directory-ancestry config discovery). The redirect is
  activated **job-scoped** on the build command via `cargo --config` (stable
  since 1.63): crates-io and vendored-sources, the two blocks `cargo vendor`
  emits for a crates.io-only graph (`mnemonic-io-lib` included). An external
  rebuilder MUST pass the same two `--config` overrides (or use an isolated
  `$CARGO_HOME/config.toml` carrying the verbatim `cargo vendor` output). **The
  miniscript git-fork stanza is absent** — that one exists only in the toolkit.
- **`--locked --offline`** + the committed `vendor/` tree mean the compile
  touches **no live external registry or git host** at build OR vendor time.

This is the EXACT command the maintainer's CI runs (re-homed into the same
container — see `.github/workflows/man-release.yml` `musl-binaries` x86_64 leg).

## 5. Expected per-artifact SHA-256 + the provenance tuple

Each release attaches, per arch:

- `ms-<VER>-<arch>-linux-musl.tar.gz` — the static musl binary tarball.
- `SHA256SUMS.<arch>` — its SHA-256.
- `PROVENANCE.<arch>.txt` — the tuple. For x86_64:

  ```
  artifact:          ms-<VER>-x86_64-linux-musl.tar.gz
  sha256:            <hash>
  source_commit:     <full 40-char SHA>
  source_date_epoch: <epoch>
  container_image:   ghcr.io/bg002h/repro-musl-mnemonic-secret@sha256:<BUILT-DIGEST>
  ```

  For aarch64 the tuple cites the **`cross` image digest** instead (see §9):

  ```
  artifact:          ms-<VER>-aarch64-linux-musl.tar.gz
  sha256:            <hash>
  source_commit:     <full 40-char SHA>
  source_date_epoch: <epoch>
  cross_image:       ghcr.io/cross-rs/aarch64-unknown-linux-musl@sha256:<CROSS-DIGEST>
  ```

## 6. Compare

```sh
sha256sum -c SHA256SUMS.x86_64        # OK ⇒ your rebuild matches the published artifact
```

**Signature.** The rebuild proves the tarball came from this source; the signature proves the published `SHA256SUMS.<arch>` you compared against came from the project: from the first release built by the signing
workflows on, every checksum file is signed with
[minisign](https://jedisct1.github.io/minisign/) by the m-format constellation's
release key (key id `EF2B8D34D8409754`, shared by every constellation repo):

```text
RWRUl0DYNI0r72HYC0ou+T/7pHEf0km3a8RWHwqGwZmIEMWtiSd4k0B5
```

| your download | checksum file | its signature |
|---|---|---|
| Linux x86_64 / aarch64, static musl (`ms-<ver>-<arch>-linux-musl.tar.gz`) | `SHA256SUMS.x86_64` / `SHA256SUMS.aarch64` | `SHA256SUMS.x86_64.minisig` / `SHA256SUMS.aarch64.minisig` |
| macOS amd64 / arm64, Windows amd64 | `SHA256SUMS.portable` | `SHA256SUMS.portable.minisig` |

Verify the signature first, then the checksum it vouches for:

```sh
minisign -Vm SHA256SUMS.x86_64 -P RWRUl0DYNI0r72HYC0ou+T/7pHEf0km3a8RWHwqGwZmIEMWtiSd4k0B5
sha256sum -c SHA256SUMS.x86_64 --ignore-missing
```

(Swap in the checksum file for your platform from the table. On macOS, where
`sha256sum` is usually absent, use `shasum -a 256 -c <file> --ignore-missing`.) A signature
proves who published the checksums; the checksum alone proves only that the
download is intact. Releases from before signing began have no `.minisig`.
The toolkit's `scripts/install.sh` runs this check for you when `minisign` is
installed.


On a mismatch, `diffoscope` the two tarballs. **Ignore** `target/.fingerprint`
and `.rustc_info.json` — they are non-reproducible cache artifacts, not part of
the shipped binary or tarball. Also check the gzip header: the mtime field
(offset 4–7) must be **zero** and the OS byte (offset 9) must equal the pinned
`03` (Unix) — a non-`-n` or divergent-gzip build would ship a different tarball
hash even with a byte-identical inner binary.

## 7. Scope honesty — local installs are NOT reproducible by default

Reproducibility is guaranteed **only** when building **at the fixed `/build/src`
inside the pinned container** with the env above. **`cargo install` /
`cargo install --git` at an arbitrary `$PWD` are NOT reproducible-by-default** —
they build at a path no static config canonicalizes, so the `.rodata` build-path
leak returns. (A future toolchain bump to Cargo-native `trim-paths`, which
self-canonicalizes, would restore local-install reproducibility; that is
nightly-gated on the deliberate 1.85.0 pin.)

## 8. CI proof — the two-distinct-path self-test (homed in the toolkit)

The toolkit's reusable workflow (`reproducible-musl-build.yml`, called by this
repo's `man-release.yml` `repro` job) does not just rebuild once — it builds `ms`
at **two distinct** real paths (`/build-a/src` and `/build-b/src`, both remapped
to `/build`) and asserts the two binaries + tarballs + libsecp `.o` are
byte-identical. **The two-distinct-path shape is load-bearing:** the
`--remap-path-prefix` from-side only *varies* — and the remap is only *proven
effective* — when the two real paths differ.

**The repro gate is runnable WITHOUT a release tag.** This repo's
`man-release.yml` has a `workflow_dispatch` trigger; a bare "Run workflow" click
runs the `repro` caller job (the toolkit gate) against the current branch — the
man-pages build + the release-upload steps and the whole `musl-binaries` job are
`if:`-guarded off for a manual dispatch, so only the gate runs.

**F-324 — closed by the crates.io pin.** From P2 to
FIX_bughunt1119_ms_out_writes `ms` pinned `mnemonic-io-lib` by git rev, and the
gate needed a git-source `[source]` stanza beside the crates-io one. With the
crates.io pin there is none: the `repro` caller job passes only an empty
`miniscript_rev` (no `git_source_url` / `git_source_rev`), at toolkit revision
`4120af853ec68dc515cb4ac0eb7245a018a50c78` (the `uses:` pin and `toolkit_ref`,
which must match). Before the toolkit gained the git-source inputs the gate
could not resolve the git-pinned `mnemonic-io-lib` offline and failed at the
`ms-cli-v0.17.0` tag (run 33621228397).

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
  -e SOURCE_DATE_EPOCH="$EPOCH" -e REPRO_OUT_DIR=/build/canonical \
  -e CARGO_HOME=/cargo -e RUSTUP_TOOLCHAIN=1.85.0 -e LC_ALL=C -e TZ=UTC \
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
`BIN=mnemonic`) and its miniscript fork; the `-e` lines select `ms-cli` / `ms`
and clear `MINISCRIPT_REV`, as the `repro` job's `with:` block does. `RUSTUP_TOOLCHAIN=1.85.0` selects the image's
installed toolchain, so rustup does not try to fetch the `rustfmt` and
`clippy` components that `rust-toolchain.toml` lists (the container has no
network). Each script exits non-zero on a failed check, and
`set -e` stops the block there. Building the image from `Dockerfile.repro` is
enough for this proof, which compares two builds made in one image; to match a
published hash, use the pulled image of §3 and the command of §4.

## 9. aarch64-unknown-linux-musl — built via `cross` under QEMU

The aarch64 binary has no native runner, so it is built with
[`cross`](https://github.com/cross-rs/cross) under QEMU user-mode emulation.
`cross` ships its **own bundled aarch64-musl C toolchain** inside a runtime
container image — a **different** `musl-gcc` than the x86_64 leg's container
`musl-tools`.

### 9.1 The pinned cross toolchain — `Cross.toml`, by digest

The cross runtime image is pinned **by sha256** in this repo's committed
`Cross.toml` (not a floating `cross`-version tag) — this is the real aarch64
toolchain pin:

```toml
[target.aarch64-unknown-linux-musl]
image = "ghcr.io/cross-rs/aarch64-unknown-linux-musl@sha256:702154f52b2d8091671aa2c84d5582d849f949977228c735ff8462f93cc0e1e4"
```

(Resolved at adoption time, 2026-06-24, via the GHCR registry manifest API:
`ghcr.io/cross-rs/aarch64-unknown-linux-musl` tag `0.2.5` == `latest` — the
version `cargo install --locked cross` resolves to.)

`cross` does **not** forward host env into its container automatically, so the
determinism-bearing vars are listed under `Cross.toml [build.env] passthrough`
(`SOURCE_DATE_EPOCH`, `CFLAGS`, `CFLAGS_aarch64_unknown_linux_musl`, `LC_ALL`,
`TZ`, `CARGO_BUILD_RUSTFLAGS`). Without this list the cc/rustflags mitigations
would silently never reach the aarch64 compiler.

### 9.2 The remap from-side is `/project`, not the host path

`cross` bind-mounts the project to its **fixed internal path `/project`** inside
the container (cross v0.2.5 `src/docker/local.rs`: `-v <host_root>:/project`),
sets `CARGO_HOME=/cargo`. So the in-container compiler sees the source at
`/project`, and the remap from-side is **`/project=/build`** (plus
`/cargo=/cargo`) — **not** the host checkout path.

### 9.3 The exact aarch64 build command (release leg)

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
    --config 'source.vendored-sources.directory="vendor"'
tar --sort=name --owner=0 --group=0 --numeric-owner --mtime="@$EPOCH" \
  -cf - -C target/aarch64-unknown-linux-musl/release ms \
  | gzip -n -9 > "ms-${VER}-aarch64-linux-musl.tar.gz.partial"
mv "ms-${VER}-aarch64-linux-musl.tar.gz.partial" "ms-${VER}-aarch64-linux-musl.tar.gz"
sha256sum "ms-${VER}-aarch64-linux-musl.tar.gz"
```

The same TWO-block `[source]` activation, the same `--locked
--offline`, the same gzip-pinned tar as the x86_64 leg. `cross` forwards the
`--config` flags to the inner cargo; the committed `vendor/` (at
`/project/vendor`) makes the build fully offline.

> **Honest caveat — local rebuilds may not byte-match without the pinned env.**
> A casual `cargo build`/`cross build` at an arbitrary `$PWD` without the remap +
> `SOURCE_DATE_EPOCH` + the pinned image will NOT reproduce the published hash
> (the build-path leak and `cc` timestamps return). Reproducibility holds only
> for the documented commands run in the digest-pinned environment.
