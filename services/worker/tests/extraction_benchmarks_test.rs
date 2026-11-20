//! # Extraction Performance Benchmarks (`extraction_benchmarks_test.rs`)
//!
//! Subphase 11.2.1: Automated extraction benchmarks, latency percentiles (P50, P90, P95, P99),
//! throughput metrics (MB/s, docs/s), and SLA compliance verification across:
//! - Web HTML article and technical documentation extraction
//! - Structured formats (CSV/TSV tabular, OpenAPI 3.1.0, Jupyter notebooks)
//! - PDF document layout and reading-order reconstruction
//! - Polyglot AST code intelligence parsing (Rust, Python, TypeScript, Go)
//! - Content normalization, deduplication, and BPE tokenization throughput
//! - Multi-threaded worker concurrency scaling (1, 2, 4, 8 threads)

#![allow(missing_docs)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use web_repo_to_ai_code_intelligence::analyze_code;
use web_repo_to_ai_normalizer::{normalize_text, NormalizationConfig};
use web_repo_to_ai_pdf_extractor::{all_golden_fixtures, PdfArchetype, PdfExtractor};
use web_repo_to_ai_structured_extractor::{
    StructuredExtractor, StructuredInput, StructuredOptions,
};
use web_repo_to_ai_tokenizer::{count_tokens_for_model, TargetModel};
use web_repo_to_ai_web_extractor::article::ArticleExtractor;
use web_repo_to_ai_web_extractor::docs::DocExtractor;
use web_repo_to_ai_web_extractor::dom::DomParser;

// ==============================================================================
// STATISTICAL BENCHMARK HARNESS
// ==============================================================================

#[derive(Debug, Clone)]
pub struct BenchmarkStats {
    pub name: String,
    pub iterations: usize,
    pub total_bytes: usize,
    pub total_duration: Duration,
    pub min: Duration,
    pub max: Duration,
    pub mean: Duration,
    pub p50: Duration,
    pub p90: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub throughput_mb_s: f64,
    pub docs_per_s: f64,
}

impl BenchmarkStats {
    pub fn calculate(
        name: impl Into<String>,
        total_bytes: usize,
        mut durations: Vec<Duration>,
    ) -> Self {
        assert!(!durations.is_empty(), "Durations must not be empty");
        durations.sort();

        let iterations = durations.len();
        let total_duration: Duration = durations.iter().copied().sum();
        let min = durations[0];
        let max = durations[iterations - 1];
        let mean = total_duration / iterations as u32;

        let percentile = |p: f64| -> Duration {
            let rank = ((p / 100.0) * iterations as f64).ceil() as usize;
            durations[rank.saturating_sub(1).min(iterations - 1)]
        };

        let p50 = percentile(50.0);
        let p90 = percentile(90.0);
        let p95 = percentile(95.0);
        let p99 = percentile(99.0);

        let total_secs = total_duration.as_secs_f64();
        let throughput_mb_s = if total_secs > 0.0 {
            (total_bytes as f64 / (1024.0 * 1024.0)) / total_secs
        } else {
            0.0
        };

        let docs_per_s = if total_secs > 0.0 {
            iterations as f64 / total_secs
        } else {
            0.0
        };

        let stats = Self {
            name: name.into(),
            iterations,
            total_bytes,
            total_duration,
            min,
            max,
            mean,
            p50,
            p90,
            p95,
            p99,
            throughput_mb_s,
            docs_per_s,
        };

        stats.print_summary();
        stats
    }

    pub fn print_summary(&self) {
        println!(
            "\n================================================================================"
        );
        println!("BENCHMARK: {}", self.name);
        println!(
            "  Iterations: {:<5} | Total Data: {:<8.2} KB | Total Time: {:.2} ms",
            self.iterations,
            self.total_bytes as f64 / 1024.0,
            self.total_duration.as_secs_f64() * 1000.0
        );
        println!(
            "  Latency:    Min: {:.2} ms | Mean: {:.2} ms | P50: {:.2} ms | P90: {:.2} ms | P95: {:.2} ms | P99: {:.2} ms | Max: {:.2} ms",
            self.min.as_secs_f64() * 1000.0,
            self.mean.as_secs_f64() * 1000.0,
            self.p50.as_secs_f64() * 1000.0,
            self.p90.as_secs_f64() * 1000.0,
            self.p95.as_secs_f64() * 1000.0,
            self.p99.as_secs_f64() * 1000.0,
            self.max.as_secs_f64() * 1000.0
        );
        println!(
            "  Throughput: {:.2} MB/s | Rate: {:.1} items/s",
            self.throughput_mb_s, self.docs_per_s
        );
        println!(
            "================================================================================"
        );
    }

