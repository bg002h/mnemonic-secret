# ms hashlock H1 — spec §12 acceptance, measured

Controller-run, 2026-09-05, against `ms` built from the release commit `cd0a60f`
(`ms 0.18.0`, `cargo build --locked -p ms-cli`, isolated target dir) and `me 0.8.0`
built from mnemonic-engrave master `6d8ef65` (ms-codec 0.7 pin; H0 host half
merged at `024dd08`). Every command below was run once; outputs are verbatim
except where a 64-hex preimage is elided as `<64 hex>` (it is the corpus's
`hardened_x`, checked by equality) and the plate string is shown in full because
it is a corpus fixture, not anyone's secret. Scratch: `/scratch/code/shibboleth/.tmp/h1-acc/`.

The anchor phrase is the corpus's first `derivation` row of
`crates/ms-codec/tests/vectors/hashlock-v0.8.json`: `correct horse battery staple`
(`hardened_h` `3cf5d421…4c12`, `sha256_h` `b867db87…96cb`, `hardened_x` `c3e97525…2016`).

## Item 1 — hardened, default method

```
$ ms hashlock --hashlock-phrase-stdin < phrase.txt
exit=0
stdout: hash:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
stderr line 1: THIS CARD CARRIES THE PREIMAGE -- the secret. stdout carries only the public digest.
```
**PASS** — the digest is the spec's value and the card's first line names the preimage.

## Item 2 — `--method sha256`

```
$ ms hashlock --hashlock-phrase-stdin --method sha256 < phrase.txt
exit=0
stdout: hash:b867db875479bcc0287352cdaa4a1755689b8338777d0915e9acd9f6edbc96cb
stderr: 1 line containing "brainwallet"
```
**PASS** — the W-5 value, and the brainwallet warning is present.

## Item 3 — `--out X.txt` and re-derivation

