//! `ms vectors` — print the SHA-pinned v0.1 test-vector corpus as JSON.
//!
//! Realizes SPEC §2.5. Corpus is `include_str!`-baked at compile time
//! from `crates/ms-cli/vectors/v0.1.json` (in-tree copy; parity with
//! `crates/ms-codec/tests/vectors/v0.1.json` enforced by the parity test).

use clap::Args;

use crate::error::{CliError, Result};

const VECTORS_V0_1_JSON: &str = include_str!("../../vectors/v0.1.json");

/// `ms vectors` arguments.
#[derive(Args, Debug)]
pub struct VectorsArgs {
    /// Indent the JSON output for human readability.
    #[arg(long)]
    pub pretty: bool,
}

/// Run `ms vectors`. Always exits 0 with the corpus on stdout.
pub fn run(args: VectorsArgs) -> Result<()> {
    if args.pretty {
        let parsed: serde_json::Value = serde_json::from_str(VECTORS_V0_1_JSON)
            .map_err(|e| CliError::BadInput(format!("vector corpus parse: {}", e)))?;
        let pretty = serde_json::to_string_pretty(&parsed)
            .map_err(|e| CliError::BadInput(format!("vector corpus serialize: {}", e)))?;
        println!("{}", pretty);
    } else {
        // Compact: print as-is.
        print!("{}", VECTORS_V0_1_JSON);
        if !VECTORS_V0_1_JSON.ends_with('\n') {
            println!();
        }
    }
    Ok(())
}
