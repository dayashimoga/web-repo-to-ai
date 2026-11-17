//! # Semantic HTML Document Exporter (`crates/exporters/src/html.rs`)
//!
//! Generates clean, standalone, responsive HTML5 documents from CDM documents.
//! Supports dark/light mode, collapsible sections, table of contents, code blocks
//! with optional line numbers, exclusion tables, citations, and completeness metrics.

use std::io::Write;

use chrono::SecondsFormat;
use web_repo_to_ai_cdm::{CdmDocument, CdmSection, HierarchyKind, HierarchyNode};

use crate::error::Result;
use crate::options::{ExportMetrics, HierarchyStyle, HtmlOptions};
use crate::util::{format_bytes, CountingWriter};

/// Default embedded responsive CSS stylesheet.
pub const DEFAULT_HTML_CSS: &str = r#"
:root {
  --bg: #ffffff;
  --text: #1f2328;
  --text-muted: #656d76;
  --card-bg: #f6f8fa;
  --card-border: #d0d7de;
  --code-bg: #f6f8fa;
  --code-border: #d0d7de;
  --link: #0969da;
  --link-hover: #1a7f37;
  --badge-bg: #eaeef2;
  --badge-text: #24292f;
  --summary-hover: #f3f4f6;
  --line-num: #8c959f;
  --alert-bg: #fff8c5;
  --alert-border: #d4a72c;
  --alert-text: #4d2d00;
  --tag-code: #0969da;
  --tag-doc: #1a7f37;
}
@media (prefers-color-scheme: dark) {
  :root {
    --bg: #0d1117;
    --text: #e6edf3;
    --text-muted: #8b949e;
    --card-bg: #161b22;
    --card-border: #30363d;
    --code-bg: #161b22;
    --code-border: #30363d;
    --link: #4493f8;
    --link-hover: #3fb950;
    --badge-bg: #21262d;
    --badge-text: #c9d1d9;
    --summary-hover: #1c2128;
    --line-num: #6e7681;
    --alert-bg: #342800;
    --alert-border: #9e6a03;
    --alert-text: #f0c674;
    --tag-code: #58a6ff;
    --tag-doc: #3fb950;
  }
}
* { box-sizing: border-box; }
body {
  margin: 0;
  padding: 0;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
  background-color: var(--bg);
  color: var(--text);
  line-height: 1.6;
}
.cdm-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 32px 24px;
}
header.cdm-header {
  border-bottom: 2px solid var(--card-border);
  padding-bottom: 24px;
  margin-bottom: 32px;
}
h1.cdm-title {
  margin: 0 0 12px 0;
  font-size: 2.2rem;
  letter-spacing: -0.02em;
}
.meta-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: 12px;
  margin-top: 16px;
  font-size: 0.95rem;
}
.meta-item {
  background: var(--card-bg);
  border: 1px solid var(--card-border);
  border-radius: 6px;
  padding: 8px 14px;
}
.meta-label {
  font-weight: 600;
  color: var(--text-muted);
  font-size: 0.85rem;
  text-transform: uppercase;
  display: block;
}
a { color: var(--link); text-decoration: none; }
a:hover { text-decoration: underline; }
.badge {
  display: inline-block;
  padding: 2px 8px;
  font-size: 0.75rem;
  font-weight: 600;
  border-radius: 12px;
  background-color: var(--badge-bg);
  color: var(--badge-text);
  margin-right: 6px;
}
.badge-code { color: var(--tag-code); border: 1px solid var(--tag-code); }
.badge-doc { color: var(--tag-doc); border: 1px solid var(--tag-doc); }
section.cdm-card {
  background: var(--card-bg);
  border: 1px solid var(--card-border);
  border-radius: 8px;
  padding: 20px;
  margin-bottom: 28px;
}
h2.card-title {
  margin-top: 0;
  margin-bottom: 16px;
  font-size: 1.4rem;
  border-bottom: 1px solid var(--card-border);
  padding-bottom: 8px;
}
pre.tree-pre {
  background: var(--bg);
  border: 1px solid var(--card-border);
  border-radius: 6px;
  padding: 16px;
  overflow-x: auto;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 0.9rem;
  line-height: 1.4;
}
ol.toc-list {
  padding-left: 24px;
  margin: 0;
}
ol.toc-list li {
  margin-bottom: 6px;
}
article.cdm-section {
  margin-bottom: 20px;
  border: 1px solid var(--card-border);
  border-radius: 8px;
  overflow: hidden;
  background: var(--bg);
}
details.section-details summary {
  padding: 14px 18px;
  background: var(--card-bg);
  cursor: pointer;
  font-weight: 600;
  font-size: 1.05rem;
  user-select: none;
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 8px;
}
details.section-details summary:hover {
  background: var(--summary-hover);
}
.section-body {
  padding: 18px;
}
.section-meta-bar {
  font-size: 0.85rem;
  color: var(--text-muted);
  margin-bottom: 12px;
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}
pre.code-block {
  margin: 0;
  padding: 16px;
  background: var(--code-bg);
  border: 1px solid var(--code-border);
  border-radius: 6px;
  overflow-x: auto;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 0.9rem;
  line-height: 1.45;
}
.line-num {
  color: var(--line-num);
  display: inline-block;
  width: 3.5em;
  text-align: right;
  margin-right: 1.2em;
  user-select: none;
}
table.cdm-table {
  width: 100%;
  border-collapse: collapse;
  margin-top: 12px;
  font-size: 0.9rem;
}
table.cdm-table th, table.cdm-table td {
  border: 1px solid var(--card-border);
  padding: 10px 14px;
  text-align: left;
}
table.cdm-table th {
  background: var(--card-bg);
  font-weight: 600;
}
table.cdm-table tr:nth-child(even) {
  background: var(--card-bg);
}
.budget-alert {
  background: var(--alert-bg);
  border: 1px solid var(--alert-border);
  color: var(--alert-text);
  border-radius: 6px;
  padding: 14px 18px;
  margin: 20px 0;
  font-weight: 500;
}
footer.cdm-footer {
  margin-top: 48px;
  padding-top: 24px;
  border-top: 1px solid var(--card-border);
  text-align: center;
  color: var(--text-muted);
  font-size: 0.85rem;
}
"#;

