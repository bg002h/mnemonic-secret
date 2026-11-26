//! `ms repair` — BCH error-correction for ms1 strings.
//!
//! Realizes plan §2.B.3 (v0.22.x follow-ups Tranche B.5). Wraps
//! `ms_codec::decode_with_correction` (which performs full BCH(93,80,8)
//! correction up to t=4 and returns
//! `(Tag, Payload, Vec<CorrectionDetail>)`) and renders a per-input repair
//! report.
//!
//! Single-chunk context: ms1 is single-chunk per codex32 spec, so the
//! `--ms1` flag is non-repeating (cf. mk-cli's variadic `mk1_strings`).
//! Single-HRP context (always `ms`): no `--hrp` flag and no Levenshtein
//! "did you mean" suggestion — `decode_with_correction` validates HRP
//! internally against `"ms"`. HRP mismatches surface as exit 2 (via
//! `From<ms_codec::Error>` mapping `WrongHrp` → `CliError::FormatViolation`).
//!
//! Exit codes (the constellation exit-code spec's bits; D26's meanings, Cycle F
//! `ms1-repair-demote-to-candidate`):
//!   - 0 — input was already valid (no corrections applied)
//!   - 20 — 16 REPAIRED + 4 UNVERIFIED, a VERIFY-ME candidate: a substitution
//!     correction was applied. An
//!     ms1 is a single-string bearer secret with no cross-chunk hash and no
//!     internal redundancy, so a bounded-distance BCH substitution-correction
//!     spends the checksum's error-detection budget and can alias to a
//!     DIFFERENT valid seed undetectably (BIP-93: implementations SHOULD NOT
//!     automatically proceed with a corrected codex32 string without user
//!     confirmation). `ms repair` therefore NEVER silently "recovers" an
//!     ms1 — any correction is surfaced as an unverified candidate (exit 20),
//!     mirroring the toolkit's `mnemonic repair --ms1` demotion (same
//!     `ms_codec` engine). A single ms1 has no oracle, so 16 never comes
//!     without 4 here.
//!   - 22 — 20 + 2 INVALID: a correction was applied but the corrected
//!     string is not an ms1 card this version decodes (exit-code spec
//!     decision 6). The corrected string is still printed, with the warning,
//!     for a person repairing by hand.
//!   - 2 — unrepairable input (`TooManyErrors`), or a string with a clean
//!     checksum that is not a card: nothing is printed (`?`).
//!
//! D9 secret-on-stdout: ms1 IS the seed-secret (BIP-39 entropy), so this
//! subcommand ALWAYS emits a stderr advisory when invoked (regardless of
//! whether corrections fired or whether `--json` was set — even
//! pass-through of a valid ms1 to stdout is sensitive material on stdout).
//! Emitted via `emit_output_class_advisory(PrivateKeyMaterial)` — byte-identical
//! to mnemonic-toolkit's `secret_advisory::emit_output_class_advisory`.
//!
//! Text output mirrors `mnemonic repair`'s text-form report shape (see
//! `mnemonic-toolkit/src/cmd/repair.rs::emit_repair_text`). JSON output
//! byte-matches the toolkit's standalone `RepairJson` schema (D27 — fields
//! `schema_version`, `kind`, `verdict`, `corrected_chunks`, `repairs`) so
//! cross-CLI parsers reuse the same struct. `verdict` (Cycle F
//! `ms1-repair-demote-to-candidate`) is `"blessed"` for a clean (0-correction)
//! decode, `"candidate"` for a touched substitution correction. A corrected
//! string that does not decode (exit 22) adds `not_a_card: {kind, message}`
//! after `verdict`; the field is absent otherwise.

use clap::Args;
use ms_codec::CorrectionDetail;
use serde::Serialize;
use zeroize::Zeroizing;

use crate::advisory::{emit_output_class_advisory, OutputClass};
use crate::error::{CliError, Result};
use crate::exit_code::Bits;
use crate::parse::{read_input, Source};

