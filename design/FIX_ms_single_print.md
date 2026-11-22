# FIX-ms-single-print-2026-11: `ms encode` prints the ms1 once

Model: Claude Opus 5.5 (claude-opus-5-5), 2026-11-22. Author of this FIX document (not the coordinator).
Base: `origin/master` 9c0f5f3 (ms-cli 0.20.5). Branch `fix/single-print`.

## 0. Ruling and scope

Brian 2026-11-22 (typed "B"): reverse D4 of mnemonic-engrave `design/SPEC_constellation_cli_uniformity.md`. `ms`,
`md` and `mk` print their string ONCE, on stdout; the grouped stderr copy is dropped everywhere. `mt encode` and
`mnemonic bundle` are the model. The uniformity-spec delta is on mnemonic-engrave branch `fix/single-print`
(commit 4f35eb9f).

In scope: `ms encode` (its stderr `engraving card: <grouped ms1>` line is the second copy). Out of scope, unchanged:

- `ms split`: already prints each share once (stdout, grouped by default, `share i of n:` labels on stderr). Its
  grouped default is a recorded ruling (F-284; `crates/ms-cli/tests/the_decline.rs:140`, "§6a binds the stdout rule
  to `encode` alone"), and flipping it is not part of this ruling.
- `ms hashlock`: stdout carries the public digest record; the stderr card carries the preimage ms1, which stdout
  never carries by design. That is the only copy, not a second one; its `--no-engraving-card`, `--group-size`
  (default 5) and `--separator` stay as they are.
- `ms combine`, `ms repair`, `ms derive`, `ms decode`: print no copy on stderr.

### PROPOSED decisions (architect confirms)

- P1. `--group-size` and `--separator` on `ms encode` are KEPT and group STDOUT on request, as `mt encode` and
  `mnemonic bundle` do. `--group-size` default changes 5 -> 0 (unbroken), so a plain `ms encode` prints on stdout
  exactly what it printed before. `--separator` keeps its parser and default `space`.
- P2. `--out FILE` receives exactly the bytes stdout would have carried (grouped under `--group-size`), in text mode
  and under `--json` (the file is the artifact; stdout carries the JSON report). Before: always the unbroken ms1.
  This makes `ms encode --out` differ from `ms split --out`, which keeps writing unbroken shares (out of scope above).
- P3. `--json` ignores the grouping flags (unchanged).
- P4. `--no-engraving-card` is KEPT (the brief's draft said "removed"; this document proposes otherwise). After the
  change it still does something: it suppresses the `word count:`, `language:` and `passphrase:` lines, exactly as
  `mnemonic bundle --no-engraving-card` suppresses a panel that carries no string. Removing it would break, for no
  gain, every consumer that passes it: mnemonic-gui `tests/f679_argv_secret_admission.rs` (2 runs against the
  pinned `ms`), mnemonic-engrave `design/journeys/` (5 sites: `derive-hashvault-keys.sh:40`,
  `derive-rcw-keys.sh:71`, `transcript_hashvault.sh:214`, `transcript_pathological.sh:370`,
  `transcript_tr_pathological.sh:283`), seedhammer `oracle/expect.go:607,1165` and the toolkit's technical-manual
  transcripts `ms1-encode-12word-abandon{,-unbroken}.cmd`. If the architect rules removal instead, each of those
  is a lockstep edit (drop the flag); none parses the card. Its help text changes (section 1).
- P5. The `encode` help EXAMPLES gain one grouped example and the first example's comment changes (section 1).

## 1. Spec delta (ms-local texts)

`ms encode --help`, measured on the GREEN build (long help; the short help shows the first sentence):

    --no-engraving-card
        Suppress the stderr engraving card: the word count, language and passphrase lines (for tooling). The ms1 itself is on stdout either way

    --group-size <GROUP_SIZE>
        Group the ms1 on stdout every N characters, for hand engraving (0 = unbroken, the default). `--out` receives the same bytes; `--json` stays unbroken

        [default: 0]

EXAMPLES block, first two lines (the second is new; the rest is unchanged):

    EXAMPLES:
      ms encode --in phrase.txt                     # ms1 on stdout; word count etc. on stderr
      ms encode --in phrase.txt --group-size 5      # the ms1 in groups of 5, for hand engraving

`--separator`'s help is unchanged.

`CHANGELOG.md`, under `## ms-cli [Unreleased]`:

    ### Changed
    - **`ms encode` prints the ms1 once.** The stderr `engraving card: <grouped ms1>` line is gone (Brian
      2026-11-22, reversing D4 of the constellation uniformity spec). `--group-size N` now groups stdout, on
      request; its default is `0`, so a plain `ms encode` prints on stdout exactly what it printed before.
      `--out FILE` receives the same bytes stdout would have carried. `--no-engraving-card` still suppresses the
      word count, language and passphrase lines. `--json`, `ms split` and `ms hashlock` are unchanged.

## 2. Behaviour: exact bytes per case

`P` = a file holding the 12-word all-zero-entropy test phrase (`abandon` x11 `about`). `C` =
`ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f`, `G` = `ms10e ntrsq qqqqq qqqqq qqqqq qqqqq qqqqq qqcj9 sxraq
34v7f`. `L` = the three lines `word count: 12` / `language: english (BIP-39 checksum valid)` / `passphrase: not
stored in ms1 (record separately if used)`. `W` = `warning: stdout carries private key material (can spend) —
redirect or encrypt (e.g. '> file.txt' or '| age -e ...')` (U+2014). Measured on the GREEN build; every exit is 0.

| case | stdout | stderr | before (9c0f5f3) |
|---|---|---|---|
| `ms encode --in P` | `C\n` | `L\nW\n` | stderr `engraving card: G\nL\nW\n` |
| `ms encode --in P --group-size 5` | `G\n` | `L\nW\n` | stdout `C\n`, stderr `engraving card: G\n...` |
| `ms encode --in P --group-size 0` | `C\n` | `L\nW\n` | stderr `engraving card: C\nL\nW\n` |
| `ms encode --in P --group-size 5 --out F` | empty; F (0600) = `G\n` | `L\n` (no `W`: stdout is empty, F-589, unchanged) | F = `C\n`; stderr `engraving card: G\nL\n` |
| `ms encode --in P --group-size 5 --json` | the unchanged JSON line (`"ms1":"C"`) | `W\n` | identical (unchanged) |
| `ms encode --in P --group-size 5 --json --out F` | the unchanged JSON line | `W\n` | F = `C\n` (now `G\n`) |
| `ms encode --in P --no-engraving-card --group-size 5` | `G\n` | `W\n` | stdout `C\n`, stderr `W\n` |
| `ms encode --hex - --group-size 5` (16 zero bytes on stdin) | `G\n` | `word count: 12\npassphrase: not stored in ms1 (record separately if used)\nW\n` | stdout `C\n`, stderr `engraving card: G\n...` |
| refusals, exit codes | unchanged | unchanged | — |

ms1 is a single string, so there is no multi-chunk case.

## 3. Production changes (exact; the GREEN build measured in section 6)

`cmd/encode.rs` (flag docs and default, the grouped artifact used for both stdout and `--out`, the `engraving card:`
line removed, `emit_text` loses two parameters) and `main.rs` (the EXAMPLES block).

```diff
diff --git i/crates/ms-cli/src/cmd/encode.rs w/crates/ms-cli/src/cmd/encode.rs
index 8759a5b..d6676dd 100644
--- i/crates/ms-cli/src/cmd/encode.rs
+++ w/crates/ms-cli/src/cmd/encode.rs
@@ -76,16 +76,15 @@ pub struct EncodeArgs {
     #[arg(long, default_value = "english")]
     pub language: CliLanguage,
 
-    /// Suppress the stderr engraving card (for tooling).
+    /// Suppress the stderr engraving card: the word count, language and
+    /// passphrase lines (for tooling). The ms1 itself is on stdout either way.
     #[arg(long)]
     pub no_engraving_card: bool,
 
-    /// Group the engraving card's ms1 every N characters (0 = unbroken).
-    ///
-    /// Shapes the stderr `engraving card:` line and nothing else: stdout,
-    /// `--out` and `--json` always carry the canonical, unbroken ms1, and under
-    /// `--no-engraving-card` this flag has no effect. SPEC §3.
-    #[arg(long, default_value_t = 5)]
+    /// Group the ms1 on stdout every N characters, for hand engraving
+    /// (0 = unbroken, the default). `--out` receives the same bytes; `--json`
+    /// stays unbroken.
+    #[arg(long, default_value_t = 0)]
     pub group_size: u16,
 
     /// Separator between groups: `space` (or the literal " "). SPEC §5.
@@ -266,20 +265,22 @@ pub fn run(mut args: EncodeArgs) -> Result<u8> {
     // a stream that is usually redirected into a 0644 one. The stderr engraving
     // card is unaffected, and `--json` (a REPORT, not the artifact) still goes to
     // stdout.
+    let artifact = format!(
+        "{}\n",
+        render_grouped(&ms1, args.group_size as usize, args.separator)
+    );
     if let Some(path) = args.out.as_deref() {
-        crate::out::write_artifact(path, &format!("{ms1}\n"))?;
+        crate::out::write_artifact(path, &artifact)?;
     }
 
     if args.json {
         emit_json(&ms1, language_for_card, word_count, &entropy[..])?;
     } else {
         emit_text(
-            &ms1,
+            &artifact,
             language_for_card,
             word_count,
             args.no_engraving_card,
-            args.group_size as usize,
-            args.separator,
             args.out.is_some(),
         )?;
     }
@@ -385,39 +386,22 @@ fn emit_json(ms1: &str, language: Option<&str>, word_count: usize, entropy: &[u8
     Ok(())
 }
 
-#[allow(clippy::too_many_arguments)]
 fn emit_text(
-    ms1: &str,
+    artifact: &str,
     language: Option<&str>,
     word_count: usize,
     no_engraving_card: bool,
-    group_size: usize,
-    separator: char,
     artifact_went_to_a_file: bool,
 ) -> Result<()> {
-    // **stdout carries the CANONICAL ms1 and nothing else, always ungrouped**
-    // (§6a, §6b). Grouping is a display property of a card a human reads off
-    // metal; it is not a property of the artifact, and putting it on stdout made
-    // `ms encode | me sysw pack` unclassifiable. Measured before this change:
-    // the grouped default exits 4 -- "record 0 ... is not a form this container
-    // can place" -- and writes no payload, while the same pipeline with
-    // `--group-size 0` exits 0 and writes 102 bytes at 0600.
+    // The ms1 is printed ONCE, on stdout (Brian 2026-11-22, reversing D4):
+    // grouped only when `--group-size` asks, so the default is the canonical
+    // artifact `ms encode | me sysw pack` reads. There is no stderr copy.
     if !artifact_went_to_a_file {
-        println!("{ms1}");
+        print!("{artifact}");
     }
 
     if !no_engraving_card {
         let mut stderr = std::io::stderr().lock();
-        // `--group-size` and `--separator` now bind HERE, and nowhere else. The
-        // card gains the grouped string, which after this is the only place it
-        // exists -- so `--no-engraving-card`, and any `2>/dev/null`, now throws
-        // away the form an engraver actually reads (§6c).
-        writeln!(
-            stderr,
-            "engraving card: {}",
-            render_grouped(ms1, group_size, separator)
-        )
-        .ok();
         writeln!(stderr, "word count: {}", word_count).ok();
         if let Some(lang) = language {
             writeln!(stderr, "language: {} (BIP-39 checksum valid)", lang).ok();
diff --git i/crates/ms-cli/src/main.rs w/crates/ms-cli/src/main.rs
index 8fc66f0..24803d2 100644
--- i/crates/ms-cli/src/main.rs
+++ w/crates/ms-cli/src/main.rs
@@ -80,7 +80,7 @@ enum Command {
 
     /// Encode a BIP-39 mnemonic (or hex entropy) as an ms1 string for engraving.
     #[command(
-        after_long_help = "EXAMPLES:\n  ms encode --in phrase.txt                     # ms1 on stdout, engraving card on stderr\n  ms encode --phrase - < phrase.txt             # the phrase on stdin\n  ms encode --hex - --no-engraving-card < entropy.hex\n  ms encode --in phrase.txt --out card.ms1      # the ms1 to an owner-only file\n  ms encode --in phrase.txt --json | jq .ms1\n\nThe secret never goes on the command line: ms refuses it there unless\n--allow-argv-secret is given (argv is visible to `ps` and lands in shell\nhistory). Use --in FILE, or `-` for stdin."
+        after_long_help = "EXAMPLES:\n  ms encode --in phrase.txt                     # ms1 on stdout; word count etc. on stderr\n  ms encode --in phrase.txt --group-size 5      # the ms1 in groups of 5, for hand engraving\n  ms encode --phrase - < phrase.txt             # the phrase on stdin\n  ms encode --hex - --no-engraving-card < entropy.hex\n  ms encode --in phrase.txt --out card.ms1      # the ms1 to an owner-only file\n  ms encode --in phrase.txt --json | jq .ms1\n\nThe secret never goes on the command line: ms refuses it there unless\n--allow-argv-secret is given (argv is visible to `ps` and lands in shell\nhistory). Use --in FILE, or `-` for stdin."
     )]
     Encode(cmd::encode::EncodeArgs),
 
```

## 4. RED rows

New file `crates/ms-cli/tests/single_print.rs` (verbatim below; rustfmt-clean). Row = test function name.

| row | expected after GREEN | measured on base 9c0f5f3 |
|---|---|---|
| `encode_prints_the_ms1_once` | stdout `C\n`; stderr exactly `L\nW\n` | FAIL: stderr starts `engraving card: G` (the second copy) |
| `encode_group_size_groups_stdout` | `--group-size 5`: stdout `G\n`; stderr `L\nW\n` | FAIL: stdout `C\n` |
| `encode_out_writes_the_stdout_bytes` | `--group-size 5 --out F`: stdout empty, F = `G\n`, stderr `L\n` | FAIL: F = `C\n` |
| `encode_no_engraving_card_leaves_only_the_warning` | control: stdout `C\n`, stderr `W\n` | PASS (a guard on P4, not a RED row) |

Command (run, not pinned): `CARGO_TARGET_DIR=<own> cargo nextest run --locked --build-jobs 6 -j 6 -p ms-cli --test
single_print`. Base result: `4 tests run: 1 passed, 3 failed`. Assertion excerpts from that run:

```text
    thread 'encode_prints_the_ms1_once' panicked at crates/ms-cli/tests/single_print.rs:47:5:
      left: "engraving card: ms10e ntrsq qqqqq qqqqq qqqqq qqqqq qqqqq qqcj9 sxraq 34v7f\nword count: 12\nlanguage: english (BIP-39 checksum valid)\npassphrase: not stored in ms1 (record separately if used)\nwarning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')\n"
     right: "word count: 12\nlanguage: english (BIP-39 checksum valid)\npassphrase: not stored in ms1 (record separately if used)\nwarning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')\n"
    thread 'encode_group_size_groups_stdout' panicked at crates/ms-cli/tests/single_print.rs:55:5:
      left: "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f\n"
     right: "ms10e ntrsq qqqqq qqqqq qqqqq qqqqq qqqqq qqcj9 sxraq 34v7f\n"
    thread 'encode_out_writes_the_stdout_bytes' panicked at crates/ms-cli/tests/single_print.rs:74:5:
      left: "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f\n"
     right: "ms10e ntrsq qqqqq qqqqq qqqqq qqqqq qqqqq qqcj9 sxraq 34v7f\n"
```

The RED file, verbatim:

```rust
//! `ms encode` prints the ms1 ONCE, on stdout (Brian 2026-11-22, reversing D4
//! of mnemonic-engrave `design/SPEC_constellation_cli_uniformity.md`;
//! `design/FIX_ms_single_print.md`).
//!
//! The stderr `engraving card: <grouped ms1>` line is gone. `--group-size N`
//! groups STDOUT on request (default 0, ungrouped), as `mt encode` and
//! `mnemonic bundle` do, and `--out FILE` receives exactly the bytes stdout
//! would have carried. `ms split` is unchanged: it already prints each share
//! once, grouped on stdout (F-284).

use assert_cmd::Command;

/// The 12-word all-zero-entropy test phrase (public test vector).
const Z12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about\n";
const CANON: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
const CANON5: &str = "ms10e ntrsq qqqqq qqqqq qqqqq qqqqq qqqqq qqcj9 sxraq 34v7f";
const PANEL: &str = "word count: 12\n\
language: english (BIP-39 checksum valid)\n\
passphrase: not stored in ms1 (record separately if used)\n";
const WARNING: &str = "warning: stdout carries private key material (can spend) \u{2014} redirect or encrypt (e.g. '> file.txt' or '| age -e ...')\n";

fn phrase_file(dir: &tempfile::TempDir) -> String {
    let p = dir.path().join("phrase.txt");
    std::fs::write(&p, Z12).unwrap();
    p.to_str().unwrap().to_string()
}

fn ms(args: &[&str]) -> std::process::Output {
    Command::cargo_bin("ms")
        .unwrap()
        .args(args)
        .output()
        .unwrap()
}

fn text(b: &[u8]) -> String {
    String::from_utf8(b.to_vec()).unwrap()
}

#[test]
fn encode_prints_the_ms1_once() {
    let d = tempfile::tempdir().unwrap();
    let o = ms(&["encode", "--in", &phrase_file(&d)]);
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(text(&o.stdout), format!("{CANON}\n"));
    assert_eq!(text(&o.stderr), format!("{PANEL}{WARNING}"));
}

#[test]
fn encode_group_size_groups_stdout() {
    let d = tempfile::tempdir().unwrap();
    let o = ms(&["encode", "--in", &phrase_file(&d), "--group-size", "5"]);
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(text(&o.stdout), format!("{CANON5}\n"));
    assert_eq!(text(&o.stderr), format!("{PANEL}{WARNING}"));
}

#[test]
fn encode_out_writes_the_stdout_bytes() {
    let d = tempfile::tempdir().unwrap();
    let f = d.path().join("seed.ms1");
    let o = ms(&[
        "encode",
        "--in",
        &phrase_file(&d),
        "--group-size",
        "5",
        "--out",
        f.to_str().unwrap(),
    ]);
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(text(&o.stdout), "");
    assert_eq!(std::fs::read_to_string(&f).unwrap(), format!("{CANON5}\n"));
    assert_eq!(
        text(&o.stderr),
        PANEL,
        "no warning: stdout is empty (F-589)"
    );
}

/// Control (passes on the base too): `--no-engraving-card` still cuts stderr
/// to the warning alone.
#[test]
fn encode_no_engraving_card_leaves_only_the_warning() {
    let d = tempfile::tempdir().unwrap();
    let o = ms(&["encode", "--in", &phrase_file(&d), "--no-engraving-card"]);
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(text(&o.stdout), format!("{CANON}\n"));
    assert_eq!(text(&o.stderr), WARNING);
}
```

## 5. Tests that change

Measured: the full suite with the section-3 production change and the base tests failed exactly 5 rows
(`/scratch/cache/sp-ms-basetests.log`, `725 tests run: 720 passed, 5 failed`). Their change, plus one deletion:

| test | change |
|---|---|
| `encode_grouping_flags::encode_stdout_is_the_canonical_ms1_and_the_card_carries_the_grouping` | replaced by `encode_stdout_is_the_canonical_ms1_and_stderr_has_no_copy` (stdout `C\n`; stderr contains neither `C`, `ms10e` nor `engraving card:`) |
| `encode_grouping_flags::group_size_zero_collapses_the_card_and_leaves_stdout_alone` | deleted; covered by `group_size_groups_encode_stdout` |
| `encode_grouping_flags::no_display_flag_can_change_encode_stdout` | replaced by `group_size_groups_encode_stdout` (no flag, `--group-size 0` -> `C`; `--group-size 5`, with and without `--separator space` -> `G`) |
| `encode_grouping_flags::no_engraving_card_removes_the_only_grouped_form_there_is` | passes on the change but its doc ("the only grouped form there is") becomes false; deleted, covered by `single_print::encode_no_engraving_card_leaves_only_the_warning` |
| `encode_canonical_12_word::encode_12_word_abandon_about` | the stderr assertion flips: no `ms10e`, no `engraving card:` |
| `f677_help_matches_behaviour::encode_group_size_help_describes_the_card_not_stdout` | renamed `encode_group_size_help_describes_stdout`; asserts the new help wording and that `--group-size 3` groups stdout in threes |
| `encode_grouping_flags` module doc | rewritten (no behaviour) |

Every other `--group-size` / `--no-engraving-card` use in the suite (`argv_*`, `hashlock_*`, `cli_split`,
`decode_grouped`, `encode_no_engraving_card`, ...) passes unchanged.

Exact diff (`crates/ms-cli/tests/`, excluding the new `single_print.rs`):

```diff
diff --git i/crates/ms-cli/tests/encode_canonical_12_word.rs w/crates/ms-cli/tests/encode_canonical_12_word.rs
index 4a6b75d..7f4ff0e 100644
--- i/crates/ms-cli/tests/encode_canonical_12_word.rs
+++ w/crates/ms-cli/tests/encode_canonical_12_word.rs
@@ -4,8 +4,7 @@ use assert_cmd::Command;
 
 #[test]
 fn encode_12_word_abandon_about() {
-    // mstring-grouping P2: encode text is now print-once, default space/5
-    // (was `<ms1>\n\n<chunked>` print-twice). stderr engraving card unchanged.
+    // Print-once: the ms1 appears once, on stdout.
     let out = Command::cargo_bin("ms")
         .unwrap()
         .args([
@@ -20,8 +19,8 @@ fn encode_12_word_abandon_about() {
         !stdout.contains("\n\n"),
         "print-once: no blank line; got {stdout:?}"
     );
-    // §6a/§6b: stdout is the CANONICAL ms1, always ungrouped. The default
-    // space/5 grouping moved to the stderr engraving card.
+    // Single print (2026-11-22): stdout is the CANONICAL ms1, ungrouped by
+    // default, and stderr carries no copy of it.
     let line = stdout.lines().next().unwrap();
     assert_eq!(
         line, "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f",
@@ -29,8 +28,8 @@ fn encode_12_word_abandon_about() {
     );
     let stderr = String::from_utf8(out.stderr).unwrap();
     assert!(
-        stderr.contains("engraving card: ms10e ntrsq"),
-        "the default space/5 grouping is on the card now; got {stderr:?}"
+        !stderr.contains("ms10e") && !stderr.contains("engraving card:"),
+        "single print: no copy of the ms1 on stderr; got {stderr:?}"
     );
     assert!(stderr.contains("language: english"));
     assert!(stderr.contains("word count: 12"));
diff --git i/crates/ms-cli/tests/encode_grouping_flags.rs w/crates/ms-cli/tests/encode_grouping_flags.rs
index 50320c1..8f231b7 100644
--- i/crates/ms-cli/tests/encode_grouping_flags.rs
+++ w/crates/ms-cli/tests/encode_grouping_flags.rs
@@ -1,5 +1,6 @@
-//! `ms encode` / `ms split` mstring display-grouping flags (P2).
-//! Default = space/5 print-once; `--group-size 0` unbroken; `--separator`.
+//! `ms encode` / `ms split` mstring display-grouping flags.
+//! `ms encode`: default unbroken, `--group-size N` groups stdout (single print,
+//! 2026-11-22). `ms split`: default space/5 on stdout (unchanged).
 
 use assert_cmd::Command;
 
@@ -20,90 +21,41 @@ fn stdout_of(args: &[&str]) -> String {
     String::from_utf8(out.stdout).unwrap()
 }
 
-/// **MOVED, not deleted: the grouping is now on the CARD.** §6a/§6b make
-/// `ms encode`'s stdout the canonical ms1 and nothing else, always ungrouped,
-/// and `--group-size` / `--separator` bind to the stderr engraving card alone.
-/// The reason is cross-tool rather than cosmetic and was measured: piped into
-/// `me sysw pack`, the grouped default exits **4** -- "record 0 ... is not a
-/// form this container can place" -- and writes no payload, while the ungrouped
-/// form exits **0** and writes a 102-byte payload at 0600.
+/// Default: stdout is the canonical ms1 and stderr carries no copy of it.
+/// Measured, the reason for the unbroken default: piped into `me sysw pack`, a
+/// grouped ms1 exits **4** -- "record 0 ... is not a form this container can
+/// place" -- while the unbroken form exits **0**.
 #[test]
-fn encode_stdout_is_the_canonical_ms1_and_the_card_carries_the_grouping() {
+fn encode_stdout_is_the_canonical_ms1_and_stderr_has_no_copy() {
     let out = support::run(&["encode", "--phrase", Z12]);
     assert!(out.status.success());
     let s = String::from_utf8(out.stdout).unwrap();
-    assert_eq!(
-        s,
-        format!("{CANON}\n"),
-        "stdout is the canonical artifact, ungrouped, and nothing else"
-    );
-
+    assert_eq!(s, format!("{CANON}\n"), "stdout is the canonical artifact");
     let e = String::from_utf8(out.stderr).unwrap();
-    let card = e
-        .lines()
-        .find(|l| l.starts_with("engraving card: "))
-        .unwrap_or_else(|| panic!("no engraving-card line in stderr:\n{e}"));
-    let grouped = card.trim_start_matches("engraving card: ");
-    assert_eq!(
-        grouped.chars().nth(5),
-        Some(' '),
-        "the card keeps the default space/5 grouping; got {grouped:?}"
-    );
-    assert_eq!(
-        grouped.chars().filter(|c| *c != ' ').collect::<String>(),
-        CANON,
-        "and the grouped form is the same artifact"
+    assert!(
+        !e.contains(CANON) && !e.contains("ms10e") && !e.contains("engraving card:"),
+        "no copy of the ms1 on stderr:\n{e}"
     );
 }
 
-/// The flags still WORK; they just work on the card.
+/// `--group-size` and `--separator` shape STDOUT, on request.
 #[test]
-fn group_size_zero_collapses_the_card_and_leaves_stdout_alone() {
-    let out = support::run(&["encode", "--phrase", Z12, "--group-size", "0"]);
-    let s = String::from_utf8(out.stdout).unwrap();
-    let e = String::from_utf8(out.stderr).unwrap();
-    assert_eq!(s, format!("{CANON}\n"));
-    assert!(
-        e.contains(&format!("engraving card: {CANON}")),
-        "at group-size 0 the card is the unbroken string too:\n{e}"
-    );
-}
-
-/// **`--no-engraving-card` now throws away the form an engraver reads**, and so
-/// does any `2>/dev/null`. §6c names this as a real change rather than a
-/// cosmetic one, and this pins it so nobody re-discovers it on a plate.
-#[test]
-fn no_engraving_card_removes_the_only_grouped_form_there_is() {
-    let out = support::run(&["encode", "--phrase", Z12, "--no-engraving-card"]);
-    let s = String::from_utf8(out.stdout).unwrap();
-    let e = String::from_utf8(out.stderr).unwrap();
-    assert_eq!(s, format!("{CANON}\n"));
-    assert!(
-        !e.contains("engraving card:"),
-        "the card is suppressed:\n{e}"
-    );
-    assert!(
-        !e.contains("ms10e ntrsq"),
-        "and with it the grouped form, which exists nowhere else now:\n{e}"
-    );
-}
-
-/// `ms encode`'s stdout does not move, whatever the display flags say.
-#[test]
-fn no_display_flag_can_change_encode_stdout() {
-    for extra in [
-        vec![],
-        vec!["--group-size", "0"],
-        vec!["--group-size", "7"],
-        vec!["--separator", "space"],
+fn group_size_groups_encode_stdout() {
+    for (extra, want) in [
+        (vec![], CANON.to_string()),
+        (vec!["--group-size", "0"], CANON.to_string()),
+        (
+            vec!["--group-size", "5"],
+            "ms10e ntrsq qqqqq qqqqq qqqqq qqqqq qqqqq qqcj9 sxraq 34v7f".to_string(),
+        ),
+        (
+            vec!["--group-size", "5", "--separator", "space"],
+            "ms10e ntrsq qqqqq qqqqq qqqqq qqqqq qqqqq qqcj9 sxraq 34v7f".to_string(),
+        ),
     ] {
         let mut argv = vec!["encode", "--phrase", Z12];
         argv.extend(extra.iter().copied());
-        assert_eq!(
-            stdout_of(&argv),
-            format!("{CANON}\n"),
-            "{argv:?} changed the ARTIFACT, which is not a display property"
-        );
+        assert_eq!(stdout_of(&argv), format!("{want}\n"), "{argv:?}");
     }
 }
 
diff --git i/crates/ms-cli/tests/f677_help_matches_behaviour.rs w/crates/ms-cli/tests/f677_help_matches_behaviour.rs
index c0eb9fe..cd5f0c4 100644
--- i/crates/ms-cli/tests/f677_help_matches_behaviour.rs
+++ w/crates/ms-cli/tests/f677_help_matches_behaviour.rs
@@ -35,11 +35,11 @@ fn long_help(verb: &str) -> String {
 // encode --group-size
 // ---------------------------------------------------------------------------
 
-/// The help names what the flag shapes (the card) and what it does not
-/// (stdout, --out, --json). The behaviour half is pinned by
-/// `encode_grouping_flags.rs`; this pins the words to it.
+/// The help names what the flag shapes (stdout, on request) and what it does
+/// not (`--json`). The behaviour half is pinned by `encode_grouping_flags.rs`
+/// and `single_print.rs`; this pins the words to it.
 #[test]
-fn encode_group_size_help_describes_the_card_not_stdout() {
+fn encode_group_size_help_describes_stdout() {
     let h = long_help("encode");
     let para: String = h
         .split("--group-size")
@@ -49,19 +49,17 @@ fn encode_group_size_help_describes_the_card_not_stdout() {
         .split_whitespace()
         .collect::<Vec<_>>()
         .join(" ");
-    assert!(
-        !para.contains("in the emitted ms1 string"),
-        "the old wording is back: {para}"
-    );
+    for gone in ["engraving card", "in the emitted ms1 string"] {
+        assert!(!para.contains(gone), "old wording {gone:?} is back: {para}");
+    }
     for want in [
-        "engraving card",
-        "stdout, `--out` and `--json` always carry the canonical, unbroken ms1",
-        "`--no-engraving-card` this flag has no effect",
+        "Group the ms1 on stdout every N characters",
+        "`--out` receives the same bytes; `--json` stays unbroken",
     ] {
         assert!(para.contains(want), "missing {want:?} in: {para}");
     }
 
-    // And the claim is true: a non-default group size leaves stdout unbroken.
+    // And the claim is true: a non-default group size groups stdout.
     let dir = tempfile::tempdir().unwrap();
     std::fs::write(dir.path().join("p.txt"), Z12).unwrap();
     let out = ms()
@@ -70,7 +68,14 @@ fn encode_group_size_help_describes_the_card_not_stdout() {
         .output()
         .unwrap();
     assert!(out.status.success());
-    assert_eq!(String::from_utf8_lossy(&out.stdout), format!("{CANON}\n"));
+    let want: String = CANON
+        .chars()
+        .collect::<Vec<_>>()
+        .chunks(3)
+        .map(|c| c.iter().collect::<String>())
+        .collect::<Vec<_>>()
+        .join(" ");
+    assert_eq!(String::from_utf8_lossy(&out.stdout), format!("{want}\n"));
 }
 
 // ---------------------------------------------------------------------------
```

## 6. Gate (ms CI jobs, run locally on the GREEN build: section 3 + section 5 + `single_print.rs`)

| CI step | command | result |
|---|---|---|
| test | `cargo nextest run --locked --workspace --no-fail-fast` (`--build-jobs 6 -j 6`, own target dir) | 727 passed, 23 skipped as on the base (`/scratch/cache/sp-ms-full4.log`) |
| ignored decline/purge rows | `cargo nextest run -p ms-cli --test the_decline --test history_purge --run-ignored all` | 13 passed |
| doc tests | `cargo test --locked --workspace --doc` | ok |
| clippy | `cargo clippy --locked --workspace --all-targets -- -D warnings` | clean |
| fmt | `cargo fmt --all --check` (CI pins `+1.95.0`; run with the repo toolchain 1.85.0) | clean |
| docs-only self-test | `bash ci/docs-only.test.sh` | exit 0 |
| display-grouping vectors | `cd design && sha256sum -c display-grouping-vectors.tsv.sha256` | OK (file untouched) |
| miri mlock, mlock_g6_invariant, musl `--release`, freebsd, fuzz, vendor-freshness | not run | no `mlock`, dependency or codec change |

The build re-runs the whole gate on its own commits; this table is the author's pre-check.

## 7. Build order

1. RED commit: add `crates/ms-cli/tests/single_print.rs`; run the row command; see 3 FAIL + 1 PASS (the control).
2. GREEN commit: section 3, section 5, the CHANGELOG entry (section 1); full gate.
3. Release (PROPOSED next: ms-cli 0.21.0, a CLI-output change).

## 8. Lockstep

### 8a. Toolkit manual (`mnemonic-toolkit` `docs/manual/src/40-cli-reference/43-ms.md`), lands with the toolkit's ms pin bump

Line 85 (`--out` row) becomes:

    | `--out <FILE>` | write the ms1 string to FILE, owner-only (`0600`); the file holds exactly what stdout would (grouped under `--group-size`) |

Line 88 (`--no-engraving-card` row) becomes:

    | `--no-engraving-card` | suppress the stderr engraving-card lines (word count, language, passphrase); the private-material warning stays |

Line 89 (`--group-size` row) becomes:

    | `--group-size <N>` | group the ms1 on stdout every N characters, for hand engraving (default `0` = unbroken, so `ms encode \| me sysw pack` can classify it). `--out` receives the same bytes; `--json` stays unbroken. `ms split` groups each printed share by default; `ms combine` has no grouping flags and prints unbroken. Separator-stripping on intake means grouped and unbroken cards both re-ingest |

Line 90 (`--separator` row): replace "the engraving card's separator for `--group-size`" with "the separator for
`--group-size`"; the rest of the row is unchanged.

Line 248 (`ms split --group-size` row) becomes:

    | `--group-size <N>` | display grouping for each printed share (default 5; `0` = unbroken). Unlike `ms encode`, whose default is `0`, `ms split` prints the grouped form on stdout by default; `--out` writes unbroken shares |

Worked example (after line 118, inside the same `sh` block), append:

    # Grouped in fives, for hand engraving:
    ms encode --in phrase.txt --group-size 5
    # ms10e ntrsq qqqqq qqqqq qqqqq qqqqq qqqqq qqcj9 sxraq 34v7f

### 8b. Toolkit GUI manual (`docs/manual-gui/`), lands with the toolkit's gui-manual pin bump

- `docs/manual-gui/transcripts/63-ms-encode-phrase.err` is regenerated by `docs/manual-gui/tests/verify-examples-gui.sh`;
  its first line `engraving card: ms10e ntrsq ...` disappears (new content measured: `L\nW\n`).
- `docs/manual-gui/src/60-ms/63-encode.md:236-243`: the sentence "The stderr engraving card adds the same string in
  its default 5-character display grouping (the form you engrave; `--group-size` and `--separator` shape this line),
  then the phrase metadata:" becomes "stderr carries the phrase metadata (`--group-size 5` prints the ms1 in
  groups of 5 on stdout instead):", and the include becomes `lines="1-3"` with the three `L` lines.
- Lines 5-9 of the same page still describe the pre-0.17 "single string + blank line + 5-char chunked form" stdout;
  stale before this change (proposed line in section 9), not part of this lockstep.

### 8c. mnemonic-gui (`src/schema/ms.rs`, `ENCODE_FLAGS`), lands with the gui's ms pin bump

- `--group-size`: `default_value: Some("0")`; `help: "Group the ms1 on stdout every N characters, for hand
  engraving (default 0 = unbroken). Cosmetic — intake strips separators, so any grouping re-ingests."`
- `--no-engraving-card`: `help: "Suppress the word count, language and passphrase lines on stderr."`
- `--separator`, the split and hashlock flag tables: unchanged.
- `pinned-upstream.toml` `[ms] tag` moves from `ms-cli-v0.20.1` to the release carrying this change; the gui builder
  runs `schema-mirror` (its defaults-drift gate catches the 5 -> 0 default) and folds any other drift.
- The gui tests that pass `--group-size 0` and `--no-engraving-card` to the real `ms encode`
  (`tests/f679_argv_secret_admission.rs`) keep passing: both flags stay.

## 9. Proposed FOLLOWUPS lines (direct-pushed after merge, not in the PR)

For mnemonic-toolkit `design/FOLLOWUPS.md`:

    - [tk-manual-gui-ms-encode-stdout-stale] status=open grade=M kind=docs owner=none filed=2026-11-22 src=coordinator/agent-reports/fix-doc-single-print.md :: docs/manual-gui/src/60-ms/63-encode.md:5-9 and the --json section still describe ms encode stdout as single string + blank line + chunked form (pre ms-cli 0.17.0).

For mnemonic-secret `design/FOLLOWUPS.md`:

    - [ms-split-out-unbroken-vs-encode-out-grouped] status=open grade=N kind=ux owner=none filed=2026-11-22 src=coordinator/agent-reports/fix-doc-single-print.md :: After the single print, ms encode --out writes stdout's bytes (grouped under --group-size) while ms split --out always writes unbroken shares; split was left alone (F-284).

## 10. Not covered, STOP

- Not covered: miri, musl, FreeBSD, fuzz, release jobs (not run); reviewer judgement of P1-P5.
- STOP: none. No consumer parses the `engraving card:` line; every consumer of `--no-engraving-card` keeps working
  under P4.
