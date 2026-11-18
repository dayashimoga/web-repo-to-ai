//! # Debugging and Documentation Context Profile
//!
//! Tailors context specifically for production incident response, error triage, exception tracing,
//! API integration, and troubleshooting:
//! - Pinned inclusion of error definitions, enums, exception hierarchies, and error mapping
//! - Pinned inclusion of observability instrumentation, logging, tracing, and diagnostic metrics
//! - Prioritized inclusion of API contracts, route declarations, request/response DTOs, and schemas
//! - Prioritized inclusion of operational runbooks, troubleshooting guides, and API documentation
//! - Prioritized inclusion of negative/error-handling unit and integration tests
//! - Suppression of cosmetic presentation assets (CSS, styles, UI layouts) and non-technical changelogs

use super::{ContextProfile, ProfileKind, ProfileMetrics, ProfileOptimizationResult};
use crate::budget::ContextBudget;
use crate::error::ContextOptimizerError;
use crate::optimizer::{BudgetOptimizationConfig, ContextBudgetOptimizer, TruncationStrategy};
use crate::priority::SectionScorer;
use serde::{Deserialize, Serialize};
use web_repo_to_ai_cdm::{CdmDocument, CdmSection, SectionKind};

/// Categorization of sections evaluated in a debugging and documentation context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DebuggingCategory {
    /// Error definitions, error enums, exception types, and error propagation logic.
    ErrorDefinitionsAndPropagation,
    /// Logging, tracing, observability, spans, metrics, and diagnostics.
    ObservabilityAndLogging,
    /// API contracts, routes, request/response models, interfaces, and schemas.
    ApiContractsAndInterfaces,
    /// Technical documentation, troubleshooting guides, runbooks, and developer specs.
    TroubleshootingDocsAndRunbooks,
    /// Unit and integration tests exercising error paths, edge cases, and failure modes.
    NegativeAndEdgeCaseTests,
    /// Project configuration, package manifests, and environment variables.
    ManifestsAndEnvironment,
    /// Internal implementation code (preserved according to priority or trimmed).
    InternalImplementation,
    /// Benign presentation, styling, UI layouts, icons, fonts, marketing prose.
    CosmeticOrUnrelatedAsset,
}

impl DebuggingCategory {
    /// Returns a human-readable identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ErrorDefinitionsAndPropagation => "error_definitions_and_propagation",
            Self::ObservabilityAndLogging => "observability_and_logging",
            Self::ApiContractsAndInterfaces => "api_contracts_and_interfaces",
            Self::TroubleshootingDocsAndRunbooks => "troubleshooting_docs_and_runbooks",
            Self::NegativeAndEdgeCaseTests => "negative_and_edge_case_tests",
            Self::ManifestsAndEnvironment => "manifests_and_environment",
            Self::InternalImplementation => "internal_implementation",
            Self::CosmeticOrUnrelatedAsset => "cosmetic_or_unrelated_asset",
        }
    }

    /// Whether this category represents high-value debugging and diagnostic context.
    #[must_use]
    pub const fn is_debugging_critical(self) -> bool {
        matches!(
            self,
            Self::ErrorDefinitionsAndPropagation
                | Self::ObservabilityAndLogging
                | Self::ApiContractsAndInterfaces
                | Self::TroubleshootingDocsAndRunbooks
        )
    }
}

/// Configuration options for the Debugging and Documentation context profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DebuggingConfig {
    /// Whether to prioritize error definitions, types, and propagation logic (default: true).
    pub include_error_definitions: bool,
    /// Whether to prioritize logging, tracing, and observability hooks (default: true).
    pub include_observability: bool,
    /// Whether to prioritize API contracts, schemas, and interface definitions (default: true).
    pub include_api_contracts: bool,
    /// Whether to include troubleshooting runbooks and technical docs (default: true).
    pub include_troubleshooting_docs: bool,
    /// Whether to include negative, failure-mode, and edge-case unit tests (default: true).
    pub include_negative_tests: bool,
    /// Whether to include package manifests and build configs (default: true).
    pub include_manifests: bool,
    /// Whether to include general internal implementation files (default: true).
    pub include_internal_implementation: bool,
    /// Whether to include cosmetic presentation components and styles (default: false).
    pub include_cosmetic_assets: bool,
    /// Optional target error codes, keywords, or exception strings to specifically pin.
    pub target_error_patterns: Vec<String>,
    /// Explicit file paths pinned for mandatory inclusion.
    pub pinned_paths: Vec<String>,
    /// Whether individual sections may be truncated to fit headroom.
    pub allow_truncation: bool,
    /// Boundary truncation strategy to use when shortening large files.
    pub truncation_strategy: TruncationStrategy,
    /// Minimum retained token floor required to justify truncating a section.
    pub min_retained_tokens: usize,
}

