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

/// F-687b ruling 2: the one line printed when a PRIVATE channel yields an
/// EMPTY passphrase. `origin` is `stdin` or `environment variable VAR`.
/// Byte-identical to mnemonic-toolkit's `passphrase_input::empty_warning`.
pub(crate) fn empty_warning(origin: &str) -> String {
    format!("warning: --passphrase from {origin} is empty; proceeding with the EMPTY passphrase")
}

fn warn_if_empty(origin: &str, value: &str) {
    if value.is_empty() {
        eprintln!("{}", empty_warning(origin));
    }
}

/// The terminal prompt (ruling 3). Same text as mnemonic-toolkit.
pub(crate) const PROMPT: &str = "Enter passphrase: ";

/// Is the PROCESS stdin a terminal? Never in unit tests.
pub(crate) fn stdin_is_terminal() -> bool {
    use std::io::IsTerminal;
    !cfg!(test) && std::io::stdin().is_terminal()
}

/// Echo off on fd 0 for the life of the guard (Unix). `ECHONL` keeps the
/// Enter visible, so the cursor moves on without showing the secret. The
/// saved mode is restored on drop (including on an error return) AND, while
/// the guard is live, by a handler for SIGINT/SIGTERM/SIGHUP/SIGQUIT that
/// restores the mode, resets the signal to its default and re-raises it, so
/// Ctrl-C at the prompt exits with the conventional signal status and a
/// working terminal. A signal the process inherited as IGNORED stays ignored.
struct EchoOff {
    #[cfg(unix)]
    saved: Option<libc::termios>,
    #[cfg(unix)]
    old_actions: Option<[libc::sigaction; 4]>,
}

#[cfg(unix)]
mod echo_signal {
    use std::sync::atomic::{AtomicBool, Ordering};

    /// The mode to restore from the handler. Written BEFORE `ACTIVE` is set
    /// and read only while it is set.
    static mut SAVED: std::mem::MaybeUninit<libc::termios> = std::mem::MaybeUninit::uninit();
    static ACTIVE: AtomicBool = AtomicBool::new(false);
    pub(super) const SIGNALS: [libc::c_int; 4] =
        [libc::SIGINT, libc::SIGTERM, libc::SIGHUP, libc::SIGQUIT];

    /// Async-signal-safe: tcsetattr, signal and raise only.
    extern "C" fn restore_and_reraise(sig: libc::c_int) {
        if ACTIVE.swap(false, Ordering::SeqCst) {
            // SAFETY: SAVED was fully written before ACTIVE was set.
            unsafe {
                libc::tcsetattr(0, libc::TCSANOW, (*std::ptr::addr_of!(SAVED)).as_ptr());
            }
        }
        // SAFETY: default disposition, then deliver the same signal again.
        unsafe {
            libc::signal(sig, libc::SIG_DFL);
            libc::raise(sig);
        }
    }

    /// Install the handlers; returns the previous actions.
    ///
    /// # Safety
    /// Single-threaded use around one prompt at a time.
    pub(super) unsafe fn arm(saved: &libc::termios) -> [libc::sigaction; 4] {
        (*std::ptr::addr_of_mut!(SAVED)).write(*saved);
        ACTIVE.store(true, Ordering::SeqCst);
        let mut old: [libc::sigaction; 4] = std::mem::zeroed();
        for (i, s) in SIGNALS.iter().enumerate() {
            libc::sigaction(*s, std::ptr::null(), &mut old[i]);
            if old[i].sa_sigaction == libc::SIG_IGN {
                continue;
            }
            let mut sa: libc::sigaction = std::mem::zeroed();
            sa.sa_sigaction = restore_and_reraise as extern "C" fn(libc::c_int) as usize;
            libc::sigemptyset(&mut sa.sa_mask);
            libc::sigaction(*s, &sa, std::ptr::null_mut());
        }
        old
    }

    /// Put the previous actions back.
    ///
    /// # Safety
    /// `old` is what [`arm`] returned.
    pub(super) unsafe fn disarm(old: &[libc::sigaction; 4]) {
        ACTIVE.store(false, Ordering::SeqCst);
        for (i, s) in SIGNALS.iter().enumerate() {
            libc::sigaction(*s, &old[i], std::ptr::null_mut());
        }
    }
}

