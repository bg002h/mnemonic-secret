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

(none yet — repo just initialized 2026-05-03)

---

## Resolved items

(none yet)
