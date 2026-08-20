//! A recombined secret must still control the SAME funds.
//!
//! P2 of the constellation journey recon (2026-08-19). The recon measured that
//! **no test anywhere chained `split` → `combine` → `derive`**: `cli_split.rs`
//! and `cli_combine.rs` prove the shares round-trip to the original bytes, and
//! `cli_derive.rs` proves derivation works from a card/hex/phrase — but nothing
//! joined them. Verified before writing this: the only two test files
//! mentioning both `combine` and `derive` use the word incidentally (a doc
//! comment about entropy-vs-phrase, and a man-page message).
//!
//! So the structural half was covered and the functional half was covered, and
//! the property an operator actually depends on — *the shares I engraved will
//! give me back keys that control my coins* — was covered by neither.
//!
//! This is the journey definition's two-equality rule applied to K-of-N:
//! STRUCTURAL (the recombined phrase is byte-identical) **and** FUNCTIONAL (it
//! derives the same master fingerprint and account xpub). Neither alone is
//! sufficient: bytes can match while derivation is broken, and a derivation can
//! coincide while the secret silently changed.

use assert_cmd::Command;
use serde_json::Value;

/// BIP-39's own published all-zero-entropy vector. Public by construction.
const ABANDON: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fn ms(args: &[&str]) -> String {
    let out = Command::cargo_bin("ms")
        .unwrap()
        .args(args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(out).unwrap()
}

/// `ms split --json` → the share strings.
fn split(phrase: &str, k: &str, n: &str) -> Vec<String> {
    let out = ms(&["split", "--phrase", phrase, "-k", k, "-n", n, "--json"]);
    let v: Value = serde_json::from_str(&out).unwrap();
    v["shares"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

/// Pull a `key: value` line out of `ms` output.
fn field(out: &str, key: &str) -> String {
    out.lines()
        .find_map(|l| l.strip_prefix(key))
        .unwrap_or_else(|| panic!("no `{key}` line in:\n{out}"))
        .trim()
        .to_string()
}

/// `ms derive --template bip84` → (master_fingerprint, account_xpub).
fn derive(phrase: &str) -> (String, String) {
    let out = ms(&["derive", "--phrase", phrase, "--template", "bip84"]);
    (
        field(&out, "master_fingerprint:"),
        field(&out, "account_xpub:"),
    )
}

#[test]
fn recombined_secret_derives_the_same_keys() {
    let (want_fp, want_xpub) = derive(ABANDON);

    let shares = split(ABANDON, "2", "3");
    assert_eq!(shares.len(), 3, "2-of-3 must emit three shares");

    // Every K-subset must work, not just the first: a combine that silently
    // depended on share ORDER or on a particular share would pass a single-pair
    // test and lose funds for an operator who reached for a different two.
    let subsets = [(0, 1), (0, 2), (1, 2), (2, 0)];
    for (a, b) in subsets {
        let combined = ms(&["combine", &shares[a], &shares[b]]);
        let phrase = field(&combined, "phrase:");

        // STRUCTURAL equality.
        assert_eq!(
            phrase, ABANDON,
            "shares {a}+{b}: recombined phrase differs from the original"
        );

        // FUNCTIONAL equality — the half that was missing. Bytes matching is
        // not proof that the keys controlling the coins match.
        let (got_fp, got_xpub) = derive(&phrase);
        assert_eq!(
            got_fp, want_fp,
            "shares {a}+{b}: recombined secret derives a DIFFERENT master fingerprint"
        );
        assert_eq!(
            got_xpub, want_xpub,
            "shares {a}+{b}: recombined secret derives a DIFFERENT account xpub"
        );
    }
}

/// ANTI-VACUITY. If `derive` returned a constant, or the comparison were
/// vacuous, the test above would pass no matter what `combine` produced. A
/// different secret must derive different keys.
#[test]
fn a_different_secret_derives_different_keys() {
    let other = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";
    let (fp_a, xpub_a) = derive(ABANDON);
    let (fp_b, xpub_b) = derive(other);
    assert_ne!(fp_a, fp_b, "distinct secrets must not share a fingerprint");
    assert_ne!(xpub_a, xpub_b, "distinct secrets must not share an xpub");
}
