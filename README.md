# mnemonic-secret (`ms1`)

Reference implementation of the **ms1** backup format — BIP-93 codex32 directly applied to **BIP-39 entropy** (and to hashlock preimages) for steel-engravable backups with strong BCH error correction. Sibling to [`md1`](https://github.com/bg002h/descriptor-mnemonic) (wallet descriptors) and [`mk1`](https://github.com/bg002h/mnemonic-key) (xpubs); the three engrave together as a coherent restoration bundle.

Status: ms-codec 0.10.1 / ms-cli 0.21.0. Three payload kinds ship: `entr` (BIP-39 entropy), `mnem` (BIP-39 entropy plus its wordlist language) and the hashlock `preimage` (32 bytes, id `hash`). K-of-N share encoding of `entr` and `mnem` secrets ships too (`encode_shares` / `combine_shares`; `ms split` / `ms combine`). See [`MIGRATION.md`](MIGRATION.md) for when each arrived.

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

| Payload | Prefix byte | Single string | K-of-N shares |
|---|---|---|---|
| BIP-39 entropy `entr` (16/20/24/28/32 B) | `0x00` | ✓ emit + accept (id `entr`) | ✓ (`ms split` / `ms combine`) |
| BIP-39 entropy + wordlist language `mnem` | `0x02` | ✓ emit + accept (a non-English `--phrase`) | ✓ (the language survives the split) |
| Hashlock preimage (exactly 32 B) | `0x03` | ✓ emit + accept (id `hash`, `ms hashlock`) | not offered by `ms split` |
| BIP-32 master seed `seed` (64 B) | — | reserved-not-emitted; needs its own framing | — |
| BIP-32 xpriv `xprv` (78 B) | — | reserved-not-emitted; needs its own framing | — |

A reader dispatches on the payload kind: a hashlock preimage is not mnemonic
entropy, and is never printed as words.

The BIP-32 master seed backup use case is preserved at the application layer:
`BIP-39 phrase → entropy → ms1 entr → engrave → recover entropy → BIP-39 mnemonic → PBKDF2 → 64-B BIP-32 master seed`. Direct `seed` and `xprv` payloads are deferred because they overflow BIP-93 codex32's length brackets when prepended with the v0.2-migration prefix byte. See [`design/SPEC_ms_v0_1.md`](design/SPEC_ms_v0_1.md) §1.3 for full discussion.

## Engraving caveat: BIP-39 wordlist language

An ms1 card made from **hex entropy or an English phrase** (payload kind
`entr`) does NOT carry the BIP-39 wordlist language: the same entropy read
with a different wordlist is a different wallet. A card made from a
**non-English phrase** (`ms encode --phrase - --language <lang>`) is the `mnem`
kind, which carries the language on the wire, and `ms decode` reads it from
the card (`--language` is advisory there). Record your wordlist language
alongside the card anyway: an `entr` card cannot tell you, and wallet software
that does not read the language byte defaults to English. The BIP-39
passphrase is never part of the card. See
[`design/SPEC_ms_mnem_wordlist_language.md`](design/SPEC_ms_mnem_wordlist_language.md)
and SPEC §6.3.

## Documentation

- [`design/SPEC_ms_v0_1.md`](design/SPEC_ms_v0_1.md) — wire format, decoder rules, BIP-93 anchoring.
- [`design/BRAINSTORM_ms_v0_1.md`](design/BRAINSTORM_ms_v0_1.md) — the rationale chain.
- [`design/SPEC_ms_v0_2_kofn.md`](design/SPEC_ms_v0_2_kofn.md) — K-of-N shares.
- [`design/SPEC_ms_hashlock.md`](design/SPEC_ms_hashlock.md) — the hashlock preimage kind.
- [`MIGRATION.md`](MIGRATION.md) — wire and API changes per release (K-of-N shares, `mnem`, the preimage kind).
- [`design/RELEASE_PROCESS.md`](design/RELEASE_PROCESS.md) — release discipline.
- [`design/IMPLEMENTATION_PLAN_ms_v0_1.md`](design/IMPLEMENTATION_PLAN_ms_v0_1.md) — phase-by-phase build plan.

## Family

`ms-codec` is one of three sibling format crates plus the toolkit:

- **md-codec** ([repo](https://github.com/bg002h/descriptor-mnemonic)) — wallet descriptors / templates (`md1`, HRP `md`).
- **mk-codec** ([repo](https://github.com/bg002h/mnemonic-key)) — xpubs (`mk1`, HRP `mk`).
- **ms-codec** (this crate) — secret material: BIP-39 entropy and hashlock preimages (`ms1`, HRP `ms`).
- **mnemonic-toolkit** ([repo](https://github.com/bg002h/mnemonic-toolkit)) — top-level integration: take a BIP-39 phrase, emit a complete ms1 + mk1 + md1 engravable bundle.

## Verifying your download

The release `ms-<version>-x86_64-linux-musl.tar.gz` and `…-aarch64-linux-musl.tar.gz`
binaries are **reproducible** — bit-for-bit rebuildable from source. Each release
attaches `SHA256SUMS.x86_64`, `SHA256SUMS.aarch64`, and `PROVENANCE.<arch>.txt`.

**Integrity** (did my download arrive intact?):

```sh
sha256sum -c SHA256SUMS.x86_64      # or SHA256SUMS.aarch64
```

**Origin** (did the project publish these checksums?): from the first release built by the signing
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