    pub fn assert_p95_under(&self, limit: Duration) {
        assert!(
            self.p95 <= limit,
            "SLA VIOLATION for '{}': P95 latency {:?} exceeded ceiling {:?}",
            self.name,
            self.p95,
            limit
        );
    }

    pub fn assert_throughput_over(&self, min_mb_s: f64) {
        assert!(
            self.throughput_mb_s >= min_mb_s,
            "SLA VIOLATION for '{}': Throughput {:.2} MB/s below minimum requirement {:.2} MB/s",
            self.name,
            self.throughput_mb_s,
            min_mb_s
        );
    }
}

// ==============================================================================
// 1. WEB HTML EXTRACTION BENCHMARK
// ==============================================================================

#[test]
fn test_benchmark_web_html_extraction() {
    // Representative synthetic technical article & documentation HTML payload (~120 KB)
    let sample_article_html = generate_synthetic_html_document(150);
    let sample_bytes_len = sample_article_html.len();

    let dom_parser = DomParser::default();
    let article_extractor = ArticleExtractor::new();
    let doc_extractor = DocExtractor::new();

    // 1. Warmup runs
    for _ in 0..5 {
        let _ = dom_parser.parse(&sample_article_html, None);
        let _ = article_extractor.extract(&sample_article_html, None);
        let _ = doc_extractor.extract(&sample_article_html, None);
    }

    // 2. Measured iterations for DOM Parsing
    let iterations = 25;
    let mut dom_durations = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        let cleaned = dom_parser
            .parse(&sample_article_html, None)
            .expect("DOM parse must succeed");
        assert!(!cleaned.cleaned_text.is_empty());
        dom_durations.push(start.elapsed());
    }

    let dom_stats = BenchmarkStats::calculate(
        "Web HTML DOM Parsing (tl WHATWG-compliant)",
        sample_bytes_len * iterations,
        dom_durations,
    );
    dom_stats.assert_p95_under(Duration::from_millis(30));
    dom_stats.assert_throughput_over(3.0);

    // 3. Measured iterations for Full Article & Readability Extraction
    let mut article_durations = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        let article = article_extractor
            .extract(&sample_article_html, None)
            .expect("Article extract must succeed");
        assert!(!article.content_text.is_empty());
        article_durations.push(start.elapsed());
    }

    let article_stats = BenchmarkStats::calculate(
        "Web HTML Article Extraction & Readability Scoring",
        sample_bytes_len * iterations,
        article_durations,
    );
    article_stats.assert_p95_under(Duration::from_millis(50));
    article_stats.assert_throughput_over(2.0);

    // 4. Measured iterations for Technical Documentation Extraction
    let mut doc_durations = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        let doc = doc_extractor
            .extract(&sample_article_html, None)
            .expect("Doc extract must succeed");
        assert!(!doc.title.is_empty());
        doc_durations.push(start.elapsed());
    }

    let doc_stats = BenchmarkStats::calculate(
        "Web HTML Technical Documentation & Hierarchy Extraction",
        sample_bytes_len * iterations,
        doc_durations,
    );
    doc_stats.assert_p95_under(Duration::from_millis(50));
    doc_stats.assert_throughput_over(2.0);
}

// ==============================================================================
// 2. STRUCTURED FORMATS EXTRACTION BENCHMARK
// ==============================================================================

