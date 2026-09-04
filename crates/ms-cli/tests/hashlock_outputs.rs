//! stdout purity in the two configurations where a mutation can hide, the
//! card per source and method, the warnings at their boundaries, `--json`
//! in both variants (SPEC_ms_hashlock §4.4, §7; tests I-7, I-8, M-10).

use assert_cmd::Command;

const PHRASE: &str = "correct horse battery staple";
const H_HARD: &str = "hash:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12";
const H_SHA: &str = "hash:b867db875479bcc0287352cdaa4a1755689b8338777d0915e9acd9f6edbc96cb";
const HEX32: &str = "c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016";

fn ms() -> Command {
    Command::cargo_bin("ms").unwrap()
}

/// MUTATION: `--out` suppressing stdout (encode's shape) -> the first case
/// has empty stdout; a warning printed to stdout -> the second case has two
/// lines.
#[test]
fn stdout_is_exactly_the_record_under_out_and_under_sha256() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("x.txt");
    let out = ms()
        .args([
            "hashlock",
            "--hashlock-phrase-stdin",
            "--out",
            p.to_str().unwrap(),
        ])
        .write_stdin(PHRASE)
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        format!("{H_HARD}\n"),
        "under --out"
    );
    let out = ms()
        .args(["hashlock", "--hashlock-phrase-stdin", "--method", "sha256"])
        .write_stdin(PHRASE)
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        format!("{H_SHA}\n"),
        "under sha256, which always warns"
    );
    assert!(String::from_utf8_lossy(&out.stderr).contains("brainwallet"));
}

#[test]
fn the_card_names_the_preimage_on_its_first_line_and_carries_the_method_line() {
    let out = ms()
        .args(["hashlock", "--hashlock-phrase-stdin"])
        .write_stdin(PHRASE)
        .output()
        .unwrap();
    let se = String::from_utf8_lossy(&out.stderr);
    let first = se.lines().next().unwrap();
    assert!(
        first.to_ascii_uppercase().contains("PREIMAGE"),
        "first line: {first}"
    );
    assert!(se.contains("preimage = PBKDF2-HMAC-SHA256(password = phrase, salt = \"ms-hashlock-v1\", iterations = 100000, dkLen = 32)"), "{se}");
    assert!(se.contains("28 characters"), "the character count:\n{se}");
    assert!(
        se.contains("each method that shipped with the version named on this card"),
        "{se}"
    );
    assert!(se.contains("One phrase per policy"), "{se}");
    assert!(
        se.contains("OP_SIZE 32") || se.contains("32 bytes (64 hex characters)"),
        "{se}"
    );
}

/// MUTATION: hardened threshold at 19 or 21 -> one of these flips.
#[test]
fn hardened_warns_under_20_only() {
    let se19 = String::from_utf8_lossy(
        &ms()
            .args(["hashlock", "--hashlock-phrase-stdin"])
            .write_stdin("a".repeat(19))
            .output()
            .unwrap()
            .stderr,
    )
    .to_string();
    let se20 = String::from_utf8_lossy(
        &ms()
            .args(["hashlock", "--hashlock-phrase-stdin"])
            .write_stdin("a".repeat(20))
            .output()
            .unwrap()
            .stderr,
    )
    .to_string();
    assert!(se19.contains("72 days"), "19 chars must warn:\n{se19}");
    assert!(!se20.contains("72 days"), "20 chars must not warn:\n{se20}");
}

/// MUTATION: sha256 gated on length -> the 100-char case stops warning.
#[test]
fn sha256_warns_at_every_length() {
    for n in [1usize, 28, 100] {
        let se = String::from_utf8_lossy(
            &ms()
                .args(["hashlock", "--hashlock-phrase-stdin", "--method", "sha256"])
                .write_stdin("b".repeat(n))
                .output()
                .unwrap()
                .stderr,
        )
        .to_string();
        assert!(se.contains("brainwallet"), "{n} chars:\n{se}");
    }
}

#[test]
fn hex_source_gets_the_unconditional_warning_and_no_write_it_down_line() {
    let out = ms()
        .args(["hashlock", "--hex", "-"])
        .write_stdin(HEX32)
        .output()
        .unwrap();
    let se = String::from_utf8_lossy(&out.stderr);
    assert!(se.contains("publishes these 32 bytes in the clear"), "{se}");
    assert!(se.contains("preimage supplied"), "{se}");
    assert!(
        !se.contains("write the method line next to your phrase"),
        "no phrase, no instruction:\n{se}"
    );
    assert!(
        !se.contains("brainwallet") && !se.contains("72 days"),
        "method-keyed warnings must not fire:\n{se}"
    );
}

#[test]
fn random_card_names_the_file_not_a_plate() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("x.txt");
    let out = ms()
        .args(["hashlock", "--random", "--out", p.to_str().unwrap()])
        .output()
        .unwrap();
    let se = String::from_utf8_lossy(&out.stderr);
    assert!(se.contains("nothing can be remembered"), "{se}");
    assert!(
        se.contains("The file you just wrote is the only copy"),
        "{se}"
    );
    assert!(!se.contains("This plate is the only copy"), "{se}");
}

/// Both `--json` variants; every hex lowercase; the advisory fires.
#[test]
fn json_both_variants() {
    let out = ms()
        .args(["hashlock", "--hashlock-phrase-stdin", "--json"])
        .write_stdin(PHRASE)
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["hash_record"], H_HARD);
    assert_eq!(v["method"]["iterations"], 100000);
    assert_eq!(v["method"]["salt"], "ms-hashlock-v1");
    assert_eq!(v["phrase_chars"], 28);
    for k in ["digest", "preimage_hex", "sha256_operand"] {
        let s = v[k].as_str().unwrap();
        assert_eq!(s, s.to_ascii_lowercase(), "{k} must be lowercase hex");
    }
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("private key material")
            || String::from_utf8_lossy(&out.stderr)
                .to_ascii_lowercase()
                .contains("secret")
    );
    let out = ms()
        .args(["hashlock", "--hex", "-", "--json"])
        .write_stdin(HEX32)
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(
        v.get("method").is_none(),
        "method omitted for a supplied preimage"
    );
    assert!(
        v.get("phrase_chars").is_none(),
        "phrase_chars omitted for a supplied preimage"
    );
    assert_eq!(v["preimage_hex"], HEX32);
}

/// `--random --json --out FILE` succeeds (the gate is on --out, not on json).
#[test]
fn random_json_with_out_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("x.txt");
    let out = ms()
        .args([
            "hashlock",
            "--random",
            "--out",
            p.to_str().unwrap(),
            "--json",
            "--no-engraving-card",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(v["preimage_ms1"]
        .as_str()
        .unwrap()
        .starts_with("ms10hashsq"));
}

/// The record's SHAPE is what `me sysw pack` reads from stdin (§12.6: no
/// `--in`). A pure shape check; the cross-repo run is acceptance item 6.
#[test]
fn record_line_shape_is_what_me_sysw_pack_reads() {
    let out = ms()
        .args(["hashlock", "--hashlock-phrase-stdin", "--no-engraving-card"])
        .write_stdin(PHRASE)
        .output()
        .unwrap();
    let line = String::from_utf8_lossy(&out.stdout);
    assert!(line.starts_with("hash:") && line.trim().len() == 5 + 64);
    assert!(line.trim()[5..]
        .bytes()
        .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()));
}
