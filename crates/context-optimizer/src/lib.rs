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
pub mod error;
pub mod optimizer;
pub mod plan;
pub mod priority;
pub mod report;

pub use budget::{BudgetHeadroomSummary, ContextBudget};
pub use error::ContextOptimizerError;
pub use optimizer::{BudgetOptimizationConfig, ContextBudgetOptimizer, TruncationStrategy};
pub use plan::{BudgetAction, BudgetPlan, SectionBudgetDecision};
pub use priority::{PinStatus, PriorityScoreFactors, SectionPriority, SectionScorer};
pub use report::ContextBudgetReport;

use web_repo_to_ai_cdm::CdmDocument;
use web_repo_to_ai_tokenizer::TargetModel;

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
