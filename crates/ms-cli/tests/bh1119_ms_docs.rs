//! FIX_bughunt1119_ms_docs (bh-ms-I8, CLI half). Every test asserts the
//! behaviour the FIX document specifies.

use std::path::Path;
use std::process::Command;

use assert_cmd::cargo::CommandCargoExt;

const BROKEN: &str = "ms10entrsqqqqqqqqqqqzqqqqqqqqqqqqqqqqcj9sxraq34v7f";
const FIXED: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

fn readme() -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(env!("CARGO_PKG_README"));
    std::fs::read_to_string(p).expect("read ms-cli README")
}

fn repair(json: bool) -> (std::process::Output, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("broken.txt"), format!("{BROKEN}\n")).unwrap();
    let mut c = Command::cargo_bin("ms").unwrap();
    c.current_dir(dir.path()).args(["repair", "--in", "broken.txt"]);
    if json {
        c.arg("--json");
    }
    (c.output().unwrap(), dir)
}

#[test]
fn bh1119_ms_i8_readme_exit_table_documents_candidate_exit_4() {
    let r = readme();
    assert!(
        r.lines().any(|l| l.starts_with("| `4` | **`VERIFY-ME` Candidate**")),
        "README exit table has no `4` VERIFY-ME Candidate row (the code a correction returns)"
    );
    assert!(
        !r.lines().any(|l| l.starts_with("| `5` |")),
        "README exit table still has a `5` row"
    );
}

#[test]
fn bh1119_ms_i8_readme_repair_example_matches_cli() {
    let r = readme();
    assert!(r.contains(BROKEN), "README lacks the broken card");
    let (out, _dir) = repair(false);
    assert_eq!(out.status.code(), Some(4));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout.lines().last(), Some(FIXED));
    let stderr = String::from_utf8(out.stderr).unwrap();
    for l in stderr.lines() {
        let want = format!("#   {l}");
        assert!(
            r.lines().any(|x| x == want),
            "README example stderr lacks: {l}"
        );
    }
    assert!(
        r.lines().any(|x| x == "# exit status: 4"),
        "README example has no exit status line"
    );
}

#[test]
fn bh1119_ms_i8_control_readme_json_example_is_candidate() {
    let (out, _dir) = repair(true);
    assert_eq!(out.status.code(), Some(4));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["verdict"], "candidate");
    assert_eq!(v["corrected_chunks"][0], FIXED);
}
