//! Inline-secret argv-leak advisory (ported from mnemonic-toolkit's
//! `secret_advisory`). ms-cli `process_hardening` blocks cross-UID
//! `/proc/$PID/cmdline` reads, but same-UID exposure remains for inline secrets;
//! this warns and points at the stdin/`-` alternative.

use std::io::Write;

/// Emit a stderr advisory when a secret arrives inline on argv.
pub fn secret_in_argv_warning<W: Write>(stderr: &mut W, flag: &str, alternative: &str) {
    let _ = writeln!(
        stderr,
        "warning: secret material on argv ({flag}) — pipe via {alternative} to avoid /proc/$PID/cmdline exposure"
    );
}
