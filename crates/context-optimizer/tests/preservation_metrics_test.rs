//! # Integration Tests for Quality Validation & Preservation Metrics (Subphase 7.4.1)
//!
//! Validates quantitative preservation metrics calculating retained essential token ratios,
//! AST symbol coverage, structural integrity preservation, and information fidelity scores.

use chrono::Utc;
use std::collections::BTreeMap;
use web_repo_to_ai_cdm::{
    CdmDocument, CdmSection, CompletenessReport, DocumentMetadata, ExtractionStatus, HierarchyNode,
    RevisionInfo, SectionKind, SourceIdentity, SourceType,
};
use web_repo_to_ai_context_optimizer::{
    apply_code_review_profile, apply_general_ai_profile_for_model, evaluate_preservation,
    evaluate_preservation_with_config, ContextBudget, ContextOptimizerError, PreservationConfig,
    PreservationReport,
};
use web_repo_to_ai_core::DocumentId;
use web_repo_to_ai_tokenizer::TargetModel;

/// Helper to create a test document with initial metadata and sections.
fn make_test_doc(sections: Vec<CdmSection>) -> CdmDocument {
    CdmDocument {
        schema_version: "1.0.0".to_string(),
        document_id: DocumentId::new(),
        source_identity: SourceIdentity::new(
            "https://github.com/org/fidelity-engine",
            "https://github.com/org/fidelity-engine",
        ),
        source_type: SourceType::GitHubPublic,
        canonical_url: "https://github.com/org/fidelity-engine".to_string(),
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
        source_url: format!("https://github.com/org/fidelity-engine/blob/main/{path}"),
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
fn test_preservation_metrics_unmodified_fidelity() {
    let s_manifest = make_section(
        "s_manifest",
        "Cargo.toml",
        SectionKind::Manifest,
        "[package]\nname = \"fidelity-engine\"\nversion = \"0.1.0\"",
        vec![],
    );
    let s_code = make_section(
        "s_code",
        "src/lib.rs",
        SectionKind::CodeFile,
        "pub fn calculate_fidelity() -> f64 { 1.0 }\nfn internal_metric() -> usize { 42 }",
        vec![],
    );

    let doc = make_test_doc(vec![s_manifest, s_code]);
    let report = evaluate_preservation(&doc, &doc).expect("Evaluation succeeds");

    assert_eq!(report.section_metrics.retention_ratio, 1.0);
    assert_eq!(report.critical_metrics.critical_retention_ratio, 1.0);
    assert_eq!(report.symbol_metrics.symbol_retention_ratio, 1.0);
    assert_eq!(report.symbol_metrics.public_symbol_retention_ratio, 1.0);
    assert_eq!(report.token_metrics.token_retention_ratio, 1.0);
    assert_eq!(report.token_metrics.token_reduction_pct, 0.0);
    assert!(report.structural_metrics.syntax_intact);
    assert_eq!(report.structural_metrics.corrupt_sections_count, 0);
    assert_eq!(report.fidelity_score, 1.0);
    assert!(report.is_acceptable);
}

#[test]
fn test_preservation_metrics_across_budget_optimization() {
    let s_manifest = make_section(
        "s_manifest",
        "Cargo.toml",
        SectionKind::Manifest,
        "[package]\nname = \"fidelity-engine\"",
        vec![],
    );
    let s_readme = make_section(
        "s_readme",
        "README.md",
        SectionKind::Prose,
        "# Fidelity Engine\nArchitecture overview and specifications.",
        vec![],
    );
    let s_core = make_section(
        "s_core",
        "src/engine.rs",
        SectionKind::CodeFile,
        "pub struct Engine;\nimpl Engine {\n    pub fn run(&self) -> bool { true }\n    fn check_invariants(&self) {}\n}",
        vec![("code_nature", "core_implementation")],
    );
    let s_test = make_section(
        "s_test",
        "tests/integration_test.rs",
        SectionKind::CodeFile,
        "#[test]\nfn test_harness() { assert!(true); }",
        vec![("is_test", "true")],
    );

    let original_doc = make_test_doc(vec![s_manifest, s_readme, s_core, s_test]);

    // Optimize using General AI profile for GPT-4o
    let opt_result = apply_general_ai_profile_for_model(&original_doc, TargetModel::Gpt4o)
        .expect("Optimization succeeds");

    let report = evaluate_preservation(&original_doc, &opt_result.document)
        .expect("Preservation evaluation succeeds");

    // Critical content (Manifest + README) must be 100% preserved
    assert_eq!(report.critical_metrics.critical_retention_ratio, 1.0);
    assert!(report.critical_metrics.critical_omissions.is_empty());

    // Public API symbols (`Engine`, `run`) must be preserved
    assert!(report.symbol_metrics.public_symbol_retention_ratio >= 0.8);

    // Structural integrity must remain verified
    assert!(report.structural_metrics.syntax_intact);
    assert_eq!(report.structural_metrics.corrupt_sections_count, 0);

    // Overall fidelity score should satisfy default threshold
    assert!(report.fidelity_score >= 0.70);
    assert!(report.is_acceptable);
}

#[test]
fn test_preservation_metrics_detects_symbol_omissions() {
    let s_api = make_section(
        "s_api",
        "src/api.rs",
        SectionKind::CodeFile,
        "pub fn public_entrypoint() {}\npub fn secondary_entrypoint() {}",
        vec![],
    );
    let s_unrelated = make_section(
        "s_unrelated",
        "src/experimental.rs",
        SectionKind::CodeFile,
        "pub fn experimental_feature_alpha() {}\npub fn experimental_feature_beta() {}",
        vec![],
    );

    let original_doc = make_test_doc(vec![s_api.clone(), s_unrelated]);

    // Simulate an optimized doc that retained only s_api
    let optimized_doc = make_test_doc(vec![s_api]);

    let report = evaluate_preservation(&original_doc, &optimized_doc).expect("Evaluation succeeds");

    assert_eq!(report.section_metrics.retention_ratio, 0.5);
    assert_eq!(report.symbol_metrics.symbols_original_count, 4);
    assert_eq!(report.symbol_metrics.symbols_retained_count, 2);
    assert_eq!(report.symbol_metrics.symbol_retention_ratio, 0.5);

    // Omitted symbols must explicitly list the experimental features
    assert!(report
        .symbol_metrics
        .omitted_symbol_names
        .contains(&"experimental_feature_alpha".to_string()));
    assert!(report
        .symbol_metrics
        .omitted_symbol_names
        .contains(&"experimental_feature_beta".to_string()));
}

#[test]
fn test_preservation_metrics_across_code_review_profile() {
    let s_changed = make_section(
        "s_diff",
        "src/auth/jwt.rs",
        SectionKind::CodeFile,
        "pub fn issue_token() -> String { \"token\".into() }",
        vec![],
    );
    let s_unrelated_doc = make_section(
        "s_doc",
        "docs/ARCHITECTURE.md",
        SectionKind::Prose,
        "# Architecture\nOverview details.",
        vec![],
    );

    let original_doc = make_test_doc(vec![s_changed, s_unrelated_doc]);

    let budget = ContextBudget::new(5000, 300, 500, 100);
    let opt_result =
        apply_code_review_profile(&original_doc, &["src/auth/jwt.rs".to_string()], &budget)
            .expect("Code review profile succeeds");

    let report =
        evaluate_preservation(&original_doc, &opt_result.document).expect("Evaluation succeeds");

    // Changed file must be 100% preserved
    assert_eq!(report.critical_metrics.critical_retention_ratio, 1.0);
    assert!(report.symbol_metrics.public_symbol_retention_ratio >= 1.0);
    assert!(report.structural_metrics.syntax_intact);
}

#[test]
fn test_preservation_report_markdown_and_json_serialization() {
    let s1 = make_section(
        "s1",
        "Cargo.toml",
        SectionKind::Manifest,
        "[package]\nname = \"fidelity\"",
        vec![],
    );
    let s2 = make_section(
        "s2",
        "src/main.rs",
        SectionKind::CodeFile,
        "pub fn main() {}",
        vec![],
    );

    let doc = make_test_doc(vec![s1, s2]);
    let report = evaluate_preservation(&doc, &doc).expect("Evaluation succeeds");

    // Markdown summary table & report rendering
    let table = report.to_markdown_table();
    assert!(table.contains("Critical Content"));
    assert!(table.contains("Public API Symbols"));
    assert!(table.contains("Composite Fidelity"));

    let full_report = report.to_markdown_report();
    assert!(full_report.contains("Quality Validation & Information Preservation Report"));
    assert!(full_report.contains("Preservation Breakdown by Section Kind"));

    // JSON serialization roundtrip
    let json = report.to_json().expect("JSON serialization succeeds");
    assert!(json.contains("fidelity_score"));
    assert!(json.contains("critical_metrics"));

    let deserialized: PreservationReport =
        PreservationReport::from_json(&json).expect("JSON deserialization succeeds");
    assert_eq!(deserialized.fidelity_score, report.fidelity_score);
    assert!(deserialized.is_acceptable);
}

#[test]
fn test_preservation_metrics_empty_document_rejected() {
    let empty_doc = make_test_doc(vec![]);
    let s = make_section("s", "README.md", SectionKind::Prose, "# Readme", vec![]);
    let valid_doc = make_test_doc(vec![s]);

    let err = evaluate_preservation(&empty_doc, &valid_doc).expect_err("Empty doc rejected");
    assert!(matches!(err, ContextOptimizerError::DocumentEmpty));
}

#[test]
fn test_preservation_config_custom_thresholds() {
    let s1 = make_section(
        "s1",
        "src/util.rs",
        SectionKind::CodeFile,
        "fn helper() {}",
        vec![],
    );
    let s2 = make_section(
        "s2",
        "src/extra.rs",
        SectionKind::CodeFile,
        "fn extra() {}",
        vec![],
    );

    let orig_doc = make_test_doc(vec![s1.clone(), s2]);
    let opt_doc = make_test_doc(vec![s1]);

    // Strict config requiring 0.90 minimum fidelity
    let strict_config = PreservationConfig::new().with_min_acceptable_fidelity(0.90);
    let report_strict = evaluate_preservation_with_config(&orig_doc, &opt_doc, &strict_config)
        .expect("Evaluation succeeds");

    assert!(!report_strict.is_acceptable);

    // Lenient config requiring 0.40 minimum fidelity
    let lenient_config = PreservationConfig::new().with_min_acceptable_fidelity(0.40);
    let report_lenient = evaluate_preservation_with_config(&orig_doc, &opt_doc, &lenient_config)
        .expect("Evaluation succeeds");

    assert!(report_lenient.is_acceptable);
}
