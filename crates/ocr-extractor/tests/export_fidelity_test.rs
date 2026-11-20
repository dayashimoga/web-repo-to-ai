//! # OCR Downstream Multi-Format Export Fidelity & Determinism Benchmarks
//!
//! Validates downstream export fidelity, deterministic hashing, and syntax integrity:
//! - Multi-format serialization roundtrip (Markdown, Plaintext, HTML5, JSON, JSONL)
//! - Syntax validation across exported formats (GFM pipes, HTML5 tags, JSON schema)
//! - Bit-for-bit extraction determinism and BLAKE3 hash stability across runs
//! - High-throughput performance benchmark on synthetic OCR fixtures

use std::sync::Arc;
use std::time::Instant;

use web_repo_to_ai_exporters::html::export_html;
use web_repo_to_ai_exporters::integrity::{
    verify_html_syntax, verify_json_syntax, verify_jsonl_syntax, verify_markdown_syntax,
};
use web_repo_to_ai_exporters::json::export_json;
use web_repo_to_ai_exporters::jsonl::export_jsonl;
use web_repo_to_ai_exporters::markdown::export_markdown;
use web_repo_to_ai_exporters::options::{
    HtmlOptions, JsonExportOptions, JsonlOptions, MarkdownOptions, PlainTextOptions,
};
use web_repo_to_ai_exporters::plain_text::export_plain_text;
use web_repo_to_ai_ocr_extractor::{
    build_clean_document_fixture, build_dark_mode_fixture, build_multicolumn_fixture,
    MockOcrAdapter, OcrExtractor, OcrInput, OcrOptions, PreprocessingOptions,
};

#[tokio::test]
async fn test_ocr_markdown_export_fidelity() {
    let fixture = build_clean_document_fixture();
    let adapter = Arc::new(MockOcrAdapter::from_fixture(&fixture));
    let extractor = OcrExtractor::new(adapter);

    let input = OcrInput::from_bytes(fixture.image_bytes.clone())
        .with_filename("invoice.png")
        .with_source_url("file:///test/invoice.png");
    let opts =
        OcrOptions::default().with_image_dimensions(fixture.dimensions.0, fixture.dimensions.1);

    let cdm = extractor
        .extract_to_cdm(&input, &opts)
        .await
        .expect("extract to CDM");

    let md_opts = MarkdownOptions::default();
    let md = export_markdown(&cdm, &md_opts).expect("export markdown");

    // Syntax validation of Markdown output
    assert!(
        verify_markdown_syntax(&md).is_ok(),
        "Markdown syntax validation failed"
    );

    assert!(md.contains("ACME CORPORATION INVOICE"));
    assert!(md.contains("Payment Verified"));
}

#[tokio::test]
async fn test_ocr_html_export_fidelity() {
    let fixture = build_dark_mode_fixture();
    let adapter = Arc::new(MockOcrAdapter::from_fixture(&fixture));
    let extractor = OcrExtractor::new(adapter);

    let input = OcrInput::from_bytes(fixture.image_bytes.clone())
        .with_filename("terminal.png")
        .with_source_url("file:///test/terminal.png");
    let opts =
        OcrOptions::default().with_image_dimensions(fixture.dimensions.0, fixture.dimensions.1);

    let cdm = extractor
        .extract_to_cdm(&input, &opts)
        .await
        .expect("extract to CDM");

    let html_opts = HtmlOptions::default();
    let (html, _metrics) = export_html(&cdm, &html_opts).expect("export HTML");

    // Syntax validation of HTML output
    assert!(
        verify_html_syntax(&html, true).is_ok(),
        "HTML syntax validation failed"
    );

    assert!(html.contains("SYSTEM TERMINAL STATUS"));
    assert!(html.contains("tokio-worker-1"));
}

