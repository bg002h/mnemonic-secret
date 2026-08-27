//! **THE PRE-PARSER argv GUARD — the first `std::env::args()` site `ms` has
//! ever had.**
//!
//! Before P2, `ms encode --phrase "<a real seed>"` exited **0 in silence**.
//! `ms` never read its own argv, never fstat'ed its stdout, and never created a
//! file with a mode: measured, `git grep -n 'env::args'` and
//! `git grep -n 'fs::write\|OpenOptions\|set_permissions\|0o600\|0o077\|0o044\|st_mode'`
//! both scoped to `crates/` returned **zero hits**. So this is not a port of a
//! mechanism onto a tool that had a worse version of it. It is the first
//! installation, on the tool that holds the material the whole cycle is about.
//!
//! ## It runs BEFORE `Cli::try_parse()`, and that ordering is the fix
//!
//! A guard downstream of the parser has already lost. `mt`'s source records the
//! lesson from the other side: when its check lived inside the `encode`
//! subcommand, clap rejected the unexpected positional first **and clap's error
//! echoed the entire bearer transaction to stderr**. `ms` maps every clap error
//! to 64 (`main.rs`, the `Cli::try_parse()` arm) and clap names the offending
//! VALUE for any shape with no declared flag to blame, so a guard placed one
//! line lower would put the material in a second public place while refusing it.
//!
//! ## Two layers, and each covers what the other cannot
//!
//! - **Flag-keyed**, matched as strings with no parse: nine of `ms`'s fourteen
//!   secret-bearing channels are behind a flag, and behind a flag the VALUE does
//!   not have to be recognised at all. This is what makes an UPPERCASE phrase
//!   refuse with THIS message rather than with clap's wordlist error — measured
//!   before any code: eight of the ninety-two cross-product rows already exited
//!   1 silently, so *"exit non-zero and no leak"* could never have failed there.
//! - **Value-shape**, for the five channels where material arrives positionally:
//!   an `ms1` by HRP and charset, a BIP-39 phrase by wordlist, hex by charset
//!   and length.
//!
//! **Granularity is traded knowingly in one direction.** An UNQUOTED twelve-word
//! phrase is twelve separate argv tokens, each a single word, so the shape layer
//! does not reach it; only the quoted, single-token phrase is in reach. That is
//! the same cost the donor accepted, and it is what keeps `combine` (a BIP-39
//! word) from being refused as a subcommand.
//!
//! ## Every token is normalised FOUR ways
//!
//! Trim, ASCII-lowercase, the token whole, **and every `=`-split half of it**.
//! The fourth is the one a first draft dropped: `--phrase=<seed>` is ONE argv
//! token whose left half is not the flag string and whose right half is the
//! secret, so neither layer is even scoped to look at it. Splitting on every `=`
//! rather than the first costs nothing and cannot miss a shape. F-302 records
//! that the `=`-joined spelling leaks today and that a guard whose gate is built
//! from the space-joined spellings alone would pass its own gate while leaking.
//!
//! ## It honours no `--`
//!
//! `ms decode -- <ms1>` is a real shape (measured rc **0** today) and a scan of
//! raw argv reaches it precisely because it does not implement end-of-options.

/// `ms`'s twelve command words, from `ms --help`. **An ALLOWLIST, and that is
/// the whole safety argument for the purge pattern.**
///
/// The guard runs before clap has resolved anything, so the verb is whatever
/// token sits after the binary name — and it is interpolated straight into a
/// `sed` command the operator is told to run. Deriving the words instead
/// ("leading tokens that do not look like material") would admit a TRUNCATED or
/// otherwise unparseable secret into the pattern, since not-recognised is
/// exactly what a near-miss returns. An allowlist of the tool's own subcommand
/// words cannot carry material at all.
///
/// `ms` nests no subcommands, so exactly one word is ever appended.
const SUBCOMMANDS: [&str; 12] = [
    "derive",
    "encode",
    "decode",
    "inspect",
    "verify",
    "vectors",
    "gui-schema",
    "gen-man",
    "repair",
    "split",
    "combine",
    "help",
];

/// The nine flag-keyed secret channels, as strings. No parse, no clap.
///
/// `--passphrase-stdin` is deliberately NOT here and cannot be caught by
/// accident: the match is EQUALITY, not a prefix test.
const SECRET_FLAGS: [&str; 4] = ["--phrase", "--hex", "--ms1", "--passphrase"];