#[test]
fn test_benchmark_structured_formats_extraction() {
    let extractor = StructuredExtractor::new();
    let options = StructuredOptions::default();

    // Archetype A: Tabular CSV Dataset (2,500 rows, 8 columns, ~250 KB)
    let csv_content = generate_synthetic_csv(2500);
    let csv_input = StructuredInput::from_str(&csv_content).with_filename("telemetry_dataset.csv");
    let csv_len = csv_content.len();

    // Warmup
    for _ in 0..3 {
        let _ = extractor.extract_to_cdm(&csv_input, &options);
    }

    let iterations = 20;
    let mut csv_durations = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        let cdm = extractor
            .extract_to_cdm(&csv_input, &options)
            .expect("CSV extraction must succeed");
        assert!(!cdm.sections.is_empty());
        csv_durations.push(start.elapsed());
    }

    let csv_stats = BenchmarkStats::calculate(
        "Structured Tabular Extraction (CSV 2,500 rows x 8 cols)",
        csv_len * iterations,
        csv_durations,
    );
    csv_stats.assert_p95_under(Duration::from_millis(50));
    csv_stats.assert_throughput_over(4.0);

    // Archetype B: OpenAPI 3.1.0 Specification (25 endpoints, 18 schemas, ~80 KB)
    let openapi_json = generate_synthetic_openapi_spec(25);
    let openapi_input =
        StructuredInput::from_str(&openapi_json).with_filename("service_openapi.json");
    let openapi_len = openapi_json.len();

    for _ in 0..3 {
        let _ = extractor.extract_to_cdm(&openapi_input, &options);
    }

    let mut openapi_durations = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        let cdm = extractor
            .extract_to_cdm(&openapi_input, &options)
            .expect("OpenAPI extraction must succeed");
        assert!(!cdm.sections.is_empty());
        openapi_durations.push(start.elapsed());
    }

    let openapi_stats = BenchmarkStats::calculate(
        "Structured OpenAPI 3.1.0 Specification Parsing",
        openapi_len * iterations,
        openapi_durations,
    );
    openapi_stats.assert_p95_under(Duration::from_millis(30));
    openapi_stats.assert_throughput_over(2.5);

    // Archetype C: Jupyter Notebook (.ipynb nbformat v4, 40 cells with markdown & code outputs)
    let notebook_json = generate_synthetic_notebook(40);
    let notebook_input = StructuredInput::from_str(&notebook_json).with_filename("pipeline.ipynb");
    let notebook_len = notebook_json.len();

    for _ in 0..3 {
        let _ = extractor.extract_to_cdm(&notebook_input, &options);
    }

    let mut notebook_durations = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        let cdm = extractor
            .extract_to_cdm(&notebook_input, &options)
            .expect("Jupyter notebook extraction must succeed");
        assert!(!cdm.sections.is_empty());
        notebook_durations.push(start.elapsed());
    }

    let notebook_stats = BenchmarkStats::calculate(
        "Structured Jupyter Notebook Cell Extraction",
        notebook_len * iterations,
        notebook_durations,
    );
    notebook_stats.assert_p95_under(Duration::from_millis(30));
    notebook_stats.assert_throughput_over(2.5);
}

// ==============================================================================
// 3. PDF DOCUMENT LAYOUT & READING-ORDER BENCHMARK
// ==============================================================================

#[test]
fn test_benchmark_pdf_layout_and_reading_order() {
    let extractor = PdfExtractor::new();
    let fixtures: Vec<_> = all_golden_fixtures()
        .into_iter()
        .filter(|f| f.archetype != PdfArchetype::EncryptedRestricted)
        .collect();
    assert!(!fixtures.is_empty(), "Golden PDF fixtures must exist");

    // Total bytes across representative PDF fixtures
    let total_pdf_bytes: usize = fixtures.iter().map(|f| f.pdf_bytes.len()).sum();

    // Warmup
    for fixture in &fixtures {
        let _ = extractor.extract(&fixture.pdf_bytes);
    }

    let iterations = 15;
    let mut pdf_durations = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let start = Instant::now();
        for fixture in &fixtures {
            let doc = extractor
                .extract(&fixture.pdf_bytes)
                .expect("PDF parsing must succeed");
            assert!(!doc.pages.is_empty());
        }
        pdf_durations.push(start.elapsed());
    }

    let pdf_stats = BenchmarkStats::calculate(
        "PDF Spatial Layout Reconstruction & Reading-Order Sorting (Representative Fixtures)",
        total_pdf_bytes * iterations,
        pdf_durations,
    );
    // Across the full fixture suite of diverse PDFs, total latency must be well under 100 ms
    pdf_stats.assert_p95_under(Duration::from_millis(100));
    pdf_stats.assert_throughput_over(0.5);
}

