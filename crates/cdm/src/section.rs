//! # CDM Section Models
//!
//! Ordered content units (source code files, documentation sections, tables, snippets)
//! that compose a Canonical Context Document Model (`CdmDocument`).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Categorization of a content unit within a CDM document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SectionKind {
    /// Formatted prose text (paragraphs, articles, documentation sections).
    Prose,
    /// Complete standalone source code file from a repository.
    CodeFile,
    /// Embedded code fragment or snippet within documentation or an article.
    CodeSnippet,
    /// Structured data table (GFM rectangular table or serialized tabular data).
    Table,
    /// Metadata header block (YAML, TOML, JSON frontmatter).
    Frontmatter,
    /// Directory manifest or repository tree index.
    Manifest,
    /// Navigation structure (breadcrumbs, table of contents, pagination).
    Navigation,
    /// Callout or highlighted advisory block (Note, Tip, Warning, Danger).
    Admonition,
}

impl SectionKind {
    /// Returns a human-readable identifier for display and telemetry.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Prose => "prose",
            Self::CodeFile => "code_file",
            Self::CodeSnippet => "code_snippet",
            Self::Table => "table",
            Self::Frontmatter => "frontmatter",
            Self::Manifest => "manifest",
            Self::Navigation => "navigation",
            Self::Admonition => "admonition",
        }
    }
}

/// 1-indexed inclusive line span indicating location within an original file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LineRange {
    /// 1-indexed start line (inclusive).
    pub start_line: u32,
    /// 1-indexed end line (inclusive).
    pub end_line: u32,
}

impl LineRange {
    /// Creates a new line range.
    #[must_use]
    pub const fn new(start_line: u32, end_line: u32) -> Self {
        Self {
            start_line,
            end_line,
        }
    }

    /// Computes the total number of lines spanned by this range.
    #[must_use]
    pub const fn line_count(&self) -> u32 {
        if self.end_line >= self.start_line {
            (self.end_line - self.start_line) + 1
        } else {
            0
        }
    }
}

/// Byte span indicating byte offset boundaries within raw source data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ByteRange {
    /// Start byte offset (inclusive).
    pub start_byte: u64,
    /// End byte offset (exclusive).
    pub end_byte: u64,
}

impl ByteRange {
    /// Creates a new byte range.
    #[must_use]
    pub const fn new(start_byte: u64, end_byte: u64) -> Self {
        Self {
            start_byte,
            end_byte,
        }
    }

    /// Computes the total byte length spanned.
    #[must_use]
    pub const fn byte_length(&self) -> u64 {
        self.end_byte.saturating_sub(self.start_byte)
    }
}

/// Token and volume metrics associated with a section body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionTokenMetrics {
    /// Word count computed via whitespace tokenization.
    pub word_count: usize,
    /// Total character count including whitespace and symbols.
    pub char_count: usize,
    /// Total line count within the section body.
    pub line_count: usize,
    /// Estimated token count based on heuristic character/word ratios.
    pub estimated_tokens: usize,
    /// Exact token count when verified against a model-specific tokenizer (e.g. tiktoken/cl100k).
    pub exact_tokens: Option<usize>,
    /// Name of the tokenizer used for exact counting, if available.
    pub tokenizer_name: Option<String>,
}

impl SectionTokenMetrics {
    /// Computes baseline empirical metrics from a string slice without exact tokenizer.
    #[must_use]
    pub fn compute_baseline(text: &str) -> Self {
        let char_count = text.chars().count();
        let word_count = text.split_whitespace().count();
        let line_count = if text.is_empty() {
            0
        } else {
            text.lines().count().max(1)
        };
        // Standard rule of thumb: ~4 characters per token in English / code
        let estimated_tokens = char_count.div_ceil(4);

        Self {
            word_count,
            char_count,
            line_count,
            estimated_tokens,
            exact_tokens: None,
            tokenizer_name: None,
        }
    }
}

/// An individual content section in the Canonical Context Document Model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CdmSection {
    /// Unique identifier for this section within the document.
    pub section_id: String,
    /// Optional identifier of the parent section (for hierarchical nesting).
    pub parent_id: Option<String>,
    /// Optional section heading, title, or filename.
    pub title: Option<String>,
    /// The structural kind of this content unit.
    pub kind: SectionKind,
    /// Source-relative path (e.g. `crates/cdm/src/lib.rs` or DOM selector `#main-content`).
    pub original_path: Option<String>,
    /// Direct source URL or permalink referencing this section.
    pub source_url: String,
    /// Optional 1-indexed source line span.
    pub line_range: Option<LineRange>,
    /// Optional byte range in source archive or raw stream.
    pub byte_range: Option<ByteRange>,
    /// Programming or markup language identifier (e.g. `rust`, `python`, `markdown`).
    pub language: Option<String>,
    /// Cryptographic BLAKE3 hex digest calculated over the exact `body` string bytes.
    pub content_hash: String,
    /// The normalized text content of this section.
    pub body: String,
    /// Quantitative volume and token consumption metrics.
    pub token_metrics: Option<SectionTokenMetrics>,
    /// Arbitrary section-specific attributes (e.g. table alignments, Git blob SHA, admonition type).
    pub attributes: BTreeMap<String, String>,
}

impl CdmSection {
    /// Computes the canonical BLAKE3 content hash for a given string body.
    #[must_use]
    pub fn compute_hash(body: &str) -> String {
        blake3::hash(body.as_bytes()).to_hex().to_string()
    }

    /// Verifies whether the section's recorded `content_hash` matches its current `body`.
    #[must_use]
    pub fn verify_hash(&self) -> bool {
        let expected = Self::compute_hash(&self.body);
        self.content_hash.eq_ignore_ascii_case(&expected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_hash_verification() {
        let text = "fn main() { println!(\"Hello, CDM!\"); }";
        let hash = CdmSection::compute_hash(text);

        let section = CdmSection {
            section_id: "sec-001".to_string(),
            parent_id: None,
            title: Some("Main entrypoint".to_string()),
            kind: SectionKind::CodeSnippet,
            original_path: Some("src/main.rs".to_string()),
            source_url: "https://example.com/src/main.rs".to_string(),
            line_range: Some(LineRange::new(1, 3)),
            byte_range: Some(ByteRange::new(0, text.len() as u64)),
            language: Some("rust".to_string()),
            content_hash: hash,
            body: text.to_string(),
            token_metrics: Some(SectionTokenMetrics::compute_baseline(text)),
            attributes: BTreeMap::new(),
        };

        assert!(section.verify_hash());
        assert_eq!(section.line_range.unwrap().line_count(), 3);
        assert_eq!(section.byte_range.unwrap().byte_length(), text.len() as u64);
    }

    #[test]
    fn test_token_metrics_baseline() {
        let sample = "The quick brown fox jumps over the lazy dog.";
        let metrics = SectionTokenMetrics::compute_baseline(sample);
        assert_eq!(metrics.word_count, 9);
        assert_eq!(metrics.line_count, 1);
        assert!(metrics.estimated_tokens > 0);
    }
}
