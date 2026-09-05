# Decision: publish ms-codec 0.8.0 to crates.io now?

**Decision: YES — run `cargo publish -p ms-codec --locked` now, from the tag's tree.**

Stand-in architect (fable), 2026-09-05, at the controller's request. Scope is the
one outward-facing irreversible action; nothing here audits the code.

## Grounds

1. **The registry adds no commitment the release has not already made.** The
   annotated tag `ms-codec-v0.8.0` and `ms-cli-v0.18.0` sit on `cd0a60f`,
   pushed through the protected-branch ritual, with the `man-release` binaries
   on the GitHub release. Every derivation constant (`ms-hashlock-v1`, 100,000
   iterations, dkLen 32), the `0x03`/`hash` wire rule and the corpus SHA are
   already frozen by the shipped `ms` 0.18.0 binaries and by
   `tests/hashlock_derivation.rs` / `tests/hashlock_repro.rs`, which pin them
   by value. Changing any of them after 0.18.0 would be a wire break whether or
   not crates.io has the crate. The registry only makes the *version number*
   unreusable, which the tag already pledged.

2. **The API questions named in the brief are settled, not open.**
   - `TagKindMismatch` (refuse the mismatch) is OPERATOR RULING L24, "kept",
     labelled in place in `design/SPEC_ms_hashlock.md` (§1 rule 2, line ~119).
   - The `hashlock` module surface (`HASHLOCK_SALT`, `HASHLOCK_ITERATIONS`,
     `HASHLOCK_DKLEN`, `preimage_hardened`, `preimage_sha256`,
     `preimage_random`, `digest`) is exactly what the released ms-cli 0.18.0
     consumes (`crates/ms-cli/src/cmd/hashlock.rs:24`) and what MIGRATION.md
     v0.8 documents. Nothing in `design/FOLLOWUPS.md` with a `hashlock` tag is
     open against ms-codec; the three open items (inspect advisory, stdin
     echo, toolkit manual chapter) are ms-cli or toolkit only.
   - `InspectKind` NOT being `#[non_exhaustive]` is a deliberate, spec-stated
     choice ("loud, therefore safe" — SPEC §3 line ~258, MIGRATION v0.8 item
     3, CHANGELOG migration note). It is a reason to publish, not to wait:
     downstreams get a compile error at their bump instead of a silent
     catch-all. If a later stage wants it changed, that is a 0.9.0 under 0.x
     semver, which holding 0.8.0 back would not avoid.

3. **Two downstream consumers need 0.8.0 resolvable from the registry, and
   nothing else works for them.** `me` pins `ms-codec = "0.7"`
   (`mnemonic-engrave/crates/me-cli/Cargo.toml:53`) and H1b / F-473 is
   scheduled to bump it; the toolkit pins `ms-codec = "0.7"`
   (`mnemonic-toolkit/crates/mnemonic-toolkit/Cargo.toml:32`). Both build
   `--offline --locked` from a committed `vendor/` with crates.io source
   replacement, so a git/path dependency is not a substitute. The publish is
   on H1b's critical path; not publishing delays H1b for no gain.

4. **The device flash is irrelevant to this action.** L26 already ruled the
   *release* does not wait for a measured boot. What puts preimage strings in
   operators' hands is the released 0.18.0 binary, not the library on
   crates.io; a device still on 839fa5aa mis-cuts a preimage plate either way.
   Publishing the crate is a strictly smaller exposure than the release that
   has already happened.

5. **House precedent is publish-at-tag.** ms-codec 0.3.0, 0.4.3, 0.4.4 and
   0.7.0 were all published at their tags (`design/FOLLOWUPS.md` lines 590,
   599, 612; crates.io shows 0.7.0 on 2026-06-23), and
   `design/IMPLEMENTATION_PLAN_ms_v0_1.md:2607` lists `cargo publish -p
   ms-codec` as the release step. `design/RELEASE_PROCESS.md` naming only the
   dry-run (line 19) is a documentation gap, not a policy against publishing.

6. **The package is what it should be, and it is not already up.** Measured in
   this session: `cargo package -p ms-codec --list --offline --locked` includes
   `README.md`, `CHANGELOG.md`, `src/hashlock.rs`, and
   `tests/vectors/hashlock-v0.8.json`; `git diff --stat ms-codec-v0.8.0 HEAD
   -- crates/ms-codec Cargo.lock Cargo.toml` is empty (HEAD `1990648` is one
   reports-only commit past the tag); the working tree is clean; the crates.io
   API reports `max_version 0.7.0` (so no double-publish and no prior partial
   attempt).

## Conditions

- **Publish from the tag's tree** so `.cargo_vcs_info.json` records `cd0a60f`
  rather than the reports commit: `git checkout --detach ms-codec-v0.8.0`
  (or a worktree — not under `/tmp`, it is tmpfs), then
  `cargo publish -p ms-codec --locked`, then return to `master`. The crate
  bytes are identical either way (measured above); this is provenance
  hygiene, not a correctness gate, so if the checkout is inconvenient
  publishing from HEAD is acceptable.
- No `--allow-dirty`, no version bump, no edits of any kind before the
  publish. This decision is a publish of the tagged content only.
- **ms-codec only.** ms-cli 0.18.0 is out of scope and, by its own
  `Cargo.toml` (line 21: the shared IO mechanism is a git-rev dependency),
  is not publishable to crates.io as it stands.
- After it returns: confirm `max_version 0.8.0` via
  `https://crates.io/api/v1/crates/ms-codec`, record the publish (SHA,
  time, verified max_version) in the H1 acceptance/release record, and file a
  Minor follow-up to add the real publish step to
  `design/RELEASE_PROCESS.md` (after the tags: publish from the tag checkout,
  verify max_version). Neither is a precondition.

## What would have made it NO (none present)

An open ruling on any 0.8 API item; an open ms-codec follow-up that would
change the `hashlock` surface, the accept set, or the tag/kind rule before
H1b; a corpus SHA that did not match the CHANGELOG pin; the crate tree at HEAD
differing from the tag; 0.8.0 already on the registry from an earlier attempt.

## What I read (read-only)

- `design/RELEASE_PROCESS.md` (publish mentions), `design/SPEC_ms_hashlock.md`
  (rulings L24-L27, §54-62, §255-280, publish grep),
  `design/IMPLEMENTATION_PLAN_ms_hashlock_H1.md` §3765-3800,
  `design/FOLLOWUPS.md` (hashlock/publish grep), `MIGRATION.md` v0.8,
  `CHANGELOG.md` 0.8.0 section.
- `crates/ms-codec/Cargo.toml`, `crates/ms-codec/src/inspect.rs:1-35`,
  `src/hashlock.rs` pub items, `#[non_exhaustive]` / `TagKindMismatch` sites,
  `crates/ms-cli/Cargo.toml:9-22`.
- `git log`/`git tag --points-at cd0a60f`, `git diff --stat` tag vs HEAD,
  `git status`, `cargo package --list`, the crates.io API for `ms-codec`,
  `mnemonic-engrave/crates/me-cli/Cargo.toml`, F-473 in
  `mnemonic-engrave/design/FOLLOWUPS.md`, the toolkit's `ms-codec` pin, and a
  constellation-wide grep for the hashlock constants.
- No `.jsonl` file was opened. Nothing was modified other than creating this file.