/// The bech32 character set. A codex32 string's data part draws from it alone,
/// which is what separates an `ms1` string from a FILENAME that merely starts
/// with the HRP.
const BECH32_CHARSET: &str = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";

/// The shortest `ms1` string in the v0.1 length set is 50 characters and the
/// shortest share is 49; 48 is below both and above anything a subcommand word
/// or a short path could reach.
const MIN_MS1_LEN: usize = 48;

/// Every string a token could plausibly BE, normalised for classification.
///
/// **Neither trimming nor case-folding is optional.** ` ms1…`, `ms1…` and an
/// uppercase `MS1…` are the same material; a classifier that saw only the
/// literal token would let two of the four spellings through, which is measured
/// in the cross-product gate rather than assumed.
fn argv_candidates(token: &str) -> Vec<String> {
    let norm = |s: &str| s.trim().to_ascii_lowercase();
    let mut v = vec![norm(token)];
    if token.contains('=') {
        v.extend(token.split('=').map(norm));
    }
    v
}

/// What a token looks like, or `None`. The returned string NAMES A CLASS and
/// never reproduces any of the value.
fn material_class(candidate: &str) -> Option<&'static str> {
    if is_ms1_shaped(candidate) {
        return Some("an ms1 string (or one share of an ms1 share-set)");
    }
    if is_phrase_shaped(candidate) {
        return Some("a BIP-39 mnemonic");
    }
    if is_hex_entropy_shaped(candidate) {
        return Some("raw hex entropy");
    }
    None
}

/// An `ms1` string or a share of one: the HRP, then the bech32 charset alone.
///
/// **The charset half is what makes the near-miss control pass.**
/// `ms1-2026-08-23-backup.txt` is a FILENAME beginning with the HRP; it carries
/// `-` and `.`, neither of which is in the charset, so it is not material and
/// `ms verify --in ms1-2026-08-23-backup.txt` is still accepted.
fn is_ms1_shaped(s: &str) -> bool {
    // **Display separators are stripped first, because `ms` strips them on
    // INTAKE.** A share read off a plate arrives grouped -- `ms12un98 qcjj5
    // 3dhr9 ...` -- and `read_shares`/`read_input` both remove whitespace, `-`
    // and `,` before decoding. A guard that classified the RAW token would let
    // the grouped spelling of the very same secret through while refusing the
    // unbroken one. Found during P2's implementation, when a test passing
    // comma-grouped shares positionally was NOT refused.
    let t: String = crate::format::strip_display_separators(s);
    t.len() >= MIN_MS1_LEN
        && t.starts_with("ms1")
        && t[3..].chars().all(|c| BECH32_CHARSET.contains(c))
}

/// A quoted BIP-39 mnemonic: a legal word count, every word in some supported
/// wordlist.
///
/// Membership rather than `Mnemonic::parse_in`, deliberately: `parse_in` also
/// validates the CHECKSUM, so a mistyped or truncated phrase would come back
/// "not a mnemonic" — and a phrase one character wrong is still the operator's
/// seed sitting in `/proc` and in their shell history.
fn is_phrase_shaped(s: &str) -> bool {
    let words: Vec<&str> = s.split_whitespace().collect();
    if !matches!(words.len(), 12 | 15 | 18 | 21 | 24) {
        return false;
    }
    bip39::Language::ALL
        .iter()
        .any(|lang| words.iter().all(|w| lang.find_word(w).is_some()))
}

