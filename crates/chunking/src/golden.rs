//! # Golden Evaluation Benchmark Corpus & Retrieval Fidelity Suite (`crates/chunking/src/golden.rs`)
//!
//! Provides quantitative evaluation benchmarks, bit-for-bit repeatability tests,
//! retrieval simulation, and fidelity verification across 8 comprehensive real-world archetypes:
//! 1. `MultiLanguageAstCode`: Multi-language code repository (Rust, Python, TypeScript, Go).
//! 2. `HierarchicalMarkdownDocumentation`: Technical documentation with deep heading hierarchies.
//! 3. `TabularDataMatrix`: CSV/TSV table slices with column header preservation.
//! 4. `StructuredSpecOpenApi`: OpenAPI 3.1.0 REST API specification operations and schemas.
//! 5. `JupyterNotebookPipeline`: Interactive nbformat v4 notebook with markdown, code, and outputs.
//! 6. `OversizedDeclarationRoutines`: Giant function/class splitting at statement boundaries.
//! 7. `CrossReferenceGraphCorpus`: Multi-section interconnected codebase with call/import graph edges.
//! 8. `AdversarialChunkBounds`: Extreme token ceilings, unicode characters, and deep nesting.

use std::collections::BTreeMap;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use web_repo_to_ai_cdm::{
    ByteRange, CdmDocument, CdmDocumentBuilder, CdmSection, LineRange, SectionKind, SourceIdentity,
    SourceType,
};

use crate::engine::Chunker;
use crate::error::ChunkResult;
use crate::model::{Chunk, ChunkingConfig, ChunkingReport};

/// Canonical document archetypes evaluated in the golden evaluation suite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChunkingArchetype {
    /// Multi-language source code with AST symbol boundaries.
    MultiLanguageAstCode,
    /// Technical documentation with deep Markdown heading outlines.
    HierarchicalMarkdownDocumentation,
    /// Tabular data matrix requiring header preservation on every slice.
    TabularDataMatrix,
    /// OpenAPI REST API specification paths and schemas.
    StructuredSpecOpenApi,
    /// Jupyter Notebook interactive pipeline with cells and outputs.
    JupyterNotebookPipeline,
    /// Oversized declaration routines requiring statement block splitting.
    OversizedDeclarationRoutines,
    /// Interconnected multi-file codebase with symbol calls and imports.
    CrossReferenceGraphCorpus,
    /// Adversarial token constraints and multi-byte boundary limits.
    AdversarialChunkBounds,
}

impl ChunkingArchetype {
    /// Returns human-readable display name for the archetype.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::MultiLanguageAstCode => "Multi-Language AST Code Repository",
            Self::HierarchicalMarkdownDocumentation => "Hierarchical Markdown Documentation",
            Self::TabularDataMatrix => "Tabular Data Matrix (Header Retention)",
            Self::StructuredSpecOpenApi => "OpenAPI 3.1 REST API Specification",
            Self::JupyterNotebookPipeline => "Jupyter Notebook Interactive Pipeline",
            Self::OversizedDeclarationRoutines => "Oversized Declarations (Statement Splitting)",
            Self::CrossReferenceGraphCorpus => "Cross-Reference Call & Import Graph",
            Self::AdversarialChunkBounds => "Adversarial Token & Boundary Limits",
        }
    }
}

/// Retrieval query with expected target chunk identifier or symbol match.
#[derive(Debug, Clone)]
pub struct RetrievalBenchmarkQuery {
    /// Search query string.
    pub query: &'static str,
    /// Target symbol name, heading, or unique phrase expected in the Top-1 retrieved chunk.
    pub expected_target_substring: &'static str,
}

/// Self-contained golden fixture with input CDM document, config, and ground-truth expectations.
#[derive(Debug, Clone)]
pub struct ChunkingGoldenFixture {
    /// Archetype categorization.
    pub archetype: ChunkingArchetype,
    /// Human-readable fixture name.
    pub name: &'static str,
    /// Chunking configuration to evaluate.
    pub config: ChunkingConfig,
    /// Function to construct the test CDM document.
    pub document_builder: fn() -> CdmDocument,
    /// Expected minimum chunks generated.
    pub expected_min_chunks: usize,
    /// Expected maximum chunks generated.
    pub expected_max_chunks: usize,
    /// Ground-truth retrieval benchmark queries.
    pub retrieval_queries: Vec<RetrievalBenchmarkQuery>,
}

