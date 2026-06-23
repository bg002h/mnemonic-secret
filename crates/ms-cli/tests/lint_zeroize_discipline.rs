//! v0.9.0 Cycle A Phase 2 — ms-cli Zeroizing-wrapper discipline lint.
//!
//! Companion to the mnemonic-toolkit `lint_zeroize_discipline.rs` lint
//! and the ms-codec sibling lint. Authoritative reference:
//! `mnemonic-toolkit/design/SPEC_secret_memory_hygiene_v0_9_0.md` §1
//! item 2 + survey §1 ms-cli table (10 OWNED rows incl. 3 clap-field
//! rows added post-R1 C-2).
//!
//! For each enumerated OWNED-secret site in ms-cli, this lint asserts
//! the implementing source file contains a stable `Zeroizing` evidence
//! anchor. The 3 clap-field rows (`EncodeArgs::phrase`,
//! `EncodeArgs::hex`, `VerifyArgs::phrase`) use the "consume +
//! immediately wrap" pattern at `run()` entry — `let phrase =
//! Zeroizing::new(std::mem::take(&mut args.phrase).unwrap_or_default())`
//! — since clap-derive does not natively emit `Zeroizing<String>`.
//!
//! `cmd/decode.rs` post-decode stdout-emission paths (lines 67-94)
//! are STDOUT-LEAK out-of-scope per SPEC §3 OOS-10; they are NOT
//! enumerated here.
//!
//! RED on Phase 2 first commit: no source uses `Zeroizing` yet.

use std::fs;
use std::path::Path;

struct ZeroizeRow {
    label: &'static str,
    source_file: &'static str,
    evidence: &'static [&'static str],
}

/// Canonical 10-row list per survey §1 ms-cli table.
/// Per-row evidence anchors tightened post R1 I-4 fold so each row enforces
/// its specific call-site discipline.
const ZEROIZE_ROWS: &[ZeroizeRow] = &[
    // ---- 3 clap-field rows (post-R1 C-2 fold) ----
    ZeroizeRow {
        label: "EncodeArgs::phrase consume + Zeroizing wrap at run() entry",
        source_file: "src/cmd/encode.rs",
        evidence: &["std::mem::take(&mut args.phrase).map(Zeroizing::new)"],
    },
    ZeroizeRow {
        label: "EncodeArgs::hex consume + Zeroizing wrap at run() entry",
        source_file: "src/cmd/encode.rs",
        evidence: &["std::mem::take(&mut args.hex).map(Zeroizing::new)"],
    },
    ZeroizeRow {
        label: "VerifyArgs::phrase consume + Zeroizing wrap at run() entry",
        source_file: "src/cmd/verify.rs",
        evidence: &["std::mem::take(&mut args.phrase).map(Zeroizing::new)"],
    },
    // ---- parse.rs ----
    ZeroizeRow {
        label: "parse::read_phrase_input returns Zeroizing<String>",
        source_file: "src/parse.rs",
        evidence: &["pub fn read_phrase_input(arg: Option<&str>) -> Result<Zeroizing<String>>"],
    },
    ZeroizeRow {
        label: "parse::read_stdin raw buffer wrapped",
        source_file: "src/parse.rs",
        evidence: &["let mut buf: Zeroizing<String> = Zeroizing::new(String::new())"],
    },
    // ---- cmd/encode.rs run() locals ----
    ZeroizeRow {
        label: "cmd/encode::run locals (phrase / entropy) wrapped",
        source_file: "src/cmd/encode.rs",
        evidence: &["let (entropy, language_for_card): (Zeroizing<Vec<u8>>"],
    },
    ZeroizeRow {
        label: "cmd/encode entropy buffer fed to Payload via wrapped clone",
        source_file: "src/cmd/encode.rs",
        evidence: &[
            "Payload::Entr((*entropy_for_codec).clone())",
            "Payload::Entr((*entropy).clone())",
        ],
    },
    // ---- cmd/decode.rs run() locals ----
    ZeroizeRow {
        label: "cmd/decode::run locals (entropy / phrase) wrapped",
        source_file: "src/cmd/decode.rs",
        evidence: &["let entropy: Zeroizing<Vec<u8>>"],
    },
    // ---- cmd/verify.rs run() locals ----
    ZeroizeRow {
        label: "cmd/verify::run locals (entropy / supplied / derived) wrapped",
        source_file: "src/cmd/verify.rs",
        evidence: &["let entropy: Zeroizing<Vec<u8>>"],
    },
    // cycle-15 Lane M (slug #9, Minor-3): RE-POINT (not append) this existing
    // row. It previously anchored on `let derived_str: Zeroizing<String>` at
    // verify.rs:117 (inside run()), which is GREEN regardless of whether the
    // `emit_round_trip_ok` `to_string()` word-count temp at :170 is wrapped —
    // a FALSE-GREEN. Re-anchor on the `emit_round_trip_ok` word-count site so
    // the row actually guards :170.
    ZeroizeRow {
        label: "cmd/verify emit_round_trip_ok word-count temp wrapped (slug #9)",
        source_file: "src/cmd/verify.rs",
        evidence: &["let wc_src: Zeroizing<String>"],
    },
    // ---- cycle-15 Lane M: ms-cli intake/report sites ----
    ZeroizeRow {
        label: "cmd/inspect ms1 intake wrapped (slug #5)",
        source_file: "src/cmd/inspect.rs",
        evidence: &["Zeroizing::new(read_input("],
    },
    ZeroizeRow {
        label: "cmd/repair ms1 intake wrapped (slug #6)",
        source_file: "src/cmd/repair.rs",
        evidence: &["Zeroizing::new(read_input("],
    },
    ZeroizeRow {
        label: "cmd/repair RepairDetail chunk fields are Zeroizing<String> (slug #6)",
        source_file: "src/cmd/repair.rs",
        evidence: &["original_chunk: Zeroizing<String>"],
    },
    // ---- derive.rs (Wave-2 ms lane: derived-Xpriv best-effort scrub) ----
    ZeroizeRow {
        // The derived master/account `Xpriv` values are confined in the
        // binary-private move-only `ScrubbedXpriv` newtype; its `Drop` does a
        // best-effort byte-scrub (`SecretKey::non_secure_erase()` +
        // volatile chain_code zero-write). Closes the in-repo leg of
        // `ms-cli-derive-xpriv-master-not-zeroized`.
        label: "cmd/derive ScrubbedXpriv scrubs master/account Xpriv on drop (wave2)",
        source_file: "src/cmd/derive.rs",
        evidence: &["struct ScrubbedXpriv", "non_secure_erase()"],
    },
];

