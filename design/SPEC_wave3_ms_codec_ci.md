# Wave-3 lane W3-7 — ms-codec CI + one-time fmt normalization (mnemonic-secret)

Repo: `/scratch/code/shibboleth/mnemonic-secret` (ms-codec 0.6.0 + ms-cli 0.11.0).
Semver: **NO-BUMP** both crates. Ship: **2 commits, direct push to `master`** (no PR/tag/publish).
Burns down FOLLOWUP slug `ms-codec-no-ci-workflow` (FOLLOWUPS.md header at **:71**).

All citations below RE-VERIFIED against current `master` (HEAD `9d3d24ba`, 2026-06-22). Where the grounded recon diverges from ground truth, the spec carries the corrected fact and says so explicitly.

---

## 0. Ground-truth corrections vs the recon (read FIRST)

The recon's central scope claim is **empirically wrong** and the spec overrides it:

| Claim | Recon | VERIFIED ground truth |
|---|---|---|
| rustfmt that `1.95.0` resolves to | (implied older) | `rustfmt 1.9.0-stable (59807616e1 2026-04-14)` = **current stable** (`+1.95.0` ≡ `+stable`). Confirmed by `cargo +1.95.0 fmt --version` AND toolkit FOLLOWUPS `toolkit-rustfmt-1-95-0-rebaseline-divergence`. |
| chore(fmt) scope @ 1.95.0 | "15 ms-codec files; ms-cli already clean" | **43 non-mlock files** reformat: ms-codec 8 src + 7 tests = 15; **ms-cli 10 src + 18 tests = 28** (+ mlock.rs, excluded). ms-cli is NOT already clean — 1.9.0-stable reorders imports / collapses single-line `if` / wraps args across ms-cli too. |
| precedent | (none) | The TOOLKIT just shipped the identical chore in Wave 1 (28 non-mlock files, `cargo +1.95.0 fmt --all` + `git checkout -- mlock.rs`, NO-BUMP) — resolved 2026-06-22. **This lane is the sibling of that exact pattern.** |

Everything else in the recon (mlock carve-out load-bearing, g6 CI-only trap, NO-BUMP, two standalone commits, gate-grep shape) is CONFIRMED correct.

The DECISION (pin 1.95.0, exclude mlock.rs from both normalization and gate) is unchanged. Only the **file count and the "format the whole tree, not a subset" instruction** change.

---

## 1. Commit 1 — `chore(fmt): rustfmt 1.95.0 normalization (mlock.rs exempt; NO-BUMP)`

### 1.1 Current behavior
- No fmt step exists anywhere (`grep -rn 'fmt\|rustfmt' .github/workflows/` → NONE — verified).
- `RUSTUP_TOOLCHAIN=1.95.0 cargo fmt --all -- --check` exits non-zero with **44 `Diff in` files** (43 non-mlock + `crates/ms-cli/src/mlock.rs`).

### 1.2 Exact procedure (single commit)
```
cd /scratch/code/shibboleth/mnemonic-secret
RUSTUP_TOOLCHAIN=1.95.0 cargo fmt --all          # reformats the whole workspace incl. mlock.rs
git checkout -- crates/ms-cli/src/mlock.rs       # REVERT mlock.rs — keep it g6-synced/unformatted
```
Then stage ONLY the 43 reformatted non-mlock files explicitly (no `git add -A` — repo convention). Stage list = the union below. **Verify `git status` shows mlock.rs as NOT modified before commit.**

> The exact equivalent ran in the toolkit as `cargo +1.95.0 fmt --all` then `git checkout -- mlock.rs`. Use `RUSTUP_TOOLCHAIN=1.95.0` (or `cargo +1.95.0`) so the pinned-1.95.0 toolchain's rustfmt 1.9.0-stable is used, NOT the repo's `rust-toolchain.toml` 1.85.0 (which ships rustfmt 1.8.0-stable and would format differently — diverging from the toolkit and from the new CI gate).

