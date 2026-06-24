//! `ms gen-man --out <DIR>` — self-emit roff man pages from the compiled clap
//! `Command` tree (one page per subcommand) into `<DIR>`.
//!
//! The pages are clap-generated, hence binary-faithful by construction: there
//! is no hand-authored content and no content-fidelity gate (contrast the
//! hand-authored `docs/manual/` mirror).
//!
//! Mechanism (per SPEC_constellation_man_pages.md §2): call the bare, naive
//! `clap_mangen::generate_to(Cli::command(), &dir)` with **NO pre-`.build()`**.
//! `generate_to` internally does `disable_help_subcommand(true)` then builds —
//! an external `.build()` would run first and materialize the `help`
//! pseudo-subcommand shadow tree (~spurious `*-help*.1` pages, C-1). The naive
//! call is clean: one page per real (sub)command, zero `*-help*.1` pages.

use std::path::PathBuf;

use clap::CommandFactory;

use crate::error::{CliError, Result};
use crate::Cli;

#[derive(clap::Args, Debug)]
pub struct GenManArgs {
    /// Directory to write the `*.1` man pages into (created if absent).
    #[arg(long, value_name = "DIR")]
    pub out: PathBuf,
}

pub fn run(args: GenManArgs) -> Result<u8> {
    std::fs::create_dir_all(&args.out)
        .map_err(|e| CliError::BadInput(format!("cannot create --out dir: {e}")))?;
    // NAIVE call — NO pre-`.build()` (C-1). `generate_to` builds internally with
    // the help subcommand disabled, so the output carries zero `*-help*.1` pages.
    clap_mangen::generate_to(Cli::command(), &args.out)
        .map_err(|e| CliError::BadInput(format!("man-page generation failed: {e}")))?;
    Ok(0)
}
