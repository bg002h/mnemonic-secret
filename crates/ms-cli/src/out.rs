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
//! adopted rather than reimplemented. Since mnemonic-io-lib 0.1.3 it writes the new
//! contents to a temporary file beside the target (0600, synced) and renames it over
//! the target, so a failed write leaves an existing target byte-identical, and the
//! mode is set on the temporary, not on a file the run is about to truncate.

use crate::error::{CliError, Result};
use std::path::Path;

/// What ms saw at `--out` before a write (R2 of FIX_bughunt1119_ms_out_writes).
#[derive(PartialEq, Eq)]
enum Before {
    /// A regular file (symlinks followed), by identity, length and mtime.
    Regular(Stamp),
    /// `NotFound`, a dangling symlink included.
    Absent,
    /// Anything else (a directory, a device, a FIFO, another lookup error).
    Other,
}

#[derive(PartialEq, Eq)]
struct Stamp {
    /// (dev, ino) on Unix; `None` elsewhere, where only (len, mtime) are compared.
    id: Option<(u64, u64)>,
    len: u64,
    mtime: Option<std::time::SystemTime>,
}

fn stamp(md: &std::fs::Metadata) -> Stamp {
    #[cfg(unix)]
    let id = {
        use std::os::unix::fs::MetadataExt;
        Some((md.dev(), md.ino()))
    };
    #[cfg(not(unix))]
    let id = None;
    Stamp {
        id,
        len: md.len(),
        mtime: md.modified().ok(),
    }
}

fn look_before(path: &Path) -> Before {
    match std::fs::metadata(path) {
        Ok(md) if md.is_file() => Before::Regular(stamp(&md)),
        Ok(_) => Before::Other,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Before::Absent,
        Err(_) => Before::Other,
    }
}

/// R2: the one failure line for an `--out` write. A state sentence is added only
/// when a second look at the file confirms it; otherwise nothing about the disk.
fn write_failure(path: &Path, err: &std::io::Error, before: &Before) -> CliError {
    let state = match (before, std::fs::metadata(path)) {
        (Before::Regular(s), Ok(md)) if md.is_file() && stamp(&md) == *s => {
            format!(". {} is unchanged.", path.display())
        }
        (Before::Absent, Err(e)) if e.kind() == std::io::ErrorKind::NotFound => {
            ". Nothing was written.".to_string()
        }
        _ => String::new(),
    };
    CliError::OutputWrite(format!(
        "cannot write the output: {}: {}{}",
        path.display(),
        err,
        state
    ))
}

/// R3: `--out` is never the file standard output is on. Called by every verb that
/// also prints on stdout, before anything is read, drawn or written.
pub(crate) fn refuse_stdout_alias(path: &Path) -> Result<()> {
    use mnemonic_io_lib::write::{same_file, Endpoint};
    if same_file(Endpoint::Path(path), &[Endpoint::Stdout]).is_some() {
        return Err(CliError::StdoutIsOut(format!(
            "--out {} and standard output are the same file (the same path, a symlink or a hard link), so what ms prints on stdout would go into the file it writes. With >, the redirect empties that file before ms starts; with >>, the file is untouched. Check that it still holds what you expect. Write to a new name.",
            path.display()
        )));
    }
    Ok(())
}

/// The exit-64 refusal for an existing `--out` path. `what` names the artifact
/// that cannot be re-made ("a --random preimage", "a new share set").
fn exists_refusal(path: &std::path::Path, what: &str) -> CliError {
    CliError::Usage(format!(
        "--out {} already exists; {what} will not overwrite it (choose another file, or move the old one first)",
        path.display()
    ))
}

/// Refuse an existing `--out` path BEFORE any input is read or randomness drawn.
/// `symlink_metadata` does not follow links, so a dangling symlink counts as
/// existing, as it does for `O_EXCL`. Any other lookup error is left for the
/// create to report. The create stays `create_new`: this check is early, not
/// atomic.
pub(crate) fn refuse_existing_out(path: &std::path::Path, what: &str) -> Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => Err(exists_refusal(path, what)),
        Err(_) => Ok(()),
    }
}

