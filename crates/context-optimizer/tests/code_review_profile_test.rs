//! # Integration Tests for Code Review Context Profile (Subphase 7.3.2)
//!
//! Validates pinned modified file inclusion, associated test prioritization,
//! unmodified documentation suppression, boundary truncation, and telemetry serialization.

use chrono::Utc;
use std::collections::BTreeMap;
use web_repo_to_ai_cdm::{
    CdmDocument, CdmSection, CompletenessReport, DocumentMetadata, ExtractionStatus, HierarchyNode,
    RevisionInfo, SectionKind, SourceIdentity, SourceType,
};
use web_repo_to_ai_context_optimizer::{
    apply_code_review_profile, apply_code_review_profile_for_model, CodeReviewConfig,
    CodeReviewProfile, ContextBudget, ContextProfile, ProfileKind, ProfileOptimizationResult,
};
use web_repo_to_ai_core::DocumentId;
use web_repo_to_ai_tokenizer::TargetModel;

/// Helper to create a test document with initial metadata and sections.
fn make_test_doc(sections: Vec<CdmSection>) -> CdmDocument {
    CdmDocument {
        schema_version: "1.0.0".to_string(),
        document_id: DocumentId::new(),
        source_identity: SourceIdentity::new(
            "https://github.com/org/auth-engine",
            "https://github.com/org/auth-engine",
        ),
        source_type: SourceType::GitHubPublic,
        canonical_url: "https://github.com/org/auth-engine".to_string(),
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
        source_url: format!("https://github.com/org/auth-engine/blob/main/{path}"),
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
fn test_code_review_pins_changed_files_and_associated_tests() {
    let s_changed = make_section(
        "s1",
        "src/auth/jwt.rs",
        SectionKind::CodeFile,
        "pub struct JwtToken;\nimpl JwtToken {\n    pub fn verify(&self) -> bool { true }\n}\n",
        vec![("code_nature", "core_implementation")],
    );
    let s_test = make_section(
        "s2",
        "tests/jwt_test.rs",
        SectionKind::CodeFile,
        "#[test]\nfn test_jwt_verify() {\n    let token = JwtToken;\n    assert!(token.verify());\n}\n",
        vec![("is_test", "true"), ("code_nature", "test_or_fixture")],
    );
    let s_manifest = make_section(
        "s3",
        "Cargo.toml",
        SectionKind::Manifest,
        "[package]\nname = \"auth-engine\"\nversion = \"0.1.0\"\n",
        vec![],
    );
    let s_unrelated = make_section(
        "s4",
        "src/logger.rs",
        SectionKind::CodeFile,
        "pub fn log_event(msg: &str) { println!(\"{}\", msg); }\n",
        vec![("code_nature", "core_implementation")],
    );
    let s_doc = make_section(
        "s5",
        "README.md",
        SectionKind::Prose,
        "# Auth Engine\n\nGeneral authentication library.",
        vec![],
    );
    let s_changelog = make_section(
        "s6",
        "CHANGELOG.md",
        SectionKind::Prose,
        "# Changelog\n\nHistorical release notes.",
        vec![],
    );

    let doc = make_test_doc(vec![
        s_changed,
        s_test,
        s_manifest,
        s_unrelated,
        s_doc,
        s_changelog,
    ]);

    let changed_files = vec!["src/auth/jwt.rs".to_string()];
    let budget = ContextBudget::new(300, 50, 40, 10); // available = 200

    let result = apply_code_review_profile(&doc, &changed_files, &budget)
        .expect("code review optimization succeeds");

    assert_eq!(result.profile_kind, ProfileKind::CodeReview);
    assert!(result.plan.is_within_budget());
    assert!(result.plan.fitted_tokens <= 200);

    // Changed file must be included and tagged
    let changed_sec = result
        .document
        .sections
        .iter()
        .find(|s| s.original_path.as_deref() == Some("src/auth/jwt.rs"))
        .expect("changed file included");
    assert_eq!(
        changed_sec
            .attributes
            .get("is_changed_file")
            .map(|s| s.as_str()),
        Some("true")
    );

    // Associated test must be included
    let test_sec = result
        .document
        .sections
        .iter()
        .find(|s| s.original_path.as_deref() == Some("tests/jwt_test.rs"))
        .expect("associated test included");
    assert_eq!(
        test_sec
            .attributes
            .get("associated_test")
            .map(|s| s.as_str()),
        Some("true")
    );

    // Package manifest should be included
    assert!(result
        .document
        .sections
        .iter()
        .any(|s| s.original_path.as_deref() == Some("Cargo.toml")));

    // Unmodified prose docs must be excluded
    assert!(!result
        .document
        .sections
        .iter()
        .any(|s| s.original_path.as_deref() == Some("README.md")));
    assert!(!result
        .document
        .sections
        .iter()
        .any(|s| s.original_path.as_deref() == Some("CHANGELOG.md")));

    // Verify exclusions recorded
    assert!(result
        .document
        .exclusions
        .iter()
        .any(|e| e.original_path == "README.md"));
    assert!(result
        .document
        .exclusions
        .iter()
        .any(|e| e.original_path == "CHANGELOG.md"));
}

#[test]
fn test_code_review_model_targeting() {
    let s1 = make_section(
        "s1",
        "src/auth.rs",
        SectionKind::CodeFile,
        "pub fn auth() {}",
        vec![],
    );
    let s2 = make_section(
        "s2",
        "tests/auth_test.rs",
        SectionKind::CodeFile,
        "#[test] fn t() {}",
        vec![("is_test", "true")],
    );
    let doc = make_test_doc(vec![s1, s2]);

    let changed = vec!["src/auth.rs".to_string()];

    for model in [
        TargetModel::Gpt4o,
        TargetModel::Claude35Sonnet,
        TargetModel::Gemini15Flash,
        TargetModel::Llama3,
    ] {
        let result = apply_code_review_profile_for_model(&doc, &changed, model)
            .expect("model-targeted code review succeeds");

        assert_eq!(result.profile_kind, ProfileKind::CodeReview);
        assert!(result.plan.is_within_budget());
        assert_eq!(result.plan.target_model, Some(model));

        for sec in &result.document.sections {
            assert_eq!(
                sec.attributes.get("context_profile").map(|s| s.as_str()),
                Some("code_review")
            );
        }
    }
}

#[test]
fn test_code_review_tight_budget_truncation() {
    let long_code = (1..=250)
        .map(|i| format!("    pub fn handle_{i}() -> usize {{ {i} }}"))
        .collect::<Vec<_>>()
        .join("\n");

    let s_changed = make_section(
        "s1",
        "src/handler.rs",
        SectionKind::CodeFile,
        &format!("pub struct Handler;\nimpl Handler {{\n{long_code}\n}}\n"),
        vec![("code_nature", "core_implementation")],
    );

    let doc = make_test_doc(vec![s_changed]);
    let changed = vec!["src/handler.rs".to_string()];

    // Constrained budget with only 120 content tokens
    let budget = ContextBudget::new(200, 40, 30, 10);
    let config = CodeReviewConfig::new(changed)
        .with_allow_truncation(true)
        .with_min_retained_tokens(50);
    let profile = CodeReviewProfile::new(config);

    let result = profile
        .optimize(&doc, &budget)
        .expect("truncation succeeds");

    assert!(result.plan.is_within_budget());
    assert!(result.plan.fitted_tokens <= 120);
    assert_eq!(result.plan.truncated_section_count, 1);

    let truncated = &result.document.sections[0];
    assert!(truncated.verify_hash());
    assert!(truncated.body.contains("Truncated:"));
}

#[test]
fn test_code_review_zero_changed_files_fallback() {
    let s1 = make_section(
        "s1",
        "src/core.rs",
        SectionKind::CodeFile,
        "pub fn core() {}",
        vec![],
    );
    let s2 = make_section(
        "s2",
        "Cargo.toml",
        SectionKind::Manifest,
        "[package]\nname = \"pkg\"\n",
        vec![],
    );
    let s3 = make_section("s3", "README.md", SectionKind::Prose, "# Overview", vec![]);
    let doc = make_test_doc(vec![s1, s2, s3]);

    // Empty changed files slice
    let budget = ContextBudget::with_total_tokens(10_000);
    let result = apply_code_review_profile(&doc, &[], &budget).expect("optimization succeeds");

    assert_eq!(result.profile_kind, ProfileKind::CodeReview);
    assert!(result.plan.is_within_budget());
    // Core code and manifest included, unmodified documentation excluded
    assert!(result
        .document
        .sections
        .iter()
        .any(|s| s.original_path.as_deref() == Some("src/core.rs")));
    assert!(result
        .document
        .sections
        .iter()
        .any(|s| s.original_path.as_deref() == Some("Cargo.toml")));
    assert!(!result
        .document
        .sections
        .iter()
        .any(|s| s.original_path.as_deref() == Some("README.md")));
}

#[test]
fn test_code_review_report_and_json_serialization() {
    let s_changed = make_section(
        "s1",
        "src/diff.rs",
        SectionKind::CodeFile,
        "pub fn diff() {}",
        vec![],
    );
    let s_test = make_section(
        "s2",
        "tests/diff_test.rs",
        SectionKind::CodeFile,
        "#[test] fn t() {}",
        vec![("is_test", "true")],
    );
    let doc = make_test_doc(vec![s_changed, s_test]);

    let changed = vec!["src/diff.rs".to_string()];
    let budget = ContextBudget::from_model(TargetModel::Gpt4o);
    let result = apply_code_review_profile(&doc, &changed, &budget).expect("optimization succeeds");

    // Markdown summary checks
    let md = result.to_markdown_summary();
    assert!(md.contains("Context Profile Optimization: `code_review`"));
    assert!(md.contains("src/diff.rs"));
    assert!(md.contains("Token Distribution by Category"));

    // JSON roundtrip
    let json = result.to_json().expect("JSON serialization succeeds");
    let decoded =
        ProfileOptimizationResult::from_json(&json).expect("JSON deserialization succeeds");

    assert_eq!(result.profile_kind, decoded.profile_kind);
    assert_eq!(result.plan.fitted_tokens, decoded.plan.fitted_tokens);
    assert_eq!(
        result.metrics.core_code_tokens,
        decoded.metrics.core_code_tokens
    );
}