### 1.3 Exact files staged (43 — VERIFIED via fmt --check)
**ms-codec/src (8):** `bch_decode.rs consts.rs decode.rs envelope.rs error.rs lib.rs payload.rs shares.rs`
**ms-codec/tests (7):** `bch_decode.rs bip93_inline_vectors.rs forward_compat.rs mnem.rs parity_smoke.rs spike_kofn.rs uppercase_envelope.rs`
**ms-cli/src (10):** `cmd/combine.rs cmd/decode.rs cmd/derive.rs cmd/encode.rs cmd/inspect.rs cmd/repair.rs cmd/split.rs cmd/verify.rs error.rs language.rs`
**ms-cli/tests (18):** `cli_combine.rs cli_derive.rs cli_output_class.rs cli_repair.rs cli_split.rs combine_entropy_language_advisory.rs decode_grouped.rs decode_mnem_japanese.rs derive_mnem_non_english.rs encode_canonical_12_word.rs encode_grouping_flags.rs encode_hex_input.rs encode_mnem_japanese.rs encode_output_unchanged_after_split_refactor.rs inspect_mnem_string.rs inspect_share.rs lint_zeroize_discipline.rs verify_mnem_non_english.rs`
**EXCLUDED (do NOT stage; must be unmodified):** `crates/ms-cli/src/mlock.rs`

> The implementer should derive the stage list dynamically rather than trust this snapshot: `RUSTUP_TOOLCHAIN=1.95.0 cargo fmt --all -- --check 2>&1 | grep -oE '^Diff in [^:]+' | sed 's/^Diff in //' | grep -v '/mlock\.rs$' | sort -u`. (Citations decay — re-grep at write time.)

### 1.4 Why this is safe (no logic/wire change)
Changes are purely cosmetic: import-granularity reordering (`{OutputClass, emit_x}`→`{emit_x, OutputClass}`), single-line-`if` collapse, method-call-chain wrapping, macro-arg wrapping. No token/identifier/control-flow change. ms-codec wire-format, ms-cli CLI surface, and all `--json` shapes are untouched.

### 1.5 mlock.rs carve-out — PROVEN load-bearing
Formatting mlock.rs reflows two code lines (verified):
- `:169` `self.total_bytes_unlocked.fetch_add(bytes, Ordering::Relaxed);` → split across 2 lines.
- `:375` `assert_eq!(count, 3, "…");` → wrapped across 4 lines.

Plus an import reorder and a single-line-`if` collapse. The g6 `normalize()` (`crates/ms-cli/tests/mlock_g6_invariant.rs:126-136`) strips only comment/empty lines and `.trim()`s — it **never re-joins split lines**, so these reflows change normalized bytes. PROVEN: a formatted copy of mlock.rs, normalized, differs from BOTH current ms mlock.rs AND `toolkit@master` mlock.rs (diff non-empty). Since the ms g6 CI job compares ms-master-mlock vs toolkit-master-mlock (unformatted), reformatting mlock.rs ⇒ g6 RED. Hence the revert + the gate carve-out.

---

## 2. Commit 2 — `ci(ms-codec): add test+clippy+fmt CI (mlock.rs carve-out)`

