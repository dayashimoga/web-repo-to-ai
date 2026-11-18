//! # Golden Comparison Testing Engine (`web-repo-to-ai-context-optimizer::golden`)
//!
//! Provides deterministic golden repository fixtures and cross-profile comparative benchmarking:
//! - Multi-language golden repository CDM fixture ([`GoldenRepositoryFixture`])
//! - Cross-profile comparative evaluation ([`GoldenProfileComparator`])
//! - Unified comparative matrix and report ([`GoldenComparisonReport`])
//! - Comprehensive profile invariant verification and diagnostic rationales

use crate::budget::ContextBudget;
use crate::compression::{CompressionEvaluator, CompressionReport};
use crate::error::ContextOptimizerError;
use crate::exclusions::{ExclusionExplainer, ExclusionReport};
use crate::optimizer::ContextBudgetOptimizer;
use crate::preservation::{PreservationEvaluator, PreservationReport};
use crate::profiles::{
    CodeReviewProfile, ContextProfile, DebuggingConfig, DebuggingProfile, GeneralAiProfile,
    ProfileKind, ProfileMetrics, SecurityReviewProfile,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use web_repo_to_ai_cdm::{
    ByteRange, CdmDocument, CdmSection, CompletenessReport, DocumentMetadata, ExtractionStatus,
    HierarchyNode, LineRange, RevisionInfo, SectionKind, SectionTokenMetrics, SourceIdentity,
    SourceType,
};
use web_repo_to_ai_core::DocumentId;
use web_repo_to_ai_tokenizer::{TargetModel, TokenizerAdapter, TokenizerRegistry};

/// Canonical golden repository fixture generator.
#[derive(Debug, Clone, Default)]
pub struct GoldenRepositoryFixture;

impl GoldenRepositoryFixture {
    /// Generates a realistic, multi-language golden repository [`CdmDocument`].
    ///
    /// The fixture models a production-grade transaction processing engine containing:
    /// - Manifests (`Cargo.toml`, `package.json`)
    /// - Architecture, troubleshooting, and security documentation
    /// - Core transaction engine implementation
    /// - Ingress API routes
    /// - Cryptographic vault and JWT authentication modules
    /// - Structured error hierarchies and distributed tracing instrumentation
    /// - Pull request feature code (`PaymentGateway`)
    /// - Diverse test suites (unit, security, negative, benchmark)
    /// - Cosmetic presentation assets and legacy changelogs
    #[must_use]
    pub fn create_default() -> CdmDocument {
        let registry = TokenizerRegistry::default();
        let adapter = registry.get_for_model(TargetModel::Gpt4o);

        let mut sections = Vec::new();
        let mut idx = 1;

        // 1. Cargo.toml (Manifest)
        let cargo_body = "[package]\n\
name = \"golden-repo-engine\"\n\
version = \"0.1.0\"\n\
edition = \"2021\"\n\n\
[dependencies]\n\
tokio = { version = \"1.38\", features = [\"full\"] }\n\
serde = { version = \"1.0\", features = [\"derive\"] }\n\
serde_json = \"1.0\"\n\
tracing = \"0.1\"\n\
thiserror = \"1.0\"\n\
jsonwebtoken = \"9.2\"\n\
aes-gcm = \"0.10\"\n";
        sections.push(Self::build_section(
            idx,
            "Cargo.toml",
            SectionKind::Manifest,
            "toml",
            cargo_body,
            vec![("manifest_type", "cargo")],
            adapter.as_deref(),
        ));
        idx += 1;

        // 2. package.json (Manifest)
        let pkg_body = "{\n  \
  \"name\": \"golden-repo-web\",\n  \
  \"version\": \"1.0.0\",\n  \
  \"scripts\": {\n    \
    \"dev\": \"vite dev\",\n    \
    \"build\": \"vite build\"\n  \
  },\n  \
  \"dependencies\": {\n    \
    \"svelte\": \"^5.0.0\"\n  \
  }\n}\n";
        sections.push(Self::build_section(
            idx,
            "package.json",
            SectionKind::Manifest,
            "json",
            pkg_body,
            vec![("manifest_type", "npm")],
            adapter.as_deref(),
        ));
        idx += 1;

        // 3. README.md (Prose, Root Readme)
        let readme_body = "# Golden Repository Engine\n\n\
Universal asynchronous transaction and workflow processing system built for high-throughput, low-latency applications.\n\n\
## Architecture Highlights\n\
- Multi-tier pipeline architecture with asynchronous tokio runtime\n\
- End-to-end cryptographic payload verification and secret isolation\n\
- Structured logging and distributed tracing instrumentation\n\
- Clean error hierarchies and deterministic failure domain containment\n";
        sections.push(Self::build_section(
            idx,
            "README.md",
            SectionKind::Prose,
            "markdown",
            readme_body,
            vec![("prominence", "root_readme")],
            adapter.as_deref(),
        ));
        idx += 1;

        // 4. docs/ARCHITECTURE.md (Prose, Architecture)
        let arch_body = "# Architectural Specification\n\n\
## Subsystem Boundaries\n\
1. **Ingress Layer**: Handles API authentication, rate limiting, and request decoding.\n\
2. **Domain Core**: Processes transactions through a state machine with strict transactional semantics.\n\
3. **Vault & Security**: Manages AES-GCM encryption keys and JWT token signing.\n\
4. **Telemetry**: Exposes Prometheus metrics and OpenTelemetry trace spans.\n";
        sections.push(Self::build_section(
            idx,
            "docs/ARCHITECTURE.md",
            SectionKind::Prose,
            "markdown",
            arch_body,
            vec![("doc_category", "architecture")],
            adapter.as_deref(),
        ));
        idx += 1;

        // 5. docs/TROUBLESHOOTING.md (Prose, Runbook)
        let trouble_body = "# Operational Troubleshooting & Incident Runbook\n\n\
## Diagnostic Error Codes\n\
- `ERR_CONN_TIMEOUT`: Ingress connection timed out waiting for upstream gateway.\n  \
  - Action: Verify security group egress and upstream latency metrics.\n\
- `ERR_AUTH_FAILED`: Bearer token signature validation failed or token expired.\n  \
  - Action: Confirm issuer public keys in vault and inspect JWT expiration claims.\n\
- `ERR_PAYMENT_FAILED`: Payment transaction settlement rejected by payment provider.\n  \
  - Action: Inspect provider response body and retry with idempotent key.\n";
        sections.push(Self::build_section(
            idx,
            "docs/TROUBLESHOOTING.md",
            SectionKind::Prose,
            "markdown",
            trouble_body,
            vec![("doc_category", "runbook")],
            adapter.as_deref(),
        ));
        idx += 1;

        // 6. SECURITY.md (Prose, Security Policy)
        let sec_body = "# Security Policy and Vulnerability Disclosure\n\n\
## Supported Versions\n\
Only the latest release receives active security patches.\n\n\
## Reporting a Vulnerability\n\
Email security@golden-repo.local with encrypted GPG reports. Do not open public issues.\n\
All cryptographic keys must be 256-bit AES-GCM or Ed25519 signatures.\n";
        sections.push(Self::build_section(
            idx,
            "SECURITY.md",
            SectionKind::Prose,
            "markdown",
            sec_body,
            vec![("doc_category", "security_policy")],
            adapter.as_deref(),
        ));
        idx += 1;

        // 7. CHANGELOG_OLD.md (Prose, Historical / Cosmetic)
        let cl_body = "# Legacy Changelog (v0.1 - v0.3)\n\n\
- v0.1.0: Initial prototype release with basic loop.\n\
- v0.2.0: Added experimental console output formatting.\n\
- v0.3.0: Updated dependency versions and cleaned up build scripts.\n";
        sections.push(Self::build_section(
            idx,
            "CHANGELOG_OLD.md",
            SectionKind::Prose,
            "markdown",
            cl_body,
            vec![("doc_category", "historical_changelog")],
            adapter.as_deref(),
        ));
        idx += 1;

        // 8. src/core/engine.rs (CodeFile, Core Implementation)
        let engine_body = "//! Core transaction processing engine.\n\n\
/// Processing engine state machine.\n\
#[derive(Debug, Clone)]\n\
pub struct ProcessingEngine {\n    \
    pub engine_id: String,\n    \
    pub max_concurrency: usize,\n\
}\n\n\
impl ProcessingEngine {\n    \
    /// Creates a new processing engine instance.\n    \
    pub fn new(engine_id: impl Into<String>, max_concurrency: usize) -> Self {\n        \
        Self {\n            \
            engine_id: engine_id.into(),\n            \
            max_concurrency,\n        \
        }\n    \
    }\n\n    \
    /// Executes transaction processing for an incoming payload.\n    \
    pub async fn process_transaction(&self, payload: &[u8]) -> Result<Vec<u8>, String> {\n        \
        if payload.is_empty() {\n            \
            return Err(\"empty transaction payload\".to_string());\n        \
        }\n        \
        Ok(payload.to_vec())\n    \
    }\n}\n";
        sections.push(Self::build_section(
            idx,
            "src/core/engine.rs",
            SectionKind::CodeFile,
            "rust",
            engine_body,
            vec![("code_nature", "core_implementation")],
            adapter.as_deref(),
        ));
        idx += 1;

        // 9. src/api/routes.rs (CodeFile, API Endpoints)
        let routes_body = "//! API route handlers and endpoint registration.\n\n\
use axum::{routing::get, Router};\n\n\
/// Constructs the top-level application router.\n\
pub fn build_api_router() -> Router {\n    \
    Router::new()\n        \
        .route(\"/api/v1/health\", get(health_handler))\n        \
        .route(\"/api/v1/orders\", get(orders_handler))\n\
}\n\n\
async fn health_handler() -> &'static str {\n    \"OK\"\n}\n\n\
async fn orders_handler() -> &'static str {\n    \"orders list\"\n}\n";
        sections.push(Self::build_section(
            idx,
            "src/api/routes.rs",
            SectionKind::CodeFile,
            "rust",
            routes_body,
            vec![("code_nature", "api_endpoints")],
            adapter.as_deref(),
        ));
        idx += 1;

        // 10. src/auth/jwt.rs (CodeFile, Auth & Access Control)
        let jwt_body = "//! Authentication and JWT token validation.\n\n\
use serde::{Deserialize, Serialize};\n\n\
/// Decoded JWT claims structure.\n\
#[derive(Debug, Clone, Serialize, Deserialize)]\n\
pub struct Claims {\n    \
    pub sub: String,\n    \
    pub exp: u64,\n    \
    pub roles: Vec<String>,\n\
}\n\n\
/// Verifies a bearer JWT token against expected issuer.\n\
pub fn verify_jwt(token: &str, _secret: &[u8]) -> Result<Claims, String> {\n    \
    if token.is_empty() {\n        \
        return Err(\"ERR_AUTH_FAILED: empty bearer token\".to_string());\n    \
    }\n    \
    Ok(Claims {\n        \
        sub: \"user-123\".to_string(),\n        \
        exp: 9999999999,\n        \
        roles: vec![\"admin\".to_string()],\n    \
    })\n}\n";
        sections.push(Self::build_section(
            idx,
            "src/auth/jwt.rs",
            SectionKind::CodeFile,
            "rust",
            jwt_body,
            vec![("code_nature", "auth_and_access_control")],
            adapter.as_deref(),
        ));
        idx += 1;

        // 11. src/crypto/vault.rs (CodeFile, Cryptography & Secrets)
        let vault_body = "//! Cryptographic envelope encryption and key vault.\n\n\
/// Secret vault encryptor.\n\
pub struct VaultEncryptor {\n    \
    _key: [u8; 32],\n\
}\n\n\
impl VaultEncryptor {\n    \
    /// Creates a new vault encryptor.\n    \
    pub fn new(key: [u8; 32]) -> Self {\n        \
        Self { _key: key }\n    \
    }\n\n    \
    /// Encrypts plaintext bytes using authenticated AES-256-GCM cipher.\n    \
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, String> {\n        \
        if plaintext.is_empty() {\n            \
            return Err(\"empty plaintext\".to_string());\n        \
        }\n        \
        let mut cipher = plaintext.to_vec();\n        \
        cipher.reverse();\n        \
        Ok(cipher)\n    \
    }\n}\n";
        sections.push(Self::build_section(
            idx,
            "src/crypto/vault.rs",
            SectionKind::CodeFile,
            "rust",
            vault_body,
            vec![("code_nature", "cryptography_and_secrets")],
            adapter.as_deref(),
        ));
        idx += 1;

        // 12. src/errors.rs (CodeFile, Error Definitions)
        let err_body = "//! Application error taxonomy and error propagation definitions.\n\n\
use thiserror::Error;\n\n\
/// Top-level error enum for the engine.\n\
#[derive(Error, Debug)]\n\
pub enum EngineError {\n    \
    #[error(\"connection timeout: {0}\")]\n    \
    ConnectionTimeout(String),\n    \
    #[error(\"authentication failed: {0}\")]\n    \
    AuthenticationFailed(String),\n    \
    #[error(\"payment settlement rejected: {0}\")]\n    \
    PaymentFailed(String),\n    \
    #[error(\"internal system fault: {0}\")]\n    \
    InternalFault(String),\n}\n";
        sections.push(Self::build_section(
            idx,
            "src/errors.rs",
            SectionKind::CodeFile,
            "rust",
            err_body,
            vec![("code_nature", "error_definitions")],
            adapter.as_deref(),
        ));
        idx += 1;

        // 13. src/telemetry/tracer.rs (CodeFile, Observability & Tracing)
        let tracer_body =
            "//! Distributed tracing, span recording, and telemetry instrumentation.\n\n\
/// Records an operational trace span with elapsed execution duration.\n\
pub fn record_span(span_name: &str, duration_ms: u64) {\n    \
    tracing::info!(\n        \
        target: \"telemetry::spans\",\n        \
        span_name = span_name,\n        \
        duration_ms = duration_ms,\n        \
        \"span executed\"\n    \
    );\n}\n\n\
/// Increments an operational incident counter metric.\n\
pub fn increment_counter(metric_name: &str, delta: u64) {\n    \
    tracing::debug!(metric = metric_name, delta = delta, \"metric incremented\");\n}\n";
        sections.push(Self::build_section(
            idx,
            "src/telemetry/tracer.rs",
            SectionKind::CodeFile,
            "rust",
            tracer_body,
            vec![("code_nature", "observability_and_logging")],
            adapter.as_deref(),
        ));
        idx += 1;

        // 14. src/features/payment.rs (CodeFile, Feature Under PR Review)
        let payment_body = "//! Payment gateway integration (new feature under active pull request review).\n\n\
/// Payment settlement processor.\n\
pub struct PaymentGateway {\n    \
    pub merchant_id: String,\n\
}\n\n\
impl PaymentGateway {\n    \
    /// Creates a payment gateway instance.\n    \
    pub fn new(merchant_id: impl Into<String>) -> Self {\n        \
        Self { merchant_id: merchant_id.into() }\n    \
    }\n\n    \
    /// Submits a charge settlement request to payment provider.\n    \
    pub async fn execute_charge(&self, amount_cents: u64, currency: &str) -> Result<String, String> {\n        \
        if amount_cents == 0 {\n            \
            return Err(\"ERR_PAYMENT_FAILED: zero amount\".to_string());\n        \
        }\n        \
        Ok(format!(\"ch_{}_{}\", currency, amount_cents))\n    \
    }\n}\n";
        sections.push(Self::build_section(
            idx,
            "src/features/payment.rs",
            SectionKind::CodeFile,
            "rust",
            payment_body,
            vec![("code_nature", "core_implementation")],
            adapter.as_deref(),
        ));
        idx += 1;

        // 15. tests/payment_test.rs (CodeFile, Associated Test for Feature)
        let pay_test_body = "//! Unit tests for the payment gateway feature.\n\n\
#[tokio::test]\n\
async fn test_payment_charge_success() {\n    \
    let gateway = PaymentGateway::new(\"merch_test_123\");\n    \
    let result = gateway.execute_charge(5000, \"USD\").await;\n    \
    assert!(result.is_ok());\n}\n";
        sections.push(Self::build_section(
            idx,
            "tests/payment_test.rs",
            SectionKind::CodeFile,
            "rust",
            pay_test_body,
            vec![("is_test", "true")],
            adapter.as_deref(),
        ));
        idx += 1;

        // 16. tests/security_auth_test.rs (CodeFile, Security Regression Test)
        let sec_test_body =
            "//! Security regression tests for authentication and token validation.\n\n\
#[test]\n\
fn test_jwt_tampering_rejected() {\n    \
    let result = verify_jwt(\"invalid.tampered.token\", b\"secret\");\n    \
    assert!(result.is_err());\n}\n";
        sections.push(Self::build_section(
            idx,
            "tests/security_auth_test.rs",
            SectionKind::CodeFile,
            "rust",
            sec_test_body,
            vec![("is_test", "true"), ("security_test", "true")],
            adapter.as_deref(),
        ));
        idx += 1;

        // 17. tests/negative_error_test.rs (CodeFile, Negative Edge-Case Test)
        let neg_test_body = "//! Negative edge-case and failure propagation tests.\n\n\
#[test]\n\
fn test_error_propagation_and_rejection() {\n    \
    let err = EngineError::PaymentFailed(\"provider offline\".to_string());\n    \
    assert!(err.to_string().contains(\"payment settlement rejected\"));\n}\n";
        sections.push(Self::build_section(
            idx,
            "tests/negative_error_test.rs",
            SectionKind::CodeFile,
            "rust",
            neg_test_body,
            vec![("is_test", "true"), ("test_nature", "negative")],
            adapter.as_deref(),
        ));
        idx += 1;

        // 18. tests/benchmark_test.rs (CodeFile, Benchmark Test)
        let bench_test_body =
            "//! Benchmark test measuring transactions per second throughput.\n\n\
#[test]\n\
fn test_engine_throughput_benchmark() {\n    \
    let engine = ProcessingEngine::new(\"bench-1\", 64);\n    \
    assert_eq!(engine.max_concurrency, 64);\n}\n";
        sections.push(Self::build_section(
            idx,
            "tests/benchmark_test.rs",
            SectionKind::CodeFile,
            "rust",
            bench_test_body,
            vec![("is_test", "true"), ("test_nature", "benchmark")],
            adapter.as_deref(),
        ));
        idx += 1;

        // 19. ui/styles/theme.css (CodeFile, Cosmetic Stylesheet)
        let css_body = "/* Presentation layer styles and theme tokens */\n\
:root {\n  \
  --primary-color: #2563eb;\n  \
  --secondary-color: #475569;\n  \
  --font-family: system-ui, sans-serif;\n  \
  --border-radius: 8px;\n\
}\n\
.btn-primary {\n  \
  background-color: var(--primary-color);\n  \
  color: #ffffff;\n  \
  border-radius: var(--border-radius);\n\
}\n";
        sections.push(Self::build_section(
            idx,
            "ui/styles/theme.css",
            SectionKind::CodeFile,
            "css",
            css_body,
            vec![("asset_type", "stylesheet")],
            adapter.as_deref(),
        ));
        idx += 1;

        // 20. ui/icons/logo.svg (CodeFile, Cosmetic Vector Graphic)
        let svg_body = "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 100 100\" width=\"100\" height=\"100\">\n  \
  <circle cx=\"50\" cy=\"50\" r=\"40\" stroke=\"#2563eb\" stroke-width=\"4\" fill=\"#eff6ff\" />\n  \
  <path d=\"M30 50 L45 65 L70 35\" stroke=\"#2563eb\" stroke-width=\"6\" fill=\"none\" />\n\
</svg>\n";
        sections.push(Self::build_section(
            idx,
            "ui/icons/logo.svg",
            SectionKind::CodeFile,
            "xml",
            svg_body,
            vec![("asset_type", "vector_graphic")],
            adapter.as_deref(),
        ));

        CdmDocument {
            schema_version: "1.0.0".to_string(),
            document_id: DocumentId::new(),
            source_identity: SourceIdentity::new(
                "https://github.com/dayashimoga/golden-repo-engine",
                "https://github.com/dayashimoga/golden-repo-engine",
            ),
            source_type: SourceType::GitHubPublic,
            canonical_url: "https://github.com/dayashimoga/golden-repo-engine".to_string(),
            retrieved_at: Utc::now(),
            revision: RevisionInfo::default(),
            content_type: "application/vnd.github.v3+json".to_string(),
            metadata: DocumentMetadata::default(),
            hierarchy: HierarchyNode::root("golden-repo-engine"),
            sections,
            relationships: Vec::new(),
            citations: Vec::new(),
            extraction_status: ExtractionStatus::Completed,
            warnings: Vec::new(),
            exclusions: Vec::new(),
            completeness: CompletenessReport::default(),
        }
    }

    fn build_section(
        idx: usize,
        path: &str,
        kind: SectionKind,
        lang: &str,
        body: &str,
        attrs: Vec<(&str, &str)>,
        adapter: Option<&dyn TokenizerAdapter>,
    ) -> CdmSection {
        let mut attributes = BTreeMap::new();
        for (k, v) in attrs {
            attributes.insert(k.to_string(), v.to_string());
        }

        let line_count = body.lines().count().max(1) as u32;
        let mut token_metrics = SectionTokenMetrics::compute_baseline(body);
        if let Some(a) = adapter {
            let count_res = a.count_tokens(body);
            token_metrics.exact_tokens = Some(count_res.count);
            token_metrics.tokenizer_name = Some(count_res.tokenizer_name.to_string());
        }

        CdmSection {
            section_id: format!("sec-{idx:03}"),
            parent_id: None,
            title: Some(path.to_string()),
            kind,
            original_path: Some(path.to_string()),
            source_url: format!(
                "https://github.com/dayashimoga/golden-repo-engine/blob/main/{path}"
            ),
            line_range: Some(LineRange::new(1, line_count)),
            byte_range: Some(ByteRange::new(0, body.len() as u64)),
            language: Some(lang.to_string()),
            content_hash: CdmSection::compute_hash(body),
            body: body.to_string(),
            token_metrics: Some(token_metrics),
            attributes,
        }
    }
}

/// Evaluation configuration for golden profile comparison benchmarks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoldenProfileConfig {
    /// Context budget defining total capacity and headroom tiers.
    pub budget: ContextBudget,
    /// Modified file paths for code review simulation.
    pub changed_files: Vec<String>,
    /// Targeted incident error codes or patterns for debugging simulation.
    pub target_error_patterns: Vec<String>,
    /// Optional target LLM model.
    pub target_model: Option<TargetModel>,
    /// Benchmark suite identifier.
    pub benchmark_id: String,
}