fn crate_root() -> &'static Path {
    Path::new(".")
}

#[test]
fn canonical_list_has_expected_row_count() {
    let n = ZEROIZE_ROWS.len();
    assert_eq!(
        n, 14,
        "ZEROIZE_ROWS row count = {n}; expected 14 (survey §1 ms-cli table, post-R1 C-2 fold + cycle-15 Lane M: inspect-intake/repair-intake/repair-chunk-field rows, the verify row was re-pointed not added; + Wave-2 ms lane: the derive.rs ScrubbedXpriv derived-Xpriv scrub row)."
    );
}

/// cycle-15 Lane M (slug #6, RULE Z-DEBUG) — NEGATIVE anchor. Once
/// `RepairDetail`'s `original_chunk`/`corrected_chunk` are `Zeroizing<String>`,
/// a derived `Debug` would forward to `String::fmt` and LEAK the secret chunk.
/// `RepairDetail` has no `{:?}` consumer, so the cleanest fix is to drop the
/// `Debug` derive entirely. This guards that it stays dropped.
#[test]
fn repair_detail_does_not_derive_debug() {
    let src =
        fs::read_to_string(crate_root().join("src/cmd/repair.rs")).expect("read src/cmd/repair.rs");
    // The struct definition line must not derive Debug.
    assert!(
        !src.contains("#[derive(Debug, Clone)]\nstruct RepairDetail")
            && !src.contains("#[derive(Debug, Clone)]\npub struct RepairDetail")
            && !src.contains("#[derive(Clone, Debug)]\nstruct RepairDetail"),
        "RepairDetail still derives Debug — its Zeroizing<String> chunk fields would \
         leak via the derived Debug (RULE Z-DEBUG). Drop the Debug derive (keep Clone)."
    );
}

#[test]
fn every_canonical_zeroize_row_has_evidence_anchor() {
    let mut missing: Vec<String> = Vec::new();
    for row in ZEROIZE_ROWS {
        let path = crate_root().join(row.source_file);
        let source = fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!(
                "failed to read evidence source {} for row {:?}: {e}",
                path.display(),
                row.label
            )
        });
        let hit = row.evidence.iter().any(|needle| source.contains(needle));
        if !hit {
            missing.push(format!(
                "  - {} ({}): no evidence anchor; expected one of {:?}",
                row.label, row.source_file, row.evidence,
            ));
        }
    }
    assert!(
        missing.is_empty(),
        "ms-cli zeroize-discipline lint: {} row(s) missing Zeroizing evidence:\n{}",
        missing.len(),
        missing.join("\n"),
    );
}
