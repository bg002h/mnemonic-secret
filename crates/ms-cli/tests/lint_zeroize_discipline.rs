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
        evidence: &["Payload::Entr((*entropy_for_codec).clone())", "Payload::Entr((*entropy).clone())"],
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
    ZeroizeRow {
        label: "cmd/verify success-log derived_mnemonic.to_string() wrapped",
        source_file: "src/cmd/verify.rs",
        evidence: &["let derived_str: Zeroizing<String>"],
    },
];

fn crate_root() -> &'static Path {
    Path::new(".")
}

#[test]
fn canonical_list_has_expected_row_count() {
    let n = ZEROIZE_ROWS.len();
    assert_eq!(
        n, 10,
        "ZEROIZE_ROWS row count = {n}; expected 10 (survey §1 ms-cli table, post-R1 C-2 fold)."
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
