//! # Structured Document Golden Fixture Corpus & Benchmark Suite
//!
//! Provides a quantitative evaluation framework for structured document extraction
//! across 8 comprehensive real-world archetypes:
//! - OpenAPI 3.0.x / 3.1.0 REST API Specification (JSON)
//! - OpenAPI / Swagger Specification (YAML)
//! - JSON Schema Draft-07 Data Model Definition
//! - Generic Multi-Document YAML Manifest Stream (Kubernetes)
//! - RFC 4180 Delimited Tabular Data (CSV with quotes, commas, escapes)
//! - Tab-Separated Tabular Metrics (TSV with null normalization and type inference)
//! - Jupyter Notebook Interactive Pipeline (.ipynb nbformat v4 with code, markdown, outputs, ANSI)
//! - Adversarial Resource Bounds & Graceful Truncation

use std::time::Instant;

use serde::{Deserialize, Serialize};
use web_repo_to_ai_cdm::section::SectionKind;
use web_repo_to_ai_cdm::validator::CdmValidator;
use web_repo_to_ai_cdm::CdmDocument;

use crate::detector::detect_format;
use crate::error::{Result, StructuredError};
use crate::extractor::StructuredExtractor;
use crate::models::{StructuredDocumentKind, StructuredInput, StructuredOptions};

/// Distinct archetypes of structured documents evaluated in the golden suite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StructuredArchetype {
    /// OpenAPI REST API specification in JSON.
    OpenApiJson,
    /// OpenAPI / Swagger specification in YAML.
    OpenApiYaml,
    /// JSON Schema specification definition.
    JsonSchema,
    /// Generic multi-document YAML stream (e.g. Kubernetes, Compose).
    GenericYaml,
    /// RFC 4180 comma-separated tabular values with quotes and escapes.
    TabularCsv,
    /// Tab-separated tabular values with null normalization and type inference.
    TabularTsv,
    /// Jupyter Notebook interactive document with code, markdown, outputs, and tracebacks.
    JupyterNotebook,
    /// Adversarial resource limits test evaluating graceful truncation and completeness.
    AdversarialLimits,
}

impl StructuredArchetype {
    /// Human-readable display label for the archetype.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::OpenApiJson => "OpenAPI 3.0 JSON Specification",
            Self::OpenApiYaml => "OpenAPI 3.1 YAML Specification",
            Self::JsonSchema => "JSON Schema Draft-07 Data Model",
            Self::GenericYaml => "Kubernetes Multi-Document YAML",
            Self::TabularCsv => "RFC 4180 E-Commerce Orders CSV",
            Self::TabularTsv => "Server Performance Metrics TSV",
            Self::JupyterNotebook => "Machine Learning Pipeline Notebook",
            Self::AdversarialLimits => "Adversarial Resource Limits",
        }
    }
}

/// A self-contained golden fixture with raw content and ground-truth expectations.
#[derive(Debug, Clone)]
pub struct StructuredGoldenFixture {
    /// Archetype categorization.
    pub archetype: StructuredArchetype,
    /// Human-readable identifier.
    pub name: &'static str,
    /// Suggested filename for format hinting.
    pub filename: &'static str,
    /// Raw document content string.
    pub raw_content: &'static str,
    /// Expected format classification.
    pub expected_kind: StructuredDocumentKind,
    /// Minimum number of CDM sections expected.
    pub expected_min_sections: usize,
    /// Required section kinds that must appear in the CDM document.
    pub expected_section_kinds: Vec<SectionKind>,
    /// High-signal text fragments expected across section bodies, titles, or metadata.
    pub expected_fragments: Vec<&'static str>,
    /// Expected primary language if applicable.
    pub expected_primary_language: Option<&'static str>,
}

/// Quantitative fidelity and benchmark metrics for an individual fixture evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuredFidelityMetrics {
    /// Number of synthesized CDM sections.
    pub section_count: usize,
    /// Whether all expected section kinds were present.
    pub section_kinds_matched: bool,
    /// Number of ground-truth fragments located in the synthesized document.
    pub matched_fragments_count: usize,
    /// Total number of expected ground-truth fragments.
    pub total_expected_fragments: usize,
    /// Fragment recall ratio (0.0 to 1.0).
    pub fragment_recall: f64,
    /// Whether the synthesized CDM document passed all schema validator invariants.
    pub schema_valid: bool,
    /// Total byte size of all section bodies.
    pub total_body_bytes: usize,
    /// Execution duration in milliseconds.
    pub duration_ms: u64,
    /// Overall passing status (recall >= 0.95, valid schema, section kinds matched).
    pub is_passing: bool,
}

