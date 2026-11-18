//! # Cost Estimation Integration Tests (`crates/tokenizer`)
//!
//! Validates tiered model pricing schedules, standard/cached/batch calculations,
//! cross-model cost comparisons, dollar budget guardrails, and JSON telemetry serialization.

use chrono::Utc;
use std::collections::BTreeMap;
use web_repo_to_ai_cdm::{
    CdmDocument, CdmSection, CompletenessReport, DocumentMetadata, ExtractionStatus, HierarchyNode,
    LineRange, RevisionInfo, SectionKind, SourceIdentity, SourceType,
};
use web_repo_to_ai_core::DocumentId;
use web_repo_to_ai_tokenizer::{
    compare_token_costs, estimate_token_cost, CostBreakdown, CostGuardrail,
    CostGuardrailEvaluation, CrossModelCostReport, ModelPricingSchedule, TargetModel,
    TokenizerError,
};

fn make_test_section(id: &str, body: &str) -> CdmSection {
    CdmSection {
        section_id: id.to_string(),
        parent_id: None,
        title: Some(id.to_string()),
        kind: SectionKind::CodeFile,
        original_path: Some(format!("src/{id}.rs")),
        source_url: format!("https://example.com/src/{id}.rs"),
        line_range: Some(LineRange::new(1, 10)),
        byte_range: None,
        language: Some("rust".to_string()),
        content_hash: CdmSection::compute_hash(body),
        body: body.to_string(),
        token_metrics: None,
        attributes: BTreeMap::new(),
    }
}