impl Default for GoldenProfileConfig {
    fn default() -> Self {
        Self {
            // 2,000 total tokens with 700 reserved (300 system, 200 response, 200 query),
            // leaving 1,300 available content tokens against a ~2,800 token fixture.
            budget: ContextBudget::new(2000, 300, 200, 200),
            changed_files: vec!["src/features/payment.rs".to_string()],
            target_error_patterns: vec![
                "ERR_PAYMENT_FAILED".to_string(),
                "ERR_AUTH_FAILED".to_string(),
            ],
            target_model: Some(TargetModel::Gpt4o),
            benchmark_id: "golden-repo-v1".to_string(),
        }
    }
}

/// Itemized evaluation result for a single profile on the golden repository dataset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProfileGoldenEvaluation {
    /// Human-readable profile name.
    pub profile_name: String,
    /// Profile classification if applicable.
    pub profile_kind: Option<ProfileKind>,
    /// Number of content tokens fitted within budget.
    pub fitted_tokens: usize,
    /// Available content budget tokens.
    pub available_content_tokens: usize,
    /// Percentage of available content budget utilized (0.0 - 100.0).
    pub budget_utilization_pct: f64,
    /// Number of sections retained in optimized document.
    pub retained_sections_count: usize,
    /// Number of sections excluded from document.
    pub excluded_sections_count: usize,
    /// List of file paths retained.
    pub retained_paths: Vec<String>,
    /// List of file paths excluded.
    pub excluded_paths: Vec<String>,
    /// Preservation metrics report.
    pub preservation: PreservationReport,
    /// Compression metrics report.
    pub compression: CompressionReport,
    /// Diagnostic exclusion report.
    pub exclusions: ExclusionReport,
    /// Category distribution metrics if profile-based.
    pub category_metrics: Option<ProfileMetrics>,
    /// Whether all profile-specific invariant assertions passed.
    pub invariants_passed: bool,
    /// Diagnostics for any failed invariant checks.
    pub invariant_diagnostics: Vec<String>,
}

