//! # Plain-Text Exporter
//!
//! Production-grade, deterministic Plain-Text (`.txt`) generator transforming
//! [`CdmDocument`] into clean, AI-ready text context packages without markdown noise.

use crate::error::{ExporterError, Result};
use crate::options::{ExportMetrics, HierarchyStyle, PlainTextOptions};
use crate::util::{format_bytes, CountingWriter};
use std::io::Write;
use web_repo_to_ai_cdm::hierarchy::{HierarchyKind, HierarchyNode};
use web_repo_to_ai_cdm::model::CdmDocument;
use web_repo_to_ai_cdm::section::{CdmSection, SectionKind};

/// High-fidelity Plain-Text export engine with fluent configuration.
#[derive(Debug, Clone, Default)]
pub struct PlainTextExporter {
    options: PlainTextOptions,
}

impl PlainTextExporter {
    /// Creates a new Plain-Text exporter with default options.
    #[must_use]
    pub fn new() -> Self {
        Self {
            options: PlainTextOptions::default(),
        }
    }

    /// Sets the configuration options for this exporter.
    #[must_use]
    pub fn with_options(mut self, options: PlainTextOptions) -> Self {
        self.options = options;
        self
    }

    /// Exports a [`CdmDocument`] to an in-memory Plain-Text string.
    pub fn export_to_string(&self, doc: &CdmDocument) -> Result<String> {
        export_plain_text(doc, &self.options)
    }

    /// Exports a [`CdmDocument`] streaming directly into a writer.
    pub fn export_to_writer<W: Write>(
        &self,
        doc: &CdmDocument,
        writer: &mut W,
    ) -> Result<ExportMetrics> {
        export_plain_text_to_writer(doc, &self.options, writer)
    }
}

/// Exports a [`CdmDocument`] to a Plain-Text string using the provided options.
pub fn export_plain_text(doc: &CdmDocument, options: &PlainTextOptions) -> Result<String> {
    let mut buffer = Vec::with_capacity(estimate_initial_capacity(doc));
    export_plain_text_to_writer(doc, options, &mut buffer)?;
    String::from_utf8(buffer).map_err(|e| ExporterError::Formatting(e.to_string()))
}

/// Exports a [`CdmDocument`] directly to any [`Write`] destination as Plain-Text.
pub fn export_plain_text_to_writer<W: Write>(
    doc: &CdmDocument,
    options: &PlainTextOptions,
    writer: &mut W,
) -> Result<ExportMetrics> {
    let mut counting = CountingWriter::new(writer);
    let mut metrics = ExportMetrics::default();

    // 1. Header Banner
    if options.include_header_banner {
        write_header_banner(doc, options, &mut counting)?;
    }

    // 2. Document Outline / Directory Structure
    if options.hierarchy_style != HierarchyStyle::None {
        write_hierarchy_tree(&doc.hierarchy, options, &mut counting)?;
    }

    // 3. Table of Contents
    if options.include_table_of_contents && !doc.sections.is_empty() {
        write_table_of_contents(doc, options, &mut counting)?;
    }

    // 4. Sections
    write_sections(doc, options, &mut counting, &mut metrics)?;

    // 5. Sources & Citations
    if options.include_citations && !doc.citations.is_empty() {
        write_citations(doc, options, &mut counting, &mut metrics)?;
    }

    // 6. Excluded Files & Content
    if options.include_exclusions && !doc.exclusions.is_empty() {
        write_exclusions(doc, options, &mut counting, &mut metrics)?;
    }

    // 7. Completeness Accounting Report
    if options.include_completeness_report {
        write_completeness_report(doc, options, &mut counting)?;
    }

    counting.flush()?;

    metrics.bytes_written = counting.bytes_written;
    metrics.lines_written = counting.lines_written;

    Ok(metrics)
}

fn estimate_initial_capacity(doc: &CdmDocument) -> usize {
    let body_bytes: usize = doc.sections.iter().map(|s| s.body.len()).sum();
    body_bytes.saturating_add(2048)
}

fn make_divider(ch: char, width: usize) -> String {
    std::iter::repeat(ch).take(width).collect()
}