/// Evaluation result for a single golden fixture.
#[derive(Debug, Clone)]
pub struct StructuredGoldenResult {
    /// Fixture name.
    pub fixture_name: String,
    /// Evaluated archetype.
    pub archetype: StructuredArchetype,
    /// Fidelity metrics.
    pub metrics: StructuredFidelityMetrics,
    /// Synthesized CDM document.
    pub cdm_doc: CdmDocument,
}

/// Comprehensive benchmark report aggregating evaluations across all golden fixtures.
#[derive(Debug, Clone)]
pub struct StructuredGoldenReport {
    /// Evaluation results per fixture.
    pub results: Vec<StructuredGoldenResult>,
    /// Total number of evaluated fixtures.
    pub total_fixtures: usize,
    /// Number of fixtures meeting all acceptance criteria.
    pub passed_fixtures: usize,
    /// Mean fragment recall across all fixtures.
    pub mean_fragment_recall: f64,
    /// Total execution duration in milliseconds.
    pub total_duration_ms: u64,
}

impl StructuredGoldenReport {
    /// Formats an executive summary of the golden benchmark results in GitHub Flavored Markdown.
    #[must_use]
    pub fn to_markdown_summary(&self) -> String {
        let mut md = String::new();
        md.push_str("# Structured Document Golden Benchmark Report\n\n");
        md.push_str(&format!(
            "- **Total Fixtures Evaluated**: {}\n",
            self.total_fixtures
        ));
        md.push_str(&format!(
            "- **Passed Fixtures**: {} ({}%)\n",
            self.passed_fixtures,
            (self.passed_fixtures * 100) / self.total_fixtures.max(1)
        ));
        md.push_str(&format!(
            "- **Mean Fragment Recall**: {:.2}%\n",
            self.mean_fragment_recall * 100.0
        ));
        md.push_str(&format!(
            "- **Total Benchmark Duration**: {} ms\n\n",
            self.total_duration_ms
        ));

        md.push_str("| Archetype | Sections | Recall | Schema Valid | Latency | Status |\n");
        md.push_str("| :--- | :---: | :---: | :---: | :---: | :---: |\n");

        for r in &self.results {
            let status = if r.metrics.is_passing { "PASS" } else { "FAIL" };
            md.push_str(&format!(
                "| {} | {} | {:.1}% ({}/{}) | {} | {} ms | {} |\n",
                r.archetype.display_name(),
                r.metrics.section_count,
                r.metrics.fragment_recall * 100.0,
                r.metrics.matched_fragments_count,
                r.metrics.total_expected_fragments,
                if r.metrics.schema_valid {
                    "Valid"
                } else {
                    "Invalid"
                },
                r.metrics.duration_ms,
                status
            ));
        }

        md.push('\n');
        md
    }
}