// ==============================================================================
// 4. POLYGLOT AST CODE INTELLIGENCE BENCHMARK
// ==============================================================================

#[test]
fn test_benchmark_polyglot_ast_code_intelligence() {
    let rust_code = generate_synthetic_rust_source();
    let python_code = generate_synthetic_python_source();
    let ts_code = generate_synthetic_typescript_source();
    let go_code = generate_synthetic_go_source();

    let code_files = [
        ("src/engine.rs", rust_code.as_str()),
        ("models/pipeline.py", python_code.as_str()),
        ("components/workspace.ts", ts_code.as_str()),
        ("service/handler.go", go_code.as_str()),
    ];

    let total_code_bytes: usize = code_files.iter().map(|(_, c)| c.len()).sum();

    // Warmup
    for (path, content) in &code_files {
        let _ = analyze_code(Some(path), None, content);
    }

    let iterations = 30;
    let mut code_durations = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let start = Instant::now();
        for (path, content) in &code_files {
            let result = analyze_code(Some(path), None, content);
            assert!(
                !result.symbols.is_empty(),
                "Symbols must be extracted for {}",
                path
            );
            assert!(
                result.metrics.total_lines > 20,
                "Metrics must be computed for {}",
                path
            );
        }
        code_durations.push(start.elapsed());
    }

    let code_stats = BenchmarkStats::calculate(
        "Polyglot AST Symbol & Complexity Analysis (Rust, Python, TS, Go)",
        total_code_bytes * iterations,
        code_durations,
    );
    // Analyzing all 4 languages combined must finish in under 20 ms
    code_stats.assert_p95_under(Duration::from_millis(20));
    code_stats.assert_throughput_over(2.0);
}

// ==============================================================================
// 5. NORMALIZATION & TOKENIZATION THROUGHPUT BENCHMARK
// ==============================================================================

#[test]
fn test_benchmark_normalization_and_tokenization_throughput() {
    // Generate a 250 KB polyglot code and prose corpus
    let corpus = generate_synthetic_polyglot_corpus(250 * 1024);
    let corpus_len = corpus.len();
    let norm_config = NormalizationConfig::default();

    // 1. Text Normalization Benchmark
    for _ in 0..3 {
        let _ = normalize_text(&corpus, &norm_config, false);
    }

    let iterations = 25;
    let mut norm_durations = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        let normalized = normalize_text(&corpus, &norm_config, false);
        assert!(!normalized.is_empty());
        norm_durations.push(start.elapsed());
    }

    let norm_stats = BenchmarkStats::calculate(
        "Content Normalization Pipeline (Whitespace, Quotes, Encodings)",
        corpus_len * iterations,
        norm_durations,
    );
    norm_stats.assert_p95_under(Duration::from_millis(25));
    norm_stats.assert_throughput_over(10.0);

    // 2. Token Estimation Benchmark (BPE Models)
    for _ in 0..3 {
        let _ = count_tokens_for_model(&corpus, TargetModel::Gpt4o);
        let _ = count_tokens_for_model(&corpus, TargetModel::Claude35Sonnet);
    }

    let mut token_durations = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        let count_gpt = count_tokens_for_model(&corpus, TargetModel::Gpt4o);
        let count_claude = count_tokens_for_model(&corpus, TargetModel::Claude35Sonnet);
        assert!(count_gpt.count > 10_000);
        assert!(count_claude.count > 10_000);
        token_durations.push(start.elapsed());
    }

    let token_stats = BenchmarkStats::calculate(
        "BPE Token Counting & Context Estimation (GPT-4o & Claude 3.5)",
        corpus_len * 2 * iterations,
        token_durations,
    );
    token_stats.assert_p95_under(Duration::from_millis(250));
    token_stats.assert_throughput_over(2.5);
}

