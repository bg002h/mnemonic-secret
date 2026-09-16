//! `ms hashlock` (SPEC_ms_hashlock §4): derive or take a 32-byte preimage,
//! print the `hash:` record, back the preimage up as a plate string.
//!
//! THE POLARITY IS INVERTED HERE and the verb says so on stderr's first line:
//! stdout carries the PUBLIC digest record (`me sysw pack` reads it), stderr
//! carries the SECRET preimage on the card, `--out` carries it to a 0600
//! file, `--json` carries it in one object on stdout in place of the record.
//!
//! EXACTLY ONE SOURCE. Zero and two-or-more both exit 64 -- zero must not
//! default to stdin, or a bare `ms hashlock` at a terminal blocks with no
//! prompt and the phrase an operator then types lands in an ms1 parse error.
//!
//! `--random` REQUIRES `--out FILE` (`--json` is stdout, which `| jq` filters
//! away -- the constructed loss of R0 r0 adversarial C-1) and its `--out`
//! never overwrites (a random preimage is a function of nothing, so a
//! clobbered file cannot be re-made: adversarial C-2).

use std::io::Write;
use std::path::PathBuf;

use clap::{Args, ValueEnum};
use ms_codec::hashlock::{
    preimage_hardened, preimage_random, preimage_sha256, HashKind, HASHLOCK_DKLEN,
    HASHLOCK_ITERATIONS, HASHLOCK_SALT,
};
use ms_codec::{Payload, Tag};
use zeroize::Zeroizing;

use crate::advisory::{emit_output_class_advisory, OutputClass};
use crate::error::{CliError, Result};
use crate::hashlock_phrase::{read_phrase_stdin, validate_phrase};
use crate::parse::{read_input, Source};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Method {
    /// PBKDF2-HMAC-SHA256, salt "ms-hashlock-v1", 100000 iterations (default).
    Hardened,
    /// One SHA-256 of the phrase bytes -- the brainwallet construction.
    Sha256,
}

/// `ms hashlock` arguments.
#[derive(Args, Debug)]
pub struct HashlockArgs {
    /// The hashlock phrase, on argv. A SECRET channel: refused unless --allow-argv-secret.
    #[arg(long, value_name = "TEXT")]
    pub hashlock_phrase: Option<String>,
    /// Read the hashlock phrase from stdin, byte-verbatim (one trailing newline stripped).
    #[arg(long)]
    pub hashlock_phrase_stdin: bool,
    /// Which hash the SCRIPT commits to: sha256, hash256, ripemd160, hash160.
    /// Omit it and every kind's digest is listed on stderr (under --json, in
    /// the object as `digests_by_kind`, with `kind_specified: false`) -- a
    /// plate cut before this existed carries no kind, and a lookup beats an
    /// impossible check. Case is rejected, never folded.
    #[arg(long, value_name = "KIND", value_parser = parse_kind)]
    pub kind: Option<HashKind>,
    /// An existing preimage: exactly 32 bytes (64 hex characters). `-` reads stdin.
    #[arg(long, value_name = "HEX")]
    pub hex: Option<String>,
    /// A preimage-kind ms1 string, to re-derive the digest from a plate. `-` reads stdin.
    #[arg(value_name = "MS1")]
    pub ms1: Option<String>,
    /// Read the ms1 string from FILE (the six reading verbs' meaning of --in).
    #[arg(long = "in", value_name = "FILE")]
    pub in_path: Option<PathBuf>,
    /// 32 bytes from the OS random source. Requires --out FILE.
    #[arg(long)]
    pub random: bool,
    /// Phrase -> preimage method (phrase sources only).
    #[arg(long, value_enum)]
    pub method: Option<Method>,
    /// Write the preimage ms1 string here, owner-only. Never suppresses stdout.
    #[arg(long, value_name = "FILE")]
    pub out: Option<PathBuf>,
    /// One JSON object on stdout in place of the record line. Carries the secret.
    #[arg(long)]
    pub json: bool,
    /// Suppress the stderr card.
    #[arg(long)]
    pub no_engraving_card: bool,
    /// Also print a `phrase:` record for `me sysw pack --pack-preimage`, so a
    /// PHRASE-form plate can be cut from this phrase without hand-encoding the
    /// hex. Requires a phrase source: `--hex` and `--random` have no phrase to
    /// record, and asking for one there is a usage error rather than a silent
    /// omission. THE RECORD CARRIES THE PHRASE, so it goes on the card beside
    /// the preimage and never on stdout, which carries only the public digest.
    #[arg(long)]
    pub emit_record: bool,
    /// Group the ms1 on the card every N characters (0 = no grouping).
    /// `u16` and the same default as `ms encode` / `ms split`, so the same
    /// value is accepted by every verb that renders a grouped ms1 (review N-1).
    #[arg(long, default_value_t = 5)]
    pub group_size: u16,
    /// Separator: space|hyphen|comma (keyword) or the literal " "|-|, . SPEC §5.
    ///
    /// BOUND TO THE SHARED PARSER, like `ms encode` and `ms split`. Unbound, a
    /// separator inside the codex32 charset (`--separator q`) produced a card
    /// whose "plate string" `strip_display_separators` cannot clean up, so the
    /// engraved 90-character result is one `ms` itself refuses (review I-2).
    #[arg(long, default_value = "space", value_parser = crate::format::parse_separator)]
    pub separator: char,
    /// Admit a secret on argv (see `ms encode --help`).
    #[arg(long)]
    pub allow_argv_secret: bool,
    /// Confirm a phrase that LOOKS LIKE a digest in hex (F-539).
    ///
    /// A phrase of exactly 40 or 64 hex characters is very likely a digest
    /// someone means to commit to, not a phrase they mean to hash -- and
    /// hashing it commits the wallet to the ASCII of the digest instead, a
    /// preimage they do not hold. Without this flag that case WARNS AND STOPS;
    /// with it, it proceeds.
    ///
    /// It is not a refusal: an operator who really chose an all-hex phrase can
    /// still use it (operator ruling 2026-09-16). What they cannot do is walk
    /// into it silently.
    #[arg(long)]
    pub phrase_looks_like_digest_ok: bool,
}

