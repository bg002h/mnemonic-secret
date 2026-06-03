//! `ms` — engrave-friendly BIP-39 entropy backups (the `ms1` format).
//!
//! Companion CLI to the `ms-codec` library. See `design/SPEC_ms_cli_v0_1.md`
//! for the full surface specification.

#![allow(missing_docs)] // ms-cli is binary-only; field-level docs are pretty but not load-bearing for a non-published lib API. Mirror md-cli precedent at crates/md-cli/src/main.rs:1.

mod advisory;
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
mod process_hardening;

use std::io::Write;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use error::{CliError, Result};
use format::{ErrorBodyJson, ErrorEnvelopeJson};

/// Top-level `after_help` footer (renders on both `ms -h` and `ms --help`).
/// Mirror of the `mnemonic` CLI footer: points users who hold the entropy
/// (an ms1 backup / seed words) but have lost the BIP-39 passphrase at
/// btcrecover. A BIP-39 passphrase has no internal verifier, so it cannot
/// be brute-forced from the entropy alone — correctness is only definable
/// against a known address/xpub/master-fingerprint (external-derivation
/// oracle, which `ms` does not perform — `ms` only encodes/decodes
/// entropy). Date-stamped per the 2026-05-25 recon decision; guarded by
/// `tests/cli_help_pointer.rs` and mirrored in the constellation manual at
/// `mnemonic-toolkit/docs/manual/src/40-cli-reference/43-ms.md`.
const PASSPHRASE_RECOVERY_HELP: &str = "\
RECOVERING A FORGOTTEN BIP-39 PASSPHRASE:
  If you have your entropy (your ms1 backup or seed words) but not the
  BIP-39 passphrase (the optional \"25th word\"), it cannot be
  brute-forced from the entropy alone. An external open-source tool can:
  btcrecover searches passphrase candidates and confirms each by deriving
  an address / xpub / master-fingerprint at common default paths and
  matching a value you already know.
    btcrecover (maintained):  https://github.com/3rdIteration/btcrecover
    original:                 https://github.com/gurnec/btcrecover
  Pointer current as of 2026-05-25. Run untrusted recovery tools
  offline, on an air-gapped machine.";

#[derive(Parser, Debug)]
#[command(
    name = "ms",
    version,
    about = "ms — engrave-friendly BIP-39 entropy backups (the ms1 format)",
    after_help = PASSPHRASE_RECOVERY_HELP
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Derive the master fingerprint (+ account xpub with --template) — read-only public derivation.
    Derive(cmd::derive::DeriveArgs),

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

    /// Repair an ms1 string via BCH error correction (exit 5 = REPAIR_APPLIED).
    ///
    /// Single-HRP context: no `--hrp` flag. Up to BCH(93,80,8) t=4 single-chunk
    /// correction capacity via `ms_codec::decode_with_correction`. The corrected
    /// ms1 is emitted on stdout (with a stderr `PrivateKeyMaterial` advisory
    /// per D9 — ms1 is BIP-39 entropy and sensitive). Exit 5 on
    /// correction-applied (D26); exit 0 if input was already valid; exit 2
    /// if BCH-uncorrectable (`TooManyErrors`).
    #[command(
        after_long_help = "EXAMPLES:\n  ms repair --ms1 ms10entrsqq...        # text-form report on stdout\n  ms repair --ms1 - < broken.txt        # read ms1 from stdin\n  ms repair --ms1 ms10entrsqq... --json # JSON envelope on stdout"
    )]
    Repair(cmd::repair::RepairArgs),

    /// Split a secret (mnemonic / hex entropy) into N codex32 K-of-N shares.
    ///
    /// Any K of the N shares recombine to the original via `ms combine`. The
    /// whole N-share SET is secret-equivalent (a stderr `PrivateKeyMaterial`
    /// advisory is emitted). A non-English `--phrase` splits as a `mnem`
    /// share-set so the wordlist language survives the split. Bounds: 2 ≤ K ≤ N ≤ 31.
    #[command(
        after_long_help = "EXAMPLES:\n  ms split --phrase \"abandon abandon … about\" -k 2 -n 3\n  ms split --hex 00000000000000000000000000000000 -k 3 -n 5\n  ms split --language japanese --phrase \"…\" -k 2 -n 3 --json | jq .shares"
    )]
    Split(cmd::split::SplitArgs),
}

fn main() -> ExitCode {
    // argv-hardening: deny other-UID /proc/$PID/cmdline reads + core dumps.
    process_hardening::set_non_dumpable();
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

    let result: Result<u8> = match cli.command {
        Command::Derive(args) => cmd::derive::run(args),
        Command::Encode(args) => cmd::encode::run(args),
        Command::Decode(args) => cmd::decode::run(args),
        Command::Inspect(args) => cmd::inspect::run(args),
        Command::Verify(args) => cmd::verify::run(args),
        Command::Vectors(args) => cmd::vectors::run(args),
        Command::GuiSchema => cmd::gui_schema::run(),
        Command::Repair(args) => cmd::repair::run(args),
        Command::Split(args) => cmd::split::run(args),
    };

    let exit = match result {
        Ok(code) => ExitCode::from(code),
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
        Command::Derive(a) => a.json,
        Command::Encode(a) => a.json,
        Command::Decode(a) => a.json,
        Command::Inspect(a) => a.json,
        Command::Verify(a) => a.json,
        Command::Vectors(_) => false, // vectors output is always JSON-shaped
        Command::GuiSchema => false,  // gui-schema output is always JSON-shaped
        Command::Repair(a) => a.json,
        Command::Split(a) => a.json,
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
