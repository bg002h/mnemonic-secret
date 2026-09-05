//! The hashlock phrase's two channels and its one rule (SPEC_ms_hashlock §4.3).
//!
//! BYTES AS GIVEN. The reader is `Vec<u8>` over `read_to_end`, strips exactly
//! one trailing `\r?\n`, and does nothing else. It must never be
//! `parse::read_input` (strips all whitespace plus `-` and `,`) or
//! `parse::read_phrase_input` (trims and collapses runs): either silently
//! changes X while every codec vector still passes, and `-`/`,` are exactly
//! what diceware emits. A non-UTF-8 byte reaches the printable-ASCII rule and
//! is refused BY NAME, not by an io error (R0 r0 correctness M-6).
//!
//! THE RULE, identical on host and device: non-empty; every byte in
//! `0x20..=0x7E`; not ms1-shaped (tested on a normalised COPY, before the
//! cap, so a grouped plate string gets the `--in` remedy and not "too long");
//! at most 100 characters; not exactly 64 hex digits in either case (a pasted
//! preimage -- the remedy is `--hex`). Refusals name the rule and never echo
//! the phrase.

use zeroize::Zeroizing;

use crate::error::{CliError, Result};

/// Its own constant on each side, lockstep-pinned; NOT the device's
/// plate-legibility `passphrase.MaxLen` (review M-6).
pub const HASHLOCK_PHRASE_MAX_CHARS: usize = 100;

/// Why a phrase was refused. Each variant renders one sentence that names
/// the rule and, where one exists, the remedy.
#[derive(Debug, PartialEq, Eq)]
pub enum PhraseRefusal {
    Empty,
    NotPrintableAscii { byte: u8, at: usize },
    Ms1Shaped,
    TooLong { chars: usize },
    Hex64,
}

impl PhraseRefusal {
    pub fn message(&self) -> String {
        match self {
            PhraseRefusal::Empty => "the hashlock phrase is empty".to_string(),
            PhraseRefusal::NotPrintableAscii { byte, at } => format!(
                "the hashlock phrase must be printable ASCII (bytes 0x20..0x7E); byte 0x{byte:02x} at position {at} is not"
            ),
            PhraseRefusal::Ms1Shaped => "that is an ms1 string, not a hashlock phrase; pass it as the ms1 argument, `-` on stdin, or `--in FILE`".to_string(),
            PhraseRefusal::TooLong { chars } => format!(
                "the hashlock phrase is {chars} characters; at most {HASHLOCK_PHRASE_MAX_CHARS} are allowed"
            ),
            PhraseRefusal::Hex64 => "that is 64 hex characters -- a preimage, 32 bytes (64 hex characters), not a phrase; pass it with --hex".to_string(),
        }
    }
}

impl From<PhraseRefusal> for CliError {
    fn from(r: PhraseRefusal) -> Self {
        CliError::BadInput(r.message())
    }
}

/// Strip exactly one trailing `\n`, and one `\r` before it if present. Nothing else.
pub fn strip_one_trailing_newline(buf: &mut Vec<u8>) {
    if buf.last() == Some(&b'\n') {
        buf.pop();
        if buf.last() == Some(&b'\r') {
            buf.pop();
        }
    }
}

/// The one prompt line, printed iff stdin is a terminal. Split out so the
/// terminal branch is unit-tested without a pty (R0 r0 fidelity I-8).
pub fn prompt_if_terminal(is_tty: bool, stderr: &mut impl std::io::Write) {
    if is_tty {
        let _ = writeln!(stderr, "Type the hashlock phrase, then Enter.");
    }
}

/// Read the phrase bytes byte-verbatim, and END THE READ WHERE THE PROMPT SAYS
/// IT ENDS.
///
/// At a TERMINAL the read stops at the first `\n` (`read_until`), because the
/// prompt asks the operator to press Enter and Enter does not deliver EOF in
/// canonical mode: on `read_to_end` the tool sat there after Enter and only
/// Ctrl-D finished it, so the one prompt that exists to prevent a hang
/// instructed the action that could not end it (post-impl review I-1, measured
/// on a real pty). Nothing is lost by stopping at the newline: the phrase rule
/// refuses a phrase containing `\n` as a non-printable byte anyway.
///
/// On a PIPE the read stays `read_to_end`, so a phrase file with no trailing
/// newline still arrives whole and the byte-verbatim rule is unchanged.
///
/// Either way exactly one trailing `\r?\n` is stripped and nothing else.
pub fn read_phrase_from(
    is_tty: bool,
    reader: &mut impl std::io::BufRead,
) -> Result<Zeroizing<Vec<u8>>> {
    let mut buf: Zeroizing<Vec<u8>> = Zeroizing::new(Vec::new());
    let read = if is_tty {
        reader.read_until(b'\n', &mut buf)
    } else {
        reader.read_to_end(&mut buf)
    };
    read.map_err(|e| CliError::BadInput(format!("failed to read stdin: {e}")))?;
    strip_one_trailing_newline(&mut buf);
    Ok(buf)
}

