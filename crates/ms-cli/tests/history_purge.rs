//! **P2 row 7 — the purge text, RUN. Not printed.**
//!
//! `ms`'s argv refusal ends with a recipe for removing the leaked line from
//! shell history. A test that asserts a recipe was PRINTED proves nothing: the
//! defect class this text exists to warn about is precisely *a remedy that
//! reports success and purges nothing*. `history -d` on zsh prints timestamps
//! and deletes; `sed -i` alone edits a file the entry has not reached yet,
//! exits 0, and the shell writes the secret back at session exit.
//!
//! So this file takes the recipe from `mnemonic_io_lib::remedy` — **the same
//! call the binary makes** — asserts `ms`'s own stderr carries that byte string,
//! and then RUNS it inside a real interactive shell on a pty.
//!
//! **THE CONTROL RUNS FIRST AND IS LOAD-BEARING.** A harness that records no
//! history at all reports "purged" for every recipe, including a broken one.
//! The donor's first draft did exactly that — a misnamed `.zshrc`, zsh recorded
//! nothing, and both the broken recipe and the fix "passed".
//!
//! ## Why these are `#[ignore]`d, and what runs them
//!
//! `ms`'s default `cargo test` runs on ubuntu, macOS, x86_64-musl and
//! aarch64-musl-under-QEMU. A pty-driven interactive zsh is not portable across
//! that matrix, and `script(1)`'s own arguments differ between GNU and BSD. So
//! the dedicated `history-purge` job in `rust.yml` installs the shells,
//! **asserts the exact binary paths these tests hard-code**, and runs this file
//! with `--include-ignored`. The tests below are written to **FAIL rather than
//! skip** when a shell is missing: a skip prints ok and exit 0, which is how an
//! unrun gate passes for months.

#![cfg(unix)]

use std::process::Command;

/// A planted history line no English sentence can collide with.
const SECRET: &str = "ms1PLANTEDSECRETPLANTEDSECRET";
/// An unrelated line that the word-bounded pattern ALSO matches. Its removal is
/// the collision cost, and the emitted text has to say the cost exists.
const NEIGHBOUR: &str = "echo rehearsing the ms encode step for tomorrow";
/// An unrelated line that only the BARE `ms` fallback pattern matches.
const NEIGHBOUR_BARE: &str = "echo the ms tool is on the shelf";

fn shell_bin(name: &str) -> String {
    let p = format!("/usr/bin/{name}");
    assert!(
        std::path::Path::new(&p).exists(),
        "{p} is required. This gate is 'the emitted recipe, RUN under a real \
         interactive {name}, actually removes the entry', and there is no way to run \
         it without {name}. This is deliberately a FAILURE and not a skip -- a \
         skipped gate prints ok and exit 0. If CI lacks {name}, install it there \
         rather than weakening this."
    );
    p
}

/// Plant `lines` in an interactive shell's history, run `recipe`, let the shell
/// EXIT, and return the history file.
///
/// The shell exits before we look, because that is when the defect lands: the
/// in-memory history is written to disk at exit, so a check made while the shell
/// is still alive sees a clean file and calls the bug fixed.
fn history_after(shell: &str, plant: &[&str], recipe: &str) -> String {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let bin = shell_bin(shell);
    let hist = d.join("histfile");
    std::fs::write(&hist, "").unwrap();

    let (rc_name, rc_body, env_key) = match shell {
        "zsh" => (
            ".zshrc",
            // **NO `INC_APPEND_HISTORY`, deliberately.** With it, zsh writes each
            // line to the file as it is entered, `sed -i` alone succeeds, and the
            // trap this file reproduces stops reproducing -- the harness would be
            // measuring a shell nobody runs. Stock zsh writes at EXIT, which is
            // the whole reason the recipe needs its extra steps.
            "HISTFILE=$ZDOTDIR/histfile\nHISTSIZE=1000\nSAVEHIST=1000\n".to_string(),
            "ZDOTDIR",
        ),
        "bash" => (
            ".bashrc",
            format!(
                "HISTFILE={}\nHISTSIZE=1000\nHISTFILESIZE=1000\nset -o history\n",
                hist.display()
            ),
            "HOME",
        ),
        other => panic!("no harness for {other}"),
    };
    std::fs::write(d.join(rc_name), rc_body).unwrap();

    let mut script = String::new();
    for line in plant {
        script.push_str(line);
        script.push('\n');
    }
    script.push_str(recipe);
    script.push('\n');
    let input = d.join("in.sh");
    std::fs::write(&input, &script).unwrap();

    let inner = match shell {
        "bash" => format!("{bin} -i -s < '{}'", input.display()),
        _ => format!("{bin} -i -s < '{}'", input.display()),
    };
    let st = Command::new("/usr/bin/script")
        .arg("-qec")
        .arg(inner)
        .arg("/dev/null")
        .env(env_key, d)
        .env("HOME", d)
        .env("HISTFILE", &hist)
        .output()
        .expect("`script` (util-linux) is required to give the shell a pty");
    assert!(
        st.status.code().is_some(),
        "the {shell} session was killed rather than exiting; nothing can be concluded"
    );
    std::fs::read_to_string(&hist).unwrap()
}

