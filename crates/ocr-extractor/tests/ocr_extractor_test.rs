//! # OCR Extractor and CDM Pipeline Integration Tests
//!
//! Validates conversion of OCR output to Canonical Context Document Model (`CdmDocument`),
//! section attributes, hierarchy construction, and export compatibility.

use web_repo_to_ai_cdm::{ExtractionStatus, SectionKind, SourceType};
use web_repo_to_ai_exporters::{
    export_html, export_json, export_jsonl, export_markdown, HtmlOptions, JsonExportOptions,
    JsonlOptions, MarkdownOptions,
};
use web_repo_to_ai_ocr_extractor::{parse_tesseract_tsv, OcrEngineInfo, OcrExtractor, OcrInput};

const SAMPLE_TSV: &str = "\
level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext
1\t1\t0\t0\t0\t0\t0\t0\t800\t600\t-1\t
2\t1\t1\t0\t0\t0\t50\t50\t700\t200\t-1\t
3\t1\t1\t1\t0\t0\t50\t50\t700\t200\t-1\t
4\t1\t1\t1\t1\t0\t50\t50\t400\t30\t-1\t
5\t1\t1\t1\t1\t1\t50\t50\t100\t30\t95.0\tIsolated
5\t1\t1\t1\t1\t2\t160\t50\t120\t30\t92.0\tTesseract
4\t1\t1\t1\t2\t0\t50\t90\t500\t30\t-1\t
5\t1\t1\t1\t2\t1\t50\t90\t150\t30\t88.0\tOCR
5\t1\t1\t1\t2\t2\t210\t90\t80\t30\t91.0\tAdapter";

#[test]
fn test_cdm_mapping_and_attributes() {
    let engine = OcrEngineInfo {
        name: "Tesseract OCR".to_string(),
        version: "5.3.3".to_string(),
        available: true,
        path: Some("/usr/bin/tesseract".to_string()),
        supported_languages: vec!["eng".to_string()],
    };

    let ocr_result = parse_tesseract_tsv(SAMPLE_TSV, 55, engine).expect("Parse succeeds");

    let extractor = OcrExtractor::with_tesseract();
    let input = OcrInput::from_bytes(b"dummy_bytes".to_vec())
        .with_filename("invoice_scan.png")
        .with_source_url("https://example.com/docs/invoice_scan.png");

    let cdm = extractor
        .convert_to_cdm(&ocr_result, &input)
        .expect("CDM conversion must succeed");

    // Source identification
    assert_eq!(cdm.source_type, SourceType::DocumentOcr);
    assert_eq!(
        cdm.source_identity.canonical_url,
        "https://example.com/docs/invoice_scan.png"
    );
    assert_eq!(cdm.source_identity.domain.as_deref(), Some("example.com"));

    // Document Metadata
    assert_eq!(cdm.metadata.title.as_deref(), Some("invoice_scan.png"));
    assert_eq!(
        cdm.metadata.extra.get("ocr_engine").map(String::as_str),
        Some("Tesseract OCR")
    );
    assert_eq!(
        cdm.metadata
            .extra
            .get("ocr_total_words")
            .map(String::as_str),
        Some("4")
    );
    assert_eq!(
        cdm.metadata.extra.get("ocr_page_count").map(String::as_str),
        Some("1")
    );

    // Section validations
    assert_eq!(cdm.sections.len(), 1);
    let sec = &cdm.sections[0];
    assert_eq!(sec.section_id, "page_1_block_1");
    assert_eq!(sec.kind, SectionKind::Prose);
    assert_eq!(sec.body, "Isolated Tesseract\nOCR Adapter");
    assert_eq!(
        sec.attributes.get("page_number").map(String::as_str),
        Some("1")
    );
    assert_eq!(
        sec.attributes.get("block_number").map(String::as_str),
        Some("1")
    );
    assert!(sec.attributes.contains_key("confidence"));
    assert!(sec.attributes.contains_key("left"));
    assert!(sec.attributes.contains_key("top"));

    // Hierarchy validation
    assert_eq!(cdm.hierarchy.name, "invoice_scan.png");
    assert_eq!(cdm.hierarchy.children.len(), 1); // Page 1
    let page_node = &cdm.hierarchy.children[0];
    assert_eq!(page_node.name, "Page 1");
    assert_eq!(page_node.children.len(), 1); // Block 1
    assert_eq!(page_node.children[0].name, "Block 1");

    // Completeness
    assert_eq!(cdm.extraction_status, ExtractionStatus::Completed);
}

#[test]
fn test_exporter_roundtrip() {
    let engine = OcrEngineInfo::default();
    let ocr_result = parse_tesseract_tsv(SAMPLE_TSV, 40, engine).expect("Parse succeeds");
    let extractor = OcrExtractor::with_tesseract();
    let input = OcrInput::from_bytes(b"data".to_vec()).with_filename("sample.png");

    let cdm = extractor
        .convert_to_cdm(&ocr_result, &input)
        .expect("CDM succeeds");

    // Markdown exporter
    let md = export_markdown(&cdm, &MarkdownOptions::default()).expect("Markdown export succeeds");
    assert!(md.contains("Isolated Tesseract"));
    assert!(md.contains("OCR Adapter"));

    // JSONL exporter
    let (jsonl, _) = export_jsonl(&cdm, &JsonlOptions::default()).expect("JSONL export succeeds");
    assert!(jsonl.contains("Isolated Tesseract"));

    // JSON exporter
    let json_str = export_json(&cdm, &JsonExportOptions::default()).expect("JSON export succeeds");
    assert!(json_str.contains("Isolated Tesseract"));

    // HTML exporter
    let (html, _) = export_html(&cdm, &HtmlOptions::default()).expect("HTML export succeeds");
    assert!(html.contains("Isolated Tesseract"));
}
