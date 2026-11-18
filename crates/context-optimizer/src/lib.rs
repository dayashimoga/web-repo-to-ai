//! # Context Optimizer (`web-repo-to-ai-context-optimizer`)
//!
//! Fits Canonical Context Document Model ([`CdmDocument`]) instances into strict target
//! LLM context budgets without external API calls:
//! - Multi-tier context headroom modeling ([`ContextBudget`]) for system, response, and query allowances
//! - High-signal section importance and density prioritization ([`SectionScorer`], [`SectionPriority`])
//! - Greedy knapsack budget optimization engine ([`ContextBudgetOptimizer`])
//! - Deterministic boundary truncation ([`TruncationStrategy`])
//! - Auditable exclusion logging ([`web_repo_to_ai_cdm::ExclusionRecord`]) with zero silent drops
//! - Comprehensive reporting and serialization ([`ContextBudgetReport`])

#![warn(missing_docs)]

pub mod budget;
pub mod compression;
pub mod error;
pub mod optimizer;
pub mod plan;
pub mod preservation;
pub mod priority;
pub mod profiles;
pub mod report;

pub use budget::{BudgetHeadroomSummary, ContextBudget};
pub use compression::{
    CompressionEvaluator, CompressionPipelineTracker, CompressionReport, CostSavingsMetrics,
    DocumentVolume, MultiModelCostSavings, OptimizationStage, OverallCompressionMetrics,
    StageCompressionMetrics,
};
pub use error::ContextOptimizerError;
pub use optimizer::{BudgetOptimizationConfig, ContextBudgetOptimizer, TruncationStrategy};
pub use plan::{BudgetAction, BudgetPlan, SectionBudgetDecision};
pub use preservation::{
    CriticalPreservationMetrics, KindPreservation, PreservationConfig, PreservationEvaluator,
    PreservationReport, SectionPreservationMetrics, StructuralIntegrityMetrics,
    SymbolPreservationMetrics, TokenPreservationMetrics,
};
pub use priority::{PinStatus, PriorityScoreFactors, SectionPriority, SectionScorer};
pub use profiles::{
    CodeReviewConfig, CodeReviewProfile, ContextProfile, DebuggingCategory, DebuggingConfig,
    DebuggingProfile, GeneralAiConfig, GeneralAiProfile, ProfileKind, ProfileMetrics,
    ProfileOptimizationResult, SecurityCategory, SecurityReviewConfig, SecurityReviewProfile,
};
pub use report::ContextBudgetReport;

use web_repo_to_ai_cdm::CdmDocument;
use web_repo_to_ai_tokenizer::TargetModel;

/// Convenience function: Evaluates compression and token reduction between original and optimized documents.
pub fn evaluate_compression(
    original: &CdmDocument,
    optimized: &CdmDocument,
) -> Result<CompressionReport, ContextOptimizerError> {
    CompressionEvaluator::evaluate(original, optimized)
}

/// Convenience function: Evaluates compression and token reduction for a specific [`TargetModel`].
pub fn evaluate_compression_for_model(
    original: &CdmDocument,
    optimized: &CdmDocument,
    model: TargetModel,
) -> Result<CompressionReport, ContextOptimizerError> {
    CompressionEvaluator::evaluate_for_model(original, optimized, model)
}

/// Convenience function: Evaluates compression and token reduction with profile context and optional target model.
pub fn evaluate_compression_with_profile(
    original: &CdmDocument,
    optimized: &CdmDocument,
    profile_name: Option<impl Into<String>>,
    target_model: Option<TargetModel>,
) -> Result<CompressionReport, ContextOptimizerError> {
    CompressionEvaluator::evaluate_with_profile(original, optimized, profile_name, target_model)
}

/// Convenience function: Evaluates content preservation and symbol fidelity between original and optimized documents.
pub fn evaluate_preservation(
    original: &CdmDocument,
    optimized: &CdmDocument,
) -> Result<PreservationReport, ContextOptimizerError> {
    PreservationEvaluator::evaluate(original, optimized)
}

/// Convenience function: Evaluates content preservation using custom configuration.
pub fn evaluate_preservation_with_config(
    original: &CdmDocument,
    optimized: &CdmDocument,
    config: &PreservationConfig,
) -> Result<PreservationReport, ContextOptimizerError> {
    PreservationEvaluator::evaluate_with_config(original, optimized, config)
}

/// Convenience function: Optimizes a [`CdmDocument`] using the Debugging & Documentation profile for a target model.
pub fn apply_debugging_profile_for_model(
    document: &CdmDocument,
    model: TargetModel,
) -> Result<ProfileOptimizationResult, ContextOptimizerError> {
    let budget = ContextBudget::from_model(model);
    let profile = DebuggingProfile::default();
    profile.optimize(document, &budget)
}

/// Convenience function: Optimizes a [`CdmDocument`] using the Debugging & Documentation profile against a custom budget.
pub fn apply_debugging_profile(
    document: &CdmDocument,
    budget: &ContextBudget,
) -> Result<ProfileOptimizationResult, ContextOptimizerError> {
    let profile = DebuggingProfile::default();
    profile.optimize(document, budget)
}

