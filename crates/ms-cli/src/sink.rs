//! The one stdout path. Every `print!`/`println!` in ms-cli resolves to the
//! macros below (declared `#[macro_use]` first in `main.rs`, so they shadow
//! std's in every module), and so do `eprint!`/`eprintln!`. The few verbs that
//! write with `writeln!(stdout, ..)` take [`Out`] instead of
//! `std::io::stdout()`.
//!
//! std's macros panic (exit 101) when a write fails, and `.ok()` on a
//! `writeln!` hides the failure (exit 0 with the output lost). Here a write
//! error stops the output instead. A reader that closed the pipe
//! (`ms gui-schema | head -1`, `ErrorKind::BrokenPipe`) only stops it: the
//! exit code is the verb's own. Any other stdout error (a full disk,
//! `> /dev/full`) is kept, and `main` prints "ms: cannot write the output:
//! <error>" and exits 1 (crash sweep F1, grade-mk-crash-sweep-1 Q4). A stderr
//! write error is ignored. (Named `sink`, not `out`: `out.rs` is `--out FILE`.)
//! `tests/output_errors.rs` pins both cases and that no other file of the
//! crate names stdout.

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

static CLOSED: AtomicBool = AtomicBool::new(false);
static WRITE_ERROR: OnceLock<String> = OnceLock::new();

/// Writes to stdout; after the first failure, writes nothing.
pub(crate) fn stdout_write(args: std::fmt::Arguments<'_>) {
    let _ = Out.write_fmt(args);
}

/// The stdout handle for `write!`/`writeln!`: never returns an error, records it.
pub(crate) struct Out;

impl Write for Out {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if !CLOSED.load(Ordering::Relaxed) && !read_only() {
            let mut o = std::io::stdout().lock();
            if let Err(e) = o.write_all(buf).and_then(|()| o.flush()) {
                failed(&e);
            }
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(unix)]
static STDOUT_WAS_CLOSED: AtomicBool = AtomicBool::new(false);

/// A closed fd 0, 1 or 2 at exec (`ms ... >&-`). std's runtime opens
/// `/dev/null` read-write on each closed standard fd before `main`, so by
/// `main` a closed stdout looks like a writable one and its output is lost
/// with exit 0. This runs earlier, from the executable's initializer list:
/// it records a closed fd 1 and plugs each closed fd with `/dev/null` itself
/// (the lowest free fd, so it lands on that number and stays open for the
/// whole run), so no file `ms` opens later (an `--out` file) can become
/// stdout. A target without such a list keeps std's plug only.
#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "dragonfly",
    target_os = "macos"
))]
mod closed_fds {
    use std::sync::atomic::Ordering;

    extern "C" fn probe() {
        for fd in 0..=2 {
            // SAFETY: fcntl(F_GETFD) and open() of a NUL-terminated literal
            // take no other pointer; a bad fd or failed open returns -1.
            unsafe {
                if libc::fcntl(fd, libc::F_GETFD) != -1
                    || std::io::Error::last_os_error().raw_os_error() != Some(libc::EBADF)
                {
                    continue;
                }
                if fd == 1 {
                    super::STDOUT_WAS_CLOSED.store(true, Ordering::Relaxed);
                }
                let n = libc::open(c"/dev/null".as_ptr(), libc::O_RDWR);
                if n >= 0 && n != fd {
                    libc::close(n);
                }
            }
        }
    }

    #[used]
    #[cfg_attr(not(target_os = "macos"), link_section = ".init_array")]
    #[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
    static PROBE: extern "C" fn() = probe;
}

/// std's stdout reports EBADF as a successful write, so a stdout that cannot
/// write (closed at exec, see `closed_fds`; opened read-only, `1</dev/null`,
/// as O_PATH, or with Linux's access mode 3) would lose the output with
/// exit 0. Checked once,
/// at the first write (or clap's `--help`/`--version`): then EBADF is recorded
/// as the write error. A verb that writes nothing to stdout is not touched.
pub(crate) fn read_only() -> bool {
    static RO: OnceLock<bool> = OnceLock::new();
    *RO.get_or_init(|| {
        #[cfg(unix)]
        {
            // SAFETY: fcntl(F_GETFL) takes no pointer; a bad fd returns -1.
            let fl = unsafe { libc::fcntl(1, libc::F_GETFL) };
            let ro = STDOUT_WAS_CLOSED.load(Ordering::Relaxed)
                || (fl >= 0 && !matches!(fl & libc::O_ACCMODE, libc::O_WRONLY | libc::O_RDWR));
            if ro {
                failed(&std::io::Error::from_raw_os_error(libc::EBADF));
            }
            ro
        }
        // Windows: std turns only an invalid handle into a success, so a
        // read-only handle's error reaches the sink as written.
        #[cfg(not(unix))]
        {
            false
        }
    })
}

/// Records a failed stdout write (also used for clap's `--help`/`--version`).
pub(crate) fn failed(e: &std::io::Error) {
    CLOSED.store(true, Ordering::Relaxed);
    if e.kind() != std::io::ErrorKind::BrokenPipe {
        let _ = WRITE_ERROR.set(e.to_string());
    }
}

/// Writes to stderr, ignoring a failure.
pub(crate) fn stderr_write(args: std::fmt::Arguments<'_>) {
    let _ = std::io::stderr().lock().write_fmt(args);
}

/// The process exit code: `code`, unless a stdout write failed for a reason
/// other than a closed pipe. Then the error is printed, and a code that says
/// the output is the result (`success`: an `Ok` return, 0 or `repair`'s 4)
/// becomes 1; a failing code (an `Err`) is kept.
pub(crate) fn exit_code(code: u8, success: bool) -> u8 {
    match WRITE_ERROR.get() {
        None => code,
        Some(e) => {
            stderr_write(format_args!("ms: cannot write the output: {e}\n"));
            if success {
                1
            } else {
                code
            }
        }
    }
}

macro_rules! print {
    ($($a:tt)*) => { $crate::sink::stdout_write(format_args!($($a)*)) };
}
macro_rules! println {
    () => { $crate::sink::stdout_write(format_args!("\n")) };
    ($($a:tt)*) => { $crate::sink::stdout_write(format_args!("{}\n", format_args!($($a)*))) };
}
// Unused today; defined so a future `eprint!` cannot reach std's.
#[allow(unused_macros)]
macro_rules! eprint {
    ($($a:tt)*) => { $crate::sink::stderr_write(format_args!($($a)*)) };
}
macro_rules! eprintln {
    () => { $crate::sink::stderr_write(format_args!("\n")) };
    ($($a:tt)*) => { $crate::sink::stderr_write(format_args!("{}\n", format_args!($($a)*))) };
}
