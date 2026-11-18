//! # Code Review Context Profile
//!
//! Tailors context specifically for pull request review, patch evaluation, and diff analysis:
//! - Pinned inclusion of modified and changed files
//! - High prioritization of associated unit tests and regression test fixtures
//! - Inclusion of touched interface and type definitions
//! - Suppression of unrelated prose documentation, changelogs, and distant modules

use super::{ContextProfile, ProfileKind, ProfileMetrics, ProfileOptimizationResult};
use crate::budget::ContextBudget;
use crate::error::ContextOptimizerError;
use crate::optimizer::{BudgetOptimizationConfig, ContextBudgetOptimizer, TruncationStrategy};
use crate::priority::SectionScorer;
use serde::{Deserialize, Serialize};
use web_repo_to_ai_cdm::{CdmDocument, CdmSection, SectionKind};

/// Configuration options for the Code Review context profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeReviewConfig {
    /// List of file paths modified, added, or targeted for review.
    pub changed_files: Vec<String>,
    /// Whether to prioritize unit tests and fixtures associated with the changed code (default: true).
    pub include_associated_tests: bool,
    /// Whether to include package manifests and build configurations (default: true).
    pub include_manifests: bool,
    /// Whether to include interface and type definitions imported by changed code (default: true).
    pub include_interface_definitions: bool,
    /// Whether to include general unmodified prose documentation (default: false).
    pub include_unmodified_docs: bool,
    /// Whether individual sections may be truncated to fit remaining headroom.
    pub allow_truncation: bool,
    /// Boundary truncation strategy to use when shortening large files.
    pub truncation_strategy: TruncationStrategy,
    /// Minimum retained token floor required to justify truncating a section.
    pub min_retained_tokens: usize,
}

impl Default for CodeReviewConfig {
    fn default() -> Self {
        Self {
            changed_files: Vec::new(),
            include_associated_tests: true,
            include_manifests: true,
            include_interface_definitions: true,
            include_unmodified_docs: false,
            allow_truncation: true,
            truncation_strategy: TruncationStrategy::HeadAndTail,
            min_retained_tokens: 50,
        }
    }
}

impl CodeReviewConfig {
    /// Creates a new Code Review profile configuration for the specified changed files.
    #[must_use]
    pub fn new(changed_files: Vec<String>) -> Self {
        Self {
            changed_files,
            ..Default::default()
        }
    }

    /// Sets whether associated test files are prioritized.
    #[must_use]
    pub const fn with_include_associated_tests(mut self, include: bool) -> Self {
        self.include_associated_tests = include;
        self
    }

    /// Sets whether package manifests and build configs are included.
    #[must_use]
    pub const fn with_include_manifests(mut self, include: bool) -> Self {
        self.include_manifests = include;
        self
    }

    /// Sets whether interface and type definitions are included.
    #[must_use]
    pub const fn with_include_interface_definitions(mut self, include: bool) -> Self {
        self.include_interface_definitions = include;
        self
    }