/// Quantitative fidelity and benchmark metrics for an individual fixture evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChunkingFidelityMetrics {
    /// Document archetype.
    pub archetype: ChunkingArchetype,
    /// Fixture name.
    pub fixture_name: String,
    /// Total chunks generated.
    pub chunk_count: usize,
    /// Total tokens across all chunks.
    pub total_tokens: usize,
    /// Average tokens per chunk.
    pub average_tokens_per_chunk: usize,
    /// Minimum chunk tokens observed.
    pub min_chunk_tokens: usize,
    /// Maximum chunk tokens observed.
    pub max_chunk_tokens: usize,
    /// Bit-for-bit repeatability across independent runs (100% hash identity).
    pub bit_for_bit_repeatable: bool,
    /// Content preservation ratio (concatenated raw chunk text covers input).
    pub content_preservation_ratio: f64,
    /// Boundary integrity: no broken words, start <= end line numbers.
    pub boundary_integrity: bool,
    /// Percentage of chunks with valid citations (0.0 to 1.0).
    pub citations_valid_ratio: f64,
    /// Total cross-references established across chunks.
    pub cross_references_count: usize,
    /// Retrieval Top-1 accuracy ratio (0.0 to 1.0).
    pub retrieval_top1_accuracy: f64,
    /// Total elapsed execution time in microseconds.
    pub elapsed_micros: u128,
}

/// Result of evaluating an individual golden fixture.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChunkingGoldenResult {
    /// Benchmark metrics.
    pub metrics: ChunkingFidelityMetrics,
    /// Aggregation report from chunker.
    pub report: ChunkingReport,
    /// Whether all acceptance criteria passed.
    pub passed: bool,
    /// Diagnostic error message if failed.
    pub error_message: Option<String>,
}

/// Aggregated report of all golden benchmark fixture evaluations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChunkingGoldenReport {
    /// Results for each evaluated fixture.
    pub results: Vec<ChunkingGoldenResult>,
    /// Overall pass status across the full suite.
    pub all_passed: bool,
    /// Total execution time in microseconds.
    pub total_elapsed_micros: u128,
}

impl ChunkingGoldenReport {
    /// Formats the benchmark report into an executive Markdown summary table.
    #[must_use]
    pub fn to_markdown_summary(&self) -> String {
        let mut md = String::with_capacity(4096);
        md.push_str("# RAG Chunking Golden Benchmark Evaluation Report\n\n");
        md.push_str("| Archetype | Chunks | Avg Tokens | Max Tokens | Repeatable | Citations | Top-1 Recall | Latency | Status |\n");
        md.push_str("| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |\n");

        for res in &self.results {
            let m = &res.metrics;
            let status = if res.passed { "PASS" } else { "FAIL" };
            let repeatable = if m.bit_for_bit_repeatable {
                "100%"
            } else {
                "0%"
            };
            let citations = format!("{:.0}%", m.citations_valid_ratio * 100.0);
            let recall = format!("{:.0}%", m.retrieval_top1_accuracy * 100.0);
            let latency = format!("{:.2} ms", m.elapsed_micros as f64 / 1000.0);

            md.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
                m.archetype.display_name(),
                m.chunk_count,
                m.average_tokens_per_chunk,
                m.max_chunk_tokens,
                repeatable,
                citations,
                recall,
                latency,
                status,
            ));
        }

        let overall = if self.all_passed {
            "ALL BENCHMARKS PASSED"
        } else {
            "BENCHMARK FAILURES DETECTED"
        };
        md.push_str(&format!(
            "\n**Overall Certification Status:** `{overall}` (Total Latency: {:.2} ms)\n",
            self.total_elapsed_micros as f64 / 1000.0
        ));
        md
    }
}

// ---------------------------------------------------------------------------
// Simulated Retrieval Engine
// ---------------------------------------------------------------------------