/// `ms repair` arguments.
#[derive(Args, Debug)]
#[command(group = clap::ArgGroup::new("repair_input").required(true).args(["ms1", "in_path"]))]
pub struct RepairArgs {
    /// Write the canonical artifact to FILE, **owner-only (0600)**.
    ///
    /// Through the shared crate's `write::write_private`: the bytes go to a
    /// temporary `FILE.<pid>.<n>.tmp` beside FILE, created 0600 and synced, which
    /// then replaces FILE, so a failed write leaves an existing FILE unchanged.
    /// FILE's directory (its target's, for a symlink) must be writable, and a run
    /// killed mid-write can leave that owner-only temporary behind. It OVERWRITES
    /// (§6b); a symlink is followed and its target replaced. If ms would also
    /// print on stdout and stdout is FILE, the run is refused (exit 8).
    ///
    /// **Not to be confused with `ms gen-man --out <DIR>`**, which is shipped,
    /// exampled, driven by this repo's `man-release.yml` and by
    /// `scripts/install.sh` in mnemonic-toolkit, and means a DIRECTORY. P2 does
    /// not rename it; F-282 records that one binary now carries two meanings.
    #[arg(long, value_name = "FILE")]
    pub out: Option<std::path::PathBuf>,

    /// Proceed even though secret material is on argv.
    ///
    /// **Read off RAW argv before the parser, and it is a CHANNEL rather than a
    /// flag** (§6d): the admitted value is replaced by `-` and routed to the
    /// verb through a side channel, so it is never handed back to clap. It is
    /// declared here so `--help` documents it, and destructured nowhere --
    /// consulting it after parsing would be a decision reached too late.
    ///
    /// For a single-user air-gapped box, or an amnesic Tails session.
    #[arg(long)]
    pub allow_argv_secret: bool,

    /// ms1 string to attempt to repair. Use `-` to read the string from
    /// stdin (a single line). Single-chunk per codex32 spec; non-repeating.
    #[arg(long, value_name = "MS1")]
    pub ms1: Option<String>,

    /// Read the ms1 string from FILE instead of argv or stdin.
    ///
    /// The private channel that frees stdin: argv is public (/proc, `ps`, shell
    /// history), and before P2 `-` was the only alternative, so two private
    /// values could not be supplied in one invocation.
    #[arg(long = "in", value_name = "FILE")]
    pub in_path: Option<std::path::PathBuf>,

    /// Emit a single JSON envelope on stdout instead of the text-form
    /// report. Schema byte-matches `mnemonic repair --json`'s
    /// `RepairJson` shape (cross-CLI parser reuse).
    #[arg(long)]
    pub json: bool,
}

/// Per-input repair report. Mirrors toolkit's `RepairDetail` shape so
/// JSON output is byte-identical to `mnemonic repair --json`.
///
/// cycle-15 Lane M (slug #6, RULE Z-DEBUG): `original_chunk`/`corrected_chunk`
/// are decodable secret ms1 strings, held as `Zeroizing<String>` (scrub-on-
/// drop). The `#[derive(Debug)]` is DROPPED (no `{:?}` consumer exists) because
/// a derived `Debug` over a `Zeroizing<String>` forwards to `String::fmt` and
/// would LEAK the secret chunk. `Clone` is kept.
#[derive(Clone)]
struct RepairDetail {
    chunk_index: usize,
    original_chunk: Zeroizing<String>,
    corrected_chunk: Zeroizing<String>,
    /// `(position, was, now)` — `position` is 0-indexed into the data-part
    /// (chars after the `ms` HRP + `1` separator).
    corrected_positions: Vec<(usize, char, char)>,
}

