//! # Section-Aware Content Normalizer
//!
//! Applies kind-specific normalization strategies across [`CdmSection`] instances,
//! preserving 100% semantic code fidelity while standardizing whitespace, line breaks,
//! encoding, and Markdown tables.

use serde::{Deserialize, Serialize};
use web_repo_to_ai_cdm::{CdmSection, SectionKind, SectionTokenMetrics};

use crate::config::NormalizationConfig;
use crate::encoding::EncodingSanitizer;
use crate::markdown::MarkdownNormalizer;
use crate::whitespace::WhitespaceNormalizer;

/// Audit record of transformations applied to an individual section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionNormalizationAudit {
    /// Identifier of the normalized section.
    pub section_id: String,
    /// Section title or path if available.
    pub title: Option<String>,
    /// Structural kind of the section.
    pub kind: SectionKind,
    /// Cryptographic BLAKE3 hash before normalization.
    pub hash_before: String,
    /// Cryptographic BLAKE3 hash after normalization.
    pub hash_after: String,
    /// Byte length before normalization.
    pub bytes_before: usize,
    /// Byte length after normalization.
    pub bytes_after: usize,
    /// Line count before normalization.
    pub lines_before: usize,
    /// Line count after normalization.
    pub lines_after: usize,
    /// Estimated token count before normalization.
    pub tokens_before: usize,
    /// Estimated token count after normalization.
    pub tokens_after: usize,
    /// True if the section content was modified by normalization.
    pub modified: bool,
}

impl SectionNormalizationAudit {
    /// Byte reduction achieved for this section (saturating subtraction).
    #[must_use]
    pub fn byte_reduction(&self) -> usize {
        self.bytes_before.saturating_sub(self.bytes_after)
    }

    /// Estimated token reduction achieved for this section.
    #[must_use]
    pub fn token_reduction(&self) -> usize {
        self.tokens_before.saturating_sub(self.tokens_after)
    }
}

/// Normalizer engine operating on individual [`CdmSection`] content units.
pub struct SectionNormalizer;

impl SectionNormalizer {
    /// Normalizes a [`CdmSection`], returning the transformed section and its audit record.
    #[must_use]
    pub fn normalize_section(
        section: &CdmSection,
        config: &NormalizationConfig,
    ) -> (CdmSection, SectionNormalizationAudit) {
        let initial_body = &section.body;
        let initial_bytes = initial_body.len();
        let initial_lines = if initial_body.is_empty() {
            0
        } else {
            initial_body.lines().count().max(1)
        };
        let initial_tokens = section
            .token_metrics
            .as_ref()
            .map(|m| m.estimated_tokens)
            .unwrap_or_else(|| initial_body.chars().count().div_ceil(4));
        let hash_before = section.content_hash.clone();

        // 1. Kind-specific normalization strategy
        let normalized_body = match section.kind {
            SectionKind::CodeFile | SectionKind::CodeSnippet => {
                // Code normalization: strictly preserve quotes, symbols, and indentation
                let mut code_encoding = config.encoding.clone();
                // Never normalize typographic punctuation in source code
                code_encoding.normalize_typographic_punctuation = false;

                let sanitized = EncodingSanitizer::sanitize_text(initial_body, &code_encoding);
                WhitespaceNormalizer::normalize_whitespace(&sanitized, &config.whitespace, true)
            }
            SectionKind::Prose | SectionKind::Admonition | SectionKind::Navigation => {
                // Prose normalization: encoding sanitization + whitespace + markdown syntax
                let sanitized = EncodingSanitizer::sanitize_text(initial_body, &config.encoding);
                let ws_normalized = WhitespaceNormalizer::normalize_whitespace(
                    &sanitized,
                    &config.whitespace,
                    false,
                );
                MarkdownNormalizer::normalize_markdown(&ws_normalized, &config.markdown)
            }
            SectionKind::Table => {
                // Table normalization: encoding + whitespace + table rectangularization
                let sanitized = EncodingSanitizer::sanitize_text(initial_body, &config.encoding);
                let ws_normalized = WhitespaceNormalizer::normalize_whitespace(
                    &sanitized,
                    &config.whitespace,
                    false,
                );
                MarkdownNormalizer::normalize_markdown(&ws_normalized, &config.markdown)
            }
            SectionKind::Manifest | SectionKind::Frontmatter => {
                // Structured text: encoding + line endings + trailing whitespace
                let sanitized = EncodingSanitizer::sanitize_text(initial_body, &config.encoding);
                WhitespaceNormalizer::normalize_whitespace(&sanitized, &config.whitespace, true)
            }
        };

        // 2. Metrics & Hash re-computation
        let hash_after = CdmSection::compute_hash(&normalized_body);
        let modified = hash_before != hash_after;

        let final_bytes = normalized_body.len();
        let final_lines = if normalized_body.is_empty() {
            0
        } else {
            normalized_body.lines().count().max(1)
        };
        let final_metrics = SectionTokenMetrics::compute_baseline(&normalized_body);
        let final_tokens = final_metrics.estimated_tokens;

        // 3. Assemble normalized section
        let mut normalized_section = section.clone();
        normalized_section.body = normalized_body;
        normalized_section.content_hash = hash_after.clone();
        normalized_section.token_metrics = Some(final_metrics);

        let audit = SectionNormalizationAudit {
            section_id: section.section_id.clone(),
            title: section.title.clone(),
            kind: section.kind,
            hash_before,
            hash_after,
            bytes_before: initial_bytes,
            bytes_after: final_bytes,
            lines_before: initial_lines,
            lines_after: final_lines,
            tokens_before: initial_tokens,
            tokens_after: final_tokens,
            modified,
        };

        (normalized_section, audit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn sample_code_section() -> CdmSection {
        let body = "\r\n\r\n    fn compute() {\r\n        let val = 42;   \r\n\r\n\r\n\r\n        val\r\n    }\r\n\r\n";
        CdmSection {
            section_id: "sec-code-1".to_string(),
            parent_id: None,
            title: Some("compute.rs".to_string()),
            kind: SectionKind::CodeFile,
            original_path: Some("src/compute.rs".to_string()),
            source_url: "https://example.com/src/compute.rs".to_string(),
            line_range: None,
            byte_range: None,
            language: Some("rust".to_string()),
            content_hash: CdmSection::compute_hash(body),
            body: body.to_string(),
            token_metrics: Some(SectionTokenMetrics::compute_baseline(body)),
            attributes: BTreeMap::new(),
        }
    }

    #[test]
    fn test_code_section_normalization_preserves_indentation_and_hash() {
        let section = sample_code_section();
        let config = NormalizationConfig::lossless();
        let (normalized, audit) = SectionNormalizer::normalize_section(&section, &config);

        assert!(audit.modified);
        assert_eq!(
            normalized.body,
            "    fn compute() {\n        let val = 42;\n\n\n        val\n    }"
        );
        assert!(normalized.verify_hash());
        assert_eq!(normalized.content_hash, audit.hash_after);
        assert!(audit.bytes_after < audit.bytes_before);
    }
}