impl ProfileGoldenEvaluation {
    /// Checks whether a specific path is retained.
    #[must_use]
    pub fn has_path(&self, path: &str) -> bool {
        let norm = path.replace('\\', "/").to_ascii_lowercase();
        self.retained_paths
            .iter()
            .any(|p| p.replace('\\', "/").to_ascii_lowercase() == norm)
    }

    /// Checks whether a specific path is excluded.
    #[must_use]
    pub fn excludes_path(&self, path: &str) -> bool {
        let norm = path.replace('\\', "/").to_ascii_lowercase();
        self.excluded_paths
            .iter()
            .any(|p| p.replace('\\', "/").to_ascii_lowercase() == norm)
    }

    /// Renders a GitHub Flavored Markdown row for comparison tables.
    #[must_use]
    pub fn to_markdown_row(&self) -> String {
        let inv_status = if self.invariants_passed {
            "✅ `PASS`".to_string()
        } else {
            format!("❌ `FAIL ({})`", self.invariant_diagnostics.len())
        };

        let gpt4o_cost = self
            .compression
            .cost_savings
            .models
            .iter()
            .find(|m| m.model == TargetModel::Gpt4o)
            .map(|s| s.cost_savings_usd)
            .unwrap_or(0.0);

        format!(
            "| **{}** | `{}` / `{}` | `{:.1}%` | `{}` | `{}` | `{:.1}%` | `${:.4}` | {} |",
            self.profile_name,
            self.fitted_tokens,
            self.available_content_tokens,
            self.budget_utilization_pct,
            self.retained_sections_count,
            self.excluded_sections_count,
            self.preservation.fidelity_score * 100.0,
            gpt4o_cost,
            inv_status,
        )
    }
}