/// Run `ms repair`.
pub fn run(args: RepairArgs) -> Result<u8> {
    // bh-ms-I1: the report goes to stdout; stdout must not be the `--out` file.
    if let Some(path) = args.out.as_deref() {
        crate::out::refuse_stdout_alias(path)?;
    }
    // `read_input` handles the `-` stdin sentinel (single line / trimmed).
    // cycle-15 Lane M (slug #6): the ms1 intake IS secret material — scrub on drop.
    let original: Zeroizing<String> = Zeroizing::new(read_input(
        Source::new(args.ms1.as_deref(), args.in_path.as_deref()).on("--ms1"),
    )?);

    // `decode_with_correction` performs BCH correction internally; HRP /
    // length / BCH-uncorrectable rejections surface as `ms_codec::Error`
    // and route to the appropriate exit code via `From<ms_codec::Error>`
    // in error.rs (`TooManyErrors` → `FormatViolation` → exit 2 per D26).
    //
    // Exit-code spec decision 6: a CORRECTED string that does not decode is
    // still printed (a person repairing by hand needs to see it), and the run
    // adds 2. A clean string that does not decode fails atomically (2).
    let (corrections, not_a_card): (Vec<CorrectionDetail>, Option<CliError>) =
        match ms_codec::decode_with_correction(&original) {
            Ok((_tag, _payload, corrections)) => (corrections, None),
            Err(e @ ms_codec::Error::TooManyErrors { .. }) => return Err(e.into()),
            Err(e) => match correct_only(&original) {
                Some(c) if !c.is_empty() => (c, Some(CliError::from(e))),
                _ => return Err(e.into()),
            },
        };

    // The corrected ms1 is itself a valid decodable secret — hold it as Zeroizing.
    let (corrected_chunk, corrected_positions) =
        reconstruct_corrected(original.as_str(), &corrections);

    // ms1 is single-chunk → exactly one RepairDetail with `chunk_index = 0`.
    let report = RepairDetail {
        chunk_index: 0,
        original_chunk: original.clone(),
        corrected_chunk: corrected_chunk.clone(),
        corrected_positions,
    };

    // The serialized corrected chunk is stdout-bound (OOS-10 stdout-leak), but
    // hold the in-memory vec as Zeroizing too (defense-in-depth, slug #6/#8).
    let corrected_chunks: Zeroizing<Vec<String>> = Zeroizing::new(vec![(*corrected_chunk).clone()]);
    let reports = vec![report];

    // **RULED: `--out` receives the ARTIFACT LINE ALONE, and the report stays
    // on stdout.** `ms repair` is the one verb whose stdout carries a report and
    // then the artifact; a payload beginning `# Repair report` is not an ms1, and
    // the correction record is what an operator needs to SEE before trusting a
    // repaired card, so it belongs on the stream they are reading rather than
    // buried in a file they will feed to an engraver. It is also the one verb
    // that writes `--out` while exiting non-zero (20, VERIFY-ME) -- the file is
    // written all the same, because a correction the operator must confirm is
    // still the artifact they asked for.
    // A corrected string that is not a card is shown, never filed: `--out`
    // feeds an engraver, so it is not written (exit-code spec decision 6).
    if let Some(path) = args.out.as_deref() {
        if not_a_card.is_none() {
            crate::out::write_artifact(path, &format!("{}\n", *corrected_chunk))?;
        }
    }

    if args.json {
        emit_json(corrected_chunks.as_slice(), &reports, not_a_card.as_ref())?;
    } else {
        emit_text(corrected_chunks.as_slice(), &reports)?;
    }

    // D9: emit sensitive-secret stderr advisory (always — even pass-through
    // of a valid ms1 to stdout is sensitive material on stdout). Uses the
    // canonical OutputClass::PrivateKeyMaterial line (byte-identical to
    // mnemonic-toolkit's emit_output_class_advisory for PrivateKeyMaterial).
    emit_output_class_advisory(
        OutputClass::PrivateKeyMaterial,
        &mut std::io::stderr().lock(),
    );

    let any_correction = reports.iter().any(|r| !r.corrected_positions.is_empty());

    // Cycle F (`ms1-repair-demote-to-candidate`) Option B demotion: a touched
    // substitution correction is a VERIFY-ME candidate (16 + 4 = 20), never
    // a silent "recovered" — an ms1 is a single-string bearer secret
    // with no cross-chunk hash / internal redundancy, so the correction
    // cannot self-verify. The reason BODY mirrors the toolkit engine's ms1
    // `SetVerify::Unverified` reason text (prefixed `repair: ` here) (D27/D9 parity).
    if any_correction {
        eprintln!(
            "repair: correction UNVERIFIED — a corrected seed card cannot be \
             self-verified; confirm the derived address/xpub against a known-good \
             copy before use; BIP-93 recommends confirming a corrected codex32 string"
        );
    }
    let mut bits = Bits::NONE;
    if any_correction {
        bits |= Bits::REPAIRED | Bits::UNVERIFIED;
    }
    if let Some(e) = not_a_card {
        eprintln!(
            "repair: the corrected string is NOT an ms1 card this version decodes: {}",
            e.message()
        );
        // Cause first, then its consequence for `--out`.
        if let Some(path) = args.out.as_deref() {
            eprintln!(
                "repair: --out {} was not written: the corrected string is not a card",
                path.display()
            );
        }
        bits |= Bits::INVALID;
    }
    Ok(bits.to_exit_code())
}

