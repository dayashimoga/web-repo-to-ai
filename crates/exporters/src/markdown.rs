//! # Markdown Exporter
//!
//! Production-grade, deterministic Markdown generator transforming [`CdmDocument`]
//! into structured, AI-ready Markdown artifacts.

use crate::error::{ExporterError, Result};
use crate::options::{CodeFenceStyle, ExportMetrics, HierarchyStyle, MarkdownOptions};
use std::io::Write;
use web_repo_to_ai_cdm::hierarchy::{HierarchyKind, HierarchyNode};
use web_repo_to_ai_cdm::model::CdmDocument;
use web_repo_to_ai_cdm::section::{CdmSection, SectionKind};

/// High-fidelity Markdown export engine with fluent configuration.
#[derive(Debug, Clone, Default)]
pub struct MarkdownExporter {
    options: MarkdownOptions,
}

impl MarkdownExporter {
    /// Creates a new Markdown exporter with default options.
    #[must_use]
    pub fn new() -> Self {
        Self {
            options: MarkdownOptions::default(),
        }
    }

    /// Sets the configuration options for this exporter.
    #[must_use]
    pub fn with_options(mut self, options: MarkdownOptions) -> Self {
        self.options = options;
        self
    }

    /// Exports a [`CdmDocument`] to an in-memory Markdown string.
    pub fn export_to_string(&self, doc: &CdmDocument) -> Result<String> {
        export_markdown(doc, &self.options)
    }

    /// Exports a [`CdmDocument`] streaming directly into a writer.
    pub fn export_to_writer<W: Write>(
        &self,
        doc: &CdmDocument,
        writer: &mut W,
    ) -> Result<ExportMetrics> {
        export_markdown_to_writer(doc, &self.options, writer)
    }
}

/// Exports a [`CdmDocument`] to a Markdown string using the provided options.
pub fn export_markdown(doc: &CdmDocument, options: &MarkdownOptions) -> Result<String> {
    let mut buffer = Vec::with_capacity(estimate_initial_capacity(doc));
    export_markdown_to_writer(doc, options, &mut buffer)?;
    String::from_utf8(buffer).map_err(|e| ExporterError::Formatting(e.to_string()))
}

/// Exports a [`CdmDocument`] directly to any [`Write`] destination.
pub fn export_markdown_to_writer<W: Write>(
    doc: &CdmDocument,
    options: &MarkdownOptions,
    writer: &mut W,
) -> Result<ExportMetrics> {
    let mut counting = CountingWriter::new(writer);
    let mut metrics = ExportMetrics::default();

    // 1. YAML Frontmatter
    if options.include_frontmatter {
        write_frontmatter(doc, &mut counting)?;
    }

    // 2. Metadata Overview Block
    if options.include_metadata_block {
        write_metadata_block(doc, &mut counting)?;
    }

    // 3. Document Outline / Directory Structure
    if options.hierarchy_style != HierarchyStyle::None {
        write_hierarchy_tree(&doc.hierarchy, options.hierarchy_style, &mut counting)?;
    }

    // 4. Table of Contents
    if options.include_table_of_contents && !doc.sections.is_empty() {
        write_table_of_contents(doc, options, &mut counting)?;
    }

    // 5. Sections
    write_sections(doc, options, &mut counting, &mut metrics)?;

    // 6. Citations & Sources
    if options.include_citations && !doc.citations.is_empty() {
        write_citations(doc, &mut counting, &mut metrics)?;
    }

    // 7. Excluded Files & Content
    if options.include_exclusions && !doc.exclusions.is_empty() {
        write_exclusions(doc, &mut counting, &mut metrics)?;
    }

    // 8. Completeness Accounting Report
    if options.include_completeness_report {
        write_completeness_report(doc, &mut counting)?;
    }

    counting.flush()?;

    metrics.bytes_written = counting.bytes_written;
    metrics.lines_written = counting.lines_written;

    Ok(metrics)
}

fn estimate_initial_capacity(doc: &CdmDocument) -> usize {
    let body_bytes: usize = doc.sections.iter().map(|s| s.body.len()).sum();
    // Frontmatter, headers, citations, and tree typically add ~15-25% overhead
    body_bytes.saturating_add(2048)
}

