//! # PDF Multi-Format Export Fidelity & Determinism Benchmark (Subphase 10.1.4)
//!
//! Validates downstream export fidelity, deterministic hashing, and syntax integrity:
//! - Multi-format serialization roundtrip (Markdown, Plaintext, JSON, HTML, JSONL)
//! - Syntax validation across exported formats (GFM pipes, HTML5 tags, JSON schema)
//! - Bit-for-bit extraction determinism and BLAKE3 hash stability across runs
//! - Performance throughput benchmarks on multi-page PDF documents

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
use web_repo_to_ai_pdf_extractor::extractor::PdfExtractor;
use web_repo_to_ai_pdf_extractor::golden::{
    build_academic_multicolumn_fixture, build_adversarial_limits_fixture,
    build_api_specification_fixture, build_financial_report_fixture,
};

#[test]
fn test_financial_report_markdown_export_fidelity() {
    let fixture = build_financial_report_fixture();
    let extractor = PdfExtractor::new();
    let cdm = extractor
        .extract_to_cdm(&fixture.pdf_bytes)
        .expect("extract CDM");

    let opts = MarkdownOptions::default();
    let md = export_markdown(&cdm, &opts).expect("export markdown");

    // Syntax validation of Markdown output
    assert!(
        verify_markdown_syntax(&md).is_ok(),
        "Markdown syntax validation failed"
    );

    // Verify presence of table markdown pipe formatting
    assert!(md.contains("| Segment | Q1 ($M) | Q2 ($M) | Growth |"));
    assert!(md.contains("|---|---|---|---|"));
    assert!(md.contains("| Cloud Infrastructure | 145.2 | 182.6 | +25.8% |"));
    assert!(md.contains("| Hardware Solutions | 31.0 | 28.4 | -8.4% |"));

    // Verify headings and prose
    assert!(md.contains("# Acme Corp Q3 Financial Statement"));
    assert!(md.contains("Consolidated Financial Statement of Operations"));
}

#[test]
fn test_academic_paper_html_export_fidelity() {
    let fixture = build_academic_multicolumn_fixture();
    let extractor = PdfExtractor::new();
    let cdm = extractor
        .extract_to_cdm(&fixture.pdf_bytes)
        .expect("extract CDM");

    let opts = HtmlOptions::default();
    let (html, _metrics) = export_html(&cdm, &opts).expect("export html");

    // Syntax validation of HTML output
    assert!(
        verify_html_syntax(&html, true).is_ok(),
        "HTML syntax validation failed"
    );

    // Verify HTML semantics
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("<html"));
    assert!(html.contains("<article"));
    assert!(html.contains("Zero-Cost Asynchronous Architecture in Rust"));
    assert!(html.contains("1. Introduction"));
    assert!(html.contains("4. Experimental Results"));
    assert!(html.contains("</html>"));
}

#[test]
fn test_api_spec_json_cdm_roundtrip_fidelity() {
    let fixture = build_api_specification_fixture();
    let extractor = PdfExtractor::new();
    let cdm = extractor
        .extract_to_cdm(&fixture.pdf_bytes)
        .expect("extract CDM");

    let opts = JsonExportOptions::default();
    let json_str = export_json(&cdm, &opts).expect("export json");

    // Syntax validation of JSON output
    let val = verify_json_syntax(&json_str).expect("JSON syntax validation failed");

    // Verify structure
    assert_eq!(val["schema_version"], "1.0.0");
    assert_eq!(val["source_type"], "document_pdf");
    assert_eq!(val["metadata"]["title"], "WebRepoToAI REST API Reference");
    assert_eq!(val["sections"].as_array().unwrap().len(), 2);
}

#[test]
fn test_golden_jsonl_export_fidelity() {
    let fixture = build_financial_report_fixture();
    let extractor = PdfExtractor::new();
    let cdm = extractor
        .extract_to_cdm(&fixture.pdf_bytes)
        .expect("extract CDM");

    let opts = JsonlOptions::default();
    let (jsonl_str, _metrics) = export_jsonl(&cdm, &opts).expect("export jsonl");

    // Syntax validation of JSONL output
    let count = verify_jsonl_syntax(&jsonl_str).expect("JSONL syntax validation failed");
    assert!(count >= 2);
}

#[test]
fn test_golden_plaintext_export_fidelity() {
    let fixture = build_academic_multicolumn_fixture();
    let extractor = PdfExtractor::new();
    let cdm = extractor
        .extract_to_cdm(&fixture.pdf_bytes)
        .expect("extract CDM");

    let opts = PlainTextOptions::default();
    let text = export_plain_text(&cdm, &opts).expect("export plaintext");

    assert!(!text.is_empty());
    assert!(text.contains("Zero-Cost Asynchronous Architecture in Rust"));
    assert!(text.contains("1. Introduction"));
    assert!(text.contains("4. Experimental Results"));
    // Plaintext should not contain HTML tags
    assert!(!text.contains("<div>"));
    assert!(!text.contains("<article>"));
}

#[test]
fn test_bit_for_bit_extraction_determinism() {
    let fixture = build_financial_report_fixture();
    let extractor = PdfExtractor::new();
    let opts = MarkdownOptions {
        include_frontmatter: false,
        include_metadata_block: false,
        ..Default::default()
    };

    // Run extraction 5 times consecutively
    let mut hashes = Vec::new();
    for _ in 0..5 {
        let cdm = extractor
            .extract_to_cdm(&fixture.pdf_bytes)
            .expect("extract CDM");
        let md = export_markdown(&cdm, &opts).expect("export markdown");
        let hash = blake3::hash(md.as_bytes()).to_hex().to_string();
        hashes.push(hash);
    }

    // All 5 hashes must be completely identical (bit-for-bit determinism)
    for i in 1..hashes.len() {
        assert_eq!(
            hashes[0], hashes[i],
            "Non-deterministic extraction detected between run 0 and run {}",
            i
        );
    }
}

#[test]
fn test_extraction_performance_throughput_benchmark() {
    let fixture = build_adversarial_limits_fixture(); // 10-page document
    let extractor = PdfExtractor::new();

    let start = Instant::now();
    let iterations = 10;
    for _ in 0..iterations {
        let cdm = extractor
            .extract_to_cdm(&fixture.pdf_bytes)
            .expect("extract CDM");
        assert_eq!(cdm.sections.len(), 10);
    }
    let total_elapsed = start.elapsed();
    let avg_per_doc_ms = total_elapsed.as_millis() as f64 / iterations as f64;

    println!(
        "Benchmark: 10-page document extracted 10 times in {:?} (avg {:.2} ms/doc)",
        total_elapsed, avg_per_doc_ms
    );

    // Extraction of a 10-page document must comfortably take < 50ms on average
    assert!(
        avg_per_doc_ms < 50.0,
        "Extraction too slow: average {:.2} ms per 10-page document",
        avg_per_doc_ms
    );
}
