//! # Content Normalization Configuration
//!
//! Fluent configuration builder and domain profiles for character encoding repair,
//! whitespace canonicalization, line-ending standardization, and Markdown formatting.

use serde::{Deserialize, Serialize};

/// Configuration governing character encoding sanitization and Unicode normalization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncodingConfig {
    /// Strip UTF-8 Byte Order Marks (`\u{FEFF}`).
    pub strip_bom: bool,
    /// Strip ASCII null bytes (`\0`).
    pub strip_null_bytes: bool,
    /// Strip unprintable non-whitespace ASCII control characters (`\x01`..=`\x08`, `\x0B`..=`\x0C`, `\x0E`..=`\x1F`, `\x7F`).
    pub strip_control_chars: bool,
    /// Strip invisible zero-width characters (`\u{200B}`, `\u{200C}`, `\u{200D}`).
    pub strip_zero_width_chars: bool,
    /// Normalize smart quotes and typographic dashes to ASCII equivalents (in prose sections).
    pub normalize_typographic_punctuation: bool,
}

impl Default for EncodingConfig {
    fn default() -> Self {
        Self {
            strip_bom: true,
            strip_null_bytes: true,
            strip_control_chars: true,
            strip_zero_width_chars: true,
            normalize_typographic_punctuation: false,
        }
    }
}

/// Configuration governing whitespace trimming and line-ending canonicalization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhitespaceConfig {
    /// Canonicalize all line endings (CRLF `\r\n` and CR `\r`) to standard Unix LF (`\n`).
    pub normalize_line_endings: bool,
    /// Strip trailing whitespace on each line.
    pub trim_trailing_whitespace: bool,
    /// Trim leading blank lines from section bodies.
    pub trim_leading_blank_lines: bool,
    /// Trim trailing blank lines from section bodies.
    pub trim_trailing_blank_lines: bool,
    /// Maximum consecutive blank lines allowed in prose sections (collapsing 2+ down to this value).
    pub max_consecutive_blank_lines_prose: usize,
    /// Maximum consecutive blank lines allowed in code sections (collapsing 3+ down to this value).
    pub max_consecutive_blank_lines_code: usize,
    /// Collapse multiple consecutive non-leading spaces into a single space in prose.
    pub collapse_inline_prose_spaces: bool,
}

impl Default for WhitespaceConfig {
    fn default() -> Self {
        Self {
            normalize_line_endings: true,
            trim_trailing_whitespace: true,
            trim_leading_blank_lines: true,
            trim_trailing_blank_lines: true,
            max_consecutive_blank_lines_prose: 1,
            max_consecutive_blank_lines_code: 2,
            collapse_inline_prose_spaces: false,
        }
    }
}

/// Configuration governing Markdown structural standardization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkdownConfig {
    /// Standardize code block fences to triple backticks with trimmed, lowercase language specifiers.
    pub normalize_code_fences: bool,
    /// Standardize heading levels with exact single space between `#` markers and title text.
    pub normalize_heading_spacing: bool,
    /// Standardize bullet list markers (`* ` -> `- `).
    pub standardize_list_markers: bool,
    /// Rectangularize GFM markdown tables ensuring consistent column counts across all rows.
    pub rectangularize_tables: bool,
    /// Format table cells with compact spacing.
    pub compact_table_cells: bool,
}

impl Default for MarkdownConfig {
    fn default() -> Self {
        Self {
            normalize_code_fences: true,
            normalize_heading_spacing: true,
            standardize_list_markers: true,
            rectangularize_tables: true,
            compact_table_cells: true,
        }
    }
}

/// Master configuration for structural document and section normalization.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct NormalizationConfig {
    /// Character encoding repair settings.
    pub encoding: EncodingConfig,
    /// Whitespace and line-ending settings.
    pub whitespace: WhitespaceConfig,
    /// Markdown syntax and structural alignment settings.
    pub markdown: MarkdownConfig,
}

