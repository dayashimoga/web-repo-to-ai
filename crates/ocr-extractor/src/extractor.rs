//! # High-Level OCR Extractor and CDM Converter
//!
//! Orchestrates OCR image extraction and transforms extracted text, bounding boxes,
//! and confidence scores into the Canonical Context Document Model (`CdmDocument`).

use crate::adapter::OcrAdapter;
use crate::error::Result;
use crate::models::{OcrInput, OcrOptions, OcrResult};
use crate::tesseract::TesseractAdapter;
use std::collections::BTreeMap;
use std::sync::Arc;
use web_repo_to_ai_cdm::{
    CdmDocument, CdmDocumentBuilder, CdmSectionBuilder, CompletenessReport, DocumentMetadata,
    ExtractionStatus, HierarchyKind, HierarchyNode, SectionKind, SourceIdentity, SourceType,
};

/// High-level OCR extraction orchestrator bridging adapters and the CDM pipeline.
#[derive(Clone)]
pub struct OcrExtractor {
    adapter: Arc<dyn OcrAdapter>,
}

impl Default for OcrExtractor {
    fn default() -> Self {
        Self::with_tesseract()
    }
}

impl OcrExtractor {
    /// Creates a new `OcrExtractor` backed by the default `TesseractAdapter`.
    #[must_use]
    pub fn with_tesseract() -> Self {
        Self {
            adapter: Arc::new(TesseractAdapter::new()),
        }
    }

    /// Creates an extractor with a custom OCR adapter implementation.
    #[must_use]
    pub fn new(adapter: Arc<dyn OcrAdapter>) -> Self {
        Self { adapter }
    }

    /// Returns a reference to the active OCR adapter.
    #[must_use]
    pub fn adapter(&self) -> &dyn OcrAdapter {
        self.adapter.as_ref()
    }

    /// Executes OCR text extraction, producing structured domain models.
    pub async fn extract_to_result(
        &self,
        input: &OcrInput,
        options: &OcrOptions,
    ) -> Result<OcrResult> {
        self.adapter.extract_text(input, options).await
    }

    /// Executes OCR extraction and converts the result into a validated `CdmDocument`.
    pub async fn extract_to_cdm(
        &self,
        input: &OcrInput,
        options: &OcrOptions,
    ) -> Result<CdmDocument> {
        let result = self.extract_to_result(input, options).await?;
        self.convert_to_cdm(&result, input)
    }

