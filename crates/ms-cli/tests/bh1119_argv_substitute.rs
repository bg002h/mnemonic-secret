//! bh-ms-C1 / bh-ms-C2: under `--allow-argv-secret` the substitution pass must
//! not rewrite the value of `--in` / `--out`, and the `=` spelling of a secret
//! flag admits the same bytes as the space spelling.
//!
//! One `#[test]` per row of `design/FIX_bughunt1119_ms_argv_substitute.md`
//! section 4, named as the row. Each runs the built `ms` in its own tempdir
//! with empty stdin (`write_stdin("")`).

use assert_cmd::Command;
use std::path::Path;

const P12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const P15: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon address";
const C12: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
const C15: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqke34pjrgn5p6k";
const PLATE: &str = "ms10hashsqvxdut60pd6jqph6ld525scg2cgjd7wkfqwtv5d9ym0whh6462afc7jgxua504u3ms";
const SH1: &str = "ms124gwqqj5pjwlwrxuxsc8mhrxrhpjh7ny3q8mw8zv43meqgf";
const SH2: &str = "ms124gwqpklhk4n4sfrf3d7a0sfs0hk0ypwxqdj0vx0ucnm0ks";
const N: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const N64: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const SIX: &str = "59413e908f3b72e4ce859bf153fffc8054615bc3e445f7ac5e7b2adb9b8bbe70";

struct Out {
    rc: i32,
    out: String,
    err: String,
}

fn run(dir: &Path, args: &[&str]) -> Out {
    let o = Command::cargo_bin("ms")
        .unwrap()
        .current_dir(dir)
        .args(args)
        .write_stdin("")
        .output()
        .unwrap();
    Out {
        rc: o.status.code().unwrap_or(-1),
        out: String::from_utf8_lossy(&o.stdout).into_owned(),
        err: String::from_utf8_lossy(&o.stderr).into_owned(),
    }
}

fn put(dir: &Path, name: &str, body: &str) {
    std::fs::write(dir.join(name), format!("{body}\n")).unwrap();
}