impl Default for DebuggingConfig {
    fn default() -> Self {
        Self {
            include_error_definitions: true,
            include_observability: true,
            include_api_contracts: true,
            include_troubleshooting_docs: true,
            include_negative_tests: true,
            include_manifests: true,
            include_internal_implementation: true,
            include_cosmetic_assets: false,
            target_error_patterns: Vec::new(),
            pinned_paths: Vec::new(),
            allow_truncation: true,
            truncation_strategy: TruncationStrategy::HeadAndTail,
            min_retained_tokens: 50,
        }
    }
}

impl DebuggingConfig {
    /// Creates a new default Debugging profile configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets target error patterns (error codes, exception names, error messages) to pin.
    #[must_use]
    pub fn with_target_error_patterns(mut self, patterns: Vec<String>) -> Self {
        self.target_error_patterns = patterns;
        self
    }

    /// Sets explicit file paths that must be pinned for inclusion.
    #[must_use]
    pub fn with_pinned_paths(mut self, paths: Vec<String>) -> Self {
        self.pinned_paths = paths;
        self
    }

    /// Sets whether cosmetic presentation assets (styles, UI components) are included.
    #[must_use]
    pub const fn with_include_cosmetic_assets(mut self, include: bool) -> Self {
        self.include_cosmetic_assets = include;
        self
    }

    /// Sets whether negative/failure-mode test cases are included.
    #[must_use]
    pub const fn with_include_negative_tests(mut self, include: bool) -> Self {
        self.include_negative_tests = include;
        self
    }

    /// Sets whether individual sections may be truncated to fit headroom.
    #[must_use]
    pub const fn with_allow_truncation(mut self, allow: bool) -> Self {
        self.allow_truncation = allow;
        self
    }

    /// Sets the boundary truncation strategy.
    #[must_use]
    pub const fn with_truncation_strategy(mut self, strategy: TruncationStrategy) -> Self {
        self.truncation_strategy = strategy;
        self
    }

    /// Sets the minimum retained token floor.
    #[must_use]
    pub const fn with_min_retained_tokens(mut self, min: usize) -> Self {
        self.min_retained_tokens = min;
        self
    }
}

/// The Debugging and Documentation context profile engine.
#[derive(Debug, Clone, Default)]
pub struct DebuggingProfile {
    config: DebuggingConfig,
}

impl DebuggingProfile {
    /// Creates a new Debugging profile with custom configuration.
    #[must_use]
    pub const fn new(config: DebuggingConfig) -> Self {
        Self { config }
    }

    /// Creates a new Debugging profile targeting specific error patterns.
    #[must_use]
    pub fn from_error_pattern(pattern: impl Into<String>) -> Self {
        Self::new(DebuggingConfig::new().with_target_error_patterns(vec![pattern.into()]))
    }

    /// Returns a reference to the active configuration.
    #[must_use]
    pub const fn config(&self) -> &DebuggingConfig {
        &self.config
    }