fn write_header_banner<W: Write>(
    doc: &CdmDocument,
    options: &PlainTextOptions,
    writer: &mut W,
) -> Result<()> {
    let major_bar = make_divider('=', options.divider_width);
    let title = doc.metadata.title.as_deref().unwrap_or("Context Document");

    writeln!(writer, "{major_bar}")?;
    writeln!(writer, "DOCUMENT: {title}")?;
    writeln!(writer, "SOURCE:   {}", doc.canonical_url)?;
    writeln!(
        writer,
        "TYPE:     {} | RETRIEVED: {}",
        doc.source_type.as_str(),
        doc.retrieved_at.to_rfc3339()
    )?;

    if let Some(license) = &doc.metadata.license {
        writeln!(writer, "LICENSE:  {license}")?;
    }
    if let Some(lang) = &doc.metadata.primary_language {
        writeln!(writer, "LANGUAGE: {lang}")?;
    }
    if !doc.metadata.authors.is_empty() {
        writeln!(writer, "AUTHORS:  {}", doc.metadata.authors.join(", "))?;
    }
    if let Some(rev) = &doc.revision.git_commit {
        writeln!(writer, "COMMIT:   {rev}")?;
    }

    writeln!(
        writer,
        "METRICS:  {} sections | {} words | {} chars | ~{} estimated tokens",
        doc.sections.len(),
        doc.total_word_count(),
        doc.total_char_count(),
        doc.total_estimated_tokens()
    )?;
    writeln!(
        writer,
        "COMPLETENESS: {:.1}% ({} extracted / {} discovered items)",
        doc.completeness.extraction_ratio() * 100.0,
        doc.completeness.extracted_count,
        doc.completeness.discovered_count
    )?;
    writeln!(writer, "{major_bar}\n")?;

    if let Some(desc) = &doc.metadata.description {
        writeln!(writer, "{desc}\n")?;
    }

    Ok(())
}

fn write_hierarchy_tree<W: Write>(
    root: &HierarchyNode,
    options: &PlainTextOptions,
    writer: &mut W,
) -> Result<()> {
    if root.children.is_empty() && root.name.is_empty() {
        return Ok(());
    }

    let divider = make_divider(options.divider_char, options.divider_width);
    writeln!(writer, "DOCUMENT & DIRECTORY STRUCTURE")?;
    writeln!(writer, "{divider}")?;

    match options.hierarchy_style {
        HierarchyStyle::UnicodeTree | HierarchyStyle::AsciiTree => {
            render_tree_node(root, "", true, true, options.hierarchy_style, writer)?;
        }
        HierarchyStyle::BulletedList => {
            render_bulleted_node(root, 0, writer)?;
        }
        HierarchyStyle::None => {}
    }

    writeln!(writer, "{divider}\n")?;
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
            writeln!(writer, "{indent}- {name}/")?;
        }
        HierarchyKind::File => {
            let details = match (node.byte_size, node.line_count) {
                (Some(b), Some(l)) => format!(" ({b} B, {l} lines)"),
                (Some(b), None) => format!(" ({b} B)"),
                (None, Some(l)) => format!(" ({l} lines)"),
                (None, None) => String::new(),
            };
            writeln!(writer, "{indent}- {name}{details}")?;
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
    options: &PlainTextOptions,
    writer: &mut W,
) -> Result<()> {
    let divider = make_divider(options.divider_char, options.divider_width);
    writeln!(writer, "TABLE OF CONTENTS")?;
    writeln!(writer, "{divider}")?;

    let mut cumulative_tokens: usize = 0;

    for (idx, section) in doc.sections.iter().enumerate() {
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

        let num = idx + 1;
        let title = section_display_title(section);
        let prefix = match section.kind {
            SectionKind::CodeFile => "FILE: ",
            SectionKind::CodeSnippet => "SNIPPET: ",
            SectionKind::Table => "TABLE: ",
            SectionKind::Admonition => "NOTE: ",
            _ => "",
        };

        let mut meta_parts = Vec::new();
        if let Some(lang) = &section.language {
            meta_parts.push(lang.clone());
        }
        if let Some(lines) = section.line_range {
            meta_parts.push(format!("{} lines", lines.line_count()));
        }
        if let Some(bytes) = section.byte_range {
            meta_parts.push(format!("{} B", bytes.byte_length()));
        }
        if let Some(metrics) = &section.token_metrics {
            meta_parts.push(format!("~{} tokens", metrics.estimated_tokens));
        }

        let meta_str = if meta_parts.is_empty() {
            String::new()
        } else {
            format!(" ({})", meta_parts.join(", "))
        };

        writeln!(writer, "{num:>2}. {prefix}{title}{meta_str}")?;
    }

    writeln!(writer, "{divider}\n")?;
    Ok(())
}

fn write_sections<W: Write>(
    doc: &CdmDocument,
    options: &PlainTextOptions,
    writer: &mut W,
    metrics: &mut ExportMetrics,
) -> Result<()> {
    let divider = make_divider(options.divider_char, options.divider_width);

    for (idx, section) in doc.sections.iter().enumerate() {
        let estimated = section.token_metrics.as_ref().map_or_else(
            || section.body.chars().count().div_ceil(4),
            |m| m.estimated_tokens,
        );

        if let Some(budget) = options.max_token_budget {
            if metrics.estimated_tokens.saturating_add(estimated) > budget {
                metrics.truncated = true;
                writeln!(writer, "{divider}")?;
                writeln!(
                    writer,
                    "[WARNING: Token budget limit of {budget} tokens reached. Remaining sections omitted.]"
                )?;
                writeln!(writer, "{divider}\n")?;
                break;
            }
        }

        let num = idx + 1;
        let title = section_display_title(section);
        let kind_label = match section.kind {
            SectionKind::CodeFile => "FILE",
            SectionKind::CodeSnippet => "SNIPPET",
            SectionKind::Table => "TABLE",
            SectionKind::Admonition => "NOTE",
            SectionKind::Prose => "PROSE",
            SectionKind::Manifest => "MANIFEST",
            SectionKind::Navigation => "NAVIGATION",
            SectionKind::Frontmatter => "FRONTMATTER",
        };

        if options.include_section_headers {
            writeln!(writer, "{divider}")?;
            writeln!(writer, "SECTION {num}: {kind_label}: {title}")?;
            write_section_metadata(section, writer)?;
            writeln!(writer, "{divider}")?;
        }

        // Section body
        if options.include_line_numbers
            && matches!(
                section.kind,
                SectionKind::CodeFile | SectionKind::CodeSnippet
            )
        {
            write_numbered_lines(&section.body, section.line_range, writer)?;
        } else {
            write!(writer, "{}", section.body)?;
            if !section.body.ends_with('\n') {
                writeln!(writer)?;
            }
        }
        writeln!(writer)?;

        metrics.sections_exported += 1;
        metrics.estimated_tokens += estimated;
    }

    Ok(())
}

