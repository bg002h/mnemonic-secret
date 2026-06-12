//! Deterministic, re-runnable seed-corpus generator for the ms-codec fuzz
//! targets. ms phase of the constellation stress-fuzz program (Cycle C).
//!
//! Run with:
//!     cd fuzz && cargo +nightly-2026-04-27 test --test gen_corpus
//!
//! It (1) builds a fixed set of valid ms1 single-strings (entr + mnem) and
//! K-of-N share-sets, (2) writes seed files into the cargo-fuzz default
//! `corpus/<target>/` layout, and (3) — THE GATE (R0 [I6] / round-2 minor) —
//! asserts every committed seed passes the SAME call the corresponding target
//! uses. A seed that does not round-trip is a generation bug and fails loudly.
//!
//! Determinism: `ms_codec::encode` is RNG-free, so the entr/mnem single-string
//! seeds are byte-identical every run. `encode_shares` is NOT deterministic (it
//! draws a random share-set id + filler shares from getrandom), so the share
//! seeds are FROZEN literals generated once and pinned here — re-running this
//! test never churns the committed corpus.

use std::fs;
use std::path::{Path, PathBuf};

use ms_codec::{Payload, Tag, combine_shares, decode, encode, inspect};

// ---------------------------------------------------------------------------
// Fixtures.
// ---------------------------------------------------------------------------

/// Valid single-string payloads (entr at every BIP-39 length + a mnem). The
/// names key the seed filenames; `encode(Tag::ENTR, &payload)` is deterministic.
fn single_catalog() -> Vec<(&'static str, Payload)> {
    vec![
        ("entr16", Payload::Entr(vec![0xAB; 16])),
        ("entr20", Payload::Entr(vec![0x11; 20])),
        ("entr24", Payload::Entr(vec![0x22; 24])),
        ("entr28", Payload::Entr(vec![0x33; 28])),
        ("entr32", Payload::Entr(vec![0x5C; 32])),
        (
            "mnem16_lang1",
            Payload::Mnem {
                language: 1,
                entropy: vec![0x77; 16],
            },
        ),
        (
            "mnem32_lang0",
            Payload::Mnem {
                language: 0,
                entropy: vec![0x88; 32],
            },
        ),
    ]
}

/// FROZEN valid K-of-N share-sets (generated once via `encode_shares`; pinned
/// because `encode_shares` is RNG-keyed). Each inner slice is a full
/// distributed share-set; any threshold-many of its shares recombine. The gate
/// below re-verifies each set combines, so a stale literal fails the test.
fn share_set_catalog() -> Vec<(&'static str, &'static [&'static str])> {
    vec![
        (
            "entr16_2of2",
            &[
                "ms12m0ykqq5yyx35unzveezgkcffnfj7kdaqsr38tjcqf75ya3",
                "ms12m0ykpq6n73hkyuu02zk4lssk3n8dlc0asynh564p8ca38r",
            ],
        ),
        (
            "entr16_2of3",
            &[
                "ms12wrlrq376kccjhm76fwn45fvtdqcnkwuzsct7fzx4xcwnar",
                "ms12wrlrpxphg4ulgqlhmdspckf34cuglgc6s9756ggz8g0069",
                "ms12wrlrzkfqrzsgqyuqyg45v7xk5esvyz5mstg2xk6jy3vzn0",
            ],
        ),
        (
            "entr32_3of5",
            &[
                "ms1387vdqld6d3xn8h4xxd336x33jhq9p2p8c3cn2gmcych5p6hq49f2egdngg0ye0672pqqtel",
                "ms1387vdpuhktrj60gtjcl3ltueq7xldn3mreks4xw2zwdmacran34m8vh4q4yx35c2v6fgcu0s",
                "ms1387vdzm7uhdh9zql8kznw4daw4lfn3spxxx3e4vs6nnact56nykc23d2kn4p4mksuq9f64f9",
                "ms1387vdrcys3lrv2lpngsnqyh4lewkmrtmz8pele2pqex33jdsqqx28yjj9wegqkpqwsdpzzl2",
                "ms1387vdyln46klxtsxymma256zymt3c8rwgxf4lpfdm08e0rqjypefevcqc70c4xhz0jtd3pkj",
            ],
        ),
        (
            "mnem16_2of3",
            &[
                "ms123gs3qr3tdd6dckqv9tjw50evdw80amf5mukr2233pwfwzh2",
                "ms123gs3psjva7rr4rf5phln8en5awhyvxze9c9neda9fe4d0lc",
                "ms123gs3zvh9yzp3z4j4d6gam2d4ywwekglww5e29yfs3fcgc88",
            ],
        ),
    ]
}

/// The thresholds matching `share_set_catalog` (how many shares to verify-combine).
fn share_set_threshold(name: &str) -> usize {
    match name {
        "entr32_3of5" => 3,
        _ => 2,
    }
}

/// A few mutation-SELECTOR seeds for `ms1_no_secret_leak`. These are NOT valid
/// ms1 strings — they are the small byte arrays the no-leak target interprets
/// as (share, position, kind, mask). Their "validity" is only that they drive
/// the harness without crashing it (the embedded share-set is the fixture). The
/// gate runs the SAME mutation+surface calls the target does (minus the leak
/// scan) and asserts no panic.
fn no_leak_selector_seeds() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        ("share0_pos5_bitflip", vec![0, 5, 0, 0x01]),
        ("share1_pos10_truncate", vec![1, 10, 1, 0]),
        ("share2_pos3_caseflip", vec![2, 3, 2, 0]),
        ("share0_pos0_bitflip_high", vec![0, 0, 0, 0xff]),
        ("empty", vec![]),
        ("single_byte", vec![7]),
    ]
}

