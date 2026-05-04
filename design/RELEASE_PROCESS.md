# Release process

Mirrors the conventions of the sibling `descriptor-mnemonic` and `mnemonic-key` repos.

## Per-release checklist

1. **Wire-format SHA pin.** Hash the canonical test-vector corpus (`crates/ms-codec/tests/vectors/v0.1.json` for v0.1) with SHA-256 and record it in `CHANGELOG.md` under the release entry. Any subsequent change to the corpus that would alter the SHA requires a SemVer minor bump (`0.X+1.0`) per the pre-1.0 breaking-change axis convention.

2. **CHANGELOG entry.** Per-crate prefix in the section header (`## ms-codec [0.1.0]`). Categorize changes under "What's new" / "What didn't change" / "Migration notes" sections per the md-codec / md-cli precedent.

3. **CI gate green.** `cargo build`, `cargo test`, `cargo clippy --all-targets -D warnings`, `cargo fmt --check` across the three-row CI matrix (stable + beta + MSRV `1.85`).

4. **Convergence on per-phase reviews.** Each implementation phase has a reviewer-loop record (the disk-persisted reports under `design/agent-reports/`); the release tag should not land while any phase has open critical or important findings.

5. **MIGRATION.md update.** If the release introduces any wire-format or API change relative to the previous minor, add a new section to `MIGRATION.md` per the v0.1 → v0.2 precedent.

6. **Cross-repo notification.** If the release affects sibling repos (md1, mk1, future mnemonic-toolkit), add or update entries in `design/FOLLOWUPS.md` at tier `cross-repo` with companion entries in the affected siblings.

7. **`cargo publish --dry-run`** must pass for `ms-codec` before tag. (`ms-cli` is `publish = false` until its own v0.1 ships.)

8. **Tag and push.** `ms-codec-v0.X.Y` tag at the release commit; push tag to origin.

## Versioning policy

- `ms-codec` follows SemVer with the pre-1.0 breaking-change axis convention: `0.X+1.0` is breaking; `0.X.Y+1` is non-breaking.
- The `Tag` constant set (`Tag::SEED`, `Tag::ENTR`, `Tag::XPRV`, …) is SemVer-stable from v0.1.0. Adding tags is minor; removing or renaming is major.
- `Payload` and `InspectReport` are `#[non_exhaustive]` from v0.1.0 — adding variants/fields is minor; removing the attribute would be major.
- MSRV bumps follow md-codec's MSRV in lockstep, never lead.
