//! # Jupyter Notebook Export Fidelity & Determinism Benchmarks
//!
//! Validates downstream roundtrip conversion from Jupyter Notebook-extracted `CdmDocument`
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

fn sample_notebook_input() -> StructuredInput {
    let json = r#####"{
  "cells": [
    {
      "cell_type": "markdown",
      "metadata": {},
      "source": [
        "# Machine Learning Pipeline\n",
        "A reproducible training notebook for image classification."
      ]
    },
    {
      "cell_type": "markdown",
      "metadata": {},
      "source": "## 1. Data Preparation"
    },
    {
      "cell_type": "code",
      "execution_count": 1,
      "metadata": {},
      "source": "import torch\nimport torchvision\nprint(f'PyTorch version: {torch.__version__}')",
      "outputs": [
        {
          "output_type": "stream",
          "name": "stdout",
          "text": ["PyTorch version: 2.1.0\n"]
        }
      ]
    },
    {
      "cell_type": "markdown",
      "metadata": {},
      "source": "## 2. Model Training"
    },
    {
      "cell_type": "code",
      "execution_count": 2,
      "metadata": {},
      "source": "loss = 0.042\naccuracy = 0.985\nprint(f'Epoch 10: Loss={loss:.3f}, Acc={accuracy:.1%}')",
      "outputs": [
        {
          "output_type": "stream",
          "name": "stdout",
          "text": ["Epoch 10: Loss=0.042, Acc=98.5%\n"]
        }
      ]
    }
  ],
  "metadata": {
    "language_info": { "name": "python", "version": "3.11.5" },
    "kernelspec": { "name": "python3", "display_name": "Python 3 (ipykernel)" }
  },
  "nbformat": 4,
  "nbformat_minor": 5
}"#####;

    StructuredInput::from_str(json)
        .with_filename("ml_pipeline.ipynb")
        .with_source_url("file:///workspace/ml_pipeline.ipynb")
}

#[test]
fn test_notebook_markdown_export_fidelity() {
    let input = sample_notebook_input();
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
    assert!(markdown.contains("Summary & Overview"));
    assert!(markdown.contains("Machine Learning Pipeline"));
    assert!(markdown.contains("Data Preparation"));
    assert!(markdown.contains("Model Training"));
    assert!(markdown.contains("import torch"));
    assert!(markdown.contains("Epoch 10: Loss=0.042, Acc=98.5%"));
}

#[test]
fn test_notebook_html_export_fidelity() {
    let input = sample_notebook_input();
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
    assert!(html.contains("<article"));
    assert!(html.contains("Machine Learning Pipeline"));
    assert!(html.contains("import torch"));
}

#[test]
fn test_notebook_json_and_jsonl_export_fidelity() {
    let input = sample_notebook_input();
    let options = StructuredOptions::default();
    let extractor = StructuredExtractor::new();

    let doc = extractor
        .extract_to_cdm(&input, &options)
        .expect("Extraction to CDM failed");

    // JSON export
    let json_opts = JsonExportOptions::default();
    let json_str = export_json(&doc, &json_opts).expect("Export to JSON failed");
    assert!(
        verify_json_syntax(&json_str).is_ok(),
        "JSON syntax verification failed"
    );
    assert!(json_str.contains("\"content_type\": \"application/x-ipynb+json\""));
    assert!(json_str.contains("\"primary_language\": \"python\""));

    // JSONL export
    let jsonl_opts = JsonlOptions::default();
    let (jsonl_str, _metrics) = export_jsonl(&doc, &jsonl_opts).expect("Export to JSONL failed");
    assert!(
        verify_jsonl_syntax(&jsonl_str).is_ok(),
        "JSONL syntax verification failed"
    );
    assert!(jsonl_str.contains("ml_pipeline"));
    assert!(jsonl_str.contains("import torch"));

    // Plaintext export
    let plain_opts = PlainTextOptions::default();
    let plain_str = export_plain_text(&doc, &plain_opts).expect("Export to Plaintext failed");
    assert!(plain_str.contains("Machine Learning Pipeline"));
    assert!(plain_str.contains("PyTorch version: 2.1.0"));
}

#[test]
fn test_notebook_extraction_bit_for_bit_determinism() {
    let input = sample_notebook_input();
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
fn test_notebook_high_throughput_benchmark() {
    let input = sample_notebook_input();
    let options = StructuredOptions::default();
    let extractor = StructuredExtractor::new();

    let start = Instant::now();
    let iterations = 100;
    for _ in 0..iterations {
        let doc = extractor.extract_to_cdm(&input, &options).unwrap();
        assert_eq!(doc.sections.len(), 8); // 1 summary + 3 markdown + 2 code + 2 outputs
    }
    let duration = start.elapsed();

    assert!(
        duration.as_millis() < 50,
        "100 notebook iterations should complete under 50ms, took {}ms",
        duration.as_millis()
    );
}