/// BCH correction alone, for a string `decode_with_correction` corrected and
/// then refused at decode: the same steps as the codec's (symbols, residue,
/// `decode_regular_errors`, re-verify), without the decode. `None` when the
/// string does not parse or is uncorrectable; an empty vector when its
/// checksum is already clean.
fn correct_only(s: &str) -> Option<Vec<CorrectionDetail>> {
    const ALPHABET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";
    let lower = Zeroizing::new(s.to_ascii_lowercase());
    let rest = lower.strip_prefix("ms1")?;
    let mut symbols: Zeroizing<Vec<u8>> = Zeroizing::new(Vec::with_capacity(rest.len()));
    for c in rest.bytes() {
        symbols.push(ALPHABET.iter().position(|&b| b == c)? as u8);
    }
    let residue = |data: &[u8]| {
        let mut input = ms_codec::bch::hrp_expand("ms");
        input.extend_from_slice(data);
        let r = ms_codec::bch::polymod_run(&input) ^ ms_codec::bch::MS_REGULAR_CONST;
        zeroize::Zeroize::zeroize(&mut input);
        r
    };
    let r = residue(&symbols);
    if r == 0 {
        return Some(Vec::new());
    }
    let (positions, magnitudes) = ms_codec::bch_decode::decode_regular_errors(r, symbols.len())?;
    let mut details = Vec::with_capacity(positions.len());
    for (&pos, &mag) in positions.iter().zip(&magnitudes) {
        let was = *symbols.get(pos)?;
        let now = was ^ mag;
        details.push(CorrectionDetail {
            position: pos,
            was: ALPHABET[(was & 0x1F) as usize] as char,
            now: ALPHABET[(now & 0x1F) as usize] as char,
        });
        symbols[pos] = now;
    }
    (residue(&symbols) == 0).then_some(details)
}

