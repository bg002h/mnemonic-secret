//! F-687 — **the one rule for the value of `--passphrase`.** Semantics are
//! identical to mnemonic-toolkit's `passphrase_input` (the rule is shared
//! across the constellation; the vectors in `vectors/passphrase_channels.json`
//! pin it):
//!
//! | value                    | the passphrase is                             | stderr note |
//! |--------------------------|-----------------------------------------------|-------------|
//! | absent                   | the empty string (BIP-39 default)             | none        |
//! | `--passphrase-stdin`     | stdin, minus ONE trailing `\n` / `\r\n`       | none        |
//! | `-`                      | stdin, byte-identical to `--passphrase-stdin` | none        |
//! | `@env:VAR`               | `$VAR` minus ONE trailing `\n` / `\r\n` (the stdin rule); unset → error naming `VAR`; invalid name → error; set-but-empty → the empty passphrase | none |
//! | anything else            | that literal string, verbatim                 | one line    |
//!
//! Before F-687, `ms derive --passphrase -` derived with the one-character
//! passphrase `-` at exit 0, even with the real passphrase piped on stdin
//! (measured: fingerprint `66d564d1` instead of `b4e3f5ed` for "abandon ×11
//! about" + `TREZOR`), and `--passphrase @env:VAR` was refused by the argv
//! guard as material or, under `--allow-argv-secret`, taken literally.
//!
//! **A literal `-` passphrase is no longer expressible on argv**, nor one that
//! begins `@env:`. Both remain reachable through stdin or `@env:VAR`.

use zeroize::Zeroizing;

use crate::error::{CliError, Result};

/// Where the value of `--passphrase` comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PassphraseSource {
    /// Neither `--passphrase` nor `--passphrase-stdin`.
    Absent,
    /// `--passphrase-stdin`, or `--passphrase -`.
    Stdin,
    /// `--passphrase @env:VAR`.
    Env,
    /// Any other `--passphrase` value: the passphrase itself, on argv.
    Argv,
}

/// The whole-value environment sentinel. `prefix@env:VAR` is a literal.
pub(crate) const ENV_PREFIX: &str = "@env:";

/// Is this `--passphrase` VALUE (exactly as clap sees it) a private channel —
/// `-` (stdin) or `@env:VAR` — rather than the passphrase itself? The argv
/// guard and [`source`] both call THIS, so they cannot disagree about a
/// padded `" -"` (F-687 fold 1, review M2). Exact match: `" -"` is a literal.
pub(crate) fn is_channel_value(raw: &str) -> bool {
    raw == "-" || raw.starts_with(ENV_PREFIX)
}

/// Is this input PATH really stdin? `/dev/stdin`, `/dev/fd/0` and
/// `/proc/self/fd/0` by name, and on Unix anything that is the same file as
/// fd 0 (device + inode). Such a path reads the SAME stream as a stdin
/// passphrase (review M1: `ms derive --in /dev/stdin --passphrase -` drained
/// stdin into the ms1 read and derived with the EMPTY passphrase at exit 0).
/// Byte-identical rule to mnemonic-toolkit's `passphrase_input::path_is_stdin`.
pub(crate) fn path_is_stdin(path: &std::path::Path) -> bool {
    if matches!(
        path.to_str(),
        Some("/dev/stdin" | "/dev/fd/0" | "/proc/self/fd/0")
    ) {
        return true;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if let (Ok(a), Ok(b)) = (std::fs::metadata(path), std::fs::metadata("/dev/stdin")) {
            return a.dev() == b.dev() && a.ino() == b.ino();
        }
    }
    false
}

/// Classify. Pure.
///
/// `admitted` is what `--allow-argv-secret` took off argv for `--passphrase`:
/// the argv guard substitutes the VALUE with `-` and keeps the material on a
/// side channel, so with `admitted` present the `-` seen here is a
/// placeholder — the source is argv, not stdin.
pub(crate) fn source(
    value: Option<&str>,
    stdin_flag: bool,
    admitted: Option<&str>,
) -> PassphraseSource {
    if stdin_flag {
        return PassphraseSource::Stdin;
    }
    if admitted.is_some() {
        return PassphraseSource::Argv;
    }
    match value {
        None => PassphraseSource::Absent,
        Some(v) if !is_channel_value(v) => PassphraseSource::Argv,
        Some("-") => PassphraseSource::Stdin,
        Some(_) => PassphraseSource::Env,
    }
}

/// Does the passphrase consume stdin? The single-stdin guard asks THIS.
pub(crate) fn reads_stdin(value: Option<&str>, stdin_flag: bool, admitted: Option<&str>) -> bool {
    source(value, stdin_flag, admitted) == PassphraseSource::Stdin
}

/// The note printed once for a literal argv passphrase. Never carries the
/// value. Byte-identical to mnemonic-toolkit's `passphrase_input::ARGV_NOTE`.
pub(crate) const ARGV_NOTE: &str = "warning: secret material on argv (--passphrase) \u{2014} \
read it privately with --passphrase - or --passphrase-stdin (stdin), \
or --passphrase @env:VAR (environment variable)";

/// Emit [`ARGV_NOTE`] iff the passphrase is literal on argv.
pub(crate) fn emit_argv_note<W: std::io::Write>(
    value: Option<&str>,
    stdin_flag: bool,
    admitted: Option<&str>,
    stderr: &mut W,
) {
    if source(value, stdin_flag, admitted) == PassphraseSource::Argv {
        let _ = writeln!(stderr, "{ARGV_NOTE}");
    }
}

/// POSIX env-var name as the toolkit accepts it: `[A-Z_][A-Z0-9_]*`.
fn is_valid_env_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_uppercase() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

