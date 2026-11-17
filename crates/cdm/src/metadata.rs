//! # Document Metadata, Provenance, and Exclusion Accounting
//!
//! Models capturing holistic repository/document metadata, source citations,
//! and auditable exclusion logs for skipped files or sections.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// High-level descriptive metadata for an entire converted document or repository.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DocumentMetadata {
    /// Document or repository title (e.g. webpage `<title>` or repo `owner/name`).
    pub title: Option<String>,
    /// Summary or description (from OpenGraph, README, or package manifest).
    pub description: Option<String>,
    /// List of author names or contributors.
    pub authors: Vec<String>,
    /// Declared software or content license (e.g. `MIT`, `Apache-2.0`).
    pub license: Option<String>,
    /// Primary programming or natural language (e.g. `Rust`, `en-US`).
    pub primary_language: Option<String>,
    /// Distribution of programming languages by byte volume or line count.
    pub language_breakdown: BTreeMap<String, u64>,
    /// Original publication timestamp.
    pub published_at: Option<DateTime<Utc>>,
    /// Last modification timestamp.
    pub modified_at: Option<DateTime<Utc>>,
    /// Categorical tags, repository topics, or keywords.
    pub tags: Vec<String>,
    /// Open-ended key-value metadata attributes.
    pub extra: BTreeMap<String, String>,
}

/// A citation anchoring content sections to their authoritative web or repository sources.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Citation {
    /// Unique citation identifier.
    pub citation_id: String,
    /// Identifier of the CDM section this citation applies to, if section-specific.
    pub section_id: Option<String>,
    /// Fully qualified authoritative source URL.
    pub url: String,
    /// Human-readable title or label for the citation.
    pub title: Option<String>,
    /// Compact formatted citation (e.g. `[Example Docs](https://example.com/docs)`).
    pub concise: String,
    /// Multi-line markdown blockquote callout.
    pub markdown_callout: String,
    /// Plain text citation string.
    pub plaintext: String,
}

impl Citation {
    /// Creates a citation formatted across concise, markdown, and plaintext styles.
    #[must_use]
    pub fn new(
        citation_id: impl Into<String>,
        url: impl Into<String>,
        title: Option<String>,
        section_id: Option<String>,
    ) -> Self {
        let citation_id = citation_id.into();
        let url = url.into();
        let display_title = title.as_deref().unwrap_or(&url);

        let concise = format!("[{display_title}]({url})");
        let markdown_callout = format!("> Source: [{display_title}]({url})");
        let plaintext = format!("Source: {display_title} ({url})");

        Self {
            citation_id,
            section_id,
            url,
            title,
            concise,
            markdown_callout,
            plaintext,
        }
    }
}

/// Rationale for intentionally excluding a discovered file or section from conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExclusionReason {
    /// Excluded by standard ignore patterns (e.g. `.gitignore`, `.aiignore`).
    RulePattern,
    /// Exceeded maximum file size or token threshold.
    SizeLimit,
    /// Detected as non-textual binary content (executable, image, archive).
    BinaryFile,
    /// Excluded based on unapproved file extension (e.g. `.min.js`, `.lock`).
    ExcludedExtension,
    /// Content was identified as an exact or near duplicate of an included section.
    Duplicate,
    /// Explicitly excluded by user request or custom job configuration.
    UserConfig,
    /// Excluded due to security policy (SSRF target, secret leak, sensitive file).
    SecurityRestricted,
    /// Discovered format is currently unsupported by extractors.
    UnsupportedType,
}

impl ExclusionReason {
    /// Returns a human-readable identifier for display and telemetry.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RulePattern => "rule_pattern",
            Self::SizeLimit => "size_limit",
            Self::BinaryFile => "binary_file",
            Self::ExcludedExtension => "excluded_extension",
            Self::Duplicate => "duplicate",
            Self::UserConfig => "user_config",
            Self::SecurityRestricted => "security_restricted",
            Self::UnsupportedType => "unsupported_type",
        }
    }
}

/// An auditable record explaining why an item was discovered but skipped during extraction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExclusionRecord {
    /// Relative or absolute path of the excluded item.
    pub original_path: String,
    /// The primary reason category for exclusion.
    pub reason: ExclusionReason,
    /// Glob or regex pattern that matched, if applicable (e.g. `*.lock`, `target/**`).
    pub pattern: Option<String>,
    /// Human-readable explanation of why this item was excluded.
    pub details: Option<String>,
    /// Byte size of the excluded resource, if known.
    pub byte_size: Option<u64>,
}

impl ExclusionRecord {
    /// Constructs a new exclusion record.
    #[must_use]
    pub fn new(
        original_path: impl Into<String>,
        reason: ExclusionReason,
        pattern: Option<String>,
        details: Option<String>,
        byte_size: Option<u64>,
    ) -> Self {
        Self {
            original_path: original_path.into(),
            reason,
            pattern,
            details,
            byte_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_citation_formatting() {
        let cit = Citation::new(
            "cit-1",
            "https://example.com/docs",
            Some("Example Docs".to_string()),
            Some("sec-1".to_string()),
        );
        assert_eq!(cit.concise, "[Example Docs](https://example.com/docs)");
        assert_eq!(
            cit.markdown_callout,
            "> Source: [Example Docs](https://example.com/docs)"
        );
        assert_eq!(
            cit.plaintext,
            "Source: Example Docs (https://example.com/docs)"
        );
    }

    #[test]
    fn test_exclusion_record_creation() {
        let rec = ExclusionRecord::new(
            "target/debug/app.exe",
            ExclusionReason::BinaryFile,
            Some("target/**".to_string()),
            Some("Compiled binary executable".to_string()),
            Some(1024 * 1024),
        );
        assert_eq!(rec.reason, ExclusionReason::BinaryFile);
        assert_eq!(rec.reason.as_str(), "binary_file");
        assert_eq!(rec.byte_size, Some(1048576));
    }
}
