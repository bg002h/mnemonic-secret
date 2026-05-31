//! `ms derive` — read-only public derivation: the master fingerprint (always)
//! and, with `--template`, an account xpub. No master seed / root xprv / private
//! keys on stdout, no signing. The wordlist `--language` is load-bearing (the
//! seed = PBKDF2 over the language-specific mnemonic string), so the master
//! fingerprint depends on it — `ms decode`'s "DEFAULT" annotation is carried.

use std::io::Write;
use std::str::FromStr;

use bip39::Mnemonic;
use bitcoin::NetworkKind;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use clap::Args;
use ms_codec::Payload;
use zeroize::Zeroizing;

use crate::advisory::secret_in_argv_warning;
use crate::cmd::encode::parse_hex_entropy;
use crate::error::{CliError, Result};
use crate::language::CliLanguage;
use crate::parse::{is_stdin_arg, read_input, read_phrase_input};

/// `ms derive` arguments. At most one entropy source (ms1 / `--hex` / `--phrase`).
#[derive(Args, Debug)]
#[command(group = clap::ArgGroup::new("entropy_src").args(["ms1", "hex", "phrase"]))]
pub struct DeriveArgs {
    /// ms1 string. Use `-` or omit to read from stdin.
    pub ms1: Option<String>,

    /// Hex-encoded entropy (16/20/24/28/32 B), alternative to ms1.
    #[arg(long)]
    pub hex: Option<String>,

    /// BIP-39 phrase, alternative to ms1.
    #[arg(long)]
    pub phrase: Option<String>,

    /// Account-path template — emits an account xpub. Without it, only the master fingerprint.
    #[arg(long, value_enum)]
    pub template: Option<Template>,

    /// Account index (with `--template`). Default 0.
    #[arg(long, default_value_t = 0)]
    pub account: u32,

    /// Network for the account xpub serialization + coin-type. Default mainnet.
    #[arg(long, value_enum, default_value_t = Net::Mainnet)]
    pub network: Net,

    /// BIP-39 passphrase. Or `--passphrase-stdin`.
    #[arg(long)]
    pub passphrase: Option<String>,

    /// Read the BIP-39 passphrase from stdin (conflicts with `--passphrase`).
    #[arg(long, conflicts_with = "passphrase")]
    pub passphrase_stdin: bool,

    /// BIP-39 wordlist (load-bearing: forms the mnemonic → seed → fingerprint).
    /// Default english, annotated "DEFAULT" when omitted.
    #[arg(long)]
    pub language: Option<CliLanguage>,

    /// Emit a single JSON object on stdout.
    #[arg(long)]
    pub json: bool,
}

/// Single-sig account-path template.
#[derive(Copy, Clone, Debug, PartialEq, Eq, clap::ValueEnum)]
#[clap(rename_all = "lower")]
pub enum Template {
    Bip44,
    Bip49,
    Bip84,
    Bip86,
}

impl Template {
    fn purpose(self) -> u32 {
        match self {
            Template::Bip44 => 44,
            Template::Bip49 => 49,
            Template::Bip84 => 84,
            Template::Bip86 => 86,
        }
    }
}

/// Network selector (mainnet/testnet only; signet/regtest share testnet xpub bytes).
#[derive(Copy, Clone, Debug, PartialEq, Eq, clap::ValueEnum)]
#[clap(rename_all = "lower")]
pub enum Net {
    Mainnet,
    Testnet,
}

impl Net {
    fn kind(self) -> NetworkKind {
        match self {
            Net::Mainnet => NetworkKind::Main,
            Net::Testnet => NetworkKind::Test,
        }
    }
    fn coin(self) -> u32 {
        match self {
            Net::Mainnet => 0,
            Net::Testnet => 1,
        }
    }
    fn as_str(self) -> &'static str {
        match self {
            Net::Mainnet => "mainnet",
            Net::Testnet => "testnet",
        }
    }
}

