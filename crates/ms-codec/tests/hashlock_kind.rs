//! Kind rows for the `0x03` preimage kind (SPEC_ms_hashlock §1, §8).
//!
//! Every row names the door it enters by and the error it asserts: a row that
//! says "refused" without either passes on the wrong error.

use ms_codec::consts::{
    PREIMAGE_PREFIX, RESERVED_ID_BLOCKLIST, TAG_HASH, VALID_PREIMAGE_STR_LENGTHS,
};
use ms_codec::{decode, encode, Error, Payload, PayloadKind, Tag};
use zeroize::Zeroizing;

fn preimage(byte: u8) -> Payload {
    Payload::Preimage(Zeroizing::new([byte; 32]))
}

#[test]
fn constants_are_the_specs() {
    assert_eq!(PREIMAGE_PREFIX, 0x03);
    assert_eq!(TAG_HASH, *b"hash");
    assert_eq!(VALID_PREIMAGE_STR_LENGTHS, &[75]);
    assert_eq!(Tag::HASH.as_bytes(), b"hash");
    // Six entries: the five that shipped plus `hash` (spec §1 rule 3).
    assert_eq!(RESERVED_ID_BLOCKLIST.len(), 6);
    assert!(RESERVED_ID_BLOCKLIST.contains(b"hash"));
}

#[test]
fn single_tag_by_kind() {
    assert_eq!(PayloadKind::Entr.single_tag(), Tag::ENTR);
    assert_eq!(PayloadKind::Mnem.single_tag(), Tag::ENTR);
    assert_eq!(PayloadKind::Preimage.single_tag(), Tag::HASH);
}

#[test]
fn a_hash_single_round_trips_and_is_75_chars() {
    let s = encode(Tag::HASH, &preimage(0xab)).expect("encode");
    assert_eq!(s.len(), 75, "{s}");
    assert!(s.starts_with("ms10hashsq"), "{s}");
    let (tag, p) = decode(&s).expect("decode");
    assert_eq!(tag, Tag::HASH);
    assert_eq!(p.kind(), PayloadKind::Preimage);
    assert_eq!(p.as_bytes(), &[0xab; 32]);
}

#[test]
fn the_entr32_and_preimage_pair_are_adjacent_rows() {
    // Same length, same leading payload char; only the id differs.
    let e = encode(Tag::ENTR, &Payload::Entr(vec![0xab; 32])).unwrap();
    let h = encode(Tag::HASH, &preimage(0xab)).unwrap();
    assert_eq!(e.len(), 75);
    assert_eq!(h.len(), 75);
    assert!(e.starts_with("ms10entrsq"), "{e}");
    assert!(h.starts_with("ms10hashsq"), "{h}");
}

#[test]
fn id_and_prefix_must_agree_both_directions() {
    // id `hash` over a seed payload: encode refuses, and a hand-made string
    // is refused on decode -- never read as the other kind (spec §1 rule 2).
    let err = encode(Tag::HASH, &Payload::Entr(vec![0; 32])).unwrap_err();
    assert!(
        matches!(err, Error::TagKindMismatch { tag, prefix: 0x00 } if tag == *b"hash"),
        "{err:?}"
    );
    let err = encode(Tag::ENTR, &preimage(0)).unwrap_err();
    assert!(
        matches!(err, Error::TagKindMismatch { tag, prefix: 0x03 } if tag == *b"entr"),
        "{err:?}"
    );

    // Hand-made strings through the codex32 layer, bypassing encode's check.
    let forged_hash_over_seed = forge("hash", &{
        let mut v = vec![0x00u8];
        v.extend_from_slice(&[0xab; 32]);
        v
    });
    let err = decode(&forged_hash_over_seed).unwrap_err();
    assert!(
        matches!(err, Error::TagKindMismatch { prefix: 0x00, .. }),
        "{err:?}"
    );
    let forged_entr_over_preimage = forge("entr", &{
        let mut v = vec![PREIMAGE_PREFIX];
        v.extend_from_slice(&[0xab; 32]);
        v
    });
    let err = decode(&forged_entr_over_preimage).unwrap_err();
    assert!(
        matches!(err, Error::TagKindMismatch { prefix: 0x03, .. }),
        "{err:?}"
    );
}

/// Build a threshold-0 single with an arbitrary id over arbitrary payload
/// bytes, through the vendored codex32 layer -- the forger's door.
fn forge(id: &str, payload: &[u8]) -> String {
    ms_codec::codex32::Codex32String::from_seed("ms", 0, id, ms_codec::codex32::Fe::S, payload)
        .expect("from_seed")
        .to_string()
}
