//! # Integration Tests for Security Review Context Profile (Subphase 7.3.3)
//!
//! Validates authentication, crypto, attack surface, input validation, and security policy
//! prioritization, benign presentation component suppression, boundary truncation, and telemetry serialization.

use chrono::Utc;
use std::collections::BTreeMap;
use web_repo_to_ai_cdm::{
    CdmDocument, CdmSection, CompletenessReport, DocumentMetadata, ExtractionStatus, HierarchyNode,
    RevisionInfo, SectionKind, SourceIdentity, SourceType,
};
use web_repo_to_ai_context_optimizer::{
    apply_security_review_profile, apply_security_review_profile_for_model, ContextBudget,
    ContextProfile, ProfileKind, ProfileOptimizationResult, SecurityReviewConfig,
    SecurityReviewProfile,
};
use web_repo_to_ai_core::DocumentId;
use web_repo_to_ai_tokenizer::TargetModel;

/// Helper to create a test document with initial metadata and sections.
fn make_test_doc(sections: Vec<CdmSection>) -> CdmDocument {
    CdmDocument {
        schema_version: "1.0.0".to_string(),
        document_id: DocumentId::new(),
        source_identity: SourceIdentity::new(
            "https://github.com/org/security-vault",
            "https://github.com/org/security-vault",
        ),
        source_type: SourceType::GitHubPublic,
        canonical_url: "https://github.com/org/security-vault".to_string(),
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

/// Helper to create a section with specific attributes.
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
        source_url: format!("https://github.com/org/security-vault/blob/main/{path}"),
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
fn test_security_review_pins_auth_crypto_and_policy() {
    let s_auth = make_section(
        "s_auth",
        "crates/core/src/auth/token_manager.rs",
        SectionKind::CodeFile,
        "pub struct TokenManager;\nimpl TokenManager {\n    pub fn issue_token() -> String { \"token\".into() }\n}",
        vec![("code_nature", "core_implementation")],
    );

    let s_crypto = make_section(
        "s_crypto",
        "crates/core/src/crypto/aes_gcm.rs",
        SectionKind::CodeFile,
        "pub fn encrypt_aes_256_gcm(key: &[u8], plaintext: &[u8]) -> Vec<u8> {\n    vec![0u8; 32]\n}",
        vec![("code_nature", "core_implementation")],
    );

    let s_attack = make_section(
        "s_attack",
        "services/api/src/routes/auth_handler.rs",
        SectionKind::CodeFile,
        "pub async fn login_handler() -> &'static str {\n    \"authenticated\"\n}",
        vec![("code_nature", "core_implementation")],
    );

    let s_policy = make_section(
        "s_policy",
        "SECURITY.md",
        SectionKind::Prose,
        "# Security Policy\nTo report vulnerabilities, email security@example.com.",
        vec![],
    );

    let s_manifest = make_section(
        "s_manifest",
        "deny.toml",
        SectionKind::Manifest,
        "[advisories]\nvulnerability = \"deny\"\nunmaintained = \"warn\"",
        vec![],
    );

    let s_benign_ui = make_section(
        "s_ui",
        "apps/web/src/lib/components/Button.svelte",
        SectionKind::CodeFile,
        "<button class=\"px-4 py-2 bg-blue-600 text-white rounded\">Submit</button>",
        vec![("code_nature", "ui_component")],
    );

    let s_benign_doc = make_section(
        "s_changelog",
        "CHANGELOG.md",
        SectionKind::Prose,
        "# Changelog\nAll notable changes documented here.\n## [0.1.0] - Released",
        vec![],
    );

    let doc = make_test_doc(vec![
        s_auth,
        s_crypto,
        s_attack,
        s_policy,
        s_manifest,
        s_benign_ui,
        s_benign_doc,
    ]);

    let budget = ContextBudget::new(10000, 500, 1000, 200);
    let profile = SecurityReviewProfile::default();
    let result = profile
        .optimize(&doc, &budget)
        .expect("Optimization succeeds");

    assert_eq!(result.profile_kind, ProfileKind::SecurityReview);

    let included_paths: Vec<&str> = result
        .document
        .sections
        .iter()
        .filter_map(|s| s.original_path.as_deref())
        .collect();

    // Security-critical paths MUST be included
    assert!(included_paths.contains(&"crates/core/src/auth/token_manager.rs"));
    assert!(included_paths.contains(&"crates/core/src/crypto/aes_gcm.rs"));
    assert!(included_paths.contains(&"services/api/src/routes/auth_handler.rs"));
    assert!(included_paths.contains(&"SECURITY.md"));
    assert!(included_paths.contains(&"deny.toml"));

    // Benign presentation components and changelogs MUST be excluded
    assert!(!included_paths.contains(&"apps/web/src/lib/components/Button.svelte"));
    assert!(!included_paths.contains(&"CHANGELOG.md"));

    // Excluded sections must be properly audited in exclusions
    assert!(result
        .document
        .exclusions
        .iter()
        .any(|ex| ex.original_path == "apps/web/src/lib/components/Button.svelte"));
}

#[test]
fn test_security_review_elevates_security_tests_while_suppressing_unrelated_tests() {
    let s_sec_test = make_section(
        "s_sec_test",
        "tests/security_isolation_test.rs",
        SectionKind::CodeFile,
        "#[test]\nfn test_cross_tenant_token_isolation() {\n    assert!(true);\n}",
        vec![("is_test", "true")],
    );

    let s_ssrf_test = make_section(
        "s_ssrf_test",
        "crates/url-security/tests/ssrf_defense_test.rs",
        SectionKind::CodeFile,
        "#[test]\nfn test_loopback_ip_rejected() {\n    assert!(true);\n}",
        vec![("is_test", "true")],
    );

    let s_gui_test = make_section(
        "s_gui_test",
        "apps/web/tests/responsive_layout_test.ts",
        SectionKind::CodeFile,
        "test('responsive layout breakpoints', () => {\n    expect(true).toBe(true);\n});",
        vec![("is_test", "true")],
    );

    let doc = make_test_doc(vec![s_sec_test, s_ssrf_test, s_gui_test]);

    let budget = ContextBudget::new(10000, 500, 1000, 200);
    let profile = SecurityReviewProfile::default();
    let result = profile
        .optimize(&doc, &budget)
        .expect("Optimization succeeds");

    let included_paths: Vec<&str> = result
        .document
        .sections
        .iter()
        .filter_map(|s| s.original_path.as_deref())
        .collect();

    // Security tests are included
    assert!(included_paths.contains(&"tests/security_isolation_test.rs"));
    assert!(included_paths.contains(&"crates/url-security/tests/ssrf_defense_test.rs"));

    // Unrelated GUI tests are excluded
    assert!(!included_paths.contains(&"apps/web/tests/responsive_layout_test.ts"));
}

#[test]
fn test_security_review_model_targeting() {
    let s1 = make_section(
        "s1",
        "crates/core/src/auth/jwt.rs",
        SectionKind::CodeFile,
        "pub fn verify_jwt(token: &str) -> bool { token.len() > 10 }",
        vec![],
    );
    let s2 = make_section(
        "s2",
        "crates/core/src/crypto/signature.rs",
        SectionKind::CodeFile,
        "pub fn verify_signature(key: &[u8], sig: &[u8]) -> bool { true }",
        vec![],
    );

    let doc = make_test_doc(vec![s1, s2]);

    for model in [
        TargetModel::Gpt4o,
        TargetModel::Claude35Sonnet,
        TargetModel::Gemini15Pro,
        TargetModel::Llama3,
    ] {
        let result = apply_security_review_profile_for_model(&doc, model)
            .expect("Model optimization succeeds");

        assert_eq!(result.profile_kind, ProfileKind::SecurityReview);
        assert!(result.plan.is_within_budget());
        assert!(result.document.sections.len() >= 2);
    }
}

#[test]
fn test_security_review_tight_budget_truncation() {
    let large_auth_body = "pub fn handle_oauth_flow() {\n".to_string()
        + &"    // Security critical authentication and state validation logic\n    let verified = true;\n".repeat(30)
        + "}\n";

    let s_large_auth = make_section(
        "s_auth",
        "crates/core/src/auth/oauth.rs",
        SectionKind::CodeFile,
        &large_auth_body,
        vec![],
    );

    let doc = make_test_doc(vec![s_large_auth]);

    // Budget: 160 total tokens, with 50+30+10 headroom -> 70 usable tokens
    let budget = ContextBudget::new(160, 50, 30, 10);
    let profile = SecurityReviewProfile::default();
    let result = profile
        .optimize(&doc, &budget)
        .expect("Optimization succeeds");

    assert!(result.plan.is_within_budget());
    assert_eq!(result.document.sections.len(), 1);

    let section = &result.document.sections[0];
    assert!(section.body.contains("[Truncated:"));
}

#[test]
fn test_security_review_custom_keywords_and_pinned_paths() {
    let s_custom_kms = make_section(
        "s_kms",
        "crates/storage/src/hardware_security_module.rs",
        SectionKind::CodeFile,
        "pub fn fetch_master_key() -> Vec<u8> { vec![1, 2, 3] }",
        vec![],
    );

    let s_custom_kw = make_section(
        "s_kw",
        "crates/storage/src/custom_vault.rs",
        SectionKind::CodeFile,
        "pub struct CustomVaultSecretStorage;",
        vec![],
    );

    let doc = make_test_doc(vec![s_custom_kms, s_custom_kw]);

    let config = SecurityReviewConfig::new()
        .with_pinned_paths(vec!["hardware_security_module.rs".to_string()])
        .with_custom_target_keywords(vec!["CustomVault".to_string()]);

    let profile = SecurityReviewProfile::new(config);
    let budget = ContextBudget::new(10000, 500, 1000, 200);
    let result = profile
        .optimize(&doc, &budget)
        .expect("Optimization succeeds");

    assert_eq!(result.document.sections.len(), 2);
    let section1 = &result.document.sections[0];
    assert_eq!(
        section1
            .attributes
            .get("security_category")
            .map(|s| s.as_str()),
        Some("auth_and_access_control")
    );
}

#[test]
fn test_security_review_report_and_json_serialization() {
    let s_auth = make_section(
        "s_auth",
        "crates/core/src/auth.rs",
        SectionKind::CodeFile,
        "pub fn authenticate() -> bool { true }",
        vec![],
    );
    let s_policy = make_section(
        "s_policy",
        "SECURITY.md",
        SectionKind::Prose,
        "# Security Policy\nDetails here.",
        vec![],
    );

    let doc = make_test_doc(vec![s_auth, s_policy]);
    let budget = ContextBudget::new(5000, 300, 500, 100);
    let result = apply_security_review_profile(&doc, &budget).expect("Profile succeeds");

    // Markdown summary report and table validation
    let summary = result.to_markdown_summary();
    assert!(summary.contains("security_review"));
    assert!(summary.contains("Documentation & Prose"));
    assert!(summary.contains("Core Implementation Code"));
    assert!(summary.contains("Section Budget Decisions"));

    let markdown_table = result.metrics.to_markdown_table();
    assert!(markdown_table.contains("Documentation & Prose"));
    assert!(markdown_table.contains("Core Implementation Code"));

    // JSON serialization roundtrip
    let json = serde_json::to_string_pretty(&result).expect("JSON serialization succeeds");
    assert!(json.contains("security_review"));
    assert!(json.contains("profile_kind"));

    let deserialized: ProfileOptimizationResult =
        serde_json::from_str(&json).expect("JSON deserialization succeeds");
    assert_eq!(deserialized.profile_kind, ProfileKind::SecurityReview);
    assert_eq!(deserialized.document.sections.len(), 2);
}
