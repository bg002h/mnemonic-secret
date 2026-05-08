# ms-codec

Reference implementation of the **ms1** backup format — BIP-93 codex32 directly applied to **BIP-39 entropy** for steel-engravable backups with strong BCH error correction.

ms1 is a Bitcoin self-custody backup format designed to engrave alongside sibling formats `mk1` (xpubs) and `md1` (descriptors). The encoded string self-checks for up to 8 character substitutions and self-corrects up to 4 — far stronger than BIP-39's own 4-bit checksum, which is too weak to localize errors on engraved media.

## v0.1 scope

- **In:** BIP-39 entropy (16/20/24/28/32 B = 12/15/18/21/24 word counts). Tag: `entr`.
- **Out:** Direct BIP-32 master seed and serialized xpriv — reserved-not-emitted; deferred to v0.2+ with separate framing.

K-of-N share encoding planned for v0.2.

## Quickstart

```rust
use ms_codec::{encode, decode, Payload, Tag};

let entropy = vec![0xAAu8; 16]; // 16 raw bytes from a 12-word BIP-39 mnemonic
let s = encode(Tag::ENTR, &Payload::Entr(entropy.clone())).unwrap();
assert_eq!(s.len(), 50);

let (tag, payload) = decode(&s).unwrap();
assert_eq!(tag, Tag::ENTR);
assert_eq!(payload, Payload::Entr(entropy));
```

To recover a BIP-39 mnemonic from the decoded entropy, use the [`bip39`](https://crates.io/crates/bip39) crate:

```rust
# use ms_codec::Payload;
# let entropy = vec![0xAAu8; 16];
use bip39::{Language, Mnemonic};
let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy).unwrap();
println!("{}", mnemonic);
```

Then derive your BIP-32 master seed via the BIP-39 PBKDF2 (with optional passphrase) — exactly as your wallet does today.

## Engraving caveat: BIP-39 wordlist language

ms1 v0.1 does **not** carry the BIP-39 wordlist language on the wire. A user whose original wallet used a non-English wordlist who recovers via English-defaulted wallet software will silently derive a different BIP-32 master seed → different addresses → empty wallet. Users with non-English wallets MUST record their wordlist language alongside the engraved card. A future v0.2+ payload kind `mnem` (reserved tag) is allocated to address this on the wire.

## Documentation

Full design documents live in the [parent repo](https://github.com/bg002h/mnemonic-secret):

- `design/SPEC_ms_v0_1.md` — wire format, decoder rules, BIP-93 anchoring.
- `design/BRAINSTORM_ms_v0_1.md` — the rationale chain.
- `MIGRATION.md` — v0.1 → v0.2 share-encoding migration contract.

## Family

- **md-codec** ([crates.io](https://crates.io/crates/md-codec)) — wallet descriptors / templates (`md1`, HRP `md`).
- **mk-codec** ([crates.io](https://crates.io/crates/mk-codec)) — xpubs (`mk1`, HRP `mk`).
- **ms-codec** (this crate) — BIP-39 entropy (`ms1`, HRP `ms`).

## License

MIT License.
