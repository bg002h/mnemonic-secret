# FIX_bughunt1119_ms_out_writes: `--out` writes that keep the old file and never share a file with stdout (bh-ms-I1, bh-ms-I13; both Important)

Small-fix track (bughunt-1119 FIX table row #11). Repo mnemonic-secret, branch `fix/ms-out-writes` from master `08ea22c`.
Crate `ms-cli` only; no ms-codec change, no wire change, no new flag, type or command. Author: Claude Opus 5.5
(claude-opus-5-5), 2026-11-24.

Rulings used, all decided: coordinator `agent-reports/architect-bughunt-1119-questions.md` T5 (safe write lives in
mnemonic-io-lib `write_private`; `same_file` "(dev+ino on Unix; stdout by fstat of fd 1) for the alias rows ...
ms-I1"), T6 (failed-write contract), P5 (grade of a failed overwrite: I); `memory/file-output-convention-rulings-2026-11-23.md`
(Brian: Q2 output names an input: refuse, name both, "write to a new name"; Q3 replace notice) and the convention report
`agent-reports/architect-file-output-convention-1123.md` s3 (R2 one writer, R3b create-only, R5 truthful text);
`agent-reports/architect-mt-output-writes-1124.md` (the shared failure line with a second look, ruled for mt and md;
same-file refusal covers stdout; Q3 notice deferred to io-lib 0.1.3; md's input aliasing is a separate row); batch
overrides `briefs/bh1119-batch-1124-overrides.md` (io-lib = registry `"0.1.2"`). Sibling docs matched: md #9
(descriptor-mnemonic `fix/md-out-writes` 3e3432d5) and mt #12 (mnemonic-transaction `fix/mt-output-writes` 4f290a4a).

Final grades: **bh-ms-I13 Important** (P5: a failed overwrite destroys the previous valid backup), **bh-ms-I1
Important** (grade-ms: exit 0 with a JSON or digest file where the card was promised). This unit also owns FOLLOWUPS
`[ms-create-new-partial-file-blocks-retry]` (Minor, hunt R13; `design/FOLLOWUPS.md:1380`).

Every new holder-visible text is PROPOSED (architect confirms) unless it
is quoted from a ruling.

Toolchain: `export PATH=$HOME/.cargo/bin:$PATH`; cargo 1.85.0 (`rust-toolchain.toml`); `--locked`; every cargo call
with `CARGO_BUILD_JOBS=6`; nextest `--build-jobs 6 -j 6`; never `--release`. Test data only (entropy `00`x16 and
`11`x16, preimages `01`x32 and `02`x32); no wallet is involved.

## 1. The defects (RED evidence, run by the author on the base 08ea22c)

Base binary `target/debug/ms` from `cargo build --locked -p ms-cli --bin ms`. Every row below is a block of
`design/FIX_bughunt1119_ms_out_writes/red.sh` (this branch), run as `MS=<ms> bash design/FIX_bughunt1119_ms_out_writes/red.sh`.
"FSIZE8" = `bash -c "trap '' XFSZ; exec prlimit --fsize=8 ms ..."` with stderr through a pipe (an 8-byte file-size
limit with the signal ignored: the write returns EFBIG, standing in for a full disk or quota).

- **bh-ms-I13, a failed overwrite destroys the previous `--out` file.** `ms encode --hex - --out F` (F = the 51-byte
  card of `00`x16), then FSIZE8 `ms encode --hex - --out F` with `11`x16: exit 1, stderr
  `error: failed to write --out F: File too large (os error 27)`, F is now the 8 bytes `ms10entr`, `ms verify --in F`
  exits 1. The same on `ms hashlock --hex - --kind sha256 --out P` (P becomes `ms10hash`) and `ms repair --in BAD
  --out C`. An absent F is left as an 8-byte fragment. Cause: `out.rs:83-88` `write_artifact` calls
  `mnemonic_io_lib::write::write_private` from io-lib 0.1.1 (git rev d8fec678, `crates/ms-cli/Cargo.toml:31`), which
  opens, truncates, then writes in place.
- **bh-ms-I1, `--out` and standard output can be one file.** `ms encode --hex - --out F --json > F`: exit 0, F holds the
  147-byte JSON object, `ms verify --in F` exits 1. The same through a hard link (`> F2`, F2 a link to F) and a symlink.
  `ms hashlock --hex - --kind sha256 --out P > P`: exit 0, P holds the `hash:` line, reading P back
  (`ms hashlock --kind sha256 --in P`) exits 1; with `--json`, P holds 481 bytes of JSON. `ms repair --in BAD --out C
  > C`: exit 4, C holds `# Repair report ...`, verify exits 1. Cause: `encode.rs:272-277`, `hashlock.rs:428-440` and
  `repair.rs:169-177` write `--out` and then print on stdout, and nothing compares the two.
- **Also seen, the stderr alias:** `ms encode --hex - --out F 2> F` leaves F holding the engraving card (73 bytes,
  `word count: 12 ...`), verify exits 1: the card is written to stderr after the in-place write.
- **R13 (Minor, owned here):** FSIZE8 `ms split -k 2 -n 3 --hex - --out SH` leaves an 8-byte SH and exit 1; the retry
  without the limit exits 64 (`--out SH already exists; ...`). `ms hashlock --random --out R` the same.
- **The T6 gap:** every `--out` write error prints `error: failed to write --out <FILE>: <error>` (exit 1), where T6
  and the architect's 1124 ruling give `ms: cannot write the output: <FILE>: <error>` plus the state sentence.

## 2. Rulings applied (quoted)

- T5: `write_private` becomes temp file + rename ("on any error remove the temp and leave the old file
  byte-identical"; a non-regular target keeps the direct write); `same_file(out, &[inputs])` for the alias rows,
  "ms-I1" among them.
- T6: "any other stdout or `--out` write error prints `<tool>: cannot write the output: <error>` and exits 1; a verb
  already failing keeps its code".
- Architect 1124 s2 (for mt and md, "so md and mt print the same shape"): `<tool>: cannot write the output: <path>:
  <error>` followed by `. <path> is unchanged.` or `. Nothing was written.` only when a second look after the failure
  confirms it; otherwise nothing about the disk.
- Architect 1124 s4: the same-file refusal treats standard output as an output; `Write to a new name.`
- Architect 1124 s5: the Q3 replace notice waits for io-lib 0.1.3 (`Written{Created,Replaced}`); not in this row.
- Architect 1124 F2: md's input aliasing (`--in F --out F`) is a separate row; ms's is too (section 9).

## 3. Spec delta (behaviour)

- **R1 Pin and writer (T5).** ms-cli takes mnemonic-io-lib **0.1.2** (pin form: Q1).
  The version is V = the one the coordinator names at build (0.1.2 unless named otherwise, e.g. 0.1.3 after the
  stdio-append fix): every "0.1.2" in P1 (pin and comment), R6, C1, the SPEC_ms_hashlock row and gate step 0 and I6's
  gate lines reads V, and P1's "published from mnemonic-engrave e09d538d (the #41 merge)" names V's source commit as the
  coordinator gives it. `write_artifact` (`out.rs:85-88`)
  keeps calling `write_private` and adds no write logic. What the holder sees on `encode --out`, `repair --out` and
  `hashlock --out` (not `--random`), from 0.1.2's documented behaviour (`crates/mnemonic-io-lib/src/write.rs:78-98` at
  mnemonic-engrave e09d538d):
  - an existing regular FILE (or an absent one) is written to a temporary `FILE.<pid>.<n>.tmp` beside it, created
    0600, synced, and renamed over FILE; any error leaves an existing FILE byte-identical and removes the temporary;
  - a run killed mid-write can leave that temporary behind (0600);
  - FILE's directory must be writable: an existing writable FILE in a read-only directory, written in place before, is
    now refused (R2 text) and left unchanged;
  - a symlink FILE is followed and its target replaced; the link stays a link. A hard-linked FILE is replaced at this
    name only;
  - a FIFO, device or standard-stream name (`/dev/stdout`, `/dev/fd/N`, `/proc/...`) is written directly, as today. (how a regular file behind such a name is written is io-lib's; see section 9)
  - Consequence, no code: a stderr or stdout that the shell opened on FILE keeps writing to the old, now unnamed file,
    so FILE holds the artifact (rows o16, o17).
- **R2 Failure text (T6 + R5 + architect 1124 s2).** On any error writing the `--out` file, on every `--out` verb
  (`encode`, `repair`, `hashlock`, `hashlock --random`, `split`): before the write ms takes `std::fs::metadata(FILE)`
  (symlinks followed): a regular file keeps (dev, ino, len, mtime); `NotFound` (including a dangling symlink) is
  "absent"; anything else is "other". For the create-only writer (`split`, `hashlock --random`) "before" is "absent"
  (the O_EXCL create proves it). After the failure (and after R4's removal) ms takes the same metadata again, then
  exits **1** with exactly one line (text mode, stderr):
  - regular before and the same (dev, ino, len, mtime) after: `ms: cannot write the output: <FILE>: <error>. <FILE> is unchanged.`
  - absent before and `NotFound` after: `ms: cannot write the output: <FILE>: <error>. Nothing was written.`
  - anything else: `ms: cannot write the output: <FILE>: <error>`
  `<FILE>` is `--out` as typed (`Path::display()`); `<error>` is the `std::io::Error` Display, unmodified. Off Unix
  (no dev/ino) "the same" compares (len, mtime) only. Nothing is printed on stdout by a run that fails here (the
  write precedes every stdout line in all five verbs, unchanged). Both modes print that one line on stderr and nothing on stdout: in `--json` mode there is no envelope
  (T6's line; md #9 R2 and mk #10 R3 print the same shape). SPEC_ms_cli §6.3 gains the exception (I4).
  Implementation note: two new CliError variants, OutputWrite(String) (R2: exit_code 1,
  kind "BadInput", Display "ms: {message}") and StdoutIsOut(String) (R3: exit_code 64, kind "Usage", Display
  "error: {message}"), and a method stderr_only() true for both; main.rs emit_error prints the envelope only when
  json_mode && !e.stderr_only(), else the Display line on stderr.
- **R3 `--out` is never standard output (T5 `same_file`, Q2, architect 1124 s4).** New `out::refuse_stdout_alias(FILE)`:
  `mnemonic_io_lib::write::same_file(Endpoint::Path(FILE), &[Endpoint::Stdout])`; a match refuses with exit **64**
  (`CliError::Usage`, the code ms's other `--out` refusal uses) and the R3 text (section 4), and nothing is read,
  drawn or written. Identity is io-lib's: dev+ino of the file a path resolves to (symlinks followed, hard links equal);
  an absent FILE by its canonical directory plus name; a pipe, terminal or device never matches; off Unix a redirected
  stdout is never detected (io-lib's documented limit).
  - Where it runs: `encode` and `split` when `--out` and `--json` are
    both given (in text mode with `--out` neither prints on stdout: rows o16, o18), as the first step of `run` after the
    argument moves (encode before `resolve_secret_payload`, `encode.rs:245`; split before `refuse_existing_out`,
    `split.rs:107-109`); `hashlock` whenever `--out` is given, `--random` included, as the first statement of `run`
    (`hashlock.rs:391`, before `pick_source`; hashlock always prints the record or the JSON on stdout); `repair` when
    `--out` is given, as the first statement of `run` (`repair.rs:130`).
  - `--out /dev/stdout --json` with stdout on a regular file matches (the path resolves to that file) and is refused;
    with stdout on a pipe or terminal it does not match and both are written, as today.
  - Both modes: the refusal is E2 on stderr, never an envelope on stdout (stdout is FILE, and
    an envelope there would be written into it, as mk #10 R4 rules); stdout stays empty, exit 64. With `>` the shell has
    already emptied FILE, which E2 says; with `>>` FILE is left byte-identical.
- **R4 Create-only writer leaves nothing (R13; FOLLOWUPS:1380).** In `write_artifact_create_new` (`out.rs:46-77`), when
  `write_all` fails after the O_EXCL create: if `std::fs::symlink_metadata(FILE)` has the same (dev, ino) as the open
  handle's metadata, ms removes FILE (a removal error is ignored; R2's second look then reports the state). Off Unix
  nothing is removed. Then R2's text. So `split --out` and `hashlock --random --out` leave nothing at FILE on a failed
  write, and a retry succeeds (rows o9, o9b, o9c). The existing-target refusal is unchanged.
- **R5 Help, manual, changelog.** Help texts change (section 4: H1, H2). No flag is added, removed or renamed, so the
  mnemonic-gui schema mirror is not touched (`ms gui-schema` carries no help text; checked). Toolkit manual
  `docs/manual/src/40-cli-reference/43-ms.md` changes in lockstep (M1, M2). CHANGELOG C1.
- **R6 Pin consequences.** `Cargo.lock` takes io-lib 0.1.2; `vendor/mnemonic-io-lib` is re-vendored (`cargo vendor`)
  and committed with the lock. Under the registry form (Q1 (a)) also: the `crates/ms-cli/Cargo.toml:21-31` comment
  becomes text P1 (section 4), the row `mnemonic-io-lib|https://github.com/bg002h/mnemonic-engrave|crates/mnemonic-io-lib`
  leaves `ci/repro/vendor-git-sources.txt` (the file keeps its comments), and every git-source derivation listed in
  section 11 is removed. `design/SPEC_ms_hashlock.md:912` (code-fact row `--out` is a truncating `write_private`)
  becomes ``| `--out` replaces through a temp file and a rename (`write_private`, io-lib 0.1.2) | `crates/ms-cli/src/out.rs:85-88` |``.
  - `design/SPEC_ms_cli_v0_1.md` §6.3 gains, after "Process exit code matches `CliError::exit_code()`.", the paragraph:
    `Two errors are printed on stderr in both modes and never as an envelope. A failed `--out` write prints `ms: cannot
    write the output: <FILE>: <error>`, followed by `. <FILE> is unchanged.` or `. Nothing was written.` when a second look
    at FILE confirms it, and exits 1 (the constellation's failed-write contract, T6). `--out` naming the same file as
    standard output prints `error: --out <FILE> and standard output are the same file ...` and exits 64: stdout is FILE,
    so an envelope would be written into it.`
- **R7 Registry-pin edits (Q1 (a)).** Apply exactly the diff in appendix A (four files: man-release.yml,
  docs/verify-reproducibility.md, ci/bh1119-doc-lint.test.sh, ci/repro/vendor-git-sources.txt); P1 for Cargo.toml;
  `cargo update -p mnemonic-io-lib` for Cargo.lock (it removes the git entry and adds the registry one, nothing else);
  `cargo vendor --locked vendor/` (the command vendor-freshness.sh:145 names) and commit vendor/ with the lock.

## 4. Texts (exact)

- **E1 (ruled: T6 + architect 1124 s2, not new):** the three R2 lines.
- **E2, the R3 refusal, PROPOSED** (printed by `Usage`'s Display, so text mode reads `error: ...`):
  `error: --out <FILE> and standard output are the same file (the same path, a symlink or a hard link), so what ms prints on stdout would go into the file it writes. A > redirect empties that file before ms starts, so check that it still holds what you expect. Write to a new name.`
- **H1, `--out` help of `encode` (`encode.rs:31-35`) and `repair` (`repair.rs:66-70`), PROPOSED:** the paragraph
  "Through the shared crate's ... leaves no tail of the old file." becomes
  `Through the shared crate's `write::write_private`: the bytes go to a temporary `FILE.<pid>.<n>.tmp` beside FILE, created 0600 and synced, which then replaces FILE, so a failed write leaves an existing FILE unchanged. FILE's directory must be writable, and a run killed mid-write can leave that owner-only temporary behind. It OVERWRITES (§6b); a symlink is followed and its target replaced. If ms would also print on stdout and stdout is FILE, the run is refused (exit 64).`
  (the first and last paragraphs are unchanged; the doc comment wraps at the file's width).
- **H2, `hashlock --out` help (`hashlock.rs:75`), PROPOSED:** `Write the preimage ms1 string here, owner-only; a failed write leaves an existing file unchanged. Never suppresses stdout, so stdout must not be this file (refused, exit 64).`
- **M1, toolkit manual `43-ms.md:42-48` (the "Output can go to a file" paragraph), PROPOSED, whole paragraph:**
  ``Output can go to a file as well: **`--out FILE`** on `encode`, `split`, `repair` and `hashlock` writes the canonical artifact owner-only (mode `0600`, so an existing world-readable file is tightened too). The new contents go to a temporary `FILE.<pid>.<n>.tmp` beside FILE, which then replaces it, so a failed write leaves an existing FILE unchanged; FILE's directory must be writable, and a run killed mid-write can leave that owner-only temporary behind. A symlink is followed and its target replaced. An existing FILE is overwritten. `split` and `hashlock --random` are the exceptions: their output is new on every run and cannot be re-made, so they refuse an existing FILE (exit 64, naming it), and a failed write leaves nothing at FILE. When the verb also prints on stdout (`encode --json`, `split --json`, `repair`, `hashlock`, `--random` too) and stdout is FILE (the same path, a symlink or a hard link), the run is refused before anything is read (exit 64). `ms gen-man --out <DIR>` is unrelated: there it names a directory.``
- **M2, manual `43-ms.md:900` ("When stdout fails"), PROPOSED, sentence appended to the paragraph:**
  `` A write error on an `--out` file prints `ms: cannot write the output: <FILE>: <error>` and exits 1; the line ends `. <FILE> is unchanged.` or `. Nothing was written.` when ms can see that is so.``
- **C1, CHANGELOG `## ms-cli [Unreleased]` / `### Fixed`, PROPOSED (three bullets):**
  - ``**A failed `--out` write no longer destroys the existing file** (bh-ms-I13). `encode`, `repair` and `hashlock` write a temporary file beside FILE and rename it over FILE (mnemonic-io-lib 0.1.2), so a full disk or quota leaves the old file byte-identical; FILE's directory must now be writable. `split` and `hashlock --random` remove the partial file they created, so a retry works. The error is now `ms: cannot write the output: <FILE>: <error>`, followed by `. <FILE> is unchanged.` or `. Nothing was written.` when that is so (was `error: failed to write --out <FILE>: <error>`; exit 1 as before).``
  - ``**`--out` and standard output can no longer be one file** (bh-ms-I1). `ms encode --json --out F > F`, `ms hashlock --out P > P` and `ms repair --out C > C` (or a link to the file) exited 0 with the report where the card was promised; they are now refused before anything is read (exit 64). `ms hashlock --random --json --out R >> R` and `ms split --json --out SH >> SH` no longer append an error object to the file they refused to overwrite.``
  - ``**`ms encode --out F 2> F` keeps the card in F.** The engraving card on stderr no longer overwrites it.``
- **P1, `crates/ms-cli/Cargo.toml:21-31` under Q1 (a), PROPOSED (comment and pin, whole):**
  ```toml
  # THE SHARED IO MECHANISM (P2), from crates.io. mnemonic-io-lib 0.1.2 was
  # published from mnemonic-engrave e09d538d (the #41 merge), so a clean checkout
  # (freebsd-compile-gate, both musl-check targets) resolves it with no sibling
  # repository and no git source (Brian 2026-11-24). 0.1.2 writes through a temp
  # file and a rename, so a failed write leaves the old file as it was, and adds
  # `same_file`. It must carry the four items P2 adopts: `fd::mode_of`,
  # `remedy::history_purge_block`, `remedy::history_purge_recipes`,
  # `write::write_private`; `tests/crate_items_resolve.rs` goes RED if not.
  mnemonic-io-lib = "0.1.2"
  ```

Each of E2, H1, H2, M1, M2, C1 gets a `design/WORDING_REVIEW.md` row (direct-push record, after the merge).

## 5. TRACE rows (RED on the base; GREEN after the build)

New file `crates/ms-cli/tests/bh1119_ms_out_writes.rs`, one `#[test]` per row, named exactly as the row; Unix only
(`#![cfg(unix)]`); data in a `tempfile::tempdir()`. FSIZE8 rows use `std::process::Command` with `pre_exec`
(`libc::setrlimit(RLIMIT_FSIZE, 8/8)`, `libc::signal(SIGXFSZ, SIG_IGN)`), as triage test
`bh_ms_i13_failed_overwrite_preserves_previous_backup` did (`agent-reports/bughunt-1119/triage-ms-t3.md:18`), and
stderr to a pipe. A "stdout on F" row opens F with `OpenOptions::new().write(true).create(true).truncate(true)` (as
`>` does) and passes it as the child's stdout. A "stdout appended to F" row (o8, o22-o24) opens F with
`OpenOptions::new().append(true)` (as `>>` does) and passes it as the child's stdout. `CARD0` = `ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f` +
newline (the card of `00`x16); `BAD` = CARD0 with `q` at character 21 replaced by `p` (repair corrects it, exit 4).
`E27(X)` = `ms: cannot write the output: X: File too large (os error 27)`; `SAME(X)` = E2 with `<FILE>` = X. Every
base cell was seen on 08ea22c; every fixed cell was seen on a scratch prototype of R1-R4 (io-lib 0.1.2 by local path,
reverted; section 6). Base outputs: `MS=<base ms> bash design/FIX_bughunt1119_ms_out_writes/red.sh`.

| row | setup; command (after `ms`) | base (seen) | fixed (seen on prototype) |
|---|---|---|---|
| o1_encode_failed_overwrite_keeps_previous_card | F = CARD0; FSIZE8 `encode --hex - --out F` with `11`x16 | rc 1, `error: failed to write --out F: File too large (os error 27)`, F = `ms10entr` | rc 1, stderr exactly `E27(F). F is unchanged.`, stdout empty, F == CARD0, verify F rc 0, no `*.tmp` in the dir |
| o2_hashlock_failed_overwrite_keeps_previous_plate | P = `hashlock --hex - --kind sha256 --no-engraving-card --out P` of `01`x32; FSIZE8 the same with `02`x32 | rc 1, P = `ms10hash` | rc 1, stderr exactly `E27(P). P is unchanged.`, P byte-identical |
| o3_repair_failed_overwrite_keeps_previous_card | C = CARD0; FSIZE8 `repair --in BAD --out C` | rc 1, C = `ms10entr` | rc 1, stderr exactly `E27(C). C is unchanged.`, C == CARD0 |
| o4_absent_target_failed_write_leaves_nothing | FSIZE8 `encode --hex - --out N` (N absent) | rc 1, N = 8 bytes | rc 1, stderr exactly `E27(N). Nothing was written.`, N absent, no `*.tmp` |
| o5_missing_directory_text_and_exit | `encode --hex - --out nodir/x.ms1` | rc 1, `error: failed to write --out nodir/x.ms1: No such file or directory (os error 2)` | rc 1, stderr exactly `ms: cannot write the output: nodir/x.ms1: No such file or directory (os error 2). Nothing was written.` |
| o6_read_only_directory_refuses_and_keeps_file | `ro/F` = CARD0, chmod 0500 ro; `encode --hex - --out ro/F` with `11`x16 (skip when euid is 0) | rc 0, ro/F replaced | rc 1, stderr exactly `ms: cannot write the output: ro/F: Permission denied (os error 13). ro/F is unchanged.`, ro/F == CARD0 |
| o7_symlink_out_replaces_target_keeps_link | T = `OLD\n` 0644, `L -> T`; `encode --hex - --out L` | rc 0, L a symlink, T == CARD0, mode 600 | same (control) |
| o8_stdio_alias_onto_a_file_names_no_state | S2 = `OLD BACKUP LINE\n`; FSIZE8 `encode --hex - --out /dev/stdout` with stdout appended to S2 | rc 1, `error: failed to write --out /dev/stdout: File too large (os error 27)` | rc 1, stderr starts `E27(/dev/stdout)` and is either exactly `E27(/dev/stdout)` (S2 changed: io-lib 0.1.2 cuts it to `ms10entr`) or exactly `E27(/dev/stdout). /dev/stdout is unchanged.` (S2 still `OLD BACKUP LINE\n`: an io-lib that appends to a stdio alias); the test asserts the sentence is present if and only if S2 == `OLD BACKUP LINE\n` (seen: revB the first, revC the second). |
| o9_create_new_failed_write_text | FSIZE8 `split -k 2 -n 3 --hex - --out SH` | rc 1, `error: failed to write --out SH: ...`, SH = 8 bytes | rc 1, stderr last line exactly `E27(SH). Nothing was written.`, SH absent |
| o9b_create_new_retry_after_failed_write | o9, then `split -k 2 -n 3 --hex - --out SH` | rc 64, `--out SH already exists; ...` | rc 0, SH = 3 lines (153 bytes), mode 600 |
| o9c_random_preimage_failed_write_leaves_nothing | FSIZE8 `hashlock --random --no-engraving-card --out R` | rc 1, R = 8 bytes | rc 1, stderr last line exactly `E27(R). Nothing was written.`, R absent |
| o10_encode_json_stdout_is_out_refused | `encode --hex - --out F --json --no-engraving-card`, stdout on F | rc 0, F = JSON (147 bytes), verify F rc 1 | rc 64, stderr exactly SAME(F) + newline, stdout and F empty (0 bytes) |
| o11_encode_json_stdout_hard_link_refused | F empty, F2 a hard link to F; as o10 with stdout on F2 | rc 0, F = JSON | rc 64, stderr exactly SAME(F) + newline, F empty |
| o12_encode_json_stdout_symlink_refused | F empty, `LF -> F`; as o10 with stdout on LF | rc 0, F = JSON | rc 64, stderr exactly SAME(F) + newline, F empty |
| o13_hashlock_stdout_is_out_refused | `hashlock --hex - --kind sha256 --no-engraving-card --out P`, stdout on P | rc 0, P = `hash:72cd...3793` line, reading P back rc 1 | rc 64, stderr exactly SAME(P) + newline, P empty |
| o14_hashlock_json_stdout_is_out_refused | as o13 plus `--json` | rc 0, P = JSON (481 bytes) | rc 64, stderr exactly SAME(P) + newline, P empty |
| o15_repair_stdout_is_out_refused | `repair --in BAD --out C`, stdout on C | rc 4, C = `# Repair report ...` (124 bytes), verify C rc 1 | rc 64, stderr exactly SAME(C) + newline, C empty |
| o16_text_encode_out_with_stdout_on_it_still_writes | `encode --hex - --out F --no-engraving-card`, stdout on F | rc 0, F == CARD0 | same (control: text mode prints nothing on stdout) |
| o17_stderr_on_the_out_file_keeps_the_card | `encode --hex - --out F`, stderr on F | rc 0, F = the engraving card (73 bytes), verify F rc 1 | rc 0, F == CARD0, verify F rc 0 (R1 alone) |
| o18_split_out_with_stdout_on_it_refused_as_existing | `split -k 2 -n 3 --hex - --out SH`, stdout on SH | rc 64, `error: --out SH already exists; a new share set will not overwrite it (choose another file, or move the old one first)` | same (control) |
| o19_hashlock_random_out_with_stdout_on_it_refused | `hashlock --random --no-engraving-card --out R`, stdout on R | rc 64, `error: --out R already exists; a --random preimage will not overwrite it (...)` | rc 64, stderr exactly SAME(R) + newline, R empty |
| o20_fifo_out_written_directly | `--out FF`, FF a FIFO with a reader | rc 0, reader got CARD0 | same (control) |
| o21_json_out_with_piped_stdout_writes_both | `encode --hex - --out F --json --no-engraving-card`, stdout a pipe | rc 0, F == CARD0, JSON on the pipe | same (control) |
| o22_encode_json_stdout_appended_to_out_keeps_card | F = CARD0; `encode --hex - --out F --json --no-engraving-card` with `11`x16, stdout appended to F (`>>F`) | rc 0, F = the new card + the JSON object, verify F rc 1 | rc 64, stderr exactly SAME(F) + newline, F == CARD0, verify F rc 0 |
| o23_hashlock_random_json_stdout_appended_keeps_preimage | R = a `hashlock --random --no-engraving-card --out R` preimage; `hashlock --random --json --no-engraving-card --out R`, stdout appended to R | rc 64, R = preimage + envelope, `hashlock --kind sha256 --in R` rc 1 | rc 64, stderr exactly SAME(R) + newline, R byte-identical, `hashlock --kind sha256 --in R` rc 0 |
| o24_split_json_stdout_appended_keeps_shares | SH = `split -k 2 -n 3 --hex - --out SH` of `00`x16; `split -k 2 -n 3 --hex - --json --out SH`, stdout appended to SH | rc 64, SH = shares + envelope, `combine --in SH` rc 1 | rc 64, stderr exactly SAME(SH) + newline, SH byte-identical, `combine --in SH` rc 0 |
| o25_json_failed_overwrite_line_on_stderr_no_envelope | F = CARD0; FSIZE8 `encode --hex - --out F --json --no-engraving-card` with `11`x16, stdout and stderr through pipes | rc 1, stdout = envelope (`"kind":"BadInput"`, `failed to write --out F: ...`), F = `ms10entr` | rc 1, stdout empty, stderr exactly `E27(F). F is unchanged.`, F == CARD0 |

RED expectation for the builder: the 22 rows o1-o6, o8-o15, o17, o19 and o22-o25 fail on the base;
the 5 controls o7, o16, o18, o20 and o21 pass on the base and after.

Coverage map (rule -> rows): R1 old file kept -> o1, o2, o3; R1 absent target -> o4; R1 read-only directory -> o6;
R1 symlink -> o7; R1 FIFO -> o20; R1 consequence (shell-opened stream) -> o16, o17; R1 hard link and killed-run
temporary -> mnemonic-io-lib's own tests (not repeated); R2 "unchanged" -> o1, o2, o3, o6; R2 "Nothing was written"
-> o4, o5, o9, o9c; R2 no state -> o8; R2 both modes on stderr -> o25; R3 encode -> o10, o11, o12, o21 (no match on a pipe), o16 (text not checked), o22; R3 hashlock -> o13, o14, o19, o23;
R3 repair -> o15; R3 split -> o24 (JSON), o18 (text not checked); R4 -> o9, o9b, o9c; R5 -> section 4 texts and `--help` diff; R6 -> gate step 2 (vendor freshness) and
`crate_items_resolve.rs`.

## 6. Prototype and the tests that change

Measured: the scratch prototype (io-lib 0.1.2 from `git archive e09d538d crates/mnemonic-io-lib` as a path dependency,
R2 through a new `CliError` variant, R3 at the three call sites, R4 in `write_artifact_create_new`) run as
`cargo nextest run --offline --workspace --all-targets --all-features --no-fail-fast --build-jobs 6 -j 6` (not
`--locked`: the path dependency rewrites `Cargo.lock`): `786 tests run: 785 passed, 1 failed, 23 skipped`. The one
failure is the only test that changes:

- `crates/ms-cli/tests/split_out_refuses_existing.rs:139-147` `a_missing_parent_directory_is_still_exit_1`: the
  assertion `starts_with("error: failed to write --out <p>: ")` becomes stderr ==
  `ms: cannot write the output: <p>: No such file or directory (os error 2). Nothing was written.\n`; exit 1 is
  unchanged. Its doc comment ("a missing parent is still a write failure (exit 1)") stands.

Comments that go stale and are updated in the same change (no assertion): `out.rs:1-15` module header (the writer now
replaces through a temp file), `out.rs:80-84` doc of `write_artifact`, `tests/hashlock_sources.rs:306` ("the
truncating writer"). The triage tests `bh1119_ms_i1_json_stdout_aliasing_out_is_refused` (t1) and
`bh_ms_i13_failed_overwrite_preserves_previous_backup` (t3) are superseded by o10 and o1 and are not copied in.

## 7. Gate line

`bash scripts/ms-out-writes-fix-gate.sh` (the builder commits it with its coordinator catalog row; shape of
`scripts/ms-output-sink-fix-gate.sh`; every cargo call with `CARGO_BUILD_JOBS=6`; run once to a file and grep):
step 0 `cargo search mnemonic-io-lib --limit 1` lists `0.1.2` (else STOP, the overrides); step 1, in a detached
worktree at 08ea22c with only the new test file added, `cargo nextest run --locked -p ms-cli --test bh1119_ms_out_writes
--no-fail-fast` shows `27 tests run: 5 passed, 22 failed` and the 17 failing names are exactly section 5's RED set;
step 2 on the head: first, before the cargo steps, these lines (the mt #12 gate.sh pattern, ms paths) and
`actionlint .github/workflows/man-release.yml` (exit 0):

```bash
# T5 + Brian 2026-11-24: ms-cli pins mnemonic-io-lib from crates.io, no git source left anywhere.
grep -qx 'mnemonic-io-lib = "0.1.2"' crates/ms-cli/Cargo.toml || fail 'ms-cli does not pin mnemonic-io-lib = "0.1.2"'
[ "$(grep -A2 -x 'name = "mnemonic-io-lib"' Cargo.lock | tr '\n' '|')" = 'name = "mnemonic-io-lib"|version = "0.1.2"|source = "registry+https://github.com/rust-lang/crates.io-index"|' ] || fail "Cargo.lock does not resolve io-lib 0.1.2 from crates.io"
! grep -q 'git+https://github.com/bg002h/mnemonic-engrave' Cargo.lock || fail "Cargo.lock still has an io-lib git source"
! grep -q '^mnemonic-io-lib|' ci/repro/vendor-git-sources.txt || fail "vendor-git-sources.txt still has the io-lib row"
! grep -vE '^[[:space:]]*#' .github/workflows/man-release.yml | grep -qE 'IO_LIB_REV|io_lib_rev|git_source_|mnemonic-engrave\?rev' || fail "man-release.yml still derives an io-lib git source"
! grep -qE 'IO_LIB_REV|mnemonic-engrave\?rev' docs/verify-reproducibility.md ci/bh1119-doc-lint.test.sh || fail "a doc or lint still derives an io-lib git source"
```

then the same test binary `27 passed`, then the CI set:
`cargo +1.95.0 fmt --all -- --check`, `cargo nextest run --locked --workspace --all-targets --all-features
--no-fail-fast --build-jobs 6 -j 6` (expect `786 + 27` run, 0 failed), `cargo test --locked -p ms-codec --doc`,
`cargo clippy --locked -p ms-codec --all-targets -- -D warnings`, `cargo clippy --locked --all-targets -p ms-cli -- -D
warnings`, `cargo test --locked -p ms-cli --bin ms mlock::tests::g2_1 -- --include-ignored` (and g2_3, g2_4),
`cargo test --locked -p ms-cli --test history_purge -- --include-ignored`, `--test the_decline`, `--test
mlock_g6_invariant` (each `--include-ignored`), `bash ci/docs-only.test.sh`, `bash ci/bh1119-doc-lint.test.sh`,
`bash ci/code-docs-check.sh`, `(cd design && sha256sum -c display-grouping-vectors.tsv.sha256)`,
`sha256sum --check --strict ci/repro/shared-vendor.sha256`, `bash ci/repro/vendor-freshness.sh` (network for its
check 4), `bash ci/repro/vendor-freshness.test.sh`, `cargo check --locked --target x86_64-unknown-freebsd -p ms-cli`
(when the target is installed), and `MS=target/debug/ms bash design/FIX_bughunt1119_ms_out_writes/red.sh` with each
row's fixed cell grepped. Not covered: the toolkit manual edit (M1, M2; a separate toolkit change), the man-release
and release workflows (CI only; their `--offline` vendored build is the real check of the pin), the macOS, musl,
nightly, miri and fuzz legs, the WORDING_REVIEW rows, a workflow_dispatch run of man-release.yml (the repro gate is the real check of
the two-block form; the builder cannot dispatch it from a draft PR, the coordinator does after the merge).

## 8. Overlaps

- Open ms branch `fix/ms-printed-commands` (2 commits) touches none of `out.rs`, `error.rs`, `encode.rs`,
  `hashlock.rs`, `repair.rs`, `split.rs`, `Cargo.toml` (checked with `git diff --stat`).
- The other io-lib consumers take 0.1.2 in their own rows (md #9, mk #10, mt #12, me #13, tk #14).

## 9. Out of scope (filed, not fixed here)

- **Input aliasing (Brian Q2, convention G4):** `ms encode --in S --out S` (and `--phrase - < S --out S`) replaces the
  phrase file with the card at exit 0; `ms repair --ms1 - --out C < C` replaces the input with the corrected card
  (exit 4); `ms hashlock --in P --out P` exits 0. Base evidence: red.sh x1-x4. As for md (architect 1124 F2) it touches
  every ms `--in` verb, so it is its own row; proposed FOLLOWUPS line in the author's report.
- **io-lib's direct path truncates a regular file reached through a standard-stream name:** `ms encode --hex - --out
  /dev/stdout >> S` leaves S holding only the card at exit 0 (S's earlier lines are gone), on the base and on 0.1.2
  (`write_direct`, `write.rs:101-121`: `set_len(0)` without O_APPEND). Seen by the author; architect 1124 listed it as an
  unverified lead. Owner io-lib. The io-lib release the builder pins (0.1.2 or the fixed one) decides which of o8's two
  cells is seen; the rule R2 is the same.
- **Q3 replace notice** (one stderr line naming a replaced file): waits for io-lib 0.1.3 (architect 1124 s5).

## 10. FOLLOWUPS

Closes `[ms-create-new-partial-file-blocks-retry]` (R4). The closing line and the new lines are listed in the author's
report and direct-pushed after the merge (no FOLLOWUPS edit in this PR).

## 11. Q1 (RULED): registry pin. The coordinator ruled option (a) (2026-11-24): the registry
pin with the edits of R7 inside the PR, said in its description (a .github change in a normally reviewed PR;
RULES.md allows it).
