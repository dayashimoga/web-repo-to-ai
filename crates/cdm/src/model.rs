//! # Canonical Context Document Model (CDM) Core Schema
//!
//! Defines the foundational intermediate document representation (`CdmDocument`)
//! bridging all upstream extractors and downstream AI optimizers, tokenizers,
//! chunkers, and multi-format exporters.

use crate::completeness::CompletenessReport;
use crate::hierarchy::HierarchyNode;
use crate::metadata::{Citation, DocumentMetadata, ExclusionRecord};
use crate::relationships::Relationship;
use crate::section::CdmSection;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use web_repo_to_ai_core::DocumentId;

/// Classification of the acquisition origin for a converted resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    /// Static HTTP acquisition of HTML content.
    WebpageStatic,
    /// Dynamic Playwright/Chromium browser-rendered single-page application.
    WebpageDynamic,
    /// Public GitHub repository accessed via Git Trees or REST API.
    GitHubPublic,
    /// Private GitHub repository accessed via authorized GitHub App token.
    GitHubPrivate,
    /// Portable Document Format (PDF) file extraction.
    DocumentPdf,
    /// Optical Character Recognition (OCR) extracted image/document.
    DocumentOcr,
}

impl SourceType {
    /// Returns a human-readable identifier for display and telemetry.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WebpageStatic => "webpage_static",
            Self::WebpageDynamic => "webpage_dynamic",
            Self::GitHubPublic => "github_public",
            Self::GitHubPrivate => "github_private",
            Self::DocumentPdf => "document_pdf",
            Self::DocumentOcr => "document_ocr",
        }
    }
}

/// Identifiers establishing the origin and address of the acquired content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceIdentity {
    /// The original raw input string provided by the user.
    pub raw_input: String,
    /// Canonicalized, SSRF-validated target URL.
    pub canonical_url: String,
    /// Host domain name, if applicable (e.g. `docs.rs` or `github.com`).
    pub domain: Option<String>,
    /// Repository owner username or organization (for Git sources).
    pub repo_owner: Option<String>,
    /// Repository project name (for Git sources).
    pub repo_name: Option<String>,
    /// Git commit, branch, or tag reference name.
    pub git_ref: Option<String>,
    /// Relative subpath filter within the source repository or website.
    pub subpath: Option<String>,
}

impl SourceIdentity {
    /// Constructs a basic source identity from raw input and canonical URL.
    #[must_use]
    pub fn new(raw_input: impl Into<String>, canonical_url: impl Into<String>) -> Self {
        Self {
            raw_input: raw_input.into(),
            canonical_url: canonical_url.into(),
            domain: None,
            repo_owner: None,
            repo_name: None,
            git_ref: None,
            subpath: None,
        }
    }
}

/// Versioning and revision identifiers for cache validation and provenance.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RevisionInfo {
    /// Full 40-character Git commit SHA, if extracted from a Git repository.
    pub git_commit: Option<String>,
    /// Git branch name (e.g. `main`).
    pub git_branch: Option<String>,
    /// Git release or tag name (e.g. `v1.2.0`).
    pub git_tag: Option<String>,
    /// HTTP ETag response header value.
    pub http_etag: Option<String>,
    /// HTTP Last-Modified response header string.
    pub http_last_modified: Option<String>,
    /// Cryptographic BLAKE3 hex digest of the raw source payload.
    pub content_hash_blake3: Option<String>,
    /// Cryptographic SHA-256 hex digest of the raw source payload.
    pub content_hash_sha256: Option<String>,
}

/// Execution status of the extraction pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtractionStatus {
    /// All discovered and eligible content was successfully extracted.
    Completed,
    /// Content was partially extracted with non-fatal warnings or degraded fallback.
    Partial,
    /// Extraction failed fatally with zero usable content.
    Failed,
    /// Source media or architecture is unsupported by available extractors.
    Unsupported,
}

impl ExtractionStatus {
    /// Returns a human-readable identifier for display and telemetry.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Partial => "partial",
            Self::Failed => "failed",
            Self::Unsupported => "unsupported",
        }
    }
}

/// Severity classification for extraction warnings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarningSeverity {
    /// Critical diagnostic indicating major content loss or structural defect.
    Critical,
    /// Standard warning indicating degraded fidelity or missing metadata.
    Warning,
    /// Informational note regarding formatting decisions or minor normalizations.
    Info,
}

