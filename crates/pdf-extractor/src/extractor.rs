//! # High-Level PDF Extractor
//!
//! Orchestrates PDF document parsing, layout extraction, and Canonical Context
//! Document Model (`CdmDocument`) conversion.

use crate::error::Result;
use crate::layout::{LayoutOptions, PdfLayoutEngine};
use crate::models::{OutlineItem, PdfDocument, TextBlockKind};
use crate::parser::PdfParser;
use std::collections::BTreeMap;
use web_repo_to_ai_cdm::{
    CdmDocument, CdmDocumentBuilder, CdmSectionBuilder, CompletenessReport, DocumentMetadata,
    ExtractionStatus, HierarchyKind, HierarchyNode, SectionKind, SourceIdentity, SourceType,
};

/// Configuration options for PDF extraction and CDM generation.
#[derive(Debug, Clone)]
pub struct PdfExtractionOptions {
    /// Source URL or filename identifying where the PDF originated.
    pub source_url: Option<String>,
    /// Whether to include page numbers in section titles and headers.
    pub include_page_numbers: bool,
    /// Minimum font size threshold for detecting heading sections.
    pub header_font_size_threshold: f64,
    /// Whether to detect and order multi-column text layouts.
    pub detect_columns: bool,
}

impl Default for PdfExtractionOptions {
    fn default() -> Self {
        Self {
            source_url: None,
            include_page_numbers: true,
            header_font_size_threshold: 13.0,
            detect_columns: true,
        }
    }
}

/// Primary orchestrator for converting PDF documents into structured intermediate representations.
#[derive(Debug, Default)]
pub struct PdfExtractor {
    options: PdfExtractionOptions,
}

impl PdfExtractor {
    /// Creates a new `PdfExtractor` with default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `PdfExtractor` with customized options.
    #[must_use]
    pub const fn with_options(options: PdfExtractionOptions) -> Self {
        Self { options }
    }

    /// Extracts low-level structured `PdfDocument` from raw bytes.
    pub fn extract(&self, bytes: &[u8]) -> Result<PdfDocument> {
        let layout_opts = LayoutOptions {
            header_font_size_threshold: self.options.header_font_size_threshold,
            detect_columns: self.options.detect_columns,
            ..Default::default()
        };
        let layout_engine = PdfLayoutEngine::new(layout_opts);
        let parser = PdfParser::with_layout_engine(layout_engine);
        parser.parse(bytes)
    }

    /// Extracts and maps the PDF document into a Canonical Context Document Model (`CdmDocument`).
    pub fn extract_to_cdm(&self, bytes: &[u8]) -> Result<CdmDocument> {
        let pdf_doc = self.extract(bytes)?;
        self.convert_to_cdm(&pdf_doc)
    }

    /// Extracts and maps the PDF document into a `CdmDocument`, overriding the source URL.
    pub fn extract_to_cdm_with_url(&self, bytes: &[u8], source_url: &str) -> Result<CdmDocument> {
        let mut opts = self.options.clone();
        opts.source_url = Some(source_url.to_string());
        let extractor = Self::with_options(opts);
        let pdf_doc = extractor.extract(bytes)?;
        extractor.convert_to_cdm(&pdf_doc)
    }

