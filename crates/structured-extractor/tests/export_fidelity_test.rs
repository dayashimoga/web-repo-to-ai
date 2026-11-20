//! # Structured Document Multi-Format Export Fidelity and Determinism Benchmarks
//!
//! Validates roundtrip serialization through all exporters (Markdown, HTML5, JSON, JSONL, Plaintext),
//! syntax integrity validation, bit-for-bit extraction determinism, and high throughput.

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

const OPENAPI_SPEC: &str = r##"
openapi: 3.1.0
info:
  title: Orders and Billing Service
  version: 1.0.0
  description: High-throughput payment processing and invoice settlement API.
paths:
  /orders:
    post:
      summary: Create new order
      description: Charges payment method and generates invoice record.
      operationId: createOrder
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: "#/components/schemas/OrderRequest"
      responses:
        "201":
          description: Order created
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/OrderResponse"
components:
  schemas:
    OrderRequest:
      type: object
      required:
        - customer_id
        - amount_cents
      properties:
        customer_id:
          type: string
          format: uuid
        amount_cents:
          type: integer
          format: int64
    OrderResponse:
      type: object
      required:
        - order_id
        - status
      properties:
        order_id:
          type: string
          format: uuid
        status:
          type: string
"##;

#[test]
fn test_openapi_markdown_export_fidelity() {
    let extractor = StructuredExtractor::new();
    let input = StructuredInput::from_str(OPENAPI_SPEC).with_filename("billing.yaml");
    let cdm = extractor
        .extract_to_cdm(&input, &StructuredOptions::default())
        .expect("extract to CDM");

    let md_opts = MarkdownOptions::default();
    let md = export_markdown(&cdm, &md_opts).expect("export markdown");

    assert!(
        verify_markdown_syntax(&md).is_ok(),
        "Markdown syntax validation failed"
    );
    assert!(md.contains("Orders and Billing Service"));
    assert!(md.contains("POST /orders"));
    assert!(md.contains("Model: OrderRequest"));
}

#[test]
fn test_openapi_html_export_fidelity() {
    let extractor = StructuredExtractor::new();
    let input = StructuredInput::from_str(OPENAPI_SPEC).with_filename("billing.yaml");
    let cdm = extractor
        .extract_to_cdm(&input, &StructuredOptions::default())
        .expect("extract to CDM");

    let html_opts = HtmlOptions::default();
    let (html, _metrics) = export_html(&cdm, &html_opts).expect("export HTML");

    assert!(
        verify_html_syntax(&html, true).is_ok(),
        "HTML syntax validation failed"
    );
    assert!(html.contains("Orders and Billing Service"));
    assert!(html.contains("createOrder"));
}

#[test]
fn test_openapi_json_and_jsonl_export_fidelity() {
    let extractor = StructuredExtractor::new();
    let input = StructuredInput::from_str(OPENAPI_SPEC).with_filename("billing.yaml");
    let cdm = extractor
        .extract_to_cdm(&input, &StructuredOptions::default())
        .expect("extract to CDM");

    let json_opts = JsonExportOptions::default();
    let json = export_json(&cdm, &json_opts).expect("export JSON");
    assert!(
        verify_json_syntax(&json).is_ok(),
        "JSON syntax validation failed"
    );
    assert!(json.contains("Orders and Billing Service"));

    let jsonl_opts = JsonlOptions::default();
    let (jsonl, _metrics) = export_jsonl(&cdm, &jsonl_opts).expect("export JSONL");
    assert!(
        verify_jsonl_syntax(&jsonl).is_ok(),
        "JSONL syntax validation failed"
    );
    assert!(jsonl.contains("createOrder"));

    let text_opts = PlainTextOptions::default();
    let plain = export_plain_text(&cdm, &text_opts).expect("export plain text");
    assert!(plain.contains("Orders and Billing Service"));
}

#[test]
fn test_structured_extraction_bit_for_bit_determinism() {
    let extractor = StructuredExtractor::new();
    let input = StructuredInput::from_str(OPENAPI_SPEC).with_filename("billing.yaml");
    let opts = StructuredOptions::default();
    let md_opts = MarkdownOptions {
        include_frontmatter: false,
        include_metadata_block: false,
        ..Default::default()
    };

    let mut hashes = Vec::new();
    for _ in 0..5 {
        let cdm = extractor
            .extract_to_cdm(&input, &opts)
            .expect("extract CDM");
        let md = export_markdown(&cdm, &md_opts).expect("export markdown");
        let hash = blake3::hash(md.as_bytes()).to_hex().to_string();
        hashes.push(hash);
    }

    for i in 1..hashes.len() {
        assert_eq!(
            hashes[0], hashes[i],
            "Non-deterministic extraction detected between run 0 and run {i}"
        );
    }
}

#[test]
fn test_structured_high_throughput_benchmark() {
    let extractor = StructuredExtractor::new();
    let input = StructuredInput::from_str(OPENAPI_SPEC).with_filename("billing.yaml");
    let opts = StructuredOptions::default();

    let iterations = 10;
    let start = Instant::now();

    for _ in 0..iterations {
        let _ = extractor
            .extract_to_cdm(&input, &opts)
            .expect("CDM extraction");
    }

    let elapsed = start.elapsed();
    let avg_ms = elapsed.as_millis() as f64 / iterations as f64;

    assert!(
        avg_ms < 50.0,
        "Average extraction throughput must be < 50ms per run (actual: {avg_ms:.2} ms)"
    );
}