    /// Classifies a [`CdmSection`] into a [`DebuggingCategory`].
    #[must_use]
    pub fn classify_section(&self, sec: &CdmSection) -> DebuggingCategory {
        let path = sec.original_path.as_deref().unwrap_or("");
        let normalized_path = path.replace('\\', "/").to_ascii_lowercase();
        let filename = normalized_path
            .rsplit('/')
            .next()
            .unwrap_or(&normalized_path);

        // 1. Check pinned paths
        if self.config.pinned_paths.iter().any(|pp| {
            let n_pp = pp.replace('\\', "/").to_ascii_lowercase();
            normalized_path == n_pp
                || normalized_path.ends_with(&n_pp)
                || filename == n_pp.rsplit('/').next().unwrap_or(&n_pp)
        }) {
            return DebuggingCategory::ErrorDefinitionsAndPropagation;
        }

        // 2. Check target error patterns
        if self.config.target_error_patterns.iter().any(|pattern| {
            let n_pat = pattern.to_ascii_lowercase();
            normalized_path.contains(&n_pat)
                || sec
                    .title
                    .as_deref()
                    .is_some_and(|t| t.to_ascii_lowercase().contains(&n_pat))
                || sec.body.to_ascii_lowercase().contains(&n_pat)
        }) {
            return DebuggingCategory::ErrorDefinitionsAndPropagation;
        }

        // 3. Check error definitions and exception hierarchies
        let is_error_file = filename.contains("error")
            || filename.contains("exception")
            || filename.contains("failure")
            || filename.contains("fault")
            || filename.contains("status_code");

        let body_has_error_def = sec.body.contains("enum Error")
            || sec.body.contains("enum ContextOptimizerError")
            || sec.body.contains("thiserror::Error")
            || sec.body.contains("anyhow::")
            || sec.body.contains("impl std::error::Error");

        if is_error_file || (body_has_error_def && sec.kind == SectionKind::CodeFile) {
            return DebuggingCategory::ErrorDefinitionsAndPropagation;
        }

        // 4. Check observability and logging
        let is_observability_path = normalized_path.contains("observability")
            || normalized_path.contains("logging")
            || normalized_path.contains("logger")
            || normalized_path.contains("tracing")
            || normalized_path.contains("telemetry")
            || normalized_path.contains("metrics")
            || normalized_path.contains("diagnostics");

        let body_has_tracing = sec.body.contains("tracing::")
            || sec.body.contains("info!(")
            || sec.body.contains("warn!(")
            || sec.body.contains("error!(")
            || sec.body.contains("debug!(")
            || sec.body.contains("#[instrument]");

        if is_observability_path || (body_has_tracing && is_observability_path) {
            return DebuggingCategory::ObservabilityAndLogging;
        }

        // 5. Check tests (distinguish negative/error tests from regular tests)
        let is_test = sec.attributes.get("is_test").is_some_and(|v| v == "true")
            || normalized_path.contains("/tests/")
            || normalized_path.contains("/test/")
            || filename.contains("test")
            || filename.contains("spec");

        if is_test {
            let is_negative_test = sec.body.contains("should_panic")
                || sec.body.contains("assert_err")
                || sec.body.contains("expect_err")
                || sec.body.contains(".is_err()")
                || sec.body.contains(".unwrap_err()")
                || sec.body.contains("BAD_REQUEST")
                || sec.body.contains("NOT_FOUND")
                || sec.body.contains("INTERNAL_SERVER_ERROR")
                || sec.body.contains("error")
                || sec.body.contains("Error")
                || sec.body.contains("timeout")
                || sec.body.contains("rejected");

            if is_negative_test {
                return DebuggingCategory::NegativeAndEdgeCaseTests;
            }
            return DebuggingCategory::CosmeticOrUnrelatedAsset;
        }

        // 6. Check API contracts and interfaces
        let is_api_contract = normalized_path.contains("api")
            || normalized_path.contains("routes")
            || normalized_path.contains("router")
            || normalized_path.contains("handlers")
            || normalized_path.contains("interface")
            || normalized_path.contains("schema")
            || normalized_path.contains("contracts")
            || normalized_path.contains("models")
            || normalized_path.contains("types")
            || filename.ends_with(".d.ts")
            || sec
                .attributes
                .get("code_nature")
                .is_some_and(|v| v == "interface_or_type_definition" || v == "data_or_schema");

        if is_api_contract && sec.kind != SectionKind::Prose {
            return DebuggingCategory::ApiContractsAndInterfaces;
        }

        // 7. Check troubleshooting documentation and operational runbooks
        let is_troubleshooting_doc = filename.contains("troubleshoot")
            || filename.contains("runbook")
            || filename.contains("debug")
            || filename.contains("faq")
            || filename.contains("architecture")
            || filename == "development.md"
            || filename == "deployment.md"
            || filename == "api.md"
            || filename == "readme.md";

        if is_troubleshooting_doc && sec.kind == SectionKind::Prose {
            return DebuggingCategory::TroubleshootingDocsAndRunbooks;
        }

        // 8. Check manifests and environment configurations
        let is_manifest = sec.kind == SectionKind::Manifest
            || filename == "cargo.toml"
            || filename == "package.json"
            || filename.contains(".env");

        if is_manifest {
            return DebuggingCategory::ManifestsAndEnvironment;
        }

        // 9. Check cosmetic presentation and UI assets
        let is_cosmetic = normalized_path.contains("apps/web")
            || normalized_path.contains("/ui/")
            || normalized_path.contains("/components/")
            || normalized_path.contains("tokens.css")
            || normalized_path.contains("tailwind")
            || normalized_path.contains("typography")
            || normalized_path.contains("color-utils")
            || normalized_path.contains("button.svelte")
            || normalized_path.contains("heading.svelte")
            || normalized_path.contains("modal.svelte")
            || filename == "changelog.md"
            || filename == "roadmap.md"
            || filename == "design_system.md";

        if is_cosmetic {
            return DebuggingCategory::CosmeticOrUnrelatedAsset;
        }

        DebuggingCategory::InternalImplementation
    }
}