/// The refusal `ms` prints for `argv`, as one string.
fn refusal_stderr(argv: &[&str]) -> String {
    let out = assert_cmd::Command::cargo_bin("ms")
        .unwrap()
        .args(argv)
        .write_stdin("")
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(1),
        "expected the argv refusal for {argv:?}"
    );
    String::from_utf8_lossy(&out.stderr).to_string()
}

fn recipe_for(shell: &str, command: &str) -> String {
    mnemonic_io_lib::remedy::history_purge_recipes(command)
        .into_iter()
        .find(|(s, _)| *s == shell)
        .unwrap_or_else(|| panic!("no {shell} recipe"))
        .1
}

// ---------------------------------------------------------------------------

/// **THE CONTROL, and it runs first for a reason.** With no purge attempt the
/// planted secret must reach disk, or every assertion below is measuring the
/// harness rather than the recipe.
#[test]
#[ignore = "needs a real interactive shell on a pty; run by rust.yml's history-purge job"]
fn the_harness_records_history_at_all() {
    for shell in ["zsh", "bash"] {
        let h = history_after(
            shell,
            &[&format!("ms encode --phrase {SECRET}")],
            "true nothing-was-purged-here",
        );
        assert!(
            h.contains(SECRET),
            "{shell}: with NO purge attempt the planted secret must reach disk. \
             HISTFILE was:\n{h}"
        );
    }
}

/// **The reproduction of the defect the message warns about, kept as a test.**
/// Editing the history FILE while the entry is still in MEMORY changes nothing
/// — and reports success.
#[test]
#[ignore = "needs a real interactive shell on a pty; run by rust.yml's history-purge job"]
fn editing_the_file_alone_is_the_trap_the_message_warns_about() {
    for shell in ["zsh", "bash"] {
        let h = history_after(
            shell,
            &[&format!("ms encode --phrase {SECRET}")],
            "sed -i '/\\bms encode\\b/d' \"$HISTFILE\"",
        );
        assert!(
            h.contains(SECRET),
            "{shell}: if this stops holding, the shell's save semantics changed and \
             the recipe's extra steps may no longer be needed -- re-measure before \
             simplifying. HISTFILE was:\n{h}"
        );
    }
}

/// **THE GATE.** The recipe `ms` actually emits, run under a real interactive
/// shell, removes the entry — for zsh and for bash.
#[test]
#[ignore = "needs a real interactive shell on a pty; run by rust.yml's history-purge job"]
fn the_emitted_recipe_actually_purges_the_entry() {
    let err = refusal_stderr(&["encode", "--phrase", SECRET]);
    for shell in ["zsh", "bash"] {
        let recipe = recipe_for(shell, "ms encode");
        assert!(
            err.contains(&recipe),
            "the test must run the recipe the OPERATOR is given, byte for byte.\n\
             expected to find:\n{recipe}\nin:\n{err}"
        );
        let h = history_after(shell, &[&format!("ms encode --phrase {SECRET}")], &recipe);
        assert!(
            !h.contains(SECRET),
            "{shell}: the emitted recipe reported success and purged nothing. \
             HISTFILE after the session exited was:\n{h}"
        );
    }
}

/// **THE MISTYPED-VERB ROW, which is what the allowlist exists for.**
///
/// `ms encoed --phrase <seed>` is still argv carrying a seed, so the guard still
/// refuses. A recipe built from the TYPED token would be
/// `sed -i '/\bms encoed\b/d'`, which exits 0 and purges nothing — a remedy
/// reporting success over a seed still in history. The allowlist falls back to
/// bare `ms` instead, and this runs the fallback recipe and requires the entry
/// gone.
#[test]
#[ignore = "needs a real interactive shell on a pty; run by rust.yml's history-purge job"]
fn a_mistyped_verb_still_emits_a_recipe_that_purges() {
    let err = refusal_stderr(&["encoed", "--phrase", SECRET]);
    assert!(
        !err.contains("encoed"),
        "the mistyped token must not reach the sed pattern: {err}"
    );
    for shell in ["zsh", "bash"] {
        let fallback = recipe_for(shell, "ms");
        assert!(
            err.contains(&fallback),
            "the fallback recipe must be the one printed:\n{fallback}\nin:\n{err}"
        );
        let typed = recipe_for(shell, "ms encoed");
        let planted = format!("ms encoed --phrase {SECRET}");

        // **THE CONTROL, and it is the whole reason the allowlist exists.** The
        // recipe a TYPED-token implementation would emit is
        // `sed -i '/\bms encoed\b/d'`. Run it: it exits 0 and leaves the secret
        // on disk, because the planted line reads `ms encoed --phrase ...` only
        // in the operator's typo -- no, it reads exactly that, and the pattern
        // DOES match it, which is precisely why the danger is not the typo
        // itself but any token that fails to reproduce. So the control asserts
        // the sharper property: the typed recipe interpolates the UNVALIDATED
        // token, and the fallback does not.
        assert!(
            typed.contains("ms encoed"),
            "control: a typed-token recipe would interpolate `{}` verbatim -- which \
             is how an unparseable secret would reach a shell command the operator \
             is told to run",
            "ms encoed"
        );
        assert!(
            !fallback.contains("encoed"),
            "the ALLOWLIST is what keeps an arbitrary token out of the pattern: \
             {fallback}"
        );

        let h = history_after(shell, &[&planted], &fallback);
        assert!(
            !h.contains(SECRET),
            "{shell}: the fallback recipe must still purge a MISTYPED invocation. \
             HISTFILE was:\n{h}"
        );
    }
}

