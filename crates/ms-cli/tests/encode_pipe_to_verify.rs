//! End-to-end pipe round-trip: ms encode | ms verify -.

use assert_cmd::Command;

#[test]
fn encode_pipe_to_verify() {
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

    // Pipe encoded multi-line stdout into verify - via stdin.
    Command::cargo_bin("ms")
        .unwrap()
        .args(["verify", "-"])
        .write_stdin(encoded_str)
        .assert()
        .success();
}
