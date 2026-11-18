//! # Integration Tests for Context Budget Calculation & Optimization
//!
//! Validates multi-model headroom allocations, greedy knapsack section prioritization,
//! zero silent drops, auditable exclusions, boundary truncation, and telemetry reporting.

use chrono::Utc;
use std::collections::BTreeMap;
use web_repo_to_ai_cdm::{
    CdmDocument, CdmSection, CompletenessReport, DocumentMetadata, ExclusionReason,
    ExtractionStatus, HierarchyNode, RevisionInfo, SectionKind, SourceIdentity, SourceType,
};
use web_repo_to_ai_context_optimizer::{
    optimize_context_for_model, optimize_context_with_budget, BudgetAction,
    BudgetOptimizationConfig, ContextBudget, ContextBudgetOptimizer, ContextOptimizerError,
    TruncationStrategy,
};
use web_repo_to_ai_core::DocumentId;
use web_repo_to_ai_tokenizer::TargetModel;

/// Helper to create a test document with initial metadata and sections.
fn make_document(sections: Vec<CdmSection>) -> CdmDocument {
    CdmDocument {
        schema_version: "1.0.0".to_string(),
        document_id: DocumentId::new(),
        source_identity: SourceIdentity::new(
            "https://github.com/example/engine",
            "https://github.com/example/engine",
        ),
        source_type: SourceType::GitHubPublic,
        canonical_url: "https://github.com/example/engine".to_string(),
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
        source_url: format!("https://github.com/example/engine/blob/main/{path}"),
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
fn test_multi_model_budget_allocation_and_headroom() {
    let models = [
        TargetModel::Gpt4o,
        TargetModel::Claude35Sonnet,
        TargetModel::Gemini15Pro,
        TargetModel::Gemini15Flash,
        TargetModel::Llama3,
        TargetModel::MistralLarge,
    ];

    for model in models {
        let budget = ContextBudget::from_model(model);
        assert!(budget.validate().is_ok());

        assert_eq!(budget.total_tokens, model.context_window_limit());
        assert!(budget.total_reserved_tokens() > 0);
        assert!(budget.available_content_tokens() > 0);
        assert!(budget.available_content_tokens() < budget.total_tokens);
        assert!(budget.headroom_ratio() > 0.0 && budget.headroom_ratio() < 1.0);
        assert!(budget.content_ratio() > 0.0 && budget.content_ratio() < 1.0);
        assert!((budget.headroom_ratio() + budget.content_ratio() - 1.0).abs() < 1e-6);

        let summary = budget.headroom_summary();
        let table = summary.to_markdown_table();
        assert!(table.contains(model.as_str()));
        assert!(table.contains("Available Content"));
    }
}

#[test]
fn test_priority_knapsack_greedy_optimization() {
    let s_readme = make_section(
        "s1",
        "README.md",
        SectionKind::Prose,
        "# Engine Overview\n\nHigh-performance asynchronous data engine.",
        vec![],
    );
    let s_manifest = make_section(
        "s2",
        "Cargo.toml",
        SectionKind::Manifest,
        "[package]\nname = \"engine\"\nversion = \"0.1.0\"\n",
        vec![],
    );
    let s_core = make_section(
        "s3",
        "src/main.rs",
        SectionKind::CodeFile,
        "fn main() {\n    engine::start();\n}\n",
        vec![
            ("code_nature", "core_implementation"),
            ("information_density", "high"),
            ("signal_score", "0.95"),
        ],
    );
    let s_utils = make_section(
        "s4",
        "src/utils.rs",
        SectionKind::CodeFile,
        "pub fn helper() -> bool { true }\n",
        vec![("code_nature", "core_implementation")],
    );
    let s_test = make_section(
        "s5",
        "tests/integration_test.rs",
        SectionKind::CodeFile,
        &"#[test]\nfn test_something() { assert!(true); }\n".repeat(80),
        vec![("is_test", "true"), ("code_nature", "test_or_fixture")],
    );

    let doc = make_document(vec![s_readme, s_manifest, s_core, s_utils, s_test]);

    // Constrain available content tokens to 120 tokens, disallowing truncation
    let budget = ContextBudget::new(200, 40, 30, 10); // available = 120
    let config = BudgetOptimizationConfig::new(budget).with_allow_truncation(false);
    let optimizer = ContextBudgetOptimizer::new(config);

    let (optimized_doc, plan) = optimizer.optimize(&doc).expect("optimization succeeds");

    assert!(plan.is_within_budget());
    assert!(plan.fitted_tokens <= 120);
    assert_eq!(plan.original_section_count, 5);

    // High priority and MustInclude sections must be included
    assert!(optimized_doc
        .sections
        .iter()
        .any(|s| s.original_path.as_deref() == Some("README.md")));
    assert!(optimized_doc
        .sections
        .iter()
        .any(|s| s.original_path.as_deref() == Some("Cargo.toml")));
    assert!(optimized_doc
        .sections
        .iter()
        .any(|s| s.original_path.as_deref() == Some("src/main.rs")));

    // Large test section must be excluded
    assert_eq!(plan.excluded_section_count, 1);
    assert_eq!(optimized_doc.exclusions.len(), 1);
    let exclusion = &optimized_doc.exclusions[0];
    assert_eq!(exclusion.reason, ExclusionReason::SizeLimit);
    assert_eq!(exclusion.original_path, "tests/integration_test.rs");
    assert!(exclusion
        .details
        .as_deref()
        .unwrap_or("")
        .contains("Excluded to fit context budget"));

    // BLAKE3 document revision hash must be updated
    assert!(optimized_doc.revision.content_hash_blake3.is_some());
}

#[test]
fn test_clean_boundary_truncation_under_budget() {
    let s_readme = make_section(
        "s1",
        "README.md",
        SectionKind::Prose,
        "# Project Title\n\nShort description.",
        vec![],
    );
    // 300 lines of code
    let long_code = (1..=300)
        .map(|i| format!("    let v{i} = {i} * 2;"))
        .collect::<Vec<_>>()
        .join("\n");
    let s_large_code = make_section(
        "s2",
        "src/large.rs",
        SectionKind::CodeFile,
        &format!("pub fn process() {{\n{long_code}\n}}\n"),
        vec![("code_nature", "core_implementation")],
    );

    let doc = make_document(vec![s_readme, s_large_code]);

    // Budget with 150 available content tokens, truncation enabled (HeadAndTail)
    let budget = ContextBudget::new(250, 50, 40, 10); // available = 150
    let config = BudgetOptimizationConfig::new(budget)
        .with_allow_truncation(true)
        .with_truncation_strategy(TruncationStrategy::HeadAndTail);
    let optimizer = ContextBudgetOptimizer::new(config);

    let (optimized_doc, plan) = optimizer.optimize(&doc).expect("optimization succeeds");

    assert!(plan.is_within_budget());
    assert!(plan.fitted_tokens <= 150);
    assert_eq!(plan.truncated_section_count, 1);

    // Verify truncated section integrity
    let truncated_sec = optimized_doc
        .sections
        .iter()
        .find(|s| s.original_path.as_deref() == Some("src/large.rs"))
        .expect("truncated section present in output");

    assert!(truncated_sec.verify_hash());
    assert!(truncated_sec.body.contains("Truncated:"));
    assert!(truncated_sec.token_metrics.is_some());
    let metrics = truncated_sec.token_metrics.as_ref().unwrap();
    assert!(metrics.exact_tokens.unwrap() <= 150);
}

#[test]
fn test_zero_silent_drop_audit_trail() {
    let mut sections = Vec::new();
    for i in 1..=10 {
        let path = format!("src/file_{i}.rs");
        let body = format!("pub fn f{i}() -> usize {{ {i} }}\n").repeat(20);
        sections.push(make_section(
            &format!("s{i}"),
            &path,
            SectionKind::CodeFile,
            &body,
            vec![],
        ));
    }

    let doc = make_document(sections);

    // Tight budget
    let budget = ContextBudget::new(300, 60, 40, 20); // available = 180
    let config = BudgetOptimizationConfig::new(budget).with_allow_truncation(false);
    let optimizer = ContextBudgetOptimizer::new(config);

    let (optimized_doc, plan) = optimizer.optimize(&doc).expect("optimization succeeds");

    assert!(plan.is_within_budget());

    // Conservation law 1: section count
    assert_eq!(
        plan.original_section_count,
        plan.included_section_count + plan.excluded_section_count + plan.truncated_section_count
    );
    assert_eq!(
        doc.sections.len(),
        optimized_doc.sections.len() + optimized_doc.exclusions.len()
    );

    // Conservation law 2: every excluded section has an ExclusionRecord
    for decision in &plan.decisions {
        if let BudgetAction::Exclude { reason, .. } = &decision.action {
            let path = decision.path.as_deref().unwrap();
            let record = optimized_doc
                .exclusions
                .iter()
                .find(|e| e.original_path == path);
            assert!(
                record.is_some(),
                "Excluded section '{path}' must have an auditable ExclusionRecord"
            );
            assert_eq!(record.unwrap().reason, *reason);
        }
    }
}

#[test]
fn test_convenience_functions_and_serialization() {
    let s1 = make_section(
        "s1",
        "README.md",
        SectionKind::Prose,
        "# Universal Engine",
        vec![],
    );
    let s2 = make_section(
        "s2",
        "src/lib.rs",
        SectionKind::CodeFile,
        "pub fn run() {}",
        vec![],
    );
    let doc = make_document(vec![s1, s2]);

    // Test optimize_context_for_model
    let (opt_doc, report) = optimize_context_for_model(&doc, TargetModel::Gpt4o)
        .expect("convenience model optimization succeeds");

    assert_eq!(opt_doc.sections.len(), 2);
    assert!(report.plan.is_within_budget());

    // Test JSON roundtrip
    let json = report.to_json().expect("JSON serialization succeeds");
    let deserialized = web_repo_to_ai_context_optimizer::ContextBudgetReport::from_json(&json)
        .expect("JSON deserialization succeeds");

    assert_eq!(report.plan.fitted_tokens, deserialized.plan.fitted_tokens);
    assert_eq!(
        report.headroom.total_tokens,
        deserialized.headroom.total_tokens
    );

    // Test Markdown report output
    let md = report.to_markdown_summary();
    assert!(md.contains("Context Budget Optimization Report"));
    assert!(md.contains("WITHIN_BUDGET"));
    assert!(md.contains("Token Conservation Ledger"));
    assert!(md.contains("README.md"));

    // Test optimize_context_with_budget
    let custom_budget = ContextBudget::with_total_tokens(50_000);
    let (opt_doc_custom, report_custom) = optimize_context_with_budget(&doc, custom_budget)
        .expect("custom budget optimization succeeds");

    assert_eq!(opt_doc_custom.sections.len(), 2);
    assert!(report_custom.plan.is_within_budget());
}

#[test]
fn test_fail_closed_error_conditions() {
    // 1. Empty document rejection
    let empty_doc = make_document(vec![]);
    let optimizer = ContextBudgetOptimizer::for_model(TargetModel::Claude35Sonnet);
    assert!(matches!(
        optimizer.optimize(&empty_doc),
        Err(ContextOptimizerError::DocumentEmpty)
    ));

    // 2. Zero total budget rejection
    let zero_budget = ContextBudget::new(0, 0, 0, 0);
    let config = BudgetOptimizationConfig::new(zero_budget);
    let sec = make_section(
        "s1",
        "src/lib.rs",
        SectionKind::CodeFile,
        "pub fn a() {}",
        vec![],
    );
    let doc = make_document(vec![sec]);
    let opt_zero = ContextBudgetOptimizer::new(config);
    assert!(matches!(
        opt_zero.optimize(&doc),
        Err(ContextOptimizerError::InvalidBudget { .. })
    ));

    // 3. Headroom exceeds total budget rejection
    let excessive_headroom = ContextBudget::new(1000, 600, 400, 100);
    let config_excess = BudgetOptimizationConfig::new(excessive_headroom);
    let opt_excess = ContextBudgetOptimizer::new(config_excess);
    assert!(matches!(
        opt_excess.optimize(&doc),
        Err(ContextOptimizerError::InvalidBudget { .. })
    ));
}