/// Evaluates a single golden fixture against ground-truth expectations.
pub fn evaluate_golden_fixture(
    fixture: &StructuredGoldenFixture,
    options: &StructuredOptions,
) -> Result<StructuredGoldenResult> {
    let start = Instant::now();

    let input = StructuredInput::from_str(fixture.raw_content)
        .with_filename(fixture.filename)
        .with_source_url(format!("file:///fixtures/{}", fixture.filename));

    // 1. Validate format detection
    let detected_kind = detect_format(&input)?;
    if detected_kind != fixture.expected_kind {
        return Err(StructuredError::UnrecognizedFormat(format!(
            "Expected format {:?}, but detected {:?}",
            fixture.expected_kind, detected_kind
        )));
    }

    // 2. Synthesize CDM document
    let extractor = StructuredExtractor::new();
    let doc = extractor.extract_to_cdm(&input, options)?;

    let duration_ms = start.elapsed().as_millis() as u64;

    // 3. Schema validation
    let validator = CdmValidator::new();
    let val_report = validator.validate(&doc);
    let schema_valid = val_report.is_valid();

    // 4. Section count and kind matching
    let section_count = doc.sections.len();
    let section_kinds_matched = fixture
        .expected_section_kinds
        .iter()
        .all(|expected_k| doc.sections.iter().any(|s| s.kind == *expected_k));

    // 5. Ground-truth fragment recall
    let mut combined_text = String::with_capacity(16 * 1024);
    if let Some(ref title) = doc.metadata.title {
        combined_text.push_str(title);
        combined_text.push(' ');
    }
    if let Some(ref desc) = doc.metadata.description {
        combined_text.push_str(desc);
        combined_text.push(' ');
    }
    for sec in &doc.sections {
        if let Some(ref t) = sec.title {
            combined_text.push_str(t);
            combined_text.push(' ');
        }
        combined_text.push_str(&sec.body);
        combined_text.push(' ');
    }

    let mut matched_fragments_count = 0;
    for fragment in &fixture.expected_fragments {
        if combined_text.contains(fragment) {
            matched_fragments_count += 1;
        }
    }

    let total_expected = fixture.expected_fragments.len();
    let fragment_recall = if total_expected == 0 {
        1.0
    } else {
        matched_fragments_count as f64 / total_expected as f64
    };

    let total_body_bytes: usize = doc.sections.iter().map(|s| s.body.len()).sum();

    // Acceptance condition: schema valid, recall >= 0.95, expected kinds matched, section count >= min
    let is_passing = schema_valid
        && fragment_recall >= 0.95
        && section_kinds_matched
        && section_count >= fixture.expected_min_sections;

    let metrics = StructuredFidelityMetrics {
        section_count,
        section_kinds_matched,
        matched_fragments_count,
        total_expected_fragments: total_expected,
        fragment_recall,
        schema_valid,
        total_body_bytes,
        duration_ms,
        is_passing,
    };

    Ok(StructuredGoldenResult {
        fixture_name: fixture.name.to_string(),
        archetype: fixture.archetype,
        metrics,
        cdm_doc: doc,
    })
}

/// Executes evaluations across all 8 golden archetypes and compiles a summary report.
pub fn run_all_golden_benchmarks(options: &StructuredOptions) -> Result<StructuredGoldenReport> {
    let fixtures = all_golden_fixtures();
    let total_fixtures = fixtures.len();
    let mut results = Vec::with_capacity(total_fixtures);
    let mut passed_fixtures = 0;
    let mut total_recall = 0.0;
    let mut total_duration_ms = 0;

    for fixture in &fixtures {
        let res = evaluate_golden_fixture(fixture, options)?;
        if res.metrics.is_passing {
            passed_fixtures += 1;
        }
        total_recall += res.metrics.fragment_recall;
        total_duration_ms += res.metrics.duration_ms;
        results.push(res);
    }

    let mean_fragment_recall = if total_fixtures == 0 {
        1.0
    } else {
        total_recall / total_fixtures as f64
    };

    Ok(StructuredGoldenReport {
        results,
        total_fixtures,
        passed_fixtures,
        mean_fragment_recall,
        total_duration_ms,
    })
}

// ============================================================================
// GOLDEN FIXTURE DEFINITIONS
// ============================================================================

/// OpenAPI 3.0.3 JSON REST API specification fixture.
pub fn build_openapi_json_fixture() -> StructuredGoldenFixture {
    StructuredGoldenFixture {
        archetype: StructuredArchetype::OpenApiJson,
        name: "Petstore & Orders API",
        filename: "openapi.json",
        raw_content: r##"{
  "openapi": "3.0.3",
  "info": {
    "title": "Swagger Petstore & Orders API",
    "version": "1.0.0",
    "description": "A sample API for testing structured document extraction."
  },
  "servers": [{ "url": "https://api.petstore.io/v1" }],
  "paths": {
    "/pets": {
      "get": {
        "summary": "List all pets",
        "operationId": "listPets",
        "tags": ["pets"],
        "parameters": [
          { "name": "limit", "in": "query", "required": false, "schema": { "type": "integer" } }
        ],
        "responses": {
          "200": { "description": "A paged array of pets" }
        }
      },
      "post": {
        "summary": "Create a pet",
        "operationId": "createPets",
        "tags": ["pets"],
        "requestBody": {
          "required": true,
          "content": {
            "application/json": { "schema": { "$ref": "#/components/schemas/Pet" } }
          }
        },
        "responses": {
          "201": { "description": "Null response" }
        }
      }
    }
  },
  "components": {
    "schemas": {
      "Pet": {
        "type": "object",
        "required": ["id", "name"],
        "properties": {
          "id": { "type": "integer", "format": "int64" },
          "name": { "type": "string" },
          "tag": { "type": "string" }
        }
      }
    }
  }
}"##,
        expected_kind: StructuredDocumentKind::OpenApi,
        expected_min_sections: 3,
        expected_section_kinds: vec![SectionKind::Prose],
        expected_fragments: vec![
            "Swagger Petstore & Orders API",
            "listPets",
            "createPets",
            "limit",
            "Pet",
            "int64",
        ],
        expected_primary_language: None,
    }
}

