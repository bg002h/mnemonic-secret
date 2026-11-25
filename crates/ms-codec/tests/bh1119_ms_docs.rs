//! FIX_bughunt1119_ms_docs (bh-ms-I8, library half). Every test asserts the
//! behaviour the FIX document specifies.

use std::path::Path;

use ms_codec::{decode_with_correction, encode, Payload, Tag};

fn readme() -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(env!("CARGO_PKG_README"));
    let text = std::fs::read_to_string(p).expect("read ms-codec README");
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn bh1119_ms_i8_codec_readme_no_too_many_errors_guarantee() {
    let r = readme();
    assert!(
        !r.contains("When the input exceeds the `t=4` correction capacity, the call returns"),
        "README still promises TooManyErrors beyond t=4"
    );
    assert!(
        r.contains("corrects to a *different* valid string"),
        "README does not say a correction beyond t=4 can land on a different valid string"
    );
}

#[test]
fn bh1119_ms_i8_library_two_card_alias_characterisation() {
    let a = encode(Tag::ENTR, &Payload::Entr(vec![0u8; 16])).unwrap();
    let b = encode(Tag::ENTR, &Payload::Entr(vec![0x11u8; 16])).unwrap();
    assert_ne!(a, b);
    let mut chars: Vec<char> = b.chars().collect();
    let last = chars.len() - 1;
    chars[last] = if chars[last] == 'p' { 'q' } else { 'p' };
    let broken: String = chars.into_iter().collect();
    let (_tag, payload, details) = decode_with_correction(&broken).unwrap();
    assert_eq!(details.len(), 1);
    assert_eq!(payload, Payload::Entr(vec![0x11u8; 16]));
}
