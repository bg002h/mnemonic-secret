//! bh-ms-I4 (FIX #47, architect T26): `ms hashlock --emit-record
//! --no-engraving-card` without `--json` is refused (exit 64) before anything
//! is read or written, because the record has no stream to go to.

use assert_cmd::Command;

const REC: &str = "phrase:68617264656e65642c747261696c";
const HASH_LINE: &str = "hash:8d429098f143fb65ac17e10380ad2e53ccb1677bc514cd10335ba38cc1eea23c\n";
const MSG: &str = "--emit-record has nowhere to print the `phrase:` record under --no-engraving-card: in text mode the record goes on the card, and stdout carries only the public digest. Add --json to get it in the JSON object (phrase_record), or drop --no-engraving-card to get it on the card.";

fn ms(dir: &std::path::Path) -> Command {
    let mut c = Command::cargo_bin("ms").unwrap();
    c.current_dir(dir);
    c
}

fn refused(args: &[&str], stdin: &str, dir: &std::path::Path) {
    let r = ms(dir).args(args).write_stdin(stdin).assert().code(64);
    let o = r.get_output();
    assert!(o.stdout.is_empty(), "stdout must be empty");
    assert_eq!(
        String::from_utf8_lossy(&o.stderr),
        format!("error: {MSG}\n")
    );
}

#[test]
fn bh1119_ms_i4_emit_record_no_card_text_refused() {
    let d = tempfile::tempdir().unwrap();
    refused(
        &[
            "hashlock",
            "--hashlock-phrase-stdin",
            "--emit-record",
            "--no-engraving-card",
        ],
        "trail",
        d.path(),
    );
}

#[test]
fn bh1119_ms_i4_emit_record_no_card_argv_phrase_refused() {
    let d = tempfile::tempdir().unwrap();
    refused(
        &[
            "hashlock",
            "--hashlock-phrase",
            "trail",
            "--allow-argv-secret",
            "--emit-record",
            "--no-engraving-card",
        ],
        "",
        d.path(),
    );
}

#[test]
fn bh1119_ms_i4_emit_record_no_card_kind_refused() {
    let d = tempfile::tempdir().unwrap();
    refused(
        &[
            "hashlock",
            "--hashlock-phrase-stdin",
            "--emit-record",
            "--no-engraving-card",
            "--kind",
            "ripemd160",
        ],
        "trail",
        d.path(),
    );
}

#[test]
fn bh1119_ms_i4_emit_record_no_card_out_refused_before_write() {
    let d = tempfile::tempdir().unwrap();
    refused(
        &[
            "hashlock",
            "--hashlock-phrase-stdin",
            "--emit-record",
            "--no-engraving-card",
            "--out",
            "p.ms1",
        ],
        "trail",
        d.path(),
    );
    assert!(
        !d.path().join("p.ms1").exists(),
        "--out must not be created"
    );
}

#[test]
fn bh1119_ms_i4_emit_record_no_card_refused_before_reading_phrase() {
    let d = tempfile::tempdir().unwrap();
    refused(
        &[
            "hashlock",
            "--hashlock-phrase-stdin",
            "--emit-record",
            "--no-engraving-card",
        ],
        "",
        d.path(),
    );
}

#[test]
fn bh1119_ms_i4_control_card_carries_record() {
    let d = tempfile::tempdir().unwrap();
    let r = ms(d.path())
        .args(["hashlock", "--hashlock-phrase-stdin", "--emit-record"])
        .write_stdin("trail")
        .assert()
        .success();
    let o = r.get_output();
    assert_eq!(String::from_utf8_lossy(&o.stdout), HASH_LINE);
    let err = String::from_utf8_lossy(&o.stderr);
    let want = format!("record (phrase): {REC}");
    assert_eq!(err.lines().filter(|l| *l == want).count(), 1, "{err}");
}

#[test]
fn bh1119_ms_i4_control_json_no_card_carries_record() {
    let d = tempfile::tempdir().unwrap();
    let r = ms(d.path())
        .args([
            "hashlock",
            "--hashlock-phrase-stdin",
            "--emit-record",
            "--json",
            "--no-engraving-card",
        ])
        .write_stdin("trail")
        .assert()
        .success();
    let o = r.get_output();
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["phrase_record"], REC);
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(err.lines().count(), 1, "{err}");
    assert!(
        err.starts_with("warning: stdout carries private key material (can spend)"),
        "{err}"
    );
}

#[test]
fn bh1119_ms_i4_control_json_with_card_carries_record() {
    let d = tempfile::tempdir().unwrap();
    let r = ms(d.path())
        .args([
            "hashlock",
            "--hashlock-phrase-stdin",
            "--emit-record",
            "--json",
        ])
        .write_stdin("trail")
        .assert()
        .success();
    let o = r.get_output();
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["phrase_record"], REC);
    let err = String::from_utf8_lossy(&o.stderr);
    let want = "record (phrase): in the JSON (phrase_record)";
    assert_eq!(err.lines().filter(|l| *l == want).count(), 1, "{err}");
}

#[test]
fn bh1119_ms_i4_control_no_phrase_source_refusal_first() {
    let d = tempfile::tempdir().unwrap();
    let r = ms(d.path())
        .args([
            "hashlock",
            "--random",
            "--out",
            "r.ms1",
            "--emit-record",
            "--no-engraving-card",
        ])
        .assert()
        .code(64);
    let err = String::from_utf8_lossy(&r.get_output().stderr).to_string();
    assert_eq!(err.lines().count(), 1, "{err}");
    assert!(
        err.starts_with("error: --emit-record needs a phrase:"),
        "{err}"
    );
    assert!(!d.path().join("r.ms1").exists());
}

#[test]
fn bh1119_ms_i4_control_no_card_without_flag_unchanged() {
    let d = tempfile::tempdir().unwrap();
    let r = ms(d.path())
        .args(["hashlock", "--hashlock-phrase-stdin", "--no-engraving-card"])
        .write_stdin("trail")
        .assert()
        .success();
    let o = r.get_output();
    assert_eq!(String::from_utf8_lossy(&o.stdout), HASH_LINE);
    assert!(!String::from_utf8_lossy(&o.stderr).contains("phrase:"));
}
