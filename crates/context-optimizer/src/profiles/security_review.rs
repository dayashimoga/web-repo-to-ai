//! # Security Review Context Profile
//!
//! Tailors context specifically for application security reviews, penetration testing preparation,
//! threat modeling, and vulnerability audits:
//! - Pinned inclusion of authentication, authorization, session, and credential management
//! - Pinned inclusion of cryptographic operations, hashing, signature verification, and secret handling
//! - Prioritized inclusion of network attack surface, API routes, handlers, and external fetchers
//! - Prioritized inclusion of input validation, deserialization, and SSRF/injection defenses
//! - Prioritized inclusion of security policies (`SECURITY.md`), security workflows, and `deny.toml`
//! - Prioritized inclusion of security-centric unit and integration tests
//! - Suppression of benign presentation components (CSS, styling, UI layouts) and non-security docs

use super::{ContextProfile, ProfileKind, ProfileMetrics, ProfileOptimizationResult};
use crate::budget::ContextBudget;
use crate::error::ContextOptimizerError;
use crate::optimizer::{BudgetOptimizationConfig, ContextBudgetOptimizer, TruncationStrategy};
use crate::priority::SectionScorer;
use serde::{Deserialize, Serialize};
use web_repo_to_ai_cdm::{CdmDocument, CdmSection, SectionKind};

/// Security classification for content sections evaluated in a security audit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityCategory {
    /// Authentication, authorization, tokens, sessions, credentials, and RBAC.
    AuthAndAccessControl,
    /// Cryptography, encryption, key management, hashing, signatures, and secrets.
    CryptographyAndSecrets,
    /// Network attack surface, API routes, handlers, webhooks, and external fetchers.
    AttackSurfaceAndEndpoints,
    /// Input validation, deserialization, sanitization, and SSRF/injection defenses.
    InputValidationAndSanitizers,
    /// Security policies, advisories, vulnerability reporting (`SECURITY.md`).
    SecurityPolicyAndDocs,
    /// Security lint configurations, dependency manifests, and container/sandbox policies.
    SecurityManifests,
    /// Security-focused unit and integration tests (auth, isolation, permission, token tests).
    SecurityTests,
    /// General implementation code not specifically in the critical security path.
    GeneralImplementation,
    /// Benign presentation, UI templates, styling, layout, marketing copy.
    BenignPresentationOrAsset,
}

impl SecurityCategory {
    /// Returns a human-readable identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AuthAndAccessControl => "auth_and_access_control",
            Self::CryptographyAndSecrets => "cryptography_and_secrets",
            Self::AttackSurfaceAndEndpoints => "attack_surface_and_endpoints",
            Self::InputValidationAndSanitizers => "input_validation_and_sanitizers",
            Self::SecurityPolicyAndDocs => "security_policy_and_docs",
            Self::SecurityManifests => "security_manifests",
            Self::SecurityTests => "security_tests",
            Self::GeneralImplementation => "general_implementation",
            Self::BenignPresentationOrAsset => "benign_presentation_or_asset",
        }
    }

    /// Whether this category represents a high-priority security surface.
    #[must_use]
    pub const fn is_security_critical(self) -> bool {
        matches!(
            self,
            Self::AuthAndAccessControl
                | Self::CryptographyAndSecrets
                | Self::AttackSurfaceAndEndpoints
                | Self::InputValidationAndSanitizers
                | Self::SecurityPolicyAndDocs
        )
    }
}

