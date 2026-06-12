//! Fuzz target: ms1 no-secret-leak oracle (THE DELICATE ONE).
//!
//! ms phase of the constellation stress-fuzz program (Cycle C), R0 [C1] fold.
//!
//! A FIXED known-valid 2-of-3 codex32 share-set is embedded as consts (below).
//! The fuzz input drives MUTATIONS of that valid set: bytes select (which
//! share, which position, which mutation: bit-flip / truncate / case-flip).
//! After mutating, the set is fed through `decode` / `inspect` / `combine_shares`,
//! and on ANY `Err(e)` we scan BOTH `format!("{e}")` (Display) AND
//! `format!("{e:?}")` (Debug) for any ≥8-char contiguous window of any share's
//! DATA-PART (the secret symbols, i.e. the chars after the `ms1` HRP). A window
//! hit on a NON-EXCLUDED variant = the error leaked secret material into its
//! message → panic (the leak finding).
//!
//! WINDOW = 8 chars: 40 bits over the 32-symbol codex32 alphabet — false-
//! positive odds per error string are negligible (R0 [M3]).
//!
//! ========================== NO EXCLUSIONS =============================
//! Originally (stress-Cycle-C, R0 [C1]) two `ms_codec::Error` variants were
//! KNOWN echo paths and skipped: `Error::Codex32(_)` (Display `{:?}`-wrapped
//! codex32-0.1.0's `InvalidChecksum`/`MismatchedHrp`/`MismatchedId`, each
//! carrying the FULL input string) and `Error::WrongHrp { .. }` (the observed
//! HRP, which a data-char→`1` shift could stretch into a secret prefix).
//!
//! FOLLOWUP `ms-codec-error-display-echoes-input` (ms-codec 0.4.4) closed both
//! at the source: the Codex32 Display/Debug now drops the 3 leaky variants'
//! String fields, and `WrongHrp.got` is bounded to 4 chars AT CONSTRUCTION
//! (< the 8-char window). The exclusion set therefore SHRANK TO ZERO — the
//! oracle now scans EVERY error variant and is the permanent regression gate
//! guarding that fix. Of the 16 `ms_codec::Error` variants, none can now emit
//! a ≥8-char contiguous input echo (every variant carries at most a `[u8; 4]`
//! tag = 4 chars, a bounded 4-char HRP, or a single char/byte — all below the
//! window).
//! ======================================================================
#![no_main]

use libfuzzer_sys::fuzz_target;
use ms_codec::{Error, combine_shares, decode, inspect};

/// HRP prefix every ms1 string begins with.
const HRP_PREFIX: &str = "ms1";

/// Contiguous-window length for the secret-echo scan (R0 [M3]: 8 chars = 40
/// bits over the 32-symbol alphabet).
const WINDOW: usize = 8;

/// The codex32 bech32 alphabet — the universe of data-part symbols.
const CODEX32_ALPHABET: &[u8] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

// ---------------------------------------------------------------------------
// FIXED known-valid share-set fixture (a 2-of-3 split of `Entr([0xAB; 16])`,
// share-set id "2hy2"). Generated ONCE via `encode_shares(Tag::ENTR,
// Threshold::new(2), 3, &Payload::Entr(vec![0xAB; 16]))` and frozen here; any
// 2 of the 3 recombine to the secret. These are the SECRETS the oracle proves
// never leak into an error message. (encode_shares draws a random id/filler,
// so re-running the generator yields different strings — these are pinned.)
// ---------------------------------------------------------------------------
const SHARES: [&str; 3] = [
    "ms122hy2qfwhfpnw4j2urf35fz3yk5hyvkf2qn9nflgmhutruz",
    "ms122hy2ptsjm0su0tq7qnhkv6ang87849sxpzfvfs9hsmva0t",
    "ms122hy2zdmaya4rgf7c95asrmfrrm9zhejjzcayfpjrej9kns",
];

/// The secret data-part of a share: the chars after the `ms1` HRP (the codex32
/// data+checksum symbols). These are the windows the scan hunts for.
fn data_part(share: &str) -> &str {
    share.strip_prefix(HRP_PREFIX).unwrap_or(share)
}

