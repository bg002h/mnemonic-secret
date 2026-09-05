//! Source arithmetic and the argv guard on `ms hashlock` (SPEC_ms_hashlock
//! §4.1, §6). Each test names the mutation it fails under.

use assert_cmd::Command;

const PHRASE: &str = "correct horse battery staple";
const HEX32: &str = "c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016";

fn ms() -> Command {
    Command::cargo_bin("ms").unwrap()
}

/// MUTATION: drop `--hashlock-phrase` from SECRET_FLAGS -> the value is
/// accepted on argv and this fails.
#[test]
fn hashlock_phrase_on_argv_is_refused_without_the_allow_flag_and_never_echoed() {
    let out = ms()
        .args(["hashlock", "--hashlock-phrase", PHRASE])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("a hashlock phrase"),
        "flag_class must name the class:\n{err}"
    );
    assert!(
        !err.contains(PHRASE),
        "the refusal echoed the phrase:\n{err}"
    );
    assert!(!err.contains("BIP-39 passphrase"), "wrong class:\n{err}");
}

/// MUTATION: leave `hashlock` out of `override_applies` -> the allow flag
/// does nothing and this exits 1.
#[test]
fn allow_argv_secret_admits_the_phrase_through_the_side_channel() {
    let out = ms()
        .args([
            "hashlock",
            "--allow-argv-secret",
            "--hashlock-phrase",
            PHRASE,
            "--no-engraving-card",
        ])
        .write_stdin("")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let so = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        so.trim(),
        "hash:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12"
    );
}