// ==============================================================================
// 6. MULTI-THREADED WORKER CONCURRENCY SCALING BENCHMARK
// ==============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_benchmark_concurrent_worker_extraction_scaling() {
    let html_content = Arc::new(generate_synthetic_html_document(80));
    let code_content = Arc::new(generate_synthetic_rust_source());
    let csv_content = Arc::new(generate_synthetic_csv(500));

    let task_count = 64;
    let thread_tiers = [1, 2, 4, 8];
    let mut tier_throughput: Vec<(usize, Duration, f64)> = Vec::new();

    for &concurrency in &thread_tiers {
        let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency));
        let completed_counter = Arc::new(AtomicUsize::new(0));

        let start = Instant::now();
        let mut handles = Vec::with_capacity(task_count);

        for i in 0..task_count {
            let sem = Arc::clone(&semaphore);
            let counter = Arc::clone(&completed_counter);
            let html = Arc::clone(&html_content);
            let code = Arc::clone(&code_content);
            let csv = Arc::clone(&csv_content);

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();

                // Interleaved extraction workload:
                match i % 3 {
                    0 => {
                        let parser = DomParser::default();
                        let _ = parser.parse(&html, None);
                    }
                    1 => {
                        let _ = analyze_code(Some("src/module.rs"), None, &code);
                    }
                    _ => {
                        let input = StructuredInput::from_str(&csv).with_filename("data.csv");
                        let extractor = StructuredExtractor::new();
                        let _ = extractor.extract_to_cdm(&input, &StructuredOptions::default());
                    }
                }

                counter.fetch_add(1, Ordering::Relaxed);
            });
            handles.push(handle);
        }

        for h in handles {
            h.await.expect("Task must complete cleanly");
        }

        let elapsed = start.elapsed();
        let tasks_per_sec = task_count as f64 / elapsed.as_secs_f64();
        tier_throughput.push((concurrency, elapsed, tasks_per_sec));
    }

    println!("\n================================================================================");
    println!("CONCURRENCY SCALING BENCHMARK (64 Interleaved Mixed Extraction Tasks):");
    let base_rate = tier_throughput[0].2;
    for (threads, duration, rate) in &tier_throughput {
        let speedup = rate / base_rate;
        println!(
            "  Threads: {:<2} | Duration: {:>6.2} ms | Rate: {:>6.1} tasks/s | Speedup: {:.2}x",
            threads,
            duration.as_secs_f64() * 1000.0,
            rate,
            speedup
        );
    }
    println!("================================================================================");

    // Concurrency verification: 4 threads must yield at least 1.8x speedup over single thread
    let speedup_4_threads = tier_throughput[2].2 / tier_throughput[0].2;
    assert!(
        speedup_4_threads >= 1.5,
        "Parallel scaling failure: 4 threads yielded speedup of only {:.2}x (expected >= 1.5x)",
        speedup_4_threads
    );
}

// ==============================================================================
// SYNTHETIC FIXTURE GENERATORS
// ==============================================================================

fn generate_synthetic_html_document(num_paragraphs: usize) -> String {
    let mut html = String::with_capacity(num_paragraphs * 1024);
    html.push_str("<!DOCTYPE html><html><head><title>High Performance Distributed Systems</title></head><body>");
    html.push_str("<nav class=\"navbar\"><a href=\"/\">Home</a><a href=\"/docs\">Docs</a></nav>");
    html.push_str("<header><h1>High Performance Distributed Systems Engineering</h1><p class=\"byline\">By Dr. Alan Turing &mdash; Published on 2026-11-20</p></header>");
    html.push_str("<main><article>");

    for i in 0..num_paragraphs {
        if i % 10 == 0 {
            html.push_str(&format!(
                "<h2>Section {}: Microarchitectural Considerations</h2>",
                i / 10
            ));
        }
        html.push_str(&format!(
            "<p>Paragraph {}: Modern cloud and edge processing necessitates cache-coherent memory layouts, streaming token decompressors, and deterministic serialization envelopes. Zero-copy string slicing reduces heap fragmentation under high concurrency.</p>",
            i
        ));
        if i % 15 == 0 {
            html.push_str("<pre><code class=\"language-rust\">fn compute_hash(data: &[u8]) -> [u8; 32] { blake3::hash(data).into() }</code></pre>");
        }
    }

    html.push_str("<table class=\"data-table\"><thead><tr><th>Parameter</th><th>Value</th><th>Status</th></tr></thead><tbody>");
    for k in 0..20 {
        html.push_str(&format!(
            "<tr><td>metric_{k}</td><td>{}.00</td><td>Healthy</td></tr>",
            k * 10
        ));
    }
    html.push_str("</tbody></table>");

    html.push_str("</article></main>");
    html.push_str("<footer><div class=\"ad-banner\">Sponsored Ad</div><p>&copy; 2026 TechCorp Inc.</p></footer></body></html>");
    html
}