fn word_count(o: &Out) -> String {
    let k = "\"word_count\":";
    let i = o.out.find(k).map(|i| i + k.len()).unwrap_or(0);
    o.out[i..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect()
}

fn preimage(o: &Out) -> String {
    let k = "\"preimage_hex\":\"";
    let i = o.out.find(k).map(|i| i + k.len()).unwrap_or(0);
    o.out[i..]
        .chars()
        .take_while(|c| c.is_ascii_hexdigit())
        .collect()
}

#[test]
fn argv_sub_in_space_reads_named_file() {
    let d = tempfile::tempdir().unwrap();
    put(d.path(), N, P15);
    put(d.path(), "-", P12);
    let o = run(
        d.path(),
        &["encode", "--allow-argv-secret", "--in", N, "--json"],
    );
    assert_eq!(o.rc, 0, "{}", o.err);
    assert_eq!(word_count(&o), "15", "{}", o.out);
}

#[test]
fn argv_sub_in_equals_reads_named_file() {
    let d = tempfile::tempdir().unwrap();
    put(d.path(), N, P15);
    put(d.path(), "-", P12);
    let arg = format!("--in={N}");
    let o = run(d.path(), &["encode", "--allow-argv-secret", &arg, "--json"]);
    assert_eq!(o.rc, 0, "{}", o.err);
    assert_eq!(word_count(&o), "15", "{}", o.out);
}

#[test]
fn argv_sub_in_64_hex_name_reads_named_file() {
    let d = tempfile::tempdir().unwrap();
    put(d.path(), N64, C15);
    put(d.path(), "-", C12);
    let o = run(
        d.path(),
        &["inspect", "--allow-argv-secret", "--in", N64, "--json"],
    );
    assert_eq!(o.rc, 0, "{}", o.err);
    assert!(
        o.out
            .contains("\"payload_bytes_hex\":\"0000000000000000000000000000000000000000\""),
        "{}",
        o.out
    );
}

#[test]
fn argv_sub_in_ms1_shaped_name_reads_named_file() {
    let d = tempfile::tempdir().unwrap();
    put(d.path(), C12, C15);
    put(d.path(), "-", C12);
    let o = run(d.path(), &["verify", "--allow-argv-secret", "--in", C12]);
    assert_eq!(o.rc, 0, "{}", o.err);
    assert_eq!(
        o.out.lines().next().unwrap_or(""),
        "OK: valid v0.1 entr (15 words, 56 chars)"
    );
}

#[test]
fn argv_sub_in_missing_named_file_fails() {
    let want = format!("error: failed to read --in {N}: No such file or directory (os error 2)\n");
    for v in [
        "encode", "decode", "inspect", "verify", "derive", "hashlock", "repair", "split", "combine",
    ] {
        let d = tempfile::tempdir().unwrap();
        match v {
            "encode" | "split" => put(d.path(), "-", P12),
            "hashlock" => put(d.path(), "-", PLATE),
            "combine" => put(d.path(), "-", &format!("{SH1}\n{SH2}")),
            _ => put(d.path(), "-", C12),
        }
        let args: Vec<&str> = match v {
            "derive" => vec![
                "derive",
                "--allow-argv-secret",
                "--in",
                N,
                "--network",
                "testnet",
            ],
            "hashlock" => vec![
                "hashlock",
                "--allow-argv-secret",
                "--in",
                N,
                "--kind",
                "sha256",
            ],
            "split" => vec![
                "split",
                "--allow-argv-secret",
                "-k",
                "2",
                "-n",
                "3",
                "--in",
                N,
            ],
            _ => vec![v, "--allow-argv-secret", "--in", N],
        };
        let o = run(d.path(), &args);
        assert_eq!(o.rc, 32, "case {v}: {}", o.err);
        assert_eq!(o.err, want, "case {v}");
        assert!(o.out.is_empty(), "case {v}: stdout {}", o.out);
    }
}

#[test]
fn argv_sub_out_space_creates_named_file() {
    for v in ["encode", "hashlock", "repair", "split"] {
        let d = tempfile::tempdir().unwrap();
        put(d.path(), "seed.txt", P12);
        put(d.path(), "card.ms1", C12);
        let args: Vec<&str> = match v {
            "encode" => vec![
                "encode",
                "--allow-argv-secret",
                "--in",
                "seed.txt",
                "--out",
                N,
            ],
            "hashlock" => vec![
                "hashlock",
                "--allow-argv-secret",
                "--random",
                "--kind",
                "sha256",
                "--no-engraving-card",
                "--out",
                N,
            ],
            "repair" => vec![
                "repair",
                "--allow-argv-secret",
                "--in",
                "card.ms1",
                "--out",
                N,
            ],
            _ => vec![
                "split",
                "--allow-argv-secret",
                "-k",
                "2",
                "-n",
                "3",
                "--in",
                "seed.txt",
                "--out",
                N,
            ],
        };
        let o = run(d.path(), &args);
        assert_eq!(o.rc, 0, "case {v}: {}", o.err);
        let named = d.path().join(N);
        assert!(named.exists(), "case {v}: named file absent");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&named).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600, "case {v}: mode {mode:o}");
        }
        assert!(!d.path().join("-").exists(), "case {v}: stray ./-");
    }
}

#[test]
fn argv_sub_out_space_card_bytes() {
    let d = tempfile::tempdir().unwrap();
    put(d.path(), "seed.txt", P12);
    let o = run(
        d.path(),
        &[
            "encode",
            "--allow-argv-secret",
            "--in",
            "seed.txt",
            "--out",
            N,
        ],
    );
    assert_eq!(o.rc, 0, "{}", o.err);
    let c = std::fs::read_to_string(d.path().join(N)).unwrap_or_default();
    assert_eq!(c.trim_end(), C12);
}

#[test]
fn argv_sub_out_equals_creates_named_file() {
    let d = tempfile::tempdir().unwrap();
    put(d.path(), "seed.txt", P12);
    let arg = format!("--out={N}");
    let o = run(
        d.path(),
        &["encode", "--allow-argv-secret", "--in", "seed.txt", &arg],
    );
    assert_eq!(o.rc, 0, "{}", o.err);
    let c = std::fs::read_to_string(d.path().join(N)).unwrap_or_default();
    assert_eq!(c.trim_end(), C12);
    assert!(!d.path().join("-").exists(), "stray ./-");
}

