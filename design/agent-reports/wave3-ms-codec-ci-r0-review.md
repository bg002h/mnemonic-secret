## R0 Review — Wave-3 lane W3-7 (ms-codec CI + 1.95.0 fmt normalization, `mnemonic-secret`)

**VERDICT: GREEN — 0 Critical / 0 Important / 4 Minor. Cleared to implement.**

Repo verified at HEAD `9d3d24ba` (matches the spec header), branch `master`, clean tree. Every load-bearing claim was reproduced against current source, including a full throwaway-worktree dry-run of both commits.

### What I verified (all PASS)

**File scope (§0, §1.3) — exact.** `RUSTUP_TOOLCHAIN=1.95.0 cargo fmt --all -- --check` reports **44** `Diff in` files = **43 non-mlock + `crates/ms-cli/src/mlock.rs`**. Per-directory breakdown is byte-for-byte the spec's list: ms-codec src 8, ms-codec tests 7, ms-cli src 10, ms-cli tests 18. The recon's '~16 files / ms-cli already clean' was indeed wrong; the spec's correction (43 non-mlock, ms-cli NOT clean) is empirically right. ms-codec has 19 test files total; only 7 need reformatting (the other 12 are already 1.95.0-clean) — staging only the 7 changed is correct.

**rustfmt equivalence (§0) — confirmed.** `cargo +1.95.0 fmt --version` and `cargo +stable fmt --version` BOTH = `rustfmt 1.9.0-stable (59807616e1 2026-04-14)`. `rust-toolchain.toml` pins 1.85.0, so the explicit `+1.95.0` override is required — matches the toolkit's resolved `toolkit-rustfmt-1-95-0-rebaseline-divergence` ruling.

**Procedure (§1.2) — dry-run PASS.** In a throwaway worktree: `cargo fmt --all` + `git checkout -- crates/ms-cli/src/mlock.rs` ⇒ exactly 43 files modified, mlock.rs UNMODIFIED. `git status --porcelain crates/ms-cli/src/mlock.rs` empty. Both crates build+test GREEN (ms-codec 81+ tests; ms-cli all green). `ms gui-schema` output SHA **identical** before/after (`2fadbd3e…`) — proving the GUI schema-mirror wire-shape is untouched. `consts.rs` diff is a pure array line-wrap (constant bytes unchanged). Wire-format invariant test `encode_bytes_are_stable_and_decode_round_trips` passes in the formatted tree.

**mlock carve-out (§1.5) — PROVEN load-bearing.** Normalizing a formatted copy of mlock.rs yields a DIFFERENT byte string than both current ms mlock.rs and toolkit-master mlock.rs (Python repro of `normalize()`); `ms FMT norm == toolkit norm: False` vs `ms orig norm == toolkit norm: True`. So reformatting mlock.rs ⇒ g6 RED. With the revert, the g6 job stays GREEN: ran `SIBLING_REPO_PATH=…toolkit cargo test -p ms-cli --test mlock_g6_invariant -- --include-ignored` in the formatted worktree → **2 passed**. (Verified toolkit working-copy mlock == origin/master mlock, so the local repro faithfully mirrors CI's `toolkit@master` checkout.)

### CI-gate verification (incl. CI-only)

- **fmt gate (NEW, §2.1):** Copied verbatim from toolkit `rust.yml`'s fmt job (confirmed line-equal, only `@v6→@v4` differs — the spec's `@v4` choice matches this repo's existing 6 checkout uses, internally consistent). Ran the EXACT gate body against the post-commit-1 tree → `bad` empty = GREEN; the residual mlock.rs diff is correctly excluded by `grep -v '/mlock\.rs$'`. rustfmt 1.9.0 emits one `Diff in <abspath>:<line>:` header per hunk; the `grep -oE '^Diff in [^:]+'` capture works as designed (gate only checks non-empty, so the hunk-vs-file count difference is irrelevant). Ordering commit1→commit2 is correctly mandated (otherwise the gate is born RED).
- **Path-filter fix (§2.0) — REQUIRED and correct.** Current `paths:` (push 20-24, PR 26-30) lists only `crates/ms-cli/**` + Cargo.{toml,lock} + this workflow. A push touching only `crates/ms-codec/src/*.rs` matches nothing ⇒ the WHOLE workflow skips ⇒ new ms-codec jobs would never fire. Adding `crates/ms-codec/**` to both lists is load-bearing. (Same exact line numbers confirmed: 18/20/21 and 25/26/27.)
- **g6-invariant (CI-ONLY) — correctly handled.** Local repro is the `SIBLING_REPO_PATH=… --include-ignored` step (§4); the CI command at `rust.yml:184` matches the spec's repro command verbatim. Baseline 2-passed must stay 2-passed after commit 1 — confirmed.
- **sibling-pin-check — NOT triggered.** Lives only in the toolkit; scans workflow `cargo install --git … --tag` lines vs `scripts/install.sh`. This lane's new jobs use `dtolnay/rust-toolchain` + bare `cargo test/clippy` — no `--tag` line added. Out of scope, correctly excluded.
- **GUI schema-mirror — NOT triggered.** No such gate runs in mnemonic-secret CI; it lives in mnemonic-gui and fires on GUI ms-cli pin bumps. NO-BUMP ⇒ no re-pin ⇒ no fire. Belt-and-suspenders: `ms gui-schema` output proven byte-identical. `gui_schema.rs` is not even in the fmt diff.
- **manual lint — N/A.** No `docs/manual/` in this repo (toolkit-only); no CLI-surface delta.
- **No job-name collisions:** existing `test / test-release-mlock-einval / miri / clippy / g6-invariant` vs new `fmt / test-ms-codec / clippy-ms-codec` — disjoint. The existing `clippy` job's `conformance-vector checksum pin` step (consumes `design/display-grouping-vectors.tsv.sha256`, present) is untouched.

### Scope / SemVer

- **NO-BUMP justified.** ms-codec 0.6.0 + ms-cli 0.11.0 confirmed; no public API/CLI/wire change (pure whitespace + CI). No README/version-site mirrors carry `0.6.0`/`0.11.0` to update. ms-codec has no `[features]` (so `clippy -p ms-codec --all-targets -- -D warnings` with NO `--all-features` is correct; ran it locally → exit 0).
- **No scope creep.** Repo has ONLY `ms-codec` + `ms-cli` (no md crate) — the 'md-leg excluded' concern is structurally moot here; md lives in `descriptor-mnemonic`. No `export-wallet`/W3-4/W3-5 surfaces exist in this repo. The spec explicitly leaves `mlock-rs-fmt-exempt` / `mlock-g4-a-page-count-assert-flake` deferred to the next ms-cli g6-pin tag (§5), consistent with the FOLLOWUPS architect ruling.
- **Atomicity (§3) correct:** two standalone commits, single `git push origin HEAD:master`, explicit staging (no `git add -A`), HEAD:master push to dodge the detached-ref footgun.

### Minor findings (non-blocking)

Four Minor items, all citation-decay or documentation-only — none alter the design, none block the gate. The spec already instructs the implementer to derive the stage list dynamically and re-grep, which absorbs the line-number drift. See the structured findings list.

**Gate decision: GREEN. No re-dispatch required. Implementer may proceed; fold the Minor citation corrections opportunistically.**