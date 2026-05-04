# Phase 5 (Release Prep) — Opus Review r1

**Date:** 2026-05-04
**Reviewer:** feature-dev:code-reviewer (opus)
**Commits reviewed:** `f442daf` (Cargo.toml v0.1.0 + metadata) + `9cfe01c` (README + CHANGELOG)
**Tests:** 77 pass; clippy --all-targets -D warnings + fmt --check clean.

## Verdict

**Zero critical, zero important findings.** Phase 5 is ready to mark complete and proceed to user-gated `cargo publish -p ms-codec` followed by `cargo publish -p ms-cli` + tag push.

## Critical

None.

## Important

None.

## Low / Nit

- **L1.** CHANGELOG.md line 3 header: "All notable changes to `ms-codec` (and future `ms-cli`)" — parenthetical "(and future)" stale now that ms-cli [0.1.0] is the entry directly below. **APPLIED INLINE post-r1.**
- **L2.** ms-cli/README.md cites SPEC §6.3 for the language hazard; section number not independently dereferenced. Trivial doc audit; non-blocking.
- **L3.** ms-cli/README.md lacks a "Family" section listing md-codec / mk-codec / ms-codec. Optional; README already points to ms-codec which itself links the family.
- **L4.** ms-cli/Cargo.toml had `codex32 = { workspace = true }` in BOTH `[dependencies]` and `[dev-dependencies]`. Harmless but redundant. **APPLIED INLINE post-r1** — removed the dev-deps duplicate.

## Affirmations

- **Cargo.toml metadata complete and parallel to ms-codec.** All publish fields present: `description`, `documentation`, `readme`, `keywords` (5, max), `categories` (2 — adds `command-line-utilities` over ms-codec's library-only). `version = "0.1.0"` no -dev. `publish = false` removed.
- **`[[bin]] name = "ms"` preserved** so `cargo install ms-cli` produces the documented `ms` binary.
- **Pinned dep on ms-codec** (`ms-codec = { path = "../ms-codec", version = "=0.1.0" }`) correct for v0.1 lockstep.
- **README.md hits all required beats:** install, quickstart, engraving-language hazard with SPEC pointer, ms-codec pointer, CC0 license. Mirrors ms-codec/README.md shape.
- **CHANGELOG ordering correct:** ms-cli [0.1.0] (2026-05-04) above ms-codec [0.1.0] (2026-05-03) per per-crate-prefix convention.
- **CHANGELOG content** covers all required items: 5 subcommands; phrase-first encode + --json; strip-whitespace stdin (with doubling-detection); language enforcement; exit codes 0/1/2/3/4/64; engraving-friendly multi-line stdout; verify --phrase round-trip + secrets discipline; 77 tests; build/clippy/fmt clean.
- **`--help` text matches SPEC §2.6 verbatim.** Cross-checked main.rs:24-66 against SPEC §2.6 character-for-character.
- **`cargo publish -p ms-cli` will succeed** post-ms-codec-publish: no `path = ".."` without `version = `; no `publish = false` lurking; no missing required metadata.
- **bip39 features = ["all-languages"]** retained in [dependencies] — not silently dropped.

## User-gated next steps (post-Phase-5)

1. `cargo login` (interactive; paste crates.io API token).
2. `cargo publish -p ms-codec` (requires ms-codec to be the first publish since ms-cli depends on it via `=0.1.0` exact-pin).
3. `cargo publish -p ms-cli` (resolves `ms-codec` from crates.io after step 2).
4. `git push origin ms-cli-v0.1.0` (push the locally-tagged release).
5. `gh release create ms-cli-v0.1.0 --notes-file <changelog excerpt>` (create the GitHub Release).
