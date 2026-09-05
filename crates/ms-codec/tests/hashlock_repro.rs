//! The cross-tool reproduction (SPEC_ms_hashlock §8), written so it cannot
//! lie (R0 r0 tests I-9, FP-1..FP-5):
//!
//! - The salt, the iteration count, the dkLen and every expected hex are
//!   LITERALS here, independent of the crate's constants. A separate
//!   assertion pins the constants to the literals. Mutating a constant
//!   therefore moves ONE side of the comparison, and the test fails.
//! - Both external tools are RUN and their CAPTURED STDOUT compared, three
//!   ways: Rust = python, Rust = openssl, python = openssl.
//! - A missing tool FAILS the test. There is no `#[ignore]` and no cfg gate;
//!   CI additionally asserts this test ran by name (rust.yml).
//! - KNOWN LIMIT (R0 r0 tests I-3): a SHADOWED tool -- a `python3` on PATH
//!   that echoes the expected hex -- defeats any shell-out comparison. CI logs
//!   `python3 -VV` and `openssl version`; a compromised runner is out of scope.

use std::process::Command;

use ms_codec::hashlock::{
    digest, preimage_hardened, HASHLOCK_DKLEN, HASHLOCK_ITERATIONS, HASHLOCK_SALT,
};

// LITERALS. Not the crate's constants.
const SALT: &str = "ms-hashlock-v1";
const ITER: u32 = 100_000;
const DKLEN: usize = 32;
const PHRASE: &str = "correct horse battery staple";
const EXPECTED_X: &str = "c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016";
const EXPECTED_H: &str = "3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12";

fn hex(b: &[u8]) -> String {
    use std::fmt::Write;
    b.iter()
        .fold(String::with_capacity(b.len() * 2), |mut s, x| {
            let _ = write!(s, "{x:02x}");
            s
        })
}

#[test]
fn constants_equal_the_literals() {
    assert_eq!(HASHLOCK_SALT, SALT.as_bytes());
    assert_eq!(HASHLOCK_ITERATIONS, ITER);
    assert_eq!(HASHLOCK_DKLEN, DKLEN);
}

fn python_x() -> String {
    // PHRASE and SALT are plain ASCII with no quotes, so single-quoted byte
    // literals are exact.
    let script = format!(
        "import hashlib,sys;x=hashlib.pbkdf2_hmac('sha256',b'{PHRASE}',b'{SALT}',{ITER},{DKLEN});sys.stdout.write(x.hex())"
    );
    let out = Command::new("python3")
        .args(["-c", &script])
        .output()
        .expect("python3 must be present: this test FAILS on a missing tool, never skips");
    assert!(
        out.status.success(),
        "python3 failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

fn openssl_x() -> String {
    let keylen = DKLEN.to_string();
    let pass = format!("pass:{PHRASE}");
    let salt = format!("salt:{SALT}");
    let iter = format!("iter:{ITER}");
    let out = Command::new("openssl")
        .args([
            "kdf",
            "-keylen",
            &keylen,
            "-kdfopt",
            "digest:SHA256",
            "-kdfopt",
            &pass,
            "-kdfopt",
            &salt,
            "-kdfopt",
            &iter,
            "PBKDF2",
        ])
        .output()
        .expect("openssl must be present: this test FAILS on a missing tool, never skips");
    assert!(
        out.status.success(),
        "openssl kdf failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // openssl prints `AB:CD:...`; normalise to lowercase hex.
    String::from_utf8(out.stdout)
        .unwrap()
        .trim()
        .replace(':', "")
        .to_ascii_lowercase()
}

#[test]
fn hashlock_repro_three_ways() {
    let rust_x = hex(&preimage_hardened(PHRASE.as_bytes())[..]);
    let py = python_x();
    let ssl = openssl_x();
    assert_eq!(rust_x, EXPECTED_X, "Rust vs literal");
    assert_eq!(py, EXPECTED_X, "python vs literal");
    assert_eq!(ssl, EXPECTED_X, "openssl vs literal");
    assert_eq!(py, ssl, "python vs openssl");
    let mut x = [0u8; 32];
    for (i, b) in x.iter_mut().enumerate() {
        *b = u8::from_str_radix(&EXPECTED_X[2 * i..2 * i + 2], 16).unwrap();
    }
    assert_eq!(hex(&digest(&x)), EXPECTED_H, "digest of the literal X");
}