/// Configuration options for the Security Review context profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityReviewConfig {
    /// Whether to prioritize authentication, authorization, and crypto code (default: true).
    pub include_auth_and_crypto: bool,
    /// Whether to prioritize API routes, request handlers, and network attack surface (default: true).
    pub include_attack_surface: bool,
    /// Whether to prioritize input validation, deserialization, and URL sanitization (default: true).
    pub include_input_validation: bool,
    /// Whether to include security policy documentation (`SECURITY.md`) (default: true).
    pub include_security_docs: bool,
    /// Whether to include security manifests and dependency configurations (default: true).
    pub include_security_manifests: bool,
    /// Whether to include security-focused test suites (default: true).
    pub include_security_tests: bool,
    /// Whether to include benign presentation and UI components (default: false).
    pub include_benign_components: bool,
    /// Explicit paths pinned for mandatory inclusion.
    pub pinned_paths: Vec<String>,
    /// Custom security keywords that elevate section priority.
    pub custom_target_keywords: Vec<String>,
    /// Whether individual sections may be truncated to fit remaining headroom.
    pub allow_truncation: bool,
    /// Boundary truncation strategy to use when shortening large files.
    pub truncation_strategy: TruncationStrategy,
    /// Minimum retained token floor required to justify truncating a section.
    pub min_retained_tokens: usize,
}

impl Default for SecurityReviewConfig {
    fn default() -> Self {
        Self {
            include_auth_and_crypto: true,
            include_attack_surface: true,
            include_input_validation: true,
            include_security_docs: true,
            include_security_manifests: true,
            include_security_tests: true,
            include_benign_components: false,
            pinned_paths: Vec::new(),
            custom_target_keywords: Vec::new(),
            allow_truncation: true,
            truncation_strategy: TruncationStrategy::HeadAndTail,
            min_retained_tokens: 50,
        }
    }
}

impl SecurityReviewConfig {
    /// Creates a new default Security Review profile configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets explicit file paths that must be pinned for inclusion.
    #[must_use]
    pub fn with_pinned_paths(mut self, paths: Vec<String>) -> Self {
        self.pinned_paths = paths;
        self
    }

    /// Sets custom security keywords that trigger elevated prioritization.
    #[must_use]
    pub fn with_custom_target_keywords(mut self, keywords: Vec<String>) -> Self {
        self.custom_target_keywords = keywords;
        self
    }

    /// Sets whether benign presentation and UI components are included.
    #[must_use]
    pub const fn with_include_benign_components(mut self, include: bool) -> Self {
        self.include_benign_components = include;
        self
    }

