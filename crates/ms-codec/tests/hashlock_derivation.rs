//! Derivation rows, both methods, each pinning X AND H (SPEC_ms_hashlock §2,
//! §8). Every literal below was produced OUTSIDE this crate -- python3
//! hashlib and openssl kdf, cross-checked -- so a row is a correctness pin,
//! not a regression pin. `hashlock_repro.rs` re-runs those tools in CI.

use ms_codec::hashlock::{
    digest, preimage_hardened, preimage_sha256, HASHLOCK_DKLEN, HASHLOCK_ITERATIONS, HASHLOCK_SALT,
};

fn hex(b: &[u8]) -> String {
    use std::fmt::Write;
    b.iter()
        .fold(String::with_capacity(b.len() * 2), |mut s, x| {
            let _ = write!(s, "{x:02x}");
            s
        })
}

#[test]
fn constants_are_the_specs() {
    assert_eq!(HASHLOCK_SALT, b"ms-hashlock-v1");
    assert_eq!(HASHLOCK_ITERATIONS, 100_000);
    assert_eq!(HASHLOCK_DKLEN, 32);
}

/// (phrase, hardened X, hardened H, sha256 X, sha256 H) -- every row produced
/// OUTSIDE this crate by python3 hashlib, the hardened X of three rows
/// cross-checked in openssl kdf, and the whole set re-derived from the corpus
/// file by `corpus_rows_are_filled_and_re_derive` below.
const ROWS: &[(&str, &str, &str, &str, &str)] = &[
    ("correct horse battery staple", "c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016", "3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12", "c4bbcb1fbec99d65bf59d85c8cb62ee2db963f0fe106f483d9afa73bd4e39a8a", "b867db875479bcc0287352cdaa4a1755689b8338777d0915e9acd9f6edbc96cb"),
    ("z", "eda31187ec20d855d85cb69d94abac1c55b8996819d6ce3dc6cc17f79f6dd3e2", "af384a82ac8ff16b69a24392f1adc40966ab22923ae2b06d5ebc8ea6a5453b3a", "594e519ae499312b29433b7dd8a97ff068defcba9755b6d5d00e84c524d67b06", "c27cd49cb724724842a58b799b1009ecc968b3499767b73ee54693661ff723ca"),
    ("twenty characters!!!", "c9c45a47783e7cfbe4773d76a0f282d02ad077bc32d863a5b78e9fb134d0503c", "f00137a8ecf4f1b6acb592a7d00085ab30a738d936996417df098fe6d39eb4a2", "e8bf4723478e5d324b4ce75009b82a9b60ce5d4233a43e656c2ff7e4f8cba7f8", "5b891cd8cd226400ddcf25419847487f0954fc197640b6e6e5074dfb3b1bdde4"),
    ("hashlock phrase row: sixty-four printable characters, no hex!!xx", "72bd30bb4280d8db4a1db45f18ef5e03313a30d7e2440b2abe4b39ff23b62a96", "bd10cd48bffc544fa3c42cb8577db646f8603135479d73217b564e5be57b58fd", "ef2d8e668e2172c6fea55ac565db83db434cdb993bdee43e3dad3e398cd61b60", "895d7861d3c8f40ca177e30e4ac8e30004a15706cdd549aa04822c00126ec335"),
    ("hashlock phrase row: sixty-four printable characters, no hex!!xx!", "81659f096958cceca503b18498a2abe861ccd93789801c42f031f96d0ea7c9c7", "4a84ddc8d54b05c8d06cf5ba610c25a2e14ab83f79ada662a7504c1f37bf6984", "2f437817e6039e03d66badc8808dfaaf74adb0cbafdebc1f95c45e0e8fdc856b", "671c81426fa67f7b590f78f143173faeb56799da21dd316aa2762dac6ac64ff1"),
    ("hashlock phrase row: one hundred printable characters kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk", "048a9101a6c2d4d2f41a64d3cfc2fa63717eafb99ddc2c0b94183605ffd97ece", "70a5395386c769019faa4996aa61510f7760a1b32d6980173ccc57b3e68b4525", "4847734befcd471f090bfb87ea23c13e2a80dccd973dffb301be6844c53a5251", "76001f8e456719bd4d1e560ff28be3c4d75db624779079809da422607f31cde4"),
    ("hashlock phrase row: one hundred printable characters kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk!", "abe28ff3905421a9f8caae476f3685555bb94a11c55e767d3bd979e9e46f6a57", "ee37d62cd1e715c6b49cb76403aa2605c468db9c67386a7065dc6cae0b1c003b", "7c898a67500677c6d58452d6b01361c53f482c13450c9609130f84d9018a80ad", "f87e53adb8fd3338ad727cbc677be5b75b028ff481695e05d46fb33e0d8fb8b8"),
    ("  a  b ", "cae9f5663350a86462a194015516655846bc6880f134e156227e582323e0146b", "07ca621d2310d284d214f8894bc35100f467a39f9fb8155620d3a3f0d65941f6", "2438381a3894dfca639406f8a9677057050c098ad4d36ae8109db731adbb9574", "5f74bd9f51c2e64d0099927da9e472bd97bfe63537667b8a8e1cf4d4b294fe69"),
    ("correct-horse,battery staple", "4a48398f2814a30100fc29db21f2c2640774b86068bd2aa115ecb0ea3c5f5449", "528a12a16588e00171dc83975a4a511815ff33ff43788abf88f780051af350df", "6c76839064b97076384507503d4b987312c58a2fbd68d5854dee0765b03d42dc", "c0ed353a4b7f36a2802940f473a06c43c3b64c1246c58118da9e09b5ebfdf468"),
    ("a-b,c", "79324e188fd4935ef23dd5e1aa31e00cbe0d597558cea1dcd5e6a815b169900f", "8680bbf9e00acff491b41ed5ca0e6ea7c3530260690f2ea7a1145e3ac1841c37", "7a7fc2a0bffae80552a53f00a170f459d777b8b27857993fd463950ffe7fcbb7", "082f6172bde9ae5667a2493e75437dd839cc472ff54c311873aa3cb889a9fe16"),
    ("Correct Horse Battery Staple", "865125fb7ee922748fe3a53fbbf0917affce472877eb537482092572301fe650", "36d5ad9d6ec2a7bbaaa5e2ca641698f2301392076faa0c3fb0ad50f828cacea2", "af139fa284364215adfa49c889ab7feddc5e5d1c52512ffb2cfc9baeb67f220e", "95d4447031cdc4117f797040c1a9e32367af2a8d97554e442c7bfd002297a7ff"),
];

