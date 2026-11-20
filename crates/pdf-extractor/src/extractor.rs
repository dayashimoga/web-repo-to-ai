//! # High-Level PDF Extractor
//!
//! Orchestrates PDF document parsing, layout extraction, and Canonical Context
//! Document Model (`CdmDocument`) conversion.

use crate::error::Result;
use crate::layout::{LayoutOptions, PdfLayoutEngine};
use crate::models::PdfDocument;
use crate::parser::PdfParser;
use std::collections::BTreeMap;
use web_repo_to_ai_cdm::{
    CdmDocument, CdmDocumentBuilder, CdmSectionBuilder, CompletenessReport, DocumentMetadata,
    ExtractionStatus, SectionKind, SourceIdentity, SourceType,
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

        // Convert each page to a CdmSection
        for page in &pdf_doc.pages {
            let section_id = format!("page_{}", page.page_number);
            let title = if self.options.include_page_numbers {
                format!("Page {}", page.page_number)
            } else {
                String::new()
            };

            let line_count = page.raw_text.lines().count().max(1) as u32;
            let byte_count = page.raw_text.len() as u64;

            let sec_builder = CdmSectionBuilder::new(
                section_id,
                source_url.clone(),
                SectionKind::Prose,
                page.raw_text.clone(),
            )
            .title(title)
            .original_path(format!("page_{}.txt", page.page_number))
            .line_range(1, line_count)
            .byte_range(0, byte_count)
            .attribute("page_number", page.page_number.to_string())
            .attribute("width", page.dimensions.width.to_string())
            .attribute("height", page.dimensions.height.to_string())
            .attribute("rotation", page.dimensions.rotation.to_string())
            .attribute("block_count", page.blocks.len().to_string());

            builder = builder.add_section(sec_builder.build());
        }

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