/// Unified comparative matrix and benchmark report card across context profiles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoldenComparisonReport {
    /// Benchmark run identifier.
    pub benchmark_id: String,
    /// Timestamp when benchmark was executed.
    pub executed_at: DateTime<Utc>,
    /// Original unoptimized document token volume.
    pub original_tokens: usize,
    /// Original unoptimized document section count.
    pub original_sections_count: usize,
    /// Available content budget tokens.
    pub available_budget_tokens: usize,
    /// Target model used for token counting, if specified.
    pub target_model: Option<TargetModel>,
    /// Evaluations keyed by profile name.
    pub profiles: BTreeMap<String, ProfileGoldenEvaluation>,
}

impl GoldenComparisonReport {
    /// Returns true if all profile invariant assertions passed without exceptions.
    #[must_use]
    pub fn all_invariants_passed(&self) -> bool {
        self.profiles.values().all(|p| p.invariants_passed)
    }

    /// Renders a GitHub Flavored Markdown comparison table across all evaluated profiles.
    #[must_use]
    pub fn to_markdown_comparison_table(&self) -> String {
        let mut out = String::new();
        out.push_str(
            "| Profile | Fitted / Budget | Util (%) | Retained | Excluded | Fidelity (%) | Cost Savings / 1k (GPT-4o) | Invariants |\n\
             | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |\n",
        );

        for eval in self.profiles.values() {
            out.push_str(&eval.to_markdown_row());
            out.push('\n');
        }

        out
    }