impl ContextProfile for DebuggingProfile {
    fn profile_kind(&self) -> ProfileKind {
        ProfileKind::DebuggingAndDocumentation
    }

    fn create_scorer(&self) -> SectionScorer {
        SectionScorer::new()
    }

    fn prepare_sections(&self, sections: &[CdmSection]) -> Vec<CdmSection> {
        let mut prepared = Vec::with_capacity(sections.len());

        for sec in sections {
            let mut s = sec.clone();
            s.attributes.insert(
                "context_profile".to_string(),
                "debugging_and_documentation".to_string(),
            );

            let category = self.classify_section(&s);
            s.attributes.insert(
                "debugging_category".to_string(),
                category.as_str().to_string(),
            );

            match category {
                DebuggingCategory::ErrorDefinitionsAndPropagation => {
                    if self.config.include_error_definitions {
                        s.attributes
                            .insert("pin".to_string(), "must_include".to_string());
                        s.attributes
                            .insert("debugging_relevance".to_string(), "critical".to_string());
                    } else {
                        s.attributes
                            .insert("pin".to_string(), "deprioritized".to_string());
                    }
                }
                DebuggingCategory::ObservabilityAndLogging => {
                    if self.config.include_observability {
                        s.attributes
                            .insert("pin".to_string(), "must_include".to_string());
                        s.attributes
                            .insert("debugging_relevance".to_string(), "high".to_string());
                    } else {
                        s.attributes
                            .insert("pin".to_string(), "deprioritized".to_string());
                    }
                }
                DebuggingCategory::ApiContractsAndInterfaces => {
                    if self.config.include_api_contracts {
                        s.attributes
                            .insert("pin".to_string(), "must_include".to_string());
                        s.attributes
                            .insert("debugging_relevance".to_string(), "high".to_string());
                    } else {
                        s.attributes
                            .insert("pin".to_string(), "deprioritized".to_string());
                    }
                }
                DebuggingCategory::TroubleshootingDocsAndRunbooks => {
                    if self.config.include_troubleshooting_docs {
                        s.attributes.insert("pin".to_string(), "normal".to_string());
                        s.attributes
                            .insert("debugging_relevance".to_string(), "high".to_string());
                    } else {
                        s.attributes
                            .insert("pin".to_string(), "deprioritized".to_string());
                    }
                }
                DebuggingCategory::NegativeAndEdgeCaseTests => {
                    if self.config.include_negative_tests {
                        s.attributes.insert("pin".to_string(), "normal".to_string());
                        s.attributes
                            .insert("debugging_relevance".to_string(), "medium".to_string());
                    } else {
                        s.attributes
                            .insert("pin".to_string(), "deprioritized".to_string());
                    }
                }
                DebuggingCategory::ManifestsAndEnvironment => {
                    if self.config.include_manifests {
                        s.attributes.insert("pin".to_string(), "normal".to_string());
                        s.attributes
                            .insert("debugging_relevance".to_string(), "medium".to_string());
                    } else {
                        s.attributes
                            .insert("pin".to_string(), "deprioritized".to_string());
                    }
                }
                DebuggingCategory::InternalImplementation => {
                    if self.config.include_internal_implementation {
                        s.attributes
                            .insert("pin".to_string(), "deprioritized".to_string());
                        s.attributes
                            .insert("debugging_relevance".to_string(), "low".to_string());
                    } else {
                        s.attributes
                            .insert("pin".to_string(), "must_exclude".to_string());
                    }
                }
                DebuggingCategory::CosmeticOrUnrelatedAsset => {
                    if self.config.include_cosmetic_assets {
                        s.attributes
                            .insert("pin".to_string(), "deprioritized".to_string());
                        s.attributes
                            .insert("debugging_relevance".to_string(), "none".to_string());
                    } else {
                        s.attributes
                            .insert("pin".to_string(), "must_exclude".to_string());
                        s.attributes
                            .insert("debugging_relevance".to_string(), "excluded".to_string());
                    }
                }
            }

            prepared.push(s);
        }

        prepared
    }