/// Build the corrected ms1 string + `(position, was, now)` triples from
/// the `CorrectionDetail` vector emitted by `decode_with_correction`.
/// Re-encodes by substituting characters at the reported data-part
/// positions in the original string; ms-codec already verified
/// post-correction polymod, so the substituted string is a valid codeword.
fn reconstruct_corrected(
    original: &str,
    corrections: &[CorrectionDetail],
) -> (Zeroizing<String>, Vec<(usize, char, char)>) {
    // ms1 HRP + separator = "ms1" (3 chars). The data-part begins at byte
    // offset 3 in the lowercased input. `decode_with_correction` already
    // validated the HRP; this `rfind('1')` is a sanity check matching
    // mk-cli's `reconstruct_corrected` shape.
    let sep_pos = original
        .rfind('1')
        .expect("ms1 input passed BCH decode; must contain bech32 separator '1'");
    let (prefix, rest) = original.split_at(sep_pos);
    // Lowercase the data part for substitution (`decode_with_correction`
    // lowercases internally, so the `was`/`now` chars are lowercase).
    let mut data_chars: Vec<char> = rest[1..].chars().map(|c| c.to_ascii_lowercase()).collect();

    let mut corrected_positions: Vec<(usize, char, char)> = Vec::with_capacity(corrections.len());
    for c in corrections {
        // Defensive: ms-codec's `decode_with_correction` bounds-checks
        // before applying corrections (errors `TooManyErrors` otherwise),
        // so `c.position` is in-range. Mirror mk-cli's defensive pattern.
        if c.position < data_chars.len() {
            data_chars[c.position] = c.now;
        }
        corrected_positions.push((c.position, c.was, c.now));
    }

    let mut out: Zeroizing<String> =
        Zeroizing::new(String::with_capacity(prefix.len() + 1 + data_chars.len()));
    out.push_str(&prefix.to_ascii_lowercase());
    out.push('1');
    for c in &data_chars {
        out.push(*c);
    }

    (out, corrected_positions)
}

/// Text-form report: `# Repair report` header, per-chunk correction lines,
/// then corrected chunks one per line. Mirrors toolkit's
/// `cmd::repair::emit_repair_text` shape byte-exact (modulo the `ms1`-only
/// `kind_str`).
fn emit_text(corrected_chunks: &[String], reports: &[RepairDetail]) -> Result<()> {
    let any_correction = reports.iter().any(|r| !r.corrected_positions.is_empty());
    if any_correction {
        println!("# Repair report");
        for r in reports {
            if r.corrected_positions.is_empty() {
                continue;
            }
            let n = r.corrected_positions.len();
            let plural = if n == 1 { "correction" } else { "corrections" };
            let mut line = format!("#   ms1 chunk {}: {} {} at ", r.chunk_index, n, plural);
            for (i, (pos, was, now)) in r.corrected_positions.iter().enumerate() {
                if i > 0 {
                    line.push_str(", ");
                }
                line.push_str(&format!("position {pos}: '{was}' -> '{now}'"));
            }
            println!("{line}");
        }
    }
    for chunk in corrected_chunks {
        println!("{chunk}");
    }
    Ok(())
}

// JSON envelope — schema MUST byte-match toolkit's standalone `RepairJson`
// at `mnemonic-toolkit/src/cmd/repair.rs` (D27 cross-CLI parser
// reuse). Field order is part of the schema (serde preserves struct field
// order in the default JSON serializer).
#[derive(Serialize)]
struct RepairJson<'a> {
    schema_version: &'static str,
    kind: &'static str,
    /// Cycle F (`ms1-repair-demote-to-candidate`) M1/M4 — `"blessed"` for a
    /// clean (0-correction) decode, `"candidate"` for a touched substitution
    /// correction. Fixed position right after `kind` — byte-matches the
    /// toolkit's `RepairJson.verdict` field order (D27/D9).
    verdict: &'static str,
    /// Present only when the corrected string does not decode (exit 22,
    /// decision 6): why it is not a card. Absent otherwise, so the 0 and 20
    /// envelopes are unchanged.
    #[serde(skip_serializing_if = "Option::is_none")]
    not_a_card: Option<NotACardJson>,
    corrected_chunks: &'a [String],
    repairs: Vec<RepairJsonDetail<'a>>,
}

#[derive(Serialize)]
struct NotACardJson {
    kind: &'static str,
    message: String,
}

#[derive(Serialize)]
struct RepairJsonDetail<'a> {
    chunk_index: usize,
    original_chunk: &'a str,
    corrected_chunk: &'a str,
    corrected_positions: Vec<RepairJsonPosition>,
}

