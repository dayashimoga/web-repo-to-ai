//! # Tabular Export Fidelity & Determinism Benchmarks
//!
//! Validates downstream roundtrip conversion from tabular-extracted `CdmDocument`
//! through `web-repo-to-ai-exporters` into Markdown GFM, HTML5, JSON CDM v1.0, JSONL,
//! and Plaintext, proving bit-for-bit hash determinism and high throughput (< 50ms).

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
use web_repo_to_ai_structured_extractor::{
    StructuredExtractor, StructuredInput, StructuredOptions,
};

fn sample_tabular_input() -> StructuredInput {
    let csv = "\
order_id,customer_id,product_sku,quantity,unit_price,status,shipped_at\n\
ord_1001,cust_501,SKU-ALPHA,2,49.99,delivered,2026-11-15\n\
ord_1002,cust_502,SKU-BETA,1,120.00,in_transit,2026-11-18\n\
ord_1003,cust_503,SKU-GAMMA,5,15.50,processing,2026-11-20\n\
ord_1004,cust_501,SKU-DELTA,10,8.25,delivered,2026-11-16\n";

    StructuredInput::from_str(csv)
        .with_filename("orders.csv")
        .with_source_url("file:///data/orders.csv")
}

#[test]
fn test_tabular_markdown_export_fidelity() {
    let input = sample_tabular_input();
    let options = StructuredOptions::default();
    let extractor = StructuredExtractor::new();

    let doc = extractor
        .extract_to_cdm(&input, &options)
        .expect("Extraction to CDM failed");

    let md_opts = MarkdownOptions::default();
    let markdown = export_markdown(&doc, &md_opts).expect("Export to Markdown failed");
    let verification = verify_markdown_syntax(&markdown);

    assert!(
        verification.is_ok(),
        "Markdown syntax error: {:?}",
        verification.err()
    );
    assert!(markdown.contains("orders.csv Overview"));
    assert!(markdown.contains("SKU-ALPHA"));
    assert!(markdown.contains("SKU-DELTA"));
    assert!(markdown.contains("delivered"));
}

#[test]
fn test_tabular_html_export_fidelity() {
    let input = sample_tabular_input();
    let options = StructuredOptions::default();
    let extractor = StructuredExtractor::new();

    let doc = extractor
        .extract_to_cdm(&input, &options)
        .expect("Extraction to CDM failed");

    let html_opts = HtmlOptions::default();
    let (html, _metrics) = export_html(&doc, &html_opts).expect("Export to HTML failed");
    let verification = verify_html_syntax(&html, true);

    assert!(
        verification.is_ok(),
        "HTML syntax error: {:?}",
        verification.err()
    );
    assert!(html.contains("orders.csv"));
    assert!(html.contains("SKU-BETA"));
    assert!(html.contains("in_transit"));
}

#[test]
fn test_tabular_json_and_jsonl_export_fidelity() {
    let input = sample_tabular_input();
    let options = StructuredOptions::default();
    let extractor = StructuredExtractor::new();

    let doc = extractor
        .extract_to_cdm(&input, &options)
        .expect("Extraction to CDM failed");

    // 1. JSON Export
    let json_opts = JsonExportOptions::default();
    let json = export_json(&doc, &json_opts).expect("Export to JSON failed");
    assert!(
        verify_json_syntax(&json).is_ok(),
        "JSON syntax validation failed"
    );
    assert!(json.contains("orders.csv"));
    assert!(json.contains("SKU-GAMMA"));

    // 2. JSONL Export
    let jsonl_opts = JsonlOptions::default();
    let (jsonl, _metrics) = export_jsonl(&doc, &jsonl_opts).expect("Export to JSONL failed");
    assert!(
        verify_jsonl_syntax(&jsonl).is_ok(),
        "JSONL syntax validation failed"
    );
    assert!(jsonl.contains("orders.csv"));
    assert!(jsonl.contains("SKU-DELTA"));

    // 3. Plain text Export
    let text_opts = PlainTextOptions::default();
    let txt = export_plain_text(&doc, &text_opts).expect("Export to Plaintext failed");
    assert!(txt.contains("orders.csv"));
    assert!(txt.contains("SKU-ALPHA"));
}

#[test]
fn test_tabular_extraction_bit_for_bit_determinism() {
    let input = sample_tabular_input();
    let options = StructuredOptions::default();
    let extractor = StructuredExtractor::new();

    let doc1 = extractor.extract_to_cdm(&input, &options).unwrap();
    let doc2 = extractor.extract_to_cdm(&input, &options).unwrap();

    let md_opts = MarkdownOptions {
        include_frontmatter: false,
        include_metadata_block: false,
        ..Default::default()
    };

    let md1 = export_markdown(&doc1, &md_opts).unwrap();
    let md2 = export_markdown(&doc2, &md_opts).unwrap();

    let hash1 = blake3::hash(md1.as_bytes()).to_hex().to_string();
    let hash2 = blake3::hash(md2.as_bytes()).to_hex().to_string();

    assert_eq!(
        hash1, hash2,
        "Extraction is not bit-for-bit deterministic across runs"
    );
}

#[test]
fn test_tabular_high_throughput_benchmark() {
    // Generate 5,000 tabular rows
    let mut csv = String::with_capacity(300 * 1024);
    csv.push_str("id,sku,category,price,inventory,last_audit\n");
    for i in 1..=5_000 {
        csv.push_str(&format!(
            "{i},SKU-{:05},Hardware,{}.95,{},2026-11-20\n",
            i,
            i % 100,
            i * 5
        ));
    }

    let input = StructuredInput::from_str(&csv).with_filename("benchmark_5k.csv");
    let options = StructuredOptions::default();
    let extractor = StructuredExtractor::new();

    let iterations = 5;
    let start = Instant::now();

    for _ in 0..iterations {
        let doc = extractor
            .extract_to_cdm(&input, &options)
            .expect("Extraction failed on 5k dataset");
        assert_eq!(doc.sections.len(), 2);
        assert_eq!(doc.completeness.discovered_count, 5000);
        assert_eq!(doc.completeness.extracted_count, 5000);
    }

    let elapsed = start.elapsed();
    let avg_ms = elapsed.as_millis() as f64 / iterations as f64;

    assert!(
        avg_ms < 50.0,
        "5,000-row tabular extraction took {avg_ms:.2} ms per run, exceeding 50ms ceiling"
    );
}
