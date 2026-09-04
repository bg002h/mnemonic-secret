//! **`--out FILE` — the first mode-aware code `ms` has ever had.**
//!
//! Measured before P2: `git grep -n 'fs::write\|OpenOptions\|set_permissions\|0o600\|0o077\|0o044\|st_mode'`
//! scoped to `crates/` returned **zero hits**, and `ms encode > backup.txt` under
//! the default umask 022 creates **0644** holding an `ms1` that decodes to the
//! seed. `--out` is the answer, and it is deliberately the ONLY place a mode is
//! decided: P2 builds no stdout mode gate at all (§6, first bullet; F-281 carries
//! whether `ms` should ever have one, as an operator ruling).
//!
//! The write itself is the shared crate's [`mnemonic_io_lib::write::write_private`],
//! adopted rather than reimplemented because the half that is easy to leave out is
//! already solved there: `OpenOptions::mode()` binds on CREATE only, so an
//! existing `0644` target stays `0644` unless the mode is set a second time on
//! the OPEN FILE — and re-running a command over an existing file is the case an
//! operator actually hits.

use crate::error::{CliError, Result};

/// Write `body` to `path`, owner-only, naming the path on failure.
///
/// The refusal names the PATH and never the artifact: a message that echoed
/// what it failed to write would put the material in a second place at exactly
/// the moment the operator is looking at the screen.
/// Like `write_artifact`, but REFUSES an existing path (exit 64, naming it)
/// instead of truncating. For `--random` only: that artifact is a function of
/// nothing and cannot be re-made (SPEC_ms_hashlock §4.1).
pub(crate) fn write_artifact_create_new(path: &std::path::Path, body: &str) -> Result<()> {
    use std::io::Write;
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    // O_CREAT|O_EXCL: the check and the create are ONE syscall, so nothing can
    // slip a file in between them and be truncated (R0 r0 fidelity I-4).
    let mut f = match opts.open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(CliError::Usage(format!(
                "--out {} already exists; a --random preimage will not overwrite it (choose another file, or move the old one first)",
                path.display()
            )));
        }
        Err(e) => {
            return Err(CliError::BadInput(format!(
                "failed to write --out {}: {}",
                path.display(),
                e
            )));
        }
    };
    f.write_all(body.as_bytes()).map_err(|e| {
        CliError::BadInput(format!("failed to write --out {}: {}", path.display(), e))
    })?;
    Ok(())
}

pub(crate) fn write_artifact(path: &std::path::Path, body: &str) -> Result<()> {
    mnemonic_io_lib::write::write_private(path, body.as_bytes())
        .map_err(|e| CliError::BadInput(format!("failed to write --out {}: {}", path.display(), e)))
}