    /// Converts a parsed `PdfDocument` into a validated `CdmDocument`.
    pub fn convert_to_cdm(&self, pdf_doc: &PdfDocument) -> Result<CdmDocument> {
        let source_url = self
            .options
            .source_url
            .clone()
            .unwrap_or_else(|| "file://document.pdf".to_string());

        let mut source_identity = SourceIdentity::new(source_url.clone(), source_url.clone());
        if let Ok(parsed) = url::Url::parse(&source_url) {
            source_identity.domain = parsed.domain().map(ToString::to_string);
        }

        let mut custom_meta = BTreeMap::new();
        custom_meta.insert(
            "pdf_version".to_string(),
            pdf_doc.metadata.pdf_version.clone(),
        );
        custom_meta.insert("page_count".to_string(), pdf_doc.total_pages.to_string());
        custom_meta.insert(
            "total_characters".to_string(),
            pdf_doc.total_characters.to_string(),
        );
        custom_meta.insert(
            "total_estimated_tokens".to_string(),
            pdf_doc.total_estimated_tokens.to_string(),
        );

        let authors: Vec<String> = pdf_doc.metadata.author.clone().into_iter().collect();

        let metadata = DocumentMetadata {
            title: pdf_doc.metadata.title.clone().or_else(|| {
                // If title is missing in PDF /Info, use filename stem or fallback
                std::path::Path::new(&source_url)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(ToString::to_string)
            }),
            description: pdf_doc.metadata.subject.clone(),
            authors,
            license: None,
            primary_language: None,
            language_breakdown: BTreeMap::new(),
            published_at: pdf_doc.metadata.creation_date,
            modified_at: pdf_doc.metadata.mod_date,
            tags: pdf_doc.metadata.keywords.clone(),
            extra: custom_meta,
        };

        let mut builder =
            CdmDocumentBuilder::new(source_identity, SourceType::DocumentPdf, source_url.clone())
                .content_type("application/pdf")
                .metadata(metadata);

        // Convert each page to CdmSection(s)
        for page in &pdf_doc.pages {
            let page_sec_id = format!("page_{}", page.page_number);
            let page_title = if self.options.include_page_numbers {
                format!("Page {}", page.page_number)
            } else {
                String::new()
            };

            if page.tables.is_empty() {
                // Single prose section for the entire page
                let line_count = page.raw_text.lines().count().max(1) as u32;
                let byte_count = page.raw_text.len() as u64;

                let sec_builder = CdmSectionBuilder::new(
                    page_sec_id,
                    source_url.clone(),
                    SectionKind::Prose,
                    page.raw_text.clone(),
                )
                .title(page_title)
                .original_path(format!("page_{}.txt", page.page_number))
                .line_range(1, line_count)
                .byte_range(0, byte_count)
                .attribute("page_number", page.page_number.to_string())
                .attribute("width", page.dimensions.width.to_string())
                .attribute("height", page.dimensions.height.to_string())
                .attribute("rotation", page.dimensions.rotation.to_string())
                .attribute("block_count", page.blocks.len().to_string())
                .attribute("table_count", "0".to_string())
                .attribute("content_type", "text/prose".to_string());

                builder = builder.add_section(sec_builder.build());
            } else {
                // Page contains tables: emit prose section + table sections
                // 1. Prose blocks on the page (excluding table markdown)
                let prose_text: String = page
                    .blocks
                    .iter()
                    .filter(|b| b.kind != TextBlockKind::TableContent)
                    .map(|b| b.text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n\n");

                if !prose_text.trim().is_empty() {
                    let line_count = prose_text.lines().count().max(1) as u32;
                    let byte_count = prose_text.len() as u64;

                    let sec_builder = CdmSectionBuilder::new(
                        page_sec_id.clone(),
                        source_url.clone(),
                        SectionKind::Prose,
                        prose_text,
                    )
                    .title(page_title)
                    .original_path(format!("page_{}.txt", page.page_number))
                    .line_range(1, line_count)
                    .byte_range(0, byte_count)
                    .attribute("page_number", page.page_number.to_string())
                    .attribute("width", page.dimensions.width.to_string())
                    .attribute("height", page.dimensions.height.to_string())
                    .attribute("rotation", page.dimensions.rotation.to_string())
                    .attribute("block_count", page.blocks.len().to_string())
                    .attribute("table_count", page.tables.len().to_string())
                    .attribute("content_type", "text/prose".to_string());

                    builder = builder.add_section(sec_builder.build());
                }

                // 2. Each extracted table as a distinct SectionKind::Table section
                for (t_idx, table) in page.tables.iter().enumerate() {
                    let table_sec_id = format!("page_{}_table_{}", page.page_number, t_idx + 1);
                    let table_md = table.to_markdown();
                    let line_count = table_md.lines().count().max(1) as u32;
                    let byte_count = table_md.len() as u64;

                    let table_title = table.caption.clone().unwrap_or_else(|| {
                        format!("Page {} Table {}", page.page_number, t_idx + 1)
                    });

                    let sec_builder = CdmSectionBuilder::new(
                        table_sec_id.clone(),
                        source_url.clone(),
                        SectionKind::Table,
                        table_md,
                    )
                    .title(table_title)
                    .original_path(format!("page_{}_table_{}.md", page.page_number, t_idx + 1))
                    .line_range(1, line_count)
                    .byte_range(0, byte_count)
                    .attribute("page_number", page.page_number.to_string())
                    .attribute("table_id", table.table_id.clone())
                    .attribute("table_columns", table.column_count.to_string())
                    .attribute("table_rows", table.row_count.to_string())
                    .attribute("x0", format!("{:.2}", table.bbox.x0))
                    .attribute("y0", format!("{:.2}", table.bbox.y0))
                    .attribute("x1", format!("{:.2}", table.bbox.x1))
                    .attribute("y1", format!("{:.2}", table.bbox.y1))
                    .attribute("width", format!("{:.2}", table.bbox.width()))
                    .attribute("height", format!("{:.2}", table.bbox.height()))
                    .attribute("content_type", "table/markdown".to_string())
                    .attribute("is_table", "true".to_string());

                    builder = builder.add_section(sec_builder.build());
                }
            }
        }

        // Build document hierarchy from outlines or pages
        let mut root_node =
            HierarchyNode::root(pdf_doc.metadata.title.as_deref().unwrap_or("PDF Document"));

        if !pdf_doc.outline.is_empty() {
            for (idx, item) in pdf_doc.outline.iter().enumerate() {
                root_node.add_child(convert_outline_to_hierarchy(
                    item,
                    &format!("outline_{idx}"),
                ));
            }
        } else {
            for page in &pdf_doc.pages {
                let page_id = format!("page_{}", page.page_number);
                let mut page_node = HierarchyNode {
                    id: page_id.clone(),
                    name: format!("Page {}", page.page_number),
                    path: format!("page_{}", page.page_number),
                    kind: HierarchyKind::Section,
                    section_id: Some(page_id),
                    children: Vec::new(),
                    byte_size: Some(page.raw_text.len() as u64),
                    line_count: Some(page.raw_text.lines().count().max(1) as u32),
                };

                for (t_idx, table) in page.tables.iter().enumerate() {
                    let table_sec_id = format!("page_{}_table_{}", page.page_number, t_idx + 1);
                    let table_name = table
                        .caption
                        .clone()
                        .unwrap_or_else(|| format!("Table {}", t_idx + 1));
                    page_node.add_child(HierarchyNode {
                        id: table_sec_id.clone(),
                        name: table_name,
                        path: format!("page_{}/tables/{}", page.page_number, t_idx + 1),
                        kind: HierarchyKind::Section,
                        section_id: Some(table_sec_id),
                        children: Vec::new(),
                        byte_size: Some(table.to_markdown().len() as u64),
                        line_count: Some(table.row_count as u32 + 2),
                    });
                }

                root_node.add_child(page_node);
            }
        }

        builder = builder.hierarchy(root_node);

        // Set completeness report: 100% of discovered pages were extracted
        let completeness = CompletenessReport::new(
            pdf_doc.total_pages as u64,
            pdf_doc.total_pages as u64,
            pdf_doc.pages.len() as u64,
            0,
            0,
            0,
        );

        builder = builder
            .completeness(completeness)
            .extraction_status(ExtractionStatus::Completed);

        Ok(builder.build())
    }
}

/// Recursively converts an outline bookmark item into a CDM hierarchy node.
fn convert_outline_to_hierarchy(item: &OutlineItem, id_prefix: &str) -> HierarchyNode {
    let mut node = HierarchyNode {
        id: id_prefix.to_string(),
        name: item.title.clone(),
        path: item.title.clone(),
        kind: HierarchyKind::Heading,
        section_id: item.page_number.map(|p| format!("page_{p}")),
        children: Vec::new(),
        byte_size: None,
        line_count: None,
    };

    for (c_idx, child) in item.children.iter().enumerate() {
        node.add_child(convert_outline_to_hierarchy(
            child,
            &format!("{id_prefix}_{c_idx}"),
        ));
    }

    node
}
