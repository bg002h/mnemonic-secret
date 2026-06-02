//! BIP-39 wordlist language enum — clap value-enum + From<bip39::Language>.
//!
//! Realizes SPEC §7 (10 BIP-39 wordlists, kebab-case CLI values).

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

/// CLI-facing BIP-39 wordlist language.
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum CliLanguage {
    English,
    Japanese,
    Korean,
    Spanish,
    ChineseSimplified,
    ChineseTraditional,
    French,
    Italian,
    Czech,
    Portuguese,
}

impl CliLanguage {
    /// Wire language code (0 = English … 9 = Portuguese).
    /// Index into `ms_codec::consts::MNEM_LANGUAGE_NAMES`.
    pub fn code(self) -> u8 {
        self as u8
    }

    /// Reverse map from wire language code to `CliLanguage`.
    /// Returns `None` for codes ≥ 10 (unknown/future).
    pub fn from_code(c: u8) -> Option<CliLanguage> {
        match c {
            0 => Some(CliLanguage::English),
            1 => Some(CliLanguage::Japanese),
            2 => Some(CliLanguage::Korean),
            3 => Some(CliLanguage::Spanish),
            4 => Some(CliLanguage::ChineseSimplified),
            5 => Some(CliLanguage::ChineseTraditional),
            6 => Some(CliLanguage::French),
            7 => Some(CliLanguage::Italian),
            8 => Some(CliLanguage::Czech),
            9 => Some(CliLanguage::Portuguese),
            _ => None,
        }
    }

    /// Stable kebab-case name (for stderr / JSON output).
    pub fn as_str(self) -> &'static str {
        match self {
            CliLanguage::English => "english",
            CliLanguage::Japanese => "japanese",
            CliLanguage::Korean => "korean",
            CliLanguage::Spanish => "spanish",
            CliLanguage::ChineseSimplified => "chinese-simplified",
            CliLanguage::ChineseTraditional => "chinese-traditional",
            CliLanguage::French => "french",
            CliLanguage::Italian => "italian",
            CliLanguage::Czech => "czech",
            CliLanguage::Portuguese => "portuguese",
        }
    }
}

impl From<CliLanguage> for bip39::Language {
    fn from(l: CliLanguage) -> Self {
        match l {
            CliLanguage::English => bip39::Language::English,
            CliLanguage::Japanese => bip39::Language::Japanese,
            CliLanguage::Korean => bip39::Language::Korean,
            CliLanguage::Spanish => bip39::Language::Spanish,
            CliLanguage::ChineseSimplified => bip39::Language::SimplifiedChinese,
            CliLanguage::ChineseTraditional => bip39::Language::TraditionalChinese,
            CliLanguage::French => bip39::Language::French,
            CliLanguage::Italian => bip39::Language::Italian,
            CliLanguage::Czech => bip39::Language::Czech,
            CliLanguage::Portuguese => bip39::Language::Portuguese,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ms_codec::consts::MNEM_LANGUAGE_NAMES;

    /// Verifies that CliLanguage declaration order exactly matches
    /// MNEM_LANGUAGE_NAMES index order, and that from_code round-trips.
    /// This is the load-bearing invariant for Phase 2 mnem wire encoding.
    #[test]
    fn code_matches_mnem_language_names_and_from_code_round_trips() {
        let all_variants = [
            (CliLanguage::English, "english"),
            (CliLanguage::Japanese, "japanese"),
            (CliLanguage::Korean, "korean"),
            (CliLanguage::Spanish, "spanish"),
            (CliLanguage::ChineseSimplified, "chinese-simplified"),
            (CliLanguage::ChineseTraditional, "chinese-traditional"),
            (CliLanguage::French, "french"),
            (CliLanguage::Italian, "italian"),
            (CliLanguage::Czech, "czech"),
            (CliLanguage::Portuguese, "portuguese"),
        ];
        for (lang, expected_name) in all_variants {
            let code = lang.code();
            // MNEM_LANGUAGE_NAMES[code] must equal the canonical name.
            assert_eq!(
                MNEM_LANGUAGE_NAMES[code as usize], expected_name,
                "CliLanguage::{:?} has code {} but MNEM_LANGUAGE_NAMES[{}] = {:?}; expected {:?}",
                lang, code, code, MNEM_LANGUAGE_NAMES[code as usize], expected_name
            );
            // from_code must round-trip back to the same variant.
            assert_eq!(
                CliLanguage::from_code(code),
                Some(lang),
                "from_code({}) did not round-trip to {:?}",
                code, lang
            );
        }
        // Out-of-range codes must return None.
        assert_eq!(CliLanguage::from_code(10), None);
        assert_eq!(CliLanguage::from_code(255), None);
    }

    #[test]
    fn all_10_languages_have_kebab_case_str() {
        let cases = [
            (CliLanguage::English, "english"),
            (CliLanguage::Japanese, "japanese"),
            (CliLanguage::Korean, "korean"),
            (CliLanguage::Spanish, "spanish"),
            (CliLanguage::ChineseSimplified, "chinese-simplified"),
            (CliLanguage::ChineseTraditional, "chinese-traditional"),
            (CliLanguage::French, "french"),
            (CliLanguage::Italian, "italian"),
            (CliLanguage::Czech, "czech"),
            (CliLanguage::Portuguese, "portuguese"),
        ];
        for (lang, expected) in cases {
            assert_eq!(lang.as_str(), expected);
        }
    }

    #[test]
    fn json_round_trips_kebab_case() {
        let json = serde_json::to_string(&CliLanguage::ChineseSimplified).unwrap();
        assert_eq!(json, "\"chinese-simplified\"");
        let back: CliLanguage = serde_json::from_str(&json).unwrap();
        assert_eq!(back, CliLanguage::ChineseSimplified);
    }

    #[test]
    fn maps_to_bip39_language() {
        assert_eq!(
            bip39::Language::from(CliLanguage::English),
            bip39::Language::English
        );
        assert_eq!(
            bip39::Language::from(CliLanguage::ChineseSimplified),
            bip39::Language::SimplifiedChinese
        );
    }
}