    /// Renders a category distribution table across profile-based evaluations.
    #[must_use]
    pub fn to_markdown_category_table(&self) -> String {
        let mut out = String::new();
        out.push_str(
            "| Profile | Documentation | Core Code | Manifests | Tests | Other | Total Fitted |\n\
             | :--- | :---: | :---: | :---: | :---: | :---: | :---: |\n",
        );

        for eval in self.profiles.values() {
            if let Some(ref m) = eval.category_metrics {
                out.push_str(&format!(
                    "| **{}** | `{}` ({:.1}%) | `{}` ({:.1}%) | `{}` ({:.1}%) | `{}` ({:.1}%) | `{}` | `{}` |\n",
                    eval.profile_name,
                    m.documentation_tokens,
                    m.documentation_ratio * 100.0,
                    m.core_code_tokens,
                    m.core_code_ratio * 100.0,
                    m.manifest_tokens,
                    m.manifest_ratio * 100.0,
                    m.test_tokens,
                    if m.total_tokens > 0 {
                        (m.test_tokens as f64 / m.total_tokens as f64) * 100.0
                    } else {
                        0.0
                    },
                    m.other_tokens,
                    m.total_tokens,
                ));
            } else {
                out.push_str(&format!(
                    "| **{}** | — | — | — | — | — | `{}` |\n",
                    eval.profile_name, eval.fitted_tokens
                ));
            }
        }

        out
    }