    /// Sets whether security test suites are included.
    #[must_use]
    pub const fn with_include_security_tests(mut self, include: bool) -> Self {
        self.include_security_tests = include;
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

/// The Security Review context profile engine.
#[derive(Debug, Clone, Default)]
pub struct SecurityReviewProfile {
    config: SecurityReviewConfig,
}

impl SecurityReviewProfile {
    /// Creates a new Security Review profile with custom configuration.
    #[must_use]
    pub const fn new(config: SecurityReviewConfig) -> Self {
        Self { config }
    }

    /// Returns a reference to the active configuration.
    #[must_use]
    pub const fn config(&self) -> &SecurityReviewConfig {
        &self.config
    }

    /// Classifies a [`CdmSection`] into a [`SecurityCategory`].
    #[must_use]
    pub fn classify_section(&self, sec: &CdmSection) -> SecurityCategory {
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
            return SecurityCategory::AuthAndAccessControl;
        }

        // 2. Check security policy and documentation (e.g. SECURITY.md)
        if filename.contains("security.md")
            || filename.contains("vulnerability")
            || filename.contains("threat_model")
            || normalized_path.contains("/security/") && sec.kind == SectionKind::Prose
        {
            return SecurityCategory::SecurityPolicyAndDocs;
        }

        // 3. Check custom target keywords
        if self.config.custom_target_keywords.iter().any(|kw| {
            let n_kw = kw.to_ascii_lowercase();
            normalized_path.contains(&n_kw)
                || sec
                    .title
                    .as_deref()
                    .is_some_and(|t| t.to_ascii_lowercase().contains(&n_kw))
                || sec.body.to_ascii_lowercase().contains(&n_kw)
        }) {
            return SecurityCategory::AuthAndAccessControl;
        }

        // 4. Check security tests
        let is_test = sec.attributes.get("is_test").is_some_and(|v| v == "true")
            || normalized_path.contains("/tests/")
            || normalized_path.contains("/test/")
            || filename.contains("test")
            || filename.contains("spec");

        if is_test {
            let is_security_test = normalized_path.contains("security")
                || normalized_path.contains("ssrf")
                || normalized_path.contains("isolation")
                || normalized_path.contains("permission")
                || normalized_path.contains("unauthorized")
                || normalized_path.contains("leakage")
                || normalized_path.contains("token")
                || normalized_path.contains("auth")
                || normalized_path.contains("crypto")
                || normalized_path.contains("csrf")
                || filename.contains("sec_")
                || sec.body.contains("AuthorizationRecoveryPlan")
                || sec.body.contains("constant_time_eq")
                || sec.body.contains("InsufficientPermissions");

            if is_security_test {
                return SecurityCategory::SecurityTests;
            }
        }

        // 5. Check authentication & authorization
        let is_auth = normalized_path.contains("auth")
            || normalized_path.contains("oauth")
            || normalized_path.contains("token")
            || normalized_path.contains("session")
            || normalized_path.contains("jwt")
            || normalized_path.contains("rbac")
            || normalized_path.contains("permission")
            || normalized_path.contains("credential")
            || normalized_path.contains("login")
            || normalized_path.contains("password")
            || sec
                .attributes
                .get("security_focus")
                .is_some_and(|v| v == "auth");

        if is_auth && !is_test {
            return SecurityCategory::AuthAndAccessControl;
        }

        // 6. Check cryptography and secrets
        let is_crypto = normalized_path.contains("crypto")
            || normalized_path.contains("aes")
            || normalized_path.contains("rsa")
            || normalized_path.contains("blake3")
            || normalized_path.contains("sha")
            || normalized_path.contains("hash")
            || normalized_path.contains("encrypt")
            || normalized_path.contains("decrypt")
            || normalized_path.contains("secret")
            || normalized_path.contains("signature")
            || normalized_path.contains("redact")
            || normalized_path.contains("mask")
            || normalized_path.contains("vault")
            || normalized_path.contains("certificate")
            || normalized_path.contains("tls")
            || sec
                .attributes
                .get("security_focus")
                .is_some_and(|v| v == "crypto");

        if is_crypto && !is_test {
            return SecurityCategory::CryptographyAndSecrets;
        }

        // 7. Check attack surface & network endpoints
        let is_attack_surface = normalized_path.contains("url-security")
            || normalized_path.contains("url_security")
            || normalized_path.contains("routes")
            || normalized_path.contains("router")
            || normalized_path.contains("handlers")
            || normalized_path.contains("handler")
            || normalized_path.contains("endpoint")
            || normalized_path.contains("controller")
            || normalized_path.contains("middleware")
            || normalized_path.contains("webhook")
            || normalized_path.contains("fetcher")
            || normalized_path.contains("ssrf")
            || normalized_path.contains("proxy")
            || normalized_path.contains("cors")
            || normalized_path.contains("rate_limit")
            || normalized_path.contains("api/")
            || sec
                .attributes
                .get("security_focus")
                .is_some_and(|v| v == "network");

        if is_attack_surface && !is_test {
            return SecurityCategory::AttackSurfaceAndEndpoints;
        }

        // 8. Check input validation and sanitization
        let is_validation = normalized_path.contains("validation")
            || normalized_path.contains("validator")
            || normalized_path.contains("sanitizer")
            || normalized_path.contains("sanitize")
            || normalized_path.contains("escape")
            || normalized_path.contains("schema_validator")
            || sec
                .attributes
                .get("security_focus")
                .is_some_and(|v| v == "validation");

        if is_validation && !is_test {
            return SecurityCategory::InputValidationAndSanitizers;
        }

        // 9. Check security manifests and policies
        let is_manifest = sec.kind == SectionKind::Manifest
            || filename == "deny.toml"
            || filename == "audit.toml"
            || filename.contains("containerfile")
            || filename.contains("dockerfile")
            || normalized_path.contains(".github/workflows/security")
            || normalized_path.contains(".github/workflows/codeql");

        if is_manifest {
            return SecurityCategory::SecurityManifests;
        }

        // 10. Check benign presentation and styling components
        let is_benign = normalized_path.contains("apps/web")
            || normalized_path.contains("/ui/")
            || normalized_path.starts_with("ui/")
            || normalized_path.contains("/components/")
            || normalized_path.contains("tokens.css")
            || normalized_path.contains("tailwind")
            || normalized_path.contains("typography")
            || normalized_path.contains("color-utils")
            || normalized_path.contains("button.svelte")
            || normalized_path.contains("heading.svelte")
            || normalized_path.contains("modal.svelte")
            || filename.ends_with(".css")
            || filename.ends_with(".svg")
            || filename.ends_with(".png")
            || filename.ends_with(".ico")
            || filename.contains("changelog")
            || filename == "roadmap.md"
            || filename == "contributing.md"
            || filename == "design_system.md";

        if is_benign {
            return SecurityCategory::BenignPresentationOrAsset;
        }

        if is_test {
            return SecurityCategory::BenignPresentationOrAsset;
        }

        SecurityCategory::GeneralImplementation
    }
}

impl ContextProfile for SecurityReviewProfile {
    fn profile_kind(&self) -> ProfileKind {
        ProfileKind::SecurityReview
    }

