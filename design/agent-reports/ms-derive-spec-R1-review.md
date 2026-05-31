# R1 Re-Review — SPEC_ms_derive.md (ms-cli 0.5.0)

Reviewer: feature-dev:code-reviewer (opus). Re-review after the R0 3C/5I/9M fold. Every cite
re-grepped against live source.

## Critical / Important — None.

All folds CONFIRMED (table):
- C1 oracle needs --template (convert.rs:1169/1173); ms derive omits it by design — §4#1 + §3.1 fixed.
- C2 port argv advisory (ms-cli has none; byte-shape matches toolkit secret_advisory.rs:34).
- C3 backfill repair + add derive + pin bumps (ms.rs lacks both; v0.4.1→v0.5.0, "ms 0.2.1"→"ms 0.5.0").
- I1 Derive alphabetically before Encode (3 exhaustive sites). I2 CliError quote + no-From<bitcoin>
  →map_err. I3 single-stdin via is_stdin_arg (parse.rs:97) mirror verify.rs:50. I4 mlock scoped
  (stdin pinned :65; inline Zeroizing-only). I5 manual "Five"→Seven (43-ms.md:4).
- M1 Secp256k1::new signing (derive_slot.rs:81). M4 --account u32 default 0. M5 ValueEnum{Bip44/49/84/86}
  + inline path (template.rs:62-72 map). M6 DeriveJson skip_serializing_if. M8 ArgGroup not-required.

Drift sweep clean: C2/I4 consistent (inline-not-pinned ⇒ advisory); M8 group (ms1/phrase/hex,
not-required) lets passphrase/language/template/account/network/json coexist (encode.rs:26 precedent);
I3 guard consistent with M8 (only active source); bitcoin-0.32 API correct; no-secret-on-stdout +
language-DEFAULT-annotation intact.

## Minor (non-blocking)
- Use decode's `Option<CliLanguage>` (no default_value) so the "defaulted" signal survives — NOT
  encode's eager default. [Folded post-R1: §3.1 --language bullet now states this explicitly.]

VERDICT: GREEN (0C/0I)
