//! # Markdown Exporter Options & Metrics
//!
//! Configuration models and telemetry metrics for Markdown document generation.

use serde::{Deserialize, Serialize};

/// Style of code fence delimiters used for code blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CodeFenceStyle {
    /// Standard triple backticks (` ``` `), escalating when content contains backticks.
    #[default]
    Backticks,
    /// Standard triple tildes (` ~~~ `), escalating when content contains tildes.
    Tildes,
}

/// Style for rendering the directory/hierarchy tree in Markdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HierarchyStyle {
    /// Unicode box drawing characters (`├──`, `└──`, `│`).
    #[default]
    UnicodeTree,
    /// ASCII box drawing characters (`|--`, `\--`, `|`).
    AsciiTree,
    /// Indented Markdown bulleted list.
    BulletedList,
    /// Do not render the hierarchy tree.
    None,
}

/// Telemetry metrics recorded during Markdown export.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ExportMetrics {
    /// Total bytes written to the output stream.
    pub bytes_written: usize,
    /// Total lines written to the output stream.
    pub lines_written: usize,
    /// Total number of CDM sections successfully exported.
    pub sections_exported: usize,
    /// Total number of exclusion records documented.
    pub exclusions_exported: usize,
    /// Total number of citations documented.
    pub citations_exported: usize,
    /// Aggregate estimated token count across exported sections.
    pub estimated_tokens: usize,
    /// Whether the output was truncated due to a token budget limit.
    pub truncated: bool,
}

/// Configuration options controlling Markdown document generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkdownOptions {
    /// Whether to prepend standard YAML frontmatter (`--- ... ---`).
    pub include_frontmatter: bool,
    /// Whether to render a human/AI-readable metadata summary block.
    pub include_metadata_block: bool,
    /// Hierarchy tree style (Unicode, ASCII, Bulleted, or None).
    pub hierarchy_style: HierarchyStyle,
    /// Whether to generate a Table of Contents with section links.
    pub include_table_of_contents: bool,
    /// Whether to include per-section metadata headers (path, line ranges, token metrics).
    pub include_section_metadata: bool,
    /// Whether to append an Excluded Files section when exclusions are present.
    pub include_exclusions: bool,
    /// Whether to append a Citations & Sources section when citations are present.
    pub include_citations: bool,
    /// Whether to append a Completeness Accounting summary.
    pub include_completeness_report: bool,
    /// Code fence delimiter style (Backticks or Tildes).
    pub code_fence_style: CodeFenceStyle,
    /// Automatically escalate fence delimiter length if section body contains identical fences.
    pub auto_escalate_fences: bool,
    /// Base heading level for section titles (default: 2 -> `##`).
    pub section_heading_level: u8,
    /// Maximum allowed token budget. If reached, remaining sections are omitted with a notice.
    pub max_token_budget: Option<usize>,
    /// Prefix code blocks with line numbers (default: false).
    pub include_line_numbers: bool,
}

impl Default for MarkdownOptions {
    fn default() -> Self {
        Self {
            include_frontmatter: true,
            include_metadata_block: true,
            hierarchy_style: HierarchyStyle::UnicodeTree,
            include_table_of_contents: true,
            include_section_metadata: true,
            include_exclusions: true,
            include_citations: true,
            include_completeness_report: true,
            code_fence_style: CodeFenceStyle::Backticks,
            auto_escalate_fences: true,
            section_heading_level: 2,
            max_token_budget: None,
            include_line_numbers: false,
        }
    }
}

impl MarkdownOptions {
    /// Creates a minimal options configuration (frontmatter + sections only, no TOC or hierarchy).
    #[must_use]
    pub fn minimal() -> Self {
        Self {
            include_frontmatter: true,
            include_metadata_block: false,
            hierarchy_style: HierarchyStyle::None,
            include_table_of_contents: false,
            include_section_metadata: false,
            include_exclusions: false,
            include_citations: false,
            include_completeness_report: false,
            code_fence_style: CodeFenceStyle::Backticks,
            auto_escalate_fences: true,
            section_heading_level: 2,
            max_token_budget: None,
            include_line_numbers: false,
        }
    }

