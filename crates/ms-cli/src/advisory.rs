//! Inline-secret argv-leak advisory (ported from mnemonic-toolkit's
//! `secret_advisory`). ms-cli `process_hardening` blocks cross-UID
//! `/proc/$PID/cmdline` reads, but same-UID exposure remains for inline secrets;
//! this warns and points at the stdin/`-` alternative.
//!
//! Also provides the output-class advisory helper, a byte-for-byte duplicate of
//! mnemonic-toolkit's `secret_advisory::emit_output_class_advisory`. Cross-repo
//! byte parity is enforced by `tests/cli_output_class.rs::byte_parity_advisory_lines`.

use std::io::Write;

/// Emit a stderr advisory when a secret arrives inline on argv.
pub fn secret_in_argv_warning<W: Write>(stderr: &mut W, flag: &str, alternative: &str) {
    let _ = writeln!(
        stderr,
        "warning: secret material on argv ({flag}) — pipe via {alternative} to avoid /proc/$PID/cmdline exposure"
    );
}

/// Security class of what a command wrote to stdout.
///
/// Byte-identical to mnemonic-toolkit's `secret_advisory::OutputClass`.
/// `Template` is present for byte-parity completeness (ms-cli has no
/// template-producing command, but the enum must mirror the toolkit's variant
/// set so the advisory text is also present — enforced by the byte-parity test).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputClass {
    PrivateKeyMaterial,
    WatchOnly,
    Template,
}

/// Emit the one-line stderr class advisory. Byte-identical to mnemonic-toolkit's
/// `secret_advisory::emit_output_class_advisory` (cross-repo parity, see the
/// byte-parity test). Inert outputs do NOT call this.
pub fn emit_output_class_advisory<W: std::io::Write>(class: OutputClass, stderr: &mut W) {
    let line = match class {
        OutputClass::PrivateKeyMaterial =>
            "warning: stdout carries private key material (can spend) \u{2014} redirect or encrypt (e.g. '> file.txt' or '| age -e ...')",
        OutputClass::WatchOnly => "note: stdout is watch-only \u{2014} public keys only, cannot spend",
        OutputClass::Template => "note: stdout is a keyless descriptor template (no keys)",
    };
    let _ = writeln!(stderr, "{line}");
}