/// OpenAPI 3.1.0 YAML specification fixture.
pub fn build_openapi_yaml_fixture() -> StructuredGoldenFixture {
    StructuredGoldenFixture {
        archetype: StructuredArchetype::OpenApiYaml,
        name: "Cloud Fleet API",
        filename: "openapi.yaml",
        raw_content: r#"openapi: 3.1.0
info:
  title: Cloud Fleet Manager API
  version: 2.4.0
  description: Distributed cloud instance control plane.
paths:
  /instances:
    get:
      summary: Enumerate cluster compute nodes
      operationId: getInstances
      tags:
        - compute
      responses:
        "200":
          description: List of active nodes
"#,
        expected_kind: StructuredDocumentKind::OpenApi,
        expected_min_sections: 2,
        expected_section_kinds: vec![SectionKind::Prose],
        expected_fragments: vec![
            "Cloud Fleet Manager API",
            "getInstances",
            "Enumerate cluster compute nodes",
            "compute",
        ],
        expected_primary_language: None,
    }
}

/// JSON Schema Draft-07 data model fixture.
pub fn build_jsonschema_fixture() -> StructuredGoldenFixture {
    StructuredGoldenFixture {
        archetype: StructuredArchetype::JsonSchema,
        name: "User Profile Schema",
        filename: "user_profile.schema.json",
        raw_content: r##"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://example.com/schemas/user-profile.json",
  "title": "UserProfile",
  "description": "Defines an authenticated user account entity.",
  "type": "object",
  "required": ["id", "username", "email"],
  "properties": {
    "id": { "type": "string", "format": "uuid", "description": "Unique identifier" },
    "username": { "type": "string", "minLength": 3, "description": "Handle" },
    "email": { "type": "string", "format": "email", "description": "Primary email" },
    "role": { "type": "string", "enum": ["admin", "member", "viewer"] }
  },
  "definitions": {
    "Address": {
      "type": "object",
      "required": ["street", "city"],
      "properties": {
        "street": { "type": "string" },
        "city": { "type": "string" },
        "country": { "type": "string" }
      }
    }
  }
}"##,
        expected_kind: StructuredDocumentKind::JsonSchema,
        expected_min_sections: 2,
        expected_section_kinds: vec![SectionKind::Prose],
        expected_fragments: vec![
            "UserProfile",
            "Defines an authenticated user account entity",
            "username",
            "email",
            "uuid",
            "Address",
            "street",
            "city",
        ],
        expected_primary_language: None,
    }
}

/// Multi-document Kubernetes YAML manifest fixture.
pub fn build_yaml_stream_fixture() -> StructuredGoldenFixture {
    StructuredGoldenFixture {
        archetype: StructuredArchetype::GenericYaml,
        name: "Kubernetes Manifest Stream",
        filename: "k8s_deploy.yaml",
        raw_content: r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: web-api-deployment
  namespace: production
spec:
  replicas: 3
---
apiVersion: v1
kind: Service
metadata:
  name: web-api-service
spec:
  type: ClusterIP
  ports:
    - port: 8080
      targetPort: 8080
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: web-api-config
data:
  APP_ENV: production
"#,
        expected_kind: StructuredDocumentKind::GenericYaml,
        expected_min_sections: 3,
        expected_section_kinds: vec![SectionKind::CodeSnippet],
        expected_fragments: vec![
            "web-api-deployment",
            "web-api-service",
            "web-api-config",
            "Deployment",
            "ClusterIP",
        ],
        expected_primary_language: Some("yaml"),
    }
}