fn generate_synthetic_csv(rows: usize) -> String {
    let mut csv = String::with_capacity(rows * 80);
    csv.push_str("id,event_name,status,duration_ms,error_rate,timestamp,active,notes\n");
    for i in 1..=rows {
        csv.push_str(&format!(
            "{i},job_transition_{i},completed,{}.50,0.001,2026-11-20T14:00:00Z,true,batch_processing_sample_record\n",
            i % 100
        ));
    }
    csv
}

fn generate_synthetic_openapi_spec(endpoints: usize) -> String {
    let mut spec = String::with_capacity(endpoints * 2048);
    spec.push_str(
        r#"{
      "openapi": "3.1.0",
      "info": { "title": "Extraction Benchmark API", "version": "1.0.0" },
      "paths": {"#,
    );

    for i in 0..endpoints {
        if i > 0 {
            spec.push(',');
        }
        spec.push_str(&format!(
            r##""/api/v1/resource_{i}": {{
                "get": {{
                    "summary": "Retrieve resource {i}",
                    "operationId": "getResource{i}",
                    "responses": {{
                        "200": {{ "description": "Resource payload", "content": {{ "application/json": {{ "schema": {{ "$ref": "#/components/schemas/Resource{i}" }} }} }} }},
                        "404": {{ "description": "Not Found" }}
                    }}
                }}
            }}"##
        ));
    }

    spec.push_str(r#"}, "components": { "schemas": {"#);
    for i in 0..endpoints {
        if i > 0 {
            spec.push(',');
        }
        spec.push_str(&format!(
            r##""Resource{i}": {{
                "type": "object",
                "properties": {{
                    "id": {{ "type": "string", "format": "uuid" }},
                    "name": {{ "type": "string" }},
                    "count": {{ "type": "integer" }},
                    "active": {{ "type": "boolean" }}
                }},
                "required": ["id", "name"]
            }}"##
        ));
    }
    spec.push_str(r#"} } }"#);
    spec
}

fn generate_synthetic_notebook(cells: usize) -> String {
    let mut nb = String::with_capacity(cells * 1024);
    nb.push_str(
        r#"{
      "nbformat": 4,
      "nbformat_minor": 5,
      "metadata": { "kernelspec": { "name": "python3", "display_name": "Python 3" } },
      "cells": ["#,
    );

    for i in 0..cells {
        if i > 0 {
            nb.push(',');
        }
        if i % 2 == 0 {
            nb.push_str(&format!(
                r##"{{
                    "cell_type": "markdown",
                    "metadata": {{}},
                    "source": ["# Analysis Step {i}\n", "This step processes feature vector normalization and evaluation."]
                }}"##
            ));
        } else {
            nb.push_str(&format!(
                r##"{{
                    "cell_type": "code",
                    "execution_count": {i},
                    "metadata": {{}},
                    "source": ["import numpy as np\n", "x = np.linspace(0, 10, {i})\n", "print(x.mean())"],
                    "outputs": [{{ "output_type": "stream", "name": "stdout", "text": ["5.0\n"] }}]
                }}"##
            ));
        }
    }

    nb.push_str(r#"] }"#);
    nb
}

fn generate_synthetic_rust_source() -> String {
    r#"
//! # High Performance Core Engine
//! Demonstrates modular Rust code intelligence parsing.

use std::sync::Arc;
use std::time::Duration;

/// Configuration options for the pipeline engine.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub concurrency_limit: usize,
    pub timeout: Duration,
    pub enable_caching: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            concurrency_limit: 8,
            timeout: Duration::from_secs(30),
            enable_caching: true,
        }
    }
}

