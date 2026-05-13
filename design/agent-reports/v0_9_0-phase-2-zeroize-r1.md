# v0.9.0 Phase 2 — Zeroizing wrappers R1 (cross-repo)

**Cross-repo companion** to
`/scratch/code/shibboleth/mnemonic-toolkit/design/agent-reports/v0_9_0-phase-2-zeroize-r1.md`.

This sibling-repo report is intentionally a pointer (avoiding
divergence): the canonical R1 report lives in mnemonic-toolkit
since the cycle owner repo + the bulk of findings target toolkit
sites. ms-secret-specific findings excerpted below for local
visibility.

## ms-secret findings excerpt

- **N-1** (Notable, conf 70): `crates/ms-cli/src/cmd/encode.rs:87-88`
  has redundant `Zeroizing` indirection. Three in-flight copies;
  the middle `entropy_for_codec` is dead weight. Simplify to
  `Payload::Entr((*entropy).clone())` directly.

- ms-codec encode/decode disciplines verified correct; the
  caller-wrap contract doc at `crates/ms-codec/src/payload.rs` is
  appropriate.

## Cross-repo verdict

**0 Critical / 4 Important / 5 Notable.** The 4 Important
findings target toolkit sites (synthesize_multisig_full unwrapped
entropy, bip85 SecretKey locals, derive_child stdin_passphrase,
lint evidence-substring coarseness). See the canonical report for
fold disposition.
