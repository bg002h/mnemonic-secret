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

// ---- `ms hashlock`: the preimage ms1 is printed once (review r1, I-1) ----

/// 32 test bytes (public test material) as hex: a supplied preimage.
const HEX32: &str = "c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016";

fn hashlock(extra: &[&str]) -> std::process::Output {
    Command::cargo_bin("ms")
        .unwrap()
        .args(["hashlock", "--hex", "-"])
        .args(extra)
        .write_stdin(HEX32)
        .output()
        .unwrap()
}

/// A preimage ms1 starts `ms10h`; the card's label `preimage (ms1):` is not a
/// copy of it. "No copy" = no `ms10` and no ms1 once the spaces are removed.
fn assert_no_ms1_on(stderr: &str, ms1: &str) {
    assert!(
        !stderr.contains("ms10"),
        "a grouped ms1 is on stderr:\n{stderr}"
    );
    assert!(
        !stderr.replace(' ', "").contains(ms1),
        "the ms1 is on stderr:\n{stderr}"
    );
}

#[test]
fn hashlock_out_prints_the_preimage_once() {
    let d = tempfile::tempdir().unwrap();
    let f = d.path().join("preimage.ms1");
    let fs = f.to_str().unwrap();
    let o = hashlock(&["--out", fs]);
    assert_eq!(o.status.code(), Some(0));
    let file = std::fs::read_to_string(&f).unwrap();
    let ms1 = file.trim_end().to_string();
    assert!(ms1.starts_with("ms10h"), "F holds the preimage ms1: {file}");
    assert!(!ms1.contains(' '), "F is unbroken: {file}");
    let se = text(&o.stderr);
    assert_no_ms1_on(&se, &ms1);
    assert!(
        se.contains(&format!("preimage (ms1):  written to {fs}\n")),
        "{se}"
    );
    assert!(
        !text(&o.stdout).contains("ms10"),
        "stdout is the digest record"
    );
}

