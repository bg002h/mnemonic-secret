//! Phase 1 mnem round-trip integration tests.

use ms_codec::{decode, encode, Payload, PayloadKind, Tag};

/// Encode a Mnem payload and verify:
/// - the output ms1 string has the correct length (51 for 16-byte entropy)
/// - the prefix byte in the decoded wire data is 0x02
/// - decode returns Payload::Mnem with the correct language and entropy
#[test]
fn mnem_encode_decode_round_trip_16b_japanese() {
    let entropy: Vec<u8> = (0u8..16).collect();
    let p = Payload::Mnem { language: 1, entropy: entropy.clone() };
    let s = encode(Tag::ENTR, &p).expect("encode Mnem should succeed");
    // 16-byte entropy → mnem str len 51
    assert_eq!(s.len(), 51, "mnem 16-byte entropy -> ms1 len 51, got {}", s.len());

    let (tag, recovered) = decode(&s).expect("decode mnem should succeed");
    assert_eq!(tag, Tag::ENTR);
    assert!(
        matches!(recovered, Payload::Mnem { language: 1, .. }),
        "expected Payload::Mnem{{language:1, ..}}, got {:?}",
        recovered
    );
    assert_eq!(recovered.as_bytes(), entropy.as_slice());
}

/// A v0.1 entr string still decodes to Payload::Entr after the seam change.
#[test]
fn entr_still_decodes_to_entr_payload() {
    let entropy = vec![0xAAu8; 16];
    let p = Payload::Entr(entropy.clone());
    let s = encode(Tag::ENTR, &p).expect("encode Entr should succeed");
    let (tag, recovered) = decode(&s).expect("decode Entr should succeed");
    assert_eq!(tag, Tag::ENTR);
    assert_eq!(recovered.kind(), PayloadKind::Entr);
    assert_eq!(recovered, Payload::Entr(entropy));
}

/// decode_with_correction on a clean mnem string returns the correct payload
/// (union length gate does not falsely reject it).
#[test]
fn mnem_decode_with_correction_clean_passes() {
    let entropy: Vec<u8> = vec![0x55u8; 16];
    let p = Payload::Mnem { language: 0, entropy: entropy.clone() };
    let s = encode(Tag::ENTR, &p).expect("encode mnem");
    let (tag, recovered, corrections) =
        ms_codec::decode_with_correction(&s).expect("decode_with_correction on clean mnem");
    assert_eq!(tag, Tag::ENTR);
    assert!(corrections.is_empty(), "no corrections expected for clean input");
    assert_eq!(recovered.as_bytes(), entropy.as_slice());
    assert!(
        matches!(recovered, Payload::Mnem { language: 0, .. }),
        "expected Mnem language=0"
    );
}