Add three NEW jobs to `.github/workflows/rust.yml` (extend the existing file — keeps one workflow per repo, matches the recon's option (a) and the toolkit layout). Do NOT create a second workflow file. Do NOT touch the existing `test`/`test-release-mlock-einval`/`miri`/`clippy`/`g6-invariant` jobs.

### 2.0 Path-filter note (IMPORTANT for CI firing)
The current `rust.yml` `paths:` filter (lines 20-30) is scoped to `crates/ms-cli/**` + `Cargo.{toml,lock}` + `.github/workflows/rust.yml`. To make the NEW ms-codec jobs actually fire on ms-codec source changes, **add `crates/ms-codec/**` to BOTH the `push.paths` and `pull_request.paths` lists** (lines 20-24 and 26-30). Without this, the ms-codec test/clippy jobs only run when ms-cli or this workflow changes — defeating the purpose. The repo-wide fmt job also needs ms-codec in scope; the same filter addition covers it.

### 2.1 NEW job — `fmt` (repo-wide, pinned 1.95.0, mlock.rs carve-out)
Copy the toolkit's gate VERBATIM (toolkit `rust.yml:50-76`), adapting only the comment text. Exact step body:
```yaml
  fmt:
    name: fmt (pinned 1.95.0)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust 1.95.0 (canonical fmt toolchain)
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: 1.95.0
          components: rustfmt
      - name: Rustfmt (pinned 1.95.0, overriding rust-toolchain.toml; mlock.rs exempt)
        run: |
          out=$(cargo +1.95.0 fmt --all -- --check 2>&1) || true
          bad=$(printf '%s\n' "$out" | grep -oE '^Diff in [^:]+' \
            | sed 's/^Diff in //' | grep -v '/mlock\.rs$' || true)
          if [ -n "$bad" ]; then
            echo "::error::rustfmt 1.95.0 — these non-exempt files need formatting:"
            printf '%s\n' "$bad"
            printf '%s\n' "$out"
            exit 1
          fi
          echo "rustfmt 1.95.0 clean (mlock.rs exempt by the g6 cross-repo invariant)"
```
Include a load-bearing comment block on the `fmt` job mirroring the toolkit's (`rust.yml:29-49`): explain the 1.95.0 pin, the mlock.rs g6 exemption, and the asymmetric-pin / drop-on-next-pin-bump note. **The gate carve-out is grep-verified:** `grep -oE '^Diff in [^:]+'` captures the absolute path; `grep -v '/mlock\.rs$'` excludes only mlock.rs. Confirmed against this rustfmt's header format.

> `actions/checkout@v4` chosen to match the rest of THIS file (ms rust.yml uses `@v4` throughout; the toolkit uses `@v6`). Either works; keep the file internally consistent at `@v4`.

### 2.2 NEW job — `test-ms-codec`
```yaml
  test-ms-codec:
    name: test (ms-codec)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: cargo test -p ms-codec
        run: cargo test -p ms-codec
```
ms-codec is a pure lib crate (no mlock, no ulimit dance needed; no FAIL_MODE matrix). Single platform is sufficient (no platform-specific code; ms-cli already carries the macOS matrix for mlock). VERIFIED GREEN locally: 19 test binaries, 0 failed.

### 2.3 NEW job — `clippy-ms-codec`
```yaml
  clippy-ms-codec:
    name: clippy (ms-codec)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: cargo clippy -p ms-codec --all-targets -- -D warnings
        run: cargo clippy -p ms-codec --all-targets -- -D warnings
```
VERIFIED clean locally (exit 0). No `--all-features` (ms-codec has no `[features]`).

### 2.4 ms-cli fmt coverage
The `fmt` job (§2.1) is `--all` (whole workspace) → it ALREADY covers ms-cli with the same mlock.rs carve-out. **No separate ms-cli fmt step is needed.** (The orchestrator's "Add ms-cli fmt step with the SAME carve-out" is satisfied by the repo-wide `--all` job — ms-cli files are in the same workspace and the single gate enforces them with the identical mlock.rs exclusion.)

---

## 3. Atomicity & ordering

1. **Commit 1 (chore-fmt) lands FIRST, standalone.** Slug + recon §risks(3): keep it a single pure-whitespace commit so future `git bisect` over logic isn't polluted; never bundle into a feature/CI commit.
2. **Commit 2 (CI) lands SECOND**, same push. If commit 2 (the new fmt gate) ever preceded commit 1, the fmt job would be RED (43 unformatted files). Ordering commit1→commit2 guarantees the gate is GREEN the moment it exists.
3. **Single atomic `git push origin HEAD:master`** of both commits (avoid a split push leaving an intermediate RED state — same discipline as the sibling-pin-check atomicity rule, applied here to the fmt-gate/normalization pair). The repo is on branch `master`, HEAD `9d3d24ba`; push via `HEAD:master` (guard against any detached-HEAD/stale-ref footgun per the standing gotcha).
4. No `git add -A` — stage the 43 files (commit 1) and `.github/workflows/rust.yml` + `design/FOLLOWUPS.md` (commit 2) explicitly.

---

## 4. Verification surface (run ALL before push)

```
cd /scratch/code/shibboleth/mnemonic-secret
# after commit 1:
RUSTUP_TOOLCHAIN=1.95.0 cargo fmt --all -- --check        # expect: ONLY mlock.rs in Diff headers
git status --porcelain crates/ms-cli/src/mlock.rs          # expect: EMPTY (mlock.rs unmodified)
cargo test -p ms-codec                                      # expect: all GREEN (0 failed)
cargo clippy -p ms-codec --all-targets -- -D warnings       # expect: exit 0
cargo test -p ms-cli                                        # expect: GREEN (commit 1 touched 28 ms-cli files)
cargo clippy -p ms-cli --all-targets -- -D warnings         # expect: exit 0
# THE CI-ONLY g6 TRAP — local repro with adjacent toolkit checkout:
SIBLING_REPO_PATH=/scratch/code/shibboleth/mnemonic-toolkit \
  cargo test -p ms-cli --test mlock_g6_invariant -- --include-ignored   # expect: 2 passed
```
The g6 test is `#[ignore]`-gated and needs `SIBLING_REPO_PATH` + an adjacent toolkit checkout — a bare `cargo test` SKIPS it, so this is the load-bearing manual step that catches the CI-only g6 break a plain build would miss (the G1-B-class trap). Baseline (pre-change) already measured 2-passed; it must stay 2-passed AFTER commit 1 (proving mlock.rs was correctly NOT reformatted).

---

## 5. FOLLOWUP flips (commit 2)

`design/FOLLOWUPS.md` slug **`ms-codec-no-ci-workflow`** (header at **:71**, currently `Status: open`, `Tier: v0.1-nice-to-have`):
- Flip `**Status:** open` → `**Status:** ✓ RESOLVED (CI added + fmt-normalized; NO-BUMP; <SHA1>+<SHA2>)`.
- Add a resolution bullet recording: (a) chore(fmt) normalized 43 non-mlock files via `cargo +1.95.0 fmt --all` + `git checkout -- mlock.rs`; (b) new rust.yml jobs `fmt` (1.95.0, mlock.rs carve-out), `test-ms-codec`, `clippy-ms-codec`; (c) ms-codec's 19 test files now run in CI for the first time; (d) the `~16 files` claim was stale — actual 1.95.0/rustfmt-1.9.0 scope is 43 non-mlock files (matches the toolkit's Wave-1 `toolkit-rustfmt-1-95-0-rebaseline-divergence`).
- Correct the stale `**Where:**` text ("the only workflow is `rust.yml`") — there are now two workflows (`rust.yml` + `fuzz-smoke.yml`).

Do NOT touch `mlock-rs-fmt-exempt` / `mlock-g4-a-page-count-assert-flake` (toolkit-side; deferred to the next ms-cli g6-pin tag — see deferred_notes).

---

## 6. CI gate verification summary (HOW, incl. CI-only)

See `ci_gates_to_verify` for the per-gate HOW. The one CI-ONLY gate not reproducible by a plain local build is the **g6-invariant job** — its local repro is the `SIBLING_REPO_PATH=… --include-ignored` invocation in §4. The NEW `fmt` gate is also effectively CI-shaped (depends on the pinned 1.95.0 toolchain) but is reproduced locally via `RUSTUP_TOOLCHAIN=1.95.0`. No GUI/manual/sibling-pin gate is in scope (no CLI-surface delta, no install.sh/pin/manual change in this lane).