//! # Structured Document Elements Data Models
//!
//! Defines strongly typed representations for extracted HTML tables, code blocks,
//! hyperlinks, and composite element collections.

use serde::{Deserialize, Serialize};

/// Alignment specification for table columns and cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TableAlignment {
    /// No explicit alignment specified (default browser/viewer alignment).
    #[default]
    None,
    /// Left alignment.
    Left,
    /// Center alignment.
    Center,
    /// Right alignment.
    Right,
}

impl TableAlignment {
    /// Parses an alignment attribute or CSS text-align declaration.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        let trimmed = s.trim().to_ascii_lowercase();
        match trimmed.as_str() {
            "left" | "start" => Self::Left,
            "center" | "middle" => Self::Center,
            "right" | "end" => Self::Right,
            _ => Self::None,
        }
    }

    /// Formats the alignment into a GitHub Flavored Markdown delimiter string.
    #[must_use]
    pub fn to_markdown_delimiter(self) -> &'static str {
        match self {
            Self::Left => ":---",
            Self::Center => ":---:",
            Self::Right => "---:",
            Self::None => "---",
        }
    }
}

/// Representation of an extracted HTML table normalized into a rectangular matrix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedTable {
    /// Table caption or title, if defined in `<caption>`.
    pub caption: Option<String>,
    /// Column header titles extracted from `<thead>` or the initial `<th>` row.
    pub headers: Vec<String>,
    /// Data rows where each inner vector represents cells in that row.
    pub rows: Vec<Vec<String>>,
    /// Column alignments matching the table columns.
    pub column_alignments: Vec<TableAlignment>,
    /// Whether explicit header cells (`<th>` or `<thead>`) were detected.
    pub has_headers: bool,
    /// Total row count (including header row if present).
    pub num_rows: usize,
    /// Total column count across the normalized rectangular matrix.
    pub num_cols: usize,
}

impl ExtractedTable {
    /// Checks if the table contains zero data rows and zero headers.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.headers.is_empty() && self.rows.is_empty()
    }

    /// Formats the table into standard GitHub Flavored Markdown (GFM) table syntax.
    ///
    /// Escapes internal pipe characters (`|` -> `\|`) and converts embedded newlines to `<br>`.
    #[must_use]
    pub fn to_markdown(&self) -> String {
        if self.is_empty() {
            return String::new();
        }

        let mut out = String::new();

        // Optional caption as bold title
        if let Some(caption) = &self.caption {
            let trimmed = caption.trim();
            if !trimmed.is_empty() {
                out.push_str(&format!("**{trimmed}**\n\n"));
            }
        }

        let num_cols = if self.num_cols > 0 {
            self.num_cols
        } else if !self.headers.is_empty() {
            self.headers.len()
        } else if let Some(first_row) = self.rows.first() {
            first_row.len()
        } else {
            0
        };

        if num_cols == 0 {
            return out;
        }

        // 1. Header row
        out.push('|');
        if self.has_headers && !self.headers.is_empty() {
            for col_idx in 0..num_cols {
                let cell = self.headers.get(col_idx).map_or("", String::as_str);
                out.push(' ');
                out.push_str(&Self::escape_cell(cell));
                out.push_str(" |");
            }
        } else {
            // Synthetic headers for headless tables to satisfy Markdown spec
            for col_idx in 0..num_cols {
                out.push_str(&format!(" Column {} |", col_idx + 1));
            }
        }
        out.push('\n');

        // 2. Delimiter row
        out.push('|');
        for col_idx in 0..num_cols {
            let align = self
                .column_alignments
                .get(col_idx)
                .copied()
                .unwrap_or_default();
            out.push(' ');
            out.push_str(align.to_markdown_delimiter());
            out.push_str(" |");
        }
        out.push('\n');

        // 3. Data rows
        for row in &self.rows {
            out.push('|');
            for col_idx in 0..num_cols {
                let cell = row.get(col_idx).map_or("", String::as_str);
                out.push(' ');
                out.push_str(&Self::escape_cell(cell));
                out.push_str(" |");
            }
            out.push('\n');
        }

        out
    }

    /// Escapes markdown table special characters in a cell.
    fn escape_cell(cell: &str) -> String {
        let trimmed = cell.trim();
        let pipe_escaped = trimmed.replace('|', "\\|");
        pipe_escaped.replace(['\r', '\n'], " ")
    }
}

