//! `ms derive` — read-only public derivation: the master fingerprint (always)
//! and, with `--template`, an account xpub. No master seed / root xprv / private
//! keys on stdout, no signing. The wordlist `--language` is load-bearing (the
//! seed = PBKDF2 over the language-specific mnemonic string), so the master
//! fingerprint depends on it — `ms decode`'s "DEFAULT" annotation is carried.

use std::io::Write;
use std::str::FromStr;

use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpriv, Xpub};
use bitcoin::secp256k1::{All, Secp256k1};
use bitcoin::NetworkKind;
use clap::Args;
use zeroize::Zeroizing;

use crate::advisory::{emit_output_class_advisory, secret_in_argv_warning, OutputClass};
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

/// Binary-private, move-only newtype confining a derived `Xpriv` (root/account
/// PRIVATE key) and BEST-EFFORT byte-scrubbing it on drop. Mirrors the
/// R0-blessed `ScrubbedXpriv` shipped in
/// `mnemonic-toolkit/crates/mnemonic-toolkit/src/derive_slot.rs` (v0.70.0).
///
/// The inner `Xpriv` NEVER escapes: callers read only the PUBLIC projection
/// (`xpub` / `fingerprint`) via `&self` accessors, and derive children via the
/// `&self` `derive_priv` accessor (the parent never moves out; the returned
/// child `Xpriv` is itself re-wrapped by the caller in a fresh `ScrubbedXpriv`).
///
/// SAFETY / best-effort caveat (upstream-blocked, tracked as
/// `rust-bitcoin-xpriv-zeroize-upstream`): `bitcoin::bip32::Xpriv` is
/// `#[derive(Copy)]` (and so is its `SecretKey`), so the compiler may have
/// spilled transient bit-copies out of this newtype's reach;
/// `SecretKey::non_secure_erase` is named "non_secure" for exactly that reason.
/// This is mitigation, not a guaranteed wipe.
//
// DO NOT add Clone/Copy/into_inner/Deref<Xpriv> — re-opens the Copy-escape.
// (The absence of `Clone`/`Copy` is pinned at compile time by the
// `AmbiguousIfImpl<_>` `const _: fn()` block in `scrub_tests` below; `Copy` is
// additionally E0184-blocked by `impl Drop`.)
//
// NO `#[derive(Debug)]`: ms-cli enables bitcoin's `std` feature, so a bare
// `Xpriv` carries a `{:?}`-leaking derived `Debug`. Keeping the inner `Xpriv`
// private + NOT deriving `Debug` here REMOVES that latent leak surface
// (RULE Z-DEBUG; the same discipline the repo's
// `repair_detail_does_not_derive_debug` lint enforces).
struct ScrubbedXpriv(Xpriv);

impl ScrubbedXpriv {
    /// Take ownership of `xpriv` by value, confining it inside the move-only
    /// wrapper. The scrub runs when the returned `ScrubbedXpriv` drops.
    fn new(xpriv: Xpriv) -> Self {
        ScrubbedXpriv(xpriv)
    }

    /// Read the PUBLIC xpub. `&self`-borrowing — the inner `Xpriv` never
    /// escapes.
    fn xpub(&self, secp: &Secp256k1<All>) -> Xpub {
        Xpub::from_priv(secp, &self.0)
    }

    /// Read the master/account fingerprint. `&self`-borrowing.
    fn fingerprint(&self, secp: &Secp256k1<All>) -> Fingerprint {
        self.0.fingerprint(secp)
    }

    /// Derive a child `Xpriv` by value WITHOUT letting the parent escape. The
    /// returned child is the value the caller re-wraps in a fresh
    /// `ScrubbedXpriv` (this accessor itself does NOT scrub — it only delegates
    /// to `self.0.derive_priv`).
    fn derive_priv(
        &self,
        secp: &Secp256k1<All>,
        path: &DerivationPath,
    ) -> std::result::Result<Xpriv, bitcoin::bip32::Error> {
        self.0.derive_priv(secp, path)
    }
}