fn write_frontmatter<W: Write>(doc: &CdmDocument, writer: &mut W) -> Result<()> {
    writeln!(writer, "---")?;
    if let Some(title) = &doc.metadata.title {
        writeln!(writer, "title: {}", escape_yaml_str(title))?;
    }
    writeln!(
        writer,
        "canonical_url: {}",
        escape_yaml_str(&doc.canonical_url)
    )?;
    writeln!(writer, "source_type: {}", doc.source_type.as_str())?;
    writeln!(
        writer,
        "retrieved_at: \"{}\"",
        doc.retrieved_at.to_rfc3339()
    )?;
    writeln!(writer, "schema_version: \"{}\"", doc.schema_version)?;
    writeln!(writer, "document_id: \"{}\"", doc.document_id)?;

    if let Some(license) = &doc.metadata.license {
        writeln!(writer, "license: {}", escape_yaml_str(license))?;
    }
    if let Some(lang) = &doc.metadata.primary_language {
        writeln!(writer, "primary_language: {}", escape_yaml_str(lang))?;
    }
    if let Some(rev) = &doc.revision.git_commit {
        writeln!(writer, "git_commit: \"{rev}\"")?;
    }
    if let Some(branch) = &doc.revision.git_branch {
        writeln!(writer, "git_branch: {}", escape_yaml_str(branch))?;
    }
    if !doc.metadata.tags.is_empty() {
        writeln!(writer, "tags:")?;
        for tag in &doc.metadata.tags {
            writeln!(writer, "  - {}", escape_yaml_str(tag))?;
        }
    }

    writeln!(writer, "total_sections: {}", doc.sections.len())?;
    writeln!(writer, "total_words: {}", doc.total_word_count())?;
    writeln!(writer, "total_chars: {}", doc.total_char_count())?;
    writeln!(writer, "estimated_tokens: {}", doc.total_estimated_tokens())?;
    writeln!(writer, "---\n")?;
    Ok(())
}

fn write_metadata_block<W: Write>(doc: &CdmDocument, writer: &mut W) -> Result<()> {
    let title = doc.metadata.title.as_deref().unwrap_or("Context Document");
    writeln!(writer, "# {title}\n")?;

    writeln!(writer, "> **Source:** <{}>", doc.canonical_url)?;
    write!(
        writer,
        "> **Type:** `{}` | **Retrieved:** `{}`",
        doc.source_type.as_str(),
        doc.retrieved_at.to_rfc3339()
    )?;
    if let Some(license) = &doc.metadata.license {
        write!(writer, " | **License:** `{license}`")?;
    }
    if let Some(lang) = &doc.metadata.primary_language {
        write!(writer, " | **Language:** `{lang}`")?;
    }
    writeln!(writer)?;

    if !doc.metadata.authors.is_empty() {
        writeln!(writer, "> **Authors:** {}", doc.metadata.authors.join(", "))?;
    }

    writeln!(
        writer,
        "> **Completeness:** {:.1}% ({} extracted / {} discovered items)",
        doc.completeness.extraction_ratio() * 100.0,
        doc.completeness.extracted_count,
        doc.completeness.discovered_count
    )?;

    if let Some(desc) = &doc.metadata.description {
        writeln!(writer, "\n{desc}")?;
    }

    writeln!(writer, "\n---\n")?;
    Ok(())
}

fn write_hierarchy_tree<W: Write>(
    root: &HierarchyNode,
    style: HierarchyStyle,
    writer: &mut W,
) -> Result<()> {
    if root.children.is_empty() && root.name.is_empty() {
        return Ok(());
    }

    writeln!(writer, "## Document & Directory Structure\n")?;

    match style {
        HierarchyStyle::UnicodeTree | HierarchyStyle::AsciiTree => {
            writeln!(writer, "```")?;
            render_tree_node(root, "", true, true, style, writer)?;
            writeln!(writer, "```\n")?;
        }
        HierarchyStyle::BulletedList => {
            render_bulleted_node(root, 0, writer)?;
            writeln!(writer)?;
        }
        HierarchyStyle::None => {}
    }

    Ok(())
}

