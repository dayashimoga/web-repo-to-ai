//! # Context Optimization Exclusion Explanations Integration Tests
//!
//! Validates structured diagnostic rationales, impact severity assessment, category aggregations,
//! remediation advice generation, markdown summary tables, and JSON serialization.

use std::collections::BTreeMap;
use web_repo_to_ai_cdm::{
    CdmDocument, CdmSection, CompletenessReport, DocumentMetadata, ExclusionReason,
    ExclusionRecord, ExtractionStatus, HierarchyNode, RevisionInfo, SectionKind, SourceIdentity,
    SourceType,
};
use web_repo_to_ai_context_optimizer::{
    explain_exclusions, explain_exclusions_from_document, explain_exclusions_from_plan,
    explain_exclusions_with_profile, BudgetOptimizationConfig, ContextBudget,
    ContextBudgetOptimizer, ContextOptimizerError, ExclusionImpact, ExclusionReport,
    ExclusionSource,
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

fn make_test_document(sections: Vec<CdmSection>, exclusions: Vec<ExclusionRecord>) -> CdmDocument {
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
        exclusions,
        completeness: CompletenessReport::default(),
    }
}

#[test]
fn test_exclusion_explanations_budget_knapsack() {
    let s_manifest = make_test_section(
        "s1",
        "Cargo.toml",
        SectionKind::Manifest,
        "[package]\nname = \"demo\"",
        vec![],
    );

    let mut sections = vec![s_manifest];
    for i in 0..10 {
        sections.push(make_test_section(
            &format!("mod_{}", i),
            &format!("src/module_{}.rs", i),
            SectionKind::CodeFile,
            &format!(
                "pub fn run_{}() -> usize {{\n    let x = 100;\n{}\n    x\n}}",
                i,
                "    let _y = x + 1;\n".repeat(30)
            ),
            vec![],
        ));
    }

    let orig_doc = make_test_document(sections, vec![]);

    // Budget of 200 total tokens, 100 reserved, available 100 tokens
    let budget = ContextBudget::new(200, 50, 25, 25);
    let config = BudgetOptimizationConfig::new(budget);
    let optimizer = ContextBudgetOptimizer::new(config);
    let (opt_doc, plan) = optimizer
        .optimize(&orig_doc)
        .expect("Optimization succeeds");

    // 1. Explain from BudgetPlan
    let report_plan = explain_exclusions_from_plan(&plan, Some("BudgetOptimizer"));
    assert!(report_plan.total_exclusions > 0);
    assert!(report_plan.total_omitted_tokens > 0);
    assert_eq!(report_plan.profile_name.as_deref(), Some("BudgetOptimizer"));

    let size_limit_cat = report_plan
        .categories
        .iter()
        .find(|c| c.reason == ExclusionReason::SizeLimit)
        .expect("SizeLimit category present");
    assert!(size_limit_cat.item_count > 0);
    assert!(size_limit_cat.total_tokens > 0);
    assert!(size_limit_cat
        .primary_remediation
        .contains("Expand context token budget"));

    // 2. Explain from documents
    let report_docs =
        explain_exclusions(&orig_doc, &opt_doc).expect("Document explanation succeeds");
    assert_eq!(report_docs.total_exclusions, report_plan.total_exclusions);
    assert_eq!(report_docs.highest_impact, report_plan.highest_impact);

    let md = report_plan.to_markdown_summary();
    assert!(md.contains("## Context Optimization Exclusion Report"));
    assert!(md.contains("Context Budget Headroom"));
    assert!(md.contains("size_limit"));
}

