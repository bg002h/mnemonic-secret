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
    digest, preimage_hardened, preimage_random, preimage_sha256, HASHLOCK_DKLEN,
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
                Payload::Preimage(x) => Ok(Derived { x, method: None, phrase_chars: None, source: "preimage supplied (ms1 plate)" }),
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
    let is_random = matches!(source, SourceKind::Random);
    let d = derive(&args, source)?;
    let h = digest(&d.x);
    let record = format!("hash:{}", hex(&h));
    let ms1 = ms_codec::encode(Tag::HASH, &Payload::Preimage(d.x.clone()))?;

    if let Some(path) = args.out.as_deref() {
        if is_random {
            crate::out::write_artifact_create_new(path, &format!("{ms1}\n"))?;
        } else {
            crate::out::write_artifact(path, &format!("{ms1}\n"))?;
        }
    }

    let mut stdout = std::io::stdout().lock();
    if args.json {
        let mut o = serde_json::Map::new();
        o.insert("digest".into(), hex(&h).into());
        o.insert("hash_record".into(), record.clone().into());
        o.insert(
            "sha256_operand".into(),
            format!("sha256={}", hex(&h)).into(),
        );
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
        writeln!(stderr, "digest:          {}", hex(&h)).ok();
        writeln!(stderr, "for md compose:  --path ... sha256={}", hex(&h)).ok();
        writeln!(stderr, "preimage (ms1):  {grouped}").ok();
        writeln!(stderr, "preimage (hex):  {}", hex(&d.x[..])).ok();
        writeln!(stderr, "method:          {}", method_line(&d)).ok();
        if let Some(n) = d.phrase_chars {
            writeln!(stderr, "phrase:          {n} characters -- write the method line next to your phrase; it is on no plate; if the method line is lost, try each method that shipped with the version named on this card (ms-cli {})", env!("CARGO_PKG_VERSION")).ok();
        }
        writeln!(stderr, "The preimage must be exactly 32 bytes (64 hex characters): the script checks OP_SIZE 32 before OP_SHA256 (composer spec §8i, F-132).").ok();
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
    if args.json {
        emit_output_class_advisory(OutputClass::PrivateKeyMaterial, &mut stderr);
    }
    Ok(0)
}