fn render_tree_node<W: Write>(
    node: &HierarchyNode,
    prefix: &str,
    is_last: bool,
    is_root: bool,
    style: HierarchyStyle,
    writer: &mut W,
) -> std::io::Result<()> {
    let (branch, next_prefix) = if is_root {
        ("", prefix.to_string())
    } else {
        match style {
            HierarchyStyle::UnicodeTree => {
                let b = if is_last { "└── " } else { "├── " };
                let p = if is_last {
                    format!("{prefix}    ")
                } else {
                    format!("{prefix}│   ")
                };
                (b, p)
            }
            HierarchyStyle::AsciiTree => {
                let b = if is_last { "\\-- " } else { "|-- " };
                let p = if is_last {
                    format!("{prefix}    ")
                } else {
                    format!("{prefix}|   ")
                };
                (b, p)
            }
            _ => ("", prefix.to_string()),
        }
    };

    let name = if node.name.is_empty() {
        if node.path.is_empty() {
            "root"
        } else {
            &node.path
        }
    } else {
        &node.name
    };

    let details = match (node.byte_size, node.line_count) {
        (Some(b), Some(l)) => format!(" ({} B, {l} lines)", b),
        (Some(b), None) => format!(" ({} B)", b),
        (None, Some(l)) => format!(" ({l} lines)"),
        (None, None) => String::new(),
    };

    let suffix = match node.kind {
        HierarchyKind::Directory => "/",
        _ => "",
    };

    writeln!(writer, "{prefix}{branch}{name}{suffix}{details}")?;

    let count = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        render_tree_node(child, &next_prefix, i + 1 == count, false, style, writer)?;
    }

    Ok(())
}

fn render_bulleted_node<W: Write>(
    node: &HierarchyNode,
    depth: usize,
    writer: &mut W,
) -> std::io::Result<()> {
    let indent = "  ".repeat(depth);
    let name = if node.name.is_empty() {
        "root"
    } else {
        &node.name
    };

    match node.kind {
        HierarchyKind::Directory | HierarchyKind::Root => {
            writeln!(writer, "{indent}- **{name}/**")?;
        }
        HierarchyKind::File => {
            let details = match (node.byte_size, node.line_count) {
                (Some(b), Some(l)) => format!(" ({b} B, {l} lines)"),
                (Some(b), None) => format!(" ({b} B)"),
                (None, Some(l)) => format!(" ({l} lines)"),
                (None, None) => String::new(),
            };
            writeln!(writer, "{indent}- `{name}`{details}")?;
        }
        HierarchyKind::Section | HierarchyKind::Heading => {
            writeln!(writer, "{indent}- {name}")?;
        }
    }

    for child in &node.children {
        render_bulleted_node(child, depth + 1, writer)?;
    }

    Ok(())
}

fn write_table_of_contents<W: Write>(
    doc: &CdmDocument,
    options: &MarkdownOptions,
    writer: &mut W,
) -> Result<()> {
    writeln!(writer, "## Table of Contents\n")?;

    let mut cumulative_tokens: usize = 0;

    for section in &doc.sections {
        let estimated = section.token_metrics.as_ref().map_or_else(
            || section.body.chars().count().div_ceil(4),
            |m| m.estimated_tokens,
        );

        if let Some(budget) = options.max_token_budget {
            if cumulative_tokens.saturating_add(estimated) > budget {
                break;
            }
        }
        cumulative_tokens += estimated;

        let title = section_display_title(section);
        let prefix = match section.kind {
            SectionKind::CodeFile => "File: ",
            SectionKind::CodeSnippet => "Snippet: ",
            SectionKind::Table => "Table: ",
            SectionKind::Admonition => "Note: ",
            _ => "",
        };
        let full_heading = format!("{prefix}{title}");
        let slug = slugify(&full_heading);
        let mut meta_parts = Vec::new();

        if let Some(lang) = &section.language {
            meta_parts.push(lang.clone());
        }
        if let Some(lines) = section.line_range {
            meta_parts.push(format!("{} lines", lines.line_count()));
        }
        if let Some(metrics) = &section.token_metrics {
            meta_parts.push(format!("~{} tokens", metrics.estimated_tokens));
        }

        let meta_str = if meta_parts.is_empty() {
            String::new()
        } else {
            format!(" *({})*", meta_parts.join(", "))
        };

        writeln!(writer, "- [{title}](#{slug}){meta_str}")?;
    }

    writeln!(writer, "\n---\n")?;
    Ok(())
}