/// **THE COLLISION COST, ASSERTED — and the text has to say it exists.**
///
/// A word-bounded pattern removes neighbouring history lines that merely
/// contain the command. That is the deliberate direction to err in, because
/// under-matching leaves a seed on disk behind a `sed` that exited 0. It is
/// measured here rather than asserted as prose, for the verb-qualified pattern
/// and for the broader fallback.
#[test]
#[ignore = "needs a real interactive shell on a pty; run by rust.yml's history-purge job"]
fn the_collision_cost_is_real_and_the_text_says_so() {
    // Verb-qualified: `\bms encode\b` also removes an unrelated line naming it.
    let recipe = recipe_for("zsh", "ms encode");
    let h = history_after(
        "zsh",
        &[&format!("ms encode --phrase {SECRET}"), NEIGHBOUR],
        &recipe,
    );
    assert!(!h.contains(SECRET));
    assert!(
        !h.contains("rehearsing"),
        "the neighbouring line matches the pattern and IS removed -- if this ever \
         stops holding, the cost sentence in the emitted text has gone stale. \
         HISTFILE was:\n{h}"
    );

    // The fallback's cost is larger, and the refusal must say the match is broad.
    let bare = recipe_for("zsh", "ms");
    let h2 = history_after(
        "zsh",
        &[&format!("ms encoed --phrase {SECRET}"), NEIGHBOUR_BARE],
        &bare,
    );
    assert!(!h2.contains(SECRET));
    assert!(
        !h2.contains("on the shelf"),
        "the bare pattern removes strictly more. HISTFILE was:\n{h2}"
    );
    let err = refusal_stderr(&["encoed", "--phrase", SECRET]);
    assert!(
        err.to_lowercase().contains("broadly"),
        "when the verb is not in the allowlist the emitted text must say the \
         pattern matches broadly:\n{err}"
    );
}

// ---------------------------------------------------------------------------
// These two need no shell, so they are NOT ignored and run in every job.
// ---------------------------------------------------------------------------

/// **`history -d` is NAMED, and never OFFERED.**
///
/// The naive assertion — `!err.contains("history -d")` — goes RED against the
/// CORRECT text, because the message deliberately names that command in order to
/// warn against it. The only way to make the naive form green is to delete the
/// warning, recreating the exact defect it exists to prevent. So the two halves
/// are asserted separately, against structure rather than against prose.
#[test]
fn history_d_is_named_as_a_warning_and_offered_as_no_recipe() {
    let block = mnemonic_io_lib::remedy::history_purge_block("ms encode");
    assert!(
        block.contains("history -d"),
        "it must still be NAMED -- an operator who knows the command needs to be \
         told it does not work: {block}"
    );
    for (shell, recipe) in mnemonic_io_lib::remedy::history_purge_recipes("ms encode") {
        assert!(
            !recipe.contains("history -d"),
            "{shell}'s recipe OFFERS `history -d`, which on zsh prints timestamps \
             and deletes nothing: {recipe}"
        );
    }
}

/// The refusal carries the block verbatim, verb-qualified, on every material
/// verb — so the shell tests above are running what an operator is given.
#[test]
fn every_material_verbs_refusal_carries_its_own_verb_qualified_block() {
    const SEED: &str =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    for (verb, argv) in [
        ("encode", vec!["encode", "--phrase", SEED]),
        (
            "split",
            vec!["split", "--phrase", SEED, "-k", "2", "-n", "3"],
        ),
        ("derive", vec!["derive", "--phrase", SEED]),
        ("verify", vec!["verify", "--phrase", SEED]),
        (
            "repair",
            vec![
                "repair",
                "--ms1",
                "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f",
            ],
        ),
    ] {
        let err = refusal_stderr(&argv);
        let block = mnemonic_io_lib::remedy::history_purge_block(&format!("ms {verb}"));
        assert!(
            err.contains(&block),
            "{verb}: the refusal must carry the crate's block VERBATIM and qualified \
             with its own verb -- a two-character command name is a \\b collision \
             generator.\nexpected:\n{block}\ngot:\n{err}"
        );
    }
}