    /// Creates a compact options configuration (concise headers, no extra commentary).
    #[must_use]
    pub fn compact() -> Self {
        Self {
            include_frontmatter: true,
            include_metadata_block: true,
            hierarchy_style: HierarchyStyle::BulletedList,
            include_table_of_contents: false,
            include_section_metadata: false,
            include_exclusions: true,
            include_citations: true,
            include_completeness_report: false,
            code_fence_style: CodeFenceStyle::Backticks,
            auto_escalate_fences: true,
            section_heading_level: 2,
            max_token_budget: None,
            include_line_numbers: false,
        }
    }

    /// Configures frontmatter inclusion.
    #[must_use]
    pub const fn with_frontmatter(mut self, enabled: bool) -> Self {
        self.include_frontmatter = enabled;
        self
    }

    /// Configures metadata block inclusion.
    #[must_use]
    pub const fn with_metadata_block(mut self, enabled: bool) -> Self {
        self.include_metadata_block = enabled;
        self
    }

    /// Configures hierarchy tree rendering style.
    #[must_use]
    pub const fn with_hierarchy_style(mut self, style: HierarchyStyle) -> Self {
        self.hierarchy_style = style;
        self
    }

    /// Configures table of contents inclusion.
    #[must_use]
    pub const fn with_table_of_contents(mut self, enabled: bool) -> Self {
        self.include_table_of_contents = enabled;
        self
    }

    /// Configures section metadata headers inclusion.
    #[must_use]
    pub const fn with_section_metadata(mut self, enabled: bool) -> Self {
        self.include_section_metadata = enabled;
        self
    }

    /// Configures exclusions list inclusion.
    #[must_use]
    pub const fn with_exclusions(mut self, enabled: bool) -> Self {
        self.include_exclusions = enabled;
        self
    }

    /// Configures citations list inclusion.
    #[must_use]
    pub const fn with_citations(mut self, enabled: bool) -> Self {
        self.include_citations = enabled;
        self
    }

    /// Configures completeness report inclusion.
    #[must_use]
    pub const fn with_completeness_report(mut self, enabled: bool) -> Self {
        self.include_completeness_report = enabled;
        self
    }

    /// Configures code fence style.
    #[must_use]
    pub const fn with_code_fence_style(mut self, style: CodeFenceStyle) -> Self {
        self.code_fence_style = style;
        self
    }

    /// Configures automatic fence escalation.
    #[must_use]
    pub const fn with_auto_escalate_fences(mut self, enabled: bool) -> Self {
        self.auto_escalate_fences = enabled;
        self
    }

    /// Sets the base heading level for sections.
    #[must_use]
    pub const fn with_section_heading_level(mut self, level: u8) -> Self {
        self.section_heading_level = if level == 0 { 1 } else { level };
        self
    }

    /// Sets the maximum token budget cap.
    #[must_use]
    pub const fn with_max_token_budget(mut self, budget: Option<usize>) -> Self {
        self.max_token_budget = budget;
        self
    }

    /// Configures line numbers in code blocks.
    #[must_use]
    pub const fn with_line_numbers(mut self, enabled: bool) -> Self {
        self.include_line_numbers = enabled;
        self
    }
}

/// Configuration options controlling Plain-Text (`.txt`) document generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlainTextOptions {
    /// Whether to render the top-level document metadata banner.
    pub include_header_banner: bool,
    /// Hierarchy tree style (Unicode, ASCII, Bulleted, or None).
    pub hierarchy_style: HierarchyStyle,
    /// Whether to generate a plain-text Table of Contents.
    pub include_table_of_contents: bool,
    /// Whether to include section banner delimiters and metadata headers.
    pub include_section_headers: bool,
    /// Whether to append an Excluded Files section when exclusions are present.
    pub include_exclusions: bool,
    /// Whether to append a Citations & Sources section when citations are present.
    pub include_citations: bool,
    /// Whether to append a Completeness Accounting summary.
    pub include_completeness_report: bool,
    /// Character used for section divider lines (e.g. `'-'`, `'='`).
    pub divider_char: char,
    /// Width of divider lines in characters (default: 80).
    pub divider_width: usize,
    /// Maximum allowed token budget. If reached, remaining sections are omitted with a notice.
    pub max_token_budget: Option<usize>,
    /// Prefix code blocks with line numbers (default: false).
    pub include_line_numbers: bool,
}