fn write_sections<W: Write>(
    doc: &CdmDocument,
    options: &MarkdownOptions,
    writer: &mut W,
    metrics: &mut ExportMetrics,
) -> Result<()> {
    let heading_hashes = "#".repeat(options.section_heading_level as usize);

    for section in &doc.sections {
        let estimated = section.token_metrics.as_ref().map_or_else(
            || section.body.chars().count().div_ceil(4),
            |m| m.estimated_tokens,
        );

        // Budget check
        if let Some(budget) = options.max_token_budget {
            if metrics.estimated_tokens.saturating_add(estimated) > budget {
                metrics.truncated = true;
                writeln!(
                    writer,
                    "> [!WARNING]\n> **Token Budget Limit Exceeded:** Further sections omitted to respect the maximum budget of {budget} tokens.\n"
                )?;
                break;
            }
        }

        let title = section_display_title(section);
        let prefix = match section.kind {
            SectionKind::CodeFile => "File: ",
            SectionKind::CodeSnippet => "Snippet: ",
            SectionKind::Table => "Table: ",
            SectionKind::Admonition => "Note: ",
            _ => "",
        };

        writeln!(writer, "{heading_hashes} {prefix}{title}\n")?;

        // Section metadata callout
        if options.include_section_metadata {
            write_section_metadata(section, writer)?;
        }

        // Section body
        match section.kind {
            SectionKind::CodeFile | SectionKind::CodeSnippet => {
                let fence = if options.auto_escalate_fences {
                    compute_fence(&section.body, options.code_fence_style)
                } else {
                    match options.code_fence_style {
                        CodeFenceStyle::Backticks => "```".to_string(),
                        CodeFenceStyle::Tildes => "~~~".to_string(),
                    }
                };
                let lang = section.language.as_deref().unwrap_or("");
                writeln!(writer, "{fence}{lang}")?;
                if options.include_line_numbers {
                    write_numbered_lines(&section.body, section.line_range, writer)?;
                } else {
                    write!(writer, "{}", section.body)?;
                    if !section.body.ends_with('\n') {
                        writeln!(writer)?;
                    }
                }
                writeln!(writer, "{fence}\n")?;
            }
            SectionKind::Admonition => {
                let alert_type = section
                    .attributes
                    .get("admonition_type")
                    .map(|s| s.to_uppercase())
                    .unwrap_or_else(|| "NOTE".to_string());
                writeln!(writer, "> [!{alert_type}]")?;
                for line in section.body.lines() {
                    writeln!(writer, "> {line}")?;
                }
                writeln!(writer)?;
            }
            SectionKind::Frontmatter => {
                writeln!(writer, "```yaml\n{}\n```\n", section.body.trim())?;
            }
            _ => {
                // Prose, Table, Manifest, Navigation
                write!(writer, "{}", section.body)?;
                if !section.body.ends_with('\n') {
                    writeln!(writer)?;
                }
                writeln!(writer)?;
            }
        }

        metrics.sections_exported += 1;
        metrics.estimated_tokens += estimated;
    }

    Ok(())
}

fn write_section_metadata<W: Write>(section: &CdmSection, writer: &mut W) -> Result<()> {
    let mut parts = Vec::new();

    if let Some(path) = &section.original_path {
        parts.push(format!("**Path:** `{path}`"));
    }
    if let Some(lang) = &section.language {
        parts.push(format!("**Language:** `{lang}`"));
    }
    if let Some(lines) = section.line_range {
        parts.push(format!(
            "**Lines:** {}-{}",
            lines.start_line, lines.end_line
        ));
    }
    if let Some(bytes) = section.byte_range {
        parts.push(format!("**Bytes:** {} B", bytes.byte_length()));
    }
    if let Some(metrics) = &section.token_metrics {
        parts.push(format!("**Tokens:** ~{}", metrics.estimated_tokens));
    }
    if section.content_hash.len() >= 12 {
        parts.push(format!("**Hash:** `{}`", &section.content_hash[..12]));
    }

    if !parts.is_empty() {
        writeln!(writer, "> {}\n", parts.join(" | "))?;
    }

    Ok(())
}

fn write_numbered_lines<W: Write>(
    body: &str,
    range: Option<web_repo_to_ai_cdm::section::LineRange>,
    writer: &mut W,
) -> std::io::Result<()> {
    let start_line = range.map_or(1, |r| r.start_line);
    let total_lines = body.lines().count();
    let width = format!("{}", start_line + total_lines as u32).len();

    for (idx, line) in body.lines().enumerate() {
        let current_line = start_line + idx as u32;
        writeln!(writer, "{current_line:>width$} | {line}")?;
    }
    Ok(())
}

fn write_citations<W: Write>(
    doc: &CdmDocument,
    writer: &mut W,
    metrics: &mut ExportMetrics,
) -> Result<()> {
    writeln!(writer, "## Sources & Citations\n")?;

    for (idx, citation) in doc.citations.iter().enumerate() {
        let num = idx + 1;
        let title = citation.title.as_deref().unwrap_or(&citation.url);
        writeln!(writer, "{num}. [{title}]({})", citation.url)?;
        if !citation.concise.is_empty()
            && citation.concise != format!("[{title}]({})", citation.url)
        {
            writeln!(writer, "   - {}", citation.concise)?;
        }
        metrics.citations_exported += 1;
    }

    writeln!(writer)?;
    Ok(())
}