enum SourceKind {
    Phrase { argv: bool },
    Hex,
    Ms1,
    Random,
}

impl SourceKind {
    fn name(&self) -> &'static str {
        match self {
            SourceKind::Phrase { argv: true } => "--hashlock-phrase",
            SourceKind::Phrase { argv: false } => "--hashlock-phrase-stdin",
            SourceKind::Hex => "--hex",
            SourceKind::Ms1 => "an ms1 string (argument, `-`, or --in FILE)",
            SourceKind::Random => "--random",
        }
    }
}

const FIVE_SOURCES: &str = "exactly one source: --hashlock-phrase TEXT, --hashlock-phrase-stdin, --hex HEX, an ms1 string (argument, `-`, or --in FILE), or --random";

/// Parse the `--kind` token. Fully-qualified `core::result::Result` because this
/// crate defines a ONE-parameter `Result` alias; a bare `Result<_, _>` is E0107.
fn parse_kind(s: &str) -> core::result::Result<HashKind, String> {
    match s {
        "sha256" => Ok(HashKind::Sha256),
        "hash256" => Ok(HashKind::Hash256),
        "ripemd160" => Ok(HashKind::Ripemd160),
        "hash160" => Ok(HashKind::Hash160),
        other => Err(format!(
            "unknown hash kind {other:?}: expected sha256, hash256, ripemd160 or hash160 \
             (lowercase; case is rejected, never folded)"
        )),
    }
}

