# Follow-up tracker

Single source of truth for items that surfaced during a review or implementation pass but were not fixed in the same commit. Mirrors the conventions of the sibling `descriptor-mnemonic` and `mnemonic-key` repos.

## How to use this file

**Format for each entry:**

```markdown
### `<short-id>` — <one-line title>

- **Surfaced:** Phase X.Y review of commit <SHA>, or "inline TODO at <file>:<line>"
- **Where:** `<file>:<line>` or "design — Cargo.toml `[patch]` block"
- **What:** 1–3 sentences describing the gap or improvement opportunity
- **Why deferred:** the reason it didn't ship in the original commit
- **Status:** `open` | `resolved <COMMIT>` | `wont-fix — <one-line reason>`
- **Tier:** `v0.1-blocker` | `v0.1-nice-to-have` | `v0.2` | `cross-repo` | `v1+` | `external`
```

## Tiers (definitions)

- **`v0.1-blocker`**: must fix before tagging `ms-codec-v0.1.0`. Failing to fix = ship blocked.
- **`v0.1-nice-to-have`**: should fix before v0.1 if time permits, but won't block release.
- **`v0.2`**: explicitly deferred to v0.2 (e.g., K-of-N share encoding work).
- **`cross-repo`**: depends on coordination with sibling repos (`descriptor-mnemonic`, `mnemonic-key`, future `mnemonic-toolkit`). Mirrored by a companion entry in the affected sibling's tracker.
- **`v1+`**: deferred indefinitely.
- **`external`**: depends on work outside this repo (e.g., upstream `rust-codex32` PR merging).

---

## Open items

### `ms1-v01-payload-bracket-overflow-prefix-byte-incompatibility` — v0.1 `0x00`-prefix-byte design overflows BIP-93 codex32's long-code length bracket for `seed` / `xprv` payloads

- **Surfaced:** 2026-05-03 pre-SPEC spike against `rust-codex32 = "=0.1.0"` (in conversation; before SPEC drafted). Companion mirrors: same-id entry in `mnemonic-key/design/FOLLOWUPS.md` and `descriptor-mnemonic/design/FOLLOWUPS.md`, both at tier `cross-repo`.
- **Where:** SPEC (not yet drafted), `BRAINSTORM_ms_v0_1.md` Q4 closure (locks `seed`/`entr`/`xprv` payload set), `MIGRATION.md` invariant 1 (locks the `0x00` reserved-prefix byte), and the meta-plan `/home/bcg/.claude/plans/c-ultimately-what-we-quirky-avalanche.md` §"ms-codec v0.1 architecture" / §"v0.2 migration seam" / §"RESERVED_TAG_TABLE".
- **What:** BIP-93 codex32 (per the BIP itself, and as implemented in `rust-codex32 = "=0.1.0"`) accepts only two specific length brackets — short (raw payload 16-44 B) and long (raw payload 63-64 B). The locked v0.1 wire format prepends a `0x00` reserved-prefix byte to the raw secret to enable the v0.2 non-breaking migration; this pushes a 64-B BIP-32 master seed to a 65-B effective payload (128-char string, one past the long-bracket max of 127). Empirical spike (encode→decode against `rust-codex32 v0.1.0` over data sizes 60..82) confirmed: encoder produces a string the decoder rejects with `InvalidLength` for every size outside {16-44, 63-64} bytes. `xprv` (78 B) was never inside any BIP-93 bracket, with or without the prefix. Three locked decisions interact (payload set {seed, entr, xprv} + `0x00` reserved-prefix byte + exact-pin `=0.1.0` no-fork), but at most two are simultaneously satisfiable.
- **Why deferred:** Surfaces SPEC-blocker *before* the SPEC is drafted; cannot be deferred. Logged here so future sessions / sibling-repo readers see the discovery provenance once a remediation lands. Active candidates (in conversation): (A) drop `seed`/`xprv`; v0.1 = `entr` only — strongest fit given the engraving thesis. (B) drop the `0x00` prefix; v0.1 uses `id` as sole discriminator and the v0.2 migration loses the non-breaking-for-v0.1-strings property. (C) vendor/fork `rust-codex32` with a wider long-code — requires re-deriving BCH parameters, much heavier than originally framed.
- **Workflow lesson:** the plan-mode r1..r5 reviewer loop did logical/architectural review without an execute-encode/decode-against-locked-deps spike. Five rounds missed the issue. Future wire-format plans riding on locked external deps should include an explicit "verify round-trip against the actual pinned dep before locking the plan" step, parallel to the existing `audit_before_extending` memory entry.
- **Status:** `open` — awaiting user direction on remediation (A / B / C / other).
- **Tier:** `v0.1-blocker`

---

## Resolved items

(none yet)