/// The §6 gate for part 4: the same invocation with stdin at /dev/null (an
/// EMPTY stdin here) still derives from the FLAG's value. MUTATION: build the
/// Source without `.on("--hashlock-phrase")` -> it reads stdin, gets nothing,
/// and refuses `empty`.
#[test]
fn admitted_phrase_does_not_read_stdin() {
    let out = ms()
        .args([
            "hashlock",
            "--allow-argv-secret",
            "--hashlock-phrase",
            PHRASE,
            "--no-engraving-card",
        ])
        .write_stdin("")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Same gate for the `--hex` channel (part 5). An empty pipe and /dev/null
/// both yield zero bytes, which is what the gate needs.
#[test]
fn admitted_hex_does_not_read_stdin() {
    let out = ms()
        .args([
            "hashlock",
            "--allow-argv-secret",
            "--hex",
            HEX32,
            "--no-engraving-card",
        ])
        .write_stdin("")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "hex: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Same gate for the positional (part 6).
#[test]
fn admitted_positional_does_not_read_stdin() {
    // Get a real plate string to pass positionally.
    let s = String::from_utf8(
        ms().args(["hashlock", "--hex", "-", "--json"])
            .write_stdin(HEX32)
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    let plate = v["preimage_ms1"].as_str().unwrap().to_string();
    let out = ms()
        .args([
            "hashlock",
            "--allow-argv-secret",
            &plate,
            "--no-engraving-card",
        ])
        .write_stdin("")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "positional: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// MUTATION: zero sources defaulting to stdin -> this hangs or parses stdin
/// as an ms1; expected is exit 64 listing five sources.
#[test]
fn zero_sources_exits_64_listing_five() {
    let out = ms().args(["hashlock"]).write_stdin("").output().unwrap();
    assert_eq!(out.status.code(), Some(64));
    let err = String::from_utf8_lossy(&out.stderr);
    for s in [
        "--hashlock-phrase",
        "--hashlock-phrase-stdin",
        "--hex",
        "--in",
        "--random",
    ] {
        assert!(err.contains(s), "usage must list {s}:\n{err}");
    }
}

/// Every one of the ten two-source pairs exits 64. MUTATION: check only a
/// subset of pairs -> the stdin-contention pair passes silently.
#[test]
fn every_two_source_pair_exits_64() {
    let sources: &[&[&str]] = &[
        &["--allow-argv-secret", "--hashlock-phrase", PHRASE],
        &["--hashlock-phrase-stdin"],
        &["--hex", "-"],
        &["-"],
        &["--random", "--out", "/tmp/ms-hashlock-pair-test.txt"],
    ];
    for i in 0..sources.len() {
        for j in (i + 1)..sources.len() {
            let mut args = vec!["hashlock"];
            args.extend_from_slice(sources[i]);
            args.extend_from_slice(sources[j]);
            // Two sources are refused BEFORE anything is read, so the stdin
            // contention pair (--hashlock-phrase-stdin with `-`) exits 64 too.
            let out = ms().args(&args).write_stdin(PHRASE).output().unwrap();
            assert_eq!(
                out.status.code(),
                Some(64),
                "pair {i},{j}: {}",
                String::from_utf8_lossy(&out.stderr)
            );
            assert!(
                String::from_utf8_lossy(&out.stderr).contains("were both given"),
                "pair {i},{j}: the message, not just the code"
            );
        }
    }
    let _ = std::fs::remove_file("/tmp/ms-hashlock-pair-test.txt");
}

/// MUTATION: `--method` silently ignored with a supplied X.
#[test]
fn method_with_a_supplied_preimage_exits_64_for_all_three_sources() {
    for args in [
        vec!["hashlock", "--hex", "-", "--method", "sha256"],
        vec![
            "hashlock",
            "--random",
            "--out",
            "/tmp/ms-hashlock-method-test.txt",
            "--method",
            "hardened",
        ],
        vec!["hashlock", "-", "--method", "sha256"],
    ] {
        // `--method` is refused BEFORE any source is read, so stdin's content
        // is irrelevant here; a raw hex value on argv would be refused by the
        // guard first (exit 1), which is why --hex reads stdin.
        let out = ms().args(&args).write_stdin(HEX32).output().unwrap();
        assert_eq!(
            out.status.code(),
            Some(64),
            "{args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        // The MESSAGE too, because clap's unknown-subcommand error is also 64.
        assert!(
            String::from_utf8_lossy(&out.stderr)
                .contains("--method applies to the phrase sources only"),
            "{args:?}"
        );
    }
    let _ = std::fs::remove_file("/tmp/ms-hashlock-method-test.txt");
}

/// L21 as narrowed: `--random` needs `--out FILE`; `--json` alone does not
/// satisfy it. MUTATION: gate on `--out || --json` -> the second case exits 0.
#[test]
fn random_requires_out_file_and_json_alone_does_not_satisfy_it() {
    let out = ms()
        .args(["hashlock", "--random", "--no-engraving-card"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(64));
    assert!(String::from_utf8_lossy(&out.stderr).contains("--out"));
    let out = ms()
        .args(["hashlock", "--random", "--json", "--no-engraving-card"])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(64),
        "--json alone must not satisfy the gate"
    );
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("x.txt");
    let out = ms()
        .args([
            "hashlock",
            "--random",
            "--out",
            p.to_str().unwrap(),
            "--no-engraving-card",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(p.exists());
}

/// `--random` twice gives two different records. MUTATION: a fixed buffer.
#[test]
fn random_twice_differs() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.txt");
    let b = dir.path().join("b.txt");
    let ra = ms()
        .args([
            "hashlock",
            "--random",
            "--out",
            a.to_str().unwrap(),
            "--no-engraving-card",
        ])
        .output()
        .unwrap();
    let rb = ms()
        .args([
            "hashlock",
            "--random",
            "--out",
            b.to_str().unwrap(),
            "--no-engraving-card",
        ])
        .output()
        .unwrap();
    assert_ne!(ra.stdout, rb.stdout);
    assert_ne!(std::fs::read(&a).unwrap(), std::fs::read(&b).unwrap());
}

/// C-2 as folded: under `--random`, `--out` refuses to overwrite and leaves
/// the file's bytes unchanged; the other sources overwrite. MUTATION: use
/// the truncating writer for `--random` -> Monday's preimage is gone.
#[test]
fn random_out_refuses_to_overwrite_but_other_sources_overwrite() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("preimage.txt");
    assert!(ms()
        .args([
            "hashlock",
            "--random",
            "--out",
            p.to_str().unwrap(),
            "--no-engraving-card"
        ])
        .output()
        .unwrap()
        .status
        .success());
    let monday = std::fs::read(&p).unwrap();
    let out = ms()
        .args([
            "hashlock",
            "--random",
            "--out",
            p.to_str().unwrap(),
            "--no-engraving-card",
        ])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(64),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stderr).contains(p.to_str().unwrap()));
    assert_eq!(
        std::fs::read(&p).unwrap(),
        monday,
        "the existing preimage must be untouched"
    );
    // A phrase source overwrites (its artifact is a function of its input).
    let out = ms()
        .args([
            "hashlock",
            "--hashlock-phrase-stdin",
            "--out",
            p.to_str().unwrap(),
            "--no-engraving-card",
        ])
        .write_stdin(PHRASE)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_ne!(std::fs::read(&p).unwrap(), monday);
}

/// §11: `--hex` at 63, 64 and 65 characters, upper and lower case. MUTATION:
/// parse with encode's helper -> the 63-char refusal names entropy lengths,
/// not "32 bytes (64 hex characters)" and §8i (R0 r0 fidelity I-6, I-9).
#[test]
fn hex_at_63_64_65_chars_both_cases() {
    for (n, ok) in [(63usize, false), (64, true), (65, false)] {
        for upper in [false, true] {
            let s: String = (0..n)
                .map(|i| "0123456789abcdef".as_bytes()[i % 16] as char)
                .collect();
            let s = if upper { s.to_ascii_uppercase() } else { s };
            let out = ms()
                .args(["hashlock", "--hex", "-", "--no-engraving-card"])
                .write_stdin(s.clone())
                .output()
                .unwrap();
            if ok {
                assert!(
                    out.status.success(),
                    "{n} {upper}: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
            } else {
                assert_eq!(out.status.code(), Some(1), "{n} {upper}");
                let err = String::from_utf8_lossy(&out.stderr);
                assert!(
                    err.contains("32 bytes (64 hex characters)") && err.contains("§8i"),
                    "{n} {upper}:\n{err}"
                );
                assert!(!err.contains(&s), "echoed the value:\n{err}");
            }
        }
    }
}

/// §11: the entr-32 pair string -- the COLLIDING length -- and a mnem string
/// are refused as seed backups, with the spec's wording (R0 r0 fidelity I-7).
/// MUTATION: dispatch on string length -> the 75-char entr-32 is accepted.
#[test]
fn entr32_and_mnem_strings_are_refused_as_seed_backups() {
    let entr32 = "ms10entrsqz46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kdv3c0wn2hx0lq";
    assert_eq!(entr32.len(), 75);
    let mnem = ms_codec::encode(
        ms_codec::Tag::ENTR,
        &ms_codec::Payload::Mnem {
            language: 6,
            entropy: vec![0xab; 32],
        },
    )
    .unwrap();
    for s in [entr32.to_string(), mnem] {
        let out = ms()
            .args(["hashlock", "-", "--no-engraving-card"])
            .write_stdin(s.clone())
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(1), "{s}");
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(
            err.contains("that is a seed backup, not a hashlock preimage"),
            "{err}"
        );
    }
}

/// `--hashlock-phrase -` is refused naming the stdin flag, never derived from
/// the one-byte phrase "-" (controller default, R0 r0 fidelity I-10).
#[test]
fn hashlock_phrase_dash_is_refused_naming_the_stdin_flag() {
    let out = ms()
        .args(["hashlock", "--hashlock-phrase", "-", "--no-engraving-card"])
        .write_stdin(PHRASE)
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(64),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stderr).contains("--hashlock-phrase-stdin"));
}

/// I-3: with the value omitted, the guard used to SWALLOW the next flag and
/// admit its NAME as the phrase -- and `ms hashlock` is the first verb where
/// that produced a SUCCESS, because §4.3 admits every printable-ASCII string
/// so nothing downstream could reject `--json`. The reviewer's exact
/// counterexample, which derived
/// `hash:329367945b164ccb91c6b124ab903227e34f468e9f82c5806b1ca4a194d4c613` --
/// PBKDF2 of the six bytes `--json` -- at exit 0.
///
/// MUTATION: drop the `v.starts_with('-')` guard in `argv_guard::substitute`
/// -> exit 0 with that record on stdout.
#[test]
fn an_omitted_phrase_value_does_not_swallow_the_next_flag() {
    let out = ms()
        .args([
            "hashlock",
            "--allow-argv-secret",
            "--hashlock-phrase",
            "--json",
            "--no-engraving-card",
        ])
        .write_stdin("")
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(64),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let so = String::from_utf8_lossy(&out.stdout);
    assert!(!so.contains("hash:"), "a record was emitted anyway:\n{so}");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("--hashlock-phrase") && err.contains("flag and not a value"),
        "the refusal must name the flag:\n{err}"
    );

    // The same shape on a flag that already lived in SECRET_FLAGS.
    let out = ms()
        .args([
            "encode",
            "--allow-argv-secret",
            "--hex",
            "--json",
            "--no-engraving-card",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(64));

    // The two spellings that must still work: `-` is the stdin sentinel (this
    // verb then names --hashlock-phrase-stdin, its own controller default), and
    // `--flag=<value>` is the escape hatch for a value that begins with `-`.
    let out = ms()
        .args([
            "hashlock",
            "--allow-argv-secret",
            "--hashlock-phrase",
            "-",
            "--no-engraving-card",
        ])
        .write_stdin("")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(64));
    assert!(String::from_utf8_lossy(&out.stderr).contains("--hashlock-phrase-stdin"));

    let out = ms()
        .args([
            "hashlock",
            "--allow-argv-secret",
            "--hashlock-phrase=--json",
            "--no-engraving-card",
        ])
        .write_stdin("")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "the `=` spelling is deliberate and must still admit it: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        "hash:329367945b164ccb91c6b124ab903227e34f468e9f82c5806b1ca4a194d4c613"
    );
}

/// `--out` is 0600 (owner-only) on every source.
#[cfg(unix)]
#[test]
fn out_is_owner_only() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("x.txt");
    assert!(ms()
        .args([
            "hashlock",
            "--hashlock-phrase-stdin",
            "--out",
            p.to_str().unwrap(),
            "--no-engraving-card"
        ])
        .write_stdin(PHRASE)
        .output()
        .unwrap()
        .status
        .success());
    assert_eq!(
        std::fs::metadata(&p).unwrap().permissions().mode() & 0o777,
        0o600
    );
}