fn pick_source(args: &HashlockArgs) -> Result<SourceKind> {
    let mut chosen: Vec<SourceKind> = Vec::new();
    if args.hashlock_phrase.is_some() || crate::argv_guard::admitted("--hashlock-phrase").is_some()
    {
        chosen.push(SourceKind::Phrase { argv: true });
    }
    if args.hashlock_phrase_stdin {
        chosen.push(SourceKind::Phrase { argv: false });
    }
    if args.hex.is_some() || crate::argv_guard::admitted("--hex").is_some() {
        chosen.push(SourceKind::Hex);
    }
    if args.ms1.is_some()
        || args.in_path.is_some()
        || crate::argv_guard::admitted(crate::argv_guard::CH_POSITIONAL).is_some()
    {
        chosen.push(SourceKind::Ms1);
    }
    if args.random {
        chosen.push(SourceKind::Random);
    }
    match chosen.len() {
        1 => Ok(chosen.pop().unwrap()),
        0 => Err(CliError::Usage(format!("no source given; {FIVE_SOURCES}"))),
        _ => Err(CliError::Usage(format!(
            "{} and {} were both given; {FIVE_SOURCES}",
            chosen[0].name(),
            chosen[1].name()
        ))),
    }
}

/// The resolved preimage plus what the card must say about where it came from.
struct Derived {
    x: Zeroizing<[u8; 32]>,
    method: Option<Method>,
    phrase_chars: Option<usize>,
    source: &'static str,
    /// The phrase itself, retained ONLY for `--emit-record` (F-495) and
    /// `Zeroizing` like every other phrase-bearing value here. Without the flag
    /// this stays `None`, so the default path drops the phrase exactly where it
    /// always did — at the end of `derive` — rather than carrying it through
    /// `run` for a caller that will not use it.
    phrase: Option<Zeroizing<Vec<u8>>>,
}

fn derive(args: &HashlockArgs, source: SourceKind) -> Result<Derived> {
    match source {
        SourceKind::Phrase { argv } => {
            let bytes: Zeroizing<Vec<u8>> = if argv {
                // The admitted side channel wins over the (already rewritten)
                // argv value; the guard replaced the argv token with `-`.
                match crate::argv_guard::admitted("--hashlock-phrase") {
                    Some([first, ..]) => Zeroizing::new(first.as_bytes().to_vec()),
                    // A bare `-` here is NOT admitted material (the guard passes
                    // it through untouched) and would otherwise derive from the
                    // one-byte phrase "-". Every other secret flag treats `-` as
                    // stdin; this verb has a dedicated flag for that, so name it.
                    // CONTROLLER DEFAULT (spec §4.1 is silent; R0 r0 fidelity I-10).
                    _ if args.hashlock_phrase.as_deref() == Some("-") => {
                        return Err(CliError::Usage(
                            "--hashlock-phrase - is not a channel; to read the phrase from stdin use --hashlock-phrase-stdin".to_string(),
                        ))
                    }
                    _ => Zeroizing::new(
                        args.hashlock_phrase
                            .as_deref()
                            .unwrap_or("")
                            .as_bytes()
                            .to_vec(),
                    ),
                }
            } else {
                read_phrase_stdin()?
            };
            validate_phrase(&bytes)?;
            // F-539: WARN AND REQUIRE CONFIRMATION, never a blanket refusal.
            // The message names what is about to happen rather than the rule
            // it broke -- "this is a digest, not a phrase" is a claim about
            // the operator's intent, and the machine cannot know it.
            if let Some(chars) = ms_codec::hashlock::looks_like_digest(&bytes)
                .filter(|_| !args.phrase_looks_like_digest_ok)
            {
                return Err(CliError::BadInput(format!(
                    "that phrase is {chars} hex characters, the width of a digest. \
                     Hashing it commits the wallet to the ASCII of those characters, \
                     NOT to the digest they spell -- so if you meant to use a digest \
                     you already hold, pass it with --hex instead. If you really meant \
                     this as a phrase, re-run with --phrase-looks-like-digest-ok."
                )));
            }
            let method = args.method.unwrap_or(Method::Hardened);
            let x = match method {
                Method::Hardened => preimage_hardened(&bytes),
                Method::Sha256 => preimage_sha256(&bytes),
            };
            Ok(Derived {
                x,
                method: Some(method),
                phrase_chars: Some(bytes.len()),
                source: if argv {
                    "phrase (argv, admitted)"
                } else {
                    "phrase (stdin)"
                },
                phrase: args.emit_record.then(|| bytes.clone()),
            })
        }
        SourceKind::Hex => {
            refuse_method(args)?;
            let raw = read_input(Source::new(args.hex.as_deref(), None).on("--hex"))?;
            // Parsed HERE, not by `parse_hex_entropy`: that helper speaks for
            // `ms encode` ("expected hex of length 32/40/48/56/64 chars"), a
            // set that is wrong for this verb, and it fails before any length
            // check could name §8i (R0 r0 fidelity I-9). The predicate is the
            // `hex` crate's, the same one the phrase rule's 64-hex guard uses.
            let s = raw.trim();
            let refuse = |got: usize| {
                CliError::BadInput(format!(
                    "--hex is {got} characters; a hashlock preimage is exactly 32 bytes (64 hex characters) -- see the composer spec's §8i"
                ))
            };
            if s.len() != 64 {
                return Err(refuse(s.len()));
            }
            let bytes = hex::decode(s).map_err(|_| {
                CliError::BadInput(
                    "--hex is not hex; a hashlock preimage is exactly 32 bytes (64 hex characters) -- see the composer spec's §8i".to_string(),
                )
            })?;
            let mut x = Zeroizing::new([0u8; 32]);
            x.copy_from_slice(&bytes);
            Ok(Derived {
                x,
                method: None,
                phrase_chars: None,
                source: "preimage supplied (--hex)",
                phrase: None,
            })
        }
        SourceKind::Ms1 => {
            refuse_method(args)?;
            let s = read_input(
                Source::new(args.ms1.as_deref(), args.in_path.as_deref())
                    .on(crate::argv_guard::CH_POSITIONAL),
            )?;
            let (_tag, payload) = ms_codec::decode(&s)?;
            match payload {
                Payload::Preimage(x) => Ok(Derived { x, method: None, phrase_chars: None, source: "preimage supplied (ms1 plate)",
                    phrase: None }),
                _ => Err(CliError::BadInput(
                    "that is a seed backup, not a hashlock preimage; a preimage plate reads ms10hash... (32 bytes, 64 hex characters)".to_string(),
                )),
            }
        }
        SourceKind::Random => {
            refuse_method(args)?;
            if args.out.is_none() {
                return Err(CliError::Usage(
                    "--random needs --out FILE: a preimage that reaches no file is data loss (--json is stdout and does not count)".to_string(),
                ));
            }
            let x = preimage_random()?;
            Ok(Derived {
                x,
                method: None,
                phrase_chars: None,
                source: "random (OS CSPRNG)",
                phrase: None,
            })
        }
    }
}

