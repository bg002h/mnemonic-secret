//! **P2 row 4's gate (b) — the pinned rev carries every item `ms` adopts.**
//!
//! The retracted gate for this row was *"`git ls-tree -d origin/master crates/`
//! names `crates/mnemonic-io-lib` — it does not today"*. It did, and it did
//! before the plan was merged, so that assertion could never have failed
//! (R0 round 0's I-1). **This is the failure the retracted gate was reaching
//! for and could not express:** a rev that predates any adopted item does not
//! compile here.
//!
//! `ms` takes **4 of the crate's 11** public items and declines 7. The four are
//! named below; the declines are asserted by `tests/the_decline.rs`.
//!
//! The items are USED rather than merely imported. An `#[allow(unused_imports)]`
//! `use` line compiles against a crate whose function signatures have all
//! changed, so it would pin the module PATHS and nothing about their contracts.

use mnemonic_io_lib::{
    fd::mode_of,
    remedy::{history_purge_block, history_purge_recipes},
    write::write_private,
};

/// Every adopted item resolves, is callable, and behaves as the boundary
/// section says it does.
#[test]
fn the_four_adopted_items_are_present_and_callable_at_the_pinned_rev() {
    // `write_private` -- the whole of the 0600 `--out`.
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("card.ms1");
    write_private(&p, b"ms10entrs\n").expect("write_private must write");
    assert_eq!(std::fs::read(&p).unwrap(), b"ms10entrs\n");

    // `fd::mode_of` -- the `--out` post-write assertion. Its contract is the
    // RAW mode, not a verdict, which is what makes it usable for a 0600 check.
    let md = std::fs::metadata(&p).unwrap();
    assert_eq!(
        mode_of(&md),
        Some(0o600),
        "write_private must create owner-only, and mode_of must report the raw mode"
    );

    // `remedy::history_purge_recipes` -- the block's structured half, so a test
    // can RUN the emitted recipe rather than a copy of it.
    let recipes = history_purge_recipes("ms encode");
    let shells: Vec<&str> = recipes.iter().map(|(s, _)| *s).collect();
    assert_eq!(
        shells,
        vec!["zsh", "bash", "fish"],
        "the three shells the purge work runs are all present at this rev"
    );
    assert!(
        recipes
            .iter()
            .any(|(s, r)| *s == "fish" && r == "history clear-session"),
        "fish is PRESCRIBED at this rev, not merely described -- the plan's §3 \
         re-measurement said so and this is the check of it"
    );

    // `remedy::history_purge_block` -- what the argv refusal prints.
    let block = history_purge_block("ms encode");
    assert!(
        block.contains("ms encode"),
        "the block is verb-qualified: a two-character command name is a \\b collision \
         generator, and the crate's own doc records `\\bme\\b` removing `cd /home/me`"
    );
    for (shell, recipe) in history_purge_recipes("ms encode") {
        assert!(
            block.contains(&recipe),
            "{shell}'s recipe must appear in the printed block verbatim, or a test that \
             runs the recipe is not running what the operator is given"
        );
    }
}
