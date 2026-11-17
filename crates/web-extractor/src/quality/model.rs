//! # Content Completeness & Extraction Quality Models
//!
//! Strongly-typed representations for extraction quality assessment, multidimensional
//! completeness scoring, content metrics, and structured extraction warnings.

use serde::{Deserialize, Serialize};

use crate::provenance::WarningSeverity;

/// Overall completeness rating category for an extracted webpage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletenessLevel {
    /// Full-fidelity extraction (overall score >= 0.85 and no critical warnings).
    /// All essential metadata, structure, and content sections are intact.
    Complete,
    /// High-fidelity extraction (overall score 0.70..0.85).
    /// Substantial content preserved with minor missing metadata or minor structural gaps.
    High,
    /// Moderate-fidelity extraction (overall score 0.50..0.70).
    /// Usable content with some gaps in metadata, formatting, or document structure.
    Moderate,
    /// Partial extraction (overall score 0.25..0.50).
    /// Sparse text, missing headings, or noticeable extraction warnings.
    Partial,
    /// Degraded extraction (overall score 0.05..0.25).
    /// Heavily truncated or degraded document; dynamic fallback degraded to minimal static shell.
    Degraded,
    /// Empty extraction (overall score < 0.05).
    /// No usable or meaningful textual content recovered from the document.
    Empty,
}

impl CompletenessLevel {
    /// Returns the canonical display string token.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::High => "high",
            Self::Moderate => "moderate",
            Self::Partial => "partial",
            Self::Degraded => "degraded",
            Self::Empty => "empty",
        }
    }
}

/// Category classification for diagnostic extraction warnings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtractionWarningCategory {
    /// Document metadata gaps (missing title, description, author, dates).
    Metadata,
    /// Content volume, density, or readability concerns.
    Content,
    /// Heading hierarchy gaps or malformed markup structures.
    Structure,
    /// Unidentified programming language syntax in code blocks.
    Code,
    /// Table structures lacking header rows or rectangular normalization issues.
    Table,
    /// HTTP transport, caching, or provenance warnings.
    Transport,
    /// Browser worker rendering errors, console logs, or degradation.
    Browser,
    /// Subresources blocked by SSRF or security policies.
    Security,
}

impl ExtractionWarningCategory {
    /// Returns the string representation of the warning category.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Metadata => "metadata",
            Self::Content => "content",
            Self::Structure => "structure",
            Self::Code => "code",
            Self::Table => "table",
            Self::Transport => "transport",
            Self::Browser => "browser",
            Self::Security => "security",
        }
    }
}

/// A structured diagnostic warning detected during extraction quality scoring.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractionWarning {
    /// Machine-readable warning identifier (e.g., `"METADATA_MISSING_TITLE"`).
    pub code: String,
    /// Broad category classification.
    pub category: ExtractionWarningCategory,
    /// Severity rating (`Info`, `Warning`, `Critical`).
    pub severity: WarningSeverity,
    /// Human-readable explanation.
    pub message: String,
    /// Contextual detail (e.g. element name, heading transition, or URL).
    pub context: Option<String>,
    /// Quality score penalty applied by this warning (subtracted from raw score).
    pub score_penalty: f64,
}

impl ExtractionWarning {
    /// Constructs a new extraction warning with specified penalty.
    #[must_use]
    pub fn new(
        code: impl Into<String>,
        category: ExtractionWarningCategory,
        severity: WarningSeverity,
        message: impl Into<String>,
        score_penalty: f64,
    ) -> Self {
        Self {
            code: code.into(),
            category,
            severity,
            message: message.into(),
            context: None,
            score_penalty: score_penalty.max(0.0),
        }
    }

    /// Attaches contextual detail to the warning.
    #[must_use]
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

/// Normalized scores across the 4 key extraction quality dimensions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimensionScores {
    /// Metadata completeness score (0.0..=1.0).
    pub metadata: f64,
    /// Content substance and density score (0.0..=1.0).
    pub content: f64,
    /// Structural fidelity score (headings, code, tables, links) (0.0..=1.0).
    pub structural: f64,
    /// Provenance and acquisition integrity score (0.0..=1.0).
    pub provenance: f64,
}

/// Quantified empirical metrics extracted from the page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentMetrics {
    /// Raw HTML markup size in bytes.
    pub raw_html_bytes: usize,
    /// Extracted clean plain text size in bytes.
    pub cleaned_text_bytes: usize,
    /// Total word count in extracted text.
    pub word_count: usize,
    /// Total character count in extracted text.
    pub character_count: usize,
    /// Estimated reading time in minutes (`ceil(word_count / 200)`).
    pub reading_time_minutes: usize,
    /// Total non-empty paragraphs or blocks.
    pub paragraph_count: usize,
    /// Total headings (h1..h6) extracted.
    pub heading_count: usize,
    /// Headings count breakdown by level: index 0 = h1, index 5 = h6.
    pub headings_by_level: [usize; 6],
    /// Whether heading levels follow a valid monotonic sequence without gaps.
    pub heading_hierarchy_valid: bool,
    /// Number of extracted code blocks.
    pub code_block_count: usize,
    /// Number of code blocks with explicitly identified languages.
    pub code_blocks_with_syntax: usize,
    /// Number of extracted tables.
    pub table_count: usize,
    /// Number of extracted tables with header rows.
    pub tables_with_headers: usize,
    /// Total extracted hyperlinks.
    pub link_count: usize,
    /// Number of internal / same-domain links.
    pub internal_link_count: usize,
    /// Number of external links.
    pub external_link_count: usize,
    /// Content density ratio (`cleaned_text_bytes / raw_html_bytes`).
    pub content_density: f64,
    /// Ratio of stripped markup/boilerplate to raw HTML size.
    pub boilerplate_ratio: f64,
}

/// Complete extraction quality report for an extracted webpage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractionQualityReport {
    /// Overall normalized completeness score in `[0.0, 1.0]`, rounded to 3 decimal places.
    pub overall_score: f64,
    /// Overall qualitative completeness grade.
    pub completeness_level: CompletenessLevel,
    /// Individual dimensional component scores.
    pub dimension_scores: DimensionScores,
    /// Quantified extraction metrics.
    pub metrics: ContentMetrics,
    /// Diagnostic warnings detected during quality assessment.
    pub warnings: Vec<ExtractionWarning>,
    /// Whether the extraction quality meets the minimum threshold for CDM document conversion.
    pub is_acceptable_for_cdm: bool,
}

impl ExtractionQualityReport {
    /// Returns the list of warning code tokens.
    #[must_use]
    pub fn warning_codes(&self) -> Vec<&str> {
        self.warnings.iter().map(|w| w.code.as_str()).collect()
    }

    /// Checks whether any warning with `Critical` severity is present.
    #[must_use]
    pub fn has_critical_warnings(&self) -> bool {
        self.warnings
            .iter()
            .any(|w| w.severity == WarningSeverity::Critical)
    }

    /// Returns warnings filtered by category.
    #[must_use]
    pub fn warnings_by_category(
        &self,
        category: ExtractionWarningCategory,
    ) -> Vec<&ExtractionWarning> {
        self.warnings
            .iter()
            .filter(|w| w.category == category)
            .collect()
    }

    /// Formats a human-readable one-line quality summary string.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "Completeness: {:.1}% ({:?}) | Words: {} | Density: {:.1}% | Warnings: {}",
            self.overall_score * 100.0,
            self.completeness_level,
            self.metrics.word_count,
            self.metrics.content_density * 100.0,
            self.warnings.len(),
        )
    }
}
