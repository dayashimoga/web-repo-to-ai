//! # Context Budget Optimization Plan
//!
//! Represents the deterministic optimization decisions, token conservation accounting,
//! and audit trail produced when fitting a [`CdmDocument`] into a target context budget.

use crate::priority::SectionPriority;
use serde::{Deserialize, Serialize};
use web_repo_to_ai_cdm::{ExclusionReason, SectionKind};
use web_repo_to_ai_tokenizer::TargetModel;

/// Specific optimization action applied to an individual content section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BudgetAction {
    /// Section was included completely without truncation.
    Include,
    /// Section was truncated cleanly to fit remaining available token headroom.
    Truncate {
        /// Original token count prior to truncation.
        original_tokens: usize,
        /// Number of tokens retained in the truncated section.
        retained_tokens: usize,
        /// Number of tokens omitted to meet the budget ceiling.
        truncated_tokens: usize,
        /// Rationale or method of truncation.
        reason: String,
    },
    /// Section was excluded from the final context package.
    Exclude {
        /// Token count that would have been required by this section.
        tokens: usize,
        /// Categorized exclusion reason.
        reason: ExclusionReason,
        /// Detailed audit rationale for why the section was dropped.
        details: String,
    },
}

/// Audit record capturing the budget decision for a single content section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionBudgetDecision {
    /// Unique identifier of the section.
    pub section_id: String,
    /// Relative file path or DOM selector of the section.
    pub path: Option<String>,
    /// Section heading, title, or filename.
    pub title: Option<String>,
    /// Content kind classification.
    pub kind: SectionKind,
    /// Tokens required or allocated for this section.
    pub tokens: usize,
    /// Running cumulative token count across all included/truncated sections up to this point.
    pub cumulative_tokens: usize,
    /// Assigned priority scoring and pinning status.
    pub priority: SectionPriority,
    /// Priority utility per token (`priority.score * 1000.0 / tokens.max(1)`).
    pub utility_per_token: f64,
    /// Action taken for this section.
    pub action: BudgetAction,
}

/// Deterministic optimization plan and token conservation ledger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BudgetPlan {
    /// Target model architecture, if bound.
    pub target_model: Option<TargetModel>,
    /// Total context window capacity ceiling.
    pub total_budget_tokens: usize,
    /// Effective usable content budget after deducting reserved headroom.
    pub available_content_tokens: usize,
    /// Total reserved headroom tokens (system, output, query).
    pub total_reserved_tokens: usize,
    /// Total token volume of the original document before budget optimization.
    pub original_document_tokens: usize,
    /// Total token volume of the optimized document fitted into the budget.
    pub fitted_tokens: usize,
    /// Percentage of available content budget utilized (`fitted_tokens / available_content_tokens * 100`).
    pub budget_utilization_pct: f64,
    /// Total number of sections in the input document.
    pub original_section_count: usize,
    /// Number of sections included completely in the optimized document.
    pub included_section_count: usize,
    /// Number of sections truncated to fit the budget.
    pub truncated_section_count: usize,
    /// Number of sections excluded from the optimized document.
    pub excluded_section_count: usize,
    /// Ordered audit decisions for all sections.
    pub decisions: Vec<SectionBudgetDecision>,
}

impl BudgetPlan {
    /// Returns true if the final optimized token count is within the available content budget.
    #[must_use]
    pub fn is_within_budget(&self) -> bool {
        self.fitted_tokens <= self.available_content_tokens
    }

    /// Renders a concise human-readable summary of the optimization outcome.
    #[must_use]
    pub fn summary(&self) -> String {
        let model_str = self
            .target_model
            .map_or("Custom / Generic", TargetModel::as_str);

        format!(
            "Context Budget Optimization: fitted {} / {} available tokens ({:.1}% utilization) for model '{}'. Included: {}, Truncated: {}, Excluded: {}.",
            self.fitted_tokens,
            self.available_content_tokens,
            self.budget_utilization_pct,
            model_str,
            self.included_section_count,
            self.truncated_section_count,
            self.excluded_section_count,
        )
    }

    /// Generates a GitHub Flavored Markdown summary table of section decisions.
    #[must_use]
    pub fn to_markdown_table(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("### {}\n\n", self.summary()));
        out.push_str("| # | Section / Path | Kind | Tokens | Cumulative | Priority | Action |\n");
        out.push_str("| :---: | :--- | :---: | :---: | :---: | :---: | :---: |\n");

        for (idx, decision) in self.decisions.iter().enumerate() {
            let path_display = decision
                .path
                .as_deref()
                .or(decision.title.as_deref())
                .unwrap_or(&decision.section_id);

            let action_str = match &decision.action {
                BudgetAction::Include => "Included".to_string(),
                BudgetAction::Truncate {
                    retained_tokens,
                    original_tokens,
                    ..
                } => format!("Truncated ({retained_tokens}/{original_tokens})"),
                BudgetAction::Exclude { reason, .. } => {
                    format!("Excluded ({})", reason.as_str())
                }
            };

            out.push_str(&format!(
                "| {} | `{}` | `{}` | `{}` | `{}` | `{:.2}` ({}) | {} |\n",
                idx + 1,
                path_display,
                decision.kind.as_str(),
                decision.tokens,
                decision.cumulative_tokens,
                decision.priority.score,
                decision.priority.pin_status.as_str(),
                action_str,
            ));
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::priority::PriorityScoreFactors;

    #[test]
    fn test_budget_plan_markdown_rendering() {
        let factors = PriorityScoreFactors {
            kind_weight: 1.0,
            prominence_bonus: 0.35,
            density_modifier: 0.0,
            signal_score: None,
            nature_modifier: 0.0,
            penalty: 0.0,
        };
        let p = SectionPriority {
            pin_status: crate::priority::PinStatus::MustInclude,
            score: 1.0,
            score_factors: factors,
        };

        let decision = SectionBudgetDecision {
            section_id: "sec-1".to_string(),
            path: Some("README.md".to_string()),
            title: Some("README".to_string()),
            kind: SectionKind::Prose,
            tokens: 500,
            cumulative_tokens: 500,
            priority: p,
            utility_per_token: 2.0,
            action: BudgetAction::Include,
        };

        let plan = BudgetPlan {
            target_model: Some(TargetModel::Gpt4o),
            total_budget_tokens: 128_000,
            available_content_tokens: 120_404,
            total_reserved_tokens: 7_596,
            original_document_tokens: 500,
            fitted_tokens: 500,
            budget_utilization_pct: 0.42,
            original_section_count: 1,
            included_section_count: 1,
            truncated_section_count: 0,
            excluded_section_count: 0,
            decisions: vec![decision],
        };

        assert!(plan.is_within_budget());
        let md = plan.to_markdown_table();
        assert!(md.contains("README.md"));
        assert!(md.contains("gpt-4o"));
        assert!(md.contains("Included"));
    }
}
