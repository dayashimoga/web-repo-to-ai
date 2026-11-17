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

/// Flavor or target schema style of JSON export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JsonFormatFlavor {
    /// Full Canonical Context Document Model v1.0 schema (`cdm_json`).
    #[default]
    CanonicalCdm,
    /// Token-optimized structured envelope tailored for direct LLM context injection.
    StructuredContext,
    /// Minified single-line compact JSON without indentation or superfluous whitespace.
    Minified,
}

/// Configuration options controlling JSON document generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonExportOptions {
    /// Flavor or target schema style of the exported JSON.
    pub flavor: JsonFormatFlavor,
    /// Whether to format JSON with multi-line indentation (ignored when flavor is `Minified`).
    pub pretty: bool,
    /// Whether to include document metadata in structured context export.
    pub include_metadata: bool,
    /// Whether to include hierarchy tree in structured context export.
    pub include_hierarchy: bool,
    /// Whether to include sections array in structured context export.
    pub include_sections: bool,
    /// Whether to include citations array in structured context export.
    pub include_citations: bool,
    /// Whether to include exclusions array in structured context export.
    pub include_exclusions: bool,
    /// Whether to include completeness accounting in structured context export.
    pub include_completeness: bool,
    /// Maximum allowed token budget ceiling.
    pub max_token_budget: Option<usize>,
}

impl Default for JsonExportOptions {
    fn default() -> Self {
        Self {
            flavor: JsonFormatFlavor::CanonicalCdm,
            pretty: true,
            include_metadata: true,
            include_hierarchy: true,
            include_sections: true,
            include_citations: true,
            include_exclusions: true,
            include_completeness: true,
            max_token_budget: None,
        }
    }
}

impl JsonExportOptions {
    /// Creates a canonical CDM v1.0 export configuration (pretty-printed).
    #[must_use]
    pub fn canonical() -> Self {
        Self {
            flavor: JsonFormatFlavor::CanonicalCdm,
            pretty: true,
            ..Default::default()
        }
    }

    /// Creates a structured context export configuration tailored for LLMs.
    #[must_use]
    pub fn structured() -> Self {
        Self {
            flavor: JsonFormatFlavor::StructuredContext,
            pretty: true,
            ..Default::default()
        }
    }

    /// Creates a compact/minified export configuration.
    #[must_use]
    pub fn compact() -> Self {
        Self {
            flavor: JsonFormatFlavor::Minified,
            pretty: false,
            ..Default::default()
        }
    }

    /// Configures the JSON flavor.
    #[must_use]
    pub const fn with_flavor(mut self, flavor: JsonFormatFlavor) -> Self {
        self.flavor = flavor;
        self
    }

    /// Configures pretty-printing indentation.
    #[must_use]
    pub const fn with_pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }

    /// Configures metadata inclusion in structured context.
    #[must_use]
    pub const fn with_metadata(mut self, enabled: bool) -> Self {
        self.include_metadata = enabled;
        self
    }

    /// Configures hierarchy tree inclusion in structured context.
    #[must_use]
    pub const fn with_hierarchy(mut self, enabled: bool) -> Self {
        self.include_hierarchy = enabled;
        self
    }

    /// Configures sections inclusion in structured context.
    #[must_use]
    pub const fn with_sections(mut self, enabled: bool) -> Self {
        self.include_sections = enabled;
        self
    }

    /// Configures citations inclusion in structured context.
    #[must_use]
    pub const fn with_citations(mut self, enabled: bool) -> Self {
        self.include_citations = enabled;
        self
    }

    /// Configures exclusions inclusion in structured context.
    #[must_use]
    pub const fn with_exclusions(mut self, enabled: bool) -> Self {
        self.include_exclusions = enabled;
        self
    }

    /// Configures completeness accounting inclusion in structured context.
    #[must_use]
    pub const fn with_completeness(mut self, enabled: bool) -> Self {
        self.include_completeness = enabled;
        self
    }

    /// Configures the maximum token budget cap.
    #[must_use]
    pub const fn with_max_token_budget(mut self, budget: Option<usize>) -> Self {
        self.max_token_budget = budget;
        self
    }
}

