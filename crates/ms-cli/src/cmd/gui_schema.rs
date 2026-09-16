//! `ms gui-schema` — emit a SPEC §7 JSON description of this CLI's flag
//! surface for consumption by `mnemonic-gui`'s schema-mirror CI gate.
//!
//! Format (per `bg002h/mnemonic-gui` SPEC §7, Phase C of v0.2):
//!
//! ```json
//! {
//!   "version": 1,
//!   "cli": "ms",
//!   "subcommands": [
//!     { "name": "encode", "flags": [...], "positionals": [...] },
//!     ...
//!   ]
//! }
//! ```
//!
//! The output is intentionally lossy: GUI `FlagKind` variants beyond
//! `text` / `boolean` / `number` / `dropdown` / `path` are mapped to
//! `"text"` upstream and hand-overridden in the GUI's schema files.
//! `choices` is non-null only when `kind == "dropdown"`.
//!
//! Implementation note: we walk `clap::CommandFactory::command()`
//! reflection rather than maintaining a parallel hand-written table —
//! that way the JSON stays in lockstep with `Cli` automatically, and
//! the `mnemonic-gui` schema-mirror gate catches any drift.

use clap::{ArgAction, CommandFactory};
use serde::Serialize;

use crate::error::{CliError, Result};
use crate::Cli;

/// SPEC §7 top-level JSON object.
#[derive(Serialize)]
struct SchemaRoot<'a> {
    version: u32,
    cli: &'a str,
    subcommands: Vec<SchemaSubcommand>,
}

/// SPEC §7 per-subcommand entry.
#[derive(Serialize)]
struct SchemaSubcommand {
    name: String,
    flags: Vec<SchemaFlag>,
    positionals: Vec<SchemaPositional>,
}

/// SPEC §7 per-flag entry. `choices` is non-null only for `kind == "dropdown"`.
#[derive(Serialize)]
struct SchemaFlag {
    name: String,
    required: bool,
    kind: &'static str,
    choices: Option<Vec<String>>,
}

/// SPEC §7 per-positional entry.
#[derive(Serialize)]
struct SchemaPositional {
    name: String,
    required: bool,
    repeating: bool,
}

/// Run `ms gui-schema`. Walks the clap command tree and prints the JSON
/// blob on stdout. Always exits 0 (clap parse errors land in main.rs).
pub fn run() -> Result<u8> {
    let cmd = Cli::command();
    let mut subcommands: Vec<SchemaSubcommand> = Vec::new();
    for sub in cmd.get_subcommands() {
        // Skip clap's auto-generated `help` subcommand and the
        // `gui-schema` subcommand itself (the GUI doesn't surface either).
        let name = sub.get_name();
        if name == "help" || name == "gui-schema" {
            continue;
        }
        subcommands.push(reflect_subcommand(sub));
    }

    let root = SchemaRoot {
        version: 1,
        cli: "ms",
        subcommands,
    };
    let s = serde_json::to_string(&root)
        .map_err(|e| CliError::BadInput(format!("gui-schema serialization: {}", e)))?;
    println!("{}", s);
    Ok(0)
}

/// Reflect a single `clap::Command` into a `SchemaSubcommand` entry.
fn reflect_subcommand(sub: &clap::Command) -> SchemaSubcommand {
    let mut flags: Vec<SchemaFlag> = Vec::new();
    let mut positionals: Vec<SchemaPositional> = Vec::new();
    for arg in sub.get_arguments() {
        // Skip clap's auto-generated `--help` / `-h` (GUI doesn't surface).
        if arg.get_id() == "help" {
            continue;
        }
        // F-597: HIDDEN args are not part of the program's surface, and the
        // schema is the GUI's mirror of that surface. The only hidden arg
        // today is `derive --phrase-stdin`, which exists solely to be refused
        // -- reflecting it would have the GUI offer a control whose every
        // outcome is an error. A hidden arg that ever becomes real must be
        // un-hidden, which is the same act that re-enrols it here.
        if arg.is_hide_set() {
            continue;
        }
        if arg.is_positional() {
            positionals.push(SchemaPositional {
                name: arg.get_id().to_string(),
                required: arg.is_required_set(),
                // `Arg::get_num_args()` returns a `ValueRange`; repeating
                // = max > 1. Use `clap::builder::ValueRange::max_values`
                // via the public `get_num_args` accessor.
                repeating: arg
                    .get_num_args()
                    .map(|r| r.max_values() > 1)
                    .unwrap_or(false),
            });
        } else {
            // Named flag. Prefer the long name (`--phrase`); fall back
            // to short (`-j`). All v0.1 ms-cli flags have long forms.
            let name = if let Some(long) = arg.get_long() {
                format!("--{}", long)
            } else if let Some(short) = arg.get_short() {
                format!("-{}", short)
            } else {
                // Defensive: arg with neither long nor short shouldn't
                // exist for a named flag. Skip it.
                continue;
            };
            let (kind, choices) = classify_flag(arg);
            flags.push(SchemaFlag {
                name,
                required: arg.is_required_set(),
                kind,
                choices,
            });
        }
    }
    SchemaSubcommand {
        name: sub.get_name().to_string(),
        flags,
        positionals,
    }
}

/// Classify a clap `Arg` into a SPEC §7 `kind` + optional `choices`.
///
/// Rules (per SPEC §7):
/// - `ArgAction::SetTrue` / `SetFalse` / `Count` → `"boolean"`.
/// - `Arg::get_possible_values()` non-empty → `"dropdown"` + choices.
/// - Otherwise → `"text"` (the lossy default; complex GUI variants are
///   hand-overridden in the GUI schema file after JSON-bootstrap import).
///
/// `"number"` and `"path"` are not produced by this CLI (no flag carries
/// a numeric-only or path-only type); they remain in the SPEC for the
/// sibling CLIs (`md`, `mk`) and are documented here for completeness.
fn classify_flag(arg: &clap::Arg) -> (&'static str, Option<Vec<String>>) {
    // Boolean flags (no value): SetTrue / SetFalse / Count.
    match arg.get_action() {
        ArgAction::SetTrue | ArgAction::SetFalse | ArgAction::Count => {
            return ("boolean", None);
        }
        _ => {}
    }

    // Dropdown: clap value_enum produces a populated possible_values list.
    let possible: Vec<String> = arg
        .get_possible_values()
        .iter()
        .map(|pv| pv.get_name().to_string())
        .collect();
    if !possible.is_empty() {
        return ("dropdown", Some(possible));
    }

    // Fallback: text (lossy default per SPEC §7).
    ("text", None)
}