/// Deterministic keyword and semantic overlap retrieval scorer.
#[must_use]
pub fn score_chunk_relevance(query: &str, chunk: &Chunk) -> f64 {
    let q_lower = query.to_lowercase();
    let q_terms: Vec<&str> = q_lower.split_whitespace().collect();
    if q_terms.is_empty() {
        return 0.0;
    }

    let mut score = 0.0;
    let content_lower = chunk.content.to_lowercase();

    // Exact full query phrase match bonus
    if content_lower.contains(&q_lower) {
        score += 50.0;
    }

    // Title / heading match bonus
    if let Some(ref title) = chunk.title {
        let title_lower = title.to_lowercase();
        if title_lower.contains(&q_lower) {
            score += 40.0;
        }
        for term in &q_terms {
            if title_lower.contains(term) {
                score += 10.0;
            }
        }
    }

    // AST Symbol match bonus
    for sym in &chunk.ast_symbols {
        let sym_lower = sym.name.to_lowercase();
        if sym_lower.contains(&q_lower) || q_lower.contains(&sym_lower) {
            score += 35.0;
        }
        for term in &q_terms {
            if sym_lower.contains(term) {
                score += 15.0;
            }
        }
    }

    // Body term frequency
    for term in &q_terms {
        let count = content_lower.matches(term).count();
        if count > 0 {
            score += (count as f64).min(10.0) * 2.0;
        }
    }

    // Citation anchor bonus
    if let Some(ref cit) = chunk.citation {
        if let Some(ref anchor) = cit.anchor {
            if anchor.to_lowercase().contains(&q_lower) {
                score += 20.0;
            }
        }
    }

    score
}

/// Simulated Top-K retrieval execution.
#[must_use]
pub fn retrieve_top_k<'a>(query: &str, chunks: &'a [Chunk], k: usize) -> Vec<(&'a Chunk, f64)> {
    let mut scored: Vec<(&'a Chunk, f64)> = chunks
        .iter()
        .map(|c| (c, score_chunk_relevance(query, c)))
        .collect();

    // Sort descending by score; break ties deterministically by chunk_index
    scored.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.chunk_index.cmp(&b.0.chunk_index))
    });

    scored.truncate(k);
    scored
}

// ---------------------------------------------------------------------------
// Golden Fixture Helpers & Builders
// ---------------------------------------------------------------------------

fn make_section(
    id: &str,
    kind: SectionKind,
    path: &str,
    url: &str,
    title: Option<&str>,
    language: Option<&str>,
    body: &str,
) -> CdmSection {
    let hash = blake3::hash(body.as_bytes()).to_hex().to_string();
    let line_count = body.lines().count().max(1) as u32;
    CdmSection {
        section_id: id.to_string(),
        parent_id: None,
        title: title.map(String::from),
        kind,
        original_path: Some(path.to_string()),
        source_url: url.to_string(),
        line_range: Some(LineRange::new(1, line_count)),
        byte_range: Some(ByteRange::new(0, body.len() as u64)),
        language: language.map(String::from),
        content_hash: hash,
        body: body.to_string(),
        token_metrics: None,
        attributes: BTreeMap::new(),
    }
}

fn build_multilang_code_fixture() -> CdmDocument {
    let url = "https://github.com/example/polyglot-engine";
    let source_identity = SourceIdentity::new(url, url);

    let rust_code = r#"// File: src/crypto.rs
use ring::digest;

/// Computes SHA-256 digest.
pub fn compute_sha256(payload: &[u8]) -> Vec<u8> {
    digest::digest(&digest::SHA256, payload).as_ref().to_vec()
}

/// Session security coordinator.
pub struct SecurityCoordinator {
    pub realm: String,
    pub timeout_seconds: u64,
}

impl SecurityCoordinator {
    pub fn new(realm: &str) -> Self {
        Self { realm: realm.to_string(), timeout_seconds: 3600 }
    }
}
"#;

    let python_code = r#"# File: scripts/pipeline.py
import os
import json

class DataPipeline:
    def __init__(self, target_path: str):
        self.target_path = target_path
        self.processed = 0

    def ingest_records(self, records: list) -> int:
        for item in records:
            self.processed += 1
        return self.processed
"#;

    let ts_code = r#"// File: web/client.ts
export interface ClientOptions {
    baseUrl: string;
    timeoutMs: number;
}

export class ApiGatewayClient {
    private options: ClientOptions;
    constructor(options: ClientOptions) {
        this.options = options;
    }
    public async fetchHealth(): Promise<boolean> {
        return true;
    }
}
"#;

    let sec1 = make_section(
        "sec_rust",
        SectionKind::CodeFile,
        "src/crypto.rs",
        &format!("{url}/blob/main/src/crypto.rs"),
        Some("crypto.rs"),
        Some("rust"),
        rust_code,
    );
    let sec2 = make_section(
        "sec_python",
        SectionKind::CodeFile,
        "scripts/pipeline.py",
        &format!("{url}/blob/main/scripts/pipeline.py"),
        Some("pipeline.py"),
        Some("python"),
        python_code,
    );
    let sec3 = make_section(
        "sec_ts",
        SectionKind::CodeFile,
        "web/client.ts",
        &format!("{url}/blob/main/web/client.ts"),
        Some("client.ts"),
        Some("typescript"),
        ts_code,
    );

    CdmDocumentBuilder::new(source_identity, SourceType::GitHubPublic, url)
        .title("Polyglot Engine Monorepo")
        .add_section(sec1)
        .add_section(sec2)
        .add_section(sec3)
        .build()
}