/// Escapes special HTML characters in text to prevent XSS.
#[must_use]
pub fn escape_html(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            other => escaped.push(other),
        }
    }
    escaped
}

/// Generates a valid HTML slug for anchor IDs from section titles.
#[must_use]
pub fn slugify(title: &str) -> String {
    let mut slug = String::with_capacity(title.len());
    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else if (ch == ' ' || ch == '-' || ch == '_') && !slug.ends_with('-') && !slug.is_empty()
        {
            slug.push('-');
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        "section".to_string()
    } else {
        slug
    }
}

fn section_display_title(section: &CdmSection) -> &str {
    if let Some(ref title) = section.title {
        if !title.is_empty() {
            return title.as_str();
        }
    }
    if let Some(ref path) = section.original_path {
        return path.as_str();
    }
    "Untitled Section"
}

fn section_estimated_tokens(section: &CdmSection) -> usize {
    section.token_metrics.as_ref().map_or_else(
        || section.body.chars().count().div_ceil(4),
        |m| m.estimated_tokens,
    )
}

/// High-performance HTML document and fragment exporter.
#[derive(Debug, Clone, Default)]
pub struct HtmlExporter {
    options: HtmlOptions,
}

impl HtmlExporter {
    /// Creates a new HTML exporter with default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an HTML exporter with the specified configuration options.
    #[must_use]
    pub const fn with_options(options: HtmlOptions) -> Self {
        Self { options }
    }

    /// Exports the CDM document into an in-memory HTML string.
    ///
    /// # Errors
    /// Returns `ExporterError` on rendering failure.
    pub fn export_to_string(&self, doc: &CdmDocument) -> Result<(String, ExportMetrics)> {
        export_html(doc, &self.options)
    }

    /// Streams the HTML document directly to any `std::io::Write` sink.
    ///
    /// # Errors
    /// Returns `ExporterError` on write failure.
    pub fn export_to_writer<W: Write>(
        &self,
        doc: &CdmDocument,
        writer: &mut W,
    ) -> Result<ExportMetrics> {
        export_html_to_writer(doc, &self.options, writer)
    }
}

/// Convenience function exporting a CDM document to an in-memory HTML string.
///
/// # Errors
/// Returns `ExporterError` on formatting or encoding failure.
pub fn export_html(doc: &CdmDocument, options: &HtmlOptions) -> Result<(String, ExportMetrics)> {
    let mut buffer = Vec::with_capacity(16 * 1024);
    let metrics = export_html_to_writer(doc, options, &mut buffer)?;
    let html_string = String::from_utf8(buffer)
        .map_err(|e| crate::error::ExporterError::Formatting(e.to_string()))?;
    Ok((html_string, metrics))
}

