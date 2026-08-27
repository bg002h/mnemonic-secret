//! **Routing test material onto a PRIVATE channel — never onto the override.**
//!
//! P2 installs a pre-parser argv guard, so every invocation that used to hand
//! `ms` a seed, an `ms1` string or hex entropy on argv is now refused. 147 of
//! `ms`'s 276 integration tests live in the 31 files that pass `--phrase` or
//! `--hex`, and the cheap way to green all of them is to append
//! `--allow-argv-secret` to every invocation.
//!
//! **That is forbidden.** A suite that reaches the code only through the
//! override stops exercising what an operator experiences, and leaves the
//! refusal itself proven by a handful of cases. So this module rewrites an
//! invocation onto the channel an operator would actually use — `--in FILE`,
//! `-`, or `--passphrase-stdin` — and the override appears only in the tests
//! whose subject IS the override.
//!
//! **It fails loudly rather than guessing.** An invocation needing two stdin
//! consumers panics naming both, because the alternative is a helper that
//! silently picks one and makes a test pass for a reason nobody chose.

#![allow(dead_code)]

use std::process::Output;

/// The four secret-bearing flags. `--passphrase-stdin` is not one of them: the
/// match is equality, not a prefix test.
const SECRET_FLAGS: [&str; 4] = ["--phrase", "--hex", "--ms1", "--passphrase"];

const BECH32: &str = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";

fn is_ms1_shaped(s: &str) -> bool {
    // Separators stripped, mirroring the guard: a GROUPED card is the same
    // material as an unbroken one, because `ms` strips them on intake.
    let t: String = s
        .to_ascii_lowercase()
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-' && *c != ',')
        .collect();
    t.len() >= 48 && t.starts_with("ms1") && t[3..].chars().all(|c| BECH32.contains(c))
}

fn is_phrase_shaped(s: &str) -> bool {
    let w: Vec<&str> = s.split_whitespace().collect();
    matches!(w.len(), 12 | 15 | 18 | 21 | 24)
        && bip39::Language::ALL
            .iter()
            .any(|l| w.iter().all(|x| l.find_word(&x.to_lowercase()).is_some()))
}

fn is_hex_shaped(s: &str) -> bool {
    let t = s.trim();
    matches!(t.len(), 32 | 40 | 48 | 56 | 64) && t.chars().all(|c| c.is_ascii_hexdigit())
}

fn is_material(s: &str) -> bool {
    is_ms1_shaped(s) || is_phrase_shaped(s) || is_hex_shaped(s)
}

/// One rewritten invocation, holding the tempdir its `--in` files live in.
pub struct Private {
    pub argv: Vec<String>,
    pub stdin: Option<String>,
    _dir: tempfile::TempDir,
}

/// Rewrite `args` so no secret material rides on argv.
///
/// The per-channel routing is exactly what §6d's table says each verb has:
///
/// | shape | becomes |
/// | --- | --- |
/// | `encode`/`split` `--phrase V` | `--in FILE` — `--in` MEANS a phrase on those two verbs |
/// | any `--hex V` | `--hex -`, V on stdin — `--in` never reads hex, by ruling |
/// | `verify`/`derive` `--phrase V` | `--phrase -`, V on stdin — `--in` binds to the ms1 there |
/// | `repair --ms1 V` | `--in FILE` |
/// | `derive --passphrase V` | `--passphrase-stdin`, V on stdin |
/// | a material positional | `--in FILE` |
/// | `combine`'s material positionals | one `--in FILE`, one share per line |
pub fn rewrite(args: &[&str]) -> Private {
    let dir = tempfile::tempdir().expect("tempdir");
    let verb = args.first().copied().unwrap_or("");
    let mut argv: Vec<String> = Vec::with_capacity(args.len() + 2);
    let mut stdin: Option<String> = None;
    let mut stdin_owner: Option<String> = None;
    let mut n_files = 0usize;

    let claim_stdin =
        |who: &str, value: &str, cur: &mut Option<String>, owner: &mut Option<String>| {
            if let Some(prev) = owner {
                panic!(
                    "two channels want stdin in one invocation: `{prev}` and `{who}`. \
                 There is one stdin per invocation -- route one of them through \
                 `--in FILE`, or split the invocation in two. This helper refuses \
                 to pick for you."
                );
            }
            *cur = Some(value.to_string());
            *owner = Some(who.to_string());
        };

    let write_file = |body: &str, n: &mut usize| -> String {
        *n += 1;
        let p = dir.path().join(format!("material-{n}.txt"));
        std::fs::write(&p, body).expect("write material");
        p.display().to_string()
    };

    let mut i = 0;
    while i < args.len() {
        let a = args[i];
        if SECRET_FLAGS.contains(&a) && i + 1 < args.len() && args[i + 1] != "-" {
            let v = args[i + 1];
            match (a, verb) {
                ("--phrase", "encode") | ("--phrase", "split") => {
                    argv.push("--in".into());
                    argv.push(write_file(v, &mut n_files));
                }
                ("--ms1", _) => {
                    argv.push("--in".into());
                    argv.push(write_file(v, &mut n_files));
                }
                ("--passphrase", _) => {
                    claim_stdin("--passphrase", v, &mut stdin, &mut stdin_owner);
                    argv.push("--passphrase-stdin".into());
                }
                _ => {
                    claim_stdin(a, v, &mut stdin, &mut stdin_owner);
                    argv.push(a.into());
                    argv.push("-".into());
                }
            }
            i += 2;
            continue;
        }
        if i > 0 && is_material(a) && !a.starts_with("--") {
            // A material positional. `combine` may carry several; they become
            // one `--in` file, one share per line, which is what `--in` on
            // `combine` reads.
            let mut vals = vec![a.to_string()];
            let mut j = i + 1;
            if verb == "combine" {
                while j < args.len() && is_material(args[j]) && !args[j].starts_with("--") {
                    vals.push(args[j].to_string());
                    j += 1;
                }
            }
            argv.push("--in".into());
            argv.push(write_file(&vals.join("\n"), &mut n_files));
            i = j;
            continue;
        }
        argv.push(a.into());
        i += 1;
    }
    Private {
        argv,
        stdin,
        _dir: dir,
    }
}

/// Run `ms` with `args`, material routed privately. Drop-in for a test's own
/// `fn ms(args: &[&str]) -> Output`.
pub fn run(args: &[&str]) -> Output {
    let p = rewrite(args);
    run_with_stdin(&p, None)
}

/// As [`run`], but the caller supplies stdin. Panics if the rewrite already
/// needed it, for the same reason `rewrite` does.
pub fn run_stdin(args: &[&str], stdin: &str) -> Output {
    let p = rewrite(args);
    run_with_stdin(&p, Some(stdin))
}

fn run_with_stdin(p: &Private, extra: Option<&str>) -> Output {
    let mut cmd = assert_cmd::Command::cargo_bin("ms").unwrap();
    cmd.args(&p.argv);
    match (&p.stdin, extra) {
        (Some(_), Some(_)) => panic!(
            "the rewrite already claimed stdin for a secret channel, and the caller \
             supplied stdin too -- there is one stdin per invocation"
        ),
        (Some(s), None) => {
            cmd.write_stdin(s.clone());
        }
        (None, Some(s)) => {
            cmd.write_stdin(s.to_string());
        }
        (None, None) => {}
    }
    let out = cmd.output().unwrap();
    Output {
        status: out.status,
        stdout: out.stdout,
        stderr: out.stderr,
    }
}