impl Default for PlainTextOptions {
    fn default() -> Self {
        Self {
            include_header_banner: true,
            hierarchy_style: HierarchyStyle::AsciiTree,
            include_table_of_contents: true,
            include_section_headers: true,
            include_exclusions: true,
            include_citations: true,
            include_completeness_report: true,
            divider_char: '-',
            divider_width: 80,
            max_token_budget: None,
            include_line_numbers: false,
        }
    }
}

impl PlainTextOptions {
    /// Creates a minimal options configuration (raw sections only, no TOC or hierarchy tree).
    #[must_use]
    pub fn minimal() -> Self {
        Self {
            include_header_banner: false,
            hierarchy_style: HierarchyStyle::None,
            include_table_of_contents: false,
            include_section_headers: false,
            include_exclusions: false,
            include_citations: false,
            include_completeness_report: false,
            divider_char: '-',
            divider_width: 80,
            max_token_budget: None,
            include_line_numbers: false,
        }
    }

    /// Creates a compact options configuration (concise headers, no verbose metadata).
    #[must_use]
    pub fn compact() -> Self {
        Self {
            include_header_banner: true,
            hierarchy_style: HierarchyStyle::AsciiTree,
            include_table_of_contents: false,
            include_section_headers: true,
            include_exclusions: true,
            include_citations: true,
            include_completeness_report: false,
            divider_char: '-',
            divider_width: 80,
            max_token_budget: None,
            include_line_numbers: false,
        }
    }

    /// Configures header banner inclusion.
    #[must_use]
    pub const fn with_header_banner(mut self, enabled: bool) -> Self {
        self.include_header_banner = enabled;
        self
    }

    /// Configures hierarchy tree rendering style.
    #[must_use]
    pub const fn with_hierarchy_style(mut self, style: HierarchyStyle) -> Self {
        self.hierarchy_style = style;
        self
    }

    /// Configures table of contents inclusion.
    #[must_use]
    pub const fn with_table_of_contents(mut self, enabled: bool) -> Self {
        self.include_table_of_contents = enabled;
        self
    }

    /// Configures section headers inclusion.
    #[must_use]
    pub const fn with_section_headers(mut self, enabled: bool) -> Self {
        self.include_section_headers = enabled;
        self
    }

    /// Configures exclusions list inclusion.
    #[must_use]
    pub const fn with_exclusions(mut self, enabled: bool) -> Self {
        self.include_exclusions = enabled;
        self
    }

    /// Configures citations list inclusion.
    #[must_use]
    pub const fn with_citations(mut self, enabled: bool) -> Self {
        self.include_citations = enabled;
        self
    }

    /// Configures completeness report inclusion.
    #[must_use]
    pub const fn with_completeness_report(mut self, enabled: bool) -> Self {
        self.include_completeness_report = enabled;
        self
    }

    /// Configures divider line character and width.
    #[must_use]
    pub const fn with_divider(mut self, ch: char, width: usize) -> Self {
        self.divider_char = ch;
        self.divider_width = if width == 0 { 40 } else { width };
        self
    }

    /// Sets the maximum token budget cap.
    #[must_use]
    pub const fn with_max_token_budget(mut self, budget: Option<usize>) -> Self {
        self.max_token_budget = budget;
        self
    }

    /// Configures line numbers in code blocks.
    #[must_use]
    pub const fn with_line_numbers(mut self, enabled: bool) -> Self {
        self.include_line_numbers = enabled;
        self
    }
}