    fn create_scorer(&self) -> SectionScorer {
        SectionScorer::new()
    }

    fn prepare_sections(&self, sections: &[CdmSection]) -> Vec<CdmSection> {
        let mut prepared = Vec::with_capacity(sections.len());

        for sec in sections {
            let mut s = sec.clone();
            s.attributes
                .insert("context_profile".to_string(), "security_review".to_string());

            let category = self.classify_section(&s);
            s.attributes.insert(
                "security_category".to_string(),
                category.as_str().to_string(),
            );

            match category {
                SecurityCategory::AuthAndAccessControl
                | SecurityCategory::CryptographyAndSecrets => {
                    if self.config.include_auth_and_crypto {
                        s.attributes
                            .insert("pin".to_string(), "must_include".to_string());
                        s.attributes
                            .insert("security_relevance".to_string(), "critical".to_string());
                    } else {
                        s.attributes
                            .insert("pin".to_string(), "deprioritized".to_string());
                    }
                }
                SecurityCategory::SecurityPolicyAndDocs => {
                    if self.config.include_security_docs {
                        s.attributes
                            .insert("pin".to_string(), "must_include".to_string());
                        s.attributes
                            .insert("security_relevance".to_string(), "critical".to_string());
                    } else {
                        s.attributes
                            .insert("pin".to_string(), "must_exclude".to_string());
                    }
                }
                SecurityCategory::AttackSurfaceAndEndpoints => {
                    if self.config.include_attack_surface {
                        s.attributes.insert("pin".to_string(), "normal".to_string());
                        s.attributes
                            .insert("security_relevance".to_string(), "high".to_string());
                    } else {
                        s.attributes
                            .insert("pin".to_string(), "deprioritized".to_string());
                    }
                }
                SecurityCategory::InputValidationAndSanitizers => {
                    if self.config.include_input_validation {
                        s.attributes.insert("pin".to_string(), "normal".to_string());
                        s.attributes
                            .insert("security_relevance".to_string(), "high".to_string());
                    } else {
                        s.attributes
                            .insert("pin".to_string(), "deprioritized".to_string());
                    }
                }
                SecurityCategory::SecurityManifests => {
                    if self.config.include_security_manifests {
                        s.attributes.insert("pin".to_string(), "normal".to_string());
                        s.attributes
                            .insert("security_relevance".to_string(), "medium".to_string());
                    } else {
                        s.attributes
                            .insert("pin".to_string(), "deprioritized".to_string());
                    }
                }
                SecurityCategory::SecurityTests => {
                    if self.config.include_security_tests {
                        s.attributes.insert("pin".to_string(), "normal".to_string());
                        s.attributes
                            .insert("security_relevance".to_string(), "medium".to_string());
                    } else {
                        s.attributes
                            .insert("pin".to_string(), "deprioritized".to_string());
                    }
                }
                SecurityCategory::GeneralImplementation => {
                    s.attributes
                        .insert("pin".to_string(), "deprioritized".to_string());
                    s.attributes
                        .insert("security_relevance".to_string(), "low".to_string());
                }
                SecurityCategory::BenignPresentationOrAsset => {
                    if self.config.include_benign_components {
                        s.attributes
                            .insert("pin".to_string(), "deprioritized".to_string());
                        s.attributes
                            .insert("security_relevance".to_string(), "none".to_string());
                    } else {
                        s.attributes
                            .insert("pin".to_string(), "must_exclude".to_string());
                        s.attributes
                            .insert("security_relevance".to_string(), "excluded".to_string());
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

        // 1. Prepare and tag sections with Security Review relevance and pinning
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
            ProfileMetrics::from_sections(&optimized_doc.sections, ProfileKind::SecurityReview);

        Ok(ProfileOptimizationResult {
            profile_kind: ProfileKind::SecurityReview,
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
    fn test_security_review_classification_and_pinning() {
        let auth_sec = make_section(
            "sec_auth",
            "crates/core/src/auth/token.rs",
            SectionKind::CodeFile,
            "pub fn verify_jwt(token: &str) -> bool { true }",
            vec![],
        );

        let crypto_sec = make_section(
            "sec_crypto",
            "crates/core/src/crypto/aes.rs",
            SectionKind::CodeFile,
            "pub fn encrypt_aes(key: &[u8], data: &[u8]) -> Vec<u8> { vec![] }",
            vec![],
        );

        let policy_sec = make_section(
            "sec_policy",
            "SECURITY.md",
            SectionKind::Prose,
            "# Security Policy\nReport vulnerabilities to security@example.com",
            vec![],
        );

        let ui_sec = make_section(
            "sec_ui",
            "apps/web/src/lib/components/Button.svelte",
            SectionKind::CodeFile,
            "<button class=\"bg-blue-500\">Click me</button>",
            vec![],
        );

        let doc = make_test_doc(vec![auth_sec, crypto_sec, policy_sec, ui_sec]);
        let profile = SecurityReviewProfile::default();
        let prepared = profile.prepare_sections(&doc.sections);

        assert_eq!(prepared.len(), 4);

        // Auth should be must_include
        assert_eq!(
            prepared[0].attributes.get("pin").map(|s| s.as_str()),
            Some("must_include")
        );
        assert_eq!(
            prepared[0]
                .attributes
                .get("security_category")
                .map(|s| s.as_str()),
            Some("auth_and_access_control")
        );

        // Crypto should be must_include
        assert_eq!(
            prepared[1].attributes.get("pin").map(|s| s.as_str()),
            Some("must_include")
        );
        assert_eq!(
            prepared[1]
                .attributes
                .get("security_category")
                .map(|s| s.as_str()),
            Some("cryptography_and_secrets")
        );

        // Security policy doc should be must_include
        assert_eq!(
            prepared[2].attributes.get("pin").map(|s| s.as_str()),
            Some("must_include")
        );
        assert_eq!(
            prepared[2]
                .attributes
                .get("security_category")
                .map(|s| s.as_str()),
            Some("security_policy_and_docs")
        );

        // UI button should be must_exclude by default
        assert_eq!(
            prepared[3].attributes.get("pin").map(|s| s.as_str()),
            Some("must_exclude")
        );
        assert_eq!(
            prepared[3]
                .attributes
                .get("security_category")
                .map(|s| s.as_str()),
            Some("benign_presentation_or_asset")
        );
    }
}