/// Does `haystack` contain any ≥WINDOW-char contiguous window of `needle`?
/// Both are ASCII (codex32 alphabet); compare on bytes.
fn contains_window(haystack: &str, needle: &str) -> Option<String> {
    let n = needle.as_bytes();
    let h = haystack.as_bytes();
    if n.len() < WINDOW {
        return None;
    }
    for w in n.windows(WINDOW) {
        // `windows` over a byte slice; find it verbatim in the haystack.
        if h.windows(WINDOW).any(|hw| hw == w) {
            return Some(String::from_utf8_lossy(w).into_owned());
        }
    }
    None
}

/// Scan a rendered error message for a secret-window leak. Panics (the leak
/// finding) on a hit against a NON-excluded variant.
fn scan_for_leak(e: &Error, surface: &str, rendered: &str) {
    // No exclusions: as of ms-codec 0.4.4 the `Codex32(_)` and `WrongHrp{..}`
    // paths are sanitized at the source (codex32 String fields dropped from
    // Display/Debug; `WrongHrp.got` bounded to 4 chars at construction), so the
    // oracle now guards them too — this is the permanent regression gate for
    // FOLLOWUP `ms-codec-error-display-echoes-input`.
    for (i, share) in SHARES.iter().enumerate() {
        let secret = data_part(share);
        if let Some(hit) = contains_window(rendered, secret) {
            panic!(
                "FINDING (secret leak): {surface} error rendered an ≥{WINDOW}-char window \
                 of share[{i}]'s data-part: hit={hit:?} variant={e:?}\n\
                 full rendered message: {rendered:?}"
            );
        }
    }
}

/// Render an error both ways (Display + Debug, R0 [M3]) and scan both.
fn check_error(e: &Error, surface: &str) {
    let display = format!("{e}");
    let debug = format!("{e:?}");
    scan_for_leak(e, surface, &display);
    scan_for_leak(e, surface, &debug);
}

/// Apply a fuzz-selected mutation to a COPY of the fixture share-set, then run
/// every decode/inspect/combine surface and scan their errors.
fn run(mutated: &[String]) {
    // decode + inspect each share individually (single-string surfaces).
    for share in mutated {
        if let Err(e) = decode(share) {
            check_error(&e, "decode");
        }
        if let Err(e) = inspect(share) {
            check_error(&e, "inspect");
        }
    }
    // combine the whole mutated set (the recombination surface).
    if let Err(e) = combine_shares(mutated) {
        check_error(&e, "combine");
    }
}

fuzz_target!(|data: &[u8]| {
    // The fixture is the secret; the fuzz input only SELECTS a mutation.
    // Layout: data[0] picks the share (mod 3), data[1] picks the byte position
    // within that share's string, data[2] picks the mutation kind, data[3]
    // supplies a bit-mask / case-target byte. Missing bytes default to 0.
    let mut mutated: Vec<String> = SHARES.iter().map(|s| s.to_string()).collect();

    let pick = |i: usize| -> u8 { data.get(i).copied().unwrap_or(0) };

    let share_idx = (pick(0) as usize) % mutated.len();
    let target = &mutated[share_idx];
    let bytes = target.as_bytes();

    if !bytes.is_empty() {
        let pos = (pick(1) as usize) % bytes.len();
        let kind = pick(2) % 3;
        let mut new_bytes = bytes.to_vec();
        match kind {
            // Bit-flip: XOR the selected byte with a fuzz-chosen mask (then map
            // back onto an alphabet char so it usually stays a parseable symbol;
            // out-of-alphabet bytes exercise the InvalidChar path).
            0 => {
                let mask = pick(3);
                let flipped = new_bytes[pos] ^ mask;
                // Bias toward staying in the alphabet so we reach deeper
                // (checksum/discriminate) rejections, not just InvalidChar.
                new_bytes[pos] = CODEX32_ALPHABET[(flipped & 0x1f) as usize];
                mutated[share_idx] = String::from_utf8_lossy(&new_bytes).into_owned();
            }
            // Truncate at the selected position.
            1 => {
                new_bytes.truncate(pos);
                mutated[share_idx] = String::from_utf8_lossy(&new_bytes).into_owned();
            }
            // Case-flip the selected byte (ASCII).
            _ => {
                new_bytes[pos] = new_bytes[pos].to_ascii_uppercase();
                mutated[share_idx] = String::from_utf8_lossy(&new_bytes).into_owned();
            }
        }
    }

    run(&mutated);
});
