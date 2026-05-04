//! Friendly human-readable messages for `bip39::Error` variants.
//!
//! Realizes SPEC §6.2. Variant set verified against `bip39 = "2"` per
//! Phase 1 task 1 spike.

use bip39::Error;

/// Map each `bip39::Error` variant to a one-line user-facing message.
pub fn friendly_bip39(e: &Error) -> String {
    match e {
        Error::BadEntropyBitCount(n) => format!(
            "BIP-39 entropy bit count {} invalid (must be 128, 160, 192, 224, or 256)",
            n
        ),
        Error::BadWordCount(n) => format!(
            "BIP-39 word count {} invalid (must be 12, 15, 18, 21, or 24)",
            n
        ),
        Error::UnknownWord(idx) => format!(
            "unknown BIP-39 word at position {} (not in selected wordlist; did you pick the right --language?)",
            idx
        ),
        Error::InvalidChecksum => "BIP-39 checksum failure (last word does not match the entropy)".to_string(),
        Error::AmbiguousLanguages(_) => {
            "BIP-39 phrase parses under multiple wordlists; specify --language explicitly"
                .to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bad_word_count_message_helpful() {
        let msg = friendly_bip39(&Error::BadWordCount(13));
        assert!(msg.contains("13"));
        assert!(msg.contains("12"));
    }

    #[test]
    fn unknown_word_mentions_language() {
        let msg = friendly_bip39(&Error::UnknownWord(5));
        assert!(msg.contains("--language"));
    }

    #[test]
    fn bad_checksum_message_concise() {
        let msg = friendly_bip39(&Error::InvalidChecksum);
        assert!(msg.contains("BIP-39 checksum failure"));
    }
}
