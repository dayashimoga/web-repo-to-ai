//! # Context Optimization Compression Reporting Integration Tests
//!
//! Validates end-to-end compression reporting, stage-by-stage progression tracking,
//! multi-model financial cost savings calculation, markdown table rendering, and JSON serialization.

use std::collections::BTreeMap;
use web_repo_to_ai_cdm::{
    CdmDocument, CdmSection, CompletenessReport, DocumentMetadata, ExtractionStatus, HierarchyNode,
    RevisionInfo, SectionKind, SourceIdentity, SourceType,
};
use web_repo_to_ai_context_optimizer::{
    evaluate_compression, evaluate_compression_for_model, evaluate_compression_with_profile,
    BudgetOptimizationConfig, CompressionPipelineTracker, CompressionReport, ContextBudget,
    ContextBudgetOptimizer, ContextOptimizerError, OptimizationStage,
};
use web_repo_to_ai_core::DocumentId;
use web_repo_to_ai_tokenizer::TargetModel;

fn make_test_section(
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
        language: if path.ends_with(".rs") {
            Some("rust".to_string())
        } else {
            None
        },
        content_hash: CdmSection::compute_hash(body),
        body: body.to_string(),
        token_metrics: None,
        attributes,
    }
}

fn make_test_document(sections: Vec<CdmSection>) -> CdmDocument {
    CdmDocument {
        schema_version: "1.0.0".to_string(),
        document_id: DocumentId::new(),
        source_identity: SourceIdentity::new(
            "https://github.com/org/repo",
            "https://github.com/org/repo",
        ),
        source_type: SourceType::GitHubPublic,
        canonical_url: "https://github.com/org/repo".to_string(),
        retrieved_at: chrono::Utc::now(),
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

#[test]
fn test_compression_reporting_unmodified_document() {
    let s1 = make_test_section(
        "s1",
        "Cargo.toml",
        SectionKind::Manifest,
        "[package]\nname = \"repo\"\nversion = \"0.1.0\"",
        vec![],
    );
    let s2 = make_test_section(
        "s2",
        "src/lib.rs",
        SectionKind::CodeFile,
        "pub fn add(a: i32, b: i32) -> i32 { a + b }",
        vec![],
    );

    let doc = make_test_document(vec![s1, s2]);

    let report =
        evaluate_compression(&doc, &doc).expect("Evaluation succeeds on identical documents");

    assert_eq!(report.overall.total_token_savings, 0);
    assert_eq!(report.overall.token_reduction_pct, 0.0);
    assert_eq!(report.overall.token_compression_ratio, 1.0);
    assert_eq!(report.overall.total_byte_savings, 0);
    assert_eq!(report.overall.byte_reduction_pct, 0.0);
    assert_eq!(report.overall.byte_compression_ratio, 1.0);
    assert_eq!(report.overall.total_sections_removed, 0);
    assert_eq!(report.overall.original_sections, 2);
    assert_eq!(report.overall.retained_sections, 2);
}

#[test]
fn test_compression_reporting_budget_optimization() {
    let mut sections = Vec::new();
    sections.push(make_test_section(
        "m1",
        "Cargo.toml",
        SectionKind::Manifest,
        "[package]\nname = \"test\"",
        vec![],
    ));

    for i in 0..15 {
        sections.push(make_test_section(
            &format!("c{}", i),
            &format!("src/module_{}.rs", i),
            SectionKind::CodeFile,
            &format!(
                "pub fn run_{}() -> usize {{\n    let mut sum = 0;\n{}\n    sum\n}}",
                i,
                "    sum += 1;\n".repeat(50)
            ),
            vec![],
        ));
    }

    let orig_doc = make_test_document(sections);

    // Tight budget: 200 total, 100 reserved, leaving 100 content tokens for ~2000 token doc
    let budget = ContextBudget::new(200, 50, 25, 25);
    let config = BudgetOptimizationConfig::new(budget);
    let optimizer = ContextBudgetOptimizer::new(config);
    let (optimized_doc, _plan) = optimizer
        .optimize(&orig_doc)
        .expect("Optimization succeeds");

    let report =
        evaluate_compression(&orig_doc, &optimized_doc).expect("Compression report succeeds");

    assert!(report.overall.original_tokens > report.overall.optimized_tokens);
    assert!(report.overall.total_token_savings > 0);
    assert!(report.overall.token_reduction_pct > 0.0);
    assert!(report.overall.token_compression_ratio > 1.0);
    assert!(report.overall.total_byte_savings > 0);
    assert!(report.overall.total_sections_removed > 0);
    assert!(report.overall.retained_sections < report.overall.original_sections);

    let md = report.to_markdown_summary();
    assert!(md.contains("Context Optimization Compression Report"));
    assert!(md.contains("Cumulative Optimization Summary"));
    assert!(md.contains("Tokens"));
    assert!(md.contains("Bytes (UTF-8)"));
}

#[test]
fn test_compression_reporting_multi_stage_pipeline_tracker() {
    let s_manifest = make_test_section(
        "s1",
        "Cargo.toml",
        SectionKind::Manifest,
        "[package]\nname = \"app\"",
        vec![],
    );
    let s_code1 = make_test_section(
        "s2",
        "src/auth.rs",
        SectionKind::CodeFile,
        "pub fn authenticate() -> bool { true }",
        vec![],
    );
    let s_code2 = make_test_section(
        "s3",
        "src/dup.rs",
        SectionKind::CodeFile,
        "pub fn authenticate() -> bool { true }",
        vec![],
    );
    let s_test = make_test_section(
        "s4",
        "tests/ui_test.rs",
        SectionKind::CodeFile,
        "assert_eq!(1, 1);",
        vec![],
    );
    let s_doc = make_test_section(
        "s5",
        "README.md",
        SectionKind::Prose,
        "# App Readme\nDetailed docs here.",
        vec![],
    );

    let doc0 = make_test_document(vec![
        s_manifest.clone(),
        s_code1.clone(),
        s_code2.clone(),
        s_test.clone(),
        s_doc.clone(),
    ]);

    let mut tracker = CompressionPipelineTracker::new(&doc0)
        .expect("Tracker initializes")
        .with_profile_name("SecurityReview")
        .with_target_model(TargetModel::Claude35Sonnet)
        .with_note("Applied multi-stage security context reduction pipeline.");

    // Stage 1: Structural Normalization (clean whitespace)
    let doc1 = doc0.clone();
    tracker.record_stage(OptimizationStage::Normalization, &doc1);

    // Stage 2: Exact Deduplication (removes s_code2 duplicate)
    let doc2 = make_test_document(vec![
        s_manifest.clone(),
        s_code1.clone(),
        s_test.clone(),
        s_doc.clone(),
    ]);
    tracker.record_stage(OptimizationStage::ExactDeduplication, &doc2);

    // Stage 3: Profile Filtering (removes non-security README and cosmetic UI test)
    let doc3 = make_test_document(vec![s_manifest.clone(), s_code1.clone()]);
    tracker.record_stage(OptimizationStage::ProfileFiltering, &doc3);

    // Stage 4: Budget Knapsack (already fits, no drops)
    let doc4 = doc3.clone();
    tracker.record_stage(OptimizationStage::BudgetKnapsack, &doc4);

    let report = tracker.generate_report();

    assert_eq!(report.stages.len(), 4);
    assert_eq!(report.stages[0].stage, OptimizationStage::Normalization);
    assert_eq!(
        report.stages[1].stage,
        OptimizationStage::ExactDeduplication
    );
    assert_eq!(report.stages[2].stage, OptimizationStage::ProfileFiltering);
    assert_eq!(report.stages[3].stage, OptimizationStage::BudgetKnapsack);

    // Stage 1: no sections removed
    assert_eq!(report.stages[0].section_reduction, 0);
    // Stage 2: 1 section removed (duplicate)
    assert_eq!(report.stages[1].section_reduction, 1);
    // Stage 3: 2 sections removed (profile filter)
    assert_eq!(report.stages[2].section_reduction, 2);
    // Stage 4: 0 sections removed
    assert_eq!(report.stages[3].section_reduction, 0);

    // Overall summary matches cumulative delta
    assert_eq!(report.overall.original_sections, 5);
    assert_eq!(report.overall.retained_sections, 2);
    assert_eq!(report.overall.total_sections_removed, 3);
    assert_eq!(report.profile_name.as_deref(), Some("SecurityReview"));
    assert_eq!(report.target_model, Some(TargetModel::Claude35Sonnet));
}

#[test]
fn test_compression_reporting_multi_model_cost_savings() {
    let s1 = make_test_section(
        "s1",
        "big.rs",
        SectionKind::CodeFile,
        &"let a = 1;\n".repeat(500),
        vec![],
    );
    let s2 = make_test_section(
        "s2",
        "small.rs",
        SectionKind::CodeFile,
        "let a = 1;\n",
        vec![],
    );

    let orig_doc = make_test_document(vec![s1]);
    let opt_doc = make_test_document(vec![s2]);

    let report = evaluate_compression(&orig_doc, &opt_doc).expect("Report succeeds");

    assert!(!report.cost_savings.models.is_empty());

    // Check GPT-4o cost reduction
    let gpt4o = report
        .cost_savings
        .models
        .iter()
        .find(|m| m.model == TargetModel::Gpt4o)
        .expect("GPT-4o present");
    assert!(gpt4o.unoptimized_cost_usd > gpt4o.optimized_cost_usd);
    assert!(gpt4o.cost_savings_usd > 0.0);
    assert!(gpt4o.savings_pct > 0.0);
    assert!(!gpt4o.is_free_or_self_hosted);

    // Check Claude 3.5 Sonnet
    let claude = report
        .cost_savings
        .models
        .iter()
        .find(|m| m.model == TargetModel::Claude35Sonnet)
        .expect("Claude 3.5 Sonnet present");
    assert!(claude.cost_savings_usd > 0.0);

    // Check Llama 3 (self-hosted $0.00)
    let llama3 = report
        .cost_savings
        .models
        .iter()
        .find(|m| m.model == TargetModel::Llama3)
        .expect("Llama 3 present");
    assert!(llama3.is_free_or_self_hosted);
    assert_eq!(llama3.cost_savings_usd, 0.0);
}

#[test]
fn test_compression_reporting_markdown_and_json_serialization() {
    let s1 = make_test_section(
        "s1",
        "Cargo.toml",
        SectionKind::Manifest,
        "[package]\nname = \"test\"",
        vec![],
    );
    let s2 = make_test_section(
        "s2",
        "src/main.rs",
        SectionKind::CodeFile,
        "fn main() {}",
        vec![],
    );

    let orig_doc = make_test_document(vec![s1.clone(), s2]);
    let opt_doc = make_test_document(vec![s1]);

    let report = evaluate_compression_with_profile(
        &orig_doc,
        &opt_doc,
        Some("GeneralAi"),
        Some(TargetModel::Gpt4o),
    )
    .expect("Report generation succeeds");

    // Verify Markdown Summary
    let md = report.to_markdown_summary();
    assert!(md.contains("## Context Optimization Compression Report"));
    assert!(md.contains("GeneralAi"));
    assert!(md.contains("gpt-4o"));
    assert!(md.contains("### 1. Cumulative Optimization Summary"));
    assert!(md.contains("### 3. Estimated Financial API Cost Savings"));

    // Verify JSON roundtrip
    let json = report.to_json().expect("JSON serialization succeeds");
    let decoded = CompressionReport::from_json(&json).expect("JSON deserialization succeeds");

    assert_eq!(
        report.overall.original_tokens,
        decoded.overall.original_tokens
    );
    assert_eq!(
        report.overall.optimized_tokens,
        decoded.overall.optimized_tokens
    );
    assert_eq!(
        report.overall.total_token_savings,
        decoded.overall.total_token_savings
    );
    assert_eq!(report.profile_name, decoded.profile_name);
    assert_eq!(report.target_model, decoded.target_model);
    assert_eq!(report.stages.len(), decoded.stages.len());
}

#[test]
fn test_compression_reporting_fail_closed_empty_document() {
    let empty_doc = make_test_document(vec![]);
    let valid_doc = make_test_document(vec![make_test_section(
        "s1",
        "a.rs",
        SectionKind::CodeFile,
        "fn a() {}",
        vec![],
    )]);

    let err1 = evaluate_compression(&empty_doc, &valid_doc).expect_err("Should fail closed");
    assert!(matches!(err1, ContextOptimizerError::DocumentEmpty));

    let err2 = CompressionPipelineTracker::new(&empty_doc).expect_err("Should fail closed");
    assert!(matches!(err2, ContextOptimizerError::DocumentEmpty));
}

#[test]
fn test_compression_reporting_convenience_apis() {
    let s1 = make_test_section(
        "s1",
        "src/lib.rs",
        SectionKind::CodeFile,
        "pub fn logic() -> bool { true }",
        vec![],
    );
    let orig = make_test_document(vec![s1.clone(), s1.clone()]);
    let opt = make_test_document(vec![s1]);

    // Test evaluate_compression
    let r1 = evaluate_compression(&orig, &opt).expect("r1 succeeds");
    assert_eq!(r1.overall.original_sections, 2);
    assert_eq!(r1.overall.retained_sections, 1);

    // Test evaluate_compression_for_model
    let r2 = evaluate_compression_for_model(&orig, &opt, TargetModel::Claude35Sonnet)
        .expect("r2 succeeds");
    assert_eq!(r2.target_model, Some(TargetModel::Claude35Sonnet));

    // Test evaluate_compression_with_profile
    let r3 = evaluate_compression_with_profile(
        &orig,
        &opt,
        Some("Debugging"),
        Some(TargetModel::Gemini15Pro),
    )
    .expect("r3 succeeds");
    assert_eq!(r3.profile_name.as_deref(), Some("Debugging"));
    assert_eq!(r3.target_model, Some(TargetModel::Gemini15Pro));
}
