//! ms vectors --pretty emits indented JSON with same content.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn vectors_pretty_is_indented_and_parseable() {
    let compact = Command::cargo_bin("ms")
        .unwrap()
        .arg("vectors")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let pretty = Command::cargo_bin("ms")
        .unwrap()
        .args(["vectors", "--pretty"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\n"))
        .get_output()
        .stdout
        .clone();

    let cs: serde_json::Value = serde_json::from_slice(&compact).unwrap();
    let ps: serde_json::Value = serde_json::from_slice(&pretty).unwrap();
    assert_eq!(cs, ps);
}