/// `read_phrase_from` over the real stdin, with the prompt in front of it.
pub fn read_phrase_stdin() -> Result<Zeroizing<Vec<u8>>> {
    use std::io::IsTerminal;
    let stdin = std::io::stdin();
    let is_tty = stdin.is_terminal();
    prompt_if_terminal(is_tty, &mut std::io::stderr().lock());
    read_phrase_from(is_tty, &mut stdin.lock())
}

/// The rule. Order matters and is the spec's: empty, printable ASCII,
/// ms1-shape (BEFORE the cap), cap, 64-hex.
pub fn validate_phrase(bytes: &[u8]) -> std::result::Result<(), PhraseRefusal> {
    if bytes.is_empty() {
        return Err(PhraseRefusal::Empty);
    }
    if let Some((at, &byte)) = bytes
        .iter()
        .enumerate()
        .find(|(_, b)| !(0x20..=0x7e).contains(*b))
    {
        return Err(PhraseRefusal::NotPrintableAscii { byte, at });
    }
    // All bytes are printable ASCII now, so this is a &str.
    let s = std::str::from_utf8(bytes).expect("printable ASCII is UTF-8");
    if crate::argv_guard::looks_like_ms1(s) {
        return Err(PhraseRefusal::Ms1Shaped);
    }
    if s.len() > HASHLOCK_PHRASE_MAX_CHARS {
        return Err(PhraseRefusal::TooLong { chars: s.len() });
    }
    // The same predicate `--hex` parses with (the `hex` crate), so the two
    // cannot disagree about what a pasted preimage looks like (spec §4.3).
    if s.len() == 64 && hex::decode(s).is_ok() {
        return Err(PhraseRefusal::Hex64);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_exactly_one_newline() {
        let mut v = b"abc\r\n".to_vec();
        strip_one_trailing_newline(&mut v);
        assert_eq!(v, b"abc");
        let mut v = b"abc\n\n".to_vec();
        strip_one_trailing_newline(&mut v);
        assert_eq!(v, b"abc\n", "only ONE trailing newline is stripped");
        let mut v = b" abc ".to_vec();
        strip_one_trailing_newline(&mut v);
        assert_eq!(v, b" abc ", "spaces are bytes");
    }

    #[test]
    fn empty_is_refused() {
        assert_eq!(validate_phrase(b""), Err(PhraseRefusal::Empty));
    }

    #[test]
    fn printable_boundary_is_pinned_on_both_sides() {
        assert_eq!(validate_phrase(b" ~"), Ok(()));
        assert_eq!(
            validate_phrase(b"a\tb"),
            Err(PhraseRefusal::NotPrintableAscii { byte: 0x09, at: 1 })
        );
        assert_eq!(
            validate_phrase(b"a\x7f"),
            Err(PhraseRefusal::NotPrintableAscii { byte: 0x7f, at: 1 })
        );
        assert_eq!(
            validate_phrase(b"\xff"),
            Err(PhraseRefusal::NotPrintableAscii { byte: 0xff, at: 0 })
        );
        assert_eq!(
            validate_phrase("café".as_bytes()),
            Err(PhraseRefusal::NotPrintableAscii { byte: 0xc3, at: 3 })
        );
    }

    #[test]
    fn ms1_shape_in_four_spellings_and_before_the_cap() {
        // Shape only: the HRP, the id, the charset, 75 characters. The
        // checksum is wrong on purpose -- the shape test must not parse.
        let plate = format!("ms10hashsq{}", "q".repeat(65));
        assert_eq!(plate.len(), 75);
        assert_eq!(
            validate_phrase(plate.as_bytes()),
            Err(PhraseRefusal::Ms1Shaped),
            "lowercase"
        );
        assert_eq!(
            validate_phrase(plate.to_ascii_uppercase().as_bytes()),
            Err(PhraseRefusal::Ms1Shaped),
            "UPPERCASE"
        );
        let grouped: String = plate
            .as_bytes()
            .chunks(5)
            .map(|c| std::str::from_utf8(c).unwrap())
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(
            validate_phrase(grouped.as_bytes()),
            Err(PhraseRefusal::Ms1Shaped),
            "grouped"
        );
        assert_eq!(
            validate_phrase(format!("  {plate}  ").as_bytes()),
            Err(PhraseRefusal::Ms1Shaped),
            "padded"
        );
        let grouped2: String = plate
            .as_bytes()
            .chunks(2)
            .map(|c| std::str::from_utf8(c).unwrap())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(grouped2.len() > HASHLOCK_PHRASE_MAX_CHARS);
        assert_eq!(
            validate_phrase(grouped2.as_bytes()),
            Err(PhraseRefusal::Ms1Shaped),
            "shape test precedes the cap"
        );
    }

    #[test]
    fn cap_at_100() {
        assert_eq!(validate_phrase("a".repeat(100).as_bytes()), Ok(()));
        assert_eq!(
            validate_phrase("a".repeat(101).as_bytes()),
            Err(PhraseRefusal::TooLong { chars: 101 })
        );
    }

    #[test]
    fn hex64_either_case_refused_short_hex_accepted() {
        let lower = "c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016";
        assert_eq!(validate_phrase(lower.as_bytes()), Err(PhraseRefusal::Hex64));
        assert_eq!(
            validate_phrase(lower.to_ascii_uppercase().as_bytes()),
            Err(PhraseRefusal::Hex64)
        );
        assert_eq!(validate_phrase(b"beef"), Ok(()));
        assert_eq!(
            validate_phrase(&lower.as_bytes()[..63]),
            Ok(()),
            "63 hex characters is a phrase"
        );
    }

    #[test]
    fn prompt_only_at_a_terminal() {
        let mut tty = Vec::new();
        prompt_if_terminal(true, &mut tty);
        assert_eq!(
            String::from_utf8(tty).unwrap(),
            "Type the hashlock phrase, then Enter.\n"
        );
        let mut pipe = Vec::new();
        prompt_if_terminal(false, &mut pipe);
        assert!(
            pipe.is_empty(),
            "a pipe gets no prompt: it would land in the operator's output"
        );
    }

    /// A reader that yields its line and then PANICS if read again -- the shape
    /// of a terminal in canonical mode after Enter, where the next read blocks
    /// until EOF. `read_until(b'\n')` never asks for the second read;
    /// `read_to_end` always does, so a revert to it turns this test red instead
    /// of hanging a real operator (post-impl review I-1).
    struct OneLineThenNeverEof {
        line: &'static [u8],
        pos: usize,
    }

    impl std::io::Read for OneLineThenNeverEof {
        fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
            assert!(
                self.pos < self.line.len(),
                "read past the first line: the terminal branch would block here, \
                 which is exactly the hang the prompt promises not to cause"
            );
            let n = std::cmp::min(out.len(), self.line.len() - self.pos);
            out[..n].copy_from_slice(&self.line[self.pos..self.pos + n]);
            self.pos += n;
            Ok(n)
        }
    }

    #[test]
    fn a_terminal_read_ends_at_the_newline_the_prompt_asks_for() {
        let mut r = std::io::BufReader::new(OneLineThenNeverEof {
            line: b"correct horse battery staple\n",
            pos: 0,
        });
        let got =
            read_phrase_from(true, &mut r).expect("the tty branch must return after one line");
        assert_eq!(&got[..], b"correct horse battery staple");
    }

    #[test]
    fn a_pipe_read_still_runs_to_eof() {
        // Unchanged for a pipe: everything arrives, including embedded
        // newlines (which the phrase rule then refuses), and exactly one
        // trailing newline is stripped.
        let mut r = std::io::BufReader::new(std::io::Cursor::new(b"a\nb\n".to_vec()));
        let got = read_phrase_from(false, &mut r).unwrap();
        assert_eq!(&got[..], b"a\nb");
        let mut r = std::io::BufReader::new(std::io::Cursor::new(b"no trailing newline".to_vec()));
        let got = read_phrase_from(false, &mut r).unwrap();
        assert_eq!(&got[..], b"no trailing newline");
    }

    #[test]
    fn refusals_never_echo_the_phrase() {
        let secret = "my very secret phrase\t";
        let msg = validate_phrase(secret.as_bytes()).unwrap_err().message();
        assert!(!msg.contains("my very secret"), "{msg}");
    }
}
