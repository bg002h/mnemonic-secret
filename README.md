# mnemonic-secret (`ms1`)

Reference implementation of the **ms1** backup format — BIP-93 codex32 directly applied to **BIP-39 entropy** for steel-engravable backups with strong BCH error correction. Sibling to [`md1`](https://github.com/bg002h/descriptor-mnemonic) (wallet descriptors) and [`mk1`](https://github.com/bg002h/mnemonic-key) (xpubs); the three engrave together as a coherent restoration bundle.

Status: **v0.1.0** (entr-only). K-of-N share encoding planned for v0.2.

## What it does

Encode the entropy of a BIP-39 seed phrase as a `ms1`-prefixed BIP-93 codex32 string designed to engrave on metal. The encoded string self-checks for up to 8 character substitutions and self-corrects up to 4 — far stronger than BIP-39's own 4-bit checksum, which is too weak to localize errors on engraved media.

## Quickstart

```rust
use ms_codec::{encode, decode, Payload, Tag};

// 16 raw bytes from a 12-word BIP-39 mnemonic.
let entropy = vec![0xAAu8; 16];
let s = encode(Tag::ENTR, &Payload::Entr(entropy.clone())).unwrap();
assert_eq!(s.len(), 50);

// Engrave `s`. Recover later:
let (tag, payload) = decode(&s).unwrap();
assert_eq!(tag, Tag::ENTR);
assert_eq!(payload, Payload::Entr(entropy));
```

To recover a BIP-39 mnemonic from the decoded entropy:

```rust
use bip39::{Language, Mnemonic};
let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy).unwrap();
println!("{}", mnemonic);
```

Then derive your BIP-32 master seed via the BIP-39 PBKDF2 (with optional passphrase) — exactly as your wallet does today.

## Man pages

`ms` ships man pages generated from its own clap definition — the same source as `--help` — so they cannot drift from the binary. Three ways to install them:

1. **Automatic (default).** The [constellation installer](https://github.com/bg002h/mnemonic-toolkit) installs them alongside the binary into `~/.local/share/man/man1` — no sudo, no system files:

   ```sh
   sh -c "$(curl -fsSL https://raw.githubusercontent.com/bg002h/mnemonic-toolkit/master/scripts/install.sh)"
   ```

   Then `man ms` works (and `man ms-<subcommand>` for each subcommand). Pass `--no-man` to skip, or `--man-dir <dir>` to relocate.

2. **By hand.** If you installed the binary directly (`cargo install`), emit them yourself:

   ```sh
   ms gen-man --out ~/.local/share/man/man1
   ```

3. **Download.** Each release attaches a `ms-man.tar.gz` asset — extract it into your manpath.

If `man ms` can't find them (older `man-db`, or macOS/BSD `man` that doesn't auto-read `~/.local/share/man`): `man -M ~/.local/share/man ms`.

## Scope

| | v0.1 (this release) | v0.2 (planned) | v0.2+ |
|---|---|---|---|
| BIP-39 entropy `entr` (16/20/24/28/32 B) | ✓ emit + accept | + K-of-N share encoding | |
| BIP-32 master seed `seed` (64 B) | reserved-not-emitted | | + own framing |
| BIP-32 xpriv `xprv` (78 B) | reserved-not-emitted | | + own framing |
| K-of-N shares | not yet | ✓ for `entr` | + for other kinds |

The BIP-32 master seed backup use case is preserved at the application layer:
`BIP-39 phrase → entropy → ms1 entr → engrave → recover entropy → BIP-39 mnemonic → PBKDF2 → 64-B BIP-32 master seed`. Direct `seed` and `xprv` payloads are deferred to v0.2+ because they overflow BIP-93 codex32's length brackets when prepended with the v0.2-migration prefix byte. See [`design/SPEC_ms_v0_1.md`](design/SPEC_ms_v0_1.md) §1.3 for full discussion.

## Engraving caveat: BIP-39 wordlist language

ms1 v0.1 does NOT carry the BIP-39 wordlist language on the wire. A user whose original wallet used a non-English wordlist who recovers via English-defaulted wallet software will silently derive a different BIP-32 master seed → different addresses → empty wallet. Users with non-English wallets MUST record their wordlist language alongside the engraved card. A future v0.2+ payload kind `mnem` (reserved tag) is allocated for an "entropy + wordlist-language hint" payload that addresses this on the wire. See SPEC §6.3.

## Documentation

- [`design/SPEC_ms_v0_1.md`](design/SPEC_ms_v0_1.md) — wire format, decoder rules, BIP-93 anchoring.
- [`design/BRAINSTORM_ms_v0_1.md`](design/BRAINSTORM_ms_v0_1.md) — the rationale chain.
- [`MIGRATION.md`](MIGRATION.md) — v0.1 → v0.2 contract.
- [`design/RELEASE_PROCESS.md`](design/RELEASE_PROCESS.md) — release discipline.
- [`design/IMPLEMENTATION_PLAN_ms_v0_1.md`](design/IMPLEMENTATION_PLAN_ms_v0_1.md) — phase-by-phase build plan.

## Family

`ms-codec` is one of three sibling format crates plus a future toolkit:

- **md-codec** ([repo](https://github.com/bg002h/descriptor-mnemonic)) — wallet descriptors / templates (`md1`, HRP `md`).
- **mk-codec** ([repo](https://github.com/bg002h/mnemonic-key)) — xpubs (`mk1`, HRP `mk`).
- **ms-codec** (this crate) — BIP-39 entropy (`ms1`, HRP `ms`).
- **mnemonic-toolkit** (planned) — top-level integration: take a BIP-39 phrase, emit a complete ms1 + mk1 + md1 engravable bundle.

## Verifying your download

The release `ms-<version>-x86_64-linux-musl.tar.gz` and `…-aarch64-linux-musl.tar.gz`
binaries are **reproducible** — bit-for-bit rebuildable from source. Each release
attaches `SHA256SUMS.x86_64`, `SHA256SUMS.aarch64`, and `PROVENANCE.<arch>.txt`.

**Integrity** (did my download arrive intact?):

```sh
sha256sum -c SHA256SUMS.x86_64      # or SHA256SUMS.aarch64
```

**Provenance** (was it really built from this source — no hidden changes?):
independently rebuild and confirm you get the *same* hash. See
[`docs/verify-reproducibility.md`](docs/verify-reproducibility.md) for the exact
steps — in brief: `git checkout` the release commit (from `PROVENANCE.<arch>.txt`),
`docker pull ghcr.io/bg002h/repro-musl-mnemonic-secret@sha256:<digest>` (the pinned,
public build image), rebuild offline, and compare to `SHA256SUMS.<arch>`. A match
proves the published binary came from this source.

**Scope:** the static-musl Linux **x86_64** and **aarch64** `ms` binaries.
(gnu, macOS/Windows, and the GUI are not yet reproducible.) Note: a local
`cargo install` / `install.sh` build is *not* bit-for-bit reproducible — the
guarantee is for the published container-built release tarballs.

## License

Dual-licensed, at your option, under either the [MIT License](LICENSE) or the
[Unlicense](UNLICENSE) public-domain dedication — SPDX `MIT OR Unlicense`. Use
the Unlicense for maximal freedom, or MIT where a public-domain dedication
isn't accepted.