```
$ ms hashlock --hashlock-phrase-stdin --out X.txt < phrase.txt
exit=0  stdout: hash:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
X.txt: ms10hashsq0p7jaf9gsjjpkjvll2l274w8a388xgqzlewp73scptwxgtjugspvs8tklufg89hqj  (75 chars, mode 600)
$ ms hashlock --in X.txt
exit=0  stdout: hash:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
```
**PASS** — 75 characters, `ms10hashsq…`, mode `0600`, and the file re-derives the
same digest. (The corpus pins X and H, not the plate string; item 4's decode
shows the plate carries exactly the corpus's `hardened_x`.)

## Item 4 — the other verbs

First attempt put the plate on argv and the argv guard refused it, as designed
(this is §6, not a defect):

```
$ ms decode <plate>
exit=1  ms: argument 2 on ARGV (arguments count from 0, and 0 is `ms` itself) is an ms1 string (or one share of an ms1 share-set), 75 characters long.
```

Through the file channel:

```
$ ms decode --in X.txt
exit=0, 3 lines, no BIP-39 words:
kind:      preimage (hashlock, 32 bytes / 64 hex characters)
preimage…: <64 hex>            == corpus hardened_x (c3e97525…2016)  ✓
digest:    3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12   ✓
$ ms inspect --in X.txt
exit=0
OK: would decode v0.8
kind: preimage
$ ms derive --in X.txt
exit=1  error: this is a hashlock preimage plate, not a seed backup; use `ms hashlock <ms1>` (or `ms hashlock --in FILE`) to re-derive its digest
$ ms verify --in X.txt
exit=1  error: this is a hashlock preimage plate, not a seed backup; use `ms hashlock <ms1>` (or `ms hashlock --in FILE`) to re-derive its digest
```
**PASS** — decode prints kind, hex and digest and never words; inspect reports
the kind with no reason; derive and verify refuse with the remedy; nothing panicked.

## Item 5 — `--random` gates

```
$ ms hashlock --random
exit=64  error: --random needs --out FILE: a preimage that reaches no file is data loss (--json is stdout and does not count)
$ ms hashlock --random --json
exit=64  (same refusal)
$ ms hashlock --random --out R.txt
exit=0   stdout: hash:6d83fb7…   R.txt mode 600
$ ms hashlock --random --out R.txt
exit=64  error: --out R.txt already exists; a --random preimage will not overwrite it (choose another file, or move the old one first)
```
**PASS** (operator ruling L25).

## Item 6 — `ms hashlock … | me sysw pack`

```
$ ms hashlock --hashlock-phrase-stdin --no-engraving-card < phrase.txt | me sysw pack --out payload.bin
exit=0  payload.bin: 146 bytes
stderr: sealing: NOT SEALED — no record in this payload is secret material, so there is nothing to encrypt. The container is cleartext …
```
**PASS (host half)** — stdin, no `--in`, builds the container carrying the public
`hash:` record. The device half ("the composer's `Which hash?` payload route
offers the record") is covered by the fork's composer payload-route tests
(`gui/composer_admit_test.go`, `sysw_admit`), unchanged by H0/H1; it was not
walked on the emulator in this acceptance.

## Item 7 — a `0x03` single is inert on the device and on `me`

Fork main `c4a64fc` (H0 merged): `sysw.Classify` and `seal.Classify` classify
the plate as unknown, and no engrave path offers it — pinned by the fork's seam
test (row `preimage-plate-0x03`, `device_admits: false`) and the door tests;
measured on the emulator built from that tree (typed door → "This record is a
hashlock preimage, not a seed. It is not engraved as one."; NFC door → "Unknown
format"; the same walk on 839fa5aa reaches "Confirm Codex32 Secret" at both
doors). `me` at its 0.7 pin:

```
$ printf '%s\n' "$PLATE" | me sysw pack --out h0.bin
me: record 0 (records count from 0) is a hashlock PREIMAGE plate (kind 0x03), not a seed record; this container cannot place one yet. …   exit=4
$ printf '%s\n' "$PLATE" | me seal --seal-secret --out h0.uf2
me: this record is a hashlock PREIMAGE plate (kind 0x03), not a seed record; …   exit=4
```
**PASS as far as software goes.** The flash of `c4a64fc` to the operator's
SH2 was NOT performed by the controller; the operator said to assume the boot
and ruled L26 "release regardless of the device". `me`'s 0.8 bump (H1b) is
follow-up F-473 with the pin test as its tripwire.

## Item 8 — the release carries everything

Release commit `cd0a60f`: CHANGELOG entries for ms-codec 0.8.0 and ms-cli
0.18.0 dated 2026-09-05 with the corpus pin
`a46c197a3640fe8af4ca4370b46a9637466649227163ce6761bb032354811d30`;
MIGRATION.md v0.7 → v0.8; both version bumps and the `=0.8.0` pin (Task 1);
`cargo publish -p ms-codec --dry-run --locked` exit 0; the manual chapter is a
cross-repo follow-up (D7, mnemonic-toolkit); both tags are pushed by the
release agent on the pushed SHA (its report: `push-ms-cd0a60f-release.md`).

## Verdict

Items 1–5 PASS exactly; item 6 PASS on the host half; item 7 PASS in software
with the flash assumed by the operator; item 8 complete except the manual
chapter (filed). The crates.io publish of ms-codec 0.8.0 is a separate decision
and was not run.

## Addendum — the released binary

After the tags landed (`push-ms-cd0a60f-release.md`): downloaded
`ms-0.18.0-x86_64-linux-musl.tar.gz` + `SHA256SUMS.x86_64` from release
`ms-cli-v0.18.0`, `sha256sum -c` OK, `ms 0.18.0`:

```
item 1: hash:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
item 2: hash:b867db875479bcc0287352cdaa4a1755689b8338777d0915e9acd9f6edbc96cb
item 3 (--in X.txt): hash:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
```
Identical to the local build. Seven assets: two musl tarballs, `ms-man.tar.gz`,
two PROVENANCE files, two SHA256SUMS.