/// Raw entropy as hex, at one of the five legal lengths.
fn is_hex_entropy_shaped(s: &str) -> bool {
    matches!(s.len(), 32 | 40 | 48 | 56 | 64) && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// The invocation as a `sed`-safe pattern, plus whether the verb allowlist
/// matched.
///
/// **The command is VERB-QUALIFIED, and that is not decoration.** The purge
/// recipe is a word-bounded `sed` pattern, and a two-character command name is a
/// collision generator: the crate's own doc records `\bme\b` also removing
/// `cd /home/me` from a six-line sample. `\bms\b` is the same hazard.
///
/// **When the token is not in the allowlist the pattern falls back to bare
/// `ms`, and the caller says so in the emitted text**, because the two failure
/// directions are not symmetric: over-matching costs the operator unrelated
/// history lines, while under-matching leaves a seed in history behind a `sed`
/// that exited 0. A recipe built from a MISTYPED verb — `ms encoed` — gives
/// `sed -i '/\bms encoed\b/d'`, which exits 0 and purges nothing: a remedy
/// reporting success over a seed still on disk.
fn argv_surface(argv: &[String]) -> (String, bool) {
    match argv.get(1).map(|t| t.trim().to_ascii_lowercase()) {
        Some(word) if SUBCOMMANDS.contains(&word.as_str()) => (format!("ms {word}"), true),
        _ => ("ms".to_string(), false),
    }
}

/// **THE GUARD.** Returns the refusal, or `None`.
///
/// `argv` is the process's raw arguments, argument 0 included, so the positions
/// it names are the ones the operator typed.
pub fn argv_secret_guard(argv: &[String]) -> Option<String> {
    let hit = find_argv_material(argv)?;
    Some(refusal(argv, hit.0, hit.1, hit.2))
}

/// `(index, class, character length)` of the first argv token carrying
/// material, or `None`.
fn find_argv_material(argv: &[String]) -> Option<(usize, &'static str, usize)> {
    for (i, token) in argv.iter().enumerate().skip(1) {
        // Layer 1 -- flag-keyed. The VALUE is not examined at all: behind a
        // secret flag, anything that is not the stdin sentinel is material by
        // declaration.
        let whole = token.trim().to_ascii_lowercase();
        if SECRET_FLAGS.contains(&whole.as_str()) {
            if let Some(value) = argv.get(i + 1) {
                if value.trim() != "-" {
                    return Some((i + 1, flag_class(&whole), value.trim().chars().count()));
                }
            }
            continue;
        }
        if let Some((lhs, rhs)) = whole.split_once('=') {
            if SECRET_FLAGS.contains(&lhs) {
                // The value is everything after the FIRST `=`; a value that
                // itself contains `=` is still one value.
                let raw = token.trim().split_once('=').map(|(_, r)| r).unwrap_or("");
                if rhs.trim() != "-" {
                    return Some((i, flag_class(lhs), raw.trim().chars().count()));
                }
                continue;
            }
        }

        // Layer 2 -- value-shape, over all four normalisations.
        for cand in argv_candidates(token) {
            if let Some(class) = material_class(&cand) {
                return Some((i, class, cand.chars().count()));
            }
        }
    }
    None
}

/// What a flag-keyed channel carries. Named from the FLAG, because layer 1
/// never looks at the value.
fn flag_class(flag: &str) -> &'static str {
    match flag {
        "--phrase" => "a BIP-39 mnemonic",
        "--hex" => "raw hex entropy",
        "--ms1" => "an ms1 string",
        _ => "a BIP-39 passphrase",
    }
}

