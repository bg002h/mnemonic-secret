# PLAN — ms `combine` validates Entr payload length (audit I9)

**Status:** R0 GREEN at R1 (2026-06-10, `design/agent-reports/combine-entr-length-plan-r0-round2-review.md`) — implementation may proceed
**Source grounding:** ms-codec/ms-cli `master` @ `1ca26fc`.
**Resolves:** `design/FOLLOWUPS.md::audit-2026-06-10-backlog` item `combine-no-length-validation-panic` (audit I9, CLUSTER D).

## 0. The bug (confirmed at source)

`ms combine --to phrase` aborts with a panic (exit 101) on a **valid-checksum, non-standard-length** share set, instead of returning a clean error.

Call chain:
- `ms-cli combine.rs:58` `ms_codec::combine_shares(&shares)?` → `combine.rs:77` `emit_phrase(...)` → `combine.rs:96-97` `Mnemonic::from_entropy_in(lang, entropy).expect("combine_shares validates entropy length; from_entropy_in cannot fail")`.
- `combine_shares` (shares.rs:180, recovers via `interpolate_at(S)`) → `shares.rs:241` `dispatch_payload(&data)?`.
- `dispatch_payload` (envelope.rs:167-188): the **`Mnem` arm validates** (`p.validate()?`, :180) but the **`Entr` arm does NOT** — `:171` returns `Payload::Entr(data[1..].to_vec())` with no validation.

So a recovered Entr payload of non-standard length (entropy length ∉ `VALID_ENTR_LENGTHS` = {16,20,24,28,32}) flows unvalidated to `from_entropy_in`, which rejects it → `.expect` panics. The `.expect` message *claims* "combine_shares validates entropy length" — it currently doesn't, so the invariant is false.

**Asymmetry (why this is reachable):** the ENCODE path validates length up front — `encode_shares` (shares.rs:107) and `encode` (encode.rs:24) both call `secret.validate()?`, so ms-codec cannot *emit* a non-standard set. But codex32 share strings are an open format; an externally-constructed valid-checksum share set with a non-standard payload length is "real input" the DECODE path must reject gracefully. `dispatch_payload`'s own doc-comment (envelope.rs:155-166: "...then `validate()`") documents the intended behavior the `Entr` arm omits.

**Severity:** panic-not-corruption (no funds loss, no wrong card, no secret leak — the secret never renders; the process aborts). Audit-ranked IMPORTANT.

## 1. The fix (one match arm)

`crates/ms-codec/src/envelope.rs::dispatch_payload`, `Entr` arm (`:169-172`): mirror the `Mnem` arm — build the payload, `validate()?`, then return it:

```rust
RESERVED_PREFIX => {
    // 0x00 → Entr: strip prefix, rest is raw entropy bytes.
    let p = Payload::Entr(data[1..].to_vec());
    // Validate length immediately; rejects non-standard entropy lengths
    // (parity with the Mnem arm + this fn's doc contract). Audit I9.
    p.validate()?;
    p
}
```

This closes the gap for **all** `dispatch_payload` callers (the single-string `discriminate` path at :150 and `combine_shares` at :241; grep confirms exactly these two — no third caller). Effect by path:
- `combine_shares` → returns `Err(Error::PayloadLengthMismatch { got, .. })`, propagated via `?` at combine.rs:58 and rendered by the existing `From<ms_codec::Error>` mapping. The `.expect` at combine.rs:97 is never reached and its stated invariant becomes TRUE.
- single-string `discriminate`/`decode` → a non-standard Entr single-string now also surfaces `PayloadLengthMismatch` (defense-in-depth; the same canonical error the encode side uses). No legitimate input regresses — Entr entropy is always a standard length.

No new error variant (`PayloadLengthMismatch` already exists, returned by `Payload::validate`). The `.expect` comment at combine.rs:97 may be left as-is (now accurate) — optional one-word tweak only. **R0-m1:** `ms-cli decode.rs:92-93` carries the SAME `.expect("ms-codec validates entropy length; ...")` on the `discriminate`→`dispatch_payload` path — its invariant ALSO becomes true after this fix (no extra code; note in CHANGELOG).

## 2. Tests (TDD — write first, see them fail/panic pre-fix)

In `crates/ms-codec/src/shares.rs` `mod tests` (has `use super::*` → access to `HRP`, `non_s_index_pool`, `random_id`, `Codex32String`, `Fe`, `combine_shares`, `dispatch_payload`, `Error`).

