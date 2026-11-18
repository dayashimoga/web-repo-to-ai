//! # Integration Tests for General AI Context Profile (Subphase 7.3.1)
//!
//! Validates balanced architectural content distribution, test suppression,
//! model-specific context window targeting, boundary truncation, and telemetry serialization.

use chrono::Utc;
use std::collections::BTreeMap;
use web_repo_to_ai_cdm::{
    CdmDocument, CdmSection, CompletenessReport, DocumentMetadata, ExtractionStatus, HierarchyNode,
    RevisionInfo, SectionKind, SourceIdentity, SourceType,
};
use web_repo_to_ai_context_optimizer::{
    apply_general_ai_profile, apply_general_ai_profile_for_model, ContextBudget, ContextProfile,
    GeneralAiConfig, GeneralAiProfile, ProfileKind, ProfileOptimizationResult,
};
use web_repo_to_ai_core::DocumentId;
use web_repo_to_ai_tokenizer::TargetModel;

/// Helper to create a test document with initial metadata and sections.
fn make_test_doc(sections: Vec<CdmSection>) -> CdmDocument {
    CdmDocument {
        schema_version: "1.0.0".to_string(),
        document_id: DocumentId::new(),
        source_identity: SourceIdentity::new(
            "https://github.com/org/context-engine",
            "https://github.com/org/context-engine",
        ),
        source_type: SourceType::GitHubPublic,
        canonical_url: "https://github.com/org/context-engine".to_string(),
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
        source_url: format!("https://github.com/org/context-engine/blob/main/{path}"),
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
fn test_general_ai_balanced_content_distribution() {
    let s_readme = make_section(
        "s1",
        "README.md",
        SectionKind::Prose,
        "# Context Engine\n\nHigh-performance AI context generator and optimizer.",
        vec![],
    );
    let s_arch = make_section(
        "s2",
        "docs/ARCHITECTURE.md",
        SectionKind::Prose,
        "## System Architecture\n\nLayered pipeline with tokenizer adapters and profile engines.",
        vec![],
    );
    let s_manifest = make_section(
        "s3",
        "Cargo.toml",
        SectionKind::Manifest,
        "[package]\nname = \"context-engine\"\nversion = \"0.1.0\"\n",
        vec![],
    );
    let s_core = make_section(
        "s4",
        "src/engine.rs",
        SectionKind::CodeFile,
        "pub struct Engine;\nimpl Engine {\n    pub fn run(&self) {}\n}\n",
        vec![
            ("code_nature", "core_implementation"),
            ("information_density", "high"),
            ("signal_score", "0.95"),
        ],
    );
    let s_types = make_section(
        "s5",
        "src/model.rs",
        SectionKind::CodeFile,
        "pub struct Config {\n    pub max_tokens: usize,\n}\n",
        vec![("code_nature", "interface_or_type_definition")],
    );
    let s_test = make_section(
        "s6",
        "tests/integration_test.rs",
        SectionKind::CodeFile,
        &"#[test]\nfn test_workflow() { assert!(true); }\n".repeat(60),
        vec![("is_test", "true"), ("code_nature", "test_or_fixture")],
    );

    let doc = make_test_doc(vec![s_readme, s_arch, s_manifest, s_core, s_types, s_test]);

    // Budget allowing ~150 content tokens
    let budget = ContextBudget::new(250, 50, 40, 10);
    let result = apply_general_ai_profile(&doc, &budget).expect("general AI optimization succeeds");

    assert_eq!(result.profile_kind, ProfileKind::GeneralAi);
    assert!(result.plan.is_within_budget());
    assert!(result.plan.fitted_tokens <= 150);

    // Verify content balance
    assert!(result.metrics.documentation_tokens > 0);
    assert!(result.metrics.core_code_tokens > 0);
    assert!(result.metrics.manifest_tokens > 0);

    // Architectural assets must be retained
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
        .any(|s| s.original_path.as_deref() == Some("src/engine.rs")));

    // Large test suite must be excluded under tight budget
    assert!(result.plan.excluded_section_count >= 1);
    assert_eq!(result.document.exclusions.len(), 1);
    assert_eq!(
        result.document.exclusions[0].original_path,
        "tests/integration_test.rs"
    );
}

#[test]
fn test_general_ai_model_targeted_optimization() {
    let s1 = make_section("s1", "README.md", SectionKind::Prose, "# Overview", vec![]);
    let s2 = make_section(
        "s2",
        "src/lib.rs",
        SectionKind::CodeFile,
        "pub fn start() {}",
        vec![],
    );
    let doc = make_test_doc(vec![s1, s2]);

    for model in [
        TargetModel::Gpt4o,
        TargetModel::Claude35Sonnet,
        TargetModel::Gemini15Pro,
        TargetModel::Llama3,
    ] {
        let result = apply_general_ai_profile_for_model(&doc, model)
            .expect("model-targeted optimization succeeds");

        assert_eq!(result.profile_kind, ProfileKind::GeneralAi);
        assert!(result.plan.is_within_budget());
        assert_eq!(result.plan.target_model, Some(model));

        // Check profile attribute enrichment
        for sec in &result.document.sections {
            assert_eq!(
                sec.attributes.get("context_profile").map(|s| s.as_str()),
                Some("general_ai")
            );
        }
    }
}

#[test]
fn test_general_ai_test_inclusion_toggle() {
    let s_core = make_section(
        "s1",
        "src/core.rs",
        SectionKind::CodeFile,
        "pub fn core() {}\n",
        vec![("code_nature", "core_implementation")],
    );
    let s_test = make_section(
        "s2",
        "tests/unit_test.rs",
        SectionKind::CodeFile,
        "#[test] fn t() {}\n",
        vec![("is_test", "true")],
    );

    let doc = make_test_doc(vec![s_core, s_test]);

    // Test with default config (tests deprioritized)
    let default_profile = GeneralAiProfile::default();
    assert!(!default_profile.config().include_tests);

    // Test with include_tests: true
    let custom_config = GeneralAiConfig::new().with_include_tests(true);
    let custom_profile = GeneralAiProfile::new(custom_config);
    assert!(custom_profile.config().include_tests);

    let budget = ContextBudget::with_total_tokens(10_000);
    let result = custom_profile
        .optimize(&doc, &budget)
        .expect("optimization with tests enabled succeeds");

    assert_eq!(result.document.sections.len(), 2);
    assert!(result.metrics.test_tokens > 0);
}

#[test]
fn test_general_ai_truncation_under_tight_budget() {
    let s_readme = make_section(
        "s1",
        "README.md",
        SectionKind::Prose,
        "# Project Overview\n\nShort summary.",
        vec![],
    );
    let long_code = (1..=200)
        .map(|i| format!("    let step_{i} = calculate({i});"))
        .collect::<Vec<_>>()
        .join("\n");
    let s_code = make_section(
        "s2",
        "src/pipeline.rs",
        SectionKind::CodeFile,
        &format!("pub fn execute() {{\n{long_code}\n}}\n"),
        vec![("code_nature", "core_implementation")],
    );

    let doc = make_test_doc(vec![s_readme, s_code]);

    // Tight budget with 120 content tokens
    let budget = ContextBudget::new(200, 40, 30, 10);
    let config = GeneralAiConfig::new()
        .with_allow_truncation(true)
        .with_min_retained_tokens(50);
    let profile = GeneralAiProfile::new(config);

    let result = profile
        .optimize(&doc, &budget)
        .expect("truncation succeeds");

    assert!(result.plan.is_within_budget());
    assert!(result.plan.fitted_tokens <= 120);
    assert_eq!(result.plan.truncated_section_count, 1);

    let truncated = result
        .document
        .sections
        .iter()
        .find(|s| s.original_path.as_deref() == Some("src/pipeline.rs"))
        .expect("truncated code section present");

    assert!(truncated.verify_hash());
    assert!(truncated.body.contains("Truncated:"));
}

#[test]
fn test_general_ai_markdown_report_and_json_serialization() {
    let s1 = make_section(
        "s1",
        "README.md",
        SectionKind::Prose,
        "# Documentation",
        vec![],
    );
    let s2 = make_section(
        "s2",
        "src/main.rs",
        SectionKind::CodeFile,
        "fn main() {}",
        vec![],
    );
    let doc = make_test_doc(vec![s1, s2]);

    let budget = ContextBudget::from_model(TargetModel::Gpt4o);
    let result = apply_general_ai_profile(&doc, &budget).expect("optimization succeeds");

    // Test Markdown report
    let md = result.to_markdown_summary();
    assert!(md.contains("Context Profile Optimization: `general_ai`"));
    assert!(md.contains("Token Distribution by Category"));
    assert!(md.contains("Documentation & Prose"));
    assert!(md.contains("Core Implementation Code"));
    assert!(md.contains("README.md"));

    // Test JSON roundtrip
    let json = result.to_json().expect("JSON serialization succeeds");
    let decoded =
        ProfileOptimizationResult::from_json(&json).expect("JSON deserialization succeeds");

    assert_eq!(result.profile_kind, decoded.profile_kind);
    assert_eq!(result.plan.fitted_tokens, decoded.plan.fitted_tokens);
    assert_eq!(
        result.metrics.documentation_tokens,
        decoded.metrics.documentation_tokens
    );
}
