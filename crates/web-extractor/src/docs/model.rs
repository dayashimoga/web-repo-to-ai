//! # Technical Documentation Models
//!
//! Strongly-typed representations of documentation pages, navigation hierarchies,
//! breadcrumbs, tables of contents, callouts/admonitions, and pagination links.

use serde::{Deserialize, Serialize};

/// Known documentation generator framework patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocFramework {
    /// Docusaurus (Meta/Facebook) documentation generator.
    Docusaurus,
    /// mdBook (Rust community) static book generator.
    MdBook,
    /// Sphinx (Python / ReadTheDocs) documentation generator.
    Sphinx,
    /// MkDocs / Material for MkDocs.
    MkDocs,
    /// Rustdoc (official Rust compiler documentation tool).
    Rustdoc,
    /// GitBook platform documentation.
    GitBook,
    /// Nextra (Next.js) documentation generator.
    Nextra,
    /// VitePress (Vue.js) documentation generator.
    VitePress,
    /// Standard or unrecognized technical documentation layout.
    Generic,
}

/// Type of technical callout or admonition block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdmonitionType {
    /// Informational note (`note`).
    Note,
    /// Helpful tip or recommendation (`tip`).
    Tip,
    /// Contextual explanation or info (`info`).
    Info,
    /// Important caveat or requirement (`important`).
    Important,
    /// Warning of potential errors or misuse (`warning`).
    Warning,
    /// Caution regarding dangerous actions (`caution`).
    Caution,
    /// High-severity hazard or destructive operation (`danger`).
    Danger,
}

/// Structured callout or admonition block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Admonition {
    /// Severity / kind of admonition.
    pub kind: AdmonitionType,
    /// Optional header or custom title.
    pub title: Option<String>,
    /// Plain-text content of the admonition.
    pub content: String,
}

/// Single item in a hierarchical breadcrumb trail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BreadcrumbItem {
    /// Display text of the breadcrumb.
    pub label: String,
    /// Target URL if linked, canonicalized against the base URL.
    pub url: Option<String>,
    /// 1-based index indicating position in the breadcrumb trail.
    pub position: usize,
}

/// Node in an on-page table of contents or heading hierarchy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TocItem {
    /// Heading level (1 for `<h1>`, 2 for `<h2>`, ..., 6 for `<h6>`).
    pub level: u8,
    /// Clean plain-text title of the heading.
    pub title: String,
    /// Anchor ID fragment (without leading `#`, e.g. `"installation"`).
    pub anchor_id: Option<String>,
    /// Nested child headings under this section.
    pub children: Vec<TocItem>,
}

/// Document link with display title and resolved target URL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocLink {
    /// Link text or page title.
    pub title: String,
    /// Canonical target URL or relative path.
    pub url: String,
}

/// Pagination navigation controls (previous and next page links).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DocPagination {
    /// Link to the preceding documentation page.
    pub prev: Option<DocLink>,
    /// Link to the succeeding documentation page.
    pub next: Option<DocLink>,
}

/// Detected version information and available distribution branches.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DocVersionInfo {
    /// Currently viewed documentation version (e.g. `"v2.4"` or `"latest"`).
    pub current_version: Option<String>,
    /// Available versions detected in version selectors or dropdowns.
    pub available_versions: Vec<String>,
}

/// Structured documentation section delimited by heading boundaries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocSection {
    /// Section heading title.
    pub heading: String,
    /// Heading level (1 to 6).
    pub level: u8,
    /// Anchor ID for direct linking.
    pub anchor_id: Option<String>,
    /// Plain text body content belonging directly to this section.
    pub content_text: String,
}

/// Complete representation of an extracted technical documentation page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractedDocumentation {
    /// Primary page title or main heading.
    pub title: String,
    /// Detected documentation generator framework.
    pub framework: DocFramework,
    /// Hierarchical breadcrumb navigation trail.
    pub breadcrumbs: Vec<BreadcrumbItem>,
    /// On-page table of contents hierarchy.
    pub table_of_contents: Vec<TocItem>,
    /// Previous and next page navigation links.
    pub pagination: DocPagination,
    /// Version selector metadata if detected.
    pub version_info: Option<DocVersionInfo>,
    /// Extracted callout / admonition boxes.
    pub admonitions: Vec<Admonition>,
    /// Section breakdown segmented along heading boundaries.
    pub sections: Vec<DocSection>,
    /// Cleaned, boilerplate-free HTML markup representing the core doc content.
    pub content_html: String,
    /// Clean plain text of the documentation body.
    pub content_text: String,
    /// Total word count.
    pub word_count: usize,
    /// Estimated reading time in minutes (`word_count.div_ceil(200)`).
    pub reading_time_minutes: usize,
    /// Document natural language code if present.
    pub language: Option<String>,
}

impl ExtractedDocumentation {
    /// Checks whether the extracted documentation body is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.word_count == 0 || self.content_text.trim().is_empty()
    }
}