    /// Converts a pre-parsed or extracted `OcrResult` into a `CdmDocument`.
    pub fn convert_to_cdm(&self, ocr_result: &OcrResult, input: &OcrInput) -> Result<CdmDocument> {
        let source_url = input
            .source_url
            .clone()
            .or_else(|| input.filename.as_ref().map(|f| format!("file://{f}")))
            .unwrap_or_else(|| "file://ocr_extracted_image.png".to_string());

        let mut source_identity = SourceIdentity::new(source_url.clone(), source_url.clone());
        if let Ok(parsed) = url::Url::parse(&source_url) {
            source_identity.domain = parsed.domain().map(ToString::to_string);
        }

        let mut custom_meta = BTreeMap::new();
        custom_meta.insert(
            "ocr_engine".to_string(),
            ocr_result.engine_info.name.clone(),
        );
        custom_meta.insert(
            "ocr_version".to_string(),
            ocr_result.engine_info.version.clone(),
        );
        custom_meta.insert(
            "ocr_mean_confidence".to_string(),
            format!("{:.4}", ocr_result.mean_confidence),
        );
        custom_meta.insert(
            "ocr_total_words".to_string(),
            ocr_result.total_words.to_string(),
        );
        custom_meta.insert(
            "ocr_page_count".to_string(),
            ocr_result.pages.len().to_string(),
        );
        custom_meta.insert(
            "ocr_duration_ms".to_string(),
            ocr_result.execution_duration_ms.to_string(),
        );

        let title = input.filename.clone().or_else(|| {
            std::path::Path::new(&source_url)
                .file_name()
                .and_then(|s| s.to_str())
                .map(ToString::to_string)
        });

        let metadata = DocumentMetadata {
            title,
            description: Some(format!(
                "OCR extracted text (confidence: {:.1}%)",
                ocr_result.mean_confidence * 100.0
            )),
            authors: Vec::new(),
            license: None,
            primary_language: None,
            language_breakdown: BTreeMap::new(),
            published_at: None,
            modified_at: None,
            tags: vec!["ocr".to_string(), "image".to_string()],
            extra: custom_meta,
        };

        let content_type = input
            .mime_type
            .clone()
            .unwrap_or_else(|| "image/png".to_string());

        let mut builder =
            CdmDocumentBuilder::new(source_identity, SourceType::DocumentOcr, source_url.clone())
                .content_type(content_type)
                .metadata(metadata);

        let mut root_node = HierarchyNode::root(
            input
                .filename
                .as_deref()
                .unwrap_or("OCR Extracted Document"),
        );

        let page_count = ocr_result.pages.len().max(1) as u64;

        for page in &ocr_result.pages {
            let page_id = format!("page_{}", page.page_num);
            let mut page_node = HierarchyNode {
                id: page_id.clone(),
                name: format!("Page {}", page.page_num),
                path: format!("page_{}", page.page_num),
                kind: HierarchyKind::Section,
                section_id: Some(page_id.clone()),
                children: Vec::new(),
                byte_size: Some(page.text.len() as u64),
                line_count: Some(page.text.lines().count().max(1) as u32),
            };

            for block in &page.blocks {
                let sec_id = format!("page_{}_block_{}", page.page_num, block.block_num);
                let block_line_count = block.text.lines().count().max(1) as u32;
                let block_byte_size = block.text.len() as u64;

                let sec_builder = CdmSectionBuilder::new(
                    sec_id.clone(),
                    source_url.clone(),
                    SectionKind::Prose,
                    block.text.clone(),
                )
                .title(format!("Page {} Block {}", page.page_num, block.block_num))
                .original_path(format!(
                    "page_{}_block_{}.txt",
                    page.page_num, block.block_num
                ))
                .line_range(1, block_line_count)
                .byte_range(0, block_byte_size)
                .attribute("page_number", page.page_num.to_string())
                .attribute("block_number", block.block_num.to_string())
                .attribute("confidence", format!("{:.4}", block.confidence))
                .attribute("left", format!("{:.1}", block.bbox.left))
                .attribute("top", format!("{:.1}", block.bbox.top))
                .attribute("width", format!("{:.1}", block.bbox.width))
                .attribute("height", format!("{:.1}", block.bbox.height));

                builder = builder.add_section(sec_builder.build());

                page_node.add_child(HierarchyNode {
                    id: sec_id.clone(),
                    name: format!("Block {}", block.block_num),
                    path: format!("page_{}/blocks/{}", page.page_num, block.block_num),
                    kind: HierarchyKind::Section,
                    section_id: Some(sec_id),
                    children: Vec::new(),
                    byte_size: Some(block_byte_size),
                    line_count: Some(block_line_count),
                });
            }

            root_node.add_child(page_node);
        }

        // If no blocks were recognized (empty or image without detected text), add single empty section
        if ocr_result.pages.is_empty() || ocr_result.total_words == 0 {
            let empty_sec = CdmSectionBuilder::new(
                "ocr_empty".to_string(),
                source_url,
                SectionKind::Prose,
                String::new(),
            )
            .title("Empty OCR Result")
            .line_range(1, 1)
            .byte_range(0, 0)
            .attribute("confidence", "0.0")
            .attribute("word_count", "0");

            builder = builder.add_section(empty_sec.build());
        }

        builder = builder.hierarchy(root_node);

        let completeness = CompletenessReport::new(page_count, page_count, page_count, 0, 0, 0);

        let extraction_status = if ocr_result.total_words == 0 || ocr_result.mean_confidence < 0.35
        {
            ExtractionStatus::Partial
        } else {
            ExtractionStatus::Completed
        };

        builder = builder
            .completeness(completeness)
            .extraction_status(extraction_status);

        Ok(builder.build())
    }
}