**R0-C1 (load-bearing compile fix):** `RESERVED_PREFIX` is **NOT** pulled in by `use super::*` — `shares.rs:13` imports only `{HRP, RESERVED_ID_BLOCKLIST, SHARE_INDEX_V01}` from `crate::consts`. Add `use crate::consts::RESERVED_PREFIX;` inside `mod tests` (or use the literal `0x00u8`). All the OTHER names above ARE in scope. Without this the helper does not compile.

**Test A (end-to-end via `combine_shares` — what the audit asked for):** build a valid-checksum non-standard-length Entr share set DIRECTLY (mirroring `encode_shares`' codex32 construction but WITHOUT `secret.validate()`), then assert `combine_shares` returns `Err`, not panic.

```rust
// payload wire = [RESERVED_PREFIX] || entropy; 17-byte entropy is non-standard.
// inside `mod tests`: `use crate::consts::RESERVED_PREFIX;` (R0-C1)
fn nonstandard_entr_distributed(k: usize, n: usize, entropy_len: usize) -> Vec<String> {
    let mut bytes = vec![RESERVED_PREFIX];
    bytes.extend(std::iter::repeat(0xCDu8).take(entropy_len));
    let id = random_id();
    let secret_s = Codex32String::from_seed(HRP, k, &id, Fe::S, &bytes).unwrap();
    let pool = non_s_index_pool();
    let mut defining = vec![secret_s];
    for pidx in pool.iter().take(k - 1) {
        let filler = vec![0u8; bytes.len()];
        defining.push(Codex32String::from_seed(HRP, k, &id, *pidx, &filler).unwrap());
    }
    let mut out = Vec::new();
    for s in defining.iter().skip(1) { out.push(s.to_string()); }
    for pidx in pool.iter().take(n).skip(k - 1) {
        out.push(Codex32String::interpolate_at(&defining, *pidx).unwrap().to_string());
    }
    out
}

#[test]
fn combine_rejects_nonstandard_entr_length_not_panics() {
    let shares = nonstandard_entr_distributed(2, 2, 17);
    let res = combine_shares(&shares);
    assert!(
        matches!(res, Err(Error::PayloadLengthMismatch { got: 17, .. })),
        "expected PayloadLengthMismatch{{got:17}}, got {res:?}"
    );
}
```

**Test B (unit, pins the exact gap):** `dispatch_payload(&[RESERVED_PREFIX, 0xCD×17])` → `Err(PayloadLengthMismatch { got: 17 })`; and a positive control `dispatch_payload(&[RESERVED_PREFIX, 0xCD×16])` → `Ok(Payload::Entr)` (16 is standard) so the fix doesn't over-reject.

Pre-fix: Test A panics inside `from_entropy_in`? No — `combine_shares` returns the unvalidated `Ok(Entr)` (the panic is in the CLI's `from_entropy_in`, not in `combine_shares`). So pre-fix Test A FAILS the `matches!(Err)` assertion (it gets `Ok`); Test B FAILS (gets `Ok`). Post-fix both pass. (A CLI-level panic test is unnecessary: `combine_shares` returning `Err` is the sole precondition the `.expect` relied on — once it's `Err`, the panic site is unreachable.)

## 3. Verification, scope, release

- `cargo test -p ms-codec` green (new tests pass; nothing else regresses); `cargo test --workspace` green; clippy clean.
- Confirm the only `dispatch_payload` callers are discriminate (:150) + combine_shares (:241) — grep at impl time; if a third exists, re-confirm it tolerates the now-validating Entr arm.
- **SemVer PATCH** — bump **ms-codec only** (0.4.0 → 0.4.1; the one-arm `envelope.rs` change). ms-cli binary is unchanged (no flag/output delta; the `.expect` comment tweak is optional prose) → do NOT auto-bump ms-cli (R0-m4). Add a `### Fixed` entry to **both** the workspace-root `CHANGELOG.md` and `crates/ms-codec/CHANGELOG.md` (R0-m2).
- **No GUI schema_mirror** (no flag/value change). **No manual mirror** (no CLI surface change). **No cross-repo lockstep** — the toolkit's `mnemonic ms-shares combine` delegates to the same `combine_shares`, so it inherits the fix; note it. No new error variant → no decoder-error-variant-parity churn.
