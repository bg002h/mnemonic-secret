//! `ms` — engrave-friendly BIP-39 entropy backups (the `ms1` format).
//!
//! Companion CLI to the `ms-codec` library. See `design/SPEC_ms_cli_v0_1.md`
//! for the full surface specification.

#![allow(missing_docs)] // ms-cli is binary-only; field-level docs are pretty but not load-bearing for a non-published lib API. Mirror md-cli precedent at crates/md-cli/src/main.rs:1.

mod bip39_friendly;
mod cmd;
mod codex32_friendly;
mod error;
mod format;
mod language;
// Inline copy of mnemonic-toolkit's mlock module per SPEC §5 + §6 G6.
// Test helpers (failure_count_for_test, first_errno_for_test, etc.) are
// part of the verbatim diff manifest; they're unused in ms-cli's binary
// context (no integration tests reach them yet) but kept to preserve
// byte-equality with the toolkit's source under G6 normalization.
#[allow(dead_code)]
mod mlock;
mod parse;

use std::io::Write;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use error::{CliError, Result};
use format::{ErrorBodyJson, ErrorEnvelopeJson};

#[derive(Parser, Debug)]
#[command(
    name = "ms",
    version,
    about = "ms — engrave-friendly BIP-39 entropy backups (the ms1 format)"
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Encode a BIP-39 mnemonic (or hex entropy) as an ms1 string for engraving.
    #[command(
        after_long_help = "EXAMPLES:\n  ms encode --phrase \"abandon abandon … about\"\n  ms encode --phrase - < phrase.txt\n  ms encode --hex 00000000000000000000000000000000 --no-engraving-card\n  ms encode --phrase \"...\" --json | jq .ms1"
    )]
    Encode(cmd::encode::EncodeArgs),

    /// Decode an ms1 string back to its BIP-39 mnemonic and entropy bytes.
    #[command(
        after_long_help = "EXAMPLES:\n  ms decode ms10entrs…\n  ms decode - < engraved.txt\n  ms decode <ms1> --language french\n  ms decode <ms1> --json | jq .phrase"
    )]
    Decode(cmd::decode::DecodeArgs),

    /// Inspect an ms1 string's structural fields and decoder verdict.
    #[command(
        after_long_help = "EXAMPLES:\n  ms inspect <ms1>          # verdict + fields\n  ms inspect <ms1> --json   # structured output for tooling\n  printf \"ms10e ntrsq…\" | ms inspect -   # back-typed chunked form"
    )]
    Inspect(cmd::inspect::InspectArgs),

    /// Verify an ms1 string is valid (and optionally round-trips against a phrase).
    #[command(
        after_long_help = "EXAMPLES:\n  ms verify <ms1>                          # exit 0 = valid v0.1\n  ms verify <ms1> --phrase \"abandon … about\"   # round-trip; exit 4 on mismatch\n  ms verify <ms1> --phrase \"...\" --json    # structured outcome"
    )]
    Verify(cmd::verify::VerifyArgs),

    /// Print the SHA-pinned v0.1 test-vector corpus as JSON.
    #[command(
        after_long_help = "EXAMPLES:\n  ms vectors                # compact JSON\n  ms vectors --pretty       # indented JSON\n  ms vectors | jq '.[0]'    # filter via jq"
    )]
    Vectors(cmd::vectors::VectorsArgs),

    /// Emit a SPEC §7 JSON description of this CLI's flag surface (for `mnemonic-gui`).
    ///
    /// Consumed by the `mnemonic-gui` schema-mirror CI gate (v0.2+).
    /// Intentionally lossy: complex GUI `FlagKind` variants map to
    /// `"text"` upstream and are hand-overridden in the GUI schema
    /// file after JSON-bootstrap import. See `bg002h/mnemonic-gui`
    /// `FOLLOWUPS.md` entry `mnemonic-gui-schema-mirror`.
    #[command(
        name = "gui-schema",
        after_long_help = "EXAMPLES:\n  ms gui-schema | jq .version             # always 1\n  ms gui-schema | jq '.subcommands[].name' # list subcommands\n  ms gui-schema | jq '.subcommands[] | select(.name == \"encode\").flags'"
    )]
    GuiSchema,
}

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            // Clap returns Err for two non-error terminations: --version
            // (ErrorKind::DisplayVersion) and --help (ErrorKind::DisplayHelp).
            // Output is on stdout and the canonical Unix exit is 0. The
            // catch-all 64 below preserves SPEC §6's carve-out for *real*
            // parse errors (overrides clap's default of 2 to keep 2
            // reserved for ms1 format violations).
            e.print().ok();
            return match e.kind() {
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => {
                    ExitCode::SUCCESS
                }
                _ => ExitCode::from(64),
            };
        }
    };

    let json_mode = is_json_mode(&cli.command);

    let result: Result<()> = match cli.command {
        Command::Encode(args) => cmd::encode::run(args),
        Command::Decode(args) => cmd::decode::run(args),
        Command::Inspect(args) => cmd::inspect::run(args),
        Command::Verify(args) => cmd::verify::run(args),
        Command::Vectors(args) => cmd::vectors::run(args),
        Command::GuiSchema => cmd::gui_schema::run(),
    };

    let exit = match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            emit_error(&e, json_mode);
            ExitCode::from(e.exit_code())
        }
    };

    // Cycle B SPEC §2 row 3 + §6 G2.5 — emit a 2-line stderr summary iff
    // any pin_pages_for call soft-failed during this invocation. No-op
    // when failure_count == 0. Runs on both Ok and Err paths.
    mlock::report_at_exit();

    exit
}

fn is_json_mode(cmd: &Command) -> bool {
    match cmd {
        Command::Encode(a) => a.json,
        Command::Decode(a) => a.json,
        Command::Inspect(a) => a.json,
        Command::Verify(a) => a.json,
        Command::Vectors(_) => false, // vectors output is always JSON-shaped
        Command::GuiSchema => false,  // gui-schema output is always JSON-shaped
    }
}

fn emit_error(e: &CliError, json_mode: bool) {
    // Special case: FutureFormat is a "success-shaped" exit-3 path used by
    // verify. In text mode, cmd::verify::emit_future_format already wrote the
    // "OK: valid future format" line to stdout; emitting an "error: ..."
    // message to stderr here would contradict that. Skip the stderr write.
    // In JSON mode we DO want the error envelope (cmd handler suppressed its
    // own stdout output specifically so this path produces the envelope).
    if matches!(e, CliError::FutureFormat { .. }) && !json_mode {
        return;
    }

    if json_mode {
        // JSON-mode errors go to stdout (one stream) per SPEC §6.3.
        let envelope = ErrorEnvelopeJson {
            schema_version: "1",
            error: ErrorBodyJson {
                kind: e.kind(),
                message: e.message(),
                exit_code: e.exit_code(),
                details: e.details(),
            },
        };
        let s = serde_json::to_string(&envelope).expect("error envelope serializes");
        println!("{}", s);
    } else {
        // Text-mode errors go to stderr.
        let mut stderr = std::io::stderr().lock();
        writeln!(stderr, "{}", e).ok();
    }
}