impl NormalizationConfig {
    /// Preset: Lossless normalization preserving all semantic code and prose characters.
    ///
    /// Cleans CRLF line endings, trims trailing whitespace, removes null bytes/BOM,
    /// and collapses excessive blank lines without altering quotes, symbols, or code tokens.
    #[must_use]
    pub fn lossless() -> Self {
        Self {
            encoding: EncodingConfig {
                strip_bom: true,
                strip_null_bytes: true,
                strip_control_chars: true,
                strip_zero_width_chars: true,
                normalize_typographic_punctuation: false,
            },
            whitespace: WhitespaceConfig {
                normalize_line_endings: true,
                trim_trailing_whitespace: true,
                trim_leading_blank_lines: true,
                trim_trailing_blank_lines: true,
                max_consecutive_blank_lines_prose: 1,
                max_consecutive_blank_lines_code: 2,
                collapse_inline_prose_spaces: false,
            },
            markdown: MarkdownConfig {
                normalize_code_fences: true,
                normalize_heading_spacing: true,
                standardize_list_markers: true,
                rectangularize_tables: true,
                compact_table_cells: true,
            },
        }
    }

    /// Preset: Compact normalization maximizing token savings for LLM context windows.
    ///
    /// Extends lossless normalization with typographic punctuation normalization
    /// (smart quotes to ASCII, em-dashes), single blank line ceilings across all sections,
    /// and inline prose space collapsing.
    #[must_use]
    pub fn compact() -> Self {
        Self {
            encoding: EncodingConfig {
                strip_bom: true,
                strip_null_bytes: true,
                strip_control_chars: true,
                strip_zero_width_chars: true,
                normalize_typographic_punctuation: true,
            },
            whitespace: WhitespaceConfig {
                normalize_line_endings: true,
                trim_trailing_whitespace: true,
                trim_leading_blank_lines: true,
                trim_trailing_blank_lines: true,
                max_consecutive_blank_lines_prose: 1,
                max_consecutive_blank_lines_code: 1,
                collapse_inline_prose_spaces: true,
            },
            markdown: MarkdownConfig {
                normalize_code_fences: true,
                normalize_heading_spacing: true,
                standardize_list_markers: true,
                rectangularize_tables: true,
                compact_table_cells: true,
            },
        }
    }

    /// Preset: Code-focused normalization optimized for source code extraction.
    ///
    /// Strictly protects code indentation, comments, and string literals while
    /// standardizing line breaks, trimming trailing whitespace, and removing BOM/nulls.
    #[must_use]
    pub fn code_focused() -> Self {
        Self {
            encoding: EncodingConfig {
                strip_bom: true,
                strip_null_bytes: true,
                strip_control_chars: true,
                strip_zero_width_chars: true,
                normalize_typographic_punctuation: false,
            },
            whitespace: WhitespaceConfig {
                normalize_line_endings: true,
                trim_trailing_whitespace: true,
                trim_leading_blank_lines: true,
                trim_trailing_blank_lines: true,
                max_consecutive_blank_lines_prose: 1,
                max_consecutive_blank_lines_code: 2,
                collapse_inline_prose_spaces: false,
            },
            markdown: MarkdownConfig {
                normalize_code_fences: true,
                normalize_heading_spacing: false,
                standardize_list_markers: false,
                rectangularize_tables: false,
                compact_table_cells: false,
            },
        }
    }

    /// Fluent builder method to override encoding settings.
    #[must_use]
    pub fn with_encoding(mut self, encoding: EncodingConfig) -> Self {
        self.encoding = encoding;
        self
    }

    /// Fluent builder method to override whitespace settings.
    #[must_use]
    pub fn with_whitespace(mut self, whitespace: WhitespaceConfig) -> Self {
        self.whitespace = whitespace;
        self
    }

    /// Fluent builder method to override markdown settings.
    #[must_use]
    pub fn with_markdown(mut self, markdown: MarkdownConfig) -> Self {
        self.markdown = markdown;
        self
    }
}