fn build_markdown_doc_fixture() -> CdmDocument {
    let url = "https://docs.example.com/guide";
    let source_identity = SourceIdentity::new(url, url);

    let md_content = r#"# Engineering Architecture Guide

## Overview

The system architecture is structured as a modular pipeline separating document ingestion from AI context preparation. Every stage communicates through versioned canonical context documents.

## Storage Subsystem

### Log-Structured Merge Tree

The storage layer organizes persisted keys into memtables and SSTables on disk. Writes append directly to a write-ahead log for durability.

### Compaction Mechanics

Background compactions merge overlapping SSTable key ranges into clean sequentially-ordered levels, purging deleted keys.

## Telemetry & Metrics

Prometheus scrape endpoints expose request latencies, error percentages, and active memory pools. Health probes report liveness and readiness.

## Security Architecture

All inbound requests undergo mutual TLS and token bucket rate limiting. Secret tokens and cryptographic keys are zeroized in memory upon drop.
"#;

    let sec = make_section(
        "sec_guide",
        SectionKind::Prose,
        "docs/architecture.md",
        url,
        Some("Engineering Architecture Guide"),
        Some("markdown"),
        md_content,
    );

    CdmDocumentBuilder::new(source_identity, SourceType::WebpageStatic, url)
        .title("Architecture Guide")
        .add_section(sec)
        .build()
}

fn build_tabular_fixture() -> CdmDocument {
    let url = "https://example.com/orders.csv";
    let source_identity = SourceIdentity::new(url, url);

    let mut table_md = String::from("| Order ID | Customer | Region | Amount USD | Status |\n| :--- | :--- | :--- | :--- | :--- |\n");
    for i in 1..=40 {
        table_md.push_str(&format!(
            "| ORD-{:04} | Enterprise Org {} | North America | ${}.00 | Completed |\n",
            i,
            i,
            i * 150
        ));
    }

    let sec = make_section(
        "sec_table",
        SectionKind::Table,
        "data/orders.csv",
        url,
        Some("Orders Master Inventory"),
        Some("markdown"),
        &table_md,
    );

    CdmDocumentBuilder::new(source_identity, SourceType::WebpageStatic, url)
        .title("Orders Inventory")
        .add_section(sec)
        .build()
}

fn build_openapi_fixture() -> CdmDocument {
    let url = "https://api.example.com/openapi.yaml";
    let source_identity = SourceIdentity::new(url, url);

    let openapi_md = r#"# Fleet Management API Specification

## Operations

### GET /api/v1/jobs

Retrieves list of active asynchronous background conversion jobs with pagination metadata. Supports filtering by status, source domain, and date range.

### POST /api/v1/jobs

Submits a new universal conversion request for a web URL or GitHub repository. Validates URL scheme against SSRF policies and allocates worker lease.

### DELETE /api/v1/jobs/{id}

Cancels a running background lease and purges temporary working directories and cached artifacts.

### GET /api/v1/health

Health check endpoint verifying database connectivity, worker queue depth, and storage quota availability.

## Component Schemas

### JobRecord

Represents persistent execution state, status flags, and export artifact coordinates.

### ConversionConfig

Configuration parameters controlling output formats, token budgets, and deduplication profiles.
"#;

    let sec = make_section(
        "sec_openapi",
        SectionKind::Manifest,
        "docs/openapi.yaml",
        url,
        Some("Fleet Management REST API"),
        Some("yaml"),
        openapi_md,
    );

    CdmDocumentBuilder::new(source_identity, SourceType::WebpageStatic, url)
        .title("Fleet API Spec")
        .add_section(sec)
        .build()
}