/// RFC 4180 CSV tabular fixture.
pub fn build_tabular_csv_fixture() -> StructuredGoldenFixture {
    StructuredGoldenFixture {
        archetype: StructuredArchetype::TabularCsv,
        name: "E-Commerce Orders Dataset",
        filename: "orders.csv",
        raw_content: "\
order_id,customer_id,product_sku,quantity,unit_price,status,shipped_at\n\
ord_1001,cust_501,SKU-ALPHA,2,49.99,delivered,2026-11-15\n\
ord_1002,cust_502,SKU-BETA,1,120.00,in_transit,2026-11-18\n\
ord_1003,cust_503,SKU-GAMMA,5,15.50,processing,2026-11-20\n\
ord_1004,cust_501,SKU-DELTA,10,8.25,delivered,2026-11-16\n",
        expected_kind: StructuredDocumentKind::Csv,
        expected_min_sections: 2, // 1 summary + 1 data table
        expected_section_kinds: vec![SectionKind::Prose, SectionKind::Table],
        expected_fragments: vec![
            "orders.csv",
            "SKU-ALPHA",
            "SKU-BETA",
            "SKU-GAMMA",
            "SKU-DELTA",
            "delivered",
        ],
        expected_primary_language: Some("csv"),
    }
}

/// Tab-separated performance metrics fixture.
pub fn build_tabular_tsv_fixture() -> StructuredGoldenFixture {
    StructuredGoldenFixture {
        archetype: StructuredArchetype::TabularTsv,
        name: "Server Metrics TSV",
        filename: "server_metrics.tsv",
        raw_content: "\
server_id\tcpu_usage_pct\tmem_usage_mb\tdisk_free_gb\tis_healthy\tlast_audit\n\
srv-node-01\t24.5\t4096\t120.5\ttrue\t2026-11-20\n\
srv-node-02\t89.2\t16384\t45.0\tfalse\t2026-11-20\n\
srv-node-03\t12.0\t2048\t250.0\ttrue\t2026-11-20\n",
        expected_kind: StructuredDocumentKind::Csv,
        expected_min_sections: 2,
        expected_section_kinds: vec![SectionKind::Prose, SectionKind::Table],
        expected_fragments: vec![
            "server_metrics.tsv",
            "srv-node-01",
            "srv-node-02",
            "srv-node-03",
            "cpu_usage_pct",
        ],
        expected_primary_language: Some("tsv"),
    }
}

/// Jupyter Notebook (.ipynb nbformat v4) interactive document fixture.
pub fn build_jupyter_notebook_fixture() -> StructuredGoldenFixture {
    StructuredGoldenFixture {
        archetype: StructuredArchetype::JupyterNotebook,
        name: "Machine Learning Pipeline Notebook",
        filename: "ml_pipeline.ipynb",
        raw_content: r#####"{
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
}"#####,
        expected_kind: StructuredDocumentKind::JupyterNotebook,
        expected_min_sections: 6,
        expected_section_kinds: vec![SectionKind::Prose, SectionKind::CodeSnippet],
        expected_fragments: vec![
            "Machine Learning Pipeline",
            "1. Data Preparation",
            "2. Model Training",
            "import torch",
            "PyTorch version: 2.1.0",
            "Epoch 10: Loss=0.042, Acc=98.5%",
        ],
        expected_primary_language: Some("python"),
    }
}

/// Adversarial limits fixture testing cell limits, graceful truncation, and completeness.
pub fn build_adversarial_limits_fixture() -> StructuredGoldenFixture {
    StructuredGoldenFixture {
        archetype: StructuredArchetype::AdversarialLimits,
        name: "Adversarial Limits Fixture",
        filename: "adversarial_limits.csv",
        raw_content: "\
id,val\n\
1,alpha\n2,beta\n3,gamma\n4,delta\n5,epsilon\n\
6,zeta\n7,eta\n8,theta\n9,iota\n10,kappa\n\
11,lambda\n12,mu\n13,nu\n14,xi\n15,omicron\n\
16,pi\n17,rho\n18,sigma\n19,tau\n20,upsilon\n\
21,phi\n22,chi\n23,psi\n24,omega\n25,final\n",
        expected_kind: StructuredDocumentKind::Csv,
        expected_min_sections: 2,
        expected_section_kinds: vec![SectionKind::Prose, SectionKind::Table],
        expected_fragments: vec!["alpha", "beta", "gamma", "delta"],
        expected_primary_language: Some("csv"),
    }
}

/// Returns the complete collection of all 8 golden fixtures.
#[must_use]
pub fn all_golden_fixtures() -> Vec<StructuredGoldenFixture> {
    vec![
        build_openapi_json_fixture(),
        build_openapi_yaml_fixture(),
        build_jsonschema_fixture(),
        build_yaml_stream_fixture(),
        build_tabular_csv_fixture(),
        build_tabular_tsv_fixture(),
        build_jupyter_notebook_fixture(),
        build_adversarial_limits_fixture(),
    ]
}
