//! v0.1 wire-format constants.

/// HRP for ms1 strings (BIP-93 codex32 HRP).
pub const HRP: &str = "ms";

/// BIP-93 separator character.
pub const SEPARATOR: char = '1';

/// v0.1 reserved-prefix byte (becomes the v0.2 type discriminator).
pub const RESERVED_PREFIX: u8 = 0x00;

/// v0.1 emit-side threshold value (ASCII).
pub const THRESHOLD_V01: u8 = b'0';

/// v0.1 emit-side share-index value (ASCII; "s" denotes the unshared secret per BIP-93).
pub const SHARE_INDEX_V01: u8 = b's';

/// Short codex32 checksum length in characters.
pub const CHECKSUM_LEN_SHORT: usize = 13;

/// Allowed v0.1 entr entropy byte lengths (bijective with BIP-39 word counts {12,15,18,21,24}).
pub const VALID_ENTR_LENGTHS: &[usize] = &[16, 20, 24, 28, 32];

/// Allowed v0.1 total ms1 string lengths (HRP+sep+threshold+id+share+payload+cksum).
/// Computed: 9 fixed + ceil((entropy_bytes + 1) * 8 / 5) payload symbols + 13 cksum.
pub const VALID_STR_LENGTHS: &[usize] = &[50, 56, 62, 69, 75];

/// 4-byte type tag — v0.1 emit (also accept).
pub const TAG_ENTR: [u8; 4] = *b"entr";

/// 4-byte type tags reserved-not-emitted in v0.1 (decoder rejects).
pub const RESERVED_NOT_EMITTED_V01: &[[u8; 4]] = &[*b"seed", *b"xprv", *b"mnem", *b"prvk"];

#[cfg(test)]
mod tests {
    use super::*;

    /// Locks the bijection between VALID_ENTR_LENGTHS and VALID_STR_LENGTHS so
    /// that a future edit to one without the other fails CI loudly.
    /// Formula per SPEC §2.4: total = 9 fixed (HRP+sep+threshold+id+share) +
    /// ceil((entropy_bytes + 1) * 8 / 5) payload symbols + 13 short checksum.
    #[test]
    fn valid_str_lengths_match_entr_lengths_via_bijection() {
        assert_eq!(VALID_ENTR_LENGTHS.len(), VALID_STR_LENGTHS.len());
        for (i, &entropy_bytes) in VALID_ENTR_LENGTHS.iter().enumerate() {
            let data_bits = (entropy_bytes + 1) * 8; // +1 for the 0x00 prefix byte
            let payload_symbols = (data_bits + 4) / 5; // ceil(bits/5)
            let total = 9 + payload_symbols + CHECKSUM_LEN_SHORT;
            assert_eq!(
                total, VALID_STR_LENGTHS[i],
                "entropy {} B -> expected str.len {}, got {} (bijection drift)",
                entropy_bytes, VALID_STR_LENGTHS[i], total
            );
        }
    }
}