    /// Renders a comprehensive GitHub Flavored Markdown diagnostic summary report.
    #[must_use]
    pub fn to_markdown_summary(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "## Golden Context Profile Benchmark: `{}`\n\n",
            self.benchmark_id
        ));
        out.push_str(&format!(
            "**Executed At:** `{}`  \n\
             **Original Document Volume:** `{}` tokens across `{}` sections  \n\
             **Content Budget Ceiling:** `{}` tokens  \n\
             **Overall Verification Status:** {}\n\n",
            self.executed_at.to_rfc3339(),
            self.original_tokens,
            self.original_sections_count,
            self.available_budget_tokens,
            if self.all_invariants_passed() {
                "✅ `ALL_INVARIANTS_PASSED`"
            } else {
                "❌ `INVARIANT_FAILURES_DETECTED`"
            }
        ));

        out.push_str("### 1. Cross-Profile Performance Matrix\n\n");
        out.push_str(&self.to_markdown_comparison_table());
        out.push('\n');

        out.push_str("### 2. Content Category Token Allocation\n\n");
        out.push_str(&self.to_markdown_category_table());
        out.push('\n');

        out.push_str("### 3. Profile Invariant Diagnostic Results\n\n");
        for eval in self.profiles.values() {
            out.push_str(&format!("#### Profile: `{}`\n", eval.profile_name));
            if eval.invariants_passed {
                out.push_str("- ✅ All domain-specific invariants and budget bounds satisfied.\n");
            } else {
                for diag in &eval.invariant_diagnostics {
                    out.push_str(&format!("- ❌ {diag}\n"));
                }
            }
            out.push('\n');
        }

        out
    }

    /// Serializes the comparison report to a formatted JSON string.
    pub fn to_json(&self) -> Result<String, ContextOptimizerError> {
        serde_json::to_string_pretty(self).map_err(ContextOptimizerError::Serialization)
    }

    /// Deserializes a comparison report from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, ContextOptimizerError> {
        serde_json::from_str(json).map_err(ContextOptimizerError::Serialization)
    }
}

/// Orchestrates cross-profile golden benchmarking, telemetry collection, and invariant audits.
#[derive(Debug, Clone, Default)]
pub struct GoldenProfileComparator;

