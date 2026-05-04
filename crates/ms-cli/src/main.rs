//! `ms` — engrave-friendly BIP-39 entropy backups (the `ms1` format).
//!
//! Companion CLI to the `ms-codec` library. See `design/SPEC_ms_cli_v0_1.md`
//! for the full surface specification.

#![allow(missing_docs)]
// ms-cli is binary-only; field-level docs are pretty but not load-bearing for a non-published lib API. Mirror md-cli precedent at crates/md-cli/src/main.rs:1.
#![allow(dead_code)] // Phase 1 stub: main() is empty; leaf modules are wired but not yet called. Remove when Phase 3 clap dispatch is in place.

mod bip39_friendly;
mod codex32_friendly;
mod error;
mod format;
mod language;
mod parse;

fn main() {
    // Phase 3 replaces this with the clap dispatch.
}