/// Convenience function: Optimizes a [`CdmDocument`] targeting a specific error pattern for a target model.
pub fn apply_debugging_profile_with_target_for_model(
    document: &CdmDocument,
    target_error_pattern: impl Into<String>,
    model: TargetModel,
) -> Result<ProfileOptimizationResult, ContextOptimizerError> {
    let budget = ContextBudget::from_model(model);
    let profile = DebuggingProfile::from_error_pattern(target_error_pattern);
    profile.optimize(document, &budget)
}

/// Convenience function: Optimizes a [`CdmDocument`] targeting a specific error pattern against a custom budget.
pub fn apply_debugging_profile_with_target(
    document: &CdmDocument,
    target_error_pattern: impl Into<String>,
    budget: &ContextBudget,
) -> Result<ProfileOptimizationResult, ContextOptimizerError> {
    let profile = DebuggingProfile::from_error_pattern(target_error_pattern);
    profile.optimize(document, budget)
}

/// Convenience function: Optimizes a [`CdmDocument`] using the Security Review profile for a target model.
pub fn apply_security_review_profile_for_model(
    document: &CdmDocument,
    model: TargetModel,
) -> Result<ProfileOptimizationResult, ContextOptimizerError> {
    let budget = ContextBudget::from_model(model);
    let profile = SecurityReviewProfile::default();
    profile.optimize(document, &budget)
}

/// Convenience function: Optimizes a [`CdmDocument`] using the Security Review profile against a custom budget.
pub fn apply_security_review_profile(
    document: &CdmDocument,
    budget: &ContextBudget,
) -> Result<ProfileOptimizationResult, ContextOptimizerError> {
    let profile = SecurityReviewProfile::default();
    profile.optimize(document, budget)
}

/// Convenience function: Optimizes a [`CdmDocument`] using the Code Review profile for a target model.
pub fn apply_code_review_profile_for_model(
    document: &CdmDocument,
    changed_files: &[String],
    model: TargetModel,
) -> Result<ProfileOptimizationResult, ContextOptimizerError> {
    let budget = ContextBudget::from_model(model);
    let profile = CodeReviewProfile::from_changed_files(changed_files.to_vec());
    profile.optimize(document, &budget)
}

/// Convenience function: Optimizes a [`CdmDocument`] using the Code Review profile against a custom budget.
pub fn apply_code_review_profile(
    document: &CdmDocument,
    changed_files: &[String],
    budget: &ContextBudget,
) -> Result<ProfileOptimizationResult, ContextOptimizerError> {
    let profile = CodeReviewProfile::from_changed_files(changed_files.to_vec());
    profile.optimize(document, budget)
}

/// Convenience function: Optimizes a [`CdmDocument`] using the General AI profile for a target model.
pub fn apply_general_ai_profile_for_model(
    document: &CdmDocument,
    model: TargetModel,
) -> Result<ProfileOptimizationResult, ContextOptimizerError> {
    let budget = ContextBudget::from_model(model);
    let profile = GeneralAiProfile::default();
    profile.optimize(document, &budget)
}

/// Convenience function: Optimizes a [`CdmDocument`] using the General AI profile against a custom budget.
pub fn apply_general_ai_profile(
    document: &CdmDocument,
    budget: &ContextBudget,
) -> Result<ProfileOptimizationResult, ContextOptimizerError> {
    let profile = GeneralAiProfile::default();
    profile.optimize(document, budget)
}

/// Convenience function: Optimizes a [`CdmDocument`] to fit the context window of a target model.
pub fn optimize_context_for_model(
    document: &CdmDocument,
    model: TargetModel,
) -> Result<(CdmDocument, ContextBudgetReport), ContextOptimizerError> {
    let budget = ContextBudget::from_model(model);
    let optimizer = ContextBudgetOptimizer::for_model(model);
    let (optimized_doc, plan) = optimizer.optimize(document)?;
    let report = ContextBudgetReport::from_plan_and_budget(plan, &budget);
    Ok((optimized_doc, report))
}

/// Convenience function: Optimizes a [`CdmDocument`] against a custom [`ContextBudget`].
pub fn optimize_context_with_budget(
    document: &CdmDocument,
    budget: ContextBudget,
) -> Result<(CdmDocument, ContextBudgetReport), ContextOptimizerError> {
    let config = BudgetOptimizationConfig::new(budget.clone());
    let optimizer = ContextBudgetOptimizer::new(config);
    let (optimized_doc, plan) = optimizer.optimize(document)?;
    let report = ContextBudgetReport::from_plan_and_budget(plan, &budget);
    Ok((optimized_doc, report))
}

/// Returns the unique crate identifier name.
#[must_use]
pub fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_optimizer_metadata() {
        assert_eq!(crate_name(), "web-repo-to-ai-context-optimizer");
    }
}