/// Configuration options controlling HTML document generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HtmlOptions {
    /// Whether to generate a complete standalone HTML5 document (`<!DOCTYPE html>...`)
    /// or just the body fragment `<div>...`.
    pub standalone: bool,
    /// Document title in the `<title>` tag and page header. If None, derived from CDM metadata.
    pub title: Option<String>,
    /// Whether to embed self-contained responsive CSS stylesheet (dark/light theme aware).
    pub include_css: bool,
    /// Optional custom CSS stylesheet injected into `<style>`.
    pub custom_css: Option<String>,
    /// Whether to render the header banner (title, source URL, metadata badges).
    pub include_header: bool,
    /// Hierarchy tree style (UnicodeTree, AsciiTree, BulletedList, None).
    pub hierarchy_style: HierarchyStyle,
    /// Whether to render a Table of Contents with section links.
    pub include_table_of_contents: bool,
    /// Whether to wrap sections in collapsible `<details><summary>` elements.
    pub collapsible_sections: bool,
    /// Whether sections are initially expanded in collapsible mode (default: true).
    pub initially_expanded: bool,
    /// Whether to include line numbers on code blocks.
    pub include_line_numbers: bool,
    /// Whether to include citations & sources section.
    pub include_citations: bool,
    /// Whether to include excluded files audit table.
    pub include_exclusions: bool,
    /// Whether to include completeness accounting card/table.
    pub include_completeness_report: bool,
    /// Maximum allowed token budget. If reached, remaining sections are omitted.
    pub max_token_budget: Option<usize>,
}

impl Default for HtmlOptions {
    fn default() -> Self {
        Self {
            standalone: true,
            title: None,
            include_css: true,
            custom_css: None,
            include_header: true,
            hierarchy_style: HierarchyStyle::UnicodeTree,
            include_table_of_contents: true,
            collapsible_sections: true,
            initially_expanded: true,
            include_line_numbers: false,
            include_citations: true,
            include_exclusions: true,
            include_completeness_report: true,
            max_token_budget: None,
        }
    }
}

impl HtmlOptions {
    /// Creates a complete standalone HTML5 document configuration with default styling.
    #[must_use]
    pub fn standalone() -> Self {
        Self::default()
    }

    /// Creates an HTML fragment configuration suitable for embedding into an existing page.
    #[must_use]
    pub fn fragment() -> Self {
        Self {
            standalone: false,
            include_css: false,
            include_header: false,
            ..Default::default()
        }
    }

    /// Creates a minimal HTML configuration with no tree, no TOC, and flat sections.
    #[must_use]
    pub fn minimal() -> Self {
        Self {
            standalone: true,
            include_css: true,
            include_header: true,
            hierarchy_style: HierarchyStyle::None,
            include_table_of_contents: false,
            collapsible_sections: false,
            include_citations: false,
            include_exclusions: false,
            include_completeness_report: false,
            ..Default::default()
        }
    }

    /// Sets the standalone document mode.
    #[must_use]
    pub const fn with_standalone(mut self, standalone: bool) -> Self {
        self.standalone = standalone;
        self
    }

    /// Sets the document title.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets whether to include embedded CSS.
    #[must_use]
    pub const fn with_css(mut self, include_css: bool) -> Self {
        self.include_css = include_css;
        self
    }

    /// Sets custom CSS stylesheet.
    #[must_use]
    pub fn with_custom_css(mut self, css: impl Into<String>) -> Self {
        self.custom_css = Some(css.into());
        self
    }

    /// Sets whether to render the header banner.
    #[must_use]
    pub const fn with_header(mut self, enabled: bool) -> Self {
        self.include_header = enabled;
        self
    }

    /// Sets the directory tree rendering style.
    #[must_use]
    pub const fn with_hierarchy_style(mut self, style: HierarchyStyle) -> Self {
        self.hierarchy_style = style;
        self
    }

    /// Sets whether to include the Table of Contents.
    #[must_use]
    pub const fn with_table_of_contents(mut self, enabled: bool) -> Self {
        self.include_table_of_contents = enabled;
        self
    }