#[tokio::test]
async fn test_ocr_json_and_jsonl_export_fidelity() {
    let fixture = build_multicolumn_fixture();
    let adapter = Arc::new(MockOcrAdapter::from_fixture(&fixture));
    let extractor = OcrExtractor::new(adapter);

    let input = OcrInput::from_bytes(fixture.image_bytes.clone())
        .with_filename("multicolumn.png")
        .with_source_url("file:///test/multicolumn.png");
    let opts =
        OcrOptions::default().with_image_dimensions(fixture.dimensions.0, fixture.dimensions.1);

    let cdm = extractor
        .extract_to_cdm(&input, &opts)
        .await
        .expect("extract to CDM");

    // JSON export
    let json_opts = JsonExportOptions::default();
    let json = export_json(&cdm, &json_opts).expect("export JSON");
    assert!(
        verify_json_syntax(&json).is_ok(),
        "JSON syntax validation failed"
    );
    assert!(json.contains("SECTION A: OVERVIEW"));

    // JSONL export
    let jsonl_opts = JsonlOptions::default();
    let (jsonl, _metrics) = export_jsonl(&cdm, &jsonl_opts).expect("export JSONL");
    assert!(
        verify_jsonl_syntax(&jsonl).is_ok(),
        "JSONL syntax validation failed"
    );
    assert!(jsonl.contains("SECTION B: RESULTS"));

    // Plain text export
    let text_opts = PlainTextOptions::default();
    let plain = export_plain_text(&cdm, &text_opts).expect("export plain text");
    assert!(plain.contains("Asynchronous runtime"));
}

#[tokio::test]
async fn test_ocr_extraction_bit_for_bit_determinism() {
    let fixture = build_clean_document_fixture();
    let adapter = Arc::new(MockOcrAdapter::from_fixture(&fixture));
    let extractor = OcrExtractor::new(adapter);

    let input = OcrInput::from_bytes(fixture.image_bytes.clone())
        .with_filename("invoice.png")
        .with_source_url("file:///test/invoice.png");
    let opts =
        OcrOptions::default().with_image_dimensions(fixture.dimensions.0, fixture.dimensions.1);

    let cdm1 = extractor
        .extract_to_cdm(&input, &opts)
        .await
        .expect("extract 1");
    let cdm2 = extractor
        .extract_to_cdm(&input, &opts)
        .await
        .expect("extract 2");

    let md_opts = MarkdownOptions {
        include_frontmatter: false,
        include_metadata_block: false,
        ..Default::default()
    };
    let md1 = export_markdown(&cdm1, &md_opts).expect("export md 1");
    let md2 = export_markdown(&cdm2, &md_opts).expect("export md 2");

    let hash1 = blake3::hash(md1.as_bytes());
    let hash2 = blake3::hash(md2.as_bytes());

    assert_eq!(
        hash1, hash2,
        "Extraction must be bit-for-bit deterministic across consecutive runs"
    );
}

#[tokio::test]
async fn test_ocr_high_throughput_benchmark() {
    let fixture = build_clean_document_fixture();
    let adapter = Arc::new(MockOcrAdapter::from_fixture(&fixture));
    let extractor = OcrExtractor::new(adapter);

    let input = OcrInput::from_bytes(fixture.image_bytes.clone())
        .with_filename("invoice.png")
        .with_source_url("file:///test/invoice.png");
    let opts = OcrOptions::default()
        .with_image_dimensions(fixture.dimensions.0, fixture.dimensions.1)
        .with_preprocessing(PreprocessingOptions::disabled());

    let iterations = 10;
    let start = Instant::now();

    for _ in 0..iterations {
        let _ = extractor
            .extract_to_cdm(&input, &opts)
            .await
            .expect("CDM extraction");
    }

    let elapsed = start.elapsed();
    let avg_ms = elapsed.as_millis() as f64 / iterations as f64;

    assert!(
        avg_ms < 50.0,
        "Average in-memory extraction throughput must be < 50ms per run (actual: {avg_ms:.2} ms)"
    );
}