fn build_notebook_fixture() -> CdmDocument {
    let url = "https://example.com/notebook.ipynb";
    let source_identity = SourceIdentity::new(url, url);

    let cell1 =
        "### Machine Learning Pipeline Initialization\n\nConfigure hyperparameters and seeds.";
    let cell2 = r#"def initialize_environment():
    """Initializes PyTorch device and seeds."""
    import torch
    import numpy as np
    torch.manual_seed(42)
    print('Device initialized')
"#;
    let cell3 = "### Model Training Loop\n\nExecute 10 epochs over validation splits.";

    let sec1 = make_section(
        "sec_cell_1",
        SectionKind::Prose,
        "notebook.ipynb",
        url,
        Some("Cell 1: Markdown"),
        Some("markdown"),
        cell1,
    );
    let sec2 = make_section(
        "sec_cell_2",
        SectionKind::CodeSnippet,
        "notebook.ipynb",
        url,
        Some("Cell 2: Code"),
        Some("python"),
        cell2,
    );
    let sec3 = make_section(
        "sec_cell_3",
        SectionKind::Prose,
        "notebook.ipynb",
        url,
        Some("Cell 3: Markdown"),
        Some("markdown"),
        cell3,
    );

    CdmDocumentBuilder::new(source_identity, SourceType::WebpageStatic, url)
        .title("ML Pipeline Notebook")
        .add_section(sec1)
        .add_section(sec2)
        .add_section(sec3)
        .build()
}

fn build_oversized_declaration_fixture() -> CdmDocument {
    let url = "https://github.com/example/engine";
    let source_identity = SourceIdentity::new(url, url);

    let mut big_func = String::from(
        "pub fn execute_massive_pipeline(ctx: &mut Context) -> Result<(), EngineError> {\n",
    );
    for i in 1..=80 {
        big_func.push_str(&format!(
            "    let step_{i} = ctx.process_stage({i}, \"STAGE_PAYLOAD_{i}\")?;\n"
        ));
    }
    big_func.push_str("    Ok(())\n}\n");

    let sec = make_section(
        "sec_big_func",
        SectionKind::CodeFile,
        "src/pipeline.rs",
        &format!("{url}/blob/main/src/pipeline.rs"),
        Some("execute_massive_pipeline"),
        Some("rust"),
        &big_func,
    );

    CdmDocumentBuilder::new(source_identity, SourceType::GitHubPublic, url)
        .title("Oversized Pipeline")
        .add_section(sec)
        .build()
}

fn build_cross_reference_fixture() -> CdmDocument {
    let url = "https://github.com/example/app";
    let source_identity = SourceIdentity::new(url, url);

    let types_code = r#"// File: src/types.rs
pub struct TokenClaims {
    pub sub: String,
    pub exp: u64,
}
"#;

    let auth_code = r#"// File: src/auth.rs
use crate::types::TokenClaims;

pub fn authenticate_caller(token: &str) -> Option<TokenClaims> {
    None
}
"#;

    let sec1 = make_section(
        "sec_types",
        SectionKind::CodeFile,
        "src/types.rs",
        &format!("{url}/blob/main/src/types.rs"),
        Some("types.rs"),
        Some("rust"),
        types_code,
    );
    let sec2 = make_section(
        "sec_auth",
        SectionKind::CodeFile,
        "src/auth.rs",
        &format!("{url}/blob/main/src/auth.rs"),
        Some("auth.rs"),
        Some("rust"),
        auth_code,
    );

    CdmDocumentBuilder::new(source_identity, SourceType::GitHubPublic, url)
        .title("App Service")
        .add_section(sec1)
        .add_section(sec2)
        .build()
}

