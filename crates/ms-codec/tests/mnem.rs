//! Phase 1 mnem round-trip integration tests + wire-correctness golden vector.

use ms_codec::{decode, encode, Payload, PayloadKind, Tag};

/// Encode a Mnem payload and verify:
/// - the output ms1 string has the correct length (51 for 16-byte entropy)
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
/// This is the entr byte-identity guard: the 0x00 path must be UNCHANGED.
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
/// (union length gate does not falsely reject a mnem string through this path).
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

/// Wire-correctness golden vector: English (language=0) + fixed 16-byte entropy.
///
/// Pinned by running the encoder ONCE and recording the output. This guards
/// against a self-consistent-but-wrong packing regression: if the wire layout
/// changes (e.g. prefix order, language-byte position), this test fails
/// loudly even if every internal round-trip still passes.
///
/// Captured on branch ms-v0.2-kofn-mnem at commit c66ca2e (Phase 1 seam change).
/// Entropy (hex): 0c1e24e5917544d666c342992acfda1b
/// Language byte: 0x00 (English)
/// On-wire payload: [0x02][0x00][entropy_16_bytes] = 18 bytes
/// Expected ms1 string length: 51 (per VALID_MNEM_STR_LENGTHS[0])
#[test]
fn golden_mnem_english_16b_wire_vector() {
    let entropy: Vec<u8> = vec![
        0x0c, 0x1e, 0x24, 0xe5, 0x91, 0x75, 0x44, 0xd6,
        0x66, 0xc3, 0x42, 0x99, 0x2a, 0xcf, 0xda, 0x1b,
    ];
    let p = Payload::Mnem { language: 0, entropy: entropy.clone() };
    let s = encode(Tag::ENTR, &p).expect("encode mnem golden");

    // Pin the exact wire string byte-for-byte.
    assert_eq!(
        s,
        "ms10entrsqgqqc83yukgh23xkvmp59xf2eldpk4cdrq2y4h82yz",
        "mnem wire encoding drifted from golden vector"
    );
    assert_eq!(s.len(), 51);

    // Also verify it decodes back correctly.
    let (tag, recovered) = decode(&s).expect("decode golden");
    assert_eq!(tag, Tag::ENTR);
    assert_eq!(recovered, Payload::Mnem { language: 0, entropy });
}
