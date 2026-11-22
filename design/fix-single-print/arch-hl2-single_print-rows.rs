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
