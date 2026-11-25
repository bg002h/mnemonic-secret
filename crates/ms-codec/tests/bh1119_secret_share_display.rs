//! FIX #46 (bh-ms-R11): `Display` of `Error::IsShareNotSingleString` at share index `s`.
//! Test vectors only (all-zero data, id `test`).

use ms_codec::Error;

const S2: &str = "ms12testsqqqqqqqqqqqqqqqqqqqqqqqqqqqqkwe5qlrp6ykwa";
const A2: &str = "ms12testaqqqqqqqqqqqqqqqqqqqqqqqqqqqqzfvk7urqpuwd0";

#[test]
fn codec_display_secret_share_names_secret() {
    let e = ms_codec::decode(S2).unwrap_err();
    assert!(matches!(
        e,
        Error::IsShareNotSingleString {
            threshold: '2',
            index: 's'
        }
    ));
    assert_eq!(
        e.to_string(),
        "this is the secret share of a K-of-N set (threshold '2', index 's'); it is the secret itself, not a distributed share, and `ms combine` does not take it"
    );
}

#[test]
fn codec_display_distributed_share_keeps_combine_advice() {
    let e = ms_codec::decode(A2).unwrap_err();
    assert_eq!(
        e.to_string(),
        "this is one share of a K-of-N set (threshold '2', index 'a'); use `ms combine` to recombine K shares"
    );
}