#[test]
fn hashlock_json_prints_the_preimage_once() {
    let o = hashlock(&["--json"]);
    assert_eq!(o.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    let ms1 = v["preimage_ms1"].as_str().unwrap().to_string();
    assert!(ms1.starts_with("ms10h"), "{ms1}");
    let se = text(&o.stderr);
    assert_no_ms1_on(&se, &ms1);
    assert!(
        se.contains("preimage (ms1):  in the JSON (preimage_ms1)\n"),
        "{se}"
    );
}

/// Control (passes on the base too): text mode without `--out` keeps the
/// grouped preimage on the stderr card, its only copy.
#[test]
fn hashlock_text_keeps_the_grouped_preimage() {
    let o = hashlock(&[]);
    assert_eq!(o.status.code(), Some(0));
    let se = text(&o.stderr);
    let line = se
        .lines()
        .find_map(|l| l.strip_prefix("preimage (ms1):  "))
        .expect("the card has a preimage (ms1) line");
    assert!(line.starts_with("ms10h ") && line.contains(' '), "{line}");
    assert!(
        !text(&o.stdout).contains("ms10"),
        "stdout is the digest record"
    );
}

#[test]
fn hashlock_out_and_json_print_the_preimage_once() {
    let d = tempfile::tempdir().unwrap();
    let f = d.path().join("preimage.ms1");
    let fs = f.to_str().unwrap();
    let o = hashlock(&["--out", fs, "--json"]);
    assert_eq!(o.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    let ms1 = v["preimage_ms1"].as_str().unwrap().to_string();
    assert!(ms1.starts_with("ms10h"), "{ms1}");
    assert_eq!(
        std::fs::read_to_string(&f).unwrap(),
        format!("{ms1}\n"),
        "F holds the same unbroken ms1 as the JSON"
    );
    let se = text(&o.stderr);
    assert_no_ms1_on(&se, &ms1);
    assert!(
        se.contains(&format!(
            "preimage (ms1):  written to {fs} and in the JSON (preimage_ms1)\n"
        )),
        "{se}"
    );
}

// ---- `ms hashlock`: no form of the preimage on the card when `--out` or
// `--json` carries it (architect ruling 2026-11-22,
// coordinator/agent-reports/architect-hashlock-stderr-preimage.md) ----

const LINE1_TEXT: &str =
    "THIS CARD CARRIES THE PREIMAGE -- the secret. stdout carries only the public digest.";

fn first_line(se: &str) -> &str {
    se.lines().next().unwrap_or("")
}

/// The phrase-source runs: `--hashlock-phrase-stdin --emit-record`.
fn hashlock_phrase(extra: &[&str]) -> std::process::Output {
    Command::cargo_bin("ms")
        .unwrap()
        .args(["hashlock", "--hashlock-phrase-stdin", "--emit-record"])
        .args(extra)
        .write_stdin("correct horse battery staple")
        .output()
        .unwrap()
}

#[test]
fn hashlock_out_card_carries_no_form_of_the_preimage() {
    let d = tempfile::tempdir().unwrap();
    let f = d.path().join("preimage.ms1");
    let fs = f.to_str().unwrap();
    let o = hashlock(&["--out", fs]);
    assert_eq!(o.status.code(), Some(0));
    let se = text(&o.stderr);
    assert!(!se.contains(HEX32), "the hex preimage is on stderr:\n{se}");
    assert_eq!(
        first_line(&se),
        format!(
            "THE PREIMAGE -- the secret -- is in {fs}, not on this card. \
             stdout carries only the public digest."
        )
    );
    assert!(
        se.contains("preimage (hex):  not printed; `ms decode --in` on that file prints it\n"),
        "{se}"
    );
}

#[test]
fn hashlock_json_card_carries_no_form_of_the_preimage() {
    let o = hashlock(&["--json"]);
    assert_eq!(o.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["preimage_hex"], HEX32, "the JSON keeps preimage_hex");
    let se = text(&o.stderr);
    assert!(!se.contains(HEX32), "the hex preimage is on stderr:\n{se}");
    assert_eq!(
        first_line(&se),
        "THE PREIMAGE -- the secret -- is in the JSON on stdout, not on this card."
    );
    assert!(
        se.contains("preimage (hex):  in the JSON (preimage_hex)\n"),
        "{se}"
    );
}

#[test]
fn hashlock_out_and_json_card_carries_no_form_of_the_preimage() {
    let d = tempfile::tempdir().unwrap();
    let f = d.path().join("preimage.ms1");
    let fs = f.to_str().unwrap();
    let o = hashlock(&["--out", fs, "--json"]);
    assert_eq!(o.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["preimage_hex"], HEX32, "the JSON keeps preimage_hex");
    let se = text(&o.stderr);
    assert!(!se.contains(HEX32), "the hex preimage is on stderr:\n{se}");
    assert_eq!(
        first_line(&se),
        format!(
            "THE PREIMAGE -- the secret -- is in {fs} and in the JSON on stdout, \
             not on this card."
        )
    );
    assert!(
        se.contains("preimage (hex):  in the JSON (preimage_hex)\n"),
        "{se}"
    );
}

#[test]
fn hashlock_json_moves_the_phrase_record_to_the_json() {
    let o = hashlock_phrase(&["--json"]);
    assert_eq!(o.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    let rec = v["phrase_record"].as_str().unwrap().to_string();
    assert!(rec.starts_with("phrase:"), "{rec}");
    let se = text(&o.stderr);
    assert!(!se.contains(&rec), "the phrase record is on stderr:\n{se}");
    assert!(
        se.contains("record (phrase): in the JSON (phrase_record)\n"),
        "{se}"
    );
}

/// `--out` without `--json`: F holds only the ms1, so the card keeps the
/// phrase record (its only copy) and line 1 says the card carries it.
#[test]
fn hashlock_out_keeps_the_phrase_record_and_says_so() {
    let d = tempfile::tempdir().unwrap();
    let f = d.path().join("preimage.ms1");
    let fs = f.to_str().unwrap();
    let o = hashlock_phrase(&["--out", fs]);
    assert_eq!(o.status.code(), Some(0));
    let se = text(&o.stderr);
    assert_eq!(
        first_line(&se),
        format!(
            "THE PREIMAGE -- the secret -- is in {fs}. THIS CARD CARRIES THE PHRASE RECORD \
             -- also secret. stdout carries only the public digest."
        )
    );
    assert!(se.contains("\nrecord (phrase): phrase:"), "{se}");
}

/// Control (passes on the base too): text mode without `--out` is the only
/// copy, so line 1, the hex and the phrase record stay on the card.
#[test]
fn hashlock_text_keeps_line_one_the_hex_and_the_record() {
    let o = hashlock(&[]);
    assert_eq!(o.status.code(), Some(0));
    let se = text(&o.stderr);
    assert_eq!(first_line(&se), LINE1_TEXT);
    assert!(se.contains(&format!("preimage (hex):  {HEX32}\n")), "{se}");
    let p = hashlock_phrase(&[]);
    let pe = text(&p.stderr);
    assert_eq!(first_line(&pe), LINE1_TEXT);
    assert!(pe.contains("\nrecord (phrase): phrase:"), "{pe}");
}

/// Control (passes on the base too): the remedy the `--out` card names runs,
/// and prints the hex.
#[test]
fn hashlock_out_hex_remedy_runs() {
    let d = tempfile::tempdir().unwrap();
    let f = d.path().join("preimage.ms1");
    let fs = f.to_str().unwrap();
    assert_eq!(hashlock(&["--out", fs]).status.code(), Some(0));
    let o = ms(&["decode", "--in", fs]);
    assert_eq!(o.status.code(), Some(0));
    assert!(
        text(&o.stdout).contains(&format!("preimage:  {HEX32}\n")),
        "{}",
        text(&o.stdout)
    );
}