fn write_section_metadata<W: Write>(section: &CdmSection, writer: &mut W) -> Result<()> {
    let mut parts = Vec::new();

    if let Some(path) = &section.original_path {
        parts.push(format!("Path: {path}"));
    }
    if let Some(lang) = &section.language {
        parts.push(format!("Language: {lang}"));
    }
    if let Some(lines) = section.line_range {
        parts.push(format!("Lines: {}-{}", lines.start_line, lines.end_line));
    }
    if let Some(bytes) = section.byte_range {
        parts.push(format!("Bytes: {} B", bytes.byte_length()));
    }
    if let Some(metrics) = &section.token_metrics {
        parts.push(format!("Tokens: ~{}", metrics.estimated_tokens));
    }
    if section.content_hash.len() >= 12 {
        parts.push(format!("Hash: {}", &section.content_hash[..12]));
    }

    if !parts.is_empty() {
        writeln!(writer, "{}", parts.join(" | "))?;
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
    options: &PlainTextOptions,
    writer: &mut W,
    metrics: &mut ExportMetrics,
) -> Result<()> {
    let divider = make_divider('=', options.divider_width);
    writeln!(writer, "{divider}")?;
    writeln!(writer, "SOURCES & CITATIONS")?;
    writeln!(writer, "{divider}")?;

    for (idx, citation) in doc.citations.iter().enumerate() {
        let num = idx + 1;
        let title = citation.title.as_deref().unwrap_or(&citation.url);
        writeln!(writer, "{num}. {title}: {}", citation.url)?;
        if !citation.plaintext.is_empty()
            && citation.plaintext != format!("Source: {title} ({})", citation.url)
        {
            writeln!(writer, "   {}", citation.plaintext)?;
        }
        metrics.citations_exported += 1;
    }

    writeln!(writer)?;
    Ok(())
}

fn write_exclusions<W: Write>(
    doc: &CdmDocument,
    options: &PlainTextOptions,
    writer: &mut W,
    metrics: &mut ExportMetrics,
) -> Result<()> {
    let divider = make_divider('=', options.divider_width);
    writeln!(writer, "{divider}")?;
    writeln!(writer, "EXCLUDED FILES & CONTENT")?;
    writeln!(writer, "{divider}")?;

    for (idx, excl) in doc.exclusions.iter().enumerate() {
        let num = idx + 1;
        let size_str = excl.byte_size.map_or_else(|| "—".to_string(), format_bytes);
        let rule_str = excl.pattern.as_deref().unwrap_or("—");
        let details = excl.details.as_deref().unwrap_or("—");

        writeln!(writer, "{num}. Path: {}", excl.original_path)?;
        writeln!(
            writer,
            "   Reason: {} | Pattern: {} | Size: {}",
            excl.reason.as_str(),
            rule_str,
            size_str
        )?;
        writeln!(writer, "   Details: {details}")?;
        metrics.exclusions_exported += 1;
    }

    writeln!(writer)?;
    Ok(())
}

fn write_completeness_report<W: Write>(
    doc: &CdmDocument,
    options: &PlainTextOptions,
    writer: &mut W,
) -> Result<()> {
    let divider = make_divider('=', options.divider_width);
    writeln!(writer, "{divider}")?;
    writeln!(writer, "COMPLETENESS ACCOUNTING")?;
    writeln!(writer, "{divider}")?;
    writeln!(writer, "Status:       {}", doc.extraction_status.as_str())?;
    writeln!(
        writer,
        "Discovered:   {}",
        doc.completeness.discovered_count
    )?;
    writeln!(writer, "Extracted:    {}", doc.completeness.extracted_count)?;
    writeln!(writer, "Excluded:     {}", doc.completeness.excluded_count)?;
    writeln!(writer, "Failed:       {}", doc.completeness.failed_count)?;
    writeln!(
        writer,
        "Unsupported:  {}",
        doc.completeness.unsupported_count
    )?;
    writeln!(
        writer,
        "Completeness: {:.1}%\n",
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
