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
const SUBCOMMANDS: [&str; 13] = [
    "hashlock",
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

/// The five flag-keyed secret channels, as strings. No parse, no clap.
///
/// `--passphrase-stdin` is deliberately NOT here and cannot be caught by
/// accident: the match is EQUALITY, not a prefix test.
const SECRET_FLAGS: [&str; 5] = [
    "--phrase",
    "--hex",
    "--ms1",
    "--passphrase",
    "--hashlock-phrase",
];

/// Is `raw` (the value EXACTLY as clap will see it) a private channel rather
/// than material, for secret flag `flag`?
///
/// On `--passphrase` this is `passphrase_input::is_channel_value` -- the SAME
/// predicate the resolver classifies with, so the guard and the resolver
/// cannot disagree (F-687 fold 1, review M2: the guard used to trim, so
/// `--passphrase " -"` passed the guard as the stdin channel and then derived
/// with the literal passphrase " -"). Exactly `-` is stdin, `@env:VAR` the
/// environment; ` -`, `- `, `-\n` are material, as in mnemonic-toolkit.
/// On every other secret flag `-` keeps its trimmed stdin meaning.
fn is_channel(flag: &str, raw: &str) -> bool {
    if flag == "--passphrase" {
        crate::passphrase_input::is_channel_value(raw)
    } else {
        raw.trim() == "-"
    }
}

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
    // ONE predicate for the ms1 shape, shared with the phrase channels: the
    // normalisation is inside it (SPEC_ms_hashlock §4.3; R0 r0 tests C-1).
    if looks_like_ms1(candidate) {
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
/// `is_ms1_shaped` over the NORMALISED token: trimmed, lowercased, display
/// separators stripped. The one predicate both the argv guard and the phrase
/// channels call, so the two cannot drift (SPEC_ms_hashlock §4.3). An
/// uppercase plate string -- the BIP-173/QR spelling `ms decode` accepts --
/// is caught here and only here.
pub(crate) fn looks_like_ms1(raw: &str) -> bool {
    // DELEGATED to ms_codec::hashlock::looks_like_ms1 since H6, so the argv
    // guard, the phrase rule and `me sysw pack`'s `phrase:` record all read one
    // predicate -- and since R0 round 0 (fidelity M-3) there is NO second
    // spelling left in this crate, which is what §3.1's "there is still exactly
    // one implementation" says.
    //
    // The crate-local `is_ms1_shaped` this file used to keep "for the unit
    // tests" was not equal to the codec's, and the assertion its own comment
    // CLAIMED to make did not exist. Written, it fails: the local copy stripped
    // display separators without case-folding, so it answered FALSE for
    // `MS10ENTRSQ...` -- the uppercase spelling this function's doc comment
    // says is "caught here and only here". Production was always correct
    // (production calls THIS function); four unit rows were driving a
    // production-dead function that disagreed with it, and they now drive this
    // one.
    //
    // **Display separators are stripped before the test, because `ms` strips
    // them on INTAKE.** A share read off a plate arrives grouped --
    // `ms12un98 qcjj5 3dhr9 ...` -- and `read_shares`/`read_input` both remove
    // whitespace, `-` and `,` before decoding, so a guard that classified the
    // RAW token would let the grouped spelling of the very same secret through
    // while refusing the unbroken one. The codec's predicate does the same
    // stripping, and its own doc comment says so.
    ms_codec::hashlock::looks_like_ms1(raw)
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

/// The override flag. **It is a CHANNEL, not a flag** (§6d): its own parse
/// happens here, on raw argv, because it cannot be honoured by a layer that has
/// already handed the material to clap.
pub const ALLOW_FLAG: &str = "--allow-argv-secret";

/// The channel key for material that arrived positionally. `combine`'s variadic
/// shares share it, in argv order.
pub const CH_POSITIONAL: &str = "<positional>";

/// The material the override ADMITTED, keyed by channel.
///
/// **The admitted token is substituted, never removed, and never re-presented
/// to clap.** Removing it strands `encode` and `split`, whose required
/// `ArgGroup` then has no member -- measured: `ms encode` exits 64 with
/// `error: the following required arguments were not provided:` and the group's
/// own usage line, and removing only the value gives `error: a value is required
/// for '--phrase <PHRASE>'`. So the value becomes `-`, the stdin sentinel `ms`
/// already parses on every one of the fourteen channels, and the material comes
/// through here instead. `-` is not the material, so nothing §6d forbids is
/// re-presented: §6d rules only that admitted material is *"never re-presented
/// to clap as a positional"*, and `--phrase -` satisfies the group while
/// carrying nothing.
static ADMITTED: std::sync::OnceLock<std::collections::HashMap<String, Vec<String>>> =
    std::sync::OnceLock::new();

/// The material admitted on `channel`, in argv order, or `None`.
///
/// Consulted by `read_input` / `read_phrase_input` / `read_shares` **before**
/// stdin, which is what makes the substituted `-` a placeholder rather than a
/// real stdin read. The gate for that distinction is the same invocation run
/// with stdin at `/dev/null`: material that truly came from stdin cannot
/// survive it.
pub fn admitted(channel: &str) -> Option<&'static [String]> {
    ADMITTED.get()?.get(channel).map(|v| v.as_slice())
}

/// What the guard decided.
pub enum Decision {
    /// Refuse, with this message. Exit 1.
    Refuse(String),
    /// A USAGE error the guard itself detects on raw argv, with this message.
    /// Exit 64, the same code clap gives for its own. Distinct from `Refuse`
    /// because nothing secret was supplied: the operator wrote a malformed
    /// command line, not a leaky one (post-impl review I-3).
    Usage(String),
    /// Proceed, parsing THIS argv -- the original, or the substituted one.
    Proceed(Vec<String>),
}

/// **THE GUARD.** Scan raw argv, and either refuse or hand back the argv clap
/// should parse.
pub fn decide(argv: &[String]) -> Decision {
    if override_applies(argv) {
        return match substitute(argv) {
            Ok(v) => Decision::Proceed(v),
            Err(msg) => Decision::Usage(msg),
        };
    }
    match find_argv_material(argv) {
        Some((i, class, len)) => Decision::Refuse(refusal(argv, i, class, len)),
        None => Decision::Proceed(argv.to_vec()),
    }
}

/// Is this argv asking for the override, **on a surface that declares it**?
///
/// The flag is declared on the eight material verbs. On `vectors`,
/// `gui-schema` and `gen-man` it is not, and there is nothing to opt into --
/// those carry no material, so the guard has nothing to buy past. The surface
/// is the LITERAL token at `argv[1]`, not "the leading run of non-flag tokens":
/// the latter reading lets a FLAG VALUE spoof the surface, because filtering out
/// `-`-prefixed tokens keeps the value that follows one.
fn override_applies(argv: &[String]) -> bool {
    argv.iter().any(|t| t == ALLOW_FLAG)
        && matches!(
            argv.get(1).map(|t| t.trim()),
            Some("hashlock")
                | Some("encode")
                | Some("decode")
                | Some("inspect")
                | Some("verify")
                | Some("repair")
                | Some("split")
                | Some("combine")
                | Some("derive")
        )
}

/// Drop the override token, replace each material token with `-`, and seed the
/// material into [`ADMITTED`].
///
/// `Err` is a USAGE error (exit 64), not a secret refusal: see the flag-shaped
/// value check below.
fn substitute(argv: &[String]) -> std::result::Result<Vec<String>, String> {
    let mut map: std::collections::HashMap<String, Vec<String>> = Default::default();
    let mut out: Vec<String> = Vec::with_capacity(argv.len());
    let mut i = 0;
    while i < argv.len() {
        let token = argv[i].clone();
        if token == ALLOW_FLAG {
            i += 1;
            continue;
        }
        let whole = token.trim().to_ascii_lowercase();
        if i > 0 && SECRET_FLAGS.contains(&whole.as_str()) {
            if let Some(value) = argv.get(i + 1) {
                let v = value.trim();
                // A FLAG IS NOT A VALUE. Without this, an omitted value made the
                // guard swallow the next flag and admit its NAME as the secret:
                // `ms hashlock --allow-argv-secret --hashlock-phrase --json
                // --no-engraving-card` derived a preimage from the six-byte
                // string `--json` and exited 0 (post-impl review I-3). `ms
                // hashlock` is where that first became a SUCCESS, because the
                // phrase rule admits every printable-ASCII string, so nothing
                // downstream could reject it. A usage error, not a secret
                // refusal: nothing secret was given, so there is nothing to
                // protect and nothing to redact.
                //
                // A bare `-` is exempt -- it is the stdin sentinel, not a flag --
                // and `--flag=-value` is the escape hatch for a value that
                // really begins with `-` (handled by the `=` branch below).
                // F-691: on `--passphrase` the flag-shape test reads the value
                // EXACTLY as clap would (untrimmed), so `"- "` is refused here
                // (exit 64) as mnemonic-toolkit's clap refuses it, instead of
                // being admitted as the literal passphrase `"- "`. Exact `-`
                // only is the stdin channel (`passphrase_input::is_channel_value`).
                let flag_shaped = if whole == "--passphrase" {
                    value != "-" && value.starts_with('-')
                } else {
                    v != "-" && v.starts_with('-')
                };
                if flag_shaped {
                    // Review N2: show the value as TYPED on `--passphrase`
                    // (where the test is exact), so `"- "` is not reported
                    // as `"-"`.
                    let shown = if whole == "--passphrase" {
                        value.as_str()
                    } else {
                        v
                    };
                    return Err(format!(
                        "{whole} was given {shown:?}, which is a flag and not a value. \
                         Refusing rather than taking a flag's own name as the secret. \
                         If the value really begins with `-`, spell it {whole}=<value>; \
                         otherwise pass the secret on a private channel."
                    ));
                }
                if !is_channel(&whole, value) {
                    map.entry(whole.clone()).or_default().push(value.clone());
                    out.push(token);
                    out.push("-".to_string());
                    i += 2;
                    continue;
                }
            }
            // No next token at all, `-`, or an `@env:` channel: leave the
            // flag for clap (a valueless flag gets `a value is required for
            // '<flag>'` at exit 64); a channel value follows untouched.
            out.push(token);
            i += 1;
            continue;
        }
        if i > 0 {
            if let Some((lhs, _)) = whole.split_once('=') {
                if SECRET_FLAGS.contains(&lhs) {
                    let raw = token.trim().split_once('=').map(|(_, r)| r).unwrap_or("");
                    // The value exactly as clap sees it (no trim): what the
                    // resolver will classify, and what a literal passphrase is.
                    let exact = token.split_once('=').map(|(_, r)| r).unwrap_or("");
                    if !is_channel(lhs, exact) {
                        let admitted = if lhs == "--passphrase" { exact } else { raw };
                        map.entry(lhs.to_string())
                            .or_default()
                            .push(admitted.to_string());
                        out.push(format!("{lhs} -"));
                        // Two tokens, not one: `--phrase=-` also parses, but
                        // emitting the split form keeps the substituted argv in
                        // the same shape as the space-joined case.
                        out.pop();
                        out.push(lhs.to_string());
                        out.push("-".to_string());
                        i += 1;
                        continue;
                    }
                }
            }
            let is_material = argv_candidates(&token)
                .iter()
                .any(|c| material_class(c).is_some());
            if is_material {
                map.entry(CH_POSITIONAL.to_string())
                    .or_default()
                    .push(token);
                out.push("-".to_string());
                i += 1;
                continue;
            }
        }
        out.push(token);
        i += 1;
    }
    // `set` rather than `get_or_init`: one process, one substitution.
    let _ = ADMITTED.set(map);
    Ok(out)
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
                if !is_channel(&whole, value) {
                    return Some((i + 1, flag_class(&whole), value.trim().chars().count()));
                }
            }
            continue;
        }
        if let Some((lhs, _)) = whole.split_once('=') {
            if SECRET_FLAGS.contains(&lhs) {
                // The value is everything after the FIRST `=`; a value that
                // itself contains `=` is still one value.
                let raw = token.trim().split_once('=').map(|(_, r)| r).unwrap_or("");
                let exact = token.split_once('=').map(|(_, r)| r).unwrap_or("");
                if !is_channel(lhs, exact) {
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
/// The private channel that actually works for THIS verb and THIS material
/// (F-581).
///
/// The guard used to print one block for every verb -- `ms {verb} --in FILE`
/// and `ms {verb} -` -- plus a paragraph about `derive` whatever the verb was.
/// For `derive` BOTH suggested channels are wrong, measured:
///
/// ```text
/// ms derive --in seed.txt  -> error: string length 164 not in v0.1 set […]
/// ms derive - < seed.txt   -> error: string length 164 not in v0.1 set […]
/// ```
///
/// because `--in` and stdin on `derive` read an ms1, not a phrase. The guard
/// identified the verb and the material precisely and then prescribed a remedy
/// that fails for them -- and an operator who runs it, sees an error, and
/// retries on the command line has been taught to defeat the guard.
///
/// **Every line below was RUN before it was written here**, which is the whole
/// point of the finding: a prescribed remedy nobody executed is how this
/// happened.
fn private_channels(verb: &str, class: &str, value_is_ms1: bool) -> String {
    // Lines are joined explicitly rather than with `\`-continuations: the
    // continuation form put the caller's source indentation into the message.
    const PAD: &str = "\n      ";
    let hexish = class.contains("hex");
    // An ms1 on argv is read back through `--in`/stdin by EVERY verb, whatever
    // the verb's phrase or hex channel happens to be. This arm is first because
    // THE MATERIAL DECIDES, not the flag it arrived behind: a preimage PLATE
    // passed to `--hashlock-phrase` is classed "a hashlock phrase" by
    // `flag_class`, and the route that reads it is still `--in` -- verified,
    // `ms hashlock --in <plate>` prints the digest. `hashlock_phrase_rule`
    // asserts exactly that, and caught this when the first version of this
    // function keyed off `class` alone.
    if value_is_ms1 || class.contains("ms1") {
        return [
            format!("    ms {verb} --in FILE      # read the ms1 from a file"),
            format!("    ms {verb} -              # or pipe it on stdin"),
        ]
        .join("\n      ");
    }
    let lines: Vec<String> = match verb {
        // `--in`/stdin on `derive` read an ms1. The phrase has to become a card
        // first; that is two commands and there is no one-liner.
        "derive" => vec![
            "    ms encode --in seed.txt --out card.ms1   # phrase -> card, once".to_string(),
            "    ms derive --in card.ms1                   # then derive from the card".to_string(),
            "A passphrase goes on its own channel:".to_string(),
            "    ms derive --in card.ms1 --passphrase-stdin < pass.txt".to_string(),
        ],
        // `hashlock` takes a preimage or a phrase, each on its own flag.
        "hashlock" if hexish => {
            vec!["    ms hashlock --hex - < preimage.hex   # 64 hex on stdin".to_string()]
        }
        "hashlock" => vec!["    ms hashlock --hashlock-phrase-stdin < phrase.txt".to_string()],
        // encode/split take a phrase through `--in`, hex through `--hex -`.
        _ if hexish => vec![format!(
            "    ms {verb} --hex - < entropy.hex   # 64 hex on stdin"
        )],
        _ => vec![
            format!("    ms {verb} --in FILE      # read it from a file"),
            format!("    ms {verb} -              # or pipe it on stdin"),
        ],
    };
    lines.join(PAD)
}

fn flag_class(flag: &str) -> &'static str {
    match flag {
        "--phrase" => "a BIP-39 mnemonic",
        "--hex" => "raw hex entropy",
        "--ms1" => "an ms1 string",
        "--hashlock-phrase" => "a hashlock phrase",
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
    // The refused token, with any `--flag=` prefix stripped: `flag_class` names
    // the FLAG's kind, which is not always the value's.
    let value_is_ms1 = argv
        .get(index)
        .map(|s| {
            let v = s.split_once('=').map(|(_, v)| v).unwrap_or(s);
            looks_like_ms1(v.trim())
        })
        .unwrap_or(false);
    let channels = private_channels(&verb, class, value_is_ms1);
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
         {channels}\n\n      \
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

    /// Driven through `looks_like_ms1` -- the ONE predicate, the one production
    /// calls (H6 R0 round 0, fidelity M-3). These four rows used to drive a
    /// crate-local copy that answered differently for the uppercase spelling.
    #[test]
    fn a_filename_that_merely_starts_with_the_hrp_is_not_material() {
        // The near-miss control, as a unit test so the reason is next to the
        // rule: `-` and `.` are outside the bech32 charset.
        assert!(!looks_like_ms1("ms1-2026-08-23-backup.txt"));
        // ...and the grouped spelling of a real card IS material, because that
        // is what `ms` itself ingests.
        assert!(looks_like_ms1(
            "ms10e ntrsq qqqqq qqqqq qqqqq qqqqq qqqqq qqcj9 sxraq 34v7f"
        ));
        assert!(looks_like_ms1(
            "ms10e,ntrsq,qqqqq,qqqqq,qqqqq,qqqqq,qqqqq,qqcj9,sxraq,34v7f"
        ));
        assert!(looks_like_ms1(
            "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f"
        ));
        // THE UPPERCASE SPELLING, which the doc comment says is caught "here
        // and only here" and which the deleted crate-local copy answered FALSE
        // for. MUTATION: drop the codec's `to_ascii_lowercase` -> this row
        // fails and the BIP-173/QR spelling of a plate walks onto argv.
        assert!(looks_like_ms1(
            "MS10ENTRSQQQQQQQQQQQQQQQQQQQQQQQQQQQQCJ9SXRAQ34V7F"
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
        let msg = match decide(&argv(&["encode", "--phrase", seed])) {
            Decision::Refuse(m) => m,
            Decision::Proceed(_) => panic!("a seed on argv must be refused"),
            Decision::Usage(m) => panic!("a seed on argv is a REFUSAL, not a usage error: {m}"),
        };
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