/// Resolve `--passphrase @env:VAR`. Unset is an ERROR naming `VAR`: treating
/// it as the empty passphrase would silently derive a different wallet.
fn resolve_env(value: &str) -> Result<Zeroizing<String>> {
    let var = value.strip_prefix(ENV_PREFIX).unwrap_or(value);
    if !is_valid_env_name(var) {
        return Err(CliError::BadInput(format!(
            "--passphrase: invalid env-var name `{var}`"
        )));
    }
    let mut v = std::env::var(var).map(Zeroizing::new).map_err(|e| {
        CliError::BadInput(match e {
            // F-687 fold 1 (review N1): set-but-not-UTF-8 is not "not set".
            std::env::VarError::NotUnicode(_) => format!(
                "--passphrase: env-var {var} referenced by sentinel is set but not valid UTF-8"
            ),
            std::env::VarError::NotPresent => {
                format!("--passphrase: env-var {var} referenced by sentinel is not set")
            }
        })
    })?;
    strip_one_newline(&mut v);
    Ok(v)
}

/// Remove exactly ONE trailing `\n` (and a `\r` before it), nothing else —
/// the byte rule `read_stdin_passphrase` has always applied, now shared by
/// `@env:VAR` so the three private forms agree on the bytes.
pub(crate) fn strip_one_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

/// Resolve to the passphrase; absent = the empty passphrase. `admitted` as in
/// [`source`].
pub(crate) fn resolve_or_empty(
    value: Option<&str>,
    stdin_flag: bool,
    admitted: Option<&str>,
) -> Result<Zeroizing<String>> {
    if stdin_flag && value.is_some() {
        return Err(CliError::Usage(
            "--passphrase and --passphrase-stdin cannot both be given (one passphrase, one stdin)"
                .into(),
        ));
    }
    match source(value, stdin_flag, admitted) {
        PassphraseSource::Absent => Ok(Zeroizing::new(String::new())),
        PassphraseSource::Stdin => crate::parse::read_stdin_passphrase(),
        PassphraseSource::Env => resolve_env(value.unwrap_or("")),
        PassphraseSource::Argv => Ok(Zeroizing::new(admitted.or(value).unwrap_or("").to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classification() {
        assert_eq!(source(None, false, None), PassphraseSource::Absent);
        assert_eq!(source(None, true, None), PassphraseSource::Stdin);
        assert_eq!(source(Some("-"), false, None), PassphraseSource::Stdin);
        assert_eq!(source(Some("@env:PP"), false, None), PassphraseSource::Env);
        assert_eq!(source(Some("TREZOR"), false, None), PassphraseSource::Argv);
        assert_eq!(source(Some(""), false, None), PassphraseSource::Argv);
        assert_eq!(
            source(Some("x@env:PP"), false, None),
            PassphraseSource::Argv
        );
        assert_eq!(source(Some(" -"), false, None), PassphraseSource::Argv);
        // The override's placeholder `-` is NOT stdin.
        assert_eq!(
            source(Some("-"), false, Some("TREZOR")),
            PassphraseSource::Argv
        );
        assert!(!reads_stdin(Some("-"), false, Some("TREZOR")));
    }

    #[test]
    fn env_rules() {
        std::env::set_var("MS_F687_UNIT_SET", "TREZOR");
        assert_eq!(
            resolve_env("@env:MS_F687_UNIT_SET").unwrap().as_str(),
            "TREZOR"
        );
        std::env::set_var("MS_F687_UNIT_EMPTY", "");
        assert_eq!(resolve_env("@env:MS_F687_UNIT_EMPTY").unwrap().as_str(), "");
        let e = resolve_env("@env:MS_F687_UNIT_NEVER_SET").unwrap_err();
        assert!(format!("{e}").contains("MS_F687_UNIT_NEVER_SET"), "{e}");
        for (val, want) in [
            ("TREZOR\n", "TREZOR"),
            ("TREZOR\r\n", "TREZOR"),
            ("TREZOR\n\n", "TREZOR\n"),
            ("TRE\nZOR", "TRE\nZOR"),
            (" TREZOR ", " TREZOR "),
        ] {
            std::env::set_var("MS_F687_UNIT_NL", val);
            assert_eq!(
                resolve_env("@env:MS_F687_UNIT_NL").unwrap().as_str(),
                want,
                "{val:?}"
            );
        }
        assert!(resolve_env("@env:lower").is_err());
        assert!(resolve_env("@env:").is_err());
    }

    #[test]
    fn only_a_literal_gets_the_note() {
        for (v, flag, noted) in [
            (Some("TREZOR"), false, true),
            (Some(""), false, true),
            (Some("-"), false, false),
            (Some("@env:PP"), false, false),
            (None, true, false),
            (None, false, false),
        ] {
            let mut e = Vec::new();
            emit_argv_note(v, flag, None, &mut e);
            assert_eq!(
                e.iter().filter(|b| **b == b'\n').count(),
                usize::from(noted)
            );
        }
        let mut e = Vec::new();
        emit_argv_note(Some("-"), false, Some("TREZOR"), &mut e);
        assert_eq!(
            e.iter().filter(|b| **b == b'\n').count(),
            1,
            "admitted literal is noted"
        );
        assert!(!String::from_utf8(e).unwrap().contains("TREZOR"));
    }

    #[test]
    fn admitted_literal_wins_over_the_placeholder_dash() {
        assert_eq!(
            resolve_or_empty(Some("-"), false, Some("TREZOR"))
                .unwrap()
                .as_str(),
            "TREZOR"
        );
        assert!(resolve_or_empty(Some("-"), true, None).is_err());
    }
}
