# PLAN — fix `decode_with_correction` char-boundary panic (ms-codec 0.4.3)

Status: DRAFT (pre-mini-R0). 2026-06-12. Repo: mnemonic-secret @ 493c5de (master).
Found by: stress Cycle C phase-2 fuzzing (`ms1_decode` target) — FOLLOWUP
`decode-with-correction-panics-on-non-char-boundary-hrp-slice`.

## The bug (grep-verified at 493c5de)

`crates/ms-codec/src/decode.rs:145-153` — `parse_ms1_symbols`:

```rust
if !lower.starts_with(HRP_PREFIX) {
    let hrp_end = lower.rfind('1').map(|i| i + 1).unwrap_or(lower.len());
    let got = lower[..hrp_end.saturating_sub(1)].to_string();   // ← PANIC
    return Err(Error::WrongHrp { got });
}
```

When the input has NO `'1'`: `hrp_end = lower.len()`, so `got =
lower[..len-1]`. If `len-1` lands inside a multi-byte char (e.g. the input
is non-ASCII / the fuzz target's `String::from_utf8_lossy(&[0xaa])` =
`"\u{FFFD}"`, 3 bytes), the slice is not on a char boundary → **panic**
("end byte index 2 is not a char boundary; it is inside '�'").

Reached only via `decode_with_correction` (decode.rs:221) → `parse_ms1_symbols`.
`decode` (length-gated first) and `inspect` return clean `Err` on the same
input — verified. So this is the never-panic-charter class (a panic `ms
repair` / the indel oracle inherit), NOT a secret leak, NOT corruption.

## The fix (char-boundary-safe, behavior-preserving where it matters)

```rust
if !lower.starts_with(HRP_PREFIX) {
    // Report the observed HRP (everything before the last '1' separator)
    // so the error is actionable. '1' is ASCII, so `rfind('1')` returns a
    // char boundary; slicing there is always safe. When there is NO
    // separator the whole (malformed) string is the observed HRP — never
    // slice at `len-1`, which can land inside a multi-byte char and panic.
    let got = match lower.rfind('1') {
        Some(i) => lower[..i].to_string(),
        None => lower.clone(),
    };
    return Err(Error::WrongHrp { got });
}
```

**Behavior delta:**
- WITH a `'1'` at byte `i`: old `got = lower[..hrp_end-1] = lower[..i]`;
  new `got = lower[..i]`. **Byte-identical** — no semantic change.
- WITHOUT a `'1'`: old = `lower[..len-1]` (panics on a non-char-boundary
  tail, else chops the last byte — meaningless); new = `lower.clone()`
  (the whole observed string). **No panic; a more honest `got`.**

**Interaction with `ms-codec-error-display-echoes-input` (the open leak
FOLLOWUP):** that FOLLOWUP will bound/withhold `WrongHrp.got` (and the
codex32 echo) at the Display boundary. This fix does NOT expand the leak
surface in any way that matters: the WITH-`'1'` path (the only one a real
ms1-ish input takes) is unchanged; the no-`'1'` path is reached only by
inputs with no separator at all (not a structured ms1 share). The two
fixes are orthogonal — this one is char-safety, that one is echo-bounding.
The Cycle-C `ms1_no_secret_leak` fuzz target's exclusion set already covers
`WrongHrp{..}`, so this change does not affect that oracle.

## TDD

RED-first regression tests in `crates/ms-codec/src/decode.rs` `#[cfg(test)]`:
1. `decode_with_correction("\u{FFFD}")` returns `Err`, does NOT panic
   (the minimized fuzz reproducer; `0xaa` → lossy → U+FFFD).
2. A small table of no-`'1'`, multi-byte inputs at assorted lengths
   (so `len-1` lands inside a char at different offsets): e.g.
   `"\u{FFFD}"`, `"é"`, `"añ"`, `"\u{1F600}"` (4-byte), each → clean `Err`
   (no panic). Assert the variant is `WrongHrp` and `got` == the input
   (lowercased) for the no-`'1'` arm.
3. Preservation: an input WITH a `'1'` but wrong HRP (e.g. `"xy1qqq"`)
   still yields `WrongHrp { got: "xy" }` (byte-identical to pre-fix).
4. (Optional belt) the existing `ms1_decode` fuzz target re-run locally
   no longer crashes on the minimized input — but the unit tests are the
   durable gate.

Run RED against current source (test 1/2 panic → test failure), apply the
fix, confirm GREEN, then full `cargo test -p ms-codec` + `-p ms-cli`.

## Release mechanics

- **ms-codec** `Cargo.toml` version `0.4.2 → 0.4.3` (patch — bugfix, no API
  change). `crates/ms-codec/CHANGELOG.md` + root `CHANGELOG.md` gain a
  `[0.4.3]` entry (char-boundary panic fix; credit the fuzz harness).
- **ms-cli** `Cargo.toml` exact pin `ms-codec = "…version = "=0.4.2""` →
  `"=0.4.3"` (the established lockstep — a published ms-codec bump forces
  ms-cli's exact pin). ms-cli's OWN version: unchanged (no ms-cli surface
  change) unless the exact-pin bump is itself considered a release-worthy
  dep change — mini-R0 to confirm (prior cycles bumped ms-codec without
  always bumping ms-cli's version; check the `=0.4.0→=0.4.1` precedent).
- **Cargo.lock** at root updates for the new ms-codec version.
- **Re-enable the held-out fuzz target:** mnemonic-secret
  `.github/workflows/fuzz-smoke.yml` smoke matrix — remove the `ms1_decode`
  hold-out comment + add `ms1_decode` back to the matrix (it is now the
  regression gate). The fuzz crate path-deps ms-codec, so it builds the
  fixed code automatically.
- **Resolve the FOLLOWUP:** mark `decode-with-correction-panics-on-non-char-boundary-hrp-slice`
  resolved @ 0.4.3, noting the publish is pending.

## DEFERRED — needs user authorization (do NOT do autonomously)

- `cargo publish` of ms-codec 0.4.3 (and the forced ms-cli exact-pin
  republish if ms-cli is also published) to crates.io — irreversible,
  public. Prior ms-codec publishes were explicitly user-authorized.
- The downstream **toolkit pin bump** to ms-codec 0.4.3 (so `mnemonic` /
  `ms repair` get the fix) only matters AFTER the publish.
- This plan lands the fix + bump + ms-cli pin + fuzz re-enable IN-REPO with
  CI green; the publish + toolkit propagation are a separate authorized
  step. Flag this clearly in the ship report.

## Open questions for mini-R0

1. The `None => lower.clone()` arm — is whole-string the right `got`, or
   should it be empty / a bounded prefix (given the leak FOLLOWUP)? Argue
   minimality vs not-worsening-the-echo.
2. ms-cli version: bump alongside the exact-pin, or pin-only (per the
   `=0.4.0→=0.4.1` precedent where ms-cli's version was unchanged)?
3. Is there any OTHER non-char-boundary slice in the ms-codec decode hot
   paths (sweep `[..` / `[i..]` slices on `to_ascii_lowercase`/lossy-utf8
   strings) the fuzzer hasn't hit yet but the same class would? If so,
   fold them or file them.
4. Does `decode` truly never reach this (length gate ordering) for ALL
   no-`'1'` multi-byte inputs, or only the ones tested? Confirm the gate
   is unconditional.