/// The refusal. **It names the CLASS and the LENGTH, never the value** —
/// printing the value back would put the material in a SECOND public place,
/// which is the defect this message exists to name.
///
/// **The `/proc` sentence is `ms`'s, not `me`'s, and the difference is
/// measured.** `ms` already calls `prctl(PR_SET_DUMPABLE, 0)`
/// (`process_hardening::set_non_dumpable`), which makes `/proc/$PID/` unreadable
/// to other non-root UIDs and disables core dumps — so the shipped advisory's
/// unqualified *"to avoid /proc/$PID/cmdline exposure"* overstates what is open
/// here. Copying `me`'s wording would inherit that overstatement into a tool
/// that partly closed the hole. The live reasons on `ms` are shell history and
/// the same-UID process table, and those are what this says.
fn refusal(argv: &[String], index: usize, class: &str, len: usize) -> String {
    let (surface, allowlisted) = argv_surface(argv);
    let verb = surface.strip_prefix("ms ").unwrap_or("encode").to_string();
    let purge = mnemonic_io_lib::remedy::history_purge_block(&surface);
    let breadth = if allowlisted {
        String::new()
    } else {
        format!(
            "      (The first word of this invocation is not one of `ms`'s own \
             subcommands, so the pattern above is the bare `{surface}` and matches \
             BROADLY -- it will remove unrelated lines that merely contain it. That \
             is the deliberate direction to err in: over-matching costs you history \
             lines, under-matching leaves the material on disk behind a `sed` that \
             exited 0.)\n      "
        )
    };
    format!(
        "argument {index} on ARGV (arguments count from 0, and 0 is `ms` itself) \
         is {class}, {len} characters long.\n      \
         Refused BEFORE the command line was parsed; nothing was read and nothing \
         was written.\n      \
         `ms` already sets PR_SET_DUMPABLE to 0, so /proc/$PID/ is closed to OTHER \
         UIDs and core dumps are off -- but that is not the whole hole. Your own \
         UID and root can still read it, `ps` shows it, and your shell has ALREADY \
         written the line to its history.\n      \
         Use a private channel instead:\n      \
         \x20   ms {verb} --in FILE      # read it from a file\n      \
         \x20   ms {verb} -              # or pipe it on stdin\n      \
         A seed phrase AND a passphrase together take two commands, because \
         `--in` on `derive` reads an ms1:\n      \
         \x20   ms encode --in seed.txt --out card.ms1\n      \
         \x20   ms derive --in card.ms1 --passphrase-stdin < pass.txt\n\n      \
         {purge}\n      \
         {breadth}\
         If argv is safe where you are -- a single-user air-gapped box, an \
         amnesic Tails session -- `--allow-argv-secret` proceeds."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(words: &[&str]) -> Vec<String> {
        std::iter::once("ms".to_string())
            .chain(words.iter().map(|s| s.to_string()))
            .collect()
    }

    #[test]
    fn a_filename_that_merely_starts_with_the_hrp_is_not_material() {
        // The near-miss control, as a unit test so the reason is next to the
        // rule: `-` and `.` are outside the bech32 charset.
        assert!(!is_ms1_shaped("ms1-2026-08-23-backup.txt"));
        // ...and the grouped spelling of a real card IS material, because that
        // is what `ms` itself ingests.
        assert!(is_ms1_shaped(
            "ms10e ntrsq qqqqq qqqqq qqqqq qqqqq qqqqq qqcj9 sxraq 34v7f"
        ));
        assert!(is_ms1_shaped(
            "ms10e,ntrsq,qqqqq,qqqqq,qqqqq,qqqqq,qqqqq,qqcj9,sxraq,34v7f"
        ));
        assert!(is_ms1_shaped(
            "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f"
        ));
    }

    #[test]
    fn a_single_bip39_word_is_not_a_phrase() {
        // `combine` is a BIP-39 word. If a single word counted, the guard would
        // refuse `ms combine` as its own subcommand.
        assert!(!is_phrase_shaped("combine"));
        assert!(is_phrase_shaped(
            "legal winner thank year wave sausage worth useful legal winner thank yellow"
        ));
    }

    #[test]
    fn a_phrase_with_a_broken_checksum_is_still_material() {
        // Membership, not `parse_in`. This phrase has 12 wordlist words and a
        // checksum that does not validate; it is still the operator's seed,
        // one character wrong, sitting in /proc.
        let broken = "legal winner thank year wave sausage worth useful legal winner thank zoo";
        assert!(
            bip39::Mnemonic::parse(broken).is_err(),
            "control: it really does not parse"
        );
        assert!(is_phrase_shaped(broken));
    }

    #[test]
    fn the_surface_falls_back_to_bare_ms_for_a_mistyped_verb() {
        assert_eq!(argv_surface(&argv(&["encode"])), ("ms encode".into(), true));
        assert_eq!(argv_surface(&argv(&["encoed"])), ("ms".into(), false));
        assert_eq!(argv_surface(&argv(&[])), ("ms".into(), false));
    }

    #[test]
    fn the_equals_joined_spelling_is_reached() {
        // F-302. Neither layer is scoped to look at `--phrase=<seed>` without
        // the `=`-split, because the whole token is not the flag string.
        let seed = "legal winner thank year wave sausage worth useful legal winner thank yellow";
        assert!(find_argv_material(&argv(&["encode", &format!("--phrase={seed}")])).is_some());
        assert!(find_argv_material(&argv(&["encode", "--phrase", seed])).is_some());
        assert!(find_argv_material(&argv(&["encode", "--phrase=-"])).is_none());
        assert!(find_argv_material(&argv(&["encode", "--phrase", "-"])).is_none());
    }

    #[test]
    fn passphrase_stdin_is_not_the_passphrase_flag() {
        // Equality, not a prefix test.
        assert!(find_argv_material(&argv(&["derive", "-", "--passphrase-stdin"])).is_none());
    }

    #[test]
    fn the_refusal_never_reproduces_the_value() {
        let seed = "legal winner thank year wave sausage worth useful legal winner thank yellow";
        let msg = argv_secret_guard(&argv(&["encode", "--phrase", seed])).unwrap();
        for word in seed.split_whitespace() {
            assert!(
                !msg.to_lowercase().contains(word),
                "the refusal reproduced `{word}` -- it must name the CLASS and the \
                 LENGTH and nothing else:\n{msg}"
            );
        }
        assert!(msg.contains(&format!("{} characters long", seed.chars().count())));
    }
}