impl Drop for ScrubbedXpriv {
    fn drop(&mut self) {
        // 1) Scrub the spending secret. `SecretKey::non_secure_erase` is the
        //    upstream best-effort erase (secp256k1 0.29.x); it overwrites the
        //    32 secret bytes (to `[1u8; 32]`), destroying the key.
        self.0.private_key.non_secure_erase();
        // 2) VOLATILE zero-write over the 32 chain_code bytes. A plain
        //    assignment would be a dead store the optimizer may elide since
        //    `self` is dropping; `write_volatile` is guaranteed not elided.
        //    `ChainCode::as_mut_ptr()` (bitcoin-internals `impl_array_newtype!`)
        //    yields a `*mut u8` to the backing `[u8; 32]`.
        let cc_ptr = self.0.chain_code.as_mut_ptr();
        // SAFETY: `cc_ptr` points at a live, 32-byte, properly-aligned `[u8;
        // 32]` owned by `self.0.chain_code` (we hold `&mut self`). Each
        // in-bounds byte is written exactly once; `u8` has no invalid
        // bit-patterns and no Drop, so volatile zero-writes are sound.
        for i in 0..32 {
            unsafe {
                core::ptr::write_volatile(cc_ptr.add(i), 0u8);
            }
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

    // (cli_lang, defaulted) is the INPUT resolved from --language; it governs the
    // hex/--phrase arms (no wire byte) and is the helper input for the ms1 arm.
    // The label sites below read `effective_lang`/`effective_lang_defaulted`,
    // NOT this pair — for a mnem ms1 the WIRE language byte wins.
    let (cli_lang, defaulted) = match args.language {
        Some(l) => (l, false),
        None => (CliLanguage::English, true),
    };
    let lang: bip39::Language = cli_lang.into();

    // Resolve the mnemonic (the seed source) + the EFFECTIVE wordlist language.
    // ms1 → decode → helper (wire byte authoritative for mnem); hex/--phrase →
    // entropy/phrase under `cli_lang` (no wire byte). All three arms yield one
    // consistent `(effective_lang, effective_lang_defaulted)` for the labels.
    let (mnemonic, effective_lang, effective_lang_defaulted): (Mnemonic, CliLanguage, bool) =
        if let Some(h) = &hex_arg {
            let hex_str = Zeroizing::new(read_input(Some(h.as_str()))?);
            let entropy = Zeroizing::new(parse_hex_entropy(&hex_str)?);
            let m = Mnemonic::from_entropy_in(lang, &entropy[..]).map_err(CliError::Bip39)?;
            (m, cli_lang, defaulted)
        } else if let Some(p) = &phrase_arg {
            let phrase = read_phrase_input(Some(p.as_str()))?;
            let m = Mnemonic::parse_in(lang, phrase.as_str()).map_err(CliError::Bip39)?;
            (m, cli_lang, defaulted)
        } else {
            let ms1 = Zeroizing::new(read_input(args.ms1.as_deref())?);
            let (_tag, payload) = ms_codec::decode(&ms1)?;
            // H4: the WIRE language byte is authoritative for Payload::Mnem; a
            // --language-default fix would derive the WRONG fingerprint for a
            // non-English seed. The helper carries the disagreement note + the
            // #[non_exhaustive] guard.
            let (entropy, eff_lang, eff_defaulted): (Zeroizing<Vec<u8>>, CliLanguage, bool) =
                crate::cmd::payload_lang::payload_entropy_and_language(
                    payload,
                    cli_lang,
                    defaulted,
                    &mut stderr,
                );
            let m = Mnemonic::from_entropy_in(eff_lang.into(), &entropy[..])
                .map_err(CliError::Bip39)?;
            (m, eff_lang, eff_defaulted)
        };

    // BIP-39 passphrase (stdin or inline). C1: stdin via the byte-preserving
    // reader (NOT read_input, which strips/dedups whitespace and would mangle a
    // multi-word passphrase + disagree with the inline path).
    let passphrase: Zeroizing<String> = if args.passphrase_stdin {
        crate::parse::read_stdin_passphrase()?
    } else {
        passphrase_arg.unwrap_or_else(|| Zeroizing::new(String::new()))
    };

    // Derive (signing context required for fingerprint/derive_priv/from_priv).
    let seed: Zeroizing<[u8; 64]> = Zeroizing::new(mnemonic.to_seed(passphrase.as_str()));
    let _seed_pin = crate::mlock::pin_pages_for(&seed[..]);
    let secp = Secp256k1::new();
    // Wave-2 ms lane (in-repo leg of `ms-cli-derive-xpriv-master-not-zeroized`):
    // the derived `master`/`acct_xpriv` `Xpriv` values are confined in the
    // binary-private move-only `ScrubbedXpriv` newtype below, whose `Drop` does
    // a BEST-EFFORT byte-scrub (`SecretKey::non_secure_erase()` + a volatile
    // chain_code zero-write). `master_fp`/`acct_xpub` are materialized
    // (`.to_string()`) BEFORE either wrapper drops, so the output is byte-
    // identical and the scrub only touches post-last-use private memory.
    //
    // CAVEAT (inherent, best-effort): `bitcoin::bip32::Xpriv` is upstream
    // `#[derive(Copy)]` (and so is its `SecretKey`), so the compiler may have
    // spilled transient bit-copies we cannot reach — exactly why secp256k1
    // names its erase `non_secure_erase`. A CLEAN fix (a `Zeroize`/non-`Copy`
    // `Xpriv`) is upstream-blocked, tracked as `rust-bitcoin-xpriv-zeroize-
    // upstream`. The `seed` itself IS `Zeroizing` + mlock-pinned (above).
    let master = ScrubbedXpriv::new(
        Xpriv::new_master(args.network.kind(), &seed[..])
            .map_err(|e| CliError::BadInput(format!("master derive: {e}")))?,
    );
    let master_fp = master.fingerprint(&secp);

    let account: Option<(String, String)> = if let Some(t) = args.template {
        let path = DerivationPath::from_str(&format!(
            "m/{}'/{}'/{}'",
            t.purpose(),
            args.network.coin(),
            args.account
        ))
        .map_err(|e| CliError::BadInput(format!("account path: {e}")))?;
        let acct_xpriv = ScrubbedXpriv::new(
            master
                .derive_priv(&secp, &path)
                .map_err(|e| CliError::BadInput(format!("account derive: {e}")))?,
        );
        let acct_xpub = acct_xpriv.xpub(&secp);
        Some((
            format!(
                "m/{}'/{}'/{}'",
                t.purpose(),
                args.network.coin(),
                args.account
            ),
            acct_xpub.to_string(),
        ))
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
            language: effective_lang.as_str(),
            language_defaulted: effective_lang_defaulted,
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
        if effective_lang_defaulted {
            writeln!(
                stdout,
                "language:            {} (DEFAULT)",
                effective_lang.as_str()
            )
            .ok();
            let _ = writeln!(
                stderr,
                "note: --language defaulted to english; the master fingerprint and xpub depend on the wordlist language (record it alongside the backup)"
            );
        } else {
            writeln!(stdout, "language:            {}", effective_lang.as_str()).ok();
        }
    }
    // WatchOnly: ms derive emits only public key material (master fingerprint +
    // optional account xpub). Unconditional — covers --json AND text modes AND
    // non-defaulted language. Coexists with the language-defaulted note above.
    emit_output_class_advisory(OutputClass::WatchOnly, &mut std::io::stderr().lock());
    Ok(0)
}

#[cfg(test)]
mod scrub_tests {
    use super::*;

    // ========================================================================
    // COMPILE-TIME move-only guard for `ScrubbedXpriv` (mirrors the toolkit's
    // `derive_slot.rs` guard). `Copy` is E0184-blocked by `impl Drop`. `Clone`
    // is deliberately NOT derived; this block makes a `Clone` impl a COMPILE
    // ERROR. We cannot add `static_assertions` as a dep, and a `compile_fail`
    // doctest does NOT run for this binary-private module, so this `const _:
    // fn()` block is the load-bearing guard.
    //
    // Mechanics: two blanket `AmbiguousIfImpl` impls — one ALWAYS applies (the
    // `()` anchor), one applies ONLY when `T: Clone` (the `Invalid` marker).
    // The qualified call with an inferred `<_>` forces the compiler to UNIFY
    // the type-arg; ambiguous ⇒ compile error IFF `ScrubbedXpriv: Clone`.
    // ========================================================================
    const _: fn() = || {
        trait AmbiguousIfImpl<A> {
            fn some_item() {}
        }
        impl<T> AmbiguousIfImpl<()> for T {}
        struct Invalid;
        impl<T: Clone> AmbiguousIfImpl<Invalid> for T {}

        let _ = <ScrubbedXpriv as AmbiguousIfImpl<_>>::some_item;
    };

    /// Runtime drop-witness: build a known `Xpriv`, wrap it, assert the `&self`
    /// accessor surface (`xpub` / `fingerprint` / `derive_priv`) matches the
    /// bare upstream derivation of the same key, then let it drop (the scrub
    /// runs at end of scope). We do NOT assert post-drop byte values — reading
    /// scrubbed private memory is best-effort / UB-adjacent (the impl SAFETY
    /// note explains the inherent Copy-spill caveat); this test pins the
    /// accessor surface + that `Drop` runs, mirroring the toolkit's witness.
    #[test]
    fn scrubbed_xpriv_self_accessors_and_drop() {
        let secp = Secp256k1::new();
        let seed = [7u8; 32];
        // `master` is `Copy` upstream — keeping a bare copy here is itself an
        // independent witness of the Copy-spill caveat the scrub cannot defeat.
        let master = Xpriv::new_master(NetworkKind::Main, &seed).unwrap();
        let scrubbed = ScrubbedXpriv::new(master);

        // Public projections match the bare upstream derivation.
        assert_eq!(scrubbed.xpub(&secp), Xpub::from_priv(&secp, &master));
        assert_eq!(scrubbed.fingerprint(&secp), master.fingerprint(&secp));

        // The `&self` derive accessor matches bare upstream `derive_priv`.
        let path = DerivationPath::from_str("m/84'/0'/0'").unwrap();
        let child = scrubbed.derive_priv(&secp, &path).unwrap();
        assert_eq!(child, master.derive_priv(&secp, &path).unwrap());

        // `scrubbed` drops here → private_key.non_secure_erase() + volatile
        // chain_code zero-write run. (Best-effort; see the impl SAFETY note.)
    }
}