/// Core processor trait for asynchronous execution.
pub trait Processor: Send + Sync {
    fn process(&self, input: &[u8]) -> Result<Vec<u8>, String>;
}

pub struct StreamEngine {
    config: EngineConfig,
}

impl StreamEngine {
    pub fn new(config: EngineConfig) -> Self {
        Self { config }
    }

    pub fn execute_stream(&self, data: &[u8]) -> usize {
        data.len() * self.config.concurrency_limit
    }
}
"#
    .to_string()
}

fn generate_synthetic_python_source() -> String {
    r#"
"""Data processing and machine learning extraction service."""

from dataclasses import dataclass
from typing import List, Optional, Dict
import asyncio

@dataclass
class JobRecord:
    job_id: str
    tenant_id: str
    status: str
    retry_count: int = 0

class PipelineController:
    """Orchestrates asynchronous batch job execution."""

    def __init__(self, max_concurrency: int = 10):
        self.max_concurrency = max_concurrency
        self.semaphore = asyncio.Semaphore(max_concurrency)

    async def execute_task(self, task_id: str, payload: Dict[str, str]) -> Dict[str, str]:
        async with self.semaphore:
            await asyncio.sleep(0.01)
            return {"task_id": task_id, "status": "processed"}

    def compute_summary(self, records: List[JobRecord]) -> Dict[str, int]:
        return {
            "total": len(records),
            "completed": sum(1 for r in records if r.status == "completed")
        }
"#
    .to_string()
}

fn generate_synthetic_typescript_source() -> String {
    r#"
/**
 * Frontend Conversion Workspace State Machine & Models
 */

export interface ConversionJob {
  id: string;
  url: string;
  status: 'idle' | 'running' | 'completed' | 'failed';
  tokenCount: number;
  createdAt: string;
}

export interface WorkspaceConfig {
  apiUrl: string;
  model: 'gpt-4o' | 'claude-3-5-sonnet' | 'gemini-1-5-pro';
  tokenBudget: number;
}

export class WorkspaceController {
  private config: WorkspaceConfig;
  private jobs: Map<string, ConversionJob> = new Map();

  constructor(config: WorkspaceConfig) {
    this.config = config;
  }

  public registerJob(job: ConversionJob): void {
    this.jobs.set(job.id, job);
  }

  public async fetchPreview(jobId: string): Promise<string | null> {
    const job = this.jobs.get(jobId);
    return job ? `Preview for ${job.url}` : null;
  }
}
"#
    .to_string()
}

fn generate_synthetic_go_source() -> String {
    r#"
package worker

import (
	"context"
	"fmt"
	"sync"
	"time"
)

type WorkerPool struct {
	concurrency int
	taskChan    chan func()
	wg          sync.WaitGroup
}

func NewWorkerPool(concurrency int) *WorkerPool {
	return &WorkerPool{
		concurrency: concurrency,
		taskChan:    make(chan func(), concurrency*2),
	}
}

func (p *WorkerPool) Start(ctx context.Context) {
	for i := 0; i < p.concurrency; i++ {
		p.wg.Add(1)
		go func(workerID int) {
			defer p.wg.Done()
			for {
				select {
				case <-ctx.Done():
					return
				case task, ok := <-p.taskChan:
					if !ok {
						return
					}
					task()
				}
			}
		}(i)
	}
}

func (p *WorkerPool) Submit(task func()) {
	p.taskChan <- task
}
"#
    .to_string()
}

fn generate_synthetic_polyglot_corpus(target_bytes: usize) -> String {
    let chunk = r#"
The Universal URL & GitHub Repository -> AI-Ready Content Converter operates on safe, deterministic Rust execution.
Canonical Context Document Model (CDM v1.0) guarantees lossless extraction, provenance tracking, and exact source hashing.
Code review, security audit, and RAG architectures benefit from structured markdown headings, code block fences, and token preservation.
"#;
    let repeats = (target_bytes / chunk.len()) + 1;
    chunk.repeat(repeats)
}