    fn optimize(
        &self,
        document: &CdmDocument,
        budget: &ContextBudget,
    ) -> Result<ProfileOptimizationResult, ContextOptimizerError> {
        if document.sections.is_empty() {
            return Err(ContextOptimizerError::DocumentEmpty);
        }

        budget.validate()?;

        // 1. Prepare and tag sections with Debugging relevance and pinning
        let prepared_sections = self.prepare_sections(&document.sections);
        let mut prepared_doc = document.clone();
        prepared_doc.sections = prepared_sections;

        // 2. Configure budget optimizer
        let budget_config = BudgetOptimizationConfig::new(budget.clone())
            .with_allow_truncation(self.config.allow_truncation)
            .with_truncation_strategy(self.config.truncation_strategy)
            .with_min_retained_tokens(self.config.min_retained_tokens)
            .with_scorer(self.create_scorer());

        let optimizer = ContextBudgetOptimizer::new(budget_config);

        // 3. Optimize document to fit budget
        let (optimized_doc, plan) = optimizer.optimize(&prepared_doc)?;

        // 4. Calculate profile content distribution metrics
        let metrics = ProfileMetrics::from_sections(
            &optimized_doc.sections,
            ProfileKind::DebuggingAndDocumentation,
        );

        Ok(ProfileOptimizationResult {
            profile_kind: ProfileKind::DebuggingAndDocumentation,
            document: optimized_doc,
            plan,
            metrics,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::BTreeMap;
    use web_repo_to_ai_cdm::{
        CompletenessReport, DocumentMetadata, ExtractionStatus, HierarchyNode, RevisionInfo,
        SectionKind, SourceIdentity, SourceType,
    };
    use web_repo_to_ai_core::DocumentId;

    fn make_test_doc(sections: Vec<CdmSection>) -> CdmDocument {
        CdmDocument {
            schema_version: "1.0.0".to_string(),
            document_id: DocumentId::new(),
            source_identity: SourceIdentity::new(
                "https://github.com/org/repo",
                "https://github.com/org/repo",
            ),
            source_type: SourceType::GitHubPublic,
            canonical_url: "https://github.com/org/repo".to_string(),
            retrieved_at: Utc::now(),
            revision: RevisionInfo::default(),
            content_type: "application/vnd.github.v3+json".to_string(),
            metadata: DocumentMetadata::default(),
            hierarchy: HierarchyNode::root("root"),
            sections,
            relationships: Vec::new(),
            citations: Vec::new(),
            extraction_status: ExtractionStatus::Completed,
            warnings: Vec::new(),
            exclusions: Vec::new(),
            completeness: CompletenessReport::default(),
        }
    }

    fn make_section(
        id: &str,
        path: &str,
        kind: SectionKind,
        body: &str,
        attrs: Vec<(&str, &str)>,
    ) -> CdmSection {
        let mut attributes = BTreeMap::new();
        for (k, v) in attrs {
            attributes.insert(k.to_string(), v.to_string());
        }

        CdmSection {
            section_id: id.to_string(),
            parent_id: None,
            title: Some(path.to_string()),
            kind,
            original_path: Some(path.to_string()),
            source_url: format!("https://github.com/org/repo/blob/main/{path}"),
            line_range: None,
            byte_range: None,
            language: Some("rust".to_string()),
            content_hash: CdmSection::compute_hash(body),
            body: body.to_string(),
            token_metrics: None,
            attributes,
        }
    }

    #[test]
    fn test_debugging_classification_and_pinning() {
        let error_sec = make_section(
            "s_err",
            "crates/context-optimizer/src/error.rs",
            SectionKind::CodeFile,
            "#[derive(thiserror::Error, Debug)]\npub enum ContextOptimizerError { InvalidBudget }",
            vec![],
        );

        let obs_sec = make_section(
            "s_obs",
            "crates/observability/src/logger.rs",
            SectionKind::CodeFile,
            "pub fn init_subscriber() { tracing::info!(\"initialized\"); }",
            vec![],
        );

        let api_sec = make_section(
            "s_api",
            "services/api/src/routes/convert.rs",
            SectionKind::CodeFile,
            "pub async fn convert_handler() -> Result<Json<Response>, StatusCode> { Ok(Json(Response)) }",
            vec![("code_nature", "interface_or_type_definition")],
        );

        let runbook_sec = make_section(
            "s_runbook",
            "docs/TROUBLESHOOTING.md",
            SectionKind::Prose,
            "# Troubleshooting Guide\nWhen rate limit occurs, inspect header.",
            vec![],
        );

        let ui_sec = make_section(
            "s_ui",
            "apps/web/src/lib/components/Button.svelte",
            SectionKind::CodeFile,
            "<button class=\"bg-blue-500\">Click me</button>",
            vec![],
        );

        let doc = make_test_doc(vec![error_sec, obs_sec, api_sec, runbook_sec, ui_sec]);
        let profile = DebuggingProfile::default();
        let prepared = profile.prepare_sections(&doc.sections);

        assert_eq!(prepared.len(), 5);

        // Error definitions must be must_include
        assert_eq!(
            prepared[0].attributes.get("pin").map(|s| s.as_str()),
            Some("must_include")
        );
        assert_eq!(
            prepared[0]
                .attributes
                .get("debugging_category")
                .map(|s| s.as_str()),
            Some("error_definitions_and_propagation")
        );

        // Observability must be must_include
        assert_eq!(
            prepared[1].attributes.get("pin").map(|s| s.as_str()),
            Some("must_include")
        );
        assert_eq!(
            prepared[1]
                .attributes
                .get("debugging_category")
                .map(|s| s.as_str()),
            Some("observability_and_logging")
        );

        // API contracts must be must_include
        assert_eq!(
            prepared[2].attributes.get("pin").map(|s| s.as_str()),
            Some("must_include")
        );
        assert_eq!(
            prepared[2]
                .attributes
                .get("debugging_category")
                .map(|s| s.as_str()),
            Some("api_contracts_and_interfaces")
        );

        // Runbooks must be normal priority
        assert_eq!(
            prepared[3].attributes.get("pin").map(|s| s.as_str()),
            Some("normal")
        );
        assert_eq!(
            prepared[3]
                .attributes
                .get("debugging_category")
                .map(|s| s.as_str()),
            Some("troubleshooting_docs_and_runbooks")
        );

        // UI button must be must_exclude
        assert_eq!(
            prepared[4].attributes.get("pin").map(|s| s.as_str()),
            Some("must_exclude")
        );
        assert_eq!(
            prepared[4]
                .attributes
                .get("debugging_category")
                .map(|s| s.as_str()),
            Some("cosmetic_or_unrelated_asset")
        );
    }
}