/// Streams a CDM document as formatted HTML directly to an `io::Write` destination.
///
/// # Errors
/// Returns `ExporterError` if writing fails.
pub fn export_html_to_writer<W: Write>(
    doc: &CdmDocument,
    options: &HtmlOptions,
    writer: &mut W,
) -> Result<ExportMetrics> {
    let mut cw = CountingWriter::new(writer);
    let mut metrics = ExportMetrics::default();

    let doc_title = options
        .title
        .as_deref()
        .or(doc.metadata.title.as_deref())
        .unwrap_or("CDM Document");
    let escaped_title = escape_html(doc_title);

    // 1. Standalone HTML Head
    if options.standalone {
        writeln!(cw, "<!DOCTYPE html>")?;
        writeln!(cw, "<html lang=\"en\">")?;
        writeln!(cw, "<head>")?;
        writeln!(cw, "  <meta charset=\"utf-8\">")?;
        writeln!(
            cw,
            "  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">"
        )?;
        writeln!(cw, "  <title>{}</title>", escaped_title)?;
        if options.include_css {
            writeln!(cw, "  <style>{}</style>", DEFAULT_HTML_CSS)?;
        }
        if let Some(ref custom_css) = options.custom_css {
            writeln!(cw, "  <style>{}</style>", custom_css)?;
        }
        writeln!(cw, "</head>")?;
        writeln!(cw, "<body>")?;
        writeln!(cw, "  <main class=\"cdm-container\">")?;
    } else {
        writeln!(cw, "<div class=\"cdm-container\">")?;
    }

    // 2. Header Banner
    if options.include_header {
        writeln!(cw, "    <header class=\"cdm-header\">")?;
        writeln!(cw, "      <h1 class=\"cdm-title\">{}</h1>", escaped_title)?;
        writeln!(cw, "      <div class=\"meta-grid\">")?;

        // Source URL
        writeln!(
            cw,
            "        <div class=\"meta-item\"><span class=\"meta-label\">Source URL</span><a href=\"{}\" target=\"_blank\" rel=\"noopener noreferrer\">{}</a></div>",
            escape_html(&doc.canonical_url),
            escape_html(&doc.canonical_url)
        )?;

        // Source Type
        writeln!(
            cw,
            "        <div class=\"meta-item\"><span class=\"meta-label\">Type</span>{}</div>",
            escape_html(doc.source_type.as_str())
        )?;

        // Retrieved timestamp
        let retrieved_iso = doc.retrieved_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        writeln!(
            cw,
            "        <div class=\"meta-item\"><span class=\"meta-label\">Retrieved</span>{}</div>",
            retrieved_iso
        )?;

        // License
        if let Some(ref lic) = doc.metadata.license {
            writeln!(
                cw,
                "        <div class=\"meta-item\"><span class=\"meta-label\">License</span>{}</div>",
                escape_html(lic)
            )?;
        }

        // Language
        if let Some(ref lang) = doc.metadata.primary_language {
            writeln!(
                cw,
                "        <div class=\"meta-item\"><span class=\"meta-label\">Language</span>{}</div>",
                escape_html(lang)
            )?;
        }

        // Authors
        if !doc.metadata.authors.is_empty() {
            writeln!(
                cw,
                "        <div class=\"meta-item\"><span class=\"meta-label\">Authors</span>{}</div>",
                escape_html(&doc.metadata.authors.join(", "))
            )?;
        }

        // Commit SHA
        if let Some(ref sha) = doc.revision.git_commit {
            writeln!(
                cw,
                "        <div class=\"meta-item\"><span class=\"meta-label\">Commit</span><code>{}</code></div>",
                escape_html(sha)
            )?;
        }

        // Metrics (Sections / Tokens / Words / Chars)
        let total_chars: usize = doc.sections.iter().map(|s| s.body.len()).sum();
        writeln!(
            cw,
            "        <div class=\"meta-item\"><span class=\"meta-label\">Metrics</span>{} sections &bull; ~{} estimated tokens &bull; {}</div>",
            doc.sections.len(),
            doc.total_estimated_tokens(),
            format_bytes(total_chars as u64)
        )?;

        writeln!(cw, "      </div>")?;
        writeln!(cw, "    </header>")?;
    }

    // 3. Completeness Card
    if options.include_completeness_report {
        let comp = &doc.completeness;
        let pct = comp.extraction_ratio() * 100.0;
        writeln!(cw, "    <section class=\"cdm-card cdm-completeness\">")?;
        writeln!(
            cw,
            "      <h2 class=\"card-title\">Completeness Yield: {:.1}% ({}/{})</h2>",
            pct, comp.extracted_count, comp.discovered_count
        )?;
        writeln!(cw, "      <table class=\"cdm-table\">")?;
        writeln!(cw, "        <thead><tr><th>Discovered</th><th>Extracted</th><th>Excluded</th><th>Failed</th><th>Unsupported</th><th>Yield</th></tr></thead>")?;
        writeln!(
            cw,
            "        <tbody><tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td><strong>{:.1}%</strong></td></tr></tbody>",
            comp.discovered_count, comp.extracted_count, comp.excluded_count, comp.failed_count, comp.unsupported_count, pct
        )?;
        writeln!(cw, "      </table>")?;
        writeln!(cw, "    </section>")?;
    }

    // 4. Hierarchy / Directory Tree
    if options.hierarchy_style != HierarchyStyle::None
        && (!doc.hierarchy.children.is_empty() || !doc.hierarchy.name.is_empty())
    {
        writeln!(cw, "    <section class=\"cdm-card cdm-tree\">")?;
        writeln!(
            cw,
            "      <h2 class=\"card-title\">Directory &amp; Structure Tree</h2>"
        )?;
        match options.hierarchy_style {
            HierarchyStyle::BulletedList => {
                writeln!(cw, "      <ul class=\"tree-list\">")?;
                render_html_tree_bulleted(&mut cw, &doc.hierarchy)?;
                writeln!(cw, "      </ul>")?;
            }
            HierarchyStyle::UnicodeTree | HierarchyStyle::AsciiTree => {
                writeln!(cw, "      <pre class=\"tree-pre\"><code>")?;
                render_html_tree_ascii(
                    &mut cw,
                    &doc.hierarchy,
                    "",
                    true,
                    options.hierarchy_style == HierarchyStyle::UnicodeTree,
                )?;
                writeln!(cw, "</code></pre>")?;
            }
            HierarchyStyle::None => {}
        }
        writeln!(cw, "    </section>")?;
    }

    // 5. Table of Contents
    if options.include_table_of_contents && !doc.sections.is_empty() {
        writeln!(cw, "    <section class=\"cdm-card cdm-toc\">")?;
        writeln!(cw, "      <h2 class=\"card-title\">Table of Contents</h2>")?;
        writeln!(cw, "      <ol class=\"toc-list\">")?;
        for (idx, sec) in doc.sections.iter().enumerate() {
            let title = section_display_title(sec);
            let slug = format!("{}-{}", slugify(title), idx + 1);
            let kind_str = sec.kind.as_str();
            let lang_badge = sec
                .language
                .as_deref()
                .map(|l| format!("<span class=\"badge\">{}</span>", escape_html(l)))
                .unwrap_or_default();
            let estimated = section_estimated_tokens(sec);
            writeln!(
                cw,
                "        <li><a href=\"#section-{}\">{}</a> <span class=\"badge badge-code\">{}</span>{} <span style=\"color: var(--text-muted); font-size: 0.85rem;\">(~{} tokens)</span></li>",
                slug,
                escape_html(title),
                escape_html(kind_str),
                lang_badge,
                estimated
            )?;
        }
        writeln!(cw, "      </ol>")?;
        writeln!(cw, "    </section>")?;
    }

    // 6. Content Sections
    writeln!(cw, "    <section class=\"cdm-sections\">")?;
    let mut cumulative_tokens = 0usize;
    let mut budget_hit = false;

    for (idx, sec) in doc.sections.iter().enumerate() {
        let estimated = section_estimated_tokens(sec);
        if let Some(budget) = options.max_token_budget {
            if cumulative_tokens.saturating_add(estimated) > budget {
                budget_hit = true;
                metrics.truncated = true;
                break;
            }
        }
        cumulative_tokens = cumulative_tokens.saturating_add(estimated);
        metrics.sections_exported += 1;
        metrics.estimated_tokens += estimated;

        let title = section_display_title(sec);
        let slug = format!("{}-{}", slugify(title), idx + 1);
        let kind_str = sec.kind.as_str();
        let lang_str = sec.language.as_deref().unwrap_or("text");

        writeln!(
            cw,
            "      <article id=\"section-{}\" class=\"cdm-section\">",
            slug
        )?;

        if options.collapsible_sections {
            let open_attr = if options.initially_expanded {
                " open"
            } else {
                ""
            };
            writeln!(
                cw,
                "        <details class=\"section-details\"{}>",
                open_attr
            )?;
            writeln!(
                cw,
                "          <summary><span>{} <span class=\"badge badge-code\">{}</span> <span class=\"badge\">{}</span></span> <span style=\"font-size: 0.85rem; color: var(--text-muted); font-weight: normal;\">~{} tokens</span></summary>",
                escape_html(title),
                escape_html(kind_str),
                escape_html(lang_str),
                estimated
            )?;
            writeln!(cw, "          <div class=\"section-body\">")?;
        } else {
            writeln!(cw, "        <div class=\"section-body\">")?;
            writeln!(
                cw,
                "          <h3>{} <span class=\"badge badge-code\">{}</span> <span class=\"badge\">{}</span></h3>",
                escape_html(title),
                escape_html(kind_str),
                escape_html(lang_str)
            )?;
        }

        // Section metadata bar
        writeln!(cw, "            <div class=\"section-meta-bar\">")?;
        if let Some(ref path) = sec.original_path {
            writeln!(
                cw,
                "              <span><strong>Path:</strong> <code>{}</code></span>",
                escape_html(path)
            )?;
        }
        if let Some(lines) = sec.line_range {
            writeln!(
                cw,
                "              <span><strong>Lines:</strong> {}-{}</span>",
                lines.start_line, lines.end_line
            )?;
        }
        if let Some(bytes) = sec.byte_range {
            writeln!(
                cw,
                "              <span><strong>Size:</strong> {}</span>",
                format_bytes(bytes.byte_length())
            )?;
        }
        writeln!(
            cw,
            "              <span><strong>Tokens:</strong> ~{}</span>",
            estimated
        )?;
        writeln!(cw, "            </div>")?;

        // Section code body
        writeln!(cw, "            <pre class=\"code-block\"><code>")?;
        if options.include_line_numbers {
            let start_line = sec.line_range.map_or(1, |lr| lr.start_line as usize);
            for (line_idx, line) in sec.body.lines().enumerate() {
                let current_line = start_line + line_idx;
                writeln!(
                    cw,
                    "<span class=\"line-num\">{:4}</span>{}",
                    current_line,
                    escape_html(line)
                )?;
            }
        } else {
            writeln!(cw, "{}", escape_html(&sec.body))?;
        }
        writeln!(cw, "</code></pre>")?;

        if options.collapsible_sections {
            writeln!(cw, "          </div>")?;
            writeln!(cw, "        </details>")?;
        } else {
            writeln!(cw, "        </div>")?;
        }
        writeln!(cw, "      </article>")?;
    }

    if budget_hit {
        writeln!(
            cw,
            "      <div class=\"budget-alert\">&#9888; Content truncated: reached maximum token budget of {} tokens. Remaining sections omitted.</div>",
            options.max_token_budget.unwrap_or(0)
        )?;
    }
    writeln!(cw, "    </section>")?;

    // 7. Sources & Citations
    if options.include_citations && !doc.citations.is_empty() {
        writeln!(cw, "    <section class=\"cdm-card cdm-citations\">")?;
        writeln!(
            cw,
            "      <h2 class=\"card-title\">Sources &amp; Citations</h2>"
        )?;
        writeln!(cw, "      <table class=\"cdm-table\">")?;
        writeln!(
            cw,
            "        <thead><tr><th>#</th><th>Title</th><th>URL</th><th>Citation</th></tr></thead>"
        )?;
        writeln!(cw, "        <tbody>")?;
        for (c_idx, cite) in doc.citations.iter().enumerate() {
            metrics.citations_exported += 1;
            let display_title = cite.title.as_deref().unwrap_or(&cite.url);
            let url_link = format!(
                "<a href=\"{}\" target=\"_blank\" rel=\"noopener noreferrer\">{}</a>",
                escape_html(&cite.url),
                escape_html(&cite.url)
            );
            writeln!(
                cw,
                "          <tr><td>{}</td><td>{}</td><td>{}</td><td><code>{}</code></td></tr>",
                c_idx + 1,
                escape_html(display_title),
                url_link,
                escape_html(&cite.concise)
            )?;
        }
        writeln!(cw, "        </tbody>")?;
        writeln!(cw, "      </table>")?;
        writeln!(cw, "    </section>")?;
    }

    // 8. Excluded Content
    if options.include_exclusions && !doc.exclusions.is_empty() {
        writeln!(cw, "    <section class=\"cdm-card cdm-exclusions\">")?;
        writeln!(
            cw,
            "      <h2 class=\"card-title\">Excluded Content &amp; Files</h2>"
        )?;
        writeln!(cw, "      <table class=\"cdm-table\">")?;
        writeln!(cw, "        <thead><tr><th>Path</th><th>Reason</th><th>Rule / Pattern</th><th>Size</th><th>Details</th></tr></thead>")?;
        writeln!(cw, "        <tbody>")?;
        for excl in &doc.exclusions {
            metrics.exclusions_exported += 1;
            let reason_str = format!("{:?}", excl.reason);
            let pattern_str = excl.pattern.as_deref().unwrap_or("—");
            let size_str = excl.byte_size.map_or("—".to_string(), format_bytes);
            let details_str = excl.details.as_deref().unwrap_or("—");
            writeln!(
                cw,
                "          <tr><td><code>{}</code></td><td>{}</td><td><code>{}</code></td><td>{}</td><td>{}</td></tr>",
                escape_html(&excl.original_path),
                escape_html(&reason_str),
                escape_html(pattern_str),
                size_str,
                escape_html(details_str)
            )?;
        }
        writeln!(cw, "        </tbody>")?;
        writeln!(cw, "      </table>")?;
        writeln!(cw, "    </section>")?;
    }

    // 9. Footer
    writeln!(cw, "    <footer class=\"cdm-footer\">")?;
    writeln!(
        cw,
        "      Generated by <strong>web-repo-to-ai</strong> v0.1.0 &bull; Canonical Context Document Model (CDM v1.0)"
    )?;
    writeln!(cw, "    </footer>")?;

    // Close container & HTML tags
    if options.standalone {
        writeln!(cw, "  </main>")?;
        writeln!(cw, "</body>")?;
        writeln!(cw, "</html>")?;
    } else {
        writeln!(cw, "</div>")?;
    }

    cw.flush()?;
    metrics.bytes_written = cw.bytes_written;
    metrics.lines_written = cw.lines_written;

    Ok(metrics)
}