fn make_test_doc(sections: Vec<CdmSection>) -> CdmDocument {
    CdmDocument {
        schema_version: "1.0.0".to_string(),
        document_id: DocumentId::new(),
        source_identity: SourceIdentity::new("https://example.com", "https://example.com"),
        source_type: SourceType::GitHubPublic,
        canonical_url: "https://example.com".to_string(),
        retrieved_at: Utc::now(),
        revision: RevisionInfo::default(),
        content_type: "text/markdown".to_string(),
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
fn test_model_pricing_schedules() {
    let gpt4o = ModelPricingSchedule::for_model(TargetModel::Gpt4o);
    assert_eq!(gpt4o.input_cost_per_million, 2.50);
    assert_eq!(gpt4o.cached_input_cost_per_million, Some(1.25));
    assert_eq!(gpt4o.output_cost_per_million, 10.00);
    assert!(!gpt4o.is_free_or_self_hosted);

    let mini = ModelPricingSchedule::for_model(TargetModel::Gpt4oMini);
    assert_eq!(mini.input_cost_per_million, 0.15);
    assert_eq!(mini.cached_input_cost_per_million, Some(0.075));
    assert!(!mini.is_free_or_self_hosted);

    let claude = ModelPricingSchedule::for_model(TargetModel::Claude35Sonnet);
    assert_eq!(claude.input_cost_per_million, 3.00);
    assert_eq!(claude.cached_input_cost_per_million, Some(0.30)); // 90% discount

    let gemini_flash = ModelPricingSchedule::for_model(TargetModel::Gemini15Flash);
    assert_eq!(gemini_flash.input_cost_per_million, 0.075);

    let llama = ModelPricingSchedule::for_model(TargetModel::Llama3);
    assert!(llama.is_free_or_self_hosted);
    assert_eq!(llama.input_cost_per_million, 0.0);
}

#[test]
fn test_cost_breakdown_standard_cached_and_batch() {
    let input_tokens = 200_000;
    let output_tokens = 4_000;

    // Claude 3.5 Sonnet:
    // Standard: (200k * $3.00 + 4k * $15.00) / 1M = $0.60 + $0.06 = $0.66
    // Cached input: (200k * $0.30 + 4k * $15.00) / 1M = $0.06 + $0.06 = $0.12
    let cost = CostBreakdown::calculate(input_tokens, TargetModel::Claude35Sonnet, output_tokens);

    assert_eq!(cost.standard_cost_usd, 0.66);
    assert_eq!(cost.cached_cost_usd, Some(0.12));
    assert_eq!(cost.cache_savings_usd, 0.54);
    assert_eq!(cost.cache_savings_pct, 81.8);
    assert_eq!(cost.batch_savings_pct, 50.0);
    assert_eq!(cost.batch_cost_usd, 0.33);
}

#[test]
fn test_cross_model_cost_comparison_report() {
    let report = compare_token_costs(100_000, 2_000, &[]);

    assert_eq!(report.input_tokens, 100_000);
    assert_eq!(report.estimated_output_tokens, 2_000);
    assert!(!report.breakdowns.is_empty());

    // Gemini 1.5 Flash or GPT-4o-mini should be the cheapest commercial model
    assert!(
        report.cheapest_paid_model == Some(TargetModel::Gemini15Flash)
            || report.cheapest_paid_model == Some(TargetModel::Gpt4oMini)
    );

    assert!(report.price_ratio_expensive_vs_cheapest > 5.0);

    // Markdown summary table
    let md = report.to_markdown_table();
    assert!(md.contains("Cross-Model Inference Cost Intelligence"));
    assert!(md.contains("Target Model"));
    assert!(md.contains("Standard Cost"));
    assert!(md.contains("Cached Read Cost"));
    assert!(md.contains("Batch API Cost"));
    assert!(md.contains("Cost Optimization Insight"));

    // JSON roundtrip
    let json = serde_json::to_string_pretty(&report).expect("valid json serialization");
    let deserialized: CrossModelCostReport =
        serde_json::from_str(&json).expect("valid json deserialization");
    assert_eq!(report, deserialized);
}

#[test]
fn test_document_level_cost_estimation() {
    let s1 = make_test_section("s1", "pub fn process_a() -> i32 { 100 }");
    let s2 = make_test_section("s2", "pub fn process_b() -> i32 { 200 }");
    let doc = make_test_doc(vec![s1, s2]);

    let report = CrossModelCostReport::compare_document(&doc, 500, &[]);
    assert!(report.input_tokens > 0);
    assert!(!report.breakdowns.is_empty());
}

#[test]
fn test_cost_guardrail_within_and_exceeded() {
    let guardrail = CostGuardrail::new(0.05); // $0.05 ceiling

    // 10k tokens on GPT-4o-mini: 10k * $0.15/1M = $0.0015 -> within $0.05
    let mini_cost = estimate_token_cost(10_000, TargetModel::Gpt4oMini, 500);
    assert!(guardrail.check_or_err(&mini_cost).is_ok());

    match guardrail.evaluate(&mini_cost) {
        CostGuardrailEvaluation::WithinBudget {
            remaining_budget_usd,
        } => {
            assert!(remaining_budget_usd > 0.0);
        }
        _ => panic!("Expected WithinBudget"),
    }

    // 500k tokens on GPT-4o: 500k * $2.50/1M = $1.25 -> exceeds $0.05
    let gpt4o_cost = estimate_token_cost(500_000, TargetModel::Gpt4o, 2_000);
    assert!(guardrail.check_or_err(&gpt4o_cost).is_err());
    match guardrail.check_or_err(&gpt4o_cost) {
        Err(TokenizerError::CostBudgetExceeded {
            budget_usd,
            estimated_usd,
        }) => {
            assert_eq!(budget_usd, 0.05);
            assert!(estimated_usd > 1.0);
        }
        other => panic!("Unexpected result: {other:?}"),
    }

    match guardrail.evaluate(&gpt4o_cost) {
        CostGuardrailEvaluation::BudgetExceeded {
            excess_usd,
            suggested_alternatives,
        } => {
            assert!(excess_usd > 1.0);
            assert!(!suggested_alternatives.is_empty());
            // Llama 3 is free, so it should always be suggested
            assert!(suggested_alternatives.contains(&TargetModel::Llama3));
        }
        _ => panic!("Expected BudgetExceeded"),
    }
}

#[test]
fn test_extreme_token_scaling() {
    let large_input = 20_000_000; // 20 Million tokens (e.g. Gemini 1.5 Pro massive repository)
    let output = 8_000;
    let cost = estimate_token_cost(large_input, TargetModel::Gemini15Pro, output);

    assert!(cost.standard_cost_usd > 0.0);
    assert!(!cost.standard_cost_usd.is_nan());
    assert!(!cost.standard_cost_usd.is_infinite());
    // 20M * $1.25 = $25.00 + $0.04 = $25.04
    assert_eq!(cost.standard_cost_usd, 25.04);
}