    /// Sets whether sections are rendered inside collapsible `<details><summary>` elements.
    #[must_use]
    pub const fn with_collapsible_sections(mut self, enabled: bool) -> Self {
        self.collapsible_sections = enabled;
        self
    }

    /// Sets whether collapsible sections start expanded.
    #[must_use]
    pub const fn with_initially_expanded(mut self, expanded: bool) -> Self {
        self.initially_expanded = expanded;
        self
    }

    /// Sets whether to include code line numbers.
    #[must_use]
    pub const fn with_line_numbers(mut self, enabled: bool) -> Self {
        self.include_line_numbers = enabled;
        self
    }

    /// Sets whether to include citations & sources.
    #[must_use]
    pub const fn with_citations(mut self, enabled: bool) -> Self {
        self.include_citations = enabled;
        self
    }

    /// Sets whether to include excluded files audit table.
    #[must_use]
    pub const fn with_exclusions(mut self, enabled: bool) -> Self {
        self.include_exclusions = enabled;
        self
    }

    /// Sets whether to include completeness report.
    #[must_use]
    pub const fn with_completeness_report(mut self, enabled: bool) -> Self {
        self.include_completeness_report = enabled;
        self
    }

    /// Sets maximum token budget ceiling.
    #[must_use]
    pub const fn with_max_token_budget(mut self, budget: Option<usize>) -> Self {
        self.max_token_budget = budget;
        self
    }
}

/// Flavor format for JSON Lines (`.jsonl`) export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JsonlFlavor {
    /// RAG chunk retrieval records with document coordinates, metadata, and token counts.
    #[default]
    RagChunks,
    /// Chat conversation records with `messages` array for instruction fine-tuning (`[{"role": "user", ...}, {"role": "assistant", ...}]`).
    FineTuningChat,
    /// Prompt / Completion pairs (`{"prompt": "...", "completion": "..."}`) for traditional LLM completion fine-tuning.
    FineTuningPromptCompletion,
    /// Lossless serialized CDM section objects (one `CdmSection` JSON object per line).
    CanonicalSections,
}

/// Configuration options controlling JSON Lines (`.jsonl`) export.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct JsonlOptions {
    /// Output flavor format.
    pub flavor: JsonlFlavor,
    /// Optional system prompt injected into chat records or completion instructions.
    pub system_prompt: Option<String>,
    /// Maximum allowed token budget across exported sections.
    pub max_token_budget: Option<usize>,
}

impl JsonlOptions {
    /// Creates a RAG chunk JSONL configuration.
    #[must_use]
    pub fn rag() -> Self {
        Self {
            flavor: JsonlFlavor::RagChunks,
            system_prompt: None,
            max_token_budget: None,
        }
    }

    /// Creates a chat fine-tuning JSONL configuration with an optional system prompt.
    #[must_use]
    pub fn chat_fine_tuning(system_prompt: Option<impl Into<String>>) -> Self {
        Self {
            flavor: JsonlFlavor::FineTuningChat,
            system_prompt: system_prompt.map(Into::into),
            max_token_budget: None,
        }
    }

    /// Creates a prompt/completion fine-tuning JSONL configuration.
    #[must_use]
    pub fn prompt_completion_fine_tuning() -> Self {
        Self {
            flavor: JsonlFlavor::FineTuningPromptCompletion,
            system_prompt: None,
            max_token_budget: None,
        }
    }

    /// Creates a canonical CDM section streaming JSONL configuration.
    #[must_use]
    pub fn canonical() -> Self {
        Self {
            flavor: JsonlFlavor::CanonicalSections,
            system_prompt: None,
            max_token_budget: None,
        }
    }

    /// Sets the JSONL flavor.
    #[must_use]
    pub const fn with_flavor(mut self, flavor: JsonlFlavor) -> Self {
        self.flavor = flavor;
        self
    }

    /// Sets the optional system prompt.
    #[must_use]
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Sets the maximum token budget cap.
    #[must_use]
    pub const fn with_max_token_budget(mut self, budget: Option<usize>) -> Self {
        self.max_token_budget = budget;
        self
    }
}