#[test]
fn anchor_rows_both_methods_pin_x_and_h() {
    for (phrase, hx, hh, sx, sh) in ROWS {
        let x = preimage_hardened(phrase.as_bytes());
        assert_eq!(hex(&x[..]), *hx, "hardened X for {phrase:?}");
        assert_eq!(hex(&digest(&x)), *hh, "hardened H for {phrase:?}");
        let x = preimage_sha256(phrase.as_bytes());
        assert_eq!(hex(&x[..]), *sx, "sha256 X for {phrase:?}");
        assert_eq!(hex(&digest(&x)), *sh, "sha256 H for {phrase:?}");
    }
}

#[test]
fn the_two_methods_differ_on_every_row() {
    for (phrase, hx, _, sx, _) in ROWS {
        assert_ne!(
            hx, sx,
            "{phrase:?}: a swap of the two methods must be visible"
        );
    }
}

#[test]
fn bytes_are_used_verbatim() {
    // A trailing space changes X. If the codec ever trimmed, this fails.
    let a = preimage_sha256(b"a");
    let b = preimage_sha256(b"a ");
    assert_ne!(&a[..], &b[..]);
    let a = preimage_hardened(b"a-b,c");
    let b = preimage_hardened(b"abc");
    assert_ne!(&a[..], &b[..], "hyphen and comma are bytes, not separators");
}

