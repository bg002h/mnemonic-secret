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
