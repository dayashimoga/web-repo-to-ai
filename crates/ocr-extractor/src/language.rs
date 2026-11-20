//! # OCR Language Selection and Validation
//!
//! Provides ISO 639-2/639-3 language mapping, multi-language combination parsing,
//! Tesseract model validation, and configurable fallback policies.

use crate::error::{OcrError, Result};
use serde::{Deserialize, Serialize};

/// Fallback policy applied when a requested language model is missing or unsupported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LanguageFallbackPolicy {
    /// Return an explicit `OcrError::UnsupportedLanguage` error.
    StrictError,
    /// Fall back to the default language model (typically "eng") if any requested language is missing.
    #[default]
    FallbackToDefault,
    /// Filter out missing languages, keeping all available ones (falling back to default if none available).
    FilterUnavailable,
}

/// Normalizes language names or short codes into canonical Tesseract 3-letter codes.
#[must_use]
pub fn normalize_language_code(input: &str) -> String {
    let trimmed = input.trim().to_ascii_lowercase();
    match trimmed.as_str() {
        "en" | "eng" | "english" => "eng".to_string(),
        "de" | "deu" | "ger" | "german" => "deu".to_string(),
        "fr" | "fra" | "fre" | "french" => "fra".to_string(),
        "es" | "spa" | "spanish" => "spa".to_string(),
        "it" | "ita" | "italian" => "ita".to_string(),
        "pt" | "por" | "portuguese" => "por".to_string(),
        "nl" | "nld" | "dut" | "dutch" => "nld".to_string(),
        "ru" | "rus" | "russian" => "rus".to_string(),
        "zh" | "zh-cn" | "chi_sim" | "chinese" | "chinese-simplified" => "chi_sim".to_string(),
        "zh-tw" | "chi_tra" | "chinese-traditional" => "chi_tra".to_string(),
        "ja" | "jpn" | "japanese" => "jpn".to_string(),
        "ko" | "kor" | "korean" => "kor".to_string(),
        "ar" | "ara" | "arabic" => "ara".to_string(),
        "hi" | "hin" | "hindi" => "hin".to_string(),
        "osd" => "osd".to_string(),
        other => other.to_string(),
    }
}

/// Parses multi-language specifications delimited by '+', ',', ';', or whitespace.
#[must_use]
pub fn parse_language_string(languages: &str) -> Vec<String> {
    let mut result = Vec::new();

    for token in languages.split(|c: char| c == '+' || c == ',' || c == ';' || c.is_whitespace()) {
        let trimmed = token.trim();
        if !trimmed.is_empty() {
            let normalized = normalize_language_code(trimmed);
            if !result.contains(&normalized) {
                result.push(normalized);
            }
        }
    }

    if result.is_empty() {
        vec!["eng".to_string()]
    } else {
        result
    }
}

/// Formats a list of languages into a canonical Tesseract multi-language CLI string (e.g. "eng+fra").
#[must_use]
pub fn canonicalize_languages(languages: &str) -> String {
    parse_language_string(languages).join("+")
}

/// Resolves and validates requested languages against engine-supported languages.
pub fn resolve_languages(
    requested: &str,
    supported: &[String],
    policy: LanguageFallbackPolicy,
    default_lang: &str,
) -> Result<String> {
    let parsed = parse_language_string(requested);

    // If engine provided no supported list, pass canonicalized string as-is
    if supported.is_empty() {
        return Ok(parsed.join("+"));
    }

    let is_supported =
        |lang: &str| -> bool { supported.iter().any(|s| s.eq_ignore_ascii_case(lang)) };

    let all_supported = parsed.iter().all(|l| is_supported(l));

    if all_supported {
        return Ok(parsed.join("+"));
    }

    match policy {
        LanguageFallbackPolicy::StrictError => {
            let missing: Vec<String> = parsed.into_iter().filter(|l| !is_supported(l)).collect();
            Err(OcrError::UnsupportedLanguage {
                requested: missing.join("+"),
                available: supported.to_vec(),
            })
        }
        LanguageFallbackPolicy::FallbackToDefault => {
            let default_norm = normalize_language_code(default_lang);
            if is_supported(&default_norm) {
                Ok(default_norm)
            } else if !supported.is_empty() {
                Ok(supported[0].clone())
            } else {
                Ok("eng".to_string())
            }
        }
        LanguageFallbackPolicy::FilterUnavailable => {
            let available_ones: Vec<String> =
                parsed.into_iter().filter(|l| is_supported(l)).collect();

            if !available_ones.is_empty() {
                Ok(available_ones.join("+"))
            } else {
                let default_norm = normalize_language_code(default_lang);
                if is_supported(&default_norm) {
                    Ok(default_norm)
                } else if !supported.is_empty() {
                    Ok(supported[0].clone())
                } else {
                    Ok("eng".to_string())
                }
            }
        }
    }
}