    /// Sets whether unmodified prose documentation is included.
    #[must_use]
    pub const fn with_include_unmodified_docs(mut self, include: bool) -> Self {
        self.include_unmodified_docs = include;
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

/// The Code Review context profile engine.
#[derive(Debug, Clone, Default)]
pub struct CodeReviewProfile {
    config: CodeReviewConfig,
}

impl CodeReviewProfile {
    /// Creates a new Code Review profile with custom configuration.
    #[must_use]
    pub const fn new(config: CodeReviewConfig) -> Self {
        Self { config }
    }

    /// Creates a new Code Review profile targeting the given changed files.
    #[must_use]
    pub fn from_changed_files(changed_files: Vec<String>) -> Self {
        Self::new(CodeReviewConfig::new(changed_files))
    }

    /// Returns a reference to the active configuration.
    #[must_use]
    pub const fn config(&self) -> &CodeReviewConfig {
        &self.config
    }

    /// Determines whether a section path matches one of the specified changed files.
    #[must_use]
    pub fn is_changed_file(&self, path: &str) -> bool {
        if self.config.changed_files.is_empty() {
            return false;
        }

        let normalized_path = path.replace('\\', "/").to_ascii_lowercase();
        let filename = normalized_path
            .rsplit('/')
            .next()
            .unwrap_or(&normalized_path);

        self.config.changed_files.iter().any(|cf| {
            let n_cf = cf.replace('\\', "/").to_ascii_lowercase();
            let cf_filename = n_cf.rsplit('/').next().unwrap_or(&n_cf);

            normalized_path == n_cf
                || normalized_path.ends_with(&n_cf)
                || n_cf.ends_with(&normalized_path)
                || filename == cf_filename
        })
    }

    /// Determines whether a section path represents an associated test for a changed file.
    #[must_use]
    pub fn is_associated_test(&self, path: &str) -> bool {
        let normalized_path = path.replace('\\', "/").to_ascii_lowercase();
        let is_test_path = normalized_path.contains("/tests/")
            || normalized_path.contains("/test/")
            || normalized_path.contains("test")
            || normalized_path.contains("spec");

        if !is_test_path {
            return false;
        }

        if self.config.changed_files.is_empty() {
            return true;
        }

        // Check if test filename or directory matches any changed file stem
        self.config.changed_files.iter().any(|cf| {
            let n_cf = cf.replace('\\', "/").to_ascii_lowercase();
            let cf_filename = n_cf.rsplit('/').next().unwrap_or(&n_cf);
            let stem = cf_filename.split('.').next().unwrap_or(cf_filename);

            if stem.len() >= 3 {
                normalized_path.contains(stem)
            } else {
                false
            }
        })
    }
}

impl ContextProfile for CodeReviewProfile {
    fn profile_kind(&self) -> ProfileKind {
        ProfileKind::CodeReview
    }

    fn create_scorer(&self) -> SectionScorer {
        SectionScorer::new()
    }

    fn prepare_sections(&self, sections: &[CdmSection]) -> Vec<CdmSection> {
        let mut prepared = Vec::with_capacity(sections.len());

        for sec in sections {
            let mut s = sec.clone();
            s.attributes
                .insert("context_profile".to_string(), "code_review".to_string());

            let path = s.original_path.as_deref().unwrap_or("");
            let filename = path.rsplit('/').next().unwrap_or(path).to_ascii_lowercase();

            let is_changed = self.is_changed_file(path);
            let is_test = s.attributes.get("is_test").is_some_and(|v| v == "true")
                || self.is_associated_test(path);

            let is_generated = s
                .attributes
                .get("is_generated")
                .is_some_and(|v| v == "true")
                || filename.ends_with(".min.js")
                || filename.ends_with(".min.css")
                || filename.ends_with(".lock")
                || filename.contains(".generated.");

            let is_benchmark = path.contains("/benches/")
                || path.contains("/benchmarks/")
                || filename.contains("bench");

            // Apply Code Review prioritization policy
            if is_generated || is_benchmark {
                s.attributes
                    .insert("pin".to_string(), "must_exclude".to_string());
            } else if is_changed {
                // Modified files have absolute top priority
                s.attributes
                    .insert("pin".to_string(), "must_include".to_string());
                s.attributes
                    .insert("is_changed_file".to_string(), "true".to_string());
            } else if is_test {
                if self.config.include_associated_tests {
                    // Associated test files are prioritized for review verification
                    s.attributes.insert("pin".to_string(), "normal".to_string());
                    s.attributes
                        .insert("associated_test".to_string(), "true".to_string());
                } else {
                    s.attributes
                        .insert("pin".to_string(), "deprioritized".to_string());
                }
            } else if s.kind == SectionKind::Manifest {
                if self.config.include_manifests {
                    s.attributes.insert("pin".to_string(), "normal".to_string());
                } else {
                    s.attributes
                        .insert("pin".to_string(), "deprioritized".to_string());
                }
            } else if s.kind == SectionKind::Prose {
                // In code review, unmodified documentation is excluded unless specifically requested
                let is_doc_changed = is_changed;
                if !is_doc_changed && !self.config.include_unmodified_docs {
                    s.attributes
                        .insert("pin".to_string(), "must_exclude".to_string());
                } else {
                    s.attributes
                        .insert("pin".to_string(), "deprioritized".to_string());
                }
            } else {
                // Unchanged implementation files
                let code_nature = s.attributes.get("code_nature").map(|v| v.as_str());
                if code_nature == Some("interface_or_type_definition")
                    || code_nature == Some("data_or_schema")
                {
                    if self.config.include_interface_definitions {
                        s.attributes.insert("pin".to_string(), "normal".to_string());
                    } else {
                        s.attributes
                            .insert("pin".to_string(), "deprioritized".to_string());
                    }
                } else {
                    s.attributes
                        .insert("pin".to_string(), "deprioritized".to_string());
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

        // 1. Prepare and tag sections with Code Review relevance and pinning
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
            ProfileMetrics::from_sections(&optimized_doc.sections, ProfileKind::CodeReview);

        Ok(ProfileOptimizationResult {
            profile_kind: ProfileKind::CodeReview,
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
    fn test_code_review_changed_file_pinning() {
        let s_changed = make_section(
            "s1",
            "src/parser.rs",
            SectionKind::CodeFile,
            "pub fn parse() {}",
            vec![],
        );
        let s_test = make_section(
            "s2",
            "tests/parser_test.rs",
            SectionKind::CodeFile,
            "#[test] fn test_parse() {}",
            vec![("is_test", "true")],
        );
        let s_doc = make_section(
            "s3",
            "README.md",
            SectionKind::Prose,
            "# Documentation",
            vec![],
        );
        let s_other = make_section(
            "s4",
            "src/unrelated.rs",
            SectionKind::CodeFile,
            "pub fn unrelated() {}",
            vec![],
        );

        let doc = make_test_doc(vec![s_changed, s_test, s_doc, s_other]);

        let profile = CodeReviewProfile::from_changed_files(vec!["src/parser.rs".to_string()]);
        let budget = ContextBudget::new(300, 50, 40, 10); // available = 200

        let result = profile
            .optimize(&doc, &budget)
            .expect("code review optimization succeeds");

        assert_eq!(result.profile_kind, ProfileKind::CodeReview);
        assert!(result.plan.is_within_budget());

        // Changed file must be included and pinned
        let changed = result
            .document
            .sections
            .iter()
            .find(|s| s.original_path.as_deref() == Some("src/parser.rs"));
        assert!(changed.is_some());
        assert_eq!(
            changed
                .unwrap()
                .attributes
                .get("is_changed_file")
                .map(|s| s.as_str()),
            Some("true")
        );

        // Associated test should be included
        let test_sec = result
            .document
            .sections
            .iter()
            .find(|s| s.original_path.as_deref() == Some("tests/parser_test.rs"));
        assert!(test_sec.is_some());

        // Unmodified README should be excluded under Code Review policy
        assert!(!result
            .document
            .sections
            .iter()
            .any(|s| s.original_path.as_deref() == Some("README.md")));
    }
}
