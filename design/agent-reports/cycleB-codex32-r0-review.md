## R0 Review — Cycle-B codex32-vendor SPEC (`SPEC_codex32_vendor_fork_cluster.md`)

**Verdict: GREEN — 0 Critical / 0 Important / 6 Minor. Cleared to begin implementation.**

Reviewed against live source: `mnemonic-secret` @ `6e3ee8e`, `mnemonic-toolkit` @ `684e510`, vendored `codex32-0.1.0` (checksum `d230935f…918e9`, CC0-1.0). Every cited path/line/symbol was re-grepped against current source; the load-bearing invariants all hold.

### The single most load-bearing invariant — WIRE-BYTE-IDENTITY — is genuinely protected

- **Copy boundaries are exact.** `lib.rs` runtime body 1–429 (blank at 429/430, `#[cfg(test)]` at 431, `mod tests` at 432); `field.rs` 1–263 (blank 263, `#[cfg(test)]` at 265); `checksum.rs` 1–191 (no test module). The SPEC's ranges match byte-for-byte. The `#[cfg(test)]` modules and the dev-only `src/bin/correction-table.rs` are correctly excluded (verified not referenced by ms-codec/ms-cli; the `[[bin]]` lives only in the upstream Cargo.toml, which is not vendored).
- **Encoding paths are touched NOWHERE.** P2's only edit is the `Codex32String` derive list + a hand-rolled `Debug` + (implicit) `Drop` via `ZeroizeOnDrop`. `from_seed`/`from_string`/`interpolate_at`/`Parts::data`/`checksum.rs`/`field.rs` are untouched. Verified no `.0` private-field access, no destructuring move-out, no `const`/`static Codex32String`, no `HashSet<Codex32String>` — so adding a `Drop` impl is structurally safe (no Drop-incompatibility).
- **The parity assertion is real and sufficient.** The new `codex32_vendor_parity.rs` pins to PUBLISHED BIP-93 strings (e.g. `from_seed(\"ms\",0,\"leet\",Fe::S,&seed_b)` → `ms10leetsllhdmn9m42vcsamx24zrxgs3qrl7ahwvhw4fnzrhve25gvezzyqqtum9pgv99ycma`, the exact upstream `bip_vector_4` golden, verified present in upstream lib.rs:527-529 AND already pinned independently at `bip93_inline_vectors.rs:118`) + a captured pre-vendor golden set — it pins to the BIP, not to itself. Combined with the existing GREEN corpus (`bip93_inline_vectors` 5+64, `bip93_cross_format`, `spike_kofn`, `codex32_upstream_recovery_regression`) and the re-run-after-P2 discipline, the byte-identity guard is sound. The \"STOP, do not patch\" failure rule is correctly carried.

### Inline (shape A) is the right call and is handled

A is decisively argued: codex32 has exactly one logical consumer (ms-codec's ms1 domain), the forced consumer migration is identical under A and B, A removes the dormant dep outright (the stated intent) where B re-creates it, and the registry-publish constraint is satisfied trivially. CC0 LICENSE vendoring + attribution header are specified. The `ms_codec::Error::Codex32` inner-type-path move is correctly characterized as pre-1.0 breaking → MINOR. codex32 has ZERO dependencies (verified), so nothing transitive to add to ms-codec.

### The ZeroizeOnDrop mechanics are correct (a non-obvious point the SPEC gets right)

Deriving `ZeroizeOnDrop` ALONE on `Codex32String(String)` — without a separate `#[derive(Zeroize)]` — is correct: `zeroize_derive` 1.4.3 generates a `Drop` that calls per-field `zeroize_or_on_drop()`, and `impl Zeroize for String` exists in zeroize 1.8.2 (lib.rs:589). The toolkit already carries `zeroize features=[\"derive\"]`; ms-codec must add it (currently `zeroize = \"1.8\"` bare — verified). The `String::clear`-style truncate-to-zero semantics + reallocation caveat (zeroize lib.rs:195-201) are honestly noted in §2.3. `Clone`/`PartialEq`/`Eq`/`Hash` retention is load-bearing and verified: the `interpolate_at` self-return clone (lib.rs:262) and the M6 `derived != parsed[j]` compare (shares.rs:304) both depend on them; `derived` drops scrubbed each loop iteration with no use-after-drop on `parsed[j]` (borrowed, not moved).

### Every remaining bare-String secret leg is accounted for

All shares.rs citations verified at live line numbers: `secret_s`:141, `defining`:147, `distributed`:159, `parsed`:206/221, recovered `secret`:291 (the recon's DRIFTED-by-10 binding — SPEC uses the corrected :291), M6 compare:304. The `Codex32String`-backed legs auto-scrub once `ZeroizeOnDrop` lands; the `distributed: Vec<String>` return-value residue is HONESTLY documented under the caller-wrap contract (no false GREEN) — correct, since it IS the public return type and cannot be wrapped without an API change. The L22-class `Debug` leak (upstream `#[derive(...Debug)]` at lib.rs:101) is correctly removed and replaced by a length-only redacting impl mirroring ms-codec's Z-DEBUG discipline.

### Lint floor + g6 + SemVer

- Lint: `canonical_list_has_expected_row_count` asserts `n == 4` (message \"expected 4\") at lint_zeroize_discipline.rs:81; 4 rows at 45/50/55/64; the `every_canonical_zeroize_row_has_evidence_anchor` uses `.any()` substring matching. The 4→5 bump + new row (evidence `zeroize::ZeroizeOnDrop`, `impl fmt::Debug for Codex32String` resolving against `src/codex32/mod.rs`) is correct and is a real RED-tripwire.
- g6: this cycle touches NO `mlock.rs` in either repo; `mlock_g6_invariant.rs` byte-compares toolkit `mlock.rs` against the sibling — stays GREEN. The \"NEVER cargo fmt mlock.rs\" rule is carried.
- SemVer: ms-codec 0.6.0→0.7.0, ms-cli 0.11.0→0.12.0, toolkit 0.71.0→0.72.0 all MINOR — defensible (verified current versions). Wire format byte-identical, no `ms1` output change.

### 6-phase decomposition is sound

Each phase is independently R0-able with its own TDD + persisted review. One sequencing note folded below (P4's lint-row evidence anchor depends on P2's impls existing — the SPEC §4.3 already acknowledges this).

### What keeps this GREEN despite 6 Minors

None of the findings touch a load-bearing invariant: wire-byte-identity is protected, every secret leg is scrubbed-or-honestly-documented, the cross-repo rewrite keeps both crates compiling, and all three SemVer bumps are correct. The Minors are citation-accuracy (the off-by-one \"15 vs 16\" friendly.rs count + mislabeled line 94; stale FOLLOWUPS sub-citations) and completeness-of-disclosure (the unmentioned-but-safe edition delta; the \"no widening\" baseline conflation; the slightly-overstated toolkit bump rationale; an over-strict publish order). They should be folded into the SPEC before P1 for a clean audit trail, but per the convention they do not block the start-coding gate. Fold the 6 Minors, re-persist, and proceed to P1.