fn refuse_method(args: &HashlockArgs) -> Result<()> {
    if args.method.is_some() {
        return Err(CliError::Usage(
            "--method applies to the phrase sources only; with --hex, --random or an ms1 string the preimage is already given".to_string(),
        ));
    }
    Ok(())
}

fn hex(b: &[u8]) -> String {
    use std::fmt::Write;
    b.iter()
        .fold(String::with_capacity(b.len() * 2), |mut s, x| {
            let _ = write!(s, "{x:02x}");
            s
        })
}

fn method_line(d: &Derived) -> String {
    match d.method {
        Some(Method::Hardened) => format!(
            "preimage = PBKDF2-HMAC-SHA256(password = phrase, salt = \"{}\", iterations = {HASHLOCK_ITERATIONS}, dkLen = {HASHLOCK_DKLEN})",
            String::from_utf8_lossy(HASHLOCK_SALT)
        ),
        Some(Method::Sha256) => "preimage = SHA-256(phrase)".to_string(),
        None => "preimage supplied".to_string(),
    }
}

pub fn run(args: HashlockArgs) -> Result<u8> {
    let source = pick_source(&args)?;
    // Refused here rather than ignored: a flag that asks for a record and
    // silently produces none sends the operator looking for output that was
    // never going to come (F-495).
    if args.emit_record && !matches!(source, SourceKind::Phrase { .. }) {
        return Err(CliError::Usage(
            "--emit-record needs a phrase: the `phrase:` record carries the phrase and the \
             method that derives its preimage, and --hex, --random and an ms1 input have no \
             phrase to record. Cut a PREIMAGE plate from the ms1 string instead."
                .into(),
        ));
    }
    let is_random = matches!(source, SourceKind::Random);
    let d = derive(&args, source)?;
    let kind = args.kind.unwrap_or(HashKind::Sha256);
    let hb = kind.digest(&d.x);
    let h = hb.as_slice();
    // SPEC S6's producer rule: BARE for sha256, explicit for the other three.
    // A bare record MEANS sha256, so emitting one for another kind hands
    // `me sysw pack` a digest it will read as sha256 -- the scheme S6 names as
    // rejected, because it composes a wallet nobody can spend.
    let record = if kind == HashKind::Sha256 {
        format!("hash:{}", hex(h))
    } else {
        format!("hash:{}:{}", kind.token(), hex(h))
    };
    let ms1 = ms_codec::encode(Tag::HASH, &Payload::Preimage(d.x.clone()))?;

    if let Some(path) = args.out.as_deref() {
        if is_random {
            crate::out::write_artifact_create_new(path, &format!("{ms1}\n"))?;
        } else {
            crate::out::write_artifact(path, &format!("{ms1}\n"))?;
        }
    }

    // F-495: the `phrase:` record `me sysw pack --pack-preimage` admits, so a
    // PHRASE-form plate can be cut without hand-encoding hex. The wire form
    // belongs to mnemonic-engrave (`sysw::composer_records::phrase_record`);
    // this composes the same two fields and `tests/hashlock_emit_record.rs`
    // pins the bytes against that repo's committed corpus rows, so the two
    // spellings cannot drift without this suite going red.
    let phrase_record: Option<Zeroizing<String>> = match (&d.phrase, d.method) {
        (Some(ph), Some(m)) => {
            let method = match m {
                Method::Hardened => "hardened",
                Method::Sha256 => "sha256",
            };
            let mut body = Zeroizing::new(String::with_capacity(
                "phrase:".len() + (method.len() + 1 + ph.len()) * 2,
            ));
            body.push_str("phrase:");
            for b in method.as_bytes().iter().chain(b",").chain(ph.iter()) {
                use std::fmt::Write as _;
                write!(&mut *body, "{b:02x}").ok();
            }
            Some(body)
        }
        _ => None,
    };

    let mut stdout = std::io::stdout().lock();
    if args.json {
        let mut o = serde_json::Map::new();
        o.insert("digest".into(), hex(h).into());
        o.insert("hash_record".into(), record.clone().into());
        o.insert(
            "hash_operand".into(),
            format!("{}={}", kind.token(), hex(h)).into(),
        );
        o.insert("kind".into(), kind.token().into());
        if args.kind.is_none() {
            // SPEC §13.4 for MACHINE consumers. Under `--json
            // --no-engraving-card` -- and ONLY that pair -- stderr is pinned to
            // exactly the advisory (§4.4, §11), so the stderr listing stands
            // down there and the notice travels in this object instead. Under
            // `--json` with the card it does BOTH, because the human is reading
            // the card. (This comment said "suppressed under --json" for one
            // round, which was the guard being one flag wider than its
            // contract -- R0 round 5, I-1.) A consumer reading `hash_operand`
            // alone would otherwise take the sha256 default for a stated
            // choice.
            o.insert("kind_specified".into(), false.into());
            let mut by = serde_json::Map::new();
            for k in [
                HashKind::Sha256,
                HashKind::Hash256,
                HashKind::Ripemd160,
                HashKind::Hash160,
            ] {
                by.insert(k.token().into(), hex(k.digest(&d.x).as_slice()).into());
            }
            o.insert("digests_by_kind".into(), by.into());
        }
        o.insert("preimage_hex".into(), hex(&d.x[..]).into());
        o.insert("preimage_ms1".into(), ms1.clone().into());
        o.insert("source".into(), d.source.into());
        match d.method {
            Some(Method::Hardened) => {
                o.insert("method".into(), serde_json::json!({"kdf": "PBKDF2-HMAC-SHA256", "hash": "SHA-256", "salt": String::from_utf8_lossy(HASHLOCK_SALT), "iterations": HASHLOCK_ITERATIONS, "dklen": HASHLOCK_DKLEN}));
            }
            Some(Method::Sha256) => {
                o.insert("method".into(), serde_json::json!({"hash": "SHA-256"}));
            }
            None => {}
        }
        if let Some(n) = d.phrase_chars {
            o.insert("phrase_chars".into(), (n as u64).into());
        }
        // Only under --emit-record, and this object already announces that it
        // carries the secret; the record carries the phrase verbatim.
        if let Some(r) = phrase_record.as_deref() {
            o.insert("phrase_record".into(), r.as_str().into());
        }
        writeln!(stdout, "{}", serde_json::Value::Object(o)).ok();
    } else {
        writeln!(stdout, "{record}").ok();
    }
    drop(stdout);

    let mut stderr = std::io::stderr().lock();
    if !args.no_engraving_card {
        let grouped = crate::format::render_grouped(&ms1, args.group_size as usize, args.separator);
        writeln!(
            stderr,
            "THIS CARD CARRIES THE PREIMAGE -- the secret. stdout carries only the public digest."
        )
        .ok();
        writeln!(stderr, "digest:          {}", hex(h)).ok();
        // SPEC §11: the ONLY line keeping the digest function and `md compose`'s
        // option name in agreement. It shipped once saying `sha256=` under every
        // kind, which tells the operator to build a wallet whose hashlock nobody
        // can satisfy.
        writeln!(
            stderr,
            "for md compose:  --path ... {}={}",
            kind.token(),
            hex(h)
        )
        .ok();
        if args.kind.is_some_and(|k| k != HashKind::Sha256) {
            // The line above proposes a command fragment with full confidence.
            // `md compose` refuses a non-sha256 operand until phase 1 ships, and
            // its refusal says "unknown option ripemd160" -- which reads as a
            // typo, not as "not wired up yet". Written as a CONDITIONAL so that
            // phase 1 shipping does not make this sentence false.
            writeln!(
                stderr,
                "                 requires `{0}=` support in `md compose`. If it answers \
                 \"unknown option `{0}`\", that support has not shipped in your `md` yet -- \
                 the preimage and digest above are still correct, and `md` is what has to \
                 catch up.",
                kind.token()
            )
            .ok();
        }
        writeln!(stderr, "preimage (ms1):  {grouped}").ok();
        writeln!(stderr, "preimage (hex):  {}", hex(&d.x[..])).ok();
        writeln!(stderr, "method:          {}", method_line(&d)).ok();
        if let Some(r) = phrase_record.as_deref() {
            writeln!(stderr, "record (phrase): {r}").ok();
            writeln!(
                stderr,
                "                 THIS RECORD CARRIES THE PHRASE. Feed it to `me sysw pack --pack-preimage` \
                 to cut a HASHLOCK PHRASE plate; treat the file you put it in like the phrase itself."
            )
            .ok();
        }
        if let Some(n) = d.phrase_chars {
            writeln!(stderr, "phrase:          {n} characters -- write the method line AND the hash line ({}) next to your phrase unless the phrase is cut on a HASHLOCK PHRASE plate, which carries both; if the method line is lost, try each method that shipped with the version named on this card (ms-cli {}), and if the hash line is lost, re-run with no --kind and match the digest against your descriptor", kind.token(), env!("CARGO_PKG_VERSION")).ok();
        }
        // The OPCODE VARIES WITH THE KIND and the 32 does not (spec §3 F1:
        // `OP_SIZE <32> OP_EQUALVERIFY <hashop> <h> OP_EQUAL`). This line said
        // OP_SHA256 under every kind for one release -- a false statement about
        // the object in the operator's hand, in the line they would self-check
        // against.
        writeln!(stderr, "The preimage must be exactly 32 bytes (64 hex characters) for every kind: the script checks OP_SIZE 32 before {} (composer spec §8i, F-132).", kind.opcode()).ok();
        writeln!(stderr, "One phrase per policy. Spending any path of a wsh wallet publishes this digest. Never use this phrase as a passphrase or a password anywhere else -- a spend publishes the preimage, and anyone can then test guesses at the phrase itself.").ok();
        match d.method {
            Some(Method::Sha256) => {
                writeln!(stderr, "WARNING: This is the brainwallet construction: anyone holding the digest tests 10^10 phrases per second. A phrase a person chose is not safe here; use six diceware words or --random.").ok();
            }
            Some(Method::Hardened) => {
                if d.phrase_chars.unwrap_or(0) < 20 {
                    writeln!(stderr, "WARNING: a 20-character phrase falls in about 72 days on one GPU; choose it from a generator.").ok();
                }
            }
            None => {}
        }
        if d.source.starts_with("preimage supplied (--hex)") {
            writeln!(stderr, "WARNING: the first spend of this hash path publishes these 32 bytes in the clear, forever. If this value is also anything else's secret -- a seed's entropy, a key -- every use of that secret is public with it.").ok();
        }
        if is_random {
            writeln!(stderr, "No phrase exists, so nothing can be guessed, and nothing can be remembered. The file you just wrote is the only copy until you cut the plate.").ok();
        }
        writeln!(stderr, "source:          {}", d.source).ok();
    }
    // ─── NOTICES THAT SURVIVE --no-engraving-card ──────────────────────────
    //
    // THE RULE, and it has TWO clauses. Everything above this line is THE CARD
    // -- what an operator transcribes or engraves -- and `--no-engraving-card`
    // may suppress it. Everything below is a NOTICE about a hazard in what was
    // just emitted, and suppressing the card is not consent to lose it. A new
    // line goes BELOW unless it is genuinely part of the card.
    //
    // SECOND CLAUSE, AND IT IS NOT OPTIONAL: every notice below still carries
    // `!(args.json && args.no_engraving_card)`. That PAIR is a machine-output
    // purity contract -- `hashlock_outputs.rs::json_both_variants` pins stderr
    // to exactly the PrivateKeyMaterial advisory there -- and a notice does not
    // get to break it. The machine-readable form of the same fact travels in
    // the `--json` object instead (`kind`, `kind_specified`, `digests_by_kind`).
    //
    // Reading clause one as licence to drop clause two is R0 round 8's I-1: the
    // hash256 notice shipped UNGUARDED for one round and put 485 bytes on a
    // stream pinned to one line.
    //
    // This boundary exists because the same mistake happened THREE TIMES in one
    // cycle -- R0 r4 C-1 (the §13.4 listing nested in the guard), r5 I-1 (the
    // guard one flag wider than its contract), r7 I-1 (the hash256 warning
    // nested in the guard). Three occurrences of one mistake is a wrong shape,
    // not three slips: before this line there was nothing in the code saying
    // which of the two kinds of thing a new `writeln!` was.
    //
    // NOT YET MOVED, and tracked as F-536: the three pre-existing `WARNING:`
    // lines above (brainwallet, short phrase, --hex publishes the preimage) are
    // hazard notices by this rule and are still card-scoped. Moving them is a
    // behaviour change no review found, so it is filed rather than folded here.

    // A NOTICE, not a card line: it is about the RECORD ON STDOUT, which ships
    // whether or not `--no-engraving-card` suppressed the card. Its twin above
    // is about the card's own `for md compose:` line and correctly goes with it.
    // Keeping both halves on the card would have lost this one on exactly the
    // piping path it describes (R0 round 8, M-3).
    if args.kind.is_some_and(|k| k != HashKind::Sha256) && !(args.json && args.no_engraving_card) {
        writeln!(
            stderr,
            "the record on stdout needs `{0}` support in `me sysw pack`. If it asks for \
             \"exactly 64 hex characters\", that support has not shipped in your `me` yet -- \
             the record is correct and `me` is what has to catch up. Do not reshape the \
             record to satisfy it.",
            kind.token()
        )
        .ok();
        // THE THIRD CONSUMER, and it is the one that stays unready longest.
        //
        // The two lines above were written as conditionals so that md and me
        // shipping could not FALSIFY them -- and it worked, they are still
        // true. What it did not prevent is the list becoming INCOMPLETE: both
        // are now satisfiable, so an operator runs the list to exhaustion,
        // comes up clean, and the consumer that is NOT ready was never on it.
        // A caveat list that can be exhausted turns silence into assent
        // (P3 journey walk, J-2).
        //
        // Phrased about the FIRMWARE rather than a release, because a device
        // never flashed stays unable to read the tag forever -- "not yet"
        // would expire into a lie while the condition still holds.
        writeln!(
            stderr,
            "...and the SeedHammer II needs firmware with hashlock-kind \
             support to read a `{0}` record at all. Firmware without it counts the record \
             in the door's \"not understood\" total and builds NO hashlock path from it -- \
             it does not misread the digest, it ignores it. Do not strip the kind tag to \
             make it parse.",
            kind.token()
        )
        .ok();
    }
    // The object's `kind` field is the machine-readable form of this hazard, so
    // nothing is lost under the pinned pair -- only moved, as with §13.4 below.
    if kind == HashKind::Hash256 && !(args.json && args.no_engraving_card) {
        // Spec §13.2's operator-facing Critical, at the moment of emission:
        // hash256 and sha256 are BOTH 64 hex, so the tag is the only thing
        // distinguishing them. `me sysw pack` refuses the tagged record with
        // "hash: must be exactly 64 hex characters" -- and obeying that
        // literally means deleting `hash256:`, which leaves a record that
        // pack ACCEPTS and `me sysw show` then labels "sha256 hashlock".
        // A funded wallet whose hashlock the plate does not satisfy.
        writeln!(
                stderr,
                "WARNING: this record's `hash256:` tag is the ONLY thing distinguishing it from a \
                 sha256 record -- both digests are 64 hex. If a tool refuses it asking for \"exactly \
                 64 hex characters\", DO NOT DELETE THE TAG to satisfy it: the result is accepted as \
                 a SHA256 hashlock and the wallet it builds cannot be spent with this preimage. The \
                 tool is what needs hash256 support."
            )
            .ok();
    }

    if args.kind.is_none() && !(args.json && args.no_engraving_card) {
        // SPEC §13.4: a plate cut before --kind existed carries no kind, so
        // listing all four turns an impossible check into a lookup. What is
        // forbidden is assuming sha256 in silence -- which is what this did.
        //
        // OUTSIDE the --no-engraving-card guard ON PURPOSE. §13.4's clause is
        // unconditional, and the operator who suppresses the card is the one
        // who most needs this: the card carries the PREIMAGE, so suppressing
        // it is the safety-conscious choice, and nesting this inside it hands
        // that operator one unlabelled sha256 digest instead. It shipped that
        // way once (R0 round 4, C-1). Do not fold it back in.
        //
        // The guard is `!(json && no_engraving_card)` -- BOTH flags -- because
        // that pair, and only that pair, is what `hashlock_outputs.rs` pins to
        // exactly the PrivateKeyMaterial advisory (§4.4, §11). There the notice
        // changes channel instead, into `kind_specified` and `digests_by_kind`
        // on the object.
        //
        // It was `!args.json` alone for one round (R0 round 5, I-1), which is
        // BROADER than the contract. Under plain `--json > out.json` -- the
        // normal way to use it -- the card still prints to the terminal, its
        // `for md compose:` line still says `sha256=`, and the §13.4 notice had
        // gone into the redirected file. The human was shown one unlabelled
        // sha256 operand with nothing saying a kind was never chosen.
        writeln!(
            stderr,
            // "preimage", not "phrase": --random, --hex and the ms1-plate route have
            // no phrase at all, and on --random this line lands two lines after
            // "No phrase exists". The digest is of the PREIMAGE on every route,
            // so one word is true everywhere. (Journey walk M-1 -- the same
            // object-conflation class as R0 round 5's Critical.)
            "no --kind given; stdout carries the sha256 record. This preimage's digest under each kind:"
        )
        .ok();
        for k in [
            HashKind::Sha256,
            HashKind::Hash256,
            HashKind::Ripemd160,
            HashKind::Hash160,
        ] {
            writeln!(
                stderr,
                "  {:<10} {}",
                k.token(),
                hex(k.digest(&d.x).as_slice())
            )
            .ok();
        }
    }
    if args.json {
        emit_output_class_advisory(OutputClass::PrivateKeyMaterial, &mut stderr);
    }
    Ok(0)
}