/// Run `ms derive`.
pub fn run(mut args: DeriveArgs) -> Result<u8> {
    let mut stderr = std::io::stderr();

    // mem::take clap-owned secret slots → Zeroizing (scrub on drop).
    let hex_arg: Option<Zeroizing<String>> = std::mem::take(&mut args.hex).map(Zeroizing::new);
    let phrase_arg: Option<Zeroizing<String>> =
        std::mem::take(&mut args.phrase).map(Zeroizing::new);
    let passphrase_arg: Option<Zeroizing<String>> =
        std::mem::take(&mut args.passphrase).map(Zeroizing::new);

    // argv-leak advisories for inline secrets (not stdin/`-`).
    if let Some(ms1) = args.ms1.as_deref() {
        if !is_stdin_arg(Some(ms1)) {
            secret_in_argv_warning(&mut stderr, "ms1", "-");
        }
    }
    if let Some(h) = hex_arg.as_deref() {
        if h.as_str() != "-" {
            secret_in_argv_warning(&mut stderr, "--hex", "--hex -");
        }
    }
    if let Some(p) = phrase_arg.as_deref() {
        if p.as_str() != "-" {
            secret_in_argv_warning(&mut stderr, "--phrase", "--phrase -");
        }
    }
    if passphrase_arg.is_some() {
        secret_in_argv_warning(&mut stderr, "--passphrase", "--passphrase-stdin");
    }

    // Single-stdin guard: the ACTIVE entropy source + --passphrase-stdin cannot
    // both consume stdin.
    let entropy_reads_stdin = if hex_arg.is_some() {
        hex_arg.as_deref().map(|s| s.as_str()) == Some("-")
    } else if phrase_arg.is_some() {
        phrase_arg.as_deref().map(|s| s.as_str()) == Some("-")
    } else {
        is_stdin_arg(args.ms1.as_deref())
    };
    if args.passphrase_stdin && entropy_reads_stdin {
        return Err(CliError::BadInput(
            "cannot read both the entropy source and --passphrase from stdin (one stdin per invocation)".into(),
        ));
    }

    let (cli_lang, defaulted) = match args.language {
        Some(l) => (l, false),
        None => (CliLanguage::English, true),
    };
    let lang: bip39::Language = cli_lang.into();

    // Resolve the mnemonic (the seed source). ms1/hex → entropy → mnemonic;
    // --phrase → parse the phrase directly.
    let mnemonic: Mnemonic = if let Some(h) = &hex_arg {
        let hex_str = Zeroizing::new(read_input(Some(h.as_str()))?);
        let entropy = Zeroizing::new(parse_hex_entropy(&hex_str)?);
        Mnemonic::from_entropy_in(lang, &entropy[..]).map_err(CliError::Bip39)?
    } else if let Some(p) = &phrase_arg {
        let phrase = read_phrase_input(Some(p.as_str()))?;
        Mnemonic::parse_in(lang, phrase.as_str()).map_err(CliError::Bip39)?
    } else {
        let ms1 = Zeroizing::new(read_input(args.ms1.as_deref())?);
        let (_tag, payload) = ms_codec::decode(&ms1)?;
        let entropy: Zeroizing<Vec<u8>> = match payload {
            Payload::Entr(b) => Zeroizing::new(b),
            _ => unreachable!("ms-codec v0.1 decodes only Payload::Entr"),
        };
        Mnemonic::from_entropy_in(lang, &entropy[..]).map_err(CliError::Bip39)?
    };

    // BIP-39 passphrase (stdin or inline).
    let passphrase: Zeroizing<String> = if args.passphrase_stdin {
        Zeroizing::new(read_input(Some("-"))?)
    } else {
        passphrase_arg.unwrap_or_else(|| Zeroizing::new(String::new()))
    };

    // Derive (signing context required for fingerprint/derive_priv/from_priv).
    let seed: Zeroizing<[u8; 64]> = Zeroizing::new(mnemonic.to_seed(passphrase.as_str()));
    let _seed_pin = crate::mlock::pin_pages_for(&seed[..]);
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(args.network.kind(), &seed[..])
        .map_err(|e| CliError::BadInput(format!("master derive: {e}")))?;
    let master_fp = master.fingerprint(&secp);

    let account: Option<(String, String)> = if let Some(t) = args.template {
        let path = DerivationPath::from_str(&format!(
            "m/{}'/{}'/{}'",
            t.purpose(),
            args.network.coin(),
            args.account
        ))
        .map_err(|e| CliError::BadInput(format!("account path: {e}")))?;
        let acct_xpriv = master
            .derive_priv(&secp, &path)
            .map_err(|e| CliError::BadInput(format!("account derive: {e}")))?;
        let acct_xpub = Xpub::from_priv(&secp, &acct_xpriv);
        Some((format!("m/{}'/{}'/{}'", t.purpose(), args.network.coin(), args.account), acct_xpub.to_string()))
    } else {
        None
    };

    if args.json {
        let dj = crate::format::DeriveJson {
            schema_version: "1",
            master_fingerprint: master_fp.to_string(),
            network: args.network.as_str(),
            account_path: account.as_ref().map(|(p, _)| p.clone()),
            account_xpub: account.as_ref().map(|(_, x)| x.clone()),
            language: cli_lang.as_str(),
            language_defaulted: defaulted,
        };
        let s = serde_json::to_string(&dj)
            .map_err(|e| CliError::BadInput(format!("json serialization: {e}")))?;
        println!("{s}");
    } else {
        let mut stdout = std::io::stdout();
        writeln!(stdout, "master_fingerprint:  {master_fp}").ok();
        if let Some((path, xpub)) = &account {
            writeln!(stdout, "account_path:        {path}").ok();
            writeln!(stdout, "account_xpub:        {xpub}").ok();
        }
        if defaulted {
            writeln!(stdout, "language:            {} (DEFAULT)", cli_lang.as_str()).ok();
            let _ = writeln!(
                stderr,
                "note: --language defaulted to english; the master fingerprint and xpub depend on the wordlist language (record it alongside the backup)"
            );
        } else {
            writeln!(stdout, "language:            {}", cli_lang.as_str()).ok();
        }
    }
    Ok(0)
}