/// Checks whether a language uses CJK (Chinese, Japanese, Korean) ideographic scripts.
#[must_use]
pub fn is_cjk(lang: &str) -> bool {
    matches!(
        lang.to_ascii_lowercase().as_str(),
        "chi_sim" | "chi_tra" | "jpn" | "kor"
    )
}

/// Checks whether a language uses Right-to-Left (RTL) scripts (e.g. Arabic, Hebrew).
#[must_use]
pub fn is_rtl(lang: &str) -> bool {
    matches!(
        lang.to_ascii_lowercase().as_str(),
        "ara" | "heb" | "fas" | "urd"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_language_code() {
        assert_eq!(normalize_language_code("en"), "eng");
        assert_eq!(normalize_language_code("English"), "eng");
        assert_eq!(normalize_language_code("de"), "deu");
        assert_eq!(normalize_language_code("German"), "deu");
        assert_eq!(normalize_language_code("fr"), "fra");
        assert_eq!(normalize_language_code("Spanish"), "spa");
        assert_eq!(normalize_language_code("Chinese"), "chi_sim");
        assert_eq!(normalize_language_code("Japanese"), "jpn");
        assert_eq!(normalize_language_code("custom_model"), "custom_model");
    }

    #[test]
    fn test_parse_multi_language_string() {
        assert_eq!(
            parse_language_string("eng, fra"),
            vec!["eng".to_string(), "fra".to_string()]
        );
        assert_eq!(
            parse_language_string("english+german"),
            vec!["eng".to_string(), "deu".to_string()]
        );
        assert_eq!(
            parse_language_string("eng; spa   fra"),
            vec!["eng".to_string(), "spa".to_string(), "fra".to_string()]
        );
        assert_eq!(
            parse_language_string("eng, eng, fra"),
            vec!["eng".to_string(), "fra".to_string()]
        );
    }

    #[test]
    fn test_canonicalize_languages() {
        assert_eq!(canonicalize_languages("english, french"), "eng+fra");
        assert_eq!(canonicalize_languages(""), "eng");
    }

    #[test]
    fn test_resolve_languages_strict_and_fallback() {
        let supported = vec!["eng".to_string(), "fra".to_string(), "osd".to_string()];

        // All supported
        let res = resolve_languages(
            "eng+fra",
            &supported,
            LanguageFallbackPolicy::StrictError,
            "eng",
        )
        .expect("All supported succeeds");
        assert_eq!(res, "eng+fra");

        // Strict missing
        let err = resolve_languages(
            "eng+deu",
            &supported,
            LanguageFallbackPolicy::StrictError,
            "eng",
        )
        .expect_err("Missing deu must fail under StrictError");
        assert!(matches!(err, OcrError::UnsupportedLanguage { .. }));

        // Fallback to default
        let res_fb = resolve_languages(
            "eng+deu",
            &supported,
            LanguageFallbackPolicy::FallbackToDefault,
            "eng",
        )
        .expect("Fallback succeeds");
        assert_eq!(res_fb, "eng");

        // Filter unavailable
        let res_filter = resolve_languages(
            "eng+deu",
            &supported,
            LanguageFallbackPolicy::FilterUnavailable,
            "eng",
        )
        .expect("Filter succeeds");
        assert_eq!(res_filter, "eng");
    }
}
