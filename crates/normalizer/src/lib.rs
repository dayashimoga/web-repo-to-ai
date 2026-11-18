//! # Content Normalizer (`web-repo-to-ai-normalizer`)
//!
//! Provides deterministic document and text normalization:
//! - UTF-8 character encoding sanitization (BOM stripping, null bytes, zero-width chars, smart quotes)
//! - Whitespace and line-ending canonicalization without semantic loss (CRLF/CR -> LF, blank line limits)
//! - Markdown code fence, heading level, list marker, and table rectangularization
//! - Section-aware normalization preserving 100% code syntax and indentation fidelity
//! - Comprehensive telemetry, reduction metrics, and cryptographic hash verification

#![warn(missing_docs)]

pub mod classification_report;
pub mod classifier;
pub mod config;
pub mod document;
pub mod encoding;
pub mod error;
pub mod markdown;
pub mod report;
pub mod section;
pub mod whitespace;

pub use classification_report::ContentClassificationReport;
pub use classifier::{
    classify_document, ClassificationConfig, ContentCategory, DocumentContentClassifier,
    InformationDensity, NoiseIndicator, SectionClassification, SectionClassifier,
};
pub use config::{EncodingConfig, MarkdownConfig, NormalizationConfig, WhitespaceConfig};
pub use document::DocumentNormalizer;
pub use encoding::EncodingSanitizer;
pub use error::{NormalizerError, Result};
pub use markdown::MarkdownNormalizer;
pub use report::NormalizationReport;
pub use section::{SectionNormalizationAudit, SectionNormalizer};
pub use whitespace::WhitespaceNormalizer;

use web_repo_to_ai_cdm::CdmDocument;

/// Returns the unique crate identifier name.
#[must_use]
pub fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

/// Convenience function to normalize a [`CdmDocument`] with the specified configuration.
#[must_use]
pub fn normalize_document(
    doc: &CdmDocument,
    config: &NormalizationConfig,
) -> (CdmDocument, NormalizationReport) {
    DocumentNormalizer::new(config.clone()).normalize_document(doc)
}

/// Convenience function to normalize raw text according to a [`NormalizationConfig`].
///
/// When `is_code` is true, typographic punctuation replacement is strictly bypassed
/// and code-specific blank line limits are applied.
#[must_use]
pub fn normalize_text(text: &str, config: &NormalizationConfig, is_code: bool) -> String {
    if text.is_empty() {
        return String::new();
    }

    if is_code {
        let mut code_encoding = config.encoding.clone();
        code_encoding.normalize_typographic_punctuation = false;
        let sanitized = EncodingSanitizer::sanitize_text(text, &code_encoding);
        WhitespaceNormalizer::normalize_whitespace(&sanitized, &config.whitespace, true)
    } else {
        let sanitized = EncodingSanitizer::sanitize_text(text, &config.encoding);
        let ws_normalized =
            WhitespaceNormalizer::normalize_whitespace(&sanitized, &config.whitespace, false);
        MarkdownNormalizer::normalize_markdown(&ws_normalized, &config.markdown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalizer_metadata() {
        assert_eq!(crate_name(), "web-repo-to-ai-normalizer");
    }

    #[test]
    fn test_normalize_text_convenience() {
        let config = NormalizationConfig::compact();
        let prose = "\u{FEFF}#  Heading 1  \r\n\r\n“Smart quotes” and tabs.\t  \r\n";
        let normalized = normalize_text(prose, &config, false);
        assert_eq!(normalized, "# Heading 1\n\n\"Smart quotes\" and tabs.");
    }
}
