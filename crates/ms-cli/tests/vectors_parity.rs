//! Vector corpus parity: ms-cli's in-tree copy must JSON-equal ms-codec's canonical corpus.
//!
//! Per SPEC §10.2: parsed-equality, not byte-equality (avoids whitespace
//! / line-ending fragility).

#[test]
fn vectors_corpus_parity_with_ms_codec() {
    let cli_corpus: serde_json::Value = serde_json::from_str(include_str!("../vectors/v0.1.json"))
        .expect("ms-cli vectors corpus parses as JSON");
    let codec_corpus: serde_json::Value =
        serde_json::from_str(include_str!("../../ms-codec/tests/vectors/v0.1.json"))
            .expect("ms-codec vectors corpus parses as JSON");
    assert_eq!(
        cli_corpus, codec_corpus,
        "vectors corpus drifted between ms-cli and ms-codec"
    );
}
