//! # General AI Context Profile
//!
//! Provides a balanced, high-signal conceptual and architectural context package
//! optimized for general AI reasoning, code comprehension, and high-level query answering.

use super::{ContextProfile, ProfileKind, ProfileMetrics, ProfileOptimizationResult};
use crate::budget::ContextBudget;
use crate::error::ContextOptimizerError;
use crate::optimizer::{BudgetOptimizationConfig, ContextBudgetOptimizer, TruncationStrategy};
use crate::priority::SectionScorer;
use serde::{Deserialize, Serialize};
use web_repo_to_ai_cdm::{CdmDocument, CdmSection};

/// Configuration options for the General AI context profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneralAiConfig {
    /// Whether to include high-level prose documentation and architecture guides.
    pub include_documentation: bool,
    /// Whether to include package manifests, configurations, and repository tree indices.
    pub include_manifests: bool,
    /// Whether to include core implementation source code.
    pub include_core_code: bool,
    /// Whether to include test files and integration test fixtures (default: false for General AI).
    pub include_tests: bool,
    /// Whether to include benchmark harnesses and performance suites (default: false).
    pub include_benchmarks: bool,
    /// Whether individual sections may be truncated to fit remaining headroom.
    pub allow_truncation: bool,
    /// Boundary truncation strategy to use when shortening large files.
    pub truncation_strategy: TruncationStrategy,
    /// Minimum retained token floor required to justify truncating a section.
    pub min_retained_tokens: usize,
}

impl Default for GeneralAiConfig {
    fn default() -> Self {
        Self {
            include_documentation: true,
            include_manifests: true,
            include_core_code: true,
            include_tests: false,
            include_benchmarks: false,
            allow_truncation: true,
            truncation_strategy: TruncationStrategy::HeadAndTail,
            min_retained_tokens: 60,
        }
    }
}

impl GeneralAiConfig {
    /// Creates a default General AI profile configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether test files may be included in the context package.
    #[must_use]
    pub const fn with_include_tests(mut self, include: bool) -> Self {
        self.include_tests = include;
        self
    }

    /// Sets whether benchmark suites may be included.
    #[must_use]
    pub const fn with_include_benchmarks(mut self, include: bool) -> Self {
        self.include_benchmarks = include;
        self
    }

    /// Sets whether boundary truncation is allowed.
    #[must_use]
    pub const fn with_allow_truncation(mut self, allow: bool) -> Self {
        self.allow_truncation = allow;
        self
    }

    /// Sets the truncation strategy.
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

/// The General AI context profile engine.
#[derive(Debug, Clone, Default)]
pub struct GeneralAiProfile {
    config: GeneralAiConfig,
}

impl GeneralAiProfile {
    /// Creates a new General AI profile with custom configuration.
    #[must_use]
    pub const fn new(config: GeneralAiConfig) -> Self {
        Self { config }
    }

    /// Returns a reference to the active configuration.
    #[must_use]
    pub const fn config(&self) -> &GeneralAiConfig {
        &self.config
    }
}

impl ContextProfile for GeneralAiProfile {
    fn profile_kind(&self) -> ProfileKind {
        ProfileKind::GeneralAi
    }

    fn create_scorer(&self) -> SectionScorer {
        SectionScorer::new()
    }

    fn prepare_sections(&self, sections: &[CdmSection]) -> Vec<CdmSection> {
        let mut prepared = Vec::with_capacity(sections.len());

        for sec in sections {
            let mut s = sec.clone();
            s.attributes
                .insert("context_profile".to_string(), "general_ai".to_string());

            let path = s.original_path.as_deref().unwrap_or("");
            let filename = path.rsplit('/').next().unwrap_or(path).to_ascii_lowercase();

            let is_test = s.attributes.get("is_test").is_some_and(|v| v == "true")
                || path.contains("/tests/")
                || path.contains("/test/")
                || filename.contains("test")
                || filename.contains("spec");

            let is_benchmark = path.contains("/benches/")
                || path.contains("/benchmarks/")
                || filename.contains("bench");

            let is_generated = s
                .attributes
                .get("is_generated")
                .is_some_and(|v| v == "true")
                || filename.ends_with(".min.js")
                || filename.ends_with(".min.css")
                || filename.ends_with(".lock")
                || filename.contains(".generated.");

            // Tag pinning based on General AI policy
            if is_generated || (is_benchmark && !self.config.include_benchmarks) {
                s.attributes
                    .insert("pin".to_string(), "must_exclude".to_string());
            } else if is_test {
                if !self.config.include_tests {
                    s.attributes
                        .insert("pin".to_string(), "deprioritized".to_string());
                }
            } else if filename == "readme.md"
                || filename == "readme"
                || filename == "architecture.md"
                || s.kind == web_repo_to_ai_cdm::SectionKind::Manifest
            {
                // Ensure prominent architectural sections are pinned for inclusion
                if !s.attributes.contains_key("pin") {
                    s.attributes
                        .insert("pin".to_string(), "must_include".to_string());
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

        // 1. Prepare and tag sections with General AI relevance and pinning
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
        let metrics =
            ProfileMetrics::from_sections(&optimized_doc.sections, ProfileKind::GeneralAi);

        Ok(ProfileOptimizationResult {
            profile_kind: ProfileKind::GeneralAi,
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
    fn test_general_ai_profile_optimization() {
        let s_readme = make_section(
            "s1",
            "README.md",
            SectionKind::Prose,
            "# Overview\nUniversal AI context generator.",
            vec![],
        );
        let s_manifest = make_section(
            "s2",
            "Cargo.toml",
            SectionKind::Manifest,
            "[package]\nname = \"app\"\n",
            vec![],
        );
        let s_core = make_section(
            "s3",
            "src/lib.rs",
            SectionKind::CodeFile,
            "pub fn process() {}\n",
            vec![("code_nature", "core_implementation")],
        );
        let s_test = make_section(
            "s4",
            "tests/large_test.rs",
            SectionKind::CodeFile,
            &"#[test] fn t() {}\n".repeat(50),
            vec![("is_test", "true")],
        );

        let doc = make_test_doc(vec![s_readme, s_manifest, s_core, s_test]);

        let profile = GeneralAiProfile::default();
        let budget = ContextBudget::new(300, 50, 40, 10); // available = 200

        let result = profile
            .optimize(&doc, &budget)
            .expect("profile optimization succeeds");

        assert_eq!(result.profile_kind, ProfileKind::GeneralAi);
        assert!(result.plan.is_within_budget());
        assert!(result.metrics.documentation_tokens > 0);
        assert!(result.metrics.core_code_tokens > 0);
        assert!(result.metrics.manifest_tokens > 0);

        // README and Cargo.toml must be included
        assert!(result
            .document
            .sections
            .iter()
            .any(|s| s.original_path.as_deref() == Some("README.md")));
        assert!(result
            .document
            .sections
            .iter()
            .any(|s| s.original_path.as_deref() == Some("Cargo.toml")));
        assert!(result
            .document
            .sections
            .iter()
            .any(|s| s.original_path.as_deref() == Some("src/lib.rs")));

        // Test section should be deprioritized or excluded under constrained budget
        let md = result.to_markdown_summary();
        assert!(md.contains("general_ai"));
        assert!(md.contains("Token Distribution by Category"));
    }
}