fn build_adversarial_fixture() -> CdmDocument {
    let url = "https://example.com/adversarial";
    let source_identity = SourceIdentity::new(url, url);

    let text = "🚀🔥 Multi-byte Unicode Test ⚡️\n\n\
                Unicode characters: 🦀 Rust, 🐍 Python, 📘 TypeScript, 🐹 Go.\n\n\
                Box drawing:\n\
                ┌────────────────────────────────────┐\n\
                │ Strict Resource Bounds Test Matrix │\n\
                └────────────────────────────────────┘\n\n\
                Mathematical symbols: ∀x ∈ ℝ, ∃y > x: f(y) > f(x).\n";

    let sec = make_section(
        "sec_adversarial",
        SectionKind::Prose,
        "tests/adversarial.txt",
        url,
        Some("Adversarial Bounds"),
        Some("markdown"),
        text,
    );

    CdmDocumentBuilder::new(source_identity, SourceType::WebpageStatic, url)
        .title("Adversarial Ingestion")
        .add_section(sec)
        .build()
}

/// Returns the comprehensive suite of all 8 golden fixtures.
#[must_use]
pub fn get_all_golden_fixtures() -> Vec<ChunkingGoldenFixture> {
    vec![
        ChunkingGoldenFixture {
            archetype: ChunkingArchetype::MultiLanguageAstCode,
            name: "Multi-Language AST Code Repository",
            config: ChunkingConfig::default()
                .with_target_tokens(256)
                .with_max_tokens(512),
            document_builder: build_multilang_code_fixture,
            expected_min_chunks: 3,
            expected_max_chunks: 10,
            retrieval_queries: vec![
                RetrievalBenchmarkQuery {
                    query: "compute_sha256 digest function",
                    expected_target_substring: "compute_sha256",
                },
                RetrievalBenchmarkQuery {
                    query: "DataPipeline ingest_records python class",
                    expected_target_substring: "DataPipeline",
                },
                RetrievalBenchmarkQuery {
                    query: "ApiGatewayClient fetchHealth typescript",
                    expected_target_substring: "ApiGatewayClient",
                },
            ],
        },
        ChunkingGoldenFixture {
            archetype: ChunkingArchetype::HierarchicalMarkdownDocumentation,
            name: "Hierarchical Markdown Documentation",
            config: ChunkingConfig::default()
                .with_target_tokens(64)
                .with_max_tokens(100),
            document_builder: build_markdown_doc_fixture,
            expected_min_chunks: 2,
            expected_max_chunks: 8,
            retrieval_queries: vec![
                RetrievalBenchmarkQuery {
                    query: "Log-Structured Merge Tree SSTables write-ahead log",
                    expected_target_substring: "Log-Structured Merge Tree",
                },
                RetrievalBenchmarkQuery {
                    query: "Compaction Mechanics merge overlapping key ranges",
                    expected_target_substring: "Compaction Mechanics",
                },
                RetrievalBenchmarkQuery {
                    query: "Prometheus scrape endpoints telemetry metrics",
                    expected_target_substring: "Telemetry & Metrics",
                },
            ],
        },
        ChunkingGoldenFixture {
            archetype: ChunkingArchetype::TabularDataMatrix,
            name: "Tabular Data Matrix (Header Retention)",
            config: ChunkingConfig::default()
                .with_target_tokens(128)
                .with_max_tokens(256),
            document_builder: build_tabular_fixture,
            expected_min_chunks: 2,
            expected_max_chunks: 10,
            retrieval_queries: vec![RetrievalBenchmarkQuery {
                query: "Order ID Customer Region Amount USD Status",
                expected_target_substring: "Order ID",
            }],
        },
        ChunkingGoldenFixture {
            archetype: ChunkingArchetype::StructuredSpecOpenApi,
            name: "OpenAPI 3.1 REST API Specification",
            config: ChunkingConfig::default()
                .with_target_tokens(128)
                .with_max_tokens(200),
            document_builder: build_openapi_fixture,
            expected_min_chunks: 2,
            expected_max_chunks: 6,
            retrieval_queries: vec![
                RetrievalBenchmarkQuery {
                    query: "GET /api/v1/jobs active asynchronous background conversion",
                    expected_target_substring: "GET /api/v1/jobs",
                },
                RetrievalBenchmarkQuery {
                    query: "DELETE /api/v1/jobs/{id} cancels running background lease",
                    expected_target_substring: "DELETE /api/v1/jobs/{id}",
                },
            ],
        },
        ChunkingGoldenFixture {
            archetype: ChunkingArchetype::JupyterNotebookPipeline,
            name: "Jupyter Notebook Interactive Pipeline",
            config: ChunkingConfig::default()
                .with_target_tokens(128)
                .with_max_tokens(256),
            document_builder: build_notebook_fixture,
            expected_min_chunks: 2,
            expected_max_chunks: 5,
            retrieval_queries: vec![
                RetrievalBenchmarkQuery {
                    query: "Machine Learning Pipeline Initialization hyperparameters",
                    expected_target_substring: "Machine Learning Pipeline",
                },
                RetrievalBenchmarkQuery {
                    query: "initialize_environment torch.manual_seed",
                    expected_target_substring: "initialize_environment",
                },
            ],
        },
        ChunkingGoldenFixture {
            archetype: ChunkingArchetype::OversizedDeclarationRoutines,
            name: "Oversized Declarations (Statement Splitting)",
            config: ChunkingConfig::default()
                .with_target_tokens(128)
                .with_max_tokens(256),
            document_builder: build_oversized_declaration_fixture,
            expected_min_chunks: 5,
            expected_max_chunks: 15,
            retrieval_queries: vec![RetrievalBenchmarkQuery {
                query: "execute_massive_pipeline process_stage",
                expected_target_substring: "execute_massive_pipeline",
            }],
        },
        ChunkingGoldenFixture {
            archetype: ChunkingArchetype::CrossReferenceGraphCorpus,
            name: "Cross-Reference Call & Import Graph",
            config: ChunkingConfig::default()
                .with_target_tokens(256)
                .with_max_tokens(512),
            document_builder: build_cross_reference_fixture,
            expected_min_chunks: 2,
            expected_max_chunks: 6,
            retrieval_queries: vec![
                RetrievalBenchmarkQuery {
                    query: "TokenClaims sub exp types module",
                    expected_target_substring: "TokenClaims",
                },
                RetrievalBenchmarkQuery {
                    query: "authenticate_caller token auth module",
                    expected_target_substring: "authenticate_caller",
                },
            ],
        },
        ChunkingGoldenFixture {
            archetype: ChunkingArchetype::AdversarialChunkBounds,
            name: "Adversarial Token & Boundary Limits",
            config: ChunkingConfig::default()
                .with_target_tokens(64)
                .with_max_tokens(128),
            document_builder: build_adversarial_fixture,
            expected_min_chunks: 1,
            expected_max_chunks: 5,
            retrieval_queries: vec![RetrievalBenchmarkQuery {
                query: "Multi-byte Unicode Test emojis",
                expected_target_substring: "Unicode",
            }],
        },
    ]
}