#[test]
fn argv_sub_hashlock_phrase_equals_keeps_trailing_space() {
    let d = tempfile::tempdir().unwrap();
    let sp = run(
        d.path(),
        &[
            "hashlock",
            "--allow-argv-secret",
            "--hashlock-phrase",
            "trail ",
            "--method",
            "sha256",
            "--json",
        ],
    );
    assert_eq!(preimage(&sp), SIX, "space spelling drifted: {}", sp.err);
    let eq = run(
        d.path(),
        &[
            "hashlock",
            "--allow-argv-secret",
            "--hashlock-phrase=trail ",
            "--method",
            "sha256",
            "--json",
        ],
    );
    assert_eq!(eq.rc, 0, "{}", eq.err);
    assert_eq!(preimage(&eq), SIX, "{}", eq.out);
    assert!(eq.out.contains("\"phrase_chars\":6"), "{}", eq.out);
}

// ---- controls: hold before and after ----

#[test]
fn argv_sub_after_dashdash_still_substituted() {
    let d = tempfile::tempdir().unwrap();
    let o = run(
        d.path(),
        &["decode", "--allow-argv-secret", "--", "--in", N],
    );
    assert_eq!(o.rc, 64, "{}", o.err);
    assert!(o.err.contains("unexpected argument '-' found"), "{}", o.err);
    assert!(!o.err.contains(N), "{}", o.err);
}

#[test]
fn argv_sub_flag_match_is_exact() {
    let d = tempfile::tempdir().unwrap();
    let o = run(d.path(), &["decode", "--allow-argv-secret", "--IN", N]);
    assert_eq!(o.rc, 64, "{}", o.err);
    assert!(
        o.err.contains("unexpected argument '--IN' found"),
        "{}",
        o.err
    );
    assert!(!o.err.contains(N), "{}", o.err);
}

#[test]
fn argv_sub_non_path_value_still_substituted() {
    let d = tempfile::tempdir().unwrap();
    put(d.path(), "card.ms1", C12);
    let digits = "12345678901234567890123456789012";
    let o = run(
        d.path(),
        &[
            "derive",
            "--allow-argv-secret",
            "--in",
            "card.ms1",
            "--account",
            digits,
        ],
    );
    assert_eq!(o.rc, 64, "{}", o.err);
    assert!(
        o.err
            .contains("invalid value '-' for '--account <ACCOUNT>'"),
        "{}",
        o.err
    );
    assert!(!o.err.contains("1234567890"), "{}", o.err);
}

#[test]
fn argv_sub_without_override_still_refuses() {
    let d = tempfile::tempdir().unwrap();
    put(d.path(), N, P15);
    let o = run(d.path(), &["encode", "--in", N, "--json"]);
    assert_eq!(o.rc, 8, "{}", o.err);
    assert!(
        o.err.contains("is raw hex entropy, 32 characters long"),
        "{}",
        o.err
    );
    assert!(o.out.is_empty(), "{}", o.out);
}

#[test]
fn argv_sub_hashlock_phrase_equals_keeps_leading_space() {
    let d = tempfile::tempdir().unwrap();
    let base = ["hashlock", "--allow-argv-secret"];
    let tail = ["--method", "sha256", "--json"];
    let eq = run(
        d.path(),
        &[&base[..], &["--hashlock-phrase= trail"], &tail[..]].concat(),
    );
    let sp = run(
        d.path(),
        &[&base[..], &["--hashlock-phrase", " trail"], &tail[..]].concat(),
    );
    assert_eq!(eq.rc, 0, "{}", eq.err);
    assert!(!preimage(&eq).is_empty(), "{}", eq.out);
    assert_eq!(preimage(&eq), preimage(&sp));
}

#[test]
fn argv_sub_equals_matches_space() {
    let zeros = "00000000000000000000000000000000";
    for (f, verb, val) in [
        ("phrase", "encode", format!(" {P12} ")),
        ("hex", "encode", format!(" {zeros} ")),
        ("ms1", "repair", format!(" {C12} ")),
    ] {
        let d1 = tempfile::tempdir().unwrap();
        let d2 = tempfile::tempdir().unwrap();
        let eqarg = format!("--{f}={val}");
        let spflag = format!("--{f}");
        let eq = run(d1.path(), &[verb, "--allow-argv-secret", &eqarg, "--json"]);
        let sp = run(
            d2.path(),
            &[verb, "--allow-argv-secret", &spflag, &val, "--json"],
        );
        assert_eq!(eq.rc, 0, "case {f}: {}", eq.err);
        assert_eq!(sp.rc, 0, "case {f}: {}", sp.err);
        assert_eq!(eq.out, sp.out, "case {f}");
    }
}