/// Representation of an extracted source code block or inline code snippet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedCodeBlock {
    /// The extracted source code string with exact indentation and characters preserved.
    pub code: String,
    /// Detected or declared programming language (e.g. `"rust"`, `"python"`, `"json"`).
    pub language: Option<String>,
    /// Whether this snippet is inline code (`<code>...</code>`) rather than a multiline block.
    pub is_inline: bool,
    /// Number of lines in the code snippet.
    pub line_count: usize,
    /// Whether line number gutters were detected and stripped from the raw block.
    pub has_line_numbers_stripped: bool,
}

impl ExtractedCodeBlock {
    /// Formats the code snippet into standard Markdown (fenced code block or inline backticks).
    #[must_use]
    pub fn to_markdown(&self) -> String {
        if self.is_inline {
            if self.code.contains('`') {
                format!("`` {} ``", self.code)
            } else {
                format!("`{}`", self.code)
            }
        } else {
            let lang = self.language.as_deref().unwrap_or("");
            format!("```{lang}\n{}\n```", self.code.trim_end())
        }
    }
}

/// Classification category for extracted hyperlinks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkType {
    /// Link pointing to a page on the same host or domain as the base URL.
    Internal,
    /// Link pointing to a page on an external domain.
    External,
    /// In-page section anchor reference (e.g., `#installation`).
    Anchor,
    /// Link to a static media or downloadable asset file (e.g. `.pdf`, `.zip`, `.png`).
    Asset,
    /// Special URI scheme (`mailto:`, `tel:`, `sms:`).
    Special,
}

/// Representation of an extracted and canonicalized hyperlink.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedLink {
    /// Fully resolved absolute URL (or original scheme string if special/anchor).
    pub url: String,
    /// Original unmodified `href` attribute from the HTML source.
    pub original_href: String,
    /// Visible anchor text inside the `<a>` tag, normalized.
    pub text: String,
    /// `title` attribute of the `<a>` tag, if present.
    pub title: Option<String>,
    /// Whitespace-split `rel` attribute tokens (e.g., `["nofollow", "noopener"]`).
    pub rel: Vec<String>,
    /// Categorized link classification.
    pub link_type: LinkType,
}

/// Composite container aggregating all structured elements extracted from a document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ExtractedElements {
    /// Extracted and normalized HTML tables.
    pub tables: Vec<ExtractedTable>,
    /// Extracted multiline and inline code blocks.
    pub code_blocks: Vec<ExtractedCodeBlock>,
    /// Extracted hyperlinks categorized by scope and asset type.
    pub links: Vec<ExtractedLink>,
}

impl ExtractedElements {
    /// Total count of all extracted structured elements.
    #[must_use]
    pub fn total_count(&self) -> usize {
        self.tables.len() + self.code_blocks.len() + self.links.len()
    }

    /// Checks if no elements of any category were extracted.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tables.is_empty() && self.code_blocks.is_empty() && self.links.is_empty()
    }
}

/// Configuration options for element extraction.
#[derive(Debug, Clone)]
pub struct ElementsOptions {
    /// Whether to extract tables.
    pub extract_tables: bool,
    /// Whether to extract code blocks.
    pub extract_code_blocks: bool,
    /// Whether to extract hyperlinks.
    pub extract_links: bool,
    /// Whether to include inline `<code>` snippets in extracted code blocks.
    pub include_inline_code: bool,
    /// Whether to heuristically detect language syntax when not declared in HTML.
    pub detect_code_language: bool,
    /// Whether to detect and strip line number gutters from code blocks.
    pub strip_line_numbers: bool,
    /// Minimum total rows required for a table to be extracted (default 1).
    pub min_table_rows: usize,
}

impl Default for ElementsOptions {
    fn default() -> Self {
        Self {
            extract_tables: true,
            extract_code_blocks: true,
            extract_links: true,
            include_inline_code: false,
            detect_code_language: true,
            strip_line_numbers: true,
            min_table_rows: 1,
        }
    }
}

impl ElementsOptions {
    /// Sets whether to include inline `<code>` snippets.
    #[must_use]
    pub fn with_inline_code(mut self, include: bool) -> Self {
        self.include_inline_code = include;
        self
    }

    /// Sets whether to heuristically detect syntax for untagged code blocks.
    #[must_use]
    pub fn with_detect_language(mut self, detect: bool) -> Self {
        self.detect_code_language = detect;
        self
    }

    /// Sets minimum table rows required for extraction.
    #[must_use]
    pub fn with_min_table_rows(mut self, min: usize) -> Self {
        self.min_table_rows = min;
        self
    }
}