#[derive(Serialize)]
struct RepairJsonPosition {
    position: usize,
    was: String,
    now: String,
}

fn emit_json(
    corrected_chunks: &[String],
    reports: &[RepairDetail],
    not_a_card: Option<&CliError>,
) -> Result<()> {
    let any_correction = reports.iter().any(|r| !r.corrected_positions.is_empty());
    let envelope = RepairJson {
        schema_version: "1",
        kind: "ms1",
        verdict: if any_correction {
            "candidate"
        } else {
            "blessed"
        },
        not_a_card: not_a_card.map(|e| NotACardJson {
            kind: e.kind(),
            message: e.message(),
        }),
        corrected_chunks,
        repairs: reports
            .iter()
            // Mirror toolkit: only include entries for chunks that
            // actually had corrections applied.
            .filter(|r| !r.corrected_positions.is_empty())
            .map(|r| RepairJsonDetail {
                chunk_index: r.chunk_index,
                original_chunk: r.original_chunk.as_str(),
                corrected_chunk: r.corrected_chunk.as_str(),
                corrected_positions: r
                    .corrected_positions
                    .iter()
                    .map(|(p, w, n)| RepairJsonPosition {
                        position: *p,
                        was: w.to_string(),
                        now: n.to_string(),
                    })
                    .collect(),
            })
            .collect(),
    };
    // cycle-15 Lane M (slug #8, defense-in-depth): the serialized envelope
    // carries the corrected secret ms1 — scrub the buffer on drop.
    let body: Zeroizing<String> = Zeroizing::new(
        serde_json::to_string(&envelope)
            .map_err(|e| CliError::Internal(format!("repair JSON serialize: {e}")))?,
    );
    println!("{}", *body);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::correct_only;
    use ms_codec::{Payload, Tag};

    /// Flip symbol `i` of `s` (after `ms1`) to a different alphabet symbol.
    fn damage(s: &str, positions: &[usize]) -> String {
        let mut b: Vec<u8> = s.bytes().collect();
        for &i in positions {
            let at = 3 + i;
            b[at] = if b[at] == b'q' { b'p' } else { b'q' };
        }
        String::from_utf8(b).unwrap()
    }

    /// N2 drift guard: `correct_only` re-implements the codec's BCH steps, so
    /// on strings the codec corrects AND decodes it must report the same
    /// corrections, on clean strings none, and on uncorrectable ones fail too.
    #[test]
    fn correct_only_matches_decode_with_correction() {
        let mut checked = 0;
        for len in [16usize, 20, 24, 28, 32] {
            let entropy: Vec<u8> = (0..len as u8).map(|i| i.wrapping_mul(37)).collect();
            let card = ms_codec::encode(Tag::ENTR, &Payload::Entr(entropy)).unwrap();
            let n = card.len() - 3;
            for positions in [
                vec![],
                vec![0],
                vec![n - 1],
                vec![5, 9],
                vec![1, n / 2, n - 2],
                vec![2, 7, 11, n - 4],
            ] {
                let s = damage(&card, &positions);
                let (_t, _p, want) = ms_codec::decode_with_correction(&s)
                    .unwrap_or_else(|e| panic!("{positions:?}: {e:?}"));
                assert_eq!(want.len(), positions.len());
                assert_eq!(correct_only(&s), Some(want), "{len} B {positions:?}");
                // Upper case is the same string to both.
                let up = s.to_ascii_uppercase();
                let (_t, _p, want) = ms_codec::decode_with_correction(&up).unwrap();
                assert_eq!(correct_only(&up), Some(want));
                checked += 1;
            }
            // Beyond the correction bound both give up.
            let s = damage(&card, &[0, 3, 6, 9, 12, 15]);
            assert!(ms_codec::decode_with_correction(&s).is_err());
            assert_eq!(correct_only(&s), None, "{len} B, six errors");
        }
        assert_eq!(checked, 30);
    }
}
