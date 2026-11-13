# FIX-ms-output-sink-2026-11: a stdout that cannot take ms's output, and a reserved-tag exit 3 that says nothing (crash sweep F1 family and E4, Important)

Small-fix track. Repo: mnemonic-secret (branch `fix/ms-output-sink`, from
master `548da55a4b709442be3391ec8d44a1b74f1aa61b`), plus one section in the
toolkit manual `docs/manual/src/40-cli-reference/43-ms.md` in lockstep (a
separate toolkit change; the ms build does not edit the toolkit).

Inputs: coordinator `agent-reports/grade-ms-md-output-errors.md` (sections 1, 2,
4 and 5; these bind) and, from the same grade, the built and run fix
`ms/fix.diff` and rows `ms/rows.diff` (copied byte for byte into section 3 and
section 4 below), the rule `agent-reports/grade-mk-crash-sweep-1.md` Q4
(L102-124), and for E4 `agent-reports/architect-crash-sweep-exit-sets.md` Q4
(L176-191). The same fix in mnemonic-key is `design/FIX_mk_output_sink.md`.

Everything below is transcribed from the grade, with the read-only stdout probe
of the architect ruling (coordinator `agent-reports/architect-output-sink-read-only-stdout.md`) added. The grade built its diff and the
architect built and ran this exact one both ways (RED: 11 tests run, 1 passed,
10 failed; GREEN: workspace 679 tests run, 679 passed, 23 skipped; the grade's
18 plants die, and the architect's two). Where the grade did
not decide a point that a holder can see, it is a STOP in section 10, not a
choice.

**No STOP is open.** S1 is ruled (section 8). Section 10 records two decisions
(P-1, P-2, both ruled) on points of process and scope the grade left unsaid; neither
changes a text, a code or a row a holder sees.

## 1. Spec delta (exact texts)

There is no change to the wire spec: this is CLI behaviour only. The
holder-visible texts are these, all from the grade:

1. Stderr line (grade section 5 "Exact texts"; rule Q4 (b)), printed once, at
   exit, with a newline:

   > ms: cannot write the output: <error>

   `<error>` is the `io::Error`'s Display, e.g.
   `No space left on device (os error 28)`. The prefix is `ms:`, as in mr
   (`45-mr.md:48`) and mk, not ms's own `error:`.

2. E4, stderr line for a reserved-tag ms1 in `decode`, `derive`, `hashlock` and
   `repair` (text mode; exit stays 3):

   > error: tag "seed" reserved-not-emitted in v0.1; deferred to v0.2+

   (`seed` is the example tag of the grade; the line is ms's existing
   `FutureFormat` error text, now printed for these four verbs.)

3. CHANGELOG, `CHANGELOG.md`, under `## ms-cli [Unreleased]`, a new `### Fixed`
   (it appears verbatim in the diff of section 3.1):

   > ### Fixed
   > - **A stdout that cannot take the output no longer panics or reports success.**
   >   Most verbs wrote with `println!`, which panicked (exit 101, "failed printing
   >   to stdout") on a full disk or `> /dev/full`; `split`, `derive`, `hashlock`,
   >   `decode` of a hashlock preimage plate, `--help` and `--version` dropped the
   >   error and exited 0 with the output lost. Now any stdout write error other
   >   than a closed pipe prints `ms: cannot write the output: <error>` on stderr
   >   and exits 1, also when a verb's `--out FILE` was already written (the file
   >   stays); a verb that was already failing keeps its own code (`verify`'s 3 and
   >   4 included). `repair`'s 4 is a result, not a failure: a candidate that never
   >   reached stdout exits 1. A reader that closes the pipe early (`ms gui-schema |
   >   head -1`) only stops the output, and the exit code is the verb's own, where
   >   0.20.3 could panic with exit 101. Text on stderr (the engraving card, the
   >   advisories) is printed as before. `tests/output_errors.rs`.
   > - **A reserved-tag ms1 now says why it exits 3.** `decode`, `derive`,
   >   `hashlock` and `repair` exited 3 with nothing on stdout or stderr for an ms1
   >   whose tag is reserved for a later format (`seed`, for example). They now
   >   print `error: tag "seed" reserved-not-emitted in v0.1; deferred to v0.2+`
   >   on stderr; the exit code is still 3. `verify` is unchanged: it prints
   >   `OK: valid future format (v0.2+, tag seed)` on stdout and exits 3.
   >   `tests/future_format_names_the_problem.rs`.
   >

4. Manual section (section 8).

Version: **no bump** (grade section 5: no test or CI rule forces one, there is
no `version_is_honest` test and no `Unreleased` check under tests, ci or
`.github`, and ms bumps in `release:` commits). The next release is the patch
ms-cli 0.20.4 ("Fixed" only). `Cargo.toml` and `Cargo.lock` are not touched.

## 2. Behaviour, refusals, exit codes, error variants

The rule (grade section 5 and rule Q4 (a) to (c)). One write path records the
first stdout error; `--help` and `--version` go through the same path. A verb's
code is decided by its return: an `Ok(code)` return is a run whose result is
the stdout output (0, and `repair`'s 4); an `Err` return is a failing code.

- Closed reader (`ErrorKind::BrokenPipe`): the output stops, nothing is
  printed about it, the exit code is the verb's own (no death by signal; ms did
  not restore SIGPIPE to default and does not now).
- Any other stdout write error, from a verb or `--help`/`--version`/`help`: the
  output stops; at exit `ms: cannot write the output: <error>` is printed on
  stderr. An `Ok` code (0, and `repair`'s 4) becomes **1**; an `Err` code is
  kept (1, 2, 3, 4 and 64 included, so `verify`'s 3 and its mismatch 4 stay).
  A stdout opened read-only (`ms vectors 1</dev/null`) is such an error: std's
  stdout reports EBADF as a successful write, so the sink checks fd 1 once
  (`fcntl(F_GETFL)`, unix only) at the first write or `--help`/`--version` and
  records `Bad file descriptor (os error 9)`. A verb that writes nothing to
  stdout is not touched.
- Stderr write errors are ignored (this covers `2>&1 | head`). Stderr text
  keeps being printed after stdout is lost; the rule stops stdout only
  (grade section 2, ruling 5).
- E4: the text-mode skip of the `FutureFormat` stderr line applies to `verify`
  only. JSON mode is unchanged (the error envelope is printed).
- No new exit code, no new error variant, no flag change.

| # | situation | stdout | stderr | exit | row |
|---|---|---|---|---|---|
| 1 | closed reader, verb's code 0 (`gui-schema`, `vectors`, `vectors --pretty`, `encode`, `decode`, `derive`, `split`, `hashlock`) | stops, no text | no `cannot write`, no panic | 0 | `closed_reader_exits_with_the_verbs_code` |
| 1b | closed reader, `repair` of a damaged card (candidate, code 4) | stops | none | 4 (not 1, no signal) | `closed_reader_exits_with_the_verbs_code` |
| 1c | closed reader, `verify --json` that fails (`--phrase x`, an `Err`) | stops | none | 1 (its own) | `closed_reader_exits_with_the_verbs_code` |
| 2 | other stdout error (`/dev/full`, ENOSPC), verb's code 0: every verb in text and `--json` (the 29 invocations of the row, among them `split`, `derive`, `hashlock`) | stops | `ms: cannot write the output: No space left on device (os error 28)` last line | 1 | `full_stdout_exits_1_with_the_line` |
| 2a | the same, the verbs that exited 0 with the output lost before: `split`, `derive`, `hashlock` (text and `--json`), `decode` of a hashlock preimage plate (text and `--json`), `--help`, `--version`; and `decode` of an entropy plate (panic, 101, before) | stops | the line | 1 | `full_stdout_is_not_a_silent_success` |
| 2b | the same, `repair` of a damaged card (code 4 on a good stdout) | stops | the line | 1 (not 4) | `full_stdout_after_a_repair_exits_1_not_4` |
| 3 | the same, verb already failing (an `Err`): `decode --json ms1notastring` (envelope on stdout) | stops | the line | 1 (kept) | `full_stdout_keeps_a_failing_code` |
| 3b | the same, `verify --json` with a phrase mismatch | stops | the line | 4 (kept) | `full_stdout_keeps_a_failing_code` |
| 3c | the same, `verify` of a reserved-tag ms1 (its OK line is lost) | stops | the line | 3 (kept) | `full_stdout_keeps_a_failing_code` |
| 3d | the same, `hashlock --random --json` (a usage error, envelope on stdout) | stops | the line | 64 (kept) | `full_stdout_keeps_a_failing_code` |
| 4 | `--help` / `--version` / `help` / `decode --help`, closed reader | stops | none | 0 | `closed_reader_exits_with_the_verbs_code` (`--help`, `--version`) |
| 5 | `--help` / `--version` / `help` / `decode --help`, other stdout error | stops | the line | 1 | `full_stdout_exits_1_with_the_line` |
| 5b | stdout opened read-only (`1</dev/null`): `vectors`, `gui-schema`, `decode`, `--help`, `--version` | lost (before: lost with exit 0) | `ms: cannot write the output: Bad file descriptor (os error 9)` last line | 1 | `read_only_stdout_exits_1_with_the_line` |
| 5c | the same, `encode --out FILE` (writes nothing to stdout) | none | no `cannot write` | 0 | `read_only_stdout_exits_1_with_the_line` |
| 6 | stderr write error (stdout and stderr both closed: `encode`, `decode`, `repair` of a damaged card) | as case 1 | ignored | the verb's own (0, 0, 4) | `closed_reader_on_stdout_and_stderr_exits_the_verbs_code` |
| 7 | `--out FILE` written, then stdout lost (`hashlock --out FILE`, the public digest record is on stdout) | stops | the `THIS CARD CARRIES THE PREIMAGE` card is still printed, then the line last | 1; the file stays and holds the card (`ms decode FILE` rebuilds the record) | `full_stdout_after_an_out_file_exits_1_and_keeps_the_file` |
| 8 (E4) | reserved-tag ms1 (`seed`) into `decode -`, `derive -`, `hashlock -`, `repair --ms1 -` | empty | `error: tag "seed" reserved-not-emitted in v0.1; deferred to v0.2+` last | 3 (unchanged) | `a_reserved_tag_exits_3_with_the_error_line` |
| 8b (E4) | reserved-tag ms1 into `verify -` (text) | `OK: valid future format (v0.2+, tag seed)` | empty | 3 (unchanged) | `verify_says_ok_on_stdout_and_nothing_on_stderr` |
| 9 | clap parse error (printed on stderr) | none | clap's text; a stderr failure is ignored | 64 (unchanged) | none new (no change in behaviour) |

In text mode `encode --out` and `split --out` write nothing to stdout, so
they cannot reach case 7 (grade section 2, ruling 4). The grade measured on the
fixed build `ms repair ... 2>/dev/full` exits 4 where head exits 101 (grade
section 1, ms-d); the row for a stderr error is case 6.

Grades (grade section 1): ms-a full stdout panic Important; ms-b silent exit 0
Important; ms-c closed-reader panic Minor (fixed here at no cost); ms-d stderr
write panic Important; R1 (triage rows that passed without testing what they
named, now pinned exactly) Important, fixed inline; E4 Important (architect, not
re-graded).

## 3. Production changes (exact)

The eight blocks 3.1 to 3.8 are `fix.diff` split by file, in the diff's own
order, byte for byte: concatenated they equal the grade's file
(sha256 `f739e9e69bc414001b5fb91e73c77d4fcf22a6800232ecf90fcc232f51bf09ec`, checked by the gate). Apply them with `git apply`. They
touch `src/sink.rs` (new, 120 lines), `src/main.rs`, `cmd/decode.rs`,
`cmd/derive.rs`, `cmd/hashlock.rs`, `cmd/split.rs`, `CHANGELOG.md` and
`design/FOLLOWUPS.md`. The FOLLOWUPS block (3.8) is part of the grade's file and
carries `resolved <COMMIT>` as the grade left it (P-1, section 10).

### 3.1 `CHANGELOG.md`

```diff
diff --git c/CHANGELOG.md i/CHANGELOG.md
index afc8347..8b9c7ee 100644
--- c/CHANGELOG.md
+++ i/CHANGELOG.md
@@ -6,6 +6,28 @@ The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
 
 ## ms-cli [Unreleased]
 
+### Fixed
+- **A stdout that cannot take the output no longer panics or reports success.**
+  Most verbs wrote with `println!`, which panicked (exit 101, "failed printing
+  to stdout") on a full disk or `> /dev/full`; `split`, `derive`, `hashlock`,
+  `decode` of a hashlock preimage plate, `--help` and `--version` dropped the
+  error and exited 0 with the output lost. Now any stdout write error other
+  than a closed pipe prints `ms: cannot write the output: <error>` on stderr
+  and exits 1, also when a verb's `--out FILE` was already written (the file
+  stays); a verb that was already failing keeps its own code (`verify`'s 3 and
+  4 included). `repair`'s 4 is a result, not a failure: a candidate that never
+  reached stdout exits 1. A reader that closes the pipe early (`ms gui-schema |
+  head -1`) only stops the output, and the exit code is the verb's own, where
+  0.20.3 could panic with exit 101. Text on stderr (the engraving card, the
+  advisories) is printed as before. `tests/output_errors.rs`.
+- **A reserved-tag ms1 now says why it exits 3.** `decode`, `derive`,
+  `hashlock` and `repair` exited 3 with nothing on stdout or stderr for an ms1
+  whose tag is reserved for a later format (`seed`, for example). They now
+  print `error: tag "seed" reserved-not-emitted in v0.1; deferred to v0.2+`
+  on stderr; the exit code is still 3. `verify` is unchanged: it prints
+  `OK: valid future format (v0.2+, tag seed)` on stdout and exits 3.
+  `tests/future_format_names_the_problem.rs`.
+
 ## ms-cli [0.20.3] — 2026-11-04
 
 No functional change since 0.20.2. Only documentation files changed (README,
```

### 3.2 `crates/ms-cli/src/cmd/decode.rs`

```diff
diff --git c/crates/ms-cli/src/cmd/decode.rs i/crates/ms-cli/src/cmd/decode.rs
index 6ebf07f..4872a12 100644
--- c/crates/ms-cli/src/cmd/decode.rs
+++ i/crates/ms-cli/src/cmd/decode.rs
@@ -223,7 +223,7 @@ pub(crate) fn emit_preimage(x: &[u8; 32], json: bool) -> crate::error::Result<u8
         .map(|k| (k.token(), hex::encode(k.digest(x).as_slice())))
         .collect();
     let hx = hex::encode(x);
-    let mut out = std::io::stdout().lock();
+    let mut out = crate::sink::Out;
     if json {
         let digests: serde_json::Map<String, serde_json::Value> = by_kind
             .iter()
@@ -266,7 +266,6 @@ pub(crate) fn emit_preimage(x: &[u8; 32], json: bool) -> crate::error::Result<u8
         )
         .ok();
     }
-    drop(out);
     let mut err = std::io::stderr().lock();
     crate::advisory::emit_output_class_advisory(
         crate::advisory::OutputClass::PrivateKeyMaterial,
```

### 3.3 `crates/ms-cli/src/cmd/derive.rs`

```diff
diff --git c/crates/ms-cli/src/cmd/derive.rs i/crates/ms-cli/src/cmd/derive.rs
index 51033cd..8862709 100644
--- c/crates/ms-cli/src/cmd/derive.rs
+++ i/crates/ms-cli/src/cmd/derive.rs
@@ -569,7 +569,7 @@ pub fn run(mut args: DeriveArgs) -> Result<u8> {
             .map_err(|e| CliError::BadInput(format!("json serialization: {e}")))?;
         println!("{s}");
     } else {
-        let mut stdout = std::io::stdout();
+        let mut stdout = crate::sink::Out;
         writeln!(stdout, "master_fingerprint:  {master_fp}").ok();
         if let Some((path, xpub)) = &account {
             writeln!(stdout, "account_path:        {path}").ok();
```

### 3.4 `crates/ms-cli/src/cmd/hashlock.rs`

```diff
diff --git c/crates/ms-cli/src/cmd/hashlock.rs i/crates/ms-cli/src/cmd/hashlock.rs
index c91eaf0..466343e 100644
--- c/crates/ms-cli/src/cmd/hashlock.rs
+++ i/crates/ms-cli/src/cmd/hashlock.rs
@@ -445,7 +445,7 @@ pub fn run(args: HashlockArgs) -> Result<u8> {
         _ => None,
     };
 
-    let mut stdout = std::io::stdout().lock();
+    let mut stdout = crate::sink::Out;
     if args.json {
         let mut o = serde_json::Map::new();
         o.insert("digest".into(), hex(h).into());
@@ -502,7 +502,6 @@ pub fn run(args: HashlockArgs) -> Result<u8> {
     } else {
         writeln!(stdout, "{record}").ok();
     }
-    drop(stdout);
 
     let mut stderr = std::io::stderr().lock();
     if !args.no_engraving_card {
```

### 3.5 `crates/ms-cli/src/cmd/split.rs`

```diff
diff --git c/crates/ms-cli/src/cmd/split.rs i/crates/ms-cli/src/cmd/split.rs
index 1cf5210..2f7bad7 100644
--- c/crates/ms-cli/src/cmd/split.rs
+++ i/crates/ms-cli/src/cmd/split.rs
@@ -226,8 +226,7 @@ fn emit_json(
 /// (mirrors the engraving-card panel). Emit ALL stdout shares first, then the
 /// stderr labels.
 fn emit_text(shares: &[String], group_size: usize, separator: char) {
-    let stdout = std::io::stdout();
-    let mut out = stdout.lock();
+    let mut out = crate::sink::Out;
     for share in shares {
         let _ = writeln!(out, "{}", render_grouped(share, group_size, separator));
     }
```

### 3.6 `crates/ms-cli/src/main.rs`

```diff
diff --git c/crates/ms-cli/src/main.rs i/crates/ms-cli/src/main.rs
index 24ddef4..8fc66f0 100644
--- c/crates/ms-cli/src/main.rs
+++ i/crates/ms-cli/src/main.rs
@@ -5,6 +5,10 @@
 
 #![allow(missing_docs)] // ms-cli is binary-only; field-level docs are pretty but not load-bearing for a non-published lib API. Mirror md-cli precedent at crates/md-cli/src/main.rs:1.
 
+// First, so its print macros shadow std's in every module below.
+#[macro_use]
+mod sink;
+
 mod advisory;
 mod argv_guard;
 mod bip39_friendly;
@@ -232,10 +236,18 @@ fn main() -> ExitCode {
             // catch-all 64 below preserves SPEC §6's carve-out for *real*
             // parse errors (overrides clap's default of 2 to keep 2
             // reserved for ms1 format violations).
-            e.print().ok();
+            // --help and --version go to stdout under the same write-error
+            // rule as the verbs (`sink`); a parse error goes to stderr.
+            if e.use_stderr() || !sink::read_only() {
+                if let Err(w) = e.print() {
+                    if !e.use_stderr() {
+                        sink::failed(&w);
+                    }
+                }
+            }
             return match e.kind() {
                 clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => {
-                    ExitCode::SUCCESS
+                    ExitCode::from(sink::exit_code(0, true))
                 }
                 _ => ExitCode::from(64),
             };
@@ -243,6 +255,7 @@ fn main() -> ExitCode {
     };
 
     let json_mode = is_json_mode(&cli.command);
+    let verify_mode = matches!(cli.command, Command::Verify(_));
 
     let result: Result<u8> = match cli.command {
         Command::Derive(args) => cmd::derive::run(args),
@@ -259,11 +272,11 @@ fn main() -> ExitCode {
         Command::Combine(args) => cmd::combine::run(args),
     };
 
-    let exit = match result {
-        Ok(code) => ExitCode::from(code),
+    let (code, success) = match result {
+        Ok(code) => (code, true),
         Err(e) => {
-            emit_error(&e, json_mode);
-            ExitCode::from(e.exit_code())
+            emit_error(&e, json_mode, verify_mode);
+            (e.exit_code(), false)
         }
     };
 
@@ -272,7 +285,7 @@ fn main() -> ExitCode {
     // when failure_count == 0. Runs on both Ok and Err paths.
     mlock::report_at_exit();
 
-    exit
+    ExitCode::from(sink::exit_code(code, success))
 }
 
 fn is_json_mode(cmd: &Command) -> bool {
@@ -292,14 +305,17 @@ fn is_json_mode(cmd: &Command) -> bool {
     }
 }
 
-fn emit_error(e: &CliError, json_mode: bool) {
+fn emit_error(e: &CliError, json_mode: bool, verify_mode: bool) {
     // Special case: FutureFormat is a "success-shaped" exit-3 path used by
     // verify. In text mode, cmd::verify::emit_future_format already wrote the
     // "OK: valid future format" line to stdout; emitting an "error: ..."
     // message to stderr here would contradict that. Skip the stderr write.
+    // Only verify prints that line: decode, derive, hashlock and repair print
+    // nothing, so for them the error line is the only output (E4,
+    // `ms-future-format-silent-exit-3`).
     // In JSON mode we DO want the error envelope (cmd handler suppressed its
     // own stdout output specifically so this path produces the envelope).
-    if matches!(e, CliError::FutureFormat { .. }) && !json_mode {
+    if matches!(e, CliError::FutureFormat { .. }) && !json_mode && verify_mode {
         return;
     }
 
```

### 3.7 `crates/ms-cli/src/sink.rs (new)`

```diff
diff --git c/crates/ms-cli/src/sink.rs i/crates/ms-cli/src/sink.rs
new file mode 100644
index 0000000..1fc5534
--- /dev/null
+++ i/crates/ms-cli/src/sink.rs
@@ -0,0 +1,120 @@
+//! The one stdout path. Every `print!`/`println!` in ms-cli resolves to the
+//! macros below (declared `#[macro_use]` first in `main.rs`, so they shadow
+//! std's in every module), and so do `eprint!`/`eprintln!`. The few verbs that
+//! write with `writeln!(stdout, ..)` take [`Out`] instead of
+//! `std::io::stdout()`.
+//!
+//! std's macros panic (exit 101) when a write fails, and `.ok()` on a
+//! `writeln!` hides the failure (exit 0 with the output lost). Here a write
+//! error stops the output instead. A reader that closed the pipe
+//! (`ms gui-schema | head -1`, `ErrorKind::BrokenPipe`) only stops it: the
+//! exit code is the verb's own. Any other stdout error (a full disk,
+//! `> /dev/full`) is kept, and `main` prints "ms: cannot write the output:
+//! <error>" and exits 1 (crash sweep F1, grade-mk-crash-sweep-1 Q4). A stderr
+//! write error is ignored. (Named `sink`, not `out`: `out.rs` is `--out FILE`.)
+//! `tests/output_errors.rs` pins both cases and that no other file of the
+//! crate names stdout.
+
+use std::io::Write;
+use std::sync::atomic::{AtomicBool, Ordering};
+use std::sync::OnceLock;
+
+static CLOSED: AtomicBool = AtomicBool::new(false);
+static WRITE_ERROR: OnceLock<String> = OnceLock::new();
+
+/// Writes to stdout; after the first failure, writes nothing.
+pub(crate) fn stdout_write(args: std::fmt::Arguments<'_>) {
+    let _ = Out.write_fmt(args);
+}
+
+/// The stdout handle for `write!`/`writeln!`: never returns an error, records it.
+pub(crate) struct Out;
+
+impl Write for Out {
+    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
+        if !CLOSED.load(Ordering::Relaxed) && !read_only() {
+            let mut o = std::io::stdout().lock();
+            if let Err(e) = o.write_all(buf).and_then(|()| o.flush()) {
+                failed(&e);
+            }
+        }
+        Ok(buf.len())
+    }
+    fn flush(&mut self) -> std::io::Result<()> {
+        Ok(())
+    }
+}
+
+/// std's stdout reports EBADF as a successful write, so a stdout opened
+/// read-only (`1</dev/null`) would lose the output with exit 0. Checked once,
+/// at the first write (or clap's `--help`/`--version`): then EBADF is recorded
+/// as the write error. A verb that writes nothing to stdout is not touched.
+pub(crate) fn read_only() -> bool {
+    static RO: OnceLock<bool> = OnceLock::new();
+    *RO.get_or_init(|| {
+        #[cfg(unix)]
+        {
+            // SAFETY: fcntl(F_GETFL) takes no pointer; a bad fd returns -1.
+            let fl = unsafe { libc::fcntl(1, libc::F_GETFL) };
+            let ro = fl >= 0 && (fl & libc::O_ACCMODE) == libc::O_RDONLY;
+            if ro {
+                failed(&std::io::Error::from_raw_os_error(libc::EBADF));
+            }
+            ro
+        }
+        // Windows: std turns only an invalid handle into a success, so a
+        // read-only handle's error reaches the sink as written.
+        #[cfg(not(unix))]
+        {
+            false
+        }
+    })
+}
+
+/// Records a failed stdout write (also used for clap's `--help`/`--version`).
+pub(crate) fn failed(e: &std::io::Error) {
+    CLOSED.store(true, Ordering::Relaxed);
+    if e.kind() != std::io::ErrorKind::BrokenPipe {
+        let _ = WRITE_ERROR.set(e.to_string());
+    }
+}
+
+/// Writes to stderr, ignoring a failure.
+pub(crate) fn stderr_write(args: std::fmt::Arguments<'_>) {
+    let _ = std::io::stderr().lock().write_fmt(args);
+}
+
+/// The process exit code: `code`, unless a stdout write failed for a reason
+/// other than a closed pipe. Then the error is printed, and a code that says
+/// the output is the result (`success`: an `Ok` return, 0 or `repair`'s 4)
+/// becomes 1; a failing code (an `Err`) is kept.
+pub(crate) fn exit_code(code: u8, success: bool) -> u8 {
+    match WRITE_ERROR.get() {
+        None => code,
+        Some(e) => {
+            stderr_write(format_args!("ms: cannot write the output: {e}\n"));
+            if success {
+                1
+            } else {
+                code
+            }
+        }
+    }
+}
+
+macro_rules! print {
+    ($($a:tt)*) => { $crate::sink::stdout_write(format_args!($($a)*)) };
+}
+macro_rules! println {
+    () => { $crate::sink::stdout_write(format_args!("\n")) };
+    ($($a:tt)*) => { $crate::sink::stdout_write(format_args!("{}\n", format_args!($($a)*))) };
+}
+// Unused today; defined so a future `eprint!` cannot reach std's.
+#[allow(unused_macros)]
+macro_rules! eprint {
+    ($($a:tt)*) => { $crate::sink::stderr_write(format_args!($($a)*)) };
+}
+macro_rules! eprintln {
+    () => { $crate::sink::stderr_write(format_args!("\n")) };
+    ($($a:tt)*) => { $crate::sink::stderr_write(format_args!("{}\n", format_args!($($a)*))) };
+}
```

### 3.8 `design/FOLLOWUPS.md`

```diff
diff --git c/design/FOLLOWUPS.md i/design/FOLLOWUPS.md
index 0f560bc..309838c 100644
--- c/design/FOLLOWUPS.md
+++ i/design/FOLLOWUPS.md
@@ -1176,3 +1176,20 @@ deferred-minor. From the toolkit-side review report `agent-reports/whole-diff-ar
 - **Spec:** mnemonic-toolkit `design/SPEC_vendor_freshness_shared.md`.
 - **Status:** resolved -- shipped by this PR
 - Companion: mnemonic-toolkit design/FOLLOWUPS.md::vendor-freshness-shared
+
+### `ms-output-sink` — a stdout write error panicked (exit 101) or was dropped (exit 0, output lost) (repo: **mnemonic-secret**; owning phase: this PR)
+
+- **Surfaced:** 2026-11-13, the crash sweep's F1 family rule (coordinator `agent-reports/grade-mk-crash-sweep-1.md` Q4) and the ms triage (`agent-reports/triage-ms-output-errors.md`); graded in `agent-reports/grade-ms-md-output-errors.md`: full stdout Important, silent exit 0 Important, closed reader Minor (fixed here at no extra cost).
+- **Where:** every `println!`/`print!` of `crates/ms-cli/src`, the four direct writers (`cmd/decode.rs` `emit_preimage`, `cmd/split.rs` `emit_text`, `cmd/derive.rs` text, `cmd/hashlock.rs`), and clap's `e.print().ok()` in `main.rs`.
+- **What:** One stdout path, `src/sink.rs` (`out.rs` is `--out FILE`): a closed pipe stops the output and keeps the verb's code; any other stdout error prints `ms: cannot write the output: <error>` and exits 1 for an `Ok` return (0, `repair`'s 4), keeping an `Err` return's code (1, 2, 3, 4, 64); a stderr error is ignored. Rows: `tests/output_errors.rs`.
+- **Companion:** mnemonic-toolkit `design/FOLLOWUPS.md::ms-manual-when-stdout-fails` (the `43-ms.md` section; it lands with the toolkit PR that pins the first ms-cli release carrying this fix).
+- **Status:** `resolved <COMMIT>`
+- **Tier:** `cross-repo`
+
+### `ms-future-format-silent-exit-3` — a reserved-tag ms1 made decode/derive/hashlock/repair exit 3 with no output (repo: **mnemonic-secret**; owning phase: this PR)
+
+- **Surfaced:** 2026-11-13, coordinator `agent-reports/architect-crash-sweep-exit-sets.md` Q4 E4 (Important: the tool did not name the problem).
+- **Where:** `crates/ms-cli/src/main.rs` `emit_error`: the text-mode skip for `CliError::FutureFormat`.
+- **What:** The skip exists because `verify` prints its own `OK: valid future format` line on stdout (`cmd/verify.rs` `emit_future_format`); `decode`, `derive`, `hashlock` and `repair` print nothing, so a reserved-tag ms1 (`seed`) exited 3 with stdout and stderr empty. The skip now applies to `verify` only; the other four print `error: tag "seed" reserved-not-emitted in v0.1; deferred to v0.2+`. Rows: `tests/future_format_names_the_problem.rs`.
+- **Status:** `resolved <COMMIT>`
+- **Tier:** `v1+`
```


## 4. RED rows

The rows are `fix`'s `rows.diff`, byte for byte: two new files, concatenated
the grade's file (sha256 `5272296ac211a93096f0878fbed588fb8494ebcdbe2b849ee9f7e87002978831`). Apply them
first, on the unmodified base, then run each command in the `sh` block, from
the repo root with `$HOME/.cargo/bin` first in PATH (the pin is 1.85.0). The
text after `## expect:` is a substring of the failing run's output.

```diff
diff --git c/crates/ms-cli/tests/future_format_names_the_problem.rs i/crates/ms-cli/tests/future_format_names_the_problem.rs
new file mode 100644
index 0000000..5a40561
--- /dev/null
+++ i/crates/ms-cli/tests/future_format_names_the_problem.rs
@@ -0,0 +1,63 @@
+//! A reserved-tag (future format) ms1 exits 3 and says why (architect
+//! crash-sweep-exit-sets Q4 E4). Only `verify` prints its own OK line on
+//! stdout, so only `verify` (text mode) leaves stderr without the error line.
+
+use assert_cmd::Command;
+use ms_codec::codex32::{Codex32String, Fe};
+
+const LINE: &str = "error: tag \"seed\" reserved-not-emitted in v0.1; deferred to v0.2+";
+
+fn reserved_tag_ms1() -> String {
+    let mut data = vec![0x00u8];
+    data.extend_from_slice(&[0xAAu8; 16]);
+    Codex32String::from_seed("ms", 0, "seed", Fe::S, &data)
+        .unwrap()
+        .to_string()
+}
+
+#[test]
+fn a_reserved_tag_exits_3_with_the_error_line() {
+    let mut wrong = Vec::new();
+    for args in [
+        vec!["decode", "-"],
+        vec!["derive", "-"],
+        vec!["hashlock", "-"],
+        vec!["repair", "--ms1", "-"],
+    ] {
+        let o = Command::cargo_bin("ms")
+            .unwrap()
+            .args(&args)
+            .write_stdin(reserved_tag_ms1())
+            .output()
+            .unwrap();
+        let err = String::from_utf8_lossy(&o.stderr);
+        if o.status.code() != Some(3) || !o.stdout.is_empty() || err.lines().last() != Some(LINE) {
+            wrong.push(format!(
+                "{args:?}: {:?}, stdout {:?}, stderr {err:?}",
+                o.status.code(),
+                String::from_utf8_lossy(&o.stdout)
+            ));
+        }
+    }
+    assert!(wrong.is_empty(), "exit 3 without the line: {wrong:#?}");
+}
+
+#[test]
+fn verify_says_ok_on_stdout_and_nothing_on_stderr() {
+    let o = Command::cargo_bin("ms")
+        .unwrap()
+        .args(["verify", "-"])
+        .write_stdin(reserved_tag_ms1())
+        .output()
+        .unwrap();
+    assert_eq!(o.status.code(), Some(3));
+    assert_eq!(
+        String::from_utf8_lossy(&o.stdout),
+        "OK: valid future format (v0.2+, tag seed)\n"
+    );
+    assert!(
+        o.stderr.is_empty(),
+        "stderr: {:?}",
+        String::from_utf8_lossy(&o.stderr)
+    );
+}
```

```diff
diff --git c/crates/ms-cli/tests/output_errors.rs i/crates/ms-cli/tests/output_errors.rs
new file mode 100644
index 0000000..b055494
--- /dev/null
+++ i/crates/ms-cli/tests/output_errors.rs
@@ -0,0 +1,364 @@
+//! What ms does when stdout cannot take its output (crash sweep F1 and the
+//! family rule, grade-mk-crash-sweep-1 Q4; the ms/md output-error triage).
+//!
+//! - A reader that closed the pipe only stops the output: no text, no
+//!   signal, the exit code is the verb's own.
+//! - Any other write error (`/dev/full`, ENOSPC), from a verb or `--help`:
+//!   "ms: cannot write the output: <error>" on stderr, exit 1; a verb that
+//!   was already failing keeps its code. A success code (0, and `repair`'s
+//!   4 = VERIFY-ME candidate) becomes 1.
+//! - No file of the crate but `src/sink.rs` names stdout.
+#![cfg(target_os = "linux")]
+
+use std::fs::OpenOptions;
+use std::os::fd::{FromRawFd, OwnedFd};
+use std::os::unix::process::ExitStatusExt;
+use std::process::{Command, Output, Stdio};
+
+const A: &str = "--allow-argv-secret";
+const PHRASE: &str =
+    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
+const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
+// One substitution away from MS1: repair corrects it and exits 4.
+const DAMAGED: &str = "ms10entrsqpqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
+// A hashlock preimage plate: `decode` prints it through its own stdout writer.
+const PREIMAGE: &str =
+    "ms10hashsq02nfauqxa69u0g3zxl250sejulueyjpjd2xygd5snwfa2mjsrphxdd85w8w5fxwvh";
+const SHARE_A: &str = "ms12fj8uqtju0yavfp4glm0acyu6cvquey82qm0j3ar9f2fpxt";
+const SHARE_B: &str = "ms12fj8upvkrzw5jthgunaz5dwr2djqr6w7mqh6qvjfrcsxywt";
+const FULL: &str = "ms: cannot write the output: No space left on device (os error 28)";
+
+fn reserved_tag_ms1() -> String {
+    use ms_codec::codex32::{Codex32String, Fe};
+    let mut data = vec![0x00u8];
+    data.extend_from_slice(&[0xAAu8; 16]);
+    Codex32String::from_seed("ms", 0, "seed", Fe::S, &data)
+        .unwrap()
+        .to_string()
+}
+
+fn ms() -> Command {
+    Command::new(env!("CARGO_BIN_EXE_ms"))
+}
+
+fn s(v: &[&str]) -> Vec<String> {
+    v.iter().map(|x| x.to_string()).collect()
+}
+
+fn closed_pipe() -> OwnedFd {
+    let mut fds = [0i32; 2];
+    assert_eq!(unsafe { libc::pipe(fds.as_mut_ptr()) }, 0);
+    unsafe { libc::close(fds[0]) }; // the reader is gone before ms starts
+    unsafe { OwnedFd::from_raw_fd(fds[1]) }
+}
+
+fn on_full(a: &[String]) -> Output {
+    let full = OpenOptions::new().write(true).open("/dev/full").unwrap();
+    ms().args(a)
+        .stdout(Stdio::from(full))
+        .stderr(Stdio::piped())
+        .output()
+        .unwrap()
+}
+
+fn on_closed(a: &[String], stderr_too: bool) -> Output {
+    let err = if stderr_too {
+        Stdio::from(closed_pipe())
+    } else {
+        Stdio::piped()
+    };
+    ms().args(a)
+        .stdout(Stdio::from(closed_pipe()))
+        .stderr(err)
+        .output()
+        .unwrap()
+}
+
+fn full_case(a: &[String], code: i32) {
+    let o = on_full(a);
+    let err = String::from_utf8_lossy(&o.stderr);
+    assert!(!err.contains("panicked"), "{a:?}: panic: {err}");
+    assert_eq!(
+        o.status.code(),
+        Some(code),
+        "{a:?}: status {:?}: {err}",
+        o.status
+    );
+    assert_eq!(err.lines().last(), Some(FULL), "{a:?}: stderr: {err}");
+}
+
+fn closed_case(a: &[String], code: i32, stderr_too: bool) {
+    let o = on_closed(a, stderr_too);
+    let err = String::from_utf8_lossy(&o.stderr);
+    assert_eq!(
+        o.status.signal(),
+        None,
+        "{a:?}: killed by a signal: {:?}",
+        o.status
+    );
+    assert_eq!(
+        o.status.code(),
+        Some(code),
+        "{a:?}: status {:?}: {err}",
+        o.status
+    );
+    assert!(
+        !err.contains("panicked") && !err.contains("cannot write"),
+        "{a:?}: {err}"
+    );
+}
+
+#[test]
+fn full_stdout_exits_1_with_the_line() {
+    for a in [
+        s(&["gui-schema"]),
+        s(&["vectors"]),
+        s(&["vectors", "--pretty"]),
+        s(&["encode", A, "--phrase", PHRASE]),
+        s(&["encode", A, "--phrase", PHRASE, "--json"]),
+        s(&["decode", A, MS1]),
+        s(&["decode", A, "--json", MS1]),
+        s(&["inspect", A, MS1]),
+        s(&["inspect", A, "--json", MS1]),
+        s(&["inspect", A, SHARE_A]),
+        s(&["verify", A, MS1]),
+        s(&["verify", A, "--json", MS1]),
+        s(&["verify", A, MS1, "--phrase", PHRASE]),
+        s(&["repair", A, "--ms1", MS1]),
+        s(&["repair", A, "--ms1", MS1, "--json"]),
+        s(&["split", A, "-k", "2", "-n", "3", "--phrase", PHRASE]),
+        s(&[
+            "split", A, "-k", "2", "-n", "3", "--phrase", PHRASE, "--json",
+        ]),
+        s(&["combine", A, SHARE_A, SHARE_B]),
+        s(&["combine", A, SHARE_A, SHARE_B, "--to", "entropy"]),
+        s(&["combine", A, SHARE_A, SHARE_B, "--to", "ms1"]),
+        s(&["combine", A, SHARE_A, SHARE_B, "--json"]),
+        s(&["derive", A, "--phrase", PHRASE, "--template", "bip84"]),
+        s(&["derive", A, "--phrase", PHRASE, "--json"]),
+        s(&[
+            "hashlock",
+            A,
+            "--hashlock-phrase",
+            "correct horse battery staple long enough",
+            "--kind",
+            "sha256",
+        ]),
+        s(&[
+            "hashlock",
+            A,
+            "--hashlock-phrase",
+            "correct horse battery staple long enough",
+            "--kind",
+            "sha256",
+            "--json",
+        ]),
+        s(&["--help"]),
+        s(&["--version"]),
+        s(&["decode", "--help"]),
+        s(&["help"]),
+    ] {
+        full_case(&a, 1);
+    }
+}
+
+/// Verbs that swallowed the write error (`.ok()` / `let _ =` on a direct
+/// `std::io::stdout()` writer, or clap's `.print().ok()`) and exited 0 with the
+/// output lost, one case each so the first failure does not mask the rest.
+#[test]
+fn full_stdout_is_not_a_silent_success() {
+    let h = "correct horse battery staple long enough";
+    let cases = [
+        s(&["split", A, "-k", "2", "-n", "3", "--phrase", PHRASE]),
+        s(&["derive", A, "--phrase", PHRASE, "--template", "bip84"]),
+        s(&["hashlock", A, "--hashlock-phrase", h, "--kind", "sha256"]),
+        s(&[
+            "hashlock",
+            A,
+            "--hashlock-phrase",
+            h,
+            "--kind",
+            "sha256",
+            "--json",
+        ]),
+        s(&["decode", A, PREIMAGE]),
+        s(&["decode", A, "--json", PREIMAGE]),
+        s(&["decode", A, MS1]),
+        s(&["--help"]),
+        s(&["--version"]),
+    ];
+    let mut wrong = Vec::new();
+    for a in &cases {
+        let o = on_full(a);
+        if o.status.code() != Some(1) {
+            wrong.push(format!("{a:?} -> {:?}", o.status.code()));
+        }
+    }
+    assert!(wrong.is_empty(), "stdout lost without exit 1: {wrong:#?}");
+}
+
+#[test]
+fn full_stdout_after_a_repair_exits_1_not_4() {
+    let a = s(&["repair", A, "--ms1", DAMAGED]);
+    let o = ms().args(&a).output().unwrap();
+    assert_eq!(o.status.code(), Some(4), "fixture: repair should exit 4");
+    full_case(&a, 1);
+}
+
+#[test]
+fn full_stdout_keeps_a_failing_code() {
+    // A --json failure prints its envelope on stdout; the failing code stays.
+    full_case(&s(&["decode", A, "--json", "ms1notastring"]), 1);
+    // verify's phrase mismatch is a failing 4 (an Err), unlike repair's
+    // candidate 4 (an Ok): it stays 4.
+    let other = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";
+    let a = s(&["verify", A, "--json", MS1, "--phrase", other]);
+    let o = ms().args(&a).output().unwrap();
+    assert_eq!(
+        o.status.code(),
+        Some(4),
+        "fixture: a phrase mismatch exits 4"
+    );
+    full_case(&a, 4);
+    // A reserved-tag (future format) ms1: verify's text OK line is lost, the 3 stays.
+    let a = s(&["verify", A, &reserved_tag_ms1()]);
+    let o = ms().args(&a).output().unwrap();
+    assert_eq!(o.status.code(), Some(3), "fixture: a future format exits 3");
+    full_case(&a, 3);
+    // A usage error in --json mode prints its envelope on stdout: 64 stays.
+    let a = s(&["hashlock", "--random", "--json"]);
+    let o = ms().args(&a).output().unwrap();
+    assert_eq!(o.status.code(), Some(64), "fixture: {o:?}");
+    assert!(
+        String::from_utf8_lossy(&o.stdout).contains("\"Usage\""),
+        "fixture: {o:?}"
+    );
+    full_case(&a, 64);
+}
+
+/// A verb whose `--out FILE` artifact was written and whose stdout record was
+/// then lost exits 1; the file stays.
+#[test]
+fn full_stdout_after_an_out_file_exits_1_and_keeps_the_file() {
+    let f = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("output_errors_hashlock.ms1");
+    let _ = std::fs::remove_file(&f);
+    let a = s(&[
+        "hashlock",
+        A,
+        "--hashlock-phrase",
+        "correct horse battery staple long enough",
+        "--kind",
+        "sha256",
+        "--out",
+        f.to_str().unwrap(),
+    ]);
+    let o = on_full(&a);
+    let err = String::from_utf8_lossy(&o.stderr);
+    assert!(
+        err.contains("THIS CARD CARRIES THE PREIMAGE"),
+        "the stderr card is still printed: {err}"
+    );
+    full_case(&a, 1);
+    let body = std::fs::read_to_string(&f).unwrap();
+    assert!(body.starts_with("ms10hash"), "the --out file: {body:?}");
+}
+
+#[test]
+fn closed_reader_exits_with_the_verbs_code() {
+    for a in [
+        s(&["gui-schema"]),
+        s(&["vectors"]),
+        s(&["vectors", "--pretty"]),
+        s(&["encode", A, "--phrase", PHRASE]),
+        s(&["decode", A, MS1]),
+        s(&["derive", A, "--phrase", PHRASE, "--template", "bip84"]),
+        s(&["split", A, "-k", "2", "-n", "3", "--phrase", PHRASE]),
+        s(&[
+            "hashlock",
+            A,
+            "--hashlock-phrase",
+            "correct horse battery staple long enough",
+            "--kind",
+            "sha256",
+        ]),
+        s(&["--help"]),
+        s(&["--version"]),
+    ] {
+        closed_case(&a, 0, false);
+    }
+    closed_case(&s(&["repair", A, "--ms1", DAMAGED]), 4, false);
+    closed_case(&s(&["verify", A, "--json", MS1, "--phrase", "x"]), 1, false);
+}
+
+#[test]
+fn closed_reader_on_stdout_and_stderr_exits_the_verbs_code() {
+    // encode and decode print stderr advisories after stdout; repair 4 prints
+    // its UNVERIFIED line on stderr. Both streams go to a pipe with no reader.
+    closed_case(&s(&["encode", A, "--phrase", PHRASE]), 0, true);
+    closed_case(&s(&["decode", A, MS1]), 0, true);
+    closed_case(&s(&["repair", A, "--ms1", DAMAGED]), 4, true);
+}
+
+#[test]
+fn only_the_sink_names_stdout() {
+    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
+    let mut bad = Vec::new();
+    let mut stack = vec![dir];
+    while let Some(d) = stack.pop() {
+        for e in std::fs::read_dir(&d).unwrap() {
+            let p = e.unwrap().path();
+            if p.is_dir() {
+                stack.push(p);
+            } else if p.extension().is_some_and(|x| x == "rs") && !p.ends_with("src/sink.rs") {
+                let t = std::fs::read_to_string(&p).unwrap();
+                for n in ["stdout(", "std::print", "std::eprint", "core::print"] {
+                    if t.contains(n) {
+                        bad.push(format!("{}: {n}", p.display()));
+                    }
+                }
+            }
+        }
+    }
+    assert!(bad.is_empty(), "stdout named outside src/sink.rs: {bad:?}");
+}
+
+/// std's stdout reports EBADF as a successful write, so a stdout opened
+/// read-only (`ms vectors 1</dev/null`) lost the output and exited 0.
+#[test]
+fn read_only_stdout_exits_1_with_the_line() {
+    const BADF: &str = "ms: cannot write the output: Bad file descriptor (os error 9)";
+    let on_read_only = |a: &[String]| {
+        let ro = std::fs::File::open("/dev/null").unwrap();
+        ms().args(a)
+            .stdout(Stdio::from(ro))
+            .stderr(Stdio::piped())
+            .output()
+            .unwrap()
+    };
+    for a in [
+        s(&["vectors"]),
+        s(&["gui-schema"]),
+        s(&["decode", A, MS1]),
+        s(&["--help"]),
+        s(&["--version"]),
+    ] {
+        let o = on_read_only(&a);
+        let err = String::from_utf8_lossy(&o.stderr);
+        assert_eq!(o.status.code(), Some(1), "{a:?}: {err}");
+        assert_eq!(err.lines().last(), Some(BADF), "{a:?}: {err}");
+    }
+    // A verb that writes nothing to stdout keeps its code and prints no line.
+    let f = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("output_errors_ro.ms1");
+    let _ = std::fs::remove_file(&f);
+    let o = on_read_only(&s(&[
+        "encode",
+        A,
+        "--phrase",
+        PHRASE,
+        "--out",
+        f.to_str().unwrap(),
+    ]));
+    let err = String::from_utf8_lossy(&o.stderr);
+    assert_eq!(o.status.code(), Some(0), "{err}");
+    assert!(!err.contains("cannot write"), "{err}");
+}
```

```sh
cargo nextest run --locked -j 8 -E 'test(/(^|::)full_stdout_exits_1_with_the_line$/)' ## expect: failed printing to stdout: No space left on device
cargo nextest run --locked -j 8 -E 'test(/(^|::)full_stdout_is_not_a_silent_success$/)' ## expect: stdout lost without exit 1
cargo nextest run --locked -j 8 -E 'test(/(^|::)full_stdout_after_a_repair_exits_1_not_4$/)' ## expect: failed printing to stdout: No space left on device
cargo nextest run --locked -j 8 -E 'test(/(^|::)full_stdout_keeps_a_failing_code$/)' ## expect: failed printing to stdout: No space left on device
cargo nextest run --locked -j 8 -E 'test(/(^|::)full_stdout_after_an_out_file_exits_1_and_keeps_the_file$/)' ## expect: left: Some(0)
cargo nextest run --locked -j 8 -E 'test(/(^|::)closed_reader_exits_with_the_verbs_code$/)' ## expect: failed printing to stdout: Broken pipe (os error 32)
cargo nextest run --locked -j 8 -E 'test(/(^|::)closed_reader_on_stdout_and_stderr_exits_the_verbs_code$/)' ## expect: left: Some(101)
cargo nextest run --locked -j 8 -E 'test(/(^|::)only_the_sink_names_stdout$/)' ## expect: stdout named outside src/sink.rs
cargo nextest run --locked -j 8 -E 'test(/(^|::)a_reserved_tag_exits_3_with_the_error_line$/)' ## expect: exit 3 without the line
cargo nextest run --locked -j 8 -E 'test(/(^|::)read_only_stdout_exits_1_with_the_line$/)' ## expect: left: Some(0)
```

The guard row, which PASSES on the base (the gate requires it):

```text
cargo nextest run --locked -j 8 -E 'test(/(^|::)verify_says_ok_on_stdout_and_nothing_on_stderr$/)'
```

| row | expected after GREEN | on the base code (grade `red.log`) |
|---|---|---|
| `full_stdout_exits_1_with_the_line` | 29 invocations (every verb in text and `--json`, `--help`, `--version`, `decode --help`, `help`) each exit 1 with the line last on stderr, no panic | FAIL: `thread 'main' panicked at library/std/src/io/stdio.rs:1123:9` / `failed printing to stdout: No space left on device (os error 28)` |
| `full_stdout_is_not_a_silent_success` | nine cases (the silent verbs) each exit 1; one case per entry so one failure does not mask the rest | FAIL: `stdout lost without exit 1`: `split`, `derive`, `hashlock` (both), `decode` of the preimage plate (both), `--help`, `--version` give 0; `decode` of an entropy plate 101 |
| `full_stdout_after_a_repair_exits_1_not_4` | the repair fixture exits 4 on a good stdout; on a full stdout exit 1 | FAIL: the same panic |
| `full_stdout_keeps_a_failing_code` | `decode --json` bad card 1, `verify --json` mismatch 4, `verify` of a reserved tag 3, `hashlock --random --json` 64 (each fixture asserted first), each with the line | FAIL: the same panic |
| `full_stdout_after_an_out_file_exits_1_and_keeps_the_file` | the `hashlock --out FILE` run on a full stdout: the stderr card is printed, exit 1 with the line, the file starts with `ms10hash` | FAIL: `left: Some(0)` / `right: Some(1)` (exit 0, output lost) |
| `closed_reader_exits_with_the_verbs_code` | ten verbs and flags exit 0, `repair` of a damaged card 4, `verify --json --phrase x` 1; no signal, no panic, no `cannot write` | FAIL: `failed printing to stdout: Broken pipe (os error 32)`, `left: Some(101)` |
| `closed_reader_on_stdout_and_stderr_exits_the_verbs_code` | `encode` 0, `decode` 0, `repair` 4 with both streams closed | FAIL: `left: Some(101)` |
| `only_the_sink_names_stdout` | PASS: no source file but `src/sink.rs` contains `stdout(`, `std::print`, `std::eprint` or `core::print` | FAIL on the base: `stdout named outside src/sink.rs` (the direct writers: `decode`, `derive`, `hashlock`, `split`) |
| `a_reserved_tag_exits_3_with_the_error_line` | the four verbs exit 3, stdout empty, the E4 line last on stderr | FAIL: `exit 3 without the line`, each `Some(3), stdout "", stderr ""` |
| `read_only_stdout_exits_1_with_the_line` | `vectors`, `gui-schema`, `decode`, `--help`, `--version` on a stdout opened read-only each exit 1 with `ms: cannot write the output: Bad file descriptor (os error 9)` last; `encode --out FILE` exits 0 with no `cannot write` | FAIL (architect, `redbase-row.log`): `left: Some(0)` / `right: Some(1)` (exit 0, output lost) |
| `verify_says_ok_on_stdout_and_nothing_on_stderr` | PASS (exit 3, the OK line, empty stderr) | PASS on the base too: a guard, not a RED row. Its teeth are plant P18. |

Whole-set RED (architect `redbase-set.log`, with `--no-fail-fast`): **11 tests run: 1 passed, 10 failed** (the
passer is the guard). The rows are `cfg(target_os = "linux")` in
`output_errors.rs`; `future_format_names_the_problem.rs` is portable.

### Tests can fail (the grade's 18 plants and the architect's two)

Each plant was made against the fix; each fails at least one row (grade
`plants.txt`, rerun; the gate does not re-run them). P1 to P13 are the triage's
clauses, P14 to P16 the later additions, P17 and P18 are E4. The grade added the
last assertion of `only_the_sink_names_stdout` after the run; it can only add
kills.

| plant | what is planted | fails |
|---|---|---|
| P1 | BrokenPipe is recorded as a write error | `closed_reader_exits_with_the_verbs_code`, `closed_reader_on_stdout_and_stderr_exits_the_verbs_code` |
| P2 | no 0 to 1 mapping (a success code is kept) | `full_stdout_after_an_out_file_exits_1_and_keeps_the_file`, `full_stdout_after_a_repair_exits_1_not_4`, `full_stdout_exits_1_with_the_line`, `full_stdout_is_not_a_silent_success` |
| P3 | `repair`'s 4 is kept on a lost stdout | `full_stdout_after_a_repair_exits_1_not_4` |
| P4 | `--help`/`--version` not routed through the sink | `full_stdout_exits_1_with_the_line`, `full_stdout_is_not_a_silent_success` |
| P5 | a std `eprintln!` | `closed_reader_on_stdout_and_stderr_exits_the_verbs_code` |
| P6 | every code becomes 1 (a failing code is not kept) | `full_stdout_keeps_a_failing_code` |
| P7 | `stdout(` named outside `sink.rs` | `only_the_sink_names_stdout` |
| P8 | one `std::println!` site | `closed_reader_exits_with_the_verbs_code`, `full_stdout_exits_1_with_the_line`, `only_the_sink_names_stdout` |
| P9 | the `cannot write` line is not printed | `full_stdout_after_an_out_file_exits_1_and_keeps_the_file`, `full_stdout_after_a_repair_exits_1_not_4`, `full_stdout_exits_1_with_the_line`, `full_stdout_keeps_a_failing_code` |
| P10 | `derive` writes to `std::io::stdout()` directly | `full_stdout_exits_1_with_the_line`, `full_stdout_is_not_a_silent_success`, `only_the_sink_names_stdout` |
| P11 | `hashlock` writes to `std::io::stdout()` directly | `full_stdout_after_an_out_file_exits_1_and_keeps_the_file`, `full_stdout_exits_1_with_the_line`, `full_stdout_is_not_a_silent_success`, `only_the_sink_names_stdout` |
| P12 | a stderr write error is fatal | `closed_reader_on_stdout_and_stderr_exits_the_verbs_code` |
| P13 | an `Err` return is treated as a success (mapped to 1) | `full_stdout_keeps_a_failing_code` |
| P14 | `verify`'s 3 is treated as a success code (mapped to 1) | `full_stdout_keeps_a_failing_code` |
| P15 | `split` writes to `std::io::stdout()` directly | `full_stdout_exits_1_with_the_line`, `full_stdout_is_not_a_silent_success`, `only_the_sink_names_stdout` |
| P16 | the preimage writer (`emit_preimage`) writes to `std::io::stdout()` directly | `full_stdout_is_not_a_silent_success`, `only_the_sink_names_stdout` |
| P17 | E4 silent for every verb (the skip applies to all) | `a_reserved_tag_exits_3_with_the_error_line` |
| P18 | E4 errors on `verify` too (the skip removed) | `verify_says_ok_on_stdout_and_nothing_on_stderr` |
| P19 | the clap branch not probed (`if true {` for `if e.use_stderr() \|\| !sink::read_only() {`) | `read_only_stdout_exits_1_with_the_line` (at `--help`, `left: Some(0)`) |
| P20 | the probe records no error (its `failed(...EBADF)` line removed) | `read_only_stdout_exits_1_with_the_line` (`left: Some(0)`) |

P19 and P20 were run by the architect against the ruled tree (coordinator `agent-reports/architect-output-sink-read-only-stdout.md`, `plant-P1.log`, `plant-P2.log`). P1 to P18 were run against the grade's diff; P4 plants the clap branch, which the probe now wraps.

## 5. Tests that change

None changes. Two files are added: `crates/ms-cli/tests/output_errors.rs` (9
tests) and `crates/ms-cli/tests/future_format_names_the_problem.rs` (2 tests),
section 4. The triage's own rows are not committed (the grade pinned its two
weak fixtures, R1: the usage-error fixture is now `hashlock --random --json`
with 64, and the verify mismatch is asserted `== 4`).

## 6. Build order

1. Add the rows (section 4) as their own commit; run the RED commands: ten
   FAIL for the stated reasons, the guard row PASSES.
2. Apply sections 3.1 to 3.8 (the grade's diff with the architect's probe). Run the eleven rows: GREEN.
3. Gate (section 7). Then replace `<COMMIT>` in the two FOLLOWUPS entries (P-1).
4. Toolkit manual section (section 8): not part of this PR; it goes in the
   toolkit PR that pins the first ms-cli release carrying this fix (S1, ruled).
5. CI on the PR must show the legs the grade did not run (section 9).

## 7. Gate

`scripts/ms-output-sink-fix-gate.sh` runs the whole of it: it extracts this
document's blocks, checks them against the stated sha256 and the grade's files,
runs every RED row as printed on the base in its own detached worktree, applies
the code blocks, shows the rows and the full workspace suite green, and has a
self-test that goes red on a broken row. Its header states what it does not
cover. The commands it runs, each to a file, then grep:

- `cargo nextest run --locked -j 8` expecting **679 tests run: 679 passed, 23 skipped**
  (architect, `g-suite.log`: base 668 plus the 11 rows).
- `cargo nextest run --locked -j 8 -E 'binary(output_errors) | binary(future_format_names_the_problem)'`: **11 tests run: 11 passed** (a short or empty count is a FAIL).
- `cargo test --locked --workspace --doc`
- `cargo +1.95.0 fmt --all -- --check` (the CI pin; the grade ran it so)
- `cargo clippy --locked --workspace --all-targets -- -D warnings`
- Then `coordinator/scripts/mutation-gate.sh <worktree> <base>` before any review (not in the script).

## 8. Toolkit manual lockstep (43-ms.md), S1 ruled

**S1 (ruled 2026-11-13, coordinator `agent-reports/grade-ms-md-output-errors.md` section 4).**
The section below goes in the toolkit PR that moves `scripts/install.sh`'s ms
pin (`ms-cli` 0.20.2 today, L55-61) to the first ms-cli release carrying this
fix (0.20.4 or later). It goes in the same commit as that pin, and it may not
merge before that pin. It is not the release-F1 PR, which pins ms-cli 0.20.3
(no fix), unless F1 is built after a fixed release and pins it. Plain text, no
`(unreleased: ...)` marker: the manual lint reads that marker only on flag
rows, so a behaviour marker could never fail.

Insert in toolkit `docs/manual/src/40-cli-reference/43-ms.md`, after its last
line (today L890; checked on toolkit master 47baf6f2), these lines, verbatim
from the grade (a blank line, `---`, a blank line, the heading, a blank line,
the paragraph):

```text

---

## When stdout fails

A reader that closes the pipe early (`ms gui-schema | head -1`) only stops the output: the exit code is the verb's own. Any other write error on stdout (a full disk, `> /dev/full`), from a verb or `--help`, prints `ms: cannot write the output: <error>` on stderr and exits 1; a verb that was already failing keeps its own code.
```

No ms flag or help text changes, so the mnemonic-gui ms schema mirror needs no
change. The grade ran the toolkit lint with these texts (the same failure set
before and after: 10 mk markers against the local mk 0.14.0 and 3 lychee errors
from a partial archive); markdownlint and cspell pass.

Toolkit companion entry, as the text a toolkit FOLLOWUPS PR files (the section
above is held verbatim):

> ### `ms-manual-when-stdout-fails` — at the ms re-pin carrying the output sink, `43-ms.md` states what ms does when stdout fails (repo: **mnemonic-toolkit**; owning phase: **the toolkit pin bump to the ms-cli release carrying mnemonic-secret `ms-output-sink`**)
> - Surfaced: coordinator agent-reports/grade-ms-md-output-errors.md section 4 (S1). Companion: mnemonic-secret design/FOLLOWUPS.md::ms-output-sink.
> - What: insert in docs/manual/src/40-cli-reference/43-ms.md, after its last line (today L890): blank, `---`, blank, `## When stdout fails`, blank, the paragraph of section 8 of mnemonic-secret `design/FIX_ms_output_sink.md` verbatim, in the same commit as the scripts/install.sh ms pin. Plain text, no (unreleased: …) marker.
> - Status: `open`.
> - Tier: `docs`, `cross-repo`.

The Companion line in this repo's own entry is in block 3.8 (`ms-output-sink`:
"Companion: mnemonic-toolkit `design/FOLLOWUPS.md::ms-manual-when-stdout-fails`").

