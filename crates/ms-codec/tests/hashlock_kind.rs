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
#[test]
fn preimage_length_rows_through_decode_name_their_error() {
    // The wrong-length set that reaches prefix dispatch through `decode` is
    // exactly these nine (spec §1: 22 + ceil(8N/5) lands in the union length
    // set). `got` is the byte count AFTER the prefix.
    for n in [17usize, 18, 21, 22, 25, 26, 29, 30, 34] {
        let mut payload = vec![PREIMAGE_PREFIX];
        payload.extend(std::iter::repeat(0xab).take(n - 1));
        let s = forge("hash", &payload);
        let err = decode(&s).unwrap_err();
        assert!(
            matches!(err, Error::PreimageLengthMismatch { got } if got == n - 1),
            "payload {n} bytes ({} chars): {err:?}",
            s.len()
        );
    }
}

/// I-2 (R0 r0 fidelity): 0x03 left forward_compat.rs's "every undefined prefix
/// is refused" loop; this is what it does instead.
#[test]
fn preimage_prefix_is_refused_by_length_not_prefix() {
    let mut payload = vec![PREIMAGE_PREFIX];
    payload.extend_from_slice(&[0xab; 16]);
    let s = forge("hash", &payload);
    let err = decode(&s).unwrap_err();
    assert!(
        matches!(err, Error::PreimageLengthMismatch { got: 16 }),
        "{err:?}"
    );
}

#[test]
fn preimage_length_rows_refused_by_the_string_gate_first() {
    // 16, 32 and 44 never reach prefix dispatch through `decode`: their
    // strings (48, 74, 93 chars) are outside the union length set.
    for (n, chars) in [(16usize, 48usize), (32, 74), (44, 93)] {
        let mut payload = vec![PREIMAGE_PREFIX];
        payload.extend(std::iter::repeat(0xab).take(n - 1));
        let s = forge("hash", &payload);
        assert_eq!(s.len(), chars);
        let err = decode(&s).unwrap_err();
        assert!(
            matches!(err, Error::UnexpectedStringLength { got, .. } if got == chars),
            "{n}: {err:?}"
        );
    }
}

#[test]
fn preimage_length_rows_through_combine_shares() {
    // The share path has no string-length gate, so 16, 32 and 44 reach
    // `PreimageLengthMismatch` here. Build a 2-of-2 set over a bad payload by
    // hand through the codex32 layer and recombine.
    for n in [16usize, 32, 44] {
        let mut secret = vec![PREIMAGE_PREFIX];
        secret.extend(std::iter::repeat(0xab).take(n - 1));
        let shares = forge_shares(&secret, 2, 2);
        let err = ms_codec::combine_shares(&shares).unwrap_err();
        assert!(
            matches!(err, Error::PreimageLengthMismatch { got } if got == n - 1),
            "payload {n} bytes via combine: {err:?}"
        );
    }
}

#[test]
fn a_46_byte_payload_is_unconstructible() {
    let mut payload = vec![PREIMAGE_PREFIX];
    payload.extend(std::iter::repeat(0xab).take(45));
    let s = ms_codec::codex32::Codex32String::from_seed(
        "ms",
        0,
        "hash",
        ms_codec::codex32::Fe::S,
        &payload,
    )
    .expect("from_seed")
    .to_string();
    assert_eq!(s.len(), 96);
    assert!(
        ms_codec::codex32::Codex32String::from_string(s).is_err(),
        "96 chars is outside both brackets"
    );
}

#[test]
fn preimage_share_round_trip() {
    let secret = preimage(0x5a);
    let shares =
        ms_codec::encode_shares(Tag::HASH, ms_codec::Threshold::new(2).unwrap(), 3, &secret)
            .unwrap();
    for pair in [[0, 1], [0, 2], [1, 2]] {
        let (_tag, p) =
            ms_codec::combine_shares(&[shares[pair[0]].clone(), shares[pair[1]].clone()]).unwrap();
        assert_eq!(p, secret);
    }
}

/// The variant's field is `Zeroizing<[u8; 32]>` (spec §3): a type-level
/// assertion the compiler enforces, so a refactor to a bare array fails to
/// build rather than silently losing the scrub-on-drop.
#[test]
fn preimage_field_is_zeroizing() {
    let p = preimage(0x42);
    if let Payload::Preimage(z) = &p {
        let _: &Zeroizing<[u8; 32]> = z;
        assert_eq!(z.len(), 32);
    } else {
        panic!("not a preimage");
    }
}

#[test]
fn inspect_reports_the_kind() {
    let s = encode(Tag::HASH, &preimage(0x11)).unwrap();
    let r = ms_codec::inspect(&s).unwrap();
    assert_eq!(r.kind, ms_codec::InspectKind::Preimage);
    assert_eq!(r.prefix_byte, PREIMAGE_PREFIX);
    assert_eq!(r.tag, Tag::HASH);
}

#[test]
fn codeword_distance_between_entr_and_hash_ids_exceeds_the_correction_bound() {
    // Spec §1: measured, not inherited. BIP-93 corrects up to 4 errors; two
    // codewords that could be confused by a correction must be > 8 apart.
    let payload = {
        let mut v = vec![PREIMAGE_PREFIX];
        v.extend_from_slice(&[0xab; 32]);
        v
    };
    let a = forge("entr", &payload);
    let b = forge("hash", &payload);
    let distance = a.bytes().zip(b.bytes()).filter(|(x, y)| x != y).count();
    println!("codeword distance entr/hash = {distance}");
    assert!(
        distance > 8,
        "distance {distance} is within twice the correction bound"
    );
}

/// A 2-of-N share set over raw payload bytes, through the codex32 layer, so a
/// wrong-length payload can be recombined without `encode_shares` refusing it
/// (a `Payload::Preimage` cannot even be built at the wrong length, which is
/// the point of the variant). Two points fix the polynomial: the secret at
/// `S` and one random share at `A`; every other index is interpolated.
fn forge_shares(secret: &[u8], k: usize, n: usize) -> Vec<String> {
    use ms_codec::codex32::{Codex32String, Fe};
    assert_eq!(k, 2, "this forger builds 2-of-N sets");
    let s = Codex32String::from_seed("ms", k, "zzzz", Fe::S, secret).expect("secret at S");
    let mut rnd = vec![0u8; secret.len()];
    for (i, b) in rnd.iter_mut().enumerate() {
        *b = (i as u8).wrapping_mul(37).wrapping_add(11);
    }
    let a = Codex32String::from_seed("ms", k, "zzzz", Fe::A, &rnd).expect("share at A");
    let mut out = vec![a.to_string()];
    for target in [Fe::C, Fe::D].iter().take(n - 1) {
        out.push(
            Codex32String::interpolate_at(&[s.clone(), a.clone()], *target)
                .expect("interpolate")
                .to_string(),
        );
    }
    out
}