fn render_html_tree_bulleted<W: Write>(w: &mut W, node: &HierarchyNode) -> std::io::Result<()> {
    let name = if node.name.is_empty() {
        "root"
    } else {
        &node.name
    };
    let kind_badge = format!("{:?}", node.kind).to_lowercase();
    writeln!(
        w,
        "        <li><code>{}</code> <span class=\"badge badge-code\">{}</span>",
        escape_html(name),
        escape_html(&kind_badge)
    )?;
    if !node.children.is_empty() {
        writeln!(w, "          <ul>")?;
        for child in &node.children {
            render_html_tree_bulleted(w, child)?;
        }
        writeln!(w, "          </ul>")?;
    }
    writeln!(w, "        </li>")?;
    Ok(())
}

fn render_html_tree_ascii<W: Write>(
    w: &mut W,
    node: &HierarchyNode,
    prefix: &str,
    is_last: bool,
    unicode: bool,
) -> std::io::Result<()> {
    let (connector, child_prefix) = if unicode {
        if is_last {
            ("└── ", format!("{prefix}    "))
        } else {
            ("├── ", format!("{prefix}│   "))
        }
    } else if is_last {
        (r"\-- ", format!("{prefix}    "))
    } else {
        ("|-- ", format!("{prefix}|   "))
    };

    let name = if node.name.is_empty() {
        "root"
    } else {
        &node.name
    };
    let is_dir = matches!(node.kind, HierarchyKind::Directory | HierarchyKind::Root);
    let marker = if is_dir { "/" } else { "" };
    writeln!(
        w,
        "{}{}{}{}",
        escape_html(prefix),
        connector,
        escape_html(name),
        marker
    )?;

    for (idx, child) in node.children.iter().enumerate() {
        let child_is_last = idx + 1 == node.children.len();
        render_html_tree_ascii(w, child, &child_prefix, child_is_last, unicode)?;
    }

    Ok(())
}