#[test]
fn case_is_bytes_too() {
    // Spec §4.3: no case folding anywhere on the phrase (R0 r0 tests I-1).
    assert_ne!(
        &preimage_hardened(b"Abc")[..],
        &preimage_hardened(b"abc")[..]
    );
    assert_ne!(&preimage_sha256(b"Abc")[..], &preimage_sha256(b"abc")[..]);
}

/// The corpus FILE is loaded and every derivation row re-derived, so a row
/// left as a placeholder, or a value that drifted from the crate, fails here
/// -- nothing else loads the file (R0 r0 tests I-2).
#[test]
fn corpus_rows_are_filled_and_re_derive() {
    let raw = include_str!("vectors/hashlock-v0.8.json");
    let v: serde_json::Value = serde_json::from_str(raw).expect("corpus parses");
    let is_hex64 = |s: &str| {
        s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    };
    let rows = v["derivation"].as_array().expect("derivation rows");
    assert!(rows.len() >= 11, "the corpus lost rows: {}", rows.len());
    for r in rows {
        let phrase = r["phrase"].as_str().expect("phrase is a literal string");
        assert_eq!(
            r["phrase_chars"].as_u64().unwrap() as usize,
            phrase.len(),
            "{phrase:?}: phrase_chars"
        );
        for k in ["hardened_x", "hardened_h", "sha256_x", "sha256_h"] {
            let s = r[k].as_str().unwrap_or("");
            assert!(is_hex64(s), "{phrase:?}: {k} is not 64 lowercase hex (a placeholder left in the corpus?): {s:?}");
        }
        let x = preimage_hardened(phrase.as_bytes());
        assert_eq!(hex(&x[..]), r["hardened_x"], "{phrase:?}: hardened X");
        assert_eq!(hex(&digest(&x)), r["hardened_h"], "{phrase:?}: hardened H");
        let x = preimage_sha256(phrase.as_bytes());
        assert_eq!(hex(&x[..]), r["sha256_x"], "{phrase:?}: sha256 X");
        assert_eq!(hex(&digest(&x)), r["sha256_h"], "{phrase:?}: sha256 H");
    }
    // The kind row: the plate string and its entr-32 pair are the codec's own.
    let k0 = &v["kind"][0];
    let mut x = [0u8; 32];
    for (i, b) in x.iter_mut().enumerate() {
        *b = u8::from_str_radix(&k0["preimage_hex"].as_str().unwrap()[2 * i..2 * i + 2], 16)
            .unwrap();
    }
    let plate = ms_codec::encode(
        ms_codec::Tag::HASH,
        &ms_codec::Payload::Preimage(zeroize::Zeroizing::new(x)),
    )
    .unwrap();
    assert_eq!(plate, k0["ms1"].as_str().unwrap());
    let pair = ms_codec::encode(ms_codec::Tag::ENTR, &ms_codec::Payload::Entr(x.to_vec())).unwrap();
    assert_eq!(pair, k0["entr32_pair_ms1"].as_str().unwrap());
    assert_eq!(hex(&digest(&x)), k0["digest"].as_str().unwrap());
}

#[test]
fn random_preimages_differ_and_are_32_bytes() {
    let a = ms_codec::hashlock::preimage_random().expect("os randomness");
    let b = ms_codec::hashlock::preimage_random().expect("os randomness");
    assert_ne!(&a[..], &b[..]);
    assert_eq!(a.len(), 32);
}

#[test]
fn digest_is_sha256_of_x() {
    // sha256 of 32 zero bytes, a public constant.
    let x = [0u8; 32];
    assert_eq!(
        hex(&digest(&x)),
        "66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925"
    );
}