fn write_exclusions<W: Write>(
    doc: &CdmDocument,
    writer: &mut W,
    metrics: &mut ExportMetrics,
) -> Result<()> {
    writeln!(writer, "## Excluded Files & Content\n")?;
    writeln!(
        writer,
        "The following files or content items were discovered but excluded from this document:\n"
    )?;

    writeln!(
        writer,
        "| Original Path | Reason | Rule / Pattern | Size | Details |"
    )?;
    writeln!(writer, "| :--- | :--- | :--- | :---: | :--- |")?;

    for excl in &doc.exclusions {
        let size_str = excl.byte_size.map_or_else(|| "—".to_string(), format_bytes);
        let rule_str = excl.pattern.as_deref().unwrap_or("—");
        let details = excl.details.as_deref().unwrap_or("—");

        writeln!(
            writer,
            "| `{}` | {} | `{}` | {} | {} |",
            excl.original_path,
            excl.reason.as_str(),
            rule_str,
            size_str,
            details
        )?;
        metrics.exclusions_exported += 1;
    }

    writeln!(writer)?;
    Ok(())
}

fn write_completeness_report<W: Write>(doc: &CdmDocument, writer: &mut W) -> Result<()> {
    writeln!(writer, "## Completeness Accounting\n")?;
    writeln!(writer, "- **Status:** `{}`", doc.extraction_status.as_str())?;
    writeln!(
        writer,
        "- **Discovered Items:** {}",
        doc.completeness.discovered_count
    )?;
    writeln!(
        writer,
        "- **Extracted Items:** {}",
        doc.completeness.extracted_count
    )?;
    writeln!(
        writer,
        "- **Excluded Items:** {}",
        doc.completeness.excluded_count
    )?;
    writeln!(
        writer,
        "- **Failed Items:** {}",
        doc.completeness.failed_count
    )?;
    writeln!(
        writer,
        "- **Completeness Percentage:** {:.1}%\n",
        doc.completeness.extraction_ratio() * 100.0
    )?;
    Ok(())
}

fn section_display_title(section: &CdmSection) -> String {
    if let Some(title) = &section.title {
        title.clone()
    } else if let Some(path) = &section.original_path {
        path.clone()
    } else {
        section.section_id.clone()
    }
}

fn slugify(text: &str) -> String {
    let mut slug = String::with_capacity(text.len());
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else if (ch == ' ' || ch == '-') && !slug.ends_with('-') && !slug.is_empty() {
            slug.push('-');
        }
    }
    slug.trim_end_matches('-').to_string()
}

fn compute_fence(body: &str, style: CodeFenceStyle) -> String {
    let fence_char = match style {
        CodeFenceStyle::Backticks => '`',
        CodeFenceStyle::Tildes => '~',
    };

    let mut max_consecutive = 0;
    let mut current_consecutive = 0;

    for ch in body.chars() {
        if ch == fence_char {
            current_consecutive += 1;
            if current_consecutive > max_consecutive {
                max_consecutive = current_consecutive;
            }
        } else {
            current_consecutive = 0;
        }
    }

    let count = if max_consecutive >= 3 {
        max_consecutive + 1
    } else {
        3
    };

    std::iter::repeat(fence_char).take(count).collect()
}

fn escape_yaml_str(s: &str) -> String {
    if s.is_empty() {
        return "\"\"".to_string();
    }

    let needs_quotes = s.contains(':')
        || s.contains('"')
        || s.contains('\'')
        || s.contains('\n')
        || s.contains('#')
        || s.contains('[')
        || s.contains(']')
        || s.contains('{')
        || s.contains('}')
        || s.starts_with(' ')
        || s.ends_with(' ');

    if needs_quotes {
        let mut out = String::with_capacity(s.len() + 4);
        out.push('"');
        for ch in s.chars() {
            match ch {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                other => out.push(other),
            }
        }
        out.push('"');
        out
    } else {
        s.to_string()
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// A wrapper around any [`Write`] implementation tracking bytes and lines written.
struct CountingWriter<W> {
    inner: W,
    bytes_written: usize,
    lines_written: usize,
}

impl<W: Write> CountingWriter<W> {
    fn new(inner: W) -> Self {
        Self {
            inner,
            bytes_written: 0,
            lines_written: 0,
        }
    }
}

impl<W: Write> Write for CountingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.inner.write(buf)?;
        self.bytes_written += n;
        self.lines_written += buf[..n].iter().filter(|&&b| b == b'\n').count();
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}