impl WarningSeverity {
    /// Returns a human-readable identifier for display and telemetry.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::Warning => "warning",
            Self::Info => "info",
        }
    }
}

/// A diagnostic finding or degradation warning emitted during extraction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractionWarning {
    /// Stable machine-readable error or warning code.
    pub code: String,
    /// Functional category (e.g. `provenance`, `structure`, `boilerplate`, `security`).
    pub category: String,
    /// Human-readable explanation of the issue.
    pub message: String,
    /// Qualitative severity level.
    pub severity: WarningSeverity,
    /// Section ID affected by this warning, if localized to a specific section.
    pub section_id: Option<String>,
    /// Arbitrary structured context attributes.
    pub context: BTreeMap<String, String>,
}

impl ExtractionWarning {
    /// Creates a new extraction warning.
    #[must_use]
    pub fn new(
        code: impl Into<String>,
        category: impl Into<String>,
        message: impl Into<String>,
        severity: WarningSeverity,
    ) -> Self {
        Self {
            code: code.into(),
            category: category.into(),
            message: message.into(),
            severity,
            section_id: None,
            context: BTreeMap::new(),
        }
    }
}

/// The Canonical Context Document Model (CDM v1.0).
///
/// Single source of truth across all extractors and exporters in `web-repo-to-ai`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CdmDocument {
    /// Schema specification version (e.g. `"1.0.0"`).
    pub schema_version: String,
    /// Strongly-typed UUID v7 identifier for this document.
    pub document_id: DocumentId,
    /// Acquisition origin and target coordinates.
    pub source_identity: SourceIdentity,
    /// The type of source extracted.
    pub source_type: SourceType,
    /// Authoritative canonical URL.
    pub canonical_url: String,
    /// Timestamp when content was acquired.
    pub retrieved_at: DateTime<Utc>,
    /// Revision and cache validation hashes.
    pub revision: RevisionInfo,
    /// MIME media type of the source resource (e.g. `text/html`, `application/vnd.github.v3+json`).
    pub content_type: String,
    /// Holistic descriptive metadata.
    pub metadata: DocumentMetadata,
    /// Root of the document outline or repository directory tree.
    pub hierarchy: HierarchyNode,
    /// Ordered collection of content sections.
    pub sections: Vec<CdmSection>,
    /// Cross-section relationships, call graphs, and hyperlinks.
    pub relationships: Vec<Relationship>,
    /// Authoritative provenance citations.
    pub citations: Vec<Citation>,
    /// Overall job extraction status.
    pub extraction_status: ExtractionStatus,
    /// Diagnostic warnings and notices emitted during processing.
    pub warnings: Vec<ExtractionWarning>,
    /// Auditable log of intentionally skipped files or blocks.
    pub exclusions: Vec<ExclusionRecord>,
    /// Completeness accounting report.
    pub completeness: CompletenessReport,
}

impl CdmDocument {
    /// Computes the total aggregate character count across all sections.
    #[must_use]
    pub fn total_char_count(&self) -> usize {
        self.sections.iter().map(|s| s.body.chars().count()).sum()
    }

    /// Computes the total aggregate word count across all sections.
    #[must_use]
    pub fn total_word_count(&self) -> usize {
        self.sections
            .iter()
            .map(|s| {
                s.token_metrics
                    .as_ref()
                    .map_or_else(|| s.body.split_whitespace().count(), |m| m.word_count)
            })
            .sum()
    }

    /// Computes the total estimated token count across all sections.
    #[must_use]
    pub fn total_estimated_tokens(&self) -> usize {
        self.sections
            .iter()
            .map(|s| {
                s.token_metrics.as_ref().map_or_else(
                    || s.body.chars().count().div_ceil(4),
                    |m| m.estimated_tokens,
                )
            })
            .sum()
    }

    /// Finds a section by its unique identifier.
    #[must_use]
    pub fn find_section(&self, section_id: &str) -> Option<&CdmSection> {
        self.sections.iter().find(|s| s.section_id == section_id)
    }

    /// Validates all internal section hashes against their recorded bodies.
    #[must_use]
    pub fn verify_section_hashes(&self) -> bool {
        self.sections.iter().all(CdmSection::verify_hash)
    }
}