## 9. Checks recorded

- **Version:** no bump (section 1).
- **Evidence (grade, then architect for the read-only probe):** RED 11 run, 1 passed, 10 failed (`redbase-set.log`); GREEN workspace 679 run, 679 passed, 23 skipped (`g-suite.log`); `cargo check --target x86_64-unknown-freebsd -p ms-cli` rc 0; `cargo +1.95.0 fmt --all -- --check` rc 0; `clippy --workspace --all-targets -D warnings` rc 0; `cargo test --workspace --doc` rc 0. Frequency (grade `freq.txt`): `vectors | true` panics 300/300 on the base and 0/300 fixed; `vectors --pretty | head -3` and `gui-schema | head -1` 0/300 on both.
- **Not run by the grade, so CI on the PR must show them** (`.github/workflows/`; the rows are Linux-only; `sink.rs` is std plus one `libc::fcntl` under `cfg(unix)`):
  1. `rust.yml` `test (macos-latest)`: the whole suite on macOS (the `output_errors.rs` rows compile to nothing there; `future_format_names_the_problem.rs` runs).
  2. `rust.yml` `test (ubuntu-latest)`, including the G2.x fresh-subprocess steps and `test (release, ubuntu-latest, mlock einval)`.
  3. `rust.yml` `test (ms-codec)`, `clippy (ms-codec)`, `clippy` (the shared-files check, the conformance-vector checksum and `cargo clippy --all-targets -p ms-cli`), `fmt (pinned 1.95.0)`.
  4. `rust.yml` `miri (mlock unsafe)`.
  5. `rust.yml` `musl compile/test` (x86_64 and aarch64 natives, release mode) and `freebsd compile-gate`: `sink.rs` must compile there.
  6. `rust.yml` `history purge` and `g6 invariant`: both jobs run the `ms` binary (the `history_purge` and `mlock_g6_invariant` tests); they must stay green.
  7. `release.yml` on the PR: `release test (ubuntu-latest)` (the whole suite again) and `build ms (macos-x86_64)`, `build ms (macos-aarch64)` and `build ms (windows-x86_64)`, each with its "Smoke-test the binary" step (the Windows leg is the only one where `sink.rs` and the shadowing macros compile and run without SIGPIPE; it runs because the PR is not a draft and its branch does not start `release-`). `fuzz-smoke.yml` and `vendor-freshness.yml`, whichever their path filters select. `man-release.yml` runs on an `ms-cli-v*` tag only.
- **Gate coverage:** the gate script's header lists what it does not cover.

## 10. STOP (none open)

- **S1: ruled** (section 8).
- **P-1 (ruled, accepted): the `<COMMIT>` placeholder.** The grade's FOLLOWUPS entries
  read `resolved <COMMIT>` and the grade does not say who substitutes it or
  when. Decided: the builder replaces both with the short sha of the commit
  that applies sections 3.1 to 3.8, in a following commit of the same PR (the
  block stays byte-identical to the grade's so the gate can check it). Not a
  holder-visible text.
- **P-2 (ruled, accepted): no manual text for E4.** The grade rules the manual
  section of section 8 only. The E4 stderr line is new printed text, but
  `43-ms.md` has no sentence about a reserved-tag exit 3 today (grep of toolkit
  master 47baf6f2 for `reserved`, `future format`, `exit 3`: no hit on this
  point), so nothing in it becomes false. Decided: no E4 manual text. The owner of
  exit-3 manual text is the toolkit entry `ms-manual-exit-codes`.