// ---------------------------------------------------------------------------
// Corpus writing + the no-leak harness mirror (kept in sync with the target).
// ---------------------------------------------------------------------------

const CODEX32_ALPHABET: &[u8] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

/// The embedded fixture used by `ms1_no_secret_leak` (must match the target's
/// `SHARES` const). The selector seeds mutate THIS set; the gate only needs the
/// set to drive the surfaces without panicking.
const NO_LEAK_SHARES: [&str; 3] = [
    "ms122hy2qfwhfpnw4j2urf35fz3yk5hyvkf2qn9nflgmhutruz",
    "ms122hy2ptsjm0su0tq7qnhkv6ang87849sxpzfvfs9hsmva0t",
    "ms122hy2zdmaya4rgf7c95asrmfrrm9zhejjzcayfpjrej9kns",
];

/// Mirror of the target's mutation selection (kept deliberately in lockstep).
/// Returns the mutated share-set so the gate can run the surfaces.
fn mutate_fixture(data: &[u8]) -> Vec<String> {
    let mut mutated: Vec<String> = NO_LEAK_SHARES.iter().map(|s| s.to_string()).collect();
    let pick = |i: usize| -> u8 { data.get(i).copied().unwrap_or(0) };
    let share_idx = (pick(0) as usize) % mutated.len();
    let bytes = mutated[share_idx].as_bytes().to_vec();
    if !bytes.is_empty() {
        let pos = (pick(1) as usize) % bytes.len();
        let kind = pick(2) % 3;
        let mut new_bytes = bytes.clone();
        match kind {
            0 => {
                let flipped = new_bytes[pos] ^ pick(3);
                new_bytes[pos] = CODEX32_ALPHABET[(flipped & 0x1f) as usize];
                mutated[share_idx] = String::from_utf8_lossy(&new_bytes).into_owned();
            }
            1 => {
                new_bytes.truncate(pos);
                mutated[share_idx] = String::from_utf8_lossy(&new_bytes).into_owned();
            }
            _ => {
                new_bytes[pos] = new_bytes[pos].to_ascii_uppercase();
                mutated[share_idx] = String::from_utf8_lossy(&new_bytes).into_owned();
            }
        }
    }
    mutated
}

/// Run the no-leak surfaces (decode/inspect/combine) on a mutated set. The gate
/// only asserts it does not panic; the leak scan is the target's job.
fn run_no_leak_surfaces(mutated: &[String]) {
    for share in mutated {
        let _ = decode(share);
        let _ = inspect(share);
    }
    let _ = combine_shares(mutated);
}

fn corpus_dir(target: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("corpus")
        .join(target)
}

fn write_seed(dir: &Path, name: &str, bytes: &[u8]) {
    fs::create_dir_all(dir).expect("create corpus dir");
    fs::write(dir.join(name), bytes).expect("write seed");
}

#[test]
fn gen_corpus() {
    let dir_decode = corpus_dir("ms1_decode");
    let dir_combine = corpus_dir("ms1_combine");
    let dir_no_leak = corpus_dir("ms1_no_secret_leak");

    let mut decode_count = 0usize;
    let mut combine_count = 0usize;
    let mut no_leak_count = 0usize;

    // --- ms1_decode seeds: valid ms1 single-strings. ---
    for (name, payload) in single_catalog() {
        let s = encode(Tag::ENTR, &payload)
            .unwrap_or_else(|e| panic!("gen-corpus: {name} failed to encode: {e}"));
        // GATE: the seed must decode via the SAME entry the target uses.
        decode(&s)
            .unwrap_or_else(|e| panic!("gen-corpus GATE: {name} ms1 string does not decode: {e}"));
        write_seed(&dir_decode, &format!("{name}.ms1"), s.as_bytes());
        decode_count += 1;
    }

    // --- ms1_combine seeds: share-sets joined by `\n` BETWEEN shares (no
    //     trailing newline, so split('\n') reproduces the exact share strings). ---
    for (name, shares) in share_set_catalog() {
        let joined = shares.join("\n");
        let parts: Vec<String> = joined.split('\n').map(str::to_string).collect();
        // Sanity: split('\n') reproduces the exact shares (no empty trailing part).
        assert_eq!(
            parts.len(),
            shares.len(),
            "gen-corpus: {name} join/split changed share count"
        );
        // GATE: a threshold-many subset must combine via the target's split-call.
        let k = share_set_threshold(name);
        assert!(
            parts.len() >= k,
            "gen-corpus: {name} has fewer shares than k"
        );
        combine_shares(&parts).unwrap_or_else(|e| {
            panic!("gen-corpus GATE: {name} \\n-joined seed (full set) does not combine: {e}")
        });
        // Also verify the first-k subset combines (the minimal recombination).
        combine_shares(&parts[..k]).unwrap_or_else(|e| {
            panic!("gen-corpus GATE: {name} first-{k} subset does not combine: {e}")
        });
        write_seed(&dir_combine, &format!("{name}.shares"), joined.as_bytes());
        combine_count += 1;
    }

    // --- ms1_no_secret_leak seeds: mutation selectors. ---
    for (name, sel) in no_leak_selector_seeds() {
        // GATE: the selector must drive the harness without panicking.
        let mutated = mutate_fixture(&sel);
        run_no_leak_surfaces(&mutated);
        write_seed(&dir_no_leak, &format!("{name}.sel"), &sel);
        no_leak_count += 1;
    }

    // Sanity: at least one multi-symbol decode seed and a multi-share combine
    // seed exist so the splitter targets are meaningful.
    assert!(decode_count >= 5, "expected several decode seeds");
    assert!(combine_count >= 3, "expected several combine seeds");

    eprintln!(
        "gen-corpus wrote: ms1_decode={decode_count}, ms1_combine={combine_count}, \
         ms1_no_secret_leak={no_leak_count}"
    );
}