#[test]
fn test_exclusion_explanations_profile_rules() {
    let s1 = make_test_section(
        "s1",
        "Cargo.toml",
        SectionKind::Manifest,
        "[package]\nname = \"secure-app\"",
        vec![],
    );
    let s2 = make_test_section(
        "s2",
        "src/auth.rs",
        SectionKind::CodeFile,
        "pub fn authenticate() -> bool { true }",
        vec![],
    );
    let s3 = make_test_section(
        "s3",
        "src/views/styles.css",
        SectionKind::Prose,
        "body { margin: 0; }",
        vec![],
    );
    let s4 = make_test_section(
        "s4",
        "docs/CHANGELOG.md",
        SectionKind::Prose,
        "# Changelog\n- Initial release",
        vec![],
    );

    let orig = make_test_document(vec![s1.clone(), s2.clone(), s3.clone(), s4.clone()], vec![]);
    let opt = make_test_document(
        vec![s1, s2],
        vec![
            ExclusionRecord::new(
                "src/views/styles.css",
                ExclusionReason::UserConfig,
                None,
                Some("Cosmetic presentation styling excluded by SecurityReviewProfile".to_string()),
                Some(25),
            ),
            ExclusionRecord::new(
                "docs/CHANGELOG.md",
                ExclusionReason::UserConfig,
                None,
                Some("Non-security changelog documentation excluded by profile".to_string()),
                Some(40),
            ),
        ],
    );

    let report = explain_exclusions_with_profile(
        &orig,
        &opt,
        Some("SecurityReview"),
        Some(TargetModel::Claude35Sonnet),
    )
    .expect("Report generation succeeds");

    assert_eq!(report.total_exclusions, 2);
    assert_eq!(report.highest_impact, ExclusionImpact::Low);
    assert_eq!(report.high_impact_count, 0);

    let css_exp = report
        .explanations
        .iter()
        .find(|e| e.path.as_deref() == Some("src/views/styles.css"))
        .expect("CSS explanation found");
    assert_eq!(
        css_exp.source,
        ExclusionSource::ProfileRule("SecurityReview".to_string())
    );
    assert_eq!(css_exp.impact, ExclusionImpact::Low);
    assert!(css_exp
        .remediation
        .action
        .contains("include_cosmetic_assets = true"));

    let doc_exp = report
        .explanations
        .iter()
        .find(|e| e.path.as_deref() == Some("docs/CHANGELOG.md"))
        .expect("Changelog explanation found");
    assert!(doc_exp
        .remediation
        .action
        .contains("include_unmodified_docs = true"));
}

#[test]
fn test_exclusion_explanations_high_impact_critical_detection() {
    let s_manifest = make_test_section(
        "s1",
        "Cargo.toml",
        SectionKind::Manifest,
        "[package]\nname = \"core\"",
        vec![],
    );
    let s_auth = make_test_section(
        "s2",
        "src/crypto_auth.rs",
        SectionKind::CodeFile,
        "pub fn verify_signature() {}",
        vec![],
    );
    let s_normal = make_test_section(
        "s3",
        "src/helper.rs",
        SectionKind::CodeFile,
        "pub fn helper() {}",
        vec![],
    );

    let orig = make_test_document(
        vec![s_manifest.clone(), s_auth.clone(), s_normal.clone()],
        vec![],
    );
    // Simulated extreme budget drop where even critical auth and manifest were omitted
    let opt = make_test_document(
        vec![s_normal],
        vec![
            ExclusionRecord::new(
                "Cargo.toml",
                ExclusionReason::SizeLimit,
                None,
                Some("MustInclude manifest dropped".to_string()),
                Some(50),
            ),
            ExclusionRecord::new(
                "src/crypto_auth.rs",
                ExclusionReason::SizeLimit,
                None,
                Some("Auth module dropped".to_string()),
                Some(80),
            ),
        ],
    );

    let report = explain_exclusions(&orig, &opt).expect("Explanation succeeds");

    assert_eq!(report.highest_impact, ExclusionImpact::Critical);
    assert!(report.high_impact_count >= 2);

    let md = report.to_markdown_summary();
    assert!(md.contains("🔴 `CRITICAL`"));
    assert!(md.contains("Attention required:"));
}