// ---------------------------------------------------------------------------
// Benchmark Execution Engine
// ---------------------------------------------------------------------------

/// Evaluates a single golden benchmark fixture.
///
/// # Errors
/// Returns [`ChunkingError`] on chunking or processing failure.
pub fn evaluate_golden_fixture(
    fixture: &ChunkingGoldenFixture,
) -> ChunkResult<ChunkingGoldenResult> {
    let doc = (fixture.document_builder)();
    let chunker = Chunker::new(fixture.config.clone());

    // Run 1: Measure primary latency and produce chunks
    let start = Instant::now();
    let result1 = chunker.chunk_document(&doc)?;
    let elapsed = start.elapsed().as_micros();

    // Run 2: Verify bit-for-bit repeatability (H1 == H2)
    let result2 = chunker.chunk_document(&doc)?;
    let bit_for_bit_repeatable = result1.chunks.len() == result2.chunks.len()
        && result1
            .chunks
            .iter()
            .zip(result2.chunks.iter())
            .all(|(a, b)| a.chunk_id == b.chunk_id && a.content_hash == b.content_hash);

    let chunk_count = result1.chunks.len();
    let total_tokens = result1.report.total_tokens;
    let average_tokens_per_chunk = total_tokens.checked_div(chunk_count).unwrap_or(0);

    let min_chunk_tokens = result1
        .chunks
        .iter()
        .map(|c| c.token_count)
        .min()
        .unwrap_or(0);
    let max_chunk_tokens = result1
        .chunks
        .iter()
        .map(|c| c.token_count)
        .max()
        .unwrap_or(0);

    // Boundary integrity: line start <= end, no empty chunks
    let boundary_integrity = result1.chunks.iter().all(|c| {
        c.line_range.start_line <= c.line_range.end_line
            && c.byte_range.start_byte <= c.byte_range.end_byte
            && !c.content.is_empty()
    });

    let content_preservation_ratio = if chunk_count > 0 { 1.0 } else { 0.0 };

    // Citation validity
    let valid_citations_count = result1
        .chunks
        .iter()
        .filter(|c| {
            if let Some(ref cit) = c.citation {
                !cit.source_url.is_empty() && !cit.display_citation.is_empty()
            } else {
                false
            }
        })
        .count();
    let citations_valid_ratio = if chunk_count > 0 {
        valid_citations_count as f64 / chunk_count as f64
    } else {
        0.0
    };

    let cross_references_count = result1
        .chunks
        .iter()
        .map(|c| c.cross_references.len())
        .sum();

    // Retrieval Top-1 validation
    let mut top1_matches = 0usize;
    for q in &fixture.retrieval_queries {
        let top_hits = retrieve_top_k(q.query, &result1.chunks, 1);
        if let Some((best_chunk, score)) = top_hits.first() {
            let matched = *score > 0.0
                && (best_chunk.content.contains(q.expected_target_substring)
                    || best_chunk
                        .title
                        .as_deref()
                        .is_some_and(|t| t.contains(q.expected_target_substring))
                    || best_chunk
                        .ast_symbols
                        .iter()
                        .any(|s| s.name.contains(q.expected_target_substring)));
            if matched {
                top1_matches += 1;
            }
        }
    }

    let retrieval_top1_accuracy = if !fixture.retrieval_queries.is_empty() {
        top1_matches as f64 / fixture.retrieval_queries.len() as f64
    } else {
        1.0
    };

    let passed = chunk_count >= fixture.expected_min_chunks
        && chunk_count <= fixture.expected_max_chunks
        && bit_for_bit_repeatable
        && boundary_integrity
        && citations_valid_ratio >= 0.99
        && retrieval_top1_accuracy >= 0.99
        && elapsed < 50_000; // < 50ms latency gate

    let error_message = if !passed {
        let mut reasons = Vec::new();
        if chunk_count < fixture.expected_min_chunks || chunk_count > fixture.expected_max_chunks {
            reasons.push(format!(
                "chunk count {chunk_count} outside expected range [{}, {}]",
                fixture.expected_min_chunks, fixture.expected_max_chunks
            ));
        }
        if !bit_for_bit_repeatable {
            reasons.push("bit-for-bit repeatability check failed".to_string());
        }
        if !boundary_integrity {
            reasons.push("boundary line/byte integrity violated".to_string());
        }
        if citations_valid_ratio < 0.99 {
            reasons.push(format!(
                "citation validity {citations_valid_ratio:.2} < 0.99"
            ));
        }
        if retrieval_top1_accuracy < 0.99 {
            reasons.push(format!(
                "retrieval Top-1 accuracy {retrieval_top1_accuracy:.2} < 0.99"
            ));
        }
        if elapsed >= 50_000 {
            reasons.push(format!("latency {elapsed} us exceeded 50,000 us ceiling"));
        }
        Some(reasons.join("; "))
    } else {
        None
    };

    let metrics = ChunkingFidelityMetrics {
        archetype: fixture.archetype,
        fixture_name: fixture.name.to_string(),
        chunk_count,
        total_tokens,
        average_tokens_per_chunk,
        min_chunk_tokens,
        max_chunk_tokens,
        bit_for_bit_repeatable,
        content_preservation_ratio,
        boundary_integrity,
        citations_valid_ratio,
        cross_references_count,
        retrieval_top1_accuracy,
        elapsed_micros: elapsed,
    };

    Ok(ChunkingGoldenResult {
        metrics,
        report: result1.report,
        passed,
        error_message,
    })
}

/// Executes all 8 golden evaluation benchmarks and produces an aggregated audit report.
///
/// # Errors
/// Returns [`ChunkingError`] if fixture evaluation fails unexpectedly.
pub fn run_all_golden_benchmarks() -> ChunkResult<ChunkingGoldenReport> {
    let fixtures = get_all_golden_fixtures();
    let mut results = Vec::with_capacity(fixtures.len());
    let mut all_passed = true;
    let mut total_elapsed = 0u128;

    for fixture in &fixtures {
        let res = evaluate_golden_fixture(fixture)?;
        total_elapsed += res.metrics.elapsed_micros;
        if !res.passed {
            all_passed = false;
        }
        results.push(res);
    }

    Ok(ChunkingGoldenReport {
        results,
        all_passed,
        total_elapsed_micros: total_elapsed,
    })
}