impl EchoOff {
    /// `(guard, echo_disabled)`.
    fn new() -> (Self, bool) {
        #[cfg(unix)]
        {
            // SAFETY: tcgetattr/tcsetattr on fd 0 with a zeroed, then
            // kernel-filled, termios; the handlers are armed before echo goes
            // off, so no window exists with echo off and no handler.
            unsafe {
                let mut t: libc::termios = std::mem::zeroed();
                if libc::tcgetattr(0, &mut t) == 0 {
                    let saved = t;
                    let old = echo_signal::arm(&saved);
                    t.c_lflag &= !libc::ECHO;
                    t.c_lflag |= libc::ECHONL;
                    if libc::tcsetattr(0, libc::TCSANOW, &t) == 0 {
                        return (
                            Self {
                                saved: Some(saved),
                                old_actions: Some(old),
                            },
                            true,
                        );
                    }
                    echo_signal::disarm(&old);
                }
            }
            (
                Self {
                    saved: None,
                    old_actions: None,
                },
                false,
            )
        }
        #[cfg(not(unix))]
        {
            (Self {}, false)
        }
    }
}

impl Drop for EchoOff {
    fn drop(&mut self) {
        #[cfg(unix)]
        {
            if let Some(t) = self.saved.take() {
                // SAFETY: restores the attributes read in `new`. Restore
                // FIRST, then disarm: a signal in between restores again.
                unsafe {
                    libc::tcsetattr(0, libc::TCSANOW, &t);
                }
            }
            if let Some(old) = self.old_actions.take() {
                // SAFETY: `old` came from `arm`.
                unsafe { echo_signal::disarm(&old) };
            }
        }
    }
}

/// F-687c (operator ruling 2026-09-25: "Yes to the paste question but print
/// what was dropped on stderr"). On the TERMINAL path only, after the
/// prompted line is read and while the prompt's mode (echo off, signal
/// handlers armed) is still in force: read whatever input is already
/// pending, so a multi-line paste or type-ahead is not left for the shell to
/// run (and record in its history). What is read is printed on stderr under a
/// clear label, then the input queue is flushed. Nothing pending → no output.
///
/// The queue is read in non-canonical mode (`VMIN = 0`, `VTIME = 1`: each
/// read waits at most 0.1 s), so a partial last line and input still arriving
/// from a paste are both caught, and the read ends 0.1 s after input stops.
/// `ISIG` stays on, so Ctrl-C during the drain is still handled by the armed
/// handler, which restores the ORIGINAL mode. The prompt's mode is restored
/// before returning; `EchoOff`'s drop then restores the original.
#[cfg(unix)]
fn drain_pending_input(noun: &str) {
    use std::io::Write as _;
    // SAFETY: tcgetattr/tcsetattr/read/tcflush on fd 0 with buffers we own.
    unsafe {
        let mut prompt_mode: libc::termios = std::mem::zeroed();
        if libc::tcgetattr(0, &mut prompt_mode) != 0 {
            return;
        }
        let mut raw = prompt_mode;
        raw.c_lflag &= !libc::ICANON;
        raw.c_cc[libc::VMIN] = 0;
        raw.c_cc[libc::VTIME] = 1;
        if libc::tcsetattr(0, libc::TCSANOW, &raw) != 0 {
            return;
        }
        let mut got: Zeroizing<Vec<u8>> = Zeroizing::new(Vec::new());
        let mut chunk = [0u8; 256];
        loop {
            let n = libc::read(0, chunk.as_mut_ptr().cast(), chunk.len());
            if n > 0 {
                got.extend_from_slice(&chunk[..n as usize]);
                if got.len() < 1 << 20 {
                    continue;
                }
            } else if n < 0
                && std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted
            {
                continue;
            }
            break;
        }
        // Belt and braces: nothing is left for the shell.
        libc::tcflush(0, libc::TCIFLUSH);
        chunk.iter_mut().for_each(|b| *b = 0);
        libc::tcsetattr(0, libc::TCSANOW, &prompt_mode);
        if got.is_empty() {
            return;
        }
        // F-687d: a MASKED preview per line, never the full text.
        let _ = write!(std::io::stderr(), "{}", drain_note(noun, &got));
    }
}

/// F-687d (operator ruling 2026-09-25: "Agree with masked echo as you
/// suggested"): the note for input discarded after a terminal prompt. Each
/// discarded line is shown MASKED, one per line under the header:
///
/// - only whitespace → `(blank)`;
/// - at most 8 characters → the line itself;
/// - longer → its first 8 characters, `…`, and `(W word(s), N chars)`.
///
/// Characters, not bytes, are counted. In what is shown, a control
/// character, a whole escape sequence (CSI `ESC [ … final`, OSC `ESC ] …
/// BEL/ST`, or `ESC x`) and the invisible reordering/formatting marks
/// (U+200B–U+200F, U+202A–U+202E, U+2066–U+2069, U+FEFF) each become `?`,
/// so a paste cannot drive the terminal. Byte-identical in mnemonic-toolkit
/// and mnemonic-secret; pinned by `drain_preview.json` in both repos.
pub(crate) fn drain_note(noun: &str, got: &[u8]) -> String {
    let text = String::from_utf8_lossy(got);
    let body = text.strip_suffix('\n').unwrap_or(&text);
    let lines: Vec<&str> = body.split('\n').collect();
    let mut out = format!(
        "note: discarded {} line(s) typed after the {noun} (not run, not used):\n",
        lines.len()
    );
    for line in lines {
        out.push_str("  ");
        out.push_str(&masked_preview(line));
        out.push('\n');
    }
    out
}

