# R0 Review — ms-codec error no-echo fix (round 2)

Reviewer: Fable 5 architect agent (a3d15516fd4f5e8bd), 2026-06-12.
Target: design/BRAINSTORM_error_display_no_echo.md (R1 fold) @ mnemonic-secret master.
Persisted verbatim per CLAUDE.md convention.

## Verdict: GREEN

The round-1 Important findings (I1 construction-time char-bound, I2 enshrining-test update) are both folded correctly and completely; all minors (M1–M7) reflected. Every load-bearing claim verified against current master + codex32-0.1.0's actual variant shapes. The subtle round-2 risk — a consumer relying on the DERIVED Debug shape (variant names) breaking under a Display-delegating custom Debug — was checked specifically and does NOT exist: every `{e:?}` use is either a `matches!`-gated panic message or the fuzz oracle itself (asserts ABSENCE of secret windows, not presence of variant names). 0 Critical / 0 Important.

## Critical
- none

## Important
- none

## Minor
- **[m1] "Both CHANGELOGs" is imprecise — there is NO `crates/ms-cli/CHANGELOG.md`.** The repo has workspace-root `CHANGELOG.md` (crate-prefixed `## ms-codec [x]` / `## ms-cli [x]`) + per-crate `crates/ms-codec/CHANGELOG.md`. ms-cli is pin-only (no version bump per the 0.4.3 precedent) → no `## ms-cli` entry. "Both CHANGELOGs" = root + `crates/ms-codec/CHANGELOG.md`.
- **[m2] A SECOND ms-cli codex32 renderer echoes HRP/ID strings, untouched by the fold — but NOT secret-data.** `crates/ms-cli/src/codex32_friendly.rs:44,48` `{:?}`-prints `MismatchedHrp`/`MismatchedId` String fields. Provenance-bounded (HRP="ms", id=4 chars, from `interpolate_at` on valid Codex32String — never the data-part); the fuzz oracle scans shares' data-part windows so it won't fire. Note in the CHANGELOG that ms-cli's `friendly_codex32` keeps HRP/ID by design. Optional defense-in-depth (trivial); not in this cycle's scoped `ms_codec::Error` goal.
- **[m3] Custom-Debug-delegates-to-Display loses structural diagnostics in panic/fuzz failure messages** (fuzz line 117 `variant={e:?}`, shares.rs:478/527). Harmless (matches!-gated, cosmetic). Implementer may prefer a structural Debug (variant name + safe fields) over pure Display delegation; the fold's "may delegate" permits both. Discretion.

## Fold verification table
| Round-1 finding | Resolved? | Notes |
|---|---|---|
| **[I1]** WrongHrp bound at CONSTRUCTION (3 sites), rendering-only rejected | YES | Fold §3 HARD requirement, rejects rendering-only, names 3 sites. Downstream re-echo verified: ms-cli error.rs:137-141 (msg + details.got) + toolkit friendly.rs:95-96 read got by value → construction-bound is the only fix. Sites: decode.rs:158-162, envelope.rs:65, envelope.rs:121-123. |
| **[I2]** Update enshrining test decode.rs:388-401; char-counted cap | YES | Fold targets the exact test, replacement `s.chars().take(4).collect::<String>().to_ascii_lowercase()`, mandates char-counted. `"é".repeat(25)` is the only got>4 assertion; `.chars().take(4)` char-safe → "éééé" (4<8). xy/ñ/xs/mq survive. |
| **[M1]** codex32 no Display → manual match mandatory | YES | Verified: codex32-0.1.0 Error is derive(Debug) only, no Display, no std::error::Error. |
| **[M2]** 16 variants, 3 leaky explicit + safe wildcard | YES | 3 (InvalidChecksum.string/MismatchedHrp/MismatchedId) explicit; 13 safe via fields; field::Error (ExtraChar/NotAByte/InvalidByte) SAFE; Fe/Case renderable. |
| **[M3]** Custom Debug kept (load-bearing for toolkit transitive Debug) | YES | ToolkitError error.rs:8 derive(Debug) wraps MsCodec(ms_codec::Error). NO test relies on the derived Debug variant-name shape. |

## Evidence log
```
error.rs: Display Codex32 "{:?}" (118), WrongHrp "{:?}" got (122), derive(Debug) (7) — 3 leak surfaces confirmed.
WrongHrp construction sites verified: decode.rs:158-162 (got from lower); envelope.rs:65 (s.to_string()); envelope.rs:121-123 (fields.hrp.to_string()). All post-lowercase → .chars().take(4) correct.
Enshrining test decode.rs:388-401 got==full lowercased for "é".repeat(25)=50B → breaks under cap. NO other got>4 test.
codex32-0.1.0 Error=16; LEAKY=3 (InvalidChecksum{checksum:&'static,string:String}, MismatchedHrp(String,String), MismatchedId(String,String)); checksum field &'static safe; field::Error SAFE; manual match compile-feasible.
Downstream I1: ms-cli error.rs:137-141 + toolkit friendly.rs:95-96 re-echo got → fixed free by construction-bound (pin-only).
2nd leak (m2): ms-cli codex32_friendly.rs:44,48 MismatchedHrp/Id {:?} — HRP/ID not data-part; provenance-bounded; oracle won't fire.
Debug-shape reliance: NONE. shares.rs:478/527 matches!-gated {err:?} cosmetic; fuzz ms1_no_secret_leak.rs:127 {e:?} is object-under-test (asserts no ≥8 window); ms-cli error.rs:246 _-arm {:?} non-leaking sink. Display-delegating Debug breaks nothing.
Release: ms-codec 0.4.3→0.4.4; ms-cli pin =0.4.3→=0.4.4 (ms-cli 0.7.0 unchanged); toolkit 0.4.3→0.4.4. CHANGELOGs: root + crates/ms-codec/CHANGELOG.md (no ms-cli changelog).
Fuzz exclusion: is_known_echo Codex32(_) (80) + WrongHrp{..} (84), call 109, WINDOW=8 (47) — delete both arms.
Tree left as found.
```

GREEN at 0C/0I — implementation may begin. The 3 Minors are doc/discretion items; recommend folding m1 (CHANGELOG targets) + m2 (ms-cli HRP/ID note) at implementation time.
