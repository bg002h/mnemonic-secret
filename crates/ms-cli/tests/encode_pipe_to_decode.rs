//! End-to-end pipe round-trip: ms encode | ms decode - recovers the phrase.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn encode_pipe_to_decode_recovers_phrase() {
    let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let encoded = Command::cargo_bin("ms")
        .unwrap()
        .args(["encode", "--phrase", phrase])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let encoded_str = String::from_utf8(encoded).unwrap();

    Command::cargo_bin("ms")
        .unwrap()
        .args(["decode", "-"])
        .write_stdin(encoded_str)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            format!("phrase: {}", phrase).as_str(),
        ));
}