/// The masked preview of ONE discarded line (see [`drain_note`]).
fn masked_preview(line: &str) -> String {
    if line.trim().is_empty() {
        return "(blank)".to_string();
    }
    let n = line.chars().count();
    if n <= 8 {
        return neutralise(line);
    }
    let head: String = line.chars().take(8).collect();
    let words = line.split_whitespace().count();
    format!(
        "{}\u{2026} ({words} word{}, {n} chars)",
        neutralise(&head),
        if words == 1 { "" } else { "s" }
    )
}

/// Replace control characters, escape sequences and invisible
/// formatting marks with `?` (one `?` per sequence).
fn neutralise(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\u{1b}' {
            match it.peek() {
                // CSI: parameters/intermediates, then a final byte @..~.
                Some('[') => {
                    it.next();
                    for d in it.by_ref() {
                        if ('\u{40}'..='\u{7e}').contains(&d) {
                            break;
                        }
                    }
                }
                // OSC: up to BEL or ST (ESC \).
                Some(']') => {
                    it.next();
                    while let Some(d) = it.next() {
                        if d == '\u{7}' {
                            break;
                        }
                        if d == '\u{1b}' && it.peek() == Some(&'\\') {
                            it.next();
                            break;
                        }
                    }
                }
                Some(_) => {
                    it.next();
                }
                None => {}
            }
            out.push('?');
        } else if c.is_control()
            || matches!(c, '\u{200b}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}' | '\u{feff}')
        {
            out.push('?');
        } else {
            out.push(c);
        }
    }
    out
}

/// Read the passphrase from stdin under the one byte rule. On a terminal:
/// prompt on stderr, echo off where possible, read ONE line. Otherwise read
/// to EOF, no prompt. Warns once if the result is empty.
pub(crate) fn read_stdin_passphrase() -> Result<Zeroizing<String>> {
    use std::io::{Read, Write};
    let mut s: Zeroizing<String> = if stdin_is_terminal() {
        let (_echo, hidden) = EchoOff::new();
        let mut e = std::io::stderr();
        let _ = write!(
            e,
            "{PROMPT}{}",
            if hidden {
                ""
            } else {
                "(input will be visible) "
            }
        );
        let _ = e.flush();
        let mut bytes: Zeroizing<Vec<u8>> = Zeroizing::new(Vec::new());
        let mut b = [0u8; 1];
        let mut stdin = std::io::stdin().lock();
        loop {
            match stdin.read(&mut b) {
                Ok(0) => break,
                Ok(_) => {
                    bytes.push(b[0]);
                    if b[0] == b'\n' {
                        break;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(CliError::BadInput(format!("failed to read stdin: {e}"))),
            }
        }
        if bytes.last() != Some(&b'\n') {
            // Ctrl-D at the prompt: move off the prompt line (review N1).
            let _ = writeln!(e);
        }
        drop(stdin);
        #[cfg(unix)]
        drain_pending_input("passphrase");
        Zeroizing::new(
            String::from_utf8(bytes.to_vec())
                .map_err(|_| CliError::BadInput("--passphrase: stdin is not valid UTF-8".into()))?,
        )
    } else {
        crate::parse::read_stdin()?
    };
    strip_one_newline(&mut s);
    warn_if_empty("stdin", &s);
    Ok(s)
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
    warn_if_empty(&format!("environment variable {var}"), &v);
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
        PassphraseSource::Stdin => read_stdin_passphrase(),
        PassphraseSource::Env => resolve_env(value.unwrap_or("")),
        PassphraseSource::Argv => Ok(Zeroizing::new(admitted.or(value).unwrap_or("").to_string())),
    }
}

#[cfg(test)]
mod tests {

    /// F-687d: the masked drain note, against the golden file that
    /// mnemonic-toolkit and mnemonic-secret carry byte-identically.
    #[test]
    fn drain_note_matches_the_shared_golden() {
        let v: serde_json::Value =
            serde_json::from_str(include_str!("../vectors/drain_preview.json")).unwrap();
        let cases = v["cases"].as_array().unwrap();
        assert!(cases.len() >= 15, "golden lost cases");
        for c in cases {
            let got = super::drain_note(
                c["noun"].as_str().unwrap(),
                c["input"].as_str().unwrap().as_bytes(),
            );
            assert_eq!(got, c["note"].as_str().unwrap(), "{}", c["why"]);
        }
    }

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