/// Like `write_artifact`, but REFUSES an existing path (exit 64, naming it)
/// instead of truncating. For artifacts that are a function of nothing and
/// cannot be re-made: `hashlock --random` (SPEC_ms_hashlock §4.1) and `split`.
///
/// The refusal names the PATH and never the artifact, for the same reason
/// `write_artifact`'s does.
pub(crate) fn write_artifact_create_new(
    path: &std::path::Path,
    body: &str,
    what: &str,
) -> Result<()> {
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
            return Err(exists_refusal(path, what));
        }
        Err(e) => return Err(write_failure(path, &e, &Before::Absent)),
    };
    if let Err(e) = f.write_all(body.as_bytes()) {
        // R4: remove the partial file this run created (the same file the handle
        // holds), so a retry is not refused as "already exists".
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if let (Ok(h), Ok(n)) = (f.metadata(), std::fs::symlink_metadata(path)) {
                if (h.dev(), h.ino()) == (n.dev(), n.ino()) {
                    let _ = std::fs::remove_file(path);
                }
            }
        }
        return Err(write_failure(path, &e, &Before::Absent));
    }
    Ok(())
}

/// Write `body` to `path`, owner-only, replacing an existing file through a
/// temporary and a rename (a failed write leaves it unchanged), naming the path on
/// failure.
///
/// The refusal names the PATH and never the artifact: a message that echoed
/// what it failed to write would put the material in a second place at exactly
/// the moment the operator is looking at the screen.
pub(crate) fn write_artifact(path: &std::path::Path, body: &str) -> Result<()> {
    let before = look_before(path);
    mnemonic_io_lib::write::write_private(path, body.as_bytes())
        .map_err(|e| write_failure(path, &e, &before))
}

#[cfg(test)]
mod second_look_tests {
    use super::*;

    fn msg(e: CliError) -> String {
        match e {
            CliError::OutputWrite(m) => m,
            _ => panic!("not an OutputWrite"),
        }
    }
    fn efbig() -> std::io::Error {
        std::io::Error::from_raw_os_error(27)
    }

    #[test]
    fn unchanged_is_claimed_for_an_untouched_file() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("F");
        std::fs::write(&p, "old\n").unwrap();
        let before = look_before(&p);
        assert!(msg(write_failure(&p, &efbig(), &before)).ends_with("F is unchanged."));
    }

    #[test]
    fn a_file_whose_length_changed_is_not_called_unchanged() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("F");
        std::fs::write(&p, "old\n").unwrap();
        let before = look_before(&p);
        std::fs::write(&p, "much longer\n").unwrap();
        assert!(!msg(write_failure(&p, &efbig(), &before)).contains("unchanged"));
    }

    #[test]
    fn a_file_whose_mtime_changed_is_not_called_unchanged() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("F");
        std::fs::write(&p, "old\n").unwrap();
        let before = look_before(&p);
        let f = std::fs::OpenOptions::new().write(true).open(&p).unwrap();
        f.set_modified(std::time::SystemTime::UNIX_EPOCH).unwrap();
        assert!(!msg(write_failure(&p, &efbig(), &before)).contains("unchanged"));
    }

    #[cfg(unix)]
    #[test]
    fn a_replaced_file_of_equal_length_and_mtime_is_not_called_unchanged() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("F");
        std::fs::write(&p, "old\n").unwrap();
        let mt = std::fs::metadata(&p).unwrap().modified().unwrap();
        let before = look_before(&p);
        let q = d.path().join("G");
        std::fs::write(&q, "new\n").unwrap();
        std::fs::OpenOptions::new()
            .write(true)
            .open(&q)
            .unwrap()
            .set_modified(mt)
            .unwrap();
        std::fs::rename(&q, &p).unwrap();
        assert!(!msg(write_failure(&p, &efbig(), &before)).contains("unchanged"));
    }

    #[test]
    fn a_file_that_appeared_is_not_called_nothing_written() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("N");
        let before = look_before(&p);
        std::fs::write(&p, "x").unwrap();
        assert!(!msg(write_failure(&p, &efbig(), &before)).contains("Nothing was written"));
    }
}