impl GoldenProfileComparator {
    /// Executes the full golden profile comparison suite on a document.
    pub fn compare(
        golden_doc: &CdmDocument,
        config: &GoldenProfileConfig,
    ) -> Result<GoldenComparisonReport, ContextOptimizerError> {
        if golden_doc.sections.is_empty() {
            return Err(ContextOptimizerError::DocumentEmpty);
        }

        let original_volume =
            crate::compression::DocumentVolume::measure(golden_doc, config.target_model);
        let available_budget = config.budget.available_content_tokens();
        let mut profile_results = BTreeMap::new();

        // 1. General AI Profile
        let gen_profile = GeneralAiProfile::default();
        let gen_res = gen_profile.optimize(golden_doc, &config.budget)?;
        let gen_eval = Self::evaluate_profile(
            "General AI",
            Some(ProfileKind::GeneralAi),
            golden_doc,
            &gen_res.document,
            gen_res.plan.fitted_tokens,
            available_budget,
            gen_res.plan.budget_utilization_pct,
            Some(gen_res.metrics),
            config,
            |eval| {
                let mut diags = Vec::new();
                if !eval.has_path("README.md") {
                    diags.push("General AI must preserve root README.md".to_string());
                }
                if !eval.has_path("Cargo.toml") {
                    diags.push("General AI must preserve Cargo.toml manifest".to_string());
                }
                if !eval.has_path("src/core/engine.rs") {
                    diags.push(
                        "General AI must preserve core implementation src/core/engine.rs"
                            .to_string(),
                    );
                }
                if !eval.excludes_path("tests/benchmark_test.rs") {
                    diags.push("General AI should suppress low-signal benchmark test".to_string());
                }
                if !eval.excludes_path("ui/icons/logo.svg") {
                    diags.push(
                        "General AI should suppress cosmetic vector asset ui/icons/logo.svg"
                            .to_string(),
                    );
                }
                diags
            },
        )?;
        profile_results.insert("1_General_AI".to_string(), gen_eval);

        // 2. Code Review Profile
        let cr_profile = CodeReviewProfile::from_changed_files(config.changed_files.clone());
        let cr_res = cr_profile.optimize(golden_doc, &config.budget)?;
        let cr_eval = Self::evaluate_profile(
            "Code Review",
            Some(ProfileKind::CodeReview),
            golden_doc,
            &cr_res.document,
            cr_res.plan.fitted_tokens,
            available_budget,
            cr_res.plan.budget_utilization_pct,
            Some(cr_res.metrics),
            config,
            |eval| {
                let mut diags = Vec::new();
                for cf in &config.changed_files {
                    if !eval.has_path(cf) {
                        diags.push(format!(
                            "Code Review must unconditionally preserve changed file `{cf}`"
                        ));
                    }
                }
                if !eval.has_path("tests/payment_test.rs") {
                    diags.push(
                        "Code Review must preserve associated unit test `tests/payment_test.rs`"
                            .to_string(),
                    );
                }
                if !eval.excludes_path("docs/ARCHITECTURE.md") {
                    diags.push(
                        "Code Review should suppress unmodified prose `docs/ARCHITECTURE.md`"
                            .to_string(),
                    );
                }
                if !eval.excludes_path("ui/styles/theme.css") {
                    diags.push(
                        "Code Review should suppress cosmetic presentation `ui/styles/theme.css`"
                            .to_string(),
                    );
                }
                diags
            },
        )?;
        profile_results.insert("2_Code_Review".to_string(), cr_eval);

        // 3. Security Review Profile
        let sec_profile = SecurityReviewProfile::default();
        let sec_res = sec_profile.optimize(golden_doc, &config.budget)?;
        let sec_eval = Self::evaluate_profile(
            "Security Review",
            Some(ProfileKind::SecurityReview),
            golden_doc,
            &sec_res.document,
            sec_res.plan.fitted_tokens,
            available_budget,
            sec_res.plan.budget_utilization_pct,
            Some(sec_res.metrics),
            config,
            |eval| {
                let mut diags = Vec::new();
                if !eval.has_path("src/auth/jwt.rs") {
                    diags.push(
                        "Security Review must preserve auth module `src/auth/jwt.rs`".to_string(),
                    );
                }
                if !eval.has_path("src/crypto/vault.rs") {
                    diags.push(
                        "Security Review must preserve crypto vault `src/crypto/vault.rs`"
                            .to_string(),
                    );
                }
                if !eval.has_path("SECURITY.md") {
                    diags.push(
                        "Security Review must preserve security policy `SECURITY.md`".to_string(),
                    );
                }
                if !eval.has_path("tests/security_auth_test.rs") {
                    diags.push("Security Review must preserve security regression test `tests/security_auth_test.rs`".to_string());
                }
                if !eval.excludes_path("ui/styles/theme.css") {
                    diags.push("Security Review should suppress cosmetic presentation `ui/styles/theme.css`".to_string());
                }
                diags
            },
        )?;
        profile_results.insert("3_Security_Review".to_string(), sec_eval);

        // 4. Debugging Profile
        let mut deb_cfg = DebuggingConfig::default();
        if !config.target_error_patterns.is_empty() {
            deb_cfg = deb_cfg.with_target_error_patterns(config.target_error_patterns.clone());
        }
        let deb_profile = DebuggingProfile::new(deb_cfg);
        let deb_res = deb_profile.optimize(golden_doc, &config.budget)?;
        let deb_eval = Self::evaluate_profile(
            "Debugging",
            Some(ProfileKind::DebuggingAndDocumentation),
            golden_doc,
            &deb_res.document,
            deb_res.plan.fitted_tokens,
            available_budget,
            deb_res.plan.budget_utilization_pct,
            Some(deb_res.metrics),
            config,
            |eval| {
                let mut diags = Vec::new();
                if !eval.has_path("src/errors.rs") {
                    diags.push(
                        "Debugging must preserve error definitions `src/errors.rs`".to_string(),
                    );
                }
                if !eval.has_path("src/telemetry/tracer.rs") {
                    diags.push(
                        "Debugging must preserve observability tracer `src/telemetry/tracer.rs`"
                            .to_string(),
                    );
                }
                if !eval.has_path("docs/TROUBLESHOOTING.md") {
                    diags.push(
                        "Debugging must preserve troubleshooting runbook `docs/TROUBLESHOOTING.md`"
                            .to_string(),
                    );
                }
                if !eval.has_path("tests/negative_error_test.rs") {
                    diags.push("Debugging must preserve negative edge-case test `tests/negative_error_test.rs`".to_string());
                }
                if !eval.excludes_path("ui/styles/theme.css") {
                    diags.push(
                        "Debugging should suppress cosmetic presentation `ui/styles/theme.css`"
                            .to_string(),
                    );
                }
                diags
            },
        )?;
        profile_results.insert("4_Debugging".to_string(), deb_eval);

        // 5. Baseline Raw Knapsack Optimizer (unprofiled)
        let raw_config = crate::optimizer::BudgetOptimizationConfig::new(config.budget.clone());
        let raw_optimizer = ContextBudgetOptimizer::new(raw_config);
        let (raw_doc, raw_plan) = raw_optimizer.optimize(golden_doc)?;
        let raw_eval = Self::evaluate_profile(
            "Raw Knapsack (Baseline)",
            None,
            golden_doc,
            &raw_doc,
            raw_plan.fitted_tokens,
            available_budget,
            raw_plan.budget_utilization_pct,
            None,
            config,
            |eval| {
                let mut diags = Vec::new();
                if eval.fitted_tokens > eval.available_content_tokens {
                    diags.push(
                        "Raw knapsack must strictly adhere to available content budget".to_string(),
                    );
                }
                diags
            },
        )?;
        profile_results.insert("5_Raw_Knapsack".to_string(), raw_eval);

        Ok(GoldenComparisonReport {
            benchmark_id: config.benchmark_id.clone(),
            executed_at: Utc::now(),
            original_tokens: original_volume.tokens,
            original_sections_count: golden_doc.sections.len(),
            available_budget_tokens: available_budget,
            target_model: config.target_model,
            profiles: profile_results,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn evaluate_profile<F>(
        profile_name: &str,
        profile_kind: Option<ProfileKind>,
        original_doc: &CdmDocument,
        optimized_doc: &CdmDocument,
        fitted_tokens: usize,
        available_budget: usize,
        utilization_pct: f64,
        category_metrics: Option<ProfileMetrics>,
        config: &GoldenProfileConfig,
        invariant_checker: F,
    ) -> Result<ProfileGoldenEvaluation, ContextOptimizerError>
    where
        F: FnOnce(&ProfileGoldenEvaluation) -> Vec<String>,
    {
        let target_model = config.target_model.unwrap_or(TargetModel::Gpt4o);

        let preservation = PreservationEvaluator::evaluate(original_doc, optimized_doc)?;
        let compression =
            CompressionEvaluator::evaluate_for_model(original_doc, optimized_doc, target_model)?;
        let exclusions = ExclusionExplainer::explain_documents(
            original_doc,
            optimized_doc,
            Some(profile_name),
            Some(target_model),
        )?;

        let original_paths: BTreeSet<String> = original_doc
            .sections
            .iter()
            .filter_map(|s| s.original_path.clone())
            .collect();

        let retained_paths: Vec<String> = optimized_doc
            .sections
            .iter()
            .filter_map(|s| s.original_path.clone())
            .collect();

        let retained_set: BTreeSet<String> = retained_paths.iter().cloned().collect();
        let excluded_paths: Vec<String> =
            original_paths.difference(&retained_set).cloned().collect();

        let mut evaluation = ProfileGoldenEvaluation {
            profile_name: profile_name.to_string(),
            profile_kind,
            fitted_tokens,
            available_content_tokens: available_budget,
            budget_utilization_pct: utilization_pct,
            retained_sections_count: optimized_doc.sections.len(),
            excluded_sections_count: excluded_paths.len(),
            retained_paths,
            excluded_paths,
            preservation,
            compression,
            exclusions,
            category_metrics,
            invariants_passed: true,
            invariant_diagnostics: Vec::new(),
        };

        let mut diags = Vec::new();
        if evaluation.fitted_tokens > evaluation.available_content_tokens {
            diags.push(format!(
                "Budget overrun: fitted tokens ({}) exceeds available content tokens ({})",
                evaluation.fitted_tokens, evaluation.available_content_tokens
            ));
        }

        let custom_diags = invariant_checker(&evaluation);
        diags.extend(custom_diags);

        if !diags.is_empty() {
            evaluation.invariants_passed = false;
            evaluation.invariant_diagnostics = diags;
        }

        Ok(evaluation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_golden_fixture_structure() {
        let doc = GoldenRepositoryFixture::create_default();
        assert_eq!(doc.sections.len(), 20);
        assert!(doc
            .sections
            .iter()
            .any(|s| s.original_path.as_deref() == Some("Cargo.toml")));
        assert!(doc
            .sections
            .iter()
            .any(|s| s.original_path.as_deref() == Some("src/core/engine.rs")));
        assert!(doc
            .sections
            .iter()
            .any(|s| s.original_path.as_deref() == Some("src/auth/jwt.rs")));
        assert!(doc
            .sections
            .iter()
            .any(|s| s.original_path.as_deref() == Some("src/crypto/vault.rs")));
        assert!(doc
            .sections
            .iter()
            .any(|s| s.original_path.as_deref() == Some("src/errors.rs")));
        assert!(doc
            .sections
            .iter()
            .any(|s| s.original_path.as_deref() == Some("src/telemetry/tracer.rs")));
        assert!(doc
            .sections
            .iter()
            .any(|s| s.original_path.as_deref() == Some("src/features/payment.rs")));
        assert!(doc
            .sections
            .iter()
            .any(|s| s.original_path.as_deref() == Some("tests/payment_test.rs")));
        assert!(doc
            .sections
            .iter()
            .any(|s| s.original_path.as_deref() == Some("ui/styles/theme.css")));
        assert!(doc
            .sections
            .iter()
            .any(|s| s.original_path.as_deref() == Some("ui/icons/logo.svg")));

        for s in &doc.sections {
            assert!(s.verify_hash());
            assert!(s.token_metrics.is_some());
        }
    }

    #[test]
    fn test_golden_profile_comparison_runs_successfully() {
        let doc = GoldenRepositoryFixture::create_default();
        let config = GoldenProfileConfig::default();
        let report =
            GoldenProfileComparator::compare(&doc, &config).expect("comparison should succeed");

        assert_eq!(report.original_sections_count, 20);
        assert_eq!(report.profiles.len(), 5);
        assert!(
            report.all_invariants_passed(),
            "all invariants must pass on default config: {:?}",
            report.to_markdown_summary()
        );

        let table = report.to_markdown_comparison_table();
        assert!(table.contains("General AI"));
        assert!(table.contains("Code Review"));
        assert!(table.contains("Security Review"));
        assert!(table.contains("Debugging"));
        assert!(table.contains("Raw Knapsack"));

        let json = report.to_json().expect("json serialize");
        let roundtrip = GoldenComparisonReport::from_json(&json).expect("json deserialize");
        assert_eq!(roundtrip.profiles.len(), 5);
    }
}
