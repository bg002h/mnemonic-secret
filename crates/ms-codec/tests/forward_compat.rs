//! SPEC §10.2 forward-compat smoke test: encode a v0.1 string, manually flip
//! the prefix byte to 0x01, confirm decoder rejects with
//! Error::ReservedPrefixViolation. Locks the v0.1 ↔ v0.2 contract.

use codex32::{Codex32String, Fe};
use ms_codec::{decode, encode, Error, Payload, Tag};

#[test]
fn flipping_prefix_byte_to_v02_value_rejects_at_v01_decoder() {
    // Encode a real v0.1 string.
    let entropy = vec![0xAAu8; 16];
    let _s_v01 = encode(Tag::ENTR, &Payload::Entr(entropy.clone())).unwrap();

    // Hand-build the same wire shape but with prefix byte = 0x01 (the future v0.2
    // entr discriminator). v0.1 decoder MUST reject this — that's the migration
    // contract from SPEC §5 invariant #1.
    let mut data = vec![0x01u8];
    data.extend_from_slice(&entropy);
    let c = Codex32String::from_seed("ms", 0, "entr", Fe::S, &data).unwrap();
    let s_v02_shaped = c.to_string();

    assert_eq!(s_v02_shaped.len(), 50);
    assert!(matches!(
        decode(&s_v02_shaped),
        Err(Error::ReservedPrefixViolation { got: 0x01 })
    ));
}

#[test]
fn all_non_zero_prefix_bytes_rejected_in_v01() {
    // Defense-in-depth: every non-zero prefix value is rejected, not just 0x01.
    let entropy = [0xAAu8; 16];
    for prefix in 1u8..=255 {
        let mut data = vec![prefix];
        data.extend_from_slice(&entropy);
        let c = Codex32String::from_seed("ms", 0, "entr", Fe::S, &data).unwrap();
        let err = decode(&c.to_string()).unwrap_err();
        assert!(
            matches!(err, Error::ReservedPrefixViolation { got } if got == prefix),
            "prefix 0x{:02x}: expected ReservedPrefixViolation, got {:?}",
            prefix,
            err
        );
    }
}