#[test]
fn test_exclusion_explanations_from_document_records() {
    let rec1 = ExclusionRecord::new(
        "assets/logo.png",
        ExclusionReason::BinaryFile,
        None,
        Some("Image asset".to_string()),
        Some(1024),
    );
    let rec2 = ExclusionRecord::new(
        ".env.secret",
        ExclusionReason::SecurityRestricted,
        None,
        Some("Sensitive secret file".to_string()),
        Some(128),
    );
    let rec3 = ExclusionRecord::new(
        "src/duplicate.rs",
        ExclusionReason::Duplicate,
        None,
        Some("Exact duplicate of src/original.rs".to_string()),
        Some(256),
    );

    let doc = make_test_document(vec![], vec![rec1, rec2, rec3]);

    let report = explain_exclusions_from_document(&doc).expect("Explanation succeeds");

    assert_eq!(report.total_exclusions, 3);
    assert_eq!(report.categories.len(), 3);

    let sec_cat = report
        .categories
        .iter()
        .find(|c| c.reason == ExclusionReason::SecurityRestricted)
        .expect("Security category found");
    assert_eq!(sec_cat.item_count, 1);
    assert_eq!(sec_cat.total_bytes, 128);
}

#[test]
fn test_exclusion_explanations_markdown_and_json_serialization() {
    let s1 = make_test_section(
        "s1",
        "Cargo.toml",
        SectionKind::Manifest,
        "[package]\nname = \"demo\"",
        vec![],
    );
    let s2 = make_test_section(
        "s2",
        "src/unused.rs",
        SectionKind::CodeFile,
        "pub fn unused() {}",
        vec![],
    );

    let orig = make_test_document(vec![s1.clone(), s2.clone()], vec![]);
    let opt = make_test_document(
        vec![s1],
        vec![ExclusionRecord::new(
            "src/unused.rs",
            ExclusionReason::SizeLimit,
            None,
            Some("Exceeded context budget limit".to_string()),
            Some(40),
        )],
    );

    let report =
        explain_exclusions_with_profile(&orig, &opt, Some("GeneralAi"), Some(TargetModel::Gpt4o))
            .expect("Report generation succeeds");

    // Verify Markdown Summary
    let md = report.to_markdown_summary();
    assert!(md.contains("## Context Optimization Exclusion Report"));
    assert!(md.contains("GeneralAi"));
    assert!(md.contains("gpt-4o"));
    assert!(md.contains("### 1. Exclusion Categories Summary"));
    assert!(md.contains("### 2. Itemized Exclusion Audit Trail"));
    assert!(md.contains("src/unused.rs"));

    // Verify JSON roundtrip
    let json = report.to_json().expect("JSON serialization succeeds");
    let decoded = ExclusionReport::from_json(&json).expect("JSON deserialization succeeds");

    assert_eq!(report.total_exclusions, decoded.total_exclusions);
    assert_eq!(report.total_omitted_tokens, decoded.total_omitted_tokens);
    assert_eq!(report.total_omitted_bytes, decoded.total_omitted_bytes);
    assert_eq!(report.highest_impact, decoded.highest_impact);
    assert_eq!(report.explanations.len(), decoded.explanations.len());
    assert_eq!(report.categories.len(), decoded.categories.len());
}

#[test]
fn test_exclusion_explanations_zero_exclusions_fidelity() {
    let s1 = make_test_section(
        "s1",
        "src/main.rs",
        SectionKind::CodeFile,
        "fn main() {}",
        vec![],
    );
    let doc = make_test_document(vec![s1], vec![]);

    let report = explain_exclusions(&doc, &doc).expect("Succeeds on identical documents");

    assert_eq!(report.total_exclusions, 0);
    assert_eq!(report.highest_impact, ExclusionImpact::Low);
    assert_eq!(report.total_omitted_tokens, 0);
    assert_eq!(report.total_omitted_bytes, 0);
    assert!(report.explanations.is_empty());
    assert!(report.categories.is_empty());

    let md = report.to_markdown_summary();
    assert!(md.contains("zero omissions"));
}

#[test]
fn test_exclusion_explanations_fail_closed_empty_doc() {
    let empty_doc = make_test_document(vec![], vec![]);
    let valid_doc = make_test_document(
        vec![make_test_section(
            "s1",
            "a.rs",
            SectionKind::CodeFile,
            "fn a() {}",
            vec![],
        )],
        vec![],
    );

    let err = explain_exclusions(&empty_doc, &valid_doc)
        .expect_err("Should fail closed on empty original doc");
    assert!(matches!(err, ContextOptimizerError::DocumentEmpty));
}